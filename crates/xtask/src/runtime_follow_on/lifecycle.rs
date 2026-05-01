use std::{collections::BTreeSet, fs, path::Path};

use crate::{
    agent_lifecycle::{
        self, load_lifecycle_state, write_lifecycle_state, DeferredSurface, ImplementationSummary,
        LandedSurface, LifecycleStage, LifecycleState, RuntimeProfile, SideState, SupportTier,
        TemplateId,
    },
    agent_registry::AgentRegistryEntry,
    approval_artifact::ApprovalArtifact,
};

use super::io::now_rfc3339;
use super::{
    models::{HandoffContract, InputContract, RuntimeContext, ValidationReport},
    Error, HANDOFF_FILE_NAME, LEGACY_REQUIRED_PUBLICATION_COMMANDS,
};

pub(super) fn validate_handoff(path: &Path, context: &RuntimeContext) -> Result<(), String> {
    let payload = fs::read_to_string(path).map_err(|err| {
        format!(
            "missing or unreadable handoff.json at {}: {err}",
            path.display()
        )
    })?;
    let parsed: serde_json::Value = serde_json::from_str(&payload)
        .map_err(|err| format!("handoff.json is not valid json: {err}"))?;
    let object = parsed
        .as_object()
        .ok_or_else(|| "handoff.json root must be an object".to_string())?;

    for key in [
        "agent_id",
        "manifest_root",
        "runtime_lane_complete",
        "publication_refresh_required",
        "required_commands",
        "blockers",
    ] {
        if !object.contains_key(key) {
            return Err(format!("handoff.json is missing required field `{key}`"));
        }
    }

    let handoff: HandoffContract = serde_json::from_value(parsed)
        .map_err(|err| format!("handoff.json failed minimum schema validation: {err}"))?;
    if handoff.agent_id != context.approval.descriptor.agent_id {
        return Err(format!(
            "handoff.json agent_id `{}` does not match approval agent_id `{}`",
            handoff.agent_id, context.approval.descriptor.agent_id
        ));
    }
    if handoff.manifest_root != context.approval.descriptor.manifest_root {
        return Err(format!(
            "handoff.json manifest_root `{}` does not match approval manifest_root `{}`",
            handoff.manifest_root, context.approval.descriptor.manifest_root
        ));
    }
    if !handoff.runtime_lane_complete {
        return Err(
            "handoff.json runtime_lane_complete must be true for a successful write run"
                .to_string(),
        );
    }
    if !handoff.publication_refresh_required {
        return Err("handoff.json publication_refresh_required must be true".to_string());
    }
    let required = agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let legacy_required = LEGACY_REQUIRED_PUBLICATION_COMMANDS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let actual = handoff
        .required_commands
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if !required.is_subset(&actual) && !legacy_required.is_subset(&actual) {
        return Err(format!(
            "handoff.json required_commands must include {}",
            agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS.join(", ")
        ));
    }
    Ok(())
}

pub(super) fn load_enrolled_lifecycle_state(
    workspace_root: &Path,
    approval: &ApprovalArtifact,
) -> Result<(String, LifecycleState), Error> {
    let lifecycle_state_path =
        agent_lifecycle::lifecycle_state_path(&approval.descriptor.onboarding_pack_prefix);
    let lifecycle_state = load_lifecycle_state(workspace_root, &lifecycle_state_path)
        .map_err(|err| Error::Validation(err.to_string()))?;
    if lifecycle_state.lifecycle_stage != LifecycleStage::Enrolled {
        return Err(Error::Validation(format!(
            "runtime-follow-on requires lifecycle stage `enrolled` at `{}` (found `{}`)",
            lifecycle_state_path,
            lifecycle_state.lifecycle_stage.as_str()
        )));
    }
    Ok((lifecycle_state_path, lifecycle_state))
}

pub(super) fn persist_successful_runtime_integration(
    workspace_root: &Path,
    context: &RuntimeContext,
    prior_contract: &InputContract,
    written_paths: &[String],
) -> Result<(), Error> {
    let mut lifecycle_state = context.lifecycle_state.clone();
    lifecycle_state.lifecycle_stage = LifecycleStage::RuntimeIntegrated;
    lifecycle_state.support_tier = SupportTier::BaselineRuntime;
    lifecycle_state.current_owner_command = "runtime-follow-on --write".to_string();
    lifecycle_state.expected_next_command = format!(
        "prepare-publication --approval {} --write",
        context.approval.relative_path
    );
    lifecycle_state.last_transition_at = now_rfc3339()?;
    lifecycle_state.last_transition_by = "xtask runtime-follow-on --write".to_string();
    lifecycle_state.required_evidence =
        agent_lifecycle::required_evidence_for_stage(LifecycleStage::RuntimeIntegrated).to_vec();
    lifecycle_state.satisfied_evidence =
        agent_lifecycle::required_evidence_for_stage(LifecycleStage::RuntimeIntegrated).to_vec();
    lifecycle_state
        .side_states
        .retain(|state| !matches!(state, SideState::Blocked | SideState::FailedRetryable));
    lifecycle_state.blocking_issues.clear();
    lifecycle_state.retryable_failures.clear();
    lifecycle_state.implementation_summary = Some(build_implementation_summary(
        context,
        prior_contract,
        written_paths,
    ));
    lifecycle_state.publication_packet_path = None;
    lifecycle_state.publication_packet_sha256 = None;

    write_lifecycle_state(
        workspace_root,
        &context.lifecycle_state_path,
        &lifecycle_state,
    )
    .map_err(|err| Error::Internal(format!("write lifecycle state: {err}")))
}

pub(super) fn persist_failed_runtime_integration(
    workspace_root: &Path,
    context: &RuntimeContext,
    report: &ValidationReport,
) -> Result<(), Error> {
    let mut lifecycle_state = context.lifecycle_state.clone();
    let blockers = best_effort_handoff_blockers(&context.run_dir.join(HANDOFF_FILE_NAME));
    let (side_state, issues) = if blockers.is_empty() {
        (SideState::FailedRetryable, report.errors.clone())
    } else {
        (SideState::Blocked, blockers)
    };

    lifecycle_state.current_owner_command = "runtime-follow-on --write".to_string();
    lifecycle_state.last_transition_at = now_rfc3339()?;
    lifecycle_state.last_transition_by = "xtask runtime-follow-on --write".to_string();
    lifecycle_state
        .side_states
        .retain(|state| !matches!(state, SideState::Blocked | SideState::FailedRetryable));
    lifecycle_state.side_states.push(side_state);
    lifecycle_state.side_states.sort();
    lifecycle_state.side_states.dedup();
    match side_state {
        SideState::Blocked => {
            lifecycle_state.blocking_issues = issues;
            lifecycle_state.retryable_failures.clear();
        }
        SideState::FailedRetryable => {
            lifecycle_state.retryable_failures = issues;
            lifecycle_state.blocking_issues.clear();
        }
        SideState::Drifted | SideState::Deprecated => {}
    }

    write_lifecycle_state(
        workspace_root,
        &context.lifecycle_state_path,
        &lifecycle_state,
    )
    .map_err(|err| Error::Internal(format!("write lifecycle state: {err}")))
}

pub(super) fn required_publication_commands() -> Vec<String> {
    agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

pub(super) fn validate_registry_alignment(
    approval: &ApprovalArtifact,
    registry_entry: &AgentRegistryEntry,
) -> Result<(), Error> {
    let descriptor = &approval.descriptor;
    let mismatches = [
        (
            "crate_path",
            descriptor.crate_path.as_str(),
            registry_entry.crate_path.as_str(),
        ),
        (
            "backend_module",
            descriptor.backend_module.as_str(),
            registry_entry.backend_module.as_str(),
        ),
        (
            "manifest_root",
            descriptor.manifest_root.as_str(),
            registry_entry.manifest_root.as_str(),
        ),
        (
            "package_name",
            descriptor.package_name.as_str(),
            registry_entry.package_name.as_str(),
        ),
        (
            "wrapper_coverage_source_path",
            descriptor.wrapper_coverage_source_path.as_str(),
            registry_entry.wrapper_coverage.source_path.as_str(),
        ),
    ]
    .into_iter()
    .filter(|(_, expected, actual)| expected != actual)
    .map(|(field, expected, actual)| format!("{field}: approval=`{expected}` registry=`{actual}`"))
    .collect::<Vec<_>>();

    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(Error::Validation(format!(
            "approval/registry mismatch: {}",
            mismatches.join("; ")
        )))
    }
}

fn build_implementation_summary(
    context: &RuntimeContext,
    prior_contract: &InputContract,
    written_paths: &[String],
) -> ImplementationSummary {
    let requested_runtime_profile = requested_runtime_profile(prior_contract);
    let primary_template = primary_template(context, requested_runtime_profile);
    let mut landed_surfaces = BTreeSet::new();
    landed_surfaces.insert(LandedSurface::WrapperRuntime);
    landed_surfaces.insert(LandedSurface::BackendHarness);
    landed_surfaces.insert(LandedSurface::WrapperCoverageSource);
    landed_surfaces.insert(LandedSurface::RuntimeManifestEvidence);

    if wrote_agent_api_onboarding_test(prior_contract, written_paths) {
        landed_surfaces.insert(LandedSurface::AgentApiOnboardingTest);
    }

    for capability in all_capability_and_extension_entries(prior_contract) {
        if let Some(surface) = rich_surface_from_entry(capability) {
            landed_surfaces.insert(surface);
        }
    }

    let deferred_surfaces = prior_contract
        .allow_rich_surface
        .iter()
        .filter_map(|surface| {
            let mapped = rich_surface_from_entry(surface)?;
            if landed_surfaces.contains(&mapped) {
                None
            } else {
                Some(DeferredSurface {
                    surface: mapped,
                    reason: "Allowed for this run but not landed in the runtime integration delta."
                        .to_string(),
                })
            }
        })
        .collect::<Vec<_>>();

    ImplementationSummary {
        requested_runtime_profile,
        achieved_runtime_profile: requested_runtime_profile,
        primary_template,
        template_lineage: vec![template_name(primary_template).to_string()],
        landed_surfaces: landed_surfaces.into_iter().collect(),
        deferred_surfaces,
        minimal_profile_justification: prior_contract.minimal_justification_text.clone(),
    }
}

fn requested_runtime_profile(prior_contract: &InputContract) -> RuntimeProfile {
    match prior_contract.requested_tier.as_str() {
        "minimal" => RuntimeProfile::Minimal,
        "feature-rich" => RuntimeProfile::FeatureRich,
        _ => RuntimeProfile::Default,
    }
}

fn primary_template(
    context: &RuntimeContext,
    requested_runtime_profile: RuntimeProfile,
) -> TemplateId {
    match context.approval.descriptor.agent_id.as_str() {
        "opencode" => TemplateId::Opencode,
        "gemini_cli" => TemplateId::GeminiCli,
        "codex" => TemplateId::Codex,
        "claude_code" => TemplateId::ClaudeCode,
        "aider" => TemplateId::Aider,
        _ => match requested_runtime_profile {
            RuntimeProfile::Minimal => TemplateId::GeminiCli,
            RuntimeProfile::FeatureRich => TemplateId::Codex,
            RuntimeProfile::Default => TemplateId::Opencode,
        },
    }
}

fn template_name(template: TemplateId) -> &'static str {
    match template {
        TemplateId::Opencode => "opencode",
        TemplateId::GeminiCli => "gemini_cli",
        TemplateId::Codex => "codex",
        TemplateId::ClaudeCode => "claude_code",
        TemplateId::Aider => "aider",
    }
}

fn wrote_agent_api_onboarding_test(
    prior_contract: &InputContract,
    written_paths: &[String],
) -> bool {
    written_paths.iter().any(|path| {
        path == &prior_contract.required_agent_api_test
            || path.starts_with(&format!(
                "crates/agent_api/tests/c1_{}_runtime_follow_on/",
                prior_contract.agent_id
            ))
    })
}

fn all_capability_and_extension_entries(prior_contract: &InputContract) -> Vec<&str> {
    prior_contract
        .always_on_capabilities
        .iter()
        .chain(prior_contract.target_gated_capabilities.iter())
        .chain(prior_contract.config_gated_capabilities.iter())
        .chain(prior_contract.backend_extensions.iter())
        .map(String::as_str)
        .collect()
}

fn rich_surface_from_entry(entry: &str) -> Option<LandedSurface> {
    let normalized = entry.replace('_', "-");
    if normalized.contains("agent-api-exec-add-dirs-v1") || normalized == "add-dirs" {
        Some(LandedSurface::AddDirs)
    } else if normalized.contains("agent-api-exec-external-sandbox-v1")
        || normalized == "external-sandbox-policy"
    {
        Some(LandedSurface::ExternalSandboxPolicy)
    } else if normalized.contains("agent-api-tools-mcp-") || normalized == "mcp-management" {
        Some(LandedSurface::McpManagement)
    } else if normalized.contains("agent-api-session-resume-v1") || normalized == "session-resume" {
        Some(LandedSurface::SessionResume)
    } else if normalized.contains("agent-api-session-fork-v1") || normalized == "session-fork" {
        Some(LandedSurface::SessionFork)
    } else if normalized.contains("agent-api-tools-structured-v1")
        || normalized == "structured-tools"
    {
        Some(LandedSurface::StructuredTools)
    } else {
        None
    }
}

fn best_effort_handoff_blockers(path: &Path) -> Vec<String> {
    let Ok(payload) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&payload) else {
        return Vec::new();
    };
    let Some(blockers) = parsed.get("blockers").and_then(|value| value.as_array()) else {
        return Vec::new();
    };
    blockers
        .iter()
        .filter_map(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter(|value| *value != "Pending runtime follow-on implementation.")
        .map(ToOwned::to_owned)
        .collect()
}
