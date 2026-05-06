use std::{collections::BTreeSet, fs, path::Path};

use crate::{
    agent_lifecycle::{self, load_lifecycle_state, LifecycleStage, PublicationReadyPacket},
    agent_registry::{
        AgentRegistryEntry, GovernanceCheck, GovernanceComparisonKind, MarkdownExtractionMode,
        REGISTRY_RELATIVE_PATH,
    },
    approval_artifact::load_approval_artifact,
    support_matrix::{BackendSupportState, SupportRow, UaaSupportState},
};

use super::{
    build_finding, shared, DriftCategory, DriftFinding, CAPABILITY_MATRIX_PATH,
    SUPPORT_MATRIX_JSON_PATH, SUPPORT_MATRIX_MARKDOWN_PATH,
};

pub(super) fn inspect_governance_docs(
    entry: &AgentRegistryEntry,
    workspace_root: &Path,
    capability_truth: Result<&BTreeSet<String>, &String>,
    expected_support_rows: Result<&Vec<SupportRow>, &String>,
) -> Option<DriftFinding> {
    let mut issues = Vec::new();
    let mut surfaces = BTreeSet::new();

    for check in &entry.maintenance.governance_checks {
        let surface = check.path.clone();
        let check_path = workspace_root.join(&surface);
        if !check_path.exists() {
            if check.required {
                issues.push(format!("{surface} is missing required governance surface"));
                surfaces.insert(surface);
            }
            continue;
        }

        let result = match check.comparison_kind {
            GovernanceComparisonKind::ApprovedAgentDescriptor => {
                inspect_approved_agent_descriptor(entry, workspace_root, check)
            }
            GovernanceComparisonKind::MarkdownCapabilityClaim => {
                inspect_markdown_capability_claim(&check_path, check, capability_truth)
            }
            GovernanceComparisonKind::MarkdownSupportClaim => match expected_support_rows {
                Ok(rows) => inspect_markdown_support_claim(&check_path, check, rows.as_slice()),
                Err(_) => GovernanceInspectionResult::default(),
            },
        };

        if !result.issues.is_empty() {
            issues.extend(result.issues);
            surfaces.insert(surface);
            surfaces.extend(result.authoritative_surfaces);
        }
    }

    let lifecycle_result = inspect_lifecycle_baseline(entry, workspace_root);
    if !lifecycle_result.issues.is_empty() {
        issues.extend(lifecycle_result.issues);
        surfaces.extend(lifecycle_result.authoritative_surfaces);
    }

    if issues.is_empty() {
        None
    } else {
        Some(build_finding(
            DriftCategory::GovernanceDoc,
            "historical governance surfaces no longer match current registry, capability, or support truth.",
            issues,
            surfaces.into_iter().collect(),
        ))
    }
}

fn inspect_lifecycle_baseline(
    entry: &AgentRegistryEntry,
    workspace_root: &Path,
) -> GovernanceInspectionResult {
    let mut result = GovernanceInspectionResult::default();
    let lifecycle_state_path =
        agent_lifecycle::lifecycle_state_path(&entry.scaffold.onboarding_pack_prefix);
    let lifecycle_absolute = workspace_root.join(&lifecycle_state_path);
    if !lifecycle_absolute.is_file() {
        return result;
    }

    result
        .authoritative_surfaces
        .insert(lifecycle_state_path.clone());
    let lifecycle_state = match load_lifecycle_state(workspace_root, &lifecycle_state_path) {
        Ok(state) => state,
        Err(err) => {
            result.issues.push(format!(
                "{} failed lifecycle baseline validation ({err})",
                lifecycle_state_path
            ));
            return result;
        }
    };

    match lifecycle_state.lifecycle_stage {
        LifecycleStage::ClosedBaseline | LifecycleStage::Published => {}
        other => {
            result.issues.push(format!(
                "{} is not yet a maintenance baseline (`{}`)",
                lifecycle_state_path,
                other.as_str()
            ));
        }
    }

    let Some(packet_path) = lifecycle_state.publication_packet_path.as_deref() else {
        result.issues.push(format!(
            "{} does not record publication_packet_path",
            lifecycle_state_path
        ));
        return result;
    };
    let Some(packet_sha) = lifecycle_state.publication_packet_sha256.as_deref() else {
        result.issues.push(format!(
            "{} does not record publication_packet_sha256",
            lifecycle_state_path
        ));
        return result;
    };
    result
        .authoritative_surfaces
        .insert(packet_path.to_string());
    match agent_lifecycle::file_sha256(workspace_root, packet_path) {
        Ok(actual_sha) if actual_sha == packet_sha => {}
        Ok(_) => result.issues.push(format!(
            "{} publication packet sha no longer matches {}",
            lifecycle_state_path, packet_path
        )),
        Err(err) => result.issues.push(format!("{}: {}", packet_path, err)),
    }

    let packet_bytes = match fs::read(workspace_root.join(packet_path)) {
        Ok(bytes) => bytes,
        Err(err) => {
            result.issues.push(format!("read {}: {err}", packet_path));
            return result;
        }
    };
    let packet: PublicationReadyPacket = match serde_json::from_slice(&packet_bytes) {
        Ok(packet) => packet,
        Err(err) => {
            result.issues.push(format!("parse {}: {err}", packet_path));
            return result;
        }
    };
    if let Err(err) = packet.validate() {
        result
            .issues
            .push(format!("{} failed packet validation ({err})", packet_path));
        return result;
    }
    if packet.approval_artifact_path != lifecycle_state.approval_artifact_path
        || packet.approval_artifact_sha256 != lifecycle_state.approval_artifact_sha256
    {
        result.issues.push(format!(
            "{} no longer matches lifecycle approval continuity in {}",
            packet_path, lifecycle_state_path
        ));
    }
    if packet.agent_id != entry.agent_id {
        result.issues.push(format!(
            "{} agent_id `{}` does not match registry agent `{}`",
            packet_path, packet.agent_id, entry.agent_id
        ));
    }
    if packet.manifest_root != entry.manifest_root {
        result.issues.push(format!(
            "{} manifest_root `{}` does not match registry manifest_root `{}`",
            packet_path, packet.manifest_root, entry.manifest_root
        ));
    }
    if let Some(closeout_path) = lifecycle_state.closeout_baseline_path.as_deref() {
        result
            .authoritative_surfaces
            .insert(closeout_path.to_string());
    } else if matches!(
        lifecycle_state.lifecycle_stage,
        LifecycleStage::ClosedBaseline
    ) {
        result.issues.push(format!(
            "{} does not record closeout_baseline_path",
            lifecycle_state_path
        ));
    }

    result
}

#[derive(Default)]
struct GovernanceInspectionResult {
    issues: Vec<String>,
    authoritative_surfaces: BTreeSet<String>,
}

fn inspect_approved_agent_descriptor(
    entry: &AgentRegistryEntry,
    workspace_root: &Path,
    check: &GovernanceCheck,
) -> GovernanceInspectionResult {
    let mut result = GovernanceInspectionResult::default();
    result
        .authoritative_surfaces
        .insert(REGISTRY_RELATIVE_PATH.to_string());

    let artifact = match load_approval_artifact(workspace_root, &check.path) {
        Ok(artifact) => artifact,
        Err(err) => {
            result.issues.push(err.to_string());
            return result;
        }
    };

    let expected_target_gated = entry
        .capability_declaration
        .target_gated
        .iter()
        .map(|gate| format!("{}:{}", gate.capability_id, gate.targets.join(",")))
        .collect::<Vec<_>>();
    let expected_config_gated = entry
        .capability_declaration
        .config_gated
        .iter()
        .map(|gate| match gate.targets.as_deref() {
            Some(targets) => format!(
                "{}:{}:{}",
                gate.capability_id,
                gate.config_key,
                targets.join(",")
            ),
            None => format!("{}:{}", gate.capability_id, gate.config_key),
        })
        .collect::<Vec<_>>();

    let mut mismatches = Vec::new();
    compare_descriptor_field(
        &mut mismatches,
        "display_name",
        &artifact.descriptor.display_name,
        &entry.display_name,
    );
    compare_descriptor_field(
        &mut mismatches,
        "crate_path",
        &artifact.descriptor.crate_path,
        &entry.crate_path,
    );
    compare_descriptor_field(
        &mut mismatches,
        "backend_module",
        &artifact.descriptor.backend_module,
        &entry.backend_module,
    );
    compare_descriptor_field(
        &mut mismatches,
        "manifest_root",
        &artifact.descriptor.manifest_root,
        &entry.manifest_root,
    );
    compare_descriptor_field(
        &mut mismatches,
        "package_name",
        &artifact.descriptor.package_name,
        &entry.package_name,
    );
    compare_descriptor_field(
        &mut mismatches,
        "wrapper_coverage_binding_kind",
        &artifact.descriptor.wrapper_coverage_binding_kind,
        &entry.wrapper_coverage.binding_kind,
    );
    compare_descriptor_field(
        &mut mismatches,
        "wrapper_coverage_source_path",
        &artifact.descriptor.wrapper_coverage_source_path,
        &entry.wrapper_coverage.source_path,
    );
    compare_descriptor_field(
        &mut mismatches,
        "docs_release_track",
        &artifact.descriptor.docs_release_track,
        &entry.release.docs_release_track,
    );
    compare_descriptor_field(
        &mut mismatches,
        "onboarding_pack_prefix",
        &artifact.descriptor.onboarding_pack_prefix,
        &entry.scaffold.onboarding_pack_prefix,
    );
    compare_descriptor_bool(
        &mut mismatches,
        "support_matrix_enabled",
        artifact.descriptor.support_matrix_enabled,
        entry.publication.support_matrix_enabled,
    );
    compare_descriptor_bool(
        &mut mismatches,
        "capability_matrix_enabled",
        artifact.descriptor.capability_matrix_enabled,
        entry.publication.capability_matrix_enabled,
    );
    if let Some(target) = artifact.descriptor.capability_matrix_target.as_deref() {
        compare_descriptor_optional_field(
            &mut mismatches,
            "capability_matrix_target",
            Some(target),
            entry.publication.capability_matrix_target.as_deref(),
        );
    }
    compare_descriptor_array(
        &mut mismatches,
        "canonical_targets",
        &artifact.descriptor.canonical_targets,
        &entry.canonical_targets,
    );
    compare_descriptor_array(
        &mut mismatches,
        "always_on_capabilities",
        &artifact.descriptor.always_on_capabilities,
        &entry.capability_declaration.always_on,
    );
    compare_descriptor_array(
        &mut mismatches,
        "target_gated_capabilities",
        &artifact.descriptor.target_gated_capabilities,
        &expected_target_gated,
    );
    compare_descriptor_array(
        &mut mismatches,
        "config_gated_capabilities",
        &artifact.descriptor.config_gated_capabilities,
        &expected_config_gated,
    );
    compare_descriptor_array(
        &mut mismatches,
        "backend_extensions",
        &artifact.descriptor.backend_extensions,
        &entry.capability_declaration.backend_extensions,
    );

    if !mismatches.is_empty() {
        result.issues.push(format!(
            "{} no longer matches registry-governed approval truth ({})",
            check.path,
            mismatches.join("; ")
        ));
    }

    result
}

fn inspect_markdown_capability_claim(
    path: &Path,
    check: &GovernanceCheck,
    capability_truth: Result<&BTreeSet<String>, &String>,
) -> GovernanceInspectionResult {
    let mut result = GovernanceInspectionResult::default();
    result
        .authoritative_surfaces
        .insert(CAPABILITY_MATRIX_PATH.to_string());

    let truth = match capability_truth {
        Ok(truth) => truth,
        Err(err) => {
            result.issues.push(err.clone());
            return result;
        }
    };

    let block = match load_markdown_block(path, check, MarkdownExtractionMode::InlineCodeIds) {
        Ok(block) => block,
        Err(err) => {
            result.issues.push(err);
            return result;
        }
    };
    let claimed = shared::inline_code_ids(&block);
    if claimed == *truth {
        return result;
    }

    let missing = truth.difference(&claimed).cloned().collect::<Vec<_>>();
    let unexpected = claimed.difference(truth).cloned().collect::<Vec<_>>();
    let mut pieces = Vec::new();
    if !missing.is_empty() {
        pieces.push(format!("missing capability ids: {}", missing.join(", ")));
    }
    if !unexpected.is_empty() {
        pieces.push(format!(
            "unexpected capability ids: {}",
            unexpected.join(", ")
        ));
    }

    result.issues.push(format!(
        "{} capability claim no longer matches current backend truth ({})",
        path.display(),
        pieces.join("; ")
    ));
    result
}

fn inspect_markdown_support_claim(
    path: &Path,
    check: &GovernanceCheck,
    expected_support_rows: &[SupportRow],
) -> GovernanceInspectionResult {
    let mut result = GovernanceInspectionResult::default();
    result
        .authoritative_surfaces
        .insert(SUPPORT_MATRIX_JSON_PATH.to_string());
    result
        .authoritative_surfaces
        .insert(SUPPORT_MATRIX_MARKDOWN_PATH.to_string());

    let block = match load_markdown_block(path, check, MarkdownExtractionMode::SupportStateLines) {
        Ok(block) => block,
        Err(err) => {
            result.issues.push(err);
            return result;
        }
    };
    let states = match shared::parse_support_state_lines(&block) {
        Ok(states) => states,
        Err(err) => {
            result.issues.push(format!(
                "{} support claim is invalid ({err})",
                path.display()
            ));
            return result;
        }
    };

    let Some(claimed_backend) = states.get("backend_support") else {
        result.issues.push(format!(
            "{} support claim is missing required key `backend_support`",
            path.display()
        ));
        return result;
    };
    let Some(claimed_uaa) = states.get("uaa_support") else {
        result.issues.push(format!(
            "{} support claim is missing required key `uaa_support`",
            path.display()
        ));
        return result;
    };

    let unknown_keys = states
        .keys()
        .filter(|key| key.as_str() != "backend_support" && key.as_str() != "uaa_support")
        .cloned()
        .collect::<Vec<_>>();
    if !unknown_keys.is_empty() {
        result.issues.push(format!(
            "{} support claim contains unsupported keys: {}",
            path.display(),
            unknown_keys.join(", ")
        ));
        return result;
    }

    let backend_states = expected_support_rows
        .iter()
        .map(|row| backend_support_state(row.backend_support).to_string())
        .collect::<BTreeSet<_>>();
    let uaa_states = expected_support_rows
        .iter()
        .map(|row| uaa_support_state(row.uaa_support).to_string())
        .collect::<BTreeSet<_>>();

    let mut pieces = Vec::new();
    if !backend_states.contains(claimed_backend) {
        pieces.push(format!(
            "backend_support claim `{}` does not match current states {}",
            claimed_backend,
            join_states(&backend_states)
        ));
    }
    if !uaa_states.contains(claimed_uaa) {
        pieces.push(format!(
            "uaa_support claim `{}` does not match current states {}",
            claimed_uaa,
            join_states(&uaa_states)
        ));
    }

    if !pieces.is_empty() {
        result.issues.push(format!(
            "{} support claim no longer matches current support truth ({})",
            path.display(),
            pieces.join("; ")
        ));
    }

    result
}

fn load_markdown_block(
    path: &Path,
    check: &GovernanceCheck,
    expected_mode: MarkdownExtractionMode,
) -> Result<String, String> {
    debug_assert_eq!(check.extraction_mode, Some(expected_mode));
    let text =
        fs::read_to_string(path).map_err(|err| format!("read({}): {err}", path.display()))?;
    let start_marker = check
        .start_marker
        .as_deref()
        .expect("registry validation requires start_marker");
    let end_marker = check
        .end_marker
        .as_deref()
        .expect("registry validation requires end_marker");
    shared::extract_marked_block(&text, start_marker, end_marker)
        .map_err(|err| format!("{}: {err}", path.display()))
}

fn compare_descriptor_field(
    mismatches: &mut Vec<String>,
    field_name: &str,
    actual: &str,
    expected: &str,
) {
    if actual != expected {
        mismatches.push(format!(
            "{field_name} expected `{expected}` but historical approval records `{actual}`"
        ));
    }
}

fn compare_descriptor_bool(
    mismatches: &mut Vec<String>,
    field_name: &str,
    actual: bool,
    expected: bool,
) {
    if actual != expected {
        mismatches.push(format!(
            "{field_name} expected `{expected}` but historical approval records `{actual}`"
        ));
    }
}

fn compare_descriptor_optional_field(
    mismatches: &mut Vec<String>,
    field_name: &str,
    actual: Option<&str>,
    expected: Option<&str>,
) {
    if actual != expected {
        mismatches.push(format!(
            "{field_name} expected `{}` but historical approval records `{}`",
            expected.unwrap_or("(none)"),
            actual.unwrap_or("(none)")
        ));
    }
}

fn compare_descriptor_array(
    mismatches: &mut Vec<String>,
    field_name: &str,
    actual: &[String],
    expected: &[String],
) {
    if actual != expected {
        mismatches.push(format!(
            "{field_name} expected [{}] but historical approval records [{}]",
            expected.join(", "),
            actual.join(", ")
        ));
    }
}

fn backend_support_state(value: BackendSupportState) -> &'static str {
    match value {
        BackendSupportState::Supported => "supported",
        BackendSupportState::Partial => "partial",
        BackendSupportState::Unsupported => "unsupported",
    }
}

fn uaa_support_state(value: UaaSupportState) -> &'static str {
    match value {
        UaaSupportState::Supported => "supported",
        UaaSupportState::Partial => "partial",
        UaaSupportState::Unsupported => "unsupported",
    }
}

fn join_states(states: &BTreeSet<String>) -> String {
    format!(
        "[{}]",
        states.iter().cloned().collect::<Vec<_>>().join(", ")
    )
}
