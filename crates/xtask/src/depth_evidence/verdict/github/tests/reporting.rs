//! Connected real-writer reporting populations use the production generation resolver.
use super::*;
use crate::{
    depth_acceptance::admission::{fake::MemoryHost, Host as AdmissionHost, Publication},
    depth_ci::reporting::ContextRecord,
    isolated_maintenance::EvidenceHost,
};
use std::sync::Arc;

pub(crate) fn prove_reporting(
    root: &Path,
    publication: Publication,
    host: Arc<MemoryHost>,
    context: ContextRecord,
    approve_source: impl FnOnce(&Path, &str, &Arc<MemoryHost>),
) {
    let scratch = tempfile::tempdir().unwrap();
    git(
        scratch.path(),
        &[
            "clone",
            "--quiet",
            "--no-hardlinks",
            root.to_str().unwrap(),
            ".",
        ],
    );
    let root = scratch.path();
    git(root, &["config", "user.name", "Reporting proof"]);
    git(root, &["config", "user.email", "reporting@example.invalid"]);
    let p = Profile::for_event(&context.generation.event);
    let frozen = fs::read(root.join(depth_ci::onboarding::RECORD)).unwrap();
    if p == Profile::Synthetic {
        // Fixture insertion is delayed until after genuine MA2 and its real IN1 admission.
        write(root, p.fixture(), &frozen);
    }
    let before_approval = git(root, &["rev-parse", "HEAD"]);
    let (w, implementation) = build(root, p);
    let mut api = endpoints(root, p, &w, &implementation, 1);
    let host = Arc::new(host.fork());
    // The execution exists before protected approval. No run-start-time gate is involved.
    let error = profile::approved(
        root,
        ISOLATED,
        &implementation,
        &implementation,
        &before_approval,
        p,
    )
    .unwrap_err();
    assert!(
        error.contains("producer closure is not approved on protected staging"),
        "{error}"
    );
    approve_source(root, &before_approval, &host);
    host.root(&implementation);
    profile::approved(
        root,
        ISOLATED,
        &implementation,
        &implementation,
        &implementation,
        p,
    )
    .unwrap();
    if p == Profile::Synthetic {
        super::generation::binding_matrix(root, &implementation, &api);
    }
    let publication = Publication {
        authority: publication.authority,
        host: host.clone(),
    };
    let bound = EvidenceHost::for_admission_test(ISOLATED, &implementation, publication.clone());
    let resolver = p;
    if p == Profile::Synthetic {
        let source =
            crate::runtime_advertising::source::Source::tip(root, &context.candidate.commit)
                .unwrap();
        assert!(source.optional(p.fixture()).unwrap().is_none());
    }
    let date = match &context.generation.event {
        Event::Maintenance {
            request_recorded_at,
            ..
        } => &request_recorded_at[..10],
        Event::Onboarding {
            approval_recorded_at,
            ..
        } => &approval_recorded_at[..10],
    };
    let query = format!("actions/workflows/ci.yml/runs?created=%3E%3D{date}");
    let page = format!("{query}&per_page=100&page=1");
    let execution_rows = api.json(&api.endpoint(&page)).unwrap()["workflow_runs"]
        .as_array()
        .unwrap()
        .clone();
    let checkpoint_rows = host.json("reporting/runs?per_page=100&page=1").unwrap()["workflow_runs"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(checkpoint_rows.len(), 4);
    let checkpoint_attempts = checkpoint_rows
        .iter()
        .map(|run| run["run_attempt"].as_u64().unwrap())
        .sum::<u64>() as usize;
    assert_eq!(checkpoint_rows[0]["run_attempt"], 2);
    for (key, bytes) in host
        .facts
        .lock()
        .unwrap()
        .iter()
        .filter(|(key, _)| key.starts_with("actions/"))
    {
        api.0.insert(api.endpoint(key), bytes.clone());
    }
    api.page(&query, Some("workflow_runs"), checkpoint_rows.clone());
    let checkpoints =
        population::resolve_generation_inputs(&api, root, Some(&bound), None, &resolver).unwrap();
    assert_eq!(checkpoints.reporting.len(), checkpoint_attempts);
    assert!(checkpoints.runs.is_empty());
    assert!(assembly::assemble(&checkpoints)
        .unwrap()
        .selected
        .is_empty());
    let proof: Value =
        serde_json::from_str(&prove_resolved(root, ISOLATED, &checkpoints).unwrap()).unwrap();
    assert_eq!(proof["result"], "unverified");
    assert_eq!(proof["reporting_attempts"], checkpoint_attempts);
    let mut rows = checkpoint_rows.clone();
    rows.extend(execution_rows.clone());
    api.page(&query, Some("workflow_runs"), rows.clone());
    for reverse in [false, true] {
        if reverse {
            rows.reverse();
            api.page(&query, Some("workflow_runs"), rows.clone());
        }
        let resolved =
            population::resolve_generation_inputs(&api, root, Some(&bound), None, &resolver)
                .unwrap();
        assert_eq!(resolved.reporting.len(), checkpoint_attempts);
        assert_eq!(resolved.runs.len(), p.obligations(&resolved.original).len());
        assert!(resolved
            .runs
            .iter()
            .all(|r| r.receipt.producer.run_id == w.run_id
                && r.receipt.build.source_commit == implementation));
        let proof: Value =
            serde_json::from_str(&prove_resolved(root, ISOLATED, &resolved).unwrap()).unwrap();
        assert_eq!(proof["result"], "verified", "{proof}");
    }
    if p == Profile::Synthetic {
        associated_descendants(root, &implementation, &api, &bound, &query);
    }
    // A later protected source S2 cannot approve this retained S1 execution.
    let original_source = fs::read(root.join(depth_ci::COLLECTOR)).unwrap();
    write(root, depth_ci::COLLECTOR, b"changed protected collector S2");
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "protected source S2"]);
    let changed = git(root, &["rev-parse", "HEAD"]);
    let mut changed_api = api.clone();
    changed_api.put("git/ref/heads/staging", json!({"object":{"sha":changed}}));
    let changed_host = EvidenceHost::for_test(ISOLATED, &changed);
    let error =
        population::resolve_profile_inputs(&changed_api, root, p, Some(&changed_host), None)
            .unwrap_err();
    assert!(
        error.contains("producer closure is not approved on protected staging"),
        "{error}"
    );
    git(root, &["reset", "--hard", &implementation]);
    assert_eq!(
        fs::read(root.join(depth_ci::COLLECTOR)).unwrap(),
        original_source
    );
    let absent = EvidenceHost::for_test(ISOLATED, &implementation);
    let error = population::resolve_generation_inputs(&api, root, Some(&absent), None, &resolver)
        .unwrap_err();
    assert!(
        error.contains("producer closure") || error.contains("witness artifact"),
        "{error}"
    );
    let failed_job = format!(
        "actions/runs/{}/attempts/{}/jobs?per_page=100&page=1",
        context.invocation.claims.run, context.invocation.claims.attempt
    );
    let prior_jobs = host.json(&failed_job).unwrap();
    host.change(&failed_job, |value| {
        value["jobs"][0]["conclusion"] = json!("failure")
    });
    let error = population::resolve_generation_inputs(&api, root, Some(&bound), None, &resolver)
        .unwrap_err();
    assert_eq!(error, "failed reporting producer or incomplete collection");
    host.put(&failed_job, prior_jobs);
    unexpected_execution(root, &publication, &host, &w, p);
    // Missing a later exact-context IN1 retry cannot silently remove the original attempt.
    let revoked = host
        .records()
        .into_iter()
        .filter(|r| r.producer_context.is_some())
        .map(|r| r.identity().unwrap())
        .collect::<Vec<_>>();
    host.put("in1/revocations", json!({"records":revoked}));
    let error = population::resolve_generation_inputs(&api, root, Some(&bound), None, &resolver)
        .unwrap_err();
    assert_eq!(
        error,
        "reporting context lacks later exact-transition IN1 admission"
    );
}

fn associated_descendants(root: &Path, base: &str, api: &Fake, bound: &EvidenceHost, query: &str) {
    let p = Profile::Synthetic;
    let path = crate::agent_lifecycle::maintenance_request_path(p.agent_id());
    let report = depth_ci::onboarding::RECORD;
    for removed in [true, false] {
        if removed {
            fs::remove_file(root.join(&path)).unwrap();
            fs::remove_file(root.join(report)).unwrap();
        } else {
            let mut request: toml_edit::DocumentMut = fs::read_to_string(root.join(&path))
                .unwrap()
                .parse()
                .unwrap();
            request["request_commit"] = toml_edit::value("a".repeat(40));
            write(root, &path, request.to_string());
            let mut record = p
                .record(&fs::read_to_string(root.join(report)).unwrap())
                .unwrap();
            if let Event::Maintenance { request_commit, .. } = &mut record.event {
                *request_commit = "a".repeat(40);
            }
            write(root, report, serde_json::to_vec_pretty(&record).unwrap());
        }
        git(root, &["add", "."]);
        git(
            root,
            &["commit", "-qm", "removed or replaced generation inputs"],
        );
        let candidate = git(root, &["rev-parse", "HEAD"]);
        git(root, &["reset", "--hard", base]);
        let mut next = api.clone();
        let mut rows = next
            .json(&next.endpoint(&format!("{query}&per_page=100&page=1")))
            .unwrap()["workflow_runs"]
            .as_array()
            .unwrap()
            .clone();
        let mut run = rows[0].clone();
        run["id"] = json!(48);
        run["head_sha"] = json!(candidate);
        run["pull_requests"] = json!([]);
        rows.push(run);
        next.page(query, Some("workflow_runs"), rows);
        let error =
            population::resolve_profile_inputs(&next, root, p, Some(bound), None).unwrap_err();
        assert_eq!(error, "associated head replaced maintenance generation");
    }
}

pub(crate) fn prove_refused_reporting(
    root: &Path,
    publication: Publication,
    host: Arc<MemoryHost>,
    context: ContextRecord,
) {
    let scratch = tempfile::tempdir().unwrap();
    git(
        scratch.path(),
        &[
            "clone",
            "--quiet",
            "--no-hardlinks",
            root.to_str().unwrap(),
            ".",
        ],
    );
    let root = scratch.path();
    git(root, &["config", "user.name", "Readiness refusal"]);
    git(root, &["config", "user.email", "readiness@example.invalid"]);
    let p = Profile::for_event(&context.generation.event);
    let bytes = fs::read(root.join(depth_ci::onboarding::RECORD)).unwrap();
    if p == Profile::Synthetic {
        write(root, p.fixture(), bytes);
    }
    let (w, implementation) = build(root, p);
    let mut api = endpoints(root, p, &w, &implementation, 1);
    let host = Arc::new(host.fork());
    host.root(&implementation);
    let publication = Publication {
        authority: publication.authority,
        host: host.clone(),
    };
    let bound = EvidenceHost::for_admission_test(ISOLATED, &implementation, publication);
    let date = match &context.generation.event {
        Event::Maintenance {
            request_recorded_at,
            ..
        } => &request_recorded_at[..10],
        Event::Onboarding {
            approval_recorded_at,
            ..
        } => &approval_recorded_at[..10],
    };
    let query = format!("actions/workflows/ci.yml/runs?created=%3E%3D{date}");
    let page = format!("{query}&per_page=100&page=1");
    let mut rows = api.json(&api.endpoint(&page)).unwrap()["workflow_runs"]
        .as_array()
        .unwrap()
        .clone();
    let mut checkpoint = host
        .json(&format!(
            "actions/runs/{}/attempts/1",
            context.invocation.claims.run
        ))
        .unwrap();
    checkpoint["pull_requests"] = json!([{"number":4567}]);
    rows.push(checkpoint);
    for (path, bytes) in host
        .facts
        .lock()
        .unwrap()
        .iter()
        .filter(|(path, _)| path.starts_with("actions/"))
    {
        api.0.insert(api.endpoint(path), bytes.clone());
    }
    for reverse in [false, true] {
        if reverse {
            rows.reverse();
        }
        api.page(&query, Some("workflow_runs"), rows.clone());
        let error =
            population::resolve_generation_inputs(&api, root, Some(&bound), None, &p).unwrap_err();
        assert_eq!(
            error,
            "reporting context lacks later exact-transition IN1 admission"
        );
    }
}

fn unexpected_execution(
    root: &Path,
    publication: &Publication,
    host: &Arc<MemoryHost>,
    w: &Witness,
    p: Profile,
) {
    use crate::depth_ci::reporting::{self, Invocation};
    let candidate = &w.build.source_commit;
    let claims = host
        .begin(candidate, &reporting::authority(publication).policy)
        .unwrap();
    let invocation = Invocation {
        claims: claims.clone(),
        checkout: crate::depth_acceptance::admission::emitter::object(root, candidate).unwrap(),
        checkout_parents: vec![git(root, &["rev-parse", &format!("{candidate}^")])],
        dispatch_inputs: [("depth_mode".into(), "reporting_only".into())].into(),
        configuration: [("profile".into(), "synthetic-wrapper-v1".into())].into(),
        substantive_invoked: false,
    };
    host.put(
        &format!("reporting/candidates/{candidate}"),
        serde_json::to_value(&invocation).unwrap(),
    );
    host.put(
        &format!("reporting/invocations/{}/1", claims.run),
        serde_json::to_value(&invocation).unwrap(),
    );
    let raw = fs::read_to_string(root.join(depth_ci::onboarding::RECORD)).unwrap();
    let original = p.record(&raw).unwrap();
    reporting::dispatch(root, publication, candidate, &original)
        .unwrap()
        .unwrap();
    let bound = EvidenceHost::for_admission_test(ISOLATED, candidate, publication.clone());
    let artifact_path = format!("actions/runs/{}/artifacts", claims.run);
    let context_artifacts = host
        .json(&format!("{artifact_path}?per_page=100&page=1"))
        .unwrap()["artifacts"]
        .as_array()
        .unwrap()
        .clone();
    for failure in [false, true] {
        let mut witness = w.clone();
        if failure {
            witness.cases.executed[0].outcome = CaseOutcome::Failed;
        }
        let mut api = fixture::endpoints_for_run(root, p, &witness, candidate, claims.run, 1);
        let inventory = api
            .0
            .keys()
            .find(|path| path.contains("/runs?created="))
            .unwrap()
            .clone();
        let mut rows: Value = serde_json::from_slice(&api.0[&inventory]).unwrap();
        rows["workflow_runs"][0]["event"] = json!(claims.event);
        rows["workflow_runs"][0]["workflow_id"] = json!(publication.authority.policy.workflow_id);
        api.0.insert(inventory, serde_json::to_vec(&rows).unwrap());
        let witness_artifacts = api
            .json(&api.endpoint(&format!("{artifact_path}?per_page=100&page=1")))
            .unwrap()["artifacts"]
            .as_array()
            .unwrap()
            .clone();
        let mut artifacts = context_artifacts.clone();
        artifacts.extend(witness_artifacts);
        api.page(&artifact_path, Some("artifacts"), artifacts.clone());
        host.page(&artifact_path, "artifacts", artifacts);

        let job_path = format!(
            "actions/runs/{}/attempts/1/jobs?per_page=100&page=1",
            claims.run
        );
        let mut jobs = api.json(&api.endpoint(&job_path)).unwrap();
        let old_job = jobs["jobs"][0]["id"].as_u64().unwrap();
        let witness_log = api
            .bytes(
                &api.endpoint(&format!("actions/jobs/{old_job}/logs")),
                1_048_576,
            )
            .unwrap();
        jobs["jobs"][0]["id"] = json!(claims.job);
        jobs["jobs"][0]["check_run_id"] = json!(claims.check);
        jobs["jobs"][0]["conclusion"] = json!(if failure { "failure" } else { "success" });
        api.put(&job_path, jobs);
        let mut logs = host
            .bytes(&format!("actions/jobs/{}/logs", claims.job), 1_048_576)
            .unwrap();
        logs.extend(witness_log);
        api.0.insert(
            api.endpoint(&format!("actions/jobs/{}/logs", claims.job)),
            logs,
        );
        let run_path = format!("actions/runs/{}/attempts/1", claims.run);
        let mut run = host.json(&run_path).unwrap();
        run["status"] = json!("completed");
        api.put(&run_path, run);
        host.change(&job_path, |value| {
            value["jobs"][0]["conclusion"] = json!(if failure { "failure" } else { "success" })
        });
        let resolved =
            population::resolve_generation_inputs(&api, root, Some(&bound), None, &p).unwrap();
        assert_eq!(resolved.reporting_conflicts, [(claims.run, 1)].into());
        let proof: Value =
            serde_json::from_str(&prove_resolved(root, ISOLATED, &resolved).unwrap()).unwrap();
        assert_eq!(
            proof["result"],
            if failure { "failed" } else { "unverified" },
            "{proof}"
        );
    }
}
