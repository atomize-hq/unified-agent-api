#![allow(dead_code)]

#[path = "support/onboard_agent_harness.rs"]
mod harness;

mod agent_registry {
    pub use xtask::agent_registry::*;
}

#[path = "../src/approval_artifact.rs"]
mod approval_artifact;

use approval_artifact::{
    load_approval_artifact, load_approval_artifact_for_validation, ApprovalArtifactError,
    ApprovalMaintenanceMode,
};
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
    maintenance: &str,
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
    contents.push('\n');
    contents.push_str(maintenance);
    contents
}

fn enrolled_maintenance() -> &'static str {
    concat!(
        "[descriptor.maintenance]\n",
        "mode = \"release_watch_enrolled\"\n",
        "\n",
        "[descriptor.maintenance.release_watch]\n",
        "enabled = true\n",
        "version_policy = \"latest_stable_minus_one\"\n",
        "dispatch_kind = \"workflow_dispatch\"\n",
        "dispatch_workflow = \"agent-maintenance-release-watch.yml\"\n",
        "\n",
        "[descriptor.maintenance.release_watch.upstream]\n",
        "source_kind = \"github_releases\"\n",
        "owner = \"atomize-hq\"\n",
        "repo = \"opencode\"\n",
        "tag_prefix = \"v\"\n",
    )
}

fn deferred_maintenance(reason: &str, follow_up: &str) -> String {
    format!(
        concat!(
            "[descriptor.maintenance]\n",
            "mode = \"explicitly_deferred\"\n",
            "\n",
            "[descriptor.maintenance.deferral]\n",
            "reason = \"{reason}\"\n",
            "follow_up = \"{follow_up}\"\n",
            "approved_scope = \"create_lane_closeout\"\n",
        ),
        reason = reason,
        follow_up = follow_up,
    )
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
            enrolled_maintenance(),
        ),
    );

    let artifact = load_approval_artifact(&fixture, approval_rel).expect("artifact should load");
    assert_eq!(artifact.descriptor.agent_id, "opencode");
    assert_eq!(
        artifact.descriptor.onboarding_pack_prefix,
        "opencode-onboarding"
    );
    match artifact.maintenance.mode {
        ApprovalMaintenanceMode::ReleaseWatchEnrolled {
            release_watch_sha256,
            ..
        } => {
            assert_eq!(release_watch_sha256.len(), 64);
            assert_eq!(artifact.maintenance.section_sha256.len(), 64);
        }
        ApprovalMaintenanceMode::ExplicitlyDeferred { .. } => {
            panic!("expected release-watch-enrolled maintenance")
        }
    }
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
            enrolled_maintenance(),
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
            enrolled_maintenance(),
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
            enrolled_maintenance(),
        ),
    );

    let err = load_approval_artifact(&fixture, approval_rel).expect_err("pack prefix should fail");
    assert!(
        matches!(err, ApprovalArtifactError::Validation(message) if message.contains("belongs to onboarding_pack_prefix"))
    );
}

#[test]
fn staged_path_is_rejected_by_default_loader() {
    let fixture = fixture_root("recommend-approval-staged-default-reject");
    seed_canonical_packet(&fixture);
    let approval_rel =
        "docs/agents/lifecycle/.staging/20260427-opencode/opencode-onboarding/governance/approved-agent.toml";
    write_text(
        &fixture.join(approval_rel),
        &recommendation_approval(
            "opencode",
            "opencode",
            "opencode-onboarding",
            None,
            "docs/agents/selection/cli-agent-selection-packet.md",
            enrolled_maintenance(),
        ),
    );

    let err = load_approval_artifact(&fixture, approval_rel)
        .expect_err("default loader should reject staged path");
    assert!(
        matches!(err, ApprovalArtifactError::Validation(message) if message.contains("must be repo-relative and match"))
    );
}

#[test]
fn staged_path_is_accepted_for_validation_only() {
    let fixture = fixture_root("recommend-approval-staged-validation");
    seed_canonical_packet(&fixture);
    let approval_rel =
        "docs/agents/lifecycle/.staging/20260427-opencode/opencode-onboarding/governance/approved-agent.toml";
    write_text(
        &fixture.join(approval_rel),
        &recommendation_approval(
            "opencode",
            "opencode",
            "opencode-onboarding",
            None,
            "docs/agents/selection/cli-agent-selection-packet.md",
            enrolled_maintenance(),
        ),
    );

    let artifact = load_approval_artifact_for_validation(&fixture, approval_rel)
        .expect("validation-only loader should accept staged path");
    assert_eq!(
        artifact.descriptor.onboarding_pack_prefix,
        "opencode-onboarding"
    );
    assert_eq!(artifact.relative_path, approval_rel);
}

#[test]
fn missing_maintenance_table_is_rejected() {
    let fixture = fixture_root("recommend-approval-missing-maintenance");
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
            "",
        ),
    );

    let err = load_approval_artifact(&fixture, approval_rel)
        .expect_err("missing maintenance should fail");
    assert!(
        matches!(err, ApprovalArtifactError::Validation(message) if message.contains("missing required table `maintenance`"))
    );
}

#[test]
fn deferred_maintenance_requires_fixed_scope() {
    let fixture = fixture_root("recommend-approval-bad-maintenance-scope");
    seed_canonical_packet(&fixture);
    let approval_rel = "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let maintenance = concat!(
        "[descriptor.maintenance]\n",
        "mode = \"explicitly_deferred\"\n",
        "\n",
        "[descriptor.maintenance.deferral]\n",
        "reason = \"release watch is intentionally deferred\"\n",
        "follow_up = \"revisit after proving run closeout\"\n",
        "approved_scope = \"later\"\n",
    );
    write_text(
        &fixture.join(approval_rel),
        &recommendation_approval(
            "gemini_cli",
            "gemini_cli",
            "gemini-cli-onboarding",
            None,
            "docs/agents/selection/cli-agent-selection-packet.md",
            maintenance,
        ),
    );

    let err =
        load_approval_artifact(&fixture, approval_rel).expect_err("bad deferred scope should fail");
    assert!(
        matches!(err, ApprovalArtifactError::Validation(message) if message.contains("approved_scope"))
    );
}

#[test]
fn mixed_maintenance_branches_are_rejected() {
    let fixture = fixture_root("recommend-approval-mixed-maintenance");
    seed_canonical_packet(&fixture);
    let approval_rel = "docs/agents/lifecycle/opencode-onboarding/governance/approved-agent.toml";
    let maintenance = concat!(
        "[descriptor.maintenance]\n",
        "mode = \"release_watch_enrolled\"\n",
        "\n",
        "[descriptor.maintenance.release_watch]\n",
        "enabled = true\n",
        "version_policy = \"latest_stable_minus_one\"\n",
        "dispatch_kind = \"packet_pr\"\n",
        "\n",
        "[descriptor.maintenance.release_watch.upstream]\n",
        "source_kind = \"github_releases\"\n",
        "owner = \"atomize-hq\"\n",
        "repo = \"opencode\"\n",
        "tag_prefix = \"v\"\n",
        "\n",
        "[descriptor.maintenance.deferral]\n",
        "reason = \"not allowed\"\n",
        "follow_up = \"not allowed\"\n",
        "approved_scope = \"create_lane_closeout\"\n",
    );
    write_text(
        &fixture.join(approval_rel),
        &recommendation_approval(
            "opencode",
            "opencode",
            "opencode-onboarding",
            None,
            "docs/agents/selection/cli-agent-selection-packet.md",
            maintenance,
        ),
    );

    let err = load_approval_artifact(&fixture, approval_rel)
        .expect_err("mixed maintenance branches should fail");
    assert!(
        matches!(err, ApprovalArtifactError::Validation(message) if message.contains("must omit `descriptor.maintenance.deferral`"))
    );
}

#[test]
fn deferred_maintenance_hashes_are_whitespace_stable() {
    let fixture = fixture_root("recommend-approval-deferred-hash-stable");
    seed_canonical_packet(&fixture);
    let approval_a = "docs/agents/lifecycle/opencode-onboarding/governance/approved-agent.toml";
    let approval_b = "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    write_text(
        &fixture.join(approval_a),
        &recommendation_approval(
            "opencode",
            "opencode",
            "opencode-onboarding",
            None,
            "docs/agents/selection/cli-agent-selection-packet.md",
            &deferred_maintenance(
                "release watch rollout is deferred",
                "revisit after closeout",
            ),
        ),
    );
    write_text(
        &fixture.join(approval_b),
        &recommendation_approval(
            "gemini_cli",
            "gemini_cli",
            "gemini-cli-onboarding",
            None,
            "docs/agents/selection/cli-agent-selection-packet.md",
            &deferred_maintenance(
                "  release watch rollout is deferred  ",
                " revisit after closeout ",
            ),
        ),
    );

    let artifact_a = load_approval_artifact(&fixture, approval_a).expect("load deferred A");
    let artifact_b = load_approval_artifact(&fixture, approval_b).expect("load deferred B");

    match (&artifact_a.maintenance.mode, &artifact_b.maintenance.mode) {
        (
            ApprovalMaintenanceMode::ExplicitlyDeferred {
                deferral_sha256: sha_a,
                ..
            },
            ApprovalMaintenanceMode::ExplicitlyDeferred {
                deferral_sha256: sha_b,
                ..
            },
        ) => {
            assert_eq!(sha_a, sha_b);
            assert_eq!(
                artifact_a.maintenance.section_sha256,
                artifact_b.maintenance.section_sha256
            );
        }
        _ => panic!("expected deferred maintenance in both approvals"),
    }
}
