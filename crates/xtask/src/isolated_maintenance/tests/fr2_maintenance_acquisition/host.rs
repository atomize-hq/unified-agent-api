//! Explicit fake-host retained authority. No candidate JSON can install these associations.
use super::*;
use crate::{
    depth_acceptance::admission::{
        canonical, digest, fake::MemoryHost, Object, Policy, Publication,
    },
    isolated_maintenance::acquisition::{self, Observation, Reference},
};

pub(super) fn observe(host: &MemoryHost, observation: &Observation) {
    let Reference {
        run,
        attempt,
        job,
        artifact,
    } = observation.reference;
    let policy = &observation.policy;
    host.put("acquisition-policy", serde_json::to_value(policy).unwrap());
    host.put(
        &format!("acquisition-inputs/{}", observation.candidate.commit),
        serde_json::to_value(&observation.reference).unwrap(),
    );
    host.put(&format!("actions/runs/{run}/attempts/{attempt}"), json!({
        "id":run,"run_attempt":attempt,"workflow_id":policy.workflow_id,"path":policy.workflow_path,
        "policy":policy,"event":"workflow_dispatch","head_sha":observation.predecessor.commit,
        "head_repository":{"full_name":observation.repository.name},"repository_id":observation.repository.id,
        "owner_id":observation.repository.owner,"inputs":observation.inputs,
        "predecessor":observation.predecessor,"candidate":observation.candidate,
    }));
    host.page(&format!("actions/runs/{run}/attempts/{attempt}/jobs"), "jobs", vec![json!({
        "id":job,"run_id":run,"run_attempt":attempt,"workflow_job_key":policy.job_key,"app":{"id":policy.app_id},
        "status":"completed","conclusion":"success","binding_started_at":"2026-11-14T00:00:00Z",
        "binding_completed_at":"2026-11-14T00:00:02Z",
    })]);
    let bytes = canonical(observation).unwrap();
    host.page(
        &format!("actions/runs/{run}/artifacts"),
        "artifacts",
        vec![json!({
            "id":artifact,"name":format!("ma2-checked-acquisition-{run}-{attempt}"),"expired":false,
            "size_in_bytes":bytes.len(),"digest":format!("sha256:{}",digest(b"",&bytes)),
            "workflow_run":{"id":run,"head_sha":observation.predecessor.commit},
        })],
    );
    host.facts.lock().unwrap().insert(
        format!("actions/artifacts/{artifact}/acquisition-observation"),
        bytes,
    );
    host.facts.lock().unwrap().insert(
        format!("actions/jobs/{job}/logs"),
        format!(
            "2026-11-14T00:00:01Z {}\n",
            acquisition::commitment(&observation.identity().unwrap(), artifact)
        )
        .into_bytes(),
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn capture(
    publication: &Publication,
    predecessor: &Object,
    candidate: &Object,
    record: &DepthRecord,
    plan: &crate::manifest_acquisition::AcquisitionPlan,
    branch: &str,
    bundle: &Path,
    modes: &std::collections::BTreeMap<String, String>,
) -> Observation {
    let outputs = plan
        .output_paths
        .iter()
        .map(|path| {
            let bytes = fs::read(bundle.join(path));
            let entry = match bytes {
                Ok(bytes) => Some(crate::freeze_transition::Entry::file(&modes[path], bytes)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                Err(e) => panic!("{e}"),
            };
            (path.clone(), entry)
        })
        .collect();
    Observation {
        schema_version: 1,
        repository: publication.authority.repository.clone(),
        predecessor: predecessor.clone(),
        candidate: candidate.clone(),
        generation: Generation::from_record(record),
        policy: Policy {
            workflow_id: 70,
            workflow_path: ".github/workflows/parity-acquire.yml".into(),
            job_key: "union".into(),
            ..publication.authority.policy.clone()
        },
        reference: Reference {
            run: 700,
            attempt: 2,
            job: 701,
            artifact: 702,
        },
        inputs: json!({"agent_id":plan.agent_id,"target_version":plan.version,"ref":branch,"commit":true}),
        plan: serde_json::to_value(plan).unwrap(),
        outputs,
    }
}
