use std::{fs, io::Write, path::Path};

use serde::{Deserialize, Serialize};

use super::{
    now_rfc3339,
    types::{CodexExecutionEvidence, Context, InputContract, RunStatus, ValidationReport},
    Error, CODEX_EXECUTION_FILE_NAME, EXECUTE_HOST_SURFACE, INPUT_CONTRACT_FILE_NAME,
    PROMPT_FILE_NAME, RUN_STATUS_FILE_NAME, RUN_SUMMARY_FILE_NAME, VALIDATION_REPORT_FILE_NAME,
    WORKFLOW_VERSION, WRITTEN_PATHS_FILE_NAME,
};

pub(super) fn write_preview<W: Write>(
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

pub(super) fn persist_run_packet(
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

pub(super) fn load_prepared_packet(context: &Context) -> Result<(InputContract, String), Error> {
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

pub(super) fn build_run_status(
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

pub(super) fn write_string(path: &Path, value: &str) -> Result<(), Error> {
    write_bytes(path, value.as_bytes())
}

fn write_list<W: Write>(writer: &mut W, label: &str, items: &[String]) -> Result<(), Error> {
    writeln!(writer, "{label}:").map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    for item in items {
        writeln!(writer, "- {item}")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
    Ok(())
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
