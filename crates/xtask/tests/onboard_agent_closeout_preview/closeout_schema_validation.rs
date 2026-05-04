use std::path::Path;

use serde_json::json;
use xtask::proving_run_closeout;

use super::{
    harness::{fixture_root, seed_gemini_approval_artifact, sha256_hex, write_text},
    run_cli,
};

#[test]
fn closeout_without_approval_linkage_fails_with_exit_code_2() {
    let fixture = fixture_root("close-proving-run-missing-approval");
    let approval_rel = "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let closeout_path = fixture
        .join("docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json");
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
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json"
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
    let approval_rel = "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path =
        seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let approval_sha256 = sha256_hex(&fixture.join(&approval_path));
    let closeout_path = fixture
        .join("docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json");
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
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json"
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
    let approval_rel = "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path =
        seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let approval_sha256 = sha256_hex(&fixture.join(&approval_path));
    let closeout_path = fixture
        .join("docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json");
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
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json"
                .to_string(),
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output
        .stderr
        .contains("exactly one of `residual_friction` or `explicit_none_reason` is required"));
}

#[test]
fn prepared_closeout_round_trips_through_shared_parser_and_serializer() {
    let fixture = fixture_root("close-proving-run-prepared-roundtrip");
    let approval_rel = "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path =
        seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let closeout_rel =
        "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json";
    let closeout_path = fixture.join(closeout_rel);
    let prepared = proving_run_closeout::build_prepared_closeout(
        proving_run_closeout::ProvingRunCloseoutMachineFields {
            approval_ref: approval_path.clone(),
            approval_sha256: sha256_hex(&fixture.join(&approval_path)),
            approval_source: "prepare-proving-run-closeout".to_string(),
            preflight_passed: true,
            recorded_at: "2026-04-21T11:23:09Z".to_string(),
            commit: "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8".to_string(),
        },
    )
    .expect("build prepared closeout");
    let rendered =
        proving_run_closeout::render_closeout_json(&prepared).expect("render prepared closeout");
    write_text(&closeout_path, &rendered);

    let parsed = proving_run_closeout::load_validated_closeout_with_states(
        &fixture,
        Path::new(closeout_rel),
        &closeout_path,
        proving_run_closeout::ProvingRunCloseoutExpected {
            approval_path: Some(Path::new(&approval_path)),
            onboarding_pack_prefix: "gemini-cli-onboarding",
        },
        proving_run_closeout::ProvingRunCloseoutState::all(),
    )
    .expect("load prepared closeout");

    assert_eq!(parsed.state.as_str(), "prepared");
    assert!(proving_run_closeout::has_unresolved_placeholders(&parsed));
    assert_eq!(
        proving_run_closeout::render_closeout_json(&parsed).expect("re-render prepared closeout"),
        rendered
    );
}
