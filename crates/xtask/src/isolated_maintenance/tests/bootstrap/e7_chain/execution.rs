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
    refused_attempt(&f, &script, "depth write boundary forbids snapshots: cli_manifests/depth_fixture/snapshots/0.2.0/union.json", 0);
}

#[test]
fn e7_interrupted_offline_executor_retains_only_trusted_failure_diagnostics() {
    let f = setup(true);
    let _route = f.install();
    runtime::run_in_workspace(f.root.path(), args(), &mut Vec::new()).unwrap();
    f.commit();
    refused_attempt(&f, "kill -TERM $$", "codex exec failed with exit code 1", 1);
}

fn refused_attempt(f: &Fixture, script: &str, reason: &str, exit_code: i32) {
    let before = tree::capture(f.root.path(), &Default::default()).unwrap();
    let original_state = json_file(f, STATE);
    let original_bindings: std::collections::BTreeMap<String, Box<serde_json::value::RawValue>> =
        serde_json::from_slice(&fs::read(f.root.path().join(STATE)).unwrap()).unwrap();
    let immutable_run_files = ["input-contract.json", "codex-prompt.md", "handoff.json"];
    for name in immutable_run_files {
        assert!(matches!(
            before.get(&format!("{RUN}/{name}")),
            Some(tree::Node::File(..))
        ));
    }
    assert_eq!(
        json_file(f, &format!("{RUN}/handoff.json"))["runtime_lane_complete"],
        false
    );
    let executor = executor(
        &format!("printf 'offline stdout\\n'\nprintf 'offline stderr\\n' >&2\n{script}"),
        0,
    );
    let mut write_args = args();
    write_args.write = true;
    write_args.dry_run = false;
    write_args.codex_binary = Some(executor.path().join("executor.sh").to_string_lossy().into());
    let started = time::OffsetDateTime::now_utc();
    let error = runtime::run_in_workspace(f.root.path(), write_args, &mut Vec::new())
        .unwrap_err()
        .to_string();
    let finished = time::OffsetDateTime::now_utc();
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
    let state = json_file(f, STATE);
    let timestamp = time::OffsetDateTime::parse(
        state["last_transition_at"].as_str().unwrap(),
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap();
    // The controller records seconds. Bound the trusted attempt clock to this invocation.
    assert!(timestamp.unix_timestamp() >= started.unix_timestamp());
    assert!(timestamp.unix_timestamp() <= finished.unix_timestamp());
    let mut expected_state = original_state;
    // These are the complete permitted lifecycle attempt keys. All other keys stay equal.
    expected_state["current_owner_command"] = json!("runtime-follow-on --write");
    expected_state["last_transition_by"] = json!("xtask runtime-follow-on --write");
    expected_state["last_transition_at"] = state["last_transition_at"].clone();
    expected_state["active_runtime_evidence_run_id"] = Value::Null;
    expected_state["side_states"] = json!(["failed_retryable"]);
    expected_state["blocking_issues"] = json!([]);
    expected_state["retryable_failures"] =
        json!(["write attempt freeze started; consult run diagnostics"]);
    assert_eq!(state, expected_state);
    let bindings: std::collections::BTreeMap<String, Box<serde_json::value::RawValue>> =
        serde_json::from_slice(&fs::read(f.root.path().join(STATE)).unwrap()).unwrap();
    assert!(original_bindings["support_depth"].get().starts_with('{'));
    assert_eq!(
        bindings["support_depth"].get().as_bytes(),
        original_bindings["support_depth"].get().as_bytes()
    );

    let status = json_file(f, &format!("{RUN}/run-status.json"));
    assert_eq!(status["mode"], "write");
    assert_eq!(status["status"], "write_failed");
    assert_eq!(status["validation_passed"], false);
    assert_eq!(status["handoff_ready"], false);
    assert_eq!(status["written_paths"], json!([]));
    assert_eq!(status["errors"], report["errors"]);
    let execution = json_file(f, &format!("{RUN}/codex-execution.json"));
    assert_eq!(execution["run_id"], "freeze");
    assert_eq!(execution["exit_code"], exit_code);
    for (name, bytes) in [
        ("codex-stdout.log", b"offline stdout\n".as_slice()),
        ("codex-stderr.log", b"offline stderr\n".as_slice()),
    ] {
        assert_eq!(fs::read(f.root.path().join(RUN).join(name)).unwrap(), bytes);
    }
    let summary = fs::read_to_string(f.root.path().join(RUN).join("run-summary.md")).unwrap();
    assert!(summary.contains("- status: `fail`"));
    assert!(summary.contains("## Written Paths\n- none detected"));
    assert!(summary.contains(reason));
    let recovery =
        fs::read_to_string(f.root.path().join(RUN).join("recovery-candidate.txt")).unwrap();
    let candidate = Path::new(recovery.trim());
    assert!(candidate.is_dir());
    assert!(candidate.join("crates/depth_fixture/src/lib.rs").is_file());
    let candidate_handoff: Value =
        serde_json::from_slice(&fs::read(candidate.join(RUN).join("handoff.json")).unwrap())
            .unwrap();
    assert_eq!(candidate_handoff["runtime_lane_complete"], true);

    let mut after = tree::capture(f.root.path(), &Default::default()).unwrap();
    let mut before = before;
    // Compare every node, byte and mode except the explicitly checked trusted outputs.
    let mutable_files = [
        "validation-report.json",
        "written-paths.json",
        "run-status.json",
        "run-summary.md",
        "recovery-candidate.txt",
        "codex-execution.json",
        "codex-stdout.log",
        "codex-stderr.log",
    ];
    for path in
        std::iter::once(STATE.to_string()).chain(mutable_files.map(|name| format!("{RUN}/{name}")))
    {
        let Some(tree::Node::File(_, mode)) = after.remove(&path) else {
            panic!("trusted output must be a regular file: {path}");
        };
        let expected_mode = match before.remove(&path) {
            Some(tree::Node::File(_, original_mode)) => original_mode,
            None => 0o644,
            other => panic!("unexpected original trusted output: {path}: {other:?}"),
        };
        assert_eq!(mode, expected_mode, "trusted output mode: {path}");
    }
    for name in immutable_run_files {
        let path = format!("{RUN}/{name}");
        assert_eq!(
            after.get(&path),
            before.get(&path),
            "immutable run file {name}"
        );
    }
    assert_eq!(after, before);
}
