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

type ReportingUploadHook = Box<dyn FnOnce(&MemoryHost) + Send>;

pub(crate) struct MemoryHost {
    pub facts: Mutex<BTreeMap<String, Vec<u8>>>,
    next: AtomicU64,
    next_retry: Mutex<Option<Record>>,
    pub after_reporting_upload: Mutex<Option<ReportingUploadHook>>,
    pub fail_reporting_execution_once: std::sync::atomic::AtomicBool,
    pub fail_freeze_after_upload: std::sync::atomic::AtomicBool,
    pub fail_freeze_after_capture: std::sync::atomic::AtomicBool,
    pub fail_history_after_capture: std::sync::atomic::AtomicBool,
    pub fail_upload: std::sync::atomic::AtomicBool,
    pub fail_commit: std::sync::atomic::AtomicBool,
}
impl Default for MemoryHost {
    fn default() -> Self {
        let host = Self {
            facts: Mutex::new(BTreeMap::new()),
            next: AtomicU64::new(1000),
            next_retry: Mutex::new(None),
            after_reporting_upload: Mutex::new(None),
            fail_reporting_execution_once: false.into(),
            fail_freeze_after_upload: false.into(),
            fail_freeze_after_capture: false.into(),
            fail_history_after_capture: false.into(),
            fail_upload: false.into(),
            fail_commit: false.into(),
        };
        host.put("in1/revocations", json!({"records":[]}));
        host.page(transport::RUNS, "workflow_runs", vec![]);
        host.page("reporting/runs", "workflow_runs", vec![]);
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
            after_reporting_upload: Mutex::new(None),
            fail_reporting_execution_once: false.into(),
            fail_freeze_after_upload: false.into(),
            fail_freeze_after_capture: false.into(),
            fail_history_after_capture: false.into(),
            fail_upload: false.into(),
            fail_commit: false.into(),
        }
    }
    pub(crate) fn inputs_identity(&self, record: &Record) -> String {
        let records = self.records();
        let predecessors: Vec<_> = records
            .iter()
            .filter(|prior| record.predecessors.contains(&prior.identity().unwrap()))
            .collect();
        record.inputs_identity(&predecessors).unwrap()
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
        if path.is_empty()
            && !observations::capturing()
            && self
                .fail_history_after_capture
                .swap(false, Ordering::SeqCst)
        {
            return Err("fake post-capture history read failed".into());
        }
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
    fn freeze_inputs(
        &self,
        candidate: &str,
    ) -> Result<Option<freeze::authority::Observation>, String> {
        if !observations::capturing()
            && self.fail_freeze_after_capture.swap(false, Ordering::SeqCst)
        {
            return Err("fake post-capture freeze read failed".into());
        }
        self.facts
            .lock()
            .unwrap()
            .get(&format!("freeze-inputs/{candidate}"))
            .map(|bytes| serde_json::from_slice(bytes).map_err(|e| e.to_string()))
            .transpose()
    }
    fn acquisition_reference(
        &self,
        candidate: &str,
    ) -> Result<Option<freeze::acquisition::Reference>, String> {
        self.facts
            .lock()
            .unwrap()
            .get(&format!("acquisition-inputs/{candidate}"))
            .map(|b| serde_json::from_slice(b).map_err(|e| e.to_string()))
            .transpose()
    }
    fn acquisition_policy(&self) -> Result<Option<Policy>, String> {
        self.facts
            .lock()
            .unwrap()
            .get("acquisition-policy")
            .map(|b| serde_json::from_slice(b).map_err(|e| e.to_string()))
            .transpose()
    }
    fn reporting_candidate(&self) -> Result<Option<String>, String> {
        self.facts
            .lock()
            .unwrap()
            .get("reporting/current")
            .map(|bytes| serde_json::from_slice(bytes).map_err(|e| e.to_string()))
            .transpose()
    }
    fn reporting_invocation(
        &self,
        candidate: &str,
    ) -> Result<Option<crate::depth_ci::reporting::Invocation>, String> {
        self.facts
            .lock()
            .unwrap()
            .get(&format!("reporting/candidates/{candidate}"))
            .map(|b| serde_json::from_slice(b).map_err(|e| e.to_string()))
            .transpose()
    }
    fn reporting_attempt(
        &self,
        run: u64,
        attempt: u64,
    ) -> Result<Option<crate::depth_ci::reporting::Invocation>, String> {
        self.facts
            .lock()
            .unwrap()
            .get(&format!("reporting/invocations/{run}/{attempt}"))
            .map(|b| serde_json::from_slice(b).map_err(|e| e.to_string()))
            .transpose()
    }
    fn reporting_execution(&self, run: u64, attempt: u64) -> Result<bool, String> {
        if self
            .fail_reporting_execution_once
            .swap(false, Ordering::SeqCst)
        {
            return Err("fake reporting execution observation unavailable".into());
        }
        self.facts
            .lock()
            .unwrap()
            .get(&format!("reporting/execution/{run}/{attempt}"))
            .map_or(Ok(false), |bytes| {
                serde_json::from_slice(bytes).map_err(|e| e.to_string())
            })
    }
    fn event(&self) -> Result<String, String> {
        Ok(self.next_retry.lock().unwrap().as_ref().map_or_else(
            || "pull_request".into(),
            |record| record.claims.event.clone(),
        ))
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
        let inventory = if policy.job_key == "rust-tests" {
            "reporting/runs"
        } else {
            transport::RUNS
        };
        self.change(&format!("{inventory}?per_page=100&page=1"), |v| {
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
        let value: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        let reporting = value["kind"] == "reporting_only";
        if reporting {
            crate::depth_ci::reporting::ContextRecord::parse(bytes)?;
        } else {
            Record::parse(bytes)?;
        }
        let prefix = if reporting {
            "reporting-payload"
        } else {
            "payload"
        };
        let zip = archive(bytes)?;
        let path = format!("actions/runs/{}/artifacts", claims.run);
        let mut rows = self.json(&format!("{path}?per_page=100&page=1"))?["artifacts"]
            .as_array()
            .unwrap()
            .clone();
        let artifact = self.next.fetch_add(1, Ordering::SeqCst) + 1_000_000;
        self.facts.lock().unwrap().insert(
            format!("{prefix}/{}-{}", claims.run, claims.attempt),
            bytes.to_vec(),
        );
        self.facts
            .lock()
            .unwrap()
            .insert(format!("actions/artifacts/{artifact}/zip"), zip.clone());
        rows.push(json!({"id":artifact,"name":format!("in1-admission-{}-{}",claims.run,claims.attempt),"expired":false,"size_in_bytes":zip.len(),"digest":format!("sha256:{}",digest(b"",&zip)),"workflow_run":{"id":claims.run,"head_sha":value["candidate"]["commit"]}}));
        self.page(&path, "artifacts", rows);
        if reporting {
            let after = self.after_reporting_upload.lock().unwrap().take();
            if let Some(after) = after {
                after(self);
            }
        }
        if self.fail_freeze_after_upload.swap(false, Ordering::SeqCst) {
            self.fail_freeze_after_capture.store(true, Ordering::SeqCst);
        }
        Ok(artifact)
    }
    fn commit(&self, claims: &Claims, line: &str) -> Result<(), String> {
        if self.fail_commit.load(Ordering::SeqCst) {
            return Err("fake IN1 commitment failed".into());
        }
        let reporting = line.starts_with(crate::depth_ci::reporting::PREFIX);
        let prefix = if reporting {
            "reporting-payload"
        } else {
            "payload"
        };
        let bytes = self.bytes(
            &format!("{prefix}/{}-{}", claims.run, claims.attempt),
            RECORD_LIMIT,
        )?;
        let success = if reporting {
            crate::depth_ci::reporting::ContextRecord::parse(&bytes)?.successful()
        } else {
            Record::parse(&bytes)?.successful()
        };
        self.change(
            &format!(
                "actions/runs/{}/attempts/{}/jobs?per_page=100&page=1",
                claims.run, claims.attempt
            ),
            |v| {
                v["jobs"][0]["status"] = json!("completed");
                v["jobs"][0]["conclusion"] = json!(if success { "success" } else { "failure" });
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
