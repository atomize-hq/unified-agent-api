//! Validate prospective bytes against the caller index that copy-back will preserve.
use super::{
    candidate_git::{self, GitState},
    index_lock::IndexLock,
    tree::{io, Node, Tree},
    Error,
};
use std::{collections::BTreeSet, fs, io::Write, path::Path, process::Stdio};

/// A new E-bound evidence file, as the validated snapshot holds it.
pub(super) struct Bound {
    path: String,
    mode: &'static str,
    bytes: Vec<u8>,
}
/// A [`Bound`] file whose blob is written and whose index entry this relay owns.
#[derive(Debug)]
pub(super) struct Published {
    path: String,
    entry: String,
}

type Staged = std::collections::BTreeMap<String, String>;

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(bytes))
}

/// Whether `path` lies in an evidence destination: `.../reports/<version>/evidence/...`.
fn evidence_destination(path: &str) -> bool {
    let segments: Vec<_> = path.split('/').collect();
    segments
        .windows(3)
        .take(segments.len().saturating_sub(3))
        .any(|w| w[0] == "reports" && w[2] == "evidence")
}

/// Every path an E in `tree` binds (in any depth record, changed or not) and every payload a
/// bound file references by `{path, sha256}` anywhere in its JSON, transitively.
fn bound_closure(tree: &Tree, out: &mut BTreeSet<String>) {
    fn references(value: &serde_json::Value, out: &mut Vec<String>) {
        match value {
            serde_json::Value::Object(map) => {
                if let (Some(serde_json::Value::String(path)), Some(serde_json::Value::String(_))) =
                    (map.get("path"), map.get("sha256"))
                {
                    out.push(path.clone());
                }
                map.values().for_each(|v| references(v, out));
            }
            serde_json::Value::Array(items) => items.iter().for_each(|v| references(v, out)),
            _ => {}
        }
    }
    let mut queue = Vec::new();
    for (path, node) in tree {
        let Node::File(bytes, _) = node else {
            continue;
        };
        if !path.ends_with("/depth-record.json") {
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
                    queue.push(path.to_owned());
                }
            }
        }
    }
    let mut seen = 0usize;
    while let Some(path) = queue.pop() {
        seen += 1;
        if seen > 100_000 || !out.insert(path.clone()) {
            continue;
        }
        if let Some(Node::File(bytes, _)) = tree.get(&path) {
            if let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) {
                references(&value, &mut queue);
            }
        }
    }
}

/// The evidence path class: every file in an evidence destination before or after, and every
/// path an E binds before or after, with its transitive payload references.
fn evidence_paths(before: &Tree, after: &Tree, run_dir_rel: &str) -> BTreeSet<String> {
    let mut paths: BTreeSet<String> = before
        .keys()
        .chain(after.keys())
        .filter(|p| evidence_destination(p))
        .cloned()
        .collect();
    bound_closure(before, &mut paths);
    bound_closure(after, &mut paths);
    paths.retain(|p| !p.starts_with(&format!("{run_dir_rel}/")));
    paths
}

/// The host-side byte binding, by path class and independent of whether any record changed:
/// every file the candidate adds, changes or removes in the evidence path class must be
/// byte-identical to the host-staged bundle file at that path. Anything else refuses the whole
/// delivery. In production (`None`) nothing is staged, so any such change refuses.
pub(super) fn check_staged_closure(
    before: &Tree,
    after: &Tree,
    run_dir_rel: &str,
    staged: Option<&Staged>,
) -> Result<(), Error> {
    for path in evidence_paths(before, after, run_dir_rel) {
        let delivered = match (before.get(&path), after.get(&path)) {
            (Some(Node::File(old, _)), Some(Node::File(new, _))) if old == new => continue,
            (None | Some(Node::Directory(_)), None | Some(Node::Directory(_))) => continue,
            (_, Some(Node::File(bytes, _))) => Some(sha256(bytes)),
            _ => None,
        };
        let expected = staged.and_then(|s| s.get(&path));
        if delivered.is_none() || expected != delivered.as_ref() {
            let none = if staged.is_none() {
                " (nothing is staged without an isolated route)"
            } else {
                ""
            };
            return Err(Error::Validation(format!(
                "delivered evidence is not the bytes the host staged for this run{none}: {path}"
            )));
        }
    }
    Ok(())
}

/// New files of the evidence path class (E-bound receipts and the payloads they reference) that
/// are byte-identical to evidence the host itself authenticated and staged for this run (B.4
/// part 2). Publication reads bound evidence only from the index, so the relay validates, and
/// copy-back delivers, exactly the caller index plus these files. [`check_staged_closure`]
/// refuses every other change in that class. The bytes come from the validated snapshot.
pub(super) fn bound_new_evidence(
    before: &Tree,
    after: &Tree,
    run_dir_rel: &str,
    staged: Option<&Staged>,
) -> Vec<Bound> {
    let Some(staged) = staged else {
        return vec![];
    };
    evidence_paths(before, after, run_dir_rel)
        .into_iter()
        .filter(|p| before.get(p).is_none())
        .filter_map(|path| match after.get(&path) {
            Some(Node::File(bytes, mode)) if staged.get(&path) == Some(&sha256(bytes)) => {
                Some(Bound {
                    mode: if cfg!(unix) && mode & 0o111 != 0 {
                        "100755"
                    } else {
                        "100644"
                    },
                    bytes: bytes.clone(),
                    path,
                })
            }
            _ => None,
        })
        .collect()
}

/// Write each bound file's validated bytes as a blob in `root` and add exactly those index
/// entries. Live working files are never read, so bytes swapped after validation never gain an
/// index identity. The comparison and the update run under the caller's index lock, so no other
/// Git process can stage one of these paths in between. Refuses if the index is locked or already
/// holds any of these paths; on failure the index is unchanged (blobs alone are unreferenced).
pub(super) fn publish(root: &Path, bound: &[Bound]) -> Result<Vec<Published>, Error> {
    let mut published = Vec::new();
    for file in bound {
        let oid = hash_object(root, &file.bytes)?;
        published.push(Published {
            path: file.path.clone(),
            entry: format!("{} {oid} 0\t{}", file.mode, file.path),
        });
    }
    if published.is_empty() {
        return Ok(published);
    }
    let lock = IndexLock::acquire(root)?;
    if !entries(&lock, &published)?.is_empty() {
        return Err(Error::Validation(
            "caller index already holds a path this delivery would add".into(),
        ));
    }
    #[cfg(test)]
    super::index_lock::between::run(root);
    let mut args = vec!["update-index".to_owned(), "--add".to_owned()];
    for (file, entry) in bound.iter().zip(&published) {
        let oid = entry.entry.split(' ').nth(1).unwrap_or_default();
        args.extend([
            "--cacheinfo".into(),
            format!("{},{oid},{}", file.mode, file.path),
        ]);
    }
    let args: Vec<_> = args.iter().map(String::as_str).collect();
    lock.run(&args)?;
    lock.commit()?;
    Ok(published)
}

/// Remove the index entries [`publish`] added, only where the index still holds exactly them, under
/// the caller's index lock. An entry someone else changed is preserved and reported as a conflict,
/// never erased.
pub(super) fn unpublish(root: &Path, published: &[Published]) -> Result<(), Error> {
    if published.is_empty() {
        return Ok(());
    }
    let lock = IndexLock::acquire(root)?;
    let current = entries(&lock, published)?;
    let (owned, conflicts): (Vec<_>, Vec<_>) =
        published.iter().partition(|p| current.contains(&p.entry));
    #[cfg(test)]
    super::index_lock::between::run(root);
    if !owned.is_empty() {
        let mut args = vec!["update-index", "--force-remove", "--"];
        args.extend(owned.iter().map(|p| p.path.as_str()));
        lock.run(&args)?;
        lock.commit()?;
    }
    if conflicts.is_empty() {
        Ok(())
    } else {
        Err(Error::Validation(format!(
            "index restoration conflict; preserving the intervening entries: {}",
            conflicts
                .iter()
                .map(|p| p.path.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )))
    }
}

/// The stage entries the index holds for the published paths, as `mode oid stage\tpath`.
fn entries(lock: &IndexLock, published: &[Published]) -> Result<Vec<String>, Error> {
    let mut args = vec!["ls-files", "--stage", "-z", "--"];
    args.extend(published.iter().map(|p| p.path.as_str()));
    let raw = lock.run(&args)?;
    raw.split(|b| *b == 0)
        .filter(|row| !row.is_empty())
        .map(|row| String::from_utf8(row.to_vec()).map_err(|e| Error::Validation(e.to_string())))
        .collect()
}

fn hash_object(root: &Path, bytes: &[u8]) -> Result<String, Error> {
    let mut child = candidate_git::command(root)
        .args(["hash-object", "-w", "--no-filters", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(io)?;
    if let Err(e) = child
        .stdin
        .take()
        .ok_or_else(|| Error::Internal("missing Git stdin".into()))
        .and_then(|mut stdin| stdin.write_all(bytes).map_err(io))
    {
        let _ = child.kill();
        let _ = child.wait();
        return Err(e);
    }
    let output = child.wait_with_output().map_err(io)?;
    let oid = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !output.status.success() || oid.is_empty() || !oid.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Error::Validation("relay git hash-object failed".into()));
    }
    Ok(oid)
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
        if hash_object(candidate, &bytes)? != oid {
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
    indexed: &[Bound],
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
        Some(_) => publish(root, indexed).map(drop),
        None => Ok(()),
    }
    .and_then(|()| check());
    // This is private candidate metadata. Caller index and files are never written here.
    let index = jail.resolve(Path::new(".git/index"))?;
    fs::write(&index, executor).map_err(io)?;
    outcome
}

#[cfg(all(test, unix))]
#[path = "delivery_index_tests.rs"]
mod tests;
