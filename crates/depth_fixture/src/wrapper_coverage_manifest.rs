//! Wrapper coverage source of truth for the synthetic `depth_fixture` CLI.
//!
//! `xtask depth-fixture-wrapper-coverage` turns this into `wrapper_coverage.json` for the isolated
//! manifest root. The wrapper drives exactly `run --json [--model <NAME>] -- <PROMPT>`. The root
//! command, its flags and `run --help` are per-unit parity exclusions in RULES, not waivers here:
//! an intentionally unsupported command waives its whole subtree, which would hide new upstream
//! surface, and the support audit carries any waived unit as required work.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageLevel {
    Explicit,
    Passthrough,
    Unsupported,
    IntentionallyUnsupported,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrapperSurfaceScopedTargets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platforms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_triples: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrapperFlagCoverageV1 {
    pub key: String,
    pub level: CoverageLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<WrapperSurfaceScopedTargets>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrapperArgCoverageV1 {
    pub name: String,
    pub level: CoverageLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<WrapperSurfaceScopedTargets>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrapperCommandCoverageV1 {
    pub path: Vec<String>,
    pub level: CoverageLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<WrapperSurfaceScopedTargets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<WrapperFlagCoverageV1>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<WrapperArgCoverageV1>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrapperCoverageManifestV1 {
    pub schema_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrapper_version: Option<String>,
    pub coverage: Vec<WrapperCommandCoverageV1>,
}

pub fn wrapper_crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn flag(key: &str) -> WrapperFlagCoverageV1 {
    WrapperFlagCoverageV1 {
        key: key.to_owned(),
        level: CoverageLevel::Explicit,
        note: None,
        scope: None,
    }
}

/// The single source of truth for the wrapper's coverage declarations.
pub fn wrapper_coverage_manifest() -> WrapperCoverageManifestV1 {
    use CoverageLevel::Explicit;
    WrapperCoverageManifestV1 {
        schema_version: 1,
        generated_at: None,
        wrapper_version: None,
        coverage: vec![WrapperCommandCoverageV1 {
            path: vec!["run".to_owned()],
            level: Explicit,
            note: None,
            scope: None,
            flags: Some(vec![flag("--json"), flag("--model")]),
            args: Some(vec![WrapperArgCoverageV1 {
                name: "PROMPT".to_owned(),
                level: Explicit,
                note: None,
                scope: None,
            }]),
        }],
    }
}
