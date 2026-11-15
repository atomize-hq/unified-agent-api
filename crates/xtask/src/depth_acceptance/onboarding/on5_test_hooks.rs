//! Invocation-local test faults alter the actual proposed batch before its owning checks.
use std::{
    cell::Cell,
    path::{Path, PathBuf},
};
type Hook = fn(&Path, &mut Vec<(PathBuf, Vec<u8>)>) -> Result<(), String>;
thread_local! {
    static HOOK: Cell<Option<Hook>> = const { Cell::new(None) };
    static RECORDED_AT: Cell<Option<&'static str>> = const { Cell::new(None) };
}
pub(crate) fn apply(root: &Path, files: &mut Vec<(PathBuf, Vec<u8>)>) -> Result<(), String> {
    HOOK.with(|slot| slot.get())
        .map_or(Ok(()), |hook| hook(root, files))
}
pub(crate) fn with_proposal<T>(hook: Hook, action: impl FnOnce() -> T) -> T {
    struct Restore(Option<Hook>);
    impl Drop for Restore {
        fn drop(&mut self) {
            HOOK.with(|slot| slot.set(self.0));
        }
    }
    let _restore = Restore(HOOK.with(|slot| slot.replace(Some(hook))));
    action()
}

/// Override only the final closeout invocation's clock on this test thread.
pub(crate) fn recorded_at() -> Option<String> {
    RECORDED_AT.with(|slot| slot.get().map(str::to_owned))
}
pub(crate) fn with_recorded_at<T>(time: &'static str, action: impl FnOnce() -> T) -> T {
    struct Restore(Option<&'static str>);
    impl Drop for Restore {
        fn drop(&mut self) {
            RECORDED_AT.with(|slot| slot.set(self.0));
        }
    }
    let _restore = Restore(RECORDED_AT.with(|slot| slot.replace(Some(time))));
    action()
}
