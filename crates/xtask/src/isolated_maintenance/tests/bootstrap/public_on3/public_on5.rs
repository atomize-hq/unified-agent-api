//! Both public ON5 writers consume the actual ON4 output and cross real IN1 checkpoints.
use super::*;
use crate::{
    close_proving_run,
    depth_record::{AcceptanceEntry, DepthRecord},
    prepare_proving_run_closeout,
};

mod refusals;
pub(super) mod release_watch;
mod reporting;
mod rollback;

const CLOSEOUT: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/proving-run-closeout.json";
const PACKET: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/publication-ready.json";

pub(super) fn prepare(root: &Path, check: bool) -> Result<(), prepare_proving_run_closeout::Error> {
    prepare_proving_run_closeout::run_in_workspace(
        root,
        prepare_proving_run_closeout::Args::parse_from([
            "prepare-proving-run-closeout",
            if check { "--check" } else { "--write" },
            "--approval",
            APPROVAL,
        ]),
        &mut Vec::new(),
    )
}
pub(super) fn close(root: &Path) -> Result<(), close_proving_run::Error> {
    close_proving_run::run_in_workspace(
        root,
        close_proving_run::Args::parse_from([
            "close-proving-run",
            "--approval",
            APPROVAL,
            "--closeout",
            CLOSEOUT,
        ]),
        &mut Vec::new(),
    )
}
fn literal() -> Value {
    serde_json::from_str(include_str!("../vectors/on5-closeout.json")).unwrap()
}

pub(super) fn prove(
    f: &Fixture,
    root: &Path,
    host: &Arc<MemoryHost>,
    evidence: &SyntheticHost,
    api: &super::super::super::fixture::Fake,
    published: &str,
    phase: public_on4::Phase,
) {
    let refusals = matches!(phase, public_on4::Phase::CloseoutRefusals);
    eprintln!("ON5 connected proof enters prepared reporting writer");
    let packet = fs::read(root.join(PACKET)).unwrap();
    let original: DepthRecord =
        serde_json::from_slice(&fs::read(root.join(RECORD)).unwrap()).unwrap();
    let before = crate::candidate_execution::tree::capture(root, &Default::default()).unwrap();
    if refusals {
        refusals::prepared(root, &before);
    }
    prepare(root, true).unwrap();
    assert_eq!(
        crate::candidate_execution::tree::capture(root, &Default::default()).unwrap(),
        before
    );
    prepare(root, false).unwrap();
    let draft: Value = serde_json::from_slice(&fs::read(root.join(CLOSEOUT)).unwrap()).unwrap();
    assert_eq!(draft["state"], "prepared");
    assert!(draft["maintenance_settlement"].is_null());
    assert_eq!(draft["commit"], published);
    let state = life::load_lifecycle_state(root, STATE).unwrap();
    assert_eq!(state.lifecycle_stage, life::LifecycleStage::Published);
    let prepared = commit(root, api, &f.baseline, published);
    if refusals {
        refusals::prepared_candidate(f, root, api, published, &prepared);
    }
    check_integration(root, published).unwrap();
    host.root(&prepared);
    evidence.staging(&prepared);
    let (api, _route) = install_route(f, root, host);
    check_integration(root, published).unwrap();
    // Bounded human edits are a reporting checkpoint with their own independent admission.
    let mut edited = draft.clone();
    edited["recorded_at"] = json!("2020-01-01T00:00:00Z");
    edited
        .as_object_mut()
        .unwrap()
        .remove("duration_missing_reason");
    for (key, value) in literal()["human"].as_object().unwrap() {
        edited[key] = value.clone();
    }
    write(root, CLOSEOUT, serde_json::to_vec_pretty(&edited).unwrap());
    let human = commit(root, &api, &f.baseline, &prepared);
    check_integration(root, &prepared).unwrap();
    host.root(&human);
    evidence.staging(&human);
    let (api, _route) = install_route(f, root, host);
    check_integration(root, &prepared).unwrap();
    eprintln!("ON5 connected proof enters final proving writer");
    if refusals {
        refusals::local(f, root, &api, &human);
        let before = crate::candidate_execution::tree::capture(root, &Default::default()).unwrap();
        rollback::check(root, &before);
    }
    const CLOSED_AT: &str = "2026-11-14T12:34:56Z";
    crate::depth_acceptance::onboarding::on5::proposal_hooks::with_recorded_at(CLOSED_AT, || {
        close(root)
    })
    .unwrap();
    let closed_bytes = fs::read(root.join(CLOSEOUT)).unwrap();
    let closed: Value = serde_json::from_slice(&closed_bytes).unwrap();
    let expected = literal();
    assert_eq!(closed["state"], expected["state"]);
    assert_eq!(closed["approval_source"], expected["approval_source"]);
    if matches!(phase, public_on4::Phase::CloseoutReleaseWatch) {
        release_watch::assert_settlement(&closed);
    } else {
        assert_eq!(
            closed["maintenance_settlement"]["mode"],
            expected["maintenance_mode"]
        );
    }
    assert_eq!(closed["commit"], human);
    assert_ne!(closed, edited);
    assert_eq!(closed["recorded_at"], CLOSED_AT);
    assert_ne!(closed["recorded_at"], edited["recorded_at"]);
    for (key, value) in expected["human"].as_object().unwrap() {
        assert_eq!(&closed[key], value);
    }
    let next = life::load_lifecycle_state(root, STATE).unwrap();
    let actual = serde_json::to_value(&next).unwrap();
    for key in [
        "lifecycle_stage",
        "support_tier",
        "current_owner_command",
        "last_transition_by",
    ] {
        assert_eq!(actual[key], expected[key], "{key}");
    }
    assert!(next
        .satisfied_evidence
        .contains(&life::EvidenceId::ProvingRunCloseoutWritten));
    assert!(next
        .satisfied_evidence
        .contains(&life::EvidenceId::MaintenanceReadinessSettled));
    assert_eq!(next.closeout_baseline_path.as_deref(), Some(CLOSEOUT));
    assert_eq!(next.support_depth, state.support_depth);
    assert_eq!(next.written_paths_sha256, state.written_paths_sha256);
    assert_eq!(fs::read(root.join(PACKET)).unwrap(), packet);
    let record: DepthRecord =
        serde_json::from_slice(&fs::read(root.join(RECORD)).unwrap()).unwrap();
    assert_eq!(
        &record.acceptance_entries[..1],
        original.acceptance_entries.as_slice()
    );
    assert_eq!(record.event, original.event);
    assert_eq!(record.policy, original.policy);
    assert_eq!(record.obligations, original.obligations);
    assert_eq!(record.acceptance_entries.len(), 2);
    assert_eq!(
        record.acceptance_entries[1],
        AcceptanceEntry::ProvingRunCloseout {
            closeout: crate::depth_record::FileRef {
                path: CLOSEOUT.into(),
                sha256: hex::encode(Sha256::digest(&closed_bytes))
            },
            record_sha256: record.stated_sha256().unwrap(),
        }
    );
    let candidate = commit(root, &api, &f.baseline, &human);
    closure(root, &human, &candidate);
    if refusals {
        refusals::integration(f, root, &api, &human, &candidate);
    }
    check_integration(root, &human).unwrap();
    host.root(&candidate);
    evidence.staging(&candidate);
    let (api, _route) = install_route(f, root, host);
    check_integration(root, &human).unwrap();
    if refusals {
        reporting::check(f, root, &api, &candidate);
    }
    super::e7_chain::final_state(
        root,
        host,
        matches!(phase, public_on4::Phase::CloseoutReleaseWatch),
    );
    eprintln!("ON5 real writer, IN1 admission and post-tip observation completed");
    if crate::isolated_maintenance::tests::mediation_matrix::connected_enabled() {
        crate::isolated_maintenance::tests::mediation_matrix::onboarding_checkpoint(root);
    }
}

#[test]
fn public_on5_prepared_closed_real_in1_and_observation() {
    connected_with(false, true, Some(public_on4::Phase::Closeout));
}

#[test]
fn public_on5_production_and_maintenance_refuse_without_writes() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let root = f.root.path();
    let before = crate::candidate_execution::tree::capture(root, &Default::default()).unwrap();
    for installed in [false, true] {
        let _route = installed.then(|| f.install());
        assert_eq!(
            prepare(root, false).unwrap_err().to_string(),
            "prepare_proving_run_closeout: unsupported depth write (declared enrollment)"
        );
        assert_eq!(
            close(root).unwrap_err().to_string(),
            "close_proving_run: unsupported depth write (declared enrollment)"
        );
        assert_eq!(
            crate::candidate_execution::tree::capture(root, &Default::default()).unwrap(),
            before
        );
    }
}

#[test]
fn public_on5_refusals_local_final_and_conflict_preservation() {
    connected_with(false, true, Some(public_on4::Phase::CloseoutRefusals));
}
