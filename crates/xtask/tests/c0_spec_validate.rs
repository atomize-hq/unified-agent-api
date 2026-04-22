use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

const SEEDED_REGISTRY: &str = include_str!("../data/agent_registry.toml");
const VERSION: &str = "0.61.0";
const REPORTED_VERSION: &str = "0.60.0";
const TS: &str = "1970-01-01T00:00:00Z";

const REQUIRED_TARGET: &str = "x86_64-unknown-linux-musl";
const TARGETS: [&str; 3] = [
    "x86_64-unknown-linux-musl",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
];

fn workspace_root() -> PathBuf {
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
    write_text(
        &workspace_root.join("crates/xtask/data/agent_registry.toml"),
        SEEDED_REGISTRY,
    );
}

fn copy_from_repo(codex_dir: &Path, filename: &str) {
    let src = workspace_root()
        .join("cli_manifests")
        .join("codex")
        .join(filename);
    let dst = codex_dir.join(filename);
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
    let src = workspace_root().join("cli_manifests").join("opencode");
    copy_dir_recursive(&src, opencode_dir);
}

fn materialize_committed_gemini_dir(gemini_dir: &Path) {
    let src = workspace_root().join("cli_manifests").join("gemini_cli");
    copy_dir_recursive(&src, gemini_dir);
}

fn support_rows_for_agent_from_repo(agent: &str) -> Vec<Value> {
    let artifact_path = workspace_root()
        .join("cli_manifests")
        .join("support_matrix")
        .join("current.json");
    let artifact: Value = serde_json::from_str(
        &fs::read_to_string(&artifact_path)
            .unwrap_or_else(|e| panic!("read {:?}: {}", artifact_path, e)),
    )
    .unwrap_or_else(|e| panic!("parse {:?}: {}", artifact_path, e));

    artifact["rows"]
        .as_array()
        .expect("support_matrix/current.json rows array")
        .iter()
        .filter(|row| row["agent"] == agent)
        .cloned()
        .collect()
}

fn target_platform(target: &str) -> (&'static str, &'static str) {
    match target {
        "x86_64-unknown-linux-musl" => ("linux", "x86_64"),
        "aarch64-apple-darwin" => ("macos", "aarch64"),
        "x86_64-pc-windows-msvc" => ("windows", "x86_64"),
        _ => ("unknown", "unknown"),
    }
}

fn union_input_targets(union_complete: bool) -> Vec<&'static str> {
    if union_complete {
        TARGETS.to_vec()
    } else {
        vec![REQUIRED_TARGET]
    }
}

fn write_codex_version_artifacts(
    codex_dir: &Path,
    version: &str,
    status: &str,
    union_complete: bool,
) {
    let input_targets = union_input_targets(union_complete);
    let inputs = input_targets
        .iter()
        .map(|target| {
            let (os, arch) = target_platform(target);
            json!({
                "target_triple": target,
                "collected_at": TS,
                "binary": {
                    "sha256": "00",
                    "size_bytes": 0,
                    "platform": { "os": os, "arch": arch },
                    "target_triple": target,
                    "version_output": format!("codex-cli {version}"),
                    "semantic_version": version,
                    "channel": "stable",
                }
            })
        })
        .collect::<Vec<_>>();

    let union = if union_complete {
        json!({
            "snapshot_schema_version": 2,
            "tool": "codex-cli",
            "mode": "union",
            "collected_at": TS,
            "expected_targets": TARGETS,
            "complete": true,
            "inputs": inputs,
            "commands": [],
        })
    } else {
        json!({
            "snapshot_schema_version": 2,
            "tool": "codex-cli",
            "mode": "union",
            "collected_at": TS,
            "expected_targets": TARGETS,
            "complete": false,
            "missing_targets": [TARGETS[1], TARGETS[2]],
            "inputs": inputs,
            "commands": [],
        })
    };

    let union_path = codex_dir.join("snapshots").join(version).join("union.json");
    write_json(&union_path, &union);

    for target in &input_targets {
        let (os, arch) = target_platform(target);
        let per_target = json!({
            "snapshot_schema_version": 1,
            "tool": "codex-cli",
            "collected_at": TS,
            "binary": {
                "sha256": "00",
                "size_bytes": 0,
                "platform": { "os": os, "arch": arch },
                "target_triple": target,
                "version_output": format!("codex-cli {version}"),
                "semantic_version": version,
                "channel": "stable",
            },
            "commands": [],
        });

        write_json(
            &codex_dir
                .join("snapshots")
                .join(version)
                .join(format!("{target}.json")),
            &per_target,
        );
    }

    let version_metadata = json!({
        "schema_version": 1,
        "semantic_version": version,
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
    });
    write_json(
        &codex_dir.join("versions").join(format!("{version}.json")),
        &version_metadata,
    );
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

    write_codex_version_artifacts(codex_dir, VERSION, "validated", false);

    let union_text =
        fs::read_to_string(codex_dir.join("snapshots").join(VERSION).join("union.json"))
            .expect("read baseline union.json");
    write_text(&codex_dir.join("current.json"), &union_text);

    let wrapper_coverage = json!({
        "schema_version": 1,
        "generated_at": TS,
        "wrapper_version": "0.0.0-test",
        "coverage": []
    });
    write_json(&codex_dir.join("wrapper_coverage.json"), &wrapper_coverage);

    write_minimal_report_files(codex_dir, VERSION, &[REQUIRED_TARGET], false);
}

fn materialize_reported_codex_version(codex_dir: &Path, union_complete: bool) {
    write_codex_version_artifacts(codex_dir, REPORTED_VERSION, "reported", union_complete);
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

struct SupportRowSpec<'a> {
    agent: &'a str,
    version: &'a str,
    target: &'a str,
    manifest_support: &'a str,
    backend_support: &'a str,
    uaa_support: &'a str,
    pointer_promotion: &'a str,
    evidence_notes: &'a [&'a str],
}

fn support_row(spec: SupportRowSpec<'_>) -> Value {
    json!({
        "agent": spec.agent,
        "version": spec.version,
        "target": spec.target,
        "manifest_support": spec.manifest_support,
        "backend_support": spec.backend_support,
        "uaa_support": spec.uaa_support,
        "pointer_promotion": spec.pointer_promotion,
        "evidence_notes": spec.evidence_notes,
    })
}

fn codex_report_exists(codex_dir: &Path, version: &str, target: &str) -> bool {
    codex_dir
        .join("reports")
        .join(version)
        .join(format!("coverage.{target}.json"))
        .exists()
}

fn write_support_matrix_artifact(workspace_root: &Path) {
    let codex_dir = workspace_root.join("cli_manifests").join("codex");
    let mut rows = vec![
        support_row(SupportRowSpec {
            agent: "claude_code",
            version: VERSION,
            target: REQUIRED_TARGET,
            manifest_support: "supported",
            backend_support: "supported",
            uaa_support: "supported",
            pointer_promotion: "latest_supported_and_validated",
            evidence_notes: &[],
        }),
        support_row(SupportRowSpec {
            agent: "codex",
            version: VERSION,
            target: TARGETS[1],
            manifest_support: "unsupported",
            backend_support: "unsupported",
            uaa_support: "unsupported",
            pointer_promotion: "none",
            evidence_notes: &["current root snapshot omits this target"],
        }),
        support_row(SupportRowSpec {
            agent: "codex",
            version: VERSION,
            target: TARGETS[2],
            manifest_support: "unsupported",
            backend_support: "unsupported",
            uaa_support: "unsupported",
            pointer_promotion: "none",
            evidence_notes: &["current root snapshot omits this target"],
        }),
        support_row(SupportRowSpec {
            agent: "codex",
            version: VERSION,
            target: REQUIRED_TARGET,
            manifest_support: "supported",
            backend_support: "supported",
            uaa_support: "supported",
            pointer_promotion: "latest_supported_and_validated",
            evidence_notes: &[],
        }),
    ];

    if codex_dir
        .join("versions")
        .join(format!("{REPORTED_VERSION}.json"))
        .exists()
    {
        rows.insert(
            2,
            support_row(SupportRowSpec {
                agent: "codex",
                version: REPORTED_VERSION,
                target: TARGETS[1],
                manifest_support: "unsupported",
                backend_support: if codex_report_exists(&codex_dir, REPORTED_VERSION, TARGETS[1]) {
                    "supported"
                } else {
                    "unsupported"
                },
                uaa_support: "unsupported",
                pointer_promotion: "none",
                evidence_notes: &[],
            }),
        );
        rows.insert(
            4,
            support_row(SupportRowSpec {
                agent: "codex",
                version: REPORTED_VERSION,
                target: TARGETS[2],
                manifest_support: "unsupported",
                backend_support: if codex_report_exists(&codex_dir, REPORTED_VERSION, TARGETS[2]) {
                    "supported"
                } else {
                    "unsupported"
                },
                uaa_support: "unsupported",
                pointer_promotion: "none",
                evidence_notes: &[],
            }),
        );
        let reported_linux_backend_supported =
            codex_report_exists(&codex_dir, REPORTED_VERSION, REQUIRED_TARGET);
        rows.insert(
            6,
            support_row(SupportRowSpec {
                agent: "codex",
                version: REPORTED_VERSION,
                target: REQUIRED_TARGET,
                manifest_support: "supported",
                backend_support: if reported_linux_backend_supported {
                    "supported"
                } else {
                    "unsupported"
                },
                uaa_support: if reported_linux_backend_supported {
                    "supported"
                } else {
                    "unsupported"
                },
                pointer_promotion: "none",
                evidence_notes: &[],
            }),
        );
    }

    rows.extend(support_rows_for_agent_from_repo("gemini_cli"));
    rows.extend(support_rows_for_agent_from_repo("opencode"));

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

fn materialize_minimal_valid_workspace(workspace_root: &Path) -> PathBuf {
    let codex_dir = workspace_root.join("cli_manifests").join("codex");
    let claude_dir = workspace_root.join("cli_manifests").join("claude_code");
    let gemini_dir = workspace_root.join("cli_manifests").join("gemini_cli");
    let opencode_dir = workspace_root.join("cli_manifests").join("opencode");

    write_workspace_manifest(workspace_root);
    materialize_minimal_valid_codex_dir(&codex_dir);
    materialize_minimal_valid_claude_dir(&claude_dir);
    materialize_committed_gemini_dir(&gemini_dir);
    materialize_committed_opencode_dir(&opencode_dir);
    write_support_matrix_artifact(workspace_root);

    codex_dir
}

fn write_minimal_report_files(
    codex_dir: &Path,
    version: &str,
    input_targets: &[&str],
    include_all: bool,
) {
    let report = json!({
        "schema_version": 1,
        "generated_at": TS,
        "inputs": {
            "upstream": {
                "semantic_version": version,
                "mode": "union",
                "targets": input_targets
            },
            "wrapper": {
                "schema_version": 1,
                "wrapper_version": "0.0.0-test"
            },
            "rules": {
                "rules_schema_version": 1
            }
        },
        "platform_filter": {
            "mode": "any"
        },
        "deltas": {
            "missing_commands": [],
            "missing_flags": [],
            "missing_args": [],
            "intentionally_unsupported": [],
            "wrapper_only_commands": [],
            "wrapper_only_flags": [],
            "wrapper_only_args": []
        }
    });

    let reports_dir = codex_dir.join("reports").join(version);
    write_json(&reports_dir.join("coverage.any.json"), &report);
    for target in input_targets {
        write_json(
            &reports_dir.join(format!("coverage.{target}.json")),
            &report,
        );
    }
    if include_all {
        write_json(&reports_dir.join("coverage.all.json"), &report);
    }
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

#[path = "c0_spec_validate/happy_path.rs"]
mod happy_path;
#[path = "c0_spec_validate/reported_versions.rs"]
mod reported_versions;
#[path = "c0_spec_validate/validation_errors.rs"]
mod validation_errors;
