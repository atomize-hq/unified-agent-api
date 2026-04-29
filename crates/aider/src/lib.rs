#![forbid(unsafe_code)]
//! Async helper around the aider CLI headless `--message-format stream-json` surface.

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
mod stream_json;
pub mod wrapper_coverage_manifest;

pub use builder::AiderCliClientBuilder;
pub use client::AiderCliClient;
pub use error::AiderCliError;
pub use stream_json::{
    parse_stream_json_lines, AiderStreamJsonError, AiderStreamJsonErrorCode, AiderStreamJsonEvent,
    AiderStreamJsonLine, AiderStreamJsonLineOutcome, AiderStreamJsonParser,
    AiderStreamJsonResultPayload, AiderStreamJsonRunRequest, AiderToolResultError,
};

pub type DynAiderStreamJsonEventStream =
    Pin<Box<dyn Stream<Item = Result<AiderStreamJsonEvent, AiderStreamJsonError>> + Send>>;

pub type DynAiderStreamJsonCompletion =
    Pin<Box<dyn Future<Output = Result<AiderStreamJsonCompletion, AiderCliError>> + Send>>;

#[derive(Clone)]
pub struct AiderTerminationHandle {
    inner: Arc<AiderTerminationInner>,
}

#[derive(Debug)]
struct AiderTerminationInner {
    requested: AtomicBool,
    notify: Notify,
}

impl AiderTerminationHandle {
    fn new() -> Self {
        Self {
            inner: Arc::new(AiderTerminationInner {
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

impl std::fmt::Debug for AiderTerminationHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AiderTerminationHandle")
            .field("requested", &self.is_requested())
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct AiderStreamJsonCompletion {
    pub status: std::process::ExitStatus,
    pub final_text: Option<String>,
    pub session_id: Option<String>,
    pub model: Option<String>,
    pub raw_result: Option<serde_json::Value>,
}

pub struct AiderStreamJsonHandle {
    pub events: DynAiderStreamJsonEventStream,
    pub completion: DynAiderStreamJsonCompletion,
}

impl std::fmt::Debug for AiderStreamJsonHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AiderStreamJsonHandle")
            .field("events", &"<stream>")
            .field("completion", &"<future>")
            .finish()
    }
}

pub struct AiderStreamJsonControlHandle {
    pub events: DynAiderStreamJsonEventStream,
    pub completion: DynAiderStreamJsonCompletion,
    pub termination: AiderTerminationHandle,
}

impl std::fmt::Debug for AiderStreamJsonControlHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AiderStreamJsonControlHandle")
            .field("events", &"<stream>")
            .field("completion", &"<future>")
            .field("termination", &self.termination)
            .finish()
    }
}
