use std::path::Path;

use toml_edit::TableLike;

use super::ApprovalArtifactError;

pub(super) fn required_table<'a>(
    table: &'a dyn TableLike,
    key: &str,
    relative_path: &Path,
) -> Result<&'a dyn TableLike, ApprovalArtifactError> {
    let item = table.get(key).ok_or_else(|| {
        ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` is missing required table `{key}`",
            relative_path.display()
        ))
    })?;
    item.as_table_like().ok_or_else(|| {
        ApprovalArtifactError::Validation(format!(
            "approval artifact `{}` field `{key}` must be a table",
            relative_path.display()
        ))
    })
}

pub(super) fn required_string(
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

pub(super) fn optional_string(
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

pub(super) fn required_bool(
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

pub(super) fn string_array(
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

pub(super) fn target_gate_entries(
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

pub(super) fn config_gate_entries(
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
