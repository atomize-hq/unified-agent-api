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
    workspace_mutation::{
        apply_mutations, plan_create_or_replace, ApplySummary, WorkspaceMutationError,
        WorkspacePathJail,
    },
};

use super::{
    contract_policy::{
        build_execution_contract, derive_detected_release_fields, dispatch_kind_str,
        dispatch_workflow_value, opened_from_path,
    },
    docs::build_packet_docs_from_envelope,
    request::{
        self, validate_commit_value, validate_non_empty_scalar, validate_repo_relative_reference,
        DetectedRelease, ExecutionContract, MaintenanceAction, MaintenanceRequest,
        MaintenanceRequestEnvelope, RuntimeFollowupRequired, TriggerKind,
        AUTOMATED_ARTIFACT_VERSION,
    },
    support_audit::{derive_support_surface_audit, SupportSurfaceAudit},
};

#[derive(Debug, Parser, Clone)]
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
    let workspace_root = repo_root();
    let mut stdout = io::stdout();
    run_in_workspace(&workspace_root, args, &mut stdout)
}

pub fn run_in_workspace<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
) -> Result<(), Error> {
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
    Ok(())
}

pub fn build_prepare_plan(workspace_root: &Path, args: &Args) -> Result<PreparePlan, Error> {
    let registry = AgentRegistry::load(workspace_root)
        .map_err(|err| Error::Internal(format!("load agent registry: {err}")))?;
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
    let support_surface_audit =
        derive_support_surface_audit(workspace_root, entry, &detected_release)
            .map_err(Error::Internal)?;
    let request_bytes = render_request_toml(
        args,
        &basis_ref,
        &detected_release,
        &support_surface_audit,
        &execution_contract,
    )
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
        request_recorded_at: args.request_recorded_at.clone(),
        request_commit: args.request_commit.clone(),
    };
    let envelope = MaintenanceRequestEnvelope {
        request: request.clone(),
        execution_contract: Some(execution_contract),
    };

    let mut files = vec![PreparedFile {
        relative_path: request.relative_path.clone(),
        contents: request_bytes,
    }];
    for doc in build_packet_docs_from_envelope(workspace_root, &envelope)
        .map_err(|err| Error::Internal(format!("render maintenance packet docs: {err}")))?
    {
        files.push(PreparedFile {
            relative_path: doc.relative_path,
            contents: doc.contents.into_bytes(),
        });
    }
    Ok(PreparePlan { request, files })
}

pub fn apply_prepare_plan(
    workspace_root: &Path,
    plan: &PreparePlan,
) -> Result<ApplySummary, Error> {
    let jail = WorkspacePathJail::new(workspace_root)?;
    let mutations = plan
        .files
        .iter()
        .map(|file| {
            plan_create_or_replace(
                &jail,
                PathBuf::from(&file.relative_path),
                file.contents.clone(),
            )
            .map_err(Error::from)
        })
        .collect::<Result<Vec<_>, _>>()?;
    apply_mutations(workspace_root, &mutations).map_err(Into::into)
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

fn render_request_toml(
    args: &Args,
    basis_ref: &str,
    detected_release: &DetectedRelease,
    support_surface_audit: &SupportSurfaceAudit,
    execution_contract: &ExecutionContract,
) -> String {
    let mut out = String::new();
    push_toml_line(&mut out, "artifact_version", AUTOMATED_ARTIFACT_VERSION);
    push_toml_line(&mut out, "agent_id", &args.agent);
    push_toml_line(&mut out, "trigger_kind", "upstream_release_detected");
    push_toml_line(&mut out, "basis_ref", basis_ref);
    push_toml_line(
        &mut out,
        "opened_from",
        &args.opened_from.display().to_string(),
    );
    push_toml_array(
        &mut out,
        "requested_control_plane_actions",
        &["packet_doc_refresh".to_string()],
    );
    push_toml_line(&mut out, "request_recorded_at", &args.request_recorded_at);
    push_toml_line(&mut out, "request_commit", &args.request_commit);
    out.push('\n');

    out.push_str("[runtime_followup_required]\n");
    out.push_str("required = false\n");
    out.push_str("items = []\n\n");

    out.push_str("[detected_release]\n");
    push_toml_line(&mut out, "detected_by", &detected_release.detected_by);
    push_toml_line(
        &mut out,
        "current_validated",
        &detected_release.current_validated,
    );
    push_toml_line(&mut out, "target_version", &detected_release.target_version);
    push_toml_line(&mut out, "latest_stable", &detected_release.latest_stable);
    push_toml_line(&mut out, "version_policy", &detected_release.version_policy);
    push_toml_line(&mut out, "source_kind", &detected_release.source_kind);
    push_toml_line(&mut out, "source_ref", &detected_release.source_ref);
    push_toml_line(&mut out, "dispatch_kind", &detected_release.dispatch_kind);
    push_toml_line(
        &mut out,
        "dispatch_workflow",
        &detected_release.dispatch_workflow,
    );
    push_toml_line(&mut out, "branch_name", &detected_release.branch_name);
    out.push('\n');

    render_support_surface_audit(&mut out, support_surface_audit);
    out.push('\n');

    out.push_str("[execution_contract]\n");
    push_toml_line(&mut out, "executor", &execution_contract.executor);
    push_toml_line(
        &mut out,
        "prompt_template_path",
        &execution_contract.prompt_template_path,
    );
    push_toml_line(&mut out, "prompt_sha256", &execution_contract.prompt_sha256);
    push_toml_line(
        &mut out,
        "pr_summary_path",
        &execution_contract.pr_summary_path,
    );
    push_toml_line(&mut out, "closeout_path", &execution_contract.closeout_path);
    out.push_str("requires_manual_closeout = true\n");
    push_toml_array(
        &mut out,
        "writable_surfaces",
        &execution_contract.writable_surfaces,
    );
    push_toml_array(
        &mut out,
        "read_only_inputs",
        &execution_contract.read_only_inputs,
    );
    push_toml_array(
        &mut out,
        "ordered_commands",
        &execution_contract.ordered_commands,
    );
    push_toml_array(&mut out, "green_gates", &execution_contract.green_gates);
    out.push('\n');

    out.push_str("[execution_contract.recovery]\n");
    push_toml_line(
        &mut out,
        "recreate_packet_command",
        &execution_contract.recovery.recreate_packet_command,
    );
    push_toml_line(
        &mut out,
        "reopen_pr_body_path",
        &execution_contract.recovery.reopen_pr_body_path,
    );
    push_toml_line(
        &mut out,
        "reopen_pr_branch",
        &execution_contract.recovery.reopen_pr_branch,
    );
    push_toml_array(&mut out, "notes", &execution_contract.recovery.notes);
    out
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
        "support_surface_audit.discovered_upstream_surface",
        &audit.discovered_upstream_surface,
    );
    render_evidence_backed_rows(
        out,
        "support_surface_audit.removed_upstream_surface",
        &audit.removed_upstream_surface,
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
