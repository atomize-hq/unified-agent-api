use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    io::{stdout, Write},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use clap::{ArgGroup, Parser};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::workspace_mutation::{WorkspaceMutationError, WorkspacePathJail};

use super::{
    docs,
    request::{self, load_request_envelope, MaintenanceRequestEnvelope},
};

const EXECUTION_RUNS_ROOT: &str = "docs/agents/.uaa-temp/agent-maintenance/runs";
const INPUT_CONTRACT_FILE_NAME: &str = "input-contract.json";
const PROMPT_FILE_NAME: &str = "codex-prompt.md";
const RUN_STATUS_FILE_NAME: &str = "run-status.json";
const RUN_SUMMARY_FILE_NAME: &str = "run-summary.md";
const VALIDATION_REPORT_FILE_NAME: &str = "validation-report.json";
const WRITTEN_PATHS_FILE_NAME: &str = "written-paths.json";
const CODEX_EXECUTION_FILE_NAME: &str = "codex-execution.json";
const CODEX_STDOUT_FILE_NAME: &str = "codex-stdout.log";
const CODEX_STDERR_FILE_NAME: &str = "codex-stderr.log";
const WORKFLOW_VERSION: &str = "agent_maintenance_execute_v1";
const CODEX_BINARY_ENV: &str = "XTASK_AGENT_MAINTENANCE_CODEX_BINARY";
const PREFLIGHT_SENTINEL: &str = "UAA_AGENT_MAINTENANCE_PREFLIGHT_OK";
const EXECUTE_HOST_SURFACE: &str = "execute-agent-maintenance";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InputContract {
    workflow_version: String,
    generated_at: String,
    run_id: String,
    request_path: String,
    request_sha256: String,
    maintenance_root: String,
    agent_id: String,
    target_version: String,
    branch_name: String,
    executor: String,
    prompt_sha256: String,
    closeout_path: String,
    closeout_command: String,
    writable_surfaces: Vec<String>,
    read_only_inputs: Vec<String>,
    ordered_commands: Vec<String>,
    green_gates: Vec<String>,
    recovery: RecoveryContract,
    ignored_diff_roots: Vec<String>,
    baseline: WorkspaceSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecoveryContract {
    recreate_packet_command: String,
    reopen_pr_body_path: String,
    reopen_pr_branch: String,
    notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ValidationCheck {
    name: String,
    ok: bool,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ValidationReport {
    workflow_version: String,
    run_id: String,
    status: String,
    checks: Vec<ValidationCheck>,
    errors: Vec<String>,
    preflight: Option<SubprocessEvidence>,
    codex_execution: Option<CodexExecutionEvidence>,
    gate_results: Vec<GateEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunStatus {
    workflow_version: String,
    generated_at: String,
    run_id: String,
    host_surface: String,
    mode: String,
    status: String,
    validation_passed: bool,
    request_path: String,
    packet_root: String,
    agent_id: String,
    target_version: String,
    branch_name: String,
    written_paths: Vec<String>,
    errors: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubprocessEvidence {
    binary: String,
    argv: Vec<String>,
    exit_code: i32,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CodexExecutionEvidence {
    workflow_version: String,
    generated_at: String,
    run_id: String,
    binary: String,
    argv: Vec<String>,
    prompt_path: String,
    stdout_path: String,
    stderr_path: String,
    exit_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GateEvidence {
    command: String,
    exit_code: i32,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Clone)]
struct Context {
    run_id: String,
    codex_binary: String,
    run_dir: PathBuf,
    run_dir_rel: String,
    envelope: MaintenanceRequestEnvelope,
    execution_contract: request::ExecutionContract,
    rendered_packet: docs::RenderedExecutionPacket,
    closeout_command: String,
    input_contract: Option<InputContract>,
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
    let context = build_context(workspace_root, &args)?;
    write_preview(writer, &context, args.write)?;

    if args.dry_run {
        return execute_dry_run(workspace_root, &context, writer);
    }

    execute_write_mode(workspace_root, &context, writer)
}

fn validate_args(args: &Args) -> Result<(), Error> {
    if args.write && args.run_id.is_none() {
        return Err(Error::Validation(
            "--run-id is required with --write so the relay can validate against one prepared dry-run baseline".to_string(),
        ));
    }
    Ok(())
}

fn build_context(workspace_root: &Path, args: &Args) -> Result<Context, Error> {
    let envelope = load_request_envelope(workspace_root, &args.request)?;
    if !envelope.request.is_automated_watch_request() {
        return Err(Error::Validation(format!(
            "execute-agent-maintenance only supports automated upstream-release requests; `{}` has trigger_kind `{}`",
            envelope.request.relative_path,
            envelope.request.trigger_kind.as_str()
        )));
    }

    let execution_contract = envelope
        .require_execution_contract_for_relay()
        .map_err(Error::from)?
        .clone();
    let rendered_packet =
        docs::render_execution_packet(workspace_root, &envelope.request, &execution_contract)
            .map_err(Error::Validation)?;
    let detected_release = envelope.request.detected_release.as_ref().ok_or_else(|| {
        Error::Validation(format!(
            "maintenance request `{}` is missing detected_release metadata",
            envelope.request.relative_path
        ))
    })?;
    let run_id = args.run_id.clone().unwrap_or_else(generate_run_id);
    let run_dir_rel = format!("{EXECUTION_RUNS_ROOT}/{run_id}");
    let closeout_command = format!(
        "cargo run -p xtask -- close-agent-maintenance --request {} --closeout {}",
        envelope.request.relative_path, execution_contract.closeout_path
    );
    let input_contract = if args.dry_run {
        Some(InputContract {
            workflow_version: WORKFLOW_VERSION.to_string(),
            generated_at: now_rfc3339()?,
            run_id: run_id.clone(),
            request_path: envelope.request.relative_path.clone(),
            request_sha256: envelope.request.sha256.clone(),
            maintenance_root: envelope.request.maintenance_root.clone(),
            agent_id: envelope.request.agent_id.clone(),
            target_version: detected_release.target_version.clone(),
            branch_name: detected_release.branch_name.clone(),
            executor: execution_contract.executor.clone(),
            prompt_sha256: execution_contract.prompt_sha256.clone(),
            closeout_path: execution_contract.closeout_path.clone(),
            closeout_command: closeout_command.clone(),
            writable_surfaces: execution_contract.writable_surfaces.clone(),
            read_only_inputs: execution_contract.read_only_inputs.clone(),
            ordered_commands: execution_contract.ordered_commands.clone(),
            green_gates: execution_contract.green_gates.clone(),
            recovery: RecoveryContract {
                recreate_packet_command: execution_contract
                    .recovery
                    .recreate_packet_command
                    .clone(),
                reopen_pr_body_path: execution_contract.recovery.reopen_pr_body_path.clone(),
                reopen_pr_branch: execution_contract.recovery.reopen_pr_branch.clone(),
                notes: execution_contract.recovery.notes.clone(),
            },
            ignored_diff_roots: vec![EXECUTION_RUNS_ROOT.to_string()],
            baseline: snapshot_workspace(workspace_root, &[Path::new(EXECUTION_RUNS_ROOT)])?,
        })
    } else {
        None
    };

    Ok(Context {
        run_id,
        codex_binary: resolve_codex_binary(args),
        run_dir: workspace_root.join(&run_dir_rel),
        run_dir_rel,
        envelope,
        execution_contract,
        rendered_packet,
        closeout_command,
        input_contract,
    })
}

fn execute_dry_run<W: Write>(
    workspace_root: &Path,
    context: &Context,
    writer: &mut W,
) -> Result<(), Error> {
    let preflight = run_codex_preflight(workspace_root, context)?;
    let preflight_ok = preflight.errors.is_empty();
    let report = ValidationReport {
        workflow_version: WORKFLOW_VERSION.to_string(),
        run_id: context.run_id.clone(),
        status: if preflight_ok {
            "pass".to_string()
        } else {
            "fail".to_string()
        },
        checks: preflight.checks.clone(),
        errors: preflight.errors.clone(),
        preflight: Some(preflight.evidence.clone()),
        codex_execution: None,
        gate_results: Vec::new(),
    };
    let status = build_run_status(
        context,
        "dry_run",
        if preflight_ok {
            "dry_run_ready"
        } else {
            "dry_run_failed"
        },
        preflight_ok,
        Vec::new(),
        report.errors.clone(),
    )?;
    persist_run_packet(context, &report, &status, &[], None, "dry_run")?;

    if !preflight_ok {
        return Err(Error::Validation(report.errors.join("\n")));
    }

    writeln!(
        writer,
        "OK: execute-agent-maintenance dry-run packet prepared."
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "run_id: {}", context.run_id)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "run_dir: {}", context.run_dir_rel)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "closeout_command: {}", context.closeout_command)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "closeout remains manual; it is not run automatically."
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

fn execute_write_mode<W: Write>(
    workspace_root: &Path,
    context: &Context,
    writer: &mut W,
) -> Result<(), Error> {
    let (prepared_contract, frozen_prompt) = load_prepared_packet(context)?;
    validate_prepared_packet(context, &prepared_contract, &frozen_prompt)?;

    let preflight = run_codex_preflight(workspace_root, context)?;
    let mut checks = preflight.checks.clone();
    let mut errors = preflight.errors.clone();
    let mut gate_results = Vec::new();

    let codex_execution = if errors.is_empty() {
        Some(execute_codex_write(
            workspace_root,
            context,
            &frozen_prompt,
        )?)
    } else {
        None
    };

    let mut written_paths = Vec::new();
    if let Some(execution) = codex_execution.as_ref() {
        if execution.exit_code != 0 {
            errors.push(format!(
                "codex execution failed with exit code {}; inspect `{}` and `{}`",
                execution.exit_code, execution.stdout_path, execution.stderr_path
            ));
            checks.push(ValidationCheck {
                name: "codex_execution_exit".to_string(),
                ok: false,
                message: format!("codex exited with {}", execution.exit_code),
            });
        } else {
            checks.push(ValidationCheck {
                name: "codex_execution_exit".to_string(),
                ok: true,
                message: "codex exited successfully".to_string(),
            });
        }

        let snapshot_after_codex =
            snapshot_workspace(workspace_root, &[Path::new(EXECUTION_RUNS_ROOT)])?;
        let diff_after_codex = diff_snapshots(&prepared_contract.baseline, &snapshot_after_codex);
        validate_written_paths(
            workspace_root,
            context,
            &diff_after_codex,
            "post_codex",
            &mut checks,
            &mut errors,
        )?;

        if errors.is_empty() {
            gate_results = run_green_gates(
                workspace_root,
                &prepared_contract.green_gates,
                &mut checks,
                &mut errors,
            )?;
            let snapshot_after_gates =
                snapshot_workspace(workspace_root, &[Path::new(EXECUTION_RUNS_ROOT)])?;
            written_paths = diff_snapshots(&prepared_contract.baseline, &snapshot_after_gates);
            validate_written_paths(
                workspace_root,
                context,
                &written_paths,
                "post_gates",
                &mut checks,
                &mut errors,
            )?;
        } else {
            written_paths = diff_after_codex;
        }
    }

    let passed = errors.is_empty();
    let report = ValidationReport {
        workflow_version: WORKFLOW_VERSION.to_string(),
        run_id: context.run_id.clone(),
        status: if passed { "pass" } else { "fail" }.to_string(),
        checks,
        errors: errors.clone(),
        preflight: Some(preflight.evidence),
        codex_execution: codex_execution.clone(),
        gate_results,
    };
    let status = build_run_status(
        context,
        "write",
        if passed {
            "write_validated"
        } else {
            "write_failed"
        },
        passed,
        written_paths.clone(),
        errors.clone(),
    )?;
    persist_run_packet(
        context,
        &report,
        &status,
        &written_paths,
        codex_execution.as_ref(),
        "write",
    )?;

    if !passed {
        return Err(Error::Validation(errors.join("\n")));
    }

    writeln!(
        writer,
        "OK: execute-agent-maintenance write validation complete."
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "run_id: {}", context.run_id)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "written_paths: {}", written_paths.len())
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "closeout_command: {}", context.closeout_command)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "closeout remains manual; it is not run automatically."
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

fn write_preview<W: Write>(
    writer: &mut W,
    context: &Context,
    write_mode: bool,
) -> Result<(), Error> {
    let detected_release = context
        .envelope
        .request
        .detected_release
        .as_ref()
        .ok_or_else(|| {
            Error::Validation(format!(
                "maintenance request `{}` is missing detected_release metadata",
                context.envelope.request.relative_path
            ))
        })?;
    writeln!(
        writer,
        "Execution relay mode: {}",
        if write_mode { "WRITE" } else { "DRY RUN" }
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "request: {}",
        context.envelope.request.relative_path
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "agent: {}", context.envelope.request.agent_id)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "run_id: {}", context.run_id)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "run_dir: {}", context.run_dir_rel)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "target_version: {}",
        detected_release.target_version
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "branch_name: {}", detected_release.branch_name)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "executor: {}", context.execution_contract.executor)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "prompt_sha256: {}",
        context.execution_contract.prompt_sha256
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    write_list(
        writer,
        "writable_surfaces",
        &context.execution_contract.writable_surfaces,
    )?;
    write_list(
        writer,
        "read_only_inputs",
        &context.execution_contract.read_only_inputs,
    )?;
    write_list(
        writer,
        "ordered_commands",
        &context.execution_contract.ordered_commands,
    )?;
    write_list(
        writer,
        "green_gates",
        &context.execution_contract.green_gates,
    )?;
    writeln!(
        writer,
        "recreate_packet_command: {}",
        context.execution_contract.recovery.recreate_packet_command
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "reopen_pr_body_path: {}",
        context.execution_contract.recovery.reopen_pr_body_path
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "reopen_pr_branch: {}",
        context.execution_contract.recovery.reopen_pr_branch
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    write_list(
        writer,
        "recovery_notes",
        &context.execution_contract.recovery.notes,
    )?;
    writeln!(writer, "closeout_command: {}", context.closeout_command)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "closeout_manual: true")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

fn write_list<W: Write>(writer: &mut W, label: &str, items: &[String]) -> Result<(), Error> {
    writeln!(writer, "{label}:").map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    for item in items {
        writeln!(writer, "- {item}")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
    Ok(())
}

fn persist_run_packet(
    context: &Context,
    report: &ValidationReport,
    status: &RunStatus,
    written_paths: &[String],
    codex_execution: Option<&CodexExecutionEvidence>,
    mode: &str,
) -> Result<(), Error> {
    fs::create_dir_all(&context.run_dir)
        .map_err(|err| Error::Internal(format!("create {}: {err}", context.run_dir.display())))?;
    if let Some(input_contract) = context.input_contract.as_ref() {
        write_json(
            &context.run_dir.join(INPUT_CONTRACT_FILE_NAME),
            input_contract,
        )?;
    }
    if !context.run_dir.join(INPUT_CONTRACT_FILE_NAME).exists() {
        return Err(Error::Internal(format!(
            "prepared input contract missing at {}",
            context.run_dir.join(INPUT_CONTRACT_FILE_NAME).display()
        )));
    }
    write_string(
        &context.run_dir.join(PROMPT_FILE_NAME),
        &context.rendered_packet.prompt_contents,
    )?;
    write_json(&context.run_dir.join(VALIDATION_REPORT_FILE_NAME), report)?;
    write_json(&context.run_dir.join(RUN_STATUS_FILE_NAME), status)?;
    write_json(
        &context.run_dir.join(WRITTEN_PATHS_FILE_NAME),
        &written_paths,
    )?;
    write_string(
        &context.run_dir.join(RUN_SUMMARY_FILE_NAME),
        &render_run_summary(context, report, written_paths, mode),
    )?;
    if let Some(execution) = codex_execution {
        write_json(&context.run_dir.join(CODEX_EXECUTION_FILE_NAME), execution)?;
    }
    Ok(())
}

fn load_prepared_packet(context: &Context) -> Result<(InputContract, String), Error> {
    let input_path = context.run_dir.join(INPUT_CONTRACT_FILE_NAME);
    let prompt_path = context.run_dir.join(PROMPT_FILE_NAME);
    if !input_path.is_file() || !prompt_path.is_file() {
        return Err(Error::Validation(format!(
            "execute-agent-maintenance --write requires a matching dry-run packet under `{}`; rerun `execute-agent-maintenance --dry-run --request {}` first",
            context.run_dir_rel, context.envelope.request.relative_path
        )));
    }
    let input_contract = load_json::<InputContract>(&input_path)?;
    let prompt = read_string(&prompt_path)?;
    Ok((input_contract, prompt))
}

fn validate_prepared_packet(
    context: &Context,
    prepared: &InputContract,
    frozen_prompt: &str,
) -> Result<(), Error> {
    let mut mismatches = Vec::new();
    if prepared.run_id != context.run_id {
        mismatches.push(format!(
            "run_id mismatch: prepared `{}` vs requested `{}`",
            prepared.run_id, context.run_id
        ));
    }
    if prepared.request_path != context.envelope.request.relative_path {
        mismatches.push(format!(
            "request path mismatch: prepared `{}` vs live `{}`",
            prepared.request_path, context.envelope.request.relative_path
        ));
    }
    if prepared.request_sha256 != context.envelope.request.sha256 {
        mismatches.push(format!(
            "request sha256 mismatch: prepared `{}` vs live `{}`",
            prepared.request_sha256, context.envelope.request.sha256
        ));
    }
    if prepared.agent_id != context.envelope.request.agent_id {
        mismatches.push(format!(
            "agent_id mismatch: prepared `{}` vs live `{}`",
            prepared.agent_id, context.envelope.request.agent_id
        ));
    }
    let detected_release = context
        .envelope
        .request
        .detected_release
        .as_ref()
        .ok_or_else(|| {
            Error::Validation(format!(
                "maintenance request `{}` is missing detected_release metadata",
                context.envelope.request.relative_path
            ))
        })?;
    if prepared.target_version != detected_release.target_version {
        mismatches.push(format!(
            "target_version mismatch: prepared `{}` vs live `{}`",
            prepared.target_version, detected_release.target_version
        ));
    }
    if prepared.branch_name != detected_release.branch_name {
        mismatches.push(format!(
            "branch_name mismatch: prepared `{}` vs live `{}`",
            prepared.branch_name, detected_release.branch_name
        ));
    }
    if prepared.prompt_sha256 != context.execution_contract.prompt_sha256 {
        mismatches.push(format!(
            "prompt digest mismatch: prepared `{}` vs live `{}`",
            prepared.prompt_sha256, context.execution_contract.prompt_sha256
        ));
    }
    let frozen_prompt_sha256 = hex::encode(Sha256::digest(frozen_prompt.as_bytes()));
    if frozen_prompt_sha256 != prepared.prompt_sha256 {
        mismatches.push(format!(
            "frozen prompt digest mismatch: prepared `{}` vs frozen `{frozen_prompt_sha256}`",
            prepared.prompt_sha256
        ));
    }
    if frozen_prompt != context.rendered_packet.prompt_contents {
        mismatches.push(
            "frozen prompt contents diverge from current request truth; rerun execute-agent-maintenance --dry-run before write mode".to_string(),
        );
    }
    if prepared.writable_surfaces != context.execution_contract.writable_surfaces {
        mismatches.push("writable_surfaces diverged from the prepared baseline".to_string());
    }
    if prepared.green_gates != context.execution_contract.green_gates {
        mismatches.push("green_gates diverged from the prepared baseline".to_string());
    }
    if prepared.closeout_path != context.execution_contract.closeout_path {
        mismatches.push("closeout_path diverged from the prepared baseline".to_string());
    }
    if prepared.closeout_command != context.closeout_command {
        mismatches.push("closeout command diverged from the prepared baseline".to_string());
    }
    if !mismatches.is_empty() {
        return Err(Error::Validation(format!(
            "prepared run packet `{}` no longer matches request truth:\n{}",
            context.run_dir_rel,
            mismatches.join("\n")
        )));
    }
    Ok(())
}

fn run_codex_preflight(workspace_root: &Path, context: &Context) -> Result<PreflightResult, Error> {
    let before = snapshot_workspace(workspace_root, &[Path::new(EXECUTION_RUNS_ROOT)])?;
    let prompt = format!(
        "Repository preflight for execute-agent-maintenance.\nReply with exactly {PREFLIGHT_SENTINEL}.\nDo not write, edit, or delete any files.\nDo not run any commands.\n"
    );
    let (argv, output) =
        spawn_codex_exec(&context.codex_binary, workspace_root, &prompt, &[], true)?;
    let after = snapshot_workspace(workspace_root, &[Path::new(EXECUTION_RUNS_ROOT)])?;
    let changed_paths = diff_snapshots(&before, &after);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let exit_code = output.status.code().unwrap_or(1);
    let evidence = SubprocessEvidence {
        binary: context.codex_binary.clone(),
        argv,
        exit_code,
        stdout: stdout.clone(),
        stderr: stderr.clone(),
    };

    let mut checks = Vec::new();
    let mut errors = Vec::new();
    let exit_ok = exit_code == 0;
    checks.push(ValidationCheck {
        name: "codex_preflight_exit".to_string(),
        ok: exit_ok,
        message: if exit_ok {
            "codex preflight exited successfully".to_string()
        } else {
            format!("codex preflight exited with {exit_code}")
        },
    });
    if !exit_ok {
        errors.push(format!(
            "local Codex preflight failed; fix binary/auth and rerun `execute-agent-maintenance --dry-run --request {}` before write mode",
            context.envelope.request.relative_path
        ));
    }

    let output_ok = stdout.contains(PREFLIGHT_SENTINEL);
    checks.push(ValidationCheck {
        name: "codex_preflight_output".to_string(),
        ok: output_ok,
        message: if output_ok {
            "codex preflight produced the expected sentinel".to_string()
        } else {
            format!("codex preflight did not emit `{PREFLIGHT_SENTINEL}`")
        },
    });
    if !output_ok {
        errors.push(format!(
            "local Codex preflight did not confirm readiness with `{PREFLIGHT_SENTINEL}`"
        ));
    }

    let diff_ok = changed_paths.is_empty();
    checks.push(ValidationCheck {
        name: "codex_preflight_noop".to_string(),
        ok: diff_ok,
        message: if diff_ok {
            "codex preflight made no repo-owned changes".to_string()
        } else {
            format!(
                "codex preflight wrote unexpected paths: {}",
                changed_paths.join(", ")
            )
        },
    });
    if !diff_ok {
        errors.push(format!(
            "local Codex preflight must not mutate the workspace; changed paths: {}",
            changed_paths.join(", ")
        ));
    }

    Ok(PreflightResult {
        evidence,
        checks,
        errors,
    })
}

fn execute_codex_write(
    workspace_root: &Path,
    context: &Context,
    prompt: &str,
) -> Result<CodexExecutionEvidence, Error> {
    let envs = vec![
        (
            "XTASK_AGENT_MAINTENANCE_RUN_ID".to_string(),
            context.run_id.clone(),
        ),
        (
            "XTASK_AGENT_MAINTENANCE_RUN_DIR".to_string(),
            context.run_dir.to_string_lossy().into_owned(),
        ),
        (
            "XTASK_AGENT_MAINTENANCE_REQUEST_PATH".to_string(),
            context.envelope.request.relative_path.clone(),
        ),
        (
            "XTASK_AGENT_MAINTENANCE_ALLOWED_WRITE_SURFACES".to_string(),
            context.execution_contract.writable_surfaces.join("\n"),
        ),
    ];
    let (argv, output) =
        spawn_codex_exec(&context.codex_binary, workspace_root, prompt, &envs, false)?;
    let stdout_path = context.run_dir.join(CODEX_STDOUT_FILE_NAME);
    let stderr_path = context.run_dir.join(CODEX_STDERR_FILE_NAME);
    write_string(&stdout_path, &String::from_utf8_lossy(&output.stdout))?;
    write_string(&stderr_path, &String::from_utf8_lossy(&output.stderr))?;

    Ok(CodexExecutionEvidence {
        workflow_version: WORKFLOW_VERSION.to_string(),
        generated_at: now_rfc3339()?,
        run_id: context.run_id.clone(),
        binary: context.codex_binary.clone(),
        argv,
        prompt_path: context
            .run_dir
            .join(PROMPT_FILE_NAME)
            .to_string_lossy()
            .into_owned(),
        stdout_path: stdout_path.to_string_lossy().into_owned(),
        stderr_path: stderr_path.to_string_lossy().into_owned(),
        exit_code: output.status.code().unwrap_or(1),
    })
}

fn spawn_codex_exec(
    binary: &str,
    workspace_root: &Path,
    prompt: &str,
    envs: &[(String, String)],
    quiet: bool,
) -> Result<(Vec<String>, Output), Error> {
    let mut argv = vec![
        "exec".to_string(),
        "--skip-git-repo-check".to_string(),
        "--dangerously-bypass-approvals-and-sandbox".to_string(),
    ];
    if quiet {
        argv.push("--quiet".to_string());
    }
    argv.extend([
        "--cd".to_string(),
        workspace_root.to_string_lossy().into_owned(),
    ]);

    let mut command = Command::new(binary);
    command
        .current_dir(workspace_root)
        .args(&argv)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in envs {
        command.env(key, value);
    }
    let mut child = command
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
    Ok((argv, output))
}

fn run_green_gates(
    workspace_root: &Path,
    commands: &[String],
    checks: &mut Vec<ValidationCheck>,
    errors: &mut Vec<String>,
) -> Result<Vec<GateEvidence>, Error> {
    let mut results = Vec::new();
    for command in commands {
        let output = Command::new("sh")
            .current_dir(workspace_root)
            .arg("-lc")
            .arg(command)
            .output()
            .map_err(|err| Error::Internal(format!("spawn green gate `{command}`: {err}")))?;
        let exit_code = output.status.code().unwrap_or(1);
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let ok = exit_code == 0;
        checks.push(ValidationCheck {
            name: format!("green_gate::{command}"),
            ok,
            message: if ok {
                "gate passed".to_string()
            } else {
                format!("gate exited with {exit_code}")
            },
        });
        if !ok {
            errors.push(format!(
                "green gate failed: `{command}` exited with {exit_code}"
            ));
        }
        results.push(GateEvidence {
            command: command.clone(),
            exit_code,
            stdout,
            stderr,
        });
        if !ok {
            break;
        }
    }
    Ok(results)
}

fn validate_written_paths(
    workspace_root: &Path,
    context: &Context,
    changed_paths: &[String],
    phase: &str,
    checks: &mut Vec<ValidationCheck>,
    errors: &mut Vec<String>,
) -> Result<(), Error> {
    let jail = WorkspacePathJail::new(workspace_root)?;
    let mut violations = Vec::new();
    for path in changed_paths {
        jail.resolve(Path::new(path))?;
        if path == &context.execution_contract.closeout_path {
            violations.push(format!(
                "{path} (closeout remains manual and must not be written by execute-agent-maintenance)"
            ));
            continue;
        }
        if !matches_any_surface(path, &context.execution_contract.writable_surfaces)? {
            violations.push(path.clone());
        }
    }
    let boundary_ok = violations.is_empty();
    checks.push(ValidationCheck {
        name: format!("write_boundary::{phase}"),
        ok: boundary_ok,
        message: if boundary_ok {
            format!("{phase} diff stayed within writable_surfaces")
        } else {
            format!(
                "{phase} diff escaped writable_surfaces: {}",
                violations.join(", ")
            )
        },
    });
    if !boundary_ok {
        errors.push(format!(
            "write boundary violation during {phase}: {}",
            violations.join(", ")
        ));
    }

    let has_runtime_write = !changed_paths.is_empty();
    checks.push(ValidationCheck {
        name: format!("runtime_write::{phase}"),
        ok: has_runtime_write,
        message: if has_runtime_write {
            format!(
                "{phase} diff recorded {} changed paths",
                changed_paths.len()
            )
        } else {
            "no runtime-owned changes were recorded".to_string()
        },
    });
    if !has_runtime_write {
        errors.push(
            "write mode produced no runtime-owned output changes from the prepared baseline"
                .to_string(),
        );
    }
    Ok(())
}

fn matches_any_surface(path: &str, surfaces: &[String]) -> Result<bool, Error> {
    surfaces.iter().try_fold(false, |matched, surface| {
        if matched {
            return Ok(true);
        }
        Ok(glob_matches(surface, path)?)
    })
}

fn glob_matches(pattern: &str, path: &str) -> Result<bool, Error> {
    let mut regex = String::from("^");
    let mut chars = pattern.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                    regex.push_str(".*");
                } else {
                    regex.push_str("[^/]*");
                }
            }
            '.' | '+' | '(' | ')' | '|' | '^' | '$' | '{' | '}' | '[' | ']' | '\\' => {
                regex.push('\\');
                regex.push(ch);
            }
            other => regex.push(other),
        }
    }
    regex.push('$');
    let compiled = Regex::new(&regex).map_err(|err| {
        Error::Internal(format!(
            "compile writable surface pattern `{pattern}`: {err}"
        ))
    })?;
    Ok(compiled.is_match(path))
}

fn build_run_status(
    context: &Context,
    mode: &str,
    status: &str,
    validation_passed: bool,
    written_paths: Vec<String>,
    errors: Vec<String>,
) -> Result<RunStatus, Error> {
    let detected_release = context
        .envelope
        .request
        .detected_release
        .as_ref()
        .ok_or_else(|| {
            Error::Validation(format!(
                "maintenance request `{}` is missing detected_release metadata",
                context.envelope.request.relative_path
            ))
        })?;
    Ok(RunStatus {
        workflow_version: WORKFLOW_VERSION.to_string(),
        generated_at: now_rfc3339()?,
        run_id: context.run_id.clone(),
        host_surface: EXECUTE_HOST_SURFACE.to_string(),
        mode: mode.to_string(),
        status: status.to_string(),
        validation_passed,
        request_path: context.envelope.request.relative_path.clone(),
        packet_root: context.run_dir_rel.clone(),
        agent_id: context.envelope.request.agent_id.clone(),
        target_version: detected_release.target_version.clone(),
        branch_name: detected_release.branch_name.clone(),
        written_paths,
        errors,
    })
}

fn render_run_summary(
    context: &Context,
    report: &ValidationReport,
    written_paths: &[String],
    mode: &str,
) -> String {
    let detected_release = context
        .envelope
        .request
        .detected_release
        .as_ref()
        .expect("automated request requires detected_release");
    let written = if written_paths.is_empty() {
        "- none".to_string()
    } else {
        markdown_list(written_paths)
    };
    let green_gates = if report.gate_results.is_empty() {
        "- not run".to_string()
    } else {
        markdown_list(
            &report
                .gate_results
                .iter()
                .map(|gate| format!("`{}` (exit {})", gate.command, gate.exit_code))
                .collect::<Vec<_>>(),
        )
    };
    format!(
        concat!(
            "# Execute Agent Maintenance Run\n\n",
            "- mode: `{mode}`\n",
            "- status: `{status}`\n",
            "- run id: `{run_id}`\n",
            "- request: `{request_path}`\n",
            "- agent: `{agent_id}`\n",
            "- target version: `{target_version}`\n",
            "- branch: `{branch_name}`\n",
            "- executor: `{executor}`\n",
            "- prompt sha256: `{prompt_sha256}`\n\n",
            "## Writable surfaces\n\n{writable_surfaces}\n\n",
            "## Green gates\n\n{green_gates}\n\n",
            "## Written paths\n\n{written}\n\n",
            "## Recovery\n\n",
            "- recreate packet command: `{recreate}`\n",
            "- reopen PR body path: `{reopen_body}`\n",
            "- reopen PR branch: `{reopen_branch}`\n\n",
            "## Closeout\n\n",
            "`{closeout}`\n\n",
            "Closeout remains manual; execute-agent-maintenance never runs it automatically.\n"
        ),
        mode = mode,
        status = report.status,
        run_id = context.run_id,
        request_path = context.envelope.request.relative_path,
        agent_id = context.envelope.request.agent_id,
        target_version = detected_release.target_version,
        branch_name = detected_release.branch_name,
        executor = context.execution_contract.executor,
        prompt_sha256 = context.execution_contract.prompt_sha256,
        writable_surfaces = markdown_list(&context.execution_contract.writable_surfaces),
        green_gates = green_gates,
        written = written,
        recreate = context.execution_contract.recovery.recreate_packet_command,
        reopen_body = context.execution_contract.recovery.reopen_pr_body_path,
        reopen_branch = context.execution_contract.recovery.reopen_pr_branch,
        closeout = context.closeout_command,
    )
}

fn markdown_list(items: &[String]) -> String {
    items
        .iter()
        .map(|item| format!("- {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), Error> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| Error::Internal(format!("serialize {}: {err}", path.display())))?;
    bytes.push(b'\n');
    write_bytes(path, &bytes)
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
        .collect::<BTreeMap<_, _>>();
    let after_map = after
        .files
        .iter()
        .map(|file| (file.path.as_str(), file.sha256.as_str()))
        .collect::<BTreeMap<_, _>>();
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

fn resolve_codex_binary(args: &Args) -> String {
    args.codex_binary
        .clone()
        .or_else(|| std::env::var(CODEX_BINARY_ENV).ok())
        .unwrap_or_else(|| "codex".to_string())
}

fn generate_run_id() -> String {
    OffsetDateTime::now_utc()
        .format(
            &time::format_description::parse("[year][month][day]T[hour][minute][second]Z")
                .expect("valid time format"),
        )
        .unwrap_or_else(|_| "agent-maintenance-execute".to_string())
}

fn now_rfc3339() -> Result<String, Error> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|err| Error::Internal(format!("format timestamp: {err}")))
}

#[derive(Debug, Clone)]
struct PreflightResult {
    evidence: SubprocessEvidence,
    checks: Vec<ValidationCheck>,
    errors: Vec<String>,
}
