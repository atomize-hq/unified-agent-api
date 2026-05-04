use std::{
    fmt,
    io::{stdout, Write},
    path::{Path, PathBuf},
};

use clap::{ArgGroup, Parser, ValueEnum};
use serde::{Deserialize, Serialize};

pub const RESEARCH_PACKET_ROOT: &str =
    "docs/agents/.uaa-temp/recommend-next-agent/research-runs";
pub const DISCOVERY_ROOT: &str = "docs/agents/.uaa-temp/recommend-next-agent/discovery";
pub const RESEARCH_ROOT: &str = "docs/agents/.uaa-temp/recommend-next-agent/research";
pub const PYTHON_RUNS_ROOT: &str = "docs/agents/.uaa-temp/recommend-next-agent/runs";
pub const DOSSIER_CONTRACT_PATH: &str =
    "docs/specs/cli-agent-recommendation-dossier-contract.md";
pub const INPUT_CONTRACT_FILE_NAME: &str = "input-contract.json";
pub const DISCOVERY_PROMPT_FILE_NAME: &str = "discovery-prompt.md";
pub const RESEARCH_PROMPT_FILE_NAME: &str = "research-prompt.md";
pub const DISCOVERY_EXECUTION_FILE_NAME: &str = "codex-execution.discovery.json";
pub const RESEARCH_EXECUTION_FILE_NAME: &str = "codex-execution.research.json";
pub const DISCOVERY_STDOUT_FILE_NAME: &str = "codex-stdout.discovery.log";
pub const DISCOVERY_STDERR_FILE_NAME: &str = "codex-stderr.discovery.log";
pub const RESEARCH_STDOUT_FILE_NAME: &str = "codex-stdout.research.log";
pub const RESEARCH_STDERR_FILE_NAME: &str = "codex-stderr.research.log";
pub const DISCOVERY_WRITTEN_PATHS_FILE_NAME: &str = "written-paths.discovery.json";
pub const RESEARCH_WRITTEN_PATHS_FILE_NAME: &str = "written-paths.research.json";
pub const VALIDATION_REPORT_FILE_NAME: &str = "validation-report.json";
pub const RUN_STATUS_FILE_NAME: &str = "run-status.json";
pub const RUN_SUMMARY_FILE_NAME: &str = "run-summary.md";
pub const WORKFLOW_VERSION: &str = "recommend_next_agent_research_v1";
pub const CODEX_BINARY_ENV: &str = "XTASK_RECOMMEND_NEXT_AGENT_RESEARCH_CODEX_BINARY";
pub const PASS1_QUERY_FAMILY: [&str; 3] = [
    "best AI coding CLI",
    "AI agent CLI tools",
    "developer agent command line",
];
pub const PASS2_QUERY_FAMILY_WITH_SURVIVOR: [&str; 3] = [
    "alternatives to <top surviving candidate>",
    "top coding agent CLI open source",
    "CLI coding assistant blog",
];
pub const PASS2_QUERY_FAMILY_ZERO_SURVIVOR: [&str; 2] = [
    "top coding agent CLI open source",
    "CLI coding assistant blog",
];
pub const PROVING_FLOW_ORDER: [&str; 5] = [
    "dry-run pass1",
    "write pass1",
    "generate chosen pass1 or evaluate insufficiency",
    "optional dry-run/write/generate pass2 with fresh run id",
    "promote only after parent review accepts exactly one sufficient run",
];
pub const DISCOVERY_REQUIRED_FILES: [&str; 3] = [
    "candidate-seed.generated.toml",
    "discovery-summary.md",
    "sources.lock.json",
];
pub const RESEARCH_REQUIRED_FILES: [&str; 3] = [
    "seed.snapshot.toml",
    "research-summary.md",
    "research-metadata.json",
];

#[derive(Debug, Parser, Clone)]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(["dry_run", "write"])
        .multiple(false)
))]
pub struct Args {
    /// Materialize the execution packet without invoking Codex.
    #[arg(long)]
    pub dry_run: bool,

    /// Execute the frozen discovery and research flow against an existing dry-run packet.
    #[arg(long)]
    pub write: bool,

    /// Required pass selector. Pass2 requires prior insufficiency context and a fresh run id.
    #[arg(long, value_enum)]
    pub pass: Pass,

    /// Stable run identifier. Required for `--write`; optional for `--dry-run`.
    #[arg(long)]
    pub run_id: Option<String>,

    /// Required for `pass2`; forbidden for `pass1`.
    #[arg(long)]
    pub prior_run_dir: Option<String>,

    /// Explicit `codex` binary path. Falls back to XTASK_RECOMMEND_NEXT_AGENT_RESEARCH_CODEX_BINARY, then `codex`.
    #[arg(long)]
    pub codex_binary: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Pass {
    Pass1,
    Pass2,
}

impl Pass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass1 => "pass1",
            Self::Pass2 => "pass2",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunContract {
    pub workflow_version: String,
    pub pass: String,
    pub packet_root: String,
    pub discovery_root: String,
    pub research_root: String,
    pub python_runs_root: String,
    pub required_packet_files: Vec<String>,
    pub discovery_required_files: Vec<String>,
    pub research_required_files: Vec<String>,
    pub proving_flow_order: Vec<String>,
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

pub fn run(args: Args) -> Result<(), Error> {
    let workspace_root = resolve_workspace_root()?;
    let current_dir = std::env::current_dir()
        .map_err(|err| Error::Internal(format!("resolve current directory: {err}")))?;
    if current_dir != workspace_root {
        return Err(Error::Validation(format!(
            "recommend-next-agent-research must run with cwd = repo root `{}` (got `{}`)",
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
    validate_args(workspace_root, &args)?;
    let run_id = args.run_id.clone().unwrap_or_else(|| "<generated>".to_string());
    let contract = render_contract(workspace_root, &args, &run_id);

    writeln!(
        writer,
        "recommend-next-agent-research contract frozen for {}",
        args.pass.as_str()
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "mode: {}", if args.dry_run { "dry-run" } else { "write" })
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "run_id: {run_id}")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "packet_root: {}", contract.packet_root)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "status: Step 1 contract only; packet rendering and write execution remain pending"
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

pub fn render_contract(workspace_root: &Path, args: &Args, run_id: &str) -> RunContract {
    RunContract {
        workflow_version: WORKFLOW_VERSION.to_string(),
        pass: args.pass.as_str().to_string(),
        packet_root: packet_root(workspace_root, run_id).display().to_string(),
        discovery_root: discovery_root(workspace_root, run_id).display().to_string(),
        research_root: research_root(workspace_root, run_id).display().to_string(),
        python_runs_root: workspace_root.join(PYTHON_RUNS_ROOT).display().to_string(),
        required_packet_files: packet_file_names(),
        discovery_required_files: DISCOVERY_REQUIRED_FILES
            .iter()
            .map(ToString::to_string)
            .collect(),
        research_required_files: RESEARCH_REQUIRED_FILES
            .iter()
            .map(ToString::to_string)
            .collect(),
        proving_flow_order: PROVING_FLOW_ORDER
            .iter()
            .map(ToString::to_string)
            .collect(),
    }
}

pub fn packet_file_names() -> Vec<String> {
    [
        INPUT_CONTRACT_FILE_NAME,
        DISCOVERY_PROMPT_FILE_NAME,
        RESEARCH_PROMPT_FILE_NAME,
        DISCOVERY_EXECUTION_FILE_NAME,
        RESEARCH_EXECUTION_FILE_NAME,
        DISCOVERY_STDOUT_FILE_NAME,
        DISCOVERY_STDERR_FILE_NAME,
        RESEARCH_STDOUT_FILE_NAME,
        RESEARCH_STDERR_FILE_NAME,
        DISCOVERY_WRITTEN_PATHS_FILE_NAME,
        RESEARCH_WRITTEN_PATHS_FILE_NAME,
        VALIDATION_REPORT_FILE_NAME,
        RUN_STATUS_FILE_NAME,
        RUN_SUMMARY_FILE_NAME,
    ]
    .iter()
    .map(ToString::to_string)
    .collect()
}

pub fn packet_root(workspace_root: &Path, run_id: &str) -> PathBuf {
    workspace_root.join(RESEARCH_PACKET_ROOT).join(run_id)
}

pub fn discovery_root(workspace_root: &Path, run_id: &str) -> PathBuf {
    workspace_root.join(DISCOVERY_ROOT).join(run_id)
}

pub fn research_root(workspace_root: &Path, run_id: &str) -> PathBuf {
    workspace_root.join(RESEARCH_ROOT).join(run_id)
}

fn validate_args(workspace_root: &Path, args: &Args) -> Result<(), Error> {
    if args.write && args.run_id.is_none() {
        return Err(Error::Validation(
            "--run-id is required with --write so the command can validate against a prepared dry-run packet"
                .to_string(),
        ));
    }

    match args.pass {
        Pass::Pass1 => {
            if args.prior_run_dir.is_some() {
                return Err(Error::Validation(
                    "--prior-run-dir is only valid with --pass pass2".to_string(),
                ));
            }
        }
        Pass::Pass2 => {
            let prior_run_dir = args.prior_run_dir.as_deref().ok_or_else(|| {
                Error::Validation(
                    "--prior-run-dir is required with --pass pass2 because pass2 must consume prior insufficiency output"
                        .to_string(),
                )
            })?;
            let prior_run_path = workspace_root.join(prior_run_dir);
            if !prior_run_path.exists() {
                return Err(Error::Validation(format!(
                    "prior run directory `{prior_run_dir}` does not exist"
                )));
            }
        }
    }

    if args.write {
        let run_id = args.run_id.as_deref().expect("write run_id checked above");
        let packet_dir = packet_root(workspace_root, run_id);
        let input_contract = packet_dir.join(INPUT_CONTRACT_FILE_NAME);
        if !input_contract.is_file() {
            return Err(Error::Validation(format!(
                "--write requires a matching dry-run packet for run_id `{run_id}`; missing `{}`",
                input_contract.display()
            )));
        }
    }

    Ok(())
}

fn resolve_workspace_root() -> Result<PathBuf, Error> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| Error::Internal("resolve workspace root".to_string()))
}
