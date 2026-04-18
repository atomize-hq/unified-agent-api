use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use xtask::support_matrix::{
    derive_rows, derive_rows_for_test_roots, validate_publication_consistency, BackendSupportState,
    ManifestSupportState, PointerPromotionState, SupportRow, UaaSupportState,
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

fn find_row<'a>(
    rows: &'a [SupportRow],
    agent: &str,
    version: &str,
    target: &str,
) -> &'a SupportRow {
    rows.iter()
        .find(|row| row.agent == agent && row.version == version && row.target == target)
        .unwrap_or_else(|| panic!("missing row {agent} {version} {target}"))
}

#[test]
fn derives_target_scoped_rows_with_sparse_caveats_and_pointer_state() {
    let workspace = make_temp_dir("support-matrix-derivation");

    materialize_root(
        &workspace.join("cli_manifests/codex"),
        &["linux-x64", "win32-x64"],
        "1.0.0",
        &["linux-x64"],
        &[("0.9.0", &["linux-x64"]), ("1.0.0", &["linux-x64"])],
        &[("linux-x64", "0.9.0")],
        &[("linux-x64", "1.0.0")],
        &[
            (
                "0.9.0",
                "coverage.linux-x64.json",
                json!({
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
            ),
            (
                "1.0.0",
                "coverage.linux-x64.json",
                json!({
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
            ),
            (
                "1.0.0",
                "coverage.win32-x64.json",
                json!({
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
            ),
        ],
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
            json!({
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
        &workspace.join("cli_manifests/opencode"),
        &["linux-x64"],
        "3.0.0",
        &["linux-x64"],
        &[("3.0.0", &["linux-x64"])],
        &[],
        &[],
        &[],
    );

    let rows = derive_rows(&workspace).expect("derive rows");
    validate_publication_consistency(&workspace, &rows)
        .expect("derived rows should satisfy the shared consistency helper");
    assert_eq!(
        rows.len(),
        6,
        "expected two codex versions x two targets + one claude row + one opencode row"
    );

    let claude_row = find_row(&rows, "claude_code", "2.0.0", "linux-x64");
    assert_eq!(claude_row.manifest_support, ManifestSupportState::Supported);
    assert_eq!(claude_row.backend_support, BackendSupportState::Supported);
    assert_eq!(claude_row.uaa_support, UaaSupportState::Supported);
    assert_eq!(
        claude_row.pointer_promotion,
        PointerPromotionState::LatestSupportedAndValidated
    );
    assert!(claude_row.evidence_notes.is_empty());

    let codex_historical = find_row(&rows, "codex", "0.9.0", "linux-x64");
    assert_eq!(
        codex_historical.manifest_support,
        ManifestSupportState::Supported
    );
    assert_eq!(
        codex_historical.backend_support,
        BackendSupportState::Supported
    );
    assert_eq!(codex_historical.uaa_support, UaaSupportState::Supported);
    assert_eq!(
        codex_historical.pointer_promotion,
        PointerPromotionState::LatestSupported
    );
    assert!(
        codex_historical.evidence_notes.is_empty(),
        "historical rows should still derive even when current.json points at a newer version"
    );

    let codex_current = find_row(&rows, "codex", "1.0.0", "linux-x64");
    assert_eq!(
        codex_current.manifest_support,
        ManifestSupportState::Supported
    );
    assert_eq!(codex_current.backend_support, BackendSupportState::Partial);
    assert_eq!(codex_current.uaa_support, UaaSupportState::Partial);
    assert_eq!(
        codex_current.pointer_promotion,
        PointerPromotionState::LatestValidated
    );
    assert_eq!(
        codex_current.evidence_notes,
        vec!["backend report includes backend-only surface outside unified support".to_string()]
    );

    let codex_missing_target = find_row(&rows, "codex", "1.0.0", "win32-x64");
    assert_eq!(
        codex_missing_target.manifest_support,
        ManifestSupportState::Unsupported
    );
    assert_eq!(
        codex_missing_target.backend_support,
        BackendSupportState::Partial
    );
    assert_eq!(
        codex_missing_target.uaa_support,
        UaaSupportState::Unsupported
    );
    assert_eq!(
        codex_missing_target.pointer_promotion,
        PointerPromotionState::None
    );
    assert_eq!(
        codex_missing_target.evidence_notes,
        vec![
            "backend report includes backend-only surface outside unified support".to_string(),
            "current root snapshot omits this target".to_string()
        ]
    );

    let opencode_row = find_row(&rows, "opencode", "3.0.0", "linux-x64");
    assert_eq!(
        opencode_row.manifest_support,
        ManifestSupportState::Supported
    );
    assert_eq!(
        opencode_row.backend_support,
        BackendSupportState::Unsupported
    );
    assert_eq!(opencode_row.uaa_support, UaaSupportState::Unsupported);
    assert_eq!(opencode_row.pointer_promotion, PointerPromotionState::None);
    assert!(opencode_row.evidence_notes.is_empty());

    assert_eq!(rows[0].agent, "claude_code");
    assert_eq!(rows[1].agent, "codex");
    assert_eq!(rows[1].target, "linux-x64");
    assert_eq!(rows[1].version, "1.0.0");
    assert_eq!(rows[2].version, "0.9.0");
    assert_eq!(rows[5].agent, "opencode");
}

#[test]
fn derives_rows_for_codex_claude_and_synthetic_future_agent_roots() {
    let workspace = make_temp_dir("support-matrix-derivation-future-agent");

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
            json!({
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
            json!({
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
        &workspace.join("cli_manifests/future_agent"),
        &["linux-x64"],
        "3.0.0",
        &["linux-x64"],
        &[("3.0.0", &["linux-x64"])],
        &[("linux-x64", "3.0.0")],
        &[("linux-x64", "3.0.0")],
        &[(
            "3.0.0",
            "coverage.linux-x64.json",
            json!({
                "deltas": {
                    "missing_commands": [],
                    "missing_flags": [],
                    "missing_args": [],
                    "intentionally_unsupported": [
                        { "path": ["future-only"] }
                    ],
                    "wrapper_only_commands": [],
                    "wrapper_only_flags": [],
                    "wrapper_only_args": [],
                }
            }),
        )],
    );

    let rows = derive_rows_for_test_roots(
        &workspace,
        &[
            ("codex", "cli_manifests/codex"),
            ("claude_code", "cli_manifests/claude_code"),
            ("future_agent", "cli_manifests/future_agent"),
        ],
    )
    .expect("derive rows for codex, claude, and future-agent-shaped roots");

    assert_eq!(rows.len(), 3, "expected one row per fixture root");

    let codex_row = find_row(&rows, "codex", "1.0.0", "linux-x64");
    assert_eq!(codex_row.manifest_support, ManifestSupportState::Supported);
    assert_eq!(codex_row.backend_support, BackendSupportState::Supported);
    assert_eq!(codex_row.uaa_support, UaaSupportState::Supported);

    let claude_row = find_row(&rows, "claude_code", "2.0.0", "linux-x64");
    assert_eq!(claude_row.manifest_support, ManifestSupportState::Supported);
    assert_eq!(claude_row.backend_support, BackendSupportState::Supported);
    assert_eq!(claude_row.uaa_support, UaaSupportState::Supported);

    let future_row = find_row(&rows, "future_agent", "3.0.0", "linux-x64");
    assert_eq!(future_row.manifest_support, ManifestSupportState::Supported);
    assert_eq!(future_row.backend_support, BackendSupportState::Partial);
    assert_eq!(future_row.uaa_support, UaaSupportState::Partial);
    assert_eq!(
        future_row.pointer_promotion,
        PointerPromotionState::LatestSupportedAndValidated
    );
    assert_eq!(
        future_row.evidence_notes,
        vec![
            "backend report includes intentionally unsupported surface outside unified support"
                .to_string()
        ]
    );
}

#[test]
fn missing_version_coverage_defaults_to_manifest_unsupported() {
    let workspace = make_temp_dir("support-matrix-derivation-missing-coverage");

    materialize_root(
        &workspace.join("cli_manifests/opencode"),
        &["linux-x64"],
        "1.4.9",
        &["linux-x64"],
        &[("1.4.9", &["linux-x64"])],
        &[],
        &[],
        &[],
    );
    write_json(
        &workspace.join("cli_manifests/opencode/versions/1.4.9.json"),
        &json!({
            "semantic_version": "1.4.9",
            "status": "snapshotted",
        }),
    );

    let rows = derive_rows_for_test_roots(&workspace, &[("opencode", "cli_manifests/opencode")])
        .expect("derive rows for opencode root without coverage block");
    let row = find_row(&rows, "opencode", "1.4.9", "linux-x64");

    assert_eq!(row.manifest_support, ManifestSupportState::Unsupported);
    assert_eq!(row.backend_support, BackendSupportState::Unsupported);
    assert_eq!(row.uaa_support, UaaSupportState::Unsupported);
    assert_eq!(row.pointer_promotion, PointerPromotionState::None);
    assert!(row.evidence_notes.is_empty());
}
