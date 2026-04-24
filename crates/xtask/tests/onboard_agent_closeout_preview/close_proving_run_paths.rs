use std::fs;

use serde_json::json;

use super::{
    harness::{
        fixture_root, seed_gemini_approval_artifact, seed_release_touchpoints, sha256_hex,
        write_text,
    },
    run_cli,
};

#[test]
fn close_proving_run_accepts_absolute_closeout_path_inside_workspace() {
    let fixture = fixture_root("close-proving-run-absolute-inside");
    seed_release_touchpoints(&fixture);
    let approval_rel =
        "docs/reports/agent-lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path =
        seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let closeout_path = fixture.join(
        "docs/reports/agent-lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json",
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
            "docs/reports/agent-lifecycle/gemini-cli-onboarding/governance/approved-agent.toml"
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
        "docs/reports/agent-lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
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
        "docs/reports/agent-lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json",
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
            "docs/reports/agent-lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json"
                .to_string(),
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output.stderr.contains("symlinked component"));
    assert!(!fixture
        .join("docs/reports/agent-lifecycle/gemini-cli-onboarding/README.md")
        .exists());
}

#[cfg(unix)]
#[test]
fn close_proving_run_rejects_symlinked_output_target_without_partial_refresh() {
    use std::os::unix::fs::symlink;

    let fixture = fixture_root("close-proving-run-symlinked-output");
    seed_release_touchpoints(&fixture);
    let approval_rel =
        "docs/reports/agent-lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path =
        seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    let approval_sha256 = sha256_hex(&fixture.join(&approval_path));
    let closeout_path = fixture.join(
        "docs/reports/agent-lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json",
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

    let packet_root = fixture.join("docs/reports/agent-lifecycle/gemini-cli-onboarding");
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
            "docs/reports/agent-lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json"
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
