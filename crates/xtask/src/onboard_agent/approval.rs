use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use sha2::{Digest, Sha256};
use toml_edit::{DocumentMut, TableLike};

use super::{DraftDescriptorInput, Error, WorkspacePathJail, DOCS_NEXT_ROOT};

const APPROVAL_FILE_NAME: &str = "approved-agent.toml";
const GOVERNANCE_DIR_NAME: &str = "governance";
const FACTORY_VALIDATION_MODE: &str = "factory_validation";
const FRONTIER_EXPANSION_MODE: &str = "frontier_expansion";

pub(super) fn load_descriptor_input(
    approval_path: &str,
    jail: &WorkspacePathJail,
) -> Result<DraftDescriptorInput, Error> {
    let relative_path = PathBuf::from(approval_path);
    validate_approval_path(&relative_path)?;
    let resolved_path = jail.resolve(&relative_path)?;
    let bytes = fs::read(&resolved_path)
        .map_err(|err| Error::Internal(format!("read {}: {err}", relative_path.display())))?;
    let text = std::str::from_utf8(&bytes).map_err(|err| {
        Error::Validation(format!(
            "approval artifact `{}` must be valid utf-8: {err}",
            relative_path.display()
        ))
    })?;
    let document = text.parse::<DocumentMut>().map_err(|err| {
        Error::Validation(format!(
            "parse approval artifact `{}`: {err}",
            relative_path.display()
        ))
    })?;

    parse_approval_document(document, &relative_path, &bytes)
}

fn parse_approval_document(
    document: DocumentMut,
    relative_path: &Path,
    bytes: &[u8],
) -> Result<DraftDescriptorInput, Error> {
    let root = document.as_item().as_table_like().ok_or_else(|| {
        Error::Validation(format!(
            "approval artifact `{}` root must be a table",
            relative_path.display()
        ))
    })?;

    let artifact_version = required_string(root, "artifact_version", relative_path)?;
    let comparison_ref = required_string(root, "comparison_ref", relative_path)?;
    let selection_mode = required_string(root, "selection_mode", relative_path)?;
    match selection_mode.as_str() {
        FACTORY_VALIDATION_MODE | FRONTIER_EXPANSION_MODE => {}
        _ => {
            return Err(Error::Validation(format!(
                "approval artifact `{}` has invalid `selection_mode` `{selection_mode}`; expected `{FACTORY_VALIDATION_MODE}` or `{FRONTIER_EXPANSION_MODE}`",
                relative_path.display()
            )));
        }
    }
    let recommended_agent_id = required_string(root, "recommended_agent_id", relative_path)?;
    let approved_agent_id = required_string(root, "approved_agent_id", relative_path)?;
    let approval_commit = required_string(root, "approval_commit", relative_path)?;
    let approval_recorded_at = required_string(root, "approval_recorded_at", relative_path)?;
    let override_reason = optional_string(root, "override_reason", relative_path)?;
    if recommended_agent_id != approved_agent_id
        && override_reason.as_deref().unwrap_or_default().is_empty()
    {
        return Err(Error::Validation(format!(
            "approval artifact `{}` must include `override_reason` when `recommended_agent_id` and `approved_agent_id` differ",
            relative_path.display()
        )));
    }

    let descriptor = root.get("descriptor").ok_or_else(|| {
        Error::Validation(format!(
            "approval artifact `{}` is missing required table `descriptor`",
            relative_path.display()
        ))
    })?;
    let descriptor = descriptor.as_table_like().ok_or_else(|| {
        Error::Validation(format!(
            "approval artifact `{}` field `descriptor` must be a table",
            relative_path.display()
        ))
    })?;

    let agent_id = required_string(descriptor, "agent_id", relative_path)?;
    if agent_id != approved_agent_id {
        return Err(Error::Validation(format!(
            "approval artifact `{}` has `descriptor.agent_id` `{agent_id}` that does not match `approved_agent_id` `{approved_agent_id}`",
            relative_path.display()
        )));
    }

    let _ = (
        artifact_version,
        comparison_ref,
        approval_commit,
        approval_recorded_at,
    );

    Ok(DraftDescriptorInput {
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
        backend_extensions: string_array(descriptor, "backend_extensions", relative_path, false)?,
        support_matrix_enabled: required_bool(descriptor, "support_matrix_enabled", relative_path)?,
        capability_matrix_enabled: required_bool(
            descriptor,
            "capability_matrix_enabled",
            relative_path,
        )?,
        docs_release_track: required_string(descriptor, "docs_release_track", relative_path)?,
        onboarding_pack_prefix: required_string(
            descriptor,
            "onboarding_pack_prefix",
            relative_path,
        )?,
        approval_artifact_path: Some(relative_path.display().to_string()),
        approval_artifact_sha256: Some(hex::encode(Sha256::digest(bytes))),
    })
}

fn validate_approval_path(relative_path: &Path) -> Result<(), Error> {
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
    if rooted_components.len() < 3
        || !matches!(
            rooted_components[rooted_components.len() - 2],
            Component::Normal(part) if part == GOVERNANCE_DIR_NAME
        )
        || !matches!(
            rooted_components[rooted_components.len() - 1],
            Component::Normal(part) if part == APPROVAL_FILE_NAME
        )
    {
        return Err(invalid_approval_path(relative_path));
    }

    Ok(())
}

fn invalid_approval_path(relative_path: &Path) -> Error {
    Error::Validation(format!(
        "approval path `{}` must be repo-relative and rooted under `{DOCS_NEXT_ROOT}/**/{GOVERNANCE_DIR_NAME}/{APPROVAL_FILE_NAME}`",
        relative_path.display()
    ))
}

fn required_string(
    table: &dyn TableLike,
    key: &str,
    relative_path: &Path,
) -> Result<String, Error> {
    let item = table.get(key).ok_or_else(|| {
        Error::Validation(format!(
            "approval artifact `{}` is missing required field `{key}`",
            relative_path.display()
        ))
    })?;
    let value = item.as_str().ok_or_else(|| {
        Error::Validation(format!(
            "approval artifact `{}` field `{key}` must be a string",
            relative_path.display()
        ))
    })?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(Error::Validation(format!(
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
) -> Result<Option<String>, Error> {
    let Some(item) = table.get(key) else {
        return Ok(None);
    };
    let value = item.as_str().ok_or_else(|| {
        Error::Validation(format!(
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

fn required_bool(table: &dyn TableLike, key: &str, relative_path: &Path) -> Result<bool, Error> {
    let item = table.get(key).ok_or_else(|| {
        Error::Validation(format!(
            "approval artifact `{}` is missing required field `{key}`",
            relative_path.display()
        ))
    })?;
    item.as_bool().ok_or_else(|| {
        Error::Validation(format!(
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
) -> Result<Vec<String>, Error> {
    let Some(item) = table.get(key) else {
        if required {
            return Err(Error::Validation(format!(
                "approval artifact `{}` is missing required field `{key}`",
                relative_path.display()
            )));
        }
        return Ok(Vec::new());
    };
    let values = item.as_array().ok_or_else(|| {
        Error::Validation(format!(
            "approval artifact `{}` field `{key}` must be an array of strings",
            relative_path.display()
        ))
    })?;
    values
        .iter()
        .map(|value| {
            let value = value.as_str().ok_or_else(|| {
                Error::Validation(format!(
                    "approval artifact `{}` field `{key}` must contain only strings",
                    relative_path.display()
                ))
            })?;
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(Error::Validation(format!(
                    "approval artifact `{}` field `{key}` must not contain empty strings",
                    relative_path.display()
                )));
            }
            Ok(trimmed.to_string())
        })
        .collect()
}

fn target_gate_entries(table: &dyn TableLike, relative_path: &Path) -> Result<Vec<String>, Error> {
    let Some(item) = table.get("target_gated_capabilities") else {
        return Ok(Vec::new());
    };
    let entries = item.as_array_of_tables().ok_or_else(|| {
        Error::Validation(format!(
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

fn config_gate_entries(table: &dyn TableLike, relative_path: &Path) -> Result<Vec<String>, Error> {
    let Some(item) = table.get("config_gated_capabilities") else {
        return Ok(Vec::new());
    };
    let entries = item.as_array_of_tables().ok_or_else(|| {
        Error::Validation(format!(
            "approval artifact `{}` field `descriptor.config_gated_capabilities` must be an array of tables",
            relative_path.display()
        ))
    })?;
    entries
        .iter()
        .map(|entry| {
            let capability_id = required_string(entry, "capability_id", relative_path)?;
            let config_key = required_string(entry, "config_key", relative_path)?;
            let targets = optional_string_array(entry, "targets", relative_path)?;
            let mut encoded = format!("{capability_id}:{config_key}");
            if let Some(targets) = targets {
                encoded.push(':');
                encoded.push_str(&targets.join(","));
            }
            Ok(encoded)
        })
        .collect()
}

fn optional_string_array(
    table: &dyn TableLike,
    key: &str,
    relative_path: &Path,
) -> Result<Option<Vec<String>>, Error> {
    let Some(item) = table.get(key) else {
        return Ok(None);
    };
    let values = item.as_array().ok_or_else(|| {
        Error::Validation(format!(
            "approval artifact `{}` field `{key}` must be an array of strings",
            relative_path.display()
        ))
    })?;
    let parsed = values
        .iter()
        .map(|value| {
            let value = value.as_str().ok_or_else(|| {
                Error::Validation(format!(
                    "approval artifact `{}` field `{key}` must contain only strings",
                    relative_path.display()
                ))
            })?;
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(Error::Validation(format!(
                    "approval artifact `{}` field `{key}` must not contain empty strings",
                    relative_path.display()
                )));
            }
            Ok(trimmed.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(parsed))
}
