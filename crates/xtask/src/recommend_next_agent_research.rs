use std::{
    collections::{BTreeMap, BTreeSet},
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

mod contract;
mod io_support;
mod render;
mod validation;

use self::{contract::*, io_support::*, render::*, validation::*};

pub const RESEARCH_PACKET_ROOT: &str = "docs/agents/.uaa-temp/recommend-next-agent/research-runs";
pub const DISCOVERY_ROOT: &str = "docs/agents/.uaa-temp/recommend-next-agent/discovery";
pub const RESEARCH_ROOT: &str = "docs/agents/.uaa-temp/recommend-next-agent/research";
pub const PYTHON_RUNS_ROOT: &str = "docs/agents/.uaa-temp/recommend-next-agent/runs";
pub const DISCOVERY_HINTS_PATH: &str = "docs/agents/selection/discovery-hints.json";
pub const LIVE_SEED_PATH: &str = "docs/agents/selection/candidate-seed.toml";
pub const DOSSIER_CONTRACT_PATH: &str = "docs/specs/cli-agent-recommendation-dossier-contract.md";
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
pub const MIN_DISCOVERY_CANDIDATES: usize = 3;
pub const DISCOVERY_SOURCE_KINDS: [&str; 4] = [
    "web_search_result",
    "official_doc",
    "github",
    "package_registry",
];
pub const DISCOVERY_SOURCE_ROLES: [&str; 4] = [
    "frontier_signal",
    "discovery_seed",
    "install_surface",
    "docs_surface",
];

#[derive(Debug, Parser, Clone)]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(["dry_run", "write"])
        .multiple(false)
))]
pub struct Args {
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub write: bool,
    #[arg(long, value_enum)]
    pub pass: Pass,
    #[arg(long)]
    pub run_id: Option<String>,
    #[arg(long)]
    pub prior_run_dir: Option<String>,
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
        writeln!(
            writer,
            "OK: recommend-next-agent-research dry-run packet prepared."
        )
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
        let discovery_prompt = read_string(&context.packet_dir.join(DISCOVERY_PROMPT_FILE_NAME))?;
        let before_discovery = snapshot_workspace(workspace_root, &[context.packet_dir.as_path()])?;
        let executed_discovery =
            execute_codex_phase(workspace_root, context, Phase::Discovery, &discovery_prompt)?;
        write_json(
            &context
                .packet_dir
                .join(Phase::Discovery.execution_file_name()),
            &executed_discovery,
        )?;
        discovery_execution = Some(executed_discovery.clone());
        let after_discovery = snapshot_workspace(workspace_root, &[context.packet_dir.as_path()])?;
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
            format!(
                "Discovery writes stayed under `{}`",
                context.discovery_dir_rel
            ),
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
        validate_discovery_candidate_count(&context.discovery_dir).map_err(|message| {
            push_failed_check(&mut report, "discovery_candidate_minimum", message.clone());
            Error::Validation(message)
        })?;
        push_passed_check(
            &mut report,
            "discovery_candidate_minimum",
            format!(
                "Discovery seed satisfied the minimum candidate pool of {MIN_DISCOVERY_CANDIDATES}"
            ),
        );
        validate_discovery_summary_contract(&context.discovery_dir, &context.run_id).map_err(
            |message| {
                push_failed_check(&mut report, "discovery_summary_contract", message.clone());
                Error::Validation(message)
            },
        )?;
        push_passed_check(
            &mut report,
            "discovery_summary_contract",
            "Discovery summary mentions the run id plus each seeded candidate id and display name"
                .to_string(),
        );
        validate_discovery_sources_lock_contract(&context.discovery_dir, &context.run_id).map_err(
            |message| {
                push_failed_check(
                    &mut report,
                    "discovery_sources_lock_contract",
                    message.clone(),
                );
                Error::Validation(message)
            },
        )?;
        push_passed_check(
            &mut report,
            "discovery_sources_lock_contract",
            "Discovery sources.lock.json matches the frozen schema, enums, and canonical sha256 rules"
                .to_string(),
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
        let before_research = snapshot_workspace(workspace_root, &[context.packet_dir.as_path()])?;
        let executed_research =
            execute_codex_phase(workspace_root, context, Phase::Research, &research_prompt)?;
        write_json(
            &context
                .packet_dir
                .join(Phase::Research.execution_file_name()),
            &executed_research,
        )?;
        research_execution = Some(executed_research.clone());
        let after_research = snapshot_workspace(workspace_root, &[context.packet_dir.as_path()])?;
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
            format!(
                "Research writes stayed under `{}`",
                context.research_dir_rel
            ),
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
        execute_research_contract_validation(workspace_root, context).map_err(|message| {
            push_failed_check(&mut report, "research_schema_contract", message.clone());
            Error::Validation(message)
        })?;
        push_passed_check(
            &mut report,
            "research_schema_contract",
            "Research metadata and dossiers passed the repo-owned Python contract validators"
                .to_string(),
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

    outcome?;

    writeln!(
        writer,
        "OK: recommend-next-agent-research write validation complete."
    )
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
