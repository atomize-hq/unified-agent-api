#[path = "proving_run_closeout/loading.rs"]
mod loading;
#[path = "proving_run_closeout/validation.rs"]
mod validation;

use std::{fmt, path::Path};

use serde::Deserialize;

use self::validation::validate_loaded_closeout_fields;

pub const PREPARED_CLOSEOUT_DURATION_PLACEHOLDER: &str =
    "TODO(closeout): replace with duration_seconds or a truthful duration_missing_reason.";
pub const PREPARED_CLOSEOUT_RESIDUAL_FRICTION_PLACEHOLDER: &str =
    "TODO(closeout): replace with residual_friction items or a truthful explicit_none_reason.";
pub const MACHINE_OWNED_FIELDS: [&str; 8] = [
    "state",
    "approval_ref",
    "approval_sha256",
    "approval_source",
    "maintenance_settlement",
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
    pub maintenance_settlement: Option<MaintenanceSettlement>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintenanceSettlement {
    pub mode: MaintenanceSettlementMode,
    pub approval_section_sha256: String,
    pub release_watch_sha256: Option<String>,
    pub deferral_sha256: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaintenanceSettlementMode {
    ReleaseWatchEnrolled,
    ExplicitlyDeferred,
}

impl MaintenanceSettlementMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReleaseWatchEnrolled => "release_watch_enrolled",
            Self::ExplicitlyDeferred => "explicitly_deferred",
        }
    }
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
    pub maintenance_settlement: Option<MaintenanceSettlement>,
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
    maintenance_settlement: Option<RawMaintenanceSettlement>,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMaintenanceSettlement {
    mode: String,
    approval_section_sha256: Option<String>,
    release_watch_sha256: Option<String>,
    deferral_sha256: Option<String>,
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
        maintenance_settlement: machine.maintenance_settlement,
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
        "maintenance_settlement": closeout
            .maintenance_settlement
            .as_ref()
            .map(render_maintenance_settlement)
            .unwrap_or(serde_json::Value::Null),
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

pub fn load_validated_closeout_with_states(
    workspace_root: &Path,
    closeout_path: &Path,
    resolved_closeout_path: &Path,
    expected: ProvingRunCloseoutExpected<'_>,
    allowed_states: &[ProvingRunCloseoutState],
) -> Result<ProvingRunCloseout, ProvingRunCloseoutError> {
    loading::load_validated_closeout_with_states(
        workspace_root,
        closeout_path,
        resolved_closeout_path,
        expected,
        allowed_states,
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

pub fn load_validated_closeout_if_present_with_states(
    workspace_root: &Path,
    closeout_path: &Path,
    resolved_closeout_path: &Path,
    expected: ProvingRunCloseoutExpected<'_>,
    allowed_states: &[ProvingRunCloseoutState],
) -> Result<Option<ProvingRunCloseout>, ProvingRunCloseoutError> {
    loading::load_validated_closeout_if_present_with_states(
        workspace_root,
        closeout_path,
        resolved_closeout_path,
        expected,
        allowed_states,
    )
}

fn render_maintenance_settlement(settlement: &MaintenanceSettlement) -> serde_json::Value {
    serde_json::json!({
        "mode": settlement.mode.as_str(),
        "approval_section_sha256": settlement.approval_section_sha256,
        "release_watch_sha256": settlement.release_watch_sha256,
        "deferral_sha256": settlement.deferral_sha256,
    })
}
