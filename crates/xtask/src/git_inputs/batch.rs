use super::*;

type Entries = BTreeMap<Vec<u8>, Vec<u8>>;

/// One operation owns one index snapshot and one streaming child, including recursive walks.
/// Only the current working file and blob are buffered; no payload cache is retained.
pub(super) struct Session<'a> {
    root: &'a Path,
    borrowed: Option<&'a (PathBuf, String)>,
    entries: std::sync::OnceLock<Result<Entries, String>>,
    blobs: immutable::BlobReader,
    objects: BTreeSet<String>,
    read_objects: Vec<String>,
}

impl<'a> Session<'a> {
    pub(super) fn new(root: &'a Path, borrowed: Option<&'a (PathBuf, String)>) -> Self {
        Self {
            root,
            borrowed,
            entries: std::sync::OnceLock::new(),
            blobs: immutable::BlobReader::new(borrowed.map_or(root, |(caller, _)| caller)),
            objects: BTreeSet::new(),
            read_objects: Vec::new(),
        }
    }

    fn entries(&self) -> Result<&Entries, &String> {
        self.entries
            .get_or_init(|| load_entries(self.root, self.borrowed))
            .as_ref()
    }

    pub(super) fn listing(&self) -> listing::Listing {
        if self.borrowed.is_some() {
            return listing::Listing::default();
        }
        listing::Listing::from_paths(
            self.entries()
                .ok()
                .map(|entries| entries.keys().cloned().collect()),
        )
    }

    pub(super) fn read(&mut self, path: &str) -> Result<Input, String> {
        let full = safe_path(self.root, path)?;
        let entries = match self.entries() {
            Ok(entries) => entries,
            // Unavailable unrelated subtrees and malformed tree ordering require Git's
            // original bounded pathspec walk, preserving its results and diagnostics.
            Err(_) => return read(self.root, path, self.borrowed),
        };
        let index = entry(entries, path, self.borrowed.is_some());
        let (input, object) = prepare(&full, path, index, self.borrowed.is_some())?;
        if let Some(object) = object {
            if input.bytes.as_ref() != Some(&self.blobs.read(&object)?) {
                return Err(format!("working bytes disagree with index: {path}"));
            }
            if self.objects.insert(object.clone()) {
                self.read_objects.push(object);
            }
        }
        Ok(input)
    }

    /// Capture the ending stage/mode/object/path snapshot only after all filesystem checks.
    pub(super) fn finish_index(
        &self,
        files: &BTreeMap<String, Input>,
    ) -> Result<(listing::Listing, bool), String> {
        if self.borrowed.is_some() {
            return Ok((listing::Listing::default(), false));
        }
        let end = load_entries(self.root, None)?;
        let start = self.entries().map_err(Clone::clone)?;
        for path in files.keys() {
            if entry(start, path, false) != entry(&end, path, false) {
                return Err(format!("bound input changed during freeze: {path}"));
            }
        }
        let changed = start != &end;
        Ok((
            listing::Listing::from_paths(Some(end.into_keys().collect())),
            changed,
        ))
    }

    pub(super) fn check_objects(&self) -> Result<(), String> {
        let root = self.borrowed.map_or(self.root, |(caller, _)| caller);
        immutable::check_blobs(root, &self.read_objects)
    }
}

fn entry(entries: &Entries, path: &str, borrowed: bool) -> Vec<u8> {
    let mut index = entries.get(path.as_bytes()).cloned().unwrap_or_default();
    if !borrowed {
        // Literal ls-files pathspecs include descendants of a requested directory.
        let prefix = format!("{path}/").into_bytes();
        for (_, rows) in entries
            .range(prefix.clone()..)
            .take_while(|(p, _)| p.starts_with(&prefix))
        {
            index.extend(rows);
        }
    }
    index
}

#[cfg(test)]
pub(super) fn read_many<'a>(
    root: &'a Path,
    paths: &[&str],
    borrowed: Option<&'a (PathBuf, String)>,
) -> impl Iterator<Item = Result<Input, String>> + 'a {
    let mut session = Session::new(root, borrowed);
    let paths: Vec<_> = paths.iter().map(|path| (*path).to_owned()).collect();
    paths.into_iter().map(move |path| session.read(&path))
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
