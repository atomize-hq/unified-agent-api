//! Preserve the dry-run reader's declaration requirement around the shared finite projection.
use super::*;

pub(super) fn read(
    raw: &[u8],
    path: &str,
    captured: &CapturedRegistry,
) -> Result<OnboardingApproval, String> {
    let document: Value = toml_edit::de::from_str(text(raw)?).map_err(|e| e.to_string())?;
    let approval = captured.onboarding_approval(raw, path)?;
    if approval.support_depth_version.is_none() {
        return Err("charter rule 3 unresolved: approval states no depth version".into());
    }
    let entry = captured
        .registry()
        .find(&approval.agent_id)
        .ok_or("unknown approval agent")?;
    if entry.support_depth.is_none() {
        return Err(
            "charter rule 3 unresolved: no enabled onboarding enrollment for the approved version"
                .into(),
        );
    }
    if entry.backend_module.is_some() {
        return Err(
            "private approval reader requires the compiled non-runtime depth subject".into(),
        );
    }
    crate::isolated_maintenance::onboarding::approval::validate(document, approval, entry)
}

/// An internal execution view after strict finite validation, never a production loader.
pub(crate) fn execution(
    workspace: &Path,
    raw: &[u8],
    path: &str,
    captured: &CapturedRegistry,
) -> Result<crate::approval_artifact::ApprovalArtifact, String> {
    let approval = read(raw, path, captured)?;
    let entry = captured
        .registry()
        .find(&approval.agent_id)
        .ok_or("unknown approval agent")?;
    execution_for_entry(workspace, raw, path, entry)
}

pub(super) fn execution_for_entry(
    workspace: &Path,
    raw: &[u8],
    path: &str,
    entry: &crate::agent_registry::AgentRegistryEntry,
) -> Result<crate::approval_artifact::ApprovalArtifact, String> {
    use crate::approval_artifact::{ApprovalArtifact, ApprovalDescriptor};
    use sha2::{Digest, Sha256};
    let approval = crate::isolated_maintenance::onboarding::approval::read(raw, path, entry)?;
    let descriptor = ApprovalDescriptor {
        agent_id: entry.agent_id.clone(),
        display_name: entry.display_name.clone(),
        crate_path: entry.crate_path.clone(),
        backend_module: String::new(),
        manifest_root: entry.manifest_root.clone(),
        package_name: entry.package_name.clone(),
        canonical_targets: entry.canonical_targets.clone(),
        wrapper_coverage_binding_kind: entry.wrapper_coverage.binding_kind.clone(),
        wrapper_coverage_source_path: entry.wrapper_coverage.source_path.clone(),
        always_on_capabilities: entry.capability_declaration.always_on.clone(),
        target_gated_capabilities: vec![],
        config_gated_capabilities: vec![],
        backend_extensions: entry.capability_declaration.backend_extensions.clone(),
        support_matrix_enabled: entry.publication.support_matrix_enabled,
        capability_matrix_enabled: entry.publication.capability_matrix_enabled,
        capability_matrix_target: entry.publication.capability_matrix_target.clone(),
        docs_release_track: entry.release.docs_release_track.clone(),
        onboarding_pack_prefix: entry.scaffold.onboarding_pack_prefix.clone(),
    };
    Ok(ApprovalArtifact {
        relative_path: path.into(),
        canonical_path: workspace.join(path),
        sha256: hex::encode(Sha256::digest(raw)),
        descriptor,
        maintenance: crate::approval_artifact::parse_approval_maintenance(text(raw)?, path)
            .map_err(|e| e.to_string())?,
        support_depth_version: approval.support_depth_version,
    })
}
