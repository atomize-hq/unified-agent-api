use std::path::{Path, PathBuf};

use xtask::{
    agent_lifecycle::{
        approval_artifact_path_for_entry, is_resting_stage_v1, lifecycle_state_path_for_entry,
        load_lifecycle_state, required_evidence_for_stage, validate_stage_support_tier, EvidenceId,
        LifecycleStage, SupportTier, REQUIRED_PUBLICATION_COMMANDS,
    },
    agent_registry::AgentRegistry,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn resting_stage_rule_matches_v1_contract() {
    assert!(!is_resting_stage_v1(LifecycleStage::Approved));
    assert!(is_resting_stage_v1(LifecycleStage::Enrolled));
    assert!(is_resting_stage_v1(LifecycleStage::RuntimeIntegrated));
    assert!(is_resting_stage_v1(LifecycleStage::PublicationReady));
    assert!(!is_resting_stage_v1(LifecycleStage::Published));
    assert!(is_resting_stage_v1(LifecycleStage::ClosedBaseline));
}

#[test]
fn stage_support_tier_matrix_matches_plan() {
    assert!(validate_stage_support_tier(LifecycleStage::Approved, SupportTier::Bootstrap).is_ok());
    assert!(validate_stage_support_tier(LifecycleStage::Enrolled, SupportTier::Bootstrap).is_ok());
    assert!(
        validate_stage_support_tier(LifecycleStage::RuntimeIntegrated, SupportTier::Bootstrap)
            .is_ok()
    );
    assert!(validate_stage_support_tier(
        LifecycleStage::RuntimeIntegrated,
        SupportTier::BaselineRuntime
    )
    .is_ok());
    assert!(validate_stage_support_tier(
        LifecycleStage::PublicationReady,
        SupportTier::BaselineRuntime
    )
    .is_ok());
    assert!(validate_stage_support_tier(
        LifecycleStage::ClosedBaseline,
        SupportTier::PublicationBacked
    )
    .is_ok());
    assert!(
        validate_stage_support_tier(LifecycleStage::ClosedBaseline, SupportTier::FirstClass)
            .is_ok()
    );

    assert!(
        validate_stage_support_tier(LifecycleStage::Approved, SupportTier::FirstClass).is_err()
    );
    assert!(validate_stage_support_tier(
        LifecycleStage::PublicationReady,
        SupportTier::PublicationBacked
    )
    .is_err());
    assert!(
        validate_stage_support_tier(LifecycleStage::ClosedBaseline, SupportTier::Bootstrap)
            .is_err()
    );
}

#[test]
fn required_publication_command_set_is_frozen() {
    assert_eq!(
        REQUIRED_PUBLICATION_COMMANDS,
        [
            "cargo run -p xtask -- support-matrix --check",
            "cargo run -p xtask -- capability-matrix --check",
            "cargo run -p xtask -- capability-matrix-audit",
            "make preflight",
        ]
    );
}

#[test]
fn stage_minimum_evidence_helper_matches_contract() {
    assert_eq!(
        required_evidence_for_stage(LifecycleStage::Enrolled),
        &[
            EvidenceId::RegistryEntry,
            EvidenceId::DocsPack,
            EvidenceId::ManifestRootSkeleton,
        ]
    );
    assert_eq!(
        required_evidence_for_stage(LifecycleStage::RuntimeIntegrated),
        &[
            EvidenceId::RegistryEntry,
            EvidenceId::DocsPack,
            EvidenceId::ManifestRootSkeleton,
            EvidenceId::RuntimeWriteComplete,
            EvidenceId::ImplementationSummaryPresent,
        ]
    );
    assert_eq!(
        required_evidence_for_stage(LifecycleStage::PublicationReady),
        &[
            EvidenceId::RegistryEntry,
            EvidenceId::DocsPack,
            EvidenceId::ManifestRootSkeleton,
            EvidenceId::RuntimeWriteComplete,
            EvidenceId::ImplementationSummaryPresent,
            EvidenceId::PublicationPacketWritten,
        ]
    );
}

#[test]
fn backfilled_lifecycle_states_validate_for_registry_targets() {
    let workspace_root = repo_root();
    let registry = AgentRegistry::load(&workspace_root).expect("load agent registry");

    let expectations = [
        (
            "codex",
            LifecycleStage::ClosedBaseline,
            SupportTier::FirstClass,
        ),
        (
            "claude_code",
            LifecycleStage::ClosedBaseline,
            SupportTier::FirstClass,
        ),
        (
            "opencode",
            LifecycleStage::ClosedBaseline,
            SupportTier::PublicationBacked,
        ),
        (
            "gemini_cli",
            LifecycleStage::ClosedBaseline,
            SupportTier::PublicationBacked,
        ),
        (
            "aider",
            LifecycleStage::RuntimeIntegrated,
            SupportTier::BaselineRuntime,
        ),
    ];

    for (agent_id, expected_stage, expected_tier) in expectations {
        let entry = registry.find(agent_id).expect("registry entry");
        let lifecycle_path = lifecycle_state_path_for_entry(entry);
        let approval_path = approval_artifact_path_for_entry(entry);
        let state = load_lifecycle_state(&workspace_root, &lifecycle_path)
            .unwrap_or_else(|err| panic!("validate {lifecycle_path}: {err}"));

        assert_eq!(state.agent_id, agent_id);
        assert_eq!(state.lifecycle_stage, expected_stage);
        assert_eq!(state.support_tier, expected_tier);
        assert_eq!(state.approval_artifact_path, approval_path);
    }
}
