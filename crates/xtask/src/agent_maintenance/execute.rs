#[path = "execute/packet.rs"]
mod packet;
#[path = "execute/runtime.rs"]
mod runtime;
#[path = "execute/types.rs"]
mod types;
#[path = "execute/validate.rs"]
mod validate;
#[path = "execute/workflow.rs"]
mod workflow;

use std::{
    fmt, fs,
    io::{stdout, Write},
    path::{Path, PathBuf},
};

use clap::{ArgGroup, Parser};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::workspace_mutation::WorkspaceMutationError;

use super::request;

pub(super) const EXECUTION_RUNS_ROOT: &str = "docs/agents/.uaa-temp/agent-maintenance/runs";
pub(super) const INPUT_CONTRACT_FILE_NAME: &str = "input-contract.json";
pub(super) const PROMPT_FILE_NAME: &str = "codex-prompt.md";
pub(super) const RUN_STATUS_FILE_NAME: &str = "run-status.json";
pub(super) const RUN_SUMMARY_FILE_NAME: &str = "run-summary.md";
pub(super) const VALIDATION_REPORT_FILE_NAME: &str = "validation-report.json";
pub(super) const WRITTEN_PATHS_FILE_NAME: &str = "written-paths.json";
pub(super) const CODEX_EXECUTION_FILE_NAME: &str = "codex-execution.json";
pub(super) const CODEX_STDOUT_FILE_NAME: &str = "codex-stdout.log";
pub(super) const CODEX_STDERR_FILE_NAME: &str = "codex-stderr.log";
pub(super) const WORKFLOW_VERSION: &str = "agent_maintenance_execute_v1";
pub(super) const CODEX_BINARY_ENV: &str = "XTASK_AGENT_MAINTENANCE_CODEX_BINARY";
pub(super) const PREFLIGHT_SENTINEL: &str = "UAA_AGENT_MAINTENANCE_PREFLIGHT_OK";
pub(super) const EXECUTE_HOST_SURFACE: &str = "execute-agent-maintenance";

#[derive(Debug, Parser, Clone)]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(["dry_run", "write"])
        .multiple(false)
))]
pub struct Args {
    #[arg(long)]
    pub request: PathBuf,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub write: bool,
    #[arg(long)]
    pub run_id: Option<String>,
    #[arg(long)]
    pub codex_binary: Option<String>,
}

#[derive(Debug)]
pub enum Error {
    Validation(String),
    Internal(String),
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation(_) => 2,
            Self::Internal(_) => 1,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(message) | Self::Internal(message) => f.write_str(message),
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

impl From<WorkspaceMutationError> for Error {
    fn from(value: WorkspaceMutationError) -> Self {
        match value {
            WorkspaceMutationError::Validation(message) => Self::Validation(message),
            WorkspaceMutationError::Internal(message) => Self::Internal(message),
        }
    }
}

pub fn run(args: Args) -> Result<(), Error> {
    let workspace_root = resolve_workspace_root()?;
    let current_dir = std::env::current_dir()
        .map_err(|err| Error::Internal(format!("resolve current directory: {err}")))?;
    if current_dir != workspace_root {
        return Err(Error::Validation(format!(
            "execute-agent-maintenance must run with cwd = repo root `{}` (got `{}`)",
            workspace_root.display(),
            current_dir.display()
        )));
    }

    let mut writer = stdout();
    run_in_workspace(&workspace_root, args, &mut writer)
}

pub fn run_in_workspace<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
) -> Result<(), Error> {
    validate_args(&args)?;
    let context = workflow::build_context(workspace_root, &args)?;
    packet::write_preview(writer, &context, args.write)?;

    if args.dry_run {
        return workflow::execute_dry_run(workspace_root, &context, writer);
    }

    workflow::execute_write_mode(workspace_root, &context, writer)
}

fn validate_args(args: &Args) -> Result<(), Error> {
    if args.write && args.run_id.is_none() {
        return Err(Error::Validation(
            "--run-id is required with --write so the relay can validate against one prepared dry-run baseline".to_string(),
        ));
    }
    Ok(())
}

fn resolve_workspace_root() -> Result<PathBuf, Error> {
    let current_dir = std::env::current_dir()
        .map_err(|err| Error::Internal(format!("resolve current directory: {err}")))?;
    for candidate in current_dir.ancestors() {
        let cargo_toml = candidate.join("Cargo.toml");
        let Ok(text) = fs::read_to_string(&cargo_toml) else {
            continue;
        };
        if text.contains("[workspace]") {
            return Ok(candidate.to_path_buf());
        }
    }
    Err(Error::Internal(format!(
        "could not resolve workspace root from {}",
        current_dir.display()
    )))
}

pub(super) fn resolve_codex_binary(args: &Args) -> String {
    args.codex_binary
        .clone()
        .or_else(|| std::env::var(CODEX_BINARY_ENV).ok())
        .unwrap_or_else(|| "codex".to_string())
}

pub(super) fn generate_run_id() -> String {
    OffsetDateTime::now_utc()
        .format(
            &time::format_description::parse("[year][month][day]T[hour][minute][second]Z")
                .expect("valid time format"),
        )
        .unwrap_or_else(|_| "agent-maintenance-execute".to_string())
}

pub(super) fn now_rfc3339() -> Result<String, Error> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|err| Error::Internal(format!("format timestamp: {err}")))
}
