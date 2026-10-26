//! Ordered, run-owned native requests and immutable file-change snapshots.
use super::super::{
    interaction::{CodexInteractionReply, CodexInteractionRequest, Handler},
    CodexBackendError, CodexBackendEvent, PINNED_APPROVAL_REQUIRED,
};
use codex::mcp::{
    AppNotification, AppServerMessage, AppServerMessageStream, AppServerRequest,
    AppServerRetention, McpError,
};
use futures_util::{stream::FuturesUnordered, FutureExt, StreamExt};
use serde_json::Value;
use std::{
    collections::{BTreeMap, VecDeque},
    future::Future,
    panic::AssertUnwindSafe,
    pin::Pin,
};
use tokio::sync::mpsc;

const FRAME: usize = 1024 * 1024;
const COUNT: usize = 16;
pub(super) enum Scope {
    Thread(String),
    Turn(String),
}
struct Context {
    item: Value,
    _retention: AppServerRetention,
}
type ReplyFuture = Pin<
    Box<
        dyn Future<
                Output = (
                    usize,
                    Option<(AppServerRequest, Result<CodexInteractionReply, ()>)>,
                ),
            > + Send,
    >,
>;
fn failure() -> CodexBackendError {
    CodexBackendError::AppServer(McpError::Server(PINNED_APPROVAL_REQUIRED.into()))
}
fn protocol() -> CodexBackendError {
    CodexBackendError::AppServer(McpError::Server("invalid turn lifecycle".into()))
}
fn size(value: &Value) -> usize {
    serde_json::to_vec(value).map_or(usize::MAX, |bytes| bytes.len())
}
fn identity<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|id| !id.trim().is_empty())
}
fn valid_item(item: &Value) -> bool {
    identity(item, "id").is_some()
        && item.get("type").and_then(Value::as_str) == Some("fileChange")
        && item.get("status").and_then(Value::as_str) == Some("inProgress")
        && item
            .get("changes")
            .and_then(Value::as_array)
            .is_some_and(|changes| {
                changes.iter().all(|change| {
                    change.get("path").is_some_and(Value::is_string)
                        && change.get("diff").is_some_and(Value::is_string)
                        && change
                            .get("kind")
                            .and_then(|kind| kind.get("type"))
                            .and_then(Value::as_str)
                            .is_some_and(|kind| {
                                matches!(kind, "add" | "delete" | "update")
                                    && (kind != "update"
                                        || change
                                            .get("kind")
                                            .and_then(|kind| kind.get("move_path"))
                                            .map_or(true, |path| {
                                                path.is_null() || path.is_string()
                                            }))
                            })
                })
            })
}
/// A bounded handoff polled alongside native requests. Cleanup precedes final draining.
pub(super) struct Output {
    sender: mpsc::Sender<CodexBackendEvent>,
    queue: VecDeque<CodexBackendEvent>,
}
impl Output {
    pub(super) fn new(sender: mpsc::Sender<CodexBackendEvent>) -> Self {
        Self {
            sender,
            queue: VecDeque::new(),
        }
    }
    fn push(&mut self, event: CodexBackendEvent) -> Result<(), CodexBackendError> {
        if self.queue.len() >= crate::backend_harness::DEFAULT_EVENT_CHANNEL_CAPACITY {
            return Err(CodexBackendError::AppServer(McpError::Server(
                "native output buffer exceeded".into(),
            )));
        }
        self.queue.push_back(event);
        Ok(())
    }
    async fn next(&mut self) {
        if let Ok(permit) = self.sender.reserve().await {
            if let Some(event) = self.queue.pop_front() {
                permit.send(event);
            }
        } else {
            self.queue.clear();
        }
    }
    pub(super) async fn flush(&mut self) {
        while !self.queue.is_empty() {
            self.next().await;
        }
    }
}
struct Broker {
    thread: Option<String>,
    turn: Option<String>,
    queue: VecDeque<(AppServerMessage, Option<AppServerRetention>)>,
    contexts: BTreeMap<String, Context>,
    pending: BTreeMap<usize, Option<String>>,
    callbacks: FuturesUnordered<ReplyFuture>,
    serial: usize,
    final_text: Option<String>,
}
impl Broker {
    fn queue(
        &mut self,
        message: AppServerMessage,
        stream: &AppServerMessageStream,
    ) -> Result<(), CodexBackendError> {
        let retention = match &message {
            AppServerMessage::Request(_) => None,
            AppServerMessage::Notification { params, .. } => {
                Some(stream.retain_json(size(params)).map_err(|_| failure())?)
            }
        };
        let requests = self
            .queue
            .iter()
            .filter(|(message, _)| matches!(message, AppServerMessage::Request(_)))
            .count();
        if matches!(message, AppServerMessage::Request(_)) && requests + self.pending.len() >= COUNT
        {
            return Err(failure());
        }
        self.queue.push_back((message, retention));
        Ok(())
    }
    fn ready(&self, message: &AppServerMessage) -> bool {
        if self.thread.is_none() {
            return false;
        }
        if let AppServerMessage::Notification { method, .. } = message {
            if method == "thread/closed" {
                return true;
            }
        }
        if let AppServerMessage::Request(request) = message {
            if request.method() == "mcpServer/elicitation/request"
                && request.params().get("turnId").map_or(true, Value::is_null)
            {
                return true;
            }
        }
        self.turn.is_some()
    }
    fn request(
        &mut self,
        request: AppServerRequest,
        handler: Option<&Handler>,
        stream: &AppServerMessageStream,
    ) -> Result<(), CodexBackendError> {
        if !request.is_active() {
            return Ok(());
        }
        let params = request.params();
        let standalone = request.method() == "mcpServer/elicitation/request"
            && params.get("turnId").map_or(true, Value::is_null);
        if identity(params, "threadId") != self.thread.as_deref()
            || (!standalone && identity(params, "turnId") != self.turn.as_deref())
        {
            let _ = request.unsupported();
            return Err(failure());
        }
        let supported = matches!(
            request.method(),
            "item/commandExecution/requestApproval"
                | "item/fileChange/requestApproval"
                | "item/permissions/requestApproval"
                | "item/tool/requestUserInput"
                | "mcpServer/elicitation/request"
        );
        if !supported || handler.is_none() {
            let _ = request.unsupported();
            return Err(failure());
        }
        if request.method() != "mcpServer/elicitation/request"
            && identity(params, "itemId").is_none()
        {
            let _ = request.unsupported();
            return Err(failure());
        }
        let context = if request.method() == "item/fileChange/requestApproval" {
            let item = identity(params, "itemId")
                .and_then(|id| self.contexts.get(id))
                .ok_or_else(failure)?;
            Some(item.item.clone())
        } else {
            None
        };
        let bytes = b"{\"method\":,\"params\":,\"item_context\":}"
            .len()
            .saturating_add(size(&Value::String(request.method().into())))
            .saturating_add(size(params))
            .saturating_add(context.as_ref().map_or(4, size));
        let item_id = if context.is_some() {
            identity(params, "itemId").map(str::to_string)
        } else {
            None
        };
        if bytes > FRAME || self.pending.len() >= COUNT {
            let _ = request.unsupported();
            return Err(failure());
        }
        let retention = stream.retain_json(bytes).map_err(|_| failure())?;
        let delivery = CodexInteractionRequest {
            method: request.method().into(),
            params: params.clone(),
            item_context: context,
        };
        let handler = handler.expect("checked handler").clone();
        self.serial += 1;
        let serial = self.serial;
        self.pending.insert(serial, item_id);
        self.callbacks.push(Box::pin(async move {
            let _retention = retention;
            let mut request = request;
            // Catch both constructing the host future and polling it. No host task is detached.
            let work = AssertUnwindSafe(async move {
                let future = (handler.0)(delivery);
                future.await.map_err(|_| ())
            })
            .catch_unwind();
            tokio::pin!(work);
            let result = tokio::select! {
                biased;
                _ = request.cancelled() => return (serial,None),
                result = &mut work => result.unwrap_or(Err(())),
            };
            (serial, Some((request, result)))
        }));
        Ok(())
    }
    fn notification(
        &mut self,
        method: String,
        params: Value,
        events: &mut Output,
        stream: &AppServerMessageStream,
    ) -> Result<bool, CodexBackendError> {
        if identity(&params, "threadId") != self.thread.as_deref() {
            return Ok(false);
        }
        if method == "thread/closed" {
            return Err(failure());
        }
        if method == "turn/completed" {
            let turn = params.get("turn").ok_or_else(protocol)?;
            let id = identity(turn, "id").ok_or_else(protocol)?;
            if params
                .get("turnId")
                .is_some_and(|value| value.as_str() != Some(id))
            {
                return Err(protocol());
            }
            if Some(id) != self.turn.as_deref() {
                return Ok(false);
            }
            if turn.get("items").and_then(Value::as_array).is_none()
                || turn.get("error").is_some_and(|error| !error.is_null())
            {
                return Err(protocol());
            }
            match turn.get("status").and_then(Value::as_str) {
                Some("completed") => {}
                Some("interrupted") => {
                    return Err(CodexBackendError::AppServer(McpError::Cancelled))
                }
                _ => return Err(protocol()),
            }
            events.push(CodexBackendEvent::AppServerNotification(
                AppNotification::Raw { method, params },
            ))?;
            return Ok(true);
        }
        let turn = if method == "turn/started" {
            params
                .get("turn")
                .and_then(|turn| turn.get("id"))
                .and_then(Value::as_str)
        } else {
            identity(&params, "turnId")
        };
        if turn != self.turn.as_deref() {
            return Ok(false);
        }
        if method == "error" && params.get("willRetry").and_then(Value::as_bool) != Some(true) {
            return Err(protocol());
        }
        if method == "item/started" || method == "item/completed" {
            let item = params.get("item").ok_or_else(failure)?;
            let id = identity(item, "id").ok_or_else(failure)?.to_string();
            if method == "item/completed" {
                // Completing a prompted file item invalidates every owned callback before replies.
                if self.contexts.remove(&id).is_some()
                    && self
                        .pending
                        .values()
                        .any(|item| item.as_deref() == Some(id.as_str()))
                {
                    return Err(failure());
                }
                if item.get("type").and_then(Value::as_str) == Some("agentMessage") {
                    if let Some(text) = item.get("text").and_then(Value::as_str) {
                        if matches!(
                            item.get("phase").and_then(Value::as_str),
                            None | Some("final_answer")
                        ) {
                            self.final_text =
                                crate::bounds::enforce_final_text_bound(Some(text.to_string()));
                        }
                    }
                }
            } else if item.get("type").and_then(Value::as_str) == Some("fileChange") {
                if size(&params) > FRAME || !valid_item(item) {
                    return Err(failure());
                }
                if let Some(previous) = self.contexts.get(&id) {
                    if previous.item != *item {
                        return Err(failure());
                    }
                } else {
                    let bytes = size(item);
                    if self.contexts.len() >= COUNT {
                        return Err(failure());
                    }
                    let retention = stream.retain_json(bytes).map_err(|_| failure())?;
                    self.contexts.insert(
                        id,
                        Context {
                            item: item.clone(),
                            _retention: retention,
                        },
                    );
                }
            }
        }
        if method == "item/fileChange/patchUpdated" {
            if let Some(context) = identity(&params, "itemId").and_then(|id| self.contexts.get(id))
            {
                if params.get("changes") != context.item.get("changes") {
                    return Err(failure());
                }
            }
        }
        if super::is_approval_request_notification(&method, &params) {
            return Err(failure());
        }
        let params = if matches!(method.as_str(), "item/started" | "item/completed")
            && params
                .get("item")
                .and_then(|item| item.get("type"))
                .and_then(Value::as_str)
                == Some("fileChange")
        {
            serde_json::json!({"threadId":params["threadId"],"turnId":params["turnId"],"item":{"id":params["item"]["id"],"type":"fileChange"}})
        } else {
            params
        };
        events.push(CodexBackendEvent::AppServerNotification(
            AppNotification::Raw { method, params },
        ))?;
        Ok(false)
    }
    fn process(
        &mut self,
        message: AppServerMessage,
        handler: Option<&Handler>,
        events: &mut Output,
        stream: &AppServerMessageStream,
    ) -> Result<bool, CodexBackendError> {
        match message {
            AppServerMessage::Request(request) => {
                self.request(request, handler, stream)?;
                Ok(false)
            }
            AppServerMessage::Notification { method, params } => {
                self.notification(method, params, events, stream)
            }
        }
    }
}
pub(super) async fn run(
    mut stream: AppServerMessageStream,
    mut scope: mpsc::UnboundedReceiver<Scope>,
    handler: Option<Handler>,
    events: &mut Output,
) -> Result<Option<String>, CodexBackendError> {
    let mut state = Broker {
        thread: None,
        turn: None,
        queue: VecDeque::new(),
        contexts: BTreeMap::new(),
        pending: BTreeMap::new(),
        callbacks: FuturesUnordered::new(),
        serial: 0,
        final_text: None,
    };
    loop {
        tokio::select! {
            biased;
            _=events.next(),if !events.queue.is_empty()=>{},
            Some(update)=scope.recv()=>{
                match update {Scope::Thread(id)=>state.thread=Some(id),Scope::Turn(id)=>state.turn=Some(id)}
                let mut held=VecDeque::new();
                while let Some((message,retention))=state.queue.pop_front() {
                    if state.ready(&message) {
                        drop(retention);
                        if state.process(message,handler.as_ref(),events,&stream)? {return Ok(state.final_text);}
                    } else {held.push_back((message,retention));}
                }
                state.queue=held;
            }
            message=stream.recv()=>{
                let message=message.ok_or(CodexBackendError::AppServer(McpError::ChannelClosed))?.map_err(|error|match error{McpError::Transport(_)=>failure(),other=>CodexBackendError::AppServer(other)})?;
                if state.ready(&message) {
                    if state.process(message,handler.as_ref(),events,&stream)? {return Ok(state.final_text);}
                } else {state.queue(message,&stream)?;}
            }
            Some((serial,result))=state.callbacks.next(),if !state.callbacks.is_empty()=>{
                state.pending.remove(&serial);
                if let Some((request,result))=result {
                    if !request.is_active() {continue;}
                    match result {
                        Ok(CodexInteractionReply::Result(value))=>request.reply(value).map_err(|_|failure())?,
                        _=>{let _=request.unsupported();return Err(failure());}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Output;
    use crate::{
        backend_harness::DEFAULT_EVENT_CHANNEL_CAPACITY, backends::codex::CodexBackendEvent,
    };
    #[tokio::test]
    async fn output_capacity_fails_explicitly_without_evicting_prior_events() {
        let (sender, _receiver) = tokio::sync::mpsc::channel(1);
        let mut output = Output::new(sender);
        for index in 0..DEFAULT_EVENT_CHANNEL_CAPACITY {
            output
                .push(CodexBackendEvent::TerminalError {
                    message: index.to_string(),
                })
                .unwrap();
        }
        assert!(output
            .push(CodexBackendEvent::TerminalError {
                message: "overflow".into()
            })
            .is_err());
        assert_eq!(output.queue.len(), DEFAULT_EVENT_CHANNEL_CAPACITY);
        let CodexBackendEvent::TerminalError { message } = output.queue.front().unwrap() else {
            panic!("wrong event")
        };
        assert_eq!(message, "0");
    }
}
