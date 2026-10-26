//! DEF-01/AS-2 and DEF-02/AS-3: one owner and deadline, including startup.
use super::*;
use codex::mcp::{ClientInfo, CodexAppServer, McpError, StdioServerConfig};

pub(super) async fn run(
    req: ForkFlowRequest,
    cwd: PathBuf,
    mut cancel: mpsc::UnboundedReceiver<()>,
    events: &mpsc::UnboundedSender<CodexBackendEvent>,
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
    let server = tokio::select! {
        biased;
        Some(()) = cancel.recv() => return Err(cancelled()),
        _ = &mut expired => return Err(timed_out()),
        server = CodexAppServer::spawn_uninitialized(config) => server.map_err(CodexBackendError::AppServer)?,
    };
    let mut thread_id = None;
    let mut active_turn = None;
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
                SessionSelectorV1::Id { id } => id,
                SessionSelectorV1::Last => server
                    .select_last_thread_id(cwd.clone())
                    .await
                    .map_err(CodexBackendError::AppServer)?
                    .ok_or(CodexBackendError::ForkSelectionEmpty)?,
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
            let forked = server
                .thread_fork(codex::mcp::ThreadForkParams {
                    thread_id: source,
                    cwd: Some(cwd.clone()),
                    approval_policy: approval.clone(),
                    sandbox: Some(map_sandbox_mode(&sandbox).into()),
                    persist_extended_history: None,
                })
                .await
                .map_err(|err| match err {
                    McpError::Rpc { ref message, .. }
                        if super::super::is_not_found_signal(message) =>
                    {
                        CodexBackendError::ForkSessionNotFound
                    }
                    other => CodexBackendError::AppServer(other),
                })?;
            if forked.thread.id.trim().is_empty() {
                return Err(CodexBackendError::AppServer(McpError::Server(
                    "invalid forked thread identity".into(),
                )));
            }
            install_thread_id(&req.handle_state, &forked.thread.id);
            thread_id = Some(forked.thread.id.clone());
            let call = server
                .turn_start_v2(codex::mcp::TurnStartParamsV2 {
                    thread_id: forked.thread.id.clone(),
                    input: vec![codex::mcp::UserInputV2::text(req.prompt)],
                    approval_policy: approval,
                    cwd: Some(cwd),
                })
                .await
                .map_err(CodexBackendError::AppServer)?;
            super::turn::run_turn(forked.thread.id, call, &mut active_turn, events).await
        };
        tokio::pin!(execution);
        tokio::select! {
            biased;
            Some(()) = cancel.recv() => Err(cancelled()),
            _ = &mut expired => Err(timed_out()),
            result = &mut execution => result,
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
    outcome?;
    let status = status?;
    if !status.success() {
        return Err(CodexBackendError::AppServer(McpError::Server(
            "app-server exit failed".into(),
        )));
    }
    Ok(CodexBackendCompletion {
        status,
        final_text: None,
        backend_error_message: None,
        selection_failure_message: None,
    })
}
