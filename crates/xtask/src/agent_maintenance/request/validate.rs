use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
};

use time::{format_description::well_known::Rfc3339, OffsetDateTime, UtcOffset};

use crate::workspace_mutation::WorkspacePathJail;

use super::{
    raw::RawRuntimeFollowupRequired, MaintenanceAction, MaintenanceRequestError,
    RuntimeFollowupRequired,
};

pub(crate) fn validate_non_empty_scalar(
    request_path: &Path,
    field_name: &str,
    value: &str,
) -> Result<(), MaintenanceRequestError> {
    if value.trim().is_empty() {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must be a non-empty string",
            request_path.display()
        )));
    }
    Ok(())
}

pub(crate) fn validate_repo_relative_reference(
    jail: &WorkspacePathJail,
    request_path: &Path,
    field_name: &str,
    value: &str,
) -> Result<(), MaintenanceRequestError> {
    validate_non_empty_scalar(request_path, field_name, value)?;
    let relative = PathBuf::from(value);
    if relative.components().next().is_none()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must be a repo-relative path with only normal components",
            request_path.display()
        )));
    }

    let resolved = jail.resolve(&relative).map_err(map_jail_error)?;
    let metadata = fs::symlink_metadata(&resolved).map_err(|err| {
        MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must point to an existing file: {err}",
            request_path.display()
        ))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must point to an existing file",
            request_path.display()
        )));
    }
    Ok(())
}

pub(crate) fn validate_actions(
    request_path: &Path,
    values: &[String],
) -> Result<Vec<MaintenanceAction>, MaintenanceRequestError> {
    if values.is_empty() {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `requested_control_plane_actions` must be a non-empty array",
            request_path.display()
        )));
    }

    let mut seen = BTreeSet::new();
    let mut parsed = Vec::with_capacity(values.len());
    for value in values {
        let action = MaintenanceAction::parse(value, request_path)?;
        if !seen.insert(action) {
            return Err(MaintenanceRequestError::Validation(format!(
                "maintenance request `{}` field `requested_control_plane_actions` contains duplicate action `{}`",
                request_path.display(),
                action.as_str()
            )));
        }
        parsed.push(action);
    }
    Ok(parsed)
}

pub(super) fn validate_runtime_followup_required(
    request_path: &Path,
    raw: RawRuntimeFollowupRequired,
) -> Result<RuntimeFollowupRequired, MaintenanceRequestError> {
    let mut items = Vec::with_capacity(raw.items.len());
    for item in raw.items {
        if item.trim().is_empty() {
            return Err(MaintenanceRequestError::Validation(format!(
                "maintenance request `{}` field `runtime_followup_required.items` must not contain blank entries",
                request_path.display()
            )));
        }
        items.push(item);
    }

    if raw.required && items.is_empty() {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` has `runtime_followup_required.required = true` but no follow-up items were provided",
            request_path.display()
        )));
    }
    if !raw.required && !items.is_empty() {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` has `runtime_followup_required.required = false` and therefore requires `items = []`",
            request_path.display()
        )));
    }

    Ok(RuntimeFollowupRequired {
        required: raw.required,
        items,
    })
}

pub(crate) fn validate_rfc3339_utc(
    request_path: &Path,
    field_name: &str,
    value: &str,
) -> Result<(), MaintenanceRequestError> {
    let parsed = OffsetDateTime::parse(value, &Rfc3339).map_err(|err| {
        MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must be RFC3339 UTC: {err}",
            request_path.display()
        ))
    })?;
    if parsed.offset() != UtcOffset::UTC {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must use UTC (`Z`) offset",
            request_path.display()
        )));
    }
    Ok(())
}

pub(crate) fn validate_commit_value(
    request_path: &Path,
    field_name: &str,
    value: &str,
) -> Result<(), MaintenanceRequestError> {
    let is_valid = (7..=40).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'));
    if !is_valid {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must be 7-40 lowercase hex characters",
            request_path.display()
        )));
    }
    Ok(())
}

pub(super) fn validate_sha256_value(
    request_path: &Path,
    field_name: &str,
    value: &str,
) -> Result<(), MaintenanceRequestError> {
    let is_valid = value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'));
    if !is_valid {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must be 64 lowercase hex characters",
            request_path.display()
        )));
    }
    Ok(())
}

pub(super) fn validate_repo_relative_glob_path(
    request_path: &Path,
    field_name: &str,
    value: &str,
) -> Result<(), MaintenanceRequestError> {
    validate_non_empty_scalar(request_path, field_name, value)?;
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().next().is_none()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must be a repo-relative path with only normal components",
            request_path.display()
        )));
    }
    Ok(())
}

pub(super) fn validate_repo_relative_string_array(
    request_path: &Path,
    field_name: &str,
    values: &[String],
    require_non_empty: bool,
) -> Result<Vec<String>, MaintenanceRequestError> {
    if require_non_empty && values.is_empty() {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must be a non-empty array",
            request_path.display()
        )));
    }
    let mut parsed = Vec::with_capacity(values.len());
    for value in values {
        validate_repo_relative_glob_path(request_path, field_name, value)?;
        parsed.push(value.clone());
    }
    Ok(parsed)
}

pub(super) fn validate_existing_repo_relative_string_array(
    jail: &WorkspacePathJail,
    request_path: &Path,
    field_name: &str,
    values: &[String],
) -> Result<Vec<String>, MaintenanceRequestError> {
    let mut parsed = Vec::with_capacity(values.len());
    for value in values {
        validate_repo_relative_reference(jail, request_path, field_name, value)?;
        parsed.push(value.clone());
    }
    Ok(parsed)
}

pub(super) fn validate_non_empty_string_array(
    request_path: &Path,
    field_name: &str,
    values: &[String],
    require_non_empty: bool,
) -> Result<Vec<String>, MaintenanceRequestError> {
    if require_non_empty && values.is_empty() {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must be a non-empty array",
            request_path.display()
        )));
    }
    let mut parsed = Vec::with_capacity(values.len());
    for value in values {
        if value.trim().is_empty() {
            return Err(MaintenanceRequestError::Validation(format!(
                "maintenance request `{}` field `{field_name}` must not contain blank entries",
                request_path.display()
            )));
        }
        parsed.push(value.clone());
    }
    Ok(parsed)
}

pub(super) fn map_jail_error(
    err: crate::workspace_mutation::WorkspaceMutationError,
) -> MaintenanceRequestError {
    match err {
        crate::workspace_mutation::WorkspaceMutationError::Validation(message) => {
            MaintenanceRequestError::Validation(message)
        }
        crate::workspace_mutation::WorkspaceMutationError::Internal(message) => {
            MaintenanceRequestError::Internal(message)
        }
    }
}
