use super::{request, CodexBackend, CodexBackendConfig};
use agent_api::AgentWrapperBackend;
use serde_json::json;

#[tokio::test]
async fn ambient_binary_selection_is_tested_in_an_isolated_process() {
    const CHILD: &str = "UAA_NATIVE_BINARY_CHILD";
    if std::env::var_os(CHILD).is_none() {
        for explicit in [false, true] {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "environment::ambient_binary_selection_is_tested_in_an_isolated_process",
                    "--nocapture",
                ])
                .env("PATH", "/no/codex/on/path")
                .env(CHILD, if explicit { "explicit" } else { "ambient" })
                .env(
                    "CODEX_BINARY",
                    if explicit {
                        "/no/ambient/codex"
                    } else {
                        env!("CARGO_BIN_EXE_fake_codex_native_bridge")
                    },
                )
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        return;
    }
    for selector in [None, Some("agent_api.session.resume.v1")] {
        let explicit = std::env::var(CHILD).unwrap() == "explicit";
        {
            let backend = CodexBackend::new(CodexBackendConfig {
                binary: explicit.then(|| env!("CARGO_BIN_EXE_fake_codex_native_bridge").into()),
                // Per-call env must not change which executable the wrapper selected.
                env: [("CODEX_BINARY".into(), "/not/the/selected/binary".into())]
                    .into_iter()
                    .collect(),
                ..Default::default()
            });
            let mut req = request();
            if let Some(selector) = selector {
                req.extensions
                    .insert(selector.into(), json!({"selector":"id","id":"source"}));
            }
            let run = backend.run(req).await.unwrap();
            drop(run.events);
            assert!(run.completion.await.unwrap().status.success());
        }
    }
}
