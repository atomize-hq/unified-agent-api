use super::{
    error_event, status_event, AgentWrapperEvent, AgentWrapperEventKind, AgentWrapperKind, Value,
};

fn tools_facet_data(
    backend_item_id: Option<&str>,
    thread_id: Option<&str>,
    turn_id: Option<&str>,
    kind: &str,
    phase: &str,
    status: &str,
) -> Value {
    serde_json::json!({
        "schema": super::super::TOOLS_FACET_SCHEMA,
        "tool": {
            "backend_item_id": backend_item_id,
            "thread_id": thread_id,
            "turn_id": turn_id,
            "kind": kind,
            "phase": phase,
            "status": status,
            "exit_code": null,
            "bytes": { "stdout": 0, "stderr": 0, "diff": 0, "result": 0 },
            "tool_name": null,
            "tool_use_id": null
        }
    })
}

pub(in crate::backends::codex) fn map_app_server_notification(
    method: &str,
    params: &Value,
) -> Option<AgentWrapperEvent> {
    match method {
        "item/agentMessage/delta" | "item/reasoning/textDelta" | "agentMessage/delta" => {
            let delta = params
                .get("delta")
                .and_then(Value::as_str)
                .or_else(|| params.as_str())?;
            Some(AgentWrapperEvent {
                agent_kind: AgentWrapperKind("codex".to_string()),
                kind: AgentWrapperEventKind::TextOutput,
                channel: Some("assistant".to_string()),
                text: Some(delta.to_string()),
                message: None,
                data: None,
            })
        }
        "reasoning/text/delta" => {
            let text = params
                .get("content")
                .and_then(|content| content.get("text"))
                .and_then(Value::as_str)
                .or_else(|| params.get("text").and_then(Value::as_str))
                .or_else(|| params.as_str())?;
            Some(AgentWrapperEvent {
                agent_kind: AgentWrapperKind("codex".to_string()),
                kind: AgentWrapperEventKind::TextOutput,
                channel: Some("assistant".to_string()),
                text: Some(text.to_string()),
                message: None,
                data: None,
            })
        }
        "turn/started" | "turn/completed" => Some(status_event(None)),
        "item/started" => {
            let item_id = params
                .get("item")
                .and_then(|v| v.get("id"))
                .or_else(|| params.get("item_id"))
                .and_then(Value::as_str);
            let thread_id = params
                .get("threadId")
                .or_else(|| params.get("thread_id"))
                .and_then(Value::as_str);
            let turn_id = params
                .get("turnId")
                .or_else(|| params.get("turn_id"))
                .and_then(Value::as_str);
            let kind = params
                .get("item")
                .and_then(|v| v.get("type"))
                .or_else(|| params.get("item_type"))
                .and_then(Value::as_str)
                .unwrap_or("unknown");

            Some(AgentWrapperEvent {
                agent_kind: AgentWrapperKind("codex".to_string()),
                kind: AgentWrapperEventKind::ToolCall,
                channel: Some("tool".to_string()),
                text: None,
                message: None,
                data: Some(tools_facet_data(
                    item_id, thread_id, turn_id, kind, "start", "running",
                )),
            })
        }
        "item/completed" => {
            let item_id = params
                .get("item")
                .and_then(|v| v.get("id"))
                .or_else(|| params.get("item_id"))
                .and_then(Value::as_str);
            let thread_id = params
                .get("threadId")
                .or_else(|| params.get("thread_id"))
                .and_then(Value::as_str);
            let turn_id = params
                .get("turnId")
                .or_else(|| params.get("turn_id"))
                .and_then(Value::as_str);
            let kind = params
                .get("item")
                .and_then(|v| v.get("type"))
                .or_else(|| params.get("item_type"))
                .and_then(Value::as_str)
                .unwrap_or("unknown");

            Some(AgentWrapperEvent {
                agent_kind: AgentWrapperKind("codex".to_string()),
                kind: AgentWrapperEventKind::ToolResult,
                channel: Some("tool".to_string()),
                text: None,
                message: None,
                data: Some(tools_facet_data(
                    item_id,
                    thread_id,
                    turn_id,
                    kind,
                    "complete",
                    "completed",
                )),
            })
        }
        "error" => {
            // DEF-03: backend messages can contain credentials even when retrying.
            if params.get("willRetry").and_then(Value::as_bool) == Some(true) {
                Some(status_event(Some(
                    "codex app-server retrying after error (details redacted)".into(),
                )))
            } else {
                Some(error_event(
                    "codex app-server error (details redacted)".into(),
                ))
            }
        }
        _ => None,
    }
}

pub(in crate::backends::codex) fn is_approval_request_notification(
    method: &str,
    params: &Value,
) -> bool {
    if matches!(
        method,
        "item/commandExecution/requestApproval"
            | "item/fileChange/requestApproval"
            | "item/permissions/requestApproval"
            | "execCommandApproval"
            | "applyPatchApproval"
    ) {
        return true;
    }
    if method != "codex/event" {
        return false;
    }

    let payload = params.get("msg").unwrap_or(params);
    let payload = match payload.as_object() {
        Some(payload) => payload,
        None => return false,
    };

    let event_type = payload.get("type").and_then(Value::as_str);
    if !matches!(event_type, Some("approval_required" | "approval")) {
        return false;
    }

    let approval_id = payload
        .get("approval_id")
        .or_else(|| payload.get("id"))
        .and_then(Value::as_str);
    approval_id.is_some_and(|id| !id.trim().is_empty())
}
