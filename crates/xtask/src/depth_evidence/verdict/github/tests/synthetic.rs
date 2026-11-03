//! SA-P3: the read-only synthetic-wrapper-v1 verdict. It reads only the isolated repository an
//! authenticated IR-P1 binding names, and admits an attempt once its producer job is terminal
//! (IR-P3), while the enclosing workflow may still run. Machinery proof only.
use super::*;
use crate::isolated_maintenance::EvidenceHost;

const RUNS: &str = "actions/workflows/ci.yml/runs?created=%3E%3D2026-11-02&per_page=100&page=1";
const ATTEMPT: &str = "actions/runs/1/attempts/1";
const JOBS: &str = "actions/runs/1/attempts/1/jobs?per_page=100&page=1";

fn synthetic() -> (Fixture, EvidenceHost) {
    let f = Fixture::new_for(Profile::Synthetic);
    let host = EvidenceHost::for_test(ISOLATED, &f.commit);
    (f, host)
}
fn running(f: &mut Fixture) {
    f.api.change(RUNS, |v| {
        v["workflow_runs"][0]["status"] = json!("in_progress")
    });
    f.api
        .change(ATTEMPT, |v| v["status"] = json!("in_progress"));
}
fn refusal(f: &Fixture, p: Profile, host: Option<&EvidenceHost>) -> String {
    resolve_profile_in(&f.api, f.root.path(), p, host)
        .err()
        .expect("refused")
}

#[test]
fn terminal_producer_job_verifies_every_due_obligation_while_the_workflow_runs() {
    let (mut f, host) = synthetic();
    running(&mut f);
    let proof: Value = serde_json::from_str(
        &prove_profile_in(f.root.path(), &f.api, Profile::Synthetic, Some(&host)).unwrap(),
    )
    .unwrap();
    let obligations = proof["obligations"].as_array().unwrap();
    assert_eq!(obligations.len(), 11);
    for o in obligations {
        assert_eq!(o["result"], "verified", "{o}");
        assert_eq!(o["obligation"]["target"], "x86_64-unknown-linux-gnu");
        assert!(o["obligation"]["capability"].is_null());
    }
    assert_eq!(proof["result"], "verified");
    assert_eq!(proof["production_enrollment"], false);
    let resolved =
        resolve_profile_in(&f.api, f.root.path(), Profile::Synthetic, Some(&host)).unwrap();
    for run in &resolved.runs {
        let r = &run.receipt;
        assert_eq!(r.producer.repository, ISOLATED);
        assert_eq!(r.scope.agent_id, "depth_fixture");
        assert_eq!(r.adequacy.endorsement.repository, ISOLATED);
        assert!(receipt_ref(r).unwrap().path.starts_with(
            "cli_manifests/depth_fixture/reports/0.2.0/evidence/synthetic-wrapper-v1/1-1/"
        ));
    }
}

#[test]
fn synthetic_requires_the_authenticated_isolated_binding() {
    let (f, host) = synthetic();
    for error in [
        refusal(&f, Profile::Synthetic, None),
        resolve_profile(&f.api, f.root.path(), Profile::Synthetic)
            .err()
            .unwrap(),
    ] {
        assert!(
            error.contains("requires an authenticated isolated evidence host"),
            "{error}"
        );
    }
    // A binding for another repository, or one naming the canonical repository, is refused.
    let other = EvidenceHost::for_test("fixture/other", &f.commit);
    assert!(refusal(&f, Profile::Synthetic, Some(&other)).contains("isolated evidence host"));
    let mut canonical = f.api.clone();
    canonical.1 = None;
    let named = EvidenceHost::for_test(depth_ci::REPOSITORY, &f.commit);
    assert!(
        resolve_profile_in(&canonical, f.root.path(), Profile::Synthetic, Some(&named)).is_err()
    );
    // Channel/CORE never take an isolated binding or an isolated reader.
    assert!(refusal(&f, Profile::Core, Some(&host)).contains("canonical repository"));
    assert!(refusal(&f, Profile::Core, None).contains("canonical repository"));
}

#[test]
fn protected_tip_must_match_the_binding() {
    let (f, _) = synthetic();
    let stale = EvidenceHost::for_test(ISOLATED, &"f".repeat(40));
    let error = refusal(&f, Profile::Synthetic, Some(&stale));
    assert!(
        error.contains("differs from the authenticated binding"),
        "{error}"
    );
}

#[test]
fn nonterminal_or_failed_producer_job_never_verifies() {
    for edit in 0..3 {
        let (mut f, host) = synthetic();
        running(&mut f);
        f.api.change(JOBS, |v| match edit {
            0 => v["jobs"][0]["status"] = json!("in_progress"),
            1 => v["jobs"][0]["steps"][0]["conclusion"] = json!("failure"),
            _ => {
                v["jobs"][0]["steps"].as_array_mut().unwrap().remove(2);
            }
        });
        let error = refusal(&f, Profile::Synthetic, Some(&host));
        let expected = if edit == 0 {
            "nonterminal/wrong-attempt producer job"
        } else {
            "step did not complete"
        };
        assert!(error.contains(expected), "producer edit {edit}: {error}");
    }
}

#[test]
fn runs_outside_the_isolated_repository_are_not_applicable() {
    let (mut f, host) = synthetic();
    f.api.change(RUNS, |v| {
        v["workflow_runs"][0]["head_repository"]["full_name"] = json!(depth_ci::REPOSITORY)
    });
    let error = refusal(&f, Profile::Synthetic, Some(&host));
    assert!(error.contains("no applicable producer attempts"), "{error}");
}

#[test]
fn core_still_requires_whole_workflow_completion() {
    let mut f = Fixture::new_for(Profile::Core);
    f.api
        .change(ATTEMPT, |v| v["status"] = json!("in_progress"));
    let error = refusal(&f, Profile::Core, None);
    assert!(error.contains("nonterminal"), "{error}");
}

#[test]
fn uninstalled_isolated_profile_yields_no_evidence_host() {
    let (f, _) = synthetic();
    let error = crate::isolated_maintenance::evidence_host(f.root.path())
        .err()
        .unwrap();
    assert!(error.contains("not installed"), "{error}");
}
