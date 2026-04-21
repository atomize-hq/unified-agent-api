use std::{fs, path::Path};

use clap::{Parser, Subcommand};
use serde_json::json;
use xtask::onboard_agent;

#[allow(dead_code)]
#[path = "../src/close_proving_run.rs"]
mod close_proving_run;

#[path = "support/onboard_agent_harness.rs"]
mod harness;

use harness::{
    fixture_root, gemini_dry_run_args, repo_root, seed_release_touchpoints, sha256_hex, write_text,
    HarnessOutput,
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
    CloseProvingRun(close_proving_run::Args),
    OnboardAgent(Box<onboard_agent::Args>),
}

#[test]
fn committed_gemini_preview_renders_closed_packet_from_valid_m3_closeout() {
    let workspace_root = repo_root();
    let output = run_cli(gemini_dry_run_args(), &workspace_root);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert!(output
        .stdout
        .contains("Shared onboarding plan preview; no filesystem writes performed."));
    assert!(output
        .stdout
        .contains("This packet records the closed proving run for `Gemini CLI`."));
    assert!(output
        .stdout
        .contains("- Packet state: `closed_proving_run`"));
    assert!(output.stdout.contains(
        "Closeout metadata is recorded in `docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json`."
    ));
    assert!(output
        .stdout
        .contains("Approval linkage: `historical-m3-backfill` via"));
}

#[test]
fn legacy_metrics_alone_does_not_close_packet() {
    let fixture = fixture_root("onboard-agent-legacy-metrics");
    seed_release_touchpoints(&fixture);
    write_text(
        &fixture.join(
            "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-metrics.json",
        ),
        &serde_json::to_string_pretty(&json!({
            "manual_control_plane_edits": 0,
            "partial_write_incidents": 0,
            "ambiguous_ownership_incidents": 0,
            "control_plane_mutation_duration_seconds": 7,
            "control_plane_mutation_duration_recorded": true,
            "preflight_passed": true,
            "residual_friction": [],
            "recorded_at": "2026-04-21T11:23:09Z",
            "commit": "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8"
        }))
        .expect("serialize metrics"),
    );

    let output = run_cli(gemini_dry_run_args(), &fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert!(output
        .stdout
        .contains("This packet captures the next executable onboarding step for `gemini_cli`."));
    assert!(output.stdout.contains("- Packet state: `execution`"));
    assert!(!output.stdout.contains("closed_proving_run"));
    assert!(!output.stdout.contains("Approval linkage:"));
}

#[test]
fn onboard_agent_dry_run_fails_closed_when_closeout_json_is_malformed() {
    let fixture = fixture_root("onboard-agent-malformed-closeout");
    seed_release_touchpoints(&fixture);
    write_text(
        &fixture.join(
            "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json",
        ),
        "{ not valid json }\n",
    );

    let output = run_cli(gemini_dry_run_args(), &fixture);

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output.stderr.contains(
        "parse docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json"
    ));
    assert!(!output
        .stdout
        .contains("This packet captures the next executable onboarding step for `gemini_cli`."));
    assert!(!output.stdout.contains("- Packet state: `execution`"));
}

#[test]
fn onboard_agent_dry_run_rejects_invalid_closeout_truth_instead_of_falling_back_to_execution() {
    let fixture = fixture_root("onboard-agent-invalid-closeout");
    seed_release_touchpoints(&fixture);
    let approval_rel =
        "docs/project_management/next/gemini-cli-onboarding/governance/approved-agent.toml";
    seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    write_text(
        &fixture.join(
            "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json",
        ),
        &serde_json::to_string_pretty(&json!({
            "state": "closed",
            "approval_ref": approval_rel,
            "approval_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
            "approval_source": "governance-review",
            "manual_control_plane_edits": 0,
            "partial_write_incidents": 0,
            "ambiguous_ownership_incidents": 0,
            "duration_seconds": 17,
            "explicit_none_reason": "No residual friction remained.",
            "preflight_passed": true,
            "recorded_at": "2026-04-21T11:23:09Z",
            "commit": "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8"
        }))
        .expect("serialize closeout"),
    );

    let output = run_cli(gemini_dry_run_args(), &fixture);

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output.stderr.contains("approval_sha256 does not match"));
    assert!(!output
        .stdout
        .contains("This packet captures the next executable onboarding step for `gemini_cli`."));
    assert!(!output.stdout.contains("- Packet state: `execution`"));
}

#[test]
fn onboard_agent_write_does_not_rewrite_packet_files_when_closeout_is_invalid() {
    let fixture = fixture_root("onboard-agent-invalid-closeout-write");
    seed_release_touchpoints(&fixture);
    let approval_rel =
        "docs/project_management/next/gemini-cli-onboarding/governance/approved-agent.toml";
    seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let readme_path = fixture.join("docs/project_management/next/gemini-cli-onboarding/README.md");
    let handoff_path =
        fixture.join("docs/project_management/next/gemini-cli-onboarding/HANDOFF.md");
    write_text(&readme_path, "existing readme\n");
    write_text(&handoff_path, "existing handoff\n");
    write_text(
        &fixture.join(
            "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json",
        ),
        &serde_json::to_string_pretty(&json!({
            "state": "closed",
            "approval_ref": approval_rel,
            "approval_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
            "approval_source": "governance-review",
            "manual_control_plane_edits": 0,
            "partial_write_incidents": 0,
            "ambiguous_ownership_incidents": 0,
            "duration_seconds": 17,
            "explicit_none_reason": "No residual friction remained.",
            "preflight_passed": true,
            "recorded_at": "2026-04-21T11:23:09Z",
            "commit": "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8"
        }))
        .expect("serialize closeout"),
    );

    let mut args = gemini_dry_run_args();
    let mode_index = args
        .iter()
        .position(|arg| arg == "--dry-run")
        .expect("dry-run arg present");
    args[mode_index] = "--write".to_string();

    let output = run_cli(args, &fixture);

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output.stderr.contains("approval_sha256 does not match"));
    assert_eq!(
        fs::read_to_string(&readme_path).expect("read readme"),
        "existing readme\n"
    );
    assert_eq!(
        fs::read_to_string(&handoff_path).expect("read handoff"),
        "existing handoff\n"
    );
    assert!(!fixture
        .join("docs/project_management/next/gemini-cli-onboarding/scope_brief.md")
        .exists());
}

#[test]
fn close_proving_run_validates_and_refreshes_packet_docs() {
    let fixture = fixture_root("close-proving-run-pass");
    seed_release_touchpoints(&fixture);
    let approval_rel =
        "docs/project_management/next/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path =
        seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let approval_sha256 = sha256_hex(&fixture.join(&approval_path));
    let closeout_path = fixture.join(
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json",
    );
    write_text(
        &closeout_path,
        &serde_json::to_string_pretty(&json!({
            "state": "closed",
            "approval_ref": approval_rel,
            "approval_sha256": approval_sha256,
            "approval_source": "governance-review",
            "manual_control_plane_edits": 0,
            "partial_write_incidents": 0,
            "ambiguous_ownership_incidents": 0,
            "duration_seconds": 17,
            "residual_friction": ["Manual review step still took coordination."],
            "preflight_passed": true,
            "recorded_at": "2026-04-21T11:23:09Z",
            "commit": "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8"
        }))
        .expect("serialize closeout"),
    );

    let closeout_output = run_cli(
        vec![
            "xtask".to_string(),
            "close-proving-run".to_string(),
            "--approval".to_string(),
            approval_path.clone(),
            "--closeout".to_string(),
            "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json"
                .to_string(),
        ],
        &fixture,
    );

    assert_eq!(
        closeout_output.exit_code, 0,
        "stderr:\n{}",
        closeout_output.stderr
    );
    assert!(closeout_output
        .stdout
        .contains("OK: close-proving-run write complete."));

    let readme = fs::read_to_string(
        fixture.join("docs/project_management/next/gemini-cli-onboarding/README.md"),
    )
    .expect("read refreshed readme");
    let handoff = fs::read_to_string(
        fixture.join("docs/project_management/next/gemini-cli-onboarding/HANDOFF.md"),
    )
    .expect("read refreshed handoff");

    assert!(readme.contains("- Packet state: `closed_proving_run`"));
    assert!(readme.contains("Approval linkage: `governance-review` via"));
    assert!(handoff.contains("- approval ref: `docs/project_management/next/gemini-cli-onboarding/governance/approved-agent.toml`"));
    assert!(handoff.contains("- closeout metadata: `docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json`"));

    let preview_output = run_cli(gemini_dry_run_args(), &fixture);
    assert_eq!(
        preview_output.exit_code, 0,
        "stderr:\n{}",
        preview_output.stderr
    );
    assert!(preview_output
        .stdout
        .contains("This packet records the closed proving run for `Gemini CLI`."));
    assert!(preview_output
        .stdout
        .contains("- Packet state: `closed_proving_run`"));
    assert!(preview_output
        .stdout
        .contains("Approval linkage: `governance-review` via"));
}

#[test]
fn close_proving_run_accepts_absolute_closeout_path_inside_workspace() {
    let fixture = fixture_root("close-proving-run-absolute-inside");
    seed_release_touchpoints(&fixture);
    let approval_rel =
        "docs/project_management/next/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path =
        seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let closeout_path = fixture.join(
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json",
    );
    let approval_sha256 = sha256_hex(&fixture.join(&approval_path));
    write_text(
        &closeout_path,
        &serde_json::to_string_pretty(&json!({
            "state": "closed",
            "approval_ref": approval_rel,
            "approval_sha256": approval_sha256,
            "approval_source": "governance-review",
            "manual_control_plane_edits": 0,
            "partial_write_incidents": 0,
            "ambiguous_ownership_incidents": 0,
            "duration_seconds": 17,
            "residual_friction": ["Manual review step still took coordination."],
            "preflight_passed": true,
            "recorded_at": "2026-04-21T11:23:09Z",
            "commit": "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8"
        }))
        .expect("serialize closeout"),
    );

    let output = run_cli(
        vec![
            "xtask".to_string(),
            "close-proving-run".to_string(),
            "--approval".to_string(),
            approval_path,
            "--closeout".to_string(),
            closeout_path.display().to_string(),
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert!(output
        .stdout
        .contains("OK: close-proving-run write complete."));
}

#[test]
fn close_proving_run_rejects_absolute_closeout_path_outside_workspace() {
    let fixture = fixture_root("close-proving-run-absolute-outside");
    seed_release_touchpoints(&fixture);
    let outside = fixture_root("close-proving-run-outside-file");
    let outside_closeout = outside.join("proving-run-closeout.json");
    write_text(&outside_closeout, "{}\n");

    let output = run_cli(
        vec![
            "xtask".to_string(),
            "close-proving-run".to_string(),
            "--approval".to_string(),
            "docs/project_management/next/gemini-cli-onboarding/governance/approved-agent.toml"
                .to_string(),
            "--closeout".to_string(),
            outside_closeout.display().to_string(),
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output.stderr.contains("must be inside workspace root"));
}

#[cfg(unix)]
#[test]
fn close_proving_run_rejects_symlinked_closeout_path() {
    use std::os::unix::fs::symlink;

    let fixture = fixture_root("close-proving-run-symlinked-closeout");
    seed_release_touchpoints(&fixture);
    let approval_rel =
        "docs/project_management/next/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path =
        seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let approval_sha256 = sha256_hex(&fixture.join(&approval_path));
    let outside = fixture_root("close-proving-run-symlinked-closeout-target");
    let outside_closeout = outside.join("proving-run-closeout.json");
    write_text(
        &outside_closeout,
        &serde_json::to_string_pretty(&json!({
            "state": "closed",
            "approval_ref": approval_rel,
            "approval_sha256": approval_sha256,
            "approval_source": "governance-review",
            "manual_control_plane_edits": 0,
            "partial_write_incidents": 0,
            "ambiguous_ownership_incidents": 0,
            "duration_seconds": 17,
            "residual_friction": ["Manual review step still took coordination."],
            "preflight_passed": true,
            "recorded_at": "2026-04-21T11:23:09Z",
            "commit": "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8"
        }))
        .expect("serialize closeout"),
    );
    let closeout_path = fixture.join(
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json",
    );
    if let Some(parent) = closeout_path.parent() {
        fs::create_dir_all(parent).expect("create governance dir");
    }
    symlink(&outside_closeout, &closeout_path).expect("create closeout symlink");

    let output = run_cli(
        vec![
            "xtask".to_string(),
            "close-proving-run".to_string(),
            "--approval".to_string(),
            approval_path,
            "--closeout".to_string(),
            "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json"
                .to_string(),
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output.stderr.contains("symlinked component"));
    assert!(!fixture
        .join("docs/project_management/next/gemini-cli-onboarding/README.md")
        .exists());
}

#[cfg(unix)]
#[test]
fn close_proving_run_rejects_symlinked_output_target_without_partial_refresh() {
    use std::os::unix::fs::symlink;

    let fixture = fixture_root("close-proving-run-symlinked-output");
    seed_release_touchpoints(&fixture);
    let approval_rel =
        "docs/project_management/next/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path =
        seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let approval_sha256 = sha256_hex(&fixture.join(&approval_path));
    let closeout_path = fixture.join(
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json",
    );
    write_text(
        &closeout_path,
        &serde_json::to_string_pretty(&json!({
            "state": "closed",
            "approval_ref": approval_rel,
            "approval_sha256": approval_sha256,
            "approval_source": "governance-review",
            "manual_control_plane_edits": 0,
            "partial_write_incidents": 0,
            "ambiguous_ownership_incidents": 0,
            "duration_seconds": 17,
            "residual_friction": ["Manual review step still took coordination."],
            "preflight_passed": true,
            "recorded_at": "2026-04-21T11:23:09Z",
            "commit": "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8"
        }))
        .expect("serialize closeout"),
    );

    let packet_root = fixture.join("docs/project_management/next/gemini-cli-onboarding");
    let readme_path = packet_root.join("README.md");
    write_text(&readme_path, "before refresh\n");
    let outside = fixture_root("close-proving-run-symlinked-output-target");
    let outside_target = outside.join("handoff.md");
    write_text(&outside_target, "outside target should not change\n");
    if let Some(parent) = packet_root.join("HANDOFF.md").parent() {
        fs::create_dir_all(parent).expect("create packet dir");
    }
    symlink(&outside_target, packet_root.join("HANDOFF.md")).expect("create handoff symlink");

    let output = run_cli(
        vec![
            "xtask".to_string(),
            "close-proving-run".to_string(),
            "--approval".to_string(),
            approval_path,
            "--closeout".to_string(),
            "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json"
                .to_string(),
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output.stderr.contains("symlinked component"));
    assert!(output.stderr.contains("HANDOFF.md"));
    assert_eq!(
        fs::read_to_string(&readme_path).expect("read readme"),
        "before refresh\n"
    );
    assert_eq!(
        fs::read_to_string(&outside_target).expect("read outside target"),
        "outside target should not change\n"
    );
}

#[test]
fn close_proving_run_rejects_noncanonical_approval_artifact_even_when_ref_and_sha_match() {
    let fixture = fixture_root("close-proving-run-noncanonical-approval");
    seed_release_touchpoints(&fixture);
    let approval_rel =
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-approval.md";
    let approval_path = fixture.join(approval_rel);
    write_text(&approval_path, "# not an approval artifact\n");
    let approval_sha256 = sha256_hex(&approval_path);
    let closeout_path = fixture.join(
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json",
    );
    write_text(
        &closeout_path,
        &serde_json::to_string_pretty(&json!({
            "state": "closed",
            "approval_ref": approval_rel,
            "approval_sha256": approval_sha256,
            "approval_source": "governance-review",
            "manual_control_plane_edits": 0,
            "partial_write_incidents": 0,
            "ambiguous_ownership_incidents": 0,
            "duration_seconds": 17,
            "residual_friction": ["Manual review step still took coordination."],
            "preflight_passed": true,
            "recorded_at": "2026-04-21T11:23:09Z",
            "commit": "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8"
        }))
        .expect("serialize closeout"),
    );

    let output = run_cli(
        vec![
            "xtask".to_string(),
            "close-proving-run".to_string(),
            "--approval".to_string(),
            approval_rel.to_string(),
            "--closeout".to_string(),
            "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json"
                .to_string(),
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output.stderr.contains("must be repo-relative and match"));
}

#[test]
fn close_proving_run_rejects_invalid_toml_at_canonical_approval_path() {
    let fixture = fixture_root("close-proving-run-invalid-approval-toml");
    seed_release_touchpoints(&fixture);
    let approval_rel =
        "docs/project_management/next/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path = fixture.join(approval_rel);
    write_text(&approval_path, "not = [valid toml\n");
    let approval_sha256 = sha256_hex(&approval_path);
    let closeout_path = fixture.join(
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json",
    );
    write_text(
        &closeout_path,
        &serde_json::to_string_pretty(&json!({
            "state": "closed",
            "approval_ref": approval_rel,
            "approval_sha256": approval_sha256,
            "approval_source": "governance-review",
            "manual_control_plane_edits": 0,
            "partial_write_incidents": 0,
            "ambiguous_ownership_incidents": 0,
            "duration_seconds": 17,
            "residual_friction": ["Manual review step still took coordination."],
            "preflight_passed": true,
            "recorded_at": "2026-04-21T11:23:09Z",
            "commit": "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8"
        }))
        .expect("serialize closeout"),
    );

    let output = run_cli(
        vec![
            "xtask".to_string(),
            "close-proving-run".to_string(),
            "--approval".to_string(),
            approval_rel.to_string(),
            "--closeout".to_string(),
            "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json"
                .to_string(),
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output.stderr.contains("parse approval artifact"));
}

#[test]
fn close_proving_run_rejects_schema_invalid_approval_artifact_at_canonical_path() {
    let fixture = fixture_root("close-proving-run-invalid-approval-schema");
    seed_release_touchpoints(&fixture);
    let approval_rel =
        "docs/project_management/next/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path = fixture.join(approval_rel);
    write_text(
        &approval_path,
        concat!(
            "artifact_version = \"1\"\n",
            "comparison_ref = \"docs/project_management/next/comparisons/gemini.md\"\n",
            "selection_mode = \"factory_validation\"\n",
            "recommended_agent_id = \"gemini_cli\"\n",
            "approved_agent_id = \"gemini_cli\"\n",
            "approval_commit = \"deadbeef\"\n",
            "approval_recorded_at = \"2026-04-21T11:23:09Z\"\n",
        ),
    );
    let approval_sha256 = sha256_hex(&approval_path);
    let closeout_path = fixture.join(
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json",
    );
    write_text(
        &closeout_path,
        &serde_json::to_string_pretty(&json!({
            "state": "closed",
            "approval_ref": approval_rel,
            "approval_sha256": approval_sha256,
            "approval_source": "governance-review",
            "manual_control_plane_edits": 0,
            "partial_write_incidents": 0,
            "ambiguous_ownership_incidents": 0,
            "duration_seconds": 17,
            "residual_friction": ["Manual review step still took coordination."],
            "preflight_passed": true,
            "recorded_at": "2026-04-21T11:23:09Z",
            "commit": "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8"
        }))
        .expect("serialize closeout"),
    );

    let output = run_cli(
        vec![
            "xtask".to_string(),
            "close-proving-run".to_string(),
            "--approval".to_string(),
            approval_rel.to_string(),
            "--closeout".to_string(),
            "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json"
                .to_string(),
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output
        .stderr
        .contains("missing required table `descriptor`"));
}

#[test]
fn close_proving_run_rejects_invalid_approval_metadata_from_shared_loader() {
    let fixture = fixture_root("close-proving-run-invalid-approval-metadata");
    seed_release_touchpoints(&fixture);
    let approval_rel =
        "docs/project_management/next/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path =
        seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let approval_file = fixture.join(&approval_path);
    let approval_contents = fs::read_to_string(&approval_file).expect("read approval");
    write_text(
        &approval_file,
        &approval_contents.replace(
            "approval_recorded_at = \"2026-04-21T11:23:09Z\"",
            "approval_recorded_at = \"not-a-timestamp\"",
        ),
    );
    let approval_sha256 = sha256_hex(&approval_file);
    let closeout_path = fixture.join(
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json",
    );
    write_text(
        &closeout_path,
        &serde_json::to_string_pretty(&json!({
            "state": "closed",
            "approval_ref": approval_rel,
            "approval_sha256": approval_sha256,
            "approval_source": "governance-review",
            "manual_control_plane_edits": 0,
            "partial_write_incidents": 0,
            "ambiguous_ownership_incidents": 0,
            "duration_seconds": 17,
            "residual_friction": ["Manual review step still took coordination."],
            "preflight_passed": true,
            "recorded_at": "2026-04-21T11:23:09Z",
            "commit": "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8"
        }))
        .expect("serialize closeout"),
    );

    let output = run_cli(
        vec![
            "xtask".to_string(),
            "close-proving-run".to_string(),
            "--approval".to_string(),
            approval_rel.to_string(),
            "--closeout".to_string(),
            "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json"
                .to_string(),
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output
        .stderr
        .contains("`approval_recorded_at` must be RFC3339"));
}

#[test]
fn close_proving_run_rejects_pack_mismatched_approval_artifact() {
    let fixture = fixture_root("close-proving-run-pack-mismatch");
    seed_release_touchpoints(&fixture);
    let approval_rel =
        "docs/project_management/next/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path = seed_gemini_approval_artifact(&fixture, approval_rel, "other-gemini-pack");
    let approval_sha256 = sha256_hex(&fixture.join(&approval_path));
    let closeout_path = fixture.join(
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json",
    );
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
            "duration_seconds": 17,
            "residual_friction": ["Manual review step still took coordination."],
            "preflight_passed": true,
            "recorded_at": "2026-04-21T11:23:09Z",
            "commit": "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8"
        }))
        .expect("serialize closeout"),
    );

    let output = run_cli(
        vec![
            "xtask".to_string(),
            "close-proving-run".to_string(),
            "--approval".to_string(),
            approval_path.clone(),
            "--closeout".to_string(),
            "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json"
                .to_string(),
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output.stderr.contains("belongs to onboarding_pack_prefix"));
}

#[test]
fn closeout_without_approval_linkage_fails_with_exit_code_2() {
    let fixture = fixture_root("close-proving-run-missing-approval");
    let approval_rel =
        "docs/project_management/next/gemini-cli-onboarding/governance/approved-agent.toml";
    seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let closeout_path = fixture.join(
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json",
    );
    write_text(
        &closeout_path,
        &serde_json::to_string_pretty(&json!({
            "state": "closed",
            "manual_control_plane_edits": 0,
            "partial_write_incidents": 0,
            "ambiguous_ownership_incidents": 0,
            "duration_seconds": 17,
            "explicit_none_reason": "No residual friction remained.",
            "preflight_passed": true,
            "recorded_at": "2026-04-21T11:23:09Z",
            "commit": "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8"
        }))
        .expect("serialize closeout"),
    );

    let output = run_cli(
        vec![
            "xtask".to_string(),
            "close-proving-run".to_string(),
            "--approval".to_string(),
            approval_rel.to_string(),
            "--closeout".to_string(),
            "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json"
                .to_string(),
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output
        .stderr
        .contains("missing required field `approval_ref`"));
}

#[test]
fn closeout_without_duration_truth_fails_with_exit_code_2() {
    let fixture = fixture_root("close-proving-run-missing-duration");
    let approval_rel =
        "docs/project_management/next/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path =
        seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let approval_sha256 = sha256_hex(&fixture.join(&approval_path));
    let closeout_path = fixture.join(
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json",
    );
    write_text(
        &closeout_path,
        &serde_json::to_string_pretty(&json!({
            "state": "closed",
            "approval_ref": approval_rel,
            "approval_sha256": approval_sha256,
            "approval_source": "governance-review",
            "manual_control_plane_edits": 0,
            "partial_write_incidents": 0,
            "ambiguous_ownership_incidents": 0,
            "explicit_none_reason": "No residual friction remained.",
            "preflight_passed": true,
            "recorded_at": "2026-04-21T11:23:09Z",
            "commit": "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8"
        }))
        .expect("serialize closeout"),
    );

    let output = run_cli(
        vec![
            "xtask".to_string(),
            "close-proving-run".to_string(),
            "--approval".to_string(),
            approval_rel.to_string(),
            "--closeout".to_string(),
            "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json"
                .to_string(),
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output
        .stderr
        .contains("exactly one of `duration_seconds` or `duration_missing_reason` is required"));
}

#[test]
fn closeout_without_residual_friction_truth_fails_with_exit_code_2() {
    let fixture = fixture_root("close-proving-run-missing-residual");
    let approval_rel =
        "docs/project_management/next/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path =
        seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let approval_sha256 = sha256_hex(&fixture.join(&approval_path));
    let closeout_path = fixture.join(
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json",
    );
    write_text(
        &closeout_path,
        &serde_json::to_string_pretty(&json!({
            "state": "closed",
            "approval_ref": approval_rel,
            "approval_sha256": approval_sha256,
            "approval_source": "governance-review",
            "manual_control_plane_edits": 0,
            "partial_write_incidents": 0,
            "ambiguous_ownership_incidents": 0,
            "duration_seconds": 17,
            "preflight_passed": true,
            "recorded_at": "2026-04-21T11:23:09Z",
            "commit": "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8"
        }))
        .expect("serialize closeout"),
    );

    let output = run_cli(
        vec![
            "xtask".to_string(),
            "close-proving-run".to_string(),
            "--approval".to_string(),
            approval_rel.to_string(),
            "--closeout".to_string(),
            "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-closeout.json"
                .to_string(),
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output
        .stderr
        .contains("exactly one of `residual_friction` or `explicit_none_reason` is required"));
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
                    match onboard_agent::run_in_workspace(workspace_root, *args, &mut stdout) {
                        Ok(()) => 0,
                        Err(err) => {
                            stderr = format!("{err}\n");
                            err.exit_code()
                        }
                    }
                }
                Command::CloseProvingRun(args) => {
                    match close_proving_run::run_in_workspace(workspace_root, args, &mut stdout) {
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

fn seed_gemini_approval_artifact(
    root: &Path,
    relative_path: &str,
    onboarding_pack_prefix: &str,
) -> String {
    let contents = format!(
        concat!(
            "artifact_version = \"1\"\n",
            "comparison_ref = \"docs/project_management/next/comparisons/gemini.md\"\n",
            "selection_mode = \"factory_validation\"\n",
            "recommended_agent_id = \"gemini_cli\"\n",
            "approved_agent_id = \"gemini_cli\"\n",
            "approval_commit = \"deadbeef\"\n",
            "approval_recorded_at = \"2026-04-21T11:23:09Z\"\n",
            "\n",
            "[descriptor]\n",
            "agent_id = \"gemini_cli\"\n",
            "display_name = \"Gemini CLI\"\n",
            "crate_path = \"crates/gemini_cli\"\n",
            "backend_module = \"crates/agent_api/src/backends/gemini_cli\"\n",
            "manifest_root = \"cli_manifests/gemini_cli\"\n",
            "package_name = \"unified-agent-api-gemini-cli\"\n",
            "canonical_targets = [\"darwin-arm64\"]\n",
            "wrapper_coverage_binding_kind = \"generated_from_wrapper_crate\"\n",
            "wrapper_coverage_source_path = \"crates/gemini_cli\"\n",
            "always_on_capabilities = [\"agent_api.run\"]\n",
            "backend_extensions = []\n",
            "support_matrix_enabled = true\n",
            "capability_matrix_enabled = true\n",
            "docs_release_track = \"crates-io\"\n",
            "onboarding_pack_prefix = \"{onboarding_pack_prefix}\"\n",
        ),
        onboarding_pack_prefix = onboarding_pack_prefix,
    );
    write_text(&root.join(relative_path), &contents);
    relative_path.to_string()
}
