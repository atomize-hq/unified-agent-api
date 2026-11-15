//! Invoke the shared mechanical writer's deterministic partial-failure seam.
use std::cell::Cell;
thread_local! {
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
