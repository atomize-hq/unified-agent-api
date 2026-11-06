//! `onboard-agent` scaffolds a new manifest root's governing data (roadmap E.2): `RULES.json`,
//! `SCHEMA.json`, `VERSION_METADATA_SCHEMA.json`, `VALIDATOR_SPEC.md`, the root version pointers
//! and an empty wrapper coverage, and lists every written path in `written-paths.json`.

use std::{collections::BTreeSet, fs, path::Path};

use serde_json::Value;

use super::{
    harness::{
        approval_args, args_with_overrides, base_args, fixture_root, seed_approval_artifact,
        seed_release_touchpoints, snapshot_files, write_args, write_text,
    },
    run_cli,
};

pub(super) const GOVERNING_FILES: [&str; 4] = [
    "RULES.json",
    "SCHEMA.json",
    "VALIDATOR_SPEC.md",
    "VERSION_METADATA_SCHEMA.json",
];
/// The root version metadata: no version is known yet, so every pointer is `none`.
const VERSION_POINTERS: [&str; 4] = [
    "latest_validated.txt",
    "min_supported.txt",
    "pointers/latest_supported/linux-x64.txt",
    "pointers/latest_validated/linux-x64.txt",
];
pub(super) const GNU: &str = "x86_64-unknown-linux-gnu";
const INVENTORY: &str = "docs/agents/lifecycle/cursor-cli-onboarding/governance/written-paths.json";

pub(super) fn json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("read json")).expect("parse json")
}

/// Onboard `depth_fixture` on the gnu target: the synthetic subject's identity and target.
pub(super) fn depth_fixture_args(mode_flag: &str) -> Vec<String> {
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

fn manifest_section(stdout: &str) -> &str {
    stdout
        .split("== MANIFEST ROOT PREVIEW ==")
        .nth(1)
        .and_then(|rest| rest.split("== RELEASE/PUBLICATION TOUCHPOINTS ==").next())
        .expect("manifest root preview section")
}

#[test]
fn dry_run_plan_lists_the_governing_files_without_writing_them() {
    let fixture = fixture_root("onboard-agent-governing-dry-run");
    seed_release_touchpoints(&fixture);
    let before = snapshot_files(&fixture);

    let output = run_cli(base_args("cursor"), &fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert_eq!(before, snapshot_files(&fixture), "dry run must not write");
    let section = manifest_section(&output.stdout);
    let mut names = GOVERNING_FILES.to_vec();
    names.extend(VERSION_POINTERS);
    names.push("wrapper_coverage.json");
    for name in &names {
        assert!(
            section.contains(&format!("Path: cli_manifests/cursor/{name}")),
            "{name} missing from the plan:\n{section}"
        );
    }
    assert!(section.contains("\"tool_name\": \"cursor-cli\""));
    assert!(section.contains("\"required_target\": \"linux-x64\""));
    assert!(section.contains("# Cursor CLI Parity Validator Spec"));
    assert!(output.stdout.contains(&format!("Path: {INVENTORY}")));

    // The preview shows exactly what --write writes.
    let write_fixture = fixture_root("onboard-agent-governing-dry-run-write");
    seed_release_touchpoints(&write_fixture);
    let write = run_cli(write_args("cursor"), &write_fixture);
    assert_eq!(write.exit_code, 0, "stderr:\n{}", write.stderr);
    for name in &names {
        let written = fs::read_to_string(write_fixture.join("cli_manifests/cursor").join(name))
            .expect("read scaffolded file");
        assert!(
            section.contains(&written),
            "{name}: preview differs from the written file"
        );
    }
    let inventory = fs::read_to_string(write_fixture.join(INVENTORY)).expect("read inventory");
    assert!(output.stdout.contains(&inventory));
}

#[test]
fn write_scaffolds_deterministic_governing_data_and_leaves_other_roots_alone() {
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
    let second = snapshot_files(&second_fixture);
    for (path, bytes) in &after {
        if path.starts_with("cli_manifests/cursor/") || path == INVENTORY {
            assert_eq!(
                second.get(path),
                Some(bytes),
                "{path} must be deterministic"
            );
        }
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
    assert_eq!(
        json(&root.join("SCHEMA.json"))["$id"],
        "cli_manifests/cursor/SCHEMA.json"
    );
    assert_eq!(
        json(&root.join("VERSION_METADATA_SCHEMA.json"))["$id"],
        "cli_manifests/cursor/VERSION_METADATA_SCHEMA.json"
    );
    let spec = fs::read_to_string(root.join("VALIDATOR_SPEC.md")).expect("read spec");
    assert!(spec.contains("`cargo run -p xtask -- manifest-validate --root cli_manifests/cursor`"));
    assert!(spec.contains("This root does not adopt the depth record"));
    assert!(!spec.contains("{{"), "unrendered placeholder:\n{spec}");
    // Root version metadata: no version is known, so every pointer says so.
    for pointer in VERSION_POINTERS {
        assert_eq!(
            fs::read_to_string(root.join(pointer)).expect("read pointer"),
            "none\n",
            "{pointer}"
        );
    }
    assert_eq!(
        json(&root.join("wrapper_coverage.json")),
        serde_json::json!({"schema_version": 1, "coverage": []})
    );
}

#[test]
fn written_paths_inventory_lists_every_written_file_and_replays_identically() {
    let fixture = fixture_root("onboard-agent-governing-inventory");
    seed_release_touchpoints(&fixture);
    let before = snapshot_files(&fixture);

    let first = run_cli(write_args("cursor"), &fixture);
    assert_eq!(first.exit_code, 0, "stderr:\n{}", first.stderr);
    let after = snapshot_files(&fixture);

    let inventory: Vec<String> =
        serde_json::from_value(json(&fixture.join(INVENTORY))).expect("string array");
    let mut sorted = inventory.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(inventory, sorted, "inventory must be sorted and unique");
    let changed = after
        .iter()
        .filter(|(path, bytes)| before.get(*path) != Some(*bytes))
        .map(|(path, _)| path.clone())
        .collect::<BTreeSet<_>>();
    // Every file the write changed is listed, and every listed path exists after the write.
    let listed = inventory.iter().cloned().collect::<BTreeSet<_>>();
    assert!(
        changed.is_subset(&listed),
        "unlisted: {:?}",
        changed.difference(&listed).collect::<Vec<_>>()
    );
    for path in &listed {
        assert!(after.contains_key(path), "{path} listed but not written");
    }
    assert!(listed.contains(INVENTORY));
    assert!(listed.contains("crates/xtask/data/agent_registry.toml"));
    assert!(listed.contains("Cargo.toml"));
    assert!(listed.contains("docs/crates-io-release.md"));
    assert_eq!(
        inventory.len(),
        24,
        "inventory must list each planned mutation"
    );
    assert!(first
        .stdout
        .contains("Mutation summary: 24 written, 0 identical, 24 total planned."));

    let replay = run_cli(write_args("cursor"), &fixture);
    assert_eq!(replay.exit_code, 0, "stderr:\n{}", replay.stderr);
    assert!(replay
        .stdout
        .contains("Mutation summary: 0 written, 24 identical, 24 total planned."));
    assert_eq!(after, snapshot_files(&fixture), "replay must be a no-op");
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

/// Target order cannot choose the required target: every multi-target scaffold is refused, in
/// either order, before anything is written.
#[test]
fn multi_target_scaffolds_are_refused_whatever_the_target_order() {
    for (first, second) in [("linux-x64", "darwin-arm64"), ("darwin-arm64", "linux-x64")] {
        for mode in ["--dry-run", "--write"] {
            let fixture = fixture_root("onboard-agent-governing-multi-target");
            seed_release_touchpoints(&fixture);
            let mut args = write_args("cursor");
            args[2] = mode.to_string();
            let at = args
                .iter()
                .position(|arg| arg == "--canonical-target")
                .expect("flag");
            args[at + 1] = first.to_string();
            args.insert(at + 2, "--canonical-target".to_string());
            args.insert(at + 3, second.to_string());
            let before = snapshot_files(&fixture);

            let output = run_cli(args, &fixture);

            assert_eq!(output.exit_code, 2, "{first},{second} {mode}");
            assert!(
                output.stderr.contains("exactly one canonical target"),
                "{}",
                output.stderr
            );
            assert!(output.stdout.is_empty(), "no plan is printed");
            assert_eq!(before, snapshot_files(&fixture));
        }
    }
}

/// An approval that states an exact upstream version asks for version metadata onboarding cannot
/// write truthfully, so both modes refuse it before anything is written.
#[test]
fn approvals_stating_a_depth_version_are_refused() {
    for mode in ["--dry-run", "--write"] {
        let fixture = fixture_root("onboard-agent-governing-depth-version");
        seed_release_touchpoints(&fixture);
        let approval_rel =
            "docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml";
        let approval = seed_approval_artifact(&fixture, approval_rel, "cursor", "cursor", None);
        let path = fixture.join(&approval);
        let mut text = fs::read_to_string(&path).expect("read approval");
        text.push_str("\n[support_depth]\nversion = \"1.2.3\"\n");
        write_text(&path, &text);
        let before = snapshot_files(&fixture);

        let output = run_cli(approval_args(mode, &approval), &fixture);

        assert_eq!(output.exit_code, 2, "{mode}: {}", output.stderr);
        assert!(
            output
                .stderr
                .contains("`[support_depth].version = \"1.2.3\"`"),
            "{}",
            output.stderr
        );
        assert_eq!(before, snapshot_files(&fixture));
    }
}
