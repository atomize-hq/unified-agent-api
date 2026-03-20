use std::{collections::BTreeMap, fs, path::PathBuf, time::Duration};

use tempfile::tempdir;

use super::support::*;

fn add_dirs_json(dirs: &[PathBuf]) -> JsonValue {
    add_dirs_payload(
        &dirs
            .iter()
            .map(|dir| dir.display().to_string())
            .collect::<Vec<_>>(),
    )
}

async fn run_claude_assertion(
    prompt: &str,
    scenario: &str,
    mut env: BTreeMap<String, String>,
    extensions: BTreeMap<String, JsonValue>,
) {
    env.insert("FAKE_CLAUDE_SCENARIO".to_string(), scenario.to_string());
    env.insert("FAKE_CLAUDE_EXPECT_PROMPT".to_string(), prompt.to_string());

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env,
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            extensions,
            ..Default::default()
        })
        .await
        .expect("Claude run should start");

    let mut events = handle.events;
    let _seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let completion = tokio::time::timeout(Duration::from_secs(2), handle.completion)
        .await
        .expect("completion resolves")
        .expect("run completes successfully");
    assert!(
        completion.status.success(),
        "expected successful fake Claude run"
    );
}

#[tokio::test]
async fn fresh_run_emits_one_variadic_add_dir_group_before_verbose() {
    let prompt = "hello world";
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    fs::create_dir_all(&dir_a).expect("alpha dir");
    fs::create_dir_all(&dir_b).expect("beta dir");
    let add_dirs = vec![dir_a, dir_b];

    let env = expected_add_dirs_env(&add_dirs);
    let extensions = [(
        "agent_api.exec.add_dirs.v1".to_string(),
        add_dirs_json(&add_dirs),
    )]
    .into_iter()
    .collect();

    run_claude_assertion(prompt, "fresh_assert", env, extensions).await;
}

#[tokio::test]
async fn fresh_run_omits_add_dir_when_extension_key_is_absent() {
    run_claude_assertion(
        "hello world",
        "fresh_assert",
        expect_no_add_dir_env(),
        BTreeMap::new(),
    )
    .await;
}

#[tokio::test]
async fn resume_last_keeps_add_dir_group_before_continue() {
    let prompt = "hello world";
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    fs::create_dir_all(&dir_a).expect("alpha dir");
    fs::create_dir_all(&dir_b).expect("beta dir");
    let add_dirs = vec![dir_a, dir_b];

    let env = expected_add_dirs_env(&add_dirs);
    let extensions = [
        (
            "agent_api.exec.add_dirs.v1".to_string(),
            add_dirs_json(&add_dirs),
        ),
        (
            "agent_api.session.resume.v1".to_string(),
            serde_json::json!({"selector": "last"}),
        ),
    ]
    .into_iter()
    .collect();

    run_claude_assertion(prompt, "resume_last_assert", env, extensions).await;
}

#[tokio::test]
async fn resume_id_keeps_add_dir_group_before_resume_flag() {
    let prompt = "hello world";
    let resume_id = "sess-123";
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    fs::create_dir_all(&dir_a).expect("alpha dir");
    fs::create_dir_all(&dir_b).expect("beta dir");
    let add_dirs = vec![dir_a, dir_b];

    let mut env = expected_add_dirs_env(&add_dirs);
    env.insert(
        "FAKE_CLAUDE_EXPECT_RESUME_ID".to_string(),
        resume_id.to_string(),
    );
    let extensions = [
        (
            "agent_api.exec.add_dirs.v1".to_string(),
            add_dirs_json(&add_dirs),
        ),
        (
            "agent_api.session.resume.v1".to_string(),
            serde_json::json!({"selector": "id", "id": resume_id}),
        ),
    ]
    .into_iter()
    .collect();

    run_claude_assertion(prompt, "resume_id_assert", env, extensions).await;
}

#[tokio::test]
async fn fork_last_keeps_add_dir_group_before_continue_and_fork_session() {
    let prompt = "hello world";
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    fs::create_dir_all(&dir_a).expect("alpha dir");
    fs::create_dir_all(&dir_b).expect("beta dir");
    let add_dirs = vec![dir_a, dir_b];

    let env = expected_add_dirs_env(&add_dirs);
    let extensions = [
        (
            "agent_api.exec.add_dirs.v1".to_string(),
            add_dirs_json(&add_dirs),
        ),
        (
            "agent_api.session.fork.v1".to_string(),
            serde_json::json!({"selector": "last"}),
        ),
    ]
    .into_iter()
    .collect();

    run_claude_assertion(prompt, "fork_last_assert", env, extensions).await;
}

#[tokio::test]
async fn fork_id_keeps_add_dir_group_before_fork_session_and_resume() {
    let prompt = "hello world";
    let fork_id = "sess-123";
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    fs::create_dir_all(&dir_a).expect("alpha dir");
    fs::create_dir_all(&dir_b).expect("beta dir");
    let add_dirs = vec![dir_a, dir_b];

    let mut env = expected_add_dirs_env(&add_dirs);
    env.insert(
        "FAKE_CLAUDE_EXPECT_RESUME_ID".to_string(),
        fork_id.to_string(),
    );
    let extensions = [
        (
            "agent_api.exec.add_dirs.v1".to_string(),
            add_dirs_json(&add_dirs),
        ),
        (
            "agent_api.session.fork.v1".to_string(),
            serde_json::json!({"selector": "id", "id": fork_id}),
        ),
    ]
    .into_iter()
    .collect();

    run_claude_assertion(prompt, "fork_id_assert", env, extensions).await;
}
