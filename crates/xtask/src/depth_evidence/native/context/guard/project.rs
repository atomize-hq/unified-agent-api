//! Bounded metadata-only comparison of the disposable project, never native home or auth.
use super::{role, Refusal};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::SystemTime,
};
#[derive(Debug, PartialEq, Eq)]
struct Entry {
    directory: bool,
    len: u64,
    modified: SystemTime,
    identity: (u64, u64, u32, i64, i64),
}
fn snapshot(root: &Path) -> Result<BTreeMap<PathBuf, Entry>, Refusal> {
    let mut entries = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        let meta = std::fs::symlink_metadata(&path).map_err(|_| Refusal::InputDrift)?;
        if meta.is_symlink() || !(meta.is_file() || meta.is_dir()) {
            return Err(Refusal::PathRole);
        }
        #[cfg(unix)]
        let identity = {
            use std::os::unix::fs::MetadataExt;
            (
                meta.dev(),
                meta.ino(),
                meta.mode(),
                meta.ctime(),
                meta.ctime_nsec(),
            )
        };
        #[cfg(not(unix))]
        let identity = (0, 0, 0, 0, 0);
        entries.insert(
            path.strip_prefix(root)
                .map_err(|_| Refusal::PathRole)?
                .to_path_buf(),
            Entry {
                directory: meta.is_dir(),
                len: meta.len(),
                modified: meta.modified().map_err(|_| Refusal::InputDrift)?,
                identity,
            },
        );
        if entries.len() > 4096 {
            return Err(Refusal::InputDrift);
        }
        if meta.is_dir() {
            for child in std::fs::read_dir(path).map_err(|_| Refusal::InputDrift)? {
                if pending.len() + entries.len() >= 4096 {
                    return Err(Refusal::InputDrift);
                }
                pending.push(child.map_err(|_| Refusal::InputDrift)?.path());
            }
        }
    }
    Ok(entries)
}
pub struct ProjectGuard {
    root: PathBuf,
    entries: BTreeMap<PathBuf, Entry>,
}
impl ProjectGuard {
    pub fn inspect(root: &Path) -> Result<Self, Refusal> {
        let root = role(root)?;
        let entries = snapshot(&root)?;
        Ok(Self { root, entries })
    }
    pub fn unchanged(&self) -> bool {
        snapshot(&self.root).is_ok_and(|s| s == self.entries)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn project_metadata_detects_mutation_without_reading_contents_or_following_links() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path().join("work");
        std::fs::create_dir(&work).unwrap();
        std::fs::write(work.join("fixture"), "nonsecret source-owned fixture").unwrap();
        let guard = ProjectGuard::inspect(&work).unwrap();
        assert!(guard.unchanged());
        std::fs::write(tmp.path().join("auth.json"), "sk-do-not-inspect").unwrap();
        assert!(guard.unchanged());
        std::fs::write(work.join("fixture"), "changed fixture").unwrap();
        assert!(!guard.unchanged());
        let guard = ProjectGuard::inspect(&work).unwrap();
        std::fs::rename(work.join("fixture"), work.join("renamed")).unwrap();
        assert!(!guard.unchanged());
        #[cfg(unix)]
        {
            let guard = ProjectGuard::inspect(&work).unwrap();
            std::os::unix::fs::symlink(tmp.path(), work.join("escape")).unwrap();
            assert!(!guard.unchanged());
            assert!(ProjectGuard::inspect(&work).is_err());
        }
    }
}
