#![forbid(unsafe_code)]

use std::{collections::BTreeMap, path::PathBuf, time::Duration};

const AGENT_KIND: &str = "aider";
const CHANNEL_ASSISTANT: &str = "assistant";
const CHANNEL_TOOL: &str = "tool";

mod backend;
mod harness;
mod mapping;

#[derive(Clone, Debug, Default)]
pub struct AiderBackendConfig {
    pub binary: Option<PathBuf>,
    pub default_timeout: Option<Duration>,
    pub env: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct AiderBackend {
    config: AiderBackendConfig,
}

impl AiderBackend {
    pub fn new(config: AiderBackendConfig) -> Self {
        Self { config }
    }
}
