use serde_json::json;

use super::{
    harness::{
        fixture_root, seed_gemini_approval_artifact, seed_release_touchpoints, sha256_hex,
        write_text,
    },
    run_cli,
};

#[test]
fn close_proving_run_rejects_noncanonical_approval_artifact_even_when_ref_and_sha_match() {
    let fixture = fixture_root("close-proving-run-noncanonical-approval");
    seed_release_touchpoints(&fixture);
    let approval_rel =
        "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-approval.md";
    let approval_path = fixture.join(approval_rel);
    write_text(&approval_path, "# not an approval artifact\n");
    let approval_sha256 = sha256_hex(&approval_path);
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
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json"
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
    let approval_rel = "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path = fixture.join(approval_rel);
    write_text(&approval_path, "not = [valid toml\n");
    let approval_sha256 = sha256_hex(&approval_path);
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
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json"
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
    let approval_rel = "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path = fixture.join(approval_rel);
    write_text(
        &approval_path,
        concat!(
            "artifact_version = \"1\"\n",
            "comparison_ref = \"docs/agents/selection/cli-agent-selection-packet.md\"\n",
            "selection_mode = \"factory_validation\"\n",
            "recommended_agent_id = \"gemini_cli\"\n",
            "approved_agent_id = \"gemini_cli\"\n",
            "approval_commit = \"deadbeef\"\n",
            "approval_recorded_at = \"2026-04-21T11:23:09Z\"\n",
        ),
    );
    let approval_sha256 = sha256_hex(&approval_path);
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
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json"
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
    let approval_rel = "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path =
        seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let approval_file = fixture.join(&approval_path);
    let approval_contents = std::fs::read_to_string(&approval_file).expect("read approval");
    write_text(
        &approval_file,
        &approval_contents.replace(
            "approval_recorded_at = \"2026-04-21T11:23:09Z\"",
            "approval_recorded_at = \"not-a-timestamp\"",
        ),
    );
    let approval_sha256 = sha256_hex(&approval_file);
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
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json"
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
    let approval_rel = "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path = seed_gemini_approval_artifact(&fixture, approval_rel, "other-gemini-pack");
    let approval_sha256 = sha256_hex(&fixture.join(&approval_path));
    let closeout_path = fixture
        .join("docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json");
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
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json"
                .to_string(),
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output.stderr.contains("belongs to onboarding_pack_prefix"));
}
