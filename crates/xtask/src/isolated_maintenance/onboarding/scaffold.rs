//! Compiled bootstrap inputs shared by public ON1 and independent creation replay.
use super::*;

pub(crate) struct Scaffold {
    pub(crate) entry: AgentRegistryEntry,
    pub(crate) approval_path: String,
    pub(crate) approval_bytes: Vec<u8>,
}

impl Adapter {
    pub(super) fn scaffold(&self, source: &Source) -> Result<Scaffold, String> {
        let (approval_bytes, _) = self.read_approval(source)?;
        Ok(Scaffold {
            entry: self.scaffold.clone(),
            approval_path: self.approval_path(),
            approval_bytes,
        })
    }
}

impl CapturedRegistry {
    pub(crate) fn onboarding_scaffold(&self, root: &Path) -> Result<Option<Scaffold>, String> {
        self.recheck(root)?;
        let context = self.route.context();
        let Some(adapter) = context.profile.onboarding()? else {
            return Ok(None);
        };
        if !adapter.undeclared(&context.registry) {
            return Ok(None);
        }
        let scaffold = adapter.scaffold(&Source::workspace(root)?)?;
        self.recheck(root)?;
        Ok(Some(scaffold))
    }
}

/// The effect detector must read ON1's exact empty current after the first record appears.
/// This finite read projection grants no registry loading or write authority.
pub(crate) fn reporting_targets(
    entry: &AgentRegistryEntry,
    raw: &str,
) -> Result<Option<Vec<String>>, String> {
    if entry
        .support_depth
        .as_ref()
        .and_then(|d| d.enabled_paths.as_deref())
        != Some(&[crate::agent_registry::support_depth::LifecyclePath::Onboarding])
    {
        return Ok(None);
    }
    let mut adapter = Adapter::synthetic()?;
    if entry.agent_id != adapter.scaffold.agent_id {
        return Ok(None);
    }
    // The captured registry authenticates this optional charter rule 12 settlement.
    // Keep every other compiled descriptor and selection check in registry_delta.
    adapter.declared.maintenance.release_watch = entry.maintenance.release_watch.clone();
    let old = AgentRegistry {
        agents: vec![adapter.scaffold.clone()],
    };
    let new = AgentRegistry {
        agents: vec![entry.clone()],
    };
    // Reuse the complete finite entry comparison, including reviewed classification fields.
    if adapter.registry_delta(&old, &new).is_err() {
        return Ok(None);
    }
    if raw != crate::onboard_agent::render_current_json(&adapter.scaffold.canonical_targets) {
        return Ok(None);
    }
    Ok(Some(adapter.scaffold.canonical_targets))
}

impl CapturedRegistry {
    /// Declared ON readers use the same compiled descriptor as bootstrap, with the optional
    /// reviewed release-watch settlement already authenticated by the captured registry.
    pub(crate) fn onboarding_approval(
        &self,
        raw: &[u8],
        path: &str,
    ) -> Result<OnboardingApproval, String> {
        let context = self.route.context();
        let adapter = context
            .profile
            .onboarding()?
            .ok_or("only the synthetic subject has an onboarding adapter")?;
        let mut expected = adapter.scaffold.clone();
        let declared = self
            .registry()
            .find(&expected.agent_id)
            .ok_or("unknown approval agent")?;
        expected.maintenance = declared.maintenance.clone();
        let approval = approval::read(raw, path, &expected)?;
        if approval.support_depth_version.as_deref() != Some(adapter.version) {
            return Err(
                "charter rule 3 unresolved: approval version differs from compiled selection"
                    .into(),
            );
        }
        Ok(approval)
    }
}
