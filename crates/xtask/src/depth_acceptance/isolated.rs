//! The isolated owning host (CP-P1, G1). It runs the unchanged closeout and promotion predicates
//! with a borrowed, resolved isolated context in place of the production host. It never resolves
//! authority from a root, never trusts E labels and never reseals an earlier verdict set.
use super::manual;
use crate::{
    agent_registry::REGISTRY_RELATIVE_PATH,
    depth_evidence::EvidenceVerdicts,
    depth_record::{DepthRecord, ObligationsPart, PolicyPart},
    isolated_maintenance::Context,
};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Closeout,
    Promotion,
}

pub(crate) struct Host<'a> {
    context: &'a Context,
    kind: Kind,
}
impl<'a> Host<'a> {
    pub(crate) fn closeout(context: &'a Context) -> Self {
        Self {
            context,
            kind: Kind::Closeout,
        }
    }
    pub(crate) fn promotion(context: &'a Context) -> Self {
        Self {
            context,
            kind: Kind::Promotion,
        }
    }
}
impl manual::Host for Host<'_> {
    /// The production binding derivation, from the caller's Git inputs, plus a check that the
    /// registry it parses is exactly the context's scoped registry. The context is rechecked
    /// before and after, so the derivation cannot read a moved workspace.
    fn bindings(
        &self,
        root: &Path,
        record: &DepthRecord,
    ) -> Result<(PolicyPart, ObligationsPart), String> {
        let raw = crate::git_inputs::GitInputs::new(root)?.required(REGISTRY_RELATIVE_PATH)?;
        let raw = std::str::from_utf8(&raw).map_err(|e| e.to_string())?;
        self.context.require_registry(root, raw)?;
        let bindings = match self.kind {
            Kind::Closeout => crate::support_depth::current_closeout_bindings(root, record),
            Kind::Promotion => crate::support_depth::current_promotion_bindings(root, record),
        }?;
        self.context.recheck(root)?;
        Ok(bindings)
    }
    /// Fresh authentication of the exact `record` through the context's evidence host, on every
    /// call. The verdicts are sealed to `record` alone.
    fn evidence(&self, root: &Path, record: &DepthRecord) -> Result<EvidenceVerdicts, String> {
        let host = self.context.evidence_host(root)?;
        let verdicts = EvidenceVerdicts::authenticate(root, record, &host)?;
        self.context.recheck(root)?;
        Ok(verdicts)
    }
    /// The authenticated protected tip T. The local integration ref must equal it.
    fn integration_tip(
        &self,
        root: &Path,
        local: fn(&Path) -> Result<String, String>,
    ) -> Result<String, String> {
        let local = local(root)?;
        if local != self.context.protected_tip(root)? {
            return Err(
                "local integration ref differs from the authenticated protected tip".into(),
            );
        }
        Ok(local)
    }
}

#[cfg(test)]
impl Host<'_> {
    pub(crate) fn test_bindings(
        &self,
        root: &Path,
        record: &DepthRecord,
    ) -> Result<(PolicyPart, ObligationsPart), String> {
        manual::Host::bindings(self, root, record)
    }
    pub(crate) fn test_evidence(
        &self,
        root: &Path,
        record: &DepthRecord,
    ) -> Result<EvidenceVerdicts, String> {
        manual::Host::evidence(self, root, record)
    }
    pub(crate) fn test_tip(
        &self,
        root: &Path,
        local: fn(&Path) -> Result<String, String>,
    ) -> Result<String, String> {
        manual::Host::integration_tip(self, root, local)
    }
}
