use std::{future::Future, pin::Pin, sync::Arc};

use super::{
    harness::{new_harness_adapter, CodexTerminationHandle},
    mcp_management, CodexBackend,
};
use crate::{
    backend_harness::BackendDefaults,
    mcp::{
        normalize_mcp_add_request, normalize_mcp_get_request, normalize_mcp_remove_request,
        AgentWrapperMcpAddRequest, AgentWrapperMcpCommandOutput, AgentWrapperMcpGetRequest,
        AgentWrapperMcpListRequest, AgentWrapperMcpRemoveRequest, CAPABILITY_MCP_ADD_V1,
        CAPABILITY_MCP_GET_V1, CAPABILITY_MCP_LIST_V1, CAPABILITY_MCP_REMOVE_V1,
    },
    AgentWrapperBackend, AgentWrapperCapabilities, AgentWrapperError, AgentWrapperKind,
    AgentWrapperRunControl, AgentWrapperRunHandle, AgentWrapperRunRequest,
};

fn unsupported_capability<T>(
    agent_kind: String,
    capability: &'static str,
) -> Pin<Box<dyn Future<Output = Result<T, AgentWrapperError>> + Send + 'static>> {
    Box::pin(async move {
        Err(AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability: capability.to_string(),
        })
    })
}

impl AgentWrapperBackend for CodexBackend {
    fn kind(&self) -> AgentWrapperKind {
        AgentWrapperKind("codex".to_string())
    }

    fn capabilities(&self) -> AgentWrapperCapabilities {
        AgentWrapperCapabilities {
            ids: crate::runtime_advertising::evaluator::evaluate(
                &crate::runtime_advertising::data::CODEX,
                crate::runtime_advertising::host(),
                crate::runtime_advertising::evaluator::ConfigValues {
                    allow_mcp_write: self.config.allow_mcp_write,
                    allow_external_sandbox_exec: self.config.allow_external_sandbox_exec,
                },
            ),
        }
    }

    fn mcp_list(
        &self,
        request: AgentWrapperMcpListRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AgentWrapperMcpCommandOutput, AgentWrapperError>>
                + Send
                + '_,
        >,
    > {
        if !self.capabilities().contains(CAPABILITY_MCP_LIST_V1) {
            return unsupported_capability(
                self.kind().as_str().to_string(),
                CAPABILITY_MCP_LIST_V1,
            );
        }

        let config = self.config.clone();
        Box::pin(async move {
            mcp_management::run_codex_mcp(
                config,
                mcp_management::codex_mcp_list_argv(),
                request.context,
            )
            .await
        })
    }

    fn mcp_get(
        &self,
        request: AgentWrapperMcpGetRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AgentWrapperMcpCommandOutput, AgentWrapperError>>
                + Send
                + '_,
        >,
    > {
        if !self.capabilities().contains(CAPABILITY_MCP_GET_V1) {
            return unsupported_capability(self.kind().as_str().to_string(), CAPABILITY_MCP_GET_V1);
        }

        let request = match normalize_mcp_get_request(request) {
            Ok(request) => request,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let config = self.config.clone();
        let argv = mcp_management::codex_mcp_get_argv(&request.name);
        Box::pin(async move { mcp_management::run_codex_mcp(config, argv, request.context).await })
    }

    fn mcp_add(
        &self,
        request: AgentWrapperMcpAddRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AgentWrapperMcpCommandOutput, AgentWrapperError>>
                + Send
                + '_,
        >,
    > {
        if !self.capabilities().contains(CAPABILITY_MCP_ADD_V1) {
            return unsupported_capability(self.kind().as_str().to_string(), CAPABILITY_MCP_ADD_V1);
        }

        let request = match normalize_mcp_add_request(request) {
            Ok(request) => request,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let config = self.config.clone();
        let argv = mcp_management::codex_mcp_add_argv(&request.name, &request.transport);
        Box::pin(async move { mcp_management::run_codex_mcp(config, argv, request.context).await })
    }

    fn mcp_remove(
        &self,
        request: AgentWrapperMcpRemoveRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AgentWrapperMcpCommandOutput, AgentWrapperError>>
                + Send
                + '_,
        >,
    > {
        if !self.capabilities().contains(CAPABILITY_MCP_REMOVE_V1) {
            return unsupported_capability(
                self.kind().as_str().to_string(),
                CAPABILITY_MCP_REMOVE_V1,
            );
        }

        let request = match normalize_mcp_remove_request(request) {
            Ok(request) => request,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let config = self.config.clone();
        let argv = mcp_management::codex_mcp_remove_argv(&request.name);
        Box::pin(async move { mcp_management::run_codex_mcp(config, argv, request.context).await })
    }

    fn run(
        &self,
        request: AgentWrapperRunRequest,
    ) -> Pin<Box<dyn Future<Output = Result<AgentWrapperRunHandle, AgentWrapperError>> + Send + '_>>
    {
        let config = self.config.clone();
        let run_start_cwd = std::env::current_dir().ok();
        let interaction_handler = self.interaction_handler.clone();
        Box::pin(async move {
            let mut adapter = new_harness_adapter(config.clone(), run_start_cwd, None);
            adapter.interaction_handler = interaction_handler;
            let adapter = Arc::new(adapter);

            let defaults = BackendDefaults {
                env: config.env,
                default_timeout: config.default_timeout,
            };

            crate::backend_harness::run_harnessed_backend(adapter, defaults, request).await
        })
    }

    fn run_control(
        &self,
        request: AgentWrapperRunRequest,
    ) -> Pin<Box<dyn Future<Output = Result<AgentWrapperRunControl, AgentWrapperError>> + Send + '_>>
    {
        if !self
            .capabilities()
            .contains(crate::CAPABILITY_CONTROL_CANCEL_V1)
        {
            return unsupported_capability(
                self.kind().as_str().to_string(),
                crate::CAPABILITY_CONTROL_CANCEL_V1,
            );
        }

        let config = self.config.clone();
        let run_start_cwd = std::env::current_dir().ok();
        let interaction_handler = self.interaction_handler.clone();
        Box::pin(async move {
            let termination_state: Arc<
                super::super::termination::TerminationState<CodexTerminationHandle>,
            > = Arc::new(super::super::termination::TerminationState::new());
            let request_termination: Option<Arc<dyn Fn() + Send + Sync + 'static>> = Some({
                let termination_state = Arc::clone(&termination_state);
                Arc::new(move || termination_state.request())
            });

            let mut adapter =
                new_harness_adapter(config.clone(), run_start_cwd, Some(termination_state));
            adapter.interaction_handler = interaction_handler;
            let adapter = Arc::new(adapter);

            let defaults = BackendDefaults {
                env: config.env,
                default_timeout: config.default_timeout,
            };

            crate::backend_harness::run_harnessed_backend_control(
                adapter,
                defaults,
                request,
                request_termination,
            )
            .await
        })
    }
}
