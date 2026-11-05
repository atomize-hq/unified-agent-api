//! The caller index under Git's own lockfile protocol. The relay creates `index.lock` exclusively
//! before it reads anything it compares, works on a private copy of the index through
//! `GIT_INDEX_FILE`, and installs that copy with an atomic rename while it still holds the lock.
//! No other Git process can change the index between the comparison and the installation; one
//! that tries fails on the lock instead of being overwritten.
use super::{candidate_git, tree::io, Error};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) struct IndexLock {
    root: PathBuf,
    index: PathBuf,
    lock: PathBuf,
    work: PathBuf,
}
impl IndexLock {
    pub fn acquire(root: &Path) -> Result<Self, Error> {
        let relative = String::from_utf8(candidate_git::run(
            root,
            &["rev-parse", "--git-path", "index"],
        )?)
        .map_err(|e| Error::Validation(e.to_string()))?;
        let index = fs::canonicalize(root).map_err(io)?.join(relative.trim());
        let suffixed = |suffix: &str| {
            let mut name = index.as_os_str().to_owned();
            name.push(suffix);
            PathBuf::from(name)
        };
        let (lock, work) = (suffixed(".lock"), suffixed(".uaa-relay"));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock)
        {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(Error::Validation(
                    "the caller index is locked by another Git process (index.lock exists); \
                     the index was not changed"
                        .into(),
                ))
            }
            Err(e) => return Err(io(e)),
        }
        // From here the lock is ours, and `Drop` releases it on every path.
        let locked = Self {
            root: root.to_owned(),
            index,
            lock,
            work,
        };
        let _ = fs::remove_file(&locked.work);
        match fs::copy(&locked.index, &locked.work) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(io(e)),
        }
        Ok(locked)
    }
    /// Run Git against the locked copy of the index.
    pub fn run(&self, args: &[&str]) -> Result<Vec<u8>, Error> {
        let output = candidate_git::command(&self.root)
            .env("GIT_INDEX_FILE", &self.work)
            .args(args)
            .output()
            .map_err(io)?;
        if !output.status.success() {
            return Err(Error::Validation(format!(
                "relay git {} failed",
                args.first().unwrap_or(&"command")
            )));
        }
        Ok(output.stdout)
    }
    /// Install the locked copy over the index, then release the lock.
    pub fn commit(self) -> Result<(), Error> {
        fs::rename(&self.work, &self.index).map_err(io)
    }
}
impl Drop for IndexLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.work);
        let _ = fs::remove_file(&self.lock);
    }
}

/// A test may act between a relay's index comparison and its installation, on its own thread.
#[cfg(test)]
pub(super) mod between {
    use std::{cell::RefCell, path::Path};
    type Hook = Box<dyn Fn(&Path)>;
    thread_local!(static HOOK: RefCell<Option<Hook>> = const { RefCell::new(None) });
    pub(in super::super) fn set(hook: impl Fn(&Path) + 'static) {
        HOOK.with(|h| *h.borrow_mut() = Some(Box::new(hook)));
    }
    pub(in super::super) fn clear() {
        HOOK.with(|h| *h.borrow_mut() = None);
    }
    pub(in super::super) fn run(root: &Path) {
        HOOK.with(|h| {
            if let Some(hook) = h.borrow().as_ref() {
                hook(root);
            }
        });
    }
}
