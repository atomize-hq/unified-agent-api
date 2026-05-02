use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};

use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use super::{
    file_sha256, DeferredSurface, LandedSurface, LifecycleError, LifecycleStage, LifecycleState,
    SideState, LIFECYCLE_SCHEMA_VERSION, REQUIRED_PUBLICATION_COMMANDS,
};
use crate::runtime_evidence_run;

pub(super) fn validate_schema_version(value: &str, surface: &str) -> Result<(), LifecycleError> {
    if value == LIFECYCLE_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(LifecycleError::Validation(format!(
            "{surface} schema_version must equal `{LIFECYCLE_SCHEMA_VERSION}`"
        )))
    }
}

pub(super) fn validate_pack_prefix(field: &str, value: &str) -> Result<(), LifecycleError> {
    validate_non_empty(field, value)?;
    if value.contains('/') || value.contains('\\') {
        return Err(LifecycleError::Validation(format!(
            "{field} must be a pack prefix, not a nested path"
        )));
    }
    Ok(())
}

pub(super) fn validate_optional_path_pair(
    path_field: &str,
    path: &Option<String>,
    sha_field: &str,
    sha: &Option<String>,
) -> Result<(), LifecycleError> {
    match (path.as_deref(), sha.as_deref()) {
        (Some(path), Some(sha)) => {
            validate_repo_relative_path(path_field, path)?;
            validate_sha256(sha_field, sha)?;
            Ok(())
        }
        (None, None) => Ok(()),
        _ => Err(LifecycleError::Validation(format!(
            "{path_field} and {sha_field} must either both be present or both be null"
        ))),
    }
}

pub(super) fn validate_optional_repo_relative_path(
    field: &str,
    value: &Option<String>,
) -> Result<(), LifecycleError> {
    if let Some(value) = value {
        validate_repo_relative_path(field, value)?;
    }
    Ok(())
}

pub(super) fn validate_runtime_evidence_run_id(
    stage: LifecycleStage,
    value: &Option<String>,
) -> Result<(), LifecycleError> {
    match (stage, value.as_deref()) {
        (LifecycleStage::RuntimeIntegrated, Some(run_id)) => {
            runtime_evidence_run::validate_run_id(run_id)
                .map_err(LifecycleError::Validation)?;
            Ok(())
        }
        (LifecycleStage::RuntimeIntegrated, None) => Err(LifecycleError::Validation(
            "active_runtime_evidence_run_id is required when lifecycle_stage is `runtime_integrated`"
                .to_string(),
        )),
        (
            LifecycleStage::Approved
            | LifecycleStage::Enrolled
            | LifecycleStage::PublicationReady
            | LifecycleStage::Published
            | LifecycleStage::ClosedBaseline,
            Some(_),
        ) => Err(LifecycleError::Validation(
            "active_runtime_evidence_run_id is only valid when lifecycle_stage is `runtime_integrated`"
                .to_string(),
        )),
        _ => Ok(()),
    }
}

pub(super) fn validate_path_hash_pair(
    workspace_root: &Path,
    path_field: &str,
    relative_path: &str,
    sha_field: &str,
    expected_sha: &str,
) -> Result<(), LifecycleError> {
    ensure_repo_relative_file_exists(workspace_root, path_field, relative_path)?;
    let actual_sha = file_sha256(workspace_root, relative_path)?;
    if actual_sha == expected_sha {
        Ok(())
    } else {
        Err(LifecycleError::Validation(format!(
            "{sha_field} does not match {path_field}"
        )))
    }
}

pub(super) fn ensure_repo_relative_file_exists(
    workspace_root: &Path,
    field: &str,
    relative_path: &str,
) -> Result<(), LifecycleError> {
    let resolved = resolve_repo_relative_path(workspace_root, relative_path)?;
    if resolved.is_file() {
        Ok(())
    } else {
        Err(LifecycleError::Validation(format!(
            "{field} `{relative_path}` does not exist"
        )))
    }
}

pub(super) fn resolve_repo_relative_path(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<PathBuf, LifecycleError> {
    validate_repo_relative_path("path", relative_path)?;
    Ok(workspace_root.join(relative_path))
}

pub(super) fn resolve_repo_relative_path_for_write(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<PathBuf, LifecycleError> {
    validate_repo_relative_path("path", relative_path)?;
    Ok(workspace_root.join(relative_path))
}

pub(super) fn validate_repo_relative_path(field: &str, value: &str) -> Result<(), LifecycleError> {
    validate_non_empty(field, value)?;
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(LifecycleError::Validation(format!(
            "{field} must be repo-relative, not absolute"
        )));
    }
    if value.contains('\\') {
        return Err(LifecycleError::Validation(format!(
            "{field} must use `/` separators"
        )));
    }
    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(LifecycleError::Validation(format!(
                    "{field} must not contain `..`"
                )))
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(LifecycleError::Validation(format!(
                    "{field} must be repo-relative"
                )))
            }
        }
    }
    Ok(())
}

pub(super) fn validate_sha256(field: &str, value: &str) -> Result<(), LifecycleError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(LifecycleError::Validation(format!(
            "{field} must be 64 lowercase hex characters"
        )))
    }
}

pub(super) fn validate_non_empty(field: &str, value: &str) -> Result<(), LifecycleError> {
    if value.trim().is_empty() {
        Err(LifecycleError::Validation(format!(
            "{field} must not be empty"
        )))
    } else {
        Ok(())
    }
}

pub(super) fn validate_rfc3339(field: &str, value: &str) -> Result<(), LifecycleError> {
    OffsetDateTime::parse(value, &Rfc3339)
        .map(|_| ())
        .map_err(|err| LifecycleError::Validation(format!("{field} must be RFC3339: {err}")))
}

pub(super) fn validate_string_list(field: &str, values: &[String]) -> Result<(), LifecycleError> {
    let mut seen = BTreeSet::new();
    for value in values {
        validate_non_empty(field, value)?;
        if !seen.insert(value) {
            return Err(LifecycleError::Validation(format!(
                "{field} contains duplicate value `{value}`"
            )));
        }
    }
    Ok(())
}

pub(super) fn validate_unique_copy<T: Copy + Ord>(
    field: &str,
    values: &[T],
    render: fn(T) -> &'static str,
) -> Result<(), LifecycleError> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(*value) {
            return Err(LifecycleError::Validation(format!(
                "{field} contains duplicate value `{}`",
                render(*value)
            )));
        }
    }
    Ok(())
}

pub(super) fn validate_subset<T: Copy + Ord>(
    field: &str,
    values: &[T],
    allowed_field: &str,
    allowed: &[T],
    render: fn(T) -> &'static str,
) -> Result<(), LifecycleError> {
    let allowed = allowed.iter().copied().collect::<BTreeSet<_>>();
    for value in values {
        if !allowed.contains(value) {
            return Err(LifecycleError::Validation(format!(
                "{field} value `{}` is not present in {allowed_field}",
                render(*value)
            )));
        }
    }
    Ok(())
}

pub(super) fn validate_stage_minimum_evidence(
    stage: LifecycleStage,
    field: &str,
    values: &[super::EvidenceId],
) -> Result<(), LifecycleError> {
    for required in super::required_evidence_for_stage(stage) {
        if !values.contains(required) {
            return Err(LifecycleError::Validation(format!(
                "{field} is missing required evidence `{}` for lifecycle_stage `{}`",
                required.as_str(),
                stage.as_str()
            )));
        }
    }
    Ok(())
}

pub(super) fn validate_stage_field_presence(
    stage: LifecycleStage,
    field: &str,
    is_present: bool,
    required_stages: &[LifecycleStage],
) -> Result<(), LifecycleError> {
    if required_stages.contains(&stage) && !is_present {
        return Err(LifecycleError::Validation(format!(
            "{field} is required when lifecycle_stage is `{}`",
            stage.as_str()
        )));
    }
    Ok(())
}

pub(super) fn validate_side_state_issues(state: &LifecycleState) -> Result<(), LifecycleError> {
    let side_states = state.side_states.iter().copied().collect::<BTreeSet<_>>();
    if side_states.contains(&SideState::Blocked) == state.blocking_issues.is_empty() {
        return Err(LifecycleError::Validation(
            "side_state `blocked` must appear if and only if blocking_issues is non-empty"
                .to_string(),
        ));
    }
    if side_states.contains(&SideState::FailedRetryable) == state.retryable_failures.is_empty() {
        return Err(LifecycleError::Validation(
            "side_state `failed_retryable` must appear if and only if retryable_failures is non-empty"
                .to_string(),
        ));
    }
    if side_states.contains(&SideState::Drifted)
        && !matches!(
            state.lifecycle_stage,
            LifecycleStage::Published | LifecycleStage::ClosedBaseline
        )
    {
        return Err(LifecycleError::Validation(
            "side_state `drifted` is only valid after publication truth exists".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn validate_template_lineage(values: &[String]) -> Result<(), LifecycleError> {
    validate_string_list("template_lineage", values)?;
    if values.is_empty() {
        return Err(LifecycleError::Validation(
            "template_lineage must contain at least one entry".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn validate_deferred_surfaces(values: &[DeferredSurface]) -> Result<(), LifecycleError> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value.surface) {
            return Err(LifecycleError::Validation(format!(
                "deferred_surfaces contains duplicate surface `{}`",
                landed_surface_name(value.surface)
            )));
        }
        validate_non_empty("deferred_surfaces.reason", &value.reason)?;
    }
    Ok(())
}

pub(super) fn validate_required_publication_commands(
    values: &[String],
) -> Result<(), LifecycleError> {
    if values.len() != REQUIRED_PUBLICATION_COMMANDS.len() {
        return Err(LifecycleError::Validation(format!(
            "required_commands must contain exactly {} entries",
            REQUIRED_PUBLICATION_COMMANDS.len()
        )));
    }
    let expected = REQUIRED_PUBLICATION_COMMANDS
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    if values == expected {
        Ok(())
    } else {
        Err(LifecycleError::Validation(
            "required_commands must match the frozen publication command set exactly".to_string(),
        ))
    }
}

pub(super) fn landed_surface_name(value: LandedSurface) -> &'static str {
    match value {
        LandedSurface::WrapperRuntime => "wrapper_runtime",
        LandedSurface::BackendHarness => "backend_harness",
        LandedSurface::AgentApiOnboardingTest => "agent_api_onboarding_test",
        LandedSurface::WrapperCoverageSource => "wrapper_coverage_source",
        LandedSurface::RuntimeManifestEvidence => "runtime_manifest_evidence",
        LandedSurface::AddDirs => "add_dirs",
        LandedSurface::ExternalSandboxPolicy => "external_sandbox_policy",
        LandedSurface::McpManagement => "mcp_management",
        LandedSurface::SessionResume => "session_resume",
        LandedSurface::SessionFork => "session_fork",
        LandedSurface::StructuredTools => "structured_tools",
    }
}
