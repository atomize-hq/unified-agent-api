//! Explicitly passed hosts for the immutable admission-transport proof fixture.
use super::*;
use crate::depth_acceptance::admission::{fake::MemoryHost, Publication};

pub(crate) fn fixture() -> (tempfile::TempDir, Publication, Arc<MemoryHost>) {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let tip = head(&f);
    local(&f, &tip);
    candidate(&f, &tip);
    // Exercise the real IN1 predicate with this captured route, without a thread-local host.
    Route(f.resolve().unwrap())
        .with_admitted_publication(f.root.path(), &tip, || Ok(()))
        .unwrap();
    f.host();
    local(&f, &head(&f));
    let publication = f.resolve().unwrap().admission_publication().unwrap();
    (f.root, publication, f.api.3.clone())
}

#[test]
fn in1_publication_failure_refuses_success_and_retains_an_incomplete_attempt() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let tip = head(&f);
    local(&f, &tip);
    let next = candidate(&f, &tip);
    f.api
        .3
        .fail_upload
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let error = Route(f.resolve().unwrap())
        .with_admitted_publication(f.root.path(), &tip, || Ok(()))
        .unwrap_err();
    assert!(error.contains("publication incomplete"), "{error}");
    let publication = f.resolve().unwrap().admission_publication().unwrap();
    assert!(publication
        .resolve(f.root.path(), &[tip.clone(), next])
        .is_err());
    assert!(f.api.3.records().is_empty());
}

#[test]
fn in1_retains_reached_refusals_without_granting_admission() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let tip = head(&f);
    local(&f, &tip);
    let next = candidate(&f, &tip);
    let error = Route(f.resolve().unwrap())
        .with_admitted_publication(f.root.path(), &tip, || {
            Err::<(), _>("forced owning validation refusal".into())
        })
        .unwrap_err();
    assert!(error.contains("forced owning validation refusal"));
    let records = f.api.3.records();
    assert_eq!(records.len(), 1);
    assert_eq!(
        records[0].decision,
        crate::depth_acceptance::admission::Decision::Refused
    );
    let publication = f.resolve().unwrap().admission_publication().unwrap();
    let history = publication.resolve(f.root.path(), &[tip, next]).unwrap();
    assert!(!history.eligible(&records[0].identity().unwrap()).unwrap());
}

#[test]
fn in1_missing_job_commitment_leaves_an_uploaded_attempt_incomplete() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let tip = head(&f);
    local(&f, &tip);
    let next = candidate(&f, &tip);
    f.api
        .3
        .fail_commit
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let error = Route(f.resolve().unwrap())
        .with_admitted_publication(f.root.path(), &tip, || Ok(()))
        .unwrap_err();
    assert!(error.contains("publication incomplete"));
    assert_eq!(f.api.3.records().len(), 1);
    let publication = f.resolve().unwrap().admission_publication().unwrap();
    assert!(publication.resolve(f.root.path(), &[tip, next]).is_err());
}
