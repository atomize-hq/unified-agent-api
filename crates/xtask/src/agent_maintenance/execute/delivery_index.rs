//! Validate prospective bytes against the caller index that copy-back will preserve.
use super::{
    candidate_git::{self, GitState},
    tree::{io, Node, Tree},
    Error,
};
use std::{collections::BTreeSet, fs, io::Write, path::Path, process::Stdio};

/// New files that a changed depth record's E binds as evidence (B.4 part 2). Publication reads
/// bound evidence only from the index, so the relay validates, and copy-back delivers, exactly
/// the caller index plus these paths. Nothing else is ever indexed.
pub(super) fn bound_new_evidence(before: &Tree, after: &Tree, run_dir_rel: &str) -> Vec<String> {
    let mut bound = BTreeSet::new();
    for (path, node) in after {
        let Node::File(bytes, _) = node else {
            continue;
        };
        let unchanged = matches!(before.get(path), Some(Node::File(old, _)) if old == bytes);
        if !path.ends_with("/depth-record.json") || unchanged {
            continue;
        }
        let Ok(record) = serde_json::from_slice::<serde_json::Value>(bytes) else {
            continue;
        };
        for result in record["execution"]["results"]
            .as_array()
            .into_iter()
            .flatten()
        {
            for file in result["evidence"].as_array().into_iter().flatten() {
                if let Some(path) = file["path"].as_str() {
                    bound.insert(path.to_owned());
                }
            }
        }
    }
    bound
        .into_iter()
        .filter(|p| {
            before.get(p).is_none()
                && matches!(after.get(p), Some(Node::File(..)))
                && !p.starts_with(&format!("{run_dir_rel}/"))
        })
        .collect()
}

/// Add `paths` to the index of `root`, as their current working bytes.
pub(super) fn add_to_index(root: &Path, paths: &[String]) -> Result<(), Error> {
    if paths.is_empty() {
        return Ok(());
    }
    let mut args = vec!["update-index", "--add", "--"];
    args.extend(paths.iter().map(String::as_str));
    candidate_git::run(root, &args).map(drop)
}

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
    indexed: &[String],
    check: impl FnOnce() -> Result<T, Error>,
) -> Result<T, Error> {
    let jail = crate::workspace_mutation::WorkspacePathJail::new(root)?;
    let index = jail.resolve(Path::new(".git/index"))?;
    let executor = fs::read(&index).map_err(io)?;
    match state.and_then(|s| s.index_file.as_ref()) {
        Some(bytes) => fs::write(&index, bytes).map_err(io)?,
        None => fs::remove_file(&index).map_err(io)?,
    }
    let outcome = match state {
        Some(_) => add_to_index(root, indexed),
        None => Ok(()),
    }
    .and_then(|()| check());
    // This is private candidate metadata. Caller index and files are never written here.
    let index = jail.resolve(Path::new(".git/index"))?;
    fs::write(&index, executor).map_err(io)?;
    outcome
}
