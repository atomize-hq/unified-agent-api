use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use agent_api::AgentWrapperCapabilities;
use serde::Deserialize;

use crate::{
    agent_lifecycle::{
        self, approval_artifact_path_for_entry, lifecycle_state_path_for_entry,
        load_lifecycle_state, LifecycleStage,
    },
    agent_registry::{AgentRegistry, AgentRegistryEntry},
    approval_artifact::load_approval_artifact,
    capability_projection::{
        project_advertised_capabilities, resolve_capability_publication_target,
        CapabilityCommandView, CapabilityManifestView, CapabilityPublicationTarget,
    },
};

pub const CURRENT_MANIFEST_FILENAME: &str = "current.json";
pub const AGENT_API_ORTHOGONALITY_ALLOWLIST: [&str; 8] = [
    "agent_api.run",
    "agent_api.events",
    "agent_api.events.live",
    "agent_api.exec.non_interactive",
    "agent_api.tools.mcp.list.v1",
    "agent_api.tools.mcp.get.v1",
    "agent_api.tools.mcp.add.v1",
    "agent_api.tools.mcp.remove.v1",
];

#[derive(Debug, Clone)]
pub struct CapabilityPublicationInventory {
    pub backends: BTreeMap<String, AgentWrapperCapabilities>,
    pub canonical_target_header: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ManifestCurrent {
    #[serde(default)]
    pub expected_targets: Vec<String>,
    #[serde(default)]
    pub commands: Vec<ManifestCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ManifestCommand {
    pub path: Vec<String>,
    #[serde(default)]
    pub available_on: Vec<String>,
}

#[derive(Debug)]
struct PublicationContext {
    approval: crate::approval_artifact::ApprovalArtifact,
    lifecycle_state_path: String,
    lifecycle_stage: LifecycleStage,
    manifest: ManifestCurrent,
}

pub fn collect_capability_publication_inventory(
    workspace_root: &Path,
) -> Result<CapabilityPublicationInventory, String> {
    let registry = AgentRegistry::load(workspace_root).map_err(|err| err.to_string())?;
    let mut backends = BTreeMap::<String, AgentWrapperCapabilities>::new();
    let mut eligible_entries = Vec::<&AgentRegistryEntry>::new();

    for entry in registry.capability_matrix_entries() {
        let Some(context) = maybe_load_eligible_publication_context(workspace_root, entry)? else {
            continue;
        };
        let advertised = project_manifest_advertised_capabilities(entry, &context.manifest)?;
        validate_projected_capabilities_against_approval(entry, &context.approval, &advertised)?;
        backends.insert(entry.agent_id.clone(), AgentWrapperCapabilities { ids: advertised });
        eligible_entries.push(entry);
    }

    Ok(CapabilityPublicationInventory {
        backends,
        canonical_target_header: render_canonical_target_header(&eligible_entries)?,
    })
}

pub fn collect_published_backend_capabilities(
    workspace_root: &Path,
) -> Result<BTreeMap<String, AgentWrapperCapabilities>, String> {
    collect_capability_publication_inventory(workspace_root).map(|inventory| inventory.backends)
}

pub fn collect_agent_capabilities(
    workspace_root: &Path,
    agent_id: &str,
) -> Result<AgentWrapperCapabilities, String> {
    let registry = AgentRegistry::load(workspace_root).map_err(|err| err.to_string())?;
    let entry = registry.find(agent_id).ok_or_else(|| {
        format!(
            "capability publication registry entry `{agent_id}` is not present in {}",
            crate::agent_registry::REGISTRY_RELATIVE_PATH
        )
    })?;
    let context = load_required_publication_context(workspace_root, entry)?;
    let advertised = project_manifest_advertised_capabilities(entry, &context.manifest)?;
    validate_projected_capabilities_against_approval(entry, &context.approval, &advertised)?;
    Ok(AgentWrapperCapabilities { ids: advertised })
}

pub fn validate_agent_publication_continuity(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
) -> Result<(), String> {
    if !entry.publication.capability_matrix_enabled {
        return Ok(());
    }

    let context = load_required_publication_context(workspace_root, entry)?;
    if !is_publication_eligible_stage(context.lifecycle_stage) {
        return Err(format!(
            "capability publication for `{}` requires lifecycle stage `runtime_integrated`, `publication_ready`, `published`, or `closed_baseline` at `{}` (found `{}`)",
            entry.agent_id,
            context.lifecycle_state_path,
            context.lifecycle_stage.as_str()
        ));
    }

    let advertised = project_manifest_advertised_capabilities(entry, &context.manifest)?;
    validate_projected_capabilities_against_approval(entry, &context.approval, &advertised)
}

pub fn validate_registry_approval_alignment(
    approval: &crate::approval_artifact::ApprovalArtifact,
    entry: &AgentRegistryEntry,
) -> Result<(), String> {
    let descriptor = &approval.descriptor;
    let approval_path = approval_artifact_path_for_entry(entry);
    let publication_path = agent_lifecycle::publication_ready_path_for_entry(entry);

    let target_gated_capabilities = entry
        .capability_declaration
        .target_gated
        .iter()
        .map(|gate| format!("{}:{}", gate.capability_id, gate.targets.join(",")))
        .collect::<Vec<_>>();
    let config_gated_capabilities = entry
        .capability_declaration
        .config_gated
        .iter()
        .map(|gate| match gate.targets.as_ref() {
            Some(targets) if !targets.is_empty() => {
                format!(
                    "{}:{}:{}",
                    gate.capability_id,
                    gate.config_key,
                    targets.join(",")
                )
            }
            _ => format!("{}:{}", gate.capability_id, gate.config_key),
        })
        .collect::<Vec<_>>();

    let mut mismatches = Vec::<String>::new();
    push_mismatch(
        &mut mismatches,
        "crate_path",
        &descriptor.crate_path,
        &entry.crate_path,
    );
    push_mismatch(
        &mut mismatches,
        "backend_module",
        &descriptor.backend_module,
        &entry.backend_module,
    );
    push_mismatch(
        &mut mismatches,
        "manifest_root",
        &descriptor.manifest_root,
        &entry.manifest_root,
    );
    push_mismatch(
        &mut mismatches,
        "package_name",
        &descriptor.package_name,
        &entry.package_name,
    );
    push_mismatch(
        &mut mismatches,
        "wrapper_coverage_source_path",
        &descriptor.wrapper_coverage_source_path,
        &entry.wrapper_coverage.source_path,
    );
    push_mismatch(
        &mut mismatches,
        "approved_agent_path",
        &approval.relative_path,
        &approval_path,
    );
    push_mismatch(
        &mut mismatches,
        "publication_ready_path",
        &publication_path,
        &agent_lifecycle::publication_ready_path_for_entry(entry),
    );
    push_vec_mismatch(
        &mut mismatches,
        "canonical_targets",
        &descriptor.canonical_targets,
        &entry.canonical_targets,
    );
    push_vec_mismatch(
        &mut mismatches,
        "always_on_capabilities",
        &descriptor.always_on_capabilities,
        &entry.capability_declaration.always_on,
    );
    push_vec_mismatch(
        &mut mismatches,
        "target_gated_capabilities",
        &descriptor.target_gated_capabilities,
        &target_gated_capabilities,
    );
    push_vec_mismatch(
        &mut mismatches,
        "config_gated_capabilities",
        &descriptor.config_gated_capabilities,
        &config_gated_capabilities,
    );
    push_vec_mismatch(
        &mut mismatches,
        "backend_extensions",
        &descriptor.backend_extensions,
        &entry.capability_declaration.backend_extensions,
    );
    push_mismatch(
        &mut mismatches,
        "support_matrix_enabled",
        &descriptor.support_matrix_enabled.to_string(),
        &entry.publication.support_matrix_enabled.to_string(),
    );
    push_mismatch(
        &mut mismatches,
        "capability_matrix_enabled",
        &descriptor.capability_matrix_enabled.to_string(),
        &entry.publication.capability_matrix_enabled.to_string(),
    );
    push_option_mismatch(
        &mut mismatches,
        "capability_matrix_target",
        descriptor.capability_matrix_target.as_deref(),
        entry.publication.capability_matrix_target.as_deref(),
    );
    push_mismatch(
        &mut mismatches,
        "onboarding_pack_prefix",
        &descriptor.onboarding_pack_prefix,
        &entry.scaffold.onboarding_pack_prefix,
    );

    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "approval/registry mismatch for `{}`: {}",
            entry.agent_id,
            mismatches.join("; ")
        ))
    }
}

pub fn audit_capability_publication(
    backends: &BTreeMap<String, AgentWrapperCapabilities>,
) -> Result<(), String> {
    let all_capability_ids = backends
        .values()
        .flat_map(|caps| caps.ids.iter().cloned())
        .collect::<BTreeSet<_>>();

    let mut violations = Vec::<(String, Vec<String>)>::new();
    for capability_id in all_capability_ids {
        if !capability_id.starts_with("agent_api.") {
            continue;
        }
        if AGENT_API_ORTHOGONALITY_ALLOWLIST.contains(&capability_id.as_str()) {
            continue;
        }

        let supported_by = backends
            .iter()
            .filter_map(|(backend_id, capabilities)| {
                if capabilities.contains(&capability_id) {
                    Some(backend_id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if supported_by.len() < 2 {
            violations.push((capability_id, supported_by));
        }
    }

    if violations.is_empty() {
        return Ok(());
    }

    violations.sort_by(|a, b| a.0.cmp(&b.0));
    let published_agents = backends.keys().map(String::as_str).collect::<Vec<_>>();
    let mut allowlist_sorted = AGENT_API_ORTHOGONALITY_ALLOWLIST.to_vec();
    allowlist_sorted.sort();

    let mut out = String::new();
    out.push_str(&format!(
        "published agents: [{}]\n",
        published_agents.join(", ")
    ));
    out.push_str(&format!(
        "capability-matrix-audit failed: {} violation(s)\n",
        violations.len()
    ));
    for (capability_id, supported_by) in violations {
        out.push_str(&format!(
            "- {capability_id}: supported by {} agent(s): [{}]\n",
            supported_by.len(),
            supported_by.join(", ")
        ));
    }
    out.push_str(&format!("allowlist: [{}]\n", allowlist_sorted.join(", ")));
    Err(out)
}

pub fn audit_current_capability_publication(workspace_root: &Path) -> Result<(), String> {
    let backends = collect_published_backend_capabilities(workspace_root)?;
    audit_capability_publication(&backends)
}

pub fn manifest_command_available_on_target(
    manifest: &ManifestCurrent,
    path: &[&str],
    target: &str,
) -> bool {
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

pub fn project_manifest_advertised_capabilities(
    entry: &AgentRegistryEntry,
    manifest: &ManifestCurrent,
) -> Result<BTreeSet<String>, String> {
    let command_views = manifest
        .commands
        .iter()
        .map(|command| CapabilityCommandView {
            path: command.path.as_slice(),
            available_on: command.available_on.as_slice(),
        })
        .collect::<Vec<_>>();
    project_advertised_capabilities(
        entry,
        CapabilityManifestView {
            expected_targets: &manifest.expected_targets,
            commands: &command_views,
        },
    )
    .map_err(|err| {
        format!(
            "capability publication registry entry `{}` has invalid publication projection: {err}",
            entry.agent_id
        )
    })
}

pub fn validate_capability_publication_target(
    entry: &AgentRegistryEntry,
    manifest: &ManifestCurrent,
) -> Result<(), String> {
    match resolve_capability_publication_target(entry)? {
        CapabilityPublicationTarget::DefaultBuiltInConfig => Ok(()),
        CapabilityPublicationTarget::Explicit(target) => {
            if manifest
                .expected_targets
                .iter()
                .any(|candidate| candidate == target)
            {
                Ok(())
            } else {
                Err(format!(
                    "capability publication manifest `{}{}` is missing publication.capability_matrix_target `{target}` in expected_targets",
                    entry.manifest_root, "/current.json"
                ))
            }
        }
    }
}

pub fn render_publication_target_description(
    entry: &AgentRegistryEntry,
) -> Result<String, String> {
    Ok(match resolve_capability_publication_target(entry)? {
        CapabilityPublicationTarget::DefaultBuiltInConfig => {
            "the default publication target profile".to_string()
        }
        CapabilityPublicationTarget::Explicit(target) => {
            format!("publication.capability_matrix_target `{target}`")
        }
    })
}

pub fn render_canonical_target_header(entries: &[&AgentRegistryEntry]) -> Result<String, String> {
    if entries.is_empty() {
        return Ok(
            "Canonical publication target profile: none; no lifecycle-eligible agents are currently enrolled.\n"
                .to_string(),
        );
    }

    let mut canonical_targets = Vec::<String>::new();
    let mut default_backends = Vec::<String>::new();

    for entry in entries.iter().copied() {
        match resolve_capability_publication_target(entry)? {
            CapabilityPublicationTarget::Explicit(target) => {
                canonical_targets.push(format!("`{}={target}`", entry.agent_id));
            }
            CapabilityPublicationTarget::DefaultBuiltInConfig => {
                default_backends.push(format!("`{}`", entry.agent_id));
            }
        }
    }

    let mut out = String::from("Canonical publication target profile: ");
    if !canonical_targets.is_empty() {
        out.push_str(&canonical_targets.join(", "));
    }
    if !canonical_targets.is_empty() && !default_backends.is_empty() {
        out.push_str("; ");
    }
    if !default_backends.is_empty() {
        out.push_str(&default_backends.join(", "));
        out.push(' ');
        out.push_str(if default_backends.len() == 1 {
            "uses its"
        } else {
            "use their"
        });
        out.push_str(" default lifecycle-backed target profile");
    }
    out.push_str(".\n");
    Ok(out)
}

pub fn read_manifest_current(path: &Path) -> Result<ManifestCurrent, String> {
    let text = fs::read_to_string(path).map_err(|err| format!("read({path:?}): {err}"))?;
    serde_json::from_str(&text).map_err(|err| format!("parse({path:?}): {err}"))
}

fn maybe_load_eligible_publication_context(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
) -> Result<Option<PublicationContext>, String> {
    let lifecycle_state_path = lifecycle_state_path_for_entry(entry);
    if !workspace_root.join(&lifecycle_state_path).is_file() {
        return Ok(None);
    }

    let context = load_publication_context(workspace_root, entry, &lifecycle_state_path)?;
    if is_publication_eligible_stage(context.lifecycle_stage) {
        Ok(Some(context))
    } else {
        Ok(None)
    }
}

fn load_required_publication_context(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
) -> Result<PublicationContext, String> {
    let lifecycle_state_path = lifecycle_state_path_for_entry(entry);
    load_publication_context(workspace_root, entry, &lifecycle_state_path)
}

fn load_publication_context(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    lifecycle_state_path: &str,
) -> Result<PublicationContext, String> {
    let approval_path = approval_artifact_path_for_entry(entry);
    let approval = load_approval_artifact(workspace_root, &approval_path)
        .map_err(|err| format!("load approval artifact `{approval_path}`: {err}"))?;
    validate_registry_approval_alignment(&approval, entry)?;

    let lifecycle_state = load_lifecycle_state(workspace_root, lifecycle_state_path)
        .map_err(|err| format!("load lifecycle state `{lifecycle_state_path}`: {err}"))?;
    validate_lifecycle_approval_alignment(
        entry,
        lifecycle_state_path,
        &lifecycle_state,
        &approval,
    )?;

    let manifest = load_manifest_for_entry(workspace_root, entry)?;
    validate_manifest_targets(entry, &manifest)?;

    Ok(PublicationContext {
        approval,
        lifecycle_state_path: lifecycle_state_path.to_string(),
        lifecycle_stage: lifecycle_state.lifecycle_stage,
        manifest,
    })
}

fn load_manifest_for_entry(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
) -> Result<ManifestCurrent, String> {
    let manifest_path = workspace_root
        .join(&entry.manifest_root)
        .join(CURRENT_MANIFEST_FILENAME);
    read_manifest_current(&manifest_path)
}

fn validate_lifecycle_approval_alignment(
    entry: &AgentRegistryEntry,
    lifecycle_state_path: &str,
    lifecycle_state: &crate::agent_lifecycle::LifecycleState,
    approval: &crate::approval_artifact::ApprovalArtifact,
) -> Result<(), String> {
    if lifecycle_state.agent_id != entry.agent_id {
        return Err(format!(
            "`{lifecycle_state_path}` agent_id `{}` does not match registry entry `{}`",
            lifecycle_state.agent_id, entry.agent_id
        ));
    }
    if lifecycle_state.onboarding_pack_prefix != entry.scaffold.onboarding_pack_prefix {
        return Err(format!(
            "`{lifecycle_state_path}` onboarding_pack_prefix `{}` does not match registry entry `{}`",
            lifecycle_state.onboarding_pack_prefix, entry.scaffold.onboarding_pack_prefix
        ));
    }
    if lifecycle_state.approval_artifact_path != approval.relative_path {
        return Err(format!(
            "`{lifecycle_state_path}` approval_artifact_path `{}` does not match `{}`",
            lifecycle_state.approval_artifact_path, approval.relative_path
        ));
    }
    if lifecycle_state.approval_artifact_sha256 != approval.sha256 {
        return Err(format!(
            "`{lifecycle_state_path}` approval_artifact_sha256 does not match `{}`",
            approval.relative_path
        ));
    }
    Ok(())
}

fn validate_manifest_targets(
    entry: &AgentRegistryEntry,
    manifest: &ManifestCurrent,
) -> Result<(), String> {
    if manifest.expected_targets.is_empty() {
        return Err(format!(
            "capability publication manifest `{}{}` must declare at least one expected target",
            entry.manifest_root, "/current.json"
        ));
    }

    let missing_targets = entry
        .canonical_targets
        .iter()
        .filter(|target| {
            !manifest
                .expected_targets
                .iter()
                .any(|candidate| candidate == *target)
        })
        .cloned()
        .collect::<Vec<_>>();
    if !missing_targets.is_empty() {
        return Err(format!(
            "capability publication manifest `{}{}` is missing registry canonical target(s): {}",
            entry.manifest_root,
            "/current.json",
            missing_targets.join(", ")
        ));
    }

    validate_capability_publication_target(entry, manifest)
}

fn validate_projected_capabilities_against_approval(
    entry: &AgentRegistryEntry,
    approval: &crate::approval_artifact::ApprovalArtifact,
    advertised: &BTreeSet<String>,
) -> Result<(), String> {
    let approved = approval_capability_universe(approval);
    let unexpected = advertised
        .difference(&approved)
        .cloned()
        .collect::<Vec<_>>();
    if unexpected.is_empty() {
        return Ok(());
    }

    Err(format!(
        "capability publication registry entry `{}` advertises capabilities beyond approval truth on {}: {}",
        entry.agent_id,
        render_publication_target_description(entry)?,
        unexpected.join(", ")
    ))
}

fn approval_capability_universe(
    approval: &crate::approval_artifact::ApprovalArtifact,
) -> BTreeSet<String> {
    let descriptor = &approval.descriptor;
    descriptor
        .always_on_capabilities
        .iter()
        .cloned()
        .chain(
            descriptor
                .target_gated_capabilities
                .iter()
                .map(|value| gate_capability_id(value)),
        )
        .chain(
            descriptor
                .config_gated_capabilities
                .iter()
                .map(|value| gate_capability_id(value)),
        )
        .chain(descriptor.backend_extensions.iter().cloned())
        .collect()
}

fn gate_capability_id(value: &str) -> String {
    value
        .split_once(':')
        .map(|(capability_id, _)| capability_id.to_string())
        .unwrap_or_else(|| value.to_string())
}

fn is_publication_eligible_stage(stage: LifecycleStage) -> bool {
    matches!(
        stage,
        LifecycleStage::RuntimeIntegrated
            | LifecycleStage::PublicationReady
            | LifecycleStage::Published
            | LifecycleStage::ClosedBaseline
    )
}

fn push_mismatch(mismatches: &mut Vec<String>, field: &str, expected: &str, actual: &str) {
    if expected != actual {
        mismatches.push(format!("{field}: approval=`{expected}` registry=`{actual}`"));
    }
}

fn push_vec_mismatch(
    mismatches: &mut Vec<String>,
    field: &str,
    expected: &[String],
    actual: &[String],
) {
    if expected != actual {
        mismatches.push(format!(
            "{field}: approval=[{}] registry=[{}]",
            expected.join(", "),
            actual.join(", ")
        ));
    }
}

fn push_option_mismatch(
    mismatches: &mut Vec<String>,
    field: &str,
    expected: Option<&str>,
    actual: Option<&str>,
) {
    if expected != actual {
        mismatches.push(format!(
            "{field}: approval={} registry={}",
            expected.unwrap_or("<none>"),
            actual.unwrap_or("<none>")
        ));
    }
}
