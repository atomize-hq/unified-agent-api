use std::{collections::BTreeMap, path::PathBuf, time::Duration};

use crate::AiderCliClient;

#[derive(Clone, Debug)]
pub struct AiderCliClientBuilder {
    binary: PathBuf,
    env: BTreeMap<String, String>,
    timeout: Option<Duration>,
}

impl Default for AiderCliClientBuilder {
    fn default() -> Self {
        Self {
            binary: std::env::var_os("AIDER_BINARY")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("aider")),
            env: BTreeMap::new(),
            timeout: None,
        }
    }
}

impl AiderCliClientBuilder {
    pub fn binary(mut self, path: impl Into<PathBuf>) -> Self {
        self.binary = path.into();
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> AiderCliClient {
        AiderCliClient {
            binary: self.binary,
            env: self.env,
            timeout: self.timeout,
        }
    }
}
