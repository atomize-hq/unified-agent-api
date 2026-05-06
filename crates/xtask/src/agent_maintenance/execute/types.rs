use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::agent_maintenance::{docs, request};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct InputContract {
    pub(super) workflow_version: String,
    pub(super) generated_at: String,
    pub(super) run_id: String,
    pub(super) request_path: String,
    pub(super) request_sha256: String,
    pub(super) maintenance_root: String,
    pub(super) agent_id: String,
    pub(super) target_version: String,
    pub(super) branch_name: String,
    pub(super) executor: String,
    pub(super) prompt_sha256: String,
    pub(super) closeout_path: String,
    pub(super) closeout_command: String,
    pub(super) writable_surfaces: Vec<String>,
    pub(super) read_only_inputs: Vec<String>,
    pub(super) ordered_commands: Vec<String>,
    pub(super) green_gates: Vec<String>,
    pub(super) recovery: RecoveryContract,
    pub(super) ignored_diff_roots: Vec<String>,
    pub(super) baseline: WorkspaceSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct RecoveryContract {
    pub(super) recreate_packet_command: String,
    pub(super) reopen_pr_body_path: String,
    pub(super) reopen_pr_branch: String,
    pub(super) notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ValidationCheck {
    pub(super) name: String,
    pub(super) ok: bool,
    pub(super) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ValidationReport {
    pub(super) workflow_version: String,
    pub(super) run_id: String,
    pub(super) status: String,
    pub(super) checks: Vec<ValidationCheck>,
    pub(super) errors: Vec<String>,
    pub(super) preflight: Option<SubprocessEvidence>,
    pub(super) codex_execution: Option<CodexExecutionEvidence>,
    pub(super) gate_results: Vec<GateEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct RunStatus {
    pub(super) workflow_version: String,
    pub(super) generated_at: String,
    pub(super) run_id: String,
    pub(super) host_surface: String,
    pub(super) mode: String,
    pub(super) status: String,
    pub(super) validation_passed: bool,
    pub(super) request_path: String,
    pub(super) packet_root: String,
    pub(super) agent_id: String,
    pub(super) target_version: String,
    pub(super) branch_name: String,
    pub(super) written_paths: Vec<String>,
    pub(super) errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct SnapshotFile {
    pub(super) path: String,
    pub(super) sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct WorkspaceSnapshot {
    pub(super) files: Vec<SnapshotFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct SubprocessEvidence {
    pub(super) binary: String,
    pub(super) argv: Vec<String>,
    pub(super) exit_code: i32,
    pub(super) stdout: String,
    pub(super) stderr: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct GateEvidence {
    pub(super) command: String,
    pub(super) exit_code: i32,
    pub(super) stdout: String,
    pub(super) stderr: String,
}

#[derive(Debug, Clone)]
pub(super) struct Context {
    pub(super) run_id: String,
    pub(super) codex_binary: String,
    pub(super) run_dir: PathBuf,
    pub(super) run_dir_rel: String,
    pub(super) envelope: request::MaintenanceRequestEnvelope,
    pub(super) execution_contract: request::ExecutionContract,
    pub(super) rendered_packet: docs::RenderedExecutionPacket,
    pub(super) closeout_command: String,
    pub(super) input_contract: Option<InputContract>,
}

#[derive(Debug, Clone)]
pub(super) struct PreflightResult {
    pub(super) evidence: SubprocessEvidence,
    pub(super) checks: Vec<ValidationCheck>,
    pub(super) errors: Vec<String>,
}
