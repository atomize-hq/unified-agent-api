use std::collections::BTreeSet;

use crate::agent_registry::AgentRegistryEntry;

pub const ALLOWED_CONFIG_KEYS: [&str; 2] = ["allow_external_sandbox_exec", "allow_mcp_write"];
pub const CAPABILITY_MCP_LIST_V1: &str = "agent_api.tools.mcp.list.v1";
pub const CAPABILITY_MCP_GET_V1: &str = "agent_api.tools.mcp.get.v1";
pub const CAPABILITY_MCP_ADD_V1: &str = "agent_api.tools.mcp.add.v1";
pub const CAPABILITY_MCP_REMOVE_V1: &str = "agent_api.tools.mcp.remove.v1";

#[derive(Debug, Clone, Copy)]
pub struct CapabilityManifestView<'a> {
    pub expected_targets: &'a [String],
    pub commands: &'a [CapabilityCommandView<'a>],
}

#[derive(Debug, Clone, Copy)]
pub struct CapabilityCommandView<'a> {
    pub path: &'a [String],
    pub available_on: &'a [String],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityPublicationTarget<'a> {
    DefaultBuiltInConfig,
    Explicit(&'a str),
}

impl CapabilityPublicationTarget<'_> {
    pub fn render_for_header(self, agent_id: &str) -> Option<String> {
        match self {
            Self::DefaultBuiltInConfig => None,
            Self::Explicit(target) => Some(format!("`{agent_id}={target}`")),
        }
    }
}

pub fn validate_config_key_allowlist(config_key: &str, field_name: &str) -> Result<(), String> {
    if ALLOWED_CONFIG_KEYS.contains(&config_key) {
        return Ok(());
    }

    Err(format!(
        "{field_name} must be one of `{}` (got `{config_key}`)",
        ALLOWED_CONFIG_KEYS.join("`, `")
    ))
}

pub fn requires_explicit_publication_target(
    capability_matrix_enabled: bool,
    has_target_gated_capabilities: bool,
    has_target_scoped_config_gated_capabilities: bool,
) -> bool {
    capability_matrix_enabled
        && (has_target_gated_capabilities || has_target_scoped_config_gated_capabilities)
}

pub fn resolve_capability_publication_target(
    entry: &AgentRegistryEntry,
) -> Result<CapabilityPublicationTarget<'_>, String> {
    let explicit_required = requires_explicit_publication_target(
        entry.publication.capability_matrix_enabled,
        !entry.capability_declaration.target_gated.is_empty(),
        entry
            .capability_declaration
            .config_gated
            .iter()
            .any(|gate| {
                gate.targets
                    .as_ref()
                    .is_some_and(|targets| !targets.is_empty())
            }),
    );

    match entry.publication.capability_matrix_target.as_deref() {
        Some(target) => {
            if !entry
                .canonical_targets
                .iter()
                .any(|candidate| candidate == target)
            {
                return Err(format!(
                    "publication.capability_matrix_target `{target}` must be listed in canonical_targets"
                ));
            }
            Ok(CapabilityPublicationTarget::Explicit(target))
        }
        None if explicit_required => Err(
            "publication.capability_matrix_target must be set when capability-matrix projection uses target-scoped declarations".to_string(),
        ),
        None => Ok(CapabilityPublicationTarget::DefaultBuiltInConfig),
    }
}

pub fn project_advertised_capabilities(
    entry: &AgentRegistryEntry,
    manifest: CapabilityManifestView<'_>,
) -> Result<BTreeSet<String>, String> {
    let publication_target = resolve_capability_publication_target(entry)?;
    let mut advertised = BTreeSet::new();
    advertised.extend(entry.capability_declaration.always_on.iter().cloned());
    advertised.extend(
        entry
            .capability_declaration
            .backend_extensions
            .iter()
            .cloned(),
    );

    let CapabilityPublicationTarget::Explicit(target) = publication_target else {
        return Ok(advertised);
    };

    if !manifest
        .expected_targets
        .iter()
        .any(|candidate| candidate == target)
    {
        return Err(format!(
            "manifest expected_targets is missing publication.capability_matrix_target `{target}`"
        ));
    }

    for gate in &entry.capability_declaration.target_gated {
        if !gate.targets.iter().any(|candidate| candidate == target) {
            continue;
        }

        if manifest_projected_capability_is_available(manifest, &gate.capability_id, target) {
            advertised.insert(gate.capability_id.clone());
        }
    }

    Ok(advertised)
}

pub fn manifest_projected_capability_path(capability_id: &str) -> Option<&'static [&'static str]> {
    match capability_id {
        CAPABILITY_MCP_LIST_V1 => Some(&["mcp", "list"]),
        CAPABILITY_MCP_GET_V1 => Some(&["mcp", "get"]),
        CAPABILITY_MCP_ADD_V1 => Some(&["mcp", "add"]),
        CAPABILITY_MCP_REMOVE_V1 => Some(&["mcp", "remove"]),
        _ => None,
    }
}

fn manifest_projected_capability_is_available(
    manifest: CapabilityManifestView<'_>,
    capability_id: &str,
    target: &str,
) -> bool {
    let Some(path) = manifest_projected_capability_path(capability_id) else {
        return true;
    };

    manifest.commands.iter().any(|command| {
        command
            .path
            .iter()
            .map(String::as_str)
            .eq(path.iter().copied())
            && command
                .available_on
                .iter()
                .any(|candidate| candidate == target)
    })
}
