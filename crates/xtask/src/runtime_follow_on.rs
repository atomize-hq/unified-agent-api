use std::{
    fmt, fs,
    io::{stdout, Write},
    path::Path,
};

use clap::{ArgGroup, Parser, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::{agent_registry::AgentRegistry, approval_artifact::load_approval_artifact};

mod codex_exec;
mod context;
mod validation;
use context::build_context;
use validation::{validate_observed_write_mode, validate_write_mode};
pub(crate) mod depth;
#[cfg(test)]
pub(crate) fn isolated_depth_dry_run(
    workspace: &Path,
    args: Args,
    writer: &mut impl Write,
) -> Result<(), Error> {
    depth::dry_run(workspace, args, writer)
}
#[path = "candidate_execution/create_lane.rs"]
mod depth_candidate;
mod io;
mod lifecycle;
mod models;
mod render;

pub(crate) use models::SupportDepthContract;

use self::{
    codex_exec::{
        allowed_write_paths, approval_descriptor_view, docs_to_read, execute_codex_write,
        is_allowed_write_path, is_publication_owned_manifest_path, map_approval_error,
        required_agent_api_test, resolve_workspace_root,
    },
    io::{
        diff_snapshots, generate_run_id, load_json, now_rfc3339, read_string, snapshot_workspace,
        write_json, write_string,
    },
    lifecycle::{
        load_enrolled_lifecycle_state, persist_failed_runtime_integration,
        persist_successful_runtime_integration, required_publication_commands, validate_handoff,
        validate_registry_alignment,
    },
    models::{
        CodexExecutionEvidence, HandoffContract, InputContract, RuntimeContext, ValidationCheck,
        ValidationReport,
    },
    render::{
        render_dry_run_summary, render_prompt, render_run_status, render_run_summary, write_header,
    },
};

const RUNTIME_RUNS_ROOT: &str = "docs/agents/.uaa-temp/runtime-follow-on/runs";
const SKILL_PATH: &str = ".codex/skills/runtime-follow-on/SKILL.md";
const HANDOFF_FILE_NAME: &str = "handoff.json";
const INPUT_CONTRACT_FILE_NAME: &str = "input-contract.json";
const CODEX_EXECUTION_FILE_NAME: &str = "codex-execution.json";
const CODEX_STDOUT_FILE_NAME: &str = "codex-stdout.log";
const CODEX_STDERR_FILE_NAME: &str = "codex-stderr.log";
const PROMPT_FILE_NAME: &str = "codex-prompt.md";
const RUN_STATUS_FILE_NAME: &str = "run-status.json";
const RUN_SUMMARY_FILE_NAME: &str = "run-summary.md";
const VALIDATION_REPORT_FILE_NAME: &str = "validation-report.json";
const WRITTEN_PATHS_FILE_NAME: &str = "written-paths.json";
pub(crate) const WORKFLOW_VERSION: &str = "runtime_follow_on_v2";
const CODEX_BINARY_ENV: &str = "XTASK_RUNTIME_FOLLOW_ON_CODEX_BINARY";
const WRAPPER_COVERAGE_MANIFEST_PATH: &str = "src/wrapper_coverage_manifest.rs";
const PROMPT_TEMPLATE: &str = include_str!("../templates/runtime_follow_on_codex_prompt.md");

#[derive(Debug, Parser, Clone)]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(["dry_run", "write"])
        .multiple(false)
))]
pub struct Args {
    /// Repo-relative approved onboarding artifact under docs/agents/lifecycle/**/governance/approved-agent.toml.
    #[arg(long)]
    pub approval: String,

    /// Materialize the packet and prompt without validating runtime outputs.
    #[arg(long)]
    pub dry_run: bool,

    /// Validate the runtime lane outputs and handoff contract using a prepared run id.
    #[arg(long)]
    pub write: bool,

    /// Requested implementation tier.
    #[arg(long, value_enum, default_value_t = RequestedTier::Default)]
    pub requested_tier: RequestedTier,

    /// Required when requested tier is `minimal`.
    #[arg(long)]
    pub minimal_justification_file: Option<String>,

    /// Explicit richer surfaces approved for this run.
    #[arg(long)]
    pub allow_rich_surface: Vec<String>,

    /// Stable run identifier. Required for `--write`; optional for `--dry-run`.
    #[arg(long)]
    pub run_id: Option<String>,

    /// Explicit `codex` binary path. Falls back to XTASK_RUNTIME_FOLLOW_ON_CODEX_BINARY, then `codex`.
    #[arg(long)]
    pub codex_binary: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum RequestedTier {
    Default,
    Minimal,
    FeatureRich,
}

impl RequestedTier {
    fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Minimal => "minimal",
            Self::FeatureRich => "feature-rich",
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
            "runtime-follow-on must run with cwd = repo root `{}` (got `{}`)",
            workspace_root.display(),
            current_dir.display()
        )));
    }
    let mut stdout = stdout();
    run_in_workspace(&workspace_root, args, &mut stdout)
}

pub fn run_in_workspace<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
) -> Result<(), Error> {
    if let Some(route) =
        crate::isolated_maintenance::route(workspace_root).map_err(Error::Validation)?
    {
        if route
            .context()
            .bootstrap_pending()
            .map_err(Error::Validation)?
            || route.context().bootstrap_declared_onboarding()
        {
            return depth::run(workspace_root, args, route, writer);
        }
    }
    if args.write {
        // Ordinary approvals keep the original workspace refusal, even inside an isolated route.
        let depth_approval = load_approval_artifact(workspace_root, &args.approval)
            .ok()
            .is_some_and(|a| a.support_depth_version.is_some());
        if !depth_approval
            || crate::isolated_maintenance::route(workspace_root)
                .map_err(Error::Validation)?
                .is_none()
        {
            xtask::depth_routes::refuse_workspace(workspace_root, "runtime_follow_on")
                .map_err(Error::Validation)?;
        }
    }
    validate_args(&args, workspace_root)?;
    let context = build_context(workspace_root, &args)?;
    write_header(writer, &context, args.write)?;

    if args.dry_run {
        persist_dry_run_artifacts(&context)?;
        writeln!(writer, "OK: runtime-follow-on dry-run packet prepared.")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        writeln!(writer, "run_id: {}", context.run_id)
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        writeln!(writer, "run_dir: {}", context.run_dir.display())
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        return Ok(());
    }

    let prior_contract =
        load_json::<InputContract>(&context.run_dir.join(INPUT_CONTRACT_FILE_NAME))?;
    prior_contract.check_support_depth(
        workspace_root,
        &context.approval,
        &context.lifecycle_state,
    )?;
    let prompt = read_string(&context.run_dir.join(PROMPT_FILE_NAME))?;
    if context.approval.support_depth_version.is_some() {
        return depth_candidate::execute(
            workspace_root,
            &context,
            &args,
            &prior_contract,
            &prompt,
            writer,
        );
    }
    let codex_execution = execute_codex_write(workspace_root, &context, &args, &prompt)?;
    write_json(
        &context.run_dir.join(CODEX_EXECUTION_FILE_NAME),
        &codex_execution,
    )?;
    let (report, written_paths) =
        validate_write_mode(workspace_root, &context, &prior_contract, &codex_execution)?;
    let passed = report.status == "pass";
    if passed {
        persist_successful_runtime_integration(
            workspace_root,
            &context,
            &prior_contract,
            &written_paths,
        )?;
    } else {
        persist_failed_runtime_integration(workspace_root, &context, &report)?;
    }
    write_json(&context.run_dir.join(VALIDATION_REPORT_FILE_NAME), &report)?;
    let status = render_run_status(
        &context,
        "write",
        if passed {
            "write_validated"
        } else {
            "write_failed"
        },
        passed,
        passed,
        written_paths.clone(),
        report.errors.clone(),
    );
    write_json(&context.run_dir.join(RUN_STATUS_FILE_NAME), &status)?;
    write_json(
        &context.run_dir.join(WRITTEN_PATHS_FILE_NAME),
        &written_paths,
    )?;
    write_string(
        &context.run_dir.join(RUN_SUMMARY_FILE_NAME),
        &render_run_summary(&context, &report, &written_paths, Some(&codex_execution)),
    )?;

    if !passed {
        return Err(Error::Validation(report.errors.join("\n")));
    }

    writeln!(writer, "OK: runtime-follow-on write validation complete.")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "run_id: {}", context.run_id)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "validated_paths: {}", written_paths.len())
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

fn validate_args(args: &Args, workspace_root: &Path) -> Result<(), Error> {
    if args.write && args.run_id.is_none() {
        return Err(Error::Validation(
            "--run-id is required with --write so the command can validate against a prepared dry-run baseline".to_string(),
        ));
    }
    if matches!(args.requested_tier, RequestedTier::Minimal)
        && args.minimal_justification_file.is_none()
    {
        return Err(Error::Validation(
            "--minimal-justification-file is required when --requested-tier minimal".to_string(),
        ));
    }
    if let Some(path) = &args.minimal_justification_file {
        let candidate = workspace_root.join(path);
        if !candidate.is_file() {
            return Err(Error::Validation(format!(
                "minimal justification file `{path}` does not exist"
            )));
        }
    }
    Ok(())
}

fn persist_dry_run_artifacts(context: &RuntimeContext) -> Result<(), Error> {
    fs::create_dir_all(&context.run_dir)
        .map_err(|err| Error::Internal(format!("create {}: {err}", context.run_dir.display())))?;
    write_json(
        &context.run_dir.join(INPUT_CONTRACT_FILE_NAME),
        &context.input_contract,
    )?;
    let prompt = if context.approval.support_depth_version.is_some() {
        depth_candidate::render_depth_prompt(context)
    } else {
        render_prompt(context)
    };
    write_string(&context.run_dir.join(PROMPT_FILE_NAME), &prompt)?;
    write_json(
        &context.run_dir.join(RUN_STATUS_FILE_NAME),
        &render_run_status(
            context,
            "dry_run",
            "dry_run_ready",
            true,
            false,
            Vec::new(),
            Vec::new(),
        ),
    )?;
    write_string(
        &context.run_dir.join(RUN_SUMMARY_FILE_NAME),
        &render_dry_run_summary(context),
    )?;
    write_json(
        &context.run_dir.join(VALIDATION_REPORT_FILE_NAME),
        &ValidationReport {
            workflow_version: WORKFLOW_VERSION.to_string(),
            generated_at: now_rfc3339()?,
            run_id: context.run_id.clone(),
            status: "prepared".to_string(),
            checks: vec![
                ValidationCheck {
                    name: "approval_registry_alignment".to_string(),
                    ok: true,
                    message: "approval artifact and registry entry are aligned".to_string(),
                },
                ValidationCheck {
                    name: "prompt_packet_prepared".to_string(),
                    ok: true,
                    message: "dry-run wrote the frozen prompt and input contract".to_string(),
                },
            ],
            errors: Vec::new(),
        },
    )?;
    write_json(
        &context.run_dir.join(WRITTEN_PATHS_FILE_NAME),
        &Vec::<String>::new(),
    )?;
    write_json(
        &context.run_dir.join(HANDOFF_FILE_NAME),
        &HandoffContract {
            agent_id: context.approval.descriptor.agent_id.clone(),
            manifest_root: context.approval.descriptor.manifest_root.clone(),
            runtime_lane_complete: false,
            publication_refresh_required: true,
            required_commands: required_publication_commands(),
            blockers: vec!["Pending runtime follow-on implementation.".to_string()],
        },
    )?;
    Ok(())
}
