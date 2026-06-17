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
    pub(super) support_surface_audit: Option<RawSupportSurfaceAudit>,
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
pub(super) struct RawSupportSurfaceAudit {
    pub(super) required: bool,
    pub(super) surface_kinds: Vec<String>,
    pub(super) excluded_surface_kinds: Vec<String>,
    pub(super) allowed_deferrals: Vec<String>,
    pub(super) pre_run_debt_count: usize,
    pub(super) expected_post_run_debt_count: usize,
    #[serde(default)]
    pub(super) discovered_upstream_surface: Vec<RawEvidenceBackedSurface>,
    #[serde(default)]
    pub(super) removed_upstream_surface: Vec<RawEvidenceBackedSurface>,
    #[serde(default)]
    pub(super) preexisting_unsupported_surface: Vec<RawDebtBackedSurface>,
    #[serde(default)]
    pub(super) eligible_preexisting_surface: Vec<RawEligibleSurface>,
    #[serde(default)]
    pub(super) missing_wrapper_support: Vec<RawSurfaceIdentity>,
    #[serde(default)]
    pub(super) missing_backend_support: Vec<RawSurfaceIdentity>,
    #[serde(default)]
    pub(super) required_uplifts_this_run: Vec<RawRequiredUplift>,
    #[serde(default)]
    pub(super) deferred_preexisting_gaps: Vec<RawDeferredGap>,
    #[serde(default)]
    pub(super) publication_impacts: Vec<RawPublicationImpact>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawSurfaceIdentity {
    pub(super) surface_kind: String,
    pub(super) command_path: String,
    pub(super) surface_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawEvidenceBackedSurface {
    pub(super) surface_kind: String,
    pub(super) command_path: String,
    pub(super) surface_id: String,
    pub(super) evidence_ref: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawDebtBackedSurface {
    pub(super) surface_kind: String,
    pub(super) command_path: String,
    pub(super) surface_id: String,
    pub(super) debt_ref: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawEligibleSurface {
    pub(super) surface_kind: String,
    pub(super) command_path: String,
    pub(super) surface_id: String,
    pub(super) eligibility_reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawRequiredUplift {
    pub(super) surface_kind: String,
    pub(super) command_path: String,
    pub(super) surface_id: String,
    pub(super) reason: String,
    pub(super) required_writes: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawDeferredGap {
    pub(super) surface_kind: String,
    pub(super) command_path: String,
    pub(super) surface_id: String,
    pub(super) defer_reason: String,
    #[serde(default)]
    pub(super) blocking_follow_on: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawPublicationImpact {
    pub(super) surface_kind: String,
    pub(super) command_path: String,
    pub(super) surface_id: String,
    pub(super) surface_doc: String,
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
