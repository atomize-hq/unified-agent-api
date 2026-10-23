//! One isolated CI producer. Generated witnesses and previews grant no write/admission authority.
use crate::{
    depth_evidence::{
        payload::{DependencyGraph, Environment},
        Build, Cases, Provenance,
    },
    depth_record::{DepthRecord, FileRef},
};
use serde::{Deserialize, Serialize};
use std::{path::Path, process::Command};

mod collector;
mod graph;
pub const REPOSITORY: &str = "atomize-hq/unified-agent-api";
pub const FIXTURE: &str = "crates/xtask/tests/fixtures/depth-ci/record.json";
pub const CONTRACT: &str = "crates/xtask/tests/fixtures/depth-ci/contract.md";
pub const ASSERTION: &str = "crates/wrapper_events/tests/depth_ci_channel.rs";
pub const PRODUCTION: &str = "crates/wrapper_events/src/channel.rs";
pub const COLLECTOR: &str = "crates/xtask/src/depth_ci/collector.rs";
pub const ARCHIVE: &str = "scripts/depth_ci_archive.py";
pub const TARGET: &str = "x86_64-unknown-linux-gnu";
pub const TOOLCHAIN: &str = "1.89.0";
pub const CASES: [&str; 8] = [
    "accepts_complete_ascii_grammar",
    "accepts_exact_maximum",
    "preserves_case_and_path_separators",
    "rejects_empty",
    "rejects_invalid_characters",
    "rejects_invalid_prefix",
    "rejects_non_ascii",
    "rejects_over_limit",
];

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Witness {
    pub schema_version: u64,
    pub run_id: u64,
    pub run_attempt: u64,
    pub workflow_commit: String,
    pub build: Build,
    pub graph: DependencyGraph,
    pub environment: Environment,
    pub provenance: Vec<Provenance>,
    pub cases: Cases,
    pub toolchain_report: String,
    pub raw_output: String,
}
pub fn collect(workspace: &Path) -> Result<(), String> {
    collector::collect(workspace)
}
pub(crate) fn reference(path: impl Into<String>, bytes: &[u8]) -> FileRef {
    use sha2::{Digest, Sha256};
    FileRef {
        path: path.into(),
        sha256: hex::encode(Sha256::digest(bytes)),
    }
}
pub(crate) fn record(raw: &str) -> Result<DepthRecord, String> {
    DepthRecord::parse(raw, "codex", "0.0.0")
}
pub(crate) fn output(command: &mut Command) -> Result<Vec<u8>, String> {
    let out = command.output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err("isolated producer command failed; inspect local/CI command logs".into());
    }
    Ok(out.stdout)
}
