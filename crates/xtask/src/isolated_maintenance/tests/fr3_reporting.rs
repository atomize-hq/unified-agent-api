//! Shared ON2/MA2 proofs on each existing immutable real-writer fixture.
use super::*;
use crate::{
    depth_acceptance::{
        admission::{self, fake::MemoryHost, Host, Publication},
        check_integration,
    },
    depth_ci::reporting::{self, ContextRecord, Invocation},
    depth_record::DepthRecord,
};

pub(super) fn before(
    f: &Fixture,
    publication: &Publication,
    record: &DepthRecord,
    candidate: &str,
) -> ContextRecord {
    publication_closure(f, candidate);
    let policy = reporting::authority(publication).policy;
    let claims = f.api.3.begin(candidate, &policy).unwrap();
    let invocation = Invocation {
        claims,
        checkout: admission::emitter::object(f.root.path(), candidate).unwrap(),
        checkout_parents: vec![git(f.root.path(), &["rev-parse", "HEAD^"])],
        dispatch_inputs: [("depth_mode".into(), "reporting_only".into())].into(),
        configuration: [("profile".into(), "synthetic-wrapper-v1".into())].into(),
        substantive_invoked: false,
    };
    install(&f.api.3, candidate, &invocation);
    crate::depth_ci::collect_synthetic(f.root.path()).unwrap();
    let context = reporting::authenticate(
        f.root.path(),
        publication,
        invocation.claims.run,
        invocation.claims.attempt,
    )
    .unwrap();
    assert!(context.successful());
    assert!(f
        .api
        .3
        .records()
        .iter()
        .all(|r| r.producer_context.as_ref() != Some(&context.identity().unwrap())));
    let tip = publication.host.json("git/ref/heads/staging").unwrap()["object"]["sha"]
        .as_str()
        .unwrap()
        .to_owned();
    let history = publication
        .resolve(f.root.path(), &[tip, candidate.into()])
        .unwrap();
    assert!(history
        .require_reporting(&context)
        .unwrap_err()
        .contains("protected lineage"));
    let claims = &context.invocation.claims;
    let invocation_path = format!("reporting/invocations/{}/{}", claims.run, claims.attempt);
    f.api.3.change(&invocation_path, |value| {
        value["dispatch_inputs"]["ignored_prose"] = json!("candidate text");
        value["configuration"]["ignored_prose"] = json!("candidate text");
    });
    f.api
        .3
        .change(&format!("reporting/candidates/{candidate}"), |value| {
            value["dispatch_inputs"]["ignored_prose"] = json!("unrelated source text");
        });
    assert_eq!(
        reporting::authenticate(f.root.path(), publication, claims.run, claims.attempt).unwrap(),
        context
    );
    refusals(f, publication, &f.api.3, &context);
    dirty_checkout_and_pinned_context(f, publication, &context, record);
    checkout_relationships(f, publication, &context, record);
    context
}

fn publication_closure(f: &Fixture, candidate: &str) {
    use crate::support_matrix as matrix;
    let root = f.root.path();
    let parent = git(root, &["rev-parse", &format!("{candidate}^")]);
    for path in [
        "crates",
        "Cargo.toml",
        "Cargo.lock",
        ".github/workflows/ci.yml",
        crate::depth_ci::ARCHIVE,
    ] {
        assert_eq!(
            git(root, &["rev-parse", &format!("{candidate}:{path}")]),
            git(root, &["rev-parse", &format!("{parent}:{path}")]),
            "freeze changed execution closure {path}"
        );
    }
    let rendered = matrix::generate_publication_artifacts(root).unwrap();
    for (path, bytes) in [
        (matrix::JSON_OUTPUT_PATH, rendered.json),
        (matrix::MARKDOWN_OUTPUT_PATH, rendered.markdown),
        (
            matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
            rendered.runtime_support_data,
        ),
    ] {
        assert_eq!(
            fs::read(root.join(path)).unwrap(),
            bytes.as_bytes(),
            "publication changed {path}"
        );
    }
}
fn install(host: &MemoryHost, candidate: &str, invocation: &Invocation) {
    host.put(
        &format!("reporting/candidates/{candidate}"),
        serde_json::to_value(invocation).unwrap(),
    );
    host.put(
        &format!(
            "reporting/invocations/{}/{}",
            invocation.claims.run, invocation.claims.attempt
        ),
        serde_json::to_value(invocation).unwrap(),
    );
}
fn refusals(f: &Fixture, publication: &Publication, host: &MemoryHost, context: &ContextRecord) {
    let root = f.root.path();
    let c = &context.invocation.claims;
    let jobs = format!(
        "actions/runs/{}/attempts/{}/jobs?per_page=100&page=1",
        c.run, c.attempt
    );
    let invocation = format!("reporting/invocations/{}/{}", c.run, c.attempt);
    for mutation in 0..10 {
        let host = Arc::new(host.fork());
        match mutation {
            0 => host.change(&jobs, |v| {
                v["jobs"][0]["workflow_job_key"] = json!("candidate")
            }),
            1 => host.change(&jobs, |v| v["jobs"][0]["run_attempt"] = json!(2)),
            2 => host.change(&invocation, |v| {
                v["checkout"]["tree"] = json!("f".repeat(40))
            }),
            3 => host.change(&invocation, |v| {
                v["dispatch_inputs"]["depth_mode"] = json!("execute")
            }),
            4 => host.change(&invocation, |v| {
                v["checkout_parents"] = json!(["f".repeat(40)])
            }),
            5 => host.change(&jobs, |v| v["jobs"][0]["conclusion"] = json!("failure")),
            6 => host.change(&jobs, |v| v["jobs"][0]["conclusion"] = json!("cancelled")),
            7 => host.change(&jobs, |v| v["jobs"][0]["status"] = json!("in_progress")),
            8 => host.change(
                &format!("actions/runs/{}/attempts/{}", c.run, c.attempt),
                |v| v["policy"]["implementation"] = json!("f".repeat(64)),
            ),
            _ => {
                host.facts.lock().unwrap().remove(&invocation);
            }
        }
        let p = Publication {
            authority: publication.authority.clone(),
            host,
        };
        let error = reporting::authenticate(root, &p, c.run, c.attempt).unwrap_err();
        if matches!(mutation, 3 | 9) {
            let mut emitter = f.resolve().unwrap().admission_emitter().unwrap();
            emitter.parent = Some(context.predecessor.commit.clone());
            emitter.publication = p.clone();
            assert_eq!(emitter.prepare(root).unwrap_err(), error);
            assert!(emitter.prepared.is_none());
        }
        assert!(
            error.contains("job identity")
                || error.contains("wrong-attempt job")
                || error.contains("host association")
                || error.contains("provenance")
                || error.contains("invocation unavailable"),
            "{mutation}: {error}"
        );
    }
    let host = Arc::new(host.fork());
    let p = Publication {
        authority: publication.authority.clone(),
        host: host.clone(),
    };
    let artifacts = format!("actions/runs/{}/artifacts?per_page=100&page=1", c.run);
    let artifact = host.json(&artifacts).unwrap()["artifacts"][0]["id"]
        .as_u64()
        .unwrap();
    host.facts.lock().unwrap().insert(
        format!("actions/artifacts/{artifact}/zip"),
        b"substituted".to_vec(),
    );
    assert_eq!(
        reporting::authenticate(root, &p, c.run, c.attempt).unwrap_err(),
        "IN1 artifact digest/size mismatch"
    );
    // A copied candidate payload never installs the independently observed invocation.
    write(
        root,
        "target/candidate-reporting-context.json",
        context.canonical().unwrap(),
    );
    let absent = Arc::new(f.api.3.fork());
    absent.facts.lock().unwrap().remove(&invocation);
    let p = Publication {
        authority: publication.authority.clone(),
        host: absent,
    };
    assert_eq!(
        reporting::authenticate(root, &p, c.run, c.attempt).unwrap_err(),
        "reporting historical invocation unavailable"
    );
    fs::remove_file(root.join("target/candidate-reporting-context.json")).unwrap();
}

pub(super) fn after(f: &Fixture, publication: &Publication, context: &ContextRecord) {
    let root = f.root.path();
    let history = publication
        .resolve(root, std::slice::from_ref(&context.candidate.commit))
        .unwrap();
    history.require_reporting(context).unwrap();
    assert!(history.is_request_freeze(&context.candidate.commit, &context.generation));
    let admitted = f
        .api
        .3
        .records()
        .into_iter()
        .find(|r| r.producer_context.as_ref() == Some(&context.identity().unwrap()))
        .unwrap();
    assert_eq!(admitted.checkpoint, admission::Checkpoint::RequestFreeze);
    // Every retry has its own immutable context, and receives its own later real IN1 decision.
    let mut invocation = context.invocation.clone();
    invocation.claims.attempt += 1;
    invocation.claims.job += 100_000;
    invocation.claims.check += 100_000;
    let c = &invocation.claims;
    let mut details = f
        .api
        .3
        .json(&format!("actions/runs/{}/attempts/1", c.run))
        .unwrap();
    details["run_attempt"] = json!(c.attempt);
    f.api.3.put(
        &format!("actions/runs/{}/attempts/{}", c.run, c.attempt),
        details,
    );
    let mut jobs = f
        .api
        .3
        .json(&format!(
            "actions/runs/{}/attempts/1/jobs?per_page=100&page=1",
            c.run
        ))
        .unwrap()["jobs"]
        .as_array()
        .unwrap()
        .clone();
    jobs[0]["id"] = json!(c.job);
    jobs[0]["check_run_id"] = json!(c.check);
    jobs[0]["run_attempt"] = json!(c.attempt);
    jobs[0]["status"] = json!("in_progress");
    f.api.3.page(
        &format!("actions/runs/{}/attempts/{}/jobs", c.run, c.attempt),
        "jobs",
        jobs,
    );
    f.api.3.change("reporting/runs?per_page=100&page=1", |v| {
        v["workflow_runs"][0]["run_attempt"] = json!(c.attempt)
    });
    install(&f.api.3, &context.candidate.commit, &invocation);
    let path = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";
    let depth = DepthRecord::parse(
        &fs::read_to_string(root.join(path)).unwrap(),
        "depth_fixture",
        "0.2.0",
    )
    .unwrap();
    let retry = reporting::dispatch(root, publication, &context.candidate.commit, &depth)
        .unwrap()
        .unwrap();
    assert_ne!(retry.identity().unwrap(), context.identity().unwrap());
    let history = publication
        .resolve(root, std::slice::from_ref(&context.candidate.commit))
        .unwrap();
    assert_eq!(
        history.require_reporting(&retry).unwrap_err(),
        "reporting context lacks later exact-transition IN1 admission"
    );
    check_integration(root, &context.predecessor.commit).unwrap();
    let history = publication
        .resolve(root, std::slice::from_ref(&context.candidate.commit))
        .unwrap();
    history.require_reporting(&retry).unwrap();
    history.require_reporting(context).unwrap();
    // Same-generation producers on other PRs and all supported events retain their own context.
    for event in ["merge_group", "workflow_dispatch", "push"] {
        let policy = reporting::authority(publication).policy;
        let mut invocation = context.invocation.clone();
        invocation.claims = f.api.3.begin(&context.candidate.commit, &policy).unwrap();
        invocation.claims.event = event.into();
        let claims = &invocation.claims;
        let path = format!("actions/runs/{}/attempts/1", claims.run);
        f.api.3.change(&path, |value| {
            value["event"] = json!(event);
            value["pull_requests"] = json!([{"number":claims.run}]);
        });
        f.api
            .3
            .change("reporting/runs?per_page=100&page=1", |value| {
                let run = value["workflow_runs"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|run| run["id"] == claims.run)
                    .unwrap();
                run["event"] = json!(event);
                run["pull_requests"] = json!([{"number":claims.run}]);
            });
        install(&f.api.3, &context.candidate.commit, &invocation);
        let emitted = reporting::dispatch(root, publication, &context.candidate.commit, &depth)
            .unwrap()
            .unwrap();
        check_integration(root, &context.predecessor.commit).unwrap();
        let history = publication
            .resolve(root, std::slice::from_ref(&context.candidate.commit))
            .unwrap();
        history.require_reporting(&emitted).unwrap();
    }
    crate::depth_evidence::prove_reporting(
        root,
        publication.clone(),
        f.api.3.clone(),
        context.clone(),
        |root, parent, host| approve_source(f, root, parent, host),
    );
}

fn dirty_checkout_and_pinned_context(
    f: &Fixture,
    publication: &Publication,
    context: &ContextRecord,
    depth: &DepthRecord,
) {
    let root = f.root.path();
    let host = Arc::new(f.api.3.fork());
    let publication = Publication {
        authority: publication.authority.clone(),
        host: host.clone(),
    };
    let path = "docs/input.txt";
    let bytes = fs::read(root.join(path)).unwrap();
    git(root, &["update-index", "--assume-unchanged", path]);
    write(root, path, b"hidden changed actual checkout");
    let error =
        reporting::dispatch(root, &publication, &context.candidate.commit, depth).unwrap_err();
    assert!(
        error.contains("working bytes disagree with index"),
        "{error}"
    );
    write(root, path, &bytes);
    git(root, &["update-index", "--no-assume-unchanged", path]);
    let mut emitter = f.resolve().unwrap().admission_emitter().unwrap();
    emitter.parent = Some(context.predecessor.commit.clone());
    emitter.publication = publication;
    emitter.prepare(root).unwrap();
    let pinned = emitter
        .prepared
        .as_ref()
        .unwrap()
        .0
        .producer_context
        .clone();
    assert_eq!(pinned.as_ref(), Some(&context.identity().unwrap()));
    host.change(
        &format!("reporting/candidates/{}", context.candidate.commit),
        |value| {
            value["dispatch_inputs"]["depth_mode"] = json!("execute");
        },
    );
    let error = emitter
        .finish(
            root,
            Ok(()),
            Ok(admission::observations::Authority {
                identity: "f".repeat(64),
                incomplete: false,
                decision: Some(admission::Decision::Admitted),
            }),
        )
        .unwrap_err();
    assert!(error.contains("changed after input capture"), "{error}");
    let retained = host
        .records()
        .into_iter()
        .find(|record| record.candidate == context.candidate)
        .unwrap();
    assert_eq!(retained.producer_context, pinned);
    assert_eq!(retained.decision, admission::Decision::Admitted);
    assert!(retained
        .reasons
        .contains(&admission::Reason::ObservationIncomplete));
}

fn checkout_relationships(
    f: &Fixture,
    publication: &Publication,
    context: &ContextRecord,
    depth: &DepthRecord,
) {
    let root = f.root.path();
    for mutation in 0..3 {
        let host = Arc::new(f.api.3.fork());
        let publication = Publication {
            authority: publication.authority.clone(),
            host: host.clone(),
        };
        let mut invocation = context.invocation.clone();
        invocation.claims = host
            .begin(
                &context.candidate.commit,
                &reporting::authority(&publication).policy,
            )
            .unwrap();
        let (tree, first, second) = match mutation {
            1 => (
                &context.predecessor.tree,
                &context.predecessor.commit,
                &context.candidate.commit,
            ),
            2 => (
                &context.candidate.tree,
                &context.candidate.commit,
                &context.predecessor.commit,
            ),
            _ => (
                &context.candidate.tree,
                &context.predecessor.commit,
                &context.candidate.commit,
            ),
        };
        let merge = git(
            root,
            &[
                "commit-tree",
                tree,
                "-p",
                first,
                "-p",
                second,
                "-m",
                "modeled PR test merge",
            ],
        );
        invocation.checkout = admission::emitter::object(root, &merge).unwrap();
        invocation.checkout_parents = vec![first.clone(), second.clone()];
        install(&host, &context.candidate.commit, &invocation);
        git(root, &["reset", "--hard", &merge]);
        let result = reporting::dispatch(root, &publication, &context.candidate.commit, depth);
        git(root, &["reset", "--hard", &context.candidate.commit]);
        if mutation == 0 {
            let merged = result.unwrap().unwrap();
            assert_eq!(merged.head_relationship, "pr_test_merge");
            assert_eq!(merged.invocation.checkout.commit, merge);
        } else {
            assert_eq!(
                result.unwrap_err(),
                "reporting checkout has no sealed permitted head relationship"
            );
        }
    }
}

pub(super) fn freeze_after_readiness(
    f: &Fixture,
    publication: &Publication,
    freeze: &str,
    ready: &str,
    depth: &DepthRecord,
) {
    let root = f.root.path();
    git(root, &["reset", "--hard", ready]);
    f.host();
    git(root, &["update-ref", "refs/remotes/origin/staging", ready]);
    git(root, &["read-tree", "--reset", "-u", freeze]);
    let candidate = super::in1_integration::candidate(f, ready);
    let claims = f
        .api
        .3
        .begin(&candidate, &reporting::authority(publication).policy)
        .unwrap();
    let invocation = Invocation {
        claims,
        checkout: admission::emitter::object(root, &candidate).unwrap(),
        checkout_parents: vec![ready.into()],
        dispatch_inputs: [("depth_mode".into(), "reporting_only".into())].into(),
        configuration: [("profile".into(), "synthetic-wrapper-v1".into())].into(),
        substantive_invoked: false,
    };
    install(&f.api.3, &candidate, &invocation);
    let context = reporting::dispatch(root, publication, &candidate, depth)
        .unwrap()
        .unwrap();
    let observation = admission::freeze::authority::Observation {
        repository: publication.authority.repository.clone(),
        predecessor: admission::emitter::object(root, ready).unwrap(),
        generation: admission::Generation::from_record(depth),
        policy: publication.authority.policy.clone(),
        writer: admission::freeze::authority::Writer::Maintenance,
    };
    f.api.3.put(
        &format!("freeze-inputs/{candidate}"),
        serde_json::to_value(observation).unwrap(),
    );
    check_integration(root, ready).unwrap();
    let admitted = f
        .api
        .3
        .records()
        .into_iter()
        .find(|record| record.candidate.commit == candidate)
        .unwrap();
    assert_eq!(
        admitted.checkpoint,
        admission::Checkpoint::ExecutionRequired
    );
    assert!(admitted.reasons.iter().any(|reason| matches!(
        reason,
        admission::Reason::FreezeTreeMismatch | admission::Reason::FreezePlanningRefused
    )));
    f.host();
    git(
        root,
        &["update-ref", "refs/remotes/origin/staging", &candidate],
    );
    crate::depth_evidence::prove_refused_reporting(
        root,
        publication.clone(),
        f.api.3.clone(),
        context,
    );
}

/// Real protected IN1 admits the execution source after its modeled execution already exists.
fn approve_source(f: &Fixture, root: &Path, parent: &str, host: &Arc<MemoryHost>) {
    let api = Arc::new(super::fixture::Fake(
        std::sync::Mutex::new(f.api.0.lock().unwrap().clone()),
        Default::default(),
        Default::default(),
        host.clone(),
        Default::default(),
    ));
    let baseline = f.baseline.clone();
    let subject = f.subject;
    let candidate = git(root, &["rev-parse", "HEAD"]);
    git(
        root,
        &[
            "remote",
            "set-url",
            "origin",
            "https://github.com/fixture/isolated.git",
        ],
    );
    git(root, &["update-ref", "refs/remotes/origin/staging", parent]);
    for base in [&baseline, parent] {
        api.put(
            &format!("repos/fixture/isolated/compare/{base}...{candidate}"),
            json!({"status":"ahead","merge_base_commit":{"sha":base}}),
        );
    }
    api.put(
        &format!("repos/fixture/isolated/commits/{candidate}"),
        json!({"sha":candidate}),
    );
    let _installed = super::fixture::install(Box::new(move || {
        (
            super::fixture::profile_with(baseline.clone(), subject),
            api.clone() as Arc<dyn Api>,
        )
    }));
    check_integration(root, parent).unwrap();
    assert!(host
        .records()
        .iter()
        .any(|r| r.candidate.commit == candidate && r.successful()));
}

#[path = "fr2_maintenance/reporting.rs"]
mod maintenance;
