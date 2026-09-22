//! `uaa-0051`: the effective-flags model must be honoured on the wrapper side of the comparison,
//! not only on the union side.
//!
//! `manifest_union`'s `normalize_union_commands` deletes a subcommand's copy of a root flag, so a
//! wrapper entry claiming that flag at the subcommand finds no counterpart there. Before this
//! change the row was emitted as `wrapper_only` — a surface the wrapper claims and upstream does
//! not have — which is false: upstream has it at root, and the model's own definition says the
//! subcommand inherits it.
//!
//! The union fixtures below are written in post-normalization shape (the root flag appears once,
//! at root) because that is what the report actually reads.

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

/// `(path, key)` for every `wrapper_only_flags` row.
fn wrapper_only_flags(deltas: &Value) -> Vec<(Vec<String>, String)> {
    deltas["wrapper_only_flags"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .map(|row| {
            (
                serde_json::from_value(row["path"].clone()).expect("path tokens"),
                row["key"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect()
}

/// Runs `manifest-report` against codex's committed descriptor, optionally with the effective-flags
/// model turned off, over a union whose root carries `--global` and whose `sub` command carries
/// nothing — the shape the union normalization leaves behind.
fn report_deltas(model_enabled: bool, wrapper_flags: Value) -> Value {
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

    if !model_enabled {
        let rules_path = codex_dir.join("RULES.json");
        let mut rules: Value =
            serde_json::from_str(&fs::read_to_string(&rules_path).expect("read RULES.json"))
                .expect("parse RULES.json");
        rules["globals"]["effective_flags_model"]["enabled"] = json!(false);
        write_json(&rules_path, &rules);
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
                { "path": [], "available_on": TARGETS, "flags": [flag("--global")], "args": [] },
                { "path": ["sub"], "available_on": TARGETS, "flags": [], "args": [] },
            ],
        }),
    );
    write_json(
        &codex_dir.join("wrapper_coverage.json"),
        &json!({
            "schema_version": 1, "generated_at": TS, "wrapper_version": "0.0.0-test",
            "coverage": [
                { "path": [], "level": "explicit", "flags": [{ "key": "--global", "level": "explicit" }] },
                { "path": ["sub"], "level": "explicit", "flags": wrapper_flags },
            ],
        }),
    );

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

#[test]
fn a_wrapper_claim_at_a_subcommand_is_credited_against_the_root_flag() {
    let deltas = report_deltas(true, json!([{ "key": "--global", "level": "explicit" }]));

    assert!(
        wrapper_only_flags(&deltas).is_empty(),
        "a global flag claimed at a subcommand is not wrapper-only; got {:?}",
        wrapper_only_flags(&deltas)
    );
}

#[test]
fn the_credit_is_withheld_when_the_model_is_disabled() {
    // The gate is the descriptor's own declaration, not a universal rule. An agent whose CLI does
    // not repeat its globals never had the subcommand copy deleted, so a claim at a subcommand
    // that upstream does not offer there is still a real wrapper-only row.
    let deltas = report_deltas(false, json!([{ "key": "--global", "level": "explicit" }]));

    assert_eq!(
        wrapper_only_flags(&deltas),
        vec![(vec!["sub".to_string()], "--global".to_string())],
        "with the model off the row must stand"
    );
}

#[test]
fn a_claim_that_matches_no_root_flag_is_still_wrapper_only() {
    // The credit must not become a blanket amnesty for anything the union does not show at that
    // path. Only a root flag of the same canonical key licenses it.
    let deltas = report_deltas(true, json!([{ "key": "--invented", "level": "explicit" }]));

    assert_eq!(
        wrapper_only_flags(&deltas),
        vec![(vec!["sub".to_string()], "--invented".to_string())],
        "a key with no root counterpart stays wrapper-only"
    );
}
