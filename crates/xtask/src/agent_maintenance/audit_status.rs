use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use clap::Parser;
use serde::Serialize;
use thiserror::Error;

use crate::agent_registry::AgentRegistry;

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
    reconciliation: Option<ReconciliationProjection>,
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
    let validated = request::load_request_envelope_validated(workspace_root, request_path)?;
    let request = &validated.envelope.request;
    let detected_release = require_automated_support_audit_request(request)?;

    let registry = AgentRegistry::load(workspace_root)
        .map_err(|err| Error::Internal(format!("load agent registry: {err}")))?;
    let entry = registry.find(&request.agent_id).ok_or_else(|| {
        Error::Internal(format!(
            "validated maintenance request `{}` references agent `{}` but the committed registry no longer contains it",
            request.relative_path, request.agent_id
        ))
    })?;

    // Validation re-checks the frozen packet contract. This second call is the actual gate:
    // derive the audit again from the live repo so closeout reflects today's support surface.
    let live_audit =
        support_audit::derive_support_surface_audit(workspace_root, entry, detected_release)
            .map_err(|err| {
                Error::Internal(format!(
                    "derive live support-surface audit for `{}` target `{}`: {err}",
                    request.agent_id, detected_release.target_version
                ))
            })?;

    let projection = build_projection(
        request,
        &live_audit,
        detected_release,
        validated.support_surface_audit_reconciliation,
    );
    let outcome = if projection.uplifts_required {
        AuditStatusOutcome::UpliftsRequired
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
    reconciliation: Option<AuditReconciliation>,
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
        reconciliation: reconciliation.map(Into::into),
        discovered_upstream_surface: live_audit.discovered_upstream_surface.len(),
        preexisting_unsupported_surface: live_audit.preexisting_unsupported_surface.len(),
        missing_wrapper_support: live_audit.missing_wrapper_support.len(),
        missing_backend_support: live_audit.missing_backend_support.len(),
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
