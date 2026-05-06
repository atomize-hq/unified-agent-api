use std::fs;

use serde_json::json;

use super::{
    harness::{
        args_with_overrides, assert_sections_in_order, fixture_root, seed_approval_artifact,
        seed_release_touchpoints, sha256_hex, snapshot_files, write_args, write_text,
    },
    run_cli,
};

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
        "== LIFECYCLE STATE PREVIEW ==",
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
        .contains("Mutation summary: 16 written, 0 identical, 16 total planned."));
    assert!(second.stdout.contains("OK: onboard-agent write complete."));
    assert!(second
        .stdout
        .contains("Mutation summary: 0 written, 16 identical, 16 total planned."));

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

    let readme =
        fs::read_to_string(fixture.join("docs/agents/lifecycle/cursor-cli-onboarding/README.md"))
            .expect("read docs README");
    assert!(readme.contains("# Cursor CLI onboarding pack"));
    let handoff =
        fs::read_to_string(fixture.join("docs/agents/lifecycle/cursor-cli-onboarding/HANDOFF.md"))
            .expect("read handoff");
    assert!(handoff.contains("runtime-follow-on --dry-run"));
    assert!(handoff.contains(
        "Populate committed runtime evidence only under `cli_manifests/cursor/snapshots/**` and `cli_manifests/cursor/supplement/**`."
    ));
    assert!(!handoff.contains("current.json`, pointers, versions, and reports"));
    assert!(!handoff.contains("Regenerate support and capability publication artifacts"));

    let current_json = fs::read_to_string(fixture.join("cli_manifests/cursor/current.json"))
        .expect("read current");
    assert!(current_json.contains("\"expected_targets\": ["));
    let lifecycle_state = fs::read_to_string(
        fixture.join("docs/agents/lifecycle/cursor-cli-onboarding/governance/lifecycle-state.json"),
    )
    .expect("read lifecycle state");
    assert!(lifecycle_state.contains("\"lifecycle_stage\": \"enrolled\""));
    assert!(lifecycle_state.contains("\"support_tier\": \"bootstrap\""));
    assert!(lifecycle_state.contains("\"current_owner_command\": \"onboard-agent --write\""));
    assert!(lifecycle_state
        .contains("\"expected_next_command\": \"scaffold-wrapper-crate --agent cursor --write\""));
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
fn onboard_agent_write_rejects_divergent_lifecycle_state_without_mutating_other_files() {
    let fixture = fixture_root("onboard-agent-divergent-lifecycle-state");
    seed_release_touchpoints(&fixture);

    let first = run_cli(write_args("cursor"), &fixture);
    assert_eq!(first.exit_code, 0, "stderr:\n{}", first.stderr);

    write_text(
        &fixture
            .join("docs/agents/lifecycle/cursor-cli-onboarding/governance/lifecycle-state.json"),
        "{\n  \"tampered\": true\n}\n",
    );
    let before = snapshot_files(&fixture);
    let second = run_cli(write_args("cursor"), &fixture);
    let after = snapshot_files(&fixture);

    assert_eq!(second.exit_code, 2);
    assert!(second
        .stderr
        .contains("docs/agents/lifecycle/cursor-cli-onboarding/governance/lifecycle-state.json"));
    assert!(second.stderr.contains("divergent"));
    assert_eq!(
        before, after,
        "divergent lifecycle replay must not leave partial writes"
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
    let approval_rel = "docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml";
    let approval_path = seed_approval_artifact(&fixture, approval_rel, "cursor", "cursor", None);
    let closeout_path = fixture
        .join("docs/agents/lifecycle/cursor-cli-onboarding/governance/proving-run-closeout.json");
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
    let metrics_path = fixture
        .join("docs/agents/lifecycle/cursor-cli-onboarding/governance/proving-run-metrics.json");
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
        .contains("Mutation summary: 16 written, 0 identical, 16 total planned."));
    assert!(second
        .stdout
        .contains("Mutation summary: 0 written, 16 identical, 16 total planned."));
    assert_eq!(metrics_before, metrics_after_first);
    assert_eq!(metrics_before, metrics_after_second);
    assert_eq!(after_first, after_second);

    let handoff =
        fs::read_to_string(fixture.join("docs/agents/lifecycle/cursor-cli-onboarding/HANDOFF.md"))
            .expect("read closeout handoff");
    assert!(handoff.contains("This packet records the closed proving run for `cursor`."));
    assert!(handoff.contains("approval source: `governance-review`"));
    assert!(handoff.contains("manual control-plane file edits by maintainers: `0`"));
    assert!(handoff
        .contains("approved-agent to repo-ready control-plane mutation time: `missing (Exact duration not recoverable from committed evidence.)`"));
    assert!(handoff.contains("closeout metadata: `docs/agents/lifecycle/cursor-cli-onboarding/governance/proving-run-closeout.json`"));
    assert!(handoff.contains("No open runtime next step remains in this packet."));

    let remediation = fs::read_to_string(
        fixture.join("docs/agents/lifecycle/cursor-cli-onboarding/governance/remediation-log.md"),
    )
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
    fs::create_dir_all(fixture.join("docs/agents/lifecycle")).expect("create docs parent");
    symlink(&outside, fixture.join("docs/agents/lifecycle/linked")).expect("create symlink");

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
        .contains("docs/agents/lifecycle/linked/escape-pack"));
}
