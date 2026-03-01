use std::{
    collections::BTreeMap,
    ffi::OsString,
    path::PathBuf,
    process::ExitStatus,
    sync::atomic::{AtomicU64, Ordering},
    sync::{Arc, Mutex},
    time::Duration,
};

use futures_util::stream;
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};

use crate::{backend_harness::BackendSpawn, AgentWrapperError, AgentWrapperRunRequest};

use super::super::session_selectors::{
    parse_session_fork_v1, SessionSelectorV1, EXT_SESSION_FORK_V1,
};

use super::{
    CodexApprovalPolicy, CodexBackendCompletion, CodexBackendConfig, CodexBackendError,
    CodexBackendEvent, CodexHandleFacetState, CodexSandboxMode,
};

pub(super) struct AppServerTurnCancelHandle {
    server: Arc<codex::mcp::CodexAppServer>,
    request_id: Arc<AtomicU64>,
}

impl super::super::termination::TerminationHandle for AppServerTurnCancelHandle {
    fn request_termination(&self) {
        let request_id = self.request_id.load(Ordering::SeqCst);
        if request_id == 0 {
            return;
        }

        let _ = self.server.cancel(request_id);
    }
}

pub(super) fn extract_fork_selector_v1(
    request: &AgentWrapperRunRequest,
) -> Result<Option<SessionSelectorV1>, AgentWrapperError> {
    request
        .extensions
        .get(EXT_SESSION_FORK_V1)
        .map(parse_session_fork_v1)
        .transpose()
}

pub(super) struct ForkFlowRequest {
    pub(super) selector: SessionSelectorV1,
    pub(super) prompt: String,
    pub(super) working_dir: Option<PathBuf>,
    pub(super) env: BTreeMap<String, String>,
    pub(super) config: CodexBackendConfig,
    pub(super) run_start_cwd: Option<PathBuf>,
    pub(super) termination:
        Option<Arc<super::super::termination::TerminationState<super::CodexTerminationHandle>>>,
    pub(super) non_interactive: bool,
    pub(super) approval_policy: Option<CodexApprovalPolicy>,
    pub(super) sandbox_mode: CodexSandboxMode,
    pub(super) handle_state: Arc<Mutex<CodexHandleFacetState>>,
}

pub(super) fn map_app_server_notification(
    method: &str,
    params: &Value,
) -> Option<crate::AgentWrapperEvent> {
    match method {
        "agentMessage/delta" | "reasoning/text/delta" => {
            let delta = extract_text_delta(params)?;
            Some(crate::AgentWrapperEvent {
                agent_kind: crate::AgentWrapperKind("codex".to_string()),
                kind: crate::AgentWrapperEventKind::TextOutput,
                channel: Some("assistant".to_string()),
                text: Some(delta),
                message: None,
                data: None,
            })
        }
        "item/started" => Some(tool_event(
            crate::AgentWrapperEventKind::ToolCall,
            params,
            "start",
            "running",
        )),
        "item/completed" => Some(tool_event(
            crate::AgentWrapperEventKind::ToolResult,
            params,
            "complete",
            "completed",
        )),
        "turn/started" | "turn/completed" => Some(super::mapping::status_event(None)),
        "error" => Some(super::mapping::error_event(extract_error_message(params))),
        _ => None,
    }
}

pub(super) fn is_approval_request_notification(method: &str, params: &Value) -> bool {
    if method != "codex/event" {
        return false;
    }

    let payload = params.get("msg").unwrap_or(params);
    let Some(payload) = payload.as_object() else {
        return false;
    };

    let Some(event_type) = payload.get("type").and_then(Value::as_str) else {
        return false;
    };

    matches!(event_type, "approval_required" | "approval")
}

fn extract_text_delta(params: &Value) -> Option<String> {
    if let Some(delta) = params.as_str() {
        return Some(delta.to_string());
    }

    let obj = params.as_object()?;
    for key in ["delta", "text", "text_delta"] {
        if let Some(delta) = obj.get(key).and_then(Value::as_str) {
            return Some(delta.to_string());
        }
    }

    obj.get("content")
        .and_then(Value::as_object)
        .and_then(|content| content.get("text"))
        .and_then(Value::as_str)
        .map(|s| s.to_string())
}

fn extract_error_message(params: &Value) -> String {
    if let Some(message) = params
        .get("error")
        .and_then(|err| err.get("message"))
        .and_then(Value::as_str)
    {
        return message.to_string();
    }

    if let Some(message) = params.get("message").and_then(Value::as_str) {
        return message.to_string();
    }

    "codex app-server error".to_string()
}

fn tool_event(
    kind: crate::AgentWrapperEventKind,
    params: &Value,
    phase: &str,
    status: &str,
) -> crate::AgentWrapperEvent {
    let backend_item_id = params
        .get("item_id")
        .and_then(Value::as_str)
        .or_else(|| params.get("itemId").and_then(Value::as_str))
        .or_else(|| {
            params
                .get("item")
                .and_then(|item| item.get("id"))
                .and_then(Value::as_str)
        })
        .map(|s| s.to_string());
    let thread_id = params
        .get("thread_id")
        .and_then(Value::as_str)
        .or_else(|| params.get("threadId").and_then(Value::as_str))
        .map(|s| s.to_string());
    let turn_id = params
        .get("turn_id")
        .and_then(Value::as_str)
        .or_else(|| params.get("turnId").and_then(Value::as_str))
        .map(|s| s.to_string());

    let tool_kind = params
        .get("item_type")
        .and_then(Value::as_str)
        .or_else(|| params.get("itemType").and_then(Value::as_str))
        .or_else(|| {
            params
                .get("item")
                .and_then(|item| item.get("type").and_then(Value::as_str))
        })
        .or_else(|| {
            params
                .get("item")
                .and_then(|item| item.get("item_type").and_then(Value::as_str))
        })
        .unwrap_or("unknown")
        .to_string();

    crate::AgentWrapperEvent {
        agent_kind: crate::AgentWrapperKind("codex".to_string()),
        kind,
        channel: Some("tool".to_string()),
        text: None,
        message: None,
        data: Some(serde_json::json!({
            "schema": super::TOOLS_FACET_SCHEMA,
            "tool": {
                "backend_item_id": backend_item_id,
                "thread_id": thread_id,
                "turn_id": turn_id,
                "kind": tool_kind,
                "phase": phase,
                "status": status,
                "exit_code": null,
                "bytes": { "stdout": 0, "stderr": 0, "diff": 0, "result": 0 },
                "tool_name": null,
                "tool_use_id": null
            }
        })),
    }
}

pub(super) async fn spawn_fork_v1_flow(
    req: ForkFlowRequest,
) -> Result<
    BackendSpawn<CodexBackendEvent, CodexBackendCompletion, CodexBackendError>,
    CodexBackendError,
> {
    let ForkFlowRequest {
        selector,
        prompt,
        working_dir,
        env,
        config,
        run_start_cwd,
        termination,
        non_interactive,
        approval_policy,
        sandbox_mode,
        handle_state,
    } = req;

    let working_dir = resolve_effective_working_dir(
        working_dir,
        config.default_working_dir.clone(),
        run_start_cwd.clone(),
    )?;

    let binary = config
        .binary
        .clone()
        .or_else(|| std::env::var_os("CODEX_BINARY").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("codex"));
    let app_server_env: Vec<(OsString, OsString)> = env
        .into_iter()
        .map(|(k, v)| (OsString::from(k), OsString::from(v)))
        .collect();

    let server = codex::mcp::CodexAppServer::start_experimental(
        codex::mcp::StdioServerConfig {
            binary,
            code_home: config.codex_home.clone(),
            current_dir: Some(working_dir.clone()),
            env: app_server_env,
            app_server_analytics_default_enabled: false,
            mirror_stdio: false,
            startup_timeout: Duration::from_secs(5),
        },
        codex::mcp::ClientInfo {
            name: "agent_api".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    )
    .await
    .map_err(CodexBackendError::AppServer)?;

    let server = Arc::new(server);
    let turn_start_request_id = Arc::new(AtomicU64::new(0));

    if let Some(state) = termination.as_ref() {
        state.set_handle(super::CodexTerminationHandle::AppServerTurn(
            AppServerTurnCancelHandle {
                server: Arc::clone(&server),
                request_id: Arc::clone(&turn_start_request_id),
            },
        ));
    }

    let source_thread_id = match &selector {
        SessionSelectorV1::Last => {
            let selected = match server.select_last_thread_id(working_dir.clone()).await {
                Ok(selected) => selected,
                Err(err) => {
                    let _ = server.shutdown().await;
                    return Err(CodexBackendError::AppServer(err));
                }
            };
            match selected {
                Some(thread_id) => thread_id,
                None => {
                    let _ = server.shutdown().await;
                    return Err(CodexBackendError::ForkSelectionEmpty);
                }
            }
        }
        SessionSelectorV1::Id { id } => id.clone(),
    };

    let approval_policy = app_server_approval_policy(non_interactive, approval_policy);
    let sandbox = Some(app_server_sandbox_mode(&sandbox_mode).to_string());

    let forked = match server
        .thread_fork(codex::mcp::ThreadForkParams {
            thread_id: source_thread_id,
            cwd: Some(working_dir.clone()),
            approval_policy: approval_policy.clone(),
            sandbox,
            persist_extended_history: None,
        })
        .await
    {
        Ok(forked) => forked,
        Err(codex::mcp::McpError::Rpc { .. })
            if matches!(selector, SessionSelectorV1::Id { .. }) =>
        {
            let _ = server.shutdown().await;
            return Err(CodexBackendError::ForkSessionNotFound);
        }
        Err(err) => {
            let _ = server.shutdown().await;
            return Err(CodexBackendError::AppServer(err));
        }
    };

    let forked_thread_id = forked.thread.id;
    if forked_thread_id.trim().is_empty() {
        let _ = server.shutdown().await;
        return Err(CodexBackendError::AppServer(codex::mcp::McpError::Server(
            "thread/fork returned empty thread id".to_string(),
        )));
    }

    if let Ok(mut state) = handle_state.lock() {
        state.thread_id = Some(forked_thread_id.clone());
    }

    let handle = match server
        .turn_start_v2(codex::mcp::TurnStartParamsV2 {
            thread_id: forked_thread_id,
            input: vec![codex::mcp::UserInputV2::text(prompt)],
            approval_policy,
            cwd: Some(working_dir),
        })
        .await
    {
        Ok(handle) => handle,
        Err(err) => {
            let _ = server.shutdown().await;
            return Err(CodexBackendError::AppServer(err));
        }
    };

    let codex::mcp::AppCallHandle {
        request_id,
        events: app_notifications,
        response,
    } = handle;

    turn_start_request_id.store(request_id, Ordering::SeqCst);

    let (events_tx, events_rx) =
        mpsc::unbounded_channel::<Result<CodexBackendEvent, CodexBackendError>>();
    let (stop_tx, stop_rx) = oneshot::channel::<()>();
    let terminal_error = Arc::new(Mutex::new(None::<&'static str>));

    let server_for_events = Arc::clone(&server);
    let turn_start_request_id_for_events = Arc::clone(&turn_start_request_id);
    let terminal_error_for_events = Arc::clone(&terminal_error);

    tokio::spawn(async move {
        let mut stop_rx = stop_rx;
        let mut app_events = app_notifications;
        loop {
            tokio::select! {
                _ = &mut stop_rx => break,
                maybe = app_events.recv() => {
                    let Some(notification) = maybe else { break; };
                    if non_interactive {
                        if let codex::mcp::AppNotification::Raw { method, params } = &notification {
                            if is_approval_request_notification(method, params) {
                                let id = turn_start_request_id_for_events.load(Ordering::SeqCst);
                                if id != 0 {
                                    let _ = server_for_events.cancel(id);
                                }

                                if let Ok(mut guard) = terminal_error_for_events.lock() {
                                    if guard.is_none() {
                                        *guard = Some(super::PINNED_APPROVAL_REQUIRED);
                                    }
                                }

                                let _ = events_tx.send(Ok(CodexBackendEvent::TerminalError {
                                    message: super::PINNED_APPROVAL_REQUIRED.to_string(),
                                }));
                                break;
                            }
                        }
                    }
                    let _ = events_tx.send(Ok(CodexBackendEvent::AppServerNotification(notification)));
                }
            }
        }
    });

    let terminal_error_for_completion = Arc::clone(&terminal_error);
    let completion = Box::pin(async move {
        let outcome = response
            .await
            .unwrap_or(Err(codex::mcp::McpError::ChannelClosed));

        let _ = stop_tx.send(());
        let _ = server.shutdown().await;

        if let Some(message) = terminal_error_for_completion
            .lock()
            .ok()
            .and_then(|guard| *guard)
        {
            return Ok(CodexBackendCompletion {
                status: success_exit_status(),
                final_text: None,
                selection_failure_message: Some(message.to_string()),
            });
        }

        match outcome {
            Ok(_) => Ok(CodexBackendCompletion {
                status: success_exit_status(),
                final_text: None,
                selection_failure_message: None,
            }),
            Err(err) => Err(CodexBackendError::AppServer(err)),
        }
    });

    let events = Box::pin(stream::unfold(events_rx, |mut rx| async move {
        rx.recv().await.map(|item| (item, rx))
    }));

    Ok(BackendSpawn { events, completion })
}

fn resolve_effective_working_dir(
    request_working_dir: Option<PathBuf>,
    default_working_dir: Option<PathBuf>,
    run_start_cwd: Option<PathBuf>,
) -> Result<PathBuf, CodexBackendError> {
    let mut working_dir = request_working_dir
        .or(default_working_dir)
        .or_else(|| run_start_cwd.clone())
        .ok_or(CodexBackendError::WorkingDirectoryUnresolved)?;

    if working_dir.is_relative() {
        if let Some(run_start_cwd) = run_start_cwd {
            working_dir = run_start_cwd.join(working_dir);
        }
    }

    Ok(working_dir)
}

fn app_server_approval_policy(
    non_interactive: bool,
    approval_policy: Option<CodexApprovalPolicy>,
) -> Option<String> {
    if non_interactive {
        return Some("never".to_string());
    }

    approval_policy
        .map(|policy| match policy {
            CodexApprovalPolicy::Untrusted => "untrusted",
            CodexApprovalPolicy::OnFailure => "on-failure",
            CodexApprovalPolicy::OnRequest => "on-request",
            CodexApprovalPolicy::Never => "never",
        })
        .map(str::to_string)
}

fn app_server_sandbox_mode(mode: &CodexSandboxMode) -> &'static str {
    match mode {
        CodexSandboxMode::ReadOnly => "read-only",
        CodexSandboxMode::WorkspaceWrite => "workspace-write",
        CodexSandboxMode::DangerFullAccess => "danger-full-access",
    }
}

fn success_exit_status() -> ExitStatus {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(0)
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::ExitStatusExt;
        ExitStatus::from_raw(0)
    }
}
