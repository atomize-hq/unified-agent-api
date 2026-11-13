//! Coherent producer-source projections, not ON3–ON5 owner-admission proofs. ON2 outputs and
//! reporting E come from real writers; later lifecycle/entry fixtures use validated typed forms.
use super::*;
use crate::agent_lifecycle::{self as life, LifecycleStage as Stage, SupportTier};

pub(super) fn advance(f: &Fixture, record: &mut DepthRecord, stage: Stage) {
    let mut state = life::load_lifecycle_state(f.root.path(), inputs::STATE).unwrap();
    assert!(state.lifecycle_stage < stage);
    state.lifecycle_stage = stage;
    state.support_tier = if stage == Stage::PublicationReady {
        SupportTier::BaselineRuntime
    } else {
        SupportTier::PublicationBacked
    };
    state.active_runtime_evidence_run_id = None;
    state.required_evidence = life::required_evidence_for_stage(stage).to_vec();
    state.satisfied_evidence = state.required_evidence.clone();
    state.current_owner_command = match stage {
        Stage::PublicationReady => "prepare-publication --write",
        Stage::Published => "refresh-publication --write",
        Stage::ClosedBaseline => "closeout-proving-run --write",
        _ => unreachable!(),
    }
    .into();
    state.expected_next_command = match stage {
        Stage::PublicationReady => "refresh-publication --write",
        Stage::Published => "prepare-proving-run-closeout --write",
        Stage::ClosedBaseline => "none",
        _ => unreachable!(),
    }
    .into();
    state.last_transition_by = format!("xtask {}", state.current_owner_command);
    let pack = state.onboarding_pack_prefix.clone();
    let packet_path = format!("{pack}/governance/publication-ready.json");
    if stage == Stage::PublicationReady {
        state.validate().unwrap();
        let packet = life::PublicationReadyPacket {
            schema_version: state.schema_version.clone(),
            agent_id: state.agent_id.clone(),
            approval_artifact_path: inputs::APPROVAL.into(),
            approval_artifact_sha256: state.approval_artifact_sha256.clone(),
            lifecycle_state_path: inputs::STATE.into(),
            lifecycle_state_sha256: depth_ci::reference(
                "",
                &serde_json::to_vec_pretty(&state).unwrap(),
            )
            .sha256,
            lifecycle_stage: Stage::PublicationReady,
            support_tier_at_emit: SupportTier::BaselineRuntime,
            manifest_root: "cli_manifests/depth_fixture".into(),
            expected_targets: vec![depth_ci::TARGET.into()],
            capability_publication_enabled: false,
            support_publication_enabled: true,
            capability_matrix_target: None,
            required_commands: life::REQUIRED_PUBLICATION_COMMANDS
                .iter()
                .map(|s| (*s).into())
                .collect(),
            required_publication_outputs: vec![
                crate::support_matrix::JSON_OUTPUT_PATH.into(),
                crate::support_matrix::MARKDOWN_OUTPUT_PATH.into(),
            ],
            runtime_evidence_paths: vec![],
            publication_owned_paths: vec![inputs::STATE.into(), packet_path],
            blocking_issues: vec![],
            implementation_summary: state.implementation_summary.clone().unwrap(),
        };
        packet.validate().unwrap();
        write(
            f.root.path(),
            &packet.publication_owned_paths[1],
            serde_json::to_vec_pretty(&packet).unwrap(),
        );
    } else if stage == Stage::Published {
        let packet = depth_ci::reference(
            &packet_path,
            &fs::read(f.root.path().join(&packet_path)).unwrap(),
        );
        state.publication_packet_path = Some(packet.path.clone());
        state.publication_packet_sha256 = Some(packet.sha256.clone());
        record
            .acceptance_entries
            .push(crate::depth_record::AcceptanceEntry::Publication {
                packet,
                record_sha256: record.stated_sha256().unwrap(),
            });
    } else {
        use crate::proving_run_closeout::*;
        let maintenance = crate::approval_artifact::parse_approval_maintenance(
            &fs::read_to_string(f.root.path().join(inputs::APPROVAL)).unwrap(),
            inputs::APPROVAL,
        )
        .unwrap();
        let crate::approval_artifact::ApprovalMaintenanceMode::ExplicitlyDeferred {
            deferral_sha256,
            ..
        } = maintenance.mode
        else {
            panic!("fixture must settle deferred maintenance")
        };
        let closeout = build_closeout(
            ProvingRunCloseoutState::Closed,
            ProvingRunCloseoutMachineFields {
                approval_ref: inputs::APPROVAL.into(),
                approval_sha256: state.approval_artifact_sha256.clone(),
                approval_source: "governance-review".into(),
                maintenance_settlement: Some(MaintenanceSettlement {
                    mode: MaintenanceSettlementMode::ExplicitlyDeferred,
                    approval_section_sha256: maintenance.section_sha256,
                    release_watch_sha256: None,
                    deferral_sha256: Some(deferral_sha256),
                }),
                preflight_passed: true,
                recorded_at: state.last_transition_at.clone(),
                commit: git(f.root.path(), &["rev-parse", "HEAD"]),
            },
            ProvingRunCloseoutHumanFields {
                manual_control_plane_edits: 0,
                partial_write_incidents: 0,
                ambiguous_ownership_incidents: 0,
                duration: DurationTruth::MissingReason(
                    "In-process producer projection; no independent host run".into(),
                ),
                residual_friction: ResidualFrictionTruth::ExplicitNone(
                    "Producer authentication fixture only".into(),
                ),
            },
        )
        .unwrap();
        let closeout_path = format!("{pack}/governance/proving-run-closeout.json");
        let reference = write(
            f.root.path(),
            &closeout_path,
            render_closeout_json(&closeout).unwrap(),
        );
        state.closeout_baseline_path = Some(closeout_path);
        record
            .acceptance_entries
            .push(crate::depth_record::AcceptanceEntry::ProvingRunCloseout {
                closeout: reference,
                record_sha256: record.stated_sha256().unwrap(),
            });
    }
    state.validate().unwrap();
    write(
        f.root.path(),
        inputs::STATE,
        serde_json::to_vec_pretty(&state).unwrap(),
    );
    // Normal typed record parsing checks E and entry shape. Owner predicates are later slices.
    DepthRecord::parse(
        &serde_json::to_string(record).unwrap(),
        P.agent_id(),
        P.version(),
    )
    .unwrap();
    write(
        f.root.path(),
        inputs::RECORD,
        serde_json::to_vec_pretty(record).unwrap(),
    );
}

pub(super) fn reporting(f: &Fixture, resolved: &Resolved) -> DepthRecord {
    let assembled = assembly::assemble(resolved).unwrap();
    for (file, bytes) in assembled.files {
        write(f.root.path(), &file.path, bytes);
    }
    let mut record = resolved.original.clone();
    record.execution = Some(assembled.execution);
    record
}

/// Exercise the real complete-population evaluator using immutable inputs already authenticated
/// by discovery. Preserve later entries; materialize E with the real assembly writer.
pub(super) fn evaluate_population(
    f: &Fixture,
    resolved: &Resolved,
    obligation: Option<ObligationRef>,
) -> Verdict {
    let assembly = assembly::assemble(resolved).unwrap();
    for (file, bytes) in assembly.files {
        write(f.root.path(), &file.path, bytes);
    }
    let mut record = DepthRecord::parse(
        &fs::read_to_string(f.root.path().join(inputs::RECORD)).unwrap(),
        P.agent_id(),
        P.version(),
    )
    .unwrap();
    record.execution = Some(assembly.execution);
    write(
        f.root.path(),
        inputs::RECORD,
        serde_json::to_vec_pretty(&record).unwrap(),
    );
    git(f.root.path(), &["add", "."]);
    let obligation = obligation.unwrap_or_else(|| P.obligations(&record).remove(0));
    let current = Evaluation {
        current: profile::scope_for(&record, P, obligation.clone()),
        definition_sha256: resolved.definition.clone(),
        record,
        environment: resolved.runs[0].witness.environment.clone(),
        toolchain: resolved.runs[0].witness.build.toolchain.clone(),
    };
    evaluate(
        f.root.path(),
        &assembly.selected[&obligation],
        &current,
        resolved,
    )
    .unwrap()
}
