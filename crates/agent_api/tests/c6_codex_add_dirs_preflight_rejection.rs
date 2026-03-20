#![cfg(feature = "codex")]

#[allow(dead_code, unused_imports)]
#[path = "c2_codex_session_resume_v1/support.rs"]
mod support;

use agent_api::{AgentWrapperBackend, AgentWrapperError};

use support::{
    add_dirs_extension, add_dirs_fixture, base_env, build_probe_only_backend, run_request,
    AddDirProbeMode, ADD_DIRS_RUNTIME_REJECTION_MESSAGE,
};

#[cfg(unix)]
#[tokio::test]
async fn exec_add_dirs_preflight_rejection_fails_before_returning_handle() {
    let fixture = add_dirs_fixture();
    let fixture_backend =
        build_probe_only_backend(AddDirProbeMode::Unsupported, base_env(), None, false);

    let err = fixture_backend
        .backend
        .run(run_request(
            "hello world",
            [add_dirs_extension(&fixture.dirs)],
        ))
        .await
        .expect_err("preflight rejection should fail backend.run directly");

    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(message, ADD_DIRS_RUNTIME_REJECTION_MESSAGE);
        }
        other => panic!("expected Backend error, got: {other:?}"),
    }
    assert!(
        !fixture_backend.exec_log.exists(),
        "preflight rejection should not invoke codex exec"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn exec_add_dirs_preflight_rejection_beats_external_sandbox_startup_stream() {
    let fixture = add_dirs_fixture();
    let fixture_backend =
        build_probe_only_backend(AddDirProbeMode::Unsupported, base_env(), None, true);

    let err = fixture_backend
        .backend
        .run(run_request(
            "hello world",
            [
                add_dirs_extension(&fixture.dirs),
                (
                    "agent_api.exec.external_sandbox.v1".to_string(),
                    serde_json::json!(true),
                ),
            ],
        ))
        .await
        .expect_err("preflight rejection should fail before any synthetic startup stream");

    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(message, ADD_DIRS_RUNTIME_REJECTION_MESSAGE);
        }
        other => panic!("expected Backend error, got: {other:?}"),
    }
    assert!(
        !fixture_backend.exec_log.exists(),
        "preflight rejection should not invoke codex exec"
    );
}
