//! Slice 3 uses only the installed fake route; no onboarding enrollment or production profile.
use super::*;
use crate::{agent_lifecycle as life, runtime_follow_on as runtime};
use sha2::{Digest, Sha256};
use std::os::unix::fs::PermissionsExt;

const APPROVAL: &str = "docs/agents/lifecycle/codex-cli-onboarding/governance/approved-agent.toml";
const STATE: &str = "docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json";
const RECORD: &str = "cli_manifests/codex/reports/0.156.1/depth-record.json";
const STATEMENT: &str =
    "docs/agents/lifecycle/codex-cli-onboarding/governance/depth-acquisition/0.156.1.toml";
const RUN: &str = "docs/agents/.uaa-temp/runtime-follow-on/runs/create-candidate";
const SOURCE: &str = "crates/codex/src/create_candidate.rs";
const TEST: &str = "crates/agent_api/tests/c1_codex_runtime_follow_on.rs";

fn repo() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn setup() -> Fixture {
    let f = Fixture::new();
    let root = f.root.path();
    let mut approval = fs::read_to_string(repo().join(APPROVAL)).unwrap();
    approval.push_str("\n[descriptor.maintenance]\nmode = \"explicitly_deferred\"\n[descriptor.maintenance.deferral]\nreason = \"fixture\"\nfollow_up = \"fixture\"\napproved_scope = \"create_lane_closeout\"\n[support_depth]\nversion = \"0.156.1\"\n");
    write(root, APPROVAL, &approval);
    write(
        root,
        "docs/agents/selection/cli-agent-selection-packet.md",
        "fixture\n",
    );
    write(root, ".gitignore", "docs/agents/.uaa-temp/\n");
    write(root, STATEMENT, "# protected acquisition statement\n");
    let mut record: crate::depth_record::DepthRecord = serde_json::from_str(include_str!(
        "../../../tests/fixtures/depth-core/record.json"
    ))
    .unwrap();
    // Scope remains maintenance-only in slice 3. Exercise the candidate boundary with the
    // installed fixture's exact maintenance selection; the create freeze is a later slice.
    let table: crate::agent_registry::support_depth::SupportDepth =
        serde_json::from_value(declaration()).unwrap();
    record.policy.selection.enrollment = table.enrollments.unwrap().remove(0);
    record.policy.identity = crate::depth_record::part_identity(&record.policy).unwrap();
    record.obligations = None;
    record.execution = None;
    record.acceptance_entries.clear();
    write(root, RECORD, serde_json::to_vec_pretty(&record).unwrap());
    crate::depth_record::DepthRecord::parse(
        &fs::read_to_string(root.join(RECORD)).unwrap(),
        "codex",
        "0.156.1",
    )
    .unwrap();
    let mut state: life::LifecycleState =
        serde_json::from_slice(&fs::read(repo().join(STATE)).unwrap()).unwrap();
    state.lifecycle_stage = life::LifecycleStage::Enrolled;
    state.support_tier = life::SupportTier::Bootstrap;
    state.approval_artifact_sha256 = hex::encode(Sha256::digest(approval.as_bytes()));
    state.active_runtime_evidence_run_id = None;
    state.required_evidence =
        life::required_evidence_for_stage(life::LifecycleStage::Enrolled).to_vec();
    state.satisfied_evidence = state.required_evidence.clone();
    state.publication_packet_path = None;
    state.publication_packet_sha256 = None;
    state.implementation_summary = None;
    state.support_depth = Some(
        life::Preserved::new(life::LifecycleDepth {
            policy_identity: record.policy.identity,
            obligations_identity: None,
        })
        .unwrap(),
    );
    state.validate().unwrap();
    write(root, STATE, serde_json::to_vec_pretty(&state).unwrap());
    f.commit();
    f
}
fn args(write: bool, binary: &str) -> runtime::Args {
    runtime::Args {
        approval: APPROVAL.into(),
        dry_run: !write,
        write,
        requested_tier: runtime::RequestedTier::Default,
        minimal_justification_file: None,
        allow_rich_surface: vec![],
        run_id: Some("create-candidate".into()),
        codex_binary: Some(binary.into()),
    }
}
fn executor(dir: &Path, script: &str) -> String {
    let binary = dir.join("executor.sh");
    fs::write(&binary, format!("#!/bin/sh\ncat >/dev/null\n{script}\n")).unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    binary.to_string_lossy().into_owned()
}
fn valid_output() -> String {
    let handoff = json!({"agent_id":"codex","manifest_root":"cli_manifests/codex", "runtime_lane_complete":true,
        "publication_refresh_required":true,"required_commands": life::REQUIRED_PUBLICATION_COMMANDS,"blockers":[]});
    format!("mkdir -p crates/codex/src crates/agent_api/tests\nprintf '// runtime output\\n' > {SOURCE}\nprintf '// runtime test\\n' > {TEST}\ncat > {RUN}/handoff.json <<'JSON'\n{handoff}\nJSON\necho host-stdout\necho host-stderr >&2")
}
fn run(f: &Fixture, binary: &str) -> Result<(), runtime::Error> {
    runtime::run_in_workspace(f.root.path(), args(true, binary), &mut vec![])
}
fn prepare(f: &Fixture, binary: &str) {
    runtime::run_in_workspace(f.root.path(), args(false, binary), &mut vec![]).unwrap();
}
fn json_file(root: &Path, path: &str) -> Value {
    serde_json::from_slice(&fs::read(root.join(path)).unwrap()).unwrap()
}

#[test]
fn depth_create_success_uses_controller_observation_and_binds_written_paths() {
    let f = setup();
    let _installed = f.install();
    let host = tempfile::tempdir().unwrap();
    let binary = executor(host.path(), &valid_output());
    prepare(&f, &binary);
    let contract = json_file(f.root.path(), &format!("{RUN}/input-contract.json"));
    assert!(!contract["allowed_write_paths"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p.as_str().unwrap().contains("/snapshots/")));
    run(&f, &binary).unwrap();
    let root = f.root.path();
    assert!(root.join(SOURCE).is_file());
    let state = json_file(root, STATE);
    let written = fs::read(root.join(format!("{RUN}/written-paths.json"))).unwrap();
    assert_eq!(state["lifecycle_stage"], "runtime_integrated");
    assert_eq!(
        state["written_paths_sha256"],
        hex::encode(Sha256::digest(&written))
    );
    let approval = crate::approval_artifact::load_approval_artifact(root, APPROVAL).unwrap();
    let validate = || {
        crate::prepare_publication::validate_runtime_evidence_run_for_approval(
            root,
            &approval,
            "create-candidate",
        )
    };
    validate().unwrap();
    // Even a formatting-only substitution changes the exact file the success state bound.
    let mut substituted = written.clone();
    substituted.push(b'\n');
    write(root, &format!("{RUN}/written-paths.json"), substituted);
    assert!(validate()
        .unwrap_err()
        .to_string()
        .contains("differs from lifecycle"));
    write(root, &format!("{RUN}/written-paths.json"), &written);
    let mut missing = state.clone();
    missing
        .as_object_mut()
        .unwrap()
        .remove("written_paths_sha256");
    write(root, STATE, serde_json::to_vec_pretty(&missing).unwrap());
    assert!(validate()
        .unwrap_err()
        .to_string()
        .contains("has no lifecycle written_paths_sha256"));
    write(root, STATE, serde_json::to_vec_pretty(&state).unwrap());
    // Publication-ready states clear the active run locator and preserve the success digest.
    let mut ready: life::LifecycleState = serde_json::from_value(state.clone()).unwrap();
    ready.lifecycle_stage = life::LifecycleStage::PublicationReady;
    ready.support_tier = life::SupportTier::BaselineRuntime;
    ready.active_runtime_evidence_run_id = None;
    ready.required_evidence = life::required_evidence_for_stage(ready.lifecycle_stage).to_vec();
    ready.satisfied_evidence = ready.required_evidence.clone();
    write(root, STATE, serde_json::to_vec_pretty(&ready).unwrap());
    validate().unwrap();
    write(root, STATE, serde_json::to_vec_pretty(&state).unwrap());
    let paths: Vec<String> = serde_json::from_slice(&written).unwrap();
    assert_eq!(paths, vec![TEST.to_owned(), SOURCE.to_owned()]);
    assert_eq!(
        json_file(root, &format!("{RUN}/validation-report.json"))["status"],
        "pass"
    );
    assert!(
        fs::read_to_string(root.join(format!("{RUN}/codex-stdout.log")))
            .unwrap()
            .contains("host-stdout")
    );
}

#[test]
fn depth_create_rejects_frozen_inputs_snapshots_and_controller_file_forgery() {
    for path in [
        format!("{RUN}/input-contract.json"),
        format!("{RUN}/codex-prompt.md"),
        STATE.into(),
        RECORD.into(),
        STATEMENT.into(),
        APPROVAL.into(),
        "cli_manifests/codex/snapshots/0.156.1/forged.json".into(),
        format!("{RUN}/written-paths.json"),
        format!("{RUN}/validation-report.json"),
        "docs/agents/.uaa-temp/runtime-follow-on/runs/other/handoff.json".into(),
        "forbidden.txt".into(),
    ] {
        let f = setup();
        let _installed = f.install();
        let host = tempfile::tempdir().unwrap();
        let script = format!(
            "{}\nmkdir -p '{}'\nprintf 'forged' > '{}'",
            valid_output(),
            Path::new(&path).parent().unwrap().display(),
            path
        );
        let binary = executor(host.path(), &script);
        prepare(&f, &binary);
        let before = fs::read(f.root.path().join(&path)).ok();
        let error = run(&f, &binary).unwrap_err().to_string();
        assert!(
            error.contains("protected")
                || error.contains("boundary")
                || error.contains("controller"),
            "{path}: {error}"
        );
        assert!(!f.root.path().join(SOURCE).exists(), "{path}");
        if path != STATE && !path.starts_with(RUN) {
            assert_eq!(fs::read(f.root.path().join(&path)).ok(), before, "{path}");
        }
        assert_eq!(
            json_file(f.root.path(), STATE)["lifecycle_stage"],
            "enrolled"
        );
        assert_eq!(
            json_file(f.root.path(), &format!("{RUN}/written-paths.json")),
            json!([])
        );
        assert_eq!(
            json_file(f.root.path(), &format!("{RUN}/validation-report.json"))["status"],
            "fail"
        );
    }
}

#[test]
fn depth_create_nonzero_signal_and_spawn_failure_leave_only_host_diagnostics() {
    for script in [
        format!("{}\nexit 7", valid_output()),
        format!("{}\nkill -TERM $$", valid_output()),
        "missing binary".into(),
    ] {
        let f = setup();
        let _installed = f.install();
        let host = tempfile::tempdir().unwrap();
        let binary = if script == "missing binary" {
            host.path().join("absent").display().to_string()
        } else {
            executor(host.path(), &script)
        };
        prepare(&f, &binary);
        assert!(run(&f, &binary).is_err());
        assert!(!f.root.path().join(SOURCE).exists());
        assert!(!f.root.path().join(TEST).exists());
        let state = json_file(f.root.path(), STATE);
        assert_eq!(state["lifecycle_stage"], "enrolled");
        assert!(state["written_paths_sha256"].is_null());
        assert_eq!(state["side_states"], json!(["failed_retryable"]));
        assert_eq!(
            json_file(f.root.path(), &format!("{RUN}/handoff.json"))["runtime_lane_complete"],
            false
        );
        assert_eq!(
            json_file(f.root.path(), &format!("{RUN}/written-paths.json")),
            json!([])
        );
    }
}

#[test]
fn depth_create_preserves_concurrent_caller_edits_and_refuses_production() {
    let f = setup();
    let host = tempfile::tempdir().unwrap();
    let script = format!(
        "{}\nprintf concurrent > '{}/docs/input.txt'",
        valid_output(),
        f.root.path().display()
    );
    let binary = executor(host.path(), &script);
    {
        let _installed = f.install();
        prepare(&f, &binary);
        assert!(run(&f, &binary)
            .unwrap_err()
            .to_string()
            .contains("uncommitted"));
    }
    assert_eq!(
        fs::read_to_string(f.root.path().join("docs/input.txt")).unwrap(),
        "concurrent"
    );
    assert!(!f.root.path().join(SOURCE).exists());
    assert!(run(&f, &binary)
        .unwrap_err()
        .to_string()
        .contains("unsupported depth write"));
}

#[test]
fn a_forged_saved_baseline_cannot_hide_a_forbidden_write() {
    let f = setup();
    let _installed = f.install();
    let host = tempfile::tempdir().unwrap();
    let binary = executor(
        host.path(),
        &format!("{}\nprintf forbidden > forbidden.txt", valid_output()),
    );
    prepare(&f, &binary);
    let contract_path = format!("{RUN}/input-contract.json");
    let mut contract = json_file(f.root.path(), &contract_path);
    contract["baseline"]["files"]
        .as_array_mut()
        .unwrap()
        .push(json!({"path":"forbidden.txt","sha256":hex::encode(Sha256::digest(b"forbidden"))}));
    write(
        f.root.path(),
        &contract_path,
        serde_json::to_vec_pretty(&contract).unwrap(),
    );
    let error = run(&f, &binary).unwrap_err().to_string();
    assert!(
        error.contains("write boundary violation: forbidden.txt"),
        "{error}"
    );
    assert!(!f.root.path().join("forbidden.txt").exists());
    assert!(!f.root.path().join(SOURCE).exists());
}

#[test]
fn depth_create_refuses_a_locked_index_and_preserves_a_concurrent_lifecycle_edit() {
    for lifecycle_edit in [false, true] {
        let f = setup();
        let _installed = f.install();
        let host = tempfile::tempdir().unwrap();
        let target = if lifecycle_edit {
            STATE
        } else {
            ".git/index.lock"
        };
        let binary = executor(
            host.path(),
            &format!(
                "{}\nprintf concurrent > '{}/{}'",
                valid_output(),
                f.root.path().display(),
                target
            ),
        );
        prepare(&f, &binary);
        let error = run(&f, &binary).unwrap_err().to_string();
        assert!(
            error.contains("divergent") || error.contains("index is locked"),
            "{error}"
        );
        assert_eq!(
            fs::read_to_string(f.root.path().join(target)).unwrap(),
            "concurrent"
        );
        assert!(!f.root.path().join(SOURCE).exists());
        assert_eq!(
            json_file(f.root.path(), &format!("{RUN}/written-paths.json")),
            json!([])
        );
        assert_eq!(
            json_file(f.root.path(), &format!("{RUN}/handoff.json"))["runtime_lane_complete"],
            false
        );
    }
}
