use std::path::Path;

use crate::workspace_mutation::{
    apply_mutations, plan_create_or_replace, PlannedMutation, WorkspacePathJail,
};
use crate::{
    agent_lifecycle::{self, load_lifecycle_state, write_lifecycle_state, EvidenceId, SideState},
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
    let jail = WorkspacePathJail::new(workspace_root)?;
    let handoff_path = linked.maintenance_pack_root.join("HANDOFF.md");
    let remediation_log_path = linked
        .maintenance_pack_root
        .join("governance/remediation-log.md");

    let closeout_bytes = serialize_closeout_json(&linked.closeout)?;
    let handoff_bytes = render_markdown_file(render_handoff_body(linked)).into_bytes();
    let remediation_log_bytes =
        render_markdown_file(render_remediation_log_body(linked)).into_bytes();

    Ok(vec![
        plan_create_or_replace(&jail, linked.closeout_path.clone(), closeout_bytes)?,
        plan_create_or_replace(&jail, handoff_path, handoff_bytes)?,
        plan_create_or_replace(&jail, remediation_log_path, remediation_log_bytes)?,
    ])
}

pub fn write_closeout_outputs(
    workspace_root: &Path,
    request_path: &Path,
    closeout_path: &Path,
) -> Result<CloseoutWriteSummary, MaintenanceCloseoutError> {
    let linked = super::load_linked_closeout(workspace_root, request_path, closeout_path)?;
    let mutations = plan_closeout_mutations(workspace_root, &linked)?;
    let apply = apply_mutations(workspace_root, &mutations)?;
    update_lifecycle_state_after_maintenance_closeout(workspace_root, &linked)?;
    Ok(CloseoutWriteSummary {
        agent_id: linked.request.agent_id.clone(),
        maintenance_pack_prefix: linked.maintenance_pack_prefix.clone(),
        request_path: linked.request_path.clone(),
        closeout_path: linked.closeout_path.clone(),
        apply,
    })
}

fn update_lifecycle_state_after_maintenance_closeout(
    workspace_root: &Path,
    linked: &LinkedMaintenanceCloseout,
) -> Result<(), MaintenanceCloseoutError> {
    let registry = AgentRegistry::load(workspace_root).map_err(|err| {
        MaintenanceCloseoutError::Validation(format!("load agent registry: {err}"))
    })?;
    let Some(entry) = registry.find(&linked.request.agent_id) else {
        return Ok(());
    };

    let lifecycle_state_path =
        agent_lifecycle::lifecycle_state_path(&entry.scaffold.onboarding_pack_prefix);
    let lifecycle_state_absolute = workspace_root.join(&lifecycle_state_path);
    if !lifecycle_state_absolute.is_file() {
        return Ok(());
    }

    let mut lifecycle_state = load_lifecycle_state(workspace_root, &lifecycle_state_path)
        .map_err(|err| MaintenanceCloseoutError::Validation(err.to_string()))?;
    lifecycle_state.current_owner_command = "close-agent-maintenance --write".to_string();
    lifecycle_state.expected_next_command =
        format!("check-agent-drift --agent {}", linked.request.agent_id);
    lifecycle_state.last_transition_at = agent_lifecycle::now_rfc3339()
        .map_err(|err| MaintenanceCloseoutError::Internal(err.to_string()))?;
    lifecycle_state.last_transition_by = "xtask close-agent-maintenance --write".to_string();
    lifecycle_state
        .side_states
        .retain(|state| !matches!(state, SideState::Drifted));
    lifecycle_state
        .satisfied_evidence
        .retain(|evidence| *evidence != EvidenceId::MaintenanceCloseoutWritten);
    lifecycle_state
        .satisfied_evidence
        .push(EvidenceId::MaintenanceCloseoutWritten);
    lifecycle_state.satisfied_evidence.sort();
    lifecycle_state.satisfied_evidence.dedup();

    write_lifecycle_state(workspace_root, &lifecycle_state_path, &lifecycle_state)
        .map_err(|err| MaintenanceCloseoutError::Internal(format!("write lifecycle state: {err}")))
}
