//! Application-owned native interactions. Payloads cross a trusted host boundary.
use serde_json::Value;
use std::{future::Future, pin::Pin, sync::Arc};

/// Complete original native parameters and a separate immutable file-change item.
#[derive(Clone, Debug)]
pub struct CodexInteractionRequest {
    pub method: String,
    pub params: Value,
    pub item_context: Option<Value>,
}
/// A native structured response, or an explicit refusal to handle the request.
#[derive(Clone, Debug)]
pub enum CodexInteractionReply {
    Result(Value),
    Unsupported,
}
/// An application interaction failure. Diagnostics deliberately contain no payload.
#[derive(Clone, Debug, Default)]
pub struct CodexInteractionError;
impl CodexInteractionError {
    pub fn new() -> Self {
        Self
    }
}
impl std::fmt::Display for CodexInteractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Codex interaction failed")
    }
}
impl std::error::Error for CodexInteractionError {}
pub(super) type InteractionFuture =
    Pin<Box<dyn Future<Output = Result<CodexInteractionReply, CodexInteractionError>> + Send>>;
#[derive(Clone)]
pub(super) struct Handler(
    pub Arc<dyn Fn(CodexInteractionRequest) -> InteractionFuture + Send + Sync>,
);
impl std::fmt::Debug for Handler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CodexInteractionHandler")
    }
}
