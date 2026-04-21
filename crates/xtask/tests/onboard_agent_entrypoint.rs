use std::{fs, path::Path};

use clap::{CommandFactory, Parser, Subcommand};
use serde_json::json;
use xtask::onboard_agent;

#[path = "support/onboard_agent_harness.rs"]
mod harness;

use harness::{
    approval_args, args_with_overrides, assert_sections_in_order, base_args, base_args_with_mode,
    base_args_with_package_name, fixture_root, seed_approval_artifact, seed_release_touchpoints,
    sha256_hex, snapshot_files, write_args, write_text, HarnessOutput,
};

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Project automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Preview the next control-plane onboarding packet without writing files.
    OnboardAgent(onboard_agent::Args),
}

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
fn onboard_agent_dry_run_preview_is_deterministic_and_writes_nothing() {
    let fixture = fixture_root("onboard-agent-preview");
    write_text(
        &fixture.join("docs/crates-io-release.md"),
        "# Release docs\n\nManual contract text.\n",
    );
    write_text(
        &fixture.join(".github/workflows/publish-crates.yml"),
        "name: publish-crates\n",
    );
    write_text(
        &fixture.join("scripts/publish_crates.py"),
        "print('publish')\n",
    );
    write_text(
        &fixture.join("scripts/validate_publish_versions.py"),
        "print('validate')\n",
    );
    write_text(
        &fixture.join("scripts/check_publish_readiness.py"),
        "print('readiness')\n",
    );

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
    assert!(first
        .stdout
        .contains("Path: docs/project_management/next/cursor-cli-onboarding/README.md"));
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
        .contains("Shared onboarding plan preview; no filesystem writes performed."));
    assert!(!first.stdout.contains(" M1"));
    assert!(!first.stdout.contains("future M2"));
    assert!(!first.stdout.contains("dry-run mode"));
    assert!(!first.stdout.contains("Create the wrapper crate"));
}

#[test]
fn onboard_agent_approval_dry_run_matches_raw_descriptor_preview_and_writes_nothing() {
    let fixture = fixture_root("onboard-agent-approval-preview");
    seed_release_touchpoints(&fixture);
    let approval_path = seed_approval_artifact(
        &fixture,
        "docs/project_management/next/cursor-cli-onboarding/governance/approved-agent.toml",
        "cursor",
        "cursor",
        None,
    );

    let before = snapshot_files(&fixture);
    let approval = run_cli(approval_args("--dry-run", &approval_path), &fixture);
    let raw = run_cli(base_args("cursor"), &fixture);
    let after = snapshot_files(&fixture);

    assert_eq!(approval.exit_code, 0, "stderr:\n{}", approval.stderr);
    assert_eq!(raw.exit_code, 0, "stderr:\n{}", raw.stderr);
    assert_eq!(before, after, "approval dry-run must not write any files");
    assert!(approval
        .stdout
        .contains("Shared onboarding plan preview; no filesystem writes performed."));
    assert!(raw
        .stdout
        .contains("Shared onboarding plan preview; no filesystem writes performed."));
    assert!(approval.stdout.contains("agent_id: cursor"));
    assert!(raw.stdout.contains("agent_id: cursor"));
    assert!(approval
        .stdout
        .contains("approval_artifact_path: docs/project_management/next/cursor-cli-onboarding/governance/approved-agent.toml"));
    assert!(approval.stdout.contains("approval_artifact_sha256: "));
    assert!(approval.stdout.contains(
        "Approval linkage: `docs/project_management/next/cursor-cli-onboarding/governance/approved-agent.toml` (`sha256:"
    ));
    assert!(approval.stdout.contains(
        "Approval linkage via `docs/project_management/next/cursor-cli-onboarding/governance/approved-agent.toml` (`sha256:"
    ));
    assert!(approval.stdout.contains("## Approval provenance"));
    assert!(approval.stdout.contains(
        "- approval ref: `docs/project_management/next/cursor-cli-onboarding/governance/approved-agent.toml`"
    ));
    assert!(!raw.stdout.contains("approval_artifact_path:"));
    assert!(!raw.stdout.contains("approval_artifact_sha256:"));
    assert!(!raw.stdout.contains("## Approval provenance"));
    assert!(!raw.stdout.contains(
        "Approval linkage: `docs/project_management/next/cursor-cli-onboarding/governance/approved-agent.toml` (`sha256:"
    ));
}

#[test]
fn onboard_agent_write_applies_plan_and_replays_identically() {
    let fixture = fixture_root("onboard-agent-write");
    seed_release_touchpoints(&fixture);

    let before = snapshot_files(&fixture);
    let first = run_cli(write_args("cursor"), &fixture);
    let after_first = snapshot_files(&fixture);
    let second = run_cli(write_args("cursor"), &fixture);
    let after_second = snapshot_files(&fixture);

    assert_eq!(first.exit_code, 0, "stderr:\n{}", first.stderr);
    assert_eq!(second.exit_code, 0, "stderr:\n{}", second.stderr);
    assert_ne!(before, after_first, "write mode must mutate the workspace");
    assert_eq!(
        after_first, after_second,
        "identical replay must be a no-op for filesystem state"
    );

    let sections = [
        "== ONBOARD-AGENT WRITE ==",
        "== INPUT SUMMARY ==",
        "== REGISTRY ENTRY PREVIEW ==",
        "== DOCS SCAFFOLD PREVIEW ==",
        "== MANIFEST ROOT PREVIEW ==",
        "== RELEASE/PUBLICATION TOUCHPOINTS ==",
        "== MANUAL FOLLOW-UP ==",
        "== RESULT ==",
    ];
    assert_sections_in_order(&first.stdout, &sections);
    assert_sections_in_order(&second.stdout, &sections);

    assert!(first
        .stdout
        .contains("Shared onboarding plan preview before apply."));
    assert!(first.stdout.contains("OK: onboard-agent write complete."));
    assert!(first
        .stdout
        .contains("Mutation summary: 15 written, 0 identical, 15 total planned."));
    assert!(second.stdout.contains("OK: onboard-agent write complete."));
    assert!(second
        .stdout
        .contains("Mutation summary: 0 written, 15 identical, 15 total planned."));

    let registry = fs::read_to_string(fixture.join("crates/xtask/data/agent_registry.toml"))
        .expect("read registry");
    assert!(registry.contains("agent_id = \"cursor\""));
    assert!(registry.contains("package_name = \"unified-agent-api-cursor\""));

    let root_manifest = fs::read_to_string(fixture.join("Cargo.toml")).expect("read root manifest");
    assert!(root_manifest.contains("\"crates/cursor\""));
    let wrapper_events_index = root_manifest
        .find("\"crates/wrapper_events\"")
        .expect("wrapper_events member");
    let cursor_index = root_manifest
        .find("\"crates/cursor\"")
        .expect("cursor member");
    assert!(cursor_index < wrapper_events_index);

    let readme = fs::read_to_string(
        fixture.join("docs/project_management/next/cursor-cli-onboarding/README.md"),
    )
    .expect("read docs README");
    assert!(readme.contains("# Cursor CLI onboarding pack"));

    let current_json = fs::read_to_string(fixture.join("cli_manifests/cursor/current.json"))
        .expect("read current");
    assert!(current_json.contains("\"expected_targets\": ["));
    let release_doc =
        fs::read_to_string(fixture.join("docs/crates-io-release.md")).expect("read release doc");
    assert!(release_doc
        .contains("<!-- generated-by: xtask onboard-agent; section: crates-io-release -->"));
    assert!(release_doc.contains("`unified-agent-api-cursor`"));
    assert!(release_doc.contains("## Publish order"));
    assert!(fixture
        .join("cli_manifests/cursor/versions/.gitkeep")
        .is_file());
    assert!(fixture
        .join("cli_manifests/cursor/pointers/latest_supported/.gitkeep")
        .is_file());
}

#[test]
fn onboard_agent_approval_write_applies_plan_and_replays_identically() {
    let fixture = fixture_root("onboard-agent-approval-write");
    seed_release_touchpoints(&fixture);
    let approval_path = seed_approval_artifact(
        &fixture,
        "docs/project_management/next/cursor-cli-onboarding/governance/approved-agent.toml",
        "cursor",
        "cursor",
        None,
    );

    let before = snapshot_files(&fixture);
    let first = run_cli(approval_args("--write", &approval_path), &fixture);
    let after_first = snapshot_files(&fixture);
    let second = run_cli(approval_args("--write", &approval_path), &fixture);
    let after_second = snapshot_files(&fixture);

    assert_eq!(first.exit_code, 0, "stderr:\n{}", first.stderr);
    assert_eq!(second.exit_code, 0, "stderr:\n{}", second.stderr);
    assert_ne!(
        before, after_first,
        "approval write mode must mutate the workspace"
    );
    assert_eq!(after_first, after_second);
    assert!(first.stdout.contains("OK: onboard-agent write complete."));
    assert!(first.stdout.contains("approval_artifact_path: docs/project_management/next/cursor-cli-onboarding/governance/approved-agent.toml"));
    assert!(first.stdout.contains("## Approval provenance"));
    assert!(second
        .stdout
        .contains("Mutation summary: 0 written, 15 identical, 15 total planned."));

    let readme = fs::read_to_string(
        fixture.join("docs/project_management/next/cursor-cli-onboarding/README.md"),
    )
    .expect("read approval-mode readme");
    let scope_brief = fs::read_to_string(
        fixture.join("docs/project_management/next/cursor-cli-onboarding/scope_brief.md"),
    )
    .expect("read approval-mode scope brief");
    let handoff = fs::read_to_string(
        fixture.join("docs/project_management/next/cursor-cli-onboarding/HANDOFF.md"),
    )
    .expect("read approval-mode handoff");

    assert!(readme.contains(
        "Approval linkage: `docs/project_management/next/cursor-cli-onboarding/governance/approved-agent.toml` (`sha256:"
    ));
    assert!(scope_brief.contains(
        "Approval linkage via `docs/project_management/next/cursor-cli-onboarding/governance/approved-agent.toml` (`sha256:"
    ));
    assert!(handoff.contains("## Approval provenance"));
    assert!(handoff.contains(
        "- approval ref: `docs/project_management/next/cursor-cli-onboarding/governance/approved-agent.toml`"
    ));
    assert!(handoff.contains("- approval artifact sha256: `"));
}

#[test]
fn onboard_agent_rejects_mixed_approval_and_descriptor_flags() {
    let fixture = fixture_root("onboard-agent-approval-mixed-flags");
    let approval_path = seed_approval_artifact(
        &fixture,
        "docs/project_management/next/cursor-cli-onboarding/governance/approved-agent.toml",
        "cursor",
        "cursor",
        None,
    );
    let mut args = base_args("cursor");
    args.extend(["--approval".to_string(), approval_path]);

    let output = run_cli(args, &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("--approval cannot be mixed with semantic descriptor flags"));
}

#[test]
fn onboard_agent_approval_requires_override_reason_for_nonrecommended_selection() {
    let fixture = fixture_root("onboard-agent-approval-override-required");
    let approval_path = seed_approval_artifact(
        &fixture,
        "docs/project_management/next/cursor-cli-onboarding/governance/approved-agent.toml",
        "codex",
        "cursor",
        None,
    );

    let output = run_cli(approval_args("--dry-run", &approval_path), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("override_reason"));
}

#[test]
fn onboard_agent_approval_rejects_paths_outside_governance_roots() {
    let fixture = fixture_root("onboard-agent-approval-invalid-path");
    let invalid_path = "docs/project_management/next/cursor-cli-onboarding/approved-agent.toml";
    seed_approval_artifact(&fixture, invalid_path, "cursor", "cursor", None);

    let output = run_cli(approval_args("--dry-run", invalid_path), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("must be repo-relative and rooted under"));
}

#[test]
fn onboard_agent_write_rejects_divergent_replay_without_mutating_other_files() {
    let fixture = fixture_root("onboard-agent-divergent-replay");
    seed_release_touchpoints(&fixture);

    let first = run_cli(write_args("cursor"), &fixture);
    assert_eq!(first.exit_code, 0, "stderr:\n{}", first.stderr);

    write_text(
        &fixture.join("cli_manifests/cursor/current.json"),
        "{\n  \"tampered\": true\n}\n",
    );
    let before = snapshot_files(&fixture);
    let second = run_cli(write_args("cursor"), &fixture);
    let after = snapshot_files(&fixture);

    assert_eq!(second.exit_code, 2);
    assert!(second.stderr.contains("cli_manifests/cursor/current.json"));
    assert!(second.stderr.contains("divergent"));
    assert_eq!(
        before, after,
        "divergent replay must not leave partial writes"
    );
}

#[test]
fn onboard_agent_write_allows_preexisting_runtime_owned_directories() {
    let fixture = fixture_root("onboard-agent-preexisting-runtime-dirs");
    seed_release_touchpoints(&fixture);
    fs::create_dir_all(fixture.join("crates/cursor")).expect("create runtime crate dir");
    fs::create_dir_all(fixture.join("crates/agent_api/src/backends/cursor"))
        .expect("create backend module dir");

    let output = run_cli(write_args("cursor"), &fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert!(output.stdout.contains("OK: onboard-agent write complete."));
}

#[test]
fn onboard_agent_closeout_packet_replays_identically_without_rewriting_manual_metrics() {
    let fixture = fixture_root("onboard-agent-closeout");
    seed_release_touchpoints(&fixture);
    let approval_rel =
        "docs/project_management/next/cursor-cli-onboarding/governance/approved-agent.toml";
    let approval_path = seed_approval_artifact(&fixture, approval_rel, "cursor", "cursor", None);
    let closeout_path = fixture.join(
        "docs/project_management/next/cursor-cli-onboarding/governance/proving-run-closeout.json",
    );
    let approval_sha256 = sha256_hex(&fixture.join(&approval_path));
    write_text(
        &closeout_path,
        &serde_json::to_string_pretty(&json!({
            "state": "closed",
            "approval_ref": approval_path,
            "approval_sha256": approval_sha256,
            "approval_source": "governance-review",
            "manual_control_plane_edits": 0,
            "partial_write_incidents": 0,
            "ambiguous_ownership_incidents": 0,
            "duration_missing_reason": "Exact duration not recoverable from committed evidence.",
            "preflight_passed": true,
            "residual_friction": [
                "Runtime-owned evidence capture still requires a local CLI install."
            ],
            "recorded_at": "2026-04-21T11:23:09Z",
            "commit": "deadbeef"
        }))
        .expect("serialize closeout"),
    );
    let metrics_path = fixture.join(
        "docs/project_management/next/cursor-cli-onboarding/governance/proving-run-metrics.json",
    );
    let metrics = concat!(
        "{\n",
        "  \"manual_control_plane_edits\": 0,\n",
        "  \"partial_write_incidents\": 0,\n",
        "  \"ambiguous_ownership_incidents\": 0,\n",
        "  \"control_plane_mutation_duration_seconds\": null,\n",
        "  \"control_plane_mutation_duration_recorded\": false,\n",
        "  \"control_plane_mutation_duration_note\": \"Exact duration not recoverable from committed evidence.\",\n",
        "  \"preflight_passed\": true,\n",
        "  \"residual_friction\": [\n",
        "    \"Runtime-owned evidence capture still requires a local CLI install.\"\n",
        "  ],\n",
        "  \"recorded_at\": \"2026-04-21T11:23:09Z\",\n",
        "  \"commit\": \"test-closeout-commit\"\n",
        "}\n"
    );
    write_text(&metrics_path, metrics);

    let metrics_before = fs::read(&metrics_path).expect("read initial metrics");
    let first = run_cli(write_args("cursor"), &fixture);
    let after_first = snapshot_files(&fixture);
    let metrics_after_first = fs::read(&metrics_path).expect("read metrics after first write");
    let second = run_cli(write_args("cursor"), &fixture);
    let after_second = snapshot_files(&fixture);
    let metrics_after_second = fs::read(&metrics_path).expect("read metrics after second write");

    assert_eq!(first.exit_code, 0, "stderr:\n{}", first.stderr);
    assert_eq!(second.exit_code, 0, "stderr:\n{}", second.stderr);
    assert!(first
        .stdout
        .contains("Mutation summary: 15 written, 0 identical, 15 total planned."));
    assert!(second
        .stdout
        .contains("Mutation summary: 0 written, 15 identical, 15 total planned."));
    assert_eq!(metrics_before, metrics_after_first);
    assert_eq!(metrics_before, metrics_after_second);
    assert_eq!(after_first, after_second);

    let handoff = fs::read_to_string(
        fixture.join("docs/project_management/next/cursor-cli-onboarding/HANDOFF.md"),
    )
    .expect("read closeout handoff");
    assert!(handoff.contains("This packet records the closed proving run for `cursor`."));
    assert!(handoff.contains("approval source: `governance-review`"));
    assert!(handoff.contains("manual control-plane file edits by maintainers: `0`"));
    assert!(handoff
        .contains("approved-agent to repo-ready control-plane mutation time: `missing (Exact duration not recoverable from committed evidence.)`"));
    assert!(handoff.contains("closeout metadata: `docs/project_management/next/cursor-cli-onboarding/governance/proving-run-closeout.json`"));
    assert!(handoff.contains("No open runtime next step remains in this packet."));

    let remediation =
        fs::read_to_string(fixture.join(
            "docs/project_management/next/cursor-cli-onboarding/governance/remediation-log.md",
        ))
        .expect("read remediation log");
    assert!(
        remediation.contains("Runtime-owned evidence capture still requires a local CLI install.")
    );
    assert!(remediation.contains(
        "Duration missing reason: Exact duration not recoverable from committed evidence."
    ));
}

#[cfg(unix)]
#[test]
fn onboard_agent_write_rejects_symlink_escape_paths() {
    use std::os::unix::fs::symlink;

    let fixture = fixture_root("onboard-agent-symlink-escape");
    seed_release_touchpoints(&fixture);
    let outside = fixture_root("onboard-agent-symlink-outside");
    fs::create_dir_all(fixture.join("docs/project_management/next")).expect("create docs parent");
    symlink(
        &outside,
        fixture.join("docs/project_management/next/linked"),
    )
    .expect("create symlink");

    let output = run_cli(
        args_with_overrides(
            "--write",
            "cursor",
            "unified-agent-api-cursor",
            &[("--onboarding-pack-prefix", "linked/escape-pack")],
            false,
        ),
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("symlinked component"));
    assert!(output
        .stderr
        .contains("docs/project_management/next/linked/escape-pack"));
}

fn run_cli<I, S>(argv: I, workspace_root: &Path) -> HarnessOutput
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = argv
        .into_iter()
        .map(|arg| arg.as_ref().to_string())
        .collect::<Vec<_>>();

    match Cli::try_parse_from(args) {
        Ok(cli) => {
            let mut stdout = Vec::new();
            let mut stderr = String::new();
            let exit_code = match cli.command {
                Command::OnboardAgent(args) => {
                    match onboard_agent::run_in_workspace(workspace_root, args, &mut stdout) {
                        Ok(()) => 0,
                        Err(err) => {
                            stderr = format!("{err}\n");
                            err.exit_code()
                        }
                    }
                }
            };
            HarnessOutput {
                exit_code,
                stdout: String::from_utf8(stdout).expect("stdout must be utf-8"),
                stderr,
            }
        }
        Err(err) => HarnessOutput {
            exit_code: err.exit_code(),
            stdout: String::new(),
            stderr: err.to_string(),
        },
    }
}
