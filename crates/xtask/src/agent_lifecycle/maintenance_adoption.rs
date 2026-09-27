use std::{fs, path::Path};

use serde::Deserialize;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use super::{
    validation::{
        resolve_repo_relative_path, validate_optional_path_pair, validate_path_hash_pair,
    },
    EvidenceId, LifecycleError, LifecycleStage, LifecycleState,
};
use crate::agent_registry::{normalized_release_watch_sha256, AgentRegistry};

// This exact Claude approval was backfilled before maintenance settlement existed.
// The old approval, proving-run closeout, and publication packet stay immutable.
const APPROVAL_SHA256: &str = "9bd2dc39909499598791fcdd84a2c0960f68d70f167e05fc98264b5099397fc3";
const CLOSEOUT_SHA256: &str = "f11ea2a8dea65b4266135536ab30efac63614e3c4caadcb6a73e3730f97026b8";
const PUBLICATION_SHA256: &str = "bfbf90819b93c58eef1e44c3c2430c5525c1f3ffa2a0df847a48cde348cbea3c";
const APPROVAL_PATH: &str =
    "docs/agents/lifecycle/claude-code-cli-onboarding/governance/approved-agent.toml";
const CLOSEOUT_PATH: &str =
    "docs/agents/lifecycle/claude-code-cli-onboarding/governance/proving-run-closeout.json";
const PUBLICATION_PATH: &str =
    "docs/agents/lifecycle/claude-code-cli-onboarding/governance/publication-ready.json";
const ADOPTION_PATH: &str =
    "docs/agents/lifecycle/claude-code-cli-onboarding/governance/maintenance-readiness-adoption.json";
const HISTORICAL_AT: &str = "2026-02-12T09:34:04-05:00";

pub(super) fn is_historical_baseline(state: &LifecycleState) -> bool {
    state.lifecycle_stage == LifecycleStage::ClosedBaseline
        && state.agent_id == "claude_code"
        && state.onboarding_pack_prefix == "claude-code-cli-onboarding"
        && state.approval_artifact_path == APPROVAL_PATH
        && state.approval_artifact_sha256 == APPROVAL_SHA256
        && state.closeout_baseline_path.as_deref() == Some(CLOSEOUT_PATH)
        && state.last_transition_by == "historical-lifecycle-backfill"
        && state.last_transition_at == HISTORICAL_AT
        && !state
            .required_evidence
            .contains(&EvidenceId::MaintenanceReadinessSettled)
        && !state
            .satisfied_evidence
            .contains(&EvidenceId::MaintenanceReadinessSettled)
}

pub(super) fn validate_state_fields(state: &LifecycleState) -> Result<(), LifecycleError> {
    validate_optional_path_pair(
        "maintenance_readiness_adoption_path",
        &state.maintenance_readiness_adoption_path,
        "maintenance_readiness_adoption_sha256",
        &state.maintenance_readiness_adoption_sha256,
    )?;
    if let Some(path) = state.maintenance_readiness_adoption_path.as_deref() {
        if state.agent_id != "claude_code"
            || state.onboarding_pack_prefix != "claude-code-cli-onboarding"
            || state.approval_artifact_path != APPROVAL_PATH
            || state.approval_artifact_sha256 != APPROVAL_SHA256
            || state.lifecycle_stage != LifecycleStage::ClosedBaseline
            || path != ADOPTION_PATH
            || state.closeout_baseline_path.as_deref() != Some(CLOSEOUT_PATH)
            || state.publication_packet_path.as_deref() != Some(PUBLICATION_PATH)
            || state.publication_packet_sha256.as_deref() != Some(PUBLICATION_SHA256)
        {
            return Err(LifecycleError::Validation(
                "maintenance readiness adoption is limited to the historical Claude baseline"
                    .to_string(),
            ));
        }
    }
    if state.agent_id == "claude_code"
        && state.approval_artifact_sha256 == APPROVAL_SHA256
        && state
            .satisfied_evidence
            .contains(&EvidenceId::MaintenanceReadinessSettled)
        && state.maintenance_readiness_adoption_path.is_none()
    {
        return Err(LifecycleError::Validation(
            "historical Claude maintenance readiness requires a current adoption record"
                .to_string(),
        ));
    }
    Ok(())
}

pub(super) fn validate_workspace_link(
    workspace_root: &Path,
    state: &LifecycleState,
) -> Result<(), LifecycleError> {
    if let (Some(path), Some(sha)) = (
        state.maintenance_readiness_adoption_path.as_deref(),
        state.maintenance_readiness_adoption_sha256.as_deref(),
    ) {
        validate_path_hash_pair(
            workspace_root,
            "maintenance_readiness_adoption_path",
            path,
            "maintenance_readiness_adoption_sha256",
            sha,
        )?;
        validate(workspace_root, state, path)?;
    }
    Ok(())
}

pub(super) fn restore_historical_publication_transition(state: &mut LifecycleState) {
    if state.agent_id == "claude_code"
        && state.approval_artifact_sha256 == APPROVAL_SHA256
        && state.maintenance_readiness_adoption_path.is_some()
    {
        state.last_transition_at = HISTORICAL_AT.to_string();
        state.last_transition_by = "historical-lifecycle-backfill".to_string();
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Adoption {
    schema_version: String,
    agent_id: String,
    onboarding_pack_prefix: String,
    mode: String,
    approval_ref: String,
    approval_sha256: String,
    proving_run_closeout_ref: String,
    proving_run_closeout_sha256: String,
    publication_packet_ref: String,
    publication_packet_sha256: String,
    registry_release_watch_sha256: String,
    recorded_at: String,
    source_commit: String,
}

pub(super) fn validate(
    workspace_root: &Path,
    state: &LifecycleState,
    adoption_path: &str,
) -> Result<(), LifecycleError> {
    let absolute = resolve_repo_relative_path(workspace_root, adoption_path)?;
    let bytes = fs::read(&absolute).map_err(|err| {
        LifecycleError::Validation(format!("read maintenance readiness adoption: {err}"))
    })?;
    let adoption: Adoption = serde_json::from_slice(&bytes).map_err(|err| {
        LifecycleError::Validation(format!("parse maintenance readiness adoption: {err}"))
    })?;
    let closeout_ref = state.closeout_baseline_path.as_deref().ok_or_else(|| {
        LifecycleError::Validation(
            "maintenance readiness adoption requires a proving-run closeout".to_string(),
        )
    })?;
    let closeout_sha = super::file_sha256(workspace_root, closeout_ref)?;
    let closeout_bytes = fs::read(workspace_root.join(closeout_ref)).map_err(|err| {
        LifecycleError::Validation(format!("read historical proving-run closeout: {err}"))
    })?;
    let closeout: serde_json::Value = serde_json::from_slice(&closeout_bytes).map_err(|err| {
        LifecycleError::Validation(format!("parse historical proving-run closeout: {err}"))
    })?;
    let closeout_at = closeout
        .get("recorded_at")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            LifecycleError::Validation(
                "historical proving-run closeout has no recorded_at".to_string(),
            )
        })?;
    let adopted_at = OffsetDateTime::parse(&adoption.recorded_at, &Rfc3339).map_err(|err| {
        LifecycleError::Validation(format!("invalid adoption recorded_at: {err}"))
    })?;
    let historical_at = OffsetDateTime::parse(closeout_at, &Rfc3339).map_err(|err| {
        LifecycleError::Validation(format!("invalid historical closeout recorded_at: {err}"))
    })?;

    let registry = AgentRegistry::load(workspace_root)
        .map_err(|err| LifecycleError::Validation(format!("load adoption registry: {err}")))?;
    let entry = registry.find(&state.agent_id).ok_or_else(|| {
        LifecycleError::Validation("adoption agent is absent from registry".to_string())
    })?;
    let watch = entry.maintenance.release_watch.as_ref().ok_or_else(|| {
        LifecycleError::Validation("adoption requires enrolled release watch".to_string())
    })?;
    let watch_sha = normalized_release_watch_sha256(watch)
        .map_err(|err| LifecycleError::Validation(format!("normalize release watch: {err}")))?;

    if adoption.schema_version != "1"
        || adoption.agent_id != state.agent_id
        || adoption.onboarding_pack_prefix != state.onboarding_pack_prefix
        || adoption.mode != "release_watch_enrolled"
        || adoption.approval_ref != state.approval_artifact_path
        || adoption.approval_sha256 != state.approval_artifact_sha256
        || adoption.proving_run_closeout_ref != closeout_ref
        || adoption.proving_run_closeout_sha256 != closeout_sha
        || closeout_sha != CLOSEOUT_SHA256
        || Some(adoption.publication_packet_ref.as_str())
            != state.publication_packet_path.as_deref()
        || Some(adoption.publication_packet_sha256.as_str())
            != state.publication_packet_sha256.as_deref()
        || adoption.registry_release_watch_sha256 != watch_sha
        || closeout
            .get("approval_sha256")
            .and_then(serde_json::Value::as_str)
            != Some(state.approval_artifact_sha256.as_str())
        || adopted_at <= historical_at
        || state.last_transition_at != adoption.recorded_at
        || state.last_transition_by != "maintenance-readiness-adoption"
        || adoption.source_commit.len() != 40
        || !adoption
            .source_commit
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        || !state
            .required_evidence
            .contains(&EvidenceId::MaintenanceReadinessSettled)
        || !state
            .satisfied_evidence
            .contains(&EvidenceId::MaintenanceReadinessSettled)
    {
        return Err(LifecycleError::Validation(
            "maintenance readiness adoption does not match historical and current evidence"
                .to_string(),
        ));
    }
    Ok(())
}
