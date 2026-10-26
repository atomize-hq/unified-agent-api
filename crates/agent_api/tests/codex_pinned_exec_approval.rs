#![cfg(feature = "codex")]
//! Source-bound fixture checks; actual pinned binary policy observations are recorded separately.
use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperBackend, AgentWrapperRunRequest,
};
use futures_util::StreamExt;
use serde_json::json;
use std::{collections::BTreeMap, time::Duration};

#[tokio::test]
async fn never_intent_overrides_auto_review_config_for_initial_and_resume() {
    for selector in [
        None,
        Some(json!({"selector":"last"})),
        Some(json!({"selector":"id","id":"thread-1"})),
    ] {
        for mode in ["default", "never", "absent", "external"] {
            let backend = CodexBackend::new(CodexBackendConfig {
                binary: Some(
                    if matches!(mode, "never" | "absent") {
                        env!("CARGO_BIN_EXE_fake_codex_native_bridge")
                    } else {
                        env!("CARGO_BIN_EXE_fake_codex_stream_exec_scenarios_agent_api")
                    }
                    .into(),
                ),
                allow_external_sandbox_exec: true,
                env: [
                    (
                        "NATIVE_POLICY".into(),
                        if mode == "absent" {
                            "null"
                        } else {
                            "\"never\""
                        }
                        .into(),
                    ),
                    (
                        "NATIVE_SOURCE".into(),
                        if selector
                            .as_ref()
                            .and_then(|value| value.get("id"))
                            .is_some()
                        {
                            "thread-1"
                        } else {
                            "source"
                        }
                        .into(),
                    ),
                    ("FAKE_CODEX_AUTOREVIEW_APPROVAL".into(), "on-request".into()),
                    (
                        "FAKE_CODEX_EXPECT_EFFECTIVE_APPROVAL".into(),
                        if mode == "absent" {
                            "on-request"
                        } else {
                            "never"
                        }
                        .into(),
                    ),
                ]
                .into_iter()
                .collect(),
                ..Default::default()
            });
            let mut extensions = BTreeMap::new();
            if let Some(selector) = selector.clone() {
                extensions.insert("agent_api.session.resume.v1".into(), selector);
            }
            if mode == "never" {
                extensions.insert("agent_api.exec.non_interactive".into(), json!(false));
                extensions.insert("backend.codex.exec.approval_policy".into(), json!("never"));
            }
            if mode == "absent" {
                extensions.insert("agent_api.exec.non_interactive".into(), json!(false));
            }
            if mode == "external" {
                extensions.insert("agent_api.exec.external_sandbox.v1".into(), json!(true));
            }
            let run = backend
                .run(AgentWrapperRunRequest {
                    prompt: "hello".into(),
                    extensions,
                    ..Default::default()
                })
                .await
                .unwrap();
            let (_, completion) = tokio::time::timeout(Duration::from_secs(3), async {
                tokio::join!(run.events.collect::<Vec<_>>(), run.completion)
            })
            .await
            .unwrap();
            assert!(
                completion.unwrap().status.success(),
                "selector={selector:?} mode={mode}"
            );
        }
    }
}
