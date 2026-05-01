mod storage;
mod validation;

use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use self::validation::{
    ensure_repo_relative_file_exists, landed_surface_name, validate_deferred_surfaces,
    validate_non_empty, validate_optional_path_pair, validate_optional_repo_relative_path,
    validate_pack_prefix, validate_path_hash_pair, validate_repo_relative_path,
    validate_required_publication_commands, validate_rfc3339, validate_schema_version,
    validate_sha256, validate_side_state_issues, validate_stage_field_presence,
    validate_stage_minimum_evidence, validate_string_list, validate_subset,
    validate_template_lineage, validate_unique_copy,
};
use crate::agent_registry::AgentRegistryEntry;

pub const LIFECYCLE_SCHEMA_VERSION: &str = "1";
pub const LIFECYCLE_DOCS_ROOT: &str = "docs/agents/lifecycle";
pub const GOVERNANCE_DIR_NAME: &str = "governance";
pub const LIFECYCLE_STATE_FILE_NAME: &str = "lifecycle-state.json";
pub const APPROVED_AGENT_FILE_NAME: &str = "approved-agent.toml";
pub const PUBLICATION_READY_FILE_NAME: &str = "publication-ready.json";
pub const PROVING_RUN_CLOSEOUT_FILE_NAME: &str = "proving-run-closeout.json";
pub const PUBLICATION_READY_NEXT_COMMAND: &str =
    "support-matrix --check && capability-matrix --check && capability-matrix-audit && make preflight && close-proving-run --write";

pub const REQUIRED_PUBLICATION_COMMANDS: [&str; 4] = [
    "cargo run -p xtask -- support-matrix --check",
    "cargo run -p xtask -- capability-matrix --check",
    "cargo run -p xtask -- capability-matrix-audit",
    "make preflight",
];

#[derive(Debug, Error)]
pub enum LifecycleError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Internal(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStage {
    Approved,
    Enrolled,
    RuntimeIntegrated,
    PublicationReady,
    Published,
    ClosedBaseline,
}

impl LifecycleStage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Enrolled => "enrolled",
            Self::RuntimeIntegrated => "runtime_integrated",
            Self::PublicationReady => "publication_ready",
            Self::Published => "published",
            Self::ClosedBaseline => "closed_baseline",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportTier {
    Bootstrap,
    BaselineRuntime,
    PublicationBacked,
    FirstClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SideState {
    Blocked,
    FailedRetryable,
    Drifted,
    Deprecated,
}

impl SideState {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Blocked => "blocked",
            Self::FailedRetryable => "failed_retryable",
            Self::Drifted => "drifted",
            Self::Deprecated => "deprecated",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeProfile {
    Minimal,
    Default,
    FeatureRich,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateId {
    Opencode,
    GeminiCli,
    Codex,
    ClaudeCode,
    Aider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LandedSurface {
    WrapperRuntime,
    BackendHarness,
    AgentApiOnboardingTest,
    WrapperCoverageSource,
    RuntimeManifestEvidence,
    AddDirs,
    ExternalSandboxPolicy,
    McpManagement,
    SessionResume,
    SessionFork,
    StructuredTools,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceId {
    RegistryEntry,
    DocsPack,
    ManifestRootSkeleton,
    RuntimeWriteComplete,
    ImplementationSummaryPresent,
    PublicationPacketWritten,
    SupportMatrixCheckGreen,
    CapabilityMatrixCheckGreen,
    CapabilityMatrixAuditGreen,
    PreflightGreen,
    ProvingRunCloseoutWritten,
    MaintenanceCloseoutWritten,
}

impl EvidenceId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RegistryEntry => "registry_entry",
            Self::DocsPack => "docs_pack",
            Self::ManifestRootSkeleton => "manifest_root_skeleton",
            Self::RuntimeWriteComplete => "runtime_write_complete",
            Self::ImplementationSummaryPresent => "implementation_summary_present",
            Self::PublicationPacketWritten => "publication_packet_written",
            Self::SupportMatrixCheckGreen => "support_matrix_check_green",
            Self::CapabilityMatrixCheckGreen => "capability_matrix_check_green",
            Self::CapabilityMatrixAuditGreen => "capability_matrix_audit_green",
            Self::PreflightGreen => "preflight_green",
            Self::ProvingRunCloseoutWritten => "proving_run_closeout_written",
            Self::MaintenanceCloseoutWritten => "maintenance_closeout_written",
        }
    }

    pub const fn all() -> &'static [Self] {
        &[
            Self::RegistryEntry,
            Self::DocsPack,
            Self::ManifestRootSkeleton,
            Self::RuntimeWriteComplete,
            Self::ImplementationSummaryPresent,
            Self::PublicationPacketWritten,
            Self::SupportMatrixCheckGreen,
            Self::CapabilityMatrixCheckGreen,
            Self::CapabilityMatrixAuditGreen,
            Self::PreflightGreen,
            Self::ProvingRunCloseoutWritten,
            Self::MaintenanceCloseoutWritten,
        ]
    }
}

const ENROLLED_MINIMUM_EVIDENCE: [EvidenceId; 3] = [
    EvidenceId::RegistryEntry,
    EvidenceId::DocsPack,
    EvidenceId::ManifestRootSkeleton,
];

const RUNTIME_INTEGRATED_MINIMUM_EVIDENCE: [EvidenceId; 5] = [
    EvidenceId::RegistryEntry,
    EvidenceId::DocsPack,
    EvidenceId::ManifestRootSkeleton,
    EvidenceId::RuntimeWriteComplete,
    EvidenceId::ImplementationSummaryPresent,
];

const PUBLICATION_READY_MINIMUM_EVIDENCE: [EvidenceId; 6] = [
    EvidenceId::RegistryEntry,
    EvidenceId::DocsPack,
    EvidenceId::ManifestRootSkeleton,
    EvidenceId::RuntimeWriteComplete,
    EvidenceId::ImplementationSummaryPresent,
    EvidenceId::PublicationPacketWritten,
];

const PUBLISHED_MINIMUM_EVIDENCE: [EvidenceId; 10] = [
    EvidenceId::RegistryEntry,
    EvidenceId::DocsPack,
    EvidenceId::ManifestRootSkeleton,
    EvidenceId::RuntimeWriteComplete,
    EvidenceId::ImplementationSummaryPresent,
    EvidenceId::PublicationPacketWritten,
    EvidenceId::SupportMatrixCheckGreen,
    EvidenceId::CapabilityMatrixCheckGreen,
    EvidenceId::CapabilityMatrixAuditGreen,
    EvidenceId::PreflightGreen,
];

const CLOSED_BASELINE_MINIMUM_EVIDENCE: [EvidenceId; 11] = [
    EvidenceId::RegistryEntry,
    EvidenceId::DocsPack,
    EvidenceId::ManifestRootSkeleton,
    EvidenceId::RuntimeWriteComplete,
    EvidenceId::ImplementationSummaryPresent,
    EvidenceId::PublicationPacketWritten,
    EvidenceId::SupportMatrixCheckGreen,
    EvidenceId::CapabilityMatrixCheckGreen,
    EvidenceId::CapabilityMatrixAuditGreen,
    EvidenceId::PreflightGreen,
    EvidenceId::ProvingRunCloseoutWritten,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeferredSurface {
    pub surface: LandedSurface,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImplementationSummary {
    pub requested_runtime_profile: RuntimeProfile,
    pub achieved_runtime_profile: RuntimeProfile,
    pub primary_template: TemplateId,
    pub template_lineage: Vec<String>,
    pub landed_surfaces: Vec<LandedSurface>,
    pub deferred_surfaces: Vec<DeferredSurface>,
    pub minimal_profile_justification: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleState {
    pub schema_version: String,
    pub agent_id: String,
    pub onboarding_pack_prefix: String,
    pub approval_artifact_path: String,
    pub approval_artifact_sha256: String,
    pub lifecycle_stage: LifecycleStage,
    pub support_tier: SupportTier,
    pub side_states: Vec<SideState>,
    pub current_owner_command: String,
    pub expected_next_command: String,
    pub last_transition_at: String,
    pub last_transition_by: String,
    pub required_evidence: Vec<EvidenceId>,
    pub satisfied_evidence: Vec<EvidenceId>,
    pub blocking_issues: Vec<String>,
    pub retryable_failures: Vec<String>,
    pub implementation_summary: Option<ImplementationSummary>,
    pub publication_packet_path: Option<String>,
    pub publication_packet_sha256: Option<String>,
    pub closeout_baseline_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicationReadyPacket {
    pub schema_version: String,
    pub agent_id: String,
    pub approval_artifact_path: String,
    pub approval_artifact_sha256: String,
    pub lifecycle_state_path: String,
    pub lifecycle_state_sha256: String,
    pub lifecycle_stage: LifecycleStage,
    pub support_tier_at_emit: SupportTier,
    pub manifest_root: String,
    pub expected_targets: Vec<String>,
    pub capability_publication_enabled: bool,
    pub support_publication_enabled: bool,
    pub capability_matrix_target: Option<String>,
    pub required_commands: Vec<String>,
    pub required_publication_outputs: Vec<String>,
    pub runtime_evidence_paths: Vec<String>,
    pub publication_owned_paths: Vec<String>,
    pub blocking_issues: Vec<String>,
    pub implementation_summary: ImplementationSummary,
}

impl LifecycleState {
    pub fn validate(&self) -> Result<(), LifecycleError> {
        validate_schema_version(&self.schema_version, "lifecycle-state.json")?;
        validate_non_empty("agent_id", &self.agent_id)?;
        validate_pack_prefix("onboarding_pack_prefix", &self.onboarding_pack_prefix)?;
        validate_repo_relative_path("approval_artifact_path", &self.approval_artifact_path)?;
        validate_sha256("approval_artifact_sha256", &self.approval_artifact_sha256)?;
        validate_stage_support_tier(self.lifecycle_stage, self.support_tier)?;
        validate_unique_copy("side_states", &self.side_states, SideState::as_str)?;
        validate_non_empty("current_owner_command", &self.current_owner_command)?;
        validate_non_empty("expected_next_command", &self.expected_next_command)?;
        validate_non_empty("last_transition_by", &self.last_transition_by)?;
        validate_rfc3339("last_transition_at", &self.last_transition_at)?;
        validate_unique_copy(
            "required_evidence",
            &self.required_evidence,
            EvidenceId::as_str,
        )?;
        validate_unique_copy(
            "satisfied_evidence",
            &self.satisfied_evidence,
            EvidenceId::as_str,
        )?;
        validate_stage_minimum_evidence(
            self.lifecycle_stage,
            "required_evidence",
            &self.required_evidence,
        )?;
        validate_stage_minimum_evidence(
            self.lifecycle_stage,
            "satisfied_evidence",
            &self.satisfied_evidence,
        )?;
        validate_subset(
            "satisfied_evidence",
            &self.satisfied_evidence,
            "required_evidence",
            &self.required_evidence,
            EvidenceId::as_str,
        )?;
        validate_string_list("blocking_issues", &self.blocking_issues)?;
        validate_string_list("retryable_failures", &self.retryable_failures)?;
        validate_side_state_issues(self)?;
        validate_optional_path_pair(
            "publication_packet_path",
            &self.publication_packet_path,
            "publication_packet_sha256",
            &self.publication_packet_sha256,
        )?;
        validate_optional_repo_relative_path(
            "closeout_baseline_path",
            &self.closeout_baseline_path,
        )?;
        validate_stage_field_presence(
            self.lifecycle_stage,
            "publication_packet_path",
            self.publication_packet_path.is_some(),
            &[LifecycleStage::Published, LifecycleStage::ClosedBaseline],
        )?;
        validate_stage_field_presence(
            self.lifecycle_stage,
            "publication_packet_sha256",
            self.publication_packet_sha256.is_some(),
            &[LifecycleStage::Published, LifecycleStage::ClosedBaseline],
        )?;
        validate_stage_field_presence(
            self.lifecycle_stage,
            "closeout_baseline_path",
            self.closeout_baseline_path.is_some(),
            &[LifecycleStage::ClosedBaseline],
        )?;

        match self.lifecycle_stage {
            LifecycleStage::Approved | LifecycleStage::Enrolled => {
                if self.implementation_summary.is_some() {
                    return Err(LifecycleError::Validation(
                        "implementation_summary must be null before runtime integration"
                            .to_string(),
                    ));
                }
            }
            LifecycleStage::RuntimeIntegrated
            | LifecycleStage::PublicationReady
            | LifecycleStage::Published
            | LifecycleStage::ClosedBaseline => {
                self.implementation_summary
                    .as_ref()
                    .ok_or_else(|| {
                        LifecycleError::Validation(
                            "implementation_summary is required at runtime_integrated and later stages"
                                .to_string(),
                        )
                    })?
                    .validate()?;
            }
        }

        Ok(())
    }

    pub fn validate_in_workspace(&self, workspace_root: &Path) -> Result<(), LifecycleError> {
        self.validate()?;
        validate_path_hash_pair(
            workspace_root,
            "approval_artifact_path",
            &self.approval_artifact_path,
            "approval_artifact_sha256",
            &self.approval_artifact_sha256,
        )?;
        if let (Some(path), Some(sha)) = (
            self.publication_packet_path.as_deref(),
            self.publication_packet_sha256.as_deref(),
        ) {
            validate_path_hash_pair(
                workspace_root,
                "publication_packet_path",
                path,
                "publication_packet_sha256",
                sha,
            )?;
        }
        if let Some(path) = &self.closeout_baseline_path {
            ensure_repo_relative_file_exists(workspace_root, "closeout_baseline_path", path)?;
        }
        Ok(())
    }
}

impl ImplementationSummary {
    pub fn validate(&self) -> Result<(), LifecycleError> {
        validate_template_lineage(&self.template_lineage)?;
        validate_unique_copy(
            "landed_surfaces",
            &self.landed_surfaces,
            landed_surface_name,
        )?;
        validate_deferred_surfaces(&self.deferred_surfaces)?;

        if matches!(self.requested_runtime_profile, RuntimeProfile::Minimal)
            || matches!(self.achieved_runtime_profile, RuntimeProfile::Minimal)
        {
            let justification = self
                .minimal_profile_justification
                .as_deref()
                .unwrap_or_default();
            if justification.trim().is_empty() {
                return Err(LifecycleError::Validation(
                    "minimal_profile_justification is required when either runtime profile is minimal"
                        .to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl PublicationReadyPacket {
    pub fn validate(&self) -> Result<(), LifecycleError> {
        validate_schema_version(&self.schema_version, "publication-ready.json")?;
        validate_non_empty("agent_id", &self.agent_id)?;
        validate_repo_relative_path("approval_artifact_path", &self.approval_artifact_path)?;
        validate_sha256("approval_artifact_sha256", &self.approval_artifact_sha256)?;
        validate_repo_relative_path("lifecycle_state_path", &self.lifecycle_state_path)?;
        validate_sha256("lifecycle_state_sha256", &self.lifecycle_state_sha256)?;

        if self.lifecycle_stage != LifecycleStage::PublicationReady {
            return Err(LifecycleError::Validation(format!(
                "publication-ready.json lifecycle_stage must be `publication_ready` (got `{}`)",
                self.lifecycle_stage.as_str()
            )));
        }
        if self.support_tier_at_emit != SupportTier::BaselineRuntime {
            return Err(LifecycleError::Validation(
                "publication-ready.json support_tier_at_emit must be `baseline_runtime`"
                    .to_string(),
            ));
        }

        validate_repo_relative_path("manifest_root", &self.manifest_root)?;
        validate_string_list("expected_targets", &self.expected_targets)?;
        validate_required_publication_commands(&self.required_commands)?;
        validate_string_list(
            "required_publication_outputs",
            &self.required_publication_outputs,
        )?;
        validate_string_list("runtime_evidence_paths", &self.runtime_evidence_paths)?;
        validate_string_list("publication_owned_paths", &self.publication_owned_paths)?;
        validate_string_list("blocking_issues", &self.blocking_issues)?;
        self.implementation_summary.validate()?;

        for (field, values) in [
            (
                "required_publication_outputs",
                &self.required_publication_outputs,
            ),
            ("runtime_evidence_paths", &self.runtime_evidence_paths),
            ("publication_owned_paths", &self.publication_owned_paths),
        ] {
            for value in values {
                validate_repo_relative_path(field, value)?;
            }
        }

        if let Some(target) = &self.capability_matrix_target {
            validate_non_empty("capability_matrix_target", target)?;
        }

        Ok(())
    }

    pub fn validate_in_workspace(&self, workspace_root: &Path) -> Result<(), LifecycleError> {
        self.validate()?;
        validate_path_hash_pair(
            workspace_root,
            "approval_artifact_path",
            &self.approval_artifact_path,
            "approval_artifact_sha256",
            &self.approval_artifact_sha256,
        )?;
        validate_path_hash_pair(
            workspace_root,
            "lifecycle_state_path",
            &self.lifecycle_state_path,
            "lifecycle_state_sha256",
            &self.lifecycle_state_sha256,
        )?;
        Ok(())
    }
}

pub fn lifecycle_state_path(pack_prefix: &str) -> String {
    format!("{LIFECYCLE_DOCS_ROOT}/{pack_prefix}/{GOVERNANCE_DIR_NAME}/{LIFECYCLE_STATE_FILE_NAME}")
}

pub fn publication_ready_path(pack_prefix: &str) -> String {
    format!(
        "{LIFECYCLE_DOCS_ROOT}/{pack_prefix}/{GOVERNANCE_DIR_NAME}/{PUBLICATION_READY_FILE_NAME}"
    )
}

pub fn approval_artifact_path(pack_prefix: &str) -> String {
    format!("{LIFECYCLE_DOCS_ROOT}/{pack_prefix}/{GOVERNANCE_DIR_NAME}/{APPROVED_AGENT_FILE_NAME}")
}

pub fn proving_run_closeout_path(pack_prefix: &str) -> String {
    format!(
        "{LIFECYCLE_DOCS_ROOT}/{pack_prefix}/{GOVERNANCE_DIR_NAME}/{PROVING_RUN_CLOSEOUT_FILE_NAME}"
    )
}

pub fn maintenance_request_path(agent_id: &str) -> String {
    format!(
        "{LIFECYCLE_DOCS_ROOT}/{agent_id}-maintenance/{GOVERNANCE_DIR_NAME}/maintenance-request.toml"
    )
}

pub fn lifecycle_state_path_for_entry(entry: &AgentRegistryEntry) -> String {
    lifecycle_state_path(&entry.scaffold.onboarding_pack_prefix)
}

pub fn publication_ready_path_for_entry(entry: &AgentRegistryEntry) -> String {
    publication_ready_path(&entry.scaffold.onboarding_pack_prefix)
}

pub fn approval_artifact_path_for_entry(entry: &AgentRegistryEntry) -> String {
    approval_artifact_path(&entry.scaffold.onboarding_pack_prefix)
}

pub fn required_evidence_for_stage(stage: LifecycleStage) -> &'static [EvidenceId] {
    match stage {
        LifecycleStage::Approved => &[],
        LifecycleStage::Enrolled => &ENROLLED_MINIMUM_EVIDENCE,
        LifecycleStage::RuntimeIntegrated => &RUNTIME_INTEGRATED_MINIMUM_EVIDENCE,
        LifecycleStage::PublicationReady => &PUBLICATION_READY_MINIMUM_EVIDENCE,
        LifecycleStage::Published => &PUBLISHED_MINIMUM_EVIDENCE,
        LifecycleStage::ClosedBaseline => &CLOSED_BASELINE_MINIMUM_EVIDENCE,
    }
}

pub fn validate_stage_support_tier(
    stage: LifecycleStage,
    tier: SupportTier,
) -> Result<(), LifecycleError> {
    let allowed = match stage {
        LifecycleStage::Approved | LifecycleStage::Enrolled => {
            matches!(tier, SupportTier::Bootstrap)
        }
        LifecycleStage::RuntimeIntegrated => {
            matches!(tier, SupportTier::Bootstrap | SupportTier::BaselineRuntime)
        }
        LifecycleStage::PublicationReady => matches!(tier, SupportTier::BaselineRuntime),
        LifecycleStage::Published | LifecycleStage::ClosedBaseline => {
            matches!(
                tier,
                SupportTier::PublicationBacked | SupportTier::FirstClass
            )
        }
    };

    if allowed {
        Ok(())
    } else {
        Err(LifecycleError::Validation(format!(
            "lifecycle_stage `{}` cannot pair with support_tier `{:?}`",
            stage.as_str(),
            tier
        )))
    }
}

pub fn is_resting_stage_v1(stage: LifecycleStage) -> bool {
    matches!(
        stage,
        LifecycleStage::Enrolled
            | LifecycleStage::RuntimeIntegrated
            | LifecycleStage::PublicationReady
            | LifecycleStage::ClosedBaseline
    )
}

pub fn reconstruct_publication_ready_state_from_closed_baseline(
    state: &LifecycleState,
) -> LifecycleState {
    let mut historical = state.clone();
    historical.lifecycle_stage = LifecycleStage::PublicationReady;
    historical.support_tier = SupportTier::BaselineRuntime;
    historical.current_owner_command = "prepare-publication --write".to_string();
    historical.expected_next_command = PUBLICATION_READY_NEXT_COMMAND.to_string();
    historical.required_evidence =
        required_evidence_for_stage(LifecycleStage::PublicationReady).to_vec();
    historical.satisfied_evidence =
        required_evidence_for_stage(LifecycleStage::PublicationReady).to_vec();
    historical.side_states.retain(|side_state| {
        !matches!(
            side_state,
            SideState::Blocked | SideState::FailedRetryable | SideState::Drifted
        )
    });
    historical.blocking_issues.clear();
    historical.retryable_failures.clear();
    historical.publication_packet_path = None;
    historical.publication_packet_sha256 = None;
    historical.closeout_baseline_path = None;
    historical
}

pub fn now_rfc3339() -> Result<String, LifecycleError> {
    storage::now_rfc3339()
}

pub fn file_sha256(workspace_root: &Path, relative_path: &str) -> Result<String, LifecycleError> {
    storage::file_sha256(workspace_root, relative_path)
}

pub fn write_lifecycle_state(
    workspace_root: &Path,
    relative_path: &str,
    state: &LifecycleState,
) -> Result<(), LifecycleError> {
    state.validate()?;
    storage::write_json_file(workspace_root, relative_path, state)
}

pub fn load_lifecycle_state(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<LifecycleState, LifecycleError> {
    let state: LifecycleState = storage::load_json_file(workspace_root, relative_path)?;
    state.validate_in_workspace(workspace_root)?;
    Ok(state)
}

pub fn write_publication_ready_packet(
    workspace_root: &Path,
    relative_path: &str,
    packet: &PublicationReadyPacket,
) -> Result<(), LifecycleError> {
    packet.validate()?;
    storage::write_json_file(workspace_root, relative_path, packet)
}

pub fn load_publication_ready_packet(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<PublicationReadyPacket, LifecycleError> {
    let packet: PublicationReadyPacket = storage::load_json_file(workspace_root, relative_path)?;
    packet.validate_in_workspace(workspace_root)?;
    Ok(packet)
}
