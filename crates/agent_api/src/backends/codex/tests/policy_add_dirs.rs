use std::fs;

use serde_json::json;
use tempfile::tempdir;

use super::support::*;

const EXT_ADD_DIRS_V1: &str = "agent_api.exec.add_dirs.v1";

#[test]
fn codex_policy_add_dirs_absent_key_returns_empty_vec() {
    let adapter = test_adapter();
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };

    let policy = adapter
        .validate_and_extract_policy(&request)
        .expect("policy extraction should succeed");

    assert!(policy.add_dirs.is_empty());
}

#[test]
fn codex_policy_add_dirs_request_working_dir_beats_default_and_run_start_cwd() {
    let temp = tempdir().expect("tempdir");
    let request_root = temp.path().join("request_root");
    let default_root = temp.path().join("default_root");
    let run_start_root = temp.path().join("run_start_root");
    let request_target = request_root.join("rel-target");
    let default_target = default_root.join("rel-target");
    let run_start_target = run_start_root.join("rel-target");
    fs::create_dir_all(&request_target).expect("create request target");
    fs::create_dir_all(&default_target).expect("create default target");
    fs::create_dir_all(&run_start_target).expect("create run-start target");

    let adapter = test_adapter_with_config_and_run_start_cwd(
        CodexBackendConfig {
            default_working_dir: Some(default_root),
            ..Default::default()
        },
        Some(run_start_root),
    );
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        working_dir: Some(request_root),
        extensions: [(EXT_ADD_DIRS_V1.to_string(), add_dirs_payload(&["rel-target"]))]
            .into_iter()
            .collect(),
        ..Default::default()
    };

    let policy = adapter
        .validate_and_extract_policy(&request)
        .expect("policy extraction should succeed");

    assert_eq!(policy.add_dirs, vec![request_target]);
}

#[test]
fn codex_policy_add_dirs_default_working_dir_beats_run_start_cwd() {
    let temp = tempdir().expect("tempdir");
    let default_root = temp.path().join("default_root");
    let run_start_root = temp.path().join("run_start_root");
    let default_target = default_root.join("rel-target");
    let run_start_target = run_start_root.join("rel-target");
    fs::create_dir_all(&default_target).expect("create default target");
    fs::create_dir_all(&run_start_target).expect("create run-start target");

    let adapter = test_adapter_with_config_and_run_start_cwd(
        CodexBackendConfig {
            default_working_dir: Some(default_root),
            ..Default::default()
        },
        Some(run_start_root),
    );
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        extensions: [(EXT_ADD_DIRS_V1.to_string(), add_dirs_payload(&["rel-target"]))]
            .into_iter()
            .collect(),
        ..Default::default()
    };

    let policy = adapter
        .validate_and_extract_policy(&request)
        .expect("policy extraction should succeed");

    assert_eq!(policy.add_dirs, vec![default_target]);
}

#[test]
fn codex_policy_add_dirs_invalid_input_beats_fork_handling_and_stays_redacted() {
    let temp = tempdir().expect("tempdir");
    let request_root = temp.path().join("request_root");
    fs::create_dir_all(&request_root).expect("create request root");
    let leaked = "missing-secret-dir";

    let adapter = test_adapter_with_run_start_cwd(Some(temp.path().join("run_start_root")));
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        working_dir: Some(request_root),
        extensions: [
            (EXT_ADD_DIRS_V1.to_string(), add_dirs_payload(&[leaked])),
            (
                EXT_SESSION_FORK_V1.to_string(),
                json!({
                    "selector": "last",
                }),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    };

    let err = adapter_error(adapter.validate_and_extract_policy(&request));
    match &err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(message, "invalid agent_api.exec.add_dirs.v1.dirs[0]");
        }
        other => panic!("expected InvalidRequest, got: {other:?}"),
    }
    assert!(
        !err.to_string().contains(leaked),
        "error display leaked raw path text"
    );
}

fn adapter_error(
    result: Result<super::super::CodexExecPolicy, AgentWrapperError>,
) -> AgentWrapperError {
    result.expect_err("policy extraction should fail")
}
