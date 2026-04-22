use std::{
    fs,
    process::ExitStatus,
    sync::OnceLock,
    task::{Context, Poll},
    time::Duration,
};

mod support_paths;

use futures_util::{task::noop_waker, StreamExt};
use gemini_cli::{
    GeminiCliClient, GeminiCliError, GeminiStreamJsonEvent, GeminiStreamJsonRunRequest,
};
use tempfile::NamedTempFile;

static FAKE_BINARY: OnceLock<std::path::PathBuf> = OnceLock::new();

fn fake_gemini_binary() -> std::path::PathBuf {
    FAKE_BINARY
        .get_or_init(|| {
            let binary = support_paths::target_debug_binary("fake_gemini_stream_json");
            if binary.exists() {
                return binary;
            }

            let output = std::process::Command::new("cargo")
                .args([
                    "build",
                    "-p",
                    "unified-agent-api-gemini-cli",
                    "--bin",
                    "fake_gemini_stream_json",
                ])
                .current_dir(support_paths::repo_root())
                .output()
                .expect("spawn cargo build for fake gemini binary");

            assert!(
                output.status.success(),
                "cargo build failed: status={:?}, stderr={}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                binary.exists(),
                "fake gemini binary should exist after cargo build"
            );
            binary
        })
        .clone()
}

fn make_fake_client(scenario: &str) -> GeminiCliClient {
    GeminiCliClient::builder()
        .binary(fake_gemini_binary())
        .env("FAKE_GEMINI_SCENARIO", scenario)
        .build()
}

#[tokio::test]
async fn stream_json_yields_events_incrementally_and_tracks_completion_text() {
    let client = make_fake_client("three_events_delayed");
    let handle = client
        .stream_json(GeminiStreamJsonRunRequest::new("Reply with OK."))
        .await
        .unwrap();

    let mut events = handle.events;
    let mut completion = handle.completion;

    let first = tokio::select! {
        biased;
        result = &mut completion => panic!("completion resolved before first event: {result:?}"),
        item = events.next() => item.expect("stream open").expect("event parses"),
    };
    assert!(matches!(first, GeminiStreamJsonEvent::Init { .. }));

    let second = tokio::time::timeout(Duration::from_secs(1), events.next())
        .await
        .expect("second event timeout")
        .expect("stream open")
        .expect("event parses");
    match second {
        GeminiStreamJsonEvent::Message { content, .. } => assert_eq!(content, "OK"),
        other => panic!("expected message event, got {other:?}"),
    }

    let third = tokio::time::timeout(Duration::from_secs(1), events.next())
        .await
        .expect("third event timeout")
        .expect("stream open")
        .expect("event parses");
    assert!(matches!(third, GeminiStreamJsonEvent::Result { .. }));

    assert!(
        events.next().await.is_none(),
        "expected event stream to close"
    );

    let completion = completion.await.unwrap();
    assert!(completion.status.success());
    assert_eq!(completion.final_text.as_deref(), Some("OK"));
    assert_eq!(completion.session_id.as_deref(), Some("session-test"));
    assert_eq!(completion.model.as_deref(), Some("gemini-2.5-flash"));
    assert!(completion.raw_result.is_some());
}

#[tokio::test]
async fn stream_json_ignores_crlf_and_blank_lines() {
    let client = make_fake_client("crlf_blank_lines");
    let handle = client
        .stream_json(GeminiStreamJsonRunRequest::new("Reply with OK."))
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;

    assert!(matches!(
        events.next().await.unwrap().unwrap(),
        GeminiStreamJsonEvent::Init { .. }
    ));
    assert!(matches!(
        events.next().await.unwrap().unwrap(),
        GeminiStreamJsonEvent::Message { .. }
    ));
    assert!(matches!(
        events.next().await.unwrap().unwrap(),
        GeminiStreamJsonEvent::Result { .. }
    ));
    assert!(events.next().await.is_none());

    let completion = completion.await.unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn stream_json_surfaces_parse_errors_without_leaking_raw_lines() {
    let client = make_fake_client("parse_error_redaction");
    let handle = client
        .stream_json(GeminiStreamJsonRunRequest::new("Reply with OK."))
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
        GeminiStreamJsonEvent::Init { .. }
    ));
    assert!(matches!(
        events.next().await.unwrap().unwrap(),
        GeminiStreamJsonEvent::Message { .. }
    ));
    assert!(matches!(
        events.next().await.unwrap().unwrap(),
        GeminiStreamJsonEvent::Result { .. }
    ));
    assert!(events.next().await.is_none());

    let completion = completion.await.unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn stream_json_maps_headless_tool_events() {
    let client = make_fake_client("tool_roundtrip");
    let handle = client
        .stream_json(GeminiStreamJsonRunRequest::new("Run pwd."))
        .await
        .unwrap();

    let events = handle
        .events
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|item| item.expect("event parses"))
        .collect::<Vec<_>>();

    assert!(matches!(events[0], GeminiStreamJsonEvent::Init { .. }));
    assert!(matches!(events[1], GeminiStreamJsonEvent::ToolUse { .. }));
    assert!(matches!(
        events[2],
        GeminiStreamJsonEvent::ToolResult { .. }
    ));
    assert!(matches!(events[3], GeminiStreamJsonEvent::Message { .. }));
    assert!(matches!(events[4], GeminiStreamJsonEvent::Result { .. }));

    let completion = handle.completion.await.unwrap();
    assert_eq!(completion.final_text.as_deref(), Some("done"));
}

#[tokio::test]
async fn stream_json_control_termination_closes_stream_and_yields_non_success_status() {
    let client = make_fake_client("slow_until_killed");
    let control = client
        .stream_json_control(GeminiStreamJsonRunRequest::new("Reply with OK."))
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
        GeminiStreamJsonEvent::Init { .. }
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
async fn stream_json_passes_only_the_headless_contract_and_cwd() {
    let capture_file = NamedTempFile::new().unwrap();
    let capture_path = capture_file.path().to_path_buf();
    let working_dir = tempfile::tempdir().unwrap();

    let client = GeminiCliClient::builder()
        .binary(fake_gemini_binary())
        .env("FAKE_GEMINI_SCENARIO", "capture_args")
        .env("FAKE_GEMINI_CAPTURE", capture_path.display().to_string())
        .build();

    let request = GeminiStreamJsonRunRequest::new("Reply with OK.")
        .model("gemini-2.5-flash")
        .working_dir(working_dir.path());

    let handle = client.stream_json(request).await.unwrap();
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
            "--prompt",
            "Reply with OK.",
            "--output-format",
            "stream-json",
            "--model",
            "gemini-2.5-flash",
        ]
    );
    let expected_cwd = std::fs::canonicalize(working_dir.path())
        .expect("canonical working dir")
        .display()
        .to_string();
    assert_eq!(capture["cwd"].as_str(), Some(expected_cwd.as_str()));
}

#[tokio::test]
async fn stream_json_rejects_empty_prompts_before_spawn() {
    let client = make_fake_client("capture_args");
    let error = client
        .stream_json(GeminiStreamJsonRunRequest::new("   "))
        .await
        .unwrap_err();

    match error {
        GeminiCliError::InvalidRequest(message) => {
            assert_eq!(message, "prompt must not be empty");
        }
        other => panic!("expected InvalidRequest, got {other:?}"),
    }
}

#[tokio::test]
async fn stream_json_nonzero_exit_uses_documented_exit_codes() {
    let client = make_fake_client("turn_limit_exceeded");
    let handle = client
        .stream_json(GeminiStreamJsonRunRequest::new("Reply with OK."))
        .await
        .unwrap();

    let events = handle
        .events
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|item| item.expect("event parses"))
        .collect::<Vec<_>>();
    assert!(matches!(
        events.last(),
        Some(GeminiStreamJsonEvent::Result { .. })
    ));

    let error = handle
        .completion
        .await
        .expect_err("nonzero exit should surface completion error");
    match error {
        GeminiCliError::RunFailed {
            exit_code, message, ..
        } => {
            assert_eq!(exit_code, Some(53));
            assert_eq!(message, "turn limit exceeded");
        }
        other => panic!("expected run failure, got {other:?}"),
    }
}

#[tokio::test]
async fn completion_waits_for_event_stream_finality() {
    let client = make_fake_client("three_events_delayed");
    let handle = client
        .stream_json(GeminiStreamJsonRunRequest::new("Reply with OK."))
        .await
        .unwrap();

    let mut events = handle.events;
    let mut completion = handle.completion;

    let _ = events
        .next()
        .await
        .expect("stream open")
        .expect("event parses");

    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    assert!(matches!(completion.as_mut().poll(&mut cx), Poll::Pending));

    while events.next().await.is_some() {}

    let completion = completion.await.expect("completion succeeds");
    assert!(completion.status.success());
}

#[allow(dead_code)]
fn _assert_exit_status_send_sync(_: ExitStatus) {}
