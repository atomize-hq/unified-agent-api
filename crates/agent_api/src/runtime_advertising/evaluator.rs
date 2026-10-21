//! Revision 1's complete, private runtime advertising semantics. Also compiled by xtask.
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ConfigKey {
    AllowMcpWrite,
    AllowExternalSandboxExec,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ConfigValues {
    pub(crate) allow_mcp_write: bool,
    pub(crate) allow_external_sandbox_exec: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct HostFacts<'a> {
    pub(crate) os: &'a str,
    pub(crate) arch: &'a str,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Gate<'a> {
    pub(crate) id: &'a str,
    pub(crate) hosts: Option<&'a [HostFacts<'a>]>,
    pub(crate) config: Option<ConfigKey>,
}

#[derive(Debug)]
pub(crate) struct BackendModel<'a> {
    pub(crate) agent_id: &'a str,
    pub(crate) unconditional: &'a [&'a str],
    pub(crate) gates: &'a [Gate<'a>],
}

pub(crate) fn evaluate(
    model: &BackendModel<'_>,
    host: HostFacts<'_>,
    config: ConfigValues,
) -> BTreeSet<String> {
    let mut ids: BTreeSet<String> = model
        .unconditional
        .iter()
        .map(|id| (*id).to_owned())
        .collect();
    for gate in model.gates {
        let host_matches = gate.hosts.map_or(true, |hosts| {
            hosts.iter().any(|h| h.os == host.os && h.arch == host.arch)
        });
        let config_matches = match gate.config {
            None => true,
            Some(ConfigKey::AllowMcpWrite) => config.allow_mcp_write,
            Some(ConfigKey::AllowExternalSandboxExec) => config.allow_external_sandbox_exec,
        };
        if host_matches && config_matches {
            ids.insert(gate.id.to_owned());
        }
    }
    ids
}
