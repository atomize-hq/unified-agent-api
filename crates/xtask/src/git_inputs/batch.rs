use super::*;
use std::{io::Write, process::Stdio};

/// Keep failures per path so callers can interleave freeze comparisons and recording in the
/// original order. A later read failure must not hide an earlier changed-during-freeze error.
pub(super) fn read_many(
    root: &Path,
    paths: &[&str],
    borrowed: Option<&(PathBuf, String)>,
) -> Vec<Result<Input, String>> {
    let full: Vec<_> = paths.iter().map(|path| safe_path(root, path)).collect();
    if full.iter().all(Result::is_err) {
        return full
            .into_iter()
            .map(|result| Err(result.unwrap_err()))
            .collect();
    }
    let entries = match entries(root, borrowed) {
        Ok(entries) => entries,
        // A recursive tree listing can encounter an unavailable unrelated subtree. Preserve
        // the bounded single-path result and exact Git errors on exceptional repositories.
        Err(_) => {
            return paths
                .iter()
                .map(|path| read(root, path, borrowed))
                .collect()
        }
    };
    let pending: Vec<_> = paths
        .iter()
        .zip(full)
        .map(|(path, full)| {
            let full = full?;
            let mut index = entries.get(path.as_bytes()).cloned().unwrap_or_default();
            if borrowed.is_none() {
                // Literal ls-files pathspecs include descendants, even for a requested directory.
                let prefix = format!("{path}/").into_bytes();
                for (_, rows) in entries
                    .range(prefix.clone()..)
                    .take_while(|(p, _)| p.starts_with(&prefix))
                {
                    index.extend(rows);
                }
            }
            prepare(&full, path, index, borrowed.is_some())
        })
        .collect();
    let objects: Vec<_> = pending
        .iter()
        .filter_map(|result| {
            result
                .as_ref()
                .ok()
                .and_then(|(_, object)| object.as_deref())
        })
        .collect();
    let object_root = borrowed.map_or(root, |(caller, _)| caller.as_path());
    let mut blobs = blobs(object_root, &objects).into_iter();
    pending
        .into_iter()
        .zip(paths)
        .map(|(result, path)| {
            let (input, object) = result?;
            if object.is_some() {
                let staged = blobs.next().ok_or("missing batch blob result")??;
                if input.bytes.as_ref() != Some(&staged) {
                    return Err(format!("working bytes disagree with index: {path}"));
                }
            }
            Ok(input)
        })
        .collect()
}

fn entries(
    root: &Path,
    borrowed: Option<&(PathBuf, String)>,
) -> Result<BTreeMap<Vec<u8>, Vec<u8>>, String> {
    // Neither ls-files nor ls-tree supports stdin pathspecs. Read the index/tree once and
    // select literal paths locally, keeping argv bounded and unrelated non-UTF8 paths opaque.
    let rows = if let Some((caller, commit)) = borrowed {
        git(
            caller,
            &[
                "--literal-pathspecs",
                "ls-tree",
                "-r",
                "-t",
                "-z",
                "--full-tree",
                commit,
            ],
        )?
    } else {
        git(root, &["--literal-pathspecs", "ls-files", "--stage", "-z"])?
    };
    let mut entries = BTreeMap::<Vec<u8>, Vec<u8>>::new();
    for row in rows.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let tab = row
            .iter()
            .position(|b| *b == b'\t')
            .ok_or("invalid Git index entry")?;
        let entry = entries.entry(row[tab + 1..].to_vec()).or_default();
        entry.extend_from_slice(row);
        entry.push(0);
    }
    Ok(entries)
}

pub(super) fn prepare(
    full: &Path,
    path: &str,
    index: Vec<u8>,
    borrowed: bool,
) -> Result<(Input, Option<String>), String> {
    if index.is_empty() {
        if fs::symlink_metadata(full).is_ok() {
            return Err(format!("untracked bound input: {path}"));
        }
        return Ok((Input { index, bytes: None }, None));
    }
    let entries: Vec<_> = index.split(|b| *b == 0).filter(|e| !e.is_empty()).collect();
    if entries.len() != 1 {
        return Err(format!("unmerged or non-file input: {path}"));
    }
    let entry = std::str::from_utf8(entries[0]).map_err(|e| e.to_string())?;
    let (meta, indexed_path) = entry.split_once('\t').ok_or("invalid Git index entry")?;
    let meta: Vec<_> = meta.split(' ').collect();
    if meta.len() != 3
        || !matches!(meta[0], "100644" | "100755")
        || if borrowed {
            meta[1] != "blob"
        } else {
            meta[2] != "0"
        }
        || indexed_path != path
    {
        return Err(format!("input must be a regular stage-0 file: {path}"));
    }
    if !fs::symlink_metadata(full)
        .map_err(|e| format!("{path}: {e}"))?
        .is_file()
    {
        return Err(format!("non-file input: {path}"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let working_executable = fs::metadata(full)
            .map_err(|e| e.to_string())?
            .permissions()
            .mode()
            & 0o100
            != 0;
        if working_executable != (meta[0] == "100755") {
            return Err(format!(
                "working executable mode disagrees with index: {path}"
            ));
        }
    }
    let bytes = fs::read(full).map_err(|e| format!("{path}: {e}"))?;
    let object = if borrowed { meta[2] } else { meta[1] }.to_owned();
    Ok((
        Input {
            index,
            bytes: Some(bytes),
        },
        Some(object),
    ))
}

fn blobs(root: &Path, objects: &[&str]) -> Vec<Result<Vec<u8>, String>> {
    if objects.is_empty() {
        return Vec::new();
    }
    let output = batch_output(root, objects);
    let mut remaining = output.as_deref().unwrap_or_default();
    objects
        .iter()
        .map(|object| {
            // Exceptional missing/corrupt/non-blob objects use the original command for its exact
            // error (and Git's blob conversion semantics). Successful batches use two processes.
            take_blob(&mut remaining, object).or_else(|_| git(root, &["cat-file", "blob", object]))
        })
        .collect()
}

fn batch_output(root: &Path, objects: &[&str]) -> Result<Vec<u8>, String> {
    let mut child = command(root)
        .args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut stdin = child.stdin.take().ok_or("missing Git batch stdin")?;
    // Drain stdout/stderr while writing requests, including batches larger than pipe capacity.
    std::thread::scope(|scope| {
        let writer = scope.spawn(move || {
            for object in objects {
                writeln!(stdin, "{object}")?;
            }
            Ok::<_, std::io::Error>(())
        });
        let output = child.wait_with_output().map_err(|e| e.to_string());
        let written = writer
            .join()
            .map_err(|_| "Git batch writer panicked".to_owned())?;
        let output = output?;
        if !output.status.success() {
            return Err(format!(
                "git cat-file: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        written.map_err(|e| e.to_string())?;
        Ok(output.stdout)
    })
}

fn take_blob(remaining: &mut &[u8], object: &str) -> Result<Vec<u8>, String> {
    let newline = remaining
        .iter()
        .position(|b| *b == b'\n')
        .ok_or("invalid Git batch header")?;
    let header = std::str::from_utf8(&remaining[..newline]).map_err(|e| e.to_string())?;
    let fields: Vec<_> = header.split(' ').collect();
    *remaining = &remaining[newline + 1..];
    if fields.len() == 2 && fields[1] == "missing" {
        return Err("missing Git batch object".into());
    }
    if fields.len() != 3 || fields[0] != object {
        return Err("invalid Git batch object".into());
    }
    let size: usize = fields[2].parse().map_err(|_| "invalid Git batch size")?;
    let end = size.checked_add(1).ok_or("invalid Git batch size")?;
    let bytes = remaining.get(..end).ok_or("truncated Git batch object")?;
    if bytes[size] != b'\n' {
        return Err("invalid Git batch terminator".into());
    }
    *remaining = &remaining[end..];
    if fields[1] != "blob" {
        return Err("non-blob Git batch object".into());
    }
    Ok(bytes[..size].to_vec())
}
