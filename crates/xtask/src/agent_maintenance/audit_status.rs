use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use clap::Parser;
use serde::{de::IgnoredAny, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use toml_edit::de::from_str;

use crate::agent_registry::{AgentRegistry, AgentRegistryEntry};

use super::{
    request::{self, AuditReconciliation, DetectedRelease, MaintenanceRequest},
    support_audit::{self, SupportSurfaceAudit},
};

const EXIT_INTERNAL: i32 = 1;
const EXIT_VALIDATION: i32 = 2;

/// Exit code meaning the live support-surface audit still needs contributor relay work.
///
/// Closeout callers need a dedicated non-error signal here: "uplifts still required" is an
/// expected gate result, not the same class of failure as a malformed request or broken I/O.
pub const EXIT_UPLIFTS_REQUIRED: i32 = 3;

#[derive(Debug, Parser, Clone)]
pub struct Args {
    /// Maintenance request whose live support-surface audit should be re-derived.
    #[arg(long)]
    pub request: PathBuf,

    /// Write the audit status JSON here instead of stdout.
    #[arg(long)]
    pub emit_json: Option<PathBuf>,

    /// Workspace root containing the committed registry and manifest artifacts (default: cwd).
    #[arg(long)]
    pub workspace_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditStatusOutcome {
    Clean,
    UpliftsRequired,
}

impl AuditStatusOutcome {
    pub fn exit_code(self) -> i32 {
        match self {
            Self::Clean => 0,
            Self::UpliftsRequired => EXIT_UPLIFTS_REQUIRED,
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Internal(String),
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation(_) => EXIT_VALIDATION,
            Self::Internal(_) => EXIT_INTERNAL,
        }
    }
}

impl From<request::MaintenanceRequestError> for Error {
    fn from(value: request::MaintenanceRequestError) -> Self {
        match value {
            request::MaintenanceRequestError::Validation(message) => Self::Validation(message),
            request::MaintenanceRequestError::Internal(message) => Self::Internal(message),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AuditStatusProjection {
    agent_id: String,
    target_version: String,
    uplifts_required: bool,
    required_uplifts: Vec<RequiredUpliftProjection>,
    reconciliation: ReconciliationProjection,
    discovered_upstream_surface: usize,
    preexisting_unsupported_surface: usize,
    missing_wrapper_support: usize,
    missing_backend_support: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct RequiredUpliftProjection {
    surface_kind: String,
    command_path: String,
    surface_id: String,
    reason: String,
    required_writes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum ReconciliationProjection {
    Exact,
    Satisfied,
    Drifted,
}

impl From<AuditReconciliation> for ReconciliationProjection {
    fn from(value: AuditReconciliation) -> Self {
        match value {
            AuditReconciliation::Exact => Self::Exact,
            AuditReconciliation::Satisfied => Self::Satisfied,
        }
    }
}

struct DerivedAuditStatus {
    projection: AuditStatusProjection,
    outcome: AuditStatusOutcome,
}

struct ReconciliationStatus {
    projection: ReconciliationProjection,
    drift_message: Option<String>,
}

pub fn run(args: Args) -> Result<AuditStatusOutcome, Error> {
    let workspace_root = match args.workspace_root.as_ref() {
        Some(root) => root.to_path_buf(),
        None => std::env::current_dir()
            .map_err(|err| Error::Internal(format!("resolve current directory: {err}")))?,
    };
    let mut stdout = io::stdout();
    run_in_workspace(&workspace_root, args, &mut stdout)
}

pub fn run_in_workspace<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
) -> Result<AuditStatusOutcome, Error> {
    if let Some(path) = args.emit_json.as_ref() {
        remove_stale_projection(path)?;
    }

    let status = derive_audit_status(workspace_root, &args.request)?;
    let rendered = format!(
        "{}\n",
        serde_json::to_string_pretty(&status.projection)
            .map_err(|err| Error::Internal(format!("serialize maintenance audit status: {err}")))?
    );

    match args.emit_json.as_ref() {
        Some(path) => write_projection(path, &rendered)?,
        None => writer
            .write_all(rendered.as_bytes())
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?,
    }

    Ok(status.outcome)
}

fn derive_audit_status(
    workspace_root: &Path,
    request_path: &Path,
) -> Result<DerivedAuditStatus, Error> {
    let envelope = load_request_envelope_for_live_audit(workspace_root, request_path)?;
    let request = &envelope.request;
    let detected_release = require_automated_support_audit_request(request)?;

    let registry = AgentRegistry::load(workspace_root)
        .map_err(|err| Error::Internal(format!("load agent registry: {err}")))?;
    let entry = registry.find(&request.agent_id).ok_or_else(|| {
        Error::Internal(format!(
            "validated maintenance request `{}` references agent `{}` but the committed registry no longer contains it",
            request.relative_path, request.agent_id
        ))
    })?;

    let live_audit =
        support_audit::derive_support_surface_audit(workspace_root, entry, detected_release)
            .map_err(|err| {
                Error::Internal(format!(
                    "derive live support-surface audit for `{}` target `{}`: {err}",
                    request.agent_id, detected_release.target_version
                ))
            })?;
    let uplifts_required = !live_audit.required_uplifts_this_run.is_empty();
    if !uplifts_required {
        require_live_acquisition_evidence(workspace_root, entry, detected_release)?;
    }

    let reconciliation = derive_reconciliation_status(workspace_root, request_path)?;
    let projection = build_projection(
        request,
        &live_audit,
        detected_release,
        reconciliation.projection,
    );
    let outcome = if uplifts_required {
        AuditStatusOutcome::UpliftsRequired
    } else if reconciliation.projection == ReconciliationProjection::Drifted {
        return Err(Error::Validation(
            reconciliation
                .drift_message
                .unwrap_or_else(|| "maintenance request reconciliation drifted".to_string()),
        ));
    } else {
        AuditStatusOutcome::Clean
    };

    Ok(DerivedAuditStatus {
        projection,
        outcome,
    })
}

fn require_automated_support_audit_request(
    request: &MaintenanceRequest,
) -> Result<&DetectedRelease, Error> {
    if request.support_surface_audit.is_none() || request.detected_release.is_none() {
        return Err(Error::Validation(format!(
            "maintenance-audit-status requires a maintenance request with both `[detected_release]` and `[support_surface_audit]`; `{}` has trigger_kind `{}`",
            request.relative_path,
            request.trigger_kind.as_str()
        )));
    }

    Ok(request
        .detected_release
        .as_ref()
        .expect("checked detected_release presence above"))
}

fn build_projection(
    request: &MaintenanceRequest,
    live_audit: &SupportSurfaceAudit,
    detected_release: &DetectedRelease,
    reconciliation: ReconciliationProjection,
) -> AuditStatusProjection {
    let mut required_uplifts = live_audit
        .required_uplifts_this_run
        .iter()
        .map(|row| {
            let mut required_writes = row.required_writes.clone();
            required_writes.sort();
            RequiredUpliftProjection {
                surface_kind: row.surface_kind.clone(),
                command_path: row.command_path.clone(),
                surface_id: row.surface_id.clone(),
                reason: row.reason.clone(),
                required_writes,
            }
        })
        .collect::<Vec<_>>();
    required_uplifts.sort();

    AuditStatusProjection {
        agent_id: request.agent_id.clone(),
        target_version: detected_release.target_version.clone(),
        uplifts_required: !required_uplifts.is_empty(),
        required_uplifts,
        reconciliation,
        discovered_upstream_surface: live_audit.discovered_upstream_surface.len(),
        preexisting_unsupported_surface: live_audit.preexisting_unsupported_surface.len(),
        missing_wrapper_support: live_audit.missing_wrapper_support.len(),
        missing_backend_support: live_audit.missing_backend_support.len(),
    }
}

fn load_request_envelope_for_live_audit(
    workspace_root: &Path,
    request_path: &Path,
) -> Result<request::MaintenanceRequestEnvelope, Error> {
    match request::load_request_envelope(workspace_root, request_path) {
        Ok(envelope) => Ok(envelope),
        Err(request::MaintenanceRequestError::Validation(message))
            if is_recoverable_reconciliation_validation(&message) =>
        {
            recover_request_envelope_for_live_audit(workspace_root, request_path)
        }
        Err(err) => Err(err.into()),
    }
}

fn is_recoverable_reconciliation_validation(message: &str) -> bool {
    message.contains(
        "field `support_surface_audit` no longer matches the live derived maintenance contract",
    ) || message.contains(
        "field `support_surface_audit` cannot confirm reconciliation because live coverage report evidence",
    )
}

fn recover_request_envelope_for_live_audit(
    workspace_root: &Path,
    request_path: &Path,
) -> Result<request::MaintenanceRequestEnvelope, Error> {
    let workspace_root = fs::canonicalize(workspace_root).map_err(|err| {
        Error::Internal(format!("canonicalize {}: {err}", workspace_root.display()))
    })?;
    let lexical_path = if request_path.is_absolute() {
        request_path.to_path_buf()
    } else {
        workspace_root.join(request_path)
    };
    let canonical_path = fs::canonicalize(&lexical_path).map_err(|err| {
        Error::Validation(format!(
            "maintenance request `{}` does not resolve: {err}",
            request_path.display()
        ))
    })?;
    if !canonical_path.starts_with(&workspace_root) {
        return Err(Error::Validation(format!(
            "maintenance request `{}` resolves outside workspace root",
            request_path.display()
        )));
    }

    let relative_path = canonical_path
        .strip_prefix(&workspace_root)
        .map_err(|err| {
            Error::Internal(format!(
                "strip request path `{}` from workspace root `{}`: {err}",
                canonical_path.display(),
                workspace_root.display()
            ))
        })?
        .to_path_buf();
    let bytes = fs::read(&canonical_path).map_err(|err| {
        Error::Validation(format!(
            "read maintenance request `{}`: {err}",
            relative_path.display()
        ))
    })?;
    let text = std::str::from_utf8(&bytes).map_err(|err| {
        Error::Validation(format!(
            "maintenance request `{}` must be valid utf-8: {err}",
            relative_path.display()
        ))
    })?;
    let raw: RawMaintenanceRequestForLiveAudit = from_str(text).map_err(|err| {
        Error::Validation(format!(
            "parse maintenance request `{}`: {err}",
            relative_path.display()
        ))
    })?;
    let maintenance_pack_prefix = maintenance_pack_prefix_from_relative_path(&relative_path)
        .ok_or_else(|| {
            Error::Validation(format!(
                "maintenance request `{}` must live under `docs/agents/lifecycle/<agent>-maintenance/`",
                relative_path.display()
            ))
        })?;

    Ok(request::MaintenanceRequestEnvelope {
        request: MaintenanceRequest {
            relative_path: relative_path.display().to_string(),
            canonical_path,
            sha256: hex::encode(Sha256::digest(&bytes)),
            maintenance_pack_prefix: maintenance_pack_prefix.clone(),
            maintenance_root: Path::new("docs/agents/lifecycle")
                .join(&maintenance_pack_prefix)
                .display()
                .to_string(),
            agent_id: raw.agent_id,
            trigger_kind: parse_trigger_kind(&raw.trigger_kind, &relative_path)?,
            basis_ref: raw.basis_ref,
            opened_from: raw.opened_from,
            requested_control_plane_actions: raw
                .requested_control_plane_actions
                .iter()
                .map(|value| parse_maintenance_action(value, &relative_path))
                .collect::<Result<Vec<_>, _>>()?,
            runtime_followup_required: request::RuntimeFollowupRequired {
                required: raw.runtime_followup_required.required,
                items: raw.runtime_followup_required.items,
            },
            detected_release: raw.detected_release.map(map_detected_release),
            support_surface_audit: raw
                .support_surface_audit
                .map(|_| placeholder_support_surface_audit()),
            request_recorded_at: raw.request_recorded_at,
            request_commit: raw.request_commit,
        },
        execution_contract: None,
    })
}

fn maintenance_pack_prefix_from_relative_path(relative_path: &Path) -> Option<String> {
    let parts = relative_path
        .iter()
        .map(|segment| segment.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    if parts.len() < 4 || parts[0] != "docs" || parts[1] != "agents" || parts[2] != "lifecycle" {
        return None;
    }
    Some(parts[3].clone())
}

fn parse_trigger_kind(value: &str, request_path: &Path) -> Result<request::TriggerKind, Error> {
    match value {
        "drift_detected" => Ok(request::TriggerKind::DriftDetected),
        "manual_reopen" => Ok(request::TriggerKind::ManualReopen),
        "post_release_audit" => Ok(request::TriggerKind::PostReleaseAudit),
        "upstream_release_detected" => Ok(request::TriggerKind::UpstreamReleaseDetected),
        other => Err(Error::Validation(format!(
            "maintenance request `{}` has invalid `trigger_kind` `{other}`; expected `drift_detected`, `manual_reopen`, `post_release_audit`, or `upstream_release_detected`",
            request_path.display()
        ))),
    }
}

fn parse_maintenance_action(
    value: &str,
    request_path: &Path,
) -> Result<request::MaintenanceAction, Error> {
    match value {
        "packet_doc_refresh" => Ok(request::MaintenanceAction::PacketDocRefresh),
        "support_matrix_refresh" => Ok(request::MaintenanceAction::SupportMatrixRefresh),
        "capability_matrix_refresh" => Ok(request::MaintenanceAction::CapabilityMatrixRefresh),
        "release_doc_refresh" => Ok(request::MaintenanceAction::ReleaseDocRefresh),
        other => Err(Error::Validation(format!(
            "maintenance request `{}` requested runtime-owned or unsupported action `{other}`; allowed actions: `packet_doc_refresh`, `support_matrix_refresh`, `capability_matrix_refresh`, `release_doc_refresh`",
            request_path.display()
        ))),
    }
}

fn map_detected_release(raw: RawDetectedReleaseForLiveAudit) -> DetectedRelease {
    DetectedRelease {
        detected_by: raw.detected_by,
        current_validated: raw.current_validated,
        target_version: raw.target_version,
        latest_stable: raw.latest_stable,
        version_policy: raw.version_policy,
        source_kind: raw.source_kind,
        source_ref: raw.source_ref,
        dispatch_kind: raw.dispatch_kind,
        dispatch_workflow: raw.dispatch_workflow,
        branch_name: raw.branch_name,
    }
}

fn placeholder_support_surface_audit() -> SupportSurfaceAudit {
    SupportSurfaceAudit {
        required: true,
        surface_kinds: Vec::new(),
        excluded_surface_kinds: Vec::new(),
        allowed_deferrals: Vec::new(),
        pre_run_debt_count: 0,
        expected_post_run_debt_count: 0,
        discovered_upstream_surface: Vec::new(),
        removed_upstream_surface: Vec::new(),
        preexisting_unsupported_surface: Vec::new(),
        eligible_preexisting_surface: Vec::new(),
        missing_wrapper_support: Vec::new(),
        missing_backend_support: Vec::new(),
        required_uplifts_this_run: Vec::new(),
        deferred_preexisting_gaps: Vec::new(),
        publication_impacts: Vec::new(),
    }
}

fn require_live_acquisition_evidence(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    detected_release: &DetectedRelease,
) -> Result<(), Error> {
    let report_dir = format!(
        "{}/reports/{}",
        entry.manifest_root, detected_release.target_version
    );
    // This command re-derives the audit from live artifacts, so "clean" cannot mean both
    // "acquisition found no new surface" and "acquisition produced no artifacts at all".
    match support_audit::coverage_report_present_for_target(
        workspace_root,
        entry,
        &detected_release.target_version,
    ) {
        Ok(true) => Ok(()),
        Ok(false) => Err(Error::Validation(format!(
            "maintenance-audit-status requires live coverage report evidence for target version `{}` under `{}` before reporting a clean result",
            detected_release.target_version, report_dir
        ))),
        Err(error) => Err(Error::Validation(format!(
            "maintenance-audit-status could not read live coverage report evidence for target version `{}` under `{}`: {}",
            detected_release.target_version, report_dir, error
        ))),
    }
}

fn derive_reconciliation_status(
    workspace_root: &Path,
    request_path: &Path,
) -> Result<ReconciliationStatus, Error> {
    match request::load_request_envelope_validated(workspace_root, request_path) {
        Ok(validated) => Ok(ReconciliationStatus {
            projection: validated
                .support_surface_audit_reconciliation
                .map(Into::into)
                .ok_or_else(|| {
                    Error::Internal(format!(
                        "validated maintenance request `{}` is missing support-surface reconciliation metadata",
                        request_path.display()
                    ))
                })?,
            drift_message: None,
        }),
        Err(request::MaintenanceRequestError::Validation(message)) => Ok(ReconciliationStatus {
            projection: ReconciliationProjection::Drifted,
            drift_message: Some(message),
        }),
        Err(err) => Err(err.into()),
    }
}

fn write_projection(path: &Path, rendered: &str) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|err| Error::Internal(format!("create {}: {err}", parent.display())))?;
        }
    }
    fs::write(path, rendered)
        .map_err(|err| Error::Internal(format!("write {}: {err}", path.display())))?;
    Ok(())
}

fn remove_stale_projection(path: &Path) -> Result<(), Error> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(Error::Internal(format!(
            "remove stale maintenance audit status projection {}: {err}",
            path.display()
        ))),
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMaintenanceRequestForLiveAudit {
    #[serde(rename = "artifact_version")]
    _artifact_version: String,
    agent_id: String,
    trigger_kind: String,
    basis_ref: String,
    opened_from: String,
    requested_control_plane_actions: Vec<String>,
    runtime_followup_required: RawRuntimeFollowupRequired,
    #[serde(default)]
    detected_release: Option<RawDetectedReleaseForLiveAudit>,
    #[serde(default)]
    support_surface_audit: Option<IgnoredAny>,
    #[serde(default, rename = "execution_contract")]
    _execution_contract: Option<IgnoredAny>,
    request_recorded_at: String,
    request_commit: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRuntimeFollowupRequired {
    required: bool,
    items: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDetectedReleaseForLiveAudit {
    detected_by: String,
    current_validated: String,
    target_version: String,
    latest_stable: String,
    version_policy: String,
    source_kind: String,
    source_ref: String,
    dispatch_kind: String,
    dispatch_workflow: String,
    branch_name: String,
}
