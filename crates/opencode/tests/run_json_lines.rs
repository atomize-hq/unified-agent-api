use opencode::{parse_run_json_lines, OpencodeRunJsonLineOutcome};

#[test]
fn parse_run_json_lines_is_tolerant() {
    let input = r#"
{"type":"step_start","session_id":"s1"}
not json
{"type":"text","session_id":"s1","text":"OK"}
"#;

    let outcomes = parse_run_json_lines(input);
    assert_eq!(outcomes.len(), 3);

    match &outcomes[0] {
        OpencodeRunJsonLineOutcome::Ok { event, .. } => {
            assert_eq!(event.event_type(), "step_start");
        }
        other => panic!("expected ok outcome, got {other:?}"),
    }

    match &outcomes[1] {
        OpencodeRunJsonLineOutcome::Err { line, error } => {
            assert!(line.raw.contains("not json"));
            assert_eq!(error.code, opencode::OpencodeRunJsonErrorCode::JsonParse);
        }
        other => panic!("expected error outcome, got {other:?}"),
    }

    match &outcomes[2] {
        OpencodeRunJsonLineOutcome::Ok { event, .. } => match event {
            opencode::OpencodeRunJsonEvent::Text { text, .. } => assert_eq!(text, "OK"),
            other => panic!("expected text event, got {other:?}"),
        },
        other => panic!("expected ok outcome, got {other:?}"),
    }
}
