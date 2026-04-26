use std::fs;

use serde_json::json;

use super::{
    harness::{
        fixture_root, gemini_dry_run_args, seed_gemini_approval_artifact, seed_release_touchpoints,
        sha256_hex, write_text,
    },
    run_cli,
};

#[test]
fn onboard_agent_write_does_not_rewrite_packet_files_when_closeout_is_invalid() {
    let fixture = fixture_root("onboard-agent-invalid-closeout-write");
    seed_release_touchpoints(&fixture);
    let approval_rel = "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let readme_path = fixture.join("docs/agents/lifecycle/gemini-cli-onboarding/README.md");
    let handoff_path = fixture.join("docs/agents/lifecycle/gemini-cli-onboarding/HANDOFF.md");
    write_text(&readme_path, "existing readme\n");
    write_text(&handoff_path, "existing handoff\n");
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
        .join("docs/agents/lifecycle/gemini-cli-onboarding/scope_brief.md")
        .exists());
}

#[test]
fn close_proving_run_validates_and_refreshes_packet_docs() {
    let fixture = fixture_root("close-proving-run-pass");
    seed_release_touchpoints(&fixture);
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
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json"
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

    let readme =
        fs::read_to_string(fixture.join("docs/agents/lifecycle/gemini-cli-onboarding/README.md"))
            .expect("read refreshed readme");
    let handoff =
        fs::read_to_string(fixture.join("docs/agents/lifecycle/gemini-cli-onboarding/HANDOFF.md"))
            .expect("read refreshed handoff");

    assert!(readme.contains("- Packet state: `closed_proving_run`"));
    assert!(readme.contains("Approval linkage: `governance-review` via"));
    assert!(handoff.contains("- approval ref: `docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml`"));
    assert!(handoff.contains("- closeout metadata: `docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json`"));

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
