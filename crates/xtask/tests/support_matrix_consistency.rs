#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

#[path = "../src/support_matrix.rs"]
mod support_matrix;
#[path = "../src/wrapper_coverage_shared.rs"]
mod wrapper_coverage_shared;

use support_matrix::{
    derive_rows, validate_publication_consistency, BackendSupportState, ManifestSupportState,
    PointerPromotionState, UaaSupportState,
};

fn make_temp_dir(prefix: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after unix epoch");
    let dir = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        now.as_nanos()
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn write_text(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

fn write_json(path: &Path, value: &Value) {
    let text = serde_json::to_string_pretty(value).expect("serialize json");
    write_text(path, &format!("{text}\n"));
}

#[allow(clippy::too_many_arguments)]
fn materialize_root(
    root: &Path,
    expected_targets: &[&str],
    current_version: &str,
    current_targets: &[&str],
    versions: &[(&str, &[&str])],
    pointers_supported: &[(&str, &str)],
    pointers_validated: &[(&str, &str)],
    reports: &[(&str, &str, Value)],
) {
    let inputs = current_targets
        .iter()
        .map(|target| {
            json!({
                "target_triple": target,
                "binary": { "semantic_version": current_version },
            })
        })
        .collect::<Vec<_>>();

    write_json(
        &root.join("current.json"),
        &json!({
            "expected_targets": expected_targets,
            "inputs": inputs,
        }),
    );

    for (version, supported_targets) in versions {
        write_json(
            &root.join("versions").join(format!("{version}.json")),
            &json!({
                "semantic_version": version,
                "coverage": {
                    "supported_targets": supported_targets,
                },
            }),
        );
    }

    for target in expected_targets {
        let latest_supported = pointers_supported
            .iter()
            .find_map(|(candidate, version)| (*candidate == *target).then_some(*version))
            .unwrap_or("none");
        let latest_validated = pointers_validated
            .iter()
            .find_map(|(candidate, version)| (*candidate == *target).then_some(*version))
            .unwrap_or("none");
        write_text(
            &root
                .join("pointers/latest_supported")
                .join(format!("{target}.txt")),
            &format!("{latest_supported}\n"),
        );
        write_text(
            &root
                .join("pointers/latest_validated")
                .join(format!("{target}.txt")),
            &format!("{latest_validated}\n"),
        );
    }

    for (version, report_name, report) in reports {
        write_json(
            &root.join("reports").join(version).join(report_name),
            report,
        );
    }
}

#[test]
fn publication_consistency_passes_for_matching_rows() {
    let workspace = make_temp_dir("support-matrix-consistency-pass");

    materialize_root(
        &workspace.join("cli_manifests/codex"),
        &["linux-x64", "win32-x64"],
        "1.0.0",
        &["linux-x64"],
        &[("0.9.0", &["linux-x64"]), ("1.0.0", &["linux-x64"])],
        &[("linux-x64", "0.9.0")],
        &[("linux-x64", "1.0.0")],
        &[(
            "1.0.0",
            "coverage.linux-x64.json",
            serde_json::json!({
                "deltas": {
                    "missing_commands": [],
                    "missing_flags": [],
                    "missing_args": [],
                    "intentionally_unsupported": [],
                    "wrapper_only_commands": [
                        { "path": ["backend-only"] }
                    ],
                    "wrapper_only_flags": [],
                    "wrapper_only_args": [],
                }
            }),
        )],
    );

    materialize_root(
        &workspace.join("cli_manifests/claude_code"),
        &["linux-x64"],
        "2.0.0",
        &["linux-x64"],
        &[("2.0.0", &["linux-x64"])],
        &[("linux-x64", "2.0.0")],
        &[("linux-x64", "2.0.0")],
        &[(
            "2.0.0",
            "coverage.linux-x64.json",
            serde_json::json!({
                "deltas": {
                    "missing_commands": [],
                    "missing_flags": [],
                    "missing_args": [],
                    "intentionally_unsupported": [],
                    "wrapper_only_commands": [],
                    "wrapper_only_flags": [],
                    "wrapper_only_args": [],
                }
            }),
        )],
    );

    let rows = derive_rows(&workspace).expect("derive rows");
    validate_publication_consistency(&workspace, &rows).expect("matching rows should pass");
}

#[test]
fn publication_consistency_rejects_pointer_promotion_drift() {
    let workspace = make_temp_dir("support-matrix-consistency-pointer");

    materialize_root(
        &workspace.join("cli_manifests/codex"),
        &["linux-x64"],
        "1.0.0",
        &["linux-x64"],
        &[("1.0.0", &["linux-x64"])],
        &[("linux-x64", "1.0.0")],
        &[("linux-x64", "1.0.0")],
        &[(
            "1.0.0",
            "coverage.linux-x64.json",
            serde_json::json!({
                "deltas": {
                    "missing_commands": [],
                    "missing_flags": [],
                    "missing_args": [],
                    "intentionally_unsupported": [],
                    "wrapper_only_commands": [],
                    "wrapper_only_flags": [],
                    "wrapper_only_args": [],
                }
            }),
        )],
    );

    materialize_root(
        &workspace.join("cli_manifests/claude_code"),
        &["linux-x64"],
        "2.0.0",
        &["linux-x64"],
        &[("2.0.0", &["linux-x64"])],
        &[("linux-x64", "2.0.0")],
        &[("linux-x64", "2.0.0")],
        &[(
            "2.0.0",
            "coverage.linux-x64.json",
            serde_json::json!({
                "deltas": {
                    "missing_commands": [],
                    "missing_flags": [],
                    "missing_args": [],
                    "intentionally_unsupported": [],
                    "wrapper_only_commands": [],
                    "wrapper_only_flags": [],
                    "wrapper_only_args": [],
                }
            }),
        )],
    );

    let mut rows = derive_rows(&workspace).expect("derive rows");
    let row = rows
        .iter_mut()
        .find(|row| row.agent == "codex" && row.version == "1.0.0" && row.target == "linux-x64")
        .expect("expected codex row");
    row.pointer_promotion = PointerPromotionState::None;

    let issues = validate_publication_consistency(&workspace, &rows)
        .expect_err("pointer drift should be rejected");
    assert!(
        issues
            .iter()
            .any(|issue| issue.code == "SUPPORT_MATRIX_POINTER_PROMOTION_MISMATCH"),
        "expected pointer promotion mismatch, got: {issues:#?}"
    );
}

#[test]
fn publication_consistency_rejects_omission_claim_and_note_drift() {
    let workspace = make_temp_dir("support-matrix-consistency-omission");

    materialize_root(
        &workspace.join("cli_manifests/codex"),
        &["linux-x64", "win32-x64"],
        "1.0.0",
        &["linux-x64"],
        &[("1.0.0", &["linux-x64"])],
        &[("linux-x64", "1.0.0")],
        &[("linux-x64", "1.0.0")],
        &[(
            "1.0.0",
            "coverage.linux-x64.json",
            serde_json::json!({
                "deltas": {
                    "missing_commands": [],
                    "missing_flags": [],
                    "missing_args": [],
                    "intentionally_unsupported": [],
                    "wrapper_only_commands": [],
                    "wrapper_only_flags": [],
                    "wrapper_only_args": [],
                }
            }),
        )],
    );

    materialize_root(
        &workspace.join("cli_manifests/claude_code"),
        &["linux-x64"],
        "2.0.0",
        &["linux-x64"],
        &[("2.0.0", &["linux-x64"])],
        &[("linux-x64", "2.0.0")],
        &[("linux-x64", "2.0.0")],
        &[(
            "2.0.0",
            "coverage.linux-x64.json",
            serde_json::json!({
                "deltas": {
                    "missing_commands": [],
                    "missing_flags": [],
                    "missing_args": [],
                    "intentionally_unsupported": [],
                    "wrapper_only_commands": [],
                    "wrapper_only_flags": [],
                    "wrapper_only_args": [],
                }
            }),
        )],
    );

    let mut rows = derive_rows(&workspace).expect("derive rows");
    let row = rows
        .iter_mut()
        .find(|row| row.agent == "codex" && row.version == "1.0.0" && row.target == "win32-x64")
        .expect("expected omitted codex row");
    row.manifest_support = ManifestSupportState::Supported;
    row.backend_support = BackendSupportState::Supported;
    row.uaa_support = UaaSupportState::Supported;
    row.pointer_promotion = PointerPromotionState::LatestValidated;
    row.evidence_notes.clear();

    let issues = validate_publication_consistency(&workspace, &rows)
        .expect_err("omission contradiction should be rejected");
    assert!(
        issues
            .iter()
            .any(|issue| issue.code == "SUPPORT_MATRIX_CURRENT_SNAPSHOT_OMISSION_MISMATCH"),
        "expected omission mismatch, got: {issues:#?}"
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.code == "SUPPORT_MATRIX_EVIDENCE_NOTES_MISMATCH"),
        "expected note mismatch, got: {issues:#?}"
    );
}

#[test]
fn publication_consistency_rejects_status_drift_for_latest_validated_rows() {
    let workspace = make_temp_dir("support-matrix-consistency-status");

    materialize_root(
        &workspace.join("cli_manifests/codex"),
        &["linux-x64"],
        "1.0.0",
        &["linux-x64"],
        &[("1.0.0", &["linux-x64"])],
        &[("linux-x64", "1.0.0")],
        &[("linux-x64", "1.0.0")],
        &[(
            "1.0.0",
            "coverage.linux-x64.json",
            serde_json::json!({
                "deltas": {
                    "missing_commands": [],
                    "missing_flags": [],
                    "missing_args": [],
                    "intentionally_unsupported": [],
                    "wrapper_only_commands": [],
                    "wrapper_only_flags": [],
                    "wrapper_only_args": [],
                }
            }),
        )],
    );
    write_json(
        &workspace.join("cli_manifests/codex/versions/1.0.0.json"),
        &json!({
            "semantic_version": "1.0.0",
            "status": "reported",
            "coverage": {
                "supported_targets": ["linux-x64"],
            },
        }),
    );

    materialize_root(
        &workspace.join("cli_manifests/claude_code"),
        &["linux-x64"],
        "2.0.0",
        &["linux-x64"],
        &[("2.0.0", &["linux-x64"])],
        &[("linux-x64", "2.0.0")],
        &[("linux-x64", "2.0.0")],
        &[(
            "2.0.0",
            "coverage.linux-x64.json",
            serde_json::json!({
                "deltas": {
                    "missing_commands": [],
                    "missing_flags": [],
                    "missing_args": [],
                    "intentionally_unsupported": [],
                    "wrapper_only_commands": [],
                    "wrapper_only_flags": [],
                    "wrapper_only_args": [],
                }
            }),
        )],
    );

    let rows = derive_rows(&workspace).expect("derive rows");
    let issues = validate_publication_consistency(&workspace, &rows)
        .expect_err("reported status should not allow latest_validated promotion");
    assert!(
        issues
            .iter()
            .any(|issue| issue.code == "SUPPORT_MATRIX_VERSION_STATUS_MISMATCH"),
        "expected status mismatch, got: {issues:#?}"
    );
}
