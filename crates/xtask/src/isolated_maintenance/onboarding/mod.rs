//! Finite pre-enrollment adapter. Its configuration is compiled; checkpoints grant no freeze.
pub(crate) mod approval;
mod checkpoint;
mod creation;
mod encoding;
mod inputs;
mod raw_help;
mod scaffold;
mod view;
mod wrapper;

use super::*;
use crate::{
    agent_registry::{AgentRegistryEntry, REGISTRY_RELATIVE_PATH},
    depth_record::onboarding::OnboardingApproval,
};
pub(crate) use checkpoint::Checkpoint;
pub(crate) use scaffold::{reporting_targets, Scaffold};

const SCAFFOLD: &str =
    include_str!("../../../tests/fixtures/depth-synthetic/onboarding/registry-entry-scaffold.toml");
const DECLARED: &str =
    include_str!("../../../tests/fixtures/depth-synthetic/onboarding/registry-entry-deferred.toml");
const STATEMENT: &str = include_str!(
    "../../../tests/fixtures/depth-synthetic/onboarding/governance/depth-acquisition/0.2.0.toml"
);

/// Private profile data, never derived from a caller's descriptor or registry.
pub(super) struct Adapter {
    scaffold: AgentRegistryEntry,
    declared: AgentRegistryEntry,
    version: &'static str,
    subject: &'static scope::Subject,
    repository: Option<&'static str>,
    wrapper_rules: &'static str,
}
impl Profile {
    fn onboarding(&self) -> Result<Option<Adapter>, String> {
        // Profile selection belongs here, never in shared effect or integration predicates.
        if *self.subject != scope::SYNTHETIC {
            return Ok(None);
        }
        let mut adapter = Adapter::synthetic()?;
        adapter.repository = Some(self.isolated.name);
        Ok(Some(adapter))
    }
}
impl Adapter {
    fn synthetic() -> Result<Self, String> {
        let entry = |raw| {
            AgentRegistry::parse_unenforced(raw)
                .map_err(|e| e.to_string())?
                .agents
                .into_iter()
                .next()
                .ok_or_else(|| "empty compiled onboarding template".to_string())
        };
        Ok(Self {
            scaffold: entry(SCAFFOLD)?,
            declared: entry(DECLARED)?,
            version: scope::SYNTHETIC.version,
            subject: &scope::SYNTHETIC,
            repository: None,
            wrapper_rules: include_str!(
                "../../../tests/fixtures/depth-synthetic/governing/manifest-root/RULES.json"
            ),
        })
    }
    /// Expected governing bytes live with the compiled descriptor, never in shared IN1 logic.
    fn governing(&self) -> Result<Vec<(&'static str, String)>, String> {
        let templates = [
            ("RULES.json", include_str!("../../../tests/fixtures/depth-synthetic/governing/manifest-root/RULES.json")),
            ("SCHEMA.json", include_str!("../../../tests/fixtures/depth-synthetic/governing/manifest-root/SCHEMA.json")),
            ("VERSION_METADATA_SCHEMA.json", include_str!("../../../tests/fixtures/depth-synthetic/governing/manifest-root/VERSION_METADATA_SCHEMA.json")),
            ("VALIDATOR_SPEC.md", include_str!("../../../tests/fixtures/depth-synthetic/governing/manifest-root/VALIDATOR_SPEC.md")),
        ];
        let (owner, repository) = self
            .repository
            .ok_or("missing compiled repository")?
            .split_once('/')
            .ok_or("invalid compiled repository")?;
        let entry = &self.scaffold;
        let targets = serde_json::to_string(&entry.canonical_targets).map_err(|e| e.to_string())?;
        // The compiled wrapper profile has one Linux GNU target and this CLI tool identity.
        let mapping = serde_json::json!({self.subject.target: "linux"}).to_string();
        Ok(templates
            .into_iter()
            .map(|(name, template)| {
                (
                    name,
                    template
                        .replace("\"{{EXPECTED_TARGETS}}\"", &targets)
                        .replace("\"{{PLATFORM_MAPPING}}\"", &mapping)
                        .replace("{{ISOLATED_OWNER}}", owner)
                        .replace("{{ISOLATED_REPO}}", repository)
                        .replace("{{AGENT_ID}}", &entry.agent_id)
                        .replace("{{DISPLAY_NAME}}", &entry.display_name)
                        .replace("{{MANIFEST_ROOT}}", &entry.manifest_root)
                        .replace("{{TOOL_NAME}}", "depth-fixture-cli")
                        .replace("{{REQUIRED_TARGET}}", self.subject.target)
                        .replace("{{REQUIRED_PLATFORM}}", "linux"),
                )
            })
            .collect())
    }
    fn acquired_paths(&self) -> Result<Vec<String>, String> {
        let statement = crate::depth_record::acquisition::AcquisitionStatement::parse(
            &STATEMENT.replace("{{SHA256}}", &"a".repeat(64)),
            &self.scaffold.agent_id,
            &format!("{}.toml", self.version),
        )?;
        Ok(statement.inputs.into_iter().map(|i| i.path).collect())
    }
    fn approval_path(&self) -> String {
        format!(
            "docs/agents/lifecycle/{}/governance/approved-agent.toml",
            self.scaffold.scaffold.onboarding_pack_prefix
        )
    }
    fn state_path(&self) -> String {
        crate::agent_lifecycle::lifecycle_state_path(&self.scaffold.scaffold.onboarding_pack_prefix)
    }
    fn read_approval(&self, source: &Source) -> Result<(Vec<u8>, OnboardingApproval), String> {
        let path = self.approval_path();
        let raw = source
            .optional_bytes(&path)?
            .ok_or("bootstrap approval is absent")?;
        let approval = approval::read(&raw, &path, &self.scaffold)?;
        if approval.support_depth_version.as_deref() != Some(self.version) {
            return Err("bootstrap approval version differs from compiled selection".into());
        }
        Ok((raw, approval))
    }
    fn undeclared(&self, registry: &AgentRegistry) -> bool {
        registry
            .find(&self.scaffold.agent_id)
            .map_or(true, |e| e.support_depth.is_none())
    }
    /// Only the ordinary absent->scaffold delta or the separate exact declaration is possible.
    fn registry_delta(&self, old: &AgentRegistry, new: &AgentRegistry) -> Result<(), String> {
        let id = &self.scaffold.agent_id;
        if old.find(id).is_some_and(|e| e != &self.scaffold) {
            return Err("bootstrap protected entry is not the exact unenrolled scaffold".into());
        }
        let mut expected = old.clone();
        match new.find(id) {
            None if old.find(id).is_none() => {}
            Some(entry) if entry == &self.scaffold => {
                if old.find(id).is_none() {
                    expected.agents.push(self.scaffold.clone());
                }
            }
            Some(entry) if old.find(id).is_some() => {
                let mut declaration = entry.clone();
                // Reviewed CORE rows/basis are the only variable part of the compiled selection.
                if let Some(depth) = &mut declaration.support_depth {
                    for enrollment in depth.enrollments.iter_mut().flatten() {
                        scope::selection(enrollment, self.subject)?;
                        for operation in &mut enrollment.operations {
                            operation.classification_basis = None;
                            operation.classifications = None;
                        }
                    }
                }
                if declaration != self.declared {
                    return Err(
                        "first declaration differs from compiled onboarding selection".into(),
                    );
                }
                let index = expected
                    .agents
                    .iter()
                    .position(|e| &e.agent_id == id)
                    .unwrap();
                expected.agents[index] = entry.clone();
            }
            _ => {
                return Err("bootstrap requires the exact absent-to-scaffold registry delta".into())
            }
        }
        if expected != *new {
            return Err("bootstrap changed unrelated registry authority".into());
        }
        Ok(())
    }
}

/// Scope may read a pending candidate, but it captures only the protected registry. The two
/// registry exceptions never change scratch equality or turn an approval into an enrollment.
pub(super) fn load(
    root: &Path,
    inputs: &mut GitInputs,
    protected: &Source,
    old: &AgentRegistry,
    new: &AgentRegistry,
    subject: &scope::Subject,
) -> Result<Option<AgentRegistry>, String> {
    if *subject != scope::SYNTHETIC {
        return Ok(None);
    }
    let adapter = Adapter::synthetic()?;
    if !adapter.undeclared(old)
        || old
            .find(&adapter.scaffold.agent_id)
            .is_some_and(|entry| entry != &adapter.scaffold)
    {
        // Unselected maintenance entries keep their existing missing-declaration refusal.
        return Ok(None);
    }
    adapter.registry_delta(old, new)?;
    let source = Source::workspace(root)?;
    let (raw, _) = adapter.read_approval(protected)?;
    if inputs.required(&adapter.approval_path())? != raw
        || inputs.executable(&adapter.approval_path())?
        || protected.executable(&adapter.approval_path())?
    {
        return Err("bootstrap approval changed".into());
    }
    scope::no_records(root, inputs)?;
    inputs::record_free(
        &source,
        &adapter,
        new.find(&adapter.scaffold.agent_id).is_none(),
    )?;
    Ok(Some(old.clone()))
}

impl Context {
    /// A declared route is onboarding only by its captured subject declaration, never by
    /// missing registry data in IN1's parent. Scope loading validates its single enrollment.
    pub(crate) fn bootstrap_declared_onboarding(&self) -> bool {
        self.registry
            .find(self.profile.subject.agent_id)
            .and_then(|entry| entry.support_depth.as_ref())
            .and_then(|depth| depth.enrollments.as_ref())
            .is_some_and(|enrollments| {
                enrollments[0].lifecycle_path
                    == crate::agent_registry::support_depth::LifecyclePath::Onboarding
            })
    }

    /// Dispatch from the captured protected registry, without probing IN1's parent or refs.
    /// Scope loading retains that registry for bootstrap; declared routes keep their order.
    pub(crate) fn bootstrap_pending(&self) -> Result<bool, String> {
        Ok(self
            .profile
            .onboarding()?
            .is_some_and(|adapter| adapter.undeclared(&self.registry)))
    }

    /// IN1 alone uses this after authenticating candidate/parent/tip. No reusable admission token.
    pub(crate) fn bootstrap_checkpoint(
        &self,
        root: &Path,
        parent: &str,
    ) -> Result<Option<Checkpoint>, String> {
        self.recheck(root)?;
        let Some(adapter) = self.profile.onboarding()? else {
            return Ok(None);
        };
        let before = Source::tip(root, parent)?;
        let old = AgentRegistry::parse_unenforced(&before.read(REGISTRY_RELATIVE_PATH)?)
            .map_err(|e| e.to_string())?;
        if !adapter.undeclared(&old) {
            return Ok(None);
        }
        // Use the captured candidate object, never a second lookup of mutable HEAD. IN1's
        // binding has already authenticated this checkout and its pinned parent.
        let after = Source::tip(root, &self.workspace.head)?;
        let checkpoint =
            checkpoint::check(root, parent, before, after, &self.workspace.head, adapter)?;
        self.recheck(root)?;
        Ok(Some(checkpoint))
    }
}
