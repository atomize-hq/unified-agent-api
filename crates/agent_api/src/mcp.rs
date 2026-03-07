use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::time::Duration;

pub(crate) const CAPABILITY_MCP_LIST_V1: &str = "agent_api.tools.mcp.list.v1";
pub(crate) const CAPABILITY_MCP_GET_V1: &str = "agent_api.tools.mcp.get.v1";
pub(crate) const CAPABILITY_MCP_ADD_V1: &str = "agent_api.tools.mcp.add.v1";
pub(crate) const CAPABILITY_MCP_REMOVE_V1: &str = "agent_api.tools.mcp.remove.v1";

#[derive(Clone, Debug, Default)]
pub struct AgentWrapperMcpCommandContext {
    pub working_dir: Option<PathBuf>,
    pub timeout: Option<Duration>,
    pub env: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct AgentWrapperMcpCommandOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
}

#[derive(Clone, Debug, Default)]
pub struct AgentWrapperMcpListRequest {
    pub context: AgentWrapperMcpCommandContext,
}

#[derive(Clone, Debug)]
pub struct AgentWrapperMcpGetRequest {
    pub name: String,
    pub context: AgentWrapperMcpCommandContext,
}

#[derive(Clone, Debug)]
pub struct AgentWrapperMcpRemoveRequest {
    pub name: String,
    pub context: AgentWrapperMcpCommandContext,
}

#[derive(Clone, Debug)]
pub enum AgentWrapperMcpAddTransport {
    /// Launches an MCP server via stdio.
    Stdio {
        /// Command argv (MUST be non-empty).
        command: Vec<String>,
        /// Additional argv items appended after `command`.
        args: Vec<String>,
        /// Env vars injected into the MCP server process.
        env: BTreeMap<String, String>,
    },
    /// Connects to a streamable HTTP MCP server.
    Url {
        url: String,
        bearer_token_env_var: Option<String>,
    },
}

#[derive(Clone, Debug)]
pub struct AgentWrapperMcpAddRequest {
    pub name: String,
    pub transport: AgentWrapperMcpAddTransport,
    pub context: AgentWrapperMcpCommandContext,
}
