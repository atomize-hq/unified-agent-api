//! The synthetic subject's isolated governing-data templates (SA-P1), checked against the reviewed
//! synthetic record and driven through the real acquisition and manifest commands.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;
use xtask::{agent_registry::AgentRegistry, support_depth::resolve_policy};

const GOVERNING: &str = "tests/fixtures/depth-synthetic/governing";
const CANONICAL: &str = include_str!("../data/agent_registry.toml");
const TARGET: &str = "x86_64-unknown-linux-gnu";
const DEBT: &str = "### `support-debt-authorization-contract-target-version-v1`\n";

fn template(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(GOVERNING)
        .join(relative);
    fs::read_to_string(path)
        .unwrap()
        .replace("{{ISOLATED_OWNER}}", "example-owner")
        .replace("{{ISOLATED_REPO}}", "depth-isolated")
}

fn registry_with_entry() -> String {
    format!("{CANONICAL}\n{}", template("registry-entry.toml"))
}

/// The production registry still refuses depth tables (roadmap B.8), so the commands that load it
/// see the entry without its `support_depth` table.
fn registry_without_depth() -> String {
    let mut doc: toml_edit::DocumentMut = registry_with_entry().parse().unwrap();
    let entry = doc["agents"]
        .as_array_of_tables_mut()
        .unwrap()
        .iter_mut()
        .find(|a| a["agent_id"].as_str() == Some("depth_fixture"))
        .unwrap();
    assert!(entry.remove("support_depth").is_some());
    doc.to_string()
}

fn reviewed_record() -> Value {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/depth-synthetic/record.json");
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn templates_declare_the_reviewed_synthetic_selection() {
    let registry = registry_with_entry();
    let parsed = AgentRegistry::parse_unenforced(&registry).unwrap();
    let entry = parsed.find("depth_fixture").unwrap();
    let declared = serde_json::to_value(entry.support_depth.as_ref().unwrap()).unwrap();
    assert_eq!(
        declared["enabled_paths"],
        serde_json::json!(["maintenance"])
    );
    assert_eq!(
        declared["enrollments"],
        serde_json::json!([reviewed_record()["policy"]["selection"]["enrollment"]])
    );

    // RULES adopts the canonical depth-record path and the gnu target: resolution gets past every
    // RULES check and stops only at the predicate selector, which needs acquisition intake (B.3).
    let rules = template("manifest-root/RULES.json");
    let error = resolve_policy(&registry, "depth_fixture", "0.2.0", &rules, None, DEBT)
        .err()
        .unwrap();
    assert!(
        error.ends_with("selector `run-json` requires predicate/alias acquisition intake"),
        "{error}"
    );
    // The baseline version is not enrolled: it resolves to remainder, never to a policy.
    assert!(
        resolve_policy(&registry, "depth_fixture", "0.1.0", &rules, None, DEBT)
            .unwrap()
            .is_none()
    );
    assert!(AgentRegistry::parse(&registry).is_err());
    let production = AgentRegistry::parse(&registry_without_depth()).unwrap();
    let entry = production.find("depth_fixture").unwrap();
    assert!(entry.backend_module.is_none());
    // Depth records live only in published roots, so the isolated support matrix publishes the
    // subject; a non-runtime entry never publishes capabilities.
    assert!(entry.publication.support_matrix_enabled);
    assert!(!entry.publication.capability_matrix_enabled);
    // No release or release-doc drift claim: only the crates-io track is published or checked.
    let packages = xtask::release_doc::registry_release_packages(&production, "crates-io");
    assert!(!packages.contains(&entry.package_name), "{packages:?}");
}

#[test]
fn the_manifest_root_adopts_annex_b_and_validates_the_acquired_binary() {
    let spec = template("manifest-root/VALIDATOR_SPEC.md");
    assert!(spec.contains(
        "This root adopts the depth record that Annex B of `docs/specs/support-depth-contract.md` defines."
    ));
    assert!(spec.contains("cli_manifests/depth_fixture/"));
    let rules: Value = serde_json::from_str(&template("manifest-root/RULES.json")).unwrap();
    let validation = &rules["acquisition"]["validation"];
    assert_eq!(validation["env"]["DEPTH_FIXTURE_BINARY"], "{binary_path}");
    assert_eq!(
        validation["commands"],
        serde_json::json!([
            "cargo test -p unified-agent-api-depth-fixture --test acquired_binary -- --ignored"
        ])
    );
}

#[test]
fn the_canonical_registry_never_lists_the_synthetic_agent() {
    let canonical = AgentRegistry::parse(CANONICAL).unwrap();
    assert!(canonical.find("depth_fixture").is_none());
    assert!(!Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cli_manifests/depth_fixture")
        .exists());
}

fn xtask(cwd: &Path, args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(cwd)
        .env("SOURCE_DATE_EPOCH", "1793664000")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "xtask {args:?} failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Build the real fixture CLI (the baseline `0.1.0` unless the upstream version is overridden).
fn fixture_binary() -> PathBuf {
    let output = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args([
            "build",
            "-q",
            "-p",
            "unified-agent-api-depth-fixture",
            "--bin",
            "depth_fixture",
            "--message-format=json",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find_map(|m| m["executable"].as_str().map(PathBuf::from))
        .expect("cargo reported the fixture executable")
}

#[test]
fn templates_drive_the_real_acquisition_and_manifest_commands() {
    let binary = fixture_binary();
    let version_output = Command::new(&binary).arg("--version").output().unwrap();
    let version = String::from_utf8(version_output.stdout).unwrap();
    let version = version
        .trim()
        .strip_prefix("depth_fixture ")
        .unwrap()
        .to_owned();
    let v = version.as_str();
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path();
    let root = ws.join("cli_manifests/depth_fixture");
    fs::create_dir_all(ws.join("crates/xtask/data")).unwrap();
    fs::create_dir_all(&root).unwrap();
    fs::write(
        ws.join("crates/xtask/data/agent_registry.toml"),
        registry_without_depth(),
    )
    .unwrap();
    let pointers = [
        format!("pointers/latest_supported/{TARGET}.txt"),
        format!("pointers/latest_validated/{TARGET}.txt"),
    ];
    for file in ["RULES.json", "SCHEMA.json", "VERSION_METADATA_SCHEMA.json"]
        .into_iter()
        .chain(["latest_validated.txt", "min_supported.txt"])
        .chain(pointers.iter().map(String::as_str))
    {
        let path = root.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, template(&format!("manifest-root/{file}"))).unwrap();
    }
    let ws_arg = ws.to_str().unwrap();
    let root_arg = root.to_str().unwrap();

    let plan: Value = serde_json::from_str(&xtask(
        ws,
        &[
            "manifest-acquisition-plan",
            "--agent",
            "depth_fixture",
            "--version",
            v,
            "--workspace-root",
            ws_arg,
        ],
    ))
    .unwrap();
    assert_eq!(plan["snapshot"]["command"], "depth-fixture-snapshot");
    assert_eq!(
        plan["wrapper_coverage_command"],
        "depth-fixture-wrapper-coverage"
    );
    let rendered = plan.to_string();
    assert!(
        rendered.contains("example-owner/depth-isolated"),
        "{rendered}"
    );
    assert!(rendered.contains(&format!("v{v}")), "{rendered}");
    assert!(rendered.contains(TARGET), "{rendered}");

    let per_target = root.join(format!("snapshots/{v}/{TARGET}.json"));
    xtask(
        ws,
        &[
            "depth-fixture-snapshot",
            "--depth-fixture-binary",
            binary.to_str().unwrap(),
            "--out-file",
            per_target.to_str().unwrap(),
            "--capture-raw-help",
            "--raw-help-target",
            TARGET,
        ],
    );
    xtask(ws, &["manifest-union", "--root", root_arg, "--version", v]);
    xtask(
        ws,
        &[
            "depth-fixture-wrapper-coverage",
            "--out",
            root.join("wrapper_coverage.json").to_str().unwrap(),
            "--rules",
            root.join("RULES.json").to_str().unwrap(),
        ],
    );
    xtask(ws, &["manifest-report", "--root", root_arg, "--version", v]);
    let report: Value = serde_json::from_str(
        &fs::read_to_string(root.join(format!("reports/{v}/coverage.any.json"))).unwrap(),
    )
    .unwrap();
    // Every upstream unit is covered or waived, and the wrapper claims nothing the CLI lacks.
    let deltas = &report["deltas"];
    for delta in ["missing_commands", "missing_flags", "missing_args"] {
        assert_eq!(deltas[delta], serde_json::json!([]), "{delta}");
    }
    for delta in [
        "unsupported",
        "wrapper_only_commands",
        "wrapper_only_flags",
        "wrapper_only_args",
    ] {
        assert!(deltas.get(delta).is_none(), "{delta}: {}", deltas[delta]);
    }
    let waived: Vec<_> = deltas["intentionally_unsupported"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| (d["path"].clone(), d["key"].clone()))
        .collect();
    assert_eq!(
        waived,
        [(serde_json::json!(["run"]), serde_json::json!("--help"))]
    );
    assert_eq!(deltas["excluded_commands"].as_array().unwrap().len(), 1);
    assert_eq!(deltas["excluded_flags"].as_array().unwrap().len(), 3);
    // A gap is never hidden: an uncovered upstream flag surfaces as missing.
    let coverage_path = root.join("wrapper_coverage.json");
    let covered = fs::read_to_string(&coverage_path).unwrap();
    let mut gap: Value = serde_json::from_str(&covered).unwrap();
    gap["coverage"][0]["flags"]
        .as_array_mut()
        .unwrap()
        .retain(|f| f["key"] != "--model");
    fs::write(&coverage_path, gap.to_string()).unwrap();
    xtask(ws, &["manifest-report", "--root", root_arg, "--version", v]);
    let report: Value = serde_json::from_str(
        &fs::read_to_string(root.join(format!("reports/{v}/coverage.any.json"))).unwrap(),
    )
    .unwrap();
    let missing = &report["deltas"]["missing_flags"];
    assert_eq!(missing.as_array().unwrap().len(), 1, "{missing}");
    assert_eq!(missing[0]["path"], serde_json::json!(["run"]));
    assert_eq!(missing[0]["key"], "--model");
    fs::write(&coverage_path, covered).unwrap();
    xtask(ws, &["manifest-report", "--root", root_arg, "--version", v]);

    xtask(
        ws,
        &[
            "manifest-version-metadata",
            "--root",
            root_arg,
            "--version",
            v,
            "--status",
            "reported",
        ],
    );
    // The declared release validation passes, run as parity-promote runs it: `eval` from the
    // workspace root with `{binary_path}` relative to that root.
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let relative = binary
        .strip_prefix(&workspace)
        .map(|p| Path::new(".").join(p))
        .unwrap_or_else(|_| binary.clone());
    assert!(
        rendered.contains("\"./depth_fixture-x86_64-unknown-linux-gnu\""),
        "{rendered}"
    );
    let commands = plan["validation_commands"].as_array().unwrap();
    assert_eq!(commands.len(), 1);
    let status = Command::new("bash")
        .args(["-c", "set -euo pipefail; eval \"$1\"", "validate"])
        .arg(commands[0].as_str().unwrap())
        .env("DEPTH_FIXTURE_BINARY", &relative)
        .current_dir(&workspace)
        .status()
        .unwrap();
    assert!(
        status.success(),
        "release validation failed for {relative:?}"
    );

    // The acquired baseline validates. Promoting it regenerates the workspace-wide matrices, so
    // it runs in a full workspace (roadmap B.6), not in this scratch root.
    xtask(ws, &["manifest-validate", "--root", root_arg]);
    let version: Value =
        serde_json::from_str(&fs::read_to_string(root.join(format!("versions/{v}.json"))).unwrap())
            .unwrap();
    assert_eq!(version["status"], "reported");
}
