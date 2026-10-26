use super::*;
use codex::mcp::{AppCallHandle, AppNotification, CodexAppServer, McpError, TurnInterruptParams};

fn protocol_error() -> CodexBackendError {
    CodexBackendError::AppServer(McpError::Server("invalid turn lifecycle".into()))
}

pub(super) async fn interrupt(server: &CodexAppServer, thread_id: &str, turn_id: &str) {
    // Queuing the request precedes closing stdin, which drains the writer queue.
    let _ = server
        .turn_interrupt(TurnInterruptParams {
            thread_id: Some(thread_id.to_string()),
            turn_id: turn_id.to_string(),
        })
        .await;
}

pub(super) async fn run_turn(
    thread_id: String,
    mut call: AppCallHandle,
    active_id: &mut Option<String>,
    events: &mpsc::UnboundedSender<CodexBackendEvent>,
) -> Result<(), CodexBackendError> {
    let accepted = tokio::time::timeout(Duration::from_secs(3), &mut call.response)
        .await
        .map_err(|_| CodexBackendError::AppServer(McpError::Timeout(Duration::from_secs(3))))?
        .map_err(|_| CodexBackendError::AppServer(McpError::ChannelClosed))?
        .map_err(CodexBackendError::AppServer)?;
    let turn = accepted.get("turn").ok_or_else(protocol_error)?;
    if turn.get("items").and_then(Value::as_array).is_none() {
        return Err(protocol_error());
    }
    let turn_id = turn
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(protocol_error)?
        .to_string();
    if !matches!(
        turn.get("status").and_then(Value::as_str),
        Some("inProgress" | "completed" | "failed" | "interrupted")
    ) {
        return Err(protocol_error());
    }
    *active_id = Some(turn_id.clone());
    while let Some(notification) = call.events.recv().await {
        let AppNotification::Raw { method, params } = &notification else {
            continue;
        };
        if is_approval_request_notification(method, params) {
            return Err(CodexBackendError::AppServer(McpError::Server(
                PINNED_APPROVAL_REQUIRED.into(),
            )));
        }
        if params.get("threadId").and_then(Value::as_str) != Some(thread_id.as_str()) {
            continue;
        }
        if method == "turn/completed" {
            // DEF-04 / AS-1: only the schema's nested identity is authoritative.
            let turn = params.get("turn").ok_or_else(protocol_error)?;
            let nested_id = turn
                .get("id")
                .and_then(Value::as_str)
                .filter(|id| !id.trim().is_empty())
                .ok_or_else(protocol_error)?;
            if params
                .get("turnId")
                .is_some_and(|id| id.as_str() != Some(nested_id))
            {
                return Err(protocol_error());
            }
            if nested_id != turn_id {
                continue;
            }
            if turn.get("items").and_then(Value::as_array).is_none()
                || turn.get("error").is_some_and(|error| !error.is_null())
            {
                return Err(protocol_error());
            }
            match turn.get("status").and_then(Value::as_str) {
                Some("completed") => {
                    let _ = events.send(CodexBackendEvent::AppServerNotification(notification));
                    return Ok(());
                }
                Some("interrupted") => {
                    return Err(CodexBackendError::AppServer(McpError::Cancelled))
                }
                _ => return Err(protocol_error()),
            }
        }
        let notification_turn = if method == "turn/started" {
            params.get("turn").and_then(|turn| turn.get("id"))
        } else {
            params.get("turnId")
        }
        .and_then(Value::as_str);
        if notification_turn != Some(turn_id.as_str()) {
            continue;
        }
        if method == "error" && params.get("willRetry").and_then(Value::as_bool) != Some(true) {
            return Err(protocol_error());
        }
        let _ = events.send(CodexBackendEvent::AppServerNotification(notification));
    }
    Err(CodexBackendError::AppServer(McpError::ChannelClosed))
}
