//! Independently mutable fake host facts, shared explicitly with each fixture. No thread state.
use super::*;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    io::{Seek, Write},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
};

pub(crate) struct MemoryHost {
    pub facts: Mutex<BTreeMap<String, Vec<u8>>>,
    next: AtomicU64,
    next_retry: Mutex<Option<Record>>,
    pub fail_upload: std::sync::atomic::AtomicBool,
    pub fail_commit: std::sync::atomic::AtomicBool,
}
impl Default for MemoryHost {
    fn default() -> Self {
        let host = Self {
            facts: Mutex::new(BTreeMap::new()),
            next: AtomicU64::new(1000),
            next_retry: Mutex::new(None),
            fail_upload: false.into(),
            fail_commit: false.into(),
        };
        host.put("in1/revocations", json!({"records":[]}));
        host.page(transport::RUNS, "workflow_runs", vec![]);
        host
    }
}
impl MemoryHost {
    pub(crate) fn put(&self, path: &str, value: Value) {
        self.facts
            .lock()
            .unwrap()
            .insert(path.into(), serde_json::to_vec(&value).unwrap());
    }
    pub(crate) fn page(&self, path: &str, member: &str, rows: Vec<Value>) {
        self.put(
            &format!("{path}?per_page=100&page=1"),
            json!({"total_count":rows.len(),member:rows}),
        );
    }
    pub(crate) fn change(&self, path: &str, change: impl FnOnce(&mut Value)) {
        let mut facts = self.facts.lock().unwrap();
        let mut value: Value = serde_json::from_slice(&facts[path]).unwrap();
        change(&mut value);
        facts.insert(path.into(), serde_json::to_vec(&value).unwrap());
    }
    pub(crate) fn root(&self, tip: &str) {
        self.put(
            "",
            json!({"id":10,"owner":{"id":20},"full_name":"fixture/isolated"}),
        );
        self.put("git/ref/heads/staging", json!({"object":{"sha":tip}}));
    }
    pub(crate) fn retry(&self, prior: &Record) -> Claims {
        let claims = Claims {
            attempt: prior.claims.attempt + 1,
            job: prior.claims.job + 100_000,
            check: prior.claims.check + 100_000,
            ..prior.claims.clone()
        };
        let mut details = self
            .json(&format!(
                "actions/runs/{}/attempts/{}",
                claims.run, prior.claims.attempt
            ))
            .unwrap();
        details["run_attempt"] = json!(claims.attempt);
        self.put(
            &format!("actions/runs/{}/attempts/{}", claims.run, claims.attempt),
            details,
        );
        self.change(&format!("{}?per_page=100&page=1", transport::RUNS), |v| {
            v["workflow_runs"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|v| v["id"] == claims.run)
                .unwrap()["run_attempt"] = json!(claims.attempt);
        });
        let mut jobs = self
            .json(&format!(
                "actions/runs/{}/attempts/{}/jobs?per_page=100&page=1",
                claims.run, prior.claims.attempt
            ))
            .unwrap();
        jobs["jobs"][0]["id"] = json!(claims.job);
        jobs["jobs"][0]["check_run_id"] = json!(claims.check);
        jobs["jobs"][0]["run_attempt"] = json!(claims.attempt);
        jobs["jobs"][0]["status"] = json!("in_progress");
        self.put(
            &format!(
                "actions/runs/{}/attempts/{}/jobs?per_page=100&page=1",
                claims.run, claims.attempt
            ),
            jobs,
        );
        claims
    }
    pub(crate) fn retry_on_next_begin(&self, prior: &Record) {
        *self.next_retry.lock().unwrap() = Some(prior.clone());
    }
    pub(crate) fn fork(&self) -> Self {
        Self {
            facts: Mutex::new(self.facts.lock().unwrap().clone()),
            next: AtomicU64::new(self.next.load(Ordering::SeqCst)),
            next_retry: Mutex::new(None),
            fail_upload: false.into(),
            fail_commit: false.into(),
        }
    }
    pub(crate) fn records(&self) -> Vec<Record> {
        self.facts
            .lock()
            .unwrap()
            .iter()
            .filter(|(key, _)| key.starts_with("payload/"))
            .map(|(_, bytes)| Record::parse(bytes).unwrap())
            .collect()
    }
}
impl Host for MemoryHost {
    fn json(&self, path: &str) -> Result<Value, String> {
        serde_json::from_slice(&self.bytes(path, 16_777_216)?).map_err(|e| e.to_string())
    }
    fn bytes(&self, path: &str, limit: usize) -> Result<Vec<u8>, String> {
        let bytes = self
            .facts
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| format!("missing fake IN1 host fact {path}"))?;
        if bytes.len() > limit {
            return Err("fake IN1 response bound exceeded".into());
        }
        Ok(bytes)
    }
    fn begin(&self, candidate: &str, policy: &Policy) -> Result<Claims, String> {
        if let Some(prior) = self.next_retry.lock().unwrap().take() {
            if prior.candidate.commit != candidate || prior.policy != *policy {
                return Err("fake retry candidate/policy mismatch".into());
            }
            return Ok(self.retry(&prior));
        }
        let run = self.next.fetch_add(1, Ordering::SeqCst);
        let claims = Claims {
            run,
            attempt: 1,
            job: run + 10_000,
            check: run + 20_000,
            event: "pull_request".into(),
        };
        let details = json!({"id":run,"run_attempt":1,"head_sha":candidate,"head_repository":{"full_name":"fixture/isolated"},"path":policy.workflow_path,"workflow_id":policy.workflow_id,"policy":policy,"event":claims.event});
        self.put(&format!("actions/runs/{run}/attempts/1"), details.clone());
        self.change(&format!("{}?per_page=100&page=1", transport::RUNS), |v| {
            v["workflow_runs"].as_array_mut().unwrap().push(details);
            v["total_count"] = json!(v["workflow_runs"].as_array().unwrap().len());
        });
        self.page(&format!("actions/runs/{run}/attempts/1/jobs"), "jobs", vec![json!({"id":claims.job,"run_id":run,"run_attempt":1,"check_run_id":claims.check,"name":"IN1","workflow_job_key":policy.job_key,"app":{"id":policy.app_id},"status":"in_progress","conclusion":null,"binding_started_at":"2026-11-13T00:00:00Z","binding_completed_at":"2026-11-13T00:00:02Z"})]);
        self.page(
            &format!("actions/runs/{run}/artifacts"),
            "artifacts",
            vec![],
        );
        Ok(claims)
    }
    fn upload(&self, claims: &Claims, bytes: &[u8]) -> Result<u64, String> {
        if self.fail_upload.load(Ordering::SeqCst) {
            return Err("fake IN1 upload failed".into());
        }
        let record = Record::parse(bytes)?;
        let zip = archive(bytes)?;
        let artifact = claims.run + 30_000 + claims.attempt * 100_000;
        self.facts.lock().unwrap().insert(
            format!("payload/{}-{}", claims.run, claims.attempt),
            bytes.to_vec(),
        );
        self.facts
            .lock()
            .unwrap()
            .insert(format!("actions/artifacts/{artifact}/zip"), zip.clone());
        let path = format!("actions/runs/{}/artifacts", claims.run);
        let mut rows = self.json(&format!("{path}?per_page=100&page=1"))?["artifacts"]
            .as_array()
            .unwrap()
            .clone();
        rows.push(json!({"id":artifact,"name":format!("in1-admission-{}-{}",claims.run,claims.attempt),"expired":false,"size_in_bytes":zip.len(),"digest":format!("sha256:{}",digest(b"",&zip)),"workflow_run":{"id":claims.run,"head_sha":record.candidate.commit}}));
        self.page(&path, "artifacts", rows);
        Ok(artifact)
    }
    fn commit(&self, claims: &Claims, line: &str) -> Result<(), String> {
        if self.fail_commit.load(Ordering::SeqCst) {
            return Err("fake IN1 commitment failed".into());
        }
        let record = Record::parse(&self.bytes(
            &format!("payload/{}-{}", claims.run, claims.attempt),
            RECORD_LIMIT,
        )?)?;
        self.change(
            &format!(
                "actions/runs/{}/attempts/{}/jobs?per_page=100&page=1",
                claims.run, claims.attempt
            ),
            |v| {
                v["jobs"][0]["status"] = json!("completed");
                v["jobs"][0]["conclusion"] = json!(if record.successful() {
                    "success"
                } else {
                    "failure"
                });
            },
        );
        self.facts.lock().unwrap().insert(
            format!("actions/jobs/{}/logs", claims.job),
            format!("2026-11-13T00:00:01Z {line}\n").into_bytes(),
        );
        Ok(())
    }
}
pub(crate) fn archive(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut input = tempfile::tempfile().map_err(|e| e.to_string())?;
    input.write_all(bytes).map_err(|e| e.to_string())?;
    input.rewind().map_err(|e| e.to_string())?;
    let result = Command::new("python3").args(["-c", "import sys,zipfile,io; b=io.BytesIO(); z=zipfile.ZipFile(b,'w'); z.writestr('admission.json',sys.stdin.buffer.read()); z.close(); sys.stdout.buffer.write(b.getvalue())"])
        .stdin(Stdio::from(input)).output().map_err(|e| e.to_string())?;
    if !result.status.success() {
        return Err("fake artifact ZIP creation failed".into());
    }
    Ok(result.stdout)
}
