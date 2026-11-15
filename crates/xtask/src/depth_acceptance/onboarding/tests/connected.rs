use super::*;

/// Called at the actual admitted ON3 checkpoint, while its captured route and evidence host live.
#[cfg(target_os = "linux")]
pub(crate) fn connected_publication(root: &Path) {
    use clap::Parser;
    let caller_before =
        crate::candidate_execution::tree::capture(root, &Default::default()).unwrap();
    let refresh = crate::publication_refresh::Args::parse_from([
        "refresh-publication",
        "--write",
        "--approval",
        APPROVAL,
    ]);
    let error = crate::publication_refresh::run_in_workspace(root, refresh, &mut Vec::new())
        .unwrap_err()
        .to_string();
    assert_eq!(
        error,
        "publication_refresh: unsupported depth write (declared enrollment)"
    );
    let prepare = crate::prepare_proving_run_closeout::Args::parse_from([
        "prepare-proving-run-closeout",
        "--write",
        "--approval",
        APPROVAL,
    ]);
    let error =
        crate::prepare_proving_run_closeout::run_in_workspace(root, prepare, &mut Vec::new())
            .unwrap_err()
            .to_string();
    assert_eq!(
        error,
        "prepare_proving_run_closeout: unsupported depth write (declared enrollment)"
    );
    assert_eq!(
        crate::candidate_execution::tree::capture(root, &Default::default()).unwrap(),
        caller_before
    );
    let captured = crate::isolated_maintenance::route(root)
        .unwrap()
        .unwrap()
        .capture(root)
        .unwrap();
    let proposal = captured.proposal(root).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal.isolated_view(root, scratch.path()).unwrap();
    let _binding = captured.bind_view(&view).unwrap();
    let committed = Source::tip(root, "HEAD").unwrap();
    let mut state: LifecycleState = serde_json::from_str(&committed.read(STATE).unwrap()).unwrap();
    advance(&mut state, LifecycleStage::Published);
    state.publication_packet_path = Some(PACKET.into());
    state.publication_packet_sha256 = Some(artifacts::digest(&committed.read(PACKET).unwrap()));
    std::fs::write(scratch.path().join(STATE), encode(&state)).unwrap();
    let record = coherent_entry(root, &proposal, &captured, &view, Kind::Publication).unwrap();
    std::fs::write(scratch.path().join(RECORD), encode(&record)).unwrap();
    let before = crate::candidate_execution::tree::capture(root, &Default::default()).unwrap();
    let bundle = evaluate_proposed(
        root,
        &proposal,
        &captured,
        &view,
        Kind::Publication,
        |current| {
            assert_eq!(current.acceptance_entries.len(), 1);
            crate::support_matrix::generate_publication_artifacts(scratch.path())
        },
    )
    .unwrap();
    let artifact: Value = serde_json::from_str(&bundle.json).unwrap();
    let row = artifact["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["agent"] == AGENT)
        .unwrap();
    assert_eq!(row["depth"]["promises"][0]["qualified"], true);
    for (path, bytes) in [
        (
            crate::support_matrix::JSON_OUTPUT_PATH,
            bundle.json.as_bytes(),
        ),
        (
            crate::support_matrix::MARKDOWN_OUTPUT_PATH,
            bundle.markdown.as_bytes(),
        ),
        (
            crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
            bundle.runtime_support_data.as_bytes(),
        ),
    ] {
        std::fs::write(scratch.path().join(path), bytes).unwrap();
    }
    evaluate_proposed(root, &proposal, &captured, &view, Kind::Publication, |_| {
        let candidate = View::with_registry(
            Source::workspace(scratch.path())?,
            captured.registry(),
            AGENT,
        )?;
        super::super::super::publication::validate_current(scratch.path(), &candidate)
    })
    .unwrap();
    readonly_gates::check_connected(root, &proposal, &captured, &view, scratch.path(), &record);
    review_round1::connected_refusals(root, &proposal, &captured, &view, scratch.path(), &record);
    proving(root, &proposal, &captured, &view, scratch.path(), &record);
    assert_eq!(
        crate::candidate_execution::tree::capture(root, &Default::default()).unwrap(),
        before
    );
}

fn proving(
    root: &Path,
    proposal: &crate::agent_maintenance::closeout_proposal::Proposal,
    captured: &crate::isolated_maintenance::CapturedRegistry,
    view: &crate::agent_maintenance::closeout_proposal::IsolatedView<'_>,
    scratch: &Path,
    published: &DepthRecord,
) {
    use crate::proving_run_closeout::*;
    let baseline = view.evaluation_source(root).unwrap().source;
    let mut state: LifecycleState = serde_json::from_str(&baseline.read(STATE).unwrap()).unwrap();
    advance(&mut state, LifecycleStage::ClosedBaseline);
    state.closeout_baseline_path = Some(CLOSEOUT.into());
    let maintenance = crate::approval_artifact::parse_approval_maintenance(
        &baseline.read(APPROVAL).unwrap(),
        APPROVAL,
    )
    .unwrap();
    let crate::approval_artifact::ApprovalMaintenanceMode::ExplicitlyDeferred {
        deferral_sha256,
        ..
    } = maintenance.mode
    else {
        panic!("connected chain must defer maintenance")
    };
    let commit = crate::git_inputs::command(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(commit.status.success());
    let closeout = build_closeout(
        ProvingRunCloseoutState::Closed,
        ProvingRunCloseoutMachineFields {
            approval_ref: APPROVAL.into(),
            approval_sha256: state.approval_artifact_sha256.clone(),
            approval_source: "private owning-predicate proof".into(),
            maintenance_settlement: Some(MaintenanceSettlement {
                mode: MaintenanceSettlementMode::ExplicitlyDeferred,
                approval_section_sha256: maintenance.section_sha256,
                release_watch_sha256: None,
                deferral_sha256: Some(deferral_sha256),
            }),
            preflight_passed: true,
            recorded_at: STAMP.into(),
            commit: String::from_utf8(commit.stdout).unwrap().trim().into(),
        },
        ProvingRunCloseoutHumanFields {
            manual_control_plane_edits: 0,
            partial_write_incidents: 0,
            ambiguous_ownership_incidents: 0,
            duration: DurationTruth::Seconds(1),
            residual_friction: ResidualFrictionTruth::ExplicitNone(
                "Private predicate proof over actual ON3 output".into(),
            ),
        },
    )
    .unwrap();
    std::fs::write(scratch.join(STATE), encode(&state)).unwrap();
    std::fs::write(
        scratch.join(CLOSEOUT),
        render_closeout_json(&closeout).unwrap(),
    )
    .unwrap();
    let mut record = published.clone();
    let candidate = View::with_registry(
        Source::workspace(scratch).unwrap(),
        captured.registry(),
        AGENT,
    )
    .unwrap();
    let expected = entry_reference(&candidate, &baseline, &record, Kind::Proving).unwrap();
    record.acceptance_entries.push(expected.clone());
    std::fs::write(scratch.join(RECORD), encode(&record)).unwrap();
    let crate::depth_acceptance::Authority::Isolated(context) = proposal.authority() else {
        panic!("missing captured authority")
    };
    let host = captured.evidence_host(root).unwrap();
    let (snapshot, mut inputs) = context.view_inputs(view, &host).unwrap();
    let candidate =
        View::with_registry(snapshot.source.clone(), captured.registry(), AGENT).unwrap();
    let admitted = acceptance(
        &candidate,
        &baseline,
        &record,
        Kind::Proving,
        || {
            crate::support_depth::current_onboarding_bindings(
                &mut inputs,
                &snapshot.source,
                &record,
            )
        },
        || {
            let (evaluated, verdicts) =
                EvidenceVerdicts::bind_view(view, context, &host, RECORD)?.evaluate()?;
            assert_eq!(evaluated, record);
            Ok(verdicts)
        },
    )
    .unwrap();
    assert_eq!(admitted, expected);
    effects::check(
        &baseline,
        &snapshot.source,
        &candidate,
        &record,
        Kind::Proving,
    )
    .unwrap();
    inputs.verify().unwrap();
    context.verify_view(view, &snapshot, &host).unwrap();
}
