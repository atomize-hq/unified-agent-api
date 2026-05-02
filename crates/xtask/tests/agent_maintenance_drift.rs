#![allow(dead_code, unused_imports)]

use std::{fs, path::PathBuf, process::Command};

#[path = "support/agent_maintenance_drift_harness.rs"]
mod drift_harness;
#[path = "support/onboard_agent_harness.rs"]
mod harness;

mod agent_lifecycle {
    pub use xtask::agent_lifecycle::*;
}

mod prepare_publication {
    pub use xtask::prepare_publication::*;
}

mod agent_registry {
    pub use xtask::agent_registry::*;
}
mod capability_publication {
    pub use xtask::capability_publication::*;
}
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
use drift_harness::{
    release_doc_with_packages, run_xtask_check_agent_drift, seed_cli_manifest_root,
    seed_closed_governance_maintenance, seed_gemini_lifecycle_baseline,
    seed_gemini_runtime_integrated_state, seed_governance_closeouts, seed_publication_inputs,
    seed_runtime_evidence_run, set_support_matrix_enabled, LifecycleBaselineGap,
    RuntimeEvidenceTruth,
};
use harness::{fixture_root, write_text};

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
