//! Actual ON4 follows the public ON3 output, with independent IN1 before protected movement.
use super::*;
use crate::publication_refresh;

mod refusals;
mod reporting;
mod rollback;

const PACKET: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/publication-ready.json";

fn refresh(root: &Path) -> Result<(), publication_refresh::Error> {
    publication_refresh::run_in_workspace(
        root,
        publication_refresh::Args::parse_from([
            "refresh-publication",
            "--write",
            "--approval",
            APPROVAL,
        ]),
        &mut Vec::new(),
    )
}

fn green(_: &Path, commands: &[String]) -> Result<(), String> {
    let expected: Value =
        serde_json::from_str(include_str!("../vectors/on4-publication.json")).unwrap();
    assert_eq!(json!(commands), expected["proposal_subprocesses"]);
    Ok(())
}

#[derive(Clone, Copy)]
pub(super) enum Phase {
    Preparation,
    Integration,
    Closeout,
    CloseoutRefusals,
    CloseoutReleaseWatch,
}

pub(super) fn prove(
    f: &Fixture,
    root: &Path,
    host: &Arc<MemoryHost>,
    evidence: &SyntheticHost,
    api: &super::super::super::fixture::Fake,
    parent: &str,
    phase: Phase,
) {
    eprintln!("ON4 connected proof enters real publication writer");
    if matches!(phase, Phase::Preparation) {
        if crate::isolated_maintenance::tests::mediation_matrix::connected_enabled() {
            crate::isolated_maintenance::tests::mediation_matrix::on4_aggregate(root);
        }
        let before = crate::candidate_execution::tree::capture(root, &Default::default()).unwrap();
        publication_refresh::set_test_gate_runner(Some(|_, _| {
            Err("injected ON4 candidate gate failure".into())
        }));
        let error = refresh(root).unwrap_err().to_string();
        assert_eq!(error, "injected ON4 candidate gate failure");
        assert_eq!(
            crate::candidate_execution::tree::capture(root, &Default::default()).unwrap(),
            before
        );
        // Corruption exists only while the real proposal A checker executes. Its restoration
        // keeps later proposal validation green if the controller ever omits check_proposed.
        publication_refresh::set_test_gate_runner(Some(green));
        let error = crate::readonly_gates::test_hooks::with_shared_changes(
            vec![(
                crate::support_matrix::JSON_OUTPUT_PATH.into(),
                Some(b"{}\n".to_vec()),
            )],
            || refresh(root),
        )
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("is stale; regenerate with `cargo run -p xtask -- support-matrix`"),
            "{error}"
        );
        assert_eq!(
            crate::candidate_execution::tree::capture(root, &Default::default()).unwrap(),
            before
        );
        refusals::preparation(root, &before);
        rollback::check(root, &before);
        return;
    }
    publication_refresh::set_test_gate_runner(Some(green));
    let captured = crate::isolated_maintenance::route(root)
        .unwrap()
        .unwrap()
        .capture(root)
        .unwrap();
    let original: crate::depth_record::DepthRecord =
        serde_json::from_slice(&fs::read(root.join(RECORD)).unwrap()).unwrap();
    let pre_entry =
        EvidenceVerdicts::authenticate(root, &original, &captured.evidence_host(root).unwrap())
            .unwrap();
    let packet = fs::read(root.join(PACKET)).unwrap();
    refresh(root).unwrap();
    publication_refresh::set_test_gate_runner(None);
    assert_eq!(fs::read(root.join(PACKET)).unwrap(), packet);
    let record = crate::depth_record::DepthRecord::parse(
        &fs::read_to_string(root.join(RECORD)).unwrap(),
        "depth_fixture",
        "0.2.0",
    )
    .unwrap();
    assert_eq!(record.acceptance_entries.len(), 1);
    let obligation = crate::depth_record::due(&record)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(
        pre_entry.result(&original, &obligation),
        crate::depth_record::DepthResult::Verified
    );
    assert_eq!(
        pre_entry.result(&record, &obligation),
        crate::depth_record::DepthResult::Unverified
    );
    assert_eq!(
        record.acceptance_entries[0],
        crate::depth_record::AcceptanceEntry::Publication {
            packet: crate::depth_record::FileRef {
                path: PACKET.into(),
                sha256: hex::encode(Sha256::digest(&packet)),
            },
            record_sha256: record.stated_sha256().unwrap(),
        }
    );
    let next = life::load_lifecycle_state(root, STATE).unwrap();
    let expected: Value =
        serde_json::from_str(include_str!("../vectors/on4-publication.json")).unwrap();
    let actual = serde_json::to_value(&next).unwrap();
    for key in [
        "lifecycle_stage",
        "support_tier",
        "current_owner_command",
        "last_transition_by",
    ] {
        assert_eq!(actual[key], expected[key], "{key}");
    }
    assert_eq!(
        next.publication_packet_sha256.as_deref(),
        Some(hex::encode(Sha256::digest(&packet)).as_str())
    );
    qualified(root);
    let candidate = commit(root, api, &f.baseline, parent);
    closure(root, parent, &candidate);
    if matches!(phase, Phase::Integration) {
        refusals::integration(f, root, api, parent, &candidate);
    }
    crate::readonly_gates::with_committed_evaluation(root, || {
        crate::readonly_gates::check_shared(root)
    })
    .unwrap();
    check_integration(root, parent).unwrap();
    assert!(host
        .records()
        .iter()
        .any(|r| r.candidate.commit == candidate && r.successful()));
    host.root(&candidate);
    evidence.staging(&candidate);
    let (_api, _route) = install_route(f, root, host);
    check_integration(root, parent).unwrap();
    qualified(root);
    crate::readonly_gates::with_committed_evaluation(root, || {
        crate::readonly_gates::check_shared(root)
    })
    .unwrap();
    if matches!(
        phase,
        Phase::Closeout | Phase::CloseoutRefusals | Phase::CloseoutReleaseWatch
    ) {
        super::public_on5::prove(f, root, host, evidence, &_api, &candidate, phase);
        return;
    }
    reporting::check(f, root, &_api, &candidate);
    refusals::unchanged(f, root, host, &candidate, evidence);
}

fn qualified(root: &Path) {
    let artifact: Value = serde_json::from_slice(
        &fs::read(root.join(crate::support_matrix::JSON_OUTPUT_PATH)).unwrap(),
    )
    .unwrap();
    let row = artifact["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["agent"] == "depth_fixture")
        .unwrap();
    assert_eq!(row["depth"]["promises"][0]["qualified"], true);
    for pointer in [
        "min_supported.txt",
        "latest_validated.txt",
        "pointers/latest_supported/x86_64-unknown-linux-gnu.txt",
        "pointers/latest_validated/x86_64-unknown-linux-gnu.txt",
    ] {
        assert_eq!(
            fs::read_to_string(root.join(ROOT).join(pointer)).unwrap(),
            "none\n"
        );
    }
    let metadata: Value =
        serde_json::from_slice(&fs::read(root.join(ROOT).join("versions/0.2.0.json")).unwrap())
            .unwrap();
    assert_eq!(metadata["status"], "snapshotted");
}

#[test]
fn public_on4_connected_publication_real_in1_and_observation() {
    connected_with(false, true, Some(Phase::Integration));
}

#[test]
fn public_on4_production_maintenance_and_neutral_writers_refuse_without_writes() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let root = f.root.path();
    let before = crate::candidate_execution::tree::capture(root, &Default::default()).unwrap();
    assert_eq!(
        refresh(root).unwrap_err().to_string(),
        "publication_refresh: unsupported depth write (declared enrollment)"
    );
    let _route = f.install();
    assert_eq!(
        refresh(root).unwrap_err().to_string(),
        "publication_refresh: unsupported depth write (declared enrollment)"
    );
    let mutation = crate::workspace_mutation::PlannedMutation::create(
        "docs/forbidden-on4.txt",
        b"forbidden".to_vec(),
    );
    assert_eq!(
        crate::workspace_mutation::apply_mutations(root, &[mutation])
            .unwrap_err()
            .to_string(),
        "neutral mutation writer: unsupported depth write (declared enrollment)"
    );
    assert_eq!(
        crate::candidate_execution::tree::capture(root, &Default::default()).unwrap(),
        before
    );
}

#[test]
fn public_on4_preparation_gate_and_concurrent_refusals_from_connected_on3() {
    connected_with(false, true, Some(Phase::Preparation));
}
