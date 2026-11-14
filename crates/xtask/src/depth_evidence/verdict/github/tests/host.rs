//! A fake isolated host for the synthetic profile over an existing workspace, so the connected
//! MA4 chain (B.4 part 2) can authenticate a real population without any network call.
use super::*;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(crate) struct SyntheticHost(Arc<Mutex<Fake>>);

/// Commit the synthetic producer closure into `root` (keeping what it holds, including its frozen
/// record at the profile fixture path) and serve one faithful, adopted run of that commit.
pub(crate) fn synthetic_host(root: &Path) -> SyntheticHost {
    let p = Profile::Synthetic;
    let (w, commit) = build(root, p);
    SyntheticHost(Arc::new(Mutex::new(endpoints(root, p, &w, &commit, 1))))
}
impl SyntheticHost {
    /// Model one onboarding producer over the actual connected ON2 implementation.
    pub(crate) fn onboarding(root: &Path) -> Self {
        let p = Profile::SyntheticOnboarding;
        let (witness, commit) = build(root, p);
        Self(Arc::new(Mutex::new(endpoints(
            root, p, &witness, &commit, 1,
        ))))
    }

    /// Retain genuine collector reporting contexts alongside substantive execution attempts.
    pub(crate) fn include_reporting(
        &self,
        host: &crate::depth_acceptance::admission::fake::MemoryHost,
    ) {
        use crate::depth_acceptance::admission::Host;
        let mut fake = self.0.lock().unwrap();
        let query = "actions/workflows/ci.yml/runs?created=%3E%3D2026-11-05";
        let mut rows = fake
            .json(&fake.endpoint(&format!("{query}&per_page=100&page=1")))
            .unwrap()["workflow_runs"]
            .as_array()
            .unwrap()
            .clone();
        rows.extend(
            host.json("reporting/runs?per_page=100&page=1").unwrap()["workflow_runs"]
                .as_array()
                .unwrap()
                .clone(),
        );
        for (key, bytes) in host
            .facts
            .lock()
            .unwrap()
            .iter()
            .filter(|(key, _)| key.starts_with("actions/"))
        {
            let endpoint = fake.endpoint(key);
            fake.0.insert(endpoint, bytes.clone());
        }
        fake.page(query, Some("workflow_runs"), rows);
    }

    pub(crate) fn onboarding_population(
        &self,
        root: &Path,
        host: &crate::isolated_maintenance::EvidenceHost,
    ) -> (usize, usize) {
        let fake = self.0.lock().unwrap().clone();
        let resolved =
            resolve_profile_in(&fake, root, Profile::SyntheticOnboarding, Some(host)).unwrap();
        (resolved.reporting.len(), resolved.runs.len())
    }

    /// Real assembly from the separate maintenance population, retained as a cross-flow control.
    pub(crate) fn maintenance_receipt() -> Vec<u8> {
        static RECEIPT: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
        RECEIPT
            .get_or_init(|| {
                let f = Fixture::new_for(Profile::Synthetic);
                let host = crate::isolated_maintenance::EvidenceHost::for_test(ISOLATED, &f.commit);
                let resolved =
                    resolve_profile_in(&f.api, f.root.path(), Profile::Synthetic, Some(&host))
                        .unwrap();
                let assembled = assembly::assemble(&resolved).unwrap();
                let reference = &assembled.execution.results[0].evidence[0];
                assembled
                    .files
                    .iter()
                    .find(|(file, _)| file == reference)
                    .unwrap()
                    .1
                    .clone()
            })
            .clone()
    }

    /// Serve this host to the calling thread's isolated readers until the guard drops.
    pub(crate) fn install(&self) -> transport::test_reader::Installed {
        let fake = self.0.clone();
        transport::test_reader::install(Arc::new(move |endpoint, limit| {
            fake.lock().unwrap().bytes(endpoint, limit)
        }))
    }
    /// The genuine first attempt's hosted witness archive, for candidate-copy regressions.
    pub(crate) fn witness_archive(&self) -> Vec<u8> {
        let fake = self.0.lock().unwrap();
        fake.bytes(&fake.endpoint("actions/artifacts/101/zip"), 2_097_152)
            .unwrap()
    }
    /// An independent copy, so a test can change it without changing this host.
    pub(crate) fn fork(&self) -> Self {
        Self(Arc::new(Mutex::new(self.0.lock().unwrap().clone())))
    }
    /// Protected staging moved to `tip`.
    pub(crate) fn staging(&self, tip: &str) {
        self.0
            .lock()
            .unwrap()
            .put("git/ref/heads/staging", json!({"object":{"sha":tip}}));
    }
    /// The run gains a second attempt under `runs`, whose producer job `edit` describes.
    pub(crate) fn add_attempt(&self, runs: &str, edit: impl FnOnce(&mut Value)) {
        let mut fake = self.0.lock().unwrap();
        fake.change(runs, |v| v["workflow_runs"][0]["run_attempt"] = json!(2));
        let key = fake.endpoint("actions/runs/1/attempts/1");
        let mut attempt: Value = serde_json::from_slice(&fake.0[&key]).unwrap();
        attempt["run_attempt"] = json!(2);
        fake.put("actions/runs/1/attempts/2", attempt);
        let key = fake.endpoint("actions/runs/1/attempts/1/jobs?per_page=100&page=1");
        let mut jobs: Value = serde_json::from_slice(&fake.0[&key]).unwrap();
        jobs["jobs"][0]["id"] = json!(202);
        jobs["jobs"][0]["run_attempt"] = json!(2);
        edit(&mut jobs["jobs"][0]);
        fake.put("actions/runs/1/attempts/2/jobs?per_page=100&page=1", jobs);
    }
    /// The maintainer's adoption comment is gone from the packet PR.
    pub(crate) fn revoke_adoption(&self) {
        self.0
            .lock()
            .unwrap()
            .page("issues/9/comments", None, vec![]);
    }
}

impl SyntheticHost {
    /// Candidate-controlled ordinary comments are present in both evaluations.
    pub(crate) fn ordinary_comment(&self, body: &str) {
        let mut fake = self.0.lock().unwrap();
        fake.put(
            "collaborators/candidate/permission",
            json!({"permission":"read"}),
        );
        fake.change("issues/9/comments?per_page=100&page=1", |v| {
            let rows = v.as_array_mut().unwrap();
            rows.retain(|row| row["id"] != 999);
            rows.push(json!({"id":999,"body":body,"user":{"id":999,"login":"candidate"}}));
        });
    }
    /// Change ordering across multi-member populations and unrelated enclosing workflow state.
    pub(crate) fn incidental_population(&self, runs: &str, reverse: bool) {
        let mut fake = self.0.lock().unwrap();
        fake.change(runs, |v| {
            let rows = v["workflow_runs"].as_array_mut().unwrap();
            if rows.len() == 1 {
                let mut other = rows[0].clone();
                other["id"] = json!(900);
                other["head_repository"]["full_name"] = json!("unrelated/repository");
                rows.push(other);
            }
            for r in rows.iter_mut() {
                if r["id"] == 900 {
                    r["head_sha"] = json!(if reverse {
                        "ignored edited head"
                    } else {
                        "ignored original head"
                    });
                }
                r["status"] = json!(if reverse { "completed" } else { "in_progress" });
                r["conclusion"] = json!(if reverse { "success" } else { "failure" });
                r["updated_at"] = json!(if reverse { "today" } else { "yesterday" });
                r["display_title"] = json!(if reverse { "edited" } else { "original" });
            }
            if reverse {
                rows.reverse();
            }
            v["total_count"] = json!(rows.len());
        });
        for raw in fake.0.values_mut() {
            let Ok(mut value) = serde_json::from_slice::<Value>(raw) else {
                continue;
            };
            if let Some(object) = value.as_object_mut() {
                for key in ["jobs", "artifacts"] {
                    if let Some(rows) = object.get_mut(key).and_then(Value::as_array_mut) {
                        if rows.len() == 1 {
                            let mut other = rows[0].clone();
                            other["id"] = json!(900);
                            other["name"] = json!("unrelated producer");
                            rows.push(other);
                        }
                        if reverse {
                            rows.reverse();
                        }
                        let count = rows.len();
                        object.insert("total_count".into(), json!(count));
                    }
                }
            }
            if value["run_attempt"].is_number() {
                value["status"] = json!(if reverse { "completed" } else { "in_progress" });
                value["conclusion"] = json!(if reverse { "success" } else { "failure" });
                value["updated_at"] = json!(if reverse { "today" } else { "yesterday" });
            }
            *raw = serde_json::to_vec(&value).unwrap();
        }
    }
}
