use std::fs;

use serde_json::json;
use xtask::support_matrix;

use super::{
    harness::{
        fixture_root, gemini_dry_run_args, seed_gemini_approval_artifact, seed_release_touchpoints,
        sha256_hex, write_text,
    },
    run_cli,
};

fn seed_cli_manifest_root(root: &std::path::Path, manifest_root: &str, canonical_targets: &[&str]) {
    let current = serde_json::json!({
        "expected_targets": canonical_targets,
        "inputs": canonical_targets
            .iter()
            .map(|target| serde_json::json!({
                "target_triple": target,
                "binary": { "semantic_version": "1.0.0" }
            }))
            .collect::<Vec<_>>(),
        "commands": []
    });
    write_text(
        &root.join(manifest_root).join("current.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&current).expect("serialize current manifest")
        ),
    );

    let version = serde_json::json!({
        "semantic_version": "1.0.0",
        "status": "supported",
        "coverage": { "supported_targets": canonical_targets },
    });
    write_text(
        &root.join(manifest_root).join("versions/1.0.0.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&version).expect("serialize version metadata")
        ),
    );

    for target in canonical_targets {
        write_text(
            &root
                .join(manifest_root)
                .join(format!("pointers/latest_supported/{target}.txt")),
            "1.0.0\n",
        );
        write_text(
            &root
                .join(manifest_root)
                .join(format!("pointers/latest_validated/{target}.txt")),
            "1.0.0\n",
        );
        let report = serde_json::json!({
            "inputs": { "upstream": { "targets": [target] } },
            "deltas": {
                "missing_commands": [],
                "missing_flags": [],
                "missing_args": [],
                "intentionally_unsupported": [],
                "wrapper_only_commands": [],
                "wrapper_only_flags": [],
                "wrapper_only_args": []
            }
        });
        write_text(
            &root
                .join(manifest_root)
                .join(format!("reports/1.0.0/coverage.{target}.json")),
            &format!(
                "{}\n",
                serde_json::to_string_pretty(&report).expect("serialize report")
            ),
        );
    }
}

fn seed_lifecycle_eligible_audit_peers(root: &std::path::Path) {
    write_text(
        &root.join("cli_manifests/codex/current.json"),
        include_str!("../../../../cli_manifests/codex/current.json"),
    );
    write_text(
        &root.join("cli_manifests/claude_code/current.json"),
        include_str!("../../../../cli_manifests/claude_code/current.json"),
    );
    write_text(
        &root.join("docs/agents/lifecycle/codex-cli-onboarding/governance/approved-agent.toml"),
        include_str!(
            "../../../../docs/agents/lifecycle/codex-cli-onboarding/governance/approved-agent.toml"
        ),
    );
    write_text(
        &root.join("docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json"),
        include_str!("../../../../docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json"),
    );
    write_text(
        &root.join("docs/agents/lifecycle/codex-cli-onboarding/governance/publication-ready.json"),
        include_str!("../../../../docs/agents/lifecycle/codex-cli-onboarding/governance/publication-ready.json"),
    );
    write_text(
        &root.join("docs/agents/lifecycle/codex-cli-onboarding/governance/proving-run-closeout.json"),
        include_str!("../../../../docs/agents/lifecycle/codex-cli-onboarding/governance/proving-run-closeout.json"),
    );

    write_text(
        &root.join("docs/agents/lifecycle/claude-code-cli-onboarding/governance/approved-agent.toml"),
        include_str!("../../../../docs/agents/lifecycle/claude-code-cli-onboarding/governance/approved-agent.toml"),
    );
    write_text(
        &root.join("docs/agents/lifecycle/claude-code-cli-onboarding/governance/lifecycle-state.json"),
        include_str!("../../../../docs/agents/lifecycle/claude-code-cli-onboarding/governance/lifecycle-state.json"),
    );
    write_text(
        &root.join("docs/agents/lifecycle/claude-code-cli-onboarding/governance/publication-ready.json"),
        include_str!("../../../../docs/agents/lifecycle/claude-code-cli-onboarding/governance/publication-ready.json"),
    );
    write_text(
        &root.join("docs/agents/lifecycle/claude-code-cli-onboarding/governance/proving-run-closeout.json"),
        include_str!("../../../../docs/agents/lifecycle/claude-code-cli-onboarding/governance/proving-run-closeout.json"),
    );
}

fn seed_green_publication_surfaces(root: &std::path::Path) {
    write_text(
        &root.join("docs/specs/unified-agent-api/support-matrix.md"),
        "# Support matrix\n\nManual contract text.\n",
    );
    seed_cli_manifest_root(root, "cli_manifests/codex", &["x86_64-unknown-linux-musl"]);
    seed_cli_manifest_root(root, "cli_manifests/claude_code", &["linux-x64"]);
    seed_cli_manifest_root(
        root,
        "cli_manifests/opencode",
        &["linux-x64", "darwin-arm64", "win32-x64"],
    );
    seed_cli_manifest_root(root, "cli_manifests/gemini_cli", &["darwin-arm64"]);
    seed_cli_manifest_root(root, "cli_manifests/aider", &["darwin-arm64"]);

    let bundle =
        support_matrix::generate_publication_artifacts(root).expect("generate support publication");
    write_text(
        &root.join("cli_manifests/support_matrix/current.json"),
        &bundle.json,
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/support-matrix.md"),
        &bundle.markdown,
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/capability-matrix.md"),
        include_str!("../../../../docs/specs/unified-agent-api/capability-matrix.md"),
    );
    seed_lifecycle_eligible_audit_peers(root);
}

fn seed_publication_ready_baseline(root: &std::path::Path, approval_path: &str) {
    let approval_sha256 = sha256_hex(&root.join(approval_path));
    let lifecycle_path =
        root.join("docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json");
    write_text(
        &lifecycle_path,
        &serde_json::to_string_pretty(&json!({
            "schema_version": "1",
            "agent_id": "gemini_cli",
            "onboarding_pack_prefix": "gemini-cli-onboarding",
            "approval_artifact_path": approval_path,
            "approval_artifact_sha256": approval_sha256,
            "lifecycle_stage": "publication_ready",
            "support_tier": "baseline_runtime",
            "side_states": [],
            "current_owner_command": "refresh-publication --write",
            "expected_next_command": "close-proving-run --approval docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml --closeout docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json",
            "last_transition_at": "2026-05-01T00:00:00Z",
            "last_transition_by": "xtask refresh-publication --write",
            "required_evidence": [
                "registry_entry",
                "docs_pack",
                "manifest_root_skeleton",
                "runtime_write_complete",
                "implementation_summary_present",
                "publication_packet_written"
            ],
            "satisfied_evidence": [
                "registry_entry",
                "docs_pack",
                "manifest_root_skeleton",
                "runtime_write_complete",
                "implementation_summary_present",
                "publication_packet_written"
            ],
            "blocking_issues": [],
            "retryable_failures": [],
            "active_runtime_evidence_run_id": serde_json::Value::Null,
            "implementation_summary": {
                "requested_runtime_profile": "default",
                "achieved_runtime_profile": "default",
                "primary_template": "gemini_cli",
                "template_lineage": ["gemini_cli"],
                "landed_surfaces": [
                    "wrapper_runtime",
                    "backend_harness",
                    "runtime_manifest_evidence"
                ],
                "deferred_surfaces": [],
                "minimal_profile_justification": serde_json::Value::Null
            },
            "publication_packet_path": serde_json::Value::Null,
            "publication_packet_sha256": serde_json::Value::Null,
            "closeout_baseline_path": serde_json::Value::Null
        }))
        .expect("serialize lifecycle state"),
    );
    let lifecycle_sha256 = sha256_hex(&lifecycle_path);
    write_text(
        &root.join("docs/agents/lifecycle/gemini-cli-onboarding/governance/publication-ready.json"),
        &serde_json::to_string_pretty(&json!({
            "schema_version": "1",
            "agent_id": "gemini_cli",
            "approval_artifact_path": approval_path,
            "approval_artifact_sha256": approval_sha256,
            "lifecycle_state_path": "docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json",
            "lifecycle_state_sha256": lifecycle_sha256,
            "lifecycle_stage": "publication_ready",
            "support_tier_at_emit": "baseline_runtime",
            "manifest_root": "cli_manifests/gemini_cli",
            "expected_targets": ["darwin-arm64"],
            "capability_publication_enabled": true,
            "support_publication_enabled": true,
            "capability_matrix_target": serde_json::Value::Null,
            "required_commands": [
                "cargo run -p xtask -- support-matrix --check",
                "cargo run -p xtask -- capability-matrix --check",
                "cargo run -p xtask -- capability-matrix-audit",
                "make preflight"
            ],
            "required_publication_outputs": [
                "cli_manifests/support_matrix/current.json",
                "docs/specs/unified-agent-api/support-matrix.md",
                "docs/specs/unified-agent-api/capability-matrix.md"
            ],
            "runtime_evidence_paths": [
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/input-contract.json",
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/run-status.json",
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/run-summary.md",
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/validation-report.json",
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/written-paths.json",
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/handoff.json"
            ],
            "publication_owned_paths": [
                "docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json",
                "docs/agents/lifecycle/gemini-cli-onboarding/governance/publication-ready.json"
            ],
            "blocking_issues": [],
            "implementation_summary": {
                "requested_runtime_profile": "default",
                "achieved_runtime_profile": "default",
                "primary_template": "gemini_cli",
                "template_lineage": ["gemini_cli"],
                "landed_surfaces": [
                    "wrapper_runtime",
                    "backend_harness",
                    "runtime_manifest_evidence"
                ],
                "deferred_surfaces": [],
                "minimal_profile_justification": serde_json::Value::Null
            }
        }))
        .expect("serialize publication-ready packet"),
    );
}

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
    seed_publication_ready_baseline(&fixture, &approval_path);
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
