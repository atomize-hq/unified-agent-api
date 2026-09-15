use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::agent_registry::AgentRegistryEntry;

use super::super::{request::DetectedRelease, support_audit};
use super::Error;

#[derive(Debug, Deserialize)]
struct UnionSnapshotEvidence {
    complete: Option<bool>,
    expected_targets: Option<Vec<String>>,
    missing_targets: Option<Vec<String>>,
}

pub(super) fn require_live_acquisition_evidence(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    detected_release: &DetectedRelease,
) -> Result<(), Error> {
    require_complete_union_snapshot(workspace_root, entry, &detected_release.target_version)?;

    let report_dir =
        coverage_report_version_dir(workspace_root, entry, &detected_release.target_version);
    let report_dir_display = format!(
        "{}/reports/{}",
        entry.manifest_root, detected_release.target_version
    );

    if !report_dir.is_dir() {
        return Err(Error::Validation(format!(
            "maintenance-audit-status requires live coverage report evidence for target version `{}` under `{}` before reporting audit status",
            detected_release.target_version, report_dir_display
        )));
    }

    let selected_report_path = match selected_coverage_report_path(&report_dir) {
        Ok(path) => path,
        Err(Error::Validation(message))
            if message.starts_with("no coverage report found under") =>
        {
            return Err(Error::Validation(format!(
                "maintenance-audit-status requires live coverage report evidence for target version `{}` under `{}` before reporting audit status",
                detected_release.target_version, report_dir_display
            )));
        }
        Err(error) => return Err(error),
    };
    let mut report_paths = vec![selected_report_path.clone()];
    for report_path in list_coverage_report_paths(&report_dir)? {
        if report_path != selected_report_path {
            report_paths.push(report_path);
        }
    }

    for report_path in report_paths {
        validate_live_coverage_report_target_version(
            &report_path,
            &detected_release.target_version,
        )?;
    }

    Ok(())
}

fn coverage_report_version_dir(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    target_version: &str,
) -> PathBuf {
    workspace_root
        .join(&entry.manifest_root)
        .join("reports")
        .join(target_version)
}

fn union_snapshot_path(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    target_version: &str,
) -> PathBuf {
    workspace_root
        .join(&entry.manifest_root)
        .join("snapshots")
        .join(target_version)
        .join("union.json")
}

fn require_complete_union_snapshot(
    workspace_root: &Path,
    entry: &AgentRegistryEntry,
    target_version: &str,
) -> Result<(), Error> {
    let union_path = union_snapshot_path(workspace_root, entry, target_version);
    let union_path_display = format!(
        "{}/snapshots/{target_version}/union.json",
        entry.manifest_root
    );

    let text = match fs::read_to_string(&union_path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(Error::Validation(format!(
                "maintenance-audit-status requires live acquisition union snapshot `{}` for target version `{}`",
                union_path_display, target_version
            )));
        }
        Err(error) => {
            return Err(Error::Internal(format!(
                "maintenance-audit-status could not read live acquisition union snapshot `{}`: {}",
                union_path_display, error
            )));
        }
    };

    let snapshot = serde_json::from_str::<UnionSnapshotEvidence>(&text).map_err(|error| {
        Error::Validation(format!(
            "maintenance-audit-status requires live acquisition union snapshot `{}` to be valid JSON with bool `complete`: {}",
            union_path_display, error
        ))
    })?;
    let complete = snapshot.complete.ok_or_else(|| {
        Error::Validation(format!(
            "maintenance-audit-status requires live acquisition union snapshot `{}` to declare bool `complete`",
            union_path_display
        ))
    })?;
    if complete {
        return Ok(());
    }

    let incomplete_detail = if let Some(missing_targets) = snapshot
        .missing_targets
        .as_ref()
        .filter(|targets| !targets.is_empty())
    {
        format!("missing targets: {}", missing_targets.join(", "))
    } else if let Some(expected_targets) = snapshot
        .expected_targets
        .as_ref()
        .filter(|targets| !targets.is_empty())
    {
        format!(
            "`complete` is false and `missing_targets` is absent; expected targets are {}",
            expected_targets.join(", ")
        )
    } else {
        "`complete` is false and `missing_targets` is absent while `expected_targets` is missing or empty".to_string()
    };

    Err(Error::IncompleteAcquisition(format!(
        "maintenance-audit-status requires live acquisition union snapshot `{}` for target version `{}` to have `complete=true`, but {}",
        union_path_display, target_version, incomplete_detail
    )))
}

pub(crate) fn selected_coverage_report_path(version_dir: &Path) -> Result<PathBuf, Error> {
    support_audit::select_report_path(version_dir).map_err(|err| {
        if err.starts_with("no coverage report found under") {
            Error::Validation(err)
        } else {
            Error::Internal(format!(
                "select live coverage report under {}: {err}",
                version_dir.display()
            ))
        }
    })
}

fn list_coverage_report_paths(version_dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut candidates = Vec::new();
    for entry in fs::read_dir(version_dir)
        .map_err(|err| Error::Internal(format!("read_dir({}): {err}", version_dir.display())))?
    {
        let entry = entry.map_err(|err| {
            Error::Internal(format!("read_dir({}): {err}", version_dir.display()))
        })?;
        let path = entry.path();
        if is_coverage_report_path(&path)? {
            candidates.push(path);
        }
    }
    candidates.sort();
    Ok(candidates)
}

fn is_coverage_report_path(path: &Path) -> Result<bool, Error> {
    let Some(file_name) = path.file_name() else {
        return Ok(false);
    };
    let Some(file_name) = file_name.to_str() else {
        return Err(Error::Validation(format!(
            "maintenance-audit-status encountered non-UTF-8 live coverage report entry `{}`; report filenames must be UTF-8",
            path.display()
        )));
    };
    let normalized = file_name.to_ascii_lowercase();
    if !(normalized.starts_with("coverage.") && normalized.ends_with(".json")) {
        return Ok(false);
    }

    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => Ok(true),
        Ok(_) => Err(Error::Validation(format!(
            "maintenance-audit-status requires live coverage report entry `{}` to resolve to a regular file",
            path.display()
        ))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Err(Error::Validation(format!(
            "maintenance-audit-status requires live coverage report entry `{}` to resolve to a regular file",
            path.display()
        ))),
        Err(error) => Err(Error::Internal(format!("stat {}: {error}", path.display()))),
    }
}

fn read_live_coverage_report_target_version(report_path: &Path) -> Result<String, Error> {
    let text = fs::read_to_string(report_path)
        .map_err(|err| Error::Internal(format!("read {}: {err}", report_path.display())))?;
    let json = serde_json::from_str::<serde_json::Value>(&text)
        .map_err(|err| Error::Validation(format!("parse {}: {err}", report_path.display())))?;
    json.pointer("/inputs/upstream/semantic_version")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| {
            Error::Validation(format!(
                "maintenance-audit-status requires live coverage report `{}` to declare string `inputs.upstream.semantic_version`",
                report_path.display()
            ))
        })
}

fn validate_live_coverage_report_target_version(
    report_path: &Path,
    target_version: &str,
) -> Result<(), Error> {
    let actual_version = read_live_coverage_report_target_version(report_path)?;
    if actual_version != target_version {
        return Err(Error::Validation(format!(
            "maintenance-audit-status requires live coverage report `{}` to match detected release target version `{}`, but `inputs.upstream.semantic_version` is `{}`",
            report_path.display(),
            target_version,
            actual_version
        )));
    }

    Ok(())
}
