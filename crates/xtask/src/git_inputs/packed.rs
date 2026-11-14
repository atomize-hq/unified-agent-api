//! Cheap invalidation of retained object storage, including linked worktrees and alternates.
use super::git;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

type Stamp = (u64, SystemTime, SystemTime);

#[derive(Default, PartialEq, Eq)]
struct Snapshot {
    // Git retains canonical alternate paths even when they contain only loose objects.
    // Keep each directory's (device, inode) too: replacement can preserve its canonical path.
    directories: BTreeMap<PathBuf, (u64, u64)>,
    packs: BTreeMap<PathBuf, Stamp>,
    alternates: BTreeMap<PathBuf, String>,
}

pub(super) struct Packs {
    objects: PathBuf,
    snapshot: Snapshot,
}

impl Packs {
    pub(super) fn new(root: &Path) -> Result<Self, String> {
        let raw = git(
            root,
            &[
                "rev-parse",
                "--path-format=absolute",
                "--git-path",
                "objects",
            ],
        )?;
        let path = String::from_utf8(raw).map_err(|e| e.to_string())?;
        let objects = PathBuf::from(path.strip_suffix('\n').unwrap_or(&path));
        let snapshot = snapshot(&objects)?;
        Ok(Self { objects, snapshot })
    }

    pub(super) fn changed(&mut self) -> Result<bool, String> {
        let current = snapshot(&self.objects)?;
        let changed = current != self.snapshot;
        self.snapshot = current;
        Ok(changed)
    }
}

fn snapshot(objects: &Path) -> Result<Snapshot, String> {
    let mut pending = vec![objects.to_owned()];
    let mut result = Snapshot::default();
    while let Some(objects) = pending.pop() {
        let objects = fs::canonicalize(objects).map_err(|e| e.to_string())?;
        if result.directories.contains_key(&objects) {
            continue;
        }
        let identity = directory_identity(&fs::metadata(&objects).map_err(|e| e.to_string())?)?;
        result.directories.insert(objects.clone(), identity);
        match fs::read_dir(objects.join("pack")) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry.map_err(|e| e.to_string())?;
                    let metadata = entry.metadata().map_err(|e| e.to_string())?;
                    result.packs.insert(
                        entry.path(),
                        (
                            metadata.len(),
                            metadata.modified().map_err(|e| e.to_string())?,
                            metadata.created().unwrap_or(SystemTime::UNIX_EPOCH),
                        ),
                    );
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
        let alternates = objects.join("info/alternates");
        match fs::read_to_string(&alternates) {
            Ok(paths) => {
                for path in paths.lines().filter(|path| !path.is_empty()) {
                    // Git also supports quoted alternate paths. Use fresh commands for those
                    // rather than approximating its quoting parser or trusting stale mappings.
                    if path.starts_with('"') {
                        return Err("quoted Git alternate requires fresh object reads".into());
                    }
                    pending.push(objects.join(path));
                }
                // Timestamps and lengths can survive edits. Compare the actual configuration.
                result.alternates.insert(alternates, paths);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(result)
}

#[cfg(unix)]
fn directory_identity(metadata: &fs::Metadata) -> Result<(u64, u64), String> {
    use std::os::unix::fs::MetadataExt;
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(not(unix))]
fn directory_identity(_metadata: &fs::Metadata) -> Result<(u64, u64), String> {
    // Without a stable directory identity, let BlobReader use its fresh-command fallback.
    Err("Git object directory identity requires fresh object reads".into())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn replacing_primary_or_transitive_object_directories_changes_snapshot() {
        let root = tempfile::tempdir().unwrap();
        let objects = root.path().join("objects");
        let alternate = root.path().join("alternate");
        let nested = root.path().join("nested");
        for (directory, next) in [
            (&objects, "../alternate\n"),
            (&alternate, "../nested\n"),
            (&nested, "../objects\n"), // A cycle must still terminate.
        ] {
            fs::create_dir_all(directory.join("info")).unwrap();
            fs::write(directory.join("info/alternates"), next).unwrap();
        }
        for directory in [&objects, &alternate, &nested] {
            let before = snapshot(&objects).unwrap();
            let alternates = directory.join("info/alternates");
            let paths = fs::read(&alternates).unwrap();
            let modified = fs::metadata(&alternates).unwrap().modified().unwrap();
            // Keep the old inode alive; replace the directory at the same canonical path.
            fs::rename(directory, directory.with_extension("old")).unwrap();
            fs::create_dir_all(directory.join("info")).unwrap();
            fs::write(&alternates, paths).unwrap();
            fs::File::open(&alternates)
                .unwrap()
                .set_modified(modified)
                .unwrap();
            assert!(before != snapshot(&objects).unwrap(), "{directory:?}");
        }
    }

    #[test]
    fn alternates_contents_invalidate_even_with_same_directories_and_file_stamp() {
        let root = tempfile::tempdir().unwrap();
        let objects = root.path().join("objects");
        fs::create_dir_all(objects.join("info")).unwrap();
        for name in ["store-a", "store-b"] {
            fs::create_dir(root.path().join(name)).unwrap();
        }
        let alternates = objects.join("info/alternates");
        fs::write(&alternates, "../store-a\n../store-b\n").unwrap();
        let modified = fs::metadata(&alternates).unwrap().modified().unwrap();
        let before = snapshot(&objects).unwrap();
        fs::write(&alternates, "../store-b\n../store-a\n").unwrap();
        fs::File::open(&alternates)
            .unwrap()
            .set_modified(modified)
            .unwrap();
        assert!(before != snapshot(&objects).unwrap());
    }
}
