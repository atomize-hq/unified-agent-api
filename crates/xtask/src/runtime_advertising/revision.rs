use super::{
    conformance,
    features::Features,
    projection::{self, Model, AGENTS},
    source::{normalized, Source},
    DATA,
};
use crate::agent_registry::AgentRegistry;
use std::{collections::BTreeSet, path::Path};

pub struct Revision {
    models: Vec<Model>,
    features: Features,
}

#[derive(Clone, Copy, Debug)]
pub struct Context<'a> {
    pub os: &'a str,
    pub arch: &'a str,
    pub write: bool,
    pub sandbox: bool,
    pub requested: u8,
    pub default_features: bool,
    pub default_config: bool,
}

impl Revision {
    pub fn workspace(root: &Path) -> Result<Self, String> {
        Self::load(&Source::workspace(root)?)
    }
    pub fn tip(root: &Path, revision: &str) -> Result<Self, String> {
        Self::load(&Source::tip(root, revision)?)
    }
    pub fn load(source: &Source) -> Result<Self, String> {
        let registry =
            AgentRegistry::parse(&source.read(crate::agent_registry::REGISTRY_RELATIVE_PATH)?)
                .map_err(|e| e.to_string())?;
        let models = projection::project(&registry)?;
        if normalized(&source.read(DATA)?)? != normalized(&projection::render(&models))? {
            return Err("runtime advertising data is stale or unsupported".into());
        }
        conformance::check(source)?;
        let features = Features::load(source)?;
        Ok(Self { models, features })
    }

    /// The caller context is interpreted using this tree's data, feature closure and defaults.
    pub fn capabilities(
        &self,
        agent: &str,
        context: Context<'_>,
    ) -> Result<BTreeSet<String>, String> {
        let model = self
            .models
            .iter()
            .find(|m| m.agent_id == agent)
            .ok_or("unknown modeled agent")?;
        if !self
            .features
            .enabled(context.requested, context.default_features)?
            .contains(agent)
        {
            return Ok(BTreeSet::new());
        }
        // V1 source conformance admits only derived Default on plain bool fields, both false.
        let (write, sandbox) = if context.default_config {
            (false, false)
        } else {
            (context.write, context.sandbox)
        };
        Ok(model.capabilities(context.os, context.arch, write, sandbox))
    }

    /// Classify a concrete host in the exact partition used by `additions`, including OTHER.
    pub fn host_cell<'a>(&self, tip: &Self, os: &'a str, arch: &'a str) -> (&'a str, &'a str) {
        let hosts: Vec<_> = self
            .models
            .iter()
            .chain(&tip.models)
            .flat_map(|model| &model.gates)
            .flat_map(|gate| gate.hosts.iter().flatten())
            .collect();
        (
            if hosts.iter().any(|(o, _)| *o == os) {
                os
            } else {
                "other"
            },
            if hosts.iter().any(|(_, a)| *a == arch) {
                arch
            } else {
                "other"
            },
        )
    }

    pub fn additions(&self, tip: &Self) -> Result<Vec<Addition>, String> {
        let mut os_values = BTreeSet::from(["other"]);
        let mut arch_values = BTreeSet::from(["other"]);
        for model in self.models.iter().chain(&tip.models) {
            for gate in &model.gates {
                for (os, arch) in gate.hosts.iter().flatten() {
                    os_values.insert(os);
                    arch_values.insert(arch);
                }
            }
        }
        let mut added = Vec::new();
        for agent in AGENTS {
            for os in &os_values {
                for arch in &arch_values {
                    for requested in 0..32 {
                        for default_features in [false, true] {
                            for (write, sandbox, default_config) in [
                                (false, false, false),
                                (true, false, false),
                                (false, true, false),
                                (true, true, false),
                                (false, false, true),
                            ] {
                                let context = Context {
                                    os,
                                    arch,
                                    requested,
                                    default_features,
                                    write,
                                    sandbox,
                                    default_config,
                                };
                                let now = self.capabilities(agent, context)?;
                                let before = tip.capabilities(agent, context)?;
                                for id in now.difference(&before) {
                                    added.push(Addition {
                                        agent: agent.into(),
                                        capability: id.clone(),
                                        os: (*os).into(),
                                        arch: (*arch).into(),
                                        requested,
                                        default_features,
                                        write,
                                        sandbox,
                                        default_config,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(added)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Addition {
    pub agent: String,
    pub capability: String,
    pub os: String,
    pub arch: String,
    pub requested: u8,
    pub default_features: bool,
    pub write: bool,
    pub sandbox: bool,
    pub default_config: bool,
}
