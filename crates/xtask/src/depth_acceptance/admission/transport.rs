//! Attempt-specific job/log/artifact authentication over the existing proof transport.
use super::*;
use serde_json::Value;
use std::{collections::BTreeSet, path::Path, sync::Arc};

pub(crate) const RUNS: &str = "actions/workflows/ci.yml/runs";
type Observations = Vec<(String, Value)>;
const COMMITMENT: &str = "IN1_ADMISSION_V1";
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Authority {
    pub repository: Repository,
    pub anchor: Object,
    pub policy: Policy,
}
pub(crate) trait Host: Send + Sync {
    fn json(&self, path: &str) -> Result<Value, String>;
    fn bytes(&self, path: &str, limit: usize) -> Result<Vec<u8>, String>;
    /// Independent invocation observation. Production has no activated implementation.
    fn freeze_inputs(
        &self,
        _candidate: &str,
    ) -> Result<Option<freeze::authority::Observation>, String> {
        Ok(None)
    }
    fn acquisition_reference(
        &self,
        _candidate: &str,
    ) -> Result<Option<freeze::acquisition::Reference>, String> {
        Ok(None)
    }
    fn acquisition_policy(&self) -> Result<Option<Policy>, String> {
        Ok(None)
    }
    fn begin(&self, candidate: &str, policy: &Policy) -> Result<Claims, String>;
    fn upload(&self, claims: &Claims, bytes: &[u8]) -> Result<u64, String>;
    fn commit(&self, claims: &Claims, line: &str) -> Result<(), String>;
}
#[derive(Clone)]
pub(crate) struct Publication {
    pub authority: Authority,
    pub host: Arc<dyn Host>,
}
impl std::fmt::Debug for Publication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.authority.fmt(f)
    }
}
impl PartialEq for Publication {
    fn eq(&self, other: &Self) -> bool {
        self.authority == other.authority && Arc::ptr_eq(&self.host, &other.host)
    }
}
impl Eq for Publication {}
pub(crate) struct Session {
    pub publication: Publication,
    pub claims: Claims,
}
impl Publication {
    pub(crate) fn start(&self, candidate: &str) -> Result<Session, String> {
        Ok(Session {
            publication: self.clone(),
            claims: self.host.begin(candidate, &self.authority.policy)?,
        })
    }
    pub(crate) fn resolve(
        &self,
        root: &Path,
        roots: &[String],
    ) -> Result<history::History, String> {
        let snapshot = Snapshot::read(self.host.as_ref(), &self.authority)?;
        if roots.first().map(String::as_str) != snapshot.observations[1].1["object"]["sha"].as_str()
        {
            return Err("IN1 protected root differs from captured binding".into());
        }
        let mut history = history::resolve(root, &self.authority, &snapshot.records, roots)?;
        snapshot.recheck(self.host.as_ref(), &self.authority)?;
        history.revoked = snapshot.revoked.clone();
        history.observation = Some((self.clone(), snapshot));
        Ok(history)
    }
}
impl Session {
    pub(crate) fn publish(&self, record: &Record) -> Result<(), String> {
        if record.claims != self.claims {
            return Err("IN1 emitter claims changed".into());
        }
        let bytes = record.canonical()?;
        let artifact = self.publication.host.upload(&self.claims, &bytes)?;
        if artifact == 0 {
            return Err("IN1 upload returned no artifact identity".into());
        }
        self.publication
            .host
            .commit(&self.claims, &commitment(&record.identity()?, artifact))?;
        // Successful publication includes authentication through separately retrieved job facts.
        let retained = authenticate(
            self.publication.host.as_ref(),
            &self.publication.authority,
            self.claims.run,
            self.claims.attempt,
        )?;
        if self.publication.host.json("git/ref/heads/staging")?["object"]["sha"]
            != record.protected_tip.commit
        {
            return Err("IN1 protected root changed around publication".into());
        }
        if retained != *record {
            return Err("IN1 published decision differs from evaluated decision".into());
        }
        Ok(())
    }
}
pub(crate) fn commitment(digest: &str, artifact: u64) -> String {
    format!("{COMMITMENT} kind=in1_admission sha256={digest} artifact={artifact}")
}
pub(super) struct Snapshot {
    pub records: Vec<Record>,
    observations: Observations,
    revoked: BTreeSet<String>,
    inventory: Vec<Value>,
}
impl Snapshot {
    fn read(host: &dyn Host, authority: &Authority) -> Result<Self, String> {
        let mut observations = vec![];
        for path in ["", "git/ref/heads/staging", "in1/revocations"] {
            observations.push((path.into(), host.json(path)?));
        }
        let repository = &observations[0].1;
        if repository["id"] != authority.repository.id
            || repository["owner"]["id"] != authority.repository.owner
            || repository["full_name"] != authority.repository.name
        {
            return Err("IN1 host repository identity mismatch".into());
        }
        let revoked = observations[2].1["records"]
            .as_array()
            .ok_or("IN1 unavailable revocation observation")?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or("IN1 invalid revocation identity".to_owned())
            })
            .collect::<Result<BTreeSet<_>, _>>()?;
        let inventory = list(host, RUNS, "workflow_runs")?;
        observations.extend(inventory.1);
        let mut records = vec![];
        let inventory_rows = inventory.0.clone();
        for run in inventory.0 {
            let id = number(&run, "id")?;
            let count = number(&run, "run_attempt")?;
            if count > 100 {
                return Err("IN1 attempt population bound exceeded".into());
            }
            for attempt in 1..=count {
                let path = format!("actions/runs/{id}/attempts/{attempt}");
                let details = host.json(&path)?;
                for key in [
                    "head_sha",
                    "path",
                    "workflow_id",
                    "head_repository",
                    "event",
                ] {
                    if run[key] != details[key] {
                        return Err("IN1 run inventory differs from attempt association".into());
                    }
                }
                observations.push((path, details));
                for (path, member) in [
                    (format!("actions/runs/{id}/attempts/{attempt}/jobs"), "jobs"),
                    (format!("actions/runs/{id}/artifacts"), "artifacts"),
                ] {
                    observations.extend(list(host, &path, member)?.1);
                }
                let record = authenticate(host, authority, id, attempt)?;
                if run["head_sha"] != record.candidate.commit || run["run_attempt"] != count {
                    return Err("IN1 run inventory differs from attempt".into());
                }
                records.push(record);
            }
        }
        Ok(Self {
            records,
            observations,
            revoked,
            inventory: inventory_rows,
        })
    }
    pub(super) fn recheck_publication(
        &self,
        publication: &Publication,
        claims: &Claims,
    ) -> Result<(), String> {
        let rows = list(publication.host.as_ref(), RUNS, "workflow_runs")?.0;
        let mut expected = self.inventory.clone();
        let retry = expected.iter_mut().find(|row| row["id"] == claims.run);
        if let Some(prior) = retry {
            if prior["run_attempt"].as_u64().and_then(|n| n.checked_add(1)) != Some(claims.attempt)
            {
                return Err("IN1 retry skipped an attempt".into());
            }
            prior["run_attempt"] = claims.attempt.into();
        } else {
            if claims.attempt != 1 || rows.len() != expected.len() + 1 {
                return Err("IN1 first publication attempt or population changed".into());
            }
            let index = rows
                .iter()
                .position(|row| row["id"] == claims.run)
                .ok_or("IN1 publication missing from inventory")?;
            expected.insert(index, rows[index].clone());
        }
        if rows != expected {
            return Err("IN1 admission population changed around publication".into());
        }
        let artifact_path = format!("actions/runs/{}/artifacts", claims.run);
        let artifact_query = format!("{artifact_path}?");
        let mut pages = BTreeSet::new();
        let prior_artifacts: Vec<_> = self
            .observations
            .iter()
            .filter(|(path, _)| path.starts_with(&artifact_query) && pages.insert(path))
            .flat_map(|(_, value)| value["artifacts"].as_array().into_iter().flatten().cloned())
            .collect();
        if !prior_artifacts.is_empty() {
            let mut artifacts = list(publication.host.as_ref(), &artifact_path, "artifacts")?.0;
            let name = format!("in1-admission-{}-{}", claims.run, claims.attempt);
            let count = artifacts.len();
            artifacts.retain(|row| row["name"] != name);
            if count != prior_artifacts.len() + 1 || artifacts != prior_artifacts {
                return Err("IN1 earlier artifacts changed around retry publication".into());
            }
        }
        for (path, before) in &self.observations {
            if !path.starts_with(RUNS)
                && !path.starts_with(&artifact_query)
                && publication.host.json(path)? != *before
            {
                return Err("IN1 roots or earlier attempts changed around publication".into());
            }
        }
        Ok(())
    }
    pub(super) fn recheck(&self, host: &dyn Host, _: &Authority) -> Result<(), String> {
        for (path, before) in &self.observations {
            if host.json(path)? != *before {
                return Err("IN1 roots or host population changed during discovery".into());
            }
        }
        Ok(())
    }
}
pub(super) fn authenticate(
    host: &dyn Host,
    authority: &Authority,
    run: u64,
    attempt: u64,
) -> Result<Record, String> {
    let details = host.json(&format!("actions/runs/{run}/attempts/{attempt}"))?;
    if details["id"] != run
        || details["run_attempt"] != attempt
        || details["workflow_id"] != authority.policy.workflow_id
        || details["path"] != authority.policy.workflow_path
        || details["head_repository"]["full_name"] != authority.repository.name
        || details["policy"]
            != serde_json::to_value(&authority.policy).map_err(|e| e.to_string())?
    {
        return Err("IN1 wrong workflow/attempt/policy provenance".into());
    }
    let jobs = list(
        host,
        &format!("actions/runs/{run}/attempts/{attempt}/jobs"),
        "jobs",
    )?
    .0;
    // Stable workflow job identity and app provenance, independently observed by this fake host.
    let jobs: Vec<_> = jobs
        .iter()
        .filter(|j| {
            j["workflow_job_key"] == authority.policy.job_key
                && j["app"]["id"] == authority.policy.app_id
        })
        .collect();
    if jobs.len() != 1 {
        return Err("IN1 missing/ambiguous approved job identity".into());
    }
    let job = jobs[0];
    let job_id = number(job, "id")?;
    if job["run_id"] != run
        || job["run_attempt"] != attempt
        || job["status"] != "completed"
        || !matches!(job["conclusion"].as_str(), Some("success" | "failure"))
    {
        return Err("IN1 incomplete or wrong-attempt job".into());
    }
    let logs = host.bytes(&format!("actions/jobs/{job_id}/logs"), 16_777_216)?;
    if logs.len() > 16_777_216 {
        return Err("IN1 job log bound exceeded".into());
    }
    let logs = std::str::from_utf8(&logs).map_err(|e| e.to_string())?;
    let lines: Vec<_> = logs
        .lines()
        .filter_map(|line| line.split_once(' '))
        .filter(|(_, body)| body.starts_with(COMMITMENT))
        .collect();
    if lines.len() != 1 {
        return Err("IN1 missing/ambiguous job commitment".into());
    }
    let (time, line) = lines[0];
    let started = string(job, "binding_started_at")?;
    let completed = string(job, "binding_completed_at")?;
    let parse_time = |value| {
        time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
            .map_err(|_| "IN1 invalid binding timestamp")
    };
    if parse_time(time)? < parse_time(started)? || parse_time(time)? > parse_time(completed)? {
        return Err("IN1 commitment outside emitting step".into());
    }
    let tokens: Vec<_> = line.split(' ').collect();
    if tokens.len() != 4 || tokens[0] != COMMITMENT || tokens[1] != "kind=in1_admission" {
        return Err("IN1 malformed commitment".into());
    }
    let digest = tokens[2]
        .strip_prefix("sha256=")
        .ok_or("IN1 missing payload digest")?;
    sha256(digest)?;
    let artifact_id: u64 = tokens[3]
        .strip_prefix("artifact=")
        .ok_or("IN1 missing artifact identity")?
        .parse()
        .map_err(|_| "IN1 invalid artifact identity")?;
    if commitment(digest, artifact_id) != line {
        return Err("IN1 noncanonical commitment".into());
    }
    let artifacts = list(host, &format!("actions/runs/{run}/artifacts"), "artifacts")?.0;
    let artifacts: Vec<_> = artifacts
        .iter()
        .filter(|a| a["id"] == artifact_id)
        .collect();
    if artifacts.len() != 1 {
        return Err("IN1 commitment artifact missing".into());
    }
    let artifact = artifacts[0];
    if artifact["expired"] != false
        || artifact["workflow_run"]["id"] != run
        || artifact["workflow_run"]["head_sha"] != details["head_sha"]
    {
        return Err("IN1 expired or wrong-run artifact".into());
    }
    let zip = host.bytes(&format!("actions/artifacts/{artifact_id}/zip"), 2_097_152)?;
    if zip.len() > 2_097_152
        || artifact["size_in_bytes"].as_u64() != Some(zip.len() as u64)
        || artifact["digest"] != format!("sha256:{}", digest_bytes(&zip))
    {
        return Err("IN1 artifact digest/size mismatch".into());
    }
    let record = Record::parse(&decode(&zip)?)?;
    if record.identity()? != digest
        || record.claims.run != run
        || record.claims.attempt != attempt
        || record.claims.job != job_id
        || job["check_run_id"] != record.claims.check
        || details["event"] != record.claims.event
        || details["head_sha"] != record.candidate.commit
        || record.policy != authority.policy
        || record.repository != authority.repository
        || record.enrollment_anchor != authority.anchor
        || job["conclusion"]
            != if record.decision == Decision::Admitted {
                "success"
            } else {
                "failure"
            }
    {
        return Err("IN1 payload differs from independent host association".into());
    }
    Ok(record)
}
fn digest_bytes(bytes: &[u8]) -> String {
    digest(b"", bytes)
}
fn decode(zip: &[u8]) -> Result<Vec<u8>, String> {
    use std::{
        io::{Seek, Write},
        process::{Command, Stdio},
    };
    let mut input = tempfile::tempfile().map_err(|e| e.to_string())?;
    input.write_all(zip).map_err(|e| e.to_string())?;
    input.rewind().map_err(|e| e.to_string())?;
    let result = Command::new("/usr/bin/timeout")
        .args([
            "30",
            "python3",
            "-c",
            include_str!("../../../../../scripts/depth_ci_archive.py"),
            "in1",
        ])
        .stdin(Stdio::from(input))
        .output()
        .map_err(|e| e.to_string())?;
    if !result.status.success() || result.stdout.len() > RECORD_LIMIT {
        return Err("IN1 unsupported bounded artifact ZIP".into());
    }
    Ok(result.stdout)
}
pub(crate) fn list(
    host: &dyn Host,
    path: &str,
    member: &str,
) -> Result<(Vec<Value>, Observations), String> {
    let mut rows = vec![];
    let mut observations = vec![];
    let mut seen = BTreeSet::new();
    let mut total = None;
    for page in 1..=10 {
        let path = format!("{path}?per_page=100&page={page}");
        let value = host.json(&path)?;
        let count = value["total_count"]
            .as_u64()
            .ok_or("IN1 missing population total")?;
        if count > 1000 || total.replace(count).is_some_and(|old| old != count) {
            return Err("IN1 population bound/change".into());
        }
        let batch = value[member]
            .as_array()
            .ok_or("IN1 missing population rows")?;
        if batch.len() > 100 {
            return Err("IN1 pagination page bound".into());
        }
        for row in batch {
            if !seen.insert(number(row, "id")?) {
                return Err("IN1 duplicate population member".into());
            }
            rows.push(row.clone());
        }
        let last = batch.len() < 100 || rows.len() == count as usize;
        observations.push((path, value));
        if last {
            if rows.len() != count as usize {
                return Err("IN1 population gap".into());
            }
            return Ok((rows, observations));
        }
    }
    Err("IN1 host population traversal bound".into())
}
fn number(value: &Value, key: &str) -> Result<u64, String> {
    value[key]
        .as_u64()
        .filter(|n| *n > 0)
        .ok_or_else(|| format!("IN1 missing positive {key}"))
}
fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value[key]
        .as_str()
        .ok_or_else(|| format!("IN1 missing {key}"))
}
