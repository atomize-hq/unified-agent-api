use std::path::Path;

use crate::workspace_mutation::{
    apply_mutations, plan_create_or_replace, PlannedMutation, WorkspacePathJail,
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
    Ok(CloseoutWriteSummary {
        agent_id: linked.request.agent_id.clone(),
        maintenance_pack_prefix: linked.maintenance_pack_prefix.clone(),
        request_path: linked.request_path.clone(),
        closeout_path: linked.closeout_path.clone(),
        apply,
    })
}
