//! IN1 re-establishes ON4 authority from the pinned parent and exact candidate values.
use super::*;
use crate::{agent_lifecycle, depth_record::DepthRecord};

pub(super) fn candidate(
    view: &View,
    _record: &DepthRecord,
) -> Result<Option<super::super::onboarding::Kind>, String> {
    use super::super::onboarding::Kind;
    let path = agent_lifecycle::lifecycle_state_path_for_entry(&view.entry);
    let state: agent_lifecycle::LifecycleState =
        serde_json::from_str(&view.source.read(&path)?).map_err(|e| e.to_string())?;
    Ok(match state.lifecycle_stage {
        agent_lifecycle::LifecycleStage::ClosedBaseline => Some(Kind::Proving),
        agent_lifecycle::LifecycleStage::Published => Some(Kind::Publication),
        _ => None,
    })
}

pub(super) fn admit(
    root: &Path,
    old: &View,
    new: &View,
    parent: &Source,
    record: &DepthRecord,
    kind: super::super::onboarding::Kind,
) -> Result<(), String> {
    use crate::depth_acceptance::onboarding;
    let route = crate::isolated_maintenance::route(root)?
        .ok_or("unsupported onboarding lifecycle authority at integration")?;
    let captured = route.capture(root)?;
    let approval_path = agent_lifecycle::approval_artifact_path_for_entry(&new.entry);
    captured.onboarding_approval(parent.read(&approval_path)?.as_bytes(), &approval_path)?;
    let host = captured.evidence_host(root)?;
    let mut inputs = crate::git_inputs::GitInputs::new(root)?;
    let outcome = (|| {
        if kind == onboarding::Kind::Publication {
            let path = agent_lifecycle::lifecycle_state_path_for_entry(&new.entry);
            let previous: agent_lifecycle::LifecycleState =
                serde_json::from_str(&parent.read(&path)?).map_err(|e| e.to_string())?;
            if previous.lifecycle_stage == agent_lifecycle::LifecycleStage::Published {
                onboarding::on5::prepared_lifecycle(parent, &new.source, &new.entry)?;
            }
        }
        let expected = onboarding::acceptance(
            new,
            parent,
            record,
            kind,
            || crate::support_depth::current_onboarding_bindings(&mut inputs, &new.source, record),
            || crate::depth_evidence::EvidenceVerdicts::authenticate(root, record, &host),
        )?;
        if !record.acceptance_entries.contains(&expected) {
            return Err(match kind {
                onboarding::Kind::Publication => {
                    "integration publication lacks its exact current owning entry"
                }
                onboarding::Kind::Proving => {
                    "integration proving lacks its exact current owning entry"
                }
            }
            .into());
        }
        let previous = old.records.get(&record.policy.selection.enrollment.version);
        for entry in &record.acceptance_entries {
            if !previous.is_some_and(|prior| prior.acceptance_entries.contains(entry))
                && entry != &expected
            {
                return Err("new acceptance entry has no supported owning authority".into());
            }
        }
        onboarding::on5::runtime(root, &new.entry, parent, &captured)?;
        if kind == onboarding::Kind::Proving {
            let path = agent_lifecycle::proving_run_closeout_path(
                &new.entry.scaffold.onboarding_pack_prefix,
            );
            let value = new.json(&path)?;
            crate::agent_maintenance::closeout::require_commit_binding(
                root,
                super::super::input::text(&value, "commit")?,
            )
            .map_err(|e| e.to_string())?;
        }
        onboarding::check_effects(parent, &new.source, new, record, kind)
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
    kind: super::super::onboarding::Kind,
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
        if kind != super::super::onboarding::Kind::Publication {
            return Err("reporting publication changed its retained lifecycle assertion".into());
        }
        super::super::onboarding::on5::prepared_lifecycle(parent, &new.source, &new.entry)?;
    }
    // Reuse the structural binding without requiring a retained entry to describe changed E.
    super::super::onboarding::entry_reference(new, parent, record, kind)?;
    let captured = crate::isolated_maintenance::route(root)?
        .ok_or("reporting publication requires captured onboarding authority")?
        .capture(root)?;
    crate::prepare_publication::acquired_inputs::check(
        root,
        &agent_lifecycle::approval_artifact_path_for_entry(&new.entry),
    )?;
    super::super::onboarding::on5::runtime(root, &new.entry, parent, &captured)?;
    // Retain aggregate pointer, advertising, outside-generation and entry invariants.
    super::super::onboarding::check_effects(parent, &new.source, new, record, kind)?;
    captured.recheck(root)
}
