#![forbid(unsafe_code)]
//! Async helper around the official Gemini CLI headless `--output-format stream-json` surface.
//!
//! The public event types follow the documented stream-json contract, while preserving raw JSON
//! payloads and tolerating unknown future event kinds.

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

pub use builder::GeminiCliClientBuilder;
pub use client::GeminiCliClient;
pub use error::GeminiCliError;
pub use stream_json::{
    parse_stream_json_lines, GeminiStreamJsonError, GeminiStreamJsonErrorCode,
    GeminiStreamJsonEvent, GeminiStreamJsonLine, GeminiStreamJsonLineOutcome,
    GeminiStreamJsonParser, GeminiStreamJsonResultPayload, GeminiStreamJsonRunRequest,
    GeminiToolResultError,
};

pub type DynGeminiStreamJsonEventStream =
    Pin<Box<dyn Stream<Item = Result<GeminiStreamJsonEvent, GeminiStreamJsonError>> + Send>>;

pub type DynGeminiStreamJsonCompletion =
    Pin<Box<dyn Future<Output = Result<GeminiStreamJsonCompletion, GeminiCliError>> + Send>>;

#[derive(Clone)]
pub struct GeminiTerminationHandle {
    inner: Arc<GeminiTerminationInner>,
}

#[derive(Debug)]
struct GeminiTerminationInner {
    requested: AtomicBool,
    notify: Notify,
}

impl GeminiTerminationHandle {
    fn new() -> Self {
        Self {
            inner: Arc::new(GeminiTerminationInner {
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

impl std::fmt::Debug for GeminiTerminationHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeminiTerminationHandle")
            .field("requested", &self.is_requested())
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct GeminiStreamJsonCompletion {
    pub status: std::process::ExitStatus,
    pub final_text: Option<String>,
    pub session_id: Option<String>,
    pub model: Option<String>,
    pub raw_result: Option<serde_json::Value>,
}

pub struct GeminiStreamJsonHandle {
    pub events: DynGeminiStreamJsonEventStream,
    pub completion: DynGeminiStreamJsonCompletion,
}

impl std::fmt::Debug for GeminiStreamJsonHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeminiStreamJsonHandle")
            .field("events", &"<stream>")
            .field("completion", &"<future>")
            .finish()
    }
}

pub struct GeminiStreamJsonControlHandle {
    pub events: DynGeminiStreamJsonEventStream,
    pub completion: DynGeminiStreamJsonCompletion,
    pub termination: GeminiTerminationHandle,
}

impl std::fmt::Debug for GeminiStreamJsonControlHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeminiStreamJsonControlHandle")
            .field("events", &"<stream>")
            .field("completion", &"<future>")
            .field("termination", &self.termination)
            .finish()
    }
}
