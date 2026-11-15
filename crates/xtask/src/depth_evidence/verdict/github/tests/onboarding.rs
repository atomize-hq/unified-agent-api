//! Credential-free binding over genuine committed public ON2 outputs and a fake authenticated host.
use super::*;
use crate::{depth_ci::onboarding as inputs, isolated_maintenance::EvidenceHost};

pub(crate) mod admissions;
mod checkpoints;
mod history;

const P: Profile = Profile::SyntheticOnboarding;
const RUNS: &str = "actions/workflows/ci.yml/runs?created=%3E%3D2026-11-05&per_page=100&page=1";

fn onboarding() -> (Fixture, EvidenceHost) {
    let f = Fixture::new_for(P);
    let host = EvidenceHost::for_test(ISOLATED, &f.commit);
    (f, host)
}

#[test]
fn onboarding_eleven_obligations_bind_committed_on2_and_approval_adoption() {
    let (f, host) = onboarding();
    assert_eq!(
        Profile::synthetic_in(&source(f.root.path(), ISOLATED, &f.commit).unwrap()).unwrap(),
        P
    );
    let resolved = resolve_profile_in(&f.api, f.root.path(), P, Some(&host)).unwrap();
    assert_eq!(resolved.runs.len(), 11);
    for run in &resolved.runs {
        assert!(matches!(run.receipt.scope.event, Event::Onboarding { .. }));
        assert_eq!(run.receipt.adequacy.endorsement.pull_request, 9);
    }
    let proof: Value =
        serde_json::from_str(&prove_resolved(f.root.path(), ISOLATED, &resolved).unwrap()).unwrap();
    assert_eq!(proof["result"], "verified", "{proof}");
    assert_eq!(proof["obligations"].as_array().unwrap().len(), 11);
    assert!(proof["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .all(|o| o["result"] == "verified"));
    assert_eq!(proof["production_enrollment"], false);
}

/// Add a later run at its actual API-derived source head. Adoption always names its real
/// reviewed source and mapping, so incompatible source bytes cannot be hidden by this host.
fn later(f: &mut Fixture, id: u64, event: &str, label: &str, witness: &Witness) {
    git(f.root.path(), &["add", "."]);
    git(f.root.path(), &["commit", "-qm", label]);
    let commit = git(f.root.path(), &["rev-parse", "HEAD"]);
    add_run(f, id, event, &commit, witness);
}

fn add_run(f: &mut Fixture, id: u64, event: &str, commit: &str, witness: &Witness) {
    let commit = commit.to_owned();
    let mut w = witness.clone();
    w.workflow_commit = commit.clone();
    w.build.source_commit = commit.clone();
    w.build.source_tree = git(f.root.path(), &["rev-parse", &format!("{commit}^{{tree}}")]);
    let next = fixture::endpoints_for_run(f.root.path(), P, &w, &commit, id, 2);
    let mut rows = f.api.json(&f.api.endpoint(RUNS)).unwrap()["workflow_runs"]
        .as_array()
        .unwrap()
        .clone();
    let mut run = next.json(&next.endpoint(RUNS)).unwrap()["workflow_runs"][0].clone();
    run["event"] = json!(event);
    run["pull_requests"] = json!([{"number":90 + id}]);
    rows.push(run);
    // Keep the approval-linked generation PR's original, independently reviewed adoption.
    for (key, mut value) in next.0 {
        if key.contains("/attempts/") && !key.contains("/jobs") {
            let mut attempt: Value = serde_json::from_slice(&value).unwrap();
            attempt["event"] = json!(event);
            attempt["pull_requests"] = json!([{"number":90 + id}]);
            value = serde_json::to_vec(&attempt).unwrap();
        }
        if key.contains("/actions/") && !key.ends_with(RUNS) {
            f.api.0.insert(key, value);
        }
    }
    f.api
        .page(RUNS.split('&').next().unwrap(), Some("workflow_runs"), rows);
}

fn all_running(api: &mut Fake) {
    api.change(RUNS, |v| {
        for run in v["workflow_runs"].as_array_mut().unwrap() {
            run["status"] = json!("in_progress");
        }
    });
    let keys: Vec<_> = api
        .0
        .keys()
        .filter(|k| k.contains("/attempts/") && !k.contains("/jobs"))
        .cloned()
        .collect();
    for key in keys {
        let mut value: Value = serde_json::from_slice(&api.0[&key]).unwrap();
        value["status"] = json!("in_progress");
        api.0.insert(key, serde_json::to_vec(&value).unwrap());
    }
}

struct PopulationChanges<'a> {
    api: &'a Fake,
    reads: std::cell::Cell<usize>,
    field: &'static str,
    value: Value,
}
impl Api for PopulationChanges<'_> {
    fn repository(&self) -> &str {
        self.api.repository()
    }
    fn bytes(&self, endpoint: &str, limit: usize) -> Result<Vec<u8>, String> {
        let bytes = self.api.bytes(endpoint, limit)?;
        if endpoint.ends_with(RUNS) {
            let read = self.reads.get();
            self.reads.set(read + 1);
            if read > 0 {
                let mut rows: Value = serde_json::from_slice(&bytes).unwrap();
                rows["workflow_runs"][2][self.field] = self.value.clone();
                return Ok(serde_json::to_vec(&rows).unwrap());
            }
        }
        Ok(bytes)
    }
}

#[test]
fn onboarding_same_generation_later_runs_wait_for_producer_not_workflow() {
    let (mut f, _) = onboarding();
    f.install(2);
    let w = f.w.clone();
    let initial_host = EvidenceHost::for_test(ISOLATED, &f.commit);
    let initial = resolve_profile_in(&f.api, f.root.path(), P, Some(&initial_host)).unwrap();
    let mut record = checkpoints::reporting(&f, &initial);
    let freeze = git(f.root.path(), &["rev-parse", "producer-freeze"]);
    add_run(&mut f, 5, "pull_request", &freeze, &w);
    use crate::agent_lifecycle::LifecycleStage as Stage;
    checkpoints::advance(&f, &mut record, Stage::PublicationReady);
    later(&mut f, 2, "pull_request", "publication-ready", &w);
    checkpoints::advance(&f, &mut record, Stage::Published);
    later(&mut f, 3, "merge_group", "publication", &w);
    checkpoints::advance(&f, &mut record, Stage::ClosedBaseline);
    later(&mut f, 4, "push", "proving", &w);
    write(f.root.path(), "docs/post-tip.md", "post-tip observation");
    later(&mut f, 6, "push", "post-tip", &w);
    all_running(&mut f.api);
    let tip = git(f.root.path(), &["rev-parse", "HEAD"]);
    f.api
        .put("git/ref/heads/staging", json!({"object":{"sha":tip}}));
    let host = EvidenceHost::for_test(ISOLATED, &tip);
    let resolved = resolve_profile_in(&f.api, f.root.path(), P, Some(&host)).unwrap();
    let attempts: std::collections::BTreeSet<_> = resolved
        .runs
        .iter()
        .map(|r| (r.receipt.producer.run_id, r.receipt.producer.run_attempt))
        .collect();
    assert_eq!(
        attempts,
        (1..=6)
            .flat_map(|id| (1..=2).map(move |attempt| (id, attempt)))
            .collect()
    );
    assert_eq!(resolved.runs.len(), 12 * 11);
    // The separate eleven-obligation proof covers the full verdict/publisher surface. Here
    // authenticate all eleven for every attempt, then evaluate the complete attempt population
    // for one obligation; source/currentness work need not be repeated eleven times per head.
    let verdict = checkpoints::evaluate_population(&f, &resolved, None);
    assert_eq!(verdict.result, DepthResult::Verified, "{verdict:?}");
    // This is the same authenticated population an IN1 consumer evaluates after terminal
    // producer binding. Actual protected IN1/needs scheduling is the independent phase C proof.
    // Cover each applicable attempt, including every later retry. Distribute the producer
    // failure modes across those attempts so earlier green evidence cannot erase any of them.
    for (index, (id, attempt)) in attempts.iter().enumerate() {
        let jobs = format!("actions/runs/{id}/attempts/{attempt}/jobs?per_page=100&page=1");
        let mut api = f.api.clone();
        api.change(&jobs, |v| match index % 6 {
            0 => v["jobs"][0]["status"] = json!("in_progress"),
            1 => v["jobs"] = json!([]),
            2 => v["jobs"][0]["conclusion"] = json!("cancelled"),
            3 => v["jobs"][0]["conclusion"] = Value::Null,
            4 => v["jobs"][0]["steps"][0]["conclusion"] = json!("failure"),
            _ => v["jobs"][0]["steps"].as_array_mut().unwrap().clear(),
        });
        assert!(
            artifact::witness(
                &api,
                &fs::read_to_string(f.root.path().join(depth_ci::ARCHIVE)).unwrap(),
                &api.json(&api.endpoint(&format!("actions/runs/{id}/attempts/{attempt}")))
                    .unwrap(),
                *attempt,
                P,
            )
            .is_err(),
            "discarded {id}/{attempt}"
        );
    }
    let mut missing = f.api.clone();
    missing.page("actions/runs/4/artifacts", Some("artifacts"), vec![]);
    assert!(artifact::witness(
        &missing,
        &fs::read_to_string(f.root.path().join(depth_ci::ARCHIVE)).unwrap(),
        &missing
            .json(&missing.endpoint("actions/runs/4/attempts/1"))
            .unwrap(),
        1,
        P,
    )
    .is_err());
    let changed = |field, value| PopulationChanges {
        api: &f.api,
        reads: std::cell::Cell::new(0),
        field,
        value,
    };
    let query = RUNS.split("&per_page").next().unwrap();
    let anchor = f
        .api
        .json(&f.api.endpoint("git/ref/heads/staging"))
        .unwrap();
    for (field, value, refuses) in [
        ("run_attempt", json!(3), true),
        ("status", json!("completed"), false),
    ] {
        let changed = changed(field, value);
        let inventory = list(&changed, query, Some("workflow_runs")).unwrap();
        let stable = population::check_stable(&changed, query, &inventory, &anchor, true, None);
        if refuses {
            assert!(stable.unwrap_err().contains("population"));
        } else {
            stable.unwrap();
            assert_eq!(resolved.runs.len(), 132);
        }
    }

    // The earlier unsuccessful attempt remains in the population alongside every green retry.
    let mut failed = f.w.clone();
    failed.cases.executed[0].outcome = CaseOutcome::Failed;
    let earlier = fixture::endpoints_for_run(f.root.path(), P, &failed, &f.commit, 1, 1);
    let artifact_key = f
        .api
        .endpoint("actions/runs/1/artifacts?per_page=100&page=1");
    let mut artifacts: Value = serde_json::from_slice(&f.api.0[&artifact_key]).unwrap();
    let changed_artifacts: Value = serde_json::from_slice(&earlier.0[&artifact_key]).unwrap();
    artifacts["artifacts"][0] = changed_artifacts["artifacts"][0].clone();
    f.api
        .0
        .insert(artifact_key, serde_json::to_vec(&artifacts).unwrap());
    for (key, value) in earlier.0 {
        if key.contains("/actions/runs/1/attempts/1")
            || key.contains("/actions/artifacts/101/")
            || key.contains("/actions/jobs/201/")
        {
            f.api.0.insert(key, value);
        }
    }
    all_running(&mut f.api);
    let resolved = resolve_profile_in(&f.api, f.root.path(), P, Some(&host)).unwrap();
    assert_eq!(resolved.runs.len(), 132);
    assert!(resolved.runs.iter().any(|r| r.receipt.producer.run_id == 1
        && r.receipt
            .cases
            .executed
            .iter()
            .any(|c| c.outcome == CaseOutcome::Failed)));
    let obligation = resolved
        .runs
        .iter()
        .find(|r| {
            r.receipt
                .cases
                .executed
                .iter()
                .any(|c| c.outcome == CaseOutcome::Failed)
        })
        .unwrap()
        .receipt
        .scope
        .obligation
        .clone();
    let verdict = checkpoints::evaluate_population(&f, &resolved, Some(obligation));
    assert_ne!(verdict.result, DepthResult::Verified, "{verdict:?}");
}

#[test]
fn onboarding_refuses_cross_event_population_and_independent_adoption_mismatch() {
    let (mut f, host) = onboarding();
    let resolved = resolve_profile_in(&f.api, f.root.path(), P, Some(&host)).unwrap();
    let maintenance = Fixture::new_for(Profile::Synthetic);
    let maintenance_host = EvidenceHost::for_test(ISOLATED, &maintenance.commit);
    let maintenance_resolved = resolve_profile_in(
        &maintenance.api,
        maintenance.root.path(),
        Profile::Synthetic,
        Some(&maintenance_host),
    )
    .unwrap();
    for (population, foreign) in [
        (&resolved, &maintenance_resolved),
        (&maintenance_resolved, &resolved),
    ] {
        let run = &foreign.runs[0];
        let current = Evaluation {
            current: run.receipt.scope.clone(),
            definition_sha256: foreign.definition.clone(),
            record: foreign.original.clone(),
            environment: run.witness.environment.clone(),
            toolchain: run.witness.build.toolchain.clone(),
        };
        assert!(population.population(&current).unwrap().is_none());
    }
    assert!(resolve_profile_in(&f.api, f.root.path(), Profile::Synthetic, Some(&host)).is_err());
    assert!(resolve_profile_in(
        &maintenance.api,
        maintenance.root.path(),
        P,
        Some(&maintenance_host)
    )
    .is_err());
    assert!(P
        .record(
            &fs::read_to_string(maintenance.root.path().join(Profile::Synthetic.fixture()))
                .unwrap()
        )
        .is_err());

    // The generation PR is discovered from the actual approval commit, independently of any
    // PR association a workflow run claims. Ambiguous/foreign-base associations refuse.
    let Event::Onboarding {
        approval_commit, ..
    } = &resolved.original.event
    else {
        panic!()
    };
    for rows in [
        vec![],
        vec![json!({"number":9,"base":{"ref":"other","repo":{"full_name":ISOLATED}}})],
        vec![json!({"number":9,"base":{"ref":"staging","repo":{"full_name":ISOLATED}}}); 2],
    ] {
        let mut api = f.api.clone();
        api.page(&format!("commits/{approval_commit}/pulls"), None, rows);
        assert!(resolve_profile_in(&api, f.root.path(), P, Some(&host)).is_err());
    }
    for edit in 0..5 {
        let mut api = f.api.clone();
        let mut proposal = adoption::proposal_for(
            f.root.path(),
            ISOLATED,
            &f.w,
            "independent-review-invocation",
            P,
        )
        .unwrap();
        match edit {
            0 => {
                proposal["mappings"].as_array_mut().unwrap().pop();
            }
            1 => proposal["mappings"][0]["mapping_sha256"] = json!("0".repeat(64)),
            2 => proposal["mappings"][0]["independence_attested"] = json!(false),
            3 => proposal["mappings"][0]["executor_identity"] = json!("foreign-executor"),
            _ => api.put(
                "collaborators/maintainer/permission",
                json!({"permission":"read"}),
            ),
        }
        api.page("issues/9/comments", None, vec![json!({"id":33,"user":{"login":"maintainer"},"body":format!("```depth-adequacy-v1\n{proposal}\n```")})]);
        assert!(
            adoption::resolve(&api, f.root.path(), 9, &f.w, P)
                .unwrap()
                .is_empty(),
            "edit {edit}"
        );
    }
    f.api.page("issues/9/comments", None, vec![]);
    assert!(adoption::resolve(&f.api, f.root.path(), 9, &f.w, P)
        .unwrap()
        .is_empty());
}

#[test]
fn onboarding_source_adoption_and_dependency_currentness_are_independent() {
    let (mut f, host) = onboarding();
    let resolved = resolve_profile_in(&f.api, f.root.path(), P, Some(&host)).unwrap();
    let run = &resolved.runs[0];
    let mut view = GitInputs::new(f.root.path()).unwrap();
    crate::depth_evidence::currentness::check(
        f.root.path(),
        &mut view,
        &run.receipt,
        &run.witness.graph,
        true,
    )
    .unwrap();
    // Reporting E itself may change while frozen Event/P/O and immutable producer inputs hold.
    let assembly = assembly::assemble(&resolved).unwrap();
    let mut reporting = resolved.original.clone();
    reporting.execution = Some(assembly.execution);
    for (reference, bytes) in assembly.files {
        write(f.root.path(), &reference.path, bytes);
    }
    write(
        f.root.path(),
        inputs::RECORD,
        serde_json::to_vec_pretty(&reporting).unwrap(),
    );
    git(f.root.path(), &["add", "."]);
    let results = resolved.evaluate_record(f.root.path(), &reporting).unwrap();
    for obligation in P.obligations(&reporting) {
        assert_eq!(
            results.result(&reporting, &obligation),
            DepthResult::Verified
        );
    }
    let mut post_entry = reporting.clone();
    post_entry
        .acceptance_entries
        .push(crate::depth_record::AcceptanceEntry::Publication {
            packet: depth_ci::reference(
                "docs/test-publication.json",
                b"bounded entry-currentness test",
            ),
            record_sha256: reporting.stated_sha256().unwrap(),
        });
    for obligation in P.obligations(&reporting) {
        assert_eq!(
            results.result(&post_entry, &obligation),
            DepthResult::Unverified
        );
    }
    let fake = f.api.clone();
    let _reader = transport::test_reader::install(std::sync::Arc::new(move |endpoint, limit| {
        fake.bytes(endpoint, limit)
    }));
    let fresh = borrowed_record_results(f.root.path(), &post_entry, &host).unwrap();
    for obligation in P.obligations(&reporting) {
        assert_eq!(
            fresh.result(&post_entry, &obligation),
            DepthResult::Verified
        );
    }
    assert_eq!(
        P.record(&serde_json::to_string(&post_entry).unwrap())
            .unwrap(),
        resolved.original
    );
    let mut changed = reporting.clone();
    changed.event = Event::Onboarding {
        approval_commit: "f".repeat(40),
        approval_recorded_at: "2026-11-06T00:00:00Z".into(),
    };
    assert_eq!(
        results.result(&changed, &P.obligations(&changed)[0]),
        DepthResult::Unverified
    );

    let dependency = "crates/wrapper_events/src/channel.rs";
    write(
        f.root.path(),
        dependency,
        "changed dependency without any receipt edit",
    );
    git(f.root.path(), &["add", "."]);
    let mut view = GitInputs::new(f.root.path()).unwrap();
    assert!(crate::depth_evidence::currentness::check(
        f.root.path(),
        &mut view,
        &run.receipt,
        &run.witness.graph,
        true
    )
    .unwrap_err()
    .contains("changed built/current dependency"));
    assert!(resolve_profile_in(&f.api, f.root.path(), P, Some(&host))
        .unwrap_err()
        .contains("reviewed staging"));
    git(
        f.root.path(),
        &["commit", "-qm", "unreviewed source and reporting"],
    );
    let changed_commit = git(f.root.path(), &["rev-parse", "HEAD"]);
    assert!(profile::approved(
        f.root.path(),
        ISOLATED,
        &changed_commit,
        &changed_commit,
        &f.commit,
        P
    )
    .is_err());
    // Tell the fake host exactly which different bytes the maintainer reviewed. The adoption
    // resolver must compare them to the executed inputs and reject, without rewriting sources.
    f.api.change("issues/9/comments?per_page=100&page=1", |v| {
        v[0]["body"] = json!(v[0]["body"]
            .as_str()
            .unwrap()
            .replace(&f.commit, &changed_commit));
    });
    assert!(adoption::resolve(&f.api, f.root.path(), 9, &f.w, P)
        .unwrap()
        .is_empty());
}

#[test]
fn onboarding_requires_intact_on2_inputs_and_complete_immutable_graph() {
    let (f, host) = onboarding();
    let built = source(f.root.path(), ISOLATED, &f.commit).unwrap();
    let original = profile::original_for(&built, P).unwrap();
    let state = fs::read(f.root.path().join(inputs::STATE)).unwrap();
    // Stage projections must follow LifecycleState::validate, including the committed ON2
    // freeze (no implementation yet) and publication/proving states (no active runtime id).
    for stage in ["producer-freeze", "HEAD"] {
        let commit = git(f.root.path(), &["rev-parse", stage]);
        let at = source(f.root.path(), ISOLATED, &commit).unwrap();
        inputs::check(&at, P, &original).unwrap();
    }
    let raw = fs::read_to_string(f.root.path().join(inputs::APPROVAL)).unwrap();
    let mut approval: toml_edit::DocumentMut = raw.parse().unwrap();
    approval["support_depth"]["version"] = toml_edit::value("0.3.0");
    let raw = approval.to_string();
    let mut mismatched: Value = serde_json::from_slice(&state).unwrap();
    mismatched["approval_artifact_sha256"] =
        json!(depth_ci::reference(inputs::APPROVAL, raw.as_bytes()).sha256);
    let error = inputs::check_inputs(P, &original, |path| match path {
        inputs::APPROVAL => Ok(raw.clone()),
        inputs::STATE => Ok(mismatched.to_string()),
        _ => built.read(path),
    })
    .unwrap_err();
    assert!(error.contains("approval"), "{error}");
    for edit in 0..4 {
        let mut proposed: Value = serde_json::from_slice(&state).unwrap();
        match edit {
            0 => proposed.as_object_mut().unwrap().remove("support_depth"),
            1 => {
                proposed["support_depth"]["policy_identity"] = json!("f".repeat(64));
                None
            }
            2 => {
                proposed["active_runtime_evidence_run_id"] = Value::Null;
                None
            }
            _ => proposed
                .as_object_mut()
                .unwrap()
                .remove("written_paths_sha256"),
        };
        write(
            f.root.path(),
            inputs::STATE,
            serde_json::to_vec_pretty(&proposed).unwrap(),
        );
        git(f.root.path(), &["add", "."]);
        assert!(
            resolve_profile_in(&f.api, f.root.path(), P, Some(&host)).is_err(),
            "ON2 edit {edit}"
        );
    }
    git(f.root.path(), &["reset", "--hard", &f.commit]);
    fs::remove_file(f.root.path().join(inputs::APPROVAL)).unwrap();
    git(f.root.path(), &["add", "-u"]);
    assert!(resolve_profile_in(&f.api, f.root.path(), P, Some(&host)).is_err());
    git(f.root.path(), &["reset", "--hard", &f.commit]);
    for path in [
        inputs::APPROVAL,
        original.obligations.as_ref().unwrap().acquired_inputs[0]
            .path
            .as_str(),
    ] {
        let mut graph = f.w.graph.clone();
        graph.inputs.retain(|i| i.path != path);
        assert!(inputs::check_graph(&built, P, &original, &graph)
            .unwrap_err()
            .contains("immutable producer input"));
    }
}
