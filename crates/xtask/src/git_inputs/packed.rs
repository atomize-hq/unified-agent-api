//! Cheap invalidation of retained pack mappings, including linked worktrees and alternates.
use super::git;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

type Stamp = (u64, SystemTime, SystemTime);
type Snapshot = BTreeMap<PathBuf, Stamp>;

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
    let mut visited = BTreeSet::new();
    let mut result = BTreeMap::new();
    while let Some(objects) = pending.pop() {
        let objects = fs::canonicalize(objects).map_err(|e| e.to_string())?;
        if !visited.insert(objects.clone()) {
            continue;
        }
        match fs::read_dir(objects.join("pack")) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry.map_err(|e| e.to_string())?;
                    let metadata = entry.metadata().map_err(|e| e.to_string())?;
                    result.insert(
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
                let metadata = fs::metadata(&alternates).map_err(|e| e.to_string())?;
                result.insert(
                    alternates,
                    (
                        metadata.len(),
                        metadata.modified().map_err(|e| e.to_string())?,
                        SystemTime::UNIX_EPOCH,
                    ),
                );
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(result)
}
