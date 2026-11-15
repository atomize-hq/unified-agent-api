use super::*;
use crate::depth_acceptance::admission::{Decision, Host};

pub(super) fn hidden_unmerged_ancestor_refuses_real_in1(f: &Fixture) {
    let records = f.api.3.records();
    let higher = records
        .iter()
        .find(|r| {
            r.decision == Decision::Admitted
                && r.new_state
                    .as_ref()
                    .is_some_and(|s| s.stage == life::LifecycleStage::RuntimeIntegrated)
        })
        .unwrap();
    let freeze = records
        .iter()
        .find(|r| {
            r.decision == Decision::Admitted
                && r.generation.is_some()
                && r.new_state
                    .as_ref()
                    .is_some_and(|s| s.stage == life::LifecycleStage::Enrolled)
        })
        .unwrap();
    let tip = &freeze.candidate.commit;
    git(f.root.path(), &["reset", "--hard", tip]);
    f.host();
    local(f, tip);
    git(
        f.root.path(),
        &["reset", "--hard", &higher.candidate.commit],
    );
    git(f.root.path(), &["read-tree", "--reset", "-u", tip]);
    let lower = candidate(f, tip);
    assert_eq!(
        git(f.root.path(), &["rev-parse", "HEAD^{tree}"]),
        freeze.candidate.tree
    );
    let runs = "actions/workflows/ci.yml/runs?per_page=100&page=1";
    let inventory = f.api.3.json(runs).unwrap();
    let hidden: Vec<_> = inventory["workflow_runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|run| {
            records
                .iter()
                .any(|r| r.candidate == higher.candidate && run["id"] == r.claims.run)
        })
        .cloned()
        .collect();
    assert!(!hidden.is_empty());
    let higher_ids: Vec<_> = records
        .iter()
        .filter(|r| r.candidate == higher.candidate)
        .map(|r| r.identity().unwrap())
        .collect();
    assert!(records
        .iter()
        .all(|r| r.predecessors.iter().all(|id| !higher_ids.contains(id))));
    f.api.3.change(runs, |value| {
        value["workflow_runs"]
            .as_array_mut()
            .unwrap()
            .retain(|run| !hidden.contains(run));
        value["total_count"] = json!(value["workflow_runs"].as_array().unwrap().len());
    });
    // P=T and L has T's full tree. Only authenticated ancestry through unmerged H can
    // expose the missing authority. This crosses the real IN1 generation and publication path.
    let error = check_integration(f.root.path(), tip).unwrap_err();
    assert!(error.contains("history gap"), "{error}");
    let refusal = f
        .api
        .3
        .records()
        .into_iter()
        .find(|r| r.candidate.commit == lower)
        .unwrap();
    assert_eq!(refusal.decision, Decision::Refused);
    assert_eq!(refusal.parent, freeze.candidate);
    f.api.3.change(runs, |value| {
        value["workflow_runs"]
            .as_array_mut()
            .unwrap()
            .extend(hidden);
        value["total_count"] = json!(value["workflow_runs"].as_array().unwrap().len());
    });
    legacy_inputs_over_64k_remain_admissible(f, &higher.candidate.commit);
}

fn legacy_inputs_over_64k_remain_admissible(f: &Fixture, parent: &str) {
    git(f.root.path(), &["reset", "--hard", parent]);
    f.host();
    local(f, parent);
    for path in [RECORD, STATE] {
        let mut bytes = fs::read(f.root.path().join(path)).unwrap();
        bytes.extend(vec![b' '; 65_537]);
        write(f.root.path(), path, bytes);
    }
    let next = candidate(f, parent);
    check_integration(f.root.path(), parent).unwrap();
    let record = f
        .api
        .3
        .records()
        .into_iter()
        .find(|r| r.candidate.commit == next)
        .unwrap();
    assert!(record.canonical().unwrap().len() < 65_536);
    let publication = f.resolve().unwrap().admission_publication().unwrap();
    let history = publication
        .resolve(f.root.path(), &[parent.into(), next.clone()])
        .unwrap();
    let depth = crate::depth_record::DepthRecord::parse(
        &fs::read_to_string(f.root.path().join(RECORD)).unwrap(),
        "depth_fixture",
        "0.2.0",
    )
    .unwrap();
    history
        .check_candidate(f.root.path(), &next, &depth, STATE)
        .unwrap();
    assert!(history.eligible(&record.identity().unwrap()).unwrap());
}
