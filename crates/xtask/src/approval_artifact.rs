use std::{
    fmt, fs,
    path::{Component, Path, PathBuf},
};

use sha2::{Digest, Sha256};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use toml_edit::{DocumentMut, TableLike};

const DOCS_NEXT_ROOT: &str = "docs/project_management/next";
const APPROVAL_FILE_NAME: &str = "approved-agent.toml";
const GOVERNANCE_DIR_NAME: &str = "governance";
const FACTORY_VALIDATION_MODE: &str = "factory_validation";
const FRONTIER_EXPANSION_MODE: &str = "frontier_expansion";
const APPROVAL_ARTIFACT_VERSION: &str = "1";

#[derive(Debug, Clone)]
pub struct ApprovalArtifact {
    pub relative_path: String,
    pub canonical_path: PathBuf,
    pub sha256: String,
    pub descriptor: ApprovalDescriptor,
}

#[derive(Debug, Clone)]
pub struct ApprovalDescriptor {
    pub agent_id: String,
    pub display_name: String,
    pub crate_path: String,
    pub backend_module: String,
    pub manifest_root: String,
    pub package_name: String,
    pub canonical_targets: Vec<String>,
    pub wrapper_coverage_binding_kind: String,
    pub wrapper_coverage_source_path: String,
    pub always_on_capabilities: Vec<String>,
    pub target_gated_capabilities: Vec<String>,
    pub config_gated_capabilities: Vec<String>,
    pub backend_extensions: Vec<String>,
    pub support_matrix_enabled: bool,
    pub capability_matrix_enabled: bool,
    pub docs_release_track: String,
    pub onboarding_pack_prefix: String,
}

#[derive(Debug, Clone)]
pub enum ApprovalArtifactError {
    Validation(String),
    Internal(String),
}

impl fmt::Display for ApprovalArtifactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(message) | Self::Internal(message) => f.write_str(message),
        }
    }
}

pub fn load_approval_artifact(
    workspace_root: &Path,
    approval_path: &str,
) -> Result<ApprovalArtifact, ApprovalArtifactError> {
    let relative_path = PathBuf::from(approval_path);
    let path_pack_prefix = validate_approval_path(&relative_path)?;

    let workspace_root = fs::canonicalize(workspace_root).map_err(|err| {
        ApprovalArtifactError::Internal(format!("canonicalize {}: {err}", workspace_root.display()))
    })?;
    let lexical_path = workspace_root.join(&relative_path);
    let canonical_path = fs::canonicalize(&lexical_path).map_err(|err| {
        ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` does not resolve: {err}",
            relative_path.display()
        ))
    })?;
    if !canonical_path.starts_with(&workspace_root) {
        return Err(ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` resolves outside workspace root",
            relative_path.display()
        )));
    }

    let bytes = fs::read(&canonical_path).map_err(|err| {
        ApprovalArtifactError::Validation(format!(
            "read approval artifact `{}`: {err}",
            relative_path.display()
        ))
    })?;
    let text = std::str::from_utf8(&bytes).map_err(|err| {
        ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` must be valid utf-8: {err}",
            relative_path.display()
        ))
    })?;
    let document = text.parse::<DocumentMut>().map_err(|err| {
        ApprovalArtifactError::Validation(format!(
            "parse approval artifact `{}`: {err}",
            relative_path.display()
        ))
    })?;

    parse_approval_document(
        document,
        &workspace_root,
        &relative_path,
        &canonical_path,
        &path_pack_prefix,
        &bytes,
    )
}

fn parse_approval_document(
    document: DocumentMut,
    workspace_root: &Path,
    relative_path: &Path,
    canonical_path: &Path,
    path_pack_prefix: &str,
    bytes: &[u8],
) -> Result<ApprovalArtifact, ApprovalArtifactError> {
    let root = document.as_item().as_table_like().ok_or_else(|| {
        ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` root must be a table",
            relative_path.display()
        ))
    })?;

    let artifact_version = required_string(root, "artifact_version", relative_path)?;
    validate_artifact_version(relative_path, &artifact_version)?;
    let comparison_ref = required_string(root, "comparison_ref", relative_path)?;
    validate_repo_relative_existing_file(
        workspace_root,
        relative_path,
        "comparison_ref",
        &comparison_ref,
    )?;
    let selection_mode = required_string(root, "selection_mode", relative_path)?;
    match selection_mode.as_str() {
        FACTORY_VALIDATION_MODE | FRONTIER_EXPANSION_MODE => {}
        _ => {
            return Err(ApprovalArtifactError::Validation(format!(
                "approval artifact `{}` has invalid `selection_mode` `{selection_mode}`; expected `{FACTORY_VALIDATION_MODE}` or `{FRONTIER_EXPANSION_MODE}`",
                relative_path.display()
            )));
        }
    }
    let recommended_agent_id = required_string(root, "recommended_agent_id", relative_path)?;
    let approved_agent_id = required_string(root, "approved_agent_id", relative_path)?;
    let approval_commit = required_string(root, "approval_commit", relative_path)?;
    validate_commit_value(relative_path, "approval_commit", &approval_commit)?;
    let approval_recorded_at = required_string(root, "approval_recorded_at", relative_path)?;
    validate_rfc3339_value(relative_path, "approval_recorded_at", &approval_recorded_at)?;
    let override_reason = optional_string(root, "override_reason", relative_path)?;
    if recommended_agent_id != approved_agent_id
        && override_reason.as_deref().unwrap_or_default().is_empty()
    {
        return Err(ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` must include `override_reason` when `recommended_agent_id` and `approved_agent_id` differ",
            relative_path.display()
        )));
    }

    let descriptor = root.get("descriptor").ok_or_else(|| {
        ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` is missing required table `descriptor`",
            relative_path.display()
        ))
    })?;
    let descriptor = descriptor.as_table_like().ok_or_else(|| {
        ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` field `descriptor` must be a table",
            relative_path.display()
        ))
    })?;

    let agent_id = required_string(descriptor, "agent_id", relative_path)?;
    if agent_id != approved_agent_id {
        return Err(ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` has `descriptor.agent_id` `{agent_id}` that does not match `approved_agent_id` `{approved_agent_id}`",
            relative_path.display()
        )));
    }
    let onboarding_pack_prefix =
        required_string(descriptor, "onboarding_pack_prefix", relative_path)?;
    if onboarding_pack_prefix != path_pack_prefix {
        return Err(ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` belongs to onboarding_pack_prefix `{}` instead of `{}`",
            relative_path.display(),
            onboarding_pack_prefix,
            path_pack_prefix
        )));
    }

    Ok(ApprovalArtifact {
        relative_path: relative_path.display().to_string(),
        canonical_path: canonical_path.to_path_buf(),
        sha256: hex::encode(Sha256::digest(bytes)),
        descriptor: ApprovalDescriptor {
            agent_id,
            display_name: required_string(descriptor, "display_name", relative_path)?,
            crate_path: required_string(descriptor, "crate_path", relative_path)?,
            backend_module: required_string(descriptor, "backend_module", relative_path)?,
            manifest_root: required_string(descriptor, "manifest_root", relative_path)?,
            package_name: required_string(descriptor, "package_name", relative_path)?,
            canonical_targets: string_array(descriptor, "canonical_targets", relative_path, true)?,
            wrapper_coverage_binding_kind: required_string(
                descriptor,
                "wrapper_coverage_binding_kind",
                relative_path,
            )?,
            wrapper_coverage_source_path: required_string(
                descriptor,
                "wrapper_coverage_source_path",
                relative_path,
            )?,
            always_on_capabilities: string_array(
                descriptor,
                "always_on_capabilities",
                relative_path,
                false,
            )?,
            target_gated_capabilities: target_gate_entries(descriptor, relative_path)?,
            config_gated_capabilities: config_gate_entries(descriptor, relative_path)?,
            backend_extensions: string_array(
                descriptor,
                "backend_extensions",
                relative_path,
                false,
            )?,
            support_matrix_enabled: required_bool(
                descriptor,
                "support_matrix_enabled",
                relative_path,
            )?,
            capability_matrix_enabled: required_bool(
                descriptor,
                "capability_matrix_enabled",
                relative_path,
            )?,
            docs_release_track: required_string(descriptor, "docs_release_track", relative_path)?,
            onboarding_pack_prefix,
        },
    })
}

fn validate_approval_path(relative_path: &Path) -> Result<String, ApprovalArtifactError> {
    let components = relative_path.components().collect::<Vec<_>>();
    if components.is_empty()
        || components
            .iter()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(invalid_approval_path(relative_path));
    }

    let rooted = relative_path
        .strip_prefix(DOCS_NEXT_ROOT)
        .map_err(|_| invalid_approval_path(relative_path))?;
    let rooted_components = rooted.components().collect::<Vec<_>>();
    if rooted_components.len() != 3
        || !matches!(
            rooted_components[1],
            Component::Normal(part) if part == GOVERNANCE_DIR_NAME
        )
        || !matches!(
            rooted_components[2],
            Component::Normal(part) if part == APPROVAL_FILE_NAME
        )
    {
        return Err(invalid_approval_path(relative_path));
    }

    let Component::Normal(prefix) = rooted_components[0] else {
        return Err(invalid_approval_path(relative_path));
    };
    Ok(prefix.to_string_lossy().into_owned())
}

fn invalid_approval_path(relative_path: &Path) -> ApprovalArtifactError {
    ApprovalArtifactError::Validation(format!(
        "approval path `{}` must be repo-relative and match `{DOCS_NEXT_ROOT}/<prefix>/{GOVERNANCE_DIR_NAME}/{APPROVAL_FILE_NAME}`",
        relative_path.display()
    ))
}

fn validate_artifact_version(
    relative_path: &Path,
    artifact_version: &str,
) -> Result<(), ApprovalArtifactError> {
    if artifact_version != APPROVAL_ARTIFACT_VERSION {
        return Err(ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` has unsupported `artifact_version` `{artifact_version}`; expected `{APPROVAL_ARTIFACT_VERSION}`",
            relative_path.display()
        )));
    }
    Ok(())
}

fn validate_repo_relative_existing_file(
    workspace_root: &Path,
    relative_path: &Path,
    field_name: &str,
    field_value: &str,
) -> Result<(), ApprovalArtifactError> {
    let referenced = Path::new(field_value);
    let components = referenced.components().collect::<Vec<_>>();
    if components.is_empty()
        || components
            .iter()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` field `{field_name}` must be a repo-relative file path with only normal path components",
            relative_path.display()
        )));
    }

    let canonical_path = fs::canonicalize(workspace_root.join(referenced)).map_err(|err| {
        ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` field `{field_name}` does not resolve: {err}",
            relative_path.display()
        ))
    })?;
    if !canonical_path.starts_with(workspace_root) {
        return Err(ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` field `{field_name}` resolves outside workspace root",
            relative_path.display()
        )));
    }
    let metadata = fs::metadata(&canonical_path).map_err(|err| {
        ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` field `{field_name}` cannot be inspected: {err}",
            relative_path.display()
        ))
    })?;
    if !metadata.is_file() {
        return Err(ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` field `{field_name}` must point to an existing file",
            relative_path.display()
        )));
    }

    Ok(())
}

pub fn validate_commit_value(
    path: &Path,
    field_name: &str,
    value: &str,
) -> Result<(), ApprovalArtifactError> {
    let valid = (7..=40).contains(&value.len())
        && value.chars().all(|ch| matches!(ch, '0'..='9' | 'a'..='f'));
    if !valid {
        return Err(ApprovalArtifactError::Validation(format!(
            "{}: `{field_name}` must be 7-40 lowercase hex characters",
            path.display()
        )));
    }
    Ok(())
}

pub fn validate_rfc3339_value(
    path: &Path,
    field_name: &str,
    value: &str,
) -> Result<(), ApprovalArtifactError> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|err| {
        ApprovalArtifactError::Validation(format!(
            "{}: `{field_name}` must be RFC3339 ({err})",
            path.display()
        ))
    })?;
    Ok(())
}

fn required_string(
    table: &dyn TableLike,
    key: &str,
    relative_path: &Path,
) -> Result<String, ApprovalArtifactError> {
    let item = table.get(key).ok_or_else(|| {
        ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` is missing required field `{key}`",
            relative_path.display()
        ))
    })?;
    let value = item.as_str().ok_or_else(|| {
        ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` field `{key}` must be a string",
            relative_path.display()
        ))
    })?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` field `{key}` must not be empty",
            relative_path.display()
        )));
    }
    Ok(trimmed.to_string())
}

fn optional_string(
    table: &dyn TableLike,
    key: &str,
    relative_path: &Path,
) -> Result<Option<String>, ApprovalArtifactError> {
    let Some(item) = table.get(key) else {
        return Ok(None);
    };
    let value = item.as_str().ok_or_else(|| {
        ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` field `{key}` must be a string",
            relative_path.display()
        ))
    })?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    Ok(Some(trimmed.to_string()))
}

fn required_bool(
    table: &dyn TableLike,
    key: &str,
    relative_path: &Path,
) -> Result<bool, ApprovalArtifactError> {
    let item = table.get(key).ok_or_else(|| {
        ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` is missing required field `{key}`",
            relative_path.display()
        ))
    })?;
    item.as_bool().ok_or_else(|| {
        ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` field `{key}` must be a bool",
            relative_path.display()
        ))
    })
}

fn string_array(
    table: &dyn TableLike,
    key: &str,
    relative_path: &Path,
    required: bool,
) -> Result<Vec<String>, ApprovalArtifactError> {
    let Some(item) = table.get(key) else {
        if required {
            return Err(ApprovalArtifactError::Validation(format!(
                "approval artifact `{}` is missing required field `{key}`",
                relative_path.display()
            )));
        }
        return Ok(Vec::new());
    };
    let values = item.as_array().ok_or_else(|| {
        ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` field `{key}` must be an array of strings",
            relative_path.display()
        ))
    })?;
    values
        .iter()
        .map(|value| {
            let value = value.as_str().ok_or_else(|| {
                ApprovalArtifactError::Validation(format!(
                    "approval artifact `{}` field `{key}` must contain only strings",
                    relative_path.display()
                ))
            })?;
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(ApprovalArtifactError::Validation(format!(
                    "approval artifact `{}` field `{key}` must not contain empty strings",
                    relative_path.display()
                )));
            }
            Ok(trimmed.to_string())
        })
        .collect()
}

fn target_gate_entries(
    table: &dyn TableLike,
    relative_path: &Path,
) -> Result<Vec<String>, ApprovalArtifactError> {
    let Some(item) = table.get("target_gated_capabilities") else {
        return Ok(Vec::new());
    };
    let entries = item.as_array_of_tables().ok_or_else(|| {
        ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` field `descriptor.target_gated_capabilities` must be an array of tables",
            relative_path.display()
        ))
    })?;
    entries
        .iter()
        .map(|entry| {
            let capability_id = required_string(entry, "capability_id", relative_path)?;
            let targets = string_array(entry, "targets", relative_path, true)?;
            Ok(format!("{capability_id}:{}", targets.join(",")))
        })
        .collect()
}

fn config_gate_entries(
    table: &dyn TableLike,
    relative_path: &Path,
) -> Result<Vec<String>, ApprovalArtifactError> {
    let Some(item) = table.get("config_gated_capabilities") else {
        return Ok(Vec::new());
    };
    let entries = item.as_array_of_tables().ok_or_else(|| {
        ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` field `descriptor.config_gated_capabilities` must be an array of tables",
            relative_path.display()
        ))
    })?;
    entries
        .iter()
        .map(|entry| {
            let capability_id = required_string(entry, "capability_id", relative_path)?;
            let config_key = required_string(entry, "config_key", relative_path)?;
            let targets = string_array(entry, "targets", relative_path, false)?;
            if targets.is_empty() {
                Ok(format!("{capability_id}:{config_key}"))
            } else {
                Ok(format!(
                    "{capability_id}:{config_key}:{}",
                    targets.join(",")
                ))
            }
        })
        .collect()
}
