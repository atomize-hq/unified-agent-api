use std::{
    collections::BTreeMap,
    ffi::OsString,
    path::PathBuf,
    process::ExitStatus,
    sync::{Arc, Mutex},
    time::Duration,
};

use futures_util::stream;
use tokio::sync::{mpsc, oneshot};

use crate::{backend_harness::BackendSpawn, AgentWrapperError, AgentWrapperRunRequest};

use super::super::session_selectors::{
    parse_session_fork_v1, SessionSelectorV1, EXT_SESSION_FORK_V1,
};

use super::{
    CodexApprovalPolicy, CodexBackendCompletion, CodexBackendConfig, CodexBackendError,
    CodexBackendEvent, CodexHandleFacetState, CodexSandboxMode,
};

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
    pub(super) non_interactive: bool,
    pub(super) approval_policy: Option<CodexApprovalPolicy>,
    pub(super) sandbox_mode: CodexSandboxMode,
    pub(super) handle_state: Arc<Mutex<CodexHandleFacetState>>,
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

    let source_thread_id = match selector {
        SessionSelectorV1::Last => server
            .select_last_thread_id(working_dir.clone())
            .await
            .map_err(CodexBackendError::AppServer)?
            .ok_or(CodexBackendError::ForkSelectionEmpty)?,
        SessionSelectorV1::Id { id } => id,
    };

    let approval_policy = app_server_approval_policy(non_interactive, approval_policy);
    let sandbox = Some(app_server_sandbox_mode(&sandbox_mode).to_string());

    let forked = server
        .thread_fork(codex::mcp::ThreadForkParams {
            thread_id: source_thread_id,
            cwd: Some(working_dir.clone()),
            approval_policy: approval_policy.clone(),
            sandbox,
            persist_extended_history: None,
        })
        .await
        .map_err(CodexBackendError::AppServer)?;

    let forked_thread_id = forked.thread.id;
    if forked_thread_id.trim().is_empty() {
        return Err(CodexBackendError::AppServer(codex::mcp::McpError::Server(
            "thread/fork returned empty thread id".to_string(),
        )));
    }

    if let Ok(mut state) = handle_state.lock() {
        state.thread_id = Some(forked_thread_id.clone());
    }

    let handle = server
        .turn_start_v2(codex::mcp::TurnStartParamsV2 {
            thread_id: forked_thread_id,
            input: vec![codex::mcp::UserInputV2::text(prompt)],
            approval_policy,
            cwd: Some(working_dir),
        })
        .await
        .map_err(CodexBackendError::AppServer)?;

    let codex::mcp::AppCallHandle {
        request_id: _turn_start_request_id,
        events: app_notifications,
        response,
    } = handle;

    let (events_tx, events_rx) =
        mpsc::unbounded_channel::<Result<CodexBackendEvent, CodexBackendError>>();
    let (stop_tx, stop_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        let mut stop_rx = stop_rx;
        let mut app_events = app_notifications;
        loop {
            tokio::select! {
                _ = &mut stop_rx => break,
                maybe = app_events.recv() => {
                    let Some(notification) = maybe else { break; };
                    let _ = events_tx.send(Ok(CodexBackendEvent::AppServerNotification(notification)));
                }
            }
        }
    });

    let completion = Box::pin(async move {
        let outcome = response.await
            .unwrap_or(Err(codex::mcp::McpError::ChannelClosed));

        let _ = stop_tx.send(());
        let _ = server.shutdown().await;

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

    approval_policy.map(|policy| match policy {
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
