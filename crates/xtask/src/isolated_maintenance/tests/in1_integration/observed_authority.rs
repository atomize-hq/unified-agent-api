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
        // Isolated host corroboration follows the new candidate commit. Re-evaluating that
        // same candidate must preserve authority despite its candidate-owned witness copies.
        assert_ne!(current.candidate, first.candidate);
        assert_eq!(current.execution_evidence, first.execution_evidence);
        authenticate(&f, &parent).unwrap();
        let retry = f.api.3.records().pop().unwrap();
        assert_eq!(retry.observed_authority, current.observed_authority);
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

#[test]
fn ordinary_comment_edit_cannot_salt_a_reached_conflict() {
    let (f, host) = connected::frozen(false);
    let _installed = f.install();
    let _reader = host.install();
    let parent = head(&f);
    local(&f, &parent);
    let next = candidate(&f, &parent);
    host.ordinary_comment("ordinary original comment");
    authenticate(&f, &parent).unwrap();
    let first = f.api.3.records().pop().unwrap();
    host.ordinary_comment("candidate edited this ordinary comment");
    let route = Route(f.resolve().unwrap());
    let evidence = route.context().evidence_host(f.root.path()).unwrap();
    let error = route
        .with_admitted_publication(f.root.path(), &parent, || {
            crate::depth_evidence::EvidenceVerdicts::assemble_view_fixture(
                f.root.path(),
                &evidence,
            )?;
            Err::<(), _>("deliberate reached contradictory decision".into())
        })
        .unwrap_err();
    assert!(
        error.contains("deliberate reached contradictory decision"),
        "{error}"
    );
    let refusal = f.api.3.records().pop().unwrap();
    assert_eq!(refusal.reasons, [Reason::OwningPredicateRefused]);
    assert_eq!(refusal.observed_authority, first.observed_authority);
    assert_eq!(
        refusal.inputs_identity().unwrap(),
        first.inputs_identity().unwrap()
    );
    let error = f
        .resolve()
        .unwrap()
        .admission_publication()
        .unwrap()
        .resolve(f.root.path(), &[parent, next])
        .err()
        .unwrap();
    assert!(
        error.contains("conflicting decisions over identical inputs"),
        "{error}"
    );
}

#[test]
fn reordered_populations_and_unrelated_workflow_progress_preserve_identity() {
    let (f, host) = connected::frozen(false);
    let _installed = f.install();
    let _reader = host.install();
    let parent = head(&f);
    local(&f, &parent);
    candidate(&f, &parent);
    host.incidental_population(RUNS, false);
    authenticate(&f, &parent).unwrap();
    let first = f.api.3.records().pop().unwrap();
    host.incidental_population(RUNS, true);
    authenticate(&f, &parent).unwrap();
    let second = f.api.3.records().pop().unwrap();
    assert_eq!(second.observed_authority, first.observed_authority);
    assert_eq!(
        second.inputs_identity().unwrap(),
        first.inputs_identity().unwrap()
    );
}

#[test]
fn late_isolated_host_flake_is_retained_incomplete_and_retry_recovers() {
    let (f, host) = connected::frozen(false);
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    let parent = head(&f);
    local(&f, &parent);
    let next = candidate(&f, &parent);
    authenticate(&f, &parent).unwrap();
    let first = f.api.3.records().pop().unwrap();
    let route = Route(f.resolve().unwrap());
    let evidence = route.context().evidence_host(root).unwrap();
    let error = route
        .with_admitted_publication(root, &parent, || {
            crate::depth_evidence::EvidenceVerdicts::assemble_view_fixture(root, &evidence)?;
            // All producer reads have completed. The next isolated read is the final captured
            // recheck in with_integration_publication. The subsequent binding recheck succeeds.
            f.api.4.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        })
        .unwrap_err();
    assert!(error.contains("observation incomplete"), "{error}");
    assert!(!f.api.4.load(std::sync::atomic::Ordering::SeqCst));
    let refusal = f.api.3.records().pop().unwrap();
    assert_eq!(refusal.decision, Decision::Refused);
    assert_eq!(refusal.reasons, [Reason::ObservationIncomplete]);
    assert_ne!(refusal.observed_authority, first.observed_authority);
    let history = f
        .resolve()
        .unwrap()
        .admission_publication()
        .unwrap()
        .resolve(root, &[parent.clone(), next.clone()])
        .unwrap();
    assert!(!history.eligible(&refusal.identity().unwrap()).unwrap());
    authenticate(&f, &parent).unwrap();
    let retry = f.api.3.records().pop().unwrap();
    assert_eq!(retry.decision, Decision::Admitted);
    assert_eq!(
        retry.inputs_identity().unwrap(),
        first.inputs_identity().unwrap()
    );
    f.resolve()
        .unwrap()
        .admission_publication()
        .unwrap()
        .resolve(root, &[parent, next])
        .unwrap();
}
