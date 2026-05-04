use serde_json::json;
use xtask::proving_run_closeout;

use super::{
    harness::{
        fixture_root, gemini_dry_run_args, repo_root, seed_gemini_approval_artifact,
        seed_release_touchpoints, sha256_hex, write_text,
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
        "Closeout metadata is recorded in `docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json`."
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
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-metrics.json",
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
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json",
        ),
        "{ not valid json }\n",
    );

    let output = run_cli(gemini_dry_run_args(), &fixture);

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output.stderr.contains(
        "parse docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json"
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
    let approval_rel = "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    write_text(
        &fixture.join(
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json",
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
fn prepared_closeout_draft_does_not_preview_as_closed_packet() {
    let fixture = fixture_root("onboard-agent-prepared-closeout-preview");
    seed_release_touchpoints(&fixture);
    let approval_rel = "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path =
        seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let prepared = proving_run_closeout::build_prepared_closeout(
        proving_run_closeout::ProvingRunCloseoutMachineFields {
            approval_ref: approval_path.clone(),
            approval_sha256: sha256_hex(&fixture.join(&approval_path)),
            approval_source: "governance-review".to_string(),
            preflight_passed: true,
            recorded_at: "2026-04-21T11:23:09Z".to_string(),
            commit: "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8".to_string(),
        },
    )
    .expect("build prepared closeout");
    write_text(
        &fixture.join(
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json",
        ),
        &proving_run_closeout::render_closeout_json(&prepared).expect("render prepared closeout"),
    );

    let output = run_cli(gemini_dry_run_args(), &fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert!(output
        .stdout
        .contains("This packet records the prepared proving-run closeout draft for `Gemini CLI`."));
    assert!(output
        .stdout
        .contains("- Packet state: `closeout_prepared`"));
    assert!(!output.stdout.contains("closed_proving_run"));
    assert!(output
        .stdout
        .contains("Prepared closeout metadata is recorded in `docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json`; the proving run is not yet closed."));
    assert!(output.stdout.contains("Approval linkage:"));
}
