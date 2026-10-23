//! A bounded, read-only view of regular stage-0 files. This is no serialization lock or receipt.
use crate::depth_record::FileRef;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(PartialEq, Eq)]
struct Input {
    index: Vec<u8>,
    bytes: Option<Vec<u8>>,
}

pub(crate) struct GitInputs {
    root: PathBuf,
    files: BTreeMap<String, Input>,
    directories: BTreeMap<String, Vec<String>>,
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
            files: BTreeMap::new(),
            directories: BTreeMap::new(),
        })
    }

    pub(crate) fn optional(&mut self, path: &str) -> Result<Option<Vec<u8>>, String> {
        let input = read(&self.root, path)?;
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
        let names = list(&self.root, path)?;
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
        let mut result = Vec::new();
        for name in self.list(path)? {
            let child = format!("{path}/{name}");
            let full = safe_path(&self.root, &child)?;
            if fs::symlink_metadata(full).is_ok_and(|m| m.is_dir()) {
                result.extend(self.files_under(&child)?);
            } else {
                self.required(&child)?;
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

    pub(crate) fn verify(&self) -> Result<(), String> {
        for (path, expected) in &self.files {
            if read(&self.root, path)? != *expected {
                return Err(format!("bound input changed during freeze: {path}"));
            }
        }
        for (path, expected) in &self.directories {
            if list(&self.root, path)? != *expected {
                return Err(format!(
                    "acquisition directory changed during freeze: {path}"
                ));
            }
        }
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

fn read(root: &Path, path: &str) -> Result<Input, String> {
    let full = safe_path(root, path)?;
    let index = git(
        root,
        &[
            "--literal-pathspecs",
            "ls-files",
            "--stage",
            "-z",
            "--",
            path,
        ],
    )?;
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
        || meta[2] != "0"
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
    let bytes = fs::read(full).map_err(|e| format!("{path}: {e}"))?;
    let staged = git(root, &["cat-file", "blob", meta[1]])?;
    if bytes != staged {
        return Err(format!("working bytes disagree with index: {path}"));
    }
    Ok(Input {
        index,
        bytes: Some(bytes),
    })
}

fn list(root: &Path, path: &str) -> Result<Vec<String>, String> {
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
    let prefix = format!("{path}/");
    let tracked = git(
        root,
        &["--literal-pathspecs", "ls-files", "-z", "--", &prefix],
    )?;
    for file in tracked.split(|b| *b == 0).filter(|b| !b.is_empty()) {
        let file = std::str::from_utf8(file).map_err(|e| e.to_string())?;
        let tail = file.strip_prefix(&prefix).ok_or("unexpected Git path")?;
        names.insert(tail.split('/').next().ok_or("empty Git path")?.to_string());
    }
    Ok(names.into_iter().collect())
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("git")
        .arg("--no-replace-objects")
        .arg("-C")
        .arg(root)
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
mod tests;
