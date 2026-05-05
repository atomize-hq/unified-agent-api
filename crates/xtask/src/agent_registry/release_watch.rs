use serde::Deserialize;

use super::{validate_non_empty_scalar, AgentRegistryEntry, AgentRegistryError};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseWatchMetadata {
    pub enabled: bool,
    pub version_policy: ReleaseWatchVersionPolicy,
    pub dispatch_kind: ReleaseWatchDispatchKind,
    #[serde(default)]
    pub dispatch_workflow: Option<String>,
    pub upstream: ReleaseWatchUpstream,
}

impl ReleaseWatchMetadata {
    pub(super) fn validate(&self, _entry: &AgentRegistryEntry) -> Result<(), AgentRegistryError> {
        if !self.enabled {
            return Err(AgentRegistryError::Validation(
                "maintenance.release_watch.enabled=false is not allowed; omit maintenance.release_watch entirely when an agent is not enrolled".to_string(),
            ));
        }

        self.upstream.validate()?;
        self.validate_dispatch_contract()?;
        Ok(())
    }

    fn validate_dispatch_contract(&self) -> Result<(), AgentRegistryError> {
        match self.dispatch_kind {
            ReleaseWatchDispatchKind::WorkflowDispatch => {
                let workflow = self.dispatch_workflow.as_deref().ok_or_else(|| {
                    AgentRegistryError::Validation(
                        "maintenance.release_watch.dispatch_workflow is required when dispatch_kind = `workflow_dispatch`".to_string(),
                    )
                })?;
                validate_non_empty_scalar("maintenance.release_watch.dispatch_workflow", workflow)?;
                if !workflow.ends_with(".yml") {
                    return Err(AgentRegistryError::Validation(format!(
                        "maintenance.release_watch.dispatch_workflow must reference a workflow file ending in `.yml` (got `{workflow}`)"
                    )));
                }
            }
            ReleaseWatchDispatchKind::PacketPr => {
                if self.dispatch_workflow.is_some() {
                    return Err(AgentRegistryError::Validation(
                        "maintenance.release_watch.dispatch_workflow must be omitted when dispatch_kind = `packet_pr`".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseWatchVersionPolicy {
    LatestStableMinusOne,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseWatchDispatchKind {
    WorkflowDispatch,
    PacketPr,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseWatchUpstream {
    pub source_kind: ReleaseWatchSourceKind,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default)]
    pub tag_prefix: Option<String>,
    #[serde(default)]
    pub bucket: Option<String>,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub version_marker: Option<String>,
}

impl ReleaseWatchUpstream {
    fn validate(&self) -> Result<(), AgentRegistryError> {
        match self.source_kind {
            ReleaseWatchSourceKind::GithubReleases => {
                validate_required_optional_scalar(
                    "maintenance.release_watch.upstream.owner",
                    self.owner.as_deref(),
                )?;
                validate_required_optional_scalar(
                    "maintenance.release_watch.upstream.repo",
                    self.repo.as_deref(),
                )?;
                validate_required_optional_scalar(
                    "maintenance.release_watch.upstream.tag_prefix",
                    self.tag_prefix.as_deref(),
                )?;
                validate_absent_optional_scalar(
                    "maintenance.release_watch.upstream.bucket",
                    self.bucket.as_deref(),
                    self.source_kind,
                )?;
                validate_absent_optional_scalar(
                    "maintenance.release_watch.upstream.prefix",
                    self.prefix.as_deref(),
                    self.source_kind,
                )?;
                validate_absent_optional_scalar(
                    "maintenance.release_watch.upstream.version_marker",
                    self.version_marker.as_deref(),
                    self.source_kind,
                )?;
            }
            ReleaseWatchSourceKind::GcsObjectListing => {
                validate_required_optional_scalar(
                    "maintenance.release_watch.upstream.bucket",
                    self.bucket.as_deref(),
                )?;
                validate_required_optional_scalar(
                    "maintenance.release_watch.upstream.prefix",
                    self.prefix.as_deref(),
                )?;
                validate_required_optional_scalar(
                    "maintenance.release_watch.upstream.version_marker",
                    self.version_marker.as_deref(),
                )?;
                validate_absent_optional_scalar(
                    "maintenance.release_watch.upstream.owner",
                    self.owner.as_deref(),
                    self.source_kind,
                )?;
                validate_absent_optional_scalar(
                    "maintenance.release_watch.upstream.repo",
                    self.repo.as_deref(),
                    self.source_kind,
                )?;
                validate_absent_optional_scalar(
                    "maintenance.release_watch.upstream.tag_prefix",
                    self.tag_prefix.as_deref(),
                    self.source_kind,
                )?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseWatchSourceKind {
    GithubReleases,
    GcsObjectListing,
}

impl ReleaseWatchSourceKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::GithubReleases => "github_releases",
            Self::GcsObjectListing => "gcs_object_listing",
        }
    }
}

fn validate_required_optional_scalar(
    field_name: &str,
    value: Option<&str>,
) -> Result<(), AgentRegistryError> {
    let value = value.ok_or_else(|| {
        AgentRegistryError::Validation(format!("{field_name} is required for this upstream source"))
    })?;
    validate_non_empty_scalar(field_name, value)
}

fn validate_absent_optional_scalar(
    field_name: &str,
    value: Option<&str>,
    source_kind: ReleaseWatchSourceKind,
) -> Result<(), AgentRegistryError> {
    if value.is_some() {
        return Err(AgentRegistryError::Validation(format!(
            "{field_name} must not be set when maintenance.release_watch.upstream.source_kind = `{}`",
            source_kind.as_str()
        )));
    }
    Ok(())
}
