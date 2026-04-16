use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

const VERSION: &str = "0.61.0";
const TS: &str = "1970-01-01T00:00:00Z";

const REQUIRED_TARGET: &str = "x86_64-unknown-linux-musl";
const TARGETS: [&str; 3] = [
    "x86_64-unknown-linux-musl",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("CARGO_MANIFEST_DIR has crates/<crate> parent structure")
        .to_path_buf()
}

fn make_temp_dir(prefix: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after unix epoch");
    let unique = format!("{}-{}-{}", prefix, std::process::id(), now.as_nanos());

    let dir = std::env::temp_dir().join(unique);
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

fn copy_from_repo(codex_dir: &Path, filename: &str) {
    let src = repo_root()
        .join("cli_manifests")
        .join("codex")
        .join(filename);
    let dst = codex_dir.join(filename);
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).expect("mkdir dst parent");
    }
    fs::copy(&src, &dst).unwrap_or_else(|e| panic!("copy {:?} -> {:?}: {}", src, dst, e));
}

fn materialize_minimal_valid_codex_dir(codex_dir: &Path) {
    fs::create_dir_all(codex_dir).expect("mkdir codex dir");

    copy_from_repo(codex_dir, "SCHEMA.json");
    copy_from_repo(codex_dir, "RULES.json");
    copy_from_repo(codex_dir, "VERSION_METADATA_SCHEMA.json");

    write_text(
        &codex_dir.join("min_supported.txt"),
        &format!("{VERSION}\n"),
    );
    write_text(
        &codex_dir.join("latest_validated.txt"),
        &format!("{VERSION}\n"),
    );

    for target in TARGETS {
        let supported = codex_dir
            .join("pointers")
            .join("latest_supported")
            .join(format!("{target}.txt"));
        let validated = codex_dir
            .join("pointers")
            .join("latest_validated")
            .join(format!("{target}.txt"));

        if target == REQUIRED_TARGET {
            write_text(&supported, &format!("{VERSION}\n"));
            write_text(&validated, &format!("{VERSION}\n"));
        } else {
            write_text(&supported, "none\n");
            write_text(&validated, "none\n");
        }
    }

    let inputs = json!([{
        "target_triple": REQUIRED_TARGET,
        "collected_at": TS,
        "binary": {
            "sha256": "00",
            "size_bytes": 0,
            "platform": { "os": "linux", "arch": "x86_64" },
            "target_triple": REQUIRED_TARGET,
            "version_output": format!("codex-cli {VERSION}"),
            "semantic_version": VERSION,
            "channel": "stable",
        }
    }]);

    let union = json!({
        "snapshot_schema_version": 2,
        "tool": "codex-cli",
        "mode": "union",
        "collected_at": TS,
        "expected_targets": TARGETS,
        "complete": false,
        "missing_targets": [TARGETS[1], TARGETS[2]],
        "inputs": inputs,
        "commands": [],
    });

    let union_path = codex_dir.join("snapshots").join(VERSION).join("union.json");
    write_json(&union_path, &union);

    let per_target = json!({
        "snapshot_schema_version": 1,
        "tool": "codex-cli",
        "collected_at": TS,
        "binary": {
            "sha256": "00",
            "size_bytes": 0,
            "platform": { "os": "linux", "arch": "x86_64" },
            "target_triple": REQUIRED_TARGET,
            "version_output": format!("codex-cli {VERSION}"),
            "semantic_version": VERSION,
            "channel": "stable",
        },
        "commands": [],
    });
    write_json(
        &codex_dir
            .join("snapshots")
            .join(VERSION)
            .join(format!("{REQUIRED_TARGET}.json")),
        &per_target,
    );

    let union_text = fs::read_to_string(&union_path).expect("read union.json text");
    write_text(&codex_dir.join("current.json"), &union_text);

    let version_metadata = json!({
        "schema_version": 1,
        "semantic_version": VERSION,
        "status": "snapshotted",
        "updated_at": TS,
        "coverage": {
            "supported_targets": [REQUIRED_TARGET],
            "supported_required_target": true
        },
        "validation": {
            "passed_targets": [REQUIRED_TARGET],
            "failed_targets": [],
            "skipped_targets": []
        }
    });
    write_json(
        &codex_dir.join("versions").join(format!("{VERSION}.json")),
        &version_metadata,
    );

    let wrapper_coverage = json!({
        "schema_version": 1,
        "generated_at": TS,
        "wrapper_version": "0.0.0-test",
        "coverage": []
    });
    write_json(&codex_dir.join("wrapper_coverage.json"), &wrapper_coverage);
}

fn materialize_minimal_valid_claude_dir(claude_dir: &Path) {
    fs::create_dir_all(claude_dir).expect("mkdir claude dir");

    write_json(
        &claude_dir.join("current.json"),
        &json!({
            "expected_targets": [REQUIRED_TARGET],
            "inputs": [{
                "target_triple": REQUIRED_TARGET,
                "binary": {
                    "semantic_version": VERSION,
                }
            }],
        }),
    );

    write_text(
        &claude_dir
            .join("pointers")
            .join("latest_supported")
            .join(format!("{REQUIRED_TARGET}.txt")),
        &format!("{VERSION}\n"),
    );
    write_text(
        &claude_dir
            .join("pointers")
            .join("latest_validated")
            .join(format!("{REQUIRED_TARGET}.txt")),
        &format!("{VERSION}\n"),
    );

    write_json(
        &claude_dir.join("versions").join(format!("{VERSION}.json")),
        &json!({
            "semantic_version": VERSION,
            "status": "validated",
            "coverage": {
                "supported_targets": [REQUIRED_TARGET],
            },
        }),
    );

    write_json(
        &claude_dir
            .join("reports")
            .join(VERSION)
            .join(format!("coverage.{REQUIRED_TARGET}.json")),
        &json!({
            "inputs": {
                "upstream": {
                    "targets": [REQUIRED_TARGET],
                }
            },
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
    );
}

fn root_flag_from_help(help_text: &str) -> &'static str {
    if help_text.contains("--root") {
        "--root"
    } else if help_text.contains("--codex-dir") {
        "--codex-dir"
    } else {
        panic!("help did not contain --root or --codex-dir:\n{help_text}");
    }
}

fn run_xtask_validate(codex_dir: &Path) -> std::process::Output {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    let fixture_root = codex_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("codex_dir is <fixture_root>/cli_manifests/codex");

    let help = Command::new(&xtask_bin)
        .arg("codex-validate")
        .arg("--help")
        .current_dir(fixture_root)
        .output()
        .expect("spawn xtask codex-validate --help");
    assert!(
        help.status.success(),
        "xtask codex-validate --help failed:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        help.status,
        String::from_utf8_lossy(&help.stdout),
        String::from_utf8_lossy(&help.stderr)
    );
    let help_text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&help.stdout),
        String::from_utf8_lossy(&help.stderr)
    );

    let mut cmd = Command::new(xtask_bin);
    cmd.arg("codex-validate").current_dir(fixture_root);
    cmd.arg(root_flag_from_help(&help_text)).arg(codex_dir);
    cmd.output().expect("spawn xtask codex-validate")
}

fn write_invalid_report_fixture(codex_dir: &Path) {
    let report = json!({
        "schema_version": 1,
        "generated_at": TS,
        "inputs": {
            "upstream": {
                "semantic_version": VERSION,
                "mode": "union",
                "targets": [REQUIRED_TARGET]
            },
            "wrapper": {
                "schema_version": 1,
                "wrapper_version": "0.0.0-test"
            },
            "rules": {
                "rules_schema_version": 1
            }
        },
        "platform_filter": { "mode": "any" },
        "deltas": {
            "missing_commands": [],
            "missing_flags": [{
                "path": ["waived"],
                "key": "--bad",
                "upstream_available_on": [REQUIRED_TARGET],
                "wrapper_level": "intentionally_unsupported"
            }],
            "missing_args": []
        }
    });

    write_json(
        &codex_dir
            .join("reports")
            .join(VERSION)
            .join("coverage.any.json"),
        &report,
    );
}

fn write_support_matrix_artifact(workspace_root: &Path, rows: Value) {
    write_json(
        &workspace_root
            .join("cli_manifests")
            .join("support_matrix")
            .join("current.json"),
        &json!({
            "schema_version": 1,
            "rows": rows,
        }),
    );
}

fn write_complete_support_matrix_artifact(workspace_root: &Path) {
    write_support_matrix_artifact(
        workspace_root,
        json!([
            {
                "agent": "codex",
                "version": VERSION,
                "target": "aarch64-apple-darwin",
                "manifest_support": "unsupported",
                "backend_support": "unsupported",
                "uaa_support": "unsupported",
                "pointer_promotion": "none",
                "evidence_notes": ["current root snapshot omits this target"],
            },
            {
                "agent": "codex",
                "version": VERSION,
                "target": "x86_64-pc-windows-msvc",
                "manifest_support": "unsupported",
                "backend_support": "unsupported",
                "uaa_support": "unsupported",
                "pointer_promotion": "none",
                "evidence_notes": ["current root snapshot omits this target"],
            },
            {
                "agent": "codex",
                "version": VERSION,
                "target": REQUIRED_TARGET,
                "manifest_support": "supported",
                "backend_support": "unsupported",
                "uaa_support": "unsupported",
                "pointer_promotion": "latest_supported_and_validated",
                "evidence_notes": [],
            }
        ]),
    );
}

fn assert_violation_surface(output: &std::process::Output, code: &str, expected_path: &str) {
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains(code), "expected {code}, got:\n{combined}");
    assert!(
        combined.contains(expected_path),
        "expected {expected_path} in errors, got:\n{combined}"
    );
}

fn write_version_status(codex_dir: &Path, status: &str) {
    write_json(
        &codex_dir.join("versions").join(format!("{VERSION}.json")),
        &json!({
            "schema_version": 1,
            "semantic_version": VERSION,
            "status": status,
            "updated_at": TS,
            "coverage": {
                "supported_targets": [REQUIRED_TARGET],
                "supported_required_target": true
            },
            "validation": {
                "passed_targets": [REQUIRED_TARGET],
                "failed_targets": [],
                "skipped_targets": []
            }
        }),
    );
}

#[test]
fn c6_validator_rejects_missing_support_matrix_publication_artifact() {
    let temp = make_temp_dir("ccm-c6-support-matrix-artifact-missing");
    let codex_dir = temp.join("cli_manifests").join("codex");
    materialize_minimal_valid_codex_dir(&codex_dir);

    let output = run_xtask_validate(&codex_dir);
    assert!(
        !output.status.success(),
        "expected validation failure:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_violation_surface(
        &output,
        "SUPPORT_MATRIX_ARTIFACT_MISSING",
        "cli_manifests/support_matrix/current.json",
    );
}

#[test]
fn c6_validator_detects_version_status_drift_for_latest_validated_rows() {
    let temp = make_temp_dir("ccm-c6-support-matrix-status");
    let codex_dir = temp.join("cli_manifests").join("codex");
    materialize_minimal_valid_codex_dir(&codex_dir);
    write_version_status(&codex_dir, "reported");
    write_support_matrix_artifact(
        &temp,
        json!([
            {
                "agent": "codex",
                "version": VERSION,
                "target": REQUIRED_TARGET,
                "manifest_support": "supported",
                "backend_support": "unsupported",
                "uaa_support": "unsupported",
                "pointer_promotion": "latest_supported_and_validated",
                "evidence_notes": [],
            }
        ]),
    );

    let output = run_xtask_validate(&codex_dir);
    assert!(
        !output.status.success(),
        "expected validation failure:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_violation_surface(
        &output,
        "SUPPORT_MATRIX_VERSION_STATUS_MISMATCH",
        "cli_manifests/support_matrix/current.json",
    );
}

#[test]
fn c6_validator_detects_pointer_promotion_drift_in_support_matrix_publication() {
    let temp = make_temp_dir("ccm-c6-support-matrix-pointer");
    let codex_dir = temp.join("cli_manifests").join("codex");
    materialize_minimal_valid_codex_dir(&codex_dir);
    write_support_matrix_artifact(
        &temp,
        json!([
            {
                "agent": "codex",
                "version": VERSION,
                "target": REQUIRED_TARGET,
                "manifest_support": "supported",
                "backend_support": "unsupported",
                "uaa_support": "unsupported",
                "pointer_promotion": "none",
                "evidence_notes": [],
            }
        ]),
    );

    let output = run_xtask_validate(&codex_dir);
    assert!(
        !output.status.success(),
        "expected validation failure:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_violation_surface(
        &output,
        "SUPPORT_MATRIX_POINTER_PROMOTION_MISMATCH",
        "cli_manifests/support_matrix/current.json",
    );
}

#[test]
fn c6_validator_detects_support_state_drift_in_support_matrix_publication() {
    let temp = make_temp_dir("ccm-c6-support-matrix-support-state");
    let codex_dir = temp.join("cli_manifests").join("codex");
    materialize_minimal_valid_codex_dir(&codex_dir);
    write_complete_support_matrix_artifact(&temp);

    let artifact_path = temp
        .join("cli_manifests")
        .join("support_matrix")
        .join("current.json");
    let mut artifact: Value =
        serde_json::from_str(&fs::read_to_string(&artifact_path).expect("read artifact"))
            .expect("parse artifact");
    artifact["rows"][2]["manifest_support"] = json!("unsupported");
    write_json(&artifact_path, &artifact);

    let output = run_xtask_validate(&codex_dir);
    assert!(
        !output.status.success(),
        "expected validation failure:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_violation_surface(
        &output,
        "SUPPORT_MATRIX_MANIFEST_SUPPORT_MISMATCH",
        "cli_manifests/support_matrix/current.json",
    );
}

#[test]
fn c6_validator_detects_non_canonical_support_matrix_row_order() {
    let temp = make_temp_dir("ccm-c6-support-matrix-order");
    let codex_dir = temp.join("cli_manifests").join("codex");
    materialize_minimal_valid_codex_dir(&codex_dir);
    write_complete_support_matrix_artifact(&temp);

    let artifact_path = temp
        .join("cli_manifests")
        .join("support_matrix")
        .join("current.json");
    let mut artifact: Value =
        serde_json::from_str(&fs::read_to_string(&artifact_path).expect("read artifact"))
            .expect("parse artifact");
    artifact["rows"]
        .as_array_mut()
        .expect("rows array")
        .swap(0, 1);
    write_json(&artifact_path, &artifact);

    let output = run_xtask_validate(&codex_dir);
    assert!(
        !output.status.success(),
        "expected validation failure:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_violation_surface(
        &output,
        "SUPPORT_MATRIX_ROW_ORDER_MISMATCH",
        "cli_manifests/support_matrix/current.json",
    );
}

#[test]
fn c6_validator_rejects_incomplete_support_matrix_publication() {
    let temp = make_temp_dir("ccm-c6-support-matrix-missing");
    let codex_dir = temp.join("cli_manifests").join("codex");
    materialize_minimal_valid_codex_dir(&codex_dir);
    write_support_matrix_artifact(
        &temp,
        json!([
            {
                "agent": "codex",
                "version": VERSION,
                "target": REQUIRED_TARGET,
                "manifest_support": "supported",
                "backend_support": "unsupported",
                "uaa_support": "unsupported",
                "pointer_promotion": "latest_supported_and_validated",
                "evidence_notes": [],
            }
        ]),
    );

    let output = run_xtask_validate(&codex_dir);
    assert!(
        !output.status.success(),
        "expected validation failure:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_violation_surface(
        &output,
        "SUPPORT_MATRIX_ROW_MISSING",
        "cli_manifests/support_matrix/current.json",
    );
}

#[test]
fn c6_validator_rejects_missing_committed_agent_root_even_without_rows() {
    let temp = make_temp_dir("ccm-c6-support-matrix-missing-root");
    let codex_dir = temp.join("cli_manifests").join("codex");
    let claude_dir = temp.join("cli_manifests").join("claude_code");
    materialize_minimal_valid_codex_dir(&codex_dir);
    materialize_minimal_valid_claude_dir(&claude_dir);
    write_support_matrix_artifact(
        &temp,
        json!([
            {
                "agent": "codex",
                "version": VERSION,
                "target": "aarch64-apple-darwin",
                "manifest_support": "unsupported",
                "backend_support": "unsupported",
                "uaa_support": "unsupported",
                "pointer_promotion": "none",
                "evidence_notes": ["current root snapshot omits this target"],
            },
            {
                "agent": "codex",
                "version": VERSION,
                "target": "x86_64-pc-windows-msvc",
                "manifest_support": "unsupported",
                "backend_support": "unsupported",
                "uaa_support": "unsupported",
                "pointer_promotion": "none",
                "evidence_notes": ["current root snapshot omits this target"],
            },
            {
                "agent": "codex",
                "version": VERSION,
                "target": REQUIRED_TARGET,
                "manifest_support": "supported",
                "backend_support": "unsupported",
                "uaa_support": "unsupported",
                "pointer_promotion": "latest_supported_and_validated",
                "evidence_notes": [],
            }
        ]),
    );
    fs::remove_dir_all(&claude_dir).expect("remove committed claude root");

    let output = run_xtask_validate(&codex_dir);
    assert!(
        !output.status.success(),
        "expected validation failure:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_violation_surface(
        &output,
        "SUPPORT_MATRIX_ROOT_READ_ERROR",
        "cli_manifests/support_matrix/current.json",
    );
}

#[test]
fn c6_validator_detects_support_claim_drift_for_omitted_target() {
    let temp = make_temp_dir("ccm-c6-support-matrix-omission");
    let codex_dir = temp.join("cli_manifests").join("codex");
    materialize_minimal_valid_codex_dir(&codex_dir);
    write_support_matrix_artifact(
        &temp,
        json!([
            {
                "agent": "codex",
                "version": VERSION,
                "target": "aarch64-apple-darwin",
                "manifest_support": "supported",
                "backend_support": "unsupported",
                "uaa_support": "unsupported",
                "pointer_promotion": "none",
                "evidence_notes": [],
            }
        ]),
    );

    let output = run_xtask_validate(&codex_dir);
    assert!(
        !output.status.success(),
        "expected validation failure:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_violation_surface(
        &output,
        "SUPPORT_MATRIX_CURRENT_SNAPSHOT_OMISSION_MISMATCH",
        "cli_manifests/support_matrix/current.json",
    );
    assert_violation_surface(
        &output,
        "SUPPORT_MATRIX_EVIDENCE_NOTES_MISMATCH",
        "cli_manifests/support_matrix/current.json",
    );
}

#[test]
fn c6_validator_emits_report_missing_includes_intentionally_unsupported() {
    let temp = make_temp_dir("ccm-c6-report-iu-validator");
    let codex_dir = temp.join("cli_manifests").join("codex");
    materialize_minimal_valid_codex_dir(&codex_dir);
    write_invalid_report_fixture(&codex_dir);

    let output = run_xtask_validate(&codex_dir);
    assert!(
        !output.status.success(),
        "expected validation failure:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_violation_surface(
        &output,
        "REPORT_MISSING_INCLUDES_INTENTIONALLY_UNSUPPORTED",
        "reports/0.61.0/coverage.any.json",
    );
}
