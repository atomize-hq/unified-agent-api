use super::Error;
use std::{
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    process::{Command, Output, Stdio},
};

/// Host and delivery credentials that never reach the executor or a green gate. The executor's
/// own home and model-login resolution is left untouched; no auth store is opened.
pub(crate) const HOST_CREDENTIAL_ENV: &[&str] = &[
    "GH_TOKEN",
    "GITHUB_TOKEN",
    "GH_ENTERPRISE_TOKEN",
    "GITHUB_ENTERPRISE_TOKEN",
    "AUTOMATION_TOKEN",
    "GH_HOST",
    "GH_REPO",
    "GH_CONFIG_DIR",
    "CARGO_REGISTRY_TOKEN",
    "CRATES_IO_TOKEN",
    "ACTIONS_RUNTIME_TOKEN",
    "ACTIONS_ID_TOKEN_REQUEST_TOKEN",
    "ACTIONS_ID_TOKEN_REQUEST_URL",
];

/// Remove inherited Git redirections, relay variables and host credentials from a child.
pub(crate) fn scrub_host_env(command: &mut Command) {
    for (key, _) in std::env::vars_os() {
        let name = key.to_string_lossy();
        if name.starts_with("GIT_") || name.starts_with("XTASK_AGENT_MAINTENANCE_") {
            command.env_remove(&key);
        }
    }
    for key in HOST_CREDENTIAL_ENV {
        command.env_remove(key);
    }
}

/// Revokes the evidence channel when the executor spawn returns, on every path.
struct Revoke<'a>(Option<&'a crate::depth_evidence::Admission>);
impl Drop for Revoke<'_> {
    fn drop(&mut self) {
        if let Some(admission) = self.0 {
            admission.revoke();
        }
    }
}

pub(crate) fn spawn(
    binary: &str,
    workspace_root: &Path,
    prompt: &str,
    envs: &[(String, String)],
    channel: Option<&crate::depth_evidence::Admission>,
    wait: impl FnOnce(&mut std::process::Child) -> Result<std::process::ExitStatus, Error>,
) -> Result<(Vec<String>, Output), Error> {
    let revoke = Revoke(channel);
    let mut argv = vec![
        "exec".to_string(),
        "--skip-git-repo-check".to_string(),
        "--dangerously-bypass-approvals-and-sandbox".to_string(),
    ];
    argv.extend([
        "--cd".to_string(),
        workspace_root.to_string_lossy().into_owned(),
    ]);

    let mut stdout_file = tempfile::tempfile().map_err(|e| Error::Internal(e.to_string()))?;
    let mut stderr_file = tempfile::tempfile().map_err(|e| Error::Internal(e.to_string()))?;
    let mut command = Command::new(binary);
    scrub_host_env(&mut command);
    command
        .current_dir(workspace_root)
        .args(&argv)
        .stdin(Stdio::piped())
        .stdout(Stdio::from(
            stdout_file
                .try_clone()
                .map_err(|e| Error::Internal(e.to_string()))?,
        ))
        .stderr(Stdio::from(
            stderr_file
                .try_clone()
                .map_err(|e| Error::Internal(e.to_string()))?,
        ));
    for (key, value) in envs {
        command.env(key, value);
    }
    let mut child = command
        .spawn()
        .map_err(|err| Error::Internal(format!("spawn codex binary `{binary}`: {err}")))?;
    if let Some(admission) = channel {
        admission.admit(child.id());
    }
    let write_prompt = child
        .stdin
        .take()
        .ok_or_else(|| Error::Internal("codex exec stdin was not captured".into()))
        .and_then(|mut stdin| {
            stdin
                .write_all(prompt.as_bytes())
                .map_err(|e| Error::Internal(format!("write codex prompt to stdin: {e}")))
        });
    if let Err(error) = write_prompt {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    let status = wait(&mut child);
    // The executor has exited (or was killed and reaped): no process it left behind, and no gate
    // that runs next, can reach the channel.
    drop(revoke);
    let status = status?;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    stdout_file
        .seek(SeekFrom::Start(0))
        .and_then(|_| stdout_file.read_to_end(&mut stdout))
        .map_err(|e| Error::Internal(e.to_string()))?;
    stderr_file
        .seek(SeekFrom::Start(0))
        .and_then(|_| stderr_file.read_to_end(&mut stderr))
        .map_err(|e| Error::Internal(e.to_string()))?;
    let output = Output {
        status,
        stdout,
        stderr,
    };
    Ok((argv, output))
}
