#![cfg(feature = "codex")]
use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperBackend, AgentWrapperRunRequest,
};
use futures_util::StreamExt;
use serde_json::json;
use std::time::Duration;
async fn route(selector: Option<&str>) {
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(env!("CARGO_BIN_EXE_fake_codex_native_bridge").into()),
        ..Default::default()
    });
    let mut request = AgentWrapperRunRequest {
        prompt: "hello".into(),
        timeout: Some(Duration::from_secs(5)),
        ..Default::default()
    };
    request
        .extensions
        .insert("agent_api.exec.non_interactive".into(), json!(false));
    request.extensions.insert(
        "backend.codex.exec.approval_policy".into(),
        json!("on-request"),
    );
    if let Some(selector) = selector {
        request
            .extensions
            .insert(selector.into(), json!({"selector":"id","id":"source"}));
    }
    let run = backend.run(request).await.unwrap();
    let events = tokio::spawn(run.events.collect::<Vec<_>>());
    let completion = run.completion.await.unwrap();
    assert!(completion.status.success());
    assert_eq!(completion.data.unwrap()["session"]["id"], "owned");
    let events = events.await.unwrap();
    assert!(events
        .iter()
        .any(|event| event.text.as_deref() == Some("native final")));
}
#[tokio::test]
async fn interactive_initial_uses_native_route() {
    route(None).await;
}
#[tokio::test]
async fn interactive_resume_uses_native_route() {
    route(Some("agent_api.session.resume.v1")).await;
}

use agent_api::backends::codex::{CodexInteractionError, CodexInteractionReply};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
fn request() -> AgentWrapperRunRequest {
    AgentWrapperRunRequest {
        prompt: "hello".into(),
        timeout: Some(Duration::from_secs(5)),
        extensions: [
            ("agent_api.exec.non_interactive".into(), json!(false)),
            (
                "backend.codex.exec.approval_policy".into(),
                json!("on-request"),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    }
}
fn backend(scenario: &str) -> CodexBackend {
    CodexBackend::new(CodexBackendConfig {
        binary: Some(env!("CARGO_BIN_EXE_fake_codex_native_bridge").into()),
        env: [("NATIVE_SCENARIO".into(), scenario.into())]
            .into_iter()
            .collect(),
        ..Default::default()
    })
}
#[tokio::test]
async fn file_context_complete_original_and_ordered_for_two_typed_ids() {
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let backend=backend("file_two").with_interaction_handler(move |request| {
        let count=count.clone();
        async move {
            assert_eq!(request.method,"item/fileChange/requestApproval");
            let id=request.params["itemId"].as_str().unwrap();
            assert_eq!(request.params,json!({"threadId":"owned","turnId":"turn","itemId":id,"startedAtMs":0}));
            let context=request.item_context.unwrap();
            assert_eq!(context["id"],id);
            assert_eq!(context["changes"],json!([{"path":"秘密/é.txt","kind":{"type":"update","move_path":"新.txt"},"diff":"@@ -1 +1 @@\n-old\n+新\n"},{"path":"second","kind":{"type":"add"},"diff":"+line\n"}]));
            assert_eq!(context["changes"][0]["diff"].as_str().unwrap().as_bytes(),"@@ -1 +1 @@\n-old\n+新\n".as_bytes());
            count.fetch_add(1,Ordering::SeqCst);
            Ok(CodexInteractionReply::Result(json!({"decision":"acceptForSession"})))
        }
    });
    let run = backend.run(request()).await.unwrap();
    let events = tokio::spawn(run.events.collect::<Vec<_>>());
    assert!(run.completion.await.unwrap().status.success());
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    let rendered = format!("{:?}", events.await.unwrap());
    assert!(!rendered.contains("秘密"));
    assert!(!rendered.contains("+新"));
}
#[tokio::test]
async fn standalone_mcp_absent_null_and_early_reply_before_turn_ack() {
    for scenario in [
        "mcp_absent",
        "mcp_null",
        "mcp_early_absent",
        "mcp_early_null",
        "mcp_scoped",
    ] {
        let scenario = scenario.to_string();
        let expected = scenario.clone();
        let backend = backend(&scenario).with_interaction_handler(move |request| {
            let expected = expected.clone();
            async move {
                assert_eq!(request.method, "mcpServer/elicitation/request");
                assert!(request.item_context.is_none());
                if expected.contains("null") {
                    assert_eq!(request.params.get("turnId"), Some(&serde_json::Value::Null));
                } else if !expected.contains("scoped") {
                    assert!(request.params.get("turnId").is_none());
                }
                Ok(CodexInteractionReply::Result(json!({"action":"decline"})))
            }
        });
        let run = backend.run(request()).await.unwrap();
        drop(run.events);
        assert!(run.completion.await.unwrap().status.success(), "{scenario}");
    }
}
#[tokio::test]
async fn missing_malformed_oversize_wrong_scope_context_never_calls_host() {
    for scenario in [
        "file_missing",
        "file_missing_late",
        "file_prior_completed",
        "file_wrong_item",
        "file_context_thread",
        "file_context_turn",
        "mcp_early_thread_fail",
        "file_malformed",
        "file_wrong_turn",
        "file_oversize",
        "mcp_wrong_null",
    ] {
        let calls = Arc::new(AtomicUsize::new(0));
        let count = calls.clone();
        let backend = backend(scenario).with_interaction_handler(move |_| {
            count.fetch_add(1, Ordering::SeqCst);
            async {
                panic!("invalid context reached host");
                #[allow(unreachable_code)]
                Ok(CodexInteractionReply::Unsupported)
            }
        });
        let run = backend.run(request()).await.unwrap();
        drop(run.events);
        let err = run.completion.await.unwrap_err().to_string();
        assert!(!err.contains("秘密"));
        assert!(!err.contains("private"));
        assert_eq!(calls.load(Ordering::SeqCst), 0, "{scenario}");
    }
}
#[tokio::test]
async fn no_handler_and_host_failures_refuse_without_granting() {
    let run = backend("file").run(request()).await.unwrap();
    drop(run.events);
    assert!(run
        .completion
        .await
        .unwrap_err()
        .to_string()
        .contains("approval required"));
    for mode in 0..5 {
        let backend = backend("file").with_interaction_handler(move |_| {
            if mode == 0 {
                panic!("sync secret");
            }
            async move {
                match mode {
                    1 => panic!("async secret"),
                    2 => Err(CodexInteractionError::new()),
                    3 => Ok(CodexInteractionReply::Result(
                        json!({"decision":"invented"}),
                    )),
                    _ => Ok(CodexInteractionReply::Unsupported),
                }
            }
        });
        let run = backend.run(request()).await.unwrap();
        drop(run.events);
        assert!(run
            .completion
            .await
            .unwrap_err()
            .to_string()
            .contains("approval required"));
    }
}

#[cfg(unix)]
#[path = "codex_native_bridge/invalidation.rs"]
mod invalidation;
#[cfg(unix)]
struct WakeGate(std::path::PathBuf);
#[cfg(unix)]
impl Drop for WakeGate {
    fn drop(&mut self) {
        if let Ok(mut stream) = std::os::unix::net::UnixStream::connect(&self.0) {
            let _ = std::io::Write::write_all(&mut stream, b"abort");
        }
    }
}

#[tokio::test]
async fn all_native_methods_preserve_structured_decisions() {
    for scenario in ["command", "permissions", "user_input"] {
        let backend = backend(scenario).with_interaction_handler(|request| async move {
            assert!(request.item_context.is_none());
            let response = match request.method.as_str() {
                "item/commandExecution/requestApproval" => json!({"decision":"acceptForSession"}),
                "item/permissions/requestApproval" => {
                    json!({"permissions":{"network":{"enabled":false}},"scope":"turn"})
                }
                "item/tool/requestUserInput" => json!({"answers":{"q":{"answers":["a","b"]}}}),
                other => panic!("unexpected interaction {other}"),
            };
            Ok(CodexInteractionReply::Result(response))
        });
        let run = backend.run(request()).await.unwrap();
        drop(run.events);
        assert!(run.completion.await.unwrap().status.success());
    }
}
#[tokio::test]
async fn native_initial_resume_fork_policy_matrix_and_model_precedence() {
    let granular = json!({"granular":{"sandbox_approval":true,"rules":false,"mcp_elicitations":true,"request_permissions":false,"skill_approval":false}});
    for selector in [
        None,
        Some("agent_api.session.resume.v1"),
        Some("agent_api.session.fork.v1"),
    ] {
        for policy in [
            None,
            Some(json!("never")),
            Some(json!("untrusted")),
            Some(json!("on-request")),
            Some(granular.clone()),
            Some(
                json!({"granular":{"sandbox_approval":false,"rules":true,"mcp_elicitations":false}}),
            ),
        ] {
            let config = CodexBackendConfig {
                binary: Some(env!("CARGO_BIN_EXE_fake_codex_native_bridge").into()),
                env: [(
                    "NATIVE_POLICY".into(),
                    policy
                        .clone()
                        .unwrap_or(serde_json::Value::Null)
                        .to_string(),
                )]
                .into_iter()
                .collect(),
                ..Default::default()
            };
            let backend = CodexBackend::new(config);
            let mut request = request();
            request
                .extensions
                .remove("backend.codex.exec.approval_policy");
            if let Some(policy) = policy {
                request
                    .extensions
                    .insert("backend.codex.exec.approval_policy".into(), policy);
            }
            if let Some(selector) = selector {
                request
                    .extensions
                    .insert(selector.into(), json!({"selector":"id","id":"source"}));
            }
            let run = backend.run(request).await.unwrap();
            drop(run.events);
            let completion = run.completion.await.unwrap();
            assert_eq!(completion.final_text.as_deref(), Some("native final"));
        }
    }
    for selector in [None, Some("agent_api.session.resume.v1")] {
        for request_model in [None, Some("gpt-5.1-request")] {
            let model = request_model.unwrap_or("gpt-5-backend");
            let backend = CodexBackend::new(CodexBackendConfig {
                binary: Some(env!("CARGO_BIN_EXE_fake_codex_native_bridge").into()),
                model: Some("gpt-5-backend".into()),
                env: [("NATIVE_MODEL".into(), model.into())]
                    .into_iter()
                    .collect(),
                ..Default::default()
            });
            let mut request = request();
            if let Some(selector) = selector {
                request
                    .extensions
                    .insert(selector.into(), json!({"selector":"id","id":"source"}));
            }
            if let Some(model) = request_model {
                request
                    .extensions
                    .insert("agent_api.config.model.v1".into(), json!(model));
            }
            let run = backend.run(request).await.unwrap();
            drop(run.events);
            assert!(run.completion.await.unwrap().status.success());
        }
    }
}

#[tokio::test]
async fn exact_combined_delivery_and_context_count_bounds() {
    for (scenario, success) in [
        ("file_combined_exact", true),
        ("file_combined_over", false),
        ("file_count_16", true),
        ("file_count_17", false),
        ("file_aggregate", false),
    ] {
        let calls = Arc::new(AtomicUsize::new(0));
        let count = calls.clone();
        let backend = backend(scenario).with_interaction_handler(move |_| {
            let count = count.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Ok(CodexInteractionReply::Result(
                    json!({"decision":"acceptForSession"}),
                ))
            }
        });
        let run = backend.run(request()).await.unwrap();
        drop(run.events);
        let completion = run.completion.await;
        assert_eq!(completion.is_ok(), success, "{scenario}: {completion:?}");
        if success {
            assert_eq!(calls.load(Ordering::SeqCst), 1);
        } else {
            assert_eq!(calls.load(Ordering::SeqCst), 0);
        }
    }
}
#[tokio::test]
async fn native_roots_cwd_home_env_sandbox_preserve_existing_settings() {
    for selector in [None, Some("agent_api.session.resume.v1")] {
        for profile in ["legacy", "default", "requirement"] {
            for sandbox in ["workspace-write", "read-only", "danger-full-access"] {
                let dir = tempfile::tempdir().unwrap();
                let extra = dir.path().join("extra");
                std::fs::create_dir(&extra).unwrap();
                let home = dir.path().join("home");
                let cwd = dir.path().to_path_buf();
                let mut roots = vec![cwd.display().to_string(), extra.display().to_string()];
                if profile == "legacy" {
                    roots.push("/ambient".into());
                }
                let mut config = CodexBackendConfig {
                    binary: Some(env!("CARGO_BIN_EXE_fake_codex_native_bridge").into()),
                    codex_home: Some(home.clone()),
                    default_working_dir: Some(cwd.clone()),
                    env: [
                        ("NATIVE_ROOTS".into(), json!(roots).to_string()),
                        ("NATIVE_CWD".into(), cwd.display().to_string()),
                        ("NATIVE_SANDBOX".into(), sandbox.into()),
                        ("NATIVE_EXPECT_HOME".into(), home.display().to_string()),
                        ("NATIVE_EXPECT_ENV".into(), "request".into()),
                        ("ENV_PROBE".into(), "backend".into()),
                    ]
                    .into_iter()
                    .collect(),
                    ..Default::default()
                };
                if profile == "default" {
                    config
                        .env
                        .insert("NATIVE_PROFILE".into(), "workspace".into());
                }
                if profile == "requirement" {
                    config
                        .env
                        .insert("NATIVE_ALLOWED_PROFILE".into(), "true".into());
                }
                let backend = CodexBackend::new(config);
                let mut request = request();
                request.env.insert("ENV_PROBE".into(), "request".into());
                request.extensions.insert(
                    "agent_api.exec.add_dirs.v1".into(),
                    json!({"dirs":["extra", "extra"]}),
                );
                request
                    .extensions
                    .insert("backend.codex.exec.sandbox_mode".into(), json!(sandbox));
                if let Some(selector) = selector {
                    request
                        .extensions
                        .insert(selector.into(), json!({"selector":"id","id":"source"}));
                }
                let run = backend.run(request).await.unwrap();
                drop(run.events);
                assert!(run.completion.await.unwrap().status.success());
            }
        }
    }
}

#[tokio::test]
async fn current_roots_are_supplied_without_add_dirs_and_request_home_wins() {
    for selector in [None, Some("agent_api.session.resume.v1")] {
        let dir = tempfile::tempdir().unwrap();
        let cwd = dir.path().to_path_buf();
        let home = cwd.join("request-home");
        std::fs::create_dir(&home).unwrap();
        let backend = CodexBackend::new(CodexBackendConfig {
            binary: Some(env!("CARGO_BIN_EXE_fake_codex_native_bridge").into()),
            codex_home: Some(cwd.join("configured-home")),
            default_working_dir: Some(cwd.clone()),
            env: [
                (
                    "CODEX_HOME".into(),
                    cwd.join("backend-env-home").display().to_string(),
                ),
                (
                    "NATIVE_ROOTS".into(),
                    json!([cwd.display().to_string(), "/ambient"]).to_string(),
                ),
                ("NATIVE_EXPECT_HOME".into(), home.display().to_string()),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        });
        let mut request = request();
        request
            .env
            .insert("CODEX_HOME".into(), home.display().to_string());
        if let Some(selector) = selector {
            request
                .extensions
                .insert(selector.into(), json!({"selector":"id","id":"source"}));
        }
        let run = backend.run(request).await.unwrap();
        drop(run.events);
        assert!(run.completion.await.unwrap().status.success());
    }
}
#[tokio::test]
async fn granular_invalid_fields_and_noninteractive_contradictions_are_rejected() {
    for policy in [
        json!({"granular":{"rules":true}}),
        json!({"granular":{"sandbox_approval":true,"rules":true,"mcp_elicitations":true,"unknown":"secret"}}),
        json!({"granular":{"sandbox_approval":true,"rules":true,"mcp_elicitations":true,"skill_approval":1}}),
    ] {
        let mut request = request();
        request
            .extensions
            .insert("backend.codex.exec.approval_policy".into(), policy);
        let err = match backend("").run(request).await {
            Ok(_) => panic!("malformed granular accepted"),
            Err(error) => error,
        };
        assert!(!err.to_string().contains("secret"));
    }
    let mut request = request();
    request
        .extensions
        .insert("agent_api.exec.non_interactive".into(), json!(true));
    request.extensions.insert(
        "backend.codex.exec.approval_policy".into(),
        json!({"granular":{"sandbox_approval":true,"rules":true,"mcp_elicitations":true}}),
    );
    assert!(backend("").run(request).await.is_err());
}

#[tokio::test]
async fn final_text_uses_completed_native_message_phase() {
    for (scenario, expected) in [
        ("phase_commentary", None),
        ("phase_legacy", Some("native final")),
    ] {
        let run = backend(scenario).run(request()).await.unwrap();
        drop(run.events);
        assert_eq!(
            run.completion.await.unwrap().final_text.as_deref(),
            expected
        );
    }
}
#[cfg(unix)]
#[tokio::test]
async fn identical_context_repeat_preserves_pending_snapshot() {
    use std::{
        io::{Read, Write},
        os::unix::net::UnixListener,
        sync::Mutex,
    };
    use tokio::sync::oneshot;
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("gate");
    let listener = UnixListener::bind(&socket).unwrap();
    let _wake = WakeGate(socket.clone());
    let gate = tokio::task::spawn_blocking(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut ready = [0; 5];
        stream.read_exact(&mut ready).unwrap();
        stream
    });
    let (ready_tx, ready_rx) = oneshot::channel();
    let ready = Arc::new(Mutex::new(Some(ready_tx)));
    let (observed_tx, observed_rx) = oneshot::channel();
    let observed = Arc::new(Mutex::new(Some(observed_tx)));
    let (reply_tx, reply_rx) = oneshot::channel::<()>();
    let reply = Arc::new(Mutex::new(Some(reply_rx)));
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(env!("CARGO_BIN_EXE_fake_codex_native_bridge").into()),
        env: [
            ("NATIVE_SCENARIO".into(), "file_identical".into()),
            ("NATIVE_GATE".into(), socket.display().to_string()),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    })
    .with_interaction_handler(move |request| {
        let ready = ready.clone();
        let observed = observed.clone();
        let reply = reply.clone();
        async move {
            if request.method == "mcpServer/elicitation/request" {
                let _ = observed.lock().unwrap().take().unwrap().send(());
                return Ok(CodexInteractionReply::Result(json!({"action":"decline"})));
            }
            let receiver = reply.lock().unwrap().take().unwrap();
            let _ = ready.lock().unwrap().take().unwrap().send(());
            receiver.await.unwrap();
            Ok(CodexInteractionReply::Result(
                json!({"decision":"acceptForSession"}),
            ))
        }
    });
    let run = backend.run(request()).await.unwrap();
    drop(run.events);
    tokio::time::timeout(Duration::from_secs(3), ready_rx)
        .await
        .unwrap()
        .unwrap();
    let mut gate = gate.await.unwrap();
    gate.write_all(b"x").unwrap();
    tokio::time::timeout(Duration::from_secs(3), observed_rx)
        .await
        .unwrap()
        .unwrap();
    reply_tx.send(()).unwrap();
    assert!(run.completion.await.unwrap().status.success());
}

#[tokio::test]
async fn legacy_on_failure_is_forwarded_and_actual_native_rejection_is_truthful() {
    for rejects in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("policy");
        let backend = CodexBackend::new(CodexBackendConfig {
            binary: Some(env!("CARGO_BIN_EXE_fake_codex_native_bridge").into()),
            env: [
                (
                    "NATIVE_SCENARIO".into(),
                    if rejects { "policy_reject" } else { "" }.into(),
                ),
                ("NATIVE_POLICY".into(), json!("on-failure").to_string()),
                ("NATIVE_POLICY_LOG".into(), log.display().to_string()),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        });
        let mut request = request();
        request.extensions.insert(
            "backend.codex.exec.approval_policy".into(),
            json!("on-failure"),
        );
        let run = backend.run(request).await.unwrap();
        drop(run.events);
        let completion = run.completion.await;
        assert_eq!(std::fs::read_to_string(log).unwrap(), "\"on-failure\"");
        if rejects {
            assert_eq!(
                completion.unwrap_err().to_string(),
                "backend error: codex native approval policy unsupported (details redacted)"
            );
        } else {
            assert!(completion.unwrap().status.success());
        }
    }
}

#[tokio::test]
async fn overlapping_native_ids_on_two_owned_connections_remain_independent() {
    let barrier = Arc::new(tokio::sync::Barrier::new(3));
    let first = Arc::new(AtomicUsize::new(0));
    let second = Arc::new(AtomicUsize::new(0));
    let make = |count: Arc<AtomicUsize>| {
        let barrier = barrier.clone();
        backend("file").with_interaction_handler(move |request| {
            let barrier = barrier.clone();
            let count = count.clone();
            async move {
                assert_eq!(request.params["itemId"], "file");
                count.fetch_add(1, Ordering::SeqCst);
                barrier.wait().await;
                Ok(CodexInteractionReply::Result(
                    json!({"decision":"acceptForSession"}),
                ))
            }
        })
    };
    let a = make(first.clone());
    let b = make(second.clone());
    let a = a.run(request()).await.unwrap();
    let b = b.run(request()).await.unwrap();
    drop(a.events);
    drop(b.events);
    tokio::time::timeout(Duration::from_secs(3), barrier.wait())
        .await
        .unwrap();
    let (a, b) = tokio::join!(a.completion, b.completion);
    assert!(a.unwrap().status.success());
    assert!(b.unwrap().status.success());
    assert_eq!(first.load(Ordering::SeqCst), 1);
    assert_eq!(second.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn handler_registration_alone_keeps_unattended_exec_route() {
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(env!("CARGO_BIN_EXE_fake_codex_stream_exec_scenarios_agent_api").into()),
        ..Default::default()
    })
    .with_interaction_handler(|_| async {
        panic!("unattended run called host");
        #[allow(unreachable_code)]
        Ok(CodexInteractionReply::Unsupported)
    });
    let run = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    drop(run.events);
    assert!(run.completion.await.unwrap().status.success());
}

#[path = "codex_native_bridge/environment.rs"]
mod environment;

#[cfg(unix)]
#[path = "codex_native_bridge/queued_reply.rs"]
mod queued_reply;
