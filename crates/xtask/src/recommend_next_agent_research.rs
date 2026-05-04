use std::{
    collections::BTreeSet,
    fmt, fs,
    io::{stdout, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use clap::{ArgGroup, Parser, ValueEnum};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::agent_registry::AgentRegistry;

pub const RESEARCH_PACKET_ROOT: &str =
    "docs/agents/.uaa-temp/recommend-next-agent/research-runs";
pub const DISCOVERY_ROOT: &str = "docs/agents/.uaa-temp/recommend-next-agent/discovery";
pub const RESEARCH_ROOT: &str = "docs/agents/.uaa-temp/recommend-next-agent/research";
pub const PYTHON_RUNS_ROOT: &str = "docs/agents/.uaa-temp/recommend-next-agent/runs";
pub const DISCOVERY_HINTS_PATH: &str = "docs/agents/selection/discovery-hints.json";
pub const LIVE_SEED_PATH: &str = "docs/agents/selection/candidate-seed.toml";
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
pub struct InputContract {
    pub workflow_version: String,
    pub run_id: String,
    pub pass: String,
    pub prior_run_dir: Option<String>,
    pub packet_root: String,
    pub discovery_root: String,
    pub research_root: String,
    pub python_runs_root: String,
    pub discovery_hints_path: Option<String>,
    pub live_seed_path: String,
    pub dossier_contract_path: String,
    pub required_packet_files: Vec<String>,
    pub discovery_required_files: Vec<String>,
    pub research_required_files: Vec<String>,
    pub query_family: Vec<String>,
    pub excluded_candidate_ids: Vec<String>,
    pub top_surviving_candidate: Option<String>,
    pub onboarded_agent_ids: Vec<String>,
    pub proving_flow_order: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub name: String,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubprocessEvidence {
    pub binary: String,
    pub argv: Vec<String>,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub workflow_version: String,
    pub run_id: String,
    pub status: String,
    pub checks: Vec<ValidationCheck>,
    pub errors: Vec<String>,
    pub freeze_discovery: Option<SubprocessEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStatus {
    pub workflow_version: String,
    pub generated_at: String,
    pub run_id: String,
    pub host_surface: String,
    pub mode: String,
    pub pass: String,
    pub status: String,
    pub validation_passed: bool,
    pub packet_root: String,
    pub discovery_root: String,
    pub research_root: String,
    pub prior_run_dir: Option<String>,
    pub written_paths: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexExecutionEvidence {
    pub workflow_version: String,
    pub generated_at: String,
    pub run_id: String,
    pub phase: String,
    pub binary: String,
    pub argv: Vec<String>,
    pub prompt_path: String,
    pub stdout_path: String,
    pub stderr_path: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnapshotFile {
    path: String,
    sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceSnapshot {
    files: Vec<SnapshotFile>,
}

#[derive(Debug, Clone)]
struct Context {
    run_id: String,
    pass: Pass,
    prior_run_dir: Option<String>,
    codex_binary: String,
    packet_dir: PathBuf,
    packet_dir_rel: String,
    discovery_dir: PathBuf,
    discovery_dir_rel: String,
    research_dir: PathBuf,
    research_dir_rel: String,
    input_contract: InputContract,
}

#[derive(Debug, Clone, Copy)]
enum Phase {
    Discovery,
    Research,
}

impl Phase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Discovery => "discovery",
            Self::Research => "research",
        }
    }

    fn prompt_file_name(self) -> &'static str {
        match self {
            Self::Discovery => DISCOVERY_PROMPT_FILE_NAME,
            Self::Research => RESEARCH_PROMPT_FILE_NAME,
        }
    }

    fn execution_file_name(self) -> &'static str {
        match self {
            Self::Discovery => DISCOVERY_EXECUTION_FILE_NAME,
            Self::Research => RESEARCH_EXECUTION_FILE_NAME,
        }
    }

    fn stdout_file_name(self) -> &'static str {
        match self {
            Self::Discovery => DISCOVERY_STDOUT_FILE_NAME,
            Self::Research => RESEARCH_STDOUT_FILE_NAME,
        }
    }

    fn stderr_file_name(self) -> &'static str {
        match self {
            Self::Discovery => DISCOVERY_STDERR_FILE_NAME,
            Self::Research => RESEARCH_STDERR_FILE_NAME,
        }
    }

    fn written_paths_file_name(self) -> &'static str {
        match self {
            Self::Discovery => DISCOVERY_WRITTEN_PATHS_FILE_NAME,
            Self::Research => RESEARCH_WRITTEN_PATHS_FILE_NAME,
        }
    }
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
    let context = build_context(workspace_root, &args)?;
    write_header(writer, &context, args.write)?;

    if args.dry_run {
        persist_dry_run_packet(&context)?;
        writeln!(writer, "OK: recommend-next-agent-research dry-run packet prepared.")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        writeln!(writer, "run_id: {}", context.run_id)
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        writeln!(writer, "run_dir: {}", context.packet_dir_rel)
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        return Ok(());
    }

    let persisted = load_json::<InputContract>(&context.packet_dir.join(INPUT_CONTRACT_FILE_NAME))?;
    validate_matching_contract(&context, &persisted)?;
    execute_write_mode(workspace_root, &context, writer)
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

fn build_context(workspace_root: &Path, args: &Args) -> Result<Context, Error> {
    validate_args(workspace_root, args)?;

    let run_id = args.run_id.clone().unwrap_or_else(generate_run_id);
    let packet_dir = packet_root(workspace_root, &run_id);
    let discovery_dir = discovery_root(workspace_root, &run_id);
    let research_dir = research_root(workspace_root, &run_id);
    let packet_dir_rel = packet_root_rel(&run_id);
    let discovery_dir_rel = discovery_root_rel(&run_id);
    let research_dir_rel = research_root_rel(&run_id);
    let registry = AgentRegistry::load(workspace_root)
        .map_err(|err| Error::Validation(format!("load agent registry: {err}")))?;
    let mut onboarded_agent_ids = registry
        .agents
        .iter()
        .map(|entry| entry.agent_id.clone())
        .collect::<Vec<_>>();
    onboarded_agent_ids.sort();

    let pass2_state = if matches!(args.pass, Pass::Pass2) {
        Some(load_pass2_state(
            workspace_root,
            args.prior_run_dir
                .as_deref()
                .expect("pass2 prior_run_dir validated"),
            args.run_id.as_deref(),
        )?)
    } else {
        None
    };

    let discovery_hints_path = workspace_root
        .join(DISCOVERY_HINTS_PATH)
        .is_file()
        .then(|| DISCOVERY_HINTS_PATH.to_string());
    let query_family = match args.pass {
        Pass::Pass1 => PASS1_QUERY_FAMILY
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        Pass::Pass2 if pass2_state.as_ref().is_some_and(|state| state.zero_survivors) => {
            PASS2_QUERY_FAMILY_ZERO_SURVIVOR
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        }
        Pass::Pass2 => PASS2_QUERY_FAMILY_WITH_SURVIVOR
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
    };
    let input_contract = InputContract {
        workflow_version: WORKFLOW_VERSION.to_string(),
        run_id: run_id.clone(),
        pass: args.pass.as_str().to_string(),
        prior_run_dir: args.prior_run_dir.clone(),
        packet_root: packet_dir_rel.clone(),
        discovery_root: discovery_dir_rel.clone(),
        research_root: research_dir_rel.clone(),
        python_runs_root: PYTHON_RUNS_ROOT.to_string(),
        discovery_hints_path,
        live_seed_path: LIVE_SEED_PATH.to_string(),
        dossier_contract_path: DOSSIER_CONTRACT_PATH.to_string(),
        required_packet_files: packet_file_names(),
        discovery_required_files: DISCOVERY_REQUIRED_FILES
            .iter()
            .map(ToString::to_string)
            .collect(),
        research_required_files: RESEARCH_REQUIRED_FILES
            .iter()
            .map(ToString::to_string)
            .collect(),
        query_family,
        excluded_candidate_ids: pass2_state
            .as_ref()
            .map(|state| state.excluded_candidate_ids.clone())
            .unwrap_or_default(),
        top_surviving_candidate: pass2_state
            .as_ref()
            .and_then(|state| state.top_surviving_candidate.clone()),
        onboarded_agent_ids,
        proving_flow_order: PROVING_FLOW_ORDER
            .iter()
            .map(ToString::to_string)
            .collect(),
    };

    Ok(Context {
        run_id,
        pass: args.pass,
        prior_run_dir: args.prior_run_dir.clone(),
        codex_binary: resolve_codex_binary(args),
        packet_dir,
        packet_dir_rel,
        discovery_dir,
        discovery_dir_rel,
        research_dir,
        research_dir_rel,
        input_contract,
    })
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
            if !prior_run_path.is_dir() {
                return Err(Error::Validation(format!(
                    "prior run directory `{prior_run_dir}` does not exist"
                )));
            }
            let prior_basename = prior_run_path
                .file_name()
                .and_then(|value| value.to_str())
                .ok_or_else(|| {
                    Error::Validation(format!(
                        "prior run directory `{prior_run_dir}` must end with a run id basename"
                    ))
                })?;
            if args.run_id.as_deref() == Some(prior_basename) {
                return Err(Error::Validation(
                    "pass2 must use a fresh run_id instead of reusing the prior insufficiency run id"
                        .to_string(),
                ));
            }
            validate_prior_run_for_pass2(&prior_run_path)?;
        }
    }

    if args.write {
        let run_id = args.run_id.as_deref().expect("write run_id checked above");
        let input_contract = packet_root(workspace_root, run_id).join(INPUT_CONTRACT_FILE_NAME);
        if !input_contract.is_file() {
            return Err(Error::Validation(format!(
                "--write requires a matching dry-run packet for run_id `{run_id}`; missing `{}`",
                input_contract.display()
            )));
        }
    }

    Ok(())
}

fn execute_write_mode<W: Write>(
    workspace_root: &Path,
    context: &Context,
    writer: &mut W,
) -> Result<(), Error> {
    let mut report = ValidationReport {
        workflow_version: WORKFLOW_VERSION.to_string(),
        run_id: context.run_id.clone(),
        status: "in_progress".to_string(),
        checks: Vec::new(),
        errors: Vec::new(),
        freeze_discovery: None,
    };
    let mut discovery_written_paths = Vec::new();
    let mut research_written_paths = Vec::new();
    let mut discovery_execution: Option<CodexExecutionEvidence> = None;
    let mut research_execution: Option<CodexExecutionEvidence> = None;

    let outcome = (|| -> Result<(), Error> {
        let discovery_prompt =
            read_string(&context.packet_dir.join(DISCOVERY_PROMPT_FILE_NAME))?;
        let before_discovery =
            snapshot_workspace(workspace_root, &[context.packet_dir.as_path()])?;
        let executed_discovery = execute_codex_phase(
            workspace_root,
            context,
            Phase::Discovery,
            &discovery_prompt,
        )?;
        write_json(
            &context
                .packet_dir
                .join(Phase::Discovery.execution_file_name()),
            &executed_discovery,
        )?;
        discovery_execution = Some(executed_discovery.clone());
        let after_discovery =
            snapshot_workspace(workspace_root, &[context.packet_dir.as_path()])?;
        discovery_written_paths = diff_snapshots(&before_discovery, &after_discovery);
        write_json(
            &context
                .packet_dir
                .join(Phase::Discovery.written_paths_file_name()),
            &discovery_written_paths,
        )?;
        if executed_discovery.exit_code != 0 {
            let message = format!(
                "Codex discovery execution failed with exit code {}",
                executed_discovery.exit_code
            );
            push_failed_check(&mut report, "codex_discovery_execution", message.clone());
            return Err(Error::Validation(message));
        }
        push_passed_check(
            &mut report,
            "codex_discovery_execution",
            format!(
                "Codex discovery completed with {} changed paths",
                discovery_written_paths.len()
            ),
        );
        validate_written_paths(
            &discovery_written_paths,
            &context.discovery_dir_rel,
            "discovery",
        )
        .map_err(|message| {
            push_failed_check(&mut report, "discovery_write_boundary", message.clone());
            Error::Validation(message)
        })?;
        push_passed_check(
            &mut report,
            "discovery_write_boundary",
            format!("Discovery writes stayed under `{}`", context.discovery_dir_rel),
        );
        validate_discovery_artifacts(&context.discovery_dir).map_err(|message| {
            push_failed_check(&mut report, "discovery_artifact_set", message.clone());
            Error::Validation(message)
        })?;
        push_passed_check(
            &mut report,
            "discovery_artifact_set",
            "Discovery artifact set matches the frozen contract".to_string(),
        );

        let freeze = execute_freeze_discovery(workspace_root, context)?;
        report.freeze_discovery = Some(freeze.clone());
        if freeze.exit_code != 0 {
            let message = format!(
                "freeze-discovery failed with exit code {}: {}{}",
                freeze.exit_code, freeze.stdout, freeze.stderr
            );
            push_failed_check(&mut report, "freeze_discovery", message.clone());
            return Err(Error::Validation(message));
        }
        push_passed_check(
            &mut report,
            "freeze_discovery",
            "freeze-discovery completed successfully".to_string(),
        );
        validate_frozen_seed_boundary(&context.research_dir).map_err(|message| {
            push_failed_check(&mut report, "frozen_seed_boundary", message.clone());
            Error::Validation(message)
        })?;
        push_passed_check(
            &mut report,
            "frozen_seed_boundary",
            "Research root contains frozen seed and copied discovery inputs".to_string(),
        );

        let research_prompt = render_research_prompt(context);
        write_string(
            &context.packet_dir.join(RESEARCH_PROMPT_FILE_NAME),
            &research_prompt,
        )?;
        let before_research =
            snapshot_workspace(workspace_root, &[context.packet_dir.as_path()])?;
        let executed_research = execute_codex_phase(
            workspace_root,
            context,
            Phase::Research,
            &research_prompt,
        )?;
        write_json(
            &context.packet_dir.join(Phase::Research.execution_file_name()),
            &executed_research,
        )?;
        research_execution = Some(executed_research.clone());
        let after_research =
            snapshot_workspace(workspace_root, &[context.packet_dir.as_path()])?;
        research_written_paths = diff_snapshots(&before_research, &after_research);
        write_json(
            &context
                .packet_dir
                .join(Phase::Research.written_paths_file_name()),
            &research_written_paths,
        )?;
        if executed_research.exit_code != 0 {
            let message = format!(
                "Codex research execution failed with exit code {}",
                executed_research.exit_code
            );
            push_failed_check(&mut report, "codex_research_execution", message.clone());
            return Err(Error::Validation(message));
        }
        push_passed_check(
            &mut report,
            "codex_research_execution",
            format!(
                "Codex research completed with {} changed paths",
                research_written_paths.len()
            ),
        );
        validate_written_paths(
            &research_written_paths,
            &context.research_dir_rel,
            "research",
        )
        .map_err(|message| {
            push_failed_check(&mut report, "research_write_boundary", message.clone());
            Error::Validation(message)
        })?;
        push_passed_check(
            &mut report,
            "research_write_boundary",
            format!("Research writes stayed under `{}`", context.research_dir_rel),
        );
        validate_research_tree(&context.research_dir).map_err(|message| {
            push_failed_check(&mut report, "research_identity_validation", message.clone());
            Error::Validation(message)
        })?;
        push_passed_check(
            &mut report,
            "research_identity_validation",
            "Research tree passed seed and dossier identity validation".to_string(),
        );
        Ok(())
    })();

    if let Err(err) = &outcome {
        report.status = "fail".to_string();
        report.errors.push(err.to_string());
    } else {
        report.status = "pass".to_string();
    }

    write_json(
        &context.packet_dir.join(VALIDATION_REPORT_FILE_NAME),
        &report,
    )?;
    let mut all_written_paths = discovery_written_paths.clone();
    all_written_paths.extend(research_written_paths.clone());
    let run_status = RunStatus {
        workflow_version: WORKFLOW_VERSION.to_string(),
        generated_at: now_rfc3339()?,
        run_id: context.run_id.clone(),
        host_surface: "xtask recommend-next-agent-research".to_string(),
        mode: "write".to_string(),
        pass: context.pass.as_str().to_string(),
        status: if report.status == "pass" {
            "write_validated".to_string()
        } else {
            "write_failed".to_string()
        },
        validation_passed: report.status == "pass",
        packet_root: context.packet_dir_rel.clone(),
        discovery_root: context.discovery_dir_rel.clone(),
        research_root: context.research_dir_rel.clone(),
        prior_run_dir: context.prior_run_dir.clone(),
        written_paths: all_written_paths.clone(),
        errors: report.errors.clone(),
    };
    write_json(&context.packet_dir.join(RUN_STATUS_FILE_NAME), &run_status)?;
    write_string(
        &context.packet_dir.join(RUN_SUMMARY_FILE_NAME),
        &render_write_summary(
            context,
            &report,
            &discovery_written_paths,
            &research_written_paths,
            discovery_execution.as_ref(),
            research_execution.as_ref(),
        ),
    )?;

    if let Err(err) = outcome {
        return Err(err);
    }

    writeln!(writer, "OK: recommend-next-agent-research write validation complete.")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "run_id: {}", context.run_id)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "validated_paths: {}", all_written_paths.len())
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

fn persist_dry_run_packet(context: &Context) -> Result<(), Error> {
    write_json(
        &context.packet_dir.join(INPUT_CONTRACT_FILE_NAME),
        &context.input_contract,
    )?;
    write_string(
        &context.packet_dir.join(DISCOVERY_PROMPT_FILE_NAME),
        &render_discovery_prompt(context),
    )?;
    write_string(
        &context.packet_dir.join(RESEARCH_PROMPT_FILE_NAME),
        &render_research_prompt(context),
    )?;
    write_json(
        &context.packet_dir.join(DISCOVERY_EXECUTION_FILE_NAME),
        &serde_json::json!({"status": "pending", "phase": "discovery"}),
    )?;
    write_json(
        &context.packet_dir.join(RESEARCH_EXECUTION_FILE_NAME),
        &serde_json::json!({"status": "pending", "phase": "research"}),
    )?;
    write_string(&context.packet_dir.join(DISCOVERY_STDOUT_FILE_NAME), "")?;
    write_string(&context.packet_dir.join(DISCOVERY_STDERR_FILE_NAME), "")?;
    write_string(&context.packet_dir.join(RESEARCH_STDOUT_FILE_NAME), "")?;
    write_string(&context.packet_dir.join(RESEARCH_STDERR_FILE_NAME), "")?;
    write_json(
        &context.packet_dir.join(DISCOVERY_WRITTEN_PATHS_FILE_NAME),
        &Vec::<String>::new(),
    )?;
    write_json(
        &context.packet_dir.join(RESEARCH_WRITTEN_PATHS_FILE_NAME),
        &Vec::<String>::new(),
    )?;
    let report = ValidationReport {
        workflow_version: WORKFLOW_VERSION.to_string(),
        run_id: context.run_id.clone(),
        status: "dry_run_prepared".to_string(),
        checks: vec![ValidationCheck {
            name: "contract_packet_complete".to_string(),
            ok: true,
            message: "Execution packet rendered without invoking Codex".to_string(),
        }],
        errors: Vec::new(),
        freeze_discovery: None,
    };
    write_json(
        &context.packet_dir.join(VALIDATION_REPORT_FILE_NAME),
        &report,
    )?;
    let status = RunStatus {
        workflow_version: WORKFLOW_VERSION.to_string(),
        generated_at: now_rfc3339()?,
        run_id: context.run_id.clone(),
        host_surface: "xtask recommend-next-agent-research".to_string(),
        mode: "dry-run".to_string(),
        pass: context.pass.as_str().to_string(),
        status: "dry_run_prepared".to_string(),
        validation_passed: true,
        packet_root: context.packet_dir_rel.clone(),
        discovery_root: context.discovery_dir_rel.clone(),
        research_root: context.research_dir_rel.clone(),
        prior_run_dir: context.prior_run_dir.clone(),
        written_paths: Vec::new(),
        errors: Vec::new(),
    };
    write_json(&context.packet_dir.join(RUN_STATUS_FILE_NAME), &status)?;
    write_string(
        &context.packet_dir.join(RUN_SUMMARY_FILE_NAME),
        &render_dry_run_summary(context),
    )?;
    Ok(())
}

fn render_discovery_prompt(context: &Context) -> String {
    let hints = context
        .input_contract
        .discovery_hints_path
        .as_deref()
        .unwrap_or("none");
    let excluded = if context.input_contract.excluded_candidate_ids.is_empty() {
        "none".to_string()
    } else {
        context.input_contract.excluded_candidate_ids.join(", ")
    };
    let onboarded = if context.input_contract.onboarded_agent_ids.is_empty() {
        "none".to_string()
    } else {
        context.input_contract.onboarded_agent_ids.join(", ")
    };
    let top_survivor = context
        .input_contract
        .top_surviving_candidate
        .as_deref()
        .unwrap_or("none");
    format!(
        concat!(
            "# Recommendation Research Discovery Prompt\n\n",
            "Run id: `{run_id}`\n",
            "Pass: `{pass}`\n",
            "Live seed path: `{live_seed}`\n",
            "Discovery hints path: `{hints}`\n",
            "Allowed output root: `{discovery_root}`\n",
            "Execution packet root: `{packet_root}`\n",
            "Currently onboarded agent ids: `{onboarded}`\n",
            "Excluded candidate ids: `{excluded}`\n",
            "Top surviving candidate: `{top_survivor}`\n\n",
            "Read only these repo files before researching:\n",
            "- `{live_seed}`\n",
            "- `{hints}`\n",
            "- `crates/xtask/data/agent_registry.toml`\n\n",
            "Required discovery files:\n",
            "- `candidate-seed.generated.toml`\n",
            "- `discovery-summary.md`\n",
            "- `sources.lock.json`\n\n",
            "Fixed query family:\n{queries}\n\n",
            "Requirements:\n",
            "- Use only public discovery evidence relevant to the fixed query family.\n",
            "- Respect discovery hints when present.\n",
            "- Exclude already onboarded agents and every pass1 candidate listed in `Excluded candidate ids`.\n",
            "- Write exactly the three required files and nothing else.\n",
            "- Do not write `seed.snapshot.toml`; the repo owns `freeze-discovery`.\n",
            "- Do not inspect unrelated repo files or run repo-wide searches.\n",
            "- Do not write outside `{discovery_root}`.\n",
            "- `sources.lock.json` must use the frozen contract fields and stable sha256 entries.\n"
        ),
        run_id = context.run_id,
        pass = context.pass.as_str(),
        live_seed = context.input_contract.live_seed_path,
        hints = hints,
        discovery_root = context.discovery_dir_rel,
        packet_root = context.packet_dir_rel,
        onboarded = onboarded,
        excluded = excluded,
        top_survivor = top_survivor,
        queries = render_bullets(&context.input_contract.query_family),
    )
}

fn render_research_prompt(context: &Context) -> String {
    let seed_ids = extract_candidate_ids_from_seed_file(&context.research_dir.join("seed.snapshot.toml"))
        .unwrap_or_default();
    let required_dossiers = if seed_ids.is_empty() {
        "none".to_string()
    } else {
        seed_ids
            .iter()
            .map(|agent_id| format!("- `dossiers/{agent_id}.json`"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        concat!(
            "# Recommendation Research Dossier Prompt\n\n",
            "Run id: `{run_id}`\n",
            "Pass: `{pass}`\n",
            "Frozen seed snapshot: `{seed_snapshot}`\n",
            "Seed snapshot sha256: `{seed_sha}`\n",
            "Dossier contract path: `{contract_path}`\n",
            "Allowed output root: `{research_root}`\n",
            "Execution packet root: `{packet_root}`\n\n",
            "Read only these repo files before researching:\n",
            "- `{seed_snapshot}`\n",
            "- `{research_root}/discovery-input/candidate-seed.generated.toml`\n",
            "- `{research_root}/discovery-input/discovery-summary.md`\n",
            "- `{research_root}/discovery-input/sources.lock.json`\n",
            "- `{contract_path}`\n\n",
            "Required research files:\n",
            "- `research-summary.md`\n",
            "- `research-metadata.json`\n",
            "{required_dossiers}\n\n",
            "Requirements:\n",
            "- Read only the frozen seed snapshot under `{research_root}`.\n",
            "- Produce exactly one dossier file per seeded candidate id.\n",
            "- Each dossier `agent_id` must match its filename stem.\n",
            "- Each dossier `seed_snapshot_sha256` must equal `{seed_sha}`.\n",
            "- `probe_requests` are structured metadata, not shell instructions.\n",
            "- Prefer the official/install/documentation URLs already present in `discovery-input/sources.lock.json` before widening to other sources.\n",
            "- Do not inspect unrelated repo files or run repo-wide searches.\n",
            "- Do not modify discovery artifacts.\n",
            "- Do not write outside `{research_root}`.\n"
        ),
        run_id = context.run_id,
        pass = context.pass.as_str(),
        seed_snapshot = format!("{}/seed.snapshot.toml", context.research_dir_rel),
        seed_sha = sha256_hex(&context.research_dir.join("seed.snapshot.toml")).unwrap_or_default(),
        contract_path = context.input_contract.dossier_contract_path,
        research_root = context.research_dir_rel,
        packet_root = context.packet_dir_rel,
        required_dossiers = required_dossiers,
    )
}

fn render_dry_run_summary(context: &Context) -> String {
    format!(
        concat!(
            "# Recommendation Research Dry Run\n\n",
            "- run_id: `{}`\n",
            "- pass: `{}`\n",
            "- packet root: `{}`\n",
            "- discovery root: `{}`\n",
            "- research root: `{}`\n",
            "- discovery prompt: `{}`\n",
            "- research prompt: `{}`\n",
            "- query family size: `{}`\n"
        ),
        context.run_id,
        context.pass.as_str(),
        context.packet_dir_rel,
        context.discovery_dir_rel,
        context.research_dir_rel,
        DISCOVERY_PROMPT_FILE_NAME,
        RESEARCH_PROMPT_FILE_NAME,
        context.input_contract.query_family.len(),
    )
}

fn render_write_summary(
    context: &Context,
    report: &ValidationReport,
    discovery_written_paths: &[String],
    research_written_paths: &[String],
    discovery_execution: Option<&CodexExecutionEvidence>,
    research_execution: Option<&CodexExecutionEvidence>,
) -> String {
    let mut text = format!(
        concat!(
            "# Recommendation Research Validation\n\n",
            "- run_id: `{}`\n",
            "- pass: `{}`\n",
            "- status: `{}`\n",
            "- packet root: `{}`\n",
            "- discovery root: `{}`\n",
            "- research root: `{}`\n"
        ),
        context.run_id,
        context.pass.as_str(),
        report.status,
        context.packet_dir_rel,
        context.discovery_dir_rel,
        context.research_dir_rel,
    );
    if let Some(execution) = discovery_execution {
        text.push_str("\n## Discovery Execution\n");
        text.push_str(&format!("- binary: `{}`\n", execution.binary));
        text.push_str(&format!("- exit_code: `{}`\n", execution.exit_code));
        text.push_str(&format!("- prompt: `{}`\n", execution.prompt_path));
    }
    if let Some(freeze) = &report.freeze_discovery {
        text.push_str("\n## Freeze Discovery\n");
        text.push_str(&format!("- binary: `{}`\n", freeze.binary));
        text.push_str(&format!("- exit_code: `{}`\n", freeze.exit_code));
        text.push_str("- argv:\n");
        for arg in &freeze.argv {
            text.push_str(&format!("  - `{arg}`\n"));
        }
    }
    if let Some(execution) = research_execution {
        text.push_str("\n## Research Execution\n");
        text.push_str(&format!("- binary: `{}`\n", execution.binary));
        text.push_str(&format!("- exit_code: `{}`\n", execution.exit_code));
        text.push_str(&format!("- prompt: `{}`\n", execution.prompt_path));
    }
    text.push_str("\n## Checks\n");
    for check in &report.checks {
        text.push_str(&format!(
            "- {}: {} ({})\n",
            check.name,
            if check.ok { "pass" } else { "fail" },
            check.message
        ));
    }
    text.push_str("\n## Discovery Written Paths\n");
    if discovery_written_paths.is_empty() {
        text.push_str("- none detected\n");
    } else {
        for path in discovery_written_paths {
            text.push_str(&format!("- `{path}`\n"));
        }
    }
    text.push_str("\n## Research Written Paths\n");
    if research_written_paths.is_empty() {
        text.push_str("- none detected\n");
    } else {
        for path in research_written_paths {
            text.push_str(&format!("- `{path}`\n"));
        }
    }
    if !report.errors.is_empty() {
        text.push_str("\n## Errors\n");
        for error in &report.errors {
            text.push_str(&format!("- {error}\n"));
        }
    }
    text
}

fn execute_codex_phase(
    workspace_root: &Path,
    context: &Context,
    phase: Phase,
    prompt: &str,
) -> Result<CodexExecutionEvidence, Error> {
    let binary = resolve_codex_binary(&Args {
        dry_run: false,
        write: true,
        pass: context.pass,
        run_id: Some(context.run_id.clone()),
        prior_run_dir: context.prior_run_dir.clone(),
        codex_binary: Some(context.codex_binary.clone()),
    });
    let argv = vec![
        "exec".to_string(),
        "--skip-git-repo-check".to_string(),
        "--dangerously-bypass-approvals-and-sandbox".to_string(),
        "--cd".to_string(),
        workspace_root.display().to_string(),
    ];
    let mut child = Command::new(&binary)
        .current_dir(workspace_root)
        .args(&argv)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| Error::Internal(format!("spawn codex binary `{binary}`: {err}")))?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| Error::Internal("codex exec stdin was not captured".to_string()))?;
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|err| Error::Internal(format!("write codex prompt to stdin: {err}")))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|err| Error::Internal(format!("wait for codex exec: {err}")))?;
    let stdout_path = context.packet_dir.join(phase.stdout_file_name());
    let stderr_path = context.packet_dir.join(phase.stderr_file_name());
    write_string(&stdout_path, &String::from_utf8_lossy(&output.stdout))?;
    write_string(&stderr_path, &String::from_utf8_lossy(&output.stderr))?;
    Ok(CodexExecutionEvidence {
        workflow_version: WORKFLOW_VERSION.to_string(),
        generated_at: now_rfc3339()?,
        run_id: context.run_id.clone(),
        phase: phase.as_str().to_string(),
        binary,
        argv,
        prompt_path: format!("{}/{}", context.packet_dir_rel, phase.prompt_file_name()),
        stdout_path: format!("{}/{}", context.packet_dir_rel, phase.stdout_file_name()),
        stderr_path: format!("{}/{}", context.packet_dir_rel, phase.stderr_file_name()),
        exit_code: output.status.code().unwrap_or(1),
    })
}

fn execute_freeze_discovery(
    workspace_root: &Path,
    context: &Context,
) -> Result<SubprocessEvidence, Error> {
    let argv = vec![
        "scripts/recommend_next_agent.py".to_string(),
        "freeze-discovery".to_string(),
        "--discovery-dir".to_string(),
        context.discovery_dir_rel.clone(),
        "--research-dir".to_string(),
        context.research_dir_rel.clone(),
    ];
    let output = Command::new("python3")
        .current_dir(workspace_root)
        .args(&argv)
        .output()
        .map_err(|err| Error::Internal(format!("spawn freeze-discovery: {err}")))?;
    Ok(SubprocessEvidence {
        binary: "python3".to_string(),
        argv,
        exit_code: output.status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn validate_matching_contract(context: &Context, persisted: &InputContract) -> Result<(), Error> {
    if persisted.workflow_version != WORKFLOW_VERSION {
        return Err(Error::Validation(format!(
            "dry-run packet workflow version `{}` does not match `{WORKFLOW_VERSION}`",
            persisted.workflow_version
        )));
    }
    if persisted.run_id != context.run_id {
        return Err(Error::Validation(format!(
            "dry-run packet run_id `{}` does not match write run_id `{}`",
            persisted.run_id, context.run_id
        )));
    }
    if persisted.pass != context.pass.as_str() {
        return Err(Error::Validation(format!(
            "dry-run packet pass `{}` does not match write pass `{}`",
            persisted.pass,
            context.pass.as_str()
        )));
    }
    if persisted.prior_run_dir != context.prior_run_dir {
        return Err(Error::Validation(
            "dry-run packet prior run context does not match the write invocation".to_string(),
        ));
    }
    Ok(())
}

fn validate_prior_run_for_pass2(prior_run_path: &Path) -> Result<(), Error> {
    let run_status = load_json::<Value>(&prior_run_path.join("run-status.json"))?;
    let status = run_status
        .get("status")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            Error::Validation(format!(
                "prior run `{}` is missing `status` in run-status.json",
                prior_run_path.display()
            ))
        })?;
    if status != "insufficient_eligible_candidates" {
        return Err(Error::Validation(format!(
            "prior run `{}` must have status `insufficient_eligible_candidates` for pass2",
            prior_run_path.display()
        )));
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct Pass2State {
    excluded_candidate_ids: Vec<String>,
    top_surviving_candidate: Option<String>,
    zero_survivors: bool,
}

fn load_pass2_state(
    workspace_root: &Path,
    prior_run_dir: &str,
    requested_run_id: Option<&str>,
) -> Result<Pass2State, Error> {
    let prior_run_path = workspace_root.join(prior_run_dir);
    validate_prior_run_for_pass2(&prior_run_path)?;
    let candidate_pool = load_json::<Value>(&prior_run_path.join("candidate-pool.json"))?;
    let candidates = candidate_pool
        .get("candidates")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            Error::Validation(format!(
                "prior run `{}` is missing candidate-pool.json candidates array",
                prior_run_dir
            ))
        })?;
    let mut excluded = candidates
        .iter()
        .filter_map(|candidate| candidate.get("agent_id").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<Vec<_>>();
    excluded.sort();
    excluded.dedup();

    let run_status = load_json::<Value>(&prior_run_path.join("run-status.json"))?;
    let recommended = run_status
        .get("recommended_agent_id")
        .and_then(Value::as_str)
        .map(str::to_string);
    let eligible = candidates
        .iter()
        .filter(|candidate| candidate.get("status").and_then(Value::as_str) == Some("eligible"))
        .filter_map(|candidate| candidate.get("agent_id").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<Vec<_>>();
    let top_surviving_candidate = recommended.or_else(|| eligible.first().cloned());

    if let Some(run_id) = requested_run_id {
        let prior_basename = prior_run_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if prior_basename == run_id {
            return Err(Error::Validation(
                "pass2 must use a fresh run_id instead of reusing the prior insufficiency run id"
                    .to_string(),
            ));
        }
    }

    Ok(Pass2State {
        excluded_candidate_ids: excluded,
        top_surviving_candidate,
        zero_survivors: eligible.is_empty(),
    })
}

fn validate_discovery_artifacts(discovery_dir: &Path) -> Result<(), String> {
    if !discovery_dir.is_dir() {
        return Err(format!(
            "required discovery artifact directory `{}` is missing",
            discovery_dir.display()
        ));
    }
    let actual = fs::read_dir(discovery_dir)
        .map_err(|err| format!("read discovery dir {}: {err}", discovery_dir.display()))?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map(|kind| kind.is_file()).unwrap_or(false))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<BTreeSet<_>>();
    let expected = DISCOVERY_REQUIRED_FILES
        .iter()
        .map(|name| name.to_string())
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(format!(
            "discovery artifact set does not match the frozen contract: expected {:?}, got {:?}",
            expected, actual
        ));
    }
    Ok(())
}

fn validate_frozen_seed_boundary(research_dir: &Path) -> Result<(), String> {
    let seed_snapshot = research_dir.join("seed.snapshot.toml");
    if !seed_snapshot.is_file() {
        return Err(format!(
            "freeze-discovery did not produce `{}`",
            seed_snapshot.display()
        ));
    }
    let discovery_input = research_dir.join("discovery-input");
    for filename in DISCOVERY_REQUIRED_FILES {
        let path = discovery_input.join(filename);
        if !path.is_file() {
            return Err(format!(
                "freeze-discovery did not copy required discovery input `{}`",
                path.display()
            ));
        }
    }
    Ok(())
}

fn validate_research_tree(research_dir: &Path) -> Result<(), String> {
    for filename in RESEARCH_REQUIRED_FILES {
        let path = research_dir.join(filename);
        if !path.is_file() {
            return Err(format!(
                "required research artifact `{}` is missing",
                path.display()
            ));
        }
    }
    validate_frozen_seed_boundary(research_dir)?;
    let seed_snapshot = research_dir.join("seed.snapshot.toml");
    let seeded_ids = extract_candidate_ids_from_seed_file(&seed_snapshot)
        .map_err(|err| format!("parse {}: {err}", seed_snapshot.display()))?;
    if seeded_ids.is_empty() {
        return Err("seed.snapshot.toml does not define any candidate ids".to_string());
    }
    let seed_sha = sha256_hex(&seed_snapshot)
        .map_err(|err| format!("hash {}: {err}", seed_snapshot.display()))?;
    let dossier_dir = research_dir.join("dossiers");
    if !dossier_dir.is_dir() {
        return Err(format!(
            "required dossier directory `{}` is missing",
            dossier_dir.display()
        ));
    }
    let mut dossier_ids = fs::read_dir(&dossier_dir)
        .map_err(|err| format!("read dossier dir {}: {err}", dossier_dir.display()))?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map(|kind| kind.is_file()).unwrap_or(false))
        .map(|entry| {
            let path = entry.path();
            let stem = path
                .file_stem()
                .and_then(|value| value.to_str())
                .ok_or_else(|| format!("invalid dossier filename `{}`", path.display()))?
                .to_string();
            let dossier = load_json::<Value>(&path)
                .map_err(|err| format!("parse dossier {}: {err}", path.display()))?;
            let agent_id = dossier
                .get("agent_id")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("dossier `{}` is missing string agent_id", path.display()))?;
            if agent_id != stem {
                return Err(format!(
                    "dossier `{}` agent_id `{agent_id}` does not match filename stem `{stem}`",
                    path.display()
                ));
            }
            let dossier_seed = dossier
                .get("seed_snapshot_sha256")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    format!(
                        "dossier `{}` is missing string seed_snapshot_sha256",
                        path.display()
                    )
                })?;
            if dossier_seed != seed_sha {
                return Err(format!(
                    "dossier `{}` seed_snapshot_sha256 does not match the frozen seed",
                    path.display()
                ));
            }
            Ok(stem)
        })
        .collect::<Result<Vec<_>, _>>()?;
    dossier_ids.sort();
    let mut expected = seeded_ids.clone();
    expected.sort();
    if dossier_ids != expected {
        return Err(format!(
            "dossier set does not match frozen seed candidate ids: expected {:?}, got {:?}",
            expected, dossier_ids
        ));
    }
    Ok(())
}

fn validate_written_paths(
    written_paths: &[String],
    allowed_root: &str,
    phase: &str,
) -> Result<(), String> {
    let violations = written_paths
        .iter()
        .filter(|path| !path.starts_with(&(allowed_root.to_string() + "/")))
        .cloned()
        .collect::<Vec<_>>();
    if !violations.is_empty() {
        return Err(format!(
            "{phase} write boundary violation: {}",
            violations.join(", ")
        ));
    }
    Ok(())
}

fn push_passed_check(report: &mut ValidationReport, name: &str, message: String) {
    report.checks.push(ValidationCheck {
        name: name.to_string(),
        ok: true,
        message,
    });
}

fn push_failed_check(report: &mut ValidationReport, name: &str, message: String) {
    report.checks.push(ValidationCheck {
        name: name.to_string(),
        ok: false,
        message,
    });
}

fn extract_candidate_ids_from_seed_file(path: &Path) -> Result<Vec<String>, Error> {
    extract_candidate_ids_from_seed_text(&read_string(path)?)
        .map_err(|message| Error::Validation(message))
}

fn extract_candidate_ids_from_seed_text(text: &str) -> Result<Vec<String>, String> {
    let regex = Regex::new(r"(?m)^\[candidate\.([A-Za-z0-9_-]+)\]\s*$")
        .map_err(|err| format!("compile candidate id regex: {err}"))?;
    let mut ids = regex
        .captures_iter(text)
        .filter_map(|captures| captures.get(1).map(|value| value.as_str().to_string()))
        .collect::<Vec<_>>();
    let unique = ids.iter().cloned().collect::<BTreeSet<_>>();
    if unique.len() != ids.len() {
        return Err("seed snapshot contains duplicate candidate ids".to_string());
    }
    ids.sort();
    Ok(ids)
}

fn render_bullets(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("- `{value}`"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn resolve_codex_binary(args: &Args) -> String {
    args.codex_binary
        .clone()
        .or_else(|| std::env::var(CODEX_BINARY_ENV).ok())
        .unwrap_or_else(|| "codex".to_string())
}

fn packet_root(workspace_root: &Path, run_id: &str) -> PathBuf {
    workspace_root.join(RESEARCH_PACKET_ROOT).join(run_id)
}

fn discovery_root(workspace_root: &Path, run_id: &str) -> PathBuf {
    workspace_root.join(DISCOVERY_ROOT).join(run_id)
}

fn research_root(workspace_root: &Path, run_id: &str) -> PathBuf {
    workspace_root.join(RESEARCH_ROOT).join(run_id)
}

fn packet_root_rel(run_id: &str) -> String {
    format!("{RESEARCH_PACKET_ROOT}/{run_id}")
}

fn discovery_root_rel(run_id: &str) -> String {
    format!("{DISCOVERY_ROOT}/{run_id}")
}

fn research_root_rel(run_id: &str) -> String {
    format!("{RESEARCH_ROOT}/{run_id}")
}

fn write_header<W: Write>(
    writer: &mut W,
    context: &Context,
    write_mode: bool,
) -> Result<(), Error> {
    writeln!(
        writer,
        "== RECOMMEND-NEXT-AGENT-RESEARCH {} ==",
        if write_mode { "WRITE" } else { "DRY RUN" }
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "run_id: {}", context.run_id)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "pass: {}", context.pass.as_str())
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "packet_root: {}", context.packet_dir_rel)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
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

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), Error> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| Error::Internal(format!("serialize {}: {err}", path.display())))?;
    bytes.push(b'\n');
    write_bytes(path, &bytes)
}

fn read_string(path: &Path) -> Result<String, Error> {
    fs::read_to_string(path)
        .map_err(|err| Error::Validation(format!("read {}: {err}", path.display())))
}

fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, Error> {
    let bytes = fs::read(path)
        .map_err(|err| Error::Validation(format!("read {}: {err}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|err| Error::Validation(format!("parse {}: {err}", path.display())))
}

fn write_string(path: &Path, value: &str) -> Result<(), Error> {
    write_bytes(path, value.as_bytes())
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| Error::Internal(format!("create {}: {err}", parent.display())))?;
    }
    fs::write(path, bytes)
        .map_err(|err| Error::Internal(format!("write {}: {err}", path.display())))
}

fn snapshot_workspace(
    workspace_root: &Path,
    ignored_roots: &[&Path],
) -> Result<WorkspaceSnapshot, Error> {
    let mut files = Vec::new();
    let ignored = ignored_roots
        .iter()
        .map(|path| workspace_root.join(path))
        .collect::<Vec<_>>();
    collect_snapshot_files(workspace_root, workspace_root, &ignored, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(WorkspaceSnapshot { files })
}

fn collect_snapshot_files(
    workspace_root: &Path,
    current: &Path,
    ignored_roots: &[PathBuf],
    files: &mut Vec<SnapshotFile>,
) -> Result<(), Error> {
    let metadata = fs::symlink_metadata(current)
        .map_err(|err| Error::Internal(format!("stat {}: {err}", current.display())))?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if current != workspace_root {
        if let Ok(relative) = current.strip_prefix(workspace_root) {
            if ignored_roots
                .iter()
                .any(|root| current == root || current.starts_with(root))
            {
                return Ok(());
            }
            if relative == Path::new(".git") || relative.starts_with("target") {
                return Ok(());
            }
        }
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(current)
            .map_err(|err| Error::Internal(format!("read_dir {}: {err}", current.display())))?
        {
            let entry = entry
                .map_err(|err| Error::Internal(format!("read_dir {}: {err}", current.display())))?;
            collect_snapshot_files(workspace_root, &entry.path(), ignored_roots, files)?;
        }
        return Ok(());
    }
    let relative = current
        .strip_prefix(workspace_root)
        .map_err(|err| Error::Internal(format!("strip prefix {}: {err}", current.display())))?;
    let bytes = fs::read(current)
        .map_err(|err| Error::Internal(format!("read {}: {err}", current.display())))?;
    files.push(SnapshotFile {
        path: relative.to_string_lossy().replace('\\', "/"),
        sha256: hex::encode(Sha256::digest(bytes)),
    });
    Ok(())
}

fn diff_snapshots(before: &WorkspaceSnapshot, after: &WorkspaceSnapshot) -> Vec<String> {
    let before_map = before
        .files
        .iter()
        .map(|file| (file.path.as_str(), file.sha256.as_str()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let after_map = after
        .files
        .iter()
        .map(|file| (file.path.as_str(), file.sha256.as_str()))
        .collect::<std::collections::BTreeMap<_, _>>();
    before_map
        .keys()
        .chain(after_map.keys())
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|path| before_map.get(path) != after_map.get(path))
        .map(str::to_string)
        .collect()
}

fn generate_run_id() -> String {
    OffsetDateTime::now_utc()
        .format(
            &time::format_description::parse("[year][month][day]T[hour][minute][second]Z")
                .expect("valid time format"),
        )
        .unwrap_or_else(|_| "recommend-next-agent-research".to_string())
}

fn now_rfc3339() -> Result<String, Error> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|err| Error::Internal(format!("format timestamp: {err}")))
}

fn sha256_hex(path: &Path) -> Result<String, std::io::Error> {
    fs::read(path).map(|bytes| hex::encode(Sha256::digest(bytes)))
}
