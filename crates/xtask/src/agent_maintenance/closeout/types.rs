use std::{
    fmt,
    path::{Path, PathBuf},
};

use clap::Parser;

use super::super::{finding_signature::FindingSignature, request};
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
    pub detected_release: Option<DetectedRelease>,
    pub request_recorded_at: String,
    pub request_commit: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaintenanceTriggerKind {
    DriftDetected,
    ManualReopen,
    PostReleaseAudit,
    UpstreamReleaseDetected,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedRelease {
    pub detected_by: String,
    pub current_validated: String,
    pub target_version: String,
    pub latest_stable: String,
    pub version_policy: String,
    pub source_kind: String,
    pub source_ref: String,
    pub dispatch_kind: String,
    pub dispatch_workflow: String,
    pub branch_name: String,
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
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::DriftDetected => "drift_detected",
            Self::ManualReopen => "manual_reopen",
            Self::PostReleaseAudit => "post_release_audit",
            Self::UpstreamReleaseDetected => "upstream_release_detected",
        }
    }
}

impl From<request::MaintenanceRequest> for MaintenanceRequest {
    fn from(value: request::MaintenanceRequest) -> Self {
        Self {
            agent_id: value.agent_id,
            trigger_kind: value.trigger_kind.into(),
            basis_ref: value.basis_ref,
            opened_from: value.opened_from,
            requested_control_plane_actions: value
                .requested_control_plane_actions
                .into_iter()
                .map(Into::into)
                .collect(),
            runtime_followup_required: value.runtime_followup_required.into(),
            detected_release: value.detected_release.map(Into::into),
            request_recorded_at: value.request_recorded_at,
            request_commit: value.request_commit,
        }
    }
}

impl From<request::TriggerKind> for MaintenanceTriggerKind {
    fn from(value: request::TriggerKind) -> Self {
        match value {
            request::TriggerKind::DriftDetected => Self::DriftDetected,
            request::TriggerKind::ManualReopen => Self::ManualReopen,
            request::TriggerKind::PostReleaseAudit => Self::PostReleaseAudit,
            request::TriggerKind::UpstreamReleaseDetected => Self::UpstreamReleaseDetected,
        }
    }
}

impl From<request::MaintenanceAction> for MaintenanceControlPlaneAction {
    fn from(value: request::MaintenanceAction) -> Self {
        match value {
            request::MaintenanceAction::PacketDocRefresh => Self::PacketDocRefresh,
            request::MaintenanceAction::SupportMatrixRefresh => Self::SupportMatrixRefresh,
            request::MaintenanceAction::CapabilityMatrixRefresh => Self::CapabilityMatrixRefresh,
            request::MaintenanceAction::ReleaseDocRefresh => Self::ReleaseDocRefresh,
        }
    }
}

impl From<request::RuntimeFollowupRequired> for RuntimeFollowupRequired {
    fn from(value: request::RuntimeFollowupRequired) -> Self {
        Self {
            required: value.required,
            items: value.items,
        }
    }
}

impl From<request::DetectedRelease> for DetectedRelease {
    fn from(value: request::DetectedRelease) -> Self {
        Self {
            detected_by: value.detected_by,
            current_validated: value.current_validated,
            target_version: value.target_version,
            latest_stable: value.latest_stable,
            version_policy: value.version_policy,
            source_kind: value.source_kind,
            source_ref: value.source_ref,
            dispatch_kind: value.dispatch_kind,
            dispatch_workflow: value.dispatch_workflow,
            branch_name: value.branch_name,
        }
    }
}

impl MaintenanceControlPlaneAction {
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

impl MaintenanceFinding {
    pub(crate) fn signature(&self) -> FindingSignature {
        FindingSignature::new(self.category_id.as_id(), &self.surfaces)
    }
}

pub(crate) fn maintenance_pack_root(prefix: &str) -> PathBuf {
    Path::new(super::DOCS_NEXT_ROOT).join(prefix)
}
