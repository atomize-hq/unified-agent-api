//! New-attempt storage. Rechecks detect filesystem drift; this is not a filesystem lock.
use crate::{depth_ci::reference, depth_record::FileRef};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
pub(super) struct Retained {
    root: PathBuf,
    directory: String,
    identity: fs::Metadata,
    files: BTreeMap<String, Vec<u8>>,
    total: usize,
    notes: BTreeMap<String, Vec<u8>>,
}
fn regular_directory(root: &Path, relative: &str) -> Result<PathBuf, String> {
    crate::depth_evidence::validate::path(relative)?;
    let mut path = root.to_path_buf();
    for part in relative.split('/') {
        path.push(part);
        let m = fs::symlink_metadata(&path).map_err(|_| "native report directory unavailable")?;
        if !m.is_dir() || m.is_symlink() {
            return Err("native report ancestor is not a regular directory".into());
        }
    }
    Ok(path)
}
impl Retained {
    pub fn reserve(root: &Path, directory: &str) -> Result<Self, String> {
        crate::depth_evidence::validate::path(directory)?;
        let root = root
            .canonicalize()
            .map_err(|_| "native workspace unavailable")?;
        let path = Path::new(directory);
        let parent = path
            .parent()
            .and_then(Path::to_str)
            .ok_or("missing report parent")?;
        if !parent.is_empty() {
            regular_directory(&root, parent)?;
        }
        fs::create_dir(root.join(directory))
            .map_err(|_| "native attempt already exists or cannot be reserved")?;
        let identity = fs::symlink_metadata(regular_directory(&root, directory)?)
            .map_err(|_| "native attempt unavailable")?;
        Ok(Self {
            root,
            directory: directory.into(),
            identity,
            files: BTreeMap::new(),
            total: 0,
            notes: BTreeMap::new(),
        })
    }
    fn check(&self) -> Result<(), String> {
        let m = fs::metadata(regular_directory(&self.root, &self.directory)?)
            .map_err(|_| "native attempt unavailable")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if (m.dev(), m.ino()) != (self.identity.dev(), self.identity.ino()) {
                return Err("native attempt directory changed".into());
            }
        }
        #[cfg(not(unix))]
        if m.created().ok() != self.identity.created().ok() {
            return Err("native attempt directory changed".into());
        }
        Ok(())
    }
    pub fn path(&self, name: &str) -> String {
        format!("{}/{name}", self.directory)
    }
    pub fn put(&mut self, name: &str, bytes: &[u8]) -> Result<FileRef, String> {
        crate::depth_evidence::validate::path(name)?;
        if name.contains('/') || matches!(name, "bundle.json" | "registration.json") {
            return Err("native payload must be a distinct leaf".into());
        }
        let path = self.path(name);
        self.add(&path, bytes)?;
        self.write(&path, bytes)?;
        Ok(reference(path, bytes))
    }
    pub fn registration(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() > 65_536 || !self.notes.is_empty() {
            return Err("registration already retained or oversized".into());
        }
        let path = self.path("registration.json");
        self.write(&path, bytes)?;
        self.notes.insert(path, bytes.into());
        Ok(())
    }
    pub fn include(&mut self, file: &FileRef, bytes: &[u8]) -> Result<(), String> {
        if reference(&file.path, bytes) != *file {
            return Err("retained payload digest differs".into());
        }
        self.add(&file.path, bytes)
    }
    fn add(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() > 1_048_576
            || self.total + bytes.len() > 16_777_216
            || self.files.len() >= 4096
            || self.files.contains_key(path)
        {
            return Err("duplicate or oversized native payload".into());
        }
        self.total += bytes.len();
        self.files.insert(path.into(), bytes.into());
        Ok(())
    }
    fn write(&self, path: &str, bytes: &[u8]) -> Result<(), String> {
        self.check()?;
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.root.join(path))
            .map_err(|_| "cannot create native payload; prior bytes preserved")?;
        f.write_all(bytes)
            .and_then(|()| f.sync_all())
            .map_err(|_| "cannot retain native payload")?;
        self.check()
    }
    pub fn verify(&self) -> Result<(), String> {
        self.check()?;
        for (path, bytes) in self.files.iter().chain(self.notes.iter()) {
            let parent = Path::new(path)
                .parent()
                .and_then(Path::to_str)
                .ok_or("missing payload parent")?;
            regular_directory(&self.root, parent)?;
            let full = self.root.join(path);
            let m =
                fs::symlink_metadata(&full).map_err(|_| "retained native payload unavailable")?;
            if !m.is_file()
                || m.is_symlink()
                || m.len() > 1_048_576
                || fs::read(&full).map_err(|_| "retained payload unreadable")? != *bytes
            {
                return Err("retained native payload changed".into());
            }
        }
        self.check()
    }
    pub fn index(
        &self,
        plan: FileRef,
        observed_context: FileRef,
        claims: Vec<FileRef>,
    ) -> Result<FileRef, String> {
        self.verify()?;
        let files = self
            .files
            .iter()
            .map(|(path, b)| reference(path, b))
            .collect();
        let bundle = crate::depth_evidence::native::Bundle {
            schema_version: 1,
            plan,
            observed_context,
            claims,
            files,
        };
        let bytes = super::canonical(&bundle)?;
        crate::depth_evidence::native::Bundle::parse(
            std::str::from_utf8(&bytes).map_err(|_| "invalid bundle encoding")?,
        )?;
        let path = self.path("bundle.json");
        self.write(&path, &bytes)?;
        self.verify()?;
        Ok(reference(path, &bytes))
    }
}
