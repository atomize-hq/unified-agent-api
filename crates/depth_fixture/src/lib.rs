//! Wrapper for the synthetic `depth_fixture` CLI: one streaming, wrapper-only `run` operation.
//!
//! It is the subject of the isolated support-depth machinery proof and qualifies no real agent.
//! Promise, by dimension:
//! - Request: `run --json [--model <NAME>] -- <PROMPT>`; the prompt always follows `--`.
//! - Validation, before spawn: a non-empty prompt of at most `MAX_PROMPT_BYTES` without NUL; a
//!   model of 1–64 ASCII alphanumerics, `.`, `_` or `-`; a non-zero timeout.
//! - Environment: the child gets a cleared environment plus only the caller's explicit variables,
//!   null stdin, and the caller's working directory if one is set.
//! - Output: JSON Lines events; known kinds are typed, unknown kinds are surfaced, malformed or
//!   oversized lines are errors whose projection is bounded.
//! - Lifecycle: events stream before completion; end of output is not success without a terminal
//!   `run.completed` and a zero exit; timeout and cancel kill and reap the child; drop reaps it.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub const MAX_PROMPT_BYTES: usize = 8 * 1024;
pub const DEFAULT_MAX_LINE_BYTES: usize = 64 * 1024;
/// Bytes of a rejected line or of stderr kept in an error. Raw output is never returned whole.
pub const DIAGNOSTIC_BYTES: usize = 256;

#[derive(Debug, Clone)]
pub struct DepthFixture {
    pub binary: PathBuf,
    pub timeout: Duration,
    pub env: BTreeMap<String, String>,
    pub current_dir: Option<PathBuf>,
    pub max_line_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRequest {
    pub prompt: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Started { session_id: String },
    Message { text: String, model: Option<String> },
    Unknown { kind: String },
    Completed,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid request: {0}")]
    InvalidRequest(&'static str),
    #[error("failed to spawn depth_fixture: {0}")]
    Spawn(std::io::Error),
    #[error("malformed event line: {prefix}")]
    Malformed { prefix: String },
    #[error("event line exceeds {limit} bytes")]
    OversizedLine { limit: usize },
    #[error("depth_fixture exited with {code:?}: {stderr}")]
    Exit { code: Option<i32>, stderr: String },
    #[error("depth_fixture ended without a terminal run.completed event")]
    NoTerminal,
    #[error("run timed out")]
    Timeout,
    #[error("run was cancelled")]
    Cancelled,
    #[error("failed to read depth_fixture output: {0}")]
    Io(std::io::Error),
}

impl DepthFixture {
    pub fn new(binary: impl Into<PathBuf>) -> Self {
        Self {
            binary: binary.into(),
            timeout: Duration::from_secs(30),
            env: BTreeMap::new(),
            current_dir: None,
            max_line_bytes: DEFAULT_MAX_LINE_BYTES,
        }
    }

    /// Validates the request, then spawns the child. Nothing is spawned when validation fails.
    pub fn run(&self, request: &RunRequest) -> Result<Run, Error> {
        validate(self, request)?;
        let mut command = Command::new(&self.binary);
        command.arg("run").arg("--json");
        if let Some(model) = &request.model {
            command.arg("--model").arg(model);
        }
        command.arg("--").arg(&request.prompt);
        command.env_clear().envs(&self.env);
        if let Some(dir) = &self.current_dir {
            command.current_dir(dir);
        }
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn().map_err(Error::Spawn)?;
        let stdout = child.stdout.take().expect("piped stdout");
        let stderr = child.stderr.take().expect("piped stderr");
        let (sender, lines) = mpsc::channel();
        let limit = self.max_line_bytes;
        thread::spawn(move || read_lines(stdout, limit, &sender));
        let stderr = thread::spawn(move || tail(stderr));
        Ok(Run {
            child,
            lines,
            stderr: Some(stderr),
            deadline: Instant::now() + self.timeout,
            limit,
            completed: false,
            cancelled: false,
            done: false,
        })
    }
}

fn validate(fixture: &DepthFixture, request: &RunRequest) -> Result<(), Error> {
    if fixture.timeout.is_zero() {
        return Err(Error::InvalidRequest("timeout must be non-zero"));
    }
    if request.prompt.is_empty() {
        return Err(Error::InvalidRequest("prompt must not be empty"));
    }
    if request.prompt.len() > MAX_PROMPT_BYTES {
        return Err(Error::InvalidRequest("prompt exceeds MAX_PROMPT_BYTES"));
    }
    if request.prompt.contains('\0') {
        return Err(Error::InvalidRequest("prompt must not contain NUL"));
    }
    if let Some(model) = &request.model {
        let valid = (1..=64).contains(&model.len())
            && model
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'));
        if !valid {
            return Err(Error::InvalidRequest(
                "model must be 1-64 of [A-Za-z0-9._-]",
            ));
        }
    }
    Ok(())
}

enum Line {
    Text(Vec<u8>),
    Oversized,
    Failed(std::io::Error),
}

fn read_lines(stdout: impl Read, limit: usize, sender: &mpsc::Sender<Line>) {
    let mut reader = BufReader::new(stdout);
    loop {
        let mut buf = Vec::new();
        match reader
            .by_ref()
            .take(limit as u64 + 1)
            .read_until(b'\n', &mut buf)
        {
            Ok(0) => return,
            Ok(_) if buf.len() > limit && !buf.ends_with(b"\n") => {
                let _ = sender.send(Line::Oversized);
                return;
            }
            Ok(_) => {
                if sender.send(Line::Text(buf)).is_err() {
                    return;
                }
            }
            Err(error) => {
                let _ = sender.send(Line::Failed(error));
                return;
            }
        }
    }
}

fn tail(stderr: impl Read) -> String {
    let mut bytes = Vec::new();
    let _ = stderr.take(64 * 1024).read_to_end(&mut bytes);
    let start = bytes.len().saturating_sub(DIAGNOSTIC_BYTES);
    String::from_utf8_lossy(&bytes[start..]).trim().to_owned()
}

fn bounded(bytes: &[u8]) -> String {
    let end = bytes.len().min(DIAGNOSTIC_BYTES);
    String::from_utf8_lossy(&bytes[..end]).trim_end().to_owned()
}

fn parse(line: &[u8]) -> Result<Event, Error> {
    let malformed = || Error::Malformed {
        prefix: bounded(line),
    };
    let value: serde_json::Value = serde_json::from_slice(line).map_err(|_| malformed())?;
    let kind = value
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(malformed)?;
    let text = |field: &str| value.get(field).and_then(|v| v.as_str()).map(str::to_owned);
    Ok(match kind {
        "session.started" => Event::Started {
            session_id: text("session_id").ok_or_else(malformed)?,
        },
        "message" => Event::Message {
            text: text("text").ok_or_else(malformed)?,
            model: text("model"),
        },
        "run.completed" => Event::Completed,
        other => Event::Unknown {
            kind: other.to_owned(),
        },
    })
}

/// A live run. Iterate with [`Run::next_event`]; dropping it kills and reaps the child.
pub struct Run {
    child: Child,
    lines: Receiver<Line>,
    stderr: Option<JoinHandle<String>>,
    deadline: Instant,
    limit: usize,
    completed: bool,
    cancelled: bool,
    done: bool,
}

impl Run {
    /// The next event, a terminal error, or `None` after a successful completion.
    pub fn next_event(&mut self) -> Option<Result<Event, Error>> {
        if self.done {
            return std::mem::take(&mut self.cancelled).then_some(Err(Error::Cancelled));
        }
        let wait = self.deadline.saturating_duration_since(Instant::now());
        let result = match self.lines.recv_timeout(wait) {
            Ok(Line::Text(bytes)) => match parse(&bytes) {
                Ok(Event::Completed) => {
                    self.completed = true;
                    return Some(Ok(Event::Completed));
                }
                Ok(event) => return Some(Ok(event)),
                Err(error) => Some(Err(error)),
            },
            Ok(Line::Oversized) => Some(Err(Error::OversizedLine { limit: self.limit })),
            Ok(Line::Failed(error)) => Some(Err(Error::Io(error))),
            Err(RecvTimeoutError::Timeout) => Some(Err(Error::Timeout)),
            Err(RecvTimeoutError::Disconnected) => self.finish(),
        };
        self.stop();
        result
    }

    /// Collects every remaining event, stopping at the first error.
    pub fn collect(mut self) -> Result<Vec<Event>, Error> {
        let mut events = Vec::new();
        while let Some(next) = self.next_event() {
            events.push(next?);
        }
        Ok(events)
    }

    /// Kills and reaps the child. The next `next_event` returns `Cancelled` once, then `None`.
    pub fn cancel(&mut self) {
        if !self.done {
            self.cancelled = true;
        }
        self.stop();
    }

    fn finish(&mut self) -> Option<Result<Event, Error>> {
        let status = self.wait();
        let stderr = self
            .stderr
            .take()
            .and_then(|h| h.join().ok())
            .unwrap_or_default();
        match status {
            Err(error) => Some(Err(Error::Io(error))),
            Ok(status) if !status.success() => Some(Err(Error::Exit {
                code: status.code(),
                stderr,
            })),
            Ok(_) if !self.completed => Some(Err(Error::NoTerminal)),
            Ok(_) => None,
        }
    }

    fn wait(&mut self) -> std::io::Result<ExitStatus> {
        self.child.wait()
    }

    fn stop(&mut self) {
        if !self.done {
            self.done = true;
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }

    /// The child's process id, for reaping checks.
    pub fn id(&self) -> u32 {
        self.child.id()
    }
}

impl Drop for Run {
    fn drop(&mut self) {
        self.stop();
    }
}
