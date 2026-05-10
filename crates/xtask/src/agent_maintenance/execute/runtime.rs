use std::{
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

use super::{
    now_rfc3339,
    packet::write_string,
    types::{
        CodexExecutionEvidence, Context, GateEvidence, PreflightResult, SubprocessEvidence,
        ValidationCheck,
    },
    validate::{diff_snapshots, snapshot_workspace},
    Error, CODEX_STDERR_FILE_NAME, CODEX_STDOUT_FILE_NAME, EXECUTION_RUNS_ROOT, PREFLIGHT_SENTINEL,
    PROMPT_FILE_NAME, WORKFLOW_VERSION,
};

const EXECUTION_HOST_PREFLIGHT: &str = "local execution-host preflight";

pub(super) fn run_codex_preflight(
    workspace_root: &Path,
    context: &Context,
) -> Result<PreflightResult, Error> {
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
            "{EXECUTION_HOST_PREFLIGHT} failed; fix the Codex binary/auth state and rerun `execute-agent-maintenance --dry-run --request {}` before write mode",
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
            "{EXECUTION_HOST_PREFLIGHT} did not confirm readiness with `{PREFLIGHT_SENTINEL}`"
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
            "{EXECUTION_HOST_PREFLIGHT} must not mutate the workspace; changed paths: {}",
            changed_paths.join(", ")
        ));
    }

    Ok(PreflightResult {
        evidence,
        checks,
        errors,
    })
}

pub(super) fn execute_codex_write(
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

pub(super) fn run_green_gates(
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

fn spawn_codex_exec(
    binary: &str,
    workspace_root: &Path,
    prompt: &str,
    envs: &[(String, String)],
    _quiet: bool,
) -> Result<(Vec<String>, Output), Error> {
    let mut argv = vec![
        "exec".to_string(),
        "--skip-git-repo-check".to_string(),
        "--dangerously-bypass-approvals-and-sandbox".to_string(),
    ];
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
