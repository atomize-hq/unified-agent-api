#![allow(dead_code)]

#[path = "support/onboard_agent_harness.rs"]
mod harness;

#[path = "../src/approval_artifact.rs"]
mod approval_artifact;

use approval_artifact::{load_approval_artifact, ApprovalArtifactError};
use harness::{fixture_root, write_text};

fn seed_canonical_packet(root: &std::path::Path) {
    write_text(
        &root.join("docs/agents/selection/cli-agent-selection-packet.md"),
        "# Canonical packet\n",
    );
}

fn recommendation_approval(
    recommended_agent_id: &str,
    approved_agent_id: &str,
    onboarding_pack_prefix: &str,
    override_reason: Option<&str>,
    comparison_ref: &str,
) -> String {
    let mut contents = format!(
        concat!(
            "artifact_version = \"1\"\n",
            "comparison_ref = \"{comparison_ref}\"\n",
            "selection_mode = \"factory_validation\"\n",
            "recommended_agent_id = \"{recommended_agent_id}\"\n",
            "approved_agent_id = \"{approved_agent_id}\"\n",
            "approval_commit = \"deadbeef\"\n",
            "approval_recorded_at = \"2026-04-27T19:00:00Z\"\n",
        ),
        comparison_ref = comparison_ref,
        recommended_agent_id = recommended_agent_id,
        approved_agent_id = approved_agent_id,
    );
    if let Some(reason) = override_reason {
        contents.push_str(&format!("override_reason = \"{reason}\"\n"));
    }
    contents.push_str(&format!(
        concat!(
            "\n",
            "[descriptor]\n",
            "agent_id = \"{approved_agent_id}\"\n",
            "display_name = \"Approved Agent\"\n",
            "crate_path = \"crates/{approved_agent_id}\"\n",
            "backend_module = \"crates/agent_api/src/backends/{approved_agent_id}\"\n",
            "manifest_root = \"cli_manifests/{approved_agent_id}\"\n",
            "package_name = \"unified-agent-api-{package_name}\"\n",
            "canonical_targets = [\"darwin-arm64\"]\n",
            "wrapper_coverage_binding_kind = \"generated_from_wrapper_crate\"\n",
            "wrapper_coverage_source_path = \"crates/{approved_agent_id}\"\n",
            "always_on_capabilities = [\"agent_api.config.model.v1\", \"agent_api.events\", \"agent_api.events.live\", \"agent_api.run\"]\n",
            "backend_extensions = []\n",
            "support_matrix_enabled = true\n",
            "capability_matrix_enabled = true\n",
            "docs_release_track = \"crates-io\"\n",
            "onboarding_pack_prefix = \"{onboarding_pack_prefix}\"\n",
        ),
        approved_agent_id = approved_agent_id,
        package_name = approved_agent_id.replace('_', "-"),
        onboarding_pack_prefix = onboarding_pack_prefix,
    ));
    contents
}

#[test]
fn generated_recommendation_approval_is_accepted_by_the_real_loader() {
    let fixture = fixture_root("recommend-approval-valid");
    seed_canonical_packet(&fixture);
    let approval_rel = "docs/agents/lifecycle/opencode-onboarding/governance/approved-agent.toml";
    write_text(
        &fixture.join(approval_rel),
        &recommendation_approval(
            "opencode",
            "opencode",
            "opencode-onboarding",
            None,
            "docs/agents/selection/cli-agent-selection-packet.md",
        ),
    );

    let artifact = load_approval_artifact(&fixture, approval_rel).expect("artifact should load");
    assert_eq!(artifact.descriptor.agent_id, "opencode");
    assert_eq!(
        artifact.descriptor.onboarding_pack_prefix,
        "opencode-onboarding"
    );
}

#[test]
fn override_artifact_without_override_reason_is_rejected() {
    let fixture = fixture_root("recommend-approval-override-missing");
    seed_canonical_packet(&fixture);
    let approval_rel = "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    write_text(
        &fixture.join(approval_rel),
        &recommendation_approval(
            "opencode",
            "gemini_cli",
            "gemini-cli-onboarding",
            None,
            "docs/agents/selection/cli-agent-selection-packet.md",
        ),
    );

    let err = load_approval_artifact(&fixture, approval_rel).expect_err("override should fail");
    assert!(
        matches!(err, ApprovalArtifactError::Validation(message) if message.contains("override_reason"))
    );
}

#[test]
fn noncanonical_comparison_ref_is_rejected() {
    let fixture = fixture_root("recommend-approval-bad-comparison");
    seed_canonical_packet(&fixture);
    let approval_rel = "docs/agents/lifecycle/opencode-onboarding/governance/approved-agent.toml";
    write_text(
        &fixture.join(approval_rel),
        &recommendation_approval(
            "opencode",
            "opencode",
            "opencode-onboarding",
            None,
            "docs/agents/selection/generated-packet.md",
        ),
    );

    let err =
        load_approval_artifact(&fixture, approval_rel).expect_err("comparison ref should fail");
    assert!(
        matches!(err, ApprovalArtifactError::Validation(message) if message.contains("comparison_ref"))
    );
}

#[test]
fn path_prefix_mismatch_is_rejected() {
    let fixture = fixture_root("recommend-approval-pack-prefix");
    seed_canonical_packet(&fixture);
    let approval_rel = "docs/agents/lifecycle/opencode-onboarding/governance/approved-agent.toml";
    write_text(
        &fixture.join(approval_rel),
        &recommendation_approval(
            "opencode",
            "opencode",
            "different-pack",
            None,
            "docs/agents/selection/cli-agent-selection-packet.md",
        ),
    );

    let err = load_approval_artifact(&fixture, approval_rel).expect_err("pack prefix should fail");
    assert!(
        matches!(err, ApprovalArtifactError::Validation(message) if message.contains("belongs to onboarding_pack_prefix"))
    );
}
