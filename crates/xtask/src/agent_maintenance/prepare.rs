use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use clap::{ArgGroup, Parser};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{
    agent_lifecycle::maintenance_request_path,
    agent_registry::{
        AgentRegistry, AgentRegistryEntry, ReleaseWatchMetadata, ReleaseWatchSourceKind,
    },
    workspace_mutation::{
        apply_mutations, plan_create_or_replace, ApplySummary, WorkspaceMutationError,
        WorkspacePathJail,
    },
};

use super::{
    docs::build_packet_docs_from_envelope,
    request::{
        self, validate_commit_value, validate_non_empty_scalar, validate_repo_relative_reference,
        DetectedRelease, ExecutionContract, ExecutionContractRecovery, MaintenanceAction,
        MaintenanceRequest, MaintenanceRequestEnvelope, RuntimeFollowupRequired, TriggerKind,
        AUTOMATED_ARTIFACT_VERSION,
    },
};

const GENERIC_PACKET_PR_WORKFLOW: &str = "agent-maintenance-open-pr.yml";
const WORKFLOW_DISPATCH_KIND: &str = "workflow_dispatch";
const PACKET_PR_KIND: &str = "packet_pr";

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
    let detected_release = DetectedRelease {
        detected_by: args.detected_by.clone(),
        current_validated: args.current_version.clone(),
        target_version: args.target_version.clone(),
        latest_stable: args.latest_stable.clone(),
        version_policy: "latest_stable_minus_one".to_string(),
        source_kind: source_kind_str(release_watch.upstream.source_kind).to_string(),
        source_ref: source_ref(release_watch),
        dispatch_kind: args.dispatch_kind.clone(),
        dispatch_workflow: dispatch_workflow_value(args),
        branch_name: args.branch_name.clone(),
    };
    let execution_contract = build_execution_contract(
        workspace_root,
        entry,
        args,
        &request_path,
        &maintenance_root,
    )?;
    let request_bytes =
        render_request_toml(args, &basis_ref, &detected_release, &execution_contract).into_bytes();
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

    match args.dispatch_kind.as_str() {
        WORKFLOW_DISPATCH_KIND => {
            if args.dispatch_workflow.is_none() {
                return Err(Error::Validation(
                    "prepare-agent-maintenance requires --dispatch-workflow when --dispatch-kind workflow_dispatch"
                        .to_string(),
                ));
            }
        }
        PACKET_PR_KIND => {
            if let Some(dispatch_workflow) = args.dispatch_workflow.as_deref() {
                if dispatch_workflow != GENERIC_PACKET_PR_WORKFLOW {
                    return Err(Error::Validation(format!(
                        "prepare-agent-maintenance packet_pr dispatch must use `{GENERIC_PACKET_PR_WORKFLOW}` when --dispatch-workflow is provided (got `{dispatch_workflow}`)"
                    )));
                }
            }
        }
        other => {
            return Err(Error::Validation(format!(
                "prepare-agent-maintenance has unsupported --dispatch-kind `{other}`; expected `{WORKFLOW_DISPATCH_KIND}` or `{PACKET_PR_KIND}`"
            )));
        }
    }
    Ok(())
}

fn render_request_toml(
    args: &Args,
    basis_ref: &str,
    detected_release: &DetectedRelease,
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

fn build_execution_contract(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    args: &Args,
    request_path: &str,
    maintenance_root: &str,
) -> Result<ExecutionContract, Error> {
    let prompt_template_path = format!("{}/PR_BODY_TEMPLATE.md", entry.manifest_root);
    let prompt_contents =
        render_execution_prompt(workspace_root, &prompt_template_path, &args.target_version)?;
    let pr_summary_path = format!("{maintenance_root}/governance/pr-summary.md");
    let closeout_path = format!("{maintenance_root}/governance/maintenance-closeout.json");
    let green_gates = execution_green_gates(entry);

    Ok(ExecutionContract {
        executor: "codex".to_string(),
        prompt_template_path,
        prompt_sha256: hex::encode(Sha256::digest(prompt_contents.as_bytes())),
        pr_summary_path: pr_summary_path.clone(),
        closeout_path,
        requires_manual_closeout: true,
        writable_surfaces: execution_writable_surfaces(entry, maintenance_root, args),
        read_only_inputs: vec![
            format!("{}/OPS_PLAYBOOK.md", entry.manifest_root),
            format!("{}/CI_WORKFLOWS_PLAN.md", entry.manifest_root),
            format!("{}/PR_BODY_TEMPLATE.md", entry.manifest_root),
            args.opened_from.display().to_string(),
        ],
        ordered_commands: green_gates.clone(),
        green_gates,
        recovery: ExecutionContractRecovery {
            recreate_packet_command: format!(
                "cargo run -p xtask -- prepare-agent-maintenance --request {request_path} --write"
            ),
            reopen_pr_body_path: pr_summary_path,
            reopen_pr_branch: args.branch_name.clone(),
            notes: vec![
                "If PR creation fails after packet generation, rerun packet creation and reopen the PR from the generated pr-summary path.".to_string(),
                "If local Codex preflight fails, fix binary/auth and rerun execute-agent-maintenance --dry-run before write mode.".to_string(),
            ],
        },
    })
}

fn render_execution_prompt(
    workspace_root: &Path,
    prompt_template_path: &str,
    target_version: &str,
) -> Result<String, Error> {
    let template =
        fs::read_to_string(workspace_root.join(prompt_template_path)).map_err(|err| {
            Error::Internal(format!(
                "read execution contract prompt template `{prompt_template_path}`: {err}"
            ))
        })?;
    Ok(template.replace("{{VERSION}}", target_version))
}

fn execution_green_gates(entry: &AgentRegistryEntry) -> Vec<String> {
    vec![
        "cargo fmt --all".to_string(),
        format!(
            "cargo run -p xtask -- codex-validate --root {}",
            entry.manifest_root
        ),
        "cargo run -p xtask -- support-matrix --check".to_string(),
        "cargo run -p xtask -- capability-matrix --check".to_string(),
        "cargo run -p xtask -- capability-matrix-audit".to_string(),
        "make preflight".to_string(),
    ]
}

fn execution_writable_surfaces(
    entry: &AgentRegistryEntry,
    maintenance_root: &str,
    args: &Args,
) -> Vec<String> {
    let mut writable_surfaces = vec![
        format!("{maintenance_root}/**"),
        format!("{}/**", entry.crate_path),
        "crates/agent_api/**".to_string(),
        format!("{}/artifacts.lock.json", entry.manifest_root),
        format!(
            "{}/snapshots/{}/**",
            entry.manifest_root, args.target_version
        ),
        format!("{}/reports/{}/**", entry.manifest_root, args.target_version),
        format!(
            "{}/versions/{}.json",
            entry.manifest_root, args.target_version
        ),
        format!("{}/wrapper_coverage.json", entry.manifest_root),
        "cli_manifests/support_matrix/current.json".to_string(),
        "docs/specs/unified-agent-api/support-matrix.md".to_string(),
    ];
    if entry.agent_id == "codex" {
        writable_surfaces.push("docs/specs/codex-wrapper-coverage-scenarios-v1.md".to_string());
    }
    writable_surfaces
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

fn dispatch_workflow_value(args: &Args) -> String {
    args.dispatch_workflow
        .clone()
        .unwrap_or_else(|| GENERIC_PACKET_PR_WORKFLOW.to_string())
}

fn source_ref(release_watch: &ReleaseWatchMetadata) -> String {
    match release_watch.upstream.source_kind {
        ReleaseWatchSourceKind::GithubReleases => format!(
            "{}/{}",
            release_watch.upstream.owner.as_deref().unwrap_or(""),
            release_watch.upstream.repo.as_deref().unwrap_or("")
        ),
        ReleaseWatchSourceKind::GcsObjectListing => format!(
            "{}/{}",
            release_watch.upstream.bucket.as_deref().unwrap_or(""),
            release_watch.upstream.prefix.as_deref().unwrap_or("")
        ),
    }
}

fn source_kind_str(kind: ReleaseWatchSourceKind) -> &'static str {
    match kind {
        ReleaseWatchSourceKind::GithubReleases => "github_releases",
        ReleaseWatchSourceKind::GcsObjectListing => "gcs_object_listing",
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
