//! Scoped, thread-local fault injection after proposal evaluation and before A checking.
use std::{cell::RefCell, path::Path};

type Changes = Vec<(String, Option<Vec<u8>>)>;
thread_local! {
    static SHARED_CHANGES: RefCell<Changes> = const { RefCell::new(Vec::new()) };
}

pub(crate) fn with_shared_changes<T>(changes: Changes, check: impl FnOnce() -> T) -> T {
    let previous = SHARED_CHANGES.with(|slot| slot.replace(changes));
    let result = check();
    SHARED_CHANGES.with(|slot| slot.replace(previous));
    result
}

pub(super) fn enter(root: &Path) -> Restore {
    let changes = SHARED_CHANGES.with(|slot| slot.take());
    let mut restore = Restore(Vec::new());
    for (path, bytes) in changes {
        let full = root.join(path);
        restore.0.push((full.clone(), std::fs::read(&full).ok()));
        match bytes {
            Some(bytes) => {
                std::fs::create_dir_all(full.parent().unwrap()).unwrap();
                std::fs::write(full, bytes).unwrap();
            }
            None => std::fs::remove_file(full).unwrap(),
        }
    }
    restore
}

pub(super) struct Restore(Vec<(std::path::PathBuf, Option<Vec<u8>>)>);
impl Drop for Restore {
    fn drop(&mut self) {
        for (path, bytes) in &self.0 {
            match bytes {
                Some(bytes) => std::fs::write(path, bytes).unwrap(),
                None => std::fs::remove_file(path).unwrap(),
            }
        }
    }
}
