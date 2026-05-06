use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawMaintenanceRequest {
    pub(super) artifact_version: String,
    pub(super) agent_id: String,
    pub(super) trigger_kind: String,
    pub(super) basis_ref: String,
    pub(super) opened_from: String,
    pub(super) requested_control_plane_actions: Vec<String>,
    pub(super) runtime_followup_required: RawRuntimeFollowupRequired,
    #[serde(default)]
    pub(super) detected_release: Option<RawDetectedRelease>,
    #[serde(default)]
    pub(super) execution_contract: Option<RawExecutionContract>,
    pub(super) request_recorded_at: String,
    pub(super) request_commit: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawRuntimeFollowupRequired {
    pub(super) required: bool,
    pub(super) items: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawDetectedRelease {
    pub(super) detected_by: String,
    pub(super) current_validated: String,
    pub(super) target_version: String,
    pub(super) latest_stable: String,
    pub(super) version_policy: String,
    pub(super) source_kind: String,
    pub(super) source_ref: String,
    pub(super) dispatch_kind: String,
    pub(super) dispatch_workflow: String,
    pub(super) branch_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawExecutionContract {
    pub(super) executor: String,
    pub(super) prompt_template_path: String,
    pub(super) prompt_sha256: String,
    pub(super) pr_summary_path: String,
    pub(super) closeout_path: String,
    pub(super) requires_manual_closeout: bool,
    pub(super) writable_surfaces: Vec<String>,
    pub(super) read_only_inputs: Vec<String>,
    pub(super) ordered_commands: Vec<String>,
    pub(super) green_gates: Vec<String>,
    pub(super) recovery: RawExecutionContractRecovery,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawExecutionContractRecovery {
    pub(super) recreate_packet_command: String,
    pub(super) reopen_pr_body_path: String,
    pub(super) reopen_pr_branch: String,
    pub(super) notes: Vec<String>,
}
