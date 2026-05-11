use std::{fs, path::Path};

use crate::agent_lifecycle::{LifecycleState, PublicationReadyPacket};

use super::Error;

pub(super) fn load_lifecycle_state_relaxed(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<LifecycleState, Error> {
    let bytes = fs::read(workspace_root.join(relative_path))
        .map_err(|err| Error::Validation(format!("read {relative_path}: {err}")))?;
    let state: LifecycleState = serde_json::from_slice(&bytes)
        .map_err(|err| Error::Validation(format!("parse {relative_path}: {err}")))?;
    state
        .validate()
        .map_err(|err| Error::Validation(err.to_string()))?;
    Ok(state)
}

pub(super) fn load_publication_ready_packet_relaxed(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<PublicationReadyPacket, Error> {
    let bytes = fs::read(workspace_root.join(relative_path))
        .map_err(|err| Error::Validation(format!("read {relative_path}: {err}")))?;
    let packet: PublicationReadyPacket = serde_json::from_slice(&bytes)
        .map_err(|err| Error::Validation(format!("parse {relative_path}: {err}")))?;
    packet
        .validate()
        .map_err(|err| Error::Validation(err.to_string()))?;
    Ok(packet)
}

pub(super) fn serialize_json_pretty<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| Error::Internal(format!("serialize json: {err}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}
