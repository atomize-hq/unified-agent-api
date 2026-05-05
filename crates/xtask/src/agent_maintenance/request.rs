use std::{
    collections::BTreeSet,
    fmt, fs,
    path::{Component, Path, PathBuf},
};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use time::{format_description::well_known::Rfc3339, OffsetDateTime, UtcOffset};
use toml_edit::de::from_str;

use crate::{
    agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH},
    workspace_mutation::WorkspacePathJail,
};

const DOCS_NEXT_ROOT: &str = "docs/agents/lifecycle";
const GOVERNANCE_DIR_NAME: &str = "governance";
const REQUEST_FILE_NAME: &str = "maintenance-request.toml";
const LEGACY_ARTIFACT_VERSION: &str = "1";
pub(crate) const AUTOMATED_ARTIFACT_VERSION: &str = "2";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintenanceRequest {
    pub relative_path: String,
    pub canonical_path: PathBuf,
    pub sha256: String,
    pub maintenance_pack_prefix: String,
    pub maintenance_root: String,
    pub agent_id: String,
    pub trigger_kind: TriggerKind,
    pub basis_ref: String,
    pub opened_from: String,
    pub requested_control_plane_actions: Vec<MaintenanceAction>,
    pub runtime_followup_required: RuntimeFollowupRequired,
    pub detected_release: Option<DetectedRelease>,
    pub request_recorded_at: String,
    pub request_commit: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFollowupRequired {
    pub required: bool,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedRelease {
    pub detected_by: String,
    pub current_validated: String,
    pub target_version: String,
    pub latest_stable: String,
    pub version_policy: String,
    pub source_kind: String,
    pub source_ref: String,
    pub dispatch_kind: String,
    pub dispatch_workflow: String,
    pub branch_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerKind {
    DriftDetected,
    ManualReopen,
    PostReleaseAudit,
    UpstreamReleaseDetected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(clippy::enum_variant_names)]
pub enum MaintenanceAction {
    PacketDocRefresh,
    SupportMatrixRefresh,
    CapabilityMatrixRefresh,
    ReleaseDocRefresh,
}

#[derive(Debug)]
pub enum MaintenanceRequestError {
    Validation(String),
    Internal(String),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMaintenanceRequest {
    artifact_version: String,
    agent_id: String,
    trigger_kind: String,
    basis_ref: String,
    opened_from: String,
    requested_control_plane_actions: Vec<String>,
    runtime_followup_required: RawRuntimeFollowupRequired,
    #[serde(default)]
    detected_release: Option<RawDetectedRelease>,
    request_recorded_at: String,
    request_commit: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRuntimeFollowupRequired {
    required: bool,
    items: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDetectedRelease {
    detected_by: String,
    current_validated: String,
    target_version: String,
    latest_stable: String,
    version_policy: String,
    source_kind: String,
    source_ref: String,
    dispatch_kind: String,
    dispatch_workflow: String,
    branch_name: String,
}

impl fmt::Display for MaintenanceRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(message) | Self::Internal(message) => f.write_str(message),
        }
    }
}

impl TriggerKind {
    fn parse(value: &str, request_path: &Path) -> Result<Self, MaintenanceRequestError> {
        match value {
            "drift_detected" => Ok(Self::DriftDetected),
            "manual_reopen" => Ok(Self::ManualReopen),
            "post_release_audit" => Ok(Self::PostReleaseAudit),
            "upstream_release_detected" => Ok(Self::UpstreamReleaseDetected),
            other => Err(MaintenanceRequestError::Validation(format!(
                "maintenance request `{}` has invalid `trigger_kind` `{other}`; expected `drift_detected`, `manual_reopen`, `post_release_audit`, or `upstream_release_detected`",
                request_path.display()
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::DriftDetected => "drift_detected",
            Self::ManualReopen => "manual_reopen",
            Self::PostReleaseAudit => "post_release_audit",
            Self::UpstreamReleaseDetected => "upstream_release_detected",
        }
    }
}

impl MaintenanceRequest {
    pub fn is_automated_watch_request(&self) -> bool {
        matches!(self.trigger_kind, TriggerKind::UpstreamReleaseDetected)
    }
}

impl MaintenanceAction {
    fn parse(value: &str, request_path: &Path) -> Result<Self, MaintenanceRequestError> {
        match value {
            "packet_doc_refresh" => Ok(Self::PacketDocRefresh),
            "support_matrix_refresh" => Ok(Self::SupportMatrixRefresh),
            "capability_matrix_refresh" => Ok(Self::CapabilityMatrixRefresh),
            "release_doc_refresh" => Ok(Self::ReleaseDocRefresh),
            other => Err(MaintenanceRequestError::Validation(format!(
                "maintenance request `{}` requested runtime-owned or unsupported action `{other}`; allowed actions: `packet_doc_refresh`, `support_matrix_refresh`, `capability_matrix_refresh`, `release_doc_refresh`",
                request_path.display()
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::PacketDocRefresh => "packet_doc_refresh",
            Self::SupportMatrixRefresh => "support_matrix_refresh",
            Self::CapabilityMatrixRefresh => "capability_matrix_refresh",
            Self::ReleaseDocRefresh => "release_doc_refresh",
        }
    }
}

pub fn load_request(
    workspace_root: &Path,
    request_path: &Path,
) -> Result<MaintenanceRequest, MaintenanceRequestError> {
    let workspace_root = fs::canonicalize(workspace_root).map_err(|err| {
        MaintenanceRequestError::Internal(format!(
            "canonicalize {}: {err}",
            workspace_root.display()
        ))
    })?;
    let relative_path = normalize_repo_relative_path(&workspace_root, request_path)?;
    let maintenance_pack_prefix = validate_request_path(&relative_path)?;
    let maintenance_root = Path::new(DOCS_NEXT_ROOT).join(&maintenance_pack_prefix);

    let lexical_path = workspace_root.join(&relative_path);
    let canonical_path = fs::canonicalize(&lexical_path).map_err(|err| {
        MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` does not resolve: {err}",
            relative_path.display()
        ))
    })?;
    if !canonical_path.starts_with(&workspace_root) {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` resolves outside workspace root",
            relative_path.display()
        )));
    }

    let bytes = fs::read(&canonical_path).map_err(|err| {
        MaintenanceRequestError::Validation(format!(
            "read maintenance request `{}`: {err}",
            relative_path.display()
        ))
    })?;
    let text = std::str::from_utf8(&bytes).map_err(|err| {
        MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` must be valid utf-8: {err}",
            relative_path.display()
        ))
    })?;
    let raw: RawMaintenanceRequest = from_str(text).map_err(|err| {
        MaintenanceRequestError::Validation(format!(
            "parse maintenance request `{}`: {err}",
            relative_path.display()
        ))
    })?;

    validate_non_empty_scalar(&relative_path, "agent_id", &raw.agent_id)?;
    let expected_pack_prefix = format!("{}-maintenance", raw.agent_id);
    if maintenance_pack_prefix != expected_pack_prefix {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` belongs to maintenance pack `{maintenance_pack_prefix}` instead of `{expected_pack_prefix}`",
            relative_path.display()
        )));
    }

    if raw.artifact_version != LEGACY_ARTIFACT_VERSION
        && raw.artifact_version != AUTOMATED_ARTIFACT_VERSION
    {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` has unsupported `artifact_version` `{}`; expected `{LEGACY_ARTIFACT_VERSION}` or `{AUTOMATED_ARTIFACT_VERSION}`",
            relative_path.display(),
            raw.artifact_version
        )));
    }

    let registry = AgentRegistry::load(&workspace_root).map_err(|err| {
        MaintenanceRequestError::Internal(format!("load {REGISTRY_RELATIVE_PATH}: {err}"))
    })?;
    if registry.find(&raw.agent_id).is_none() {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` references unknown agent_id `{}`; onboarded agents must already exist in {REGISTRY_RELATIVE_PATH}",
            relative_path.display(),
            raw.agent_id
        )));
    }

    let trigger_kind = TriggerKind::parse(&raw.trigger_kind, &relative_path)?;
    let jail = WorkspacePathJail::new(&workspace_root).map_err(map_jail_error)?;
    validate_repo_relative_reference(&jail, &relative_path, "basis_ref", &raw.basis_ref)?;
    validate_repo_relative_reference(&jail, &relative_path, "opened_from", &raw.opened_from)?;
    let requested_control_plane_actions =
        validate_actions(&relative_path, &raw.requested_control_plane_actions)?;
    let runtime_followup_required =
        validate_runtime_followup_required(&relative_path, raw.runtime_followup_required)?;
    let detected_release =
        validate_detected_release(&relative_path, trigger_kind, raw.detected_release)?;
    validate_automated_watch_request(
        &relative_path,
        &raw.artifact_version,
        trigger_kind,
        &requested_control_plane_actions,
    )?;
    validate_rfc3339_utc(
        &relative_path,
        "request_recorded_at",
        &raw.request_recorded_at,
    )?;
    validate_commit_value(&relative_path, "request_commit", &raw.request_commit)?;

    Ok(MaintenanceRequest {
        relative_path: relative_path.display().to_string(),
        canonical_path,
        sha256: hex::encode(Sha256::digest(&bytes)),
        maintenance_pack_prefix,
        maintenance_root: maintenance_root.display().to_string(),
        agent_id: raw.agent_id,
        trigger_kind,
        basis_ref: raw.basis_ref,
        opened_from: raw.opened_from,
        requested_control_plane_actions,
        runtime_followup_required,
        detected_release,
        request_recorded_at: raw.request_recorded_at,
        request_commit: raw.request_commit,
    })
}

fn normalize_repo_relative_path(
    workspace_root: &Path,
    path: &Path,
) -> Result<PathBuf, MaintenanceRequestError> {
    let relative = if path.is_absolute() {
        path.strip_prefix(workspace_root)
            .map(Path::to_path_buf)
            .map_err(|_| {
                MaintenanceRequestError::Validation(format!(
                    "maintenance request path `{}` must be inside workspace root {}",
                    path.display(),
                    workspace_root.display()
                ))
            })?
    } else {
        path.to_path_buf()
    };

    if relative.components().next().is_none()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request path `{}` must be a repo-relative path with only normal components",
            path.display()
        )));
    }

    Ok(relative)
}

fn validate_request_path(relative_path: &Path) -> Result<String, MaintenanceRequestError> {
    let components = relative_path.components().collect::<Vec<_>>();
    let expected_prefix = [
        Component::Normal("docs".as_ref()),
        Component::Normal("agents".as_ref()),
        Component::Normal("lifecycle".as_ref()),
    ];

    if components.len() != 6
        || components[0..3] != expected_prefix
        || components[4] != Component::Normal(GOVERNANCE_DIR_NAME.as_ref())
        || components[5] != Component::Normal(REQUEST_FILE_NAME.as_ref())
    {
        return Err(MaintenanceRequestError::Validation(format!(
            "{} must point to docs/agents/lifecycle/<agent>-maintenance/governance/maintenance-request.toml",
            relative_path.display()
        )));
    }

    let Component::Normal(pack_prefix) = components[3] else {
        return Err(MaintenanceRequestError::Validation(format!(
            "{} must point to docs/agents/lifecycle/<agent>-maintenance/governance/maintenance-request.toml",
            relative_path.display()
        )));
    };
    let pack_prefix = pack_prefix.to_string_lossy().into_owned();
    if !pack_prefix.ends_with("-maintenance") {
        return Err(MaintenanceRequestError::Validation(format!(
            "{} must live under a maintenance root named `<agent>-maintenance`, not `{pack_prefix}`",
            relative_path.display()
        )));
    }

    Ok(pack_prefix)
}

pub(crate) fn validate_non_empty_scalar(
    request_path: &Path,
    field_name: &str,
    value: &str,
) -> Result<(), MaintenanceRequestError> {
    if value.trim().is_empty() {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must be a non-empty string",
            request_path.display()
        )));
    }
    Ok(())
}

pub(crate) fn validate_repo_relative_reference(
    jail: &WorkspacePathJail,
    request_path: &Path,
    field_name: &str,
    value: &str,
) -> Result<(), MaintenanceRequestError> {
    validate_non_empty_scalar(request_path, field_name, value)?;
    let relative = PathBuf::from(value);
    if relative.components().next().is_none()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must be a repo-relative path with only normal components",
            request_path.display()
        )));
    }

    let resolved = jail.resolve(&relative).map_err(map_jail_error)?;
    let metadata = fs::symlink_metadata(&resolved).map_err(|err| {
        MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must point to an existing file: {err}",
            request_path.display()
        ))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must point to an existing file",
            request_path.display()
        )));
    }
    Ok(())
}

pub(crate) fn validate_actions(
    request_path: &Path,
    values: &[String],
) -> Result<Vec<MaintenanceAction>, MaintenanceRequestError> {
    if values.is_empty() {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `requested_control_plane_actions` must be a non-empty array",
            request_path.display()
        )));
    }

    let mut seen = BTreeSet::new();
    let mut parsed = Vec::with_capacity(values.len());
    for value in values {
        let action = MaintenanceAction::parse(value, request_path)?;
        if !seen.insert(action) {
            return Err(MaintenanceRequestError::Validation(format!(
                "maintenance request `{}` field `requested_control_plane_actions` contains duplicate action `{}`",
                request_path.display(),
                action.as_str()
            )));
        }
        parsed.push(action);
    }
    Ok(parsed)
}

fn validate_runtime_followup_required(
    request_path: &Path,
    raw: RawRuntimeFollowupRequired,
) -> Result<RuntimeFollowupRequired, MaintenanceRequestError> {
    let mut items = Vec::with_capacity(raw.items.len());
    for item in raw.items {
        if item.trim().is_empty() {
            return Err(MaintenanceRequestError::Validation(format!(
                "maintenance request `{}` field `runtime_followup_required.items` must not contain blank entries",
                request_path.display()
            )));
        }
        items.push(item);
    }

    if raw.required && items.is_empty() {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` has `runtime_followup_required.required = true` but no follow-up items were provided",
            request_path.display()
        )));
    }
    if !raw.required && !items.is_empty() {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` has `runtime_followup_required.required = false` and therefore requires `items = []`",
            request_path.display()
        )));
    }

    Ok(RuntimeFollowupRequired {
        required: raw.required,
        items,
    })
}

pub(crate) fn validate_rfc3339_utc(
    request_path: &Path,
    field_name: &str,
    value: &str,
) -> Result<(), MaintenanceRequestError> {
    let parsed = OffsetDateTime::parse(value, &Rfc3339).map_err(|err| {
        MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must be RFC3339 UTC: {err}",
            request_path.display()
        ))
    })?;
    if parsed.offset() != UtcOffset::UTC {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must use UTC (`Z`) offset",
            request_path.display()
        )));
    }
    Ok(())
}

pub(crate) fn validate_commit_value(
    request_path: &Path,
    field_name: &str,
    value: &str,
) -> Result<(), MaintenanceRequestError> {
    let is_valid = (7..=40).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'));
    if !is_valid {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` field `{field_name}` must be 7-40 lowercase hex characters",
            request_path.display()
        )));
    }
    Ok(())
}

fn validate_detected_release(
    request_path: &Path,
    trigger_kind: TriggerKind,
    raw: Option<RawDetectedRelease>,
) -> Result<Option<DetectedRelease>, MaintenanceRequestError> {
    match (trigger_kind, raw) {
        (TriggerKind::UpstreamReleaseDetected, Some(raw)) => {
            validate_non_empty_scalar(request_path, "detected_release.detected_by", &raw.detected_by)?;
            validate_non_empty_scalar(
                request_path,
                "detected_release.current_validated",
                &raw.current_validated,
            )?;
            validate_non_empty_scalar(
                request_path,
                "detected_release.target_version",
                &raw.target_version,
            )?;
            validate_non_empty_scalar(
                request_path,
                "detected_release.latest_stable",
                &raw.latest_stable,
            )?;
            validate_non_empty_scalar(
                request_path,
                "detected_release.version_policy",
                &raw.version_policy,
            )?;
            validate_non_empty_scalar(
                request_path,
                "detected_release.source_kind",
                &raw.source_kind,
            )?;
            validate_non_empty_scalar(request_path, "detected_release.source_ref", &raw.source_ref)?;
            validate_non_empty_scalar(
                request_path,
                "detected_release.dispatch_kind",
                &raw.dispatch_kind,
            )?;
            validate_non_empty_scalar(
                request_path,
                "detected_release.dispatch_workflow",
                &raw.dispatch_workflow,
            )?;
            validate_non_empty_scalar(
                request_path,
                "detected_release.branch_name",
                &raw.branch_name,
            )?;
            Ok(Some(DetectedRelease {
                detected_by: raw.detected_by,
                current_validated: raw.current_validated,
                target_version: raw.target_version,
                latest_stable: raw.latest_stable,
                version_policy: raw.version_policy,
                source_kind: raw.source_kind,
                source_ref: raw.source_ref,
                dispatch_kind: raw.dispatch_kind,
                dispatch_workflow: raw.dispatch_workflow,
                branch_name: raw.branch_name,
            }))
        }
        (TriggerKind::UpstreamReleaseDetected, None) => Err(MaintenanceRequestError::Validation(
            format!(
                "maintenance request `{}` trigger_kind `upstream_release_detected` requires a `[detected_release]` table",
                request_path.display()
            ),
        )),
        (_, Some(_)) => Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` may only include `[detected_release]` when `trigger_kind = \"upstream_release_detected\"`",
            request_path.display()
        ))),
        (_, None) => Ok(None),
    }
}

fn validate_automated_watch_request(
    request_path: &Path,
    artifact_version: &str,
    trigger_kind: TriggerKind,
    requested_control_plane_actions: &[MaintenanceAction],
) -> Result<(), MaintenanceRequestError> {
    if trigger_kind != TriggerKind::UpstreamReleaseDetected {
        return Ok(());
    }
    if artifact_version != AUTOMATED_ARTIFACT_VERSION {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` trigger_kind `upstream_release_detected` requires `artifact_version = \"{AUTOMATED_ARTIFACT_VERSION}\"`",
            request_path.display()
        )));
    }
    if requested_control_plane_actions != [MaintenanceAction::PacketDocRefresh] {
        return Err(MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` trigger_kind `upstream_release_detected` requires `requested_control_plane_actions = [\"packet_doc_refresh\"]`",
            request_path.display()
        )));
    }
    Ok(())
}

fn map_jail_error(
    err: crate::workspace_mutation::WorkspaceMutationError,
) -> MaintenanceRequestError {
    match err {
        crate::workspace_mutation::WorkspaceMutationError::Validation(message) => {
            MaintenanceRequestError::Validation(message)
        }
        crate::workspace_mutation::WorkspaceMutationError::Internal(message) => {
            MaintenanceRequestError::Internal(message)
        }
    }
}
