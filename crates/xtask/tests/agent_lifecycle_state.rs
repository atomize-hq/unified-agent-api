use std::{
    fs,
    path::{Path, PathBuf},
};

use sha2::Digest;
use xtask::{
    agent_lifecycle::{
        approval_artifact_path_for_entry, file_sha256, is_resting_stage_v1,
        lifecycle_state_path_for_entry, load_lifecycle_state,
        reconstruct_publication_ready_state_from_closed_baseline, required_evidence_for_stage,
        validate_stage_support_tier, EvidenceId, LifecycleStage, LifecycleState,
        PublicationReadyPacket, SupportTier, REQUIRED_PUBLICATION_COMMANDS,
    },
    agent_registry::AgentRegistry,
    approval_artifact::load_approval_artifact,
    prepare_publication::discover_runtime_evidence_for_approval,
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

        if expected_stage != LifecycleStage::ClosedBaseline {
            continue;
        }

        assert_eq!(
            state.required_evidence,
            required_evidence_for_stage(LifecycleStage::ClosedBaseline)
        );
        assert_eq!(
            state.satisfied_evidence,
            required_evidence_for_stage(LifecycleStage::ClosedBaseline)
        );

        let packet_path = state
            .publication_packet_path
            .as_ref()
            .expect("closed baseline publication_packet_path");
        let packet_sha = state
            .publication_packet_sha256
            .as_ref()
            .expect("closed baseline publication_packet_sha256");
        let closeout_path = state
            .closeout_baseline_path
            .as_ref()
            .expect("closed baseline closeout_baseline_path");
        assert!(
            workspace_root.join(closeout_path).is_file(),
            "missing {closeout_path}"
        );
        assert_eq!(
            file_sha256(&workspace_root, packet_path).expect("hash packet"),
            *packet_sha
        );

        let packet_bytes = fs::read(workspace_root.join(packet_path)).expect("read packet bytes");
        let packet: PublicationReadyPacket =
            serde_json::from_slice(&packet_bytes).expect("parse packet");
        packet.validate().expect("packet schema validation");

        let approval = load_approval_artifact(&workspace_root, &state.approval_artifact_path)
            .expect("approval");
        let runtime_evidence = discover_runtime_evidence_for_approval(&workspace_root, &approval)
            .expect("discover runtime evidence");
        assert_eq!(
            packet.runtime_evidence_paths,
            runtime_evidence.runtime_evidence_paths
        );
        assert_eq!(
            packet.publication_owned_paths,
            vec![lifecycle_path.clone(), packet_path.clone()]
        );

        let historical_publication_state =
            reconstruct_publication_ready_state_from_closed_baseline(&state);
        historical_publication_state
            .validate()
            .expect("historical publication-ready state validates");
        assert_eq!(
            packet.lifecycle_state_sha256,
            pretty_json_sha(&historical_publication_state)
        );
    }
}

#[test]
fn closed_baseline_requires_publication_continuity_fields() {
    let mut state = sample_closed_baseline_state();
    state.publication_packet_path = None;
    state.publication_packet_sha256 = None;
    let err = state
        .validate()
        .expect_err("missing packet continuity should fail");
    assert!(err.to_string().contains("publication_packet_path"));

    let mut state = sample_closed_baseline_state();
    state.closeout_baseline_path = None;
    let err = state
        .validate()
        .expect_err("missing closeout baseline should fail");
    assert!(err.to_string().contains("closeout_baseline_path"));
}

#[test]
fn closed_baseline_requires_stage_minimum_evidence() {
    let mut state = sample_closed_baseline_state();
    state
        .required_evidence
        .retain(|evidence| *evidence != EvidenceId::PublicationPacketWritten);
    state
        .satisfied_evidence
        .retain(|evidence| *evidence != EvidenceId::PublicationPacketWritten);
    let err = state
        .validate()
        .expect_err("closed baseline missing stage minimum evidence should fail");
    assert!(
        err.to_string().contains("publication_packet_written"),
        "{}",
        err
    );
}

fn sample_closed_baseline_state() -> LifecycleState {
    let workspace_root = repo_root();
    let registry = AgentRegistry::load(&workspace_root).expect("load registry");
    let entry = registry.find("gemini_cli").expect("gemini entry");
    let lifecycle_path = lifecycle_state_path_for_entry(entry);
    load_lifecycle_state(&workspace_root, &lifecycle_path).expect("load sample lifecycle state")
}

fn pretty_json_sha<T: serde::Serialize>(value: &T) -> String {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize json");
    bytes.push(b'\n');
    hex::encode(sha2::Sha256::digest(bytes))
}
