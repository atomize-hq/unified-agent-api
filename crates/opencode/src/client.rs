use std::{
    collections::BTreeMap,
    path::PathBuf,
    pin::Pin,
    process::Stdio,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures_core::Stream;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    sync::{mpsc, oneshot},
};

use crate::{
    DynOpencodeRunJsonCompletion, DynOpencodeRunJsonEventStream, OpencodeError,
    OpencodeRunCompletion, OpencodeRunJsonControlHandle, OpencodeRunJsonEvent,
    OpencodeRunJsonHandle, OpencodeRunJsonParseError, OpencodeRunJsonParser, OpencodeRunRequest,
    OpencodeTerminationHandle,
};

const STDERR_CAPTURE_MAX_BYTES: usize = 4096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SelectionMode {
    Last,
    Id,
}

#[derive(Clone, Debug)]
pub struct OpencodeClient {
    pub(crate) binary: PathBuf,
    pub(crate) env: BTreeMap<String, String>,
    pub(crate) timeout: Option<Duration>,
}

impl OpencodeClient {
    pub fn builder() -> crate::OpencodeClientBuilder {
        crate::OpencodeClientBuilder::default()
    }

    pub async fn run_json(
        &self,
        request: OpencodeRunRequest,
    ) -> Result<OpencodeRunJsonHandle, OpencodeError> {
        let (events, completion, _termination) = self.spawn_run_json(request).await?;
        Ok(OpencodeRunJsonHandle { events, completion })
    }

    pub async fn run_json_control(
        &self,
        request: OpencodeRunRequest,
    ) -> Result<OpencodeRunJsonControlHandle, OpencodeError> {
        let (events, completion, termination) = self.spawn_run_json(request).await?;
        Ok(OpencodeRunJsonControlHandle {
            events,
            completion,
            termination,
        })
    }

    async fn spawn_run_json(
        &self,
        request: OpencodeRunRequest,
    ) -> Result<
        (
            DynOpencodeRunJsonEventStream,
            DynOpencodeRunJsonCompletion,
            OpencodeTerminationHandle,
        ),
        OpencodeError,
    > {
        let selection_mode = selection_mode(&request);
        let argv = request.argv()?;
        let mut command = tokio::process::Command::new(&self.binary);
        command
            .args(argv)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for (key, value) in &self.env {
            command.env(key, value);
        }

        let mut child = command.spawn().map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                OpencodeError::MissingBinary
            } else {
                OpencodeError::Spawn {
                    binary: self.binary.clone(),
                    source,
                }
            }
        })?;

        let stdout = child.stdout.take().ok_or(OpencodeError::MissingStdout)?;
        let stderr_capture = child
            .stderr
            .take()
            .map(|stderr| tokio::spawn(async move { capture_stderr(stderr).await }));
        let timeout = self.timeout;
        let termination = OpencodeTerminationHandle::new();
        let termination_for_runner = termination.clone();

        let (events_tx, events_rx) = mpsc::channel(32);
        let (completion_tx, completion_rx) = oneshot::channel();

        tokio::spawn(async move {
            let result = run_opencode_child(
                child,
                stdout,
                stderr_capture,
                events_tx,
                timeout,
                termination_for_runner,
                selection_mode,
            )
            .await;
            let _ = completion_tx.send(result);
        });

        let events: DynOpencodeRunJsonEventStream =
            Box::pin(OpencodeRunJsonEventChannelStream::new(events_rx));

        let completion: DynOpencodeRunJsonCompletion = Box::pin(async move {
            completion_rx
                .await
                .map_err(|_| OpencodeError::Join("run-json task dropped".to_string()))?
        });

        Ok((events, completion, termination))
    }
}

struct OpencodeRunJsonEventChannelStream {
    rx: mpsc::Receiver<Result<OpencodeRunJsonEvent, OpencodeRunJsonParseError>>,
}

impl OpencodeRunJsonEventChannelStream {
    fn new(rx: mpsc::Receiver<Result<OpencodeRunJsonEvent, OpencodeRunJsonParseError>>) -> Self {
        Self { rx }
    }
}

impl Stream for OpencodeRunJsonEventChannelStream {
    type Item = Result<OpencodeRunJsonEvent, OpencodeRunJsonParseError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().rx.poll_recv(cx)
    }
}

async fn run_opencode_child(
    mut child: tokio::process::Child,
    stdout: tokio::process::ChildStdout,
    stderr_capture: Option<tokio::task::JoinHandle<Result<Vec<u8>, std::io::Error>>>,
    events_tx: mpsc::Sender<Result<OpencodeRunJsonEvent, OpencodeRunJsonParseError>>,
    timeout: Option<Duration>,
    termination: OpencodeTerminationHandle,
    selection_mode: Option<SelectionMode>,
) -> Result<OpencodeRunCompletion, OpencodeError> {
    let mut reader = BufReader::new(stdout);
    let mut parser = OpencodeRunJsonParser::new();
    let mut line = String::new();
    let mut events_open = true;
    let mut final_text = String::new();
    let mut saw_finish = false;
    let deadline = timeout.map(|value| Instant::now() + value);

    loop {
        if let Some(deadline) = deadline {
            if Instant::now() >= deadline {
                let _ = child.start_kill();
                match child.wait().await {
                    Ok(_) => {
                        let _ = consume_stderr_capture(stderr_capture).await;
                        return Err(OpencodeError::Timeout {
                            timeout: timeout.expect("deadline implies timeout"),
                        });
                    }
                    Err(err) => return Err(OpencodeError::Wait(err)),
                }
            }
        }

        line.clear();
        let read_result = if let Some(deadline) = deadline {
            let remaining = deadline.saturating_duration_since(Instant::now());
            tokio::select! {
                _ = termination.requested() => {
                    let _ = child.start_kill();
                    break;
                }
                read = tokio::time::timeout(remaining, reader.read_line(&mut line)) => {
                    match read {
                        Ok(result) => result,
                        Err(_) => {
                            let _ = child.start_kill();
                            match child.wait().await {
                                Ok(_) => {
                                    let _ = consume_stderr_capture(stderr_capture).await;
                                    return Err(OpencodeError::Timeout {
                                        timeout: timeout.expect("deadline implies timeout"),
                                    });
                                }
                                Err(err) => return Err(OpencodeError::Wait(err)),
                            }
                        }
                    }
                }
            }
        } else {
            tokio::select! {
                _ = termination.requested() => {
                    let _ = child.start_kill();
                    break;
                }
                read = reader.read_line(&mut line) => read,
            }
        };

        let bytes = match read_result {
            Ok(bytes) => bytes,
            Err(err) => {
                let _ = child.start_kill();
                let _ = child.wait().await;
                let _ = consume_stderr_capture(stderr_capture).await;
                return Err(OpencodeError::StdoutRead(err));
            }
        };

        if bytes == 0 {
            break;
        }

        let parsed = parser.parse_line(line.trim_end_matches('\n'));
        match parsed {
            Ok(Some(event)) => {
                if let OpencodeRunJsonEvent::Text { text, .. } = &event {
                    final_text.push_str(text);
                } else if matches!(event, OpencodeRunJsonEvent::StepFinish { .. }) {
                    saw_finish = true;
                }

                if events_open && events_tx.send(Ok(event)).await.is_err() {
                    events_open = false;
                }
            }
            Ok(None) => {}
            Err(error) => {
                if events_open && events_tx.send(Err(error)).await.is_err() {
                    events_open = false;
                }
            }
        }
    }

    let status = child.wait().await.map_err(OpencodeError::Wait)?;
    let stderr = consume_stderr_capture(stderr_capture).await?;
    if !status.success() {
        if let Some(message) = classify_selection_failure(&stderr, selection_mode) {
            if events_open {
                let _ = events_tx
                    .send(Ok(OpencodeRunJsonEvent::TerminalError {
                        message: message.clone(),
                        raw: serde_json::Value::Null,
                    }))
                    .await;
            }
            drop(events_tx);
            return Err(OpencodeError::SelectionFailed { message });
        }
    }
    drop(events_tx);

    let final_text = (saw_finish && !final_text.is_empty()).then_some(final_text);

    Ok(OpencodeRunCompletion { status, final_text })
}

fn selection_mode(request: &OpencodeRunRequest) -> Option<SelectionMode> {
    if request.session_id().is_some() {
        Some(SelectionMode::Id)
    } else if request.continue_requested() {
        Some(SelectionMode::Last)
    } else {
        None
    }
}

async fn capture_stderr(
    mut stderr: tokio::process::ChildStderr,
) -> Result<Vec<u8>, std::io::Error> {
    let mut captured = Vec::new();
    let mut buffer = [0u8; 1024];

    loop {
        let read = stderr.read(&mut buffer).await?;
        if read == 0 {
            break;
        }

        if captured.len() < STDERR_CAPTURE_MAX_BYTES {
            let remaining = STDERR_CAPTURE_MAX_BYTES - captured.len();
            captured.extend_from_slice(&buffer[..read.min(remaining)]);
        }
    }

    Ok(captured)
}

async fn consume_stderr_capture(
    stderr_capture: Option<tokio::task::JoinHandle<Result<Vec<u8>, std::io::Error>>>,
) -> Result<String, OpencodeError> {
    let Some(stderr_capture) = stderr_capture else {
        return Ok(String::new());
    };

    let captured = stderr_capture
        .await
        .map_err(|err| OpencodeError::Join(format!("stderr capture task failed: {err}")))?
        .map_err(OpencodeError::StderrRead)?;

    Ok(String::from_utf8_lossy(&captured).into_owned())
}

fn classify_selection_failure(
    stderr: &str,
    selection_mode: Option<SelectionMode>,
) -> Option<String> {
    let selection_mode = selection_mode?;
    let stderr = stderr.to_ascii_lowercase();

    let saw_not_found = (stderr.contains("not found")
        && (stderr.contains("session")
            || stderr.contains("thread")
            || stderr.contains("conversation")))
        || stderr.contains("no session")
        || stderr.contains("no sessions")
        || stderr.contains("unknown session")
        || stderr.contains("no thread")
        || stderr.contains("no threads")
        || stderr.contains("unknown thread")
        || stderr.contains("no conversation")
        || stderr.contains("unknown conversation");

    if !saw_not_found {
        return None;
    }

    Some(match selection_mode {
        SelectionMode::Last => "no session found".to_string(),
        SelectionMode::Id => "session not found".to_string(),
    })
}
