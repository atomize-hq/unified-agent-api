use std::fs;

use super::{
    harness::{
        approval_args, base_args, fixture_root, seed_approval_artifact,
        seed_approval_artifact_with_pack_prefix, seed_release_touchpoints, snapshot_files,
    },
    run_cli,
};

#[test]
fn onboard_agent_approval_dry_run_matches_raw_descriptor_preview_and_writes_nothing() {
    let fixture = fixture_root("onboard-agent-approval-preview");
    seed_release_touchpoints(&fixture);
    let approval_path = seed_approval_artifact(
        &fixture,
        "docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
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
        .contains("approval_artifact_path: docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml"));
    assert!(approval.stdout.contains("approval_artifact_sha256: "));
    assert!(approval.stdout.contains(
        "Approval linkage: `docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml` (`sha256:"
    ));
    assert!(approval.stdout.contains(
        "Approval linkage via `docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml` (`sha256:"
    ));
    assert!(approval.stdout.contains("## Approval provenance"));
    assert!(approval.stdout.contains(
        "- approval ref: `docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml`"
    ));
    assert!(!raw.stdout.contains("approval_artifact_path:"));
    assert!(!raw.stdout.contains("approval_artifact_sha256:"));
    assert!(!raw.stdout.contains("## Approval provenance"));
    assert!(!raw.stdout.contains(
        "Approval linkage: `docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml` (`sha256:"
    ));
}

#[test]
fn onboard_agent_approval_write_applies_plan_and_replays_identically() {
    let fixture = fixture_root("onboard-agent-approval-write");
    seed_release_touchpoints(&fixture);
    let approval_path = seed_approval_artifact(
        &fixture,
        "docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
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
    assert!(first.stdout.contains("approval_artifact_path: docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml"));
    assert!(first.stdout.contains("## Approval provenance"));
    assert!(second
        .stdout
        .contains("Mutation summary: 0 written, 15 identical, 15 total planned."));

    let readme = fs::read_to_string(
        fixture.join("docs/reports/agent-lifecycle/cursor-cli-onboarding/README.md"),
    )
    .expect("read approval-mode readme");
    let scope_brief = fs::read_to_string(
        fixture.join("docs/reports/agent-lifecycle/cursor-cli-onboarding/scope_brief.md"),
    )
    .expect("read approval-mode scope brief");
    let handoff = fs::read_to_string(
        fixture.join("docs/reports/agent-lifecycle/cursor-cli-onboarding/HANDOFF.md"),
    )
    .expect("read approval-mode handoff");

    assert!(readme.contains(
        "Approval linkage: `docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml` (`sha256:"
    ));
    assert!(scope_brief.contains(
        "Approval linkage via `docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml` (`sha256:"
    ));
    assert!(handoff.contains("## Approval provenance"));
    assert!(handoff.contains(
        "- approval ref: `docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml`"
    ));
    assert!(handoff.contains("- approval artifact sha256: `"));
}

#[test]
fn onboard_agent_rejects_mixed_approval_and_descriptor_flags() {
    let fixture = fixture_root("onboard-agent-approval-mixed-flags");
    let approval_path = seed_approval_artifact(
        &fixture,
        "docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
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
        "docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
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
    let invalid_path = "docs/reports/agent-lifecycle/cursor-cli-onboarding/approved-agent.toml";
    seed_approval_artifact(&fixture, invalid_path, "cursor", "cursor", None);

    let output = run_cli(approval_args("--dry-run", invalid_path), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("must be repo-relative and match"));
}

#[test]
fn onboard_agent_approval_rejects_pack_prefix_mismatch() {
    let fixture = fixture_root("onboard-agent-approval-pack-prefix-mismatch");
    let approval_path = seed_approval_artifact_with_pack_prefix(
        &fixture,
        "docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
        "cursor",
        "cursor",
        None,
        "other-cursor-pack",
    );

    let output = run_cli(approval_args("--dry-run", &approval_path), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("belongs to onboarding_pack_prefix"));
}

#[test]
fn onboard_agent_approval_rejects_unsupported_artifact_version() {
    let fixture = fixture_root("onboard-agent-approval-unsupported-version");
    let approval_path = seed_approval_artifact(
        &fixture,
        "docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
        "cursor",
        "cursor",
        None,
    );
    let approval_file = fixture.join(&approval_path);
    let contents = fs::read_to_string(&approval_file).expect("read approval");
    super::harness::write_text(
        &approval_file,
        &contents.replace("artifact_version = \"1\"", "artifact_version = \"2\""),
    );

    let output = run_cli(approval_args("--dry-run", &approval_path), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("unsupported `artifact_version`"));
}

#[test]
fn onboard_agent_approval_rejects_nonexistent_comparison_ref() {
    let fixture = fixture_root("onboard-agent-approval-missing-comparison-ref");
    let approval_path = seed_approval_artifact(
        &fixture,
        "docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
        "cursor",
        "cursor",
        None,
    );
    fs::remove_file(
        fixture.join("docs/reports/verification/cli-agent-selection/cli-agent-selection-packet.md"),
    )
    .expect("remove canonical comparison packet");

    let output = run_cli(approval_args("--dry-run", &approval_path), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("field `comparison_ref` does not resolve"));
}

#[test]
fn onboard_agent_approval_rejects_non_file_comparison_ref() {
    let fixture = fixture_root("onboard-agent-approval-directory-comparison-ref");
    let approval_path = seed_approval_artifact(
        &fixture,
        "docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
        "cursor",
        "cursor",
        None,
    );
    let comparison_path =
        fixture.join("docs/reports/verification/cli-agent-selection/cli-agent-selection-packet.md");
    fs::remove_file(&comparison_path).expect("remove comparison packet file");
    fs::create_dir_all(&comparison_path).expect("create comparison dir");

    let output = run_cli(approval_args("--dry-run", &approval_path), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("field `comparison_ref` must point to an existing file"));
}

#[test]
fn onboard_agent_approval_rejects_non_normal_comparison_ref() {
    let fixture = fixture_root("onboard-agent-approval-nonnormal-comparison-ref");
    let approval_path = seed_approval_artifact(
        &fixture,
        "docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
        "cursor",
        "cursor",
        None,
    );
    let approval_file = fixture.join(&approval_path);
    let contents = fs::read_to_string(&approval_file).expect("read approval");
    super::harness::write_text(
        &approval_file,
        &contents.replace(
            "docs/reports/verification/cli-agent-selection/cli-agent-selection-packet.md",
            "../outside.md",
        ),
    );

    let output = run_cli(approval_args("--dry-run", &approval_path), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("field `comparison_ref` must equal"));
}

#[test]
fn onboard_agent_approval_rejects_invalid_approval_commit() {
    let fixture = fixture_root("onboard-agent-approval-invalid-commit");
    let approval_path = seed_approval_artifact(
        &fixture,
        "docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
        "cursor",
        "cursor",
        None,
    );
    let approval_file = fixture.join(&approval_path);
    let contents = fs::read_to_string(&approval_file).expect("read approval");
    super::harness::write_text(
        &approval_file,
        &contents.replace(
            "approval_commit = \"deadbeef\"",
            "approval_commit = \"test-approval-commit\"",
        ),
    );

    let output = run_cli(approval_args("--dry-run", &approval_path), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("`approval_commit` must be 7-40 lowercase hex characters"));
}

#[test]
fn onboard_agent_approval_rejects_invalid_approval_recorded_at() {
    let fixture = fixture_root("onboard-agent-approval-invalid-recorded-at");
    let approval_path = seed_approval_artifact(
        &fixture,
        "docs/reports/agent-lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
        "cursor",
        "cursor",
        None,
    );
    let approval_file = fixture.join(&approval_path);
    let contents = fs::read_to_string(&approval_file).expect("read approval");
    super::harness::write_text(
        &approval_file,
        &contents.replace(
            "approval_recorded_at = \"2026-04-21T11:23:09Z\"",
            "approval_recorded_at = \"not-a-timestamp\"",
        ),
    );

    let output = run_cli(approval_args("--dry-run", &approval_path), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("`approval_recorded_at` must be RFC3339"));
}
