use std::path::Path;

use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::approval_artifact::{self, ApprovalArtifact, ApprovalArtifactError};

use super::{
    DurationTruth, MaintenanceSettlement, MaintenanceSettlementMode, ProvingRunCloseout,
    ProvingRunCloseoutError, ProvingRunCloseoutState, RawMaintenanceSettlement,
    ResidualFrictionTruth,
};

pub(super) fn parse_closeout_state(
    path: &Path,
    value: &str,
) -> Result<ProvingRunCloseoutState, ProvingRunCloseoutError> {
    match value {
        "prepared" => Ok(ProvingRunCloseoutState::Prepared),
        "closed" => Ok(ProvingRunCloseoutState::Closed),
        _ => Err(ProvingRunCloseoutError::Validation(format!(
            "{}: state must equal `prepared` or `closed`",
            path.display()
        ))),
    }
}

pub(super) fn validate_allowed_state(
    path: &Path,
    state: ProvingRunCloseoutState,
    allowed_states: &[ProvingRunCloseoutState],
) -> Result<(), ProvingRunCloseoutError> {
    if allowed_states.contains(&state) {
        return Ok(());
    }

    if allowed_states == [ProvingRunCloseoutState::Closed] {
        return Err(ProvingRunCloseoutError::Validation(format!(
            "{}: state must equal `closed`",
            path.display()
        )));
    }

    let allowed = allowed_states
        .iter()
        .map(|state| format!("`{}`", state.as_str()))
        .collect::<Vec<_>>()
        .join(" or ");
    Err(ProvingRunCloseoutError::Validation(format!(
        "{}: state must equal {allowed}",
        path.display()
    )))
}

pub(super) fn parse_maintenance_settlement(
    closeout_path: &Path,
    raw: RawMaintenanceSettlement,
) -> Result<MaintenanceSettlement, ProvingRunCloseoutError> {
    let mode = match raw.mode.as_str() {
        "release_watch_enrolled" => MaintenanceSettlementMode::ReleaseWatchEnrolled,
        "explicitly_deferred" => MaintenanceSettlementMode::ExplicitlyDeferred,
        _ => {
            return Err(ProvingRunCloseoutError::Validation(format!(
                "{}: `maintenance_settlement.mode` must equal `release_watch_enrolled` or `explicitly_deferred`",
                closeout_path.display()
            )));
        }
    };
    let approval_section_sha256 = required_string(
        closeout_path,
        "maintenance_settlement.approval_section_sha256",
        raw.approval_section_sha256.as_deref(),
    )?;
    validate_sha256_field(
        closeout_path,
        "maintenance_settlement.approval_section_sha256",
        &approval_section_sha256,
    )?;
    let release_watch_sha256 = optional_sha256_field(
        closeout_path,
        "maintenance_settlement.release_watch_sha256",
        raw.release_watch_sha256.as_deref(),
    )?;
    let deferral_sha256 = optional_sha256_field(
        closeout_path,
        "maintenance_settlement.deferral_sha256",
        raw.deferral_sha256.as_deref(),
    )?;

    match mode {
        MaintenanceSettlementMode::ReleaseWatchEnrolled => {
            if release_watch_sha256.is_none() || deferral_sha256.is_some() {
                return Err(ProvingRunCloseoutError::Validation(format!(
                    "{}: enrolled `maintenance_settlement` requires `release_watch_sha256` and forbids `deferral_sha256`",
                    closeout_path.display()
                )));
            }
        }
        MaintenanceSettlementMode::ExplicitlyDeferred => {
            if deferral_sha256.is_none() || release_watch_sha256.is_some() {
                return Err(ProvingRunCloseoutError::Validation(format!(
                    "{}: deferred `maintenance_settlement` requires `deferral_sha256` and forbids `release_watch_sha256`",
                    closeout_path.display()
                )));
            }
        }
    }

    Ok(MaintenanceSettlement {
        mode,
        approval_section_sha256,
        release_watch_sha256,
        deferral_sha256,
    })
}

pub(super) fn validate_loaded_closeout_fields(
    path: &Path,
    closeout: &ProvingRunCloseout,
) -> Result<(), ProvingRunCloseoutError> {
    validate_lower_hex_sha256(path, &closeout.approval_sha256)?;
    if let Some(settlement) = &closeout.maintenance_settlement {
        validate_sha256_field(
            path,
            "maintenance_settlement.approval_section_sha256",
            &settlement.approval_section_sha256,
        )?;
        match settlement.mode {
            MaintenanceSettlementMode::ReleaseWatchEnrolled => {
                let release_watch_sha256 =
                    settlement.release_watch_sha256.as_deref().ok_or_else(|| {
                        ProvingRunCloseoutError::Validation(format!(
                            "{}: `maintenance_settlement.release_watch_sha256` is required when mode = `release_watch_enrolled`",
                            path.display()
                        ))
                    })?;
                validate_sha256_field(
                    path,
                    "maintenance_settlement.release_watch_sha256",
                    release_watch_sha256,
                )?;
                if settlement.deferral_sha256.is_some() {
                    return Err(ProvingRunCloseoutError::Validation(format!(
                        "{}: `maintenance_settlement.deferral_sha256` must be absent when mode = `release_watch_enrolled`",
                        path.display()
                    )));
                }
            }
            MaintenanceSettlementMode::ExplicitlyDeferred => {
                let deferral_sha256 =
                    settlement.deferral_sha256.as_deref().ok_or_else(|| {
                        ProvingRunCloseoutError::Validation(format!(
                            "{}: `maintenance_settlement.deferral_sha256` is required when mode = `explicitly_deferred`",
                            path.display()
                        ))
                    })?;
                validate_sha256_field(
                    path,
                    "maintenance_settlement.deferral_sha256",
                    deferral_sha256,
                )?;
                if settlement.release_watch_sha256.is_some() {
                    return Err(ProvingRunCloseoutError::Validation(format!(
                        "{}: `maintenance_settlement.release_watch_sha256` must be absent when mode = `explicitly_deferred`",
                        path.display()
                    )));
                }
            }
        }
    }
    validate_recorded_at(path, &closeout.recorded_at)?;
    validate_commit(path, &closeout.commit)?;
    match &closeout.duration {
        DurationTruth::Seconds(_) => {}
        DurationTruth::MissingReason(reason) => {
            non_empty_field(path, "duration_missing_reason", reason)?;
        }
    }
    match &closeout.residual_friction {
        ResidualFrictionTruth::Items(items) => {
            if items.is_empty() {
                return Err(ProvingRunCloseoutError::Validation(format!(
                    "{}: `residual_friction` must not be empty when present",
                    path.display()
                )));
            }
            for item in items {
                non_empty_field(path, "residual_friction[]", item)?;
            }
        }
        ResidualFrictionTruth::ExplicitNone(reason) => {
            non_empty_field(path, "explicit_none_reason", reason)?;
        }
    }
    Ok(())
}

pub(super) fn required_string(
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

pub(super) fn non_empty_field(
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

pub(super) fn validate_lower_hex_sha256(
    path: &Path,
    value: &str,
) -> Result<(), ProvingRunCloseoutError> {
    validate_sha256_field(path, "approval_sha256", value)
}

pub(super) fn validate_sha256_field(
    path: &Path,
    field_name: &str,
    value: &str,
) -> Result<(), ProvingRunCloseoutError> {
    if value.len() != 64 || !value.chars().all(|ch| matches!(ch, '0'..='9' | 'a'..='f')) {
        return Err(ProvingRunCloseoutError::Validation(format!(
            "{}: `{field_name}` must be 64 lowercase hex characters",
            path.display()
        )));
    }
    Ok(())
}

pub(super) fn optional_sha256_field(
    path: &Path,
    field_name: &str,
    value: Option<&str>,
) -> Result<Option<String>, ProvingRunCloseoutError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = non_empty_field(path, field_name, value)?;
    validate_sha256_field(path, field_name, &value)?;
    Ok(Some(value))
}

pub(super) fn validate_recorded_at(
    path: &Path,
    value: &str,
) -> Result<(), ProvingRunCloseoutError> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| {
        ProvingRunCloseoutError::Validation(format!(
            "{}: `recorded_at` must be RFC3339",
            path.display()
        ))
    })?;
    Ok(())
}

pub(super) fn validate_commit(path: &Path, value: &str) -> Result<(), ProvingRunCloseoutError> {
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

pub(super) fn load_approval_artifact(
    workspace_root: &Path,
    approval_path: &Path,
    closeout_path: &Path,
) -> Result<ApprovalArtifact, ProvingRunCloseoutError> {
    let approval_path = approval_path.to_string_lossy();
    approval_artifact::load_approval_artifact(workspace_root, &approval_path)
        .map_err(|err| map_approval_artifact_error(closeout_path, err))
}

pub(super) fn validate_same_approval_artifact(
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

pub(super) fn validate_approval_hash(
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

pub(super) fn validate_approval_pack_prefix(
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
