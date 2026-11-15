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

#[test]
fn late_parent_retry_does_not_rewrite_existing_child_predecessors() {
    use crate::depth_acceptance::admission::{Host, Record};
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let tip = head(&f);
    local(&f, &tip);
    let parent = candidate(&f, &tip);
    Route(f.resolve().unwrap())
        .with_admitted_publication(f.root.path(), &tip, || Ok(()))
        .unwrap();
    let first = f.api.3.records().pop().unwrap();
    f.host();
    local(&f, &parent);
    let child = candidate(&f, &parent);
    Route(f.resolve().unwrap())
        .with_admitted_publication(f.root.path(), &parent, || Ok(()))
        .unwrap();
    let retained = f
        .api
        .3
        .records()
        .into_iter()
        .find(|record| record.candidate.commit == child)
        .unwrap();
    let bytes = retained.canonical().unwrap();
    assert_eq!(retained.predecessors, vec![first.identity().unwrap()]);
    // Re-evaluate the original parent under its original protected context after the child
    // has been retained. The scheduler starts attempt two in that same run at emitter.begin.
    git(f.root.path(), &["reset", "--hard", &tip]);
    f.host();
    local(&f, &tip);
    git(f.root.path(), &["reset", "--hard", &parent]);
    f.api.3.retry_on_next_begin(&first);
    Route(f.resolve().unwrap())
        .with_admitted_publication(f.root.path(), &tip, || Ok(()))
        .unwrap();
    let retry = f
        .api
        .3
        .records()
        .into_iter()
        .find(|record| record.claims.run == first.claims.run && record.claims.attempt == 2)
        .unwrap();
    assert_eq!(
        f.api.3.inputs_identity(&retry),
        f.api.3.inputs_identity(&first)
    );
    assert_ne!(retry.claims.job, first.claims.job);
    assert_eq!(retry.predecessors, first.predecessors);
    let publication = f.resolve().unwrap().admission_publication().unwrap();
    for reverse in [false, true] {
        if reverse {
            f.api.3.change(
                "actions/workflows/ci.yml/runs?per_page=100&page=1",
                |value| {
                    value["workflow_runs"].as_array_mut().unwrap().reverse();
                },
            );
        }
        let history = publication
            .resolve(f.root.path(), &[tip.clone(), child.clone()])
            .unwrap();
        let unchanged = f
            .api
            .3
            .bytes(
                &format!(
                    "payload/{}-{}",
                    retained.claims.run, retained.claims.attempt
                ),
                65_536,
            )
            .unwrap();
        assert_eq!(unchanged, bytes);
        assert_eq!(Record::parse(&unchanged).unwrap(), retained);
        assert!(history.eligible(&retained.identity().unwrap()).unwrap());
        assert!(history.eligible(&retry.identity().unwrap()).unwrap());
    }
    // A further same-run retry checks repeated observations of the shared artifact pages.
    f.api.3.retry_on_next_begin(&retry);
    Route(f.resolve().unwrap())
        .with_admitted_publication(f.root.path(), &tip, || Ok(()))
        .unwrap();
    let history = publication.resolve(f.root.path(), &[tip, child]).unwrap();
    assert!(history.eligible(&retained.identity().unwrap()).unwrap());
    assert_eq!(
        f.api
            .3
            .bytes(
                &format!(
                    "payload/{}-{}",
                    retained.claims.run, retained.claims.attempt
                ),
                65_536
            )
            .unwrap(),
        bytes
    );
}
