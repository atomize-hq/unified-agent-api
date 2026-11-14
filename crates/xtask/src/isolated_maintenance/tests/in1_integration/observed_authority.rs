//! Real producer reads and IN1 publication without the socket-dependent MA4 delivery step.
use super::*;
use crate::depth_acceptance::admission::{Decision, Host, Reason};

const RUNS: &str = "actions/workflows/ci.yml/runs?created=%3E%3D2026-11-03&per_page=100&page=1";

fn authenticate(f: &Fixture, parent: &str) -> Result<(), String> {
    let route = Route(f.resolve()?);
    let host = route.context().evidence_host(f.root.path())?;
    route.with_admitted_publication(f.root.path(), parent, || {
        crate::depth_evidence::EvidenceVerdicts::assemble_view_fixture(f.root.path(), &host)
            .map(|_| ())
    })
}

#[test]
fn in1_retains_disturbed_and_restored_host_authority_without_a_conflict() {
    let (f, host) = connected::frozen(false);
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    let parent = head(&f);
    local(&f, &parent);
    let next = candidate(&f, &parent);
    authenticate(&f, &parent).unwrap();
    let first = f.api.3.records().pop().unwrap();
    for edit in 0..3 {
        let changed = host.fork();
        match edit {
            0 => changed.add_attempt(RUNS, |job| job["status"] = json!("in_progress")),
            1 => changed.add_attempt(RUNS, |job| {
                job["steps"][0]["conclusion"] = json!("failure");
                job["conclusion"] = json!("failure");
            }),
            _ => changed.revoke_adoption(),
        }
        {
            let _changed = changed.install();
            let error = authenticate(&f, &parent).unwrap_err();
            assert!(
                error.contains("missing/ambiguous attempt witness artifact")
                    || error.contains("missing exact authenticated adequacy adoption"),
                "{error}"
            );
        }
        let refusal = f.api.3.records().pop().unwrap();
        assert_eq!(refusal.decision, Decision::Refused);
        assert_ne!(refusal.observed_authority, first.observed_authority);
        assert_eq!(refusal.inputs, first.inputs);
        assert_ne!(
            refusal.inputs_identity().unwrap(),
            first.inputs_identity().unwrap()
        );
        let _restored = host.install();
        authenticate(&f, &parent).unwrap();
        let restored = f.api.3.records().pop().unwrap();
        assert_eq!(restored.observed_authority, first.observed_authority);
        assert_eq!(
            restored.inputs_identity().unwrap(),
            first.inputs_identity().unwrap()
        );
    }
    f.resolve()
        .unwrap()
        .admission_publication()
        .unwrap()
        .resolve(root, &[parent, next])
        .unwrap();
}

#[test]
fn candidate_witness_copy_cannot_select_observed_authority_or_avoid_conflicts() {
    let (f, host) = connected::frozen(false);
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    let parent = head(&f);
    local(&f, &parent);
    candidate(&f, &parent);
    authenticate(&f, &parent).unwrap();
    let first = f.api.3.records().pop().unwrap();
    // Copy the actual hosted witness archive into candidate-owned files.
    let witness = host.witness_archive();
    for copy in [
        b"candidate-selected authority".as_slice(),
        witness.as_slice(),
    ] {
        write(root, "candidate-witness.zip", copy);
        write(
            root,
            "admission.json",
            br#"{"observed_authority":"candidate-selected"}"#,
        );
        candidate(&f, &parent);
        authenticate(&f, &parent).unwrap();
        let current = f.api.3.records().pop().unwrap();
        assert_eq!(current.observed_authority, first.observed_authority);
    }
    let mut refusal = f.api.3.records().pop().unwrap();
    refusal.decision = Decision::Refused;
    refusal.reasons = vec![Reason::OwningPredicateRefused];
    let session = f
        .api
        .3
        .begin(&refusal.candidate.commit, &refusal.policy)
        .unwrap();
    refusal.claims = session.clone();
    let artifact = f
        .api
        .3
        .upload(&session, &refusal.canonical().unwrap())
        .unwrap();
    f.api
        .3
        .commit(
            &session,
            &format!(
                "IN1_ADMISSION_V1 kind=in1_admission sha256={} artifact={artifact}",
                refusal.identity().unwrap()
            ),
        )
        .unwrap();
    let error = f
        .resolve()
        .unwrap()
        .admission_publication()
        .unwrap()
        .resolve(root, &[parent, head(&f)])
        .err()
        .unwrap();
    assert!(
        error.contains("conflicting decisions over identical inputs"),
        "{error}"
    );
}
