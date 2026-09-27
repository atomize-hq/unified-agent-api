use std::{
    fs,
    path::{Path, PathBuf},
};

use sha2::Digest;
use xtask::{
    agent_lifecycle::{
        approval_artifact_path_for_entry, file_sha256, is_resting_stage_v1,
        lifecycle_state_path_for_entry, publication_ready_expected_next_commands,
        publication_ready_refresh_command, published_prepare_closeout_command,
        reconstruct_publication_ready_state_from_closed_baseline, required_evidence_for_stage,
        validate_stage_support_tier, EvidenceId, LifecycleStage, LifecycleState,
        PublicationReadyPacket, SupportTier, REQUIRED_PUBLICATION_COMMANDS,
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
    assert!(is_resting_stage_v1(LifecycleStage::Published));
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
fn publication_ready_next_command_templates_match_refresh_then_prepare_closeout_contract() {
    let approval_path =
        "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
    let expected = publication_ready_expected_next_commands(approval_path, "gemini-cli-onboarding");
    assert_eq!(
        expected,
        [
            publication_ready_refresh_command(approval_path),
            published_prepare_closeout_command(approval_path),
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
        ("codex", LifecycleStage::Published, SupportTier::FirstClass),
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
            LifecycleStage::ClosedBaseline,
            SupportTier::PublicationBacked,
        ),
    ];

    for (agent_id, expected_stage, expected_tier) in expectations {
        let entry = registry.find(agent_id).expect("registry entry");
        let lifecycle_path = lifecycle_state_path_for_entry(entry);
        let approval_path = approval_artifact_path_for_entry(entry);
        let state = load_repo_lifecycle_state_for_contract_test(&workspace_root, &lifecycle_path)
            .unwrap_or_else(|err| panic!("validate {lifecycle_path}: {err}"));

        assert_eq!(state.agent_id, agent_id);
        assert_eq!(state.lifecycle_stage, expected_stage);
        assert_eq!(state.support_tier, expected_tier);
        assert_eq!(state.approval_artifact_path, approval_path);
        state.validate().expect("lifecycle schema validation");
        if agent_id == "claude_code" {
            state
                .validate_in_workspace(&workspace_root)
                .expect("Claude adoption linkage validates");
        }

        match expected_stage {
            LifecycleStage::PublicationReady => {
                assert_eq!(
                    state.required_evidence,
                    required_evidence_for_stage(LifecycleStage::PublicationReady)
                );
                assert_eq!(
                    state.satisfied_evidence,
                    required_evidence_for_stage(LifecycleStage::PublicationReady)
                );
                assert!(state.publication_packet_path.is_none());
                assert!(state.publication_packet_sha256.is_none());
                assert!(state.closeout_baseline_path.is_none());
            }
            LifecycleStage::Published | LifecycleStage::ClosedBaseline => {
                for evidence in required_evidence_for_stage(expected_stage) {
                    assert!(
                        state.required_evidence.contains(evidence),
                        "required_evidence for {agent_id} missing {}",
                        evidence.as_str()
                    );
                    assert!(
                        state.satisfied_evidence.contains(evidence),
                        "satisfied_evidence for {agent_id} missing {}",
                        evidence.as_str()
                    );
                }
                assert!(
                    state
                        .required_evidence
                        .contains(&EvidenceId::MaintenanceCloseoutWritten)
                        == state
                            .satisfied_evidence
                            .contains(&EvidenceId::MaintenanceCloseoutWritten),
                    "maintenance_closeout_written must appear in both evidence sets or neither"
                );

                let packet_path = state
                    .publication_packet_path
                    .as_ref()
                    .expect("publication continuity packet path");
                let packet_sha = state
                    .publication_packet_sha256
                    .as_ref()
                    .expect("publication continuity packet sha");
                assert_eq!(
                    file_sha256(&workspace_root, packet_path).expect("hash packet"),
                    *packet_sha
                );

                match expected_stage {
                    LifecycleStage::Published => {
                        assert!(state.closeout_baseline_path.is_none());
                    }
                    LifecycleStage::ClosedBaseline => {
                        let closeout_path = state
                            .closeout_baseline_path
                            .as_ref()
                            .expect("closed baseline closeout_baseline_path");
                        assert!(
                            workspace_root.join(closeout_path).is_file(),
                            "missing {closeout_path}"
                        );
                    }
                    _ => unreachable!("covered by outer match"),
                }

                let packet_bytes =
                    fs::read(workspace_root.join(packet_path)).expect("read packet bytes");
                let packet: PublicationReadyPacket =
                    serde_json::from_slice(&packet_bytes).expect("parse packet");
                packet.validate().expect("packet schema validation");
                assert_eq!(
                    packet.publication_owned_paths,
                    vec![lifecycle_path.clone(), packet_path.clone()]
                );

                if expected_stage == LifecycleStage::ClosedBaseline && agent_id != "aider" {
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
            LifecycleStage::Approved
            | LifecycleStage::Enrolled
            | LifecycleStage::RuntimeIntegrated => {
                unreachable!("unexpected registry target stage for this validation test")
            }
        }
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

#[test]
fn runtime_integrated_requires_active_runtime_evidence_run_id() {
    let mut state = sample_runtime_integrated_state();
    state.active_runtime_evidence_run_id = None;
    let err = state
        .validate()
        .expect_err("runtime_integrated missing selector should fail");
    assert!(err.to_string().contains("active_runtime_evidence_run_id"));
}

#[test]
fn non_runtime_integrated_forbids_active_runtime_evidence_run_id() {
    let mut state = sample_closed_baseline_state();
    state.active_runtime_evidence_run_id =
        Some("historical-gemini_cli-runtime-follow-on".to_string());
    let err = state
        .validate()
        .expect_err("closed_baseline selector should fail");
    assert!(err
        .to_string()
        .contains("only valid when lifecycle_stage is `runtime_integrated`"));
}

#[test]
fn runtime_integrated_rejects_invalid_active_runtime_evidence_run_id_shape() {
    let mut state = sample_runtime_integrated_state();
    state.active_runtime_evidence_run_id = Some("../escape".to_string());
    let err = state
        .validate()
        .expect_err("invalid runtime evidence run id should fail");
    assert!(err.to_string().contains("single path segment"));
}

fn sample_closed_baseline_state() -> LifecycleState {
    let workspace_root = repo_root();
    let registry = AgentRegistry::load(&workspace_root).expect("load registry");
    let entry = registry.find("gemini_cli").expect("gemini entry");
    let lifecycle_path = lifecycle_state_path_for_entry(entry);
    load_repo_lifecycle_state_for_contract_test(&workspace_root, &lifecycle_path)
        .expect("load sample lifecycle state")
}

#[test]
fn claude_adoption_requires_current_watch_and_unchanged_historical_evidence() {
    let source = repo_root();
    let fixture = tempfile::tempdir().expect("fixture");
    let root = fixture.path();
    let relative_paths = [
        "crates/xtask/data/agent_registry.toml",
        "docs/agents/lifecycle/claude-code-cli-onboarding/governance/approved-agent.toml",
        "docs/agents/lifecycle/claude-code-cli-onboarding/governance/publication-ready.json",
        "docs/agents/lifecycle/claude-code-cli-onboarding/governance/proving-run-closeout.json",
        "docs/agents/lifecycle/claude-code-cli-onboarding/governance/maintenance-readiness-adoption.json",
    ];
    for path in relative_paths {
        let destination = root.join(path);
        fs::create_dir_all(destination.parent().expect("parent")).expect("mkdir");
        fs::copy(source.join(path), destination).expect("copy evidence");
    }
    let state_path =
        "docs/agents/lifecycle/claude-code-cli-onboarding/governance/lifecycle-state.json";
    let state: LifecycleState =
        serde_json::from_slice(&fs::read(source.join(state_path)).expect("read state"))
            .expect("parse state");
    state.validate_in_workspace(root).expect("valid adoption");

    let mut missing_pointer = state.clone();
    missing_pointer.maintenance_readiness_adoption_path = None;
    missing_pointer.maintenance_readiness_adoption_sha256 = None;
    assert!(missing_pointer.validate().is_err());

    let mut another_agent = sample_closed_baseline_state();
    another_agent.maintenance_readiness_adoption_path =
        state.maintenance_readiness_adoption_path.clone();
    another_agent.maintenance_readiness_adoption_sha256 =
        state.maintenance_readiness_adoption_sha256.clone();
    assert!(another_agent.validate().is_err());

    let mut changed_approval = state.clone();
    changed_approval.approval_artifact_sha256 = "0".repeat(64);
    assert!(changed_approval.validate().is_err());

    let publication_ref = state
        .publication_packet_path
        .as_deref()
        .expect("publication path");
    let publication_path = root.join(publication_ref);
    let mut changed_publication: serde_json::Value =
        serde_json::from_slice(&fs::read(&publication_path).expect("read publication"))
            .expect("parse publication");
    changed_publication["blocking_issues"] = serde_json::json!(["replacement"]);
    let changed_publication_bytes = format!(
        "{}\n",
        serde_json::to_string_pretty(&changed_publication).expect("serialize publication")
    );
    fs::write(&publication_path, &changed_publication_bytes).expect("tamper publication");
    let mut revised_publication_state = state.clone();
    revised_publication_state.publication_packet_sha256 = Some(hex::encode(sha2::Sha256::digest(
        changed_publication_bytes.as_bytes(),
    )));
    assert!(revised_publication_state
        .validate_in_workspace(root)
        .is_err());
    fs::copy(source.join(publication_ref), &publication_path).expect("restore publication");

    let mut historical = state.clone();
    historical.last_transition_at = "2026-02-12T09:34:04-05:00".to_string();
    historical.last_transition_by = "historical-lifecycle-backfill".to_string();
    historical.maintenance_readiness_adoption_path = None;
    historical.maintenance_readiness_adoption_sha256 = None;
    for field in [
        &mut historical.required_evidence,
        &mut historical.satisfied_evidence,
    ] {
        field.retain(|evidence| *evidence != EvidenceId::MaintenanceReadinessSettled);
    }
    historical.validate().expect("exact historical baseline");
    historical.approval_artifact_sha256 = "0".repeat(64);
    assert!(historical.validate().is_err());

    let adoption_path = root.join(
        state
            .maintenance_readiness_adoption_path
            .as_deref()
            .expect("adoption path"),
    );
    let original = fs::read(&adoption_path).expect("read adoption");
    let mut adoption: serde_json::Value =
        serde_json::from_slice(&original).expect("parse adoption");
    adoption["registry_release_watch_sha256"] = serde_json::json!("0".repeat(64));
    let changed = format!(
        "{}\n",
        serde_json::to_string_pretty(&adoption).expect("serialize")
    );
    fs::write(&adoption_path, &changed).expect("write changed adoption");
    let mut changed_state = state.clone();
    changed_state.maintenance_readiness_adoption_sha256 =
        Some(hex::encode(sha2::Sha256::digest(changed.as_bytes())));
    assert!(changed_state.validate_in_workspace(root).is_err());

    fs::write(&adoption_path, original).expect("restore adoption");
    let mut backdated_adoption: serde_json::Value =
        serde_json::from_slice(&fs::read(&adoption_path).expect("read adoption"))
            .expect("parse adoption");
    backdated_adoption["recorded_at"] = serde_json::json!("2026-02-11T00:00:00Z");
    let backdated_bytes = format!(
        "{}\n",
        serde_json::to_string_pretty(&backdated_adoption).expect("serialize")
    );
    fs::write(&adoption_path, &backdated_bytes).expect("write backdated adoption");
    let mut backdated_state = state.clone();
    backdated_state.last_transition_at = "2026-02-11T00:00:00Z".to_string();
    backdated_state.maintenance_readiness_adoption_sha256 = Some(hex::encode(
        sha2::Sha256::digest(backdated_bytes.as_bytes()),
    ));
    assert!(backdated_state.validate_in_workspace(root).is_err());

    fs::copy(
        source.join(
            state
                .maintenance_readiness_adoption_path
                .as_deref()
                .expect("path"),
        ),
        &adoption_path,
    )
    .expect("restore adoption");
    let closeout_path = root.join(
        state
            .closeout_baseline_path
            .as_deref()
            .expect("closeout path"),
    );
    let mut changed_closeout: serde_json::Value =
        serde_json::from_slice(&fs::read(&closeout_path).expect("read closeout"))
            .expect("parse closeout");
    changed_closeout["explicit_none_reason"] = serde_json::json!("replacement");
    let changed_closeout_bytes = format!(
        "{}\n",
        serde_json::to_string_pretty(&changed_closeout).expect("serialize closeout")
    );
    fs::write(&closeout_path, &changed_closeout_bytes).expect("tamper closeout");
    let mut revised_adoption: serde_json::Value =
        serde_json::from_slice(&fs::read(&adoption_path).expect("read adoption"))
            .expect("parse adoption");
    revised_adoption["proving_run_closeout_sha256"] = serde_json::json!(hex::encode(
        sha2::Sha256::digest(changed_closeout_bytes.as_bytes())
    ));
    let revised_adoption_bytes = format!(
        "{}\n",
        serde_json::to_string_pretty(&revised_adoption).expect("serialize adoption")
    );
    fs::write(&adoption_path, &revised_adoption_bytes).expect("write revised adoption");
    let mut revised_state = state.clone();
    revised_state.maintenance_readiness_adoption_sha256 = Some(hex::encode(sha2::Sha256::digest(
        revised_adoption_bytes.as_bytes(),
    )));
    assert!(revised_state.validate_in_workspace(root).is_err());
}

fn sample_runtime_integrated_state() -> LifecycleState {
    let workspace_root = repo_root();
    let registry = AgentRegistry::load(&workspace_root).expect("load registry");
    let entry = registry.find("aider").expect("aider entry");
    let lifecycle_path = lifecycle_state_path_for_entry(entry);
    let mut state = load_repo_lifecycle_state_for_contract_test(&workspace_root, &lifecycle_path)
        .expect("load aider lifecycle");
    state.lifecycle_stage = LifecycleStage::RuntimeIntegrated;
    state.support_tier = SupportTier::BaselineRuntime;
    state.current_owner_command = "runtime-follow-on --write".to_string();
    state.expected_next_command = format!(
        "prepare-publication --approval {} --write",
        state.approval_artifact_path
    );
    state.required_evidence =
        required_evidence_for_stage(LifecycleStage::RuntimeIntegrated).to_vec();
    state.satisfied_evidence =
        required_evidence_for_stage(LifecycleStage::RuntimeIntegrated).to_vec();
    state.active_runtime_evidence_run_id = Some("synthetic-runtime-integrated".to_string());
    state.publication_packet_path = None;
    state.publication_packet_sha256 = None;
    state.closeout_baseline_path = None;
    state
}

fn load_repo_lifecycle_state_for_contract_test(
    workspace_root: &Path,
    lifecycle_path: &str,
) -> Result<LifecycleState, String> {
    let bytes = fs::read(workspace_root.join(lifecycle_path))
        .map_err(|err| format!("read {lifecycle_path}: {err}"))?;
    let state: LifecycleState =
        serde_json::from_slice(&bytes).map_err(|err| format!("parse {lifecycle_path}: {err}"))?;
    Ok(state)
}

fn pretty_json_sha<T: serde::Serialize>(value: &T) -> String {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize json");
    bytes.push(b'\n');
    hex::encode(sha2::Sha256::digest(bytes))
}
