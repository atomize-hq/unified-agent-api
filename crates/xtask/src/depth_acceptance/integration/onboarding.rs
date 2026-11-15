//! IN1 re-establishes ON4 authority from the pinned parent and exact candidate values.
use super::*;
use crate::{agent_lifecycle, depth_record::DepthRecord};

pub(super) fn publication_candidate(view: &View, record: &DepthRecord) -> Result<bool, String> {
    if !record
        .acceptance_entries
        .iter()
        .any(|entry| matches!(entry, AcceptanceEntry::Publication { .. }))
    {
        return Ok(false);
    }
    let path = agent_lifecycle::lifecycle_state_path_for_entry(&view.entry);
    let state: agent_lifecycle::LifecycleState =
        serde_json::from_str(&view.source.read(&path)?).map_err(|e| e.to_string())?;
    Ok(state.lifecycle_stage == agent_lifecycle::LifecycleStage::Published)
}

pub(super) fn admit(
    root: &Path,
    old: &View,
    new: &View,
    parent: &Source,
    record: &DepthRecord,
) -> Result<(), String> {
    use crate::depth_acceptance::onboarding::{self, Kind};
    let route = crate::isolated_maintenance::route(root)?
        .ok_or("unsupported onboarding lifecycle authority at integration")?;
    let captured = route.capture(root)?;
    let approval_path = agent_lifecycle::approval_artifact_path_for_entry(&new.entry);
    captured.onboarding_approval(parent.read(&approval_path)?.as_bytes(), &approval_path)?;
    let host = captured.evidence_host(root)?;
    let mut inputs = crate::git_inputs::GitInputs::new(root)?;
    let outcome = (|| {
        let expected = onboarding::acceptance(
            new,
            parent,
            record,
            Kind::Publication,
            || crate::support_depth::current_onboarding_bindings(&mut inputs, &new.source, record),
            || crate::depth_evidence::EvidenceVerdicts::authenticate(root, record, &host),
        )?;
        if !record.acceptance_entries.contains(&expected) {
            return Err("integration publication lacks its exact current owning entry".into());
        }
        let previous = old.records.get(&record.policy.selection.enrollment.version);
        for entry in &record.acceptance_entries {
            if !previous.is_some_and(|prior| prior.acceptance_entries.contains(entry))
                && entry != &expected
            {
                return Err("new acceptance entry has no supported owning authority".into());
            }
        }
        onboarding::check_effects(parent, &new.source, new, record, Kind::Publication)
    })();
    let input_check = inputs.verify();
    captured.recheck(root)?;
    input_check?;
    outcome
}
