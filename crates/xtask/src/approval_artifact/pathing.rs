use std::{
    fs,
    path::{Component, Path},
};

use super::{
    ApprovalArtifactError, APPROVAL_FILE_NAME, DOCS_NEXT_ROOT, GOVERNANCE_DIR_NAME,
    STAGING_DIR_NAME,
};

#[derive(Debug, Clone)]
pub(super) struct ApprovalPath {
    pub(super) pack_prefix: String,
    pub(super) staged: bool,
}

pub(super) fn validate_approval_path(
    relative_path: &Path,
    allow_staged_paths: bool,
) -> Result<String, ApprovalArtifactError> {
    parse_approval_path(relative_path, allow_staged_paths).map(|path| path.pack_prefix)
}

pub(super) fn parse_approval_path(
    relative_path: &Path,
    allow_staged_paths: bool,
) -> Result<ApprovalPath, ApprovalArtifactError> {
    let components = relative_path.components().collect::<Vec<_>>();
    if components.is_empty()
        || components
            .iter()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(invalid_approval_path(relative_path, allow_staged_paths));
    }

    let rooted = relative_path
        .strip_prefix(DOCS_NEXT_ROOT)
        .map_err(|_| invalid_approval_path(relative_path, allow_staged_paths))?;
    let rooted_components = rooted.components().collect::<Vec<_>>();
    if rooted_components.len() == 3
        && matches!(
            rooted_components[1],
            Component::Normal(part) if part == GOVERNANCE_DIR_NAME
        )
        && matches!(
            rooted_components[2],
            Component::Normal(part) if part == APPROVAL_FILE_NAME
        )
    {
        let Component::Normal(prefix) = rooted_components[0] else {
            return Err(invalid_approval_path(relative_path, allow_staged_paths));
        };
        return Ok(ApprovalPath {
            pack_prefix: prefix.to_string_lossy().into_owned(),
            staged: false,
        });
    }

    if allow_staged_paths
        && rooted_components.len() == 5
        && matches!(
            rooted_components[0],
            Component::Normal(part) if part == STAGING_DIR_NAME
        )
        && matches!(
            rooted_components[3],
            Component::Normal(part) if part == GOVERNANCE_DIR_NAME
        )
        && matches!(
            rooted_components[4],
            Component::Normal(part) if part == APPROVAL_FILE_NAME
        )
    {
        let Component::Normal(prefix) = rooted_components[2] else {
            return Err(invalid_approval_path(relative_path, allow_staged_paths));
        };
        return Ok(ApprovalPath {
            pack_prefix: prefix.to_string_lossy().into_owned(),
            staged: true,
        });
    }

    Err(invalid_approval_path(relative_path, allow_staged_paths))
}

pub(super) fn validate_repo_relative_existing_file(
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

fn invalid_approval_path(relative_path: &Path, allow_staged_paths: bool) -> ApprovalArtifactError {
    let expected = if allow_staged_paths {
        format!(
            "`{DOCS_NEXT_ROOT}/<prefix>/{GOVERNANCE_DIR_NAME}/{APPROVAL_FILE_NAME}` or `{DOCS_NEXT_ROOT}/{STAGING_DIR_NAME}/<run_id>/<prefix>/{GOVERNANCE_DIR_NAME}/{APPROVAL_FILE_NAME}`"
        )
    } else {
        format!("`{DOCS_NEXT_ROOT}/<prefix>/{GOVERNANCE_DIR_NAME}/{APPROVAL_FILE_NAME}`")
    };
    ApprovalArtifactError::Validation(format!(
        "approval path `{}` must be repo-relative and match {expected}",
        relative_path.display()
    ))
}
