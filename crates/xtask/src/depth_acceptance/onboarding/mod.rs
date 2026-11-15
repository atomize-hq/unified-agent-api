//! ON4/ON5 owning predicates shared by controllers and independent IN1 admission.
//! Entry locators alone grant no authority. Later controllers must evaluate the coherent
//! post-entry proposal freshly, render under that evaluation and re-admit the aggregate batch.
#![allow(dead_code)] // Slices 8–9 connect these private cores to their owning controllers.
use super::{adequacy, artifacts, input::View};
use crate::{
    agent_lifecycle::{
        self, EvidenceId, LifecycleStage, LifecycleState, PublicationReadyPacket, SideState,
    },
    agent_registry::support_depth::LifecyclePath,
    depth_evidence::EvidenceVerdicts,
    depth_record::{
        onboarding::OnboardingApproval, AcceptanceEntry, DepthRecord, Event, FileRef,
        ObligationsPart, PolicyPart,
    },
    runtime_advertising::source::Source,
};

mod advertising;
mod binding;
mod effects;
pub(super) use effects::check as check_effects;
mod proposed;
#[allow(unused_imports)]
pub(crate) use proposed::{coherent_entry, evaluate_proposed};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Publication,
    Proving,
}

/// Structural exact-artifact locator only. It supplies no adequacy or write admission.
pub(super) fn entry_reference(
    view: &View,
    committed: &Source,
    record: &DepthRecord,
    kind: Kind,
) -> Result<AcceptanceEntry, String> {
    let state = binding::check(view, committed, record)?;
    let packet_path = agent_lifecycle::publication_ready_path_for_entry(&view.entry);
    let raw = view.source.read(&packet_path)?;
    let packet: PublicationReadyPacket =
        serde_json::from_value(super::input::json(&raw)?).map_err(|e| e.to_string())?;
    packet.validate().map_err(|e| e.to_string())?;
    let state_path = agent_lifecycle::lifecycle_state_path_for_entry(&view.entry);
    if packet.agent_id != view.entry.agent_id
        || packet.manifest_root != view.entry.manifest_root
        || packet.approval_artifact_path != state.approval_artifact_path
        || packet.approval_artifact_sha256 != state.approval_artifact_sha256
        || packet.lifecycle_state_path != state_path
        || state.publication_packet_path.as_ref() != Some(&packet_path)
        || state.publication_packet_sha256.as_deref() != Some(artifacts::digest(&raw).as_str())
    {
        return Err("onboarding publication is not bound to its exact packet".into());
    }
    let prior_state: LifecycleState =
        serde_json::from_str(&committed.read(&state_path)?).map_err(|e| e.to_string())?;
    if prior_state.lifecycle_stage == LifecycleStage::PublicationReady
        && artifacts::digest(&committed.read(&state_path)?) != packet.lifecycle_state_sha256
    {
        return Err("onboarding packet does not bind its publication_ready predecessor".into());
    }
    // The packet hashes the publication_ready predecessor, before ON4 advances the state.
    if committed.read(&packet_path)? != raw {
        return Err("onboarding publication packet differs from committed authority".into());
    }
    let digest = record.stated_sha256()?;
    match kind {
        Kind::Publication => {
            if !matches!(
                prior_state.lifecycle_stage,
                LifecycleStage::PublicationReady | LifecycleStage::Published
            ) || state.lifecycle_stage != LifecycleStage::Published
            {
                return Err("onboarding publication requires exact published assertion".into());
            }
            Ok(AcceptanceEntry::Publication {
                packet: FileRef {
                    path: packet_path,
                    sha256: artifacts::digest(&raw),
                },
                record_sha256: digest,
            })
        }
        Kind::Proving => {
            if !matches!(
                prior_state.lifecycle_stage,
                LifecycleStage::Published | LifecycleStage::ClosedBaseline
            ) || state.lifecycle_stage != LifecycleStage::ClosedBaseline
                || !state
                    .satisfied_evidence
                    .contains(&EvidenceId::ProvingRunCloseoutWritten)
            {
                return Err(
                    "onboarding proving requires closed_baseline and proving evidence id".into(),
                );
            }
            let path = format!(
                "docs/agents/lifecycle/{}/governance/proving-run-closeout.json",
                view.entry.scaffold.onboarding_pack_prefix
            );
            if state.closeout_baseline_path.as_ref() != Some(&path) {
                return Err("onboarding proving requires its exact closeout path".into());
            }
            binding::proving(view, &state, &path)?;
            Ok(AcceptanceEntry::ProvingRunCloseout {
                closeout: FileRef {
                    sha256: artifacts::digest(&view.source.read(&path)?),
                    path,
                },
                record_sha256: digest,
            })
        }
    }
}

pub(super) fn acceptance(
    view: &View,
    committed: &Source,
    record: &DepthRecord,
    kind: Kind,
    bindings: impl FnOnce() -> Result<(PolicyPart, ObligationsPart), String>,
    evidence: impl FnOnce() -> Result<EvidenceVerdicts, String>,
) -> Result<AcceptanceEntry, String> {
    let entry = entry_reference(view, committed, record, kind)?;
    adequacy::current(record, "onboarding", bindings)?;
    let execution = record
        .execution
        .as_ref()
        .ok_or("onboarding requires current E")?;
    if !record.policy.debt_baseline.is_empty()
        || !execution.debt_grants.is_empty()
        || !execution.debt_retired.is_empty()
    {
        return Err("onboarding debt grants, retirements and baseline debt are unsupported".into());
    }
    let inventory = crate::agent_maintenance::support_audit::NON_TUI_SUPPORT_DEBT_PATH;
    if view.source.read(inventory)? != committed.read(inventory)? {
        return Err("onboarding debt inventory drift is unsupported".into());
    }
    let rows = crate::agent_maintenance::support_audit::authorization::parse_debt_inventory(
        &view.source.read(inventory)?,
        std::path::Path::new(inventory),
    )?;
    if rows.iter().any(|row| row.agent_id == view.entry.agent_id) {
        return Err("onboarding requires a debt-free baseline and inventory".into());
    }
    adequacy::evidence(view, record, &evidence()?, "onboarding")?;
    Ok(entry)
}

#[cfg(test)]
pub(crate) mod tests;
