//! Missing E.7 execution negatives use the existing small declared ON2 refusal fixture.
//! This is not a substitute for the new-entry connected positive chain.
use super::super::bootstrap::public_on2::executor;
use super::*;
use crate::candidate_execution::tree;

#[test]
fn e7_forged_serialized_baseline_cannot_hide_a_snapshot_edit() {
    let f = setup(true);
    let _route = f.install();
    runtime::run_in_workspace(f.root.path(), args(), &mut Vec::new()).unwrap();
    f.commit();
    let snapshot = format!("{ROOT}/snapshots/0.2.0/union.json");
    let mut changed = fs::read(f.root.path().join(&snapshot)).unwrap();
    changed.push(b'\n');
    let mut contract = json_file(&f, &format!("{RUN}/input-contract.json"));
    let files = contract["baseline"]["files"].as_array_mut().unwrap();
    assert!(!files.is_empty());
    let row = files.iter_mut().find(|r| r["path"] == snapshot).unwrap();
    row["sha256"] = json!(hex::encode(Sha256::digest(changed)));
    write(
        f.root.path(),
        &format!("{RUN}/input-contract.json"),
        serde_json::to_vec_pretty(&contract).unwrap(),
    );
    let script = format!("printf '\\n' >> {snapshot}");
    refused_attempt(&f, &script, "depth write boundary forbids snapshots: cli_manifests/depth_fixture/snapshots/0.2.0/union.json");
}

#[test]
fn e7_interrupted_offline_executor_retains_only_trusted_failure_diagnostics() {
    let f = setup(true);
    let _route = f.install();
    runtime::run_in_workspace(f.root.path(), args(), &mut Vec::new()).unwrap();
    f.commit();
    refused_attempt(&f, "kill -TERM $$", "codex exec failed with exit code 1");
}

fn refused_attempt(f: &Fixture, script: &str, reason: &str) {
    let before = tree::capture(f.root.path(), &Default::default()).unwrap();
    let executor = executor(script, 0);
    let mut write_args = args();
    write_args.write = true;
    write_args.dry_run = false;
    write_args.codex_binary = Some(executor.path().join("executor.sh").to_string_lossy().into());
    let error = runtime::run_in_workspace(f.root.path(), write_args, &mut Vec::new())
        .unwrap_err()
        .to_string();
    assert!(error.contains(reason), "{error}");
    let report = json_file(f, &format!("{RUN}/validation-report.json"));
    assert_eq!(report["status"], "fail");
    let errors = report["errors"].as_array().unwrap();
    assert!(!errors.is_empty());
    assert!(
        errors.iter().any(|v| v.as_str().unwrap().contains(reason)),
        "{errors:?}"
    );
    assert_eq!(
        json_file(f, &format!("{RUN}/written-paths.json")),
        json!([])
    );
    let state = life::load_lifecycle_state(f.root.path(), STATE).unwrap();
    assert_eq!(state.lifecycle_stage, life::LifecycleStage::Enrolled);
    assert!(state
        .side_states
        .contains(&life::SideState::FailedRetryable));
    assert!(state.active_runtime_evidence_run_id.is_none());
    assert!(state.written_paths_sha256.is_none());
    let mut after = tree::capture(f.root.path(), &Default::default()).unwrap();
    let mut before = before;
    // ON2's documented exception is limited to its trusted attempt and diagnostics.
    for captured in [&mut before, &mut after] {
        captured.retain(|p, _| p != STATE && !p.starts_with(&format!("{RUN}/")));
    }
    assert_eq!(after, before);
}
