use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

const SEEDED_REGISTRY: &str = include_str!("../data/agent_registry.toml");

const OPENCODE_REQUIRED_TARGET: &str = "linux-x64";
const OPENCODE_TARGETS: [&str; 3] = ["linux-x64", "darwin-arm64", "win32-x64"];

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

fn write_seeded_agent_registry(workspace_root: &Path) {
    write_text(
        &workspace_root.join("crates/xtask/data/agent_registry.toml"),
        SEEDED_REGISTRY,
    );
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

fn stale_generated_block(original: &str) -> String {
    let start = original
        .find("<!-- support-matrix-published:start -->")
        .expect("generated start marker");
    let end = original
        .find("<!-- support-matrix-published:end -->")
        .expect("generated end marker");

    let mut stale = String::new();
    stale.push_str(&original[..start]);
    stale.push_str("<!-- support-matrix-published:start -->\n");
    stale.push_str("### `codex`\n\n");
    stale.push_str("| agent | version | target | manifest_support | backend_support | uaa_support | pointer_promotion | evidence_notes |\n");
    stale.push_str("|---|---|---|---|---|---|---|---|\n");
    stale.push_str("| `codex` | `1.0.0` | `linux-x64` | `supported` | `partial` | `partial` | `latest_validated` | stale block |\n");
    stale.push_str("<!-- support-matrix-published:end -->");
    stale.push_str(&original[end + "<!-- support-matrix-published:end -->".len()..]);
    stale
}

fn reverse_generated_rows(path: &Path) {
    let text = fs::read_to_string(path).expect("read generated json");
    let mut artifact: Value = serde_json::from_str(&text).expect("parse generated json");
    let rows = artifact
        .get_mut("rows")
        .and_then(Value::as_array_mut)
        .expect("generated json rows array");
    rows.reverse();
    write_json(path, &artifact);
}

fn materialize_minimal_valid_opencode_root(fixture_root: &Path) {
    materialize_root(
        &fixture_root.join("cli_manifests/opencode"),
        &OPENCODE_TARGETS,
        "1.4.11",
        &[OPENCODE_REQUIRED_TARGET],
        &[("1.4.11", &[OPENCODE_REQUIRED_TARGET])],
        &[(OPENCODE_REQUIRED_TARGET, "1.4.11")],
        &[(OPENCODE_REQUIRED_TARGET, "1.4.11")],
        &[(
            "1.4.11",
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
}

#[test]
fn support_matrix_check_rejects_stale_generated_markdown_block() {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    let fixture_root = make_temp_dir("support-matrix-staleness");

    write_text(
        &fixture_root.join("Cargo.toml"),
        "[workspace]\nmembers = []\n",
    );
    write_seeded_agent_registry(&fixture_root);
    write_text(
        &fixture_root.join("docs/specs/unified-agent-api/support-matrix.md"),
        "# Support Matrix Spec — Unified Agent API\n\n## Purpose\nManual contract text.\n\n## Change control\nManual footer.\n",
    );

    materialize_root(
        &fixture_root.join("cli_manifests/codex"),
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
        &fixture_root.join("cli_manifests/claude_code"),
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
    materialize_minimal_valid_opencode_root(&fixture_root);

    let generate = Command::new(&xtask_bin)
        .arg("support-matrix")
        .current_dir(&fixture_root)
        .output()
        .expect("spawn xtask support-matrix");
    assert!(
        generate.status.success(),
        "xtask support-matrix failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&generate.stdout),
        String::from_utf8_lossy(&generate.stderr)
    );

    let markdown_path = fixture_root.join("docs/specs/unified-agent-api/support-matrix.md");
    let current_markdown = fs::read_to_string(&markdown_path).expect("read generated markdown");
    let stale_markdown = stale_generated_block(&current_markdown);
    fs::write(&markdown_path, stale_markdown).expect("write stale markdown");

    let check = Command::new(&xtask_bin)
        .arg("support-matrix")
        .arg("--check")
        .current_dir(&fixture_root)
        .output()
        .expect("spawn xtask support-matrix --check");
    assert!(
        !check.status.success(),
        "xtask support-matrix --check should fail for stale markdown"
    );

    let stderr = String::from_utf8_lossy(&check.stderr);
    assert!(stderr.contains("generated block is stale"));
    assert!(stderr.contains("support-matrix.md"));
    assert!(stderr.contains("regenerate with `cargo run -p xtask -- support-matrix`"));
}

#[test]
fn support_matrix_check_rejects_stale_generated_json_row_order() {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    let fixture_root = make_temp_dir("support-matrix-json-order-staleness");

    write_text(
        &fixture_root.join("Cargo.toml"),
        "[workspace]\nmembers = []\n",
    );
    write_seeded_agent_registry(&fixture_root);
    write_text(
        &fixture_root.join("docs/specs/unified-agent-api/support-matrix.md"),
        "# Support Matrix Spec — Unified Agent API\n\n## Purpose\nManual contract text.\n\n## Change control\nManual footer.\n",
    );

    materialize_root(
        &fixture_root.join("cli_manifests/codex"),
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
        &fixture_root.join("cli_manifests/claude_code"),
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
    materialize_minimal_valid_opencode_root(&fixture_root);

    let generate = Command::new(&xtask_bin)
        .arg("support-matrix")
        .current_dir(&fixture_root)
        .output()
        .expect("spawn xtask support-matrix");
    assert!(
        generate.status.success(),
        "xtask support-matrix failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&generate.stdout),
        String::from_utf8_lossy(&generate.stderr)
    );

    reverse_generated_rows(&fixture_root.join("cli_manifests/support_matrix/current.json"));

    let check = Command::new(&xtask_bin)
        .arg("support-matrix")
        .arg("--check")
        .current_dir(&fixture_root)
        .output()
        .expect("spawn xtask support-matrix --check");
    assert!(
        !check.status.success(),
        "xtask support-matrix --check should fail for stale json row order"
    );

    let stderr = String::from_utf8_lossy(&check.stderr);
    assert!(stderr.contains("current.json is stale"));
    assert!(stderr.contains("cli_manifests/support_matrix/current.json"));
    assert!(stderr.contains("regenerate with `cargo run -p xtask -- support-matrix`"));
}
