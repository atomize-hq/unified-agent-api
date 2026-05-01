use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use super::{validation, LifecycleError};

pub(super) fn now_rfc3339() -> Result<String, LifecycleError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|err| LifecycleError::Internal(format!("format timestamp: {err}")))
}

pub(super) fn file_sha256(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<String, LifecycleError> {
    let resolved = validation::resolve_repo_relative_path(workspace_root, relative_path)?;
    let bytes = fs::read(&resolved)
        .map_err(|err| LifecycleError::Internal(format!("read {}: {err}", resolved.display())))?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

pub(super) fn load_json_file<T: for<'de> Deserialize<'de>>(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<T, LifecycleError> {
    let resolved = validation::resolve_repo_relative_path(workspace_root, relative_path)?;
    let bytes = fs::read(&resolved)
        .map_err(|err| LifecycleError::Validation(format!("read {}: {err}", resolved.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|err| LifecycleError::Validation(format!("parse {}: {err}", resolved.display())))
}

pub(super) fn write_json_file<T: Serialize>(
    workspace_root: &Path,
    relative_path: &str,
    value: &T,
) -> Result<(), LifecycleError> {
    let resolved = validation::resolve_repo_relative_path_for_write(workspace_root, relative_path)?;
    if let Some(parent) = resolved.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            LifecycleError::Internal(format!("create {}: {err}", parent.display()))
        })?;
    }
    let mut json = serde_json::to_vec_pretty(value)
        .map_err(|err| LifecycleError::Internal(format!("serialize {relative_path}: {err}")))?;
    json.push(b'\n');
    fs::write(&resolved, json)
        .map_err(|err| LifecycleError::Internal(format!("write {}: {err}", resolved.display())))
}
