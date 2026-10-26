#![cfg(feature = "codex")]
use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperBackend, AgentWrapperEventKind, AgentWrapperRunRequest,
};
use futures_util::StreamExt;
use serde_json::json;
use std::time::Duration;
use tempfile::tempdir;

async fn exercise(
    scenario: &str,
    last: bool,
) -> (
    Vec<agent_api::AgentWrapperEvent>,
    Result<agent_api::AgentWrapperCompletion, agent_api::AgentWrapperError>,
) {
    exercise_with_policy(scenario, last, None).await
}

async fn exercise_with_policy(
    scenario: &str,
    last: bool,
    policy: Option<Option<&str>>,
) -> (
    Vec<agent_api::AgentWrapperEvent>,
    Result<agent_api::AgentWrapperCompletion, agent_api::AgentWrapperError>,
) {
    let dir = tempdir().unwrap();
    let exit = dir.path().join("exit");
    let log = dir.path().join("methods");
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(env!("CARGO_BIN_EXE_fake_codex_app_server_jsonrpc_agent_api").into()),
        env: [
            (
                "FAKE_CODEX_APP_SERVER_EXPECT_POLICY".into(),
                policy
                    .flatten()
                    .unwrap_or(if policy.is_some() { "absent" } else { "never" })
                    .into(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_REQUEST_LOG".into(),
                log.display().to_string(),
            ),
            ("FAKE_CODEX_APP_SERVER_SCENARIO".into(), scenario.into()),
            (
                "FAKE_CODEX_APP_SERVER_EXPECT_CWD".into(),
                dir.path().display().to_string(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_EXIT_MARKER".into(),
                exit.display().to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });
    let mut extensions = std::collections::BTreeMap::from([(
        "agent_api.session.fork.v1".into(),
        if last {
            json!({"selector":"last"})
        } else {
            json!({"selector":"id","id":"source"})
        },
    )]);
    if let Some(policy) = policy {
        extensions.insert("agent_api.exec.non_interactive".into(), json!(false));
        if let Some(policy) = policy {
            extensions.insert("backend.codex.exec.approval_policy".into(), json!(policy));
        }
    }
    let run = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".into(),
            working_dir: Some(dir.path().into()),
            timeout: Some(Duration::from_millis(800)),
            extensions,
            ..Default::default()
        })
        .await
        .unwrap();
    let (events, completion) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(run.events.collect::<Vec<_>>(), run.completion)
    })
    .await
    .expect("bounded protocol lifecycle");
    assert!(
        exit.exists(),
        "{scenario}: completion must follow actual child exit"
    );
    let methods = std::fs::read_to_string(log).unwrap();
    assert!(!methods.contains("$/cancelRequest"));
    if matches!(
        scenario,
        "pinned_block" | "pinned_approval" | "pinned_failed"
    ) {
        assert!(
            methods.lines().any(|method| method == "turn/interrupt"),
            "{scenario}: {methods}"
        );
    }
    (events, completion)
}

#[tokio::test]
async fn pinned_handshake_ready_then_complete_and_reap() {
    let (events, completion) = exercise("pinned_handshake", false).await;
    assert!(completion.unwrap().status.success());
    assert!(events
        .iter()
        .any(|e| e.text.as_deref() == Some("final output")));
}

#[tokio::test]
async fn pinned_acceptance_is_not_completion_and_final_output_is_drained() {
    let (events, completion) = exercise("pinned_success", false).await;
    assert!(completion.unwrap().status.success());
    assert!(
        events
            .iter()
            .any(|e| e.text.as_deref() == Some("final output")),
        "lost final output: {events:?}"
    );
    assert!(events.iter().any(|e| e.text.as_deref() == Some("reason")));
    assert!(!events.iter().any(|e| e.text.as_deref() == Some("FOREIGN")));
}

#[tokio::test]
async fn pinned_last_selects_exec_sources_with_all_pages() {
    let (_, completion) = exercise("pinned_last", true).await;
    assert!(completion.unwrap().status.success());
}

#[tokio::test]
async fn pinned_server_approval_fails_safely_without_hanging() {
    let (events, completion) = exercise("pinned_approval", false).await;
    assert_eq!(
        completion.unwrap_err().to_string(),
        "backend error: approval required"
    );
    assert_eq!(
        events.last().unwrap().message.as_deref(),
        Some("approval required")
    );
    assert_eq!(
        events
            .iter()
            .filter(|e| e.kind == AgentWrapperEventKind::Error)
            .count(),
        1
    );
    assert!(!format!("{events:?}").contains("SECRET"));
}

#[tokio::test]
async fn pinned_adverse_terminal_paths_never_succeed() {
    for scenario in [
        "pinned_failed",
        "pinned_disconnect",
        "pinned_malformed_acceptance",
        "pinned_block",
        "pinned_empty",
        "pinned_cursor",
        "pinned_fork_approval",
        "pinned_wrong_response",
        "pinned_malformed_terminal",
        "pinned_error",
    ] {
        let (_, completion) = exercise(
            scenario,
            matches!(scenario, "pinned_empty" | "pinned_cursor"),
        )
        .await;
        assert!(completion.is_err(), "{scenario} incorrectly succeeded");
    }
}

#[tokio::test]
async fn pinned_explicit_cancel_interrupts_the_accepted_turn_and_waits_for_exit() {
    use agent_api::{AgentWrapperGateway, AgentWrapperKind};
    let dir = tempdir().unwrap();
    let exit = dir.path().join("exit");
    let log = dir.path().join("methods");
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(env!("CARGO_BIN_EXE_fake_codex_app_server_jsonrpc_agent_api").into()),
        env: [
            (
                "FAKE_CODEX_APP_SERVER_SCENARIO".into(),
                "pinned_block".into(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_EXIT_MARKER".into(),
                exit.display().to_string(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_REQUEST_LOG".into(),
                log.display().to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });
    let mut gateway = AgentWrapperGateway::new();
    gateway.register(std::sync::Arc::new(backend)).unwrap();
    let run = gateway
        .run_control(
            &AgentWrapperKind::new("codex").unwrap(),
            AgentWrapperRunRequest {
                prompt: "hello".into(),
                extensions: [(
                    "agent_api.session.fork.v1".into(),
                    json!({"selector":"id","id":"source"}),
                )]
                .into_iter()
                .collect(),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let mut events = run.handle.events;
    assert!(tokio::time::timeout(Duration::from_secs(3), events.next())
        .await
        .unwrap()
        .is_some());
    run.cancel.cancel();
    run.cancel.cancel();
    let (_, completion) = tokio::time::timeout(Duration::from_secs(3), async {
        tokio::join!(events.collect::<Vec<_>>(), run.handle.completion)
    })
    .await
    .unwrap();
    assert_eq!(
        completion.unwrap_err().to_string(),
        "backend error: cancelled"
    );
    assert!(exit.exists());
    let methods = std::fs::read_to_string(log).unwrap();
    assert_eq!(
        methods.lines().filter(|m| *m == "turn/interrupt").count(),
        1
    );
    assert!(!methods.contains("$/cancelRequest"));
}

#[tokio::test]
async fn pinned_fork_forwards_supported_policies_and_preserves_absence() {
    for policy in [None, Some("never"), Some("untrusted"), Some("on-request")] {
        let (_, completion) = exercise_with_policy("pinned_policy", false, Some(policy)).await;
        assert!(completion.unwrap().status.success(), "{policy:?}");
    }
    let (events, completion) =
        exercise_with_policy("pinned_policy", false, Some(Some("on-failure"))).await;
    assert_eq!(
        completion.unwrap_err().to_string(),
        "backend error: codex app-server rpc error"
    );
    assert!(!format!("{events:?}").contains("SECRET"));
}

async fn wrapper_paths(scenario: &str) {
    use codex::mcp::{ClientInfo, CodexAppServer, McpError, StdioServerConfig, ThreadForkParams};
    let dir = tempdir().unwrap();
    let server = CodexAppServer::start_experimental(
        StdioServerConfig {
            binary: env!("CARGO_BIN_EXE_fake_codex_app_server_jsonrpc_agent_api").into(),
            code_home: None,
            current_dir: Some(dir.path().into()),
            env: vec![
                ("FAKE_CODEX_APP_SERVER_SCENARIO".into(), scenario.into()),
                (
                    "FAKE_CODEX_APP_SERVER_EXPECT_CWD".into(),
                    dir.path().as_os_str().to_owned(),
                ),
            ],
            app_server_analytics_default_enabled: false,
            mirror_stdio: false,
            startup_timeout: Duration::from_secs(2),
        },
        ClientInfo {
            name: "compatibility-test".into(),
            version: "0".into(),
        },
    )
    .await
    .unwrap();
    if scenario == "pinned_last" {
        let result = server.select_last_thread_id(dir.path().into()).await;
        server.shutdown().await.unwrap();
        assert_eq!(
            result.expect("explicit sourceKinds must include exec sessions"),
            Some("z".into())
        );
    } else {
        let result = tokio::time::timeout(
            Duration::from_millis(800),
            server.thread_fork(ThreadForkParams {
                thread_id: "source".into(),
                cwd: None,
                approval_policy: Some("never".into()),
                sandbox: None,
                persist_extended_history: None,
            }),
        )
        .await;
        server.shutdown().await.unwrap();
        assert!(
            matches!(result, Ok(Err(McpError::Server(ref message))) if message == "approval required"),
            "server approval request was not surfaced: {result:?}"
        );
    }
}

#[tokio::test]
async fn pinned_wrapper_last_filter_covers_exec_sources() {
    wrapper_paths("pinned_last").await;
}

#[tokio::test]
async fn pinned_wrapper_server_approval_request_is_not_a_client_response() {
    wrapper_paths("pinned_fork_approval").await;
}

// DEF-03: retry semantics must survive without exposing backend diagnostics.
#[tokio::test]
async fn retrying_errors_do_not_leak_raw_diagnostics() {
    let (events, completion) = exercise("pinned_retry_secret", false).await;
    assert!(completion.unwrap().status.success());
    assert!(!format!("{events:?}").contains("CREDENTIAL-SECRET"));
    assert!(events
        .iter()
        .any(|event| event.kind == AgentWrapperEventKind::Status
            && event.message.as_deref()
                == Some("codex app-server retrying after error (details redacted)")));
    assert!(!events
        .iter()
        .any(|event| event.kind == AgentWrapperEventKind::Error));
    assert!(events
        .iter()
        .any(|event| event.text.as_deref() == Some("final output")));
}

// DEF-04 / AS-1: terminal identity belongs to turn.id, never an extra turnId field.
#[tokio::test]
async fn terminal_nested_identity_cannot_be_missing_blank_or_shadowed() {
    for scenario in [
        "pinned_terminal_absent",
        "pinned_terminal_blank",
        "pinned_terminal_conflict",
    ] {
        let (_, completion) = exercise(scenario, false).await;
        assert!(completion.is_err(), "{scenario} supplied false success");
    }
}
