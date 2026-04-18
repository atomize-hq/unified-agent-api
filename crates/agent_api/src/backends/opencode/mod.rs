#![forbid(unsafe_code)]

use std::{collections::BTreeMap, path::PathBuf, time::Duration};

const AGENT_KIND: &str = "opencode";

mod backend;
mod harness;
mod mapping;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Default)]
pub struct OpencodeBackendConfig {
    pub binary: Option<PathBuf>,
    pub default_timeout: Option<Duration>,
    pub env: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct OpencodeBackend {
    config: OpencodeBackendConfig,
}

impl OpencodeBackend {
    pub fn new(config: OpencodeBackendConfig) -> Self {
        Self { config }
    }
}
