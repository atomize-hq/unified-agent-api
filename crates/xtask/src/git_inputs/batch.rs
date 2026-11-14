use super::*;

/// Stream one path and blob at a time, reusing the entry listing and cat-file child. This
/// bounds additional payload memory to one working file plus one blob, including files larger
/// than any fixed chunk budget, without imposing an input size limit. Consumers compare and
/// record each result before requesting the next, preserving per-path error precedence.
pub(super) fn read_many(
    root: &Path,
    paths: &[&str],
    borrowed: Option<&(PathBuf, String)>,
) -> impl Iterator<Item = Result<Input, String>> {
    let root = root.to_owned();
    let borrowed = borrowed.cloned();
    let paths: Vec<_> = paths.iter().map(|path| (*path).to_owned()).collect();
    let mut entries = None;
    let object_root = borrowed
        .as_ref()
        .map_or(root.as_path(), |(caller, _)| caller.as_path());
    let mut blobs = immutable::BlobReader::new(object_root);
    paths.into_iter().map(move |path| {
        let full = safe_path(&root, &path)?;
        let entries = entries.get_or_insert_with(|| load_entries(&root, borrowed.as_ref()));
        let entries = match entries {
            Ok(entries) => entries,
            // Recursive listings can fail on unrelated unavailable subtrees. Unsorted and
            // duplicate tree entries also need Git's original bounded pathspec walk.
            Err(_) => return read(&root, &path, borrowed.as_ref()),
        };
        let mut index = entries.get(path.as_bytes()).cloned().unwrap_or_default();
        if borrowed.is_none() {
            // Literal ls-files pathspecs include descendants of a requested directory.
            let prefix = format!("{path}/").into_bytes();
            for (_, rows) in entries
                .range(prefix.clone()..)
                .take_while(|(p, _)| p.starts_with(&prefix))
            {
                index.extend(rows);
            }
        }
        let (input, object) = prepare(&full, &path, index, borrowed.is_some())?;
        if let Some(object) = object {
            if input.bytes.as_ref() != Some(&blobs.read(&object)?) {
                return Err(format!("working bytes disagree with index: {path}"));
            }
        }
        Ok(input)
    })
}

fn load_entries(
    root: &Path,
    borrowed: Option<&(PathBuf, String)>,
) -> Result<BTreeMap<Vec<u8>, Vec<u8>>, String> {
    // Keep argv bounded and unrelated non-UTF8 paths opaque. Both borrowed readers share
    // the ordering/ancestor/duplicate validation before selecting from a complete tree.
    if let Some((caller, commit)) = borrowed {
        return Ok(immutable::Index::load(caller, commit)?.into_entries());
    }
    let rows = git(root, &["--literal-pathspecs", "ls-files", "--stage", "-z"])?;
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
