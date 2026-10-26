//! Owned native app-server interactions, distinct from MCP approval notifications.
use super::{interaction_validation, McpError};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
    task::Poll,
};
use tokio::sync::{mpsc, watch, OwnedSemaphorePermit, Semaphore};

pub(super) const MAX_FRAME: usize = 1024 * 1024;
const MAX_PENDING: usize = 16;
const MAX_RETAINED: usize = 16 * MAX_FRAME;

/// Opaque reservation in this connection's shared 16 MiB retained-JSON budget.
/// Retain one lease alongside each additional owned JSON allocation; moving ownership
/// needs no second lease. Sharing the same immutable allocation needs only one lease.
pub struct AppServerRetention {
    _bytes: OwnedSemaphorePermit,
}

pub(super) struct Delivery {
    pub(super) message: AppServerMessage,
    pub(super) _bytes: Option<OwnedSemaphorePermit>,
}

/// A complete native notification or request, in stdout transport order.
pub enum AppServerMessage {
    Notification { method: String, params: Value },
    Request(AppServerRequest),
}

/// Ordered delivery installed before process startup. Drop invalidates pending replies.
pub struct AppServerMessageStream {
    pub(super) rx: mpsc::Receiver<Delivery>,
    pub(super) errors: mpsc::UnboundedReceiver<McpError>,
    pub(super) state: Arc<InteractionState>,
    pub(super) terminal: bool,
}
impl AppServerMessageStream {
    /// Charge additional retained JSON, such as a private item snapshot in an adapter.
    pub fn retain_json(&self, bytes: usize) -> Result<AppServerRetention, McpError> {
        self.state
            .reserve(bytes)
            .map(|permit| AppServerRetention { _bytes: permit })
    }
    pub async fn recv(&mut self) -> Option<Result<AppServerMessage, McpError>> {
        if self.terminal {
            return None;
        }
        tokio::select! {
            biased;
            Some(delivery) = self.rx.recv() => Some(Ok(delivery.message)),
            Some(error) = self.errors.recv() => {self.terminal=true; self.rx.close(); Some(Err(error))},
            else => None,
        }
    }
}
impl Drop for AppServerMessageStream {
    fn drop(&mut self) {
        self.state.close();
    }
}

/// One native request. RPC identity and connection ownership are deliberately private.
/// Dropping an unanswered request safely refuses it. Debug never exposes its payload.
pub struct AppServerRequest {
    method: String,
    params: Value,
    token: u64,
    active: watch::Receiver<bool>,
    state: Arc<InteractionState>,
    answered: bool,
    _bytes: Arc<OwnedSemaphorePermit>,
}
impl AppServerRequest {
    pub fn method(&self) -> &str {
        &self.method
    }
    pub fn params(&self) -> &Value {
        &self.params
    }
    pub fn is_active(&self) -> bool {
        *self.active.borrow()
    }
    /// Wait for native resolution, owner invalidation, shutdown or disconnect.
    pub async fn cancelled(&mut self) {
        while self.is_active() && self.active.changed().await.is_ok() {}
    }
    /// Invalidate this ownership, for example after a file-change context replacement.
    pub fn invalidate(&self) {
        self.state.invalidate(self.token);
    }
    /// Consume ownership and queue the complete validated native result unchanged.
    pub fn reply(mut self, result: Value) -> Result<(), McpError> {
        if !interaction_validation::valid(&self.method, &result, true)
            || result.to_string().len() > MAX_FRAME
        {
            self.state.refuse(self.token);
            self.answered = true;
            return Err(McpError::Transport(
                "invalid native interaction reply".into(),
            ));
        }
        self.answered = true;
        self.state.queue(self.token, Some(result))
    }
    /// Consume ownership and return an explicit unsupported RPC error.
    pub fn unsupported(mut self) -> Result<(), McpError> {
        self.answered = true;
        self.state.queue(self.token, None)
    }
}
impl Drop for AppServerRequest {
    fn drop(&mut self) {
        if !self.answered {
            self.state.refuse(self.token);
        }
    }
}

pub(super) struct Reply {
    id: Value,
    thread_id: String,
    active: watch::Sender<bool>,
    queued: bool,
    _bytes: Arc<OwnedSemaphorePermit>,
}
#[derive(Default)]
struct Pending {
    closed: bool,
    next_token: u64,
    requests: HashMap<u64, Reply>,
}
pub(super) struct InteractionState {
    pending: Mutex<Pending>,
    writer: mpsc::UnboundedSender<super::jsonrpc::WriteMessage>,
    errors: mpsc::UnboundedSender<McpError>,
    budget: Arc<Semaphore>,
}
impl InteractionState {
    pub(super) fn new(
        writer: mpsc::UnboundedSender<super::jsonrpc::WriteMessage>,
        errors: mpsc::UnboundedSender<McpError>,
    ) -> Arc<Self> {
        Arc::new(Self {
            pending: Mutex::new(Pending::default()),
            writer,
            errors,
            budget: Arc::new(Semaphore::new(MAX_RETAINED)),
        })
    }
    pub(super) fn request(
        self: &Arc<Self>,
        id: Value,
        method: String,
        params: Value,
        frame_bytes: usize,
    ) -> Result<AppServerRequest, McpError> {
        if frame_bytes > MAX_FRAME
            || !(id.is_string() || id.as_i64().is_some())
            || !interaction_validation::valid(&method, &params, false)
        {
            return Err(McpError::Transport(
                "unsupported or malformed native interaction".into(),
            ));
        }
        let bytes = Arc::new(self.reserve(frame_bytes)?);
        let mut p = self.pending.lock().expect("interaction lock");
        if p.closed || p.requests.len() >= MAX_PENDING || p.requests.values().any(|r| r.id == id) {
            return Err(McpError::Transport(
                "native interaction ownership limit or collision".into(),
            ));
        }
        p.next_token += 1;
        let token = p.next_token;
        let (active, rx) = watch::channel(true);
        p.requests.insert(
            token,
            Reply {
                id,
                thread_id: params["threadId"]
                    .as_str()
                    .expect("validated thread")
                    .to_owned(),
                active,
                queued: false,
                _bytes: bytes.clone(),
            },
        );
        Ok(AppServerRequest {
            method,
            params,
            token,
            active: rx,
            state: self.clone(),
            answered: false,
            _bytes: bytes,
        })
    }
    pub(super) fn reserve(&self, size: usize) -> Result<OwnedSemaphorePermit, McpError> {
        self.budget
            .clone()
            .try_acquire_many_owned(u32::try_from(size).map_err(|_| {
                McpError::Transport("native interaction retained-byte limit exceeded".into())
            })?)
            .map_err(|_| {
                McpError::Transport("native interaction retained-byte limit exceeded".into())
            })
    }
    fn queue(self: &Arc<Self>, token: u64, result: Option<Value>) -> Result<(), McpError> {
        let mut p = self.pending.lock().expect("interaction lock");
        let r = p.requests.get_mut(&token).ok_or(McpError::Cancelled)?;
        if r.queued {
            return Err(McpError::Cancelled);
        }
        r.queued = true;
        self.writer
            .send(super::jsonrpc::WriteMessage::Reply {
                state: self.clone(),
                token,
                result,
            })
            .map_err(|_| McpError::ChannelClosed)
    }
    pub(super) fn render_reply(&self, token: u64, result: Option<Value>) -> Option<String> {
        let p = self.pending.lock().expect("interaction lock");
        let r = p.requests.get(&token)?;
        Some(match result { Some(result)=>json!({"jsonrpc":"2.0","id":r.id,"result":result}), None=>json!({"jsonrpc":"2.0","id":r.id,"error":{"code":-32601,"message":"client interaction unsupported"}}) }.to_string())
    }
    pub(super) fn reply_watch(&self, token: u64) -> Option<watch::Receiver<bool>> {
        self.pending
            .lock()
            .expect("interaction lock")
            .requests
            .get(&token)
            .map(|r| r.active.subscribe())
    }
    /// Linearize ownership with the first actual I/O poll, never while merely queued.
    /// The mutex covers one nonblocking poll only; no guard crosses an await.
    pub(super) fn poll_reply(
        &self,
        token: u64,
        write: impl FnOnce() -> Poll<io::Result<usize>>,
    ) -> Poll<io::Result<Option<usize>>> {
        let mut p = self.pending.lock().expect("interaction lock");
        if !p.requests.contains_key(&token) {
            return Poll::Ready(Ok(None));
        }
        match write() {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => {
                if let Some(r) = p.requests.remove(&token) {
                    let _ = r.active.send(false);
                }
                Poll::Ready(result.map(Some))
            }
        }
    }
    fn refuse(self: &Arc<Self>, token: u64) {
        let _ = self.queue(token, None);
    }
    fn invalidate(&self, token: u64) {
        if let Some(r) = self
            .pending
            .lock()
            .expect("interaction lock")
            .requests
            .remove(&token)
        {
            let _ = r.active.send(false);
        }
    }
    pub(super) fn resolved(&self, id: &Value, thread_id: Option<&str>) {
        let mut p = self.pending.lock().expect("interaction lock");
        p.requests.retain(|_, r| {
            if &r.id == id && Some(r.thread_id.as_str()) == thread_id {
                let _ = r.active.send(false);
                false
            } else {
                true
            }
        });
    }
    pub(super) fn close(&self) {
        let mut p = self.pending.lock().expect("interaction lock");
        p.closed = true;
        self.budget.close();
        for (_, r) in p.requests.drain() {
            let _ = r.active.send(false);
        }
    }
    pub(super) fn fail(&self, message: &str) {
        self.close();
        let _ = self.errors.send(McpError::Transport(message.into()));
    }
}

#[cfg(test)]
mod tests {
    use super::{InteractionState, MAX_FRAME, MAX_RETAINED};
    use crate::mcp::jsonrpc::WriteMessage;
    use serde_json::json;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn queued_reply_rechecks_ownership_at_writer_and_dropped_requests_refuse() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (errors, _) = mpsc::unbounded_channel();
        let state = InteractionState::new(tx, errors);
        let params = json!({"threadId":"t","turnId":"u","itemId":"i","startedAtMs":0});
        let request = state
            .request(
                json!(1),
                "item/fileChange/requestApproval".into(),
                params.clone(),
                100,
            )
            .unwrap();
        request.reply(json!({"decision":"accept"})).unwrap();
        let WriteMessage::Reply {
            state: queued,
            token,
            result,
        } = rx.recv().await.unwrap()
        else {
            panic!("reply")
        };
        let watch = queued.reply_watch(token).unwrap();
        assert!(queued
            .poll_reply(token, || std::task::Poll::Pending)
            .is_pending());
        queued.resolved(&json!(1), Some("wrong"));
        assert!(!watch.has_changed().unwrap());
        queued.resolved(&json!(1), Some("t"));
        assert!(
            !*watch.borrow(),
            "resolution records inactive ownership before closing sender"
        );
        assert!(matches!(
            queued.poll_reply(token, || panic!("invalidated reply must not poll I/O")),
            std::task::Poll::Ready(Ok(None))
        ));
        assert!(
            queued.render_reply(token, result).is_none(),
            "resolved queued reply must never reach writer"
        );
        let request = state
            .request(
                json!("1"),
                "item/fileChange/requestApproval".into(),
                params,
                100,
            )
            .unwrap();
        drop(request);
        let WriteMessage::Reply {
            state: queued,
            token,
            result,
        } = rx.recv().await.unwrap()
        else {
            panic!("refusal")
        };
        let value: serde_json::Value =
            serde_json::from_str(&queued.render_reply(token, result).unwrap()).unwrap();
        assert_eq!(value["id"], "1");
        assert_eq!(value["error"]["code"], -32601);
        assert!(value.get("result").is_none());
    }

    #[test]
    fn retained_byte_budget_is_exact_shared_and_released() {
        let (tx, _) = mpsc::unbounded_channel();
        let (errors, _) = mpsc::unbounded_channel();
        let state = InteractionState::new(tx, errors);
        let mut permits = Vec::new();
        for _ in 0..(MAX_RETAINED / MAX_FRAME) {
            permits.push(state.reserve(MAX_FRAME).unwrap());
        }
        assert!(state.reserve(1).is_err());
        permits.pop();
        assert!(state.reserve(MAX_FRAME).is_ok());
        state.close();
        assert!(state.reserve(1).is_err());
    }
}
