use std::{
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
    docs::build_packet_docs,
    request::{
        self, validate_commit_value, validate_non_empty_scalar, validate_repo_relative_reference,
        DetectedRelease, MaintenanceAction, MaintenanceRequest, RuntimeFollowupRequired,
        TriggerKind, AUTOMATED_ARTIFACT_VERSION,
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
    let request_bytes = render_request_toml(entry, release_watch, args).into_bytes();
    let request = MaintenanceRequest {
        relative_path: request_path.clone(),
        canonical_path: workspace_root.join(&request_path),
        sha256: hex::encode(Sha256::digest(&request_bytes)),
        maintenance_pack_prefix: format!("{}-maintenance", args.agent),
        maintenance_root: format!("docs/agents/lifecycle/{}-maintenance", args.agent),
        agent_id: args.agent.clone(),
        trigger_kind: TriggerKind::UpstreamReleaseDetected,
        basis_ref: format!("{}/latest_validated.txt", entry.manifest_root),
        opened_from: args.opened_from.display().to_string(),
        requested_control_plane_actions: vec![MaintenanceAction::PacketDocRefresh],
        runtime_followup_required: RuntimeFollowupRequired {
            required: false,
            items: Vec::new(),
        },
        detected_release: Some(DetectedRelease {
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
        }),
        request_recorded_at: args.request_recorded_at.clone(),
        request_commit: args.request_commit.clone(),
    };

    let mut files = vec![PreparedFile {
        relative_path: request.relative_path.clone(),
        contents: request_bytes,
    }];
    for doc in build_packet_docs(workspace_root, &request)
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
    entry: &AgentRegistryEntry,
    release_watch: &ReleaseWatchMetadata,
    args: &Args,
) -> String {
    format!(
        concat!(
            "artifact_version = \"{artifact_version}\"\n",
            "agent_id = \"{agent_id}\"\n",
            "trigger_kind = \"upstream_release_detected\"\n",
            "basis_ref = \"{basis_ref}\"\n",
            "opened_from = \"{opened_from}\"\n",
            "requested_control_plane_actions = [\n",
            "  \"packet_doc_refresh\",\n",
            "]\n",
            "request_recorded_at = \"{request_recorded_at}\"\n",
            "request_commit = \"{request_commit}\"\n",
            "\n",
            "[runtime_followup_required]\n",
            "required = false\n",
            "items = []\n",
            "\n",
            "[detected_release]\n",
            "detected_by = \"{detected_by}\"\n",
            "current_validated = \"{current_version}\"\n",
            "target_version = \"{target_version}\"\n",
            "latest_stable = \"{latest_stable}\"\n",
            "version_policy = \"latest_stable_minus_one\"\n",
            "source_kind = \"{source_kind}\"\n",
            "source_ref = \"{source_ref}\"\n",
            "dispatch_kind = \"{dispatch_kind}\"\n",
            "dispatch_workflow = \"{dispatch_workflow}\"\n",
            "branch_name = \"{branch_name}\"\n"
        ),
        artifact_version = AUTOMATED_ARTIFACT_VERSION,
        agent_id = args.agent,
        basis_ref = format!("{}/latest_validated.txt", entry.manifest_root),
        opened_from = args.opened_from.display(),
        request_recorded_at = args.request_recorded_at,
        request_commit = args.request_commit,
        detected_by = args.detected_by,
        current_version = args.current_version,
        target_version = args.target_version,
        latest_stable = args.latest_stable,
        source_kind = source_kind_str(release_watch.upstream.source_kind),
        source_ref = source_ref(release_watch),
        dispatch_kind = args.dispatch_kind,
        dispatch_workflow = dispatch_workflow_value(args),
        branch_name = args.branch_name,
    )
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
