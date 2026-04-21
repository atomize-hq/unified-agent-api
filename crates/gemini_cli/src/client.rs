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
    DynGeminiStreamJsonCompletion, DynGeminiStreamJsonEventStream, GeminiCliError,
    GeminiStreamJsonCompletion, GeminiStreamJsonControlHandle, GeminiStreamJsonError,
    GeminiStreamJsonEvent, GeminiStreamJsonHandle, GeminiStreamJsonResultPayload,
    GeminiStreamJsonRunRequest, GeminiTerminationHandle,
};

const STDERR_CAPTURE_MAX_BYTES: usize = 4096;
const RUN_FAILED_MESSAGE: &str = "gemini run failed";
const INVALID_INPUT_MESSAGE: &str = "invalid input";
const TURN_LIMIT_EXCEEDED_MESSAGE: &str = "turn limit exceeded";

#[derive(Clone, Debug)]
pub struct GeminiCliClient {
    pub(crate) binary: PathBuf,
    pub(crate) env: BTreeMap<String, String>,
    pub(crate) timeout: Option<Duration>,
}

impl GeminiCliClient {
    pub fn builder() -> crate::GeminiCliClientBuilder {
        crate::GeminiCliClientBuilder::default()
    }

    pub async fn stream_json(
        &self,
        request: GeminiStreamJsonRunRequest,
    ) -> Result<GeminiStreamJsonHandle, GeminiCliError> {
        let (events, completion, _termination) = self.spawn_stream_json(request).await?;
        Ok(GeminiStreamJsonHandle { events, completion })
    }

    pub async fn stream_json_control(
        &self,
        request: GeminiStreamJsonRunRequest,
    ) -> Result<GeminiStreamJsonControlHandle, GeminiCliError> {
        let (events, completion, termination) = self.spawn_stream_json(request).await?;
        Ok(GeminiStreamJsonControlHandle {
            events,
            completion,
            termination,
        })
    }

    async fn spawn_stream_json(
        &self,
        request: GeminiStreamJsonRunRequest,
    ) -> Result<
        (
            DynGeminiStreamJsonEventStream,
            DynGeminiStreamJsonCompletion,
            GeminiTerminationHandle,
        ),
        GeminiCliError,
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
                GeminiCliError::MissingBinary
            } else {
                GeminiCliError::Spawn {
                    binary: self.binary.clone(),
                    source,
                }
            }
        })?;

        let stdout = child.stdout.take().ok_or(GeminiCliError::MissingStdout)?;
        let stderr_capture = child
            .stderr
            .take()
            .map(|stderr| tokio::spawn(async move { capture_stderr(stderr).await }));
        let timeout = self.timeout;
        let termination = GeminiTerminationHandle::new();
        let termination_for_runner = termination.clone();

        let (events_tx, events_rx) = mpsc::channel(32);
        let (completion_tx, completion_rx) = oneshot::channel();

        tokio::spawn(async move {
            let result = run_gemini_child(
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

        let events: DynGeminiStreamJsonEventStream =
            Box::pin(GeminiStreamJsonEventChannelStream::new(events_rx));

        let completion: DynGeminiStreamJsonCompletion = Box::pin(async move {
            completion_rx
                .await
                .map_err(|_| GeminiCliError::Join("stream-json task dropped".to_string()))?
        });

        Ok((events, completion, termination))
    }
}

struct GeminiStreamJsonEventChannelStream {
    rx: mpsc::Receiver<Result<GeminiStreamJsonEvent, GeminiStreamJsonError>>,
}

impl GeminiStreamJsonEventChannelStream {
    fn new(rx: mpsc::Receiver<Result<GeminiStreamJsonEvent, GeminiStreamJsonError>>) -> Self {
        Self { rx }
    }
}

impl Stream for GeminiStreamJsonEventChannelStream {
    type Item = Result<GeminiStreamJsonEvent, GeminiStreamJsonError>;

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
    fn observe(&mut self, event: &GeminiStreamJsonEvent) {
        match event {
            GeminiStreamJsonEvent::Init {
                session_id, model, ..
            } => {
                self.session_id = Some(session_id.clone());
                self.model = Some(model.clone());
            }
            GeminiStreamJsonEvent::Message {
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
            GeminiStreamJsonEvent::Result { payload } => {
                self.raw_result = Some(payload.raw.clone());
            }
            _ => {}
        }
    }

    fn final_text(&self) -> Option<String> {
        (!self.assistant_text.is_empty()).then(|| self.assistant_text.clone())
    }
}

async fn run_gemini_child(
    mut child: tokio::process::Child,
    stdout: tokio::process::ChildStdout,
    stderr_capture: Option<tokio::task::JoinHandle<Result<Vec<u8>, std::io::Error>>>,
    events_tx: mpsc::Sender<Result<GeminiStreamJsonEvent, GeminiStreamJsonError>>,
    timeout: Option<Duration>,
    termination: GeminiTerminationHandle,
) -> Result<GeminiStreamJsonCompletion, GeminiCliError> {
    let mut reader = BufReader::new(stdout);
    let mut parser = crate::GeminiStreamJsonParser::new();
    let mut line = String::new();
    let mut events_open = true;
    let mut completion = CompletionAccumulator::default();
    let mut last_result: Option<GeminiStreamJsonResultPayload> = None;
    let mut termination_requested = false;
    let deadline = timeout.map(|value| Instant::now() + value);

    loop {
        if let Some(deadline) = deadline {
            if Instant::now() >= deadline {
                match wait_for_child_exit(&mut child, timeout, Some(deadline)).await {
                    Ok(_) => {
                        let _ = consume_stderr_capture(stderr_capture).await;
                        return Err(GeminiCliError::Timeout {
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
                                Ok(_) => {
                                    let _ = consume_stderr_capture(stderr_capture).await;
                                    return Err(GeminiCliError::Timeout {
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
                return Err(GeminiCliError::StdoutRead(err));
            }
        };

        if bytes == 0 {
            break;
        }

        let parsed = parser.parse_line(line.trim_end_matches('\n'));
        match parsed {
            Ok(Some(event)) => {
                completion.observe(&event);
                if let GeminiStreamJsonEvent::Result { payload } = &event {
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

    let status = match wait_for_child_exit(&mut child, timeout, deadline).await {
        Ok(status) => status,
        Err(err @ GeminiCliError::Timeout { .. }) => {
            let _ = consume_stderr_capture(stderr_capture).await;
            return Err(err);
        }
        Err(err) => return Err(err),
    };

    let _stderr = consume_stderr_capture(stderr_capture).await?;

    if !status.success() {
        if termination_requested {
            drop(events_tx);
            return Ok(GeminiStreamJsonCompletion {
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
                .send(Ok(GeminiStreamJsonEvent::Error {
                    severity: "error".to_string(),
                    message: message.clone(),
                    raw: Value::Null,
                }))
                .await;
        }
        drop(events_tx);
        return Err(GeminiCliError::RunFailed {
            status,
            exit_code,
            message,
            result_error_type: last_result
                .as_ref()
                .and_then(|payload| payload.error_type.clone()),
        });
    }

    drop(events_tx);
    Ok(GeminiStreamJsonCompletion {
        status,
        final_text: completion.final_text(),
        session_id: completion.session_id,
        model: completion.model,
        raw_result: completion.raw_result,
    })
}

async fn wait_for_child_exit(
    child: &mut tokio::process::Child,
    timeout: Option<Duration>,
    deadline: Option<Instant>,
) -> Result<std::process::ExitStatus, GeminiCliError> {
    match deadline {
        None => child.wait().await.map_err(GeminiCliError::Wait),
        Some(deadline) => {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                let timeout = timeout.expect("deadline implies timeout");
                let _ = child.start_kill();
                match child.wait().await {
                    Ok(_status) => Err(GeminiCliError::Timeout { timeout }),
                    Err(err) => Err(GeminiCliError::Wait(err)),
                }
            } else {
                match tokio::time::timeout(remaining, child.wait()).await {
                    Ok(result) => result.map_err(GeminiCliError::Wait),
                    Err(_) => {
                        let timeout = timeout.expect("deadline implies timeout");
                        let _ = child.start_kill();
                        match child.wait().await {
                            Ok(_status) => Err(GeminiCliError::Timeout { timeout }),
                            Err(err) => Err(GeminiCliError::Wait(err)),
                        }
                    }
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
) -> Result<String, GeminiCliError> {
    let Some(stderr_capture) = stderr_capture else {
        return Ok(String::new());
    };

    let captured = stderr_capture
        .await
        .map_err(|err| GeminiCliError::Join(format!("stderr capture task failed: {err}")))?
        .map_err(GeminiCliError::StderrRead)?;

    Ok(String::from_utf8_lossy(&captured).into_owned())
}

fn classify_run_failure(
    exit_code: Option<i32>,
    result: Option<&GeminiStreamJsonResultPayload>,
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
