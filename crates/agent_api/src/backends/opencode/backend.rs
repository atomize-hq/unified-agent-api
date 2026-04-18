use std::{future::Future, pin::Pin, sync::Arc};

use crate::{
    backend_harness::{run_harnessed_backend, BackendDefaults},
    AgentWrapperBackend, AgentWrapperCapabilities, AgentWrapperError, AgentWrapperKind,
    AgentWrapperRunHandle, AgentWrapperRunRequest,
};

use super::OpencodeBackend;

impl AgentWrapperBackend for OpencodeBackend {
    fn kind(&self) -> AgentWrapperKind {
        AgentWrapperKind(super::AGENT_KIND.to_string())
    }

    fn capabilities(&self) -> AgentWrapperCapabilities {
        AgentWrapperCapabilities::default()
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
