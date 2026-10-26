use super::support::*;
use serde_json::{json, Value};

#[test]
fn app_server_agent_message_delta_maps_to_text_output_without_data() {
    let event =
        fork::map_app_server_notification("agentMessage/delta", &json!("hello")).expect("mapped");
    assert_eq!(event.kind, AgentWrapperEventKind::TextOutput);
    assert_eq!(event.channel.as_deref(), Some("assistant"));
    assert_eq!(event.text.as_deref(), Some("hello"));
    assert_eq!(event.data, None);
}

#[test]
fn app_server_reasoning_delta_prefers_content_text_field_when_present() {
    let params = json!({"content": {"text": "hi"}});
    let event = fork::map_app_server_notification("reasoning/text/delta", &params).expect("mapped");
    assert_eq!(event.kind, AgentWrapperEventKind::TextOutput);
    assert_eq!(event.channel.as_deref(), Some("assistant"));
    assert_eq!(event.text.as_deref(), Some("hi"));
    assert_eq!(event.data, None);
}

#[test]
fn app_server_item_started_maps_to_tool_call_with_metadata_only_facet() {
    let params = json!({
        "item_id": "item-1",
        "thread_id": "thread-1",
        "turn_id": "turn-1",
        "item_type": "command_execution"
    });
    let event = fork::map_app_server_notification("item/started", &params).expect("mapped");
    assert_eq!(event.kind, AgentWrapperEventKind::ToolCall);
    assert_eq!(tool_schema(&event), Some(TOOLS_FACET_SCHEMA));
    assert_eq!(
        tool_field(&event, "phase").and_then(Value::as_str),
        Some("start")
    );
    assert_eq!(
        tool_field(&event, "status").and_then(Value::as_str),
        Some("running")
    );
    assert_eq!(
        tool_field(&event, "kind").and_then(Value::as_str),
        Some("command_execution")
    );
}

#[test]
fn app_server_error_maps_to_error_event_with_safe_message_and_no_data() {
    let secret = "SECRET_SHOULD_NOT_LEAK";
    let params = json!({
        "error": {"message": "boom", "additionalDetails": {"secret": secret}},
        "message": secret
    });
    let event = fork::map_app_server_notification("error", &params).expect("mapped");
    assert_eq!(event.kind, AgentWrapperEventKind::Error);
    assert_eq!(event.message.as_deref(), Some("boom"));
    assert!(!event.message.as_deref().unwrap().contains(secret));
    assert_eq!(event.data, None);
}

#[test]
fn approval_request_detector_matches_direct_and_wrapped_payloads() {
    assert!(fork::is_approval_request_notification(
        "codex/event",
        &json!({"type": "approval_required", "approval_id": "ap-1", "kind": "exec"}),
    ));
    assert!(fork::is_approval_request_notification(
        "codex/event",
        &json!({"msg": {"type": "approval", "id": "ap-1", "approval_kind": "exec"}}),
    ));
    assert!(!fork::is_approval_request_notification(
        "codex/event",
        &json!({"type": "task_complete"}),
    ));
    assert!(!fork::is_approval_request_notification(
        "agentMessage/delta",
        &json!({"type": "approval_required"}),
    ));
}

#[test]
fn pinned_notification_names_and_nested_item_fields_are_mapped() {
    for method in ["item/agentMessage/delta", "item/reasoning/textDelta"] {
        let event = fork::map_app_server_notification(method, &json!({"threadId":"thread-1","turnId":"turn-1","itemId":"item-1","delta":"hello","contentIndex":0})).unwrap();
        assert_eq!(event.text.as_deref(), Some("hello"));
        assert_eq!(event.data, None);
        assert!(fork::map_app_server_notification(method, &json!({"delta":42})).is_none());
    }
    for (method, kind) in [
        ("item/started", AgentWrapperEventKind::ToolCall),
        ("item/completed", AgentWrapperEventKind::ToolResult),
    ] {
        let event = fork::map_app_server_notification(method, &json!({"threadId":"thread-1","turnId":"turn-1","item":{"id":"item-1","type":"commandExecution","command":"SECRET"},"startedAtMs":null,"completedAtMs":null})).unwrap();
        assert_eq!(event.kind, kind);
        assert_eq!(
            tool_field(&event, "backend_item_id").and_then(Value::as_str),
            Some("item-1")
        );
        assert_eq!(
            tool_field(&event, "thread_id").and_then(Value::as_str),
            Some("thread-1")
        );
        assert_eq!(
            tool_field(&event, "turn_id").and_then(Value::as_str),
            Some("turn-1")
        );
        assert_eq!(
            tool_field(&event, "kind").and_then(Value::as_str),
            Some("commandExecution")
        );
        assert!(!format!("{event:?}").contains("SECRET"));
    }
    assert!(fork::map_app_server_notification("future/unknown", &json!({})).is_none());
}
