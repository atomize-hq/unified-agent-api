mod support_paths;

use opencode::{
    OpencodeRunJsonErrorCode, OpencodeRunJsonEvent, OpencodeRunJsonParser,
};

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(support_paths::opencode_run_json_fixtures_dir().join(name))
        .expect("read fixture")
}

fn parse_single_line(name: &str) -> OpencodeRunJsonEvent {
    let mut parser = OpencodeRunJsonParser::new();
    let text = read_fixture(name);
    let line = text
        .lines()
        .find(|line| !line.chars().all(|ch| ch.is_whitespace()))
        .expect("fixture contains one non-empty line");
    parser.parse_line(line).unwrap().unwrap()
}

#[test]
fn parses_step_start_text_and_step_finish() {
    assert!(matches!(
        parse_single_line("step_start.jsonl"),
        OpencodeRunJsonEvent::StepStart { .. }
    ));

    match parse_single_line("text.jsonl") {
        OpencodeRunJsonEvent::Text { text, .. } => assert_eq!(text, "OK"),
        other => panic!("expected text event, got {other:?}"),
    }

    assert!(matches!(
        parse_single_line("step_finish.jsonl"),
        OpencodeRunJsonEvent::StepFinish { .. }
    ));
}

#[test]
fn unknown_outer_type_is_unknown_event_not_error() {
    match parse_single_line("unknown_outer_type.jsonl") {
        OpencodeRunJsonEvent::Unknown { event_type, .. } => assert_eq!(event_type, "tool_call"),
        other => panic!("expected unknown event, got {other:?}"),
    }
}

#[test]
fn missing_required_text_field_is_typedparse() {
    let mut parser = OpencodeRunJsonParser::new();
    let line = read_fixture("missing_text_field.jsonl");
    let err = parser.parse_line(line.trim()).unwrap_err();
    assert_eq!(err.code, OpencodeRunJsonErrorCode::TypedParse);
}

#[test]
fn blank_lines_are_ignored_and_crlf_is_tolerated() {
    let mut parser = OpencodeRunJsonParser::new();
    let text = read_fixture("blank_lines.jsonl");
    let mut count = 0usize;
    for line in text.lines() {
        if let Ok(Some(_)) = parser.parse_line(line) {
            count += 1;
        }
    }
    assert_eq!(count, 3);

    let mut parser = OpencodeRunJsonParser::new();
    let text = read_fixture("crlf_lines.jsonl");
    for line in text.lines() {
        let output = parser.parse_line(&format!("{line}\r")).unwrap();
        assert!(output.is_some());
    }
}

#[test]
fn parser_carries_forward_session_id_across_known_events() {
    let mut parser = OpencodeRunJsonParser::new();
    let step_start = read_fixture("step_start.jsonl");
    let text = read_fixture("text_without_session.jsonl");

    let first = parser.parse_line(step_start.trim()).unwrap().unwrap();
    assert_eq!(first.session_id(), Some("session-test"));

    let second = parser.parse_line(text.trim()).unwrap().unwrap();
    assert_eq!(second.session_id(), Some("session-test"));
}
