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
fn close_proving_run_validates_and_refreshes_packet_docs() {
    let fixture = fixture_root("close-proving-run-pass");
    seed_release_touchpoints(&fixture);
    let approval_rel =
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-approval.md";
    let approval_path = fixture.join(approval_rel);
    write_text(&approval_path, "# Approval\n\nM3 proving run approved.\n");
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

    let closeout_output = run_cli(
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
    assert!(handoff.contains("- approval ref: `docs/project_management/next/gemini-cli-onboarding/governance/proving-run-approval.md`"));
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
fn closeout_without_approval_linkage_fails_with_exit_code_2() {
    let fixture = fixture_root("close-proving-run-missing-approval");
    let approval_rel =
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-approval.md";
    let approval_path = fixture.join(approval_rel);
    write_text(&approval_path, "approval\n");
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
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-approval.md";
    let approval_path = fixture.join(approval_rel);
    write_text(&approval_path, "approval\n");
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
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-approval.md";
    let approval_path = fixture.join(approval_rel);
    write_text(&approval_path, "approval\n");
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
