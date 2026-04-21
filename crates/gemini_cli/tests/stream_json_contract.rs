mod support_paths;

use gemini_cli::{GeminiStreamJsonErrorCode, GeminiStreamJsonEvent, GeminiStreamJsonParser};

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(support_paths::gemini_stream_json_fixtures_dir().join(name))
        .expect("read fixture")
}

fn parse_single_line(name: &str) -> GeminiStreamJsonEvent {
    let mut parser = GeminiStreamJsonParser::new();
    let text = read_fixture(name);
    let line = text
        .lines()
        .find(|line| !line.chars().all(|ch| ch.is_whitespace()))
        .expect("fixture contains one non-empty line");
    parser.parse_line(line).unwrap().unwrap()
}

#[test]
fn parses_documented_headless_event_types() {
    assert!(matches!(
        parse_single_line("init.jsonl"),
        GeminiStreamJsonEvent::Init { .. }
    ));
    assert!(matches!(
        parse_single_line("message.jsonl"),
        GeminiStreamJsonEvent::Message { .. }
    ));
    assert!(matches!(
        parse_single_line("tool_use.jsonl"),
        GeminiStreamJsonEvent::ToolUse { .. }
    ));
    assert!(matches!(
        parse_single_line("tool_result.jsonl"),
        GeminiStreamJsonEvent::ToolResult { .. }
    ));
    assert!(matches!(
        parse_single_line("error.jsonl"),
        GeminiStreamJsonEvent::Error { .. }
    ));
    assert!(matches!(
        parse_single_line("result_success.jsonl"),
        GeminiStreamJsonEvent::Result { .. }
    ));
}

#[test]
fn unknown_event_type_is_preserved_not_rejected() {
    match parse_single_line("unknown.jsonl") {
        GeminiStreamJsonEvent::Unknown { event_type, .. } => assert_eq!(event_type, "future"),
        other => panic!("expected unknown event, got {other:?}"),
    }
}

#[test]
fn missing_required_field_is_typed_parse() {
    let mut parser = GeminiStreamJsonParser::new();
    let line = read_fixture("missing_message_content.jsonl");
    let err = parser.parse_line(line.trim()).unwrap_err();
    assert_eq!(err.code, GeminiStreamJsonErrorCode::TypedParse);
}

#[test]
fn blank_lines_and_crlf_are_tolerated() {
    let mut parser = GeminiStreamJsonParser::new();
    let text = read_fixture("blank_lines.jsonl");
    let mut count = 0usize;
    for line in text.lines() {
        if let Ok(Some(_)) = parser.parse_line(line) {
            count += 1;
        }
    }
    assert_eq!(count, 3);

    let mut parser = GeminiStreamJsonParser::new();
    let text = read_fixture("crlf_lines.jsonl");
    for line in text.lines() {
        let output = parser.parse_line(&format!("{line}\r")).unwrap();
        assert!(output.is_some());
    }
}
