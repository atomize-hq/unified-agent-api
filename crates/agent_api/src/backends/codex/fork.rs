use std::{collections::BTreeMap, ffi::OsString, path::PathBuf, sync::Arc, time::Duration};

use serde_json::Value;
use tokio::sync::{mpsc, oneshot};

use crate::{
    backend_harness::{BackendSpawn, DynBackendCompletionFuture, DynBackendEventStream},
    backends::spawn_path::resolve_effective_working_dir,
    AgentWrapperError, AgentWrapperEvent, AgentWrapperEventKind, AgentWrapperKind,
    AgentWrapperRunRequest,
};

use super::{
    mapping::{error_event, status_event},
    CodexApprovalPolicy, CodexBackendCompletion, CodexBackendConfig, CodexBackendError,
    CodexBackendEvent, CodexHandleFacetState, CodexSandboxMode, CodexTerminationHandle,
    PINNED_APPROVAL_REQUIRED,
};

use super::super::{
    session_selectors::{parse_session_fork_v1, SessionSelectorV1, EXT_SESSION_FORK_V1},
    termination::{TerminationHandle, TerminationState},
};

#[derive(Clone, Debug)]
pub(super) struct ForkFlowRequest {
    pub(super) selector: SessionSelectorV1,
    pub(super) prompt: String,
    pub(super) working_dir: Option<PathBuf>,
    pub(super) effective_timeout: Option<Duration>,
    pub(super) env: BTreeMap<String, String>,
    pub(super) config: CodexBackendConfig,
    pub(super) run_start_cwd: Option<PathBuf>,
    pub(super) termination: Option<Arc<TerminationState<CodexTerminationHandle>>>,
    pub(super) non_interactive: bool,
    pub(super) external_sandbox: bool,
    pub(super) approval_policy: Option<CodexApprovalPolicy>,
    pub(super) sandbox_mode: CodexSandboxMode,
    pub(super) handle_state: Arc<std::sync::Mutex<CodexHandleFacetState>>,
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

#[derive(Clone)]
pub(super) struct AppServerTurnCancelHandle {
    cancel: mpsc::UnboundedSender<()>,
}

impl TerminationHandle for AppServerTurnCancelHandle {
    fn request_termination(&self) {
        let _ = self.cancel.send(());
    }
}

fn map_sandbox_mode(mode: &CodexSandboxMode) -> &'static str {
    match mode {
        CodexSandboxMode::ReadOnly => "read-only",
        CodexSandboxMode::WorkspaceWrite => "workspace-write",
        CodexSandboxMode::DangerFullAccess => "danger-full-access",
    }
}

fn map_approval_policy(policy: &CodexApprovalPolicy) -> &'static str {
    match policy {
        CodexApprovalPolicy::Untrusted => "untrusted",
        CodexApprovalPolicy::OnFailure => "on-failure",
        CodexApprovalPolicy::OnRequest => "on-request",
        CodexApprovalPolicy::Never => "never",
    }
}

fn effective_approval_policy(
    non_interactive: bool,
    approval_policy: Option<&CodexApprovalPolicy>,
) -> Option<String> {
    if non_interactive {
        return Some("never".to_string());
    }
    approval_policy.map(|policy| map_approval_policy(policy).to_string())
}

fn resolve_fork_effective_cwd(
    config: &CodexBackendConfig,
    run_start_cwd: Option<&PathBuf>,
    working_dir: Option<&PathBuf>,
) -> Result<PathBuf, CodexBackendError> {
    resolve_effective_working_dir(
        working_dir.map(PathBuf::as_path),
        config.default_working_dir.as_deref(),
        run_start_cwd.map(PathBuf::as_path),
    )
    .ok_or(CodexBackendError::WorkingDirectoryUnresolved)
}

fn install_thread_id(handle_state: &Arc<std::sync::Mutex<CodexHandleFacetState>>, thread_id: &str) {
    if thread_id.trim().is_empty() {
        return;
    }
    let Ok(mut state) = handle_state.lock() else {
        return;
    };
    if state.thread_id.is_none() {
        let len = thread_id.len();
        if len <= super::SESSION_HANDLE_ID_BOUND_BYTES {
            state.thread_id = Some(thread_id.to_string());
        } else if !state.oversize_warning_emitted {
            state.oversize_warning_emitted = true;
            state.oversize_warning_len_bytes = Some(len);
        }
    }
}

mod notifications;
mod turn;
pub(super) use notifications::{is_approval_request_notification, map_app_server_notification};

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
        effective_timeout,
        env,
        config,
        run_start_cwd,
        termination,
        non_interactive,
        external_sandbox,
        approval_policy,
        sandbox_mode,
        handle_state,
    } = req;

    let timeout_for_turn: Option<Duration> = match effective_timeout {
        Some(timeout) if timeout == Duration::ZERO => None,
        other => other,
    };

    let effective_cwd =
        resolve_fork_effective_cwd(&config, run_start_cwd.as_ref(), working_dir.as_ref())?;

    let server = codex::mcp::CodexAppServer::with_capabilities(
        codex::mcp::StdioServerConfig {
            binary: config
                .binary
                .clone()
                .unwrap_or_else(|| PathBuf::from("codex")),
            code_home: config.codex_home.clone(),
            current_dir: Some(effective_cwd.clone()),
            env: env
                .iter()
                .map(|(k, v)| (OsString::from(k), OsString::from(v)))
                .collect(),
            app_server_analytics_default_enabled: false,
            mirror_stdio: false,
            startup_timeout: Duration::from_secs(3),
        },
        codex::mcp::ClientInfo {
            name: "agent_api".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        serde_json::json!({"experimentalApi": true}),
    )
    .await
    .map_err(CodexBackendError::AppServer)?;
    let server = Arc::new(server);

    let setup = async {
        let source_thread_id = match selector {
            SessionSelectorV1::Id { id } => id,
            SessionSelectorV1::Last => server
                .select_last_thread_id(effective_cwd.clone())
                .await
                .map_err(CodexBackendError::AppServer)?
                .ok_or(CodexBackendError::ForkSelectionEmpty)?,
        };

        let (approval_policy, sandbox_mode) = if external_sandbox {
            (
                Some(CodexApprovalPolicy::Never),
                CodexSandboxMode::DangerFullAccess,
            )
        } else {
            (approval_policy, sandbox_mode)
        };

        let approval_policy = effective_approval_policy(non_interactive, approval_policy.as_ref());
        let sandbox = Some(map_sandbox_mode(&sandbox_mode).to_string());

        let forked = server
            .thread_fork(codex::mcp::ThreadForkParams {
                thread_id: source_thread_id,
                cwd: Some(effective_cwd.clone()),
                approval_policy: approval_policy.clone(),
                sandbox,
                persist_extended_history: None,
            })
            .await;

        let forked = match forked {
            Ok(forked) => forked,
            Err(codex::mcp::McpError::Rpc { message, .. })
                if super::is_not_found_signal(&message) =>
            {
                return Err(CodexBackendError::ForkSessionNotFound);
            }
            Err(err) => return Err(CodexBackendError::AppServer(err)),
        };

        if forked.thread.id.trim().is_empty() {
            return Err(CodexBackendError::AppServer(codex::mcp::McpError::Server(
                "invalid forked thread identity".into(),
            )));
        }
        install_thread_id(&handle_state, forked.thread.id.as_str());

        let turn = server
            .turn_start_v2(codex::mcp::TurnStartParamsV2 {
                thread_id: forked.thread.id.clone(),
                input: vec![codex::mcp::UserInputV2::text(prompt)],
                approval_policy,
                cwd: Some(effective_cwd),
            })
            .await
            .map_err(CodexBackendError::AppServer)?;

        Ok::<_, CodexBackendError>((forked.thread.id, turn))
    }
    .await;
    let (thread_id, turn) = match setup {
        Ok(value) => value,
        Err(err) => {
            let _ = server.shutdown_and_wait().await;
            return Err(err);
        }
    };
    let (cancel_tx, cancel_rx) = mpsc::unbounded_channel();
    if let Some(state) = termination.as_ref() {
        state.set_handle(CodexTerminationHandle::AppServerTurn(
            AppServerTurnCancelHandle { cancel: cancel_tx },
        ));
    }

    let (event_tx, event_rx) = mpsc::unbounded_channel::<CodexBackendEvent>();
    let (completion_tx, completion_rx) = oneshot::channel();
    tokio::spawn(async move {
        let result = turn::run_turn(
            server,
            thread_id,
            turn,
            timeout_for_turn,
            cancel_rx,
            &event_tx,
        )
        .await;
        let _ = completion_tx.send(result);
    });

    let events: DynBackendEventStream<CodexBackendEvent, CodexBackendError> = Box::pin(
        futures_util::stream::unfold(event_rx, |mut rx| async move {
            rx.recv().await.map(|event| (Ok(event), rx))
        }),
    );

    let completion: DynBackendCompletionFuture<CodexBackendCompletion, CodexBackendError> =
        Box::pin(async move {
            completion_rx
                .await
                .unwrap_or(Err(CodexBackendError::CompletionTaskDropped))
        });

    Ok(BackendSpawn {
        events,
        completion,
        events_observability: None,
    })
}
