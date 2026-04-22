use std::{
    collections::BTreeSet,
    fs, io,
    path::{Component, Path, PathBuf},
};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::agent_registry::{AgentRegistry, AgentRegistryError, REGISTRY_RELATIVE_PATH};
use crate::workspace_mutation::WorkspacePathJail;

use super::{
    maintenance_pack_root, DeferredFindingsTruth, LinkedMaintenanceCloseout,
    LoadedMaintenanceRequest, MaintenanceCloseout, MaintenanceCloseoutError,
    MaintenanceControlPlaneAction, MaintenanceDriftCategory, MaintenanceFinding,
    MaintenanceRequest, MaintenanceTriggerKind, RuntimeFollowupRequired,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMaintenanceRequest {
    artifact_version: String,
    agent_id: String,
    trigger_kind: String,
    basis_ref: String,
    opened_from: String,
    requested_control_plane_actions: Vec<String>,
    runtime_followup_required: RawRuntimeFollowupRequired,
    request_recorded_at: String,
    request_commit: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRuntimeFollowupRequired {
    required: bool,
    items: Vec<String>,
}

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
    let request = closeout_request(&closeout_path, workspace_root, &request_path)?;
    let maintenance_pack_root = maintenance_pack_root(&maintenance_pack_prefix);

    Ok(LinkedMaintenanceCloseout {
        loaded_request,
        request_path: request_path.clone(),
        closeout_path: closeout_path.clone(),
        maintenance_pack_prefix: maintenance_pack_prefix.clone(),
        maintenance_pack_root,
        request_sha256: closeout.request_sha256.clone(),
        request,
        closeout,
    })
}

pub fn load_request_artifact(
    workspace_root: &Path,
    request_path: &Path,
) -> Result<LoadedMaintenanceRequest, MaintenanceCloseoutError> {
    let request_path = validate_repo_relative_path(request_path, "request path")?;
    let maintenance_pack_prefix = maintenance_pack_prefix_from_request_path(&request_path)?;
    let maintenance_pack_root = maintenance_pack_root(&maintenance_pack_prefix);
    let jail = WorkspacePathJail::new(workspace_root)?;
    let resolved_request_path = jail.resolve(&request_path)?;
    let request_bytes = load_request_bytes(&resolved_request_path, &request_path)?;
    let request_text = std::str::from_utf8(&request_bytes).map_err(|err| {
        MaintenanceCloseoutError::Validation(format!(
            "{}: request artifact must be valid utf-8: {err}",
            request_path.display()
        ))
    })?;
    let raw_request =
        toml_edit::de::from_str::<RawMaintenanceRequest>(request_text).map_err(|err| {
            MaintenanceCloseoutError::Validation(format!("parse {}: {err}", request_path.display()))
        })?;
    let request = validate_request(
        workspace_root,
        &request_path,
        &maintenance_pack_prefix,
        raw_request,
    )?;

    Ok(LoadedMaintenanceRequest {
        request_path,
        maintenance_pack_prefix,
        maintenance_pack_root,
        request_sha256: hex::encode(Sha256::digest(&request_bytes)),
        request,
    })
}

fn load_request_bytes(
    resolved_request_path: &Path,
    request_path: &Path,
) -> Result<Vec<u8>, MaintenanceCloseoutError> {
    fs::read(resolved_request_path).map_err(|err| match err.kind() {
        io::ErrorKind::NotFound => {
            MaintenanceCloseoutError::Validation(format!("read {}: {err}", request_path.display()))
        }
        _ => {
            MaintenanceCloseoutError::Validation(format!("read {}: {err}", request_path.display()))
        }
    })
}

fn closeout_request(
    closeout_path: &Path,
    workspace_root: &Path,
    request_path: &Path,
) -> Result<MaintenanceRequest, MaintenanceCloseoutError> {
    load_request_artifact(workspace_root, request_path)
        .map(|loaded| loaded.request)
        .map_err(|err| match err {
            MaintenanceCloseoutError::Validation(message) => {
                MaintenanceCloseoutError::Validation(format!(
                    "{}: unable to reload linked request: {message}",
                    closeout_path.display()
                ))
            }
            MaintenanceCloseoutError::Internal(message) => {
                MaintenanceCloseoutError::Internal(format!(
                    "{}: unable to reload linked request: {message}",
                    closeout_path.display()
                ))
            }
        })
}

fn validate_request(
    workspace_root: &Path,
    request_path: &Path,
    maintenance_pack_prefix: &str,
    raw: RawMaintenanceRequest,
) -> Result<MaintenanceRequest, MaintenanceCloseoutError> {
    if raw.artifact_version != "1" {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{}: `artifact_version` must equal `1`",
            request_path.display()
        )));
    }

    let registry = AgentRegistry::load(workspace_root).map_err(|err| {
        map_registry_error(request_path, err, format!("load {REGISTRY_RELATIVE_PATH}"))
    })?;
    let agent_id = non_empty_field(request_path, "agent_id", &raw.agent_id)?;
    if registry.find(&agent_id).is_none() {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{}: `agent_id` `{agent_id}` does not exist in {}",
            request_path.display(),
            REGISTRY_RELATIVE_PATH
        )));
    }
    let expected_pack_prefix = format!("{agent_id}-maintenance");
    if expected_pack_prefix != maintenance_pack_prefix {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{}: maintenance pack prefix `{maintenance_pack_prefix}` does not match agent `{agent_id}`",
            request_path.display()
        )));
    }

    let trigger_kind =
        MaintenanceTriggerKind::parse(&raw.trigger_kind).ok_or_else(|| {
            MaintenanceCloseoutError::Validation(format!(
                "{}: `trigger_kind` must be one of `drift_detected`, `manual_reopen`, or `post_release_audit`",
                request_path.display()
            ))
        })?;
    let basis_ref = validate_path_field(request_path, "basis_ref", &raw.basis_ref)?;
    let opened_from = validate_path_field(request_path, "opened_from", &raw.opened_from)?;
    let requested_control_plane_actions =
        validate_requested_actions(request_path, &raw.requested_control_plane_actions)?;
    let runtime_followup_required =
        validate_runtime_followup(request_path, raw.runtime_followup_required)?;
    validate_rfc3339_utc(
        request_path,
        "request_recorded_at",
        &raw.request_recorded_at,
    )?;
    validate_commit_shape(request_path, "request_commit", &raw.request_commit)?;

    Ok(MaintenanceRequest {
        agent_id,
        trigger_kind,
        basis_ref,
        opened_from,
        requested_control_plane_actions,
        runtime_followup_required,
        request_recorded_at: raw.request_recorded_at,
        request_commit: raw.request_commit,
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

fn maintenance_pack_prefix_from_request_path(
    request_path: &Path,
) -> Result<String, MaintenanceCloseoutError> {
    let components = request_path.components().collect::<Vec<_>>();
    let expected_prefix = [
        Component::Normal("docs".as_ref()),
        Component::Normal("project_management".as_ref()),
        Component::Normal("next".as_ref()),
    ];
    if components.len() != 6
        || components[0..3] != expected_prefix
        || components[4] != Component::Normal("governance".as_ref())
        || components[5] != Component::Normal("maintenance-request.toml".as_ref())
    {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{} must point to docs/project_management/next/<agent>-maintenance/governance/maintenance-request.toml",
            request_path.display()
        )));
    }
    path_component_to_string(components[3], request_path)
}

fn maintenance_pack_prefix_from_closeout_path(
    closeout_path: &Path,
) -> Result<String, MaintenanceCloseoutError> {
    let components = closeout_path.components().collect::<Vec<_>>();
    let expected_prefix = [
        Component::Normal("docs".as_ref()),
        Component::Normal("project_management".as_ref()),
        Component::Normal("next".as_ref()),
    ];
    if components.len() != 6
        || components[0..3] != expected_prefix
        || components[4] != Component::Normal("governance".as_ref())
        || components[5] != Component::Normal("maintenance-closeout.json".as_ref())
    {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{} must point to docs/project_management/next/<agent>-maintenance/governance/maintenance-closeout.json",
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

fn validate_requested_actions(
    request_path: &Path,
    requested_control_plane_actions: &[String],
) -> Result<Vec<MaintenanceControlPlaneAction>, MaintenanceCloseoutError> {
    if requested_control_plane_actions.is_empty() {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{}: `requested_control_plane_actions` must not be empty",
            request_path.display()
        )));
    }

    let mut seen = BTreeSet::new();
    let mut actions = Vec::with_capacity(requested_control_plane_actions.len());
    for action in requested_control_plane_actions {
        let action = non_empty_field(request_path, "requested_control_plane_actions[]", action)?;
        if !seen.insert(action.clone()) {
            return Err(MaintenanceCloseoutError::Validation(format!(
                "{}: `requested_control_plane_actions` must not contain duplicates",
                request_path.display()
            )));
        }
        let parsed = MaintenanceControlPlaneAction::parse(&action).ok_or_else(|| {
            MaintenanceCloseoutError::Validation(format!(
                "{}: `requested_control_plane_actions` contains unsupported action `{action}`",
                request_path.display()
            ))
        })?;
        actions.push(parsed);
    }
    Ok(actions)
}

fn validate_runtime_followup(
    request_path: &Path,
    runtime_followup_required: RawRuntimeFollowupRequired,
) -> Result<RuntimeFollowupRequired, MaintenanceCloseoutError> {
    let items = runtime_followup_required
        .items
        .into_iter()
        .map(|item| non_empty_field(request_path, "runtime_followup_required.items[]", &item))
        .collect::<Result<Vec<_>, _>>()?;

    if runtime_followup_required.required {
        if items.is_empty() {
            return Err(MaintenanceCloseoutError::Validation(format!(
                "{}: `runtime_followup_required.items` must not be empty when `required = true`",
                request_path.display()
            )));
        }
    } else if !items.is_empty() {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{}: `runtime_followup_required.items` must be empty when `required = false`",
            request_path.display()
        )));
    }

    Ok(RuntimeFollowupRequired {
        required: runtime_followup_required.required,
        items,
    })
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

fn map_registry_error(
    request_path: &Path,
    err: AgentRegistryError,
    context: String,
) -> MaintenanceCloseoutError {
    match err {
        AgentRegistryError::Validation(message) => MaintenanceCloseoutError::Validation(format!(
            "{}: {context}: {message}",
            request_path.display()
        )),
        _ => MaintenanceCloseoutError::Internal(format!(
            "{}: {context}: {err}",
            request_path.display()
        )),
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
