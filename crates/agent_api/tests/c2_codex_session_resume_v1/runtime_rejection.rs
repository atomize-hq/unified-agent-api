use agent_api::{AgentWrapperBackend, AgentWrapperError, AgentWrapperEventKind};
use serde_json::json;

use super::support::{
    add_dir_expectations, add_dirs_extension, add_dirs_fixture,
    assert_no_add_dir_sentinel_leaks_in_events, base_env, build_backend, drain_to_none,
    handle_facet_index, run_request, ADD_DIRS_RUNTIME_REJECTION_MESSAGE, ADD_DIR_LEAK_SENTINELS,
    STREAM_TIMEOUT,
};

async fn assert_runtime_rejection_case(
    scenario: &str,
    resume_extension: (String, serde_json::Value),
    prompt: &str,
    extra_env: impl IntoIterator<Item = (String, String)>,
) {
    let fixture = add_dirs_fixture();
    let mut env = base_env();
    env.insert("FAKE_CODEX_SCENARIO".to_string(), scenario.to_string());
    env.insert("FAKE_CODEX_EXPECT_PROMPT".to_string(), prompt.to_string());
    env.extend(add_dir_expectations(&fixture.dirs));
    env.extend(extra_env);

    let backend = build_backend(env, None, false);
    let handle = backend
        .run(run_request(
            prompt,
            [add_dirs_extension(&fixture.dirs), resume_extension],
        ))
        .await
        .unwrap();

    let mut events = handle.events;
    let seen = drain_to_none(events.as_mut(), STREAM_TIMEOUT).await;
    let handle_idx = handle_facet_index(&seen).expect("expected session handle facet");
    let error_indices: Vec<_> = seen
        .iter()
        .enumerate()
        .filter_map(|(idx, event)| (event.kind == AgentWrapperEventKind::Error).then_some(idx))
        .collect();
    assert_eq!(error_indices.len(), 1, "expected exactly one Error event");
    let error_idx = error_indices[0];
    assert!(
        handle_idx < error_idx,
        "expected thread.resumed-derived handle facet before backend error"
    );
    assert_eq!(
        seen.last().map(|event| event.kind.clone()),
        Some(AgentWrapperEventKind::Error),
        "expected Error event to be terminal"
    );
    assert_eq!(
        seen[error_idx].message.as_deref(),
        Some(ADD_DIRS_RUNTIME_REJECTION_MESSAGE)
    );
    assert_no_add_dir_sentinel_leaks_in_events(&seen);

    let err = tokio::time::timeout(STREAM_TIMEOUT, handle.completion)
        .await
        .expect("completion resolves")
        .unwrap_err();
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(message, ADD_DIRS_RUNTIME_REJECTION_MESSAGE);
            for sentinel in ADD_DIR_LEAK_SENTINELS {
                assert!(
                    !message.contains(sentinel),
                    "expected add-dir runtime rejection sentinel {sentinel} to stay out of completion error"
                );
            }
        }
        other => panic!("expected Backend error, got: {other:?}"),
    }
}

#[tokio::test]
async fn resume_last_add_dirs_runtime_rejection_emits_handle_before_backend_error() {
    assert_runtime_rejection_case(
        "add_dirs_runtime_rejection_resume_last",
        (
            "agent_api.session.resume.v1".to_string(),
            json!({"selector": "last"}),
        ),
        "hello world",
        std::iter::empty(),
    )
    .await;
}

#[tokio::test]
async fn resume_id_add_dirs_runtime_rejection_emits_handle_before_backend_error() {
    let resume_id = "thread-123";
    assert_runtime_rejection_case(
        "add_dirs_runtime_rejection_resume_id",
        (
            "agent_api.session.resume.v1".to_string(),
            json!({"selector": "id", "id": resume_id}),
        ),
        "hello world",
        [(
            "FAKE_CODEX_EXPECT_RESUME_ID".to_string(),
            resume_id.to_string(),
        )],
    )
    .await;
}
