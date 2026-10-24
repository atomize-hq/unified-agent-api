use std::path::Path;

use crate::workspace_mutation::{PlannedMutation, WorkspacePathJail};
use crate::{
    agent_lifecycle::{self, load_lifecycle_state, EvidenceId, SideState},
    agent_registry::AgentRegistry,
};

use super::{
    render::{
        render_handoff_body, render_markdown_file, render_remediation_log_body,
        serialize_closeout_json,
    },
    CloseoutWriteSummary, LinkedMaintenanceCloseout, MaintenanceCloseoutError,
};

pub fn plan_closeout_mutations(
    workspace_root: &Path,
    linked: &LinkedMaintenanceCloseout,
) -> Result<Vec<PlannedMutation>, MaintenanceCloseoutError> {
    let proposal = super::super::closeout_proposal::Proposal::capture(workspace_root)
        .map_err(MaintenanceCloseoutError::Validation)?;
    let files = closeout_files(workspace_root, linked)?;
    proposal
        .plan(workspace_root, files)
        .map_err(MaintenanceCloseoutError::Validation)
}

fn closeout_files(
    workspace_root: &Path,
    linked: &LinkedMaintenanceCloseout,
) -> Result<Vec<(std::path::PathBuf, Vec<u8>)>, MaintenanceCloseoutError> {
    let closeout_bytes = serialize_closeout_json(&linked.closeout)?;
    // A caller reaching the lower planner with a constructed LinkedMaintenanceCloseout must
    // satisfy the same canonical linkage and live truth checks as the file-addressed entry.
    let validated = super::load_linked_closeout_bytes(
        workspace_root,
        &linked.request_path,
        &linked.closeout_path,
        &closeout_bytes,
    )?;
    let linked = &validated;
    Ok(vec![
        (linked.closeout_path.clone(), closeout_bytes),
        (
            linked.maintenance_pack_root.join("HANDOFF.md"),
            render_markdown_file(render_handoff_body(linked)).into_bytes(),
        ),
        (
            linked
                .maintenance_pack_root
                .join("governance/remediation-log.md"),
            render_markdown_file(render_remediation_log_body(linked)).into_bytes(),
        ),
    ])
}

pub fn write_closeout_outputs(
    workspace_root: &Path,
    request_path: &Path,
    closeout_path: &Path,
) -> Result<CloseoutWriteSummary, MaintenanceCloseoutError> {
    let proposal = super::super::closeout_proposal::Proposal::capture(workspace_root)
        .map_err(MaintenanceCloseoutError::Validation)?;
    let linked = super::load_linked_closeout(workspace_root, request_path, closeout_path)?;
    let mut files = closeout_files(workspace_root, &linked)?;
    if let Some(file) = plan_lifecycle_state_after_maintenance_closeout(workspace_root, &linked)? {
        files.push(file);
    }
    let apply = proposal
        .apply(workspace_root, files)
        .map_err(MaintenanceCloseoutError::Validation)?;
    Ok(CloseoutWriteSummary {
        agent_id: linked.request.agent_id.clone(),
        maintenance_pack_prefix: linked.maintenance_pack_prefix.clone(),
        request_path: linked.request_path.clone(),
        closeout_path: linked.closeout_path.clone(),
        apply,
    })
}

fn plan_lifecycle_state_after_maintenance_closeout(
    workspace_root: &Path,
    linked: &LinkedMaintenanceCloseout,
) -> Result<Option<(std::path::PathBuf, Vec<u8>)>, MaintenanceCloseoutError> {
    let registry = AgentRegistry::load(workspace_root).map_err(|err| {
        MaintenanceCloseoutError::Validation(format!("load agent registry: {err}"))
    })?;
    let Some(entry) = registry.find(&linked.request.agent_id) else {
        return Ok(None);
    };

    let lifecycle_state_path =
        agent_lifecycle::lifecycle_state_path(&entry.scaffold.onboarding_pack_prefix);
    let lifecycle_state_absolute = workspace_root.join(&lifecycle_state_path);
    if !lifecycle_state_absolute.is_file() {
        return Ok(None);
    }

    let mut lifecycle_state = load_lifecycle_state(workspace_root, &lifecycle_state_path)
        .map_err(|err| MaintenanceCloseoutError::Validation(err.to_string()))?;
    lifecycle_state
        .side_states
        .retain(|state| !matches!(state, SideState::Drifted));
    lifecycle_state
        .required_evidence
        .retain(|evidence| *evidence != EvidenceId::MaintenanceCloseoutWritten);
    lifecycle_state
        .required_evidence
        .push(EvidenceId::MaintenanceCloseoutWritten);
    lifecycle_state.required_evidence.sort();
    lifecycle_state.required_evidence.dedup();
    lifecycle_state
        .satisfied_evidence
        .retain(|evidence| *evidence != EvidenceId::MaintenanceCloseoutWritten);
    lifecycle_state
        .satisfied_evidence
        .push(EvidenceId::MaintenanceCloseoutWritten);
    lifecycle_state.satisfied_evidence.sort();
    lifecycle_state.satisfied_evidence.dedup();

    lifecycle_state
        .validate_in_workspace(workspace_root)
        .map_err(|err| MaintenanceCloseoutError::Validation(err.to_string()))?;
    let mut bytes = serde_json::to_vec_pretty(&lifecycle_state).map_err(|err| {
        MaintenanceCloseoutError::Internal(format!("serialize lifecycle state: {err}"))
    })?;
    bytes.push(b'\n');
    WorkspacePathJail::new(workspace_root)?.resolve(Path::new(&lifecycle_state_path))?;
    Ok(Some((lifecycle_state_path.into(), bytes)))
}
