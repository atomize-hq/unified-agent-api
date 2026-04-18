use std::{
    collections::BTreeMap,
    task::{Context, Poll},
};

use futures_util::{task::noop_waker, StreamExt};

use crate::{AgentWrapperBackend, AgentWrapperEventKind};

use super::support::{backend_with_env, capture_json, request};

#[tokio::test]
async fn opencode_backend_maps_request_into_canonical_argv_and_working_dir() {
    let capture_dir = tempfile::tempdir().expect("create capture dir");
    let capture_path = capture_dir.path().join("capture.json");
    let working_dir = tempfile::tempdir().expect("create working dir");
    let mut env = BTreeMap::new();
    env.insert(
        "FAKE_OPENCODE_SCENARIO".to_string(),
        "capture_args".to_string(),
    );
    env.insert(
        "FAKE_OPENCODE_CAPTURE".to_string(),
        capture_path.display().to_string(),
    );

    let backend = backend_with_env(env);
    let handle = backend
        .run(request("Reply with OK.", Some(working_dir.path())))
        .await
        .expect("run should start");

    let mut events = handle.events;
    while events.next().await.is_some() {}

    let completion = handle.completion.await.expect("completion succeeds");
    assert!(completion.status.success());
    assert_eq!(completion.final_text.as_deref(), Some("OK"));

    let capture = capture_json(&capture_path);
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
            "--dir",
            working_dir.path().to_str().expect("working dir path"),
            "Reply with OK.",
        ]
    );
}

#[tokio::test]
async fn opencode_backend_redacts_parse_errors_in_public_events() {
    let mut env = BTreeMap::new();
    env.insert(
        "FAKE_OPENCODE_SCENARIO".to_string(),
        "parse_error_redaction".to_string(),
    );

    let backend = backend_with_env(env);
    let handle = backend
        .run(request("Reply with OK.", None))
        .await
        .expect("run should start");

    let mut saw_error = false;
    let mut events = handle.events;
    while let Some(event) = events.next().await {
        if event.kind == AgentWrapperEventKind::Error {
            let message = event.message.as_deref().unwrap_or("");
            assert!(!message.contains("VERY_SECRET_SHOULD_NOT_APPEAR"));
            saw_error = true;
        }
    }

    assert!(saw_error, "expected a surfaced parse error event");

    let completion = handle.completion.await.expect("completion succeeds");
    assert!(completion.status.success());
    assert_eq!(completion.final_text.as_deref(), Some("OK"));
}

#[tokio::test]
async fn opencode_backend_completion_waits_for_stream_finality_via_shared_harness() {
    let mut env = BTreeMap::new();
    env.insert(
        "FAKE_OPENCODE_SCENARIO".to_string(),
        "three_events_delayed".to_string(),
    );

    let backend = backend_with_env(env);
    let handle = backend
        .run(request("Reply with OK.", None))
        .await
        .expect("run should start");

    let mut events = handle.events;
    let mut completion = handle.completion;

    let first = tokio::select! {
        biased;
        result = completion.as_mut() => panic!("completion resolved before first event: {result:?}"),
        item = events.next() => item.expect("event stream open"),
    };

    assert_eq!(first.kind, AgentWrapperEventKind::Status);

    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    assert!(matches!(completion.as_mut().poll(&mut cx), Poll::Pending));

    while events.next().await.is_some() {}

    let completion = completion.await.expect("completion succeeds");
    assert!(completion.status.success());
    assert_eq!(completion.final_text.as_deref(), Some("OK"));
}
