//! A bounded, read-only view of regular stage-0 files. This is no serialization lock or receipt.
use crate::depth_record::FileRef;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

mod batch;
pub(crate) mod immutable;
mod listing;
mod packed;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Input {
    index: Vec<u8>,
    bytes: Option<Vec<u8>>,
}

#[derive(Clone)]
pub(crate) struct GitInputs {
    root: PathBuf,
    borrowed: Option<(PathBuf, String)>,
    immutable: Option<immutable::Immutable>,
    proposed: Option<crate::runtime_advertising::source::Source>,
    files: BTreeMap<String, Input>,
    directories: BTreeMap<String, Vec<String>>,
}

/// A frozen proposed source. It confers no authority; the attested view and caller must still
/// verify before and after use. Unlike `IsolatedView::source`, this includes scratch proposals.
pub(crate) struct EvaluationSource {
    pub(crate) source: crate::runtime_advertising::source::Source,
    pub(crate) root: PathBuf,
    pub(crate) fingerprint: String,
}

impl GitInputs {
    pub(crate) fn new(root: &Path) -> Result<Self, String> {
        let root = fs::canonicalize(root).map_err(|e| e.to_string())?;
        let top = git(&root, &["rev-parse", "--show-toplevel"])?;
        if Path::new(std::str::from_utf8(&top).map_err(|e| e.to_string())?.trim()) != root {
            return Err("freeze requires the Git workspace root".into());
        }
        Ok(Self {
            root,
            borrowed: None,
            immutable: None,
            proposed: None,
            files: BTreeMap::new(),
            directories: BTreeMap::new(),
        })
    }

    /// Bound evidence keeps the scratch working-file checks, borrowing only immutable objects.
    pub(crate) fn for_bound_evidence(root: &Path) -> Result<Self, String> {
        let Some(borrowed) = crate::isolated_maintenance::view_git_inputs(root)? else {
            return Self::new(root);
        };
        Ok(Self {
            root: fs::canonicalize(root).map_err(|e| e.to_string())?,
            immutable: Some(immutable::Immutable::new(&borrowed.0, &borrowed.1)),
            borrowed: Some(borrowed),
            proposed: None,
            files: BTreeMap::new(),
            directories: BTreeMap::new(),
        })
    }

    /// Reuse CP-P2's exact scratch binding, corroborated by the authenticated caller's inputs.
    pub(crate) fn borrow_view(
        root: &Path,
        caller: &Self,
        proposed: &crate::runtime_advertising::source::Source,
    ) -> Result<Self, String> {
        caller.verify()?;
        let mut view = Self::for_bound_evidence(root)?;
        let head = git(&caller.root, &["rev-parse", "--verify", "HEAD"])?;
        let head = std::str::from_utf8(&head)
            .map_err(|e| e.to_string())?
            .trim();
        if view
            .borrowed
            .as_ref()
            .map(|(root, commit)| (root.as_path(), commit.as_str()))
            != Some((caller.root.as_path(), head))
        {
            return Err("evaluation requires this caller's bound scratch Git inputs".into());
        }
        view.proposed = Some(proposed.clone());
        Ok(view)
    }

    pub(crate) fn is_proposed(&self) -> bool {
        self.proposed.is_some()
    }

    /// Immutable provenance always belongs to the original caller, never the scratch root.
    pub(crate) fn object_root(&self) -> &Path {
        self.borrowed
            .as_ref()
            .map_or(&self.root, |(root, _)| root.as_path())
    }

    pub(crate) fn provenance(
        &self,
        commit: &str,
    ) -> Result<crate::runtime_advertising::source::Source, String> {
        self.recheck_borrowed()?;
        crate::runtime_advertising::source::Source::tip(self.object_root(), commit)
    }

    fn recheck_borrowed(&self) -> Result<(), String> {
        if self.borrowed.is_some()
            && crate::isolated_maintenance::view_git_inputs(&self.root)? != self.borrowed
        {
            return Err("scratch Git inputs are no longer bound on this thread".into());
        }
        Ok(())
    }

    pub(crate) fn optional(&mut self, path: &str) -> Result<Option<Vec<u8>>, String> {
        if self.proposed.is_some() {
            // A proposed evaluation reads one frozen source, then verifies the full binding,
            // observed files and view before releasing any verdict. Reuse already checked
            // identities across obligations rather than reopening the caller for every read.
            if let Some(input) = self.files.get(path) {
                return Ok(input.bytes.clone());
            }
        } else {
            self.recheck_borrowed()?;
        }
        let input = read_cached(
            &self.root,
            path,
            self.borrowed.as_ref(),
            self.immutable.as_ref(),
        )?;
        self.record(path, input)
    }

    fn record(&mut self, path: &str, mut input: Input) -> Result<Option<Vec<u8>>, String> {
        if let Some(source) = &self.proposed {
            let bytes = source.optional_bytes(path)?;
            if bytes != input.bytes
                || (bytes.is_some()
                    && source.executable(path)? != input.index.starts_with(b"100755 "))
            {
                return Err(format!(
                    "proposed input changed after evaluation binding: {path}"
                ));
            }
            // Return the frozen IsolatedView source bytes, after retaining CP-P2's full
            // regular-file, pinned-blob and working-mode checks on the scratch materialization.
            input.bytes = bytes;
        }
        if self
            .files
            .get(path)
            .is_some_and(|previous| previous != &input)
        {
            return Err(format!("bound input changed during freeze: {path}"));
        }
        let bytes = input.bytes.clone();
        self.files.insert(path.into(), input);
        Ok(bytes)
    }

    pub(crate) fn required(&mut self, path: &str) -> Result<Vec<u8>, String> {
        self.optional(path)?
            .ok_or_else(|| format!("missing bound input: {path}"))
    }

    pub(crate) fn list(&mut self, path: &str) -> Result<Vec<String>, String> {
        let names = list(&self.root, path, self.borrowed.as_ref())?;
        self.record_directory(path, names)
    }

    fn record_directory(&mut self, path: &str, names: Vec<String>) -> Result<Vec<String>, String> {
        if self
            .directories
            .get(path)
            .is_some_and(|previous| previous != &names)
        {
            return Err(format!(
                "acquisition directory changed during freeze: {path}"
            ));
        }
        self.directories.insert(path.into(), names.clone());
        Ok(names)
    }

    /// Observe every file and directory in a relevant build scope, including untracked additions.
    pub(crate) fn files_under(&mut self, path: &str) -> Result<Vec<String>, String> {
        safe_path(&self.root, path)?;
        let root = self.root.clone();
        let borrowed = self.borrowed.clone();
        let mut session = batch::Session::new(&root, borrowed.as_ref());
        let start = session.listing();
        let mut observed = Vec::new();
        let result = self.files_under_listing(path, &start, &mut observed, &mut session)?;
        listing::verify(
            &self.root,
            self.borrowed.as_ref(),
            &self.files,
            &self.directories,
            observed,
            &session,
            &start,
        )?;
        Ok(result)
    }

    fn files_under_listing(
        &mut self,
        path: &str,
        listing: &listing::Listing,
        observed: &mut Vec<(String, Result<Vec<String>, String>)>,
        session: &mut batch::Session<'_>,
    ) -> Result<Vec<String>, String> {
        let names = list_cached(&self.root, path, self.borrowed.as_ref(), Some(listing))?;
        observed.push((path.to_owned(), Ok(names.clone())));
        let children: Vec<_> = self
            .record_directory(path, names)?
            .into_iter()
            .map(|name| {
                let child = format!("{path}/{name}");
                let directory = safe_path(&self.root, &child)
                    .map(|full| fs::symlink_metadata(full).is_ok_and(|m| m.is_dir()));
                (child, directory)
            })
            .collect();
        let binding = if self.proposed.is_none()
            && children
                .iter()
                .any(|(_, directory)| matches!(directory, Ok(false)))
        {
            self.recheck_borrowed()
        } else {
            Ok(())
        };
        // Refuse reads through a stale binding, consuming its error at the original optional
        // boundary so an earlier directory or path-validation failure retains precedence.
        let mut result = Vec::new();
        for (child, directory) in children {
            if directory? {
                result.extend(self.files_under_listing(&child, listing, observed, session)?);
            } else {
                let bytes = if self.proposed.is_some() && self.files.contains_key(&child) {
                    self.files[&child].bytes.clone()
                } else {
                    binding.as_ref().map_err(Clone::clone)?;
                    if self.proposed.is_none() {
                        self.recheck_borrowed()?;
                    }
                    self.record(&child, session.read(&child)?)?
                };
                bytes.ok_or_else(|| format!("missing bound input: {child}"))?;
                result.push(child);
            }
        }
        result.sort();
        Ok(result)
    }

    pub(crate) fn reference(&self, path: &str) -> Result<FileRef, String> {
        let bytes = self
            .files
            .get(path)
            .and_then(|i| i.bytes.as_ref())
            .ok_or_else(|| format!("unread input: {path}"))?;
        Ok(FileRef {
            path: path.into(),
            sha256: hex::encode(Sha256::digest(bytes)),
        })
    }

    pub(crate) fn executable(&self, path: &str) -> Result<bool, String> {
        let input = self.files.get(path).ok_or("unread input mode")?;
        if input.bytes.is_none() {
            return Err("absent input has no mode".into());
        }
        Ok(input.index.starts_with(b"100755 "))
    }

    pub(crate) fn verify(&self) -> Result<(), String> {
        self.recheck_borrowed()?;
        let mut session = batch::Session::new(&self.root, self.borrowed.as_ref());
        let start = session.listing();
        for (path, expected) in &self.files {
            if session.read(path)? != *expected {
                return Err(format!("bound input changed during freeze: {path}"));
            }
            #[cfg(test)]
            tests::file_observed(path);
        }
        let observed = self
            .directories
            .keys()
            .map(|path| {
                (
                    path.clone(),
                    list_cached(&self.root, path, self.borrowed.as_ref(), Some(&start)),
                )
            })
            .collect();
        listing::verify(
            &self.root,
            self.borrowed.as_ref(),
            &self.files,
            &self.directories,
            observed,
            &session,
            &start,
        )?;
        self.recheck_borrowed()?;
        Ok(())
    }
}

fn safe_path(root: &Path, path: &str) -> Result<PathBuf, String> {
    if path.contains('\\')
        || path
            .split('/')
            .any(|s| matches!(s, "" | "." | ".." | ".git"))
    {
        return Err(format!("invalid repository-relative input: {path}"));
    }
    let mut full = root.to_owned();
    for component in path.split('/') {
        full.push(component);
        match fs::symlink_metadata(&full) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(format!("symlink input: {path}"))
            }
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                return Err(format!("{path}: {e}"))
            }
            _ => {}
        }
    }
    Ok(full)
}

fn read_cached(
    root: &Path,
    path: &str,
    borrowed: Option<&(PathBuf, String)>,
    immutable: Option<&immutable::Immutable>,
) -> Result<Input, String> {
    let Some(cache) = immutable.filter(|_| borrowed.is_some()) else {
        return read(root, path, borrowed);
    };
    let full = safe_path(root, path)?;
    let Some(index) = cache.rows(path, false, false) else {
        return read(root, path, borrowed);
    };
    // prepare retains the single reader's index, working-file and executable checks, in order.
    let (input, object) = batch::prepare(&full, path, index, true)?;
    if let Some(object) = object {
        if input.bytes.as_ref() != Some(&cache.blob(&object)?) {
            return Err(format!("working bytes disagree with index: {path}"));
        }
    }
    Ok(input)
}

fn read(root: &Path, path: &str, borrowed: Option<&(PathBuf, String)>) -> Result<Input, String> {
    let full = safe_path(root, path)?;
    let object_root = borrowed.map_or(root, |(caller, _)| caller.as_path());
    let index = if let Some((caller, commit)) = borrowed {
        git(
            caller,
            &[
                "--literal-pathspecs",
                "ls-tree",
                "-z",
                "--full-tree",
                commit,
                "--",
                path,
            ],
        )?
    } else {
        git(
            root,
            &[
                "--literal-pathspecs",
                "ls-files",
                "--stage",
                "-z",
                "--",
                path,
            ],
        )?
    };
    if index.is_empty() {
        if fs::symlink_metadata(&full).is_ok() {
            return Err(format!("untracked bound input: {path}"));
        }
        return Ok(Input { index, bytes: None });
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
        || if borrowed.is_some() {
            meta[1] != "blob"
        } else {
            meta[2] != "0"
        }
        || indexed_path != path
    {
        return Err(format!("input must be a regular stage-0 file: {path}"));
    }
    if !fs::symlink_metadata(&full)
        .map_err(|e| format!("{path}: {e}"))?
        .is_file()
    {
        return Err(format!("non-file input: {path}"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let working_executable = fs::metadata(&full)
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
    let object = if borrowed.is_some() { meta[2] } else { meta[1] };
    let staged = git(object_root, &["cat-file", "blob", object])?;
    if bytes != staged {
        return Err(format!("working bytes disagree with index: {path}"));
    }
    Ok(Input {
        index,
        bytes: Some(bytes),
    })
}

fn list(
    root: &Path,
    path: &str,
    borrowed: Option<&(PathBuf, String)>,
) -> Result<Vec<String>, String> {
    list_cached(root, path, borrowed, None)
}

fn list_cached(
    root: &Path,
    path: &str,
    borrowed: Option<&(PathBuf, String)>,
    listing: Option<&listing::Listing>,
) -> Result<Vec<String>, String> {
    list_observed(root, path, borrowed, listing).map(|(names, _)| names)
}

fn list_observed(
    root: &Path,
    path: &str,
    borrowed: Option<&(PathBuf, String)>,
    listing: Option<&listing::Listing>,
) -> Result<(Vec<String>, BTreeSet<String>), String> {
    let full = safe_path(root, path)?;
    let mut names = BTreeSet::new();
    match fs::read_dir(full) {
        Ok(entries) => {
            for entry in entries {
                names.insert(
                    entry
                        .map_err(|e| e.to_string())?
                        .file_name()
                        .into_string()
                        .map_err(|_| "non-UTF8 input path")?,
                );
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(format!("{path}: {e}")),
    }
    let working_names = names.clone();
    let prefix = format!("{path}/");
    let tracked = if let Some((caller, commit)) = borrowed {
        git(
            caller,
            &[
                "--literal-pathspecs",
                "ls-tree",
                "-r",
                "--name-only",
                "-z",
                commit,
                "--",
                &prefix,
            ],
        )?
    } else if let Some(tracked) = listing.and_then(|listing| listing.tracked(root, &prefix)) {
        tracked
    } else {
        git(
            root,
            &["--literal-pathspecs", "ls-files", "-z", "--", &prefix],
        )?
    };
    extend_tracked_names(&mut names, &tracked, &prefix)?;
    #[cfg(test)]
    tests::directory_observed(path);
    Ok((names.into_iter().collect(), working_names))
}

fn extend_tracked_names(
    names: &mut BTreeSet<String>,
    tracked: &[u8],
    prefix: &str,
) -> Result<(), String> {
    for file in tracked.split(|b| *b == 0).filter(|b| !b.is_empty()) {
        let file = std::str::from_utf8(file).map_err(|e| e.to_string())?;
        let tail = file.strip_prefix(prefix).ok_or("unexpected Git path")?;
        names.insert(tail.split('/').next().ok_or("empty Git path")?.to_string());
    }
    Ok(())
}

/// Git state belongs to the explicit root, never an inherited index/object/worktree redirect.
/// Reads must neither refresh the caller's index nor acquire missing promisor objects. Explicit
/// mutations/acquisition at other call sites still require their own authorized commands.
pub(crate) fn command(root: &Path) -> Command {
    #[cfg(test)]
    tests::GIT_COMMANDS.with(|count| count.set(count.get() + 1));
    let mut command = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    // Install these after sanitizing the environment, even if the parent enabled writes/fetches.
    command
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_NO_LAZY_FETCH", "1")
        // Git before 2.45 ignores GIT_NO_LAZY_FETCH but rejects the unknown option, so this fails
        // closed there instead of lazily fetching.
        .args([
            "--no-optional-locks",
            "--no-replace-objects",
            "--no-lazy-fetch",
        ])
        .arg("-C")
        .arg(root);
    command
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = command(root)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "git {}: {}",
            args[0],
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(output.stdout)
}

#[cfg(test)]
pub(crate) mod tests;
