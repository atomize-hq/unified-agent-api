use super::*;
use crate::depth_acceptance::admission::{Checkpoint, Decision, Reason};
use std::sync::atomic::Ordering;

#[test]
fn post_capture_history_failure_preserves_bootstrap_refusal_conflict() {
    post_capture_history_failure(false);
}

#[test]
fn post_capture_history_failure_preserves_predecessors_and_refusal_conflict() {
    post_capture_history_failure(true);
}

#[test]
fn failed_input_capture_cannot_reach_policy_or_publish_default_projections() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let root = f.root.path();
    let parent = head(&f);
    local(&f, &parent);
    let path = f
        .resolve()
        .unwrap()
        .admission_emitter()
        .unwrap()
        .state_path
        .unwrap();
    write(root, &path, b"{malformed authenticated bytes");
    let next = candidate(&f, &parent);
    let evaluated = std::cell::Cell::new(false);
    let error = Route(f.resolve().unwrap())
        .with_admitted_publication(root, &parent, || {
            evaluated.set(true);
            Ok(())
        })
        .unwrap_err();
    assert!(error.contains("evaluated inputs unavailable"), "{error}");
    assert!(!evaluated.get());
    assert!(f.api.3.records().is_empty());
    assert!(f
        .resolve()
        .unwrap()
        .admission_publication()
        .unwrap()
        .resolve(root, &[parent, next])
        .is_err());
}

fn post_capture_history_failure(existing_frontier: bool) {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let root = f.root.path();
    let mut parent = head(&f);
    local(&f, &parent);
    let mut expected = Vec::new();
    if existing_frontier {
        let admitted = candidate(&f, &parent);
        Route(f.resolve().unwrap())
            .with_admitted_publication(root, &parent, || Ok(()))
            .unwrap();
        expected.push(f.api.3.records().pop().unwrap().identity().unwrap());
        f.host();
        local(&f, &admitted);
        parent = admitted;
    }
    let next = candidate(&f, &parent);
    let error = Route(f.resolve().unwrap())
        .with_admitted_publication(root, &parent, || {
            // This flag ignores every read while capture is active. Only retain's history
            // reconstruction can consume it, after all decision/closing capture finishes.
            f.api
                .3
                .fail_history_after_capture
                .store(true, Ordering::SeqCst);
            Err::<(), _>("deliberate reached policy refusal".into())
        })
        .unwrap_err();
    assert!(
        error.contains("deliberate reached policy refusal"),
        "{error}"
    );
    assert!(!f.api.3.fail_history_after_capture.load(Ordering::SeqCst));
    let refusal = f.api.3.records().pop().unwrap();
    assert_eq!(refusal.decision, Decision::Refused);
    assert_eq!(refusal.predecessors, expected);
    assert_eq!(
        refusal.checkpoint,
        if existing_frontier {
            Checkpoint::Unclassified
        } else {
            Checkpoint::Bootstrap
        }
    );
    assert_eq!(
        refusal.reasons,
        [
            Reason::OwningPredicateRefused,
            Reason::ObservationIncomplete
        ]
    );
    let publication = f.resolve().unwrap().admission_publication().unwrap();
    let history = publication
        .resolve(root, &[parent.clone(), next.clone()])
        .unwrap();
    assert!(!history.eligible(&refusal.identity().unwrap()).unwrap());
    Route(f.resolve().unwrap())
        .with_admitted_publication(root, &parent, || Ok(()))
        .unwrap();
    let admitted = f.api.3.records().pop().unwrap();
    assert_eq!(admitted.decision, Decision::Admitted);
    assert_eq!(admitted.observed_authority, refusal.observed_authority);
    assert_eq!(
        f.api.3.inputs_identity(&admitted),
        f.api.3.inputs_identity(&refusal)
    );
    let error = publication.resolve(root, &[parent, next]).err().unwrap();
    assert!(
        error.contains("conflicting decisions over identical inputs"),
        "{error}"
    );
}

#[test]
fn concurrent_parent_admission_quarantines_reached_child_and_healthy_retry_recovers() {
    use crate::depth_acceptance::admission::Host;
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let root = f.root.path();
    let baseline = head(&f);
    local(&f, &baseline);
    let parent = candidate(&f, &baseline);
    Route(f.resolve().unwrap())
        .with_admitted_publication(root, &baseline, || Ok(()))
        .unwrap();
    let parent_record = f.api.3.records().pop().unwrap();
    f.host();
    local(&f, &parent);
    let child = candidate(&f, &parent);
    let runs = "actions/workflows/ci.yml/runs?per_page=100&page=1";
    let parent_population = f.api.3.json(runs).unwrap();
    // Make P's publication visible precisely during C's owning policy evaluation.
    f.api
        .3
        .put(runs, json!({"total_count":0,"workflow_runs":[]}));
    let error = Route(f.resolve().unwrap())
        .with_admitted_publication(root, &parent, || {
            f.api.3.put(runs, parent_population);
            Ok(())
        })
        .unwrap_err();
    assert!(error.contains("frontier changed"), "{error}");
    let incomplete = f.api.3.records().pop().unwrap();
    assert_eq!(incomplete.decision, Decision::Admitted);
    assert_eq!(
        incomplete.reasons,
        [
            Reason::OwningPredicatesSatisfied,
            Reason::ObservationIncomplete
        ]
    );
    assert!(incomplete.predecessors.is_empty());
    let publication = f.resolve().unwrap().admission_publication().unwrap();
    let history = publication
        .resolve(root, &[parent.clone(), child.clone()])
        .unwrap();
    assert!(!history.eligible(&incomplete.identity().unwrap()).unwrap());
    assert_eq!(
        history.candidate_predecessors(&child).unwrap(),
        vec![parent_record.identity().unwrap()]
    );
    Route(f.resolve().unwrap())
        .with_admitted_publication(root, &parent, || Ok(()))
        .unwrap();
    let complete = f.api.3.records().pop().unwrap();
    assert_eq!(complete.candidate.commit, child);
    assert!(complete.successful());
    let history = publication
        .resolve(root, &[parent.clone(), child.clone()])
        .unwrap();
    assert!(history.eligible(&complete.identity().unwrap()).unwrap());
    // An independent sibling candidate also passes through prepare, policy and publication.
    git(root, &["reset", "--hard", &parent]);
    write(root, "unrelated-candidate.txt", b"independent candidate\n");
    let other = candidate(&f, &parent);
    assert_ne!(child, other);
    Route(f.resolve().unwrap())
        .with_admitted_publication(root, &parent, || Ok(()))
        .unwrap();
    publication.resolve(root, &[parent.clone(), other]).unwrap();
    git(root, &["reset", "--hard", &child]);
    let error = Route(f.resolve().unwrap())
        .with_admitted_publication(root, &parent, || {
            Err::<(), _>("contradictory complete decision".into())
        })
        .unwrap_err();
    assert!(error.contains("contradictory complete decision"), "{error}");
    assert!(publication
        .resolve(root, &[parent, child])
        .err()
        .unwrap()
        .contains("conflicting decisions over identical inputs"));
}
