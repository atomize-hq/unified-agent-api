use clap::{CommandFactory, Parser};

use super::{
    harness::{
        base_args, base_args_with_mode, base_args_with_package_name, fixture_root,
        seed_release_touchpoints, snapshot_files, write_text,
    },
    run_cli, Cli,
};

#[test]
fn onboard_agent_help_text_includes_required_surface() {
    let top_help = Cli::command().render_help().to_string();
    assert!(top_help.contains("onboard-agent"));

    let err = Cli::try_parse_from(["xtask", "onboard-agent", "--help"])
        .expect_err("subcommand help should short-circuit parsing");
    assert_eq!(err.exit_code(), 0);
    let help_text = err.to_string();
    assert!(help_text.contains("--dry-run"));
    assert!(help_text.contains("--write"));
    assert!(help_text.contains("--approval"));
    assert!(help_text.contains("--agent-id"));
    assert!(help_text.contains("--canonical-target"));
    assert!(help_text.contains("--always-on-capability"));
    assert!(help_text.contains("--target-gated-capability"));
    assert!(help_text.contains("--config-gated-capability"));
    assert!(help_text.contains("--backend-extension"));
    assert!(help_text.contains("--support-matrix-enabled"));
    assert!(help_text.contains("--capability-matrix-enabled"));
    assert!(help_text.contains("--capability-matrix-target"));
    assert!(help_text.contains("--docs-release-track"));
    assert!(help_text.contains("--onboarding-pack-prefix"));
}

#[test]
fn onboard_agent_requires_one_mode_flag() {
    let output = run_cli(
        [
            "xtask",
            "onboard-agent",
            "--agent-id",
            "cursor",
            "--display-name",
            "Cursor CLI",
            "--crate-path",
            "crates/cursor",
            "--backend-module",
            "crates/agent_api/src/backends/cursor",
            "--manifest-root",
            "cli_manifests/cursor",
            "--package-name",
            "unified-agent-api-cursor",
            "--canonical-target",
            "linux-x64",
            "--wrapper-coverage-binding-kind",
            "generated_from_wrapper_crate",
            "--wrapper-coverage-source-path",
            "crates/cursor",
            "--always-on-capability",
            "agent_api.run",
            "--support-matrix-enabled",
            "true",
            "--capability-matrix-enabled",
            "true",
            "--docs-release-track",
            "crates-io",
            "--onboarding-pack-prefix",
            "cursor-cli-onboarding",
        ],
        &fixture_root("onboard-agent-missing-dry-run"),
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("--dry-run"));
    assert!(output.stderr.contains("--write"));
}

#[test]
fn onboard_agent_rejects_conflicting_mode_flags() {
    let output = run_cli(
        base_args_with_mode("cursor", "unified-agent-api-cursor", "--dry-run", true),
        &fixture_root("onboard-agent-conflicting-modes"),
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("--dry-run"));
    assert!(output.stderr.contains("--write"));
}

#[test]
fn onboard_agent_duplicate_agent_id_exits_with_validation_code() {
    let fixture = fixture_root("onboard-agent-duplicate-id");
    let output = run_cli(base_args("codex"), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(
        output
            .stderr
            .contains("agent_id `codex` already exists in crates/xtask/data/agent_registry.toml"),
        "stderr did not mention duplicate agent id:\n{}",
        output.stderr
    );
}

#[test]
fn onboard_agent_duplicate_registry_package_name_exits_with_validation_code() {
    let fixture = fixture_root("onboard-agent-duplicate-package-name");
    let output = run_cli(
        base_args_with_package_name("cursor", "unified-agent-api-codex"),
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(
        output
            .stderr
            .contains("package_name `unified-agent-api-codex` is already owned by agent `codex`"),
        "stderr did not mention duplicate package name:\n{}",
        output.stderr
    );
}

#[test]
fn onboard_agent_duplicate_workspace_package_name_exits_with_validation_code() {
    let fixture = fixture_root("onboard-agent-workspace-package-name");
    let output = run_cli(
        base_args_with_package_name("cursor", "unified-agent-api"),
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(
        output.stderr.contains(
            "package_name `unified-agent-api` already exists in workspace member `crates/agent_api` (crates/agent_api/Cargo.toml)"
        ),
        "stderr did not mention workspace package name conflict:\n{}",
        output.stderr
    );
}

#[test]
fn onboard_agent_preexisting_target_conflict_exits_with_validation_code() {
    let fixture = fixture_root("onboard-agent-target-conflict");
    write_text(
        &fixture.join("cli_manifests/cursor/current.json"),
        "{\n  \"expected_targets\": [\"darwin-arm64\"],\n  \"inputs\": []\n}\n",
    );

    let output = run_cli(base_args("cursor"), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("conflicts with proposed canonical_targets"));
}

#[test]
fn onboard_agent_requires_explicit_capability_matrix_target_for_target_scoped_projection() {
    let fixture = fixture_root("onboard-agent-missing-capability-matrix-target");
    let mut args = base_args("cursor");
    let position = args
        .iter()
        .position(|value| value == "--capability-matrix-target")
        .expect("base args capability target flag");
    args.drain(position..=position + 1);

    let output = run_cli(args, &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("--capability-matrix-target is required when capability-matrix publication uses target-scoped declarations"));
}

#[test]
fn onboard_agent_dry_run_preview_is_deterministic_and_writes_nothing() {
    let fixture = fixture_root("onboard-agent-preview");
    seed_release_touchpoints(&fixture);

    let before = snapshot_files(&fixture);
    let first = run_cli(base_args("cursor"), &fixture);
    let second = run_cli(base_args("cursor"), &fixture);
    let after = snapshot_files(&fixture);

    assert_eq!(first.exit_code, 0, "stderr:\n{}", first.stderr);
    assert_eq!(second.exit_code, 0, "stderr:\n{}", second.stderr);
    assert_eq!(
        first.stdout, second.stdout,
        "dry-run output must be deterministic"
    );
    assert_eq!(before, after, "dry-run must not write any files");

    let sections = [
        "== ONBOARD-AGENT DRY RUN ==",
        "== INPUT SUMMARY ==",
        "== REGISTRY ENTRY PREVIEW ==",
        "== DOCS SCAFFOLD PREVIEW ==",
        "== MANIFEST ROOT PREVIEW ==",
        "== RELEASE/PUBLICATION TOUCHPOINTS ==",
        "== MANUAL FOLLOW-UP ==",
        "== RESULT ==",
    ];
    let mut cursor = 0usize;
    for section in sections {
        let found = first.stdout[cursor..]
            .find(section)
            .map(|offset| cursor + offset)
            .unwrap_or_else(|| panic!("missing section `{section}` in stdout:\n{}", first.stdout));
        cursor = found + section.len();
    }

    assert!(first
        .stdout
        .contains("Path: crates/xtask/data/agent_registry.toml"));
    assert!(first.stdout.contains("capability_matrix_target: linux-x64"));
    assert!(first
        .stdout
        .contains("capability_matrix_target = \"linux-x64\""));
    assert!(first
        .stdout
        .contains("Path: docs/agents/lifecycle/cursor-cli-onboarding/README.md"));
    assert!(first
        .stdout
        .contains("Path: cli_manifests/cursor/current.json"));
    assert!(first
        .stdout
        .contains("Path: Cargo.toml will ensure workspace member `crates/cursor` is enrolled."));
    assert!(first.stdout.contains("Path: docs/crates-io-release.md will ensure the generated release block includes `unified-agent-api-cursor` on release track `crates-io`."));
    assert!(first
        .stdout
        .contains("Workflow and script files remain unchanged:"));
    assert!(first
        .stdout
        .contains("<!-- generated-by: xtask onboard-agent; owner: control-plane -->"));
    assert!(first.stdout.contains("## Next executable runtime step"));
    assert!(first.stdout.contains("Next executable runtime step:"));
    assert!(first
        .stdout
        .contains("cargo run -p xtask -- scaffold-wrapper-crate --agent cursor --write"));
    assert!(first
        .stdout
        .contains("`onboard-agent` does not create the wrapper crate."));
    assert!(first
        .stdout
        .contains("Shared onboarding plan preview; no filesystem writes performed."));
    assert!(!first.stdout.contains(" M1"));
    assert!(!first.stdout.contains("future M2"));
    assert!(!first.stdout.contains("dry-run mode"));
    assert!(!first.stdout.contains("Create the wrapper crate"));
    assert!(!first
        .stdout
        .contains("Next executable runtime step: implement the runtime-owned wrapper crate"));
    assert!(!first
        .stdout
        .contains("When the wrapper crate is crates.io-publishable"));
    assert!(!first.stdout.contains("LICENSE-APACHE"));
    assert!(!first.stdout.contains("LICENSE-MIT"));
    assert!(!first.stdout.contains("readme = \"README.md\""));
}
