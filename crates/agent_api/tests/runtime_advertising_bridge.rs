#![cfg(all(
    feature = "codex",
    feature = "claude_code",
    feature = "aider",
    feature = "gemini_cli",
    feature = "opencode"
))]

#[path = "runtime_advertising/expected.rs"]
mod baseline;

use agent_api::{
    backends::{aider::*, claude_code::*, codex::*, gemini_cli::*, opencode::*},
    AgentWrapperBackend,
};

#[test]
fn actual_backends_preserve_complete_baseline_sets_and_identity() {
    for write in [false, true] {
        for sandbox in [false, true] {
            let backends: Vec<(&str, Box<dyn AgentWrapperBackend>)> = vec![
                (
                    "codex",
                    Box::new(CodexBackend::new(CodexBackendConfig {
                        allow_mcp_write: write,
                        allow_external_sandbox_exec: sandbox,
                        ..Default::default()
                    })),
                ),
                (
                    "claude_code",
                    Box::new(ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
                        allow_mcp_write: write,
                        allow_external_sandbox_exec: sandbox,
                        ..Default::default()
                    })),
                ),
                ("aider", Box::new(AiderBackend::new(Default::default()))),
                (
                    "gemini_cli",
                    Box::new(GeminiCliBackend::new(Default::default())),
                ),
                (
                    "opencode",
                    Box::new(OpencodeBackend::new(Default::default())),
                ),
            ];
            for (id, backend) in backends {
                assert_eq!(backend.kind().as_str(), id);
                let expected = baseline::expected(
                    id,
                    std::env::consts::OS,
                    std::env::consts::ARCH,
                    write,
                    sandbox,
                );
                let mut gateway = agent_api::AgentWrapperGateway::new();
                let backend: std::sync::Arc<dyn AgentWrapperBackend> =
                    std::sync::Arc::from(backend);
                gateway.register(backend.clone()).unwrap();
                let resolved = gateway
                    .backend(&agent_api::AgentWrapperKind::new(id).unwrap())
                    .expect("canonical gateway identity");
                assert_eq!(resolved.capabilities().ids, expected);
                assert_eq!(
                    backend.capabilities().ids,
                    baseline::expected(
                        id,
                        std::env::consts::OS,
                        std::env::consts::ARCH,
                        write,
                        sandbox
                    ),
                    "{id}, write={write}, sandbox={sandbox}"
                );
            }
        }
    }
    assert!(!CodexBackendConfig::default().allow_mcp_write);
    assert!(!CodexBackendConfig::default().allow_external_sandbox_exec);
    assert!(!ClaudeCodeBackendConfig::default().allow_mcp_write);
    assert!(!ClaudeCodeBackendConfig::default().allow_external_sandbox_exec);
}
