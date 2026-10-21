use std::collections::BTreeSet;

use super::evaluator::{self, BackendModel, ConfigKey, ConfigValues, Gate, HostFacts};
use crate::agent_registry::AgentRegistry;

pub const AGENTS: [&str; 5] = ["aider", "claude_code", "codex", "gemini_cli", "opencode"];

#[derive(Clone, Debug)]
pub struct Model {
    pub agent_id: String,
    pub unconditional: BTreeSet<String>,
    pub gates: Vec<ModelGate>,
}

#[derive(Clone, Debug)]
pub struct ModelGate {
    pub id: String,
    pub hosts: Option<Vec<(&'static str, &'static str)>>,
    pub config: Option<&'static str>,
}

pub fn host_for_target(target: &str) -> Result<(&'static str, &'static str), String> {
    match target {
        "x86_64-unknown-linux-musl" | "linux-x64" => Ok(("linux", "x86_64")),
        "darwin-arm64" => Ok(("macos", "aarch64")),
        "win32-x64" => Ok(("windows", "x86_64")),
        _ => Err(format!("unsupported runtime advertising target: {target}")),
    }
}

fn config_key(key: &str) -> Result<&'static str, String> {
    match key {
        "allow_mcp_write" => Ok("AllowMcpWrite"),
        "allow_external_sandbox_exec" => Ok("AllowExternalSandboxExec"),
        _ => Err(format!("unsupported advertising config key: {key}")),
    }
}

pub fn project(registry: &AgentRegistry) -> Result<Vec<Model>, String> {
    let mut models = Vec::new();
    for agent in AGENTS {
        let entry = registry
            .find(agent)
            .ok_or_else(|| format!("missing built-in {agent}"))?;
        if entry.backend_module != format!("crates/agent_api/src/backends/{agent}") {
            return Err(format!("{agent}: unsupported backend module"));
        }
        let declaration = &entry.capability_declaration;
        let mut gates = Vec::new();
        for gate in &declaration.target_gated {
            gates.push(ModelGate {
                id: gate.capability_id.clone(),
                hosts: Some(
                    gate.targets
                        .iter()
                        .map(|t| host_for_target(t))
                        .collect::<Result<_, _>>()?,
                ),
                config: None,
            });
        }
        for gate in &declaration.config_gated {
            if !matches!(agent, "codex" | "claude_code") {
                return Err(format!("{agent}: no runtime advertising config fields"));
            }
            gates.push(ModelGate {
                id: gate.capability_id.clone(),
                hosts: gate
                    .targets
                    .as_ref()
                    .map(|targets| targets.iter().map(|t| host_for_target(t)).collect())
                    .transpose()?,
                config: Some(config_key(&gate.config_key)?),
            });
        }
        for gate in &mut gates {
            if let Some(hosts) = &mut gate.hosts {
                hosts.sort();
                hosts.dedup();
                if hosts.is_empty() {
                    return Err("empty runtime host restriction".into());
                }
            }
        }
        gates.sort_by_key(|gate| format!("{}{:?}{:?}", gate.id, gate.config, gate.hosts));
        models.push(Model {
            agent_id: agent.into(),
            unconditional: declaration
                .always_on
                .iter()
                .chain(&declaration.backend_extensions)
                .cloned()
                .collect(),
            gates,
        });
    }
    // A new built-in must add supported identity/module/feature interpretation in the same change.
    if registry.agents.len() != AGENTS.len() {
        return Err("unsupported built-in inventory".into());
    }
    Ok(models)
}

impl Model {
    pub fn capabilities(
        &self,
        os: &str,
        arch: &str,
        write: bool,
        sandbox: bool,
    ) -> BTreeSet<String> {
        let unconditional: Vec<_> = self.unconditional.iter().map(String::as_str).collect();
        let hosts: Vec<Option<Vec<_>>> = self
            .gates
            .iter()
            .map(|gate| {
                gate.hosts.as_ref().map(|hosts| {
                    hosts
                        .iter()
                        .map(|(os, arch)| HostFacts { os, arch })
                        .collect()
                })
            })
            .collect();
        let gates: Vec<_> = self
            .gates
            .iter()
            .zip(&hosts)
            .map(|(gate, hosts)| Gate {
                id: &gate.id,
                hosts: hosts.as_deref(),
                config: gate.config.map(|key| match key {
                    "AllowMcpWrite" => ConfigKey::AllowMcpWrite,
                    "AllowExternalSandboxExec" => ConfigKey::AllowExternalSandboxExec,
                    _ => unreachable!("validated projection"),
                }),
            })
            .collect();
        evaluator::evaluate(
            &BackendModel {
                agent_id: &self.agent_id,
                unconditional: &unconditional,
                gates: &gates,
            },
            HostFacts { os, arch },
            ConfigValues {
                allow_mcp_write: write,
                allow_external_sandbox_exec: sandbox,
            },
        )
    }
}

pub fn render(models: &[Model]) -> String {
    let mut source = String::from("// Generated by cargo run -p xtask -- runtime-advertising-model.\nuse super::evaluator::{BackendModel, ConfigKey, Gate, HostFacts};\npub(crate) const MODEL_REVISION: u32 = 1;\n");
    for model in models {
        source.push_str(&format!("pub(crate) static {}: BackendModel<'static> = BackendModel {{ agent_id: {:?}, unconditional: &[", model.agent_id.to_uppercase(), model.agent_id));
        for id in &model.unconditional {
            source.push_str(&format!("{id:?},"));
        }
        source.push_str("], gates: &[");
        for gate in &model.gates {
            source.push_str(&format!("Gate {{ id: {:?}, hosts: ", gate.id));
            if let Some(hosts) = &gate.hosts {
                source.push_str("Some(&[");
                for (os, arch) in hosts {
                    source.push_str(&format!("HostFacts {{ os: {os:?}, arch: {arch:?}, }}"));
                    if hosts.len() > 1 {
                        source.push(',');
                    }
                }
                source.push_str("])");
            } else {
                source.push_str("None");
            }
            source.push_str(&format!(
                ", config: {}, }},",
                gate.config
                    .map_or("None".into(), |key| format!("Some(ConfigKey::{key})"))
            ));
        }
        source.push_str("], };\n");
    }
    source
}
