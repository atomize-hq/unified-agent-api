use std::{fs, path::Path};

use super::{
    harness::{
        approval_args, assert_sections_in_order, base_args, fixture_root, seed_approval_artifact,
        seed_approval_artifact_with_pack_prefix, seed_release_touchpoints, snapshot_files,
        write_text,
    },
    run_cli,
};

const ENROLLED_MAINTENANCE: &str = concat!(
    "\n",
    "[descriptor.maintenance]\n",
    "mode = \"release_watch_enrolled\"\n",
    "\n",
    "[descriptor.maintenance.release_watch]\n",
    "enabled = true\n",
    "version_policy = \"latest_stable_minus_one\"\n",
    "dispatch_kind = \"workflow_dispatch\"\n",
    "dispatch_workflow = \"agent-maintenance-release-watch.yml\"\n",
    "\n",
    "[descriptor.maintenance.release_watch.upstream]\n",
    "source_kind = \"github_releases\"\n",
    "owner = \"atomize-hq\"\n",
    "repo = \"cursor\"\n",
    "tag_prefix = \"v\"\n",
);

fn deferred_maintenance(reason: &str, follow_up: &str) -> String {
    format!(
        concat!(
            "\n",
            "[descriptor.maintenance]\n",
            "mode = \"explicitly_deferred\"\n",
            "\n",
            "[descriptor.maintenance.deferral]\n",
            "reason = \"{reason}\"\n",
            "follow_up = \"{follow_up}\"\n",
            "approved_scope = \"create_lane_closeout\"\n",
        ),
        reason = reason,
        follow_up = follow_up,
    )
}

fn append_approval_maintenance(root: &Path, approval_path: &str, maintenance: &str) {
    let approval_file = root.join(approval_path);
    let existing = fs::read_to_string(&approval_file).expect("read approval artifact");
    write_text(&approval_file, &(existing + maintenance));
}

fn seed_enrolled_approval_artifact(
    root: &Path,
    relative_path: &str,
    recommended_agent_id: &str,
    approved_agent_id: &str,
    override_reason: Option<&str>,
) -> String {
    let approval_path = seed_approval_artifact(
        root,
        relative_path,
        recommended_agent_id,
        approved_agent_id,
        override_reason,
    );
    append_approval_maintenance(root, &approval_path, ENROLLED_MAINTENANCE);
    approval_path
}

fn seed_enrolled_approval_artifact_with_pack_prefix(
    root: &Path,
    relative_path: &str,
    recommended_agent_id: &str,
    approved_agent_id: &str,
    override_reason: Option<&str>,
    onboarding_pack_prefix: &str,
) -> String {
    let approval_path = seed_approval_artifact_with_pack_prefix(
        root,
        relative_path,
        recommended_agent_id,
        approved_agent_id,
        override_reason,
        onboarding_pack_prefix,
    );
    append_approval_maintenance(root, &approval_path, ENROLLED_MAINTENANCE);
    approval_path
}

fn seed_deferred_approval_artifact(
    root: &Path,
    relative_path: &str,
    reason: &str,
    follow_up: &str,
) -> String {
    let approval_path = seed_approval_artifact(root, relative_path, "cursor", "cursor", None);
    append_approval_maintenance(
        root,
        &approval_path,
        &deferred_maintenance(reason, follow_up),
    );
    approval_path
}

fn preview_body(stdout: &str) -> &str {
    let start = stdout
        .find("== INPUT SUMMARY ==")
        .expect("missing input summary section");
    let end = stdout.find("== RESULT ==").expect("missing result section");
    &stdout[start..end]
}

#[test]
fn onboard_agent_approval_dry_run_matches_raw_descriptor_preview_and_writes_nothing() {
    let fixture = fixture_root("onboard-agent-approval-preview");
    seed_release_touchpoints(&fixture);
    let approval_path = seed_enrolled_approval_artifact(
        &fixture,
        "docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
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
        .contains("approval_artifact_path: docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml"));
    assert!(approval.stdout.contains("approval_artifact_sha256: "));
    assert!(approval
        .stdout
        .contains("approval_maintenance_mode: release_watch_enrolled"));
    assert!(approval
        .stdout
        .contains("approval_maintenance_section_sha256: "));
    assert!(approval.stdout.contains("approval_release_watch_sha256: "));
    assert!(approval
        .stdout
        .contains("approval_release_watch_dispatch_workflow: agent-maintenance-release-watch.yml"));
    assert!(approval.stdout.contains(
        "Approval maintenance is already enrolled for release-watch handling (workflow_dispatch via `agent-maintenance-release-watch.yml`); keep that truth intact in downstream lanes."
    ));
    assert!(approval.stdout.contains(
        "Approval linkage: `docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml` (`sha256:"
    ));
    assert!(approval.stdout.contains(
        "Approval linkage via `docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml` (`sha256:"
    ));
    assert!(approval.stdout.contains("## Approval provenance"));
    assert!(approval.stdout.contains(
        "- approval ref: `docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml`"
    ));
    assert!(!raw.stdout.contains("approval_artifact_path:"));
    assert!(!raw.stdout.contains("approval_artifact_sha256:"));
    assert!(!raw.stdout.contains("## Approval provenance"));
    assert!(!raw.stdout.contains(
        "Approval linkage: `docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml` (`sha256:"
    ));
}

#[test]
fn onboard_agent_approval_dry_run_accepts_staged_path() {
    let fixture = fixture_root("onboard-agent-approval-staged-preview");
    seed_release_touchpoints(&fixture);
    let approval_path = seed_enrolled_approval_artifact(
        &fixture,
        "docs/agents/lifecycle/.staging/20260427-cursor/cursor-cli-onboarding/governance/approved-agent.toml",
        "cursor",
        "cursor",
        None,
    );

    let output = run_cli(approval_args("--dry-run", &approval_path), &fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert!(output
        .stdout
        .contains("approval_artifact_path: docs/agents/lifecycle/.staging/20260427-cursor/cursor-cli-onboarding/governance/approved-agent.toml"));
    assert!(output
        .stdout
        .contains("OK: onboard-agent dry-run preview complete."));
}

#[test]
fn onboard_agent_approval_write_applies_plan_and_replays_identically() {
    let fixture = fixture_root("onboard-agent-approval-write");
    seed_release_touchpoints(&fixture);
    let approval_path = seed_enrolled_approval_artifact(
        &fixture,
        "docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
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
    assert!(first.stdout.contains("approval_artifact_path: docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml"));
    assert!(first
        .stdout
        .contains("approval_maintenance_mode: release_watch_enrolled"));
    assert!(first.stdout.contains("## Approval provenance"));
    assert!(second
        .stdout
        .contains("Mutation summary: 0 written, 16 identical, 16 total planned."));

    let readme =
        fs::read_to_string(fixture.join("docs/agents/lifecycle/cursor-cli-onboarding/README.md"))
            .expect("read approval-mode readme");
    let scope_brief = fs::read_to_string(
        fixture.join("docs/agents/lifecycle/cursor-cli-onboarding/scope_brief.md"),
    )
    .expect("read approval-mode scope brief");
    let handoff =
        fs::read_to_string(fixture.join("docs/agents/lifecycle/cursor-cli-onboarding/HANDOFF.md"))
            .expect("read approval-mode handoff");

    assert!(readme.contains(
        "Approval linkage: `docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml` (`sha256:"
    ));
    assert!(scope_brief.contains(
        "Approval linkage via `docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml` (`sha256:"
    ));
    assert!(handoff.contains("## Approval provenance"));
    assert!(handoff.contains(
        "- approval ref: `docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml`"
    ));
    assert!(handoff.contains("- approval artifact sha256: `"));
    let lifecycle_state = fs::read_to_string(
        fixture.join("docs/agents/lifecycle/cursor-cli-onboarding/governance/lifecycle-state.json"),
    )
    .expect("read approval-mode lifecycle state");
    assert!(lifecycle_state.contains(
        "\"approval_artifact_path\": \"docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml\""
    ));
    assert!(lifecycle_state.contains("\"lifecycle_stage\": \"enrolled\""));
}

#[test]
fn onboard_agent_approval_dry_run_and_write_share_the_same_plan_preview() {
    let fixture = fixture_root("onboard-agent-approval-preview-write-parity");
    seed_release_touchpoints(&fixture);
    let approval_path = seed_enrolled_approval_artifact(
        &fixture,
        "docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
        "cursor",
        "cursor",
        None,
    );

    let dry_run = run_cli(approval_args("--dry-run", &approval_path), &fixture);
    let write = run_cli(approval_args("--write", &approval_path), &fixture);

    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    assert_eq!(write.exit_code, 0, "stderr:\n{}", write.stderr);
    assert_eq!(
        preview_body(&dry_run.stdout),
        preview_body(&write.stdout),
        "approval-mode dry-run and write should render the same planned preview bytes"
    );

    let sections = [
        "== INPUT SUMMARY ==",
        "== REGISTRY ENTRY PREVIEW ==",
        "== DOCS SCAFFOLD PREVIEW ==",
        "== LIFECYCLE STATE PREVIEW ==",
        "== MANIFEST ROOT PREVIEW ==",
        "== RELEASE/PUBLICATION TOUCHPOINTS ==",
        "== MANUAL FOLLOW-UP ==",
    ];
    assert_sections_in_order(preview_body(&dry_run.stdout), &sections);
    assert_sections_in_order(preview_body(&write.stdout), &sections);
}

#[test]
fn onboard_agent_approval_write_rejects_staged_path() {
    let fixture = fixture_root("onboard-agent-approval-staged-write-reject");
    seed_release_touchpoints(&fixture);
    let approval_path = seed_enrolled_approval_artifact(
        &fixture,
        "docs/agents/lifecycle/.staging/20260427-cursor/cursor-cli-onboarding/governance/approved-agent.toml",
        "cursor",
        "cursor",
        None,
    );

    let output = run_cli(approval_args("--write", &approval_path), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("staged approval paths under `docs/agents/lifecycle/.staging/**` are only allowed with `--dry-run`"));
}

#[test]
fn onboard_agent_rejects_mixed_approval_and_descriptor_flags() {
    let fixture = fixture_root("onboard-agent-approval-mixed-flags");
    let approval_path = seed_enrolled_approval_artifact(
        &fixture,
        "docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
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
    let approval_path = seed_enrolled_approval_artifact(
        &fixture,
        "docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
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
    let invalid_path = "docs/agents/lifecycle/cursor-cli-onboarding/approved-agent.toml";
    let approval_path = seed_approval_artifact(&fixture, invalid_path, "cursor", "cursor", None);
    append_approval_maintenance(&fixture, &approval_path, ENROLLED_MAINTENANCE);

    let output = run_cli(approval_args("--dry-run", invalid_path), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("must be repo-relative and match"));
}

#[test]
fn onboard_agent_approval_rejects_pack_prefix_mismatch() {
    let fixture = fixture_root("onboard-agent-approval-pack-prefix-mismatch");
    let approval_path = seed_enrolled_approval_artifact_with_pack_prefix(
        &fixture,
        "docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
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
    let approval_path = seed_enrolled_approval_artifact(
        &fixture,
        "docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
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
    let approval_path = seed_enrolled_approval_artifact(
        &fixture,
        "docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
        "cursor",
        "cursor",
        None,
    );
    fs::remove_file(fixture.join("docs/agents/selection/cli-agent-selection-packet.md"))
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
    let approval_path = seed_enrolled_approval_artifact(
        &fixture,
        "docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
        "cursor",
        "cursor",
        None,
    );
    let comparison_path = fixture.join("docs/agents/selection/cli-agent-selection-packet.md");
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
    let approval_path = seed_enrolled_approval_artifact(
        &fixture,
        "docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
        "cursor",
        "cursor",
        None,
    );
    let approval_file = fixture.join(&approval_path);
    let contents = fs::read_to_string(&approval_file).expect("read approval");
    super::harness::write_text(
        &approval_file,
        &contents.replace(
            "docs/agents/selection/cli-agent-selection-packet.md",
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
    let approval_path = seed_enrolled_approval_artifact(
        &fixture,
        "docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
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
    let approval_path = seed_enrolled_approval_artifact(
        &fixture,
        "docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
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

#[test]
fn onboard_agent_approval_dry_run_surfaces_deferred_maintenance_without_enrollment() {
    let fixture = fixture_root("onboard-agent-approval-deferred-maintenance");
    seed_release_touchpoints(&fixture);
    let approval_path = seed_deferred_approval_artifact(
        &fixture,
        "docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
        "release watch stays manual until the proving run closes",
        "revisit maintenance enrollment during lane closeout",
    );

    let output = run_cli(approval_args("--dry-run", &approval_path), &fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert!(output
        .stdout
        .contains("approval_maintenance_mode: explicitly_deferred"));
    assert!(output
        .stdout
        .contains("approval_maintenance_deferral_reason: release watch stays manual until the proving run closes"));
    assert!(output
        .stdout
        .contains("approval_maintenance_deferral_follow_up: revisit maintenance enrollment during lane closeout"));
    assert!(output
        .stdout
        .contains("approval_maintenance_deferral_scope: create_lane_closeout"));
    assert!(output
        .stdout
        .contains("approval_maintenance_deferral_sha256: "));
    assert!(output.stdout.contains(
        "Approval maintenance is explicitly deferred for this onboarding packet; do not write release-watch enrollment from this lane. Reason: release watch stays manual until the proving run closes"
    ));
    assert!(output.stdout.contains(
        "Deferred maintenance follow-up remains manual until closeout: revisit maintenance enrollment during lane closeout"
    ));
    assert!(!output.stdout.contains("approval_release_watch_sha256:"));
}
