use serde_json::json;

use super::{
    harness::{
        fixture_root, gemini_dry_run_args, repo_root, seed_gemini_approval_artifact,
        seed_release_touchpoints, write_text,
    },
    run_cli,
};

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
        "Closeout metadata is recorded in `docs/reports/agent-lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json`."
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
            "docs/reports/agent-lifecycle/gemini-cli-onboarding/governance/proving-run-metrics.json",
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
            "docs/reports/agent-lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json",
        ),
        "{ not valid json }\n",
    );

    let output = run_cli(gemini_dry_run_args(), &fixture);

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output.stderr.contains(
        "parse docs/reports/agent-lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json"
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
        "docs/reports/agent-lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    write_text(
        &fixture.join(
            "docs/reports/agent-lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json",
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
