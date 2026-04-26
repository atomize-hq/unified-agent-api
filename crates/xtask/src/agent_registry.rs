use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path},
};

use crate::capability_projection::{
    resolve_capability_publication_target, validate_config_key_allowlist,
};
use serde::Deserialize;
use thiserror::Error;

pub const REGISTRY_RELATIVE_PATH: &str = "crates/xtask/data/agent_registry.toml";
const WRAPPER_COVERAGE_BINDING_KIND_GENERATED_FROM_WRAPPER_CRATE: &str =
    "generated_from_wrapper_crate";
const BACKEND_MODULE_PREFIX: &str = "crates/agent_api/src/backends/";
const CRATE_PATH_PREFIX: &str = "crates/";
const MANIFEST_ROOT_PREFIX: &str = "cli_manifests/";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentRegistry {
    pub agents: Vec<AgentRegistryEntry>,
}

impl AgentRegistry {
    pub fn load(workspace_root: &Path) -> Result<Self, AgentRegistryError> {
        let path = workspace_root.join(REGISTRY_RELATIVE_PATH);
        Self::load_from_path(&path)
    }

    pub fn load_from_path(path: &Path) -> Result<Self, AgentRegistryError> {
        let raw = fs::read_to_string(path).map_err(|source| AgentRegistryError::Read {
            path: path.display().to_string(),
            source,
        })?;
        Self::parse(&raw)
    }

    pub fn parse(raw: &str) -> Result<Self, AgentRegistryError> {
        let registry = toml_edit::de::from_str::<Self>(raw)?;
        registry.validate()?;
        Ok(registry)
    }

    pub fn find(&self, agent_id: &str) -> Option<&AgentRegistryEntry> {
        self.agents.iter().find(|agent| agent.agent_id == agent_id)
    }

    pub fn support_matrix_entries(&self) -> impl Iterator<Item = &AgentRegistryEntry> + '_ {
        self.agents
            .iter()
            .filter(|agent| agent.publication.support_matrix_enabled)
    }

    pub fn capability_matrix_entries(&self) -> impl Iterator<Item = &AgentRegistryEntry> + '_ {
        self.agents
            .iter()
            .filter(|agent| agent.publication.capability_matrix_enabled)
    }

    fn validate(&self) -> Result<(), AgentRegistryError> {
        if self.agents.is_empty() {
            return Err(AgentRegistryError::Validation(
                "registry must contain at least one [[agents]] entry".to_string(),
            ));
        }

        let mut seen_agent_ids = BTreeMap::<&str, &str>::new();
        let mut seen_crate_paths = BTreeMap::<&str, &str>::new();
        let mut seen_backend_modules = BTreeMap::<&str, &str>::new();
        let mut seen_manifest_roots = BTreeMap::<&str, &str>::new();
        let mut seen_package_names = BTreeMap::<&str, &str>::new();

        for agent in &self.agents {
            agent.validate()?;
            record_unique(
                "agent_id",
                &agent.agent_id,
                &agent.agent_id,
                &mut seen_agent_ids,
            )?;
            record_unique(
                "crate_path",
                &agent.crate_path,
                &agent.agent_id,
                &mut seen_crate_paths,
            )?;
            record_unique(
                "backend_module",
                &agent.backend_module,
                &agent.agent_id,
                &mut seen_backend_modules,
            )?;
            record_unique(
                "manifest_root",
                &agent.manifest_root,
                &agent.agent_id,
                &mut seen_manifest_roots,
            )?;
            record_unique(
                "package_name",
                &agent.package_name,
                &agent.agent_id,
                &mut seen_package_names,
            )?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentRegistryEntry {
    pub agent_id: String,
    pub display_name: String,
    pub crate_path: String,
    pub backend_module: String,
    pub manifest_root: String,
    pub package_name: String,
    pub canonical_targets: Vec<String>,
    pub wrapper_coverage: WrapperCoverageBinding,
    pub capability_declaration: CapabilityDeclaration,
    pub publication: PublicationFlags,
    pub release: ReleaseMetadata,
    pub scaffold: ScaffoldMetadata,
    #[serde(default)]
    pub maintenance: MaintenanceMetadata,
}

impl AgentRegistryEntry {
    fn validate(&self) -> Result<(), AgentRegistryError> {
        validate_non_empty_scalar("agent_id", &self.agent_id)?;
        validate_non_empty_scalar("display_name", &self.display_name)?;
        validate_non_empty_scalar("package_name", &self.package_name)?;

        validate_repo_relative_path("crate_path", &self.crate_path)?;
        validate_repo_relative_path("backend_module", &self.backend_module)?;
        validate_repo_relative_path("manifest_root", &self.manifest_root)?;

        ensure_has_prefix("crate_path", &self.crate_path, CRATE_PATH_PREFIX)?;
        ensure_has_prefix(
            "backend_module",
            &self.backend_module,
            BACKEND_MODULE_PREFIX,
        )?;
        ensure_has_prefix("manifest_root", &self.manifest_root, MANIFEST_ROOT_PREFIX)?;

        let canonical_targets =
            validate_non_empty_string_array("canonical_targets", &self.canonical_targets)?;
        self.wrapper_coverage.validate()?;
        self.capability_declaration.validate(&canonical_targets)?;
        self.publication.validate(self)?;
        self.release.validate()?;
        self.scaffold.validate()?;
        self.maintenance.validate(self)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WrapperCoverageBinding {
    pub binding_kind: String,
    pub source_path: String,
}

impl WrapperCoverageBinding {
    fn validate(&self) -> Result<(), AgentRegistryError> {
        validate_non_empty_scalar("wrapper_coverage.binding_kind", &self.binding_kind)?;
        if self.binding_kind != WRAPPER_COVERAGE_BINDING_KIND_GENERATED_FROM_WRAPPER_CRATE {
            return Err(AgentRegistryError::Validation(format!(
                "wrapper_coverage.binding_kind must be `{WRAPPER_COVERAGE_BINDING_KIND_GENERATED_FROM_WRAPPER_CRATE}` (got `{}`)",
                self.binding_kind
            )));
        }

        validate_repo_relative_path("wrapper_coverage.source_path", &self.source_path)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityDeclaration {
    pub always_on: Vec<String>,
    #[serde(default)]
    pub target_gated: Vec<TargetGatedCapability>,
    #[serde(default)]
    pub config_gated: Vec<ConfigGatedCapability>,
    pub backend_extensions: Vec<String>,
}

impl CapabilityDeclaration {
    fn validate(&self, canonical_targets: &BTreeSet<&str>) -> Result<(), AgentRegistryError> {
        validate_string_array("capability_declaration.always_on", &self.always_on)?;
        validate_string_array(
            "capability_declaration.backend_extensions",
            &self.backend_extensions,
        )?;

        for entry in &self.target_gated {
            entry.validate(canonical_targets)?;
        }
        for entry in &self.config_gated {
            entry.validate(canonical_targets)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetGatedCapability {
    pub capability_id: String,
    pub targets: Vec<String>,
}

impl TargetGatedCapability {
    fn validate(&self, canonical_targets: &BTreeSet<&str>) -> Result<(), AgentRegistryError> {
        validate_non_empty_scalar(
            "capability_declaration.target_gated.capability_id",
            &self.capability_id,
        )?;
        validate_gate_targets(
            "capability_declaration.target_gated.targets",
            &self.targets,
            canonical_targets,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigGatedCapability {
    pub capability_id: String,
    pub config_key: String,
    pub targets: Option<Vec<String>>,
}

impl ConfigGatedCapability {
    fn validate(&self, canonical_targets: &BTreeSet<&str>) -> Result<(), AgentRegistryError> {
        validate_non_empty_scalar(
            "capability_declaration.config_gated.capability_id",
            &self.capability_id,
        )?;
        validate_non_empty_scalar(
            "capability_declaration.config_gated.config_key",
            &self.config_key,
        )?;
        validate_config_key_allowlist(
            &self.config_key,
            "capability_declaration.config_gated.config_key",
        )
        .map_err(AgentRegistryError::Validation)?;
        if let Some(targets) = &self.targets {
            validate_gate_targets(
                "capability_declaration.config_gated.targets",
                targets,
                canonical_targets,
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationFlags {
    pub support_matrix_enabled: bool,
    pub capability_matrix_enabled: bool,
    #[serde(default)]
    pub capability_matrix_target: Option<String>,
}

impl PublicationFlags {
    fn validate(&self, entry: &AgentRegistryEntry) -> Result<(), AgentRegistryError> {
        if let Some(target) = &self.capability_matrix_target {
            validate_non_empty_scalar("publication.capability_matrix_target", target)?;
        }
        resolve_capability_publication_target(entry)
            .map(|_| ())
            .map_err(AgentRegistryError::Validation)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseMetadata {
    pub docs_release_track: String,
}

impl ReleaseMetadata {
    fn validate(&self) -> Result<(), AgentRegistryError> {
        validate_non_empty_scalar("release.docs_release_track", &self.docs_release_track)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaffoldMetadata {
    pub onboarding_pack_prefix: String,
}

impl ScaffoldMetadata {
    fn validate(&self) -> Result<(), AgentRegistryError> {
        validate_non_empty_scalar(
            "scaffold.onboarding_pack_prefix",
            &self.onboarding_pack_prefix,
        )?;
        if self.onboarding_pack_prefix.contains('/') {
            return Err(AgentRegistryError::Validation(
                "scaffold.onboarding_pack_prefix must be a path prefix, not a path".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaintenanceMetadata {
    #[serde(default)]
    pub governance_checks: Vec<GovernanceCheck>,
}

impl MaintenanceMetadata {
    fn validate(&self, entry: &AgentRegistryEntry) -> Result<(), AgentRegistryError> {
        let mut seen_paths = BTreeSet::new();
        for check in &self.governance_checks {
            check.validate(entry)?;
            if !seen_paths.insert(check.path.as_str()) {
                return Err(AgentRegistryError::Validation(format!(
                    "maintenance.governance_checks contains duplicate path `{}`",
                    check.path
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GovernanceCheck {
    pub path: String,
    pub required: bool,
    pub comparison_kind: GovernanceComparisonKind,
    #[serde(default)]
    pub start_marker: Option<String>,
    #[serde(default)]
    pub end_marker: Option<String>,
    #[serde(default)]
    pub extraction_mode: Option<MarkdownExtractionMode>,
}

impl GovernanceCheck {
    fn validate(&self, entry: &AgentRegistryEntry) -> Result<(), AgentRegistryError> {
        validate_repo_relative_path("maintenance.governance_checks.path", &self.path)?;

        match self.comparison_kind {
            GovernanceComparisonKind::ApprovedAgentDescriptor => {
                if !self.path.ends_with("/governance/approved-agent.toml") {
                    return Err(AgentRegistryError::Validation(format!(
                        "maintenance.governance_checks.path must end with `/governance/approved-agent.toml` for `approved_agent_descriptor` (got `{}`)",
                        self.path
                    )));
                }
                if self.path
                    != format!(
                        "docs/agents/lifecycle/{}/governance/approved-agent.toml",
                        entry.scaffold.onboarding_pack_prefix
                    )
                {
                    return Err(AgentRegistryError::Validation(format!(
                        "maintenance.governance_checks.path `{}` must match onboarding_pack_prefix `{}` for `approved_agent_descriptor`",
                        self.path, entry.scaffold.onboarding_pack_prefix
                    )));
                }
                if self.start_marker.is_some()
                    || self.end_marker.is_some()
                    || self.extraction_mode.is_some()
                {
                    return Err(AgentRegistryError::Validation(
                        "maintenance.governance_checks for `approved_agent_descriptor` must not declare markdown parser config".to_string(),
                    ));
                }
            }
            GovernanceComparisonKind::MarkdownCapabilityClaim => {
                self.validate_markdown_config(MarkdownExtractionMode::InlineCodeIds)?;
            }
            GovernanceComparisonKind::MarkdownSupportClaim => {
                self.validate_markdown_config(MarkdownExtractionMode::SupportStateLines)?;
            }
        }

        Ok(())
    }

    fn validate_markdown_config(
        &self,
        expected_mode: MarkdownExtractionMode,
    ) -> Result<(), AgentRegistryError> {
        if !self.path.ends_with(".md") {
            return Err(AgentRegistryError::Validation(format!(
                "maintenance.governance_checks.path must end with `.md` for `{}` (got `{}`)",
                self.comparison_kind.as_str(),
                self.path
            )));
        }

        let start_marker = self.start_marker.as_deref().ok_or_else(|| {
            AgentRegistryError::Validation(format!(
                "maintenance.governance_checks `{}` is missing required `start_marker` for `{}`",
                self.path,
                self.comparison_kind.as_str()
            ))
        })?;
        let end_marker = self.end_marker.as_deref().ok_or_else(|| {
            AgentRegistryError::Validation(format!(
                "maintenance.governance_checks `{}` is missing required `end_marker` for `{}`",
                self.path,
                self.comparison_kind.as_str()
            ))
        })?;
        validate_non_empty_scalar("maintenance.governance_checks.start_marker", start_marker)?;
        validate_non_empty_scalar("maintenance.governance_checks.end_marker", end_marker)?;
        if start_marker == end_marker {
            return Err(AgentRegistryError::Validation(format!(
                "maintenance.governance_checks `{}` must use distinct `start_marker` and `end_marker`",
                self.path
            )));
        }

        match self.extraction_mode {
            Some(mode) if mode == expected_mode => Ok(()),
            Some(mode) => Err(AgentRegistryError::Validation(format!(
                "maintenance.governance_checks `{}` must use extraction_mode `{}` for `{}` (got `{}`)",
                self.path,
                expected_mode.as_str(),
                self.comparison_kind.as_str(),
                mode.as_str()
            ))),
            None => Err(AgentRegistryError::Validation(format!(
                "maintenance.governance_checks `{}` is missing required `extraction_mode` for `{}`",
                self.path,
                self.comparison_kind.as_str()
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceComparisonKind {
    ApprovedAgentDescriptor,
    MarkdownCapabilityClaim,
    MarkdownSupportClaim,
}

impl GovernanceComparisonKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::ApprovedAgentDescriptor => "approved_agent_descriptor",
            Self::MarkdownCapabilityClaim => "markdown_capability_claim",
            Self::MarkdownSupportClaim => "markdown_support_claim",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkdownExtractionMode {
    InlineCodeIds,
    SupportStateLines,
}

impl MarkdownExtractionMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::InlineCodeIds => "inline_code_ids",
            Self::SupportStateLines => "support_state_lines",
        }
    }
}

#[derive(Debug, Error)]
pub enum AgentRegistryError {
    #[error("read {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("parse agent registry TOML: {0}")]
    Toml(#[from] toml_edit::de::Error),
    #[error("{0}")]
    Validation(String),
}

fn record_unique<'a>(
    field_name: &str,
    value: &'a str,
    agent_id: &'a str,
    seen: &mut BTreeMap<&'a str, &'a str>,
) -> Result<(), AgentRegistryError> {
    if let Some(previous_agent) = seen.insert(value, agent_id) {
        return Err(AgentRegistryError::Validation(format!(
            "duplicate {field_name} `{value}` for agents `{previous_agent}` and `{agent_id}`"
        )));
    }
    Ok(())
}

fn validate_non_empty_scalar(field_name: &str, value: &str) -> Result<(), AgentRegistryError> {
    if value.trim().is_empty() {
        return Err(AgentRegistryError::Validation(format!(
            "{field_name} must not be empty"
        )));
    }
    Ok(())
}

fn validate_repo_relative_path(field_name: &str, value: &str) -> Result<(), AgentRegistryError> {
    validate_non_empty_scalar(field_name, value)?;

    let path = Path::new(value);
    if path.is_absolute() {
        return Err(AgentRegistryError::Validation(format!(
            "{field_name} must be repo-relative (got absolute path `{value}`)"
        )));
    }

    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir | Component::ParentDir => {
                return Err(AgentRegistryError::Validation(format!(
                    "{field_name} must be normalized and repo-relative (got `{value}`)"
                )));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(AgentRegistryError::Validation(format!(
                    "{field_name} must be repo-relative (got `{value}`)"
                )));
            }
        }
    }

    Ok(())
}

fn ensure_has_prefix(
    field_name: &str,
    value: &str,
    expected_prefix: &str,
) -> Result<(), AgentRegistryError> {
    if !value.starts_with(expected_prefix) {
        return Err(AgentRegistryError::Validation(format!(
            "{field_name} must start with `{expected_prefix}` (got `{value}`)"
        )));
    }
    Ok(())
}

fn validate_non_empty_string_array<'a>(
    field_name: &str,
    values: &'a [String],
) -> Result<BTreeSet<&'a str>, AgentRegistryError> {
    if values.is_empty() {
        return Err(AgentRegistryError::Validation(format!(
            "{field_name} must contain at least one entry"
        )));
    }

    let mut out = BTreeSet::new();
    for value in values {
        validate_non_empty_scalar(field_name, value)?;
        if !out.insert(value.as_str()) {
            return Err(AgentRegistryError::Validation(format!(
                "{field_name} contains duplicate entry `{value}`"
            )));
        }
    }

    Ok(out)
}

fn validate_string_array(field_name: &str, values: &[String]) -> Result<(), AgentRegistryError> {
    let mut seen = BTreeSet::new();
    for value in values {
        validate_non_empty_scalar(field_name, value)?;
        if !seen.insert(value.as_str()) {
            return Err(AgentRegistryError::Validation(format!(
                "{field_name} contains duplicate entry `{value}`"
            )));
        }
    }
    Ok(())
}

fn validate_gate_targets(
    field_name: &str,
    targets: &[String],
    canonical_targets: &BTreeSet<&str>,
) -> Result<(), AgentRegistryError> {
    if targets.is_empty() {
        return Err(AgentRegistryError::Validation(format!(
            "{field_name} must contain at least one target"
        )));
    }

    let mut seen = BTreeSet::new();
    for target in targets {
        validate_non_empty_scalar(field_name, target)?;
        if !seen.insert(target.as_str()) {
            return Err(AgentRegistryError::Validation(format!(
                "{field_name} contains duplicate target `{target}`"
            )));
        }
        if !canonical_targets.contains(target.as_str()) {
            return Err(AgentRegistryError::Validation(format!(
                "{field_name} references undeclared canonical target `{target}`"
            )));
        }
    }

    Ok(())
}
