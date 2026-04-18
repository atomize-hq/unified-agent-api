#![forbid(unsafe_code)]
//! Async helper around the OpenCode CLI (`opencode`) focused on the canonical
//! `opencode run --format json` surface only.

use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use futures_core::Stream;
use tokio::sync::Notify;

mod builder;
mod client;
mod error;
mod run_json;

pub use builder::OpencodeClientBuilder;
pub use client::OpencodeClient;
pub use error::OpencodeError;
pub use run_json::{
    parse_run_json_lines, OpencodeRunCompletion, OpencodeRunJsonErrorCode, OpencodeRunJsonEvent,
    OpencodeRunJsonLine, OpencodeRunJsonLineOutcome, OpencodeRunJsonParseError,
    OpencodeRunJsonParser, OpencodeRunRequest,
};

pub type DynOpencodeRunJsonEventStream =
    Pin<Box<dyn Stream<Item = Result<OpencodeRunJsonEvent, OpencodeRunJsonParseError>> + Send>>;

pub type DynOpencodeRunJsonCompletion =
    Pin<Box<dyn Future<Output = Result<OpencodeRunCompletion, OpencodeError>> + Send>>;

#[derive(Clone)]
pub struct OpencodeTerminationHandle {
    inner: Arc<OpencodeTerminationInner>,
}

#[derive(Debug)]
struct OpencodeTerminationInner {
    requested: AtomicBool,
    notify: Notify,
}

impl OpencodeTerminationHandle {
    fn new() -> Self {
        Self {
            inner: Arc::new(OpencodeTerminationInner {
                requested: AtomicBool::new(false),
                notify: Notify::new(),
            }),
        }
    }

    pub fn request_termination(&self) {
        if !self.inner.requested.swap(true, Ordering::SeqCst) {
            self.inner.notify.notify_waiters();
        }
    }

    fn is_requested(&self) -> bool {
        self.inner.requested.load(Ordering::SeqCst)
    }

    async fn requested(&self) {
        if self.is_requested() {
            return;
        }

        let notified = self.inner.notify.notified();
        if self.is_requested() {
            return;
        }

        notified.await;
    }
}

impl std::fmt::Debug for OpencodeTerminationHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpencodeTerminationHandle")
            .field("requested", &self.is_requested())
            .finish()
    }
}

pub struct OpencodeRunJsonHandle {
    pub events: DynOpencodeRunJsonEventStream,
    pub completion: DynOpencodeRunJsonCompletion,
}

impl std::fmt::Debug for OpencodeRunJsonHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpencodeRunJsonHandle")
            .field("events", &"<stream>")
            .field("completion", &"<future>")
            .finish()
    }
}

pub struct OpencodeRunJsonControlHandle {
    pub events: DynOpencodeRunJsonEventStream,
    pub completion: DynOpencodeRunJsonCompletion,
    pub termination: OpencodeTerminationHandle,
}

impl std::fmt::Debug for OpencodeRunJsonControlHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpencodeRunJsonControlHandle")
            .field("events", &"<stream>")
            .field("completion", &"<future>")
            .field("termination", &self.termination)
            .finish()
    }
}
