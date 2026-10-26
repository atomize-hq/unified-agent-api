use super::*;
use codex::mcp::{AppCallHandle, AppNotification, CodexAppServer, McpError, TurnInterruptParams};

fn protocol_error() -> CodexBackendError {
    CodexBackendError::AppServer(McpError::Server("invalid turn lifecycle".into()))
}

async fn interrupt(server: &CodexAppServer, thread_id: &str, turn_id: &str) {
    // Queuing the request precedes closing stdin, which drains the writer queue.
    let _ = server
        .turn_interrupt(TurnInterruptParams {
            thread_id: Some(thread_id.to_string()),
            turn_id: turn_id.to_string(),
        })
        .await;
}

pub(super) async fn run_turn(
    server: Arc<CodexAppServer>,
    thread_id: String,
    mut call: AppCallHandle,
    timeout: Option<Duration>,
    mut cancel: mpsc::UnboundedReceiver<()>,
    events: &mpsc::UnboundedSender<CodexBackendEvent>,
) -> Result<CodexBackendCompletion, CodexBackendError> {
    // One budget covers both acceptance and execution. Startup acceptance is also bounded.
    let deadline = async {
        match timeout {
            Some(duration) => tokio::time::sleep(duration).await,
            None => std::future::pending().await,
        }
    };
    tokio::pin!(deadline);
    let mut active_id = None;
    let outcome = async {
        let accepted = tokio::select! {
            biased;
            Some(()) = cancel.recv() => return Err(CodexBackendError::AppServer(McpError::Cancelled)),
            _ = &mut deadline => return Err(CodexBackendError::Timeout { timeout: timeout.unwrap_or_default() }),
            result = tokio::time::timeout(Duration::from_secs(3), &mut call.response) => {
                result.map_err(|_| CodexBackendError::AppServer(McpError::Timeout(Duration::from_secs(3))))?
                    .map_err(|_| CodexBackendError::AppServer(McpError::ChannelClosed))?
                    .map_err(CodexBackendError::AppServer)?
            }
        };
        let turn = accepted.get("turn").ok_or_else(protocol_error)?;
        if turn.get("items").and_then(Value::as_array).is_none() { return Err(protocol_error()); }
        let turn_id = turn.get("id").and_then(Value::as_str).filter(|id| !id.trim().is_empty()).ok_or_else(protocol_error)?.to_string();
        if !matches!(turn.get("status").and_then(Value::as_str), Some("inProgress" | "completed" | "failed" | "interrupted")) { return Err(protocol_error()); }
        active_id = Some(turn_id.clone());
        loop {
            let notification = tokio::select! {
                biased;
                Some(()) = cancel.recv() => return Err(CodexBackendError::AppServer(McpError::Cancelled)),
                _ = &mut deadline => return Err(CodexBackendError::Timeout { timeout: timeout.unwrap_or_default() }),
                notification = call.events.recv() => notification.ok_or_else(|| CodexBackendError::AppServer(McpError::ChannelClosed))?
            };
            let AppNotification::Raw { method, params } = &notification else { continue; };
            if is_approval_request_notification(method, params) {
                return Err(CodexBackendError::AppServer(McpError::Server(PINNED_APPROVAL_REQUIRED.into())));
            }
            // The transport broadcasts notifications. Only this exact thread/turn belongs here.
            if params.get("threadId").and_then(Value::as_str) != Some(thread_id.as_str()) { continue; }
            let notification_turn = params.get("turnId").or_else(|| params.get("turn").and_then(|t| t.get("id"))).and_then(Value::as_str);
            if notification_turn != Some(turn_id.as_str()) { continue; }
            if method == "turn/completed" {
                let turn = params.get("turn").ok_or_else(protocol_error)?;
                if turn.get("items").and_then(Value::as_array).is_none() || turn.get("error").is_some_and(|error| !error.is_null()) { return Err(protocol_error()); }
                match turn.get("status").and_then(Value::as_str) {
                    Some("completed") => { let _ = events.send(CodexBackendEvent::AppServerNotification(notification)); return Ok(()); }
                    Some("interrupted") => return Err(CodexBackendError::AppServer(McpError::Cancelled)),
                    _ => return Err(protocol_error()),
                }
            }
            if method == "error" && params.get("willRetry").and_then(Value::as_bool) == Some(false) { return Err(protocol_error()); }
            let _ = events.send(CodexBackendEvent::AppServerNotification(notification));
        }
    }.await;
    if outcome.is_err() {
        if let Some(turn_id) = active_id.as_deref() {
            interrupt(&server, &thread_id, turn_id).await;
        }
    }
    let status = server
        .shutdown_and_wait()
        .await
        .map_err(CodexBackendError::AppServer);
    let outcome = outcome.and_then(|()| {
        let status = status?;
        if !status.success() {
            return Err(protocol_error());
        }
        Ok(CodexBackendCompletion {
            status,
            final_text: None,
            backend_error_message: None,
            selection_failure_message: None,
        })
    });
    if let Err(err) = &outcome {
        let message = super::super::harness::render_backend_error_message(err);
        let _ = events.send(CodexBackendEvent::TerminalError { message });
    }
    outcome
}
