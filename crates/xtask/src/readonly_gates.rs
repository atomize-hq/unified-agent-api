//! OR-P13's read-only A checks. The controller owns delivery and the S/T inventory leaves.
use crate::{
    agent_maintenance::closeout_proposal::{IsolatedView, Proposal},
    depth_acceptance::onboarding::{self, Kind},
    depth_evidence::EvidenceVerdicts,
    depth_record::DepthRecord,
    isolated_maintenance::CapturedRegistry,
};
use std::path::Path;

#[cfg(test)]
pub(crate) mod test_hooks;

/// Slice 8 calls this after materializing its complete post-entry publication in scratch.
/// The caller holds the scratch registry binding. No verdict escapes the borrowed scope.
#[allow(dead_code)] // Slice 8 owns the ON4 call site.
pub(crate) fn check_proposed(
    caller: &Path,
    proposal: &Proposal,
    captured: &CapturedRegistry,
    view: &IsolatedView<'_>,
    expected: &DepthRecord,
) -> Result<(), String> {
    onboarding::evaluate_proposed(
        caller,
        proposal,
        captured,
        view,
        Kind::Publication,
        |record| {
            if record != expected {
                return Err("read-only gates require the exact proposed record".into());
            }
            check_shared(view.evaluation_source(caller)?.root.as_path())
        },
    )
}

/// Only explicit standalone checking selects fresh committed-candidate evaluation.
/// Ordinary generation continues to use production verdicts even with an installed route.
pub(crate) fn with_committed_evaluation<T>(
    root: &Path,
    check: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let Some(route) = crate::isolated_maintenance::route(root)? else {
        return check();
    };
    let captured = route.capture(root)?;
    let result = EvidenceVerdicts::with_integration_publication(root, &captured, check);
    // Closing observations apply on success, error and unchanged-input paths.
    captured.recheck(root)?;
    result
}

/// Registry-only standalone consumers authenticate and close their own route without
/// selecting evidence evaluation for ordinary inventory, rendering or owning callers.
pub(crate) fn with_readonly_route<T>(
    root: &Path,
    check: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let route = crate::isolated_maintenance::route(root)?;
    let result = check();
    if let Some(route) = route {
        route.recheck(root)?;
    }
    result
}

/// Real checking functions shared with the commands. The owner supplies fresh proposal or
/// committed evaluation. Running these functions creates no publication or admission output.
#[allow(dead_code)] // Used by check_proposed before the ON4 controller is connected.
pub(crate) fn check_shared(root: &Path) -> Result<(), String> {
    #[cfg(test)]
    let _restore = test_hooks::enter(root);
    // The three outer commands precede preflight, which repeats these A leaves.
    crate::support_matrix::check_publication(root)?;
    crate::capability_matrix::check_publication(root)?;
    crate::capability_publication::audit_current_capability_publication(root)?;
    let registry = crate::isolated_maintenance::registry(root).map_err(|e| e.to_string())?;
    let source = crate::runtime_advertising::source::Source::workspace(root)?;
    let inputs = crate::git_inputs::GitInputs::for_bound_evidence(root)?;
    let baseline = inputs.provenance("HEAD")?;
    for entry in &registry.agents {
        // Applicability comes from the registry inventory, independent of candidate presence.
        // These two reviewed exemptions match cli_manifests/{aider,gemini_cli}/VALIDATOR_SPEC.md
        // and support-depth-contract.md's Annex B adoption inventory. Any declaration, retained
        // record or partial prerequisite still requires full validation of an exempt root.
        let schema_free = matches!(
            (entry.agent_id.as_str(), entry.manifest_root.as_str()),
            ("aider", "cli_manifests/aider") | ("gemini_cli", "cli_manifests/gemini_cli")
        );
        let reports = format!("{}/reports", entry.manifest_root);
        let records = source.paths(&reports)?;
        let prior_records = baseline.paths(&reports)?;
        let mut validate = !schema_free
            || entry.support_depth.is_some()
            || records
                .iter()
                .chain(&prior_records)
                .any(|p| p.ends_with("/depth-record.json"));
        for name in ["RULES.json", "SCHEMA.json", "VERSION_METADATA_SCHEMA.json"] {
            // Pin the prerequisite selection to the captured committed source too. Deleting
            // every prerequisite from scratch cannot turn a required check into absence.
            validate |= baseline
                .optional_bytes(&format!("{}/{name}", entry.manifest_root))?
                .is_some();
            validate |= source
                .optional_bytes(&format!("{}/{name}", entry.manifest_root))?
                .is_some();
        }
        if validate {
            crate::manifest_validate::check_root(root.join(&entry.manifest_root))?;
        }
    }
    inputs.verify()?;
    crate::support_matrix::check_publication(root)?;
    crate::runtime_advertising::check_current(root)?;
    crate::capability_matrix::check_publication(root)?;
    crate::capability_publication::audit_current_capability_publication(root)
}
