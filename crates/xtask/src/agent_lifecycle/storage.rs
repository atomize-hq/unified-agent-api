use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use super::{support_depth, validation, LifecycleError, LifecycleState, PublicationReadyPacket};

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

/// Reads a lifecycle state, refusing depth fields held in any form but the one a writer writes.
pub(super) fn load_lifecycle_state_file(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<LifecycleState, LifecycleError> {
    let resolved = validation::resolve_repo_relative_path(workspace_root, relative_path)?;
    let bytes = fs::read(&resolved)
        .map_err(|err| LifecycleError::Validation(format!("read {}: {err}", resolved.display())))?;
    parse_lifecycle_state(&bytes)
        .map_err(|err| LifecycleError::Validation(format!("{}: {err}", resolved.display())))
}

/// Parses lifecycle state bytes, refusing depth fields held in any form but the one a writer
/// writes, so that every writer that re-serializes the state preserves them byte for byte.
pub fn parse_lifecycle_state(bytes: &[u8]) -> Result<LifecycleState, LifecycleError> {
    let state = serde_json::from_slice(bytes)
        .map_err(|err| LifecycleError::Validation(format!("parse lifecycle state: {err}")))?;
    support_depth::require_written_form(bytes, &state)?;
    Ok(state)
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

pub fn write_lifecycle_state(
    workspace_root: &Path,
    relative_path: &str,
    state: &LifecycleState,
) -> Result<(), LifecycleError> {
    xtask::depth_routes::refuse_workspace(workspace_root, "lifecycle writer")
        .map_err(LifecycleError::Validation)?;
    state.validate()?;
    xtask::depth_routes::refuse_output(
        workspace_root,
        Path::new(relative_path),
        &serde_json::to_vec(state).map_err(|e| LifecycleError::Internal(e.to_string()))?,
        "lifecycle writer",
    )
    .map_err(LifecycleError::Validation)?;
    let destination =
        validation::resolve_repo_relative_path_for_write(workspace_root, relative_path)?;
    let previous = match fs::read(&destination) {
        Ok(bytes) => Some(bytes),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(err) => {
            return Err(LifecycleError::Internal(format!(
                "read {}: {err}",
                destination.display()
            )))
        }
    };
    support_depth::require_preserved(previous.as_deref(), state)?;
    write_json_file(workspace_root, relative_path, state)
}

pub fn write_publication_ready_packet(
    workspace_root: &Path,
    relative_path: &str,
    packet: &PublicationReadyPacket,
) -> Result<(), LifecycleError> {
    xtask::depth_routes::refuse_workspace(workspace_root, "publication packet writer")
        .map_err(LifecycleError::Validation)?;
    packet.validate()?;
    xtask::depth_routes::refuse_output(
        workspace_root,
        Path::new(relative_path),
        &serde_json::to_vec(packet).map_err(|e| LifecycleError::Internal(e.to_string()))?,
        "publication packet writer",
    )
    .map_err(LifecycleError::Validation)?;
    write_json_file(workspace_root, relative_path, packet)
}
