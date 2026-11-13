//! Public ON2 failures retain only trusted attempt state and truthful diagnostics.
#[cfg(unix)]
use super::super::bootstrap::public_on2::executor;
use super::*;

#[cfg(unix)]
fn public_freeze(f: &Fixture) {
    runtime::run_in_workspace(f.root.path(), args(), &mut Vec::new()).unwrap();
    f.commit();
}

#[test]
fn runtime_follow_on_public_installed_errors_never_fall_back() {
    let f = setup(true);
    edit_approval(&f, |d| d["descriptor"]["unknown"] = toml_edit::value(true));
    f.commit();
    let _installed = f.install();
    let before =
        crate::candidate_execution::tree::capture(f.root.path(), &Default::default()).unwrap();
    let error = runtime::run_in_workspace(f.root.path(), args(), &mut Vec::new()).unwrap_err();
    assert!(
        error.to_string().contains("private approval reader"),
        "{error}"
    );
    assert_eq!(
        crate::candidate_execution::tree::capture(f.root.path(), &Default::default()).unwrap(),
        before
    );
}

#[cfg(unix)]
#[test]
fn runtime_follow_on_public_failed_candidates_preserve_record_inputs_and_truthful_diagnostics() {
    for (extra, exit, message) in [
        ("", 7, "exit code 7"),
        (
            "printf '\\n' >> cli_manifests/depth_fixture/reports/0.2.0/depth-record.json",
            0,
            "protected",
        ),
        (
            "printf '\\n' >> cli_manifests/depth_fixture/snapshots/0.2.0/union.json",
            0,
            "snapshots",
        ),
        (
            "printf '\\n' >> cli_manifests/depth_fixture/wrapper_coverage.json",
            0,
            "write boundary",
        ),
        (
            "printf '[\"forged\"]' > docs/agents/.uaa-temp/runtime-follow-on/runs/freeze/written-paths.json",
            0,
            "controller run file",
        ),
        (
            "rm -rf crates/depth_fixture/src crates/depth_fixture/tests",
            0,
            "no runtime-owned",
        ),
        (
            "rm -rf crates/depth_fixture/tests",
            0,
            "wrapper-only implementation",
        ),
        ("printf '{}' > \"$target\"", 0, "handoff.json"),
    ] {
        let f = setup(true);
        let _installed = f.install();
        public_freeze(&f);
        let before =
            crate::candidate_execution::tree::capture(f.root.path(), &Default::default()).unwrap();
        let executor = executor(extra, exit);
        let mut write = args();
        write.write = true;
        write.dry_run = false;
        write.codex_binary = Some(
            executor
                .path()
                .join("executor.sh")
                .to_string_lossy()
                .into_owned(),
        );
        let error = runtime::run_in_workspace(f.root.path(), write, &mut Vec::new()).unwrap_err();
        assert!(error.to_string().contains(message), "{message}: {error}");
        let state = life::load_lifecycle_state(f.root.path(), STATE).unwrap();
        assert_eq!(state.lifecycle_stage, life::LifecycleStage::Enrolled);
        assert!(state
            .side_states
            .contains(&life::SideState::FailedRetryable));
        assert!(state.active_runtime_evidence_run_id.is_none());
        assert!(state.written_paths_sha256.is_none());
        let report = json_file(&f, &format!("{RUN}/validation-report.json"));
        assert_eq!(report["status"], "fail");
        assert!(report["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().contains(message)));
        assert_eq!(
            json_file(&f, &format!("{RUN}/written-paths.json")),
            json!([])
        );
        let mut after =
            crate::candidate_execution::tree::capture(f.root.path(), &Default::default()).unwrap();
        let mut before = before;
        for tree in [&mut before, &mut after] {
            tree.retain(|p, _| p != STATE && !p.starts_with(&format!("{RUN}/")));
        }
        assert_eq!(after, before, "failed executor output leaked: {extra}");
    }
}

#[cfg(unix)]
#[test]
fn runtime_follow_on_public_forged_prepared_packet_refuses_before_attempt() {
    let f = setup(true);
    let _installed = f.install();
    public_freeze(&f);
    let contract_path = format!("{RUN}/input-contract.json");
    let original = fs::read(f.root.path().join(&contract_path)).unwrap();
    for edit in 0..3 {
        write(f.root.path(), &contract_path, &original);
        edit_json(&f, &contract_path, |v| match edit {
            0 => v["support_depth"]["policy_identity"] = json!("a".repeat(64)),
            1 => v["allowed_write_paths"] = json!(["**"]),
            _ => v["backend_module"] = json!("crates/agent_api/src/backends/fake"),
        });
        let before = fs::read(f.root.path().join(STATE)).unwrap();
        let mut write = args();
        write.write = true;
        write.dry_run = false;
        assert!(runtime::run_in_workspace(f.root.path(), write, &mut Vec::new()).is_err());
        assert_eq!(fs::read(f.root.path().join(STATE)).unwrap(), before);
    }
}

#[cfg(unix)]
#[test]
fn runtime_follow_on_public_delivery_preserves_concurrent_caller_and_reports_refusal() {
    let f = setup(true);
    let _installed = f.install();
    public_freeze(&f);
    let record = fs::read(f.root.path().join(RECORD)).unwrap();
    let extra = format!(
        "printf 'concurrent owner' > '{}/docs/input.txt'",
        f.root.path().display()
    );
    let executor = executor(&extra, 0);
    let mut write = args();
    write.write = true;
    write.dry_run = false;
    write.codex_binary = Some(
        executor
            .path()
            .join("executor.sh")
            .to_string_lossy()
            .into_owned(),
    );
    let error = runtime::run_in_workspace(f.root.path(), write, &mut Vec::new()).unwrap_err();
    assert!(error.to_string().contains("candidate retained"), "{error}");
    assert_eq!(
        fs::read(f.root.path().join("docs/input.txt")).unwrap(),
        b"concurrent owner"
    );
    assert_eq!(fs::read(f.root.path().join(RECORD)).unwrap(), record);
    assert!(!f
        .root
        .path()
        .join("crates/depth_fixture/src/lib.rs")
        .exists());
    let state = life::load_lifecycle_state(f.root.path(), STATE).unwrap();
    assert_eq!(state.lifecycle_stage, life::LifecycleStage::Enrolled);
    assert!(state
        .side_states
        .contains(&life::SideState::FailedRetryable));
    assert!(state.active_runtime_evidence_run_id.is_none());
}
