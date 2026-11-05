//! `onboard-agent` scaffolds a new manifest root's governing data (roadmap E.2): `RULES.json`,
//! `SCHEMA.json`, `VERSION_METADATA_SCHEMA.json` and `VALIDATOR_SPEC.md`.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;

use super::{
    harness::{
        args_with_overrides, base_args, fixture_root, run_xtask, seed_release_touchpoints,
        snapshot_files, write_args, write_text,
    },
    run_cli,
};

const GOVERNING_FILES: [&str; 4] = [
    "RULES.json",
    "SCHEMA.json",
    "VALIDATOR_SPEC.md",
    "VERSION_METADATA_SCHEMA.json",
];
const GNU: &str = "x86_64-unknown-linux-gnu";

fn json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("read json")).expect("parse json")
}

/// The governing templates of the synthetic subject, the shape every committed root follows.
fn synthetic_template(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/depth-synthetic/governing/manifest-root")
        .join(name);
    json(&path)
}

/// Onboard `depth_fixture` on the gnu target: the synthetic subject's identity and target.
fn depth_fixture_args(mode_flag: &str) -> Vec<String> {
    args_with_overrides(
        mode_flag,
        "depth_fixture",
        "unified-agent-api-depth-fixture",
        &[
            ("--display-name", "Depth Fixture CLI"),
            ("--crate-path", "crates/depth_fixture"),
            (
                "--backend-module",
                "crates/agent_api/src/backends/depth_fixture",
            ),
            ("--manifest-root", "cli_manifests/depth_fixture"),
            ("--canonical-target", GNU),
            ("--wrapper-coverage-source-path", "crates/depth_fixture"),
            (
                "--target-gated-capability",
                "agent_api.tools.mcp.list.v1:x86_64-unknown-linux-gnu",
            ),
            ("--capability-matrix-target", GNU),
            ("--onboarding-pack-prefix", "depth-fixture-cli-onboarding"),
        ],
        false,
    )
}

#[test]
fn dry_run_plan_lists_the_governing_files_without_writing_them() {
    let fixture = fixture_root("onboard-agent-governing-dry-run");
    seed_release_touchpoints(&fixture);
    let before = snapshot_files(&fixture);

    let output = run_cli(base_args("cursor"), &fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert_eq!(before, snapshot_files(&fixture), "dry run must not write");
    let manifest_section = output
        .stdout
        .split("== MANIFEST ROOT PREVIEW ==")
        .nth(1)
        .and_then(|rest| rest.split("== RELEASE/PUBLICATION TOUCHPOINTS ==").next())
        .expect("manifest root preview section");
    for name in GOVERNING_FILES {
        assert!(
            manifest_section.contains(&format!("Path: cli_manifests/cursor/{name}")),
            "{name} missing from the plan:\n{manifest_section}"
        );
    }
    assert!(manifest_section.contains("\"tool_name\": \"cursor-cli\""));
    assert!(manifest_section.contains("\"required_target\": \"linux-x64\""));
    assert!(manifest_section.contains("# Cursor CLI Parity Validator Spec"));
    // The preview shows exactly what --write would write.
    let write_fixture = fixture_root("onboard-agent-governing-dry-run-write");
    seed_release_touchpoints(&write_fixture);
    let write = run_cli(write_args("cursor"), &write_fixture);
    assert_eq!(write.exit_code, 0, "stderr:\n{}", write.stderr);
    for name in GOVERNING_FILES {
        let written = fs::read_to_string(write_fixture.join("cli_manifests/cursor").join(name))
            .expect("read governing file");
        assert!(
            manifest_section.contains(&written),
            "{name}: preview differs from the written file"
        );
    }
}

#[test]
fn write_scaffolds_deterministic_governing_data_and_replays_identically() {
    let first_fixture = fixture_root("onboard-agent-governing-write-a");
    let second_fixture = fixture_root("onboard-agent-governing-write-b");
    // An existing agent's committed governing data, which onboarding must leave alone.
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for name in GOVERNING_FILES {
        let committed =
            fs::read_to_string(repo_root.join("cli_manifests/opencode").join(name)).expect("read");
        write_text(
            &first_fixture.join("cli_manifests/opencode").join(name),
            &committed,
        );
    }
    let before = snapshot_files(&first_fixture);
    for fixture in [&first_fixture, &second_fixture] {
        seed_release_touchpoints(fixture);
        let output = run_cli(write_args("cursor"), fixture);
        assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    }
    let after = snapshot_files(&first_fixture);
    for (path, bytes) in &before {
        if path.starts_with("cli_manifests/") {
            assert_eq!(after.get(path), Some(bytes), "{path} changed");
        }
    }
    let new_manifest_paths = after
        .keys()
        .filter(|path| path.starts_with("cli_manifests/") && !before.contains_key(*path))
        .collect::<Vec<_>>();
    assert!(
        new_manifest_paths
            .iter()
            .all(|path| path.starts_with("cli_manifests/cursor/")),
        "{new_manifest_paths:?}"
    );
    let after_first = snapshot_files(&first_fixture);
    let replay = run_cli(write_args("cursor"), &first_fixture);
    assert_eq!(replay.exit_code, 0, "stderr:\n{}", replay.stderr);
    assert!(replay
        .stdout
        .contains("Mutation summary: 0 written, 20 identical, 20 total planned."));
    assert_eq!(after_first, snapshot_files(&first_fixture));

    for name in GOVERNING_FILES {
        let a = fs::read(first_fixture.join("cli_manifests/cursor").join(name)).expect("read a");
        let b = fs::read(second_fixture.join("cli_manifests/cursor").join(name)).expect("read b");
        assert_eq!(a, b, "{name} must be deterministic");
    }

    let root = first_fixture.join("cli_manifests/cursor");
    let rules = json(&root.join("RULES.json"));
    assert_eq!(rules["scope"], "cursor-cli-parity");
    assert_eq!(rules["union"]["tool_name"], "cursor-cli");
    assert_eq!(rules["union"]["required_target"], "linux-x64");
    assert_eq!(
        rules["union"]["expected_targets"],
        serde_json::json!(["linux-x64"])
    );
    assert_eq!(
        rules["union"]["platform_mapping"],
        serde_json::json!({"linux-x64": "linux"})
    );
    assert_eq!(
        rules["version_metadata"]["schema"],
        "cli_manifests/cursor/VERSION_METADATA_SCHEMA.json"
    );
    // No depth path: the root does not adopt Annex B, declares no acquisition source and no
    // parity exclusions.
    assert!(rules["storage"]["committed"]["reports"]
        .get("depth_record")
        .is_none());
    assert!(rules.get("acquisition").is_none());
    assert!(rules.get("parity_exclusions").is_none());
    let schema = json(&root.join("SCHEMA.json"));
    assert_eq!(schema["$id"], "cli_manifests/cursor/SCHEMA.json");
    let version_schema = json(&root.join("VERSION_METADATA_SCHEMA.json"));
    assert_eq!(
        version_schema["$id"],
        "cli_manifests/cursor/VERSION_METADATA_SCHEMA.json"
    );
    let spec = fs::read_to_string(root.join("VALIDATOR_SPEC.md")).expect("read spec");
    assert!(spec.contains("`cargo run -p xtask -- manifest-validate --root cli_manifests/cursor`"));
    assert!(spec.contains("This root does not adopt the depth record"));
    assert!(!spec.contains("{{"), "unrendered placeholder:\n{spec}");
    // Onboarding writes no per-version metadata: it knows no acquired version.
    let versions = fs::read_dir(root.join("versions"))
        .expect("read versions")
        .map(|entry| entry.expect("entry").file_name())
        .collect::<Vec<_>>();
    assert_eq!(versions, vec![std::ffi::OsString::from(".gitkeep")]);
}

#[test]
fn write_refuses_a_divergent_governing_file_without_partial_writes() {
    let fixture = fixture_root("onboard-agent-governing-divergent");
    seed_release_touchpoints(&fixture);
    let first = run_cli(write_args("cursor"), &fixture);
    assert_eq!(first.exit_code, 0, "stderr:\n{}", first.stderr);
    write_text(
        &fixture.join("cli_manifests/cursor/RULES.json"),
        "{\"edited\": true}\n",
    );
    let before = snapshot_files(&fixture);

    let second = run_cli(write_args("cursor"), &fixture);

    assert_eq!(second.exit_code, 2);
    assert!(second.stderr.contains("cli_manifests/cursor/RULES.json"));
    assert!(second.stderr.contains("divergent"));
    assert_eq!(before, snapshot_files(&fixture));
}

#[test]
fn scaffolded_governing_data_loads_in_manifest_validate() {
    let fixture = fixture_root("onboard-agent-governing-validate");
    seed_release_touchpoints(&fixture);
    let output = run_cli(write_args("cursor"), &fixture);
    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);

    let validate = run_xtask(
        &fixture,
        [
            "manifest-validate",
            "--root",
            "cli_manifests/cursor",
            "--json",
        ],
    );

    // RULES, SCHEMA and VERSION_METADATA_SCHEMA load and compile (no exit 3). What is left is
    // the bootstrap state the runtime lane completes (no pointer files, no wrapper coverage, and
    // a bootstrap `current.json` that is not yet a union snapshot) and the workspace-wide
    // support-matrix artifact this scratch workspace does not hold.
    assert_eq!(validate.exit_code, 2, "stderr:\n{}", validate.stderr);
    let report: Value = serde_json::from_str(&validate.stdout).expect("json report");
    let codes = report["violations"]
        .as_array()
        .expect("violations")
        .iter()
        .map(|v| v["code"].as_str().expect("code").to_string())
        .collect::<BTreeSet<_>>();
    let bootstrap = [
        "CURRENT_SCHEMA_INVALID",
        "CURRENT_WRONG_KIND",
        "POINTER_MISSING_FILE",
        "SUPPORT_MATRIX_ARTIFACT_MISSING",
        "WRAPPER_COVERAGE_MISSING",
    ]
    .map(String::from)
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert!(codes.is_subset(&bootstrap), "{codes:?}");
    assert!(codes.contains("POINTER_MISSING_FILE"), "{codes:?}");
}

#[test]
fn scaffolded_governing_data_matches_the_committed_root_shape() {
    let fixture = fixture_root("onboard-agent-governing-shape");
    seed_release_touchpoints(&fixture);
    let output = run_cli(depth_fixture_args("--write"), &fixture);
    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    let root = fixture.join("cli_manifests/depth_fixture");

    // The schemas are the committed ones; only the human-facing title and description differ.
    for name in ["SCHEMA.json", "VERSION_METADATA_SCHEMA.json"] {
        let mut generated = json(&root.join(name));
        let mut template = synthetic_template(name);
        for value in [&mut generated, &mut template] {
            let object = value.as_object_mut().expect("object");
            object.remove("title");
            object.remove("description");
        }
        assert_eq!(generated, template, "{name}");
    }

    // RULES is the synthetic subject's, without what onboarding cannot know (acquisition and
    // parity exclusions) and without Annex B adoption; descriptive prose aside.
    let mut generated = json(&root.join("RULES.json"));
    let mut template = synthetic_template("RULES.json");
    let template_object = template.as_object_mut().expect("object");
    template_object.remove("acquisition");
    template_object.remove("parity_exclusions");
    template["storage"]["committed"]["reports"]
        .as_object_mut()
        .expect("reports")
        .remove("depth_record");
    for value in [&mut generated, &mut template] {
        let globals = value["globals"].as_object_mut().expect("globals");
        globals.remove("meaning");
        globals.remove("notes");
        value["globals"]["effective_flags_model"]
            .as_object_mut()
            .expect("model")
            .remove("reporting");
        value["union"]["promotion_policy"]
            .as_object_mut()
            .expect("policy")
            .remove("notes");
    }
    assert_eq!(generated, template);
}

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
        .expect("build fixture");
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

fn xtask_ok(cwd: &Path, args: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(cwd)
        .env("SOURCE_DATE_EPOCH", "1793664000")
        .output()
        .expect("run xtask");
    assert!(
        output.status.success(),
        "xtask {args:?} failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The scaffolded governing data drives the real manifest commands, as the synthetic subject's
/// templates do, and the acquired version then passes `manifest-validate` for the root.
#[test]
fn scaffolded_governing_data_drives_the_real_manifest_commands() {
    let binary = fixture_binary();
    let version_output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("fixture version");
    let version = String::from_utf8(version_output.stdout).expect("utf-8");
    let version = version
        .trim()
        .strip_prefix("depth_fixture ")
        .expect("version prefix")
        .to_owned();
    let v = version.as_str();

    let fixture = fixture_root("onboard-agent-governing-pipeline");
    seed_release_touchpoints(&fixture);
    let output = run_cli(depth_fixture_args("--write"), &fixture);
    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    let root = fixture.join("cli_manifests/depth_fixture");
    let root_arg = root.to_str().expect("utf-8 root");

    // Runtime-lane state that onboarding does not write: pointer files, all `none`, and no
    // promoted `current.json` yet (the validator's bootstrap mode).
    for pointer in [
        "min_supported.txt".to_string(),
        "latest_validated.txt".to_string(),
        format!("pointers/latest_supported/{GNU}.txt"),
        format!("pointers/latest_validated/{GNU}.txt"),
    ] {
        write_text(&root.join(pointer), "none\n");
    }
    fs::remove_file(root.join("current.json")).expect("remove bootstrap current.json");

    let per_target = root.join(format!("snapshots/{v}/{GNU}.json"));
    xtask_ok(
        &fixture,
        &[
            "depth-fixture-snapshot",
            "--depth-fixture-binary",
            binary.to_str().expect("utf-8 binary"),
            "--out-file",
            per_target.to_str().expect("utf-8 out"),
            "--capture-raw-help",
            "--raw-help-target",
            GNU,
        ],
    );
    xtask_ok(
        &fixture,
        &["manifest-union", "--root", root_arg, "--version", v],
    );
    xtask_ok(
        &fixture,
        &[
            "depth-fixture-wrapper-coverage",
            "--out",
            root.join("wrapper_coverage.json").to_str().expect("utf-8"),
            "--rules",
            root.join("RULES.json").to_str().expect("utf-8"),
        ],
    );
    xtask_ok(
        &fixture,
        &["manifest-report", "--root", root_arg, "--version", v],
    );
    let report = json(&root.join(format!("reports/{v}/coverage.any.json")));
    // Without parity exclusions every upstream unit is in parity: nothing is excluded.
    assert!(
        report["deltas"].get("excluded_commands").is_none(),
        "{report}"
    );
    xtask_ok(
        &fixture,
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
    // The root validates. The only violation left is workspace-wide: the scratch workspace holds
    // no support-matrix artifact, which promotion regenerates in a full workspace.
    let validate = run_xtask(
        &fixture,
        ["manifest-validate", "--root", root_arg, "--json"],
    );
    let report: Value = serde_json::from_str(&validate.stdout).expect("json report");
    let codes = report["violations"]
        .as_array()
        .expect("violations")
        .iter()
        .map(|v| v["code"].as_str().expect("code"))
        .collect::<Vec<_>>();
    assert_eq!(
        codes,
        ["SUPPORT_MATRIX_ARTIFACT_MISSING"],
        "{}",
        validate.stderr
    );
    let metadata = json(&root.join(format!("versions/{v}.json")));
    assert_eq!(metadata["status"], "reported");
}
