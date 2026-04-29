use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use super::{
    models::{SnapshotFile, WorkspaceSnapshot},
    Error,
};

pub(super) fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), Error> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| Error::Internal(format!("serialize {}: {err}", path.display())))?;
    write_bytes(path, &bytes)
}

pub(super) fn write_string(path: &Path, value: &str) -> Result<(), Error> {
    write_bytes(path, value.as_bytes())
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| Error::Internal(format!("create {}: {err}", parent.display())))?;
    }
    fs::write(path, bytes)
        .map_err(|err| Error::Internal(format!("write {}: {err}", path.display())))
}

pub(super) fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, Error> {
    let bytes = fs::read(path)
        .map_err(|err| Error::Validation(format!("read {}: {err}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|err| Error::Validation(format!("parse {}: {err}", path.display())))
}

pub(super) fn snapshot_workspace(
    workspace_root: &Path,
    ignored_roots: &[&Path],
) -> Result<WorkspaceSnapshot, Error> {
    let mut files = Vec::new();
    let ignored = ignored_roots
        .iter()
        .map(|path| workspace_root.join(path))
        .collect::<Vec<_>>();
    collect_snapshot_files(workspace_root, workspace_root, &ignored, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(WorkspaceSnapshot { files })
}

fn collect_snapshot_files(
    workspace_root: &Path,
    current: &Path,
    ignored_roots: &[PathBuf],
    files: &mut Vec<SnapshotFile>,
) -> Result<(), Error> {
    let metadata = fs::symlink_metadata(current)
        .map_err(|err| Error::Internal(format!("stat {}: {err}", current.display())))?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if current != workspace_root {
        if let Ok(relative) = current.strip_prefix(workspace_root) {
            if ignored_roots
                .iter()
                .any(|root| current == root || current.starts_with(root))
            {
                return Ok(());
            }
            if relative == Path::new(".git") || relative.starts_with("target") {
                return Ok(());
            }
        }
    }

    if metadata.is_dir() {
        for entry in fs::read_dir(current)
            .map_err(|err| Error::Internal(format!("read_dir {}: {err}", current.display())))?
        {
            let entry = entry
                .map_err(|err| Error::Internal(format!("read_dir {}: {err}", current.display())))?;
            collect_snapshot_files(workspace_root, &entry.path(), ignored_roots, files)?;
        }
        return Ok(());
    }

    let relative = current
        .strip_prefix(workspace_root)
        .map_err(|err| Error::Internal(format!("strip prefix {}: {err}", current.display())))?;
    let bytes = fs::read(current)
        .map_err(|err| Error::Internal(format!("read {}: {err}", current.display())))?;
    files.push(SnapshotFile {
        path: relative.to_string_lossy().replace('\\', "/"),
        sha256: hex::encode(Sha256::digest(bytes)),
    });
    Ok(())
}

pub(super) fn diff_snapshots(before: &WorkspaceSnapshot, after: &WorkspaceSnapshot) -> Vec<String> {
    let before_map = before
        .files
        .iter()
        .map(|file| (file.path.as_str(), file.sha256.as_str()))
        .collect::<BTreeMap<_, _>>();
    let after_map = after
        .files
        .iter()
        .map(|file| (file.path.as_str(), file.sha256.as_str()))
        .collect::<BTreeMap<_, _>>();
    before_map
        .keys()
        .chain(after_map.keys())
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|path| before_map.get(path) != after_map.get(path))
        .map(str::to_string)
        .collect()
}

pub(super) fn generate_run_id() -> String {
    OffsetDateTime::now_utc()
        .format(
            &time::format_description::parse("[year][month][day]T[hour][minute][second]Z")
                .expect("valid time format"),
        )
        .unwrap_or_else(|_| "runtime-follow-on".to_string())
}

pub(super) fn now_rfc3339() -> Result<String, Error> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|err| Error::Internal(format!("format timestamp: {err}")))
}
