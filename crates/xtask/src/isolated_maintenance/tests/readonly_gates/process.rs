//! A test-owned child starts with no installed profile, then installs its own fake inputs.
use super::*;
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

const CHILD: &str = "isolated_maintenance::tests::readonly_gates::process::independent_process";

#[derive(Serialize, Deserialize)]
struct Observations {
    root: PathBuf,
    baseline: String,
    route: BTreeMap<String, Value>,
    admissions: BTreeMap<String, Vec<u8>>,
    evidence: BTreeMap<String, Vec<u8>>,
}

pub(super) fn evaluate(f: &Fixture, host: &SyntheticHost) {
    let observations = Observations {
        root: f.root.path().to_path_buf(),
        baseline: f.baseline.clone(),
        route: f.api.0.lock().unwrap().clone(),
        admissions: f.api.3.facts.lock().unwrap().clone(),
        evidence: host.snapshot(),
    };
    // Execute this Linux test image's inode even if a concurrent Cargo build replaces its
    // pathname. The child must run this exact harness and initialize its own fake authority.
    let mut child = Command::new("/proc/self/exe")
        .args(["--exact", CHILD, "--ignored", "--nocapture"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&observations).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("independent read-only gate checks passed"));
}

#[test]
#[ignore = "invoked by readonly_gates_two_phases with test-owned fake host observations on stdin"]
fn independent_process() {
    let observations: Observations = serde_json::from_reader(std::io::stdin()).unwrap();
    let root = &observations.root;
    let before = super::super::ma5_closeout::snapshot(root);
    // The parent's installation and evaluation guards have no authority in this process.
    assert!(route(root).unwrap().is_none());
    assert_eq!(
        registry(root),
        Err(RegistryError::Production(PRODUCTION_REFUSAL.into()))
    );
    assert_eq!(crate::support_matrix::check_in_workspace(root).unwrap_err(),
        "load agent registry: agent `depth_fixture` carries `support_depth`, which no command enforces yet");
    assert!(EvidenceVerdicts::take_publication_evaluations().is_empty());

    let api = Arc::new(Fake::default());
    *api.0.lock().unwrap() = observations.route;
    *api.3.facts.lock().unwrap() = observations.admissions;
    let _route = super::super::fixture::install(Box::new(move || {
        (
            super::super::fixture::profile_with(observations.baseline.clone(), &scope::SYNTHETIC),
            api.clone() as Arc<dyn Api>,
        )
    }));
    let host = SyntheticHost::from_snapshot(observations.evidence);
    let _reader = host.install();
    std::env::set_current_dir(root).unwrap();
    // These are the actual command entry points, with the fixed command arguments.
    crate::support_matrix::run(crate::support_matrix::Args { check: true }).unwrap();
    crate::capability_matrix::run(crate::capability_matrix::Args {
        check: true,
        out: None,
    })
    .unwrap();
    crate::capability_publication::check_in_workspace(root).unwrap();
    crate::runtime_advertising::run(crate::runtime_advertising::Args { check: true }).unwrap();
    gates::with_committed_evaluation(root, || gates::check_shared(root)).unwrap();
    assert!(!EvidenceVerdicts::take_publication_evaluations().is_empty());
    assert_eq!(
        crate::capability_matrix::run(crate::capability_matrix::Args {
            check: false,
            out: None
        })
        .unwrap_err(),
        "capability-matrix: unsupported depth write (declared enrollment)"
    );
    assert_eq!(super::super::ma5_closeout::snapshot(root), before);
    println!("independent read-only gate checks passed");
}
