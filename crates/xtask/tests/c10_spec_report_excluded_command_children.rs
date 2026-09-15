use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};

const VERSION: &str = "0.61.0";
const TS: &str = "1970-01-01T00:00:00Z";
const TARGETS: [&str; 4] = [
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-musl",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

fn write_json(path: &Path, value: &Value) {
    fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
    fs::write(path, format!("{value:#}\n")).expect("write json");
}

fn flag(key: &str) -> Value {
    json!({ "key": key, "long": key, "takes_value": false, "available_on": TARGETS })
}

fn arg(name: &str) -> Value {
    json!({ "name": name, "available_on": TARGETS })
}

/// Rows of `deltas[list]` that carry `field`, as (path, value) pairs.
fn units(deltas: &Value, list: &str, field: &str) -> Vec<(Vec<String>, String)> {
    deltas[list]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .filter(|row| field == "path" || row.get(field).is_some())
        .map(|row| {
            let path = serde_json::from_value(row["path"].clone()).expect("path tokens");
            (path, row[field].as_str().unwrap_or_default().to_string())
        })
        .collect()
}

fn unit(path: &[&str], id: &str) -> (Vec<String>, String) {
    (
        path.iter().map(ToString::to_string).collect(),
        id.to_string(),
    )
}

/// Runs `manifest-report` against codex's committed RULES.json, which excludes the `app` command
/// together with its `PATH` arg and `--download-url` flag, plus the root `PROMPT` arg and
/// `--no-alt-screen` flag. `app` also carries a flag and an arg the rules do not name.
fn report_deltas(wrapper_coverage: Value) -> Value {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let codex_dir = temp.path().join("cli_manifests/codex");
    for file in ["SCHEMA.json", "RULES.json", "VERSION_METADATA_SCHEMA.json"] {
        let destination = codex_dir.join(file);
        fs::create_dir_all(destination.parent().expect("parent")).expect("create parent");
        fs::copy(
            repo_root().join("cli_manifests/codex").join(file),
            destination,
        )
        .expect("copy committed codex file");
    }
    let inputs = TARGETS
        .iter()
        .map(|target| {
            json!({
                "target_triple": target,
                "collected_at": TS,
                "binary": {
                    "sha256": "00", "size_bytes": 0, "platform": { "os": "linux", "arch": "x86_64" },
                    "target_triple": target, "version_output": format!("codex-cli {VERSION}"),
                    "semantic_version": VERSION, "channel": "stable",
                },
            })
        })
        .collect::<Vec<_>>();
    write_json(
        &codex_dir.join("snapshots").join(VERSION).join("union.json"),
        &json!({
            "snapshot_schema_version": 2, "tool": "codex-cli", "mode": "union", "collected_at": TS,
            "expected_targets": TARGETS, "complete": true, "inputs": inputs,
            "commands": [
                { "path": [], "available_on": TARGETS,
                  "flags": [flag("--no-alt-screen")], "args": [arg("PROMPT")] },
                { "path": ["app"], "available_on": TARGETS,
                  "flags": [flag("--download-url"), flag("--new-app-flag")],
                  "args": [arg("PATH"), arg("NEW_APP_ARG")] },
            ],
        }),
    );
    write_json(&codex_dir.join("wrapper_coverage.json"), &wrapper_coverage);

    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["manifest-report", "--root"])
        .arg(&codex_dir)
        .args(["--version", VERSION])
        .env("SOURCE_DATE_EPOCH", "0")
        .current_dir(temp.path())
        .output()
        .expect("spawn xtask manifest-report");
    assert!(
        output.status.success(),
        "manifest-report failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(
        &fs::read(
            codex_dir
                .join("reports")
                .join(VERSION)
                .join("coverage.any.json"),
        )
        .expect("read report"),
    )
    .expect("parse report");
    report["deltas"].clone()
}

fn assert_own_exclusions_apply(deltas: &Value) {
    assert_eq!(
        units(deltas, "excluded_commands", "path"),
        vec![unit(&["app"], "")]
    );
    assert_eq!(
        units(deltas, "excluded_flags", "key"),
        vec![
            unit(&[], "--no-alt-screen"),
            unit(&["app"], "--download-url")
        ]
    );
    assert_eq!(
        units(deltas, "excluded_args", "name"),
        vec![unit(&[], "PROMPT"), unit(&["app"], "PATH")]
    );
    assert!(units(deltas, "missing_commands", "path").is_empty());
}

#[test]
fn an_excluded_command_still_reports_the_flags_and_args_its_rules_do_not_name() {
    let deltas = report_deltas(json!({
        "schema_version": 1, "generated_at": TS, "wrapper_version": "0.0.0-test",
        "coverage": [{ "path": [], "level": "explicit" }],
    }));

    assert_own_exclusions_apply(&deltas);
    assert_eq!(
        units(&deltas, "missing_flags", "key"),
        vec![unit(&["app"], "--new-app-flag")]
    );
    assert_eq!(
        units(&deltas, "missing_args", "name"),
        vec![unit(&["app"], "NEW_APP_ARG")]
    );
}

#[test]
fn an_excluded_commands_unnamed_children_still_inherit_an_intentionally_unsupported_root() {
    let deltas = report_deltas(json!({
        "schema_version": 1, "generated_at": TS, "wrapper_version": "0.0.0-test",
        "coverage": [{ "path": [], "level": "intentionally_unsupported",
                       "note": "waived root subtree for the c10 exclusion test" }],
    }));

    assert_own_exclusions_apply(&deltas);
    assert!(units(&deltas, "missing_flags", "key").is_empty());
    assert!(units(&deltas, "missing_args", "name").is_empty());
    assert_eq!(
        units(&deltas, "intentionally_unsupported", "key"),
        vec![unit(&["app"], "--new-app-flag")]
    );
    assert_eq!(
        units(&deltas, "intentionally_unsupported", "name"),
        vec![unit(&["app"], "NEW_APP_ARG")]
    );
}
