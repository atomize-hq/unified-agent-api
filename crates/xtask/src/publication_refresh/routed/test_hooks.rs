//! Invoke the shared mechanical writer's deterministic partial-failure seam.
use crate::workspace_mutation::{PlannedMutation, WorkspaceMutationError};
use std::{cell::Cell, path::Path};

pub(crate) type DeliveryHook =
    fn(&Path, &[PlannedMutation], usize) -> Result<(), WorkspaceMutationError>;
thread_local! {
    static DELIVERY_HOOK: Cell<Option<DeliveryHook>> = const { Cell::new(None) };
    static DELIVERY_FAILURE: Cell<Option<usize>> = const { Cell::new(None) };
}
pub(super) fn delivery_failure() -> Option<usize> {
    DELIVERY_FAILURE.with(Cell::get)
}
pub(crate) fn with_delivery_failure<T>(index: usize, action: impl FnOnce() -> T) -> T {
    struct Restore(Option<usize>);
    impl Drop for Restore {
        fn drop(&mut self) {
            DELIVERY_FAILURE.with(|slot| slot.set(self.0));
        }
    }
    let _restore = Restore(DELIVERY_FAILURE.with(|slot| slot.replace(Some(index))));
    action()
}

pub(super) fn delivery_hook() -> Option<DeliveryHook> {
    DELIVERY_HOOK.with(Cell::get)
}
pub(crate) fn with_delivery_hook<T>(hook: DeliveryHook, action: impl FnOnce() -> T) -> T {
    struct Restore(Option<DeliveryHook>);
    impl Drop for Restore {
        fn drop(&mut self) {
            DELIVERY_HOOK.with(|slot| slot.set(self.0));
        }
    }
    let _restore = Restore(DELIVERY_HOOK.with(|slot| slot.replace(Some(hook))));
    action()
}
