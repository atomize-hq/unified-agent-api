use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::approval_artifact::ApprovalArtifact;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct InputContract {
    pub(super) workflow_version: String,
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
    pub(super) approval: ApprovalArtifact,
    pub(super) input_contract: InputContract,
    pub(super) run_id: String,
    pub(super) run_dir: PathBuf,
}
