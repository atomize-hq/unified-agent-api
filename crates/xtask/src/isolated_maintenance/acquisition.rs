//! Independently retained acquisition observations. This fake-host protocol is separate from
//! producer artifacts: an approved union job commits to the checked bundle's complete inventory.
//! Production Host defaults to no policy/reference and cannot issue acquisition authority.
use crate::depth_acceptance::admission::{
    canonical, digest, policy, transport, Generation, Host, Object, Policy, Publication, Repository,
};
use crate::freeze_transition::acquisition::Inventory;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

const LIMIT: usize = 536_870_912;
const DOMAIN: &[u8] = b"support-depth-ma2-acquisition-v1\0";
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Reference {
    pub run: u64,
    pub attempt: u64,
    pub job: u64,
    pub artifact: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Observation {
    pub schema_version: u32,
    pub repository: Repository,
    pub predecessor: Object,
    pub candidate: Object,
    pub generation: Generation,
    pub policy: Policy,
    pub reference: Reference,
    pub inputs: Value,
    pub plan: Value,
    /// Every resolved output has one entry, including absent optional outputs. Present entries
    /// contain checked bundle bytes, plus independently observed original Git mode/object kind.
    pub outputs: Inventory,
}
impl Observation {
    pub(crate) fn identity(&self) -> Result<String, String> {
        Ok(digest(DOMAIN, &canonical(self)?))
    }
}

/// Re-read this before planning and around publication. Equality includes every authenticated
/// host fact, including policy/reference, run, jobs, log, artifact metadata and bundle content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Retained {
    pub observation: Observation,
    facts: Vec<(String, Value)>,
    log: Vec<u8>,
    pub approved: Policy,
}
pub(crate) fn read(host: &dyn Host, candidate: &str) -> Result<Option<Retained>, String> {
    let Some(reference) = host.acquisition_reference(candidate)? else {
        return Ok(None);
    };
    let approved = host
        .acquisition_policy()?
        .ok_or("missing approved acquisition workflow")?;
    if [
        reference.run,
        reference.attempt,
        reference.job,
        reference.artifact,
    ]
    .contains(&0)
        || reference.attempt > 100
        || approved.workflow_id == 0
        || approved.app_id == 0
        || approved.workflow_path != ".github/workflows/parity-acquire.yml"
        || approved.job_key != "union"
    {
        return Err("invalid acquisition authority identity".into());
    }
    let Reference {
        run,
        attempt,
        job,
        artifact,
    } = reference;
    let details_path = format!("actions/runs/{run}/attempts/{attempt}");
    let details = host.json(&details_path)?;
    let mut facts = vec![(details_path, details.clone())];
    if details["id"] != run
        || details["run_attempt"] != attempt
        || details["workflow_id"] != approved.workflow_id
        || details["path"] != approved.workflow_path
        || details["policy"] != serde_json::to_value(&approved).map_err(|e| e.to_string())?
        || !matches!(
            details["event"].as_str(),
            Some("workflow_dispatch" | "workflow_call")
        )
    {
        return Err("acquisition workflow/run/attempt differs".into());
    }
    let (jobs, pages) = transport::list(
        host,
        &format!("actions/runs/{run}/attempts/{attempt}/jobs"),
        "jobs",
    )?;
    facts.extend(pages);
    let selected: Vec<_> = jobs
        .iter()
        .filter(|j| j["workflow_job_key"] == approved.job_key && j["app"]["id"] == approved.app_id)
        .collect();
    if selected.len() != 1 {
        return Err("missing/ambiguous approved acquisition job".into());
    }
    let selected = selected[0];
    if selected["id"] != job
        || selected["run_id"] != run
        || selected["run_attempt"] != attempt
        || selected["status"] != "completed"
        || selected["conclusion"] != "success"
    {
        return Err("acquisition job/attempt incomplete or differs".into());
    }
    let (artifacts, pages) =
        transport::list(host, &format!("actions/runs/{run}/artifacts"), "artifacts")?;
    facts.extend(pages);
    let artifacts: Vec<_> = artifacts.iter().filter(|a| a["id"] == artifact).collect();
    if artifacts.len() != 1 {
        return Err("missing/ambiguous acquisition bundle".into());
    }
    let metadata = artifacts[0];
    if metadata["expired"] != false
        || metadata["workflow_run"]["id"] != run
        || metadata["workflow_run"]["head_sha"] != details["head_sha"]
        || metadata["name"] != format!("ma2-checked-acquisition-{run}-{attempt}")
    {
        return Err("acquisition artifact association differs".into());
    }
    // Normalized host observation of the existing checked bundle, not a new producer file.
    let bytes = host.bytes(
        &format!("actions/artifacts/{artifact}/acquisition-observation"),
        LIMIT,
    )?;
    if bytes.len() > LIMIT
        || metadata["size_in_bytes"].as_u64() != Some(bytes.len() as u64)
        || metadata["digest"] != format!("sha256:{}", digest(b"", &bytes))
    {
        return Err("acquisition bundle size/digest differs".into());
    }
    let observation: Observation = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if canonical(&observation)? != bytes
        || observation.schema_version != 1
        || observation.reference != reference
        || observation.policy != approved
        || observation.candidate.commit != candidate
        || details["head_sha"] != observation.predecessor.commit
        || details["head_repository"]["full_name"] != observation.repository.name
        || details["repository_id"] != observation.repository.id
        || details["owner_id"] != observation.repository.owner
        || details["inputs"] != observation.inputs
        || details["candidate"]
            != serde_json::to_value(&observation.candidate).map_err(|e| e.to_string())?
        || details["predecessor"]
            != serde_json::to_value(&observation.predecessor).map_err(|e| e.to_string())?
    {
        return Err("acquisition payload differs from independent host association".into());
    }
    let log = host.bytes(&format!("actions/jobs/{job}/logs"), 16_777_216)?;
    if log.len() > 16_777_216 {
        return Err("acquisition log bound exceeded".into());
    }
    let revocations = host.json("in1/revocations")?;
    let identities = revocations["records"]
        .as_array()
        .ok_or("missing acquisition revocation observation")?;
    if identities.iter().any(|v| v.as_str().is_none())
        || identities.contains(&Value::String(observation.identity()?))
    {
        return Err("acquisition authority revoked/invalid revocations".into());
    }
    facts.push(("in1/revocations".into(), revocations));
    let text = std::str::from_utf8(&log).map_err(|e| e.to_string())?;
    let lines: Vec<_> = text
        .lines()
        .filter_map(|l| l.split_once(' '))
        .filter(|(_, l)| l.starts_with("MA2_ACQUISITION_V1"))
        .collect();
    if lines.len() != 1 || lines[0].1 != commitment(&observation.identity()?, artifact) {
        return Err("acquisition emitting-job commitment differs".into());
    }
    let stamp = |v: &str| {
        time::OffsetDateTime::parse(v, &time::format_description::well_known::Rfc3339)
            .map_err(|_| "invalid acquisition commitment time")
    };
    let start = selected["binding_started_at"]
        .as_str()
        .ok_or("missing acquisition start")?;
    let end = selected["binding_completed_at"]
        .as_str()
        .ok_or("missing acquisition end")?;
    if stamp(lines[0].0)? < stamp(start)? || stamp(lines[0].0)? > stamp(end)? {
        return Err("acquisition commitment outside emitting step".into());
    }
    Ok(Some(Retained {
        observation,
        facts,
        log,
        approved,
    }))
}
pub(crate) fn commitment(identity: &str, artifact: u64) -> String {
    format!("MA2_ACQUISITION_V1 kind=ma2_acquisition sha256={identity} artifact={artifact}")
}
impl Retained {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn require_binding(
        &self,
        root: &Path,
        publication: &Publication,
        parent: &Object,
        candidate: &Object,
        generation: &Generation,
        plan: &crate::manifest_acquisition::AcquisitionPlan,
        branch: &str,
    ) -> Result<(), String> {
        policy::authenticate(root, &self.approved, &publication.authority.policy)?;
        let o = &self.observation;
        let inputs = serde_json::json!({"agent_id":plan.agent_id,"target_version":plan.version,"ref":branch,"commit":true});
        if o.repository != publication.authority.repository
            || o.predecessor != *parent
            || o.candidate != *candidate
            || o.generation != *generation
            || o.inputs != inputs
            || o.plan != serde_json::to_value(plan).map_err(|e| e.to_string())?
        {
            return Err("acquisition binding differs from authenticated predecessor/plan".into());
        }
        Ok(())
    }
}
