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
    AiderCliError, AiderStreamJsonCompletion, AiderStreamJsonControlHandle, AiderStreamJsonError,
    AiderStreamJsonEvent, AiderStreamJsonHandle, AiderStreamJsonResultPayload,
    AiderStreamJsonRunRequest, AiderTerminationHandle, DynAiderStreamJsonCompletion,
    DynAiderStreamJsonEventStream,
};

const STDERR_CAPTURE_MAX_BYTES: usize = 4096;
const RUN_FAILED_MESSAGE: &str = "aider run failed";
const INVALID_INPUT_MESSAGE: &str = "invalid input";
const TURN_LIMIT_EXCEEDED_MESSAGE: &str = "turn limit exceeded";

#[derive(Clone, Debug)]
pub struct AiderCliClient {
    pub(crate) binary: PathBuf,
    pub(crate) env: BTreeMap<String, String>,
    pub(crate) timeout: Option<Duration>,
}

impl AiderCliClient {
    pub fn builder() -> crate::AiderCliClientBuilder {
        crate::AiderCliClientBuilder::default()
    }

    pub async fn stream_json(
        &self,
        request: AiderStreamJsonRunRequest,
    ) -> Result<AiderStreamJsonHandle, AiderCliError> {
        let (events, completion, _termination) = self.spawn_stream_json(request).await?;
        Ok(AiderStreamJsonHandle { events, completion })
    }

    pub async fn stream_json_control(
        &self,
        request: AiderStreamJsonRunRequest,
    ) -> Result<AiderStreamJsonControlHandle, AiderCliError> {
        let (events, completion, termination) = self.spawn_stream_json(request).await?;
        Ok(AiderStreamJsonControlHandle {
            events,
            completion,
            termination,
        })
    }

    async fn spawn_stream_json(
        &self,
        request: AiderStreamJsonRunRequest,
    ) -> Result<
        (
            DynAiderStreamJsonEventStream,
            DynAiderStreamJsonCompletion,
            AiderTerminationHandle,
        ),
        AiderCliError,
    > {
        let argv = request.argv()?;
        let mut command = tokio::process::Command::new(&self.binary);
        command
            .args(argv)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(working_dir) = request.working_directory() {
            command.current_dir(working_dir);
        }

        for (key, value) in &self.env {
            command.env(key, value);
        }

        let mut child = command.spawn().map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                AiderCliError::MissingBinary
            } else {
                AiderCliError::Spawn {
                    binary: self.binary.clone(),
                    source,
                }
            }
        })?;

        let stdout = child.stdout.take().ok_or(AiderCliError::MissingStdout)?;
        let stderr_capture = child
            .stderr
            .take()
            .map(|stderr| tokio::spawn(async move { capture_stderr(stderr).await }));
        let timeout = self.timeout;
        let termination = AiderTerminationHandle::new();
        let termination_for_runner = termination.clone();

        let (events_tx, events_rx) = mpsc::channel(32);
        let (completion_tx, completion_rx) = oneshot::channel();

        tokio::spawn(async move {
            let result = run_aider_child(
                child,
                stdout,
                stderr_capture,
                events_tx,
                timeout,
                termination_for_runner,
            )
            .await;
            let _ = completion_tx.send(result);
        });

        let events: DynAiderStreamJsonEventStream =
            Box::pin(AiderStreamJsonEventChannelStream::new(events_rx));

        let completion: DynAiderStreamJsonCompletion = Box::pin(async move {
            completion_rx
                .await
                .map_err(|_| AiderCliError::Join("stream-json task dropped".to_string()))?
        });

        Ok((events, completion, termination))
    }
}

struct AiderStreamJsonEventChannelStream {
    rx: mpsc::Receiver<Result<AiderStreamJsonEvent, AiderStreamJsonError>>,
}

impl AiderStreamJsonEventChannelStream {
    fn new(rx: mpsc::Receiver<Result<AiderStreamJsonEvent, AiderStreamJsonError>>) -> Self {
        Self { rx }
    }
}

impl Stream for AiderStreamJsonEventChannelStream {
    type Item = Result<AiderStreamJsonEvent, AiderStreamJsonError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().rx.poll_recv(cx)
    }
}

#[derive(Default)]
struct CompletionAccumulator {
    session_id: Option<String>,
    model: Option<String>,
    assistant_text: String,
    raw_result: Option<Value>,
}

use serde_json::Value;

impl CompletionAccumulator {
    fn observe(&mut self, event: &AiderStreamJsonEvent) {
        match event {
            AiderStreamJsonEvent::Init {
                session_id, model, ..
            } => {
                self.session_id = Some(session_id.clone());
                self.model = Some(model.clone());
            }
            AiderStreamJsonEvent::Message {
                role,
                content,
                delta,
                ..
            } if role == "assistant" => {
                if *delta || self.assistant_text.is_empty() {
                    self.assistant_text.push_str(content);
                } else {
                    self.assistant_text.push('\n');
                    self.assistant_text.push_str(content);
                }
            }
            AiderStreamJsonEvent::Result { payload } => {
                self.raw_result = Some(payload.raw.clone());
            }
            _ => {}
        }
    }

    fn final_text(&self) -> Option<String> {
        (!self.assistant_text.is_empty()).then(|| self.assistant_text.clone())
    }
}

async fn run_aider_child(
    mut child: tokio::process::Child,
    stdout: tokio::process::ChildStdout,
    stderr_capture: Option<tokio::task::JoinHandle<Result<Vec<u8>, std::io::Error>>>,
    events_tx: mpsc::Sender<Result<AiderStreamJsonEvent, AiderStreamJsonError>>,
    timeout: Option<Duration>,
    termination: AiderTerminationHandle,
) -> Result<AiderStreamJsonCompletion, AiderCliError> {
    let mut reader = BufReader::new(stdout);
    let mut parser = crate::AiderStreamJsonParser::new();
    let mut line = String::new();
    let mut events_open = true;
    let mut completion = CompletionAccumulator::default();
    let mut last_result: Option<AiderStreamJsonResultPayload> = None;
    let mut termination_requested = false;
    let deadline = timeout.map(|value| Instant::now() + value);
    let mut exit_status = None;

    loop {
        if let Some(deadline) = deadline {
            if Instant::now() >= deadline {
                match wait_for_child_exit(&mut child, timeout, Some(deadline)).await {
                    Ok(ChildExit::Exited(status)) => {
                        exit_status = Some(status);
                        break;
                    }
                    Ok(ChildExit::TimedOut) => {
                        let _ = consume_stderr_capture(stderr_capture).await;
                        return Err(AiderCliError::Timeout {
                            timeout: timeout.expect("deadline implies timeout"),
                        });
                    }
                    Err(err) => return Err(err),
                }
            }
        }

        line.clear();
        let read_result = if let Some(deadline) = deadline {
            let remaining = deadline.saturating_duration_since(Instant::now());
            tokio::select! {
                _ = termination.requested() => {
                    termination_requested = true;
                    let _ = child.start_kill();
                    break;
                }
                read = tokio::time::timeout(remaining, reader.read_line(&mut line)) => {
                    match read {
                        Ok(result) => result,
                        Err(_) => {
                            match wait_for_child_exit(&mut child, timeout, Some(deadline)).await {
                                Ok(ChildExit::Exited(status)) => {
                                    exit_status = Some(status);
                                    break;
                                }
                                Ok(ChildExit::TimedOut) => {
                                    let _ = consume_stderr_capture(stderr_capture).await;
                                    return Err(AiderCliError::Timeout {
                                        timeout: timeout.expect("deadline implies timeout"),
                                    });
                                }
                                Err(err) => return Err(err),
                            }
                        }
                    }
                }
            }
        } else {
            tokio::select! {
                _ = termination.requested() => {
                    termination_requested = true;
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
                return Err(AiderCliError::StdoutRead(err));
            }
        };

        if bytes == 0 {
            break;
        }

        let parsed = parser.parse_line(line.trim_end_matches('\n'));
        match parsed {
            Ok(Some(event)) => {
                completion.observe(&event);
                if let AiderStreamJsonEvent::Result { payload } = &event {
                    last_result = Some(payload.clone());
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

    let status = match exit_status {
        Some(status) => status,
        None => match wait_for_child_exit(&mut child, timeout, deadline).await {
            Ok(ChildExit::Exited(status)) => status,
            Ok(ChildExit::TimedOut) => {
                let _ = consume_stderr_capture(stderr_capture).await;
                return Err(AiderCliError::Timeout {
                    timeout: timeout.expect("deadline implies timeout"),
                });
            }
            Err(err) => return Err(err),
        },
    };

    let _stderr = consume_stderr_capture(stderr_capture).await?;

    if !status.success() {
        if termination_requested {
            drop(events_tx);
            return Ok(AiderStreamJsonCompletion {
                status,
                final_text: None,
                session_id: completion.session_id,
                model: completion.model,
                raw_result: completion.raw_result,
            });
        }

        let exit_code = status.code();
        let message = classify_run_failure(exit_code, last_result.as_ref());
        if last_result.is_none() && events_open {
            let _ = events_tx
                .send(Ok(AiderStreamJsonEvent::Error {
                    severity: "error".to_string(),
                    message: message.clone(),
                    raw: Value::Null,
                }))
                .await;
        }
        drop(events_tx);
        return Err(AiderCliError::RunFailed {
            status,
            exit_code,
            message,
            result_error_type: last_result
                .as_ref()
                .and_then(|payload| payload.error_type.clone()),
        });
    }

    drop(events_tx);
    Ok(AiderStreamJsonCompletion {
        status,
        final_text: completion.final_text(),
        session_id: completion.session_id,
        model: completion.model,
        raw_result: completion.raw_result,
    })
}

#[derive(Debug, Clone, Copy)]
enum ChildExit {
    Exited(std::process::ExitStatus),
    TimedOut,
}

async fn wait_for_child_exit(
    child: &mut tokio::process::Child,
    timeout: Option<Duration>,
    deadline: Option<Instant>,
) -> Result<ChildExit, AiderCliError> {
    match deadline {
        None => child
            .wait()
            .await
            .map(ChildExit::Exited)
            .map_err(AiderCliError::Wait),
        Some(deadline) => {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                match child.try_wait().map_err(AiderCliError::Wait)? {
                    Some(status) => Ok(ChildExit::Exited(status)),
                    None => {
                        timeout.expect("deadline implies timeout");
                        let _ = child.start_kill();
                        match child.wait().await {
                            Ok(_status) => Ok(ChildExit::TimedOut),
                            Err(err) => Err(AiderCliError::Wait(err)),
                        }
                    }
                }
            } else {
                match tokio::time::timeout(remaining, child.wait()).await {
                    Ok(result) => result.map(ChildExit::Exited).map_err(AiderCliError::Wait),
                    Err(_) => match child.try_wait().map_err(AiderCliError::Wait)? {
                        Some(status) => Ok(ChildExit::Exited(status)),
                        None => {
                            timeout.expect("deadline implies timeout");
                            let _ = child.start_kill();
                            match child.wait().await {
                                Ok(_status) => Ok(ChildExit::TimedOut),
                                Err(err) => Err(AiderCliError::Wait(err)),
                            }
                        }
                    },
                }
            }
        }
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
) -> Result<String, AiderCliError> {
    let Some(stderr_capture) = stderr_capture else {
        return Ok(String::new());
    };

    let captured = stderr_capture
        .await
        .map_err(|err| AiderCliError::Join(format!("stderr capture task failed: {err}")))?
        .map_err(AiderCliError::StderrRead)?;

    Ok(String::from_utf8_lossy(&captured).into_owned())
}

fn classify_run_failure(
    exit_code: Option<i32>,
    result: Option<&AiderStreamJsonResultPayload>,
) -> String {
    match exit_code {
        Some(42) => INVALID_INPUT_MESSAGE.to_string(),
        Some(53) => TURN_LIMIT_EXCEEDED_MESSAGE.to_string(),
        _ => result
            .and_then(|payload| payload.error_message.clone())
            .filter(|message| !message.trim().is_empty())
            .unwrap_or_else(|| RUN_FAILED_MESSAGE.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::process::Stdio;

    use super::{wait_for_child_exit, ChildExit};
    use std::time::{Duration, Instant};

    #[cfg(unix)]
    #[tokio::test]
    async fn wait_for_child_exit_returns_status_when_deadline_has_elapsed() {
        let mut child = tokio::process::Command::new("sh")
            .args(["-c", "exit 0"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn child");
        tokio::time::sleep(Duration::from_millis(50)).await;

        let outcome = wait_for_child_exit(
            &mut child,
            Some(Duration::from_millis(1)),
            Some(Instant::now()),
        )
        .await
        .expect("wait helper succeeds");

        match outcome {
            ChildExit::Exited(status) => assert!(status.success()),
            ChildExit::TimedOut => panic!("expected exited status, got timeout"),
        }
    }
}
