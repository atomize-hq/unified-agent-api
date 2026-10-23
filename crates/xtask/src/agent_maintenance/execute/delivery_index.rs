//! Validate prospective bytes against the caller index that copy-back will preserve.
use super::{
    candidate_git::{self, GitState},
    tree::io,
    Error,
};
use std::{fs, io::Write, path::Path, process::Stdio};

pub(super) fn import(
    caller: &Path,
    candidate: &Path,
    state: Option<&GitState>,
) -> Result<(), Error> {
    let Some(state) = state else {
        return Ok(());
    };
    for row in state.index.split(|b| *b == 0).filter(|row| !row.is_empty()) {
        let row = std::str::from_utf8(row).map_err(|e| Error::Validation(e.to_string()))?;
        let (meta, _) = row
            .split_once('\t')
            .ok_or_else(|| Error::Validation("invalid caller index".into()))?;
        let parts: Vec<_> = meta.split(' ').collect();
        if parts.len() != 3 || !matches!(parts[0], "100644" | "100755") || parts[2] != "0" {
            return Err(Error::Validation(
                "relay requires regular stage-0 caller index entries".into(),
            ));
        }
        let oid = parts[1];
        if candidate_git::command(candidate)
            .args(["cat-file", "-e", oid])
            .output()
            .map_err(io)?
            .status
            .success()
        {
            continue;
        }
        let size = String::from_utf8(candidate_git::run(caller, &["cat-file", "-s", oid])?)
            .map_err(|e| Error::Validation(e.to_string()))?;
        if size
            .trim()
            .parse::<u64>()
            .map_err(|e| Error::Validation(e.to_string()))?
            > 536_870_912
        {
            return Err(Error::Validation(
                "caller index blob exceeds relay limit".into(),
            ));
        }
        let bytes = candidate_git::run(caller, &["cat-file", "blob", oid])?;
        let mut child = candidate_git::command(candidate)
            .args(["hash-object", "-w", "--stdin"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(io)?;
        if let Err(e) = child
            .stdin
            .take()
            .ok_or_else(|| Error::Internal("missing Git stdin".into()))?
            .write_all(&bytes)
        {
            let _ = child.kill();
            let _ = child.wait();
            return Err(io(e));
        }
        let output = child.wait_with_output().map_err(io)?;
        if !output.status.success() || String::from_utf8_lossy(&output.stdout).trim() != oid {
            return Err(Error::Validation(
                "caller indexed blob identity changed".into(),
            ));
        }
    }
    Ok(())
}

pub(super) fn validate<T>(
    root: &Path,
    state: Option<&GitState>,
    check: impl FnOnce() -> Result<T, Error>,
) -> Result<T, Error> {
    let jail = crate::workspace_mutation::WorkspacePathJail::new(root)?;
    let index = jail.resolve(Path::new(".git/index"))?;
    let executor = fs::read(&index).map_err(io)?;
    match state.and_then(|s| s.index_file.as_ref()) {
        Some(bytes) => fs::write(&index, bytes).map_err(io)?,
        None => fs::remove_file(&index).map_err(io)?,
    }
    let outcome = check();
    // This is private candidate metadata. Caller index and files are never written here.
    let index = jail.resolve(Path::new(".git/index"))?;
    fs::write(&index, executor).map_err(io)?;
    outcome
}
