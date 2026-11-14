use super::*;
use crate::{
    depth_acceptance::admission::{fake::MemoryHost, Publication},
    freeze_transition::{acquisition, Entry, Plan, Tree},
    isolated_maintenance::acquisition::{self as observed, Retained},
};

pub(super) fn inventory(plan: &Plan, candidate: &Tree, observation: &observed::Observation) {
    let paths = observation.outputs.keys().cloned().collect::<Vec<_>>();
    let before = Tree::new();
    acquisition::authenticated(&before, candidate, &paths, &observation.outputs).unwrap();
    for path in &paths {
        let original = candidate.get(path).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&original.bytes).unwrap();
        for mutation in 0..6 {
            let mut changed = candidate.clone();
            match mutation {
                0 => {
                    changed.get_mut(path).unwrap().bytes.extend(b" \n");
                }
                1 => {
                    changed.remove(path);
                }
                2 => changed.get_mut(path).unwrap().mode = "100755".into(),
                3 => changed.get_mut(path).unwrap().mode = "120000".into(),
                4 => changed.get_mut(path).unwrap().kind = "commit".into(),
                _ => changed.get_mut(path).unwrap().bytes = b"{}\n".to_vec(),
            }
            if mutation == 0 {
                assert_eq!(
                    serde_json::from_slice::<serde_json::Value>(&changed[path].bytes).unwrap(),
                    value,
                    "semantic validity and values retained for {path}"
                );
            }
            assert!(
                acquisition::authenticated(&before, &changed, &paths, &observation.outputs)
                    .is_err(),
                "{path} mutation {mutation}"
            );
            assert!(plan.require_equal(&changed).is_err());
        }
        let mut omitted = observation.outputs.clone();
        omitted.remove(path);
        assert!(acquisition::authenticated(&before, candidate, &paths, &omitted).is_err());
    }
    for path in [
        "cli_manifests/depth_fixture/RULES.json",
        "cli_manifests/depth_fixture/SCHEMA.json",
        "bundle/manifest.json",
        "acquisition-inputs.json",
        "docs/unrelated.txt",
    ] {
        let mut extra = observation.outputs.clone();
        extra.insert(path.into(), Some(Entry::file("100644", b"{}".to_vec())));
        assert!(acquisition::authenticated(&before, candidate, &paths, &extra).is_err());
        let mut extra = candidate.clone();
        extra.insert(
            path.into(),
            Entry::file("100644", b"candidate fingerprint/bundle".to_vec()),
        );
        assert!(plan.require_equal(&extra).is_err());
    }
    // Optional missing output is an authenticated fact, even if it is unchanged from parent.
    let mut optional = observation.outputs.clone();
    let absent = "optional-output.json".to_owned();
    optional.insert(absent.clone(), None);
    let mut optional_paths = paths;
    optional_paths.push(absent.clone());
    let delta = acquisition::authenticated(&before, candidate, &optional_paths, &optional).unwrap();
    assert!(!delta.apply(&before).unwrap().contains_key(&absent));
    let mut changed = candidate.clone();
    changed.insert(absent, Entry::file("100644", b"unexpected".to_vec()));
    assert!(acquisition::authenticated(&before, &changed, &optional_paths, &optional).is_err());
}

pub(super) fn host(
    f: &Fixture,
    publication: &Publication,
    retained: &Retained,
    plan: &crate::manifest_acquisition::AcquisitionPlan,
    branch: &str,
    record: &DepthRecord,
    observation: &Observation,
) {
    let root = f.root.path();
    let original = &retained.observation;
    let commit = &original.candidate.commit;
    let reference = &original.reference;
    let run_path = format!(
        "actions/runs/{}/attempts/{}",
        reference.run, reference.attempt
    );
    let jobs_path = format!(
        "actions/runs/{}/attempts/{}/jobs?per_page=100&page=1",
        reference.run, reference.attempt
    );
    let artifact_path = format!(
        "actions/runs/{}/artifacts?per_page=100&page=1",
        reference.run
    );
    // Every field is independently mutated; immutable fixture construction is shared.
    for (path, field, value) in [
        (&run_path, "id", json!(999)),
        (&run_path, "run_attempt", json!(999)),
        (&run_path, "workflow_id", json!(999)),
        (&run_path, "path", json!(".github/workflows/other.yml")),
        (&run_path, "head_sha", json!("a".repeat(40))),
        (&run_path, "repository_id", json!(99)),
        (&run_path, "inputs", json!({"commit":false})),
    ] {
        let host = f.api.3.fork();
        host.change(path, |v| v[field] = value);
        assert!(observed::read(&host, commit).is_err(), "{field}");
    }
    for (field, value) in [
        ("id", json!(999)),
        ("run_id", json!(999)),
        ("run_attempt", json!(999)),
        ("workflow_job_key", json!("snapshot")),
        ("status", json!("in_progress")),
        ("conclusion", json!("failure")),
        ("app", json!({"id":999})),
        ("binding_started_at", json!("2026-11-14T00:00:02Z")),
    ] {
        let host = f.api.3.fork();
        host.change(&jobs_path, |v| v["jobs"][0][field] = value);
        assert!(observed::read(&host, commit).is_err(), "job {field}");
    }
    for (field, value) in [
        ("id", json!(999)),
        ("expired", json!(true)),
        ("digest", json!("sha256:untrusted")),
        (
            "workflow_run",
            json!({"id":999,"head_sha":original.predecessor.commit}),
        ),
        ("size_in_bytes", json!(1)),
    ] {
        let host = f.api.3.fork();
        host.change(&artifact_path, |v| v["artifacts"][0][field] = value);
        assert!(observed::read(&host, commit).is_err(), "artifact {field}");
    }
    for which in 0..4 {
        let host = f.api.3.fork();
        let path = match which {
            0 => format!("acquisition-inputs/{commit}"),
            1 => "acquisition-policy".into(),
            2 => format!("actions/jobs/{}/logs", reference.job),
            _ => format!(
                "actions/artifacts/{}/acquisition-observation",
                reference.artifact
            ),
        };
        host.facts.lock().unwrap().remove(&path);
        assert!(observed::read(&host, commit).map_or(true, |v| v.is_none()));
    }
    let revoked = f.api.3.fork();
    revoked.put(
        "in1/revocations",
        json!({"records":[original.identity().unwrap()]}),
    );
    assert!(observed::read(&revoked, commit).is_err());
    // Even a coherent retained payload/job association cannot authorize another parent,
    // candidate, generation, invocation or descriptor-resolved plan.
    for which in 0..6 {
        let host = MemoryHost::default();
        let mut changed = original.clone();
        match which {
            0 => changed.predecessor.tree = "a".repeat(40),
            1 => changed.candidate.tree = "a".repeat(40),
            2 => changed.generation.policy = "a".repeat(64),
            3 => changed.inputs["ref"] = json!("other-branch"),
            4 => changed.plan["output_paths"] = json!(["crates/xtask/src/lib.rs"]),
            _ => changed.policy.implementation = "a".repeat(64),
        }
        super::host::observe(&host, &changed);
        let observed = observed::read(&host, commit).unwrap().unwrap();
        assert!(
            observed
                .require_binding(
                    root,
                    publication,
                    &original.predecessor,
                    &original.candidate,
                    &original.generation,
                    plan,
                    branch
                )
                .is_err(),
            "binding {which}"
        );
        let classification = freeze::classify_authenticated(
            root,
            publication,
            &original.predecessor,
            &original.candidate,
            record,
            Some(observation),
            &Ok(Some(observed)),
        );
        assert_eq!(
            classification.checkpoint,
            Checkpoint::ExecutionRequired,
            "binding {which}"
        );
        assert_eq!(
            classification.reason,
            Reason::AcquisitionAuthorityUnavailable,
            "binding {which}"
        );
    }
    // Candidate-authored authority files are committed and tested at real IN1 by the caller.
    assert!(observed::read(&MemoryHost::default(), commit)
        .unwrap()
        .is_none());
}
