//! Two compiled isolated profiles; no receipt or environment value chooses trust.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Profile {
    Channel,
    Core,
}
impl Profile {
    pub(crate) fn fixture(self) -> &'static str {
        match self {
            Self::Channel => super::FIXTURE,
            Self::Core => "crates/xtask/tests/fixtures/depth-core/record.json",
        }
    }
    pub(crate) fn contract(self) -> &'static str {
        match self {
            Self::Channel => super::CONTRACT,
            Self::Core => "crates/xtask/tests/fixtures/depth-core/contract.md",
        }
    }
    pub(crate) fn directory(self) -> &'static str {
        match self {
            Self::Channel => "crates/xtask/tests/fixtures/depth-ci",
            Self::Core => "crates/xtask/tests/fixtures/depth-core",
        }
    }
    pub(crate) fn build_profile(self) -> String {
        format!("{}/build-profile.json", self.directory())
    }
    pub(crate) fn version(self) -> &'static str {
        match self {
            Self::Channel => "0.0.0",
            Self::Core => "0.156.1",
        }
    }
    pub(crate) fn package(self) -> &'static str {
        match self {
            Self::Channel => "unified-agent-api-wrapper-events",
            Self::Core => "unified-agent-api",
        }
    }
    pub(crate) fn test(self) -> &'static str {
        match self {
            Self::Channel => "depth_ci_channel",
            Self::Core => "depth_ci_core",
        }
    }
    pub(crate) fn features(self) -> Vec<String> {
        match self {
            Self::Channel => vec![],
            Self::Core => ["aider", "claude_code", "codex", "gemini_cli", "opencode"]
                .map(str::to_owned)
                .to_vec(),
        }
    }
    pub(crate) fn artifact(self) -> &'static str {
        match self {
            Self::Channel => "depth-channel-v1",
            Self::Core => "depth-codex-core-v1",
        }
    }
    pub(crate) fn executor(self) -> &'static str {
        match self {
            Self::Channel => "github-actions:rust-tests/isolated-channel-v1",
            Self::Core => "github-actions:rust-tests/isolated-codex-core-v1",
        }
    }
    pub(crate) fn trust(self) -> &'static str {
        match self {
            Self::Channel => "github-actions-channel-v1",
            Self::Core => "github-actions-codex-core-v1",
        }
    }
    pub(crate) fn evidence(self) -> String {
        format!(
            "cli_manifests/codex/reports/{}/evidence/{}",
            self.version(),
            match self {
                Self::Channel => "channel-v1",
                Self::Core => "core-v1",
            }
        )
    }
    pub(crate) fn record(self, raw: &str) -> Result<crate::depth_record::DepthRecord, String> {
        if self == Self::Channel {
            super::record(raw)
        } else {
            crate::depth_record::DepthRecord::parse(raw, "codex", self.version())
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CaseSpec {
    pub name: String,
    pub assertion: String,
    pub production_inputs: Vec<String>,
    pub counterfactual: String,
}
impl Profile {
    pub(crate) fn case_names(self, raw: Option<&str>) -> Result<Vec<String>, String> {
        if self == Self::Channel {
            return Ok(super::CASES.map(str::to_owned).to_vec());
        }
        let specs: std::collections::BTreeMap<String, Vec<CaseSpec>> =
            serde_json::from_str(raw.ok_or("missing CORE cases")?).map_err(|e| e.to_string())?;
        let mut names: Vec<_> = specs.values().flatten().map(|c| c.name.clone()).collect();
        names.sort();
        names.dedup();
        if names.is_empty() {
            return Err("empty CORE cases".into());
        }
        Ok(names)
    }
    pub(crate) fn obligations(
        self,
        record: &crate::depth_record::DepthRecord,
    ) -> Vec<crate::depth_record::ObligationRef> {
        use crate::{agent_registry::support_depth::ObligationId, depth_record::ObligationRef};
        if self == Self::Channel {
            return vec![ObligationRef {
                operation: "controlled_channel".into(),
                id: Some(ObligationId::V1),
                capability: None,
                target: super::TARGET.into(),
            }];
        }
        let Some(o) = &record.obligations else {
            return vec![];
        };
        let mut refs = vec![];
        for op in &o.operations {
            for target in &o.targets {
                refs.extend(op.obligations.iter().map(|id| ObligationRef {
                    operation: op.id.clone(),
                    id: Some(*id),
                    capability: None,
                    target: target.clone(),
                }));
                refs.extend(
                    record
                        .policy
                        .selection
                        .enrollment
                        .capabilities
                        .iter()
                        .filter(|c| c.operations.contains(&op.id))
                        .map(|c| ObligationRef {
                            operation: op.id.clone(),
                            id: None,
                            capability: Some(c.id.clone()),
                            target: target.clone(),
                        }),
                );
            }
        }
        refs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn core_profile_has_all_due_mapped_without_na() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = Profile::Core;
        let record = p
            .record(&std::fs::read_to_string(root.join(p.fixture())).unwrap())
            .unwrap();
        let mappings: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.join(p.directory()).join("cases.json")).unwrap(),
        )
        .unwrap();
        let expected = [
            "R1", "V1", "G1", "E1", "A1", "X1", "X2", "X3", "X4", "OUT3", "LC2", "M",
        ];
        assert_eq!(mappings.as_object().unwrap().len(), expected.len());
        for key in expected {
            assert!(!mappings[key].as_array().unwrap().is_empty());
        }
        assert!(record.execution.is_none());
        assert_eq!(
            record.policy.selection.enrollment.capabilities[0].id,
            "agent_api.run"
        );
        assert!(record.policy.selection.enrollment.operations[0]
            .not_applicable_permitted
            .is_empty());
    }
}
