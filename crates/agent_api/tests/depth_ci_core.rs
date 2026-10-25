//! Reuse production adapter assertions; all subprocesses below are explicitly synthetic.
#![cfg(all(feature = "codex", target_os = "linux"))]
#[path = "c1_codex_stream_exec_adapter.rs"]
mod adapter;
#[path = "c3_explicit_cancellation.rs"]
mod cancel;
#[path = "c2_codex_stream_exec_parity.rs"]
mod parity;
#[path = "c1_codex_exec_policy.rs"]
mod policy;

use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperBackend, AgentWrapperEventKind, AgentWrapperRunRequest,
};
use std::{path::PathBuf, time::Duration};

#[test]
fn pinned_upstream_grammar_and_applicability() {
    let snapshot: serde_json::Value = serde_json::from_str(include_str!(
        "../../../cli_manifests/codex/snapshots/0.156.1/x86_64-unknown-linux-musl.json"
    ))
    .unwrap();
    let binary = &snapshot["binary"];
    assert_eq!(binary["semantic_version"], "0.156.1");
    assert_eq!(binary["target_triple"], "x86_64-unknown-linux-musl");
    assert_eq!(
        binary["sha256"],
        "0b2e9301d6100dddda3b9d5c80ebaeaa3a2f1962388f2f36f6b96a9f08b1f33f"
    );
    let exec = snapshot["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["path"] == serde_json::json!(["exec"]))
        .unwrap();
    assert!(exec["flags"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["long"] == "--json"));
    assert_eq!(std::env::consts::ARCH, "x86_64");
}

async fn fixture(
    large: bool,
    cwd: Option<&std::path::Path>,
) -> (
    Vec<agent_api::AgentWrapperEvent>,
    agent_api::AgentWrapperCompletion,
) {
    let mut config = CodexBackendConfig {
        binary: Some(fixture_binary("fake_codex_depth_core")),
        ..Default::default()
    };
    if large {
        config.env.insert("DEPTH_CORE_LARGE".into(), "1".into());
    }
    if let Some(cwd) = cwd {
        config
            .env
            .insert("DEPTH_CORE_EXPECT_CWD".into(), cwd.to_str().unwrap().into());
        config.env.insert(
            "DEPTH_CORE_CREDENTIAL".into(),
            "SYNTHETIC_DEPTH_CREDENTIAL".into(),
        );
    }
    let handle = CodexBackend::new(config)
        .run(AgentWrapperRunRequest {
            prompt: "depth-core-prompt".into(),
            working_dir: cwd.map(Into::into),
            ..Default::default()
        })
        .await
        .unwrap();
    let mut events = handle.events;
    tokio::time::timeout(Duration::from_secs(5), async move {
        let collect = async {
            let mut seen = vec![];
            while let Some(e) = std::future::poll_fn(|cx| events.as_mut().poll_next(cx)).await {
                seen.push(e);
            }
            seen
        };
        let (seen, result) = tokio::join!(collect, handle.completion);
        let result = result.unwrap();
        assert!(result.status.success());
        (seen, result)
    })
    .await
    .unwrap()
}
#[tokio::test]
async fn credential_sentinel_is_not_projected() {
    let cwd = tempfile::tempdir().unwrap();
    let parent = std::env::var_os("DEPTH_CORE_CREDENTIAL");
    let (seen, result) = fixture(false, Some(cwd.path())).await;
    assert!(!seen.is_empty());
    for e in seen {
        assert!(!format!("{e:?}").contains("SYNTHETIC_DEPTH_CREDENTIAL"));
    }
    assert!(!format!("{result:?}").contains("SYNTHETIC_DEPTH_CREDENTIAL"));
    assert_eq!(std::env::var_os("DEPTH_CORE_CREDENTIAL"), parent);
    assert_eq!(std::fs::read_dir(cwd.path()).unwrap().count(), 0);
}
#[tokio::test]
async fn projected_text_is_bounded() {
    let (seen, result) = fixture(true, None).await;
    assert!(seen
        .iter()
        .any(|e| e.kind == AgentWrapperEventKind::TextOutput));
    for e in seen {
        assert!(e.text.as_ref().map_or(true, |s| s.len() <= 65_536));
        assert!(e.message.as_ref().map_or(true, |s| s.len() <= 4_096));
        assert!(e
            .data
            .as_ref()
            .map_or(true, |d| serde_json::to_vec(d).unwrap().len() <= 65_536));
    }
    assert!(result
        .final_text
        .as_ref()
        .is_some_and(|s| s.len() <= 65_536));
}
#[tokio::test]
async fn stdin_prompt_and_unknown_event_are_handled() {
    let (seen, result) = fixture(false, None).await;
    assert_eq!(result.final_text.as_deref(), Some("depth-result"));
    let text = seen
        .iter()
        .position(|e| e.kind == AgentWrapperEventKind::TextOutput)
        .unwrap();
    assert!(seen[..text]
        .iter()
        .any(|e| e.kind == AgentWrapperEventKind::Status));
    assert!(seen
        .iter()
        .any(|e| format!("{e:?}").contains("depth-core-thread")));
}
#[tokio::test]
async fn spawn_and_timeout_are_distinct() {
    let missing = tempfile::tempdir().unwrap();
    let handle = CodexBackend::new(CodexBackendConfig {
        binary: Some(missing.path().join("absent")),
        ..Default::default()
    })
    .run(AgentWrapperRunRequest {
        prompt: "hello".into(),
        ..Default::default()
    })
    .await
    .unwrap();
    let mut events = handle.events;
    let drain = async {
        while std::future::poll_fn(|cx| events.as_mut().poll_next(cx))
            .await
            .is_some()
        {}
    };
    let (_, spawn) = tokio::join!(drain, handle.completion);
    let spawn = format!("{}", spawn.unwrap_err());
    assert!(spawn.contains("spawn"));
    let handle = CodexBackend::new(CodexBackendConfig {
        binary: Some(fixture_binary("fake_codex_stream_exec_scenarios_agent_api")),
        env: [("FAKE_CODEX_SCENARIO".into(), "block_until_killed".into())]
            .into_iter()
            .collect(),
        ..Default::default()
    })
    .run(AgentWrapperRunRequest {
        prompt: "hello".into(),
        timeout: Some(Duration::from_millis(200)),
        ..Default::default()
    })
    .await
    .unwrap();
    let mut events = handle.events;
    let drain = async {
        while std::future::poll_fn(|cx| events.as_mut().poll_next(cx))
            .await
            .is_some()
        {}
    };
    let (_, timeout) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(drain, handle.completion)
    })
    .await
    .unwrap();
    let timeout = format!("{}", timeout.unwrap_err());
    assert!(timeout.contains("timeout"));
    assert_ne!(spawn, timeout);
}

fn fixture_binary(name: &str) -> PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
}
