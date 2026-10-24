#[path = "prepare/depth.rs"]
mod depth;
#[path = "prepare/guard.rs"]
mod guard;
#[path = "prepare/render.rs"]
mod render;
#[path = "prepare/report.rs"]
mod report;
pub(super) use render::render_request_toml;

use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};

use clap::{ArgGroup, Parser};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{
    agent_lifecycle::maintenance_request_path,
    agent_registry::{AgentRegistry, AgentRegistryEntry},
    workspace_mutation::{ApplySummary, WorkspaceMutationError, WorkspacePathJail},
};

use super::{
    contract_policy::{
        build_execution_contract, derive_detected_release_fields, dispatch_kind_str,
        dispatch_workflow_value, opened_from_path,
    },
    docs::build_preparation_packet_docs,
    request::{
        self, validate_commit_value, validate_non_empty_scalar, validate_repo_relative_reference,
        DetectedRelease, ExecutionContract, MaintenanceAction, MaintenanceRequest,
        RuntimeFollowupRequired, TriggerKind, AUTOMATED_ARTIFACT_VERSION,
    },
    support_audit::{derive_support_surface_audit, SupportSurfaceAudit},
};

#[path = "prepare/from_request.rs"]
mod from_request;
pub use from_request::{args_from_request, args_from_request_in_workspace};
#[path = "prepare/cli.rs"]
mod cli;
pub use cli::Cli;

#[derive(Debug, Parser, Clone, PartialEq, Eq)]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(["dry_run", "write"])
        .multiple(false)
))]
pub struct Args {
    #[arg(long)]
    pub agent: String,
    #[arg(long)]
    pub current_version: String,
    #[arg(long)]
    pub latest_stable: String,
    #[arg(long)]
    pub target_version: String,
    #[arg(long)]
    pub opened_from: PathBuf,
    #[arg(long)]
    pub detected_by: String,
    #[arg(long)]
    pub dispatch_kind: String,
    #[arg(long)]
    pub dispatch_workflow: Option<String>,
    #[arg(long)]
    pub branch_name: String,
    #[arg(long)]
    pub request_recorded_at: String,
    #[arg(long)]
    pub request_commit: String,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub write: bool,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Internal(String),
}

impl From<WorkspaceMutationError> for Error {
    fn from(value: WorkspaceMutationError) -> Self {
        match value {
            WorkspaceMutationError::Validation(message) => Self::Validation(message),
            WorkspaceMutationError::Internal(message) => Self::Internal(message),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparePlan {
    pub request: MaintenanceRequest,
    pub files: Vec<PreparedFile>,
    context: guard::Context,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedFile {
    pub relative_path: String,
    pub contents: Vec<u8>,
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation(_) => 2,
            Self::Internal(_) => 1,
        }
    }
}

impl PreparePlan {
    pub fn planned_paths(&self) -> Vec<&str> {
        self.files
            .iter()
            .map(|file| file.relative_path.as_str())
            .collect()
    }
}

pub fn run(args: Args) -> Result<(), Error> {
    run_with_report(args, None)
}

pub fn run_with_report(args: Args, report_path: Option<&Path>) -> Result<(), Error> {
    let workspace_root = repo_root();
    let mut stdout = io::stdout();
    run_in_workspace_with_report(&workspace_root, args, &mut stdout, report_path)
}

pub fn run_in_workspace<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
) -> Result<(), Error> {
    run_in_workspace_with_report(workspace_root, args, writer, None)
}

pub fn run_in_workspace_with_report<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
    report_path: Option<&Path>,
) -> Result<(), Error> {
    let report_path = report_path
        .map(|path| report::destination(workspace_root, path))
        .transpose()?;
    let plan = build_prepare_plan(workspace_root, &args)?;
    write_preview(writer, &plan, args.write)?;
    if args.write {
        let summary = apply_prepare_plan(workspace_root, &plan)?;
        writeln!(
            writer,
            "applied {} files (written {}, identical {})",
            summary.total, summary.written, summary.identical
        )
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
    if let Some(path) = report_path {
        report::write(&path, &plan, args.write)?;
    }
    Ok(())
}

pub fn build_prepare_plan(workspace_root: &Path, args: &Args) -> Result<PreparePlan, Error> {
    let registry = AgentRegistry::load(workspace_root)
        .map_err(|err| Error::Internal(format!("load agent registry: {err}")))?;
    guard::refuse_retained_depth(workspace_root, &registry)?;
    build_with_registry(workspace_root, args, &registry)
}

fn build_with_registry(
    workspace_root: &Path,
    args: &Args,
    registry: &AgentRegistry,
) -> Result<PreparePlan, Error> {
    let mut context = guard::Context::capture(workspace_root, registry, args)?;
    let entry = registry.find(&args.agent).ok_or_else(|| {
        Error::Validation(format!(
            "prepare-agent-maintenance references unknown agent_id `{}`",
            args.agent
        ))
    })?;
    let release_watch = entry.maintenance.release_watch.as_ref().ok_or_else(|| {
        Error::Validation(format!(
            "prepare-agent-maintenance requires maintenance.release_watch metadata for agent `{}`",
            args.agent
        ))
    })?;
    validate_prepare_args(workspace_root, entry, args)?;

    let request_path = maintenance_request_path(&args.agent);
    let maintenance_root = format!("docs/agents/lifecycle/{}-maintenance", args.agent);
    let basis_ref = format!("{}/latest_validated.txt", entry.manifest_root);
    let derived_detected_release =
        derive_detected_release_fields(&args.agent, release_watch).map_err(Error::Validation)?;
    let detected_release = DetectedRelease {
        detected_by: args.detected_by.clone(),
        current_validated: args.current_version.clone(),
        target_version: args.target_version.clone(),
        latest_stable: args.latest_stable.clone(),
        version_policy: derived_detected_release.version_policy,
        source_kind: derived_detected_release.source_kind,
        source_ref: derived_detected_release.source_ref,
        dispatch_kind: derived_detected_release.dispatch_kind,
        dispatch_workflow: derived_detected_release.dispatch_workflow,
        branch_name: args.branch_name.clone(),
    };
    let execution_contract = build_execution_contract(
        workspace_root,
        entry,
        &request_path,
        &maintenance_root,
        &args.opened_from.display().to_string(),
        &args.target_version,
        &args.branch_name,
    )
    .map_err(Error::Internal)?;
    let mut support_surface_audit =
        derive_support_surface_audit(workspace_root, entry, &detected_release)
            .map_err(Error::Internal)?;
    let freeze = depth::freeze(workspace_root, entry, args, &mut support_surface_audit)?;
    let support_depth = freeze.as_ref().map(|plan| request::SupportDepthBinding {
        policy_identity: plan.record.policy.identity.clone(),
        obligations_identity: plan.record.obligations.as_ref().map(|o| o.identity.clone()),
    });
    let request_bytes = render_request_toml(
        args,
        &basis_ref,
        &detected_release,
        &support_surface_audit,
        &execution_contract,
        support_depth.as_ref(),
    )?
    .into_bytes();
    let request = MaintenanceRequest {
        relative_path: request_path.clone(),
        canonical_path: workspace_root.join(&request_path),
        sha256: hex::encode(Sha256::digest(&request_bytes)),
        maintenance_pack_prefix: format!("{}-maintenance", args.agent),
        maintenance_root,
        agent_id: args.agent.clone(),
        trigger_kind: TriggerKind::UpstreamReleaseDetected,
        basis_ref,
        opened_from: args.opened_from.display().to_string(),
        requested_control_plane_actions: vec![MaintenanceAction::PacketDocRefresh],
        runtime_followup_required: RuntimeFollowupRequired {
            required: false,
            items: Vec::new(),
        },
        detected_release: Some(detected_release),
        support_surface_audit: Some(support_surface_audit),
        support_depth,
        request_recorded_at: args.request_recorded_at.clone(),
        request_commit: args.request_commit.clone(),
    };
    let mut files = vec![PreparedFile {
        relative_path: request.relative_path.clone(),
        contents: request_bytes,
    }];
    for doc in build_preparation_packet_docs(workspace_root, &request, &execution_contract, entry)
        .map_err(|err| Error::Internal(format!("render maintenance packet docs: {err}")))?
    {
        files.push(PreparedFile {
            relative_path: doc.relative_path,
            contents: doc.contents.into_bytes(),
        });
    }
    if let Some(freeze) = &freeze {
        depth::append(workspace_root, registry, freeze, &mut files)?;
    }
    context.bind_outputs(workspace_root, &files)?;
    let after = guard::Context::capture(workspace_root, registry, args)?;
    if context.inputs != after.inputs || context.index != after.index {
        return Err(Error::Validation(
            "preparation inputs changed during planning".into(),
        ));
    }
    Ok(PreparePlan {
        request,
        files,
        context,
    })
}

pub fn apply_prepare_plan(
    workspace_root: &Path,
    plan: &PreparePlan,
) -> Result<ApplySummary, Error> {
    let registry = AgentRegistry::load(workspace_root)
        .map_err(|err| Error::Validation(format!("load agent registry: {err}")))?;
    guard::refuse_retained_depth(workspace_root, &registry)?;
    guard::apply(workspace_root, plan, &registry)
}

fn validate_prepare_args(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    args: &Args,
) -> Result<(), Error> {
    let request_path = PathBuf::from(maintenance_request_path(&args.agent));
    let jail = WorkspacePathJail::new(workspace_root)?;
    validate_non_empty_scalar(&request_path, "agent", &args.agent)?;
    validate_non_empty_scalar(&request_path, "current_version", &args.current_version)?;
    validate_non_empty_scalar(&request_path, "latest_stable", &args.latest_stable)?;
    validate_non_empty_scalar(&request_path, "target_version", &args.target_version)?;
    validate_repo_relative_reference(
        &jail,
        &request_path,
        "opened_from",
        &args.opened_from.display().to_string(),
    )?;
    validate_non_empty_scalar(&request_path, "detected_by", &args.detected_by)?;
    validate_non_empty_scalar(&request_path, "dispatch_kind", &args.dispatch_kind)?;
    validate_non_empty_scalar(&request_path, "branch_name", &args.branch_name)?;
    request::validate_rfc3339_utc(
        &request_path,
        "request_recorded_at",
        &args.request_recorded_at,
    )?;
    validate_commit_value(&request_path, "request_commit", &args.request_commit)?;

    let basis_ref = workspace_root
        .join(&entry.manifest_root)
        .join("latest_validated.txt");
    if !basis_ref.is_file() {
        return Err(Error::Validation(format!(
            "prepare-agent-maintenance basis_ref for agent `{}` is missing: {}",
            args.agent,
            basis_ref.display()
        )));
    }

    let release_watch = entry.maintenance.release_watch.as_ref().ok_or_else(|| {
        Error::Validation(format!(
            "prepare-agent-maintenance requires maintenance.release_watch metadata for agent `{}`",
            entry.agent_id
        ))
    })?;
    let expected_dispatch_kind = dispatch_kind_str(release_watch.dispatch_kind);
    if args.dispatch_kind != expected_dispatch_kind {
        return Err(Error::Validation(format!(
            "prepare-agent-maintenance --dispatch-kind for agent `{}` must be `{expected_dispatch_kind}` (got `{}`)",
            entry.agent_id, args.dispatch_kind
        )));
    }
    let expected_dispatch_workflow =
        dispatch_workflow_value(&entry.agent_id, release_watch).map_err(Error::Validation)?;
    match args.dispatch_workflow.as_deref() {
        Some(dispatch_workflow) if dispatch_workflow != expected_dispatch_workflow => {
            return Err(Error::Validation(format!(
                "prepare-agent-maintenance --dispatch-workflow for agent `{}` must be `{expected_dispatch_workflow}` (got `{dispatch_workflow}`)",
                entry.agent_id
            )));
        }
        None if args.dispatch_kind == "workflow_dispatch" => {
            return Err(Error::Validation(format!(
                "prepare-agent-maintenance requires --dispatch-workflow `{expected_dispatch_workflow}` when --dispatch-kind workflow_dispatch"
            )));
        }
        _ => {}
    }
    let expected_opened_from = opened_from_path(&expected_dispatch_workflow);
    if args.opened_from.display().to_string() != expected_opened_from {
        return Err(Error::Validation(format!(
            "prepare-agent-maintenance --opened-from for agent `{}` must be `{expected_opened_from}` (got `{}`)",
            entry.agent_id,
            args.opened_from.display()
        )));
    }
    Ok(())
}

fn render_support_surface_audit(out: &mut String, audit: &SupportSurfaceAudit) {
    out.push_str("[support_surface_audit]\n");
    out.push_str("required = true\n");
    push_toml_array(out, "surface_kinds", &audit.surface_kinds);
    push_toml_array(out, "excluded_surface_kinds", &audit.excluded_surface_kinds);
    push_toml_array(out, "allowed_deferrals", &audit.allowed_deferrals);
    out.push_str(&format!(
        "pre_run_debt_count = {}\n",
        audit.pre_run_debt_count
    ));
    out.push_str(&format!(
        "expected_post_run_debt_count = {}\n",
        audit.expected_post_run_debt_count
    ));
    render_evidence_backed_rows(
        out,
        "support_surface_audit.unbaselined_gap_surface",
        &audit.unbaselined_gap_surface,
    );
    render_unmatched_debt_rows(
        out,
        "support_surface_audit.unmatched_debt_surface",
        &audit.unmatched_debt_surface,
    );
    render_debt_backed_rows(
        out,
        "support_surface_audit.preexisting_unsupported_surface",
        &audit.preexisting_unsupported_surface,
    );
    render_eligible_rows(
        out,
        "support_surface_audit.eligible_preexisting_surface",
        &audit.eligible_preexisting_surface,
    );
    render_identity_rows(
        out,
        "support_surface_audit.missing_wrapper_support",
        &audit.missing_wrapper_support,
    );
    render_identity_rows(
        out,
        "support_surface_audit.missing_backend_support",
        &audit.missing_backend_support,
    );
    render_required_uplift_rows(
        out,
        "support_surface_audit.required_uplifts_this_run",
        &audit.required_uplifts_this_run,
    );
    render_deferred_rows(
        out,
        "support_surface_audit.deferred_preexisting_gaps",
        &audit.deferred_preexisting_gaps,
    );
    render_publication_rows(
        out,
        "support_surface_audit.publication_impacts",
        &audit.publication_impacts,
    );
}

fn push_toml_line(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = \"");
    out.push_str(value);
    out.push_str("\"\n");
}

fn push_toml_array(out: &mut String, key: &str, values: &[String]) {
    out.push_str(key);
    out.push_str(" = [\n");
    for value in values {
        out.push_str("  \"");
        out.push_str(value);
        out.push_str("\",\n");
    }
    out.push_str("]\n");
}

fn render_identity_rows<T>(out: &mut String, table: &str, rows: &[T])
where
    T: IdentityRow,
{
    for row in rows {
        out.push('\n');
        out.push_str("[[");
        out.push_str(table);
        out.push_str("]]\n");
        push_toml_line(out, "surface_kind", row.surface_kind());
        push_toml_line(out, "command_path", row.command_path());
        push_toml_line(out, "surface_id", row.surface_id());
    }
}

fn render_evidence_backed_rows(
    out: &mut String,
    table: &str,
    rows: &[super::support_audit::EvidenceBackedSurface],
) {
    for row in rows {
        out.push('\n');
        out.push_str("[[");
        out.push_str(table);
        out.push_str("]]\n");
        push_toml_line(out, "surface_kind", &row.surface_kind);
        push_toml_line(out, "command_path", &row.command_path);
        push_toml_line(out, "surface_id", &row.surface_id);
        push_toml_line(out, "evidence_ref", &row.evidence_ref);
    }
}

fn render_debt_backed_rows(
    out: &mut String,
    table: &str,
    rows: &[super::support_audit::DebtBackedSurface],
) {
    for row in rows {
        out.push('\n');
        out.push_str("[[");
        out.push_str(table);
        out.push_str("]]\n");
        push_toml_line(out, "surface_kind", &row.surface_kind);
        push_toml_line(out, "command_path", &row.command_path);
        push_toml_line(out, "surface_id", &row.surface_id);
        push_toml_line(out, "debt_ref", &row.debt_ref);
    }
}

fn render_unmatched_debt_rows(
    out: &mut String,
    table: &str,
    rows: &[super::support_audit::UnmatchedDebtSurface],
) {
    for row in rows {
        out.push('\n');
        out.push_str("[[");
        out.push_str(table);
        out.push_str("]]\n");
        push_toml_line(out, "surface_kind", &row.surface_kind);
        push_toml_line(out, "command_path", &row.command_path);
        push_toml_line(out, "surface_id", &row.surface_id);
        push_toml_line(out, "debt_ref", &row.debt_ref);
        push_toml_line(out, "observation", &row.observation);
    }
}

fn render_eligible_rows(
    out: &mut String,
    table: &str,
    rows: &[super::support_audit::EligibleSurface],
) {
    for row in rows {
        out.push('\n');
        out.push_str("[[");
        out.push_str(table);
        out.push_str("]]\n");
        push_toml_line(out, "surface_kind", &row.surface_kind);
        push_toml_line(out, "command_path", &row.command_path);
        push_toml_line(out, "surface_id", &row.surface_id);
        push_toml_line(out, "eligibility_reason", &row.eligibility_reason);
    }
}

fn render_required_uplift_rows(
    out: &mut String,
    table: &str,
    rows: &[super::support_audit::RequiredUplift],
) {
    for row in rows {
        out.push('\n');
        out.push_str("[[");
        out.push_str(table);
        out.push_str("]]\n");
        push_toml_line(out, "surface_kind", &row.surface_kind);
        push_toml_line(out, "command_path", &row.command_path);
        push_toml_line(out, "surface_id", &row.surface_id);
        push_toml_line(out, "reason", &row.reason);
        push_toml_array(out, "required_writes", &row.required_writes);
    }
}

fn render_deferred_rows(out: &mut String, table: &str, rows: &[super::support_audit::DeferredGap]) {
    for row in rows {
        out.push('\n');
        out.push_str("[[");
        out.push_str(table);
        out.push_str("]]\n");
        push_toml_line(out, "surface_kind", &row.surface_kind);
        push_toml_line(out, "command_path", &row.command_path);
        push_toml_line(out, "surface_id", &row.surface_id);
        push_toml_line(out, "defer_reason", &row.defer_reason);
        if let Some(blocking_follow_on) = row.blocking_follow_on.as_ref() {
            push_toml_line(out, "blocking_follow_on", blocking_follow_on);
        }
    }
}

fn render_publication_rows(
    out: &mut String,
    table: &str,
    rows: &[super::support_audit::PublicationImpact],
) {
    for row in rows {
        out.push('\n');
        out.push_str("[[");
        out.push_str(table);
        out.push_str("]]\n");
        push_toml_line(out, "surface_kind", &row.surface_kind);
        push_toml_line(out, "command_path", &row.command_path);
        push_toml_line(out, "surface_id", &row.surface_id);
        push_toml_line(out, "surface_doc", &row.surface_doc);
    }
}

trait IdentityRow {
    fn surface_kind(&self) -> &str;
    fn command_path(&self) -> &str;
    fn surface_id(&self) -> &str;
}

impl IdentityRow for super::support_audit::SurfaceIdentity {
    fn surface_kind(&self) -> &str {
        &self.surface_kind
    }

    fn command_path(&self) -> &str {
        &self.command_path
    }

    fn surface_id(&self) -> &str {
        &self.surface_id
    }
}

fn write_preview<W: Write>(writer: &mut W, plan: &PreparePlan, writing: bool) -> Result<(), Error> {
    writeln!(writer, "request: {}", plan.request.relative_path)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    for path in plan.planned_paths() {
        writeln!(writer, "planned: {path}")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
    if !writing {
        writeln!(writer, "dry_run: true")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
    Ok(())
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("xtask crate should live under crates/xtask")
        .to_path_buf()
}

// This source is also included by the integration harness. These test-only, parent-visible
// shims exercise private cores without exporting an unchecked production writer.
#[cfg(test)]
#[allow(dead_code)]
pub(super) fn isolated_build(
    root: &Path,
    args: &Args,
    registry: &AgentRegistry,
) -> Result<PreparePlan, Error> {
    build_with_registry(root, args, registry)
}
#[cfg(test)]
#[allow(dead_code)]
pub(super) fn isolated_apply(
    root: &Path,
    plan: &PreparePlan,
    registry: &AgentRegistry,
) -> Result<ApplySummary, Error> {
    guard::apply(root, plan, registry)
}
