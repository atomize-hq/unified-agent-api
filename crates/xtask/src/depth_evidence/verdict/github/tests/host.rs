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
    /// Serve this host to the calling thread's isolated readers until the guard drops.
    pub(crate) fn install(&self) -> transport::test_reader::Installed {
        let fake = self.0.clone();
        transport::test_reader::install(Arc::new(move |endpoint, limit| {
            fake.lock().unwrap().bytes(endpoint, limit)
        }))
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
