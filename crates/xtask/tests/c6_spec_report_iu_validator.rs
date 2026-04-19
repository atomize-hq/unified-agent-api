use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use xtask::support_matrix::derive_rows_for_test_roots;

const VERSION: &str = "0.61.0";
const TS: &str = "1970-01-01T00:00:00Z";

const REQUIRED_TARGET: &str = "x86_64-unknown-linux-musl";
const TARGETS: [&str; 3] = [
    "x86_64-unknown-linux-musl",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
];
const CLAUDE_REQUIRED_TARGET: &str = "linux-x64";
const CLAUDE_TARGETS: [&str; 3] = ["linux-x64", "darwin-arm64", "win32-x64"];

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

fn write_workspace_manifest(workspace_root: &Path) {
    write_text(
        &workspace_root.join("Cargo.toml"),
        "[workspace]\nmembers = []\n",
    );
}

fn copy_from_repo(manifest_dir: &Path, agent: &str, filename: &str) {
    let src = repo_root().join("cli_manifests").join(agent).join(filename);
    let dst = manifest_dir.join(filename);
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).expect("mkdir dst parent");
    }
    fs::copy(&src, &dst).unwrap_or_else(|e| panic!("copy {:?} -> {:?}: {}", src, dst, e));
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("create destination directory");
    for entry in fs::read_dir(src).unwrap_or_else(|e| panic!("read_dir {:?}: {}", src, e)) {
        let entry = entry.unwrap_or_else(|e| panic!("read_dir entry {:?}: {}", src, e));
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if entry
            .file_type()
            .unwrap_or_else(|e| panic!("file_type {:?}: {}", src_path, e))
            .is_dir()
        {
            copy_dir_recursive(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path)
                .unwrap_or_else(|e| panic!("copy {:?} -> {:?}: {}", src_path, dst_path, e));
        }
    }
}

fn materialize_committed_opencode_dir(opencode_dir: &Path) {
    let src = repo_root().join("cli_manifests").join("opencode");
    copy_dir_recursive(&src, opencode_dir);
}

fn materialize_minimal_valid_codex_dir(codex_dir: &Path) {
    fs::create_dir_all(codex_dir).expect("mkdir codex dir");

    copy_from_repo(codex_dir, "codex", "SCHEMA.json");
    copy_from_repo(codex_dir, "codex", "RULES.json");
    copy_from_repo(codex_dir, "codex", "VERSION_METADATA_SCHEMA.json");

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
        "status": "validated",
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

    let base_report = json!({
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
        "deltas": {
            "missing_commands": [],
            "missing_flags": [],
            "missing_args": [],
            "intentionally_unsupported": [],
            "wrapper_only_commands": [],
            "wrapper_only_flags": [],
            "wrapper_only_args": [],
        }
    });
    let mut any_report = base_report.clone();
    any_report["platform_filter"] = json!({ "mode": "any" });
    write_json(
        &codex_dir
            .join("reports")
            .join(VERSION)
            .join("coverage.any.json"),
        &any_report,
    );

    let mut exact_report = base_report;
    exact_report["platform_filter"] = json!({
        "mode": "exact_target",
        "target_triple": REQUIRED_TARGET
    });
    write_json(
        &codex_dir
            .join("reports")
            .join(VERSION)
            .join(format!("coverage.{REQUIRED_TARGET}.json")),
        &exact_report,
    );
}

fn materialize_minimal_valid_claude_dir(claude_dir: &Path) {
    fs::create_dir_all(claude_dir).expect("mkdir claude dir");

    copy_from_repo(claude_dir, "claude_code", "SCHEMA.json");
    copy_from_repo(claude_dir, "claude_code", "RULES.json");
    copy_from_repo(claude_dir, "claude_code", "VERSION_METADATA_SCHEMA.json");

    write_text(
        &claude_dir.join("min_supported.txt"),
        &format!("{VERSION}\n"),
    );
    write_text(
        &claude_dir.join("latest_validated.txt"),
        &format!("{VERSION}\n"),
    );

    for target in CLAUDE_TARGETS {
        let supported = claude_dir
            .join("pointers")
            .join("latest_supported")
            .join(format!("{target}.txt"));
        let validated = claude_dir
            .join("pointers")
            .join("latest_validated")
            .join(format!("{target}.txt"));

        if target == CLAUDE_REQUIRED_TARGET {
            write_text(&supported, &format!("{VERSION}\n"));
            write_text(&validated, &format!("{VERSION}\n"));
        } else {
            write_text(&supported, "none\n");
            write_text(&validated, "none\n");
        }
    }

    let union = json!({
        "snapshot_schema_version": 2,
        "tool": "claude-code-cli",
        "mode": "union",
        "collected_at": TS,
        "expected_targets": CLAUDE_TARGETS,
        "complete": false,
        "missing_targets": [CLAUDE_TARGETS[1], CLAUDE_TARGETS[2]],
        "inputs": [{
            "target_triple": CLAUDE_REQUIRED_TARGET,
            "collected_at": TS,
            "binary": {
                "sha256": "00",
                "size_bytes": 0,
                "platform": { "os": "linux", "arch": "x86_64" },
                "target_triple": CLAUDE_REQUIRED_TARGET,
                "version_output": format!("{VERSION} (Claude Code)"),
                "semantic_version": VERSION,
            }
        }],
        "commands": [],
    });
    let union_path = claude_dir
        .join("snapshots")
        .join(VERSION)
        .join("union.json");
    write_json(&union_path, &union);

    write_json(
        &claude_dir
            .join("snapshots")
            .join(VERSION)
            .join(format!("{CLAUDE_REQUIRED_TARGET}.json")),
        &json!({
            "snapshot_schema_version": 1,
            "tool": "claude-code-cli",
            "collected_at": TS,
            "binary": {
                "sha256": "00",
                "size_bytes": 0,
                "platform": { "os": "linux", "arch": "x86_64" },
                "target_triple": CLAUDE_REQUIRED_TARGET,
                "version_output": format!("{VERSION} (Claude Code)"),
                "semantic_version": VERSION,
            },
            "commands": [],
        }),
    );

    write_json(&claude_dir.join("current.json"), &union);

    write_json(
        &claude_dir.join("versions").join(format!("{VERSION}.json")),
        &json!({
            "schema_version": 1,
            "semantic_version": VERSION,
            "status": "validated",
            "updated_at": TS,
            "artifacts": {
                "snapshots_dir": format!("snapshots/{VERSION}"),
                "reports_dir": format!("reports/{VERSION}"),
                "union_complete": false
            },
            "coverage": {
                "supported_targets": [CLAUDE_REQUIRED_TARGET],
                "supported_required_target": true
            },
            "validation": {
                "passed_targets": [CLAUDE_REQUIRED_TARGET],
                "failed_targets": [],
                "skipped_targets": []
            },
            "promotion": {
                "eligible_for_latest_validated": true
            },
        }),
    );

    let base_report = json!({
        "schema_version": 1,
        "generated_at": TS,
        "inputs": {
            "upstream": {
                "semantic_version": VERSION,
                "mode": "union",
                "targets": [CLAUDE_REQUIRED_TARGET],
            },
            "wrapper": {
                "schema_version": 1,
                "wrapper_version": "0.0.0-test"
            },
            "rules": {
                "rules_schema_version": 1
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
    });
    let mut any_report = base_report.clone();
    any_report["platform_filter"] = json!({ "mode": "any" });
    write_json(
        &claude_dir
            .join("reports")
            .join(VERSION)
            .join("coverage.any.json"),
        &any_report,
    );

    let mut exact_report = base_report;
    exact_report["platform_filter"] = json!({
        "mode": "exact_target",
        "target_triple": CLAUDE_REQUIRED_TARGET
    });
    write_json(
        &claude_dir
            .join("reports")
            .join(VERSION)
            .join(format!("coverage.{CLAUDE_REQUIRED_TARGET}.json")),
        &exact_report,
    );

    write_json(
        &claude_dir.join("wrapper_coverage.json"),
        &json!({
            "schema_version": 1,
            "generated_at": TS,
            "wrapper_version": "0.0.0-test",
            "coverage": []
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

fn run_xtask_validate(manifest_root: &Path) -> std::process::Output {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    let fixture_root = manifest_root
        .parent()
        .and_then(|p| p.parent())
        .expect("manifest_root is <fixture_root>/cli_manifests/<agent>");

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
    cmd.arg(root_flag_from_help(&help_text)).arg(manifest_root);
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
    let rows = derive_rows_for_test_roots(
        workspace_root,
        &[
            ("claude_code", "cli_manifests/claude_code"),
            ("codex", "cli_manifests/codex"),
            ("opencode", "cli_manifests/opencode"),
        ],
    )
    .expect("derive complete support matrix rows");

    write_support_matrix_artifact(
        workspace_root,
        serde_json::to_value(rows).expect("serialize complete support matrix rows"),
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

fn assert_validation_failure(
    manifest_root: &Path,
    code: &str,
    expected_path: &str,
) -> std::process::Output {
    let output = run_xtask_validate(manifest_root);
    assert!(
        !output.status.success(),
        "expected validation failure:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_violation_surface(&output, code, expected_path);
    output
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

#[path = "c6_spec_report_iu_validator/report_iu_rules.rs"]
mod report_iu_rules;
#[path = "c6_spec_report_iu_validator/support_matrix_publication.rs"]
mod support_matrix_publication;
