#![allow(dead_code, unused_imports)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[path = "support/onboard_agent_harness.rs"]
mod harness;

mod agent_lifecycle {
    pub use xtask::agent_lifecycle::*;
}

mod prepare_publication {
    pub use xtask::prepare_publication::*;
}

#[path = "../src/agent_registry.rs"]
mod agent_registry;
mod approval_artifact {
    pub use xtask::approval_artifact::*;
}
#[path = "../src/capability_projection.rs"]
mod capability_projection;
#[path = "../src/agent_maintenance/drift/mod.rs"]
mod drift;
#[path = "../src/agent_maintenance/finding_signature.rs"]
mod finding_signature;
#[path = "../src/release_doc.rs"]
mod release_doc;
#[path = "../src/root_intake_layout.rs"]
mod root_intake_layout;
#[path = "../src/support_matrix.rs"]
mod support_matrix;

use drift::{check_agent_drift, DriftCategory, DriftCheckError};
use harness::{
    fixture_root, seed_gemini_approval_artifact, seed_release_touchpoints, sha256_hex, write_text,
};

#[test]
fn check_agent_drift_reports_clean_agent() {
    let fixture = fixture_root("agent-maintenance-drift-clean");
    seed_publication_inputs(&fixture);

    let report = check_agent_drift(&fixture, "gemini_cli").expect("clean report");
    assert!(report.is_clean(), "{}", report.render());
}

#[test]
fn check_agent_drift_reports_missing_lifecycle_publication_packet_path() {
    let fixture = fixture_root("agent-maintenance-drift-missing-packet-path");
    seed_publication_inputs(&fixture);
    seed_gemini_lifecycle_baseline(&fixture, LifecycleBaselineGap::MissingPublicationPacketPath);

    let report = check_agent_drift(&fixture, "gemini_cli").expect("drift report");
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.category == DriftCategory::GovernanceDoc)
        .expect("governance doc finding");
    assert!(finding
        .summary
        .contains("historical governance surfaces no longer match"));
    assert!(
        finding.summary.contains("publication_packet_path"),
        "{}",
        finding.summary
    );
}

#[test]
fn check_agent_drift_reports_missing_lifecycle_publication_packet_sha() {
    let fixture = fixture_root("agent-maintenance-drift-missing-packet-sha");
    seed_publication_inputs(&fixture);
    seed_gemini_lifecycle_baseline(&fixture, LifecycleBaselineGap::MissingPublicationPacketSha);

    let report = check_agent_drift(&fixture, "gemini_cli").expect("drift report");
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.category == DriftCategory::GovernanceDoc)
        .expect("governance doc finding");
    assert!(finding.summary.contains("publication_packet_sha256"));
}

#[test]
fn check_agent_drift_reports_missing_lifecycle_closeout_path() {
    let fixture = fixture_root("agent-maintenance-drift-missing-closeout-path");
    seed_publication_inputs(&fixture);
    seed_gemini_lifecycle_baseline(&fixture, LifecycleBaselineGap::MissingCloseoutBaselinePath);

    let report = check_agent_drift(&fixture, "gemini_cli").expect("drift report");
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.category == DriftCategory::GovernanceDoc)
        .expect("governance doc finding");
    assert!(finding.summary.contains("closeout_baseline_path"));
}

#[test]
fn check_agent_drift_entrypoint_recovers_after_lifecycle_baseline_repair() {
    let fixture = fixture_root("agent-maintenance-drift-lifecycle-repair");
    seed_publication_inputs(&fixture);
    seed_gemini_lifecycle_baseline(&fixture, LifecycleBaselineGap::MissingPublicationPacketPath);

    let before = run_xtask_check_agent_drift(&fixture, "gemini_cli");
    assert_eq!(before.status.code(), Some(2));
    let before_stdout = String::from_utf8_lossy(&before.stdout);
    assert!(before_stdout.contains("status: drift_detected"));
    assert!(before_stdout.contains("publication_packet_path"));

    seed_gemini_lifecycle_baseline(&fixture, LifecycleBaselineGap::None);

    let after = run_xtask_check_agent_drift(&fixture, "gemini_cli");
    assert!(
        after.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&after.stdout),
        String::from_utf8_lossy(&after.stderr)
    );
    assert!(String::from_utf8_lossy(&after.stdout).contains("status: clean"));
}

#[test]
fn check_agent_drift_does_not_force_add_default_off_config_gated_capabilities() {
    let fixture = fixture_root("agent-maintenance-drift-default-off-config-gated");
    seed_publication_inputs(&fixture);

    let report = check_agent_drift(&fixture, "codex").expect("drift report");
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.category == DriftCategory::CapabilityPublication)
        .expect("capability publication finding");
    assert!(!finding.summary.contains("agent_api.tools.mcp.add.v1"));
    assert!(!finding.summary.contains("agent_api.tools.mcp.remove.v1"));
    assert!(!finding
        .summary
        .contains("agent_api.exec.external_sandbox.v1"));
}

#[test]
fn check_agent_drift_treats_absent_approval_publication_target_as_unasserted() {
    let fixture = fixture_root("agent-maintenance-drift-approval-target-optional");
    seed_publication_inputs(&fixture);

    let registry_path = fixture.join("crates/xtask/data/agent_registry.toml");
    let registry = fs::read_to_string(&registry_path).expect("read registry");
    write_text(
        &registry_path,
        &registry.replacen(
            "capability_matrix_enabled = true\n\n[agents.release]\ndocs_release_track = \"crates-io\"\n\n[agents.scaffold]\nonboarding_pack_prefix = \"gemini-cli-onboarding\"\n",
            "capability_matrix_enabled = true\ncapability_matrix_target = \"darwin-arm64\"\n\n[agents.release]\ndocs_release_track = \"crates-io\"\n\n[agents.scaffold]\nonboarding_pack_prefix = \"gemini-cli-onboarding\"\n",
            1,
        ),
    );

    let report = check_agent_drift(&fixture, "gemini_cli").expect("clean report");
    assert!(report.is_clean(), "{}", report.render());
}

#[test]
fn check_agent_drift_rejects_unknown_agent() {
    let fixture = fixture_root("agent-maintenance-drift-unknown");
    seed_publication_inputs(&fixture);

    let err = check_agent_drift(&fixture, "missing-agent").expect_err("unknown agent should fail");
    assert!(
        matches!(err, DriftCheckError::Validation(message) if message.contains("missing-agent"))
    );
}

#[test]
fn check_agent_drift_reports_support_publication_mismatch() {
    let fixture = fixture_root("agent-maintenance-drift-support");
    seed_publication_inputs(&fixture);
    seed_governance_closeouts(
        &fixture,
        &[
            "agent_api.run",
            "agent_api.events",
            "agent_api.events.live",
            "agent_api.config.model.v1",
            "agent_api.session.resume.v1",
            "agent_api.session.fork.v1",
        ],
        true,
    );

    write_text(
        &fixture.join("docs/specs/unified-agent-api/support-matrix.md"),
        "# Support matrix\n\nCorrupted support publication.\n",
    );

    let report = check_agent_drift(&fixture, "opencode").expect("drift report");
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.category == DriftCategory::SupportPublication));
}

#[test]
fn check_agent_drift_reports_capability_truth_mismatch() {
    let fixture = fixture_root("agent-maintenance-drift-capability");
    seed_publication_inputs(&fixture);
    seed_governance_closeouts(
        &fixture,
        &[
            "agent_api.run",
            "agent_api.events",
            "agent_api.events.live",
            "agent_api.config.model.v1",
            "agent_api.session.resume.v1",
            "agent_api.session.fork.v1",
        ],
        true,
    );

    write_text(
        &fixture.join("docs/specs/unified-agent-api/capability-matrix.md"),
        "# Capability matrix\n\nThis file is generated by `cargo run -p xtask -- capability-matrix`.\n\n## `agent_api.core`\n\n| capability id | `opencode` |\n|---|---|\n| `agent_api.run` | ✅ |\n",
    );

    let report = check_agent_drift(&fixture, "opencode").expect("drift report");
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.category == DriftCategory::CapabilityPublication));
}

#[test]
fn check_agent_drift_reports_governance_doc_mismatch() {
    let fixture = fixture_root("agent-maintenance-drift-governance");
    seed_publication_inputs(&fixture);
    seed_governance_closeouts(
        &fixture,
        &["agent_api.run", "agent_api.events", "agent_api.events.live"],
        true,
    );

    let report = check_agent_drift(&fixture, "opencode").expect("drift report");
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.category == DriftCategory::GovernanceDoc)
        .expect("governance finding");
    assert!(finding
        .surfaces
        .contains(&"docs/integrations/opencode/governance/seam-2-closeout.md".to_string()));
    assert!(finding
        .surfaces
        .contains(&"docs/specs/unified-agent-api/capability-matrix.md".to_string()));
}

#[test]
fn check_agent_drift_ignores_unrelated_broken_manifest_roots() {
    let fixture = fixture_root("agent-maintenance-drift-unrelated-root");
    seed_publication_inputs(&fixture);

    fs::remove_file(fixture.join("cli_manifests/gemini_cli/current.json"))
        .expect("remove unrelated manifest root");

    let report = check_agent_drift(&fixture, "opencode").expect("opencode report");
    assert_eq!(report.agent_id, "opencode");
}

#[test]
fn check_agent_drift_skips_support_derivation_when_agent_is_not_support_enrolled() {
    let fixture = fixture_root("agent-maintenance-drift-support-disabled");
    seed_publication_inputs(&fixture);
    set_support_matrix_enabled(&fixture, "opencode", false);
    fs::remove_file(fixture.join("cli_manifests/opencode/versions/1.0.0.json"))
        .expect("remove selected agent version metadata");

    let report = check_agent_drift(&fixture, "opencode").expect("opencode report");
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.category != DriftCategory::SupportPublication),
        "{}",
        report.render()
    );
}

#[test]
fn check_agent_drift_reports_gemini_approval_descriptor_mismatch() {
    let fixture = fixture_root("agent-maintenance-drift-gemini-approval");
    seed_publication_inputs(&fixture);

    write_text(
        &fixture.join("docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml"),
        concat!(
            "artifact_version = \"1\"\n",
            "comparison_ref = \"docs/agents/selection/cli-agent-selection-packet.md\"\n",
            "selection_mode = \"factory_validation\"\n",
            "recommended_agent_id = \"gemini_cli\"\n",
            "approved_agent_id = \"gemini_cli\"\n",
            "approval_commit = \"deadbeef\"\n",
            "approval_recorded_at = \"2026-04-21T11:23:09Z\"\n\n",
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
            "onboarding_pack_prefix = \"gemini-cli-onboarding\"\n",
        ),
    );

    let report = check_agent_drift(&fixture, "gemini_cli").expect("drift report");
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.category == DriftCategory::GovernanceDoc)
        .expect("governance finding");
    assert!(finding.surfaces.contains(
        &"docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml".to_string()
    ));
    assert!(finding
        .surfaces
        .contains(&"crates/xtask/data/agent_registry.toml".to_string()));
}

#[test]
fn check_agent_drift_reports_stale_runtime_evidence_for_runtime_integrated_agent() {
    let fixture = fixture_root("agent-maintenance-drift-runtime-evidence-stale");
    seed_publication_inputs(&fixture);
    seed_gemini_runtime_integrated_state(&fixture);
    seed_runtime_evidence_run(&fixture, RuntimeEvidenceTruth::Stale);

    let report = check_agent_drift(&fixture, "gemini_cli").expect("drift report");
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.category == DriftCategory::RuntimeEvidence)
        .expect("runtime evidence finding");
    assert!(finding.summary.contains("repair-runtime-evidence"));
    assert!(finding.summary.contains("runtime evidence run"));
    assert!(finding.surfaces.contains(
        &"docs/agents/.uaa-temp/runtime-follow-on/runs/repair-gemini_cli-runtime-follow-on"
            .to_string()
    ));
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.category != DriftCategory::GovernanceDoc),
        "{}",
        report.render()
    );
}

#[test]
fn check_agent_drift_clears_runtime_evidence_finding_after_truthful_repair() {
    let fixture = fixture_root("agent-maintenance-drift-runtime-evidence-repaired");
    seed_publication_inputs(&fixture);
    seed_gemini_runtime_integrated_state(&fixture);
    seed_runtime_evidence_run(&fixture, RuntimeEvidenceTruth::Stale);

    let before = check_agent_drift(&fixture, "gemini_cli").expect("stale drift report");
    assert!(before
        .findings
        .iter()
        .any(|finding| finding.category == DriftCategory::RuntimeEvidence));

    seed_runtime_evidence_run(&fixture, RuntimeEvidenceTruth::Truthful);

    let after = check_agent_drift(&fixture, "gemini_cli").expect("repaired report");
    assert!(
        after
            .findings
            .iter()
            .all(|finding| finding.category != DriftCategory::RuntimeEvidence),
        "{}",
        after.render()
    );
    assert!(after.is_clean(), "{}", after.render());
}

#[test]
fn check_agent_drift_reports_recurring_closed_governance_surface() {
    let fixture = fixture_root("agent-maintenance-drift-governance-closed");
    seed_publication_inputs(&fixture);
    seed_governance_closeouts(
        &fixture,
        &["agent_api.run", "agent_api.events", "agent_api.events.live"],
        false,
    );
    seed_cli_manifest_root(
        &fixture,
        "cli_manifests/opencode",
        &["linux-x64", "darwin-arm64", "win32-x64"],
        &[],
    );
    seed_closed_governance_maintenance(
        &fixture,
        "docs/integrations/opencode/governance/seam-2-closeout.md",
    );

    let report = check_agent_drift(&fixture, "opencode").expect("drift report");
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.category == DriftCategory::GovernanceDoc),
        "{}",
        report.render()
    );
}

#[test]
fn check_agent_drift_reports_support_derivation_failure_as_categorized_drift() {
    let fixture = fixture_root("agent-maintenance-drift-support-derivation-error");
    seed_publication_inputs(&fixture);

    fs::remove_file(fixture.join("cli_manifests/opencode/pointers/latest_supported/linux-x64.txt"))
        .expect("remove selected agent support pointer");

    let report = check_agent_drift(&fixture, "opencode").expect("drift report");
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.category == DriftCategory::SupportPublication)
        .expect("support publication finding");
    assert!(finding.summary.contains("derive support rows"));
    assert!(finding
        .surfaces
        .contains(&"cli_manifests/opencode".to_string()));
    assert!(finding
        .surfaces
        .contains(&"cli_manifests/support_matrix/current.json".to_string()));
}

#[test]
fn check_agent_drift_entrypoint_exits_two_for_support_derivation_drift() {
    let fixture = fixture_root("agent-maintenance-drift-exit-support-derivation-error");
    seed_publication_inputs(&fixture);

    fs::remove_file(fixture.join("cli_manifests/opencode/pointers/latest_supported/linux-x64.txt"))
        .expect("remove selected agent support pointer");

    let output = run_xtask_check_agent_drift(&fixture, "opencode");
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("status: drift_detected"));
    assert!(stdout.contains("category_id: support_publication_drift"));
    assert!(stdout.contains("derive support rows"));
}

#[test]
fn check_agent_drift_reports_release_doc_missing_tail_packages() {
    let fixture = fixture_root("agent-maintenance-drift-release-doc-missing-tail");
    seed_publication_inputs(&fixture);

    write_text(
        &fixture.join(release_doc::RELEASE_DOC_PATH),
        &release_doc_with_packages(&[
            "unified-agent-api-codex",
            "unified-agent-api-claude-code",
            "unified-agent-api-opencode",
            "unified-agent-api-gemini-cli",
        ]),
    );

    let report = check_agent_drift(&fixture, "opencode").expect("drift report");
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.category == DriftCategory::ReleaseDoc)
        .expect("release doc finding");
    assert!(finding.summary.contains("unified-agent-api-wrapper-events"));
    assert!(finding.summary.contains("unified-agent-api"));
}

#[test]
fn check_agent_drift_reports_release_doc_duplicate_package_entries() {
    let fixture = fixture_root("agent-maintenance-drift-release-doc-duplicate-package");
    seed_publication_inputs(&fixture);

    write_text(
        &fixture.join(release_doc::RELEASE_DOC_PATH),
        &release_doc_with_packages(&[
            "unified-agent-api-codex",
            "unified-agent-api-claude-code",
            "unified-agent-api-opencode",
            "unified-agent-api-opencode",
            "unified-agent-api-wrapper-events",
            "unified-agent-api",
        ]),
    );

    let report = check_agent_drift(&fixture, "opencode").expect("drift report");
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.category == DriftCategory::ReleaseDoc)
        .expect("release doc finding");
    assert!(finding.summary.contains("duplicate package"));
    assert!(finding.summary.contains("unified-agent-api-opencode"));
}

#[test]
fn check_agent_drift_entrypoint_exits_zero_for_clean_agent() {
    let fixture = fixture_root("agent-maintenance-drift-exit-clean");
    seed_publication_inputs(&fixture);

    let output = run_xtask_check_agent_drift(&fixture, "gemini_cli");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("status: clean"));
}

#[test]
fn check_agent_drift_entrypoint_exits_two_for_drift() {
    let fixture = fixture_root("agent-maintenance-drift-exit-drift");
    seed_publication_inputs(&fixture);
    seed_governance_closeouts(
        &fixture,
        &["agent_api.run", "agent_api.events", "agent_api.events.live"],
        true,
    );

    let output = run_xtask_check_agent_drift(&fixture, "opencode");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stdout).contains("status: drift_detected"));
}

#[test]
fn check_agent_drift_entrypoint_exits_two_for_unknown_agent() {
    let fixture = fixture_root("agent-maintenance-drift-exit-unknown");
    seed_publication_inputs(&fixture);

    let output = run_xtask_check_agent_drift(&fixture, "missing-agent");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing-agent"));
}

#[test]
fn check_agent_drift_entrypoint_exits_one_when_workspace_root_is_missing() {
    let dir = std::env::temp_dir().join(format!(
        "agent-maintenance-drift-no-workspace-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time after unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("create temp dir");

    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    let output = Command::new(xtask_bin)
        .arg("check-agent-drift")
        .arg("--agent")
        .arg("opencode")
        .current_dir(&dir)
        .output()
        .expect("spawn xtask check-agent-drift");

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("could not resolve workspace root"));
}

fn seed_publication_inputs(root: &Path) {
    write_text(
        &root.join("Cargo.toml"),
        "[workspace]\nmembers = [\n  \"crates/agent_api\",\n  \"crates/codex\",\n  \"crates/claude_code\",\n  \"crates/opencode\",\n  \"crates/gemini_cli\",\n  \"crates/aider\",\n  \"crates/wrapper_events\",\n  \"crates/xtask\",\n]\n",
    );
    seed_release_touchpoints(root);
    write_text(
        &root.join("docs/specs/unified-agent-api/support-matrix.md"),
        "# Support matrix\n\nManual contract text.\n",
    );

    seed_publishable_workspace_member(root, "crates/gemini_cli", "unified-agent-api-gemini-cli");
    seed_cli_manifest_root(
        root,
        "cli_manifests/codex",
        &["x86_64-unknown-linux-musl"],
        &[(&["mcp", "list"], &["x86_64-unknown-linux-musl"])],
    );
    seed_cli_manifest_root(
        root,
        "cli_manifests/claude_code",
        &["linux-x64"],
        &[(&["mcp", "list"], &["linux-x64"])],
    );
    seed_cli_manifest_root(root, "cli_manifests/opencode", &["linux-x64"], &[]);
    seed_cli_manifest_root(root, "cli_manifests/gemini_cli", &["darwin-arm64"], &[]);
    seed_cli_manifest_root(root, "cli_manifests/aider", &["darwin-arm64"], &[]);

    let support_bundle =
        support_matrix::generate_publication_artifacts(root).expect("generate support publication");
    write_text(
        &root.join("cli_manifests/support_matrix/current.json"),
        &support_bundle.json,
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/support-matrix.md"),
        &support_bundle.markdown,
    );

    write_text(
        &root.join("docs/specs/unified-agent-api/capability-matrix.md"),
        &default_capability_matrix_markdown(),
    );

    seed_gemini_approval_artifact(
        root,
        "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml",
        "gemini-cli-onboarding",
    );

    let release_doc = release_doc::render_release_doc(root).expect("render release doc");
    write_text(&root.join(release_doc::RELEASE_DOC_PATH), &release_doc);
}

#[derive(Clone, Copy)]
enum LifecycleBaselineGap {
    None,
    MissingPublicationPacketPath,
    MissingPublicationPacketSha,
    MissingCloseoutBaselinePath,
}

#[derive(Clone, Copy)]
enum RuntimeEvidenceTruth {
    Stale,
    Truthful,
}

fn seed_gemini_lifecycle_baseline(root: &Path, gap: LifecycleBaselineGap) {
    let approval_path =
        "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_sha = sha256_hex(&root.join(approval_path));
    let lifecycle_path =
        root.join("docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json");
    let packet_path =
        root.join("docs/agents/lifecycle/gemini-cli-onboarding/governance/publication-ready.json");
    let closeout_path = root
        .join("docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json");

    write_text(
        &packet_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "1",
                "agent_id": "gemini_cli",
                "approval_artifact_path": approval_path,
                "approval_artifact_sha256": approval_sha,
                "lifecycle_state_path": "docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json",
                "lifecycle_state_sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
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
                    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-gemini_cli-runtime-follow-on/input-contract.json",
                    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-gemini_cli-runtime-follow-on/run-status.json",
                    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-gemini_cli-runtime-follow-on/run-summary.md",
                    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-gemini_cli-runtime-follow-on/validation-report.json",
                    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-gemini_cli-runtime-follow-on/written-paths.json",
                    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-gemini_cli-runtime-follow-on/handoff.json"
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
            .expect("serialize publication-ready packet")
        ),
    );
    write_text(
        &closeout_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "state": "closed",
                "approval_ref": approval_path,
                "approval_sha256": approval_sha,
                "approval_source": "historical-lifecycle-backfill",
                "manual_control_plane_edits": 0,
                "partial_write_incidents": 0,
                "ambiguous_ownership_incidents": 0,
                "duration_missing_reason": "Exact duration not recoverable from committed evidence.",
                "explicit_none_reason": "No residual friction remained in the committed proving-run evidence.",
                "preflight_passed": true,
                "recorded_at": "2026-04-21T11:23:09Z",
                "commit": "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8"
            }))
            .expect("serialize closeout")
        ),
    );
    let packet_sha = sha256_hex(&packet_path);
    let publication_packet_path = match gap {
        LifecycleBaselineGap::MissingPublicationPacketPath => serde_json::Value::Null,
        _ => serde_json::Value::String(
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/publication-ready.json"
                .to_string(),
        ),
    };
    let publication_packet_sha = match gap {
        LifecycleBaselineGap::MissingPublicationPacketSha => serde_json::Value::Null,
        _ => serde_json::Value::String(packet_sha),
    };
    let closeout_baseline_path = match gap {
        LifecycleBaselineGap::MissingCloseoutBaselinePath => serde_json::Value::Null,
        _ => serde_json::Value::String(
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json"
                .to_string(),
        ),
    };
    write_text(
        &lifecycle_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "1",
                "agent_id": "gemini_cli",
                "onboarding_pack_prefix": "gemini-cli-onboarding",
                "approval_artifact_path": approval_path,
                "approval_artifact_sha256": approval_sha,
                "lifecycle_stage": "closed_baseline",
                "support_tier": "publication_backed",
                "side_states": [],
                "current_owner_command": "close-proving-run --write",
                "expected_next_command": "check-agent-drift --agent gemini_cli",
                "last_transition_at": "2026-04-21T11:23:09Z",
                "last_transition_by": "historical-lifecycle-backfill",
                "required_evidence": [
                    "registry_entry",
                    "docs_pack",
                    "manifest_root_skeleton",
                    "runtime_write_complete",
                    "implementation_summary_present",
                    "publication_packet_written",
                    "support_matrix_check_green",
                    "capability_matrix_check_green",
                    "capability_matrix_audit_green",
                    "preflight_green",
                    "proving_run_closeout_written"
                ],
                "satisfied_evidence": [
                    "registry_entry",
                    "docs_pack",
                    "manifest_root_skeleton",
                    "runtime_write_complete",
                    "implementation_summary_present",
                    "publication_packet_written",
                    "support_matrix_check_green",
                    "capability_matrix_check_green",
                    "capability_matrix_audit_green",
                    "preflight_green",
                    "proving_run_closeout_written"
                ],
                "blocking_issues": [],
                "retryable_failures": [],
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
                "publication_packet_path": publication_packet_path,
                "publication_packet_sha256": publication_packet_sha,
                "closeout_baseline_path": closeout_baseline_path
            }))
            .expect("serialize lifecycle state")
        ),
    );
}

fn seed_gemini_runtime_integrated_state(root: &Path) {
    let approval_path =
        "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_sha = sha256_hex(&root.join(approval_path));
    write_text(
        &root.join("docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "1",
                "agent_id": "gemini_cli",
                "onboarding_pack_prefix": "gemini-cli-onboarding",
                "approval_artifact_path": approval_path,
                "approval_artifact_sha256": approval_sha,
                "lifecycle_stage": "runtime_integrated",
                "support_tier": "baseline_runtime",
                "side_states": [],
                "current_owner_command": "runtime-follow-on --write",
                "expected_next_command": "prepare-publication --approval docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml --write",
                "last_transition_at": "2026-05-01T00:00:00Z",
                "last_transition_by": "agent-maintenance-drift-test",
                "required_evidence": [
                    "registry_entry",
                    "docs_pack",
                    "manifest_root_skeleton",
                    "runtime_write_complete",
                    "implementation_summary_present"
                ],
                "satisfied_evidence": [
                    "registry_entry",
                    "docs_pack",
                    "manifest_root_skeleton",
                    "runtime_write_complete",
                    "implementation_summary_present"
                ],
                "blocking_issues": [],
                "retryable_failures": [],
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
            .expect("serialize runtime integrated lifecycle state")
        ),
    );
}

fn seed_runtime_evidence_run(root: &Path, truth: RuntimeEvidenceTruth) {
    let approval_path =
        "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let approval_sha = sha256_hex(&root.join(approval_path));
    let run_root = root
        .join("docs/agents/.uaa-temp/runtime-follow-on/runs/repair-gemini_cli-runtime-follow-on");
    let input_sha = match truth {
        RuntimeEvidenceTruth::Stale => {
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
        }
        RuntimeEvidenceTruth::Truthful => &approval_sha,
    };

    write_text(
        &run_root.join("run-status.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "approval_artifact_path": approval_path,
                "agent_id": "gemini_cli",
                "status": "write_validated",
                "validation_passed": true,
                "handoff_ready": true,
                "run_dir": run_root.to_string_lossy(),
            }))
            .expect("serialize runtime run status")
        ),
    );
    write_text(
        &run_root.join("input-contract.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "approval_artifact_path": approval_path,
                "approval_artifact_sha256": input_sha,
                "agent_id": "gemini_cli",
                "manifest_root": "cli_manifests/gemini_cli",
                "required_handoff_commands": [
                    "cargo run -p xtask -- support-matrix --check",
                    "cargo run -p xtask -- capability-matrix --check",
                    "cargo run -p xtask -- capability-matrix-audit",
                    "make preflight"
                ]
            }))
            .expect("serialize runtime input contract")
        ),
    );
    write_text(
        &run_root.join("validation-report.json"),
        "{\n  \"status\": \"pass\"\n}\n",
    );
    write_text(
        &run_root.join("handoff.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "agent_id": "gemini_cli",
                "manifest_root": "cli_manifests/gemini_cli",
                "runtime_lane_complete": true,
                "publication_refresh_required": true,
                "required_commands": [
                    "cargo run -p xtask -- support-matrix --check",
                    "cargo run -p xtask -- capability-matrix --check",
                    "cargo run -p xtask -- capability-matrix-audit",
                    "make preflight"
                ],
                "blockers": []
            }))
            .expect("serialize runtime handoff")
        ),
    );
    write_text(
        &run_root.join("written-paths.json"),
        concat!(
            "[\n",
            "  \"crates/gemini_cli/src/lib.rs\",\n",
            "  \"cli_manifests/gemini_cli/snapshots/default.json\"\n",
            "]\n"
        ),
    );
    write_text(
        &run_root.join("run-summary.md"),
        "# Runtime follow-on\n\nCommitted runtime evidence recorded.\n",
    );
}

fn seed_governance_closeouts(
    root: &Path,
    opencode_capabilities: &[&str],
    support_unsupported: bool,
) {
    let capability_lines = opencode_capabilities
        .iter()
        .map(|capability| format!("`{capability}`"))
        .collect::<Vec<_>>()
        .join(", ");
    write_text(
        &root.join(
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
        &format!(
            "# Closeout\n\n- capability advertisement is intentionally conservative and now matches the landed backend contract and generated capability inventory:\n  <!-- xtask-governance-check:opencode-capabilities:start -->\n  {capability_lines}\n  <!-- xtask-governance-check:opencode-capabilities:end -->\n  are the claimed OpenCode v1 capability ids under the current runtime evidence\n"
        ),
    );

    let seam3_text = if support_unsupported {
        "# Closeout\n\n- the support publication artifacts now show OpenCode as manifest-supported only where committed root evidence justifies it, while\n  <!-- xtask-governance-check:opencode-support:start -->\n  backend_support = unsupported\n  uaa_support = unsupported\n  <!-- xtask-governance-check:opencode-support:end -->\n  under the current backend evidence and pointer posture\n"
    } else {
        "# Closeout\n\n- the support publication artifacts now show OpenCode as manifest-supported only where committed root evidence justifies it, while\n  <!-- xtask-governance-check:opencode-support:start -->\n  backend_support = supported\n  uaa_support = supported\n  <!-- xtask-governance-check:opencode-support:end -->\n  under the current backend evidence and pointer posture\n"
    };
    write_text(
        &root.join("docs/integrations/opencode/governance/seam-3-closeout.md"),
        seam3_text,
    );
}

fn default_capability_matrix_markdown() -> String {
    include_str!("../../../docs/specs/unified-agent-api/capability-matrix.md").to_string()
}

fn seed_closed_governance_maintenance(root: &Path, resolved_surface: &str) {
    write_text(
        &root.join(
            "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json",
        ),
        &serde_json::to_string_pretty(&serde_json::json!({
            "resolved_findings": [{
                "category_id": "governance_doc_drift",
                "surfaces": [resolved_surface],
            }]
        }))
        .expect("serialize maintenance closeout"),
    );
}

fn run_xtask_check_agent_drift(fixture_root: &Path, agent_id: &str) -> std::process::Output {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    Command::new(xtask_bin)
        .arg("check-agent-drift")
        .arg("--agent")
        .arg(agent_id)
        .current_dir(fixture_root)
        .output()
        .expect("spawn xtask check-agent-drift")
}

fn release_doc_with_packages(packages: &[&str]) -> String {
    let packages = packages
        .iter()
        .map(|package| package.to_string())
        .collect::<Vec<_>>();
    release_doc::splice_release_doc_block(
        "# Release docs\n\nManual contract text.\n",
        &release_doc::render_release_doc_block(&packages),
    )
}

fn set_support_matrix_enabled(root: &Path, agent_id: &str, enabled: bool) {
    let registry_path = root.join("crates/xtask/data/agent_registry.toml");
    let registry = fs::read_to_string(&registry_path).expect("read registry");
    let agent_marker = format!("agent_id = \"{agent_id}\"");
    let agent_start = registry.find(&agent_marker).expect("agent in registry");
    let support_marker = "support_matrix_enabled = true";
    let support_start = registry[agent_start..]
        .find(support_marker)
        .map(|offset| agent_start + offset)
        .expect("support flag after agent entry");
    let mut updated = registry;
    updated.replace_range(
        support_start..support_start + support_marker.len(),
        if enabled {
            "support_matrix_enabled = true"
        } else {
            "support_matrix_enabled = false"
        },
    );
    write_text(&registry_path, &updated);
}

fn seed_publishable_workspace_member(root: &Path, member_path: &str, package_name: &str) {
    write_text(
        &root.join(member_path).join("Cargo.toml"),
        &format!("[package]\nname = \"{package_name}\"\nversion = \"0.2.3\"\nedition = \"2021\"\n"),
    );
}

fn seed_cli_manifest_root(
    root: &Path,
    manifest_root: &str,
    canonical_targets: &[&str],
    commands: &[(&[&str], &[&str])],
) {
    let current = serde_json::json!({
        "expected_targets": canonical_targets,
        "inputs": [{
            "target_triple": canonical_targets[0],
            "binary": { "semantic_version": "1.0.0" }
        }],
        "commands": commands
            .iter()
            .map(|(path, available_on)| serde_json::json!({
                "path": path,
                "available_on": available_on,
            }))
            .collect::<Vec<_>>(),
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
                "wrapper_only_args": [],
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
