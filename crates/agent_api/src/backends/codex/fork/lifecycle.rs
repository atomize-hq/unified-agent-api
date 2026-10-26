//! DEF-01/AS-2 and DEF-02/AS-3: one owner and deadline, including startup.
use std::{path::PathBuf, time::Duration};

use tokio::sync::mpsc;

use super::super::{
    CodexApprovalPolicy, CodexBackendCompletion, CodexBackendError, CodexBackendEvent,
    CodexSandboxMode, SessionSelectorV1,
};
use super::{
    effective_approval_policy, install_thread_id, map_sandbox_mode, ForkFlowRequest, OsString,
};
use codex::mcp::{ClientInfo, CodexAppServer, McpError, StdioServerConfig};

pub(super) async fn run(
    req: ForkFlowRequest,
    cwd: PathBuf,
    mut cancel: mpsc::UnboundedReceiver<()>,
    events: &mpsc::Sender<CodexBackendEvent>,
) -> Result<CodexBackendCompletion, CodexBackendError> {
    let timeout = req.effective_timeout.filter(|duration| !duration.is_zero());
    // Captured by the adapter before the public harness returns a handle. Time spent
    // waiting for this owned task to be polled is part of the same budget.
    let deadline = timeout.and_then(|duration| req.started_at.checked_add(duration));
    let expired = async {
        match deadline {
            Some(deadline) => tokio::time::sleep_until(deadline).await,
            None => std::future::pending().await,
        }
    };
    tokio::pin!(expired);
    let cancelled = || CodexBackendError::AppServer(McpError::Cancelled);
    let timed_out = || CodexBackendError::Timeout {
        timeout: timeout.unwrap_or_default(),
    };
    let config = StdioServerConfig {
        binary: req
            .config
            .binary
            .clone()
            .unwrap_or_else(|| PathBuf::from("codex")),
        code_home: req.config.codex_home.clone(),
        current_dir: Some(cwd.clone()),
        env: req
            .env
            .iter()
            .map(|(k, v)| (OsString::from(k), OsString::from(v)))
            .collect(),
        app_server_analytics_default_enabled: false,
        mirror_stdio: false,
        startup_timeout: Duration::from_secs(3),
    };
    // The spawn future has no suspension after the child is created. Cancelling a
    // busy-executable retry has no child; otherwise we retain the returned owner.
    let (server, messages) = tokio::select! {
        biased;
        Some(()) = cancel.recv() => return Err(cancelled()),
        _ = &mut expired => return Err(timed_out()),
        server = CodexAppServer::spawn_uninitialized_with_interactions_and_config(config,[("sandbox_mode".into(),serde_json::Value::String(map_sandbox_mode(if req.external_sandbox { &CodexSandboxMode::DangerFullAccess } else { &req.sandbox_mode }).into()))].into_iter().collect()) => server.map_err(CodexBackendError::AppServer)?,
    };
    let mut thread_id = None;
    let mut active_turn = None;
    let (scope_tx, scope_rx) = mpsc::unbounded_channel();
    let mut output = super::broker::Output::new(events.clone());
    let outcome = {
        let execution = async {
            server
                .initialize(
                    ClientInfo {
                        name: "agent_api".into(),
                        version: env!("CARGO_PKG_VERSION").into(),
                    },
                    serde_json::json!({"experimentalApi":true}),
                )
                .await
                .map_err(CodexBackendError::AppServer)?;
            let source = match req.selector {
                Some(SessionSelectorV1::Id { id }) => Some(id),
                Some(SessionSelectorV1::Last) => Some(
                    server
                        .select_last_thread_id(cwd.clone())
                        .await
                        .map_err(CodexBackendError::AppServer)?
                        .ok_or(CodexBackendError::ForkSelectionEmpty)?,
                ),
                None => None,
            };
            let (approval, sandbox) = if req.external_sandbox {
                (
                    Some(CodexApprovalPolicy::Never),
                    CodexSandboxMode::DangerFullAccess,
                )
            } else {
                (req.approval_policy, req.sandbox_mode)
            };
            let approval = effective_approval_policy(req.non_interactive, approval.as_ref());
            let roots = if req.is_fork {
                None
            } else {
                super::native_config::workspace_roots(&server, &cwd, &req.add_dirs).await?
            };
            let model = if req.is_fork {
                None
            } else {
                req.model_id
                    .or(req.config.model)
                    .filter(|model| !model.trim().is_empty())
            };
            let settings = if model
                .as_deref()
                .is_some_and(|model| model.to_ascii_lowercase().starts_with("gpt-5"))
            {
                [
                    ("model_reasoning_effort", "medium"),
                    ("model_reasoning_summary", "auto"),
                    ("model_verbosity", "low"),
                ]
                .into_iter()
                .map(|(key, value)| (key.into(), serde_json::Value::String(value.into())))
                .collect()
            } else {
                std::collections::BTreeMap::new()
            };
            let call = if req.is_fork {
                server
                    .thread_fork_v2(codex::mcp::ThreadForkParamsV2 {
                        thread_id: source.expect("fork source"),
                        cwd: Some(cwd.clone()),
                        model: model.clone(),
                        approval_policy: approval.clone(),
                        sandbox: Some(map_sandbox_mode(&sandbox).into()),
                        config: settings,
                        persist_extended_history: None,
                        runtime_workspace_roots: roots,
                    })
                    .await
            } else if let Some(source) = source {
                server
                    .thread_resume_v2(codex::mcp::ThreadResumeParamsV2 {
                        thread_id: source,
                        cwd: Some(cwd.clone()),
                        model: model.clone(),
                        approval_policy: approval.clone(),
                        sandbox: Some(map_sandbox_mode(&sandbox).into()),
                        config: settings,
                        persist_extended_history: None,
                        runtime_workspace_roots: roots,
                    })
                    .await
            } else {
                server
                    .thread_start_v2(codex::mcp::ThreadStartParamsV2 {
                        cwd: Some(cwd.clone()),
                        model: model.clone(),
                        approval_policy: approval.clone(),
                        sandbox: Some(map_sandbox_mode(&sandbox).into()),
                        config: settings,
                        persist_extended_history: None,
                        runtime_workspace_roots: roots,
                    })
                    .await
            }
            .map_err(CodexBackendError::AppServer)?;
            let response = reply(call)
                .await
                .map_err(|err| native_policy_error(err, &approval))
                .map_err(|err| match err {
                    CodexBackendError::AppServer(McpError::Rpc { ref message, .. })
                        if super::super::is_not_found_signal(message) =>
                    {
                        CodexBackendError::ForkSessionNotFound
                    }
                    other => other,
                })?;
            let selected = response
                .get("thread")
                .and_then(|thread| thread.get("id"))
                .and_then(serde_json::Value::as_str)
                .filter(|id| !id.trim().is_empty())
                .ok_or_else(|| {
                    CodexBackendError::AppServer(McpError::Server("invalid thread identity".into()))
                })?
                .to_string();
            install_thread_id(&req.handle_state, &selected);
            thread_id = Some(selected.clone());
            let _ = scope_tx.send(super::broker::Scope::Thread(selected.clone()));
            let call = server
                .turn_start_v3(codex::mcp::TurnStartParamsV3 {
                    thread_id: selected,
                    input: vec![codex::mcp::UserInputV2::text(req.prompt)],
                    approval_policy: approval.clone(),
                    cwd: Some(cwd),
                    model,
                    ..Default::default()
                })
                .await
                .map_err(CodexBackendError::AppServer)?;
            let accepted = reply(call)
                .await
                .map_err(|err| native_policy_error(err, &approval))?;
            let turn = accepted.get("turn").ok_or_else(|| {
                CodexBackendError::AppServer(McpError::Server("invalid turn lifecycle".into()))
            })?;
            let id = turn
                .get("id")
                .and_then(serde_json::Value::as_str)
                .filter(|id| !id.trim().is_empty())
                .ok_or_else(|| {
                    CodexBackendError::AppServer(McpError::Server("invalid turn lifecycle".into()))
                })?
                .to_string();
            if turn
                .get("items")
                .and_then(serde_json::Value::as_array)
                .is_none()
                || !matches!(
                    turn.get("status").and_then(serde_json::Value::as_str),
                    Some("inProgress" | "completed" | "failed" | "interrupted")
                )
            {
                return Err(CodexBackendError::AppServer(McpError::Server(
                    "invalid turn lifecycle".into(),
                )));
            }
            active_turn = Some(id.clone());
            let _ = scope_tx.send(super::broker::Scope::Turn(id));
            std::future::pending::<Result<Option<String>, CodexBackendError>>().await
        };
        let handler = if req.non_interactive {
            None
        } else {
            req.interaction_handler
        };
        let interactions = super::broker::run(messages, scope_rx, handler, &mut output);
        tokio::pin!(execution, interactions);
        tokio::select! {
            biased;
            Some(())=cancel.recv()=>Err(cancelled()),
            _=&mut expired=>Err(timed_out()),
            result=&mut execution=>result,
            result=&mut interactions=>result,
        }
    };
    if outcome.is_err() {
        if let (Some(thread_id), Some(turn_id)) = (thread_id.as_deref(), active_turn.as_deref()) {
            super::turn::interrupt(&server, thread_id, turn_id).await;
        }
    }
    // Even cancellation during initialize/list/fork reaches explicit EOF + wait.
    // This retained process owner outlives the cancelled RPC future.
    let status = server
        .shutdown_and_wait()
        .await
        .map_err(CodexBackendError::AppServer);
    output.flush().await;
    let final_text = outcome?;
    let status = status?;
    if !status.success() {
        return Err(CodexBackendError::AppServer(McpError::Server(
            "app-server exit failed".into(),
        )));
    }
    Ok(CodexBackendCompletion {
        status,
        final_text,
        backend_error_message: None,
        selection_failure_message: None,
    })
}

pub(super) async fn reply(
    mut call: codex::mcp::AppCallHandle,
) -> Result<serde_json::Value, CodexBackendError> {
    tokio::time::timeout(Duration::from_secs(3), &mut call.response)
        .await
        .map_err(|_| CodexBackendError::AppServer(McpError::Timeout(Duration::from_secs(3))))?
        .map_err(|_| CodexBackendError::AppServer(McpError::ChannelClosed))?
        .map_err(CodexBackendError::AppServer)
}

fn native_policy_error(
    error: CodexBackendError,
    approval: &Option<serde_json::Value>,
) -> CodexBackendError {
    if approval.as_ref().and_then(serde_json::Value::as_str) == Some("on-failure") {
        if let CodexBackendError::AppServer(McpError::Rpc { message, .. }) = &error {
            if message.contains("unknown variant") {
                return CodexBackendError::AppServer(McpError::Server(
                    "native approval policy unsupported".into(),
                ));
            }
        }
    }
    error
}
