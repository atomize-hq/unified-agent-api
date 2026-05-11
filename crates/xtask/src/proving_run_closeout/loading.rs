use std::{fs, io, path::Path};

use super::{
    validation::{
        load_approval_artifact, non_empty_field, parse_closeout_state,
        parse_maintenance_settlement, required_string, validate_allowed_state,
        validate_approval_hash, validate_approval_pack_prefix, validate_commit,
        validate_loaded_closeout_fields, validate_lower_hex_sha256, validate_recorded_at,
        validate_same_approval_artifact,
    },
    DurationTruth, ProvingRunCloseout, ProvingRunCloseoutError, ProvingRunCloseoutExpected,
    ProvingRunCloseoutState, RawProvingRunCloseout, ResidualFrictionTruth,
};

pub(super) fn load_validated_closeout_with_states(
    workspace_root: &Path,
    closeout_path: &Path,
    resolved_closeout_path: &Path,
    expected: ProvingRunCloseoutExpected<'_>,
    allowed_states: &[ProvingRunCloseoutState],
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
    validate_closeout(workspace_root, closeout_path, raw, expected, allowed_states)
}

pub(super) fn load_validated_closeout_if_present_with_states(
    workspace_root: &Path,
    closeout_path: &Path,
    resolved_closeout_path: &Path,
    expected: ProvingRunCloseoutExpected<'_>,
    allowed_states: &[ProvingRunCloseoutState],
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
            validate_closeout(workspace_root, closeout_path, raw, expected, allowed_states)
                .map(Some)
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
    allowed_states: &[ProvingRunCloseoutState],
) -> Result<ProvingRunCloseout, ProvingRunCloseoutError> {
    let state = parse_closeout_state(closeout_path, &raw.state)?;
    validate_allowed_state(closeout_path, state, allowed_states)?;

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

    let closeout = ProvingRunCloseout {
        state,
        approval_ref,
        approval_sha256,
        approval_source,
        maintenance_settlement: raw
            .maintenance_settlement
            .map(|settlement| parse_maintenance_settlement(closeout_path, settlement))
            .transpose()?,
        manual_control_plane_edits: raw.manual_control_plane_edits,
        partial_write_incidents: raw.partial_write_incidents,
        ambiguous_ownership_incidents: raw.ambiguous_ownership_incidents,
        duration,
        residual_friction,
        preflight_passed: raw.preflight_passed,
        recorded_at: raw.recorded_at,
        commit: raw.commit,
    };
    validate_loaded_closeout_fields(closeout_path, &closeout)?;
    Ok(closeout)
}
