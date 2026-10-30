use super::*;
#[tokio::test]
async fn every_selector_refuses_invalid_policy_sandbox_and_fork_overrides_before_spawn() {
    for selector in [
        None,
        Some(("agent_api.session.resume.v1", "last")),
        Some(("agent_api.session.resume.v1", "id")),
        Some(("agent_api.session.fork.v1", "last")),
        Some(("agent_api.session.fork.v1", "id")),
    ] {
        for regime in ["unattended", "interactive", "external"] {
            let mut invalid = vec![
                (
                    "backend.codex.exec.approval_policy",
                    json!("sk-unknown-policy"),
                ),
                (
                    "backend.codex.exec.sandbox_mode",
                    json!("sk-unknown-sandbox"),
                ),
                (
                    "backend.codex.exec.approval_policy",
                    json!({"granular":{"sandbox_approval":true}}),
                ),
            ];
            if selector.is_some_and(|(key, _)| key.contains("fork")) {
                invalid.extend([
                    ("agent_api.config.model.v1", json!("gpt-5")),
                    ("agent_api.exec.add_dirs.v1", json!(["/tmp"])),
                ]);
            }
            for (key, value) in invalid {
                let tmp = tempfile::tempdir().unwrap();
                let marker = tmp.path().join("spawned");
                let backend = CodexBackend::new(CodexBackendConfig {
                    binary: Some(env!("CARGO_BIN_EXE_fake_codex_native_bridge").into()),
                    allow_external_sandbox_exec: true,
                    env: [("NATIVE_SPAWN_MARKER".into(), marker.display().to_string())]
                        .into_iter()
                        .collect(),
                    ..Default::default()
                });
                let mut request = request();
                request
                    .extensions
                    .remove("backend.codex.exec.approval_policy");
                request.extensions.insert(
                    "agent_api.exec.non_interactive".into(),
                    json!(regime != "interactive"),
                );
                if regime == "external" {
                    request
                        .extensions
                        .insert("agent_api.exec.external_sandbox.v1".into(), json!(true));
                }
                if let Some((key, selection)) = selector {
                    request.extensions.insert(
                        key.into(),
                        if selection == "last" {
                            json!({"selector":"last"})
                        } else {
                            json!({"selector":"id","id":"source"})
                        },
                    );
                }
                request.extensions.insert(key.into(), value);
                let err = match backend.run(request).await {
                    Ok(_) => panic!("invalid request accepted: {selector:?}/{regime}/{key}"),
                    Err(err) => err,
                };
                assert!(
                    matches!(
                        &err,
                        agent_api::AgentWrapperError::InvalidRequest { .. }
                            | agent_api::AgentWrapperError::UnsupportedCapability { .. }
                    ) || matches!(&err,agent_api::AgentWrapperError::Backend{message}
                        if matches!(message.as_str(),"model override unsupported for codex fork"|"add_dirs unsupported for codex fork")),
                    "{err}"
                );
                assert!(
                    !marker.exists(),
                    "invalid request spawned a native child: {selector:?}/{regime}/{key}"
                );
            }
        }
    }
}
