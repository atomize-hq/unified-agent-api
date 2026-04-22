use std::{
    fmt,
    path::{Path, PathBuf},
};

use clap::Parser;

use crate::workspace_mutation::{ApplySummary, WorkspaceMutationError};

#[derive(Debug, Parser, Clone)]
pub struct Args {
    #[arg(long)]
    pub request: PathBuf,

    #[arg(long)]
    pub closeout: PathBuf,
}

#[derive(Debug, Clone)]
pub struct MaintenanceRequest {
    pub agent_id: String,
    pub trigger_kind: MaintenanceTriggerKind,
    pub basis_ref: String,
    pub opened_from: String,
    pub requested_control_plane_actions: Vec<MaintenanceControlPlaneAction>,
    pub runtime_followup_required: RuntimeFollowupRequired,
    pub request_recorded_at: String,
    pub request_commit: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaintenanceTriggerKind {
    DriftDetected,
    ManualReopen,
    PostReleaseAudit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaintenanceControlPlaneAction {
    PacketDocRefresh,
    SupportMatrixRefresh,
    CapabilityMatrixRefresh,
    ReleaseDocRefresh,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFollowupRequired {
    pub required: bool,
    pub items: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MaintenanceCloseout {
    pub request_ref: String,
    pub request_sha256: String,
    pub resolved_findings: Vec<MaintenanceFinding>,
    pub deferred_findings: DeferredFindingsTruth,
    pub preflight_passed: bool,
    pub recorded_at: String,
    pub commit: String,
}

#[derive(Debug, Clone)]
pub enum DeferredFindingsTruth {
    Findings(Vec<MaintenanceFinding>),
    ExplicitNone(String),
}

#[derive(Debug, Clone)]
pub struct MaintenanceFinding {
    pub category_id: MaintenanceDriftCategory,
    pub summary: String,
    pub surfaces: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaintenanceDriftCategory {
    RegistryManifest,
    CapabilityPublication,
    SupportPublication,
    ReleaseDoc,
    GovernanceDoc,
}

#[derive(Debug, Clone)]
pub struct LinkedMaintenanceCloseout {
    pub loaded_request: LoadedMaintenanceRequest,
    pub request_path: PathBuf,
    pub closeout_path: PathBuf,
    pub maintenance_pack_prefix: String,
    pub maintenance_pack_root: PathBuf,
    pub request_sha256: String,
    pub request: MaintenanceRequest,
    pub closeout: MaintenanceCloseout,
}

#[derive(Debug, Clone)]
pub struct LoadedMaintenanceRequest {
    pub request_path: PathBuf,
    pub maintenance_pack_prefix: String,
    pub maintenance_pack_root: PathBuf,
    pub request_sha256: String,
    pub request: MaintenanceRequest,
}

#[derive(Debug, Clone)]
pub struct CloseoutWriteSummary {
    pub agent_id: String,
    pub maintenance_pack_prefix: String,
    pub request_path: PathBuf,
    pub closeout_path: PathBuf,
    pub apply: ApplySummary,
}

#[derive(Debug)]
pub enum MaintenanceCloseoutError {
    Validation(String),
    Internal(String),
}

impl fmt::Display for MaintenanceCloseoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(message) | Self::Internal(message) => f.write_str(message),
        }
    }
}

impl MaintenanceCloseoutError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation(_) => 2,
            Self::Internal(_) => 1,
        }
    }
}

impl From<WorkspaceMutationError> for MaintenanceCloseoutError {
    fn from(err: WorkspaceMutationError) -> Self {
        match err {
            WorkspaceMutationError::Validation(message) => Self::Validation(message),
            WorkspaceMutationError::Internal(message) => Self::Internal(message),
        }
    }
}

impl MaintenanceTriggerKind {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "drift_detected" => Some(Self::DriftDetected),
            "manual_reopen" => Some(Self::ManualReopen),
            "post_release_audit" => Some(Self::PostReleaseAudit),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::DriftDetected => "drift_detected",
            Self::ManualReopen => "manual_reopen",
            Self::PostReleaseAudit => "post_release_audit",
        }
    }
}

impl MaintenanceControlPlaneAction {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "packet_doc_refresh" => Some(Self::PacketDocRefresh),
            "support_matrix_refresh" => Some(Self::SupportMatrixRefresh),
            "capability_matrix_refresh" => Some(Self::CapabilityMatrixRefresh),
            "release_doc_refresh" => Some(Self::ReleaseDocRefresh),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::PacketDocRefresh => "packet_doc_refresh",
            Self::SupportMatrixRefresh => "support_matrix_refresh",
            Self::CapabilityMatrixRefresh => "capability_matrix_refresh",
            Self::ReleaseDocRefresh => "release_doc_refresh",
        }
    }
}

impl MaintenanceDriftCategory {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "registry_manifest_drift" => Some(Self::RegistryManifest),
            "capability_publication_drift" => Some(Self::CapabilityPublication),
            "support_publication_drift" => Some(Self::SupportPublication),
            "release_doc_drift" => Some(Self::ReleaseDoc),
            "governance_doc_drift" => Some(Self::GovernanceDoc),
            _ => None,
        }
    }

    pub fn as_id(self) -> &'static str {
        match self {
            Self::RegistryManifest => "registry_manifest_drift",
            Self::CapabilityPublication => "capability_publication_drift",
            Self::SupportPublication => "support_publication_drift",
            Self::ReleaseDoc => "release_doc_drift",
            Self::GovernanceDoc => "governance_doc_drift",
        }
    }
}

pub(crate) fn maintenance_pack_root(prefix: &str) -> PathBuf {
    Path::new(super::DOCS_NEXT_ROOT).join(prefix)
}
