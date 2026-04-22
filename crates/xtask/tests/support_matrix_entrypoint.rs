use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

const SEEDED_REGISTRY: &str = include_str!("../data/agent_registry.toml");

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

#[test]
fn support_matrix_entrypoint_publishes_json_and_hybrid_markdown() {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    let fixture_root = make_temp_dir("support-matrix-entrypoint");

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

    materialize_root(
        &fixture_root.join("cli_manifests/opencode"),
        &["linux-x64"],
        "3.0.0",
        &["linux-x64"],
        &[("3.0.0", &["linux-x64"])],
        &[],
        &[],
        &[],
    );

    materialize_root(
        &fixture_root.join("cli_manifests/gemini_cli"),
        &["darwin-arm64"],
        "0.38.2",
        &["darwin-arm64"],
        &[("0.38.2", &[])],
        &[],
        &[],
        &[],
    );

    let help = Command::new(&xtask_bin)
        .arg("--help")
        .current_dir(&fixture_root)
        .output()
        .expect("spawn xtask --help");
    assert!(
        help.status.success(),
        "xtask --help failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&help.stdout),
        String::from_utf8_lossy(&help.stderr)
    );
    let help_text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&help.stdout),
        String::from_utf8_lossy(&help.stderr)
    );
    assert!(help_text.contains("support-matrix"));
    assert!(help_text.contains("capability-matrix"));

    let sub_help = Command::new(&xtask_bin)
        .arg("support-matrix")
        .arg("--help")
        .current_dir(&fixture_root)
        .output()
        .expect("spawn xtask support-matrix --help");
    assert!(
        sub_help.status.success(),
        "xtask support-matrix --help failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&sub_help.stdout),
        String::from_utf8_lossy(&sub_help.stderr)
    );
    let sub_help_text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&sub_help.stdout),
        String::from_utf8_lossy(&sub_help.stderr)
    );
    assert!(
        sub_help_text.contains("Generate support publication JSON and Markdown outputs"),
        "support-matrix help text must reflect the implemented publication contract:\n{sub_help_text}"
    );
    assert!(sub_help_text.contains("--check"));

    let first_run = Command::new(&xtask_bin)
        .arg("support-matrix")
        .current_dir(&fixture_root)
        .output()
        .expect("spawn xtask support-matrix");
    assert!(
        first_run.status.success(),
        "xtask support-matrix failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&first_run.stdout),
        String::from_utf8_lossy(&first_run.stderr)
    );

    let json_path = fixture_root.join("cli_manifests/support_matrix/current.json");
    let markdown_path = fixture_root.join("docs/specs/unified-agent-api/support-matrix.md");
    let json_text = fs::read_to_string(&json_path).expect("read generated current.json");
    let markdown_text = fs::read_to_string(&markdown_path).expect("read generated markdown");

    let artifact: Value =
        serde_json::from_str(&json_text).expect("parse support-matrix current.json");
    assert_eq!(
        artifact
            .get("schema_version")
            .and_then(|value| value.as_u64()),
        Some(1)
    );
    assert_eq!(
        artifact
            .get("rows")
            .and_then(|value| value.as_array())
            .map(|rows| rows.len()),
        Some(4)
    );

    assert!(markdown_text.contains("## Purpose\nManual contract text."));
    assert!(markdown_text.contains("## Change control\nManual footer."));
    assert!(markdown_text.contains("## Published support matrix"));
    assert!(markdown_text.contains("<!-- support-matrix-published:start -->"));
    assert!(markdown_text.contains("| `codex` | `1.0.0` | `linux-x64` |"));
    assert!(markdown_text.contains("| `claude_code` | `2.0.0` | `linux-x64` |"));
    assert!(markdown_text.contains("| `gemini_cli` | `0.38.2` | `darwin-arm64` |"));
    assert!(markdown_text.contains("| `opencode` | `3.0.0` | `linux-x64` |"));

    let json_before_check = fs::read_to_string(&json_path).expect("read json before check");
    let markdown_before_check =
        fs::read_to_string(&markdown_path).expect("read markdown before check");

    let check_run = Command::new(&xtask_bin)
        .arg("support-matrix")
        .arg("--check")
        .current_dir(&fixture_root)
        .output()
        .expect("spawn xtask support-matrix --check");
    assert!(
        check_run.status.success(),
        "xtask support-matrix --check failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&check_run.stdout),
        String::from_utf8_lossy(&check_run.stderr)
    );

    let json_after_check = fs::read_to_string(&json_path).expect("read json after check");
    let markdown_after_check =
        fs::read_to_string(&markdown_path).expect("read markdown after check");
    assert_eq!(json_before_check, json_after_check);
    assert_eq!(markdown_before_check, markdown_after_check);

    let second_run = Command::new(&xtask_bin)
        .arg("support-matrix")
        .current_dir(&fixture_root)
        .output()
        .expect("re-run xtask support-matrix");
    assert!(
        second_run.status.success(),
        "second xtask support-matrix run failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&second_run.stdout),
        String::from_utf8_lossy(&second_run.stderr)
    );

    let markdown_text_second =
        fs::read_to_string(&markdown_path).expect("read markdown after rerun");
    assert_eq!(
        markdown_text, markdown_text_second,
        "rerun should be idempotent"
    );
}
