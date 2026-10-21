use std::{future::Future, pin::Pin, sync::Arc};

use crate::{
    backend_harness::{run_harnessed_backend, BackendDefaults},
    AgentWrapperBackend, AgentWrapperCapabilities, AgentWrapperError, AgentWrapperKind,
    AgentWrapperRunHandle, AgentWrapperRunRequest,
};

use super::AiderBackend;

impl AgentWrapperBackend for AiderBackend {
    fn kind(&self) -> AgentWrapperKind {
        AgentWrapperKind(super::AGENT_KIND.to_string())
    }

    fn capabilities(&self) -> AgentWrapperCapabilities {
        AgentWrapperCapabilities {
            ids: crate::runtime_advertising::evaluator::evaluate(
                &crate::runtime_advertising::data::AIDER,
                crate::runtime_advertising::host(),
                crate::runtime_advertising::evaluator::ConfigValues::default(),
            ),
        }
    }

    fn run(
        &self,
        request: AgentWrapperRunRequest,
    ) -> Pin<Box<dyn Future<Output = Result<AgentWrapperRunHandle, AgentWrapperError>> + Send + '_>>
    {
        let adapter = Arc::new(self.clone());
        let defaults = BackendDefaults {
            env: self.config.env.clone(),
            default_timeout: self.config.default_timeout,
        };

        Box::pin(async move { run_harnessed_backend(adapter, defaults, request).await })
    }
}
