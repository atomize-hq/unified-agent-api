use std::{fmt, fs, io, path::Path};

use crate::approval_artifact::{self, ApprovalArtifact, ApprovalArtifactError};
use serde::Deserialize;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub const PREPARED_CLOSEOUT_DURATION_PLACEHOLDER: &str =
    "TODO(closeout): replace with duration_seconds or a truthful duration_missing_reason.";
pub const PREPARED_CLOSEOUT_RESIDUAL_FRICTION_PLACEHOLDER: &str =
    "TODO(closeout): replace with residual_friction items or a truthful explicit_none_reason.";
pub const MACHINE_OWNED_FIELDS: [&str; 7] = [
    "state",
    "approval_ref",
    "approval_sha256",
    "approval_source",
    "preflight_passed",
    "recorded_at",
    "commit",
];
pub const HUMAN_OWNED_FIELDS: [&str; 5] = [
    "manual_control_plane_edits",
    "partial_write_incidents",
    "ambiguous_ownership_incidents",
    "duration_*",
    "residual_friction|explicit_none_reason",
];
pub const PLACEHOLDER_VOCABULARY: [&str; 2] = [
    PREPARED_CLOSEOUT_DURATION_PLACEHOLDER,
    PREPARED_CLOSEOUT_RESIDUAL_FRICTION_PLACEHOLDER,
];

#[derive(Debug, Clone)]
pub struct ProvingRunCloseout {
    pub state: ProvingRunCloseoutState,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProvingRunCloseoutState {
    Prepared,
    Closed,
}

impl ProvingRunCloseoutState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::Closed => "closed",
        }
    }

    pub const fn all() -> &'static [Self] {
        &[Self::Prepared, Self::Closed]
    }
}

#[derive(Debug, Clone)]
pub struct ProvingRunCloseoutMachineFields {
    pub approval_ref: String,
    pub approval_sha256: String,
    pub approval_source: String,
    pub preflight_passed: bool,
    pub recorded_at: String,
    pub commit: String,
}

#[derive(Debug, Clone)]
pub struct ProvingRunCloseoutHumanFields {
    pub manual_control_plane_edits: u64,
    pub partial_write_incidents: u64,
    pub ambiguous_ownership_incidents: u64,
    pub duration: DurationTruth,
    pub residual_friction: ResidualFrictionTruth,
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

pub fn build_closeout(
    state: ProvingRunCloseoutState,
    machine: ProvingRunCloseoutMachineFields,
    human: ProvingRunCloseoutHumanFields,
) -> Result<ProvingRunCloseout, ProvingRunCloseoutError> {
    let closeout = ProvingRunCloseout {
        state,
        approval_ref: machine.approval_ref,
        approval_sha256: machine.approval_sha256,
        approval_source: machine.approval_source,
        manual_control_plane_edits: human.manual_control_plane_edits,
        partial_write_incidents: human.partial_write_incidents,
        ambiguous_ownership_incidents: human.ambiguous_ownership_incidents,
        duration: human.duration,
        residual_friction: human.residual_friction,
        preflight_passed: machine.preflight_passed,
        recorded_at: machine.recorded_at,
        commit: machine.commit,
    };
    validate_loaded_closeout_fields(Path::new("<generated-closeout>"), &closeout)?;
    Ok(closeout)
}

pub fn build_prepared_closeout(
    machine: ProvingRunCloseoutMachineFields,
) -> Result<ProvingRunCloseout, ProvingRunCloseoutError> {
    build_closeout(
        ProvingRunCloseoutState::Prepared,
        machine,
        ProvingRunCloseoutHumanFields {
            manual_control_plane_edits: 0,
            partial_write_incidents: 0,
            ambiguous_ownership_incidents: 0,
            duration: DurationTruth::MissingReason(
                PREPARED_CLOSEOUT_DURATION_PLACEHOLDER.to_string(),
            ),
            residual_friction: ResidualFrictionTruth::ExplicitNone(
                PREPARED_CLOSEOUT_RESIDUAL_FRICTION_PLACEHOLDER.to_string(),
            ),
        },
    )
}

pub fn render_closeout_json(
    closeout: &ProvingRunCloseout,
) -> Result<String, ProvingRunCloseoutError> {
    validate_loaded_closeout_fields(Path::new("<rendered-closeout>"), closeout)?;
    let raw = serde_json::json!({
        "state": closeout.state.as_str(),
        "approval_ref": closeout.approval_ref,
        "approval_sha256": closeout.approval_sha256,
        "approval_source": closeout.approval_source,
        "manual_control_plane_edits": closeout.manual_control_plane_edits,
        "partial_write_incidents": closeout.partial_write_incidents,
        "ambiguous_ownership_incidents": closeout.ambiguous_ownership_incidents,
        "duration_seconds": match &closeout.duration {
            DurationTruth::Seconds(seconds) => serde_json::Value::from(*seconds),
            DurationTruth::MissingReason(_) => serde_json::Value::Null,
        },
        "duration_missing_reason": match &closeout.duration {
            DurationTruth::Seconds(_) => serde_json::Value::Null,
            DurationTruth::MissingReason(reason) => serde_json::Value::from(reason.clone()),
        },
        "residual_friction": match &closeout.residual_friction {
            ResidualFrictionTruth::Items(items) => serde_json::Value::from(items.clone()),
            ResidualFrictionTruth::ExplicitNone(_) => serde_json::Value::Null,
        },
        "explicit_none_reason": match &closeout.residual_friction {
            ResidualFrictionTruth::Items(_) => serde_json::Value::Null,
            ResidualFrictionTruth::ExplicitNone(reason) => serde_json::Value::from(reason.clone()),
        },
        "preflight_passed": closeout.preflight_passed,
        "recorded_at": closeout.recorded_at,
        "commit": closeout.commit,
    });
    let mut rendered = serde_json::to_string_pretty(&raw)
        .map_err(|err| ProvingRunCloseoutError::Internal(format!("serialize closeout: {err}")))?;
    rendered.push('\n');
    Ok(rendered)
}

pub fn unresolved_placeholder_fields(closeout: &ProvingRunCloseout) -> Vec<&'static str> {
    let mut unresolved = Vec::new();
    if matches!(
        &closeout.duration,
        DurationTruth::MissingReason(reason) if reason.trim() == PREPARED_CLOSEOUT_DURATION_PLACEHOLDER
    ) {
        unresolved.push("duration_missing_reason");
    }
    if matches!(
        &closeout.residual_friction,
        ResidualFrictionTruth::ExplicitNone(reason)
            if reason.trim() == PREPARED_CLOSEOUT_RESIDUAL_FRICTION_PLACEHOLDER
    ) {
        unresolved.push("explicit_none_reason");
    }
    if matches!(
        &closeout.residual_friction,
        ResidualFrictionTruth::Items(items)
            if items.iter().any(|item| item.trim() == PREPARED_CLOSEOUT_RESIDUAL_FRICTION_PLACEHOLDER)
    ) {
        unresolved.push("residual_friction");
    }
    unresolved
}

pub fn has_unresolved_placeholders(closeout: &ProvingRunCloseout) -> bool {
    !unresolved_placeholder_fields(closeout).is_empty()
}

pub fn load_validated_closeout(
    workspace_root: &Path,
    closeout_path: &Path,
    resolved_closeout_path: &Path,
    expected: ProvingRunCloseoutExpected<'_>,
) -> Result<ProvingRunCloseout, ProvingRunCloseoutError> {
    load_validated_closeout_with_states(
        workspace_root,
        closeout_path,
        resolved_closeout_path,
        expected,
        &[ProvingRunCloseoutState::Closed],
    )
}

pub fn load_validated_closeout_if_present(
    workspace_root: &Path,
    closeout_path: &Path,
    resolved_closeout_path: &Path,
    expected: ProvingRunCloseoutExpected<'_>,
) -> Result<Option<ProvingRunCloseout>, ProvingRunCloseoutError> {
    load_validated_closeout_if_present_with_states(
        workspace_root,
        closeout_path,
        resolved_closeout_path,
        expected,
        &[ProvingRunCloseoutState::Closed],
    )
}

pub fn load_validated_closeout_with_states(
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

pub fn load_validated_closeout_if_present_with_states(
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

fn parse_closeout_state(
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

fn validate_allowed_state(
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

fn validate_loaded_closeout_fields(
    path: &Path,
    closeout: &ProvingRunCloseout,
) -> Result<(), ProvingRunCloseoutError> {
    validate_lower_hex_sha256(path, &closeout.approval_sha256)?;
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
