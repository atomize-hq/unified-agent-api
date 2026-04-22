use std::{fmt, fs, io, path::Path};

use crate::approval_artifact::{self, ApprovalArtifact, ApprovalArtifactError};
use serde::Deserialize;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

#[derive(Debug, Clone)]
pub struct ProvingRunCloseout {
    pub approval_ref: String,
    pub approval_sha256: String,
    pub approval_source: String,
    pub manual_control_plane_edits: u64,
    pub partial_write_incidents: u64,
    pub ambiguous_ownership_incidents: u64,
    pub duration: DurationTruth,
    pub residual_friction: ResidualFrictionTruth,
    pub preflight_passed: bool,
    pub recorded_at: String,
    pub commit: String,
}

#[derive(Debug, Clone)]
pub enum DurationTruth {
    Seconds(u64),
    MissingReason(String),
}

#[derive(Debug, Clone)]
pub enum ResidualFrictionTruth {
    Items(Vec<String>),
    ExplicitNone(String),
}

#[derive(Debug, Clone, Copy)]
pub struct ProvingRunCloseoutExpected<'a> {
    pub approval_path: Option<&'a Path>,
    pub onboarding_pack_prefix: &'a str,
}

#[derive(Debug)]
pub enum ProvingRunCloseoutError {
    Validation(String),
    Internal(String),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProvingRunCloseout {
    state: String,
    approval_ref: Option<String>,
    approval_sha256: Option<String>,
    approval_source: Option<String>,
    manual_control_plane_edits: u64,
    partial_write_incidents: u64,
    ambiguous_ownership_incidents: u64,
    duration_seconds: Option<u64>,
    duration_missing_reason: Option<String>,
    residual_friction: Option<Vec<String>>,
    explicit_none_reason: Option<String>,
    preflight_passed: bool,
    recorded_at: String,
    commit: String,
}

impl fmt::Display for ProvingRunCloseoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(message) | Self::Internal(message) => f.write_str(message),
        }
    }
}

pub fn load_validated_closeout(
    workspace_root: &Path,
    closeout_path: &Path,
    resolved_closeout_path: &Path,
    expected: ProvingRunCloseoutExpected<'_>,
) -> Result<ProvingRunCloseout, ProvingRunCloseoutError> {
    let closeout_text =
        fs::read_to_string(resolved_closeout_path).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => ProvingRunCloseoutError::Validation(format!(
                "read {}: {err}",
                closeout_path.display()
            )),
            _ => ProvingRunCloseoutError::Validation(format!(
                "read {}: {err}",
                closeout_path.display()
            )),
        })?;
    let raw = serde_json::from_str::<RawProvingRunCloseout>(&closeout_text).map_err(|err| {
        ProvingRunCloseoutError::Validation(format!("parse {}: {err}", closeout_path.display()))
    })?;
    validate_closeout(workspace_root, closeout_path, raw, expected)
}

pub fn load_validated_closeout_if_present(
    workspace_root: &Path,
    closeout_path: &Path,
    resolved_closeout_path: &Path,
    expected: ProvingRunCloseoutExpected<'_>,
) -> Result<Option<ProvingRunCloseout>, ProvingRunCloseoutError> {
    match fs::read_to_string(resolved_closeout_path) {
        Ok(closeout_text) => {
            let raw =
                serde_json::from_str::<RawProvingRunCloseout>(&closeout_text).map_err(|err| {
                    ProvingRunCloseoutError::Validation(format!(
                        "parse {}: {err}",
                        closeout_path.display()
                    ))
                })?;
            validate_closeout(workspace_root, closeout_path, raw, expected).map(Some)
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(ProvingRunCloseoutError::Validation(format!(
            "read {}: {err}",
            closeout_path.display()
        ))),
    }
}

fn validate_closeout(
    workspace_root: &Path,
    closeout_path: &Path,
    raw: RawProvingRunCloseout,
    expected: ProvingRunCloseoutExpected<'_>,
) -> Result<ProvingRunCloseout, ProvingRunCloseoutError> {
    if raw.state != "closed" {
        return Err(ProvingRunCloseoutError::Validation(format!(
            "{}: state must equal `closed`",
            closeout_path.display()
        )));
    }

    let approval_ref = required_string(closeout_path, "approval_ref", raw.approval_ref.as_deref())?;
    let approval_sha256 = required_string(
        closeout_path,
        "approval_sha256",
        raw.approval_sha256.as_deref(),
    )?;
    let approval_source = required_string(
        closeout_path,
        "approval_source",
        raw.approval_source.as_deref(),
    )?;
    validate_lower_hex_sha256(closeout_path, &approval_sha256)?;
    validate_recorded_at(closeout_path, &raw.recorded_at)?;
    validate_commit(closeout_path, &raw.commit)?;

    let duration = match (raw.duration_seconds, raw.duration_missing_reason) {
        (Some(seconds), None) => DurationTruth::Seconds(seconds),
        (None, Some(reason)) => DurationTruth::MissingReason(non_empty_field(
            closeout_path,
            "duration_missing_reason",
            &reason,
        )?),
        _ => {
            return Err(ProvingRunCloseoutError::Validation(format!(
                "{}: exactly one of `duration_seconds` or `duration_missing_reason` is required",
                closeout_path.display()
            )));
        }
    };

    let residual_friction = match (raw.residual_friction, raw.explicit_none_reason) {
        (Some(items), None) => {
            let items = items
                .into_iter()
                .map(|item| non_empty_field(closeout_path, "residual_friction[]", &item))
                .collect::<Result<Vec<_>, _>>()?;
            if items.is_empty() {
                return Err(ProvingRunCloseoutError::Validation(format!(
                    "{}: `residual_friction` must not be empty when present",
                    closeout_path.display()
                )));
            }
            ResidualFrictionTruth::Items(items)
        }
        (None, Some(reason)) => ResidualFrictionTruth::ExplicitNone(non_empty_field(
            closeout_path,
            "explicit_none_reason",
            &reason,
        )?),
        _ => {
            return Err(ProvingRunCloseoutError::Validation(format!(
                "{}: exactly one of `residual_friction` or `explicit_none_reason` is required",
                closeout_path.display()
            )));
        }
    };

    let linked_approval =
        load_approval_artifact(workspace_root, Path::new(&approval_ref), closeout_path)?;
    if let Some(approval_path) = expected.approval_path {
        let provided_approval =
            load_approval_artifact(workspace_root, approval_path, closeout_path)?;
        validate_same_approval_artifact(closeout_path, &provided_approval, &linked_approval)?;
    }
    validate_approval_hash(closeout_path, &linked_approval, &approval_sha256)?;
    validate_approval_pack_prefix(
        closeout_path,
        &linked_approval,
        expected.onboarding_pack_prefix,
    )?;

    Ok(ProvingRunCloseout {
        approval_ref,
        approval_sha256,
        approval_source,
        manual_control_plane_edits: raw.manual_control_plane_edits,
        partial_write_incidents: raw.partial_write_incidents,
        ambiguous_ownership_incidents: raw.ambiguous_ownership_incidents,
        duration,
        residual_friction,
        preflight_passed: raw.preflight_passed,
        recorded_at: raw.recorded_at,
        commit: raw.commit,
    })
}

fn required_string(
    path: &Path,
    field_name: &str,
    value: Option<&str>,
) -> Result<String, ProvingRunCloseoutError> {
    let Some(value) = value else {
        return Err(ProvingRunCloseoutError::Validation(format!(
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
) -> Result<String, ProvingRunCloseoutError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ProvingRunCloseoutError::Validation(format!(
            "{}: `{field_name}` must not be empty",
            path.display()
        )));
    }
    Ok(trimmed.to_string())
}

fn validate_lower_hex_sha256(path: &Path, value: &str) -> Result<(), ProvingRunCloseoutError> {
    if value.len() != 64 || !value.chars().all(|ch| matches!(ch, '0'..='9' | 'a'..='f')) {
        return Err(ProvingRunCloseoutError::Validation(format!(
            "{}: `approval_sha256` must be 64 lowercase hex characters",
            path.display()
        )));
    }
    Ok(())
}

fn validate_recorded_at(path: &Path, value: &str) -> Result<(), ProvingRunCloseoutError> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| {
        ProvingRunCloseoutError::Validation(format!(
            "{}: `recorded_at` must be RFC3339",
            path.display()
        ))
    })?;
    Ok(())
}

fn validate_commit(path: &Path, value: &str) -> Result<(), ProvingRunCloseoutError> {
    let valid = (7..=40).contains(&value.len())
        && value.chars().all(|ch| matches!(ch, '0'..='9' | 'a'..='f'));
    if valid {
        Ok(())
    } else {
        Err(ProvingRunCloseoutError::Validation(format!(
            "{}: `commit` must be 7-40 lowercase hex characters",
            path.display()
        )))
    }
}

fn load_approval_artifact(
    workspace_root: &Path,
    approval_path: &Path,
    closeout_path: &Path,
) -> Result<ApprovalArtifact, ProvingRunCloseoutError> {
    let approval_path = approval_path.to_string_lossy();
    approval_artifact::load_approval_artifact(workspace_root, &approval_path)
        .map_err(|err| map_approval_artifact_error(closeout_path, err))
}

fn validate_same_approval_artifact(
    closeout_path: &Path,
    provided_approval: &ApprovalArtifact,
    linked_approval: &ApprovalArtifact,
) -> Result<(), ProvingRunCloseoutError> {
    if provided_approval.canonical_path != linked_approval.canonical_path {
        return Err(ProvingRunCloseoutError::Validation(format!(
            "{}: approval_ref `{}` does not match --approval `{}`",
            closeout_path.display(),
            linked_approval.relative_path,
            provided_approval.relative_path
        )));
    }
    Ok(())
}

fn validate_approval_hash(
    closeout_path: &Path,
    approval: &ApprovalArtifact,
    expected_sha256: &str,
) -> Result<(), ProvingRunCloseoutError> {
    if approval.sha256 != expected_sha256 {
        return Err(ProvingRunCloseoutError::Validation(format!(
            "{}: approval_sha256 does not match {}",
            closeout_path.display(),
            approval.relative_path
        )));
    }
    Ok(())
}

fn validate_approval_pack_prefix(
    closeout_path: &Path,
    approval: &ApprovalArtifact,
    onboarding_pack_prefix: &str,
) -> Result<(), ProvingRunCloseoutError> {
    if approval.descriptor.onboarding_pack_prefix != onboarding_pack_prefix {
        return Err(ProvingRunCloseoutError::Validation(format!(
            "{}: approval artifact `{}` belongs to onboarding_pack_prefix `{}` instead of `{}`",
            closeout_path.display(),
            approval.relative_path,
            approval.descriptor.onboarding_pack_prefix,
            onboarding_pack_prefix
        )));
    }
    Ok(())
}

fn map_approval_artifact_error(
    closeout_path: &Path,
    err: ApprovalArtifactError,
) -> ProvingRunCloseoutError {
    match err {
        ApprovalArtifactError::Validation(message) => {
            ProvingRunCloseoutError::Validation(format!("{}: {message}", closeout_path.display()))
        }
        ApprovalArtifactError::Internal(message) => ProvingRunCloseoutError::Internal(message),
    }
}
