use std::{path::PathBuf, process::ExitStatus, time::Duration};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GeminiCliError {
    #[error("gemini binary not found")]
    MissingBinary,
    #[error("failed to spawn gemini process (binary={binary:?}): {source}")]
    Spawn {
        binary: PathBuf,
        source: std::io::Error,
    },
    #[error("gemini process timed out after {timeout:?}")]
    Timeout { timeout: Duration },
    #[error("failed waiting for gemini process: {0}")]
    Wait(std::io::Error),
    #[error("failed reading stdout: {0}")]
    StdoutRead(std::io::Error),
    #[error("failed reading stderr: {0}")]
    StderrRead(std::io::Error),
    #[error("internal error: missing stdout pipe")]
    MissingStdout,
    #[error("internal error: join failure: {0}")]
    Join(String),
    #[error("request is invalid: {0}")]
    InvalidRequest(String),
    #[error("{message}")]
    RunFailed {
        status: ExitStatus,
        exit_code: Option<i32>,
        message: String,
        result_error_type: Option<String>,
    },
}
