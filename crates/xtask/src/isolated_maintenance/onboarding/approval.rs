//! Strict finite approval projection. Never a production execution loader.
use crate::{agent_registry::AgentRegistryEntry, depth_record::onboarding::OnboardingApproval};
use serde_json::json;
use serde_json::Value;

// Callers supply the authenticated adapter's expected entry. Comparing the whole descriptor
// rejects every extra/misplaced key, including backend_module and capabilities.
pub(crate) fn read(
    raw: &[u8],
    path: &str,
    entry: &AgentRegistryEntry,
) -> Result<OnboardingApproval, String> {
    let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
    let document: Value = toml_edit::de::from_str(text).map_err(|e| e.to_string())?;
    let approval = OnboardingApproval::parse(text, path)?;
    validate(document, approval, entry)
}

pub(crate) fn validate(
    document: Value,
    approval: OnboardingApproval,
    entry: &AgentRegistryEntry,
) -> Result<OnboardingApproval, String> {
    if approval.support_depth_version.is_none() {
        return Err("charter rule 3 unresolved: approval states no depth version".into());
    }
    if let Some(why) = approval.attribution_mismatch(
        &entry.agent_id,
        &entry.manifest_root,
        &entry.scaffold.onboarding_pack_prefix,
    ) {
        return Err(why);
    }
    let mut expected = descriptor(entry);
    let maintenance = &document["descriptor"]["maintenance"];
    match maintenance["mode"].as_str() {
        Some("explicitly_deferred") => {
            let deferral = &maintenance["deferral"];
            if maintenance.as_object().map(|m| m.len()) != Some(2)
                || deferral.as_object().map(|m| m.len()) != Some(3)
                || ["reason", "follow_up"]
                    .iter()
                    .any(|k| !deferral[*k].as_str().is_some_and(|s| !s.trim().is_empty()))
                || deferral["approved_scope"] != "create_lane_closeout"
            {
                return Err(
                    "private approval reader: invalid or unknown maintenance deferral fields"
                        .into(),
                );
            }
        }
        Some("release_watch_enrolled") => {
            let release = entry
                .maintenance
                .release_watch
                .as_ref()
                .ok_or("approval release-watch has no matching registry declaration")?;
            let stated: crate::agent_registry::ReleaseWatchMetadata =
                serde_json::from_value(maintenance["release_watch"].clone())
                    .map_err(|e| e.to_string())?;
            if maintenance.as_object().map(|m| m.len()) != Some(2) || &stated != release {
                return Err("private approval reader: release-watch differs from registry or has unknown fields".into());
            }
        }
        _ => return Err("private approval reader: undecided maintenance mode".into()),
    }
    expected["maintenance"] = maintenance.clone();
    let expected = json!({
        "artifact_version":"1",
        "comparison_ref":"docs/agents/selection/cli-agent-selection-packet.md",
        "selection_mode":"factory_validation",
        "recommended_agent_id":entry.agent_id,
        "approved_agent_id":entry.agent_id,
        "approval_commit":document["approval_commit"],
        "approval_recorded_at":document["approval_recorded_at"],
        "support_depth":{"version":approval.support_depth_version},
        "descriptor":expected,
    });
    if document != expected {
        return Err("private approval reader: unknown fields or descriptor differs from the compiled isolated subject".into());
    }
    Ok(approval)
}

fn descriptor(entry: &AgentRegistryEntry) -> Value {
    json!({
        "agent_id":entry.agent_id,
        "display_name":entry.display_name,
        "crate_path":entry.crate_path,
        "manifest_root":entry.manifest_root,
        "package_name":entry.package_name,
        "canonical_targets":entry.canonical_targets,
        "wrapper_coverage_binding_kind":entry.wrapper_coverage.binding_kind,
        "wrapper_coverage_source_path":entry.wrapper_coverage.source_path,
        "always_on_capabilities":entry.capability_declaration.always_on,
        "backend_extensions":entry.capability_declaration.backend_extensions,
        "support_matrix_enabled":entry.publication.support_matrix_enabled,
        "capability_matrix_enabled":entry.publication.capability_matrix_enabled,
        "docs_release_track":entry.release.docs_release_track,
        "onboarding_pack_prefix":entry.scaffold.onboarding_pack_prefix,
    })
}
