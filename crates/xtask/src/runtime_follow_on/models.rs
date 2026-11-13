use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    agent_lifecycle::{LifecycleDepth, LifecycleState},
    approval_artifact::ApprovalArtifact,
};

/// Reporting cross-check of the approval's version and the lifecycle state's frozen identities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SupportDepthContract {
    pub(crate) version: String,
    pub(crate) policy_identity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) obligations_identity: Option<String>,
}

impl SupportDepthContract {
    /// Copy P/O from the lifecycle object; neither the contract nor repair freezes identities.
    pub(crate) fn copy_from(
        approval: &ApprovalArtifact,
        state: &LifecycleState,
    ) -> Result<Option<Self>, String> {
        let Some(version) = &approval.support_depth_version else {
            return Ok(None);
        };
        let object = state.support_depth.as_ref().ok_or_else(|| {
            "depth input contract requires the frozen lifecycle support_depth object; depth identity is not decided yet".to_string()
        })?;
        object.validate()?;
        Ok(Some(Self {
            version: version.clone(),
            policy_identity: object.policy_identity.clone(),
            obligations_identity: object.obligations_identity.clone(),
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct InputContract {
    pub(super) workflow_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) support_depth: Option<SupportDepthContract>,
    pub(super) generated_at: String,
    pub(super) run_id: String,
    pub(super) approval_artifact_path: String,
    pub(super) approval_artifact_sha256: String,
    pub(super) agent_id: String,
    pub(super) display_name: String,
    pub(super) crate_path: String,
    pub(super) backend_module: String,
    pub(super) manifest_root: String,
    pub(super) wrapper_coverage_source_path: String,
    pub(super) canonical_targets: Vec<String>,
    pub(super) always_on_capabilities: Vec<String>,
    pub(super) target_gated_capabilities: Vec<String>,
    pub(super) config_gated_capabilities: Vec<String>,
    pub(super) backend_extensions: Vec<String>,
    pub(super) support_matrix_enabled: bool,
    pub(super) capability_matrix_enabled: bool,
    pub(super) capability_matrix_target: Option<String>,
    pub(super) requested_tier: String,
    pub(super) minimal_justification_file: Option<String>,
    pub(super) minimal_justification_text: Option<String>,
    pub(super) allow_rich_surface: Vec<String>,
    pub(super) required_agent_api_test: String,
    pub(super) required_handoff_commands: Vec<String>,
    pub(super) docs_to_read: Vec<String>,
    pub(super) allowed_write_paths: Vec<String>,
    pub(super) ignored_diff_roots: Vec<String>,
    pub(super) baseline: WorkspaceSnapshot,
}

impl InputContract {
    pub(super) fn check_support_depth(
        &self,
        workspace: &Path,
        approval: &ApprovalArtifact,
        state: &LifecycleState,
    ) -> Result<(), super::Error> {
        use super::{io::read_string, Error, WORKFLOW_VERSION};
        if self.workflow_version != WORKFLOW_VERSION
            && !(self.workflow_version == "runtime_follow_on_v1" && self.support_depth.is_none())
        {
            return Err(Error::Validation(format!(
                "input contract workflow_version `{}` is unsupported for its support_depth identity",
                self.workflow_version
            )));
        }
        let expected =
            SupportDepthContract::copy_from(approval, state).map_err(Error::Validation)?;
        if self.support_depth != expected {
            return Err(Error::Validation(
                "input contract support_depth differs from the approval version or lifecycle support_depth object".into(),
            ));
        }
        let Some(depth) = &self.support_depth else {
            return Ok(());
        };
        let record = crate::depth_record::DepthRecord::parse(
            &read_string(&workspace.join(format!(
                "{}/reports/{}/depth-record.json",
                approval.descriptor.manifest_root, depth.version
            )))?,
            &approval.descriptor.agent_id,
            &depth.version,
        )
        .map_err(Error::Validation)?;
        let current_approval = crate::depth_record::onboarding::OnboardingApproval::parse(
            &read_string(&workspace.join(&approval.relative_path))?,
            &approval.relative_path,
        )
        .map_err(Error::Validation)?;
        if current_approval.support_depth_version.as_ref() != Some(&depth.version) {
            return Err(Error::Validation(
                "input contract support_depth version differs from the approval's [support_depth] table".into(),
            ));
        }
        let identities = LifecycleDepth {
            policy_identity: depth.policy_identity.clone(),
            obligations_identity: depth.obligations_identity.clone(),
        };
        if let Some(why) = record.request_mismatch(&identities.binding(current_approval.event)) {
            return Err(Error::Validation(format!(
                "input contract support_depth: {why}"
            )));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct WorkspaceSnapshot {
    pub(super) files: Vec<SnapshotFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct SnapshotFile {
    pub(super) path: String,
    pub(super) sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct RunStatus {
    pub(super) workflow_version: String,
    pub(super) generated_at: String,
    pub(super) run_id: String,
    pub(super) approval_artifact_path: String,
    pub(super) agent_id: String,
    pub(super) requested_tier: String,
    pub(super) host_surface: String,
    pub(super) loaded_skill_ref: String,
    pub(super) mode: String,
    pub(super) status: String,
    pub(super) validation_passed: bool,
    pub(super) handoff_ready: bool,
    pub(super) run_dir: String,
    pub(super) written_paths: Vec<String>,
    pub(super) errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ValidationReport {
    pub(super) workflow_version: String,
    pub(super) generated_at: String,
    pub(super) run_id: String,
    pub(super) status: String,
    pub(super) checks: Vec<ValidationCheck>,
    pub(super) errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ValidationCheck {
    pub(super) name: String,
    pub(super) ok: bool,
    pub(super) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct HandoffContract {
    pub(super) agent_id: String,
    pub(super) manifest_root: String,
    pub(super) runtime_lane_complete: bool,
    pub(super) publication_refresh_required: bool,
    pub(super) required_commands: Vec<String>,
    pub(super) blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct CodexExecutionEvidence {
    pub(super) workflow_version: String,
    pub(super) generated_at: String,
    pub(super) run_id: String,
    pub(super) binary: String,
    pub(super) argv: Vec<String>,
    pub(super) prompt_path: String,
    pub(super) stdout_path: String,
    pub(super) stderr_path: String,
    pub(super) exit_code: i32,
}

#[derive(Debug)]
pub(super) struct RuntimeContext {
    // Set only after the finite installed adapter validates the complete non-runtime descriptor.
    pub(super) wrapper_only: bool,
    pub(super) approval: ApprovalArtifact,
    pub(super) input_contract: InputContract,
    pub(super) lifecycle_state: LifecycleState,
    pub(super) lifecycle_state_path: String,
    pub(super) run_id: String,
    pub(super) run_dir: PathBuf,
}
