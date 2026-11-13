//! Compiled isolated producer bindings; no receipt or environment value chooses trust.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Profile {
    Channel,
    Core,
    /// `synthetic-wrapper-v1`: machinery proof on `depth_fixture` (MP-P1); qualifies no agent.
    Synthetic,
    /// The same cases bound to committed ON2 inputs and an approval Event.
    SyntheticOnboarding,
}
impl Profile {
    pub(crate) fn synthetic(self) -> bool {
        matches!(self, Self::Synthetic | Self::SyntheticOnboarding)
    }
    pub(crate) fn for_event(event: &crate::depth_record::Event) -> Self {
        match event {
            crate::depth_record::Event::Onboarding { .. } => Self::SyntheticOnboarding,
            crate::depth_record::Event::Maintenance { .. } => Self::Synthetic,
        }
    }
    /// Only source-installed finite bindings are candidates. No flag or environment selects one.
    pub(crate) fn synthetic_in(
        source: &crate::runtime_advertising::source::Source,
    ) -> Result<Self, String> {
        match source.optional(super::onboarding::RECORD)? {
            Some(raw) => {
                let record =
                    crate::depth_record::DepthRecord::parse(&raw, "depth_fixture", "0.2.0")?;
                Ok(Self::for_event(&record.event))
            }
            None => Ok(Self::Synthetic),
        }
    }
    pub(crate) fn agent_id(self) -> &'static str {
        match self {
            Self::Channel | Self::Core => "codex",
            Self::Synthetic | Self::SyntheticOnboarding => "depth_fixture",
        }
    }
    /// The `rust-tests` collection, upload and binding step names use `<verb> <label> ...`.
    pub(crate) fn step_label(self) -> &'static str {
        match self {
            Self::Channel => "isolated depth",
            Self::Core => "isolated CORE depth",
            Self::Synthetic | Self::SyntheticOnboarding => "isolated synthetic depth",
        }
    }
    pub(crate) fn fixture(self) -> &'static str {
        match self {
            Self::Channel => super::FIXTURE,
            Self::Core => "crates/xtask/tests/fixtures/depth-core/record.json",
            Self::Synthetic => "crates/xtask/tests/fixtures/depth-synthetic/record.json",
            Self::SyntheticOnboarding => super::onboarding::RECORD,
        }
    }
    pub(crate) fn contract(self) -> &'static str {
        match self {
            Self::Channel => super::CONTRACT,
            Self::Core => "crates/xtask/tests/fixtures/depth-core/contract.md",
            Self::Synthetic | Self::SyntheticOnboarding => {
                "crates/xtask/tests/fixtures/depth-synthetic/contract.md"
            }
        }
    }
    pub(crate) fn directory(self) -> &'static str {
        match self {
            Self::Channel => "crates/xtask/tests/fixtures/depth-ci",
            Self::Core => "crates/xtask/tests/fixtures/depth-core",
            Self::Synthetic | Self::SyntheticOnboarding => {
                "crates/xtask/tests/fixtures/depth-synthetic"
            }
        }
    }
    pub(crate) fn requirements(self) -> Option<&'static str> {
        match self {
            Self::Channel | Self::Synthetic | Self::SyntheticOnboarding => None,
            Self::Core => Some("docs/specs"),
        }
    }
    pub(crate) fn build_profile(self) -> String {
        format!("{}/build-profile.json", self.directory())
    }
    pub(crate) fn version(self) -> &'static str {
        match self {
            Self::Channel => "0.0.0",
            Self::Core => "0.156.1",
            Self::Synthetic | Self::SyntheticOnboarding => "0.2.0",
        }
    }
    pub(crate) fn package(self) -> &'static str {
        match self {
            Self::Channel => "unified-agent-api-wrapper-events",
            Self::Core => "unified-agent-api",
            Self::Synthetic | Self::SyntheticOnboarding => "unified-agent-api-depth-fixture",
        }
    }
    pub(crate) fn test(self) -> &'static str {
        match self {
            Self::Channel => "depth_ci_channel",
            Self::Core => "depth_ci_core",
            Self::Synthetic | Self::SyntheticOnboarding => "depth_ci_synthetic",
        }
    }
    pub(crate) fn features(self) -> Vec<String> {
        match self {
            Self::Channel | Self::Synthetic | Self::SyntheticOnboarding => vec![],
            Self::Core => ["aider", "claude_code", "codex", "gemini_cli", "opencode"]
                .map(str::to_owned)
                .to_vec(),
        }
    }
    pub(crate) fn artifact(self) -> &'static str {
        match self {
            Self::Channel => "depth-channel-v1",
            Self::Core => "depth-codex-core-v1",
            Self::Synthetic | Self::SyntheticOnboarding => "depth-synthetic-wrapper-v1",
        }
    }
    pub(crate) fn executor(self) -> &'static str {
        match self {
            Self::Channel => "github-actions:rust-tests/isolated-channel-v1",
            Self::Core => "github-actions:rust-tests/isolated-codex-core-v1",
            Self::Synthetic | Self::SyntheticOnboarding => {
                "github-actions:rust-tests/isolated-synthetic-wrapper-v1"
            }
        }
    }
    pub(crate) fn trust(self) -> &'static str {
        match self {
            Self::Channel => "github-actions-channel-v1",
            Self::Core => "github-actions-codex-core-v1",
            Self::Synthetic | Self::SyntheticOnboarding => "github-actions-synthetic-wrapper-v1",
        }
    }
    pub(crate) fn evidence(self) -> String {
        format!(
            "cli_manifests/{}/reports/{}/evidence/{}",
            self.agent_id(),
            self.version(),
            match self {
                Self::Channel => "channel-v1",
                Self::Core => "core-v1",
                Self::Synthetic | Self::SyntheticOnboarding => "synthetic-wrapper-v1",
            }
        )
    }
    pub(crate) fn record(self, raw: &str) -> Result<crate::depth_record::DepthRecord, String> {
        if self == Self::Channel {
            super::record(raw)
        } else {
            let mut record =
                crate::depth_record::DepthRecord::parse(raw, self.agent_id(), self.version())?;
            if self.synthetic() {
                let onboarding =
                    matches!(record.event, crate::depth_record::Event::Onboarding { .. });
                if onboarding != (self == Self::SyntheticOnboarding) {
                    return Err("synthetic producer Event differs from its compiled binding".into());
                }
            }
            if self == Self::SyntheticOnboarding {
                // Later reporting and accepting entries preserve the producer's frozen inputs.
                record.execution = None;
                record.acceptance_entries.clear();
            }
            Ok(record)
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
            serde_json::from_str(raw.ok_or("missing profile cases")?).map_err(|e| e.to_string())?;
        let mut names: Vec<_> = specs.values().flatten().map(|c| c.name.clone()).collect();
        names.sort();
        names.dedup();
        if names.is_empty() {
            return Err("empty profile cases".into());
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

    #[test]
    fn synthetic_profile_maps_exactly_the_due_obligations_to_real_cases() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = Profile::Synthetic;
        let record = p
            .record(&std::fs::read_to_string(root.join(p.fixture())).unwrap())
            .unwrap();
        let raw = std::fs::read_to_string(root.join(p.directory()).join("cases.json")).unwrap();
        let mappings: std::collections::BTreeMap<String, Vec<CaseSpec>> =
            serde_json::from_str(&raw).unwrap();
        // Wrapper-only: BASE, OUT3 and LC2 are due; M is not, and no capability is mapped.
        let due: Vec<_> = crate::depth_record::due(&record)
            .unwrap()
            .into_iter()
            .map(|o| {
                assert!(o.capability.is_none());
                assert_eq!(o.target, "x86_64-unknown-linux-gnu");
                serde_json::to_value(o.id.unwrap())
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
            .collect();
        let mut keys: Vec<_> = mappings.keys().cloned().collect();
        let mut due_sorted = due.clone();
        keys.sort();
        due_sorted.sort();
        assert_eq!(keys, due_sorted);
        assert_eq!(keys.len(), 11);
        assert_eq!(p.obligations(&record).len(), 11);
        let enrollment = &record.policy.selection.enrollment;
        assert!(enrollment.capabilities.is_empty());
        assert!(enrollment.operations[0].not_applicable_permitted.is_empty());
        assert!(record.execution.is_none());
        for (obligation, cases) in &mappings {
            assert!(!cases.is_empty(), "{obligation}");
            for case in cases {
                let leaf = case.name.rsplit("::").next().unwrap();
                let module = case.name.split("::").next().unwrap();
                assert!(
                    case.assertion.ends_with(&format!("/{module}.rs")),
                    "{}",
                    case.name
                );
                let source = std::fs::read_to_string(root.join(&case.assertion)).unwrap();
                assert!(source.contains(&format!("fn {leaf}()")), "{}", case.name);
                assert!(!case.production_inputs.is_empty() && !case.counterfactual.is_empty());
                for input in &case.production_inputs {
                    assert!(input.starts_with("crates/depth_fixture/src/"), "{input}");
                    assert!(root.join(input).is_file(), "{input}");
                }
            }
        }
        assert_eq!(
            p.evidence(),
            "cli_manifests/depth_fixture/reports/0.2.0/evidence/synthetic-wrapper-v1"
        );
    }

    #[test]
    fn ci_workflow_names_every_profile_step_and_artifact() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let ci = std::fs::read_to_string(root.join(".github/workflows/ci.yml")).unwrap();
        let profiles = [Profile::Channel, Profile::Core, Profile::Synthetic];
        for p in profiles {
            for step in [
                "Collect {} witness",
                "Upload {} witness",
                "Bind {} artifact",
            ] {
                let name = format!("- name: {}", step.replace("{}", p.step_label()));
                assert_eq!(ci.matches(&format!("{name}\n")).count(), 1, "{name}");
            }
            let artifact = format!(
                "{}-${{{{ github.run_id }}}}-${{{{ github.run_attempt }}}}",
                p.artifact()
            );
            assert_eq!(ci.matches(&artifact).count(), 1, "{artifact}");
        }
        assert!(ci.contains("run: cargo run -p xtask -- depth-synthetic-collect\n"));
        let labels: std::collections::BTreeSet<_> = profiles.map(Profile::step_label).into();
        assert_eq!(labels.len(), 3);
    }
}

#[cfg(test)]
mod onboarding_tests {
    use super::*;

    #[test]
    fn onboarding_reuses_canonical_collection_and_has_no_packet_admission_invocation() {
        let p = Profile::SyntheticOnboarding;
        let maintenance = Profile::Synthetic;
        assert_eq!(p.test(), maintenance.test());
        assert_eq!(p.package(), maintenance.package());
        assert_eq!(p.directory(), maintenance.directory());
        assert_eq!(p.contract(), maintenance.contract());
        assert_eq!(p.artifact(), maintenance.artifact());
        assert_eq!(p.step_label(), maintenance.step_label());
        assert_eq!(p.executor(), maintenance.executor());
        assert_eq!(p.trust(), maintenance.trust());
        assert_eq!(p.evidence(), maintenance.evidence());
        assert_ne!(p.fixture(), maintenance.fixture());
        assert_eq!(
            super::super::collector::environment(p),
            super::super::collector::environment(maintenance)
        );
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let ci = std::fs::read_to_string(root.join(".github/workflows/ci.yml")).unwrap();
        let producer = ci
            .split("\n  rust-tests:\n")
            .nth(1)
            .unwrap()
            .split("\n  agent-api-all-features:")
            .next()
            .unwrap();
        assert!(!producer.contains("needs:"));
        assert!(producer.contains("cargo test --workspace --all-targets"));
        assert!(producer.contains("run: cargo run -p xtask -- depth-synthetic-collect"));
        assert!(producer.contains("if: ${{ always() }}"));
        for forbidden in [
            "make preflight",
            "manifest-validate",
            "staging-integration-gate",
        ] {
            assert!(!producer.contains(forbidden), "{forbidden}");
        }
        // Collector recursion stops at one fixture executable. Its local build closure contains
        // the wrapper and event library, and has no xtask/admission dependency.
        let build: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(root.join(p.build_profile())).unwrap())
                .unwrap();
        for package in build["packages"].as_array().unwrap() {
            assert!(!package["id"].as_str().unwrap().contains("xtask"));
        }
        let collector = include_str!("collector.rs");
        assert!(collector.contains("profile.package(),"));
        assert!(collector.contains("profile.test(),"));
        assert!(collector.contains("let executable = executable.ok_or"));
        for forbidden in [
            "check_integration(",
            "depth_acceptance::",
            "manifest-validate",
            "make preflight",
        ] {
            assert!(!collector.contains(forbidden));
            for path in [
                "crates/depth_fixture/tests/depth_ci_synthetic.rs",
                "crates/depth_fixture/tests/depth_ci_synthetic/run.rs",
                "crates/depth_fixture/tests/depth_ci_synthetic/lifecycle.rs",
            ] {
                assert!(!std::fs::read_to_string(root.join(path))
                    .unwrap()
                    .contains(forbidden));
            }
        }
        // Workspace unit-test consumers use fresh repositories and scoped fake hosts. They
        // never resolve this CI run's packet. Phase C must still execute the real needs graph,
        // failure-preserving consumers and terminal pre-MERGE gate, including this test leg.
        let transport = include_str!("../depth_evidence/verdict/github/transport.rs");
        assert!(transport
            .contains("None => Err(\"no fake isolated host is installed for this test\".into())"));
    }
}
