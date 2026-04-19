use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use codex::wrapper_coverage_manifest::{
    wrapper_coverage_manifest, wrapper_crate_version, CoverageLevel, WrapperArgCoverageV1,
    WrapperCommandCoverageV1, WrapperCoverageManifestV1, WrapperFlagCoverageV1,
    WrapperSurfaceScopedTargets,
};
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::wrapper_coverage_shared::{
    self as shared, ArgLike, CommandLike, CoverageLevelLike, FlagLike, ManifestLike, ScopeLike,
};

#[derive(Debug, Args)]
pub struct CliArgs {
    /// Output file path for `wrapper_coverage.json`.
    #[arg(long)]
    pub out: PathBuf,
    /// Path to RULES.json (used for expected target ordering + timestamp policy).
    #[arg(long, default_value = "cli_manifests/codex/RULES.json")]
    pub rules: PathBuf,
}

#[derive(Debug, Error)]
pub enum WrapperCoverageError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse rules.json: {0}")]
    RulesParse(#[from] serde_json::Error),
    #[error("unsupported rules: {0}")]
    RulesUnsupported(String),
    #[error("invalid wrapper coverage manifest: {0}")]
    ManifestInvalid(String),
}

impl From<shared::SharedError> for WrapperCoverageError {
    fn from(value: shared::SharedError) -> Self {
        match value {
            shared::SharedError::RulesUnsupported(message) => Self::RulesUnsupported(message),
            shared::SharedError::ManifestInvalid(message) => Self::ManifestInvalid(message),
        }
    }
}

pub fn run(args: CliArgs) -> Result<(), WrapperCoverageError> {
    let rules: shared::Rules = serde_json::from_slice(&fs::read(&args.rules)?)?;
    shared::assert_supported_rules(&rules)?;

    let expected_targets = rules.union.expected_targets;
    let platform_mapping = rules.union.platform_mapping;
    let platform_to_targets =
        shared::invert_platform_mapping(&expected_targets, &platform_mapping)?;

    let mut manifest: WrapperCoverageManifestV1 = wrapper_coverage_manifest();
    if manifest.schema_version != 1 {
        return Err(WrapperCoverageError::ManifestInvalid(format!(
            "schema_version must be 1 (got {})",
            manifest.schema_version
        )));
    }

    shared::normalize_manifest(&mut manifest, &expected_targets, &platform_to_targets)?;

    manifest.generated_at = Some(deterministic_rfc3339_now());
    manifest.wrapper_version = Some(wrapper_crate_version().to_string());

    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| WrapperCoverageError::ManifestInvalid(e.to_string()))?;
    write_json_pretty(&args.out, &json)?;
    Ok(())
}

fn deterministic_rfc3339_now() -> String {
    if let Ok(v) = std::env::var("SOURCE_DATE_EPOCH") {
        if let Ok(secs) = v.parse::<i64>() {
            if let Ok(ts) = OffsetDateTime::from_unix_timestamp(secs) {
                return ts
                    .format(&Rfc3339)
                    .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());
            }
        }
    }
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn write_json_pretty(
    path: &Path,
    pretty_json_without_newline: &str,
) -> Result<(), WrapperCoverageError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, format!("{pretty_json_without_newline}\n"))?;
    Ok(())
}

impl CoverageLevelLike for CoverageLevel {
    fn sort_key(self) -> u8 {
        match self {
            CoverageLevel::Explicit => 0,
            CoverageLevel::Passthrough => 1,
            CoverageLevel::IntentionallyUnsupported => 2,
            CoverageLevel::Unsupported => 3,
            CoverageLevel::Unknown => 4,
        }
    }
}

impl ScopeLike for WrapperSurfaceScopedTargets {
    fn platforms(&self) -> Option<&[String]> {
        self.platforms.as_deref()
    }

    fn target_triples(&self) -> Option<&[String]> {
        self.target_triples.as_deref()
    }

    fn set_platforms(&mut self, platforms: Option<Vec<String>>) {
        self.platforms = platforms;
    }

    fn set_target_triples(&mut self, target_triples: Option<Vec<String>>) {
        self.target_triples = target_triples;
    }
}

impl FlagLike<WrapperSurfaceScopedTargets> for WrapperFlagCoverageV1 {
    type Level = CoverageLevel;

    fn key(&self) -> &str {
        &self.key
    }

    fn level(&self) -> Self::Level {
        self.level
    }

    fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    fn scope(&self) -> Option<&WrapperSurfaceScopedTargets> {
        self.scope.as_ref()
    }

    fn scope_mut(&mut self) -> &mut Option<WrapperSurfaceScopedTargets> {
        &mut self.scope
    }
}

impl ArgLike<WrapperSurfaceScopedTargets> for WrapperArgCoverageV1 {
    type Level = CoverageLevel;

    fn name(&self) -> &str {
        &self.name
    }

    fn level(&self) -> Self::Level {
        self.level
    }

    fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    fn scope(&self) -> Option<&WrapperSurfaceScopedTargets> {
        self.scope.as_ref()
    }

    fn scope_mut(&mut self) -> &mut Option<WrapperSurfaceScopedTargets> {
        &mut self.scope
    }
}

impl CommandLike<WrapperSurfaceScopedTargets, WrapperFlagCoverageV1, WrapperArgCoverageV1>
    for WrapperCommandCoverageV1
{
    type Level = CoverageLevel;

    fn path(&self) -> &[String] {
        &self.path
    }

    fn level(&self) -> Self::Level {
        self.level
    }

    fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    fn scope(&self) -> Option<&WrapperSurfaceScopedTargets> {
        self.scope.as_ref()
    }

    fn scope_mut(&mut self) -> &mut Option<WrapperSurfaceScopedTargets> {
        &mut self.scope
    }

    fn flags_mut(&mut self) -> &mut Option<Vec<WrapperFlagCoverageV1>> {
        &mut self.flags
    }

    fn args_mut(&mut self) -> &mut Option<Vec<WrapperArgCoverageV1>> {
        &mut self.args
    }
}

impl ManifestLike<WrapperCommandCoverageV1> for WrapperCoverageManifestV1 {
    fn coverage_mut(&mut self) -> &mut Vec<WrapperCommandCoverageV1> {
        &mut self.coverage
    }
}
