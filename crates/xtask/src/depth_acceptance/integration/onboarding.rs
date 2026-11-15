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
        runtime_evidence(root, new, parent, &captured)?;
        onboarding::check_effects(parent, &new.source, new, record, Kind::Publication)
    })();
    let input_check = inputs.verify();
    captured.recheck(root)?;
    input_check?;
    outcome
}

/// Retained entries do not require a new acceptance when every current claim is reporting-only.
pub(super) fn reporting(
    root: &Path,
    old: &View,
    new: &View,
    parent: &Source,
    record: &DepthRecord,
) -> Result<(), String> {
    let prior = old
        .records
        .get(&record.policy.selection.enrollment.version)
        .ok_or("reporting publication lacks its retained record")?;
    super::super::relay::preserved(prior, record)?;
    let inventory = crate::agent_maintenance::support_audit::NON_TUI_SUPPORT_DEBT_PATH;
    if parent.read(inventory)? != new.source.read(inventory)? {
        return Err("reporting publication changed its retained debt inventory".into());
    }
    let state_path = agent_lifecycle::lifecycle_state_path_for_entry(&new.entry);
    if parent.read(&state_path)? != new.source.read(&state_path)? {
        return Err("reporting publication changed its retained lifecycle assertion".into());
    }
    // Reuse the structural binding without requiring a retained entry to describe changed E.
    super::super::onboarding::entry_reference(
        new,
        parent,
        record,
        super::super::onboarding::Kind::Publication,
    )?;
    let captured = crate::isolated_maintenance::route(root)?
        .ok_or("reporting publication requires captured onboarding authority")?
        .capture(root)?;
    crate::prepare_publication::acquired_inputs::check(
        root,
        &agent_lifecycle::approval_artifact_path_for_entry(&new.entry),
    )?;
    runtime_evidence(root, new, parent, &captured)?;
    // Retain aggregate pointer, advertising, outside-generation and entry invariants.
    super::super::onboarding::check_effects(
        parent,
        &new.source,
        new,
        record,
        super::super::onboarding::Kind::Publication,
    )?;
    captured.recheck(root)
}

fn runtime_evidence(
    root: &Path,
    new: &View,
    parent: &Source,
    captured: &crate::isolated_maintenance::CapturedRegistry,
) -> Result<(), String> {
    let approval_path = agent_lifecycle::approval_artifact_path_for_entry(&new.entry);
    let approval = crate::runtime_follow_on::depth::approval::execution(
        root,
        parent.read(&approval_path)?.as_bytes(),
        &approval_path,
        captured,
    )?;
    let packet_path = agent_lifecycle::publication_ready_path_for_entry(&new.entry);
    let packet =
        serde_json::from_str(&new.source.read(&packet_path)?).map_err(|e| e.to_string())?;
    crate::prepare_publication::validate_packet_pinned_runtime_evidence_for_approval(
        root, &approval, &packet,
    )
    .map(drop)
    .map_err(|e| e.to_string())
}
