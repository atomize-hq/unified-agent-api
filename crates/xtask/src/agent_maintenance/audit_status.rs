use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use serde::Serialize;
use thiserror::Error;

use crate::agent_registry::{AgentRegistry, AgentRegistryEntry};

use super::{
    request::{self, AuditDriftPolicy, AuditReconciliation, DetectedRelease, MaintenanceRequest},
    support_audit::{self, SupportSurfaceAudit},
};

const EXIT_INTERNAL: i32 = 1;
const EXIT_VALIDATION: i32 = 2;
const AUDIT_STATUS_SCHEMA_VERSION: u32 = 1;
const COVERAGE_REPORT_PREFERRED_FILES: [&str; 2] = ["coverage.any.json", "coverage.all.json"];

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
    schema_version: u32,
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
            AuditReconciliation::Drifted => Self::Drifted,
        }
    }
}

struct DerivedAuditStatus {
    projection: AuditStatusProjection,
    outcome: AuditStatusOutcome,
}

struct DeriveAuditStatusFailure {
    error: Error,
    live_derivation_attempted: bool,
}

impl DeriveAuditStatusFailure {
    fn preflight(error: Error) -> Self {
        Self {
            error,
            live_derivation_attempted: false,
        }
    }

    fn attempted(error: Error) -> Self {
        Self {
            error,
            live_derivation_attempted: true,
        }
    }
}

enum CoverageReportLookupError {
    Missing,
    Io(String),
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
    let status = match derive_audit_status(workspace_root, &args.request) {
        Ok(status) => status,
        Err(failure) => {
            if failure.live_derivation_attempted {
                if let Some(path) = args.emit_json.as_ref() {
                    remove_projection(path)?;
                }
            }
            return Err(failure.error);
        }
    };

    let rendered = format!(
        "{}\n",
        serde_json::to_string_pretty(&status.projection)
            .map_err(|err| Error::Internal(format!("serialize maintenance audit status: {err}")))?
    );

    match args.emit_json.as_ref() {
        Some(path) => write_projection_atomically(path, &rendered)?,
        None => writer
            .write_all(rendered.as_bytes())
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?,
    }

    Ok(status.outcome)
}

fn derive_audit_status(
    workspace_root: &Path,
    request_path: &Path,
) -> Result<DerivedAuditStatus, DeriveAuditStatusFailure> {
    let validated = request::load_request_envelope_validated_with_policy(
        workspace_root,
        request_path,
        AuditDriftPolicy::Tolerate,
    )
    .map_err(Error::from)
    .map_err(DeriveAuditStatusFailure::preflight)?;
    let reconciliation_detail = validated
        .support_surface_audit_reconciliation_detail
        .clone();
    let request = &validated.envelope.request;
    let detected_release = require_automated_support_audit_request(request)
        .map_err(DeriveAuditStatusFailure::preflight)?;
    let reconciliation = validated
        .support_surface_audit_reconciliation
        .ok_or_else(|| {
            DeriveAuditStatusFailure::preflight(Error::Internal(format!(
                "validated maintenance request `{}` is missing support-surface reconciliation metadata",
                request.relative_path
            )))
        })?;

    let registry = AgentRegistry::load(workspace_root)
        .map_err(|err| Error::Internal(format!("load agent registry: {err}")))
        .map_err(DeriveAuditStatusFailure::preflight)?;
    let entry = registry.find(&request.agent_id).ok_or_else(|| {
        DeriveAuditStatusFailure::preflight(Error::Internal(format!(
            "validated maintenance request `{}` references agent `{}` but the committed registry no longer contains it",
            request.relative_path, request.agent_id
        )))
    })?;

    let live_audit =
        support_audit::derive_support_surface_audit(workspace_root, entry, detected_release)
            .map_err(|err| {
                DeriveAuditStatusFailure::attempted(Error::Internal(format!(
                    "derive live support-surface audit for `{}` target `{}`: {err}",
                    request.agent_id, detected_release.target_version
                )))
            })?;
    let uplifts_required = !live_audit.required_uplifts_this_run.is_empty();
    if !uplifts_required {
        require_live_acquisition_evidence(workspace_root, entry, detected_release)
            .map_err(DeriveAuditStatusFailure::attempted)?;
    }

    let projection = build_projection(
        request,
        &live_audit,
        detected_release,
        reconciliation.into(),
    );
    let outcome = if uplifts_required {
        AuditStatusOutcome::UpliftsRequired
    } else if reconciliation == AuditReconciliation::Drifted {
        return Err(DeriveAuditStatusFailure::attempted(Error::Validation(
            reconciliation_detail.unwrap_or_else(|| {
                format!(
                    "maintenance request `{}` support-surface reconciliation drifted",
                    request.relative_path
                )
            }),
        )));
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
        schema_version: AUDIT_STATUS_SCHEMA_VERSION,
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

fn require_live_acquisition_evidence(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    detected_release: &DetectedRelease,
) -> Result<(), Error> {
    let report_dir =
        coverage_report_version_dir(workspace_root, entry, &detected_release.target_version);
    let report_dir_display = format!(
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
        Ok(true) => {}
        Ok(false) => {
            return Err(Error::Validation(format!(
                "maintenance-audit-status requires live coverage report evidence for target version `{}` under `{}` before reporting a clean result",
                detected_release.target_version, report_dir_display
            )));
        }
        Err(error) => {
            return Err(Error::Internal(format!(
                "maintenance-audit-status could not read live coverage report evidence for target version `{}` under `{}`: {}",
                detected_release.target_version, report_dir_display, error
            )));
        }
    }

    let report_path = select_coverage_report_path(&report_dir).map_err(|error| match error {
        CoverageReportLookupError::Missing => Error::Validation(format!(
            "maintenance-audit-status requires live coverage report evidence for target version `{}` under `{}` before reporting a clean result",
            detected_release.target_version, report_dir_display
        )),
        CoverageReportLookupError::Io(error) => Error::Internal(format!(
            "maintenance-audit-status could not read live coverage report evidence for target version `{}` under `{}`: {}",
            detected_release.target_version, report_dir_display, error
        )),
    })?;
    validate_live_coverage_report_target_version(&report_path, &detected_release.target_version)
}

fn coverage_report_version_dir(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    target_version: &str,
) -> PathBuf {
    workspace_root
        .join(&entry.manifest_root)
        .join("reports")
        .join(target_version)
}

fn select_coverage_report_path(version_dir: &Path) -> Result<PathBuf, CoverageReportLookupError> {
    for preferred in COVERAGE_REPORT_PREFERRED_FILES {
        let path = version_dir.join(preferred);
        if path.is_file() {
            return Ok(path);
        }
    }

    let mut candidates = fs::read_dir(version_dir)
        .map_err(|err| {
            CoverageReportLookupError::Io(format!("read_dir({}): {err}", version_dir.display()))
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("coverage.") && name.ends_with(".json"))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    candidates.sort();
    candidates
        .into_iter()
        .next()
        .ok_or(CoverageReportLookupError::Missing)
}

fn validate_live_coverage_report_target_version(
    report_path: &Path,
    target_version: &str,
) -> Result<(), Error> {
    let text = fs::read_to_string(report_path)
        .map_err(|err| Error::Internal(format!("read {}: {err}", report_path.display())))?;
    let json = serde_json::from_str::<serde_json::Value>(&text).map_err(|err| {
        Error::Validation(format!(
            "maintenance-audit-status requires live coverage report `{}` to be valid JSON with `inputs.upstream.semantic_version`: {err}",
            report_path.display()
        ))
    })?;
    let actual_version = json
        .pointer("/inputs/upstream/semantic_version")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            Error::Validation(format!(
                "maintenance-audit-status requires live coverage report `{}` to declare string `inputs.upstream.semantic_version`",
                report_path.display()
            ))
        })?;
    if actual_version != target_version {
        return Err(Error::Validation(format!(
            "maintenance-audit-status requires live coverage report `{}` to match detected release target version `{}`, but `inputs.upstream.semantic_version` is `{}`",
            report_path.display(),
            target_version,
            actual_version
        )));
    }

    Ok(())
}

fn write_projection_atomically(path: &Path, rendered: &str) -> Result<(), Error> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if parent != Path::new(".") {
        fs::create_dir_all(parent)
            .map_err(|err| Error::Internal(format!("create {}: {err}", parent.display())))?;
    }

    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| Error::Internal(format!("read system clock: {err}")))?
        .as_nanos();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("maintenance-audit-status.json");
    let temp_path = parent.join(format!(
        ".{file_name}.tmp-{}-{unique_suffix}",
        std::process::id()
    ));

    fs::write(&temp_path, rendered)
        .map_err(|err| Error::Internal(format!("write {}: {err}", temp_path.display())))?;
    if let Err(err) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(Error::Internal(format!(
            "rename {} -> {}: {err}",
            temp_path.display(),
            path.display()
        )));
    }

    Ok(())
}

fn remove_projection(path: &Path) -> Result<(), Error> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(Error::Internal(format!(
            "remove stale maintenance audit status projection {}: {err}",
            path.display()
        ))),
    }
}
