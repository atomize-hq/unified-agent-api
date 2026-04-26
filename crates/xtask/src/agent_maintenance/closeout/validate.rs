use std::{
    collections::BTreeSet,
    fs, io,
    path::{Component, Path, PathBuf},
};

use serde::Deserialize;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::workspace_mutation::WorkspacePathJail;

use super::super::{drift, request};
use super::{
    maintenance_pack_root, DeferredFindingsTruth, LinkedMaintenanceCloseout,
    LoadedMaintenanceRequest, MaintenanceCloseout, MaintenanceCloseoutError,
    MaintenanceDriftCategory, MaintenanceFinding,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMaintenanceCloseout {
    request_ref: Option<String>,
    request_sha256: Option<String>,
    resolved_findings: Option<Vec<RawMaintenanceFinding>>,
    deferred_findings: Option<Vec<RawMaintenanceFinding>>,
    explicit_none_reason: Option<String>,
    preflight_passed: bool,
    recorded_at: String,
    commit: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMaintenanceFinding {
    category_id: String,
    summary: String,
    surfaces: Vec<String>,
}

pub fn load_linked_closeout(
    workspace_root: &Path,
    request_path: &Path,
    closeout_path: &Path,
) -> Result<LinkedMaintenanceCloseout, MaintenanceCloseoutError> {
    let loaded_request = load_request_artifact(workspace_root, request_path)?;
    let request_path = loaded_request.request_path.clone();
    let maintenance_pack_prefix = loaded_request.maintenance_pack_prefix.clone();
    let closeout_path = validate_repo_relative_path(closeout_path, "closeout path")?;
    let closeout_pack_prefix = maintenance_pack_prefix_from_closeout_path(&closeout_path)?;
    if closeout_pack_prefix != maintenance_pack_prefix {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{} and {} must belong to the same maintenance pack",
            request_path.display(),
            closeout_path.display()
        )));
    }

    let jail = WorkspacePathJail::new(workspace_root)?;
    let resolved_closeout_path = jail.resolve(&closeout_path)?;
    let closeout_text =
        fs::read_to_string(&resolved_closeout_path).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => MaintenanceCloseoutError::Validation(format!(
                "read {}: {err}",
                closeout_path.display()
            )),
            _ => MaintenanceCloseoutError::Validation(format!(
                "read {}: {err}",
                closeout_path.display()
            )),
        })?;
    let raw_closeout =
        serde_json::from_str::<RawMaintenanceCloseout>(&closeout_text).map_err(|err| {
            MaintenanceCloseoutError::Validation(format!(
                "parse {}: {err}",
                closeout_path.display()
            ))
        })?;
    let closeout = validate_closeout(
        &closeout_path,
        &request_path,
        &loaded_request.request_sha256,
        raw_closeout,
    )?;
    validate_live_drift_truth(
        workspace_root,
        &closeout_path,
        &loaded_request.request.agent_id,
        &closeout,
    )?;
    let maintenance_pack_root = maintenance_pack_root(&maintenance_pack_prefix);

    Ok(LinkedMaintenanceCloseout {
        request: loaded_request.request.clone(),
        loaded_request,
        request_path: request_path.clone(),
        closeout_path: closeout_path.clone(),
        maintenance_pack_prefix: maintenance_pack_prefix.clone(),
        maintenance_pack_root,
        request_sha256: closeout.request_sha256.clone(),
        closeout,
    })
}

pub fn load_request_artifact(
    workspace_root: &Path,
    request_path: &Path,
) -> Result<LoadedMaintenanceRequest, MaintenanceCloseoutError> {
    let request_path = validate_repo_relative_path(request_path, "request path")?;
    let request = request::load_request(workspace_root, &request_path)
        .map_err(|err| map_request_error(&request_path, err))?;
    let maintenance_pack_prefix = request.maintenance_pack_prefix.clone();
    let maintenance_pack_root = PathBuf::from(&request.maintenance_root);

    Ok(LoadedMaintenanceRequest {
        request_path,
        maintenance_pack_prefix,
        maintenance_pack_root,
        request_sha256: request.sha256.clone(),
        request: request.into(),
    })
}

fn validate_closeout(
    closeout_path: &Path,
    expected_request_path: &Path,
    expected_request_sha256: &str,
    raw: RawMaintenanceCloseout,
) -> Result<MaintenanceCloseout, MaintenanceCloseoutError> {
    let request_ref = required_string(closeout_path, "request_ref", raw.request_ref.as_deref())?;
    let request_ref_path = validate_path_field(closeout_path, "request_ref", &request_ref)?;
    if Path::new(&request_ref_path) != expected_request_path {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{}: `request_ref` must equal `{}`",
            closeout_path.display(),
            expected_request_path.display()
        )));
    }

    let request_sha256 = required_string(
        closeout_path,
        "request_sha256",
        raw.request_sha256.as_deref(),
    )?;
    validate_sha256_shape(closeout_path, "request_sha256", &request_sha256)?;
    if request_sha256 != expected_request_sha256 {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{}: `request_sha256` does not match {}",
            closeout_path.display(),
            expected_request_path.display()
        )));
    }

    let resolved_findings = raw
        .resolved_findings
        .ok_or_else(|| {
            MaintenanceCloseoutError::Validation(format!(
                "{}: missing required field `resolved_findings`",
                closeout_path.display()
            ))
        })
        .and_then(|findings| validate_findings(closeout_path, "resolved_findings", findings))?;
    if resolved_findings.is_empty() {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{}: `resolved_findings` must not be empty",
            closeout_path.display()
        )));
    }

    let deferred_findings = match (raw.deferred_findings, raw.explicit_none_reason) {
        (Some(findings), None) => {
            let findings = validate_findings(closeout_path, "deferred_findings", findings)?;
            if findings.is_empty() {
                return Err(MaintenanceCloseoutError::Validation(format!(
                    "{}: `deferred_findings` must not be empty when present",
                    closeout_path.display()
                )));
            }
            DeferredFindingsTruth::Findings(findings)
        }
        (None, Some(reason)) => DeferredFindingsTruth::ExplicitNone(non_empty_field(
            closeout_path,
            "explicit_none_reason",
            &reason,
        )?),
        _ => {
            return Err(MaintenanceCloseoutError::Validation(format!(
                "{}: exactly one of `deferred_findings` or `explicit_none_reason` is required",
                closeout_path.display()
            )));
        }
    };

    validate_rfc3339_utc(closeout_path, "recorded_at", &raw.recorded_at)?;
    validate_commit_shape(closeout_path, "commit", &raw.commit)?;

    Ok(MaintenanceCloseout {
        request_ref,
        request_sha256,
        resolved_findings,
        deferred_findings,
        preflight_passed: raw.preflight_passed,
        recorded_at: raw.recorded_at,
        commit: raw.commit,
    })
}

pub(crate) fn validate_live_drift_truth(
    workspace_root: &Path,
    closeout_path: &Path,
    agent_id: &str,
    closeout: &MaintenanceCloseout,
) -> Result<(), MaintenanceCloseoutError> {
    let live_report = drift::check_agent_drift(workspace_root, agent_id);
    validate_live_drift_report(closeout_path, agent_id, closeout, live_report)
}

pub(crate) fn validate_live_drift_report(
    closeout_path: &Path,
    agent_id: &str,
    closeout: &MaintenanceCloseout,
    live_report: Result<drift::AgentDriftReport, drift::DriftCheckError>,
) -> Result<(), MaintenanceCloseoutError> {
    let live_report = live_report.map_err(|err| match err {
        drift::DriftCheckError::Validation(message) => {
            MaintenanceCloseoutError::Validation(format!(
                "{}: live drift re-check failed for `{agent_id}`: {message}",
                closeout_path.display()
            ))
        }
        drift::DriftCheckError::Internal(message) => MaintenanceCloseoutError::Internal(format!(
            "{}: live drift re-check failed for `{agent_id}`: {message}",
            closeout_path.display()
        )),
    })?;

    let live_signatures = live_report
        .findings
        .iter()
        .map(drift::DriftFinding::signature)
        .collect::<BTreeSet<_>>();
    let resolved_signatures = closeout
        .resolved_findings
        .iter()
        .map(MaintenanceFinding::signature)
        .collect::<BTreeSet<_>>();

    for signature in &resolved_signatures {
        if live_signatures.contains(signature) {
            return Err(MaintenanceCloseoutError::Validation(format!(
                "{}: `resolved_findings` still matches live drift for category `{}` on surfaces [{}]",
                closeout_path.display(),
                signature.category_id,
                signature.surfaces.join(", ")
            )));
        }
    }

    match &closeout.deferred_findings {
        DeferredFindingsTruth::ExplicitNone(_) => {
            if !live_signatures.is_empty() {
                return Err(MaintenanceCloseoutError::Validation(format!(
                    "{}: `explicit_none_reason` is only allowed when the live drift report is clean",
                    closeout_path.display()
                )));
            }
        }
        DeferredFindingsTruth::Findings(findings) => {
            if live_signatures.is_empty() {
                return Err(MaintenanceCloseoutError::Validation(format!(
                    "{}: `deferred_findings` must be empty when the live drift report is clean",
                    closeout_path.display()
                )));
            }

            let deferred_signatures = findings
                .iter()
                .map(MaintenanceFinding::signature)
                .collect::<BTreeSet<_>>();
            for signature in &live_signatures {
                if !deferred_signatures.contains(signature) {
                    return Err(MaintenanceCloseoutError::Validation(format!(
                        "{}: live drift category `{}` on surfaces [{}] is not accounted for in `deferred_findings`",
                        closeout_path.display(),
                        signature.category_id,
                        signature.surfaces.join(", ")
                    )));
                }
            }
            for signature in &deferred_signatures {
                if !live_signatures.contains(signature) {
                    return Err(MaintenanceCloseoutError::Validation(format!(
                        "{}: `deferred_findings` includes category `{}` on surfaces [{}] that is no longer present in the live drift report",
                        closeout_path.display(),
                        signature.category_id,
                        signature.surfaces.join(", ")
                    )));
                }
            }
        }
    }

    Ok(())
}

fn maintenance_pack_prefix_from_closeout_path(
    closeout_path: &Path,
) -> Result<String, MaintenanceCloseoutError> {
    let components = closeout_path.components().collect::<Vec<_>>();
    let expected_prefix = [
        Component::Normal("docs".as_ref()),
        Component::Normal("agents".as_ref()),
        Component::Normal("lifecycle".as_ref()),
    ];
    if components.len() != 6
        || components[0..3] != expected_prefix
        || components[4] != Component::Normal("governance".as_ref())
        || components[5] != Component::Normal("maintenance-closeout.json".as_ref())
    {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{} must point to docs/agents/lifecycle/<agent>-maintenance/governance/maintenance-closeout.json",
            closeout_path.display()
        )));
    }
    path_component_to_string(components[3], closeout_path)
}

fn path_component_to_string(
    component: Component<'_>,
    path: &Path,
) -> Result<String, MaintenanceCloseoutError> {
    let Component::Normal(value) = component else {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{} must use only normal path components",
            path.display()
        )));
    };
    Ok(value.to_string_lossy().into_owned())
}

fn validate_repo_relative_path(
    path: &Path,
    label: &str,
) -> Result<PathBuf, MaintenanceCloseoutError> {
    if path.components().next().is_none()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{label} `{}` must be a repo-relative path with only normal components",
            path.display()
        )));
    }
    Ok(path.to_path_buf())
}

fn validate_path_field(
    path: &Path,
    field_name: &str,
    value: &str,
) -> Result<String, MaintenanceCloseoutError> {
    let trimmed = non_empty_field(path, field_name, value)?;
    validate_repo_relative_path(Path::new(&trimmed), field_name)?;
    Ok(trimmed)
}

fn validate_findings(
    closeout_path: &Path,
    field_name: &str,
    findings: Vec<RawMaintenanceFinding>,
) -> Result<Vec<MaintenanceFinding>, MaintenanceCloseoutError> {
    findings
        .into_iter()
        .enumerate()
        .map(|(index, finding)| validate_finding(closeout_path, field_name, index, finding))
        .collect()
}

fn validate_finding(
    closeout_path: &Path,
    field_name: &str,
    index: usize,
    finding: RawMaintenanceFinding,
) -> Result<MaintenanceFinding, MaintenanceCloseoutError> {
    let category_id = MaintenanceDriftCategory::parse(&finding.category_id).ok_or_else(|| {
        MaintenanceCloseoutError::Validation(format!(
            "{}: `{field_name}[{index}].category_id` `{}` is not a valid maintenance drift category id",
            closeout_path.display(),
            finding.category_id
        ))
    })?;
    let summary = non_empty_field(
        closeout_path,
        &format!("{field_name}[{index}].summary"),
        &finding.summary,
    )?;
    if finding.surfaces.is_empty() {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{}: `{field_name}[{index}].surfaces` must not be empty",
            closeout_path.display()
        )));
    }
    let surfaces = finding
        .surfaces
        .into_iter()
        .map(|surface| {
            validate_path_field(
                closeout_path,
                &format!("{field_name}[{index}].surfaces[]"),
                &surface,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(MaintenanceFinding {
        category_id,
        summary,
        surfaces,
    })
}

fn map_request_error(
    request_path: &Path,
    err: request::MaintenanceRequestError,
) -> MaintenanceCloseoutError {
    match err {
        request::MaintenanceRequestError::Validation(message) => {
            MaintenanceCloseoutError::Validation(format!(
                "{}: unable to load linked request: {message}",
                request_path.display()
            ))
        }
        request::MaintenanceRequestError::Internal(message) => {
            MaintenanceCloseoutError::Internal(format!(
                "{}: unable to load linked request: {message}",
                request_path.display()
            ))
        }
    }
}

fn required_string(
    path: &Path,
    field_name: &str,
    value: Option<&str>,
) -> Result<String, MaintenanceCloseoutError> {
    let Some(value) = value else {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{}: missing required field `{field_name}`",
            path.display()
        )));
    };
    non_empty_field(path, field_name, value)
}

fn non_empty_field(
    path: &Path,
    field_name: &str,
    value: &str,
) -> Result<String, MaintenanceCloseoutError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{}: `{field_name}` must not be empty",
            path.display()
        )));
    }
    Ok(trimmed.to_string())
}

fn validate_sha256_shape(
    path: &Path,
    field_name: &str,
    value: &str,
) -> Result<(), MaintenanceCloseoutError> {
    let valid = value.len() == 64 && value.chars().all(|ch| matches!(ch, '0'..='9' | 'a'..='f'));
    if valid {
        Ok(())
    } else {
        Err(MaintenanceCloseoutError::Validation(format!(
            "{}: `{field_name}` must be 64 lowercase hex characters",
            path.display()
        )))
    }
}

fn validate_commit_shape(
    path: &Path,
    field_name: &str,
    value: &str,
) -> Result<(), MaintenanceCloseoutError> {
    let valid = (7..=40).contains(&value.len())
        && value.chars().all(|ch| matches!(ch, '0'..='9' | 'a'..='f'));
    if valid {
        Ok(())
    } else {
        Err(MaintenanceCloseoutError::Validation(format!(
            "{}: `{field_name}` must be 7-40 lowercase hex characters",
            path.display()
        )))
    }
}

fn validate_rfc3339_utc(
    path: &Path,
    field_name: &str,
    value: &str,
) -> Result<(), MaintenanceCloseoutError> {
    let parsed = OffsetDateTime::parse(value, &Rfc3339).map_err(|_| {
        MaintenanceCloseoutError::Validation(format!(
            "{}: `{field_name}` must be RFC3339 UTC",
            path.display()
        ))
    })?;
    if parsed.offset().whole_seconds() != 0 || !value.ends_with('Z') {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{}: `{field_name}` must be RFC3339 UTC",
            path.display()
        )));
    }
    Ok(())
}
