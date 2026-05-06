use std::path::{Component, Path, PathBuf};

use super::MaintenanceRequestError;

pub(super) fn normalize_repo_relative_path(
    workspace_root: &Path,
    path: &Path,
) -> Result<PathBuf, MaintenanceRequestError> {
    let relative = if path.is_absolute() {
        path.strip_prefix(workspace_root)
            .map(Path::to_path_buf)
            .map_err(|_| {
                MaintenanceRequestError::Validation(format!(
                    "maintenance request path `{}` must be inside workspace root {}",
                    path.display(),
                    workspace_root.display()
                ))
            })?
    } else {
        path.to_path_buf()
    };

    if relative.components().next().is_none()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request path `{}` must be a repo-relative path with only normal components",
            path.display()
        )));
    }

    Ok(relative)
}

pub(super) fn validate_request_path(
    relative_path: &Path,
) -> Result<String, MaintenanceRequestError> {
    let components = relative_path.components().collect::<Vec<_>>();
    let expected_prefix = [
        Component::Normal("docs".as_ref()),
        Component::Normal("agents".as_ref()),
        Component::Normal("lifecycle".as_ref()),
    ];

    if components.len() != 6
        || components[0..3] != expected_prefix
        || components[4] != Component::Normal("governance".as_ref())
        || components[5] != Component::Normal("maintenance-request.toml".as_ref())
    {
        return Err(MaintenanceRequestError::Validation(format!(
            "{} must point to docs/agents/lifecycle/<agent>-maintenance/governance/maintenance-request.toml",
            relative_path.display()
        )));
    }

    let Component::Normal(pack_prefix) = components[3] else {
        return Err(MaintenanceRequestError::Validation(format!(
            "{} must point to docs/agents/lifecycle/<agent>-maintenance/governance/maintenance-request.toml",
            relative_path.display()
        )));
    };
    let pack_prefix = pack_prefix.to_string_lossy().into_owned();
    if !pack_prefix.ends_with("-maintenance") {
        return Err(MaintenanceRequestError::Validation(format!(
            "{} must live under a maintenance root named `<agent>-maintenance`, not `{pack_prefix}`",
            relative_path.display()
        )));
    }

    Ok(pack_prefix)
}
