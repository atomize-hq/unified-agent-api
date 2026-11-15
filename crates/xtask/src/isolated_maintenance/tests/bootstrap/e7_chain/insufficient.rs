//! Sibling population contains its genuine admitted reporting checkpoint but no execution.
use super::*;
use crate::candidate_execution::tree;

fn reporting_only(evidence: &SyntheticHost) -> SyntheticHost {
    let mut observations = evidence.snapshot();
    let mut inventories = 0;
    for (path, bytes) in &mut observations {
        if !path.contains("actions/workflows/ci.yml/runs?") {
            continue;
        }
        let mut value: Value = serde_json::from_slice(bytes).unwrap();
        let rows = value["workflow_runs"].as_array_mut().unwrap();
        let before = rows.len();
        rows.retain(|r| r["id"] != 1);
        if before != rows.len() {
            inventories += 1;
        }
        value["total_count"] = json!(rows.len());
        *bytes = serde_json::to_vec(&value).unwrap();
    }
    assert!(
        inventories > 0,
        "the substantive producer inventory must exist"
    );
    SyntheticHost::from_snapshot(observations)
}

#[test]
fn e7_connected_truthful_insufficiency_refuses_on4_and_both_on5_entries() {
    let (f, qualified) = fork_checkpoint(false);
    let root = f.root.path();
    let evidence = reporting_only(&qualified);
    let host = Arc::new(f.api.3.fork());
    let (_api, _route) = install_route(&f, root, &host);
    let _reader = evidence.install();
    let captured = crate::isolated_maintenance::route(root)
        .unwrap()
        .unwrap()
        .capture(root)
        .unwrap();
    assert_eq!(
        evidence.onboarding_population(root, &captured.evidence_host(root).unwrap()),
        (1, 0)
    );
    let parent = head(&f);
    let frozen: crate::depth_record::DepthRecord =
        serde_json::from_slice(&fs::read(root.join(RECORD)).unwrap()).unwrap();
    run(root, false).unwrap();
    run(root, true).unwrap();
    let reporting: crate::depth_record::DepthRecord =
        serde_json::from_slice(&fs::read(root.join(RECORD)).unwrap()).unwrap();
    assert_eq!(reporting.event, frozen.event);
    assert_eq!(reporting.policy, frozen.policy);
    assert_eq!(reporting.obligations, frozen.obligations);
    assert!(reporting.acceptance_entries.is_empty());
    assert!(reporting.execution.as_ref().unwrap().results.is_empty());
    assert_eq!(
        life::load_lifecycle_state(root, STATE)
            .unwrap()
            .lifecycle_stage,
        life::LifecycleStage::PublicationReady
    );
    let next = commit(root, &_api, &f.baseline, &parent);
    check_integration(root, &parent).unwrap();
    host.root(&next);
    evidence.staging(&next);
    let (_api, _route) = install_route(&f, root, &host);
    check_integration(root, &parent).unwrap();
    run(root, false).unwrap();
    let captured = crate::isolated_maintenance::route(root)
        .unwrap()
        .unwrap()
        .capture(root)
        .unwrap();
    let verdicts =
        EvidenceVerdicts::authenticate(root, &reporting, &captured.evidence_host(root).unwrap())
            .unwrap();
    let due = crate::depth_record::due(&reporting).unwrap();
    assert_eq!(due.len(), 11);
    for obligation in due {
        assert_eq!(
            verdicts.result(&reporting, &obligation),
            crate::depth_record::DepthResult::Unverified
        );
    }
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
    assert_eq!(row["depth"]["promises"].as_array().unwrap().len(), 1);
    assert_eq!(row["depth"]["promises"][0]["qualified"], false);
    let before = tree::capture(root, &Default::default()).unwrap();
    let error = public_on4::refresh(root).unwrap_err().to_string();
    assert_eq!(error, "onboarding is missing a due depth obligation: ObligationRef { operation: \"run\", id: Some(R1), capability: None, target: \"x86_64-unknown-linux-gnu\" }");
    assert_eq!(tree::capture(root, &Default::default()).unwrap(), before);
    let error = public_on5::prepare(root, false).unwrap_err().to_string();
    assert_eq!(
        error,
        "prepare_proving_run_closeout: unsupported depth write (declared enrollment)"
    );
    assert_eq!(tree::capture(root, &Default::default()).unwrap(), before);
    let error = public_on5::close(root).unwrap_err().to_string();
    assert_eq!(
        error,
        "close_proving_run: unsupported depth write (declared enrollment)"
    );
    assert_eq!(tree::capture(root, &Default::default()).unwrap(), before);
}
