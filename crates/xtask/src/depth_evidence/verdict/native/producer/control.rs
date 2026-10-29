//! Catch ordinary foreground termination for this command, then finish owned cleanup/retention.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
pub(super) struct Interrupts {
    cancelled: Arc<AtomicBool>,
    task: tokio::task::JoinHandle<()>,
    _runtime: tokio::runtime::Runtime,
}
impl Interrupts {
    #[cfg(unix)]
    pub fn new() -> Result<Self, String> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .map_err(|_| "cannot initialize foreground signal handling")?;
        let (mut interrupt, mut terminate) = {
            let _guard = runtime.enter();
            use tokio::signal::unix::{signal, SignalKind};
            (
                signal(SignalKind::interrupt())
                    .map_err(|_| "cannot register foreground interrupt")?,
                signal(SignalKind::terminate())
                    .map_err(|_| "cannot register foreground termination")?,
            )
        };
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancelled);
        let task = runtime.spawn(async move {
            loop {
                tokio::select! { _=interrupt.recv()=>{}, _=terminate.recv()=>{} }
                flag.store(true, Ordering::Release);
            }
        });
        Ok(Self {
            cancelled,
            task,
            _runtime: runtime,
        })
    }
    #[cfg(not(unix))]
    pub fn new() -> Result<Self, String> {
        Err("foreground native producer requires Unix signal ownership".into())
    }
    pub fn flag(&self) -> &AtomicBool {
        &self.cancelled
    }
}
impl Drop for Interrupts {
    fn drop(&mut self) {
        self.task.abort();
    }
}
pub(super) fn check(flag: &AtomicBool) -> Result<(), String> {
    if flag.load(Ordering::Acquire) {
        Err("foreground native attempt interrupted; no further dispatch".into())
    } else {
        Ok(())
    }
}
pub(super) struct Host<'a, A> {
    pub inner: &'a A,
    pub cancel: &'a AtomicBool,
}
impl<A: super::host::Api> super::host::Api for Host<'_, A> {
    fn json(
        &self,
        repo: &super::form::Repository,
        path: &str,
    ) -> Result<serde_json::Value, String> {
        check(self.cancel)?;
        self.inner.json(repo, path)
    }
}
impl<A: super::StartApi> super::StartApi for Host<'_, A> {
    fn post_start(
        &self,
        repo: &super::form::Repository,
        pr: u64,
        body: &str,
    ) -> Result<u64, String> {
        check(self.cancel)?;
        self.inner.post_start(repo, pr, body)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Never;
    impl super::super::host::Api for Never {
        fn json(
            &self,
            _: &super::super::form::Repository,
            _: &str,
        ) -> Result<serde_json::Value, String> {
            panic!("host dispatched after interruption")
        }
    }
    impl super::super::StartApi for Never {
        fn post_start(
            &self,
            _: &super::super::form::Repository,
            _: u64,
            _: &str,
        ) -> Result<u64, String> {
            panic!("POST dispatched after interruption")
        }
    }
    #[test]
    fn cancelled_foreground_owner_dispatches_no_further_host_reads_or_registration() {
        let flag = AtomicBool::new(true);
        let api = Host {
            inner: &Never,
            cancel: &flag,
        };
        let repo = super::super::form::Repository {
            name: "fixture/isolated".into(),
            id: 1,
            owner_id: 2,
        };
        assert!(super::super::host::Api::json(&api, &repo, "")
            .unwrap_err()
            .contains("interrupted"));
        assert!(
            super::super::StartApi::post_start(&api, &repo, 1, "not posted")
                .unwrap_err()
                .contains("interrupted")
        );
    }
}
