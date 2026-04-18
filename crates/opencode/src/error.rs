use std::{path::PathBuf, time::Duration};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpencodeError {
    #[error("opencode binary not found")]
    MissingBinary,
    #[error("failed to spawn opencode process (binary={binary:?}): {source}")]
    Spawn {
        binary: PathBuf,
        source: std::io::Error,
    },
    #[error("opencode process timed out after {timeout:?}")]
    Timeout { timeout: Duration },
    #[error("failed waiting for opencode process: {0}")]
    Wait(std::io::Error),
    #[error("failed reading stdout: {0}")]
    StdoutRead(std::io::Error),
    #[error("internal error: missing stdout pipe")]
    MissingStdout,
    #[error("internal error: join failure: {0}")]
    Join(String),
    #[error("request is invalid: {0}")]
    InvalidRequest(String),
}
