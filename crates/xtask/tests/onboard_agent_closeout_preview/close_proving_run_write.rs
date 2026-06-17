#[path = "close_proving_run_write_helpers.rs"]
mod helpers;

use std::fs;

use serde_json::json;
use xtask::{approval_artifact, proving_run_closeout};

use self::helpers::{seed_green_publication_surfaces, seed_published_baseline};
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
    seed_green_publication_surfaces(&fixture);
    let approval_rel = "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path =
        seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    seed_published_baseline(&fixture, &approval_path);
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
    let lifecycle = fs::read_to_string(
        fixture.join("docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json"),
    )
    .expect("read lifecycle state");

    assert!(readme.contains("- Packet state: `closed_proving_run`"));
    assert!(readme.contains("Approval linkage: `governance-review` via"));
    assert!(handoff.contains("- approval ref: `docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml`"));
    assert!(handoff.contains("- closeout metadata: `docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json`"));
    assert!(lifecycle.contains("\"lifecycle_stage\": \"closed_baseline\""));
    assert!(lifecycle.contains("\"publication_packet_path\": \"docs/agents/lifecycle/gemini-cli-onboarding/governance/publication-ready.json\""));

    let finalized_closeout: serde_json::Value =
        serde_json::from_slice(&fs::read(&closeout_path).expect("read finalized closeout"))
            .expect("parse finalized closeout");
    let approval = approval_artifact::load_approval_artifact(&fixture, &approval_path)
        .expect("load approval artifact");
    let settlement = finalized_closeout
        .get("maintenance_settlement")
        .expect("maintenance settlement present");
    assert_eq!(
        settlement.get("mode").and_then(serde_json::Value::as_str),
        Some("explicitly_deferred")
    );
    assert_eq!(
        settlement
            .get("approval_section_sha256")
            .and_then(serde_json::Value::as_str),
        Some(approval.maintenance.section_sha256.as_str())
    );
    assert_eq!(
        settlement
            .get("deferral_sha256")
            .and_then(serde_json::Value::as_str),
        Some(match &approval.maintenance.mode {
            approval_artifact::ApprovalMaintenanceMode::ExplicitlyDeferred {
                deferral_sha256,
                ..
            } => deferral_sha256.as_str(),
            approval_artifact::ApprovalMaintenanceMode::ReleaseWatchEnrolled { .. } => {
                panic!("expected deferred fixture approval")
            }
        })
    );
    assert!(settlement
        .get("release_watch_sha256")
        .is_some_and(serde_json::Value::is_null));

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
fn close_proving_run_finalizes_truthful_prepared_draft() {
    let fixture = fixture_root("close-proving-run-finalizes-prepared");
    seed_release_touchpoints(&fixture);
    seed_green_publication_surfaces(&fixture);
    let approval_rel = "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path =
        seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    seed_published_baseline(&fixture, &approval_path);
    let closeout_path = fixture
        .join("docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json");
    let prepared = proving_run_closeout::build_closeout(
        proving_run_closeout::ProvingRunCloseoutState::Prepared,
        proving_run_closeout::ProvingRunCloseoutMachineFields {
            approval_ref: approval_path.clone(),
            approval_sha256: sha256_hex(&fixture.join(&approval_path)),
            approval_source: "governance-review".to_string(),
            maintenance_settlement: None,
            preflight_passed: true,
            recorded_at: "2026-04-21T11:23:09Z".to_string(),
            commit: "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8".to_string(),
        },
        proving_run_closeout::ProvingRunCloseoutHumanFields {
            manual_control_plane_edits: 0,
            partial_write_incidents: 0,
            ambiguous_ownership_incidents: 0,
            duration: proving_run_closeout::DurationTruth::Seconds(17),
            residual_friction: proving_run_closeout::ResidualFrictionTruth::Items(vec![
                "Manual review step still took coordination.".to_string(),
            ]),
        },
    )
    .expect("build prepared closeout");
    write_text(
        &closeout_path,
        &proving_run_closeout::render_closeout_json(&prepared).expect("render prepared closeout"),
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

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    let finalized: serde_json::Value =
        serde_json::from_slice(&fs::read(&closeout_path).expect("read finalized closeout"))
            .expect("parse finalized closeout");
    assert_eq!(
        finalized.get("state").and_then(serde_json::Value::as_str),
        Some("closed")
    );
    let approval = approval_artifact::load_approval_artifact(&fixture, &approval_path)
        .expect("load approval artifact");
    let settlement = finalized
        .get("maintenance_settlement")
        .expect("maintenance settlement present");
    assert_eq!(
        settlement.get("mode").and_then(serde_json::Value::as_str),
        Some("explicitly_deferred")
    );
    assert_eq!(
        settlement
            .get("approval_section_sha256")
            .and_then(serde_json::Value::as_str),
        Some(approval.maintenance.section_sha256.as_str())
    );
    assert_eq!(
        settlement
            .get("deferral_sha256")
            .and_then(serde_json::Value::as_str),
        Some(match &approval.maintenance.mode {
            approval_artifact::ApprovalMaintenanceMode::ExplicitlyDeferred {
                deferral_sha256,
                ..
            } => deferral_sha256.as_str(),
            approval_artifact::ApprovalMaintenanceMode::ReleaseWatchEnrolled { .. } => {
                panic!("expected deferred fixture approval")
            }
        })
    );
    assert!(settlement
        .get("release_watch_sha256")
        .is_some_and(serde_json::Value::is_null));
}

#[test]
fn close_proving_run_rejects_unresolved_placeholders_in_prepared_draft() {
    let fixture = fixture_root("close-proving-run-prepared-placeholders");
    seed_release_touchpoints(&fixture);
    seed_green_publication_surfaces(&fixture);
    write_text(
        &fixture.join("docs/agents/lifecycle/gemini-cli-onboarding/README.md"),
        "Gemini CLI packet scaffold.\n",
    );
    let approval_rel = "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_path =
        seed_gemini_approval_artifact(&fixture, approval_rel, "gemini-cli-onboarding");
    seed_published_baseline(&fixture, &approval_path);
    let closeout_path = fixture
        .join("docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json");
    let prepared = proving_run_closeout::build_prepared_closeout(
        proving_run_closeout::ProvingRunCloseoutMachineFields {
            approval_ref: approval_path.clone(),
            approval_sha256: sha256_hex(&fixture.join(&approval_path)),
            approval_source: "governance-review".to_string(),
            maintenance_settlement: None,
            preflight_passed: true,
            recorded_at: "2026-04-21T11:23:09Z".to_string(),
            commit: "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8".to_string(),
        },
    )
    .expect("build prepared closeout");
    write_text(
        &closeout_path,
        &proving_run_closeout::render_closeout_json(&prepared).expect("render prepared closeout"),
    );

    let output = run_cli(
        vec![
            "xtask".to_string(),
            "close-proving-run".to_string(),
            "--approval".to_string(),
            approval_path,
            "--closeout".to_string(),
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json"
                .to_string(),
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output.stderr.contains("placeholder"));
    assert_eq!(
        fs::read_to_string(fixture.join("docs/agents/lifecycle/gemini-cli-onboarding/README.md"))
            .expect("read original readme"),
        "Gemini CLI packet scaffold.\n"
    );
}
