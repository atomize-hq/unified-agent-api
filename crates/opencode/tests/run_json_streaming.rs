use std::{fs, time::Duration};

mod support_paths;

use futures_util::StreamExt;
use opencode::{OpencodeClient, OpencodeRunJsonEvent, OpencodeRunRequest};
use tempfile::NamedTempFile;

fn make_fake_client(scenario: &str) -> OpencodeClient {
    OpencodeClient::builder()
        .binary(support_paths::target_debug_binary("fake_opencode_run_json"))
        .env("FAKE_OPENCODE_SCENARIO", scenario)
        .build()
}

#[tokio::test]
async fn run_json_yields_events_incrementally_and_tracks_completion_text() {
    let client = make_fake_client("three_events_delayed");
    let handle = client
        .run_json(OpencodeRunRequest::new("Reply with OK."))
        .await
        .unwrap();

    let mut events = handle.events;
    let mut completion = handle.completion;

    let first = tokio::select! {
        biased;
        result = &mut completion => panic!("completion resolved before first event: {result:?}"),
        item = events.next() => item.expect("stream open").expect("event parses"),
    };
    assert!(matches!(first, OpencodeRunJsonEvent::StepStart { .. }));

    let second = tokio::time::timeout(Duration::from_secs(1), events.next())
        .await
        .expect("second event timeout")
        .expect("stream open")
        .expect("event parses");
    match second {
        OpencodeRunJsonEvent::Text { text, .. } => assert_eq!(text, "OK"),
        other => panic!("expected text event, got {other:?}"),
    }

    let third = tokio::time::timeout(Duration::from_secs(1), events.next())
        .await
        .expect("third event timeout")
        .expect("stream open")
        .expect("event parses");
    assert!(matches!(third, OpencodeRunJsonEvent::StepFinish { .. }));

    assert!(
        events.next().await.is_none(),
        "expected event stream to close"
    );

    let completion = completion.await.unwrap();
    assert!(completion.status.success());
    assert_eq!(completion.final_text.as_deref(), Some("OK"));
}

#[tokio::test]
async fn run_json_ignores_crlf_and_blank_lines() {
    let client = make_fake_client("crlf_blank_lines");
    let handle = client
        .run_json(OpencodeRunRequest::new("Reply with OK."))
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;

    assert!(matches!(
        events.next().await.unwrap().unwrap(),
        OpencodeRunJsonEvent::StepStart { .. }
    ));
    assert!(matches!(
        events.next().await.unwrap().unwrap(),
        OpencodeRunJsonEvent::Text { .. }
    ));
    assert!(matches!(
        events.next().await.unwrap().unwrap(),
        OpencodeRunJsonEvent::StepFinish { .. }
    ));
    assert!(events.next().await.is_none());

    let completion = completion.await.unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn run_json_redacts_parse_errors_and_continues() {
    let client = make_fake_client("parse_error_redaction");
    let handle = client
        .run_json(OpencodeRunRequest::new("Reply with OK."))
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;

    let first = events
        .next()
        .await
        .expect("stream open")
        .expect_err("expected parse error");
    let secret = "VERY_SECRET_SHOULD_NOT_APPEAR";
    assert!(!first.message.contains(secret));
    assert!(!first.details.contains(secret));

    assert!(matches!(
        events.next().await.unwrap().unwrap(),
        OpencodeRunJsonEvent::StepStart { .. }
    ));
    assert!(matches!(
        events.next().await.unwrap().unwrap(),
        OpencodeRunJsonEvent::Text { .. }
    ));
    assert!(matches!(
        events.next().await.unwrap().unwrap(),
        OpencodeRunJsonEvent::StepFinish { .. }
    ));
    assert!(events.next().await.is_none());

    let completion = completion.await.unwrap();
    assert!(completion.status.success());
    assert_eq!(completion.final_text.as_deref(), Some("OK"));
}

#[tokio::test]
async fn run_json_control_termination_closes_stream_and_yields_non_success_status() {
    let client = make_fake_client("slow_until_killed");
    let control = client
        .run_json_control(OpencodeRunRequest::new("Reply with OK."))
        .await
        .unwrap();

    let mut events = control.events;
    let completion = control.completion;
    let termination = control.termination;

    assert!(matches!(
        tokio::time::timeout(Duration::from_secs(1), events.next())
            .await
            .expect("first event timeout")
            .expect("stream open")
            .expect("event parses"),
        OpencodeRunJsonEvent::StepStart { .. }
    ));

    termination.request_termination();

    tokio::time::timeout(Duration::from_secs(2), async {
        while events.next().await.is_some() {}
    })
    .await
    .expect("expected event stream to close after termination");

    let completion = completion.await.unwrap();
    assert!(!completion.status.success());
    assert!(completion.final_text.is_none());
}

#[tokio::test]
async fn run_json_generic_runtime_failure_surfaces_terminal_error_and_completion_error() {
    let client = make_fake_client("runtime_failure_invalid_model");
    let handle = client
        .run_json(OpencodeRunRequest::new("Reply with OK."))
        .await
        .unwrap();

    let mut events = handle.events;
    let first = events
        .next()
        .await
        .expect("stream open")
        .expect("typed terminal error event");
    match first {
        OpencodeRunJsonEvent::TerminalError { message, .. } => {
            assert_eq!(message, "opencode run failed");
            assert!(!message.contains("SECRET_MODEL_REJECTION_DO_NOT_LEAK"));
        }
        other => panic!("expected terminal error event, got {other:?}"),
    }
    assert!(
        events.next().await.is_none(),
        "runtime failure should end the stream"
    );

    let error = handle
        .completion
        .await
        .expect_err("runtime failure must surface completion error");
    match error {
        opencode::OpencodeError::RunFailed { status, message } => {
            assert!(!status.success());
            assert_eq!(message, "opencode run failed");
            assert!(!message.contains("SECRET_MODEL_REJECTION_DO_NOT_LEAK"));
        }
        other => panic!("expected run failure, got {other:?}"),
    }
}

#[tokio::test]
async fn run_json_passes_only_the_accepted_controls_on_the_canonical_surface() {
    let capture_file = NamedTempFile::new().unwrap();
    let capture_path = capture_file.path().to_path_buf();

    let client = OpencodeClient::builder()
        .binary(support_paths::target_debug_binary("fake_opencode_run_json"))
        .env("FAKE_OPENCODE_SCENARIO", "capture_args")
        .env("FAKE_OPENCODE_CAPTURE", capture_path.display().to_string())
        .build();

    let request = OpencodeRunRequest::new("Reply with OK.")
        .model("opencode/gpt-5-nano")
        .session("session-123")
        .continue_session(true)
        .fork(true)
        .working_dir(".");

    let handle = client.run_json(request).await.unwrap();
    let mut events = handle.events;
    while events.next().await.is_some() {}
    let completion = handle.completion.await.unwrap();
    assert!(completion.status.success());

    let capture: serde_json::Value =
        serde_json::from_slice(&fs::read(&capture_path).expect("read capture")).unwrap();
    let argv = capture["argv"].as_array().expect("argv array");
    let argv = argv
        .iter()
        .map(|value| value.as_str().expect("argv string"))
        .collect::<Vec<_>>();

    assert_eq!(
        argv,
        vec![
            "run",
            "--format",
            "json",
            "--model",
            "opencode/gpt-5-nano",
            "--session",
            "session-123",
            "--continue",
            "--fork",
            "--dir",
            ".",
            "Reply with OK.",
        ]
    );
}

#[tokio::test]
async fn run_json_rejects_empty_prompts_before_spawn() {
    let client = make_fake_client("capture_args");
    let error = client
        .run_json(OpencodeRunRequest::new("   "))
        .await
        .unwrap_err();
    match error {
        opencode::OpencodeError::InvalidRequest(message) => {
            assert!(message.contains("prompt"));
        }
        other => panic!("expected invalid request, got {other:?}"),
    }
}

#[tokio::test]
async fn run_json_classifies_resume_last_selection_failure_without_leaking_stderr() {
    let client = make_fake_client("session_not_found_last");
    let handle = client
        .run_json(OpencodeRunRequest::new("Reply with OK.").continue_session(true))
        .await
        .unwrap();

    let mut events = handle.events;
    let first = events
        .next()
        .await
        .expect("stream open")
        .expect("typed terminal error event");
    match first {
        OpencodeRunJsonEvent::TerminalError { message, .. } => {
            assert_eq!(message, "no session found");
            assert!(!message.contains("SECRET_LAST_SESSION_SCOPE"));
        }
        other => panic!("expected terminal error event, got {other:?}"),
    }
    assert!(
        events.next().await.is_none(),
        "selection failure should end the stream"
    );

    let error = handle
        .completion
        .await
        .expect_err("selection failure must surface completion error");
    match error {
        opencode::OpencodeError::SelectionFailed { message } => {
            assert_eq!(message, "no session found");
            assert!(!message.contains("SECRET_LAST_SESSION_SCOPE"));
        }
        other => panic!("expected selection failure, got {other:?}"),
    }
}

#[tokio::test]
async fn run_json_classifies_resume_id_selection_failure_without_leaking_stderr() {
    let client = make_fake_client("session_not_found_id");
    let handle = client
        .run_json(OpencodeRunRequest::new("Reply with OK.").session("session-123"))
        .await
        .unwrap();

    let mut events = handle.events;
    let first = events
        .next()
        .await
        .expect("stream open")
        .expect("typed terminal error event");
    match first {
        OpencodeRunJsonEvent::TerminalError { message, .. } => {
            assert_eq!(message, "session not found");
            assert!(!message.contains("SECRET_SESSION_ID_DO_NOT_LEAK"));
        }
        other => panic!("expected terminal error event, got {other:?}"),
    }
    assert!(
        events.next().await.is_none(),
        "selection failure should end the stream"
    );

    let error = handle
        .completion
        .await
        .expect_err("selection failure must surface completion error");
    match error {
        opencode::OpencodeError::SelectionFailed { message } => {
            assert_eq!(message, "session not found");
            assert!(!message.contains("SECRET_SESSION_ID_DO_NOT_LEAK"));
        }
        other => panic!("expected selection failure, got {other:?}"),
    }
}
