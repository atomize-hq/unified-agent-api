use std::{fs, path::PathBuf};

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {}

pub fn run(_args: Args) -> Result<(), String> {
    let workspace_root = resolve_workspace_root()?;
    xtask::capability_publication::audit_current_capability_publication(&workspace_root)
}

fn resolve_workspace_root() -> Result<PathBuf, String> {
    let current_dir = std::env::current_dir().map_err(|err| format!("current_dir: {err}"))?;
    for candidate in current_dir.ancestors() {
        let cargo_toml = candidate.join("Cargo.toml");
        let Ok(text) = fs::read_to_string(&cargo_toml) else {
            continue;
        };
        if text.contains("[workspace]") {
            return Ok(candidate.to_path_buf());
        }
    }

    Err(format!(
        "could not resolve workspace root from {}",
        current_dir.display()
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use agent_api::AgentWrapperCapabilities;

    #[test]
    fn non_allowlisted_agent_api_cap_supported_by_1_backend_fails() {
        let mut backends = BTreeMap::<String, AgentWrapperCapabilities>::new();
        backends.insert(
            "claude_code".to_string(),
            AgentWrapperCapabilities {
                ids: ["agent_api.run".to_string()].into_iter().collect(),
            },
        );
        backends.insert(
            "codex".to_string(),
            AgentWrapperCapabilities {
                ids: [
                    "agent_api.run".to_string(),
                    "agent_api.tools.results.v1".to_string(),
                ]
                .into_iter()
                .collect(),
            },
        );

        let err =
            xtask::capability_publication::audit_capability_publication(&backends).unwrap_err();
        assert!(err.contains("published agents: [claude_code, codex]"));
        assert!(err.contains("agent_api.tools.results.v1"));
    }

    #[test]
    fn non_allowlisted_agent_api_cap_supported_by_2_backends_passes() {
        let mut backends = BTreeMap::<String, AgentWrapperCapabilities>::new();
        backends.insert(
            "claude_code".to_string(),
            AgentWrapperCapabilities {
                ids: [
                    "agent_api.run".to_string(),
                    "agent_api.tools.results.v1".to_string(),
                ]
                .into_iter()
                .collect(),
            },
        );
        backends.insert(
            "codex".to_string(),
            AgentWrapperCapabilities {
                ids: [
                    "agent_api.run".to_string(),
                    "agent_api.tools.results.v1".to_string(),
                ]
                .into_iter()
                .collect(),
            },
        );

        xtask::capability_publication::audit_capability_publication(&backends).unwrap();
    }

    #[test]
    fn allowlisted_agent_api_cap_supported_by_1_backend_is_ignored() {
        let mut backends = BTreeMap::<String, AgentWrapperCapabilities>::new();
        backends.insert(
            "claude_code".to_string(),
            AgentWrapperCapabilities {
                ids: ["agent_api.run".to_string()].into_iter().collect(),
            },
        );
        backends.insert(
            "codex".to_string(),
            AgentWrapperCapabilities {
                ids: Default::default(),
            },
        );

        xtask::capability_publication::audit_capability_publication(&backends).unwrap();
    }

    #[test]
    fn allowlisted_mcp_cap_supported_by_1_backend_is_ignored() {
        let mut backends = BTreeMap::<String, AgentWrapperCapabilities>::new();
        backends.insert(
            "claude_code".to_string(),
            AgentWrapperCapabilities {
                ids: ["agent_api.tools.mcp.list.v1".to_string()]
                    .into_iter()
                    .collect(),
            },
        );
        backends.insert(
            "codex".to_string(),
            AgentWrapperCapabilities {
                ids: Default::default(),
            },
        );

        xtask::capability_publication::audit_capability_publication(&backends).unwrap();
    }
}
