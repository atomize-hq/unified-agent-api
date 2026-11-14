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
        admitted.inputs_identity().unwrap(),
        refusal.inputs_identity().unwrap()
    );
    let error = publication.resolve(root, &[parent, next]).err().unwrap();
    assert!(
        error.contains("conflicting decisions over identical inputs"),
        "{error}"
    );
}
