use std::{fs, path::Path};

use serde::Deserialize;

use crate::agent_lifecycle::maintenance_request_path;

use super::{repo_root, Args, Error};

#[derive(Debug, Deserialize)]
struct RecordedRequest {
    agent_id: Option<String>,
    trigger_kind: Option<String>,
    opened_from: Option<String>,
    request_recorded_at: Option<String>,
    request_commit: Option<String>,
    detected_release: Option<RecordedRelease>,
}

#[derive(Debug, Deserialize)]
struct RecordedRelease {
    current_validated: Option<String>,
    latest_stable: Option<String>,
    target_version: Option<String>,
    detected_by: Option<String>,
    dispatch_kind: Option<String>,
    dispatch_workflow: Option<String>,
    branch_name: Option<String>,
}

pub fn args_from_request(request_path: &Path, dry_run: bool, write: bool) -> Result<Args, Error> {
    args_from_request_in_workspace(&repo_root(), request_path, dry_run, write)
}

pub fn args_from_request_in_workspace(
    workspace_root: &Path,
    request_path: &Path,
    dry_run: bool,
    write: bool,
) -> Result<Args, Error> {
    let display = request_path.display();
    if request_path.is_absolute()
        || request_path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(Error::Validation(format!(
            "prepare-agent-maintenance field `from_request` path `{display}` must be repo-relative without `.` or `..`"
        )));
    }
    let text = fs::read_to_string(workspace_root.join(request_path)).map_err(|err| {
        Error::Validation(format!(
            "prepare-agent-maintenance field `from_request` path `{display}` cannot be read: {err}"
        ))
    })?;
    let recorded = toml_edit::de::from_str::<RecordedRequest>(&text).map_err(|err| {
        Error::Validation(format!(
            "prepare-agent-maintenance field `from_request` path `{display}` does not parse as TOML: {err}"
        ))
    })?;

    let agent = required(recorded.agent_id, "agent_id", request_path)?;
    let trigger_kind = required(recorded.trigger_kind, "trigger_kind", request_path)?;
    if trigger_kind != "upstream_release_detected" {
        return Err(Error::Validation(format!(
            "prepare-agent-maintenance field `trigger_kind` in `{display}` must be `upstream_release_detected` (got `{trigger_kind}`)"
        )));
    }
    let expected_path = maintenance_request_path(&agent);
    if request_path != Path::new(&expected_path) {
        return Err(Error::Validation(format!(
            "prepare-agent-maintenance field `from_request` path `{display}` must equal `{expected_path}` for agent_id `{agent}`"
        )));
    }

    let opened_from = required(recorded.opened_from, "opened_from", request_path)?.into();
    let request_recorded_at = required(
        recorded.request_recorded_at,
        "request_recorded_at",
        request_path,
    )?;
    let request_commit = required(recorded.request_commit, "request_commit", request_path)?;
    let release = recorded
        .detected_release
        .ok_or_else(|| missing_field("detected_release", request_path))?;

    Ok(Args {
        agent,
        current_version: required(
            release.current_validated,
            "detected_release.current_validated",
            request_path,
        )?,
        latest_stable: required(
            release.latest_stable,
            "detected_release.latest_stable",
            request_path,
        )?,
        target_version: required(
            release.target_version,
            "detected_release.target_version",
            request_path,
        )?,
        opened_from,
        detected_by: required(
            release.detected_by,
            "detected_release.detected_by",
            request_path,
        )?,
        dispatch_kind: required(
            release.dispatch_kind,
            "detected_release.dispatch_kind",
            request_path,
        )?,
        dispatch_workflow: release.dispatch_workflow,
        branch_name: required(
            release.branch_name,
            "detected_release.branch_name",
            request_path,
        )?,
        request_recorded_at,
        request_commit,
        dry_run,
        write,
    })
}

fn required(value: Option<String>, field: &str, path: &Path) -> Result<String, Error> {
    value.ok_or_else(|| missing_field(field, path))
}

fn missing_field(field: &str, path: &Path) -> Error {
    Error::Validation(format!(
        "prepare-agent-maintenance field `{field}` is missing from `{}`",
        path.display()
    ))
}
