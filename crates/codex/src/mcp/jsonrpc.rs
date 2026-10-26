use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsString,
    future::poll_fn,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use serde::Deserialize;
use serde_json::Value;
use tokio::{
    io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    process::{Child, ChildStderr, ChildStdin, Command},
    sync::{mpsc, oneshot, Mutex},
    task::JoinHandle,
    time,
};
use tracing::{debug, warn};
mod reader;
use super::interactions::{AppServerMessageStream, InteractionState};
use reader::reader_task;

pub(super) enum WriteMessage {
    Line(String),
    Close,
    Reply {
        state: Arc<InteractionState>,
        token: u64,
        result: Option<Value>,
    },
}

use super::{
    AppNotification, CodexEvent, EventStream, InitializeParams, McpError, RequestId,
    StdioServerConfig, METHOD_CANCEL, METHOD_EXIT, METHOD_INITIALIZE, METHOD_SHUTDOWN,
};

#[derive(Clone)]
enum NotificationHook {
    Codex {
        sinks: Arc<Mutex<Vec<mpsc::UnboundedSender<CodexEvent>>>>,
    },
    App {
        sinks: Arc<Mutex<Vec<mpsc::UnboundedSender<AppNotification>>>>,
        interactions: Option<InteractionDelivery>,
    },
}

#[derive(Clone)]
struct InteractionDelivery {
    state: Arc<InteractionState>,
    messages: mpsc::Sender<super::interactions::Delivery>,
}

type PendingRequests = Arc<Mutex<HashMap<RequestId, oneshot::Sender<Result<Value, McpError>>>>>;

/// Internal transport that handles stdio JSON-RPC.
pub(super) struct JsonRpcTransport {
    writer: mpsc::UnboundedSender<WriteMessage>,
    pending: PendingRequests,
    notification_hook: NotificationHook,
    next_id: AtomicU64,
    tasks: Vec<JoinHandle<()>>,
    child: Arc<Mutex<Option<Child>>>,
    startup_timeout: Duration,
    is_app: bool,
}

impl JsonRpcTransport {
    pub(super) async fn spawn_mcp(config: StdioServerConfig) -> Result<Self, McpError> {
        let hook = NotificationHook::Codex {
            sinks: Arc::new(Mutex::new(Vec::new())),
        };
        Self::spawn_with_subcommand(config, "mcp-server", hook, None, BTreeMap::new()).await
    }

    pub(super) async fn spawn_app(config: StdioServerConfig) -> Result<Self, McpError> {
        let hook = NotificationHook::App {
            sinks: Arc::new(Mutex::new(Vec::new())),
            interactions: None,
        };
        Self::spawn_with_subcommand(config, "app-server", hook, None, BTreeMap::new()).await
    }

    pub(super) async fn spawn_app_with_interactions(
        config: StdioServerConfig,
        overrides: BTreeMap<String, Value>,
    ) -> Result<(Self, AppServerMessageStream), McpError> {
        let (writer, rx) = mpsc::unbounded_channel();
        let (errors, error_rx) = mpsc::unbounded_channel();
        let state = InteractionState::new(writer.clone(), errors);
        // Backpressure is bounded independently of the 16 outstanding native request limit.
        let (messages, message_rx) = mpsc::channel(16);
        let hook = NotificationHook::App {
            sinks: Arc::new(Mutex::new(Vec::new())),
            interactions: Some(InteractionDelivery {
                state: state.clone(),
                messages,
            }),
        };
        let transport =
            Self::spawn_with_subcommand(config, "app-server", hook, Some((writer, rx)), overrides)
                .await?;
        Ok((
            transport,
            AppServerMessageStream {
                rx: message_rx,
                errors: error_rx,
                state,
                terminal: false,
            },
        ))
    }

    async fn spawn_with_subcommand(
        config: StdioServerConfig,
        subcommand: &str,
        notification_hook: NotificationHook,
        writer_channel: Option<(
            mpsc::UnboundedSender<WriteMessage>,
            mpsc::UnboundedReceiver<WriteMessage>,
        )>,
        overrides: BTreeMap<String, Value>,
    ) -> Result<Self, McpError> {
        let mut command = Command::new(&config.binary);
        for (key, value) in overrides {
            let native = toml::Value::try_from(value).map_err(|_| {
                McpError::Transport(
                    "native startup configuration cannot be represented in TOML".into(),
                )
            })?;
            command.arg("--config").arg(format!("{key}={native}"));
        }
        command
            .arg(subcommand)
            .args(
                (subcommand == "app-server" && config.app_server_analytics_default_enabled)
                    .then_some(OsString::from("--analytics-default-enabled")),
            )
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        if let Some(dir) = &config.current_dir {
            command.current_dir(dir);
        }

        if let Some(code_home) = &config.code_home {
            command.env("CODEX_HOME", code_home);
        }

        for (key, value) in &config.env {
            command.env(key, value);
        }

        let command_debug = format!("{command:?}");
        let mut backoff = Duration::from_millis(2);
        let mut child = {
            let mut child = None;
            for attempt in 0..5 {
                match command.spawn() {
                    Ok(spawned) => {
                        child = Some(spawned);
                        break;
                    }
                    Err(source) => {
                        let is_busy =
                            matches!(source.kind(), std::io::ErrorKind::ExecutableFileBusy)
                                || source.raw_os_error() == Some(26);
                        if is_busy && attempt < 4 {
                            tokio::time::sleep(backoff).await;
                            backoff = std::cmp::min(backoff * 2, Duration::from_millis(50));
                            continue;
                        }
                        return Err(McpError::Spawn {
                            command: command_debug,
                            source,
                        });
                    }
                }
            }
            child.expect("spawn loop should return or set child")
        };

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| McpError::Transport("child stdout unavailable".into()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpError::Transport("child stdin unavailable".into()))?;
        let stderr = child.stderr.take();

        let pending = Arc::new(Mutex::new(HashMap::new()));
        let (writer_tx, writer_rx) = writer_channel.unwrap_or_else(mpsc::unbounded_channel);

        let writer_handle = tokio::spawn(writer_task(stdin, writer_rx, notification_hook.clone()));
        let reader_handle = tokio::spawn(reader_task(
            stdout,
            pending.clone(),
            notification_hook.clone(),
            config.mirror_stdio,
            writer_tx.clone(),
        ));

        let stderr_handle = stderr.map(|stderr| {
            tokio::spawn(stderr_task(
                stderr,
                config.mirror_stdio,
                subcommand == "app-server",
            ))
        });

        let mut tasks = vec![writer_handle, reader_handle];
        if let Some(handle) = stderr_handle {
            tasks.push(handle);
        }

        Ok(Self {
            writer: writer_tx,
            pending,
            notification_hook,
            next_id: AtomicU64::new(1),
            tasks,
            child: Arc::new(Mutex::new(Some(child))),
            startup_timeout: config.startup_timeout,
            is_app: subcommand == "app-server",
        })
    }

    pub(super) async fn initialize(
        &self,
        params: InitializeParams,
        timeout: Duration,
    ) -> Result<Value, McpError> {
        let (_, rx) = self
            .request(METHOD_INITIALIZE, serde_json::to_value(params)?)
            .await?;
        recv_with_timeout(rx, timeout).await
    }

    pub(super) async fn request(
        &self,
        method: &str,
        params: Value,
    ) -> Result<(RequestId, oneshot::Receiver<Result<Value, McpError>>), McpError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let message = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });
        let serialized = serde_json::to_string(&message)?;
        let (tx, rx) = oneshot::channel();

        {
            let mut guard = self.pending.lock().await;
            guard.insert(id, tx);
        }

        if self.writer.send(WriteMessage::Line(serialized)).is_err() {
            let mut guard = self.pending.lock().await;
            guard.remove(&id);
            return Err(McpError::ChannelClosed);
        }

        Ok((id, rx))
    }

    pub(super) async fn register_codex_listener(&self) -> EventStream<CodexEvent> {
        match &self.notification_hook {
            NotificationHook::Codex { sinks } => {
                let (tx, rx) = mpsc::unbounded_channel();
                let mut guard = sinks.lock().await;
                guard.push(tx);
                rx
            }
            _ => {
                let (_tx, rx) = mpsc::unbounded_channel();
                rx
            }
        }
    }

    pub(super) async fn register_app_listener(&self) -> EventStream<AppNotification> {
        match &self.notification_hook {
            NotificationHook::App {
                interactions: Some(_),
                ..
            } => {
                // The ordered consumer owns notifications; do not retain unbounded duplicates.
                let (_tx, rx) = mpsc::unbounded_channel();
                rx
            }
            NotificationHook::App { sinks, .. } => {
                let (tx, rx) = mpsc::unbounded_channel();
                let mut guard = sinks.lock().await;
                guard.push(tx);
                rx
            }
            _ => {
                let (_tx, rx) = mpsc::unbounded_channel();
                rx
            }
        }
    }

    pub(super) fn cancel(&self, request_id: RequestId) -> Result<(), McpError> {
        self.invalidate_interactions();
        let message = serde_json::json!({
            "jsonrpc": "2.0",
            "method": METHOD_CANCEL,
            "params": { "id": request_id }
        });
        let serialized = serde_json::to_string(&message)?;
        self.writer
            .send(WriteMessage::Line(serialized))
            .map_err(|_| McpError::ChannelClosed)
    }

    pub(super) async fn shutdown(&self) -> Result<(), McpError> {
        if let Ok((_, rx)) = self.request(METHOD_SHUTDOWN, Value::Null).await {
            let _ = recv_with_timeout(rx, Duration::from_secs(5)).await;
        }

        let exit_message = serde_json::json!({
            "jsonrpc": "2.0",
            "method": METHOD_EXIT,
            "params": Value::Null
        });

        let _ = self.writer.send(WriteMessage::Line(
            serde_json::to_string(&exit_message).unwrap_or_default(),
        ));

        Ok(())
    }

    pub(super) async fn shutdown_app(&self) -> Result<std::process::ExitStatus, McpError> {
        debug_assert!(self.is_app);
        self.invalidate_interactions();
        // The pinned app-server has no shutdown/exit RPC. EOF shuts down stdio.
        let _ = self.writer.send(WriteMessage::Close);
        let mut guard = self.child.lock().await;
        let child = guard.as_mut().ok_or(McpError::ChannelClosed)?;
        let status = match time::timeout(Duration::from_secs(2), child.wait()).await {
            Ok(result) => result,
            Err(_) => {
                let _ = child.start_kill();
                child.wait().await
            }
        }
        .map_err(|_| McpError::Transport("app-server process wait failed".into()))?;
        Ok(status)
    }

    fn invalidate_interactions(&self) {
        if let NotificationHook::App {
            interactions: Some(delivery),
            ..
        } = &self.notification_hook
        {
            delivery.state.close();
        }
    }

    pub(super) fn startup_timeout(&self) -> Duration {
        self.startup_timeout
    }
}

impl Drop for JsonRpcTransport {
    fn drop(&mut self) {
        self.invalidate_interactions();
        for handle in &self.tasks {
            handle.abort();
        }

        if let Ok(mut child_guard) = self.child.try_lock() {
            if let Some(mut child) = child_guard.take() {
                let _ = child.start_kill();
            }
        }
    }
}

async fn writer_task(
    mut stdin: ChildStdin,
    mut rx: mpsc::UnboundedReceiver<WriteMessage>,
    hook: NotificationHook,
) {
    while let Some(command) = rx.recv().await {
        let (message, offset) = match command {
            WriteMessage::Line(line) => (line, 0),
            WriteMessage::Close => break,
            WriteMessage::Reply {
                state,
                token,
                result,
            } => {
                let Some(line) = state.render_reply(token, result) else {
                    continue;
                };
                let Some(mut active) = state.reply_watch(token) else {
                    continue;
                };
                let first = tokio::select! {
                    biased;
                    first = poll_fn(|cx| state.poll_reply(token, || std::pin::Pin::new(&mut stdin).poll_write(cx,line.as_bytes()))) => first,
                    _ = active.changed() => continue,
                };
                match first {
                    Ok(Some(n)) if n > 0 => (line, n),
                    Ok(None) => continue,
                    _ => {
                        state.fail("app-server write failed");
                        break;
                    }
                }
            }
        };
        if stdin
            .write_all(&message.as_bytes()[offset..])
            .await
            .is_err()
            || stdin.write_all(b"\n").await.is_err()
            || stdin.flush().await.is_err()
        {
            if let NotificationHook::App {
                interactions: Some(delivery),
                ..
            } = &hook
            {
                delivery.state.fail("app-server write failed");
            }
            break;
        }
    }
    if let NotificationHook::App {
        interactions: Some(delivery),
        ..
    } = hook
    {
        delivery.state.close();
    }
    let _ = stdin.shutdown().await;
}

async fn stderr_task(stderr: ChildStderr, mirror_stdio: bool, is_app: bool) {
    let mut lines = BufReader::new(stderr).lines();
    loop {
        let line = match lines.next_line().await {
            Ok(Some(line)) => line,
            Ok(None) => break,
            Err(err) => {
                warn!("failed to read MCP stderr: {err}");
                break;
            }
        };

        if is_app {
            continue;
        }
        if mirror_stdio {
            eprintln!("[mcp stderr] {line}");
        } else {
            debug!("mcp stderr: {line}");
        }
    }
}

pub(super) fn map_response<T: for<'a> Deserialize<'a> + Send + 'static>(
    rx: oneshot::Receiver<Result<Value, McpError>>,
) -> oneshot::Receiver<Result<T, McpError>> {
    let (tx, mapped_rx) = oneshot::channel();
    tokio::spawn(async move {
        let mapped = match rx.await {
            Ok(Ok(value)) => serde_json::from_value::<T>(value).map_err(McpError::from),
            Ok(Err(err)) => Err(err),
            Err(_) => Err(McpError::ChannelClosed),
        };
        let _ = tx.send(mapped);
    });
    mapped_rx
}

async fn recv_with_timeout(
    rx: oneshot::Receiver<Result<Value, McpError>>,
    timeout: Duration,
) -> Result<Value, McpError> {
    match time::timeout(timeout, rx).await {
        Ok(Ok(Ok(value))) => Ok(value),
        Ok(Ok(Err(err))) => Err(err),
        Ok(Err(_)) => Err(McpError::ChannelClosed),
        Err(_) => Err(McpError::Timeout(timeout)),
    }
}
