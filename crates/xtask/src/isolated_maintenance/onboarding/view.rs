//! Filesystem engines consume only a materialization of the pinned candidate's regular blobs.
use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

pub(super) struct View {
    directory: tempfile::TempDir,
    source: Source,
    paths: Vec<String>,
}
impl View {
    pub(super) fn capture(root: &Path, head: &str) -> Result<Self, String> {
        let view = Self::pinned(root, head)?;
        view.verify(root)?;
        Ok(view)
    }
    /// Materialize every regular blob of a pinned commit without comparing a different live tip.
    pub(super) fn pinned(root: &Path, head: &str) -> Result<Self, String> {
        let directory = tempfile::tempdir().map_err(|e| e.to_string())?;
        let destination = directory
            .path()
            .to_str()
            .ok_or("non-UTF8 checkpoint view")?;
        // No checkout: never run filters/hooks or derive candidate bytes from the worktree.
        super::super::git(
            root,
            &[
                "clone",
                "--shared",
                "--no-checkout",
                "--quiet",
                "--",
                ".",
                destination,
            ],
        )?;
        super::super::git(directory.path(), &["read-tree", head])?;
        super::super::git(directory.path(), &["update-ref", "HEAD", head])?;
        let rows = super::super::git(root, &["ls-tree", "-r", "-z", "--full-tree", head])?;
        let mut files = BTreeMap::new();
        let mut size = 0usize;
        for row in rows.split('\0').filter(|r| !r.is_empty()) {
            let (meta, path) = row.split_once('\t').ok_or("invalid checkpoint tree row")?;
            let fields: Vec<_> = meta.split(' ').collect();
            if fields.len() != 3 || fields[1] != "blob" || !matches!(fields[0], "100644" | "100755")
            {
                return Err(format!("unsupported checkpoint file mode: {path}"));
            }
            let output = crate::git_inputs::command(root)
                .args(["cat-file", "blob", fields[2]])
                .output()
                .map_err(|e| e.to_string())?;
            if !output.status.success() {
                return Err(format!("cannot read pinned checkpoint blob: {path}"));
            }
            size = size
                .checked_add(output.stdout.len())
                .ok_or("checkpoint view exceeds limit")?;
            if size > 536_870_912 || files.len() >= 50_000 {
                return Err("checkpoint view exceeds limit".into());
            }
            files.insert(path.to_owned(), (output.stdout, fields[0] == "100755"));
        }
        let paths = files.keys().cloned().collect();
        let source = Source::captured(directory.path(), files)?;
        let view = Self {
            directory,
            source,
            paths,
        };
        for path in &view.paths {
            // Source validates repository-relative names before any filesystem write.
            let bytes = view
                .source
                .optional_bytes(path)?
                .ok_or("missing pinned blob")?;
            let full = view.root().join(path);
            fs::create_dir_all(full.parent().ok_or("invalid checkpoint parent")?)
                .map_err(|e| e.to_string())?;
            fs::write(&full, bytes).map_err(|e| e.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = if view.source.executable(path)? {
                    0o755
                } else {
                    0o644
                };
                fs::set_permissions(&full, fs::Permissions::from_mode(mode))
                    .map_err(|e| e.to_string())?;
            }
        }
        view.verify(view.root())?;
        Ok(view)
    }
    pub(super) fn root(&self) -> &Path {
        self.directory.path()
    }
    /// Check explicit bytes and Git's executable mode, independently of status/index flags.
    pub(super) fn verify(&self, root: &Path) -> Result<(), String> {
        let working = Source::workspace(root)?;
        for path in &self.paths {
            if working.optional_bytes(path)? != self.source.optional_bytes(path)? {
                return Err(format!(
                    "checkpoint working bytes differ from pinned commit: {path}"
                ));
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = fs::symlink_metadata(root.join(path))
                    .map_err(|e| e.to_string())?
                    .permissions()
                    .mode();
                if (mode & 0o100 != 0) != self.source.executable(path)? {
                    return Err(format!(
                        "checkpoint working mode differs from pinned commit: {path}"
                    ));
                }
            }
        }
        if root == self.root() {
            let mut expected: BTreeSet<_> = self.paths.iter().cloned().collect();
            for path in &self.paths {
                let mut parent = Path::new(path).parent();
                while let Some(path) = parent.filter(|p| !p.as_os_str().is_empty()) {
                    expected.insert(path.to_str().ok_or("non-UTF8 view path")?.to_owned());
                    parent = path.parent();
                }
            }
            let mut actual = BTreeSet::new();
            for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let name = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| "non-UTF8 view path")?;
                if name == ".git" {
                    continue;
                }
                actual.insert(name.clone());
                if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
                    actual.extend(working.paths(&name)?);
                }
            }
            if actual != expected {
                return Err("checkpoint view inventory differs from pinned commit".into());
            }
        }
        Ok(())
    }
}
