#![cfg(unix)]
//! Continue actual public ON1 outputs through ON2 and both IN1 checkpoints.
use super::*;
use clap::Parser;

fn args(write: bool) -> crate::runtime_follow_on::Args {
    crate::runtime_follow_on::Args::parse_from([
        "runtime-follow-on",
        if write { "--write" } else { "--dry-run" },
        "--approval",
        APPROVAL,
        "--run-id",
        "public-on2",
    ])
}

#[test]
fn public_on2_connected_freeze_write_admission_and_observation() {
    let f = public_on1::declared_chain();
    let _installed = f.install();
    let parent = head(&f);
    crate::runtime_follow_on::run_in_workspace(f.root.path(), args(false), &mut Vec::new())
        .unwrap();
    integrate(&f, &parent);
    let record: Value =
        serde_json::from_slice(&fs::read(f.root.path().join(RECORD)).unwrap()).unwrap();
    assert!(record["obligations"].is_object());
    assert!(record["execution"].is_null());
    assert_eq!(record["acceptance_entries"], json!([]));
    let object = life::load_lifecycle_state(f.root.path(), STATE)
        .unwrap()
        .support_depth;
    let record_bytes = fs::read(f.root.path().join(RECORD)).unwrap();
    let executor = executor("", 0);
    let mut write_args = args(true);
    write_args.codex_binary = Some(
        executor
            .path()
            .join("executor.sh")
            .to_string_lossy()
            .into_owned(),
    );
    let parent = head(&f);
    crate::runtime_follow_on::run_in_workspace(f.root.path(), write_args, &mut Vec::new()).unwrap();
    let state = life::load_lifecycle_state(f.root.path(), STATE).unwrap();
    assert_eq!(
        state.lifecycle_stage,
        life::LifecycleStage::RuntimeIntegrated
    );
    assert_eq!(state.support_depth, object);
    assert_eq!(fs::read(f.root.path().join(RECORD)).unwrap(), record_bytes);
    assert_eq!(
        state.active_runtime_evidence_run_id.as_deref(),
        Some("public-on2")
    );
    let run = f
        .root
        .path()
        .join("docs/agents/.uaa-temp/runtime-follow-on/runs/public-on2");
    let written = fs::read(run.join("written-paths.json")).unwrap();
    assert_eq!(
        state.written_paths_sha256.as_deref(),
        Some(&hex::encode(Sha256::digest(&written)))
    );
    let paths: Vec<String> = serde_json::from_slice(&written).unwrap();
    assert!(paths.iter().any(|p| p.ends_with("/src/lib.rs")));
    assert!(paths.iter().any(|p| p.contains("/tests/")));
    let surfaces = serde_json::to_value(state.implementation_summary.unwrap()).unwrap();
    assert!(!surfaces["landed_surfaces"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v == "backend_harness" || v == "agent_api_onboarding_test"));
    let implementation = candidate(&f, &parent);
    let original = fs::read(f.root.path().join(STATE)).unwrap();
    // An unchanged record cannot conceal a deleted or retargeted lifecycle P/O object.
    for remove in [false, true] {
        let mut forged: Value = serde_json::from_slice(&original).unwrap();
        if remove {
            forged.as_object_mut().unwrap().remove("support_depth");
        } else {
            forged["support_depth"]["policy_identity"] = json!("a".repeat(64));
        }
        write(
            f.root.path(),
            STATE,
            serde_json::to_vec_pretty(&forged).unwrap(),
        );
        candidate(&f, &parent);
        assert!(check_integration(f.root.path(), &parent).is_err());
        git(f.root.path(), &["reset", "--hard", &implementation]);
    }
    check_integration(f.root.path(), &parent).unwrap();
    f.host();
    local(&f, &implementation);
    check_integration(f.root.path(), &parent).unwrap();
    assert_eq!(fs::read(f.root.path().join(RECORD)).unwrap(), record_bytes);
    let before =
        crate::candidate_execution::tree::capture(f.root.path(), &Default::default()).unwrap();
    let error =
        crate::runtime_follow_on::run_in_workspace(f.root.path(), args(false), &mut Vec::new())
            .unwrap_err();
    assert!(error.to_string().contains("runtime_integrated"), "{error}");
    assert_eq!(
        crate::candidate_execution::tree::capture(f.root.path(), &Default::default()).unwrap(),
        before
    );
}

/// The subprocess installs the real wrapper implementation and its substantive offline tests.
/// No preseeded implementation outputs, fake backend or synthesized depth E are involved.
#[cfg(unix)]
pub(in crate::isolated_maintenance::tests) fn executor(
    extra: &str,
    exit: i32,
) -> tempfile::TempDir {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let handoff = json!({"agent_id":"depth_fixture", "manifest_root":ROOT,
        "runtime_lane_complete":true, "publication_refresh_required":true,
        "required_commands":life::REQUIRED_PUBLICATION_COMMANDS,"blockers":[]});
    // Read the destination from the actual frozen prompt, so an absolute caller path is caught.
    let script = format!("#!/bin/sh\nset -eu\ntarget=$(sed -n 's/^- Update `\\([^`]*\\)` so it passes validation with:/\\1/p')\ntest -n \"$target\"\nmkdir -p crates/depth_fixture\ncp -R '{repo}/crates/depth_fixture/src' crates/depth_fixture/\ncp -R '{repo}/crates/depth_fixture/tests' crates/depth_fixture/\ncat > \"$target\" <<'JSON'\n{handoff}\nJSON\n{extra}\nexit {exit}\n", repo=repo.display());
    let path = dir.path().join("executor.sh");
    fs::write(&path, script).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    dir
}

#[test]
fn public_on2_refuses_before_declaration_and_in1_refuses_hand_assembled_freeze() {
    let f = setup(false);
    public_on1::canonical_context(&f);
    let _installed = f.install();
    let parent = head(&f);
    crate::onboard_agent::run_in_workspace(
        f.root.path(),
        public_on1::approval_args(),
        &mut Vec::new(),
    )
    .unwrap();
    integrate(&f, &parent);
    let before = fs::read(f.root.path().join(STATE)).unwrap();
    let error =
        crate::runtime_follow_on::run_in_workspace(f.root.path(), args(false), &mut Vec::new())
            .unwrap_err();
    assert!(error.to_string().contains("rule 3 unresolved"), "{error}");
    assert_eq!(fs::read(f.root.path().join(STATE)).unwrap(), before);
    assert!(!f.root.path().join(RECORD).exists());
    drop(_installed);

    let f = public_on1::declared_chain();
    let _installed = f.install();
    let parent = head(&f);
    // Hand assemble a structurally valid P-only statement/object, bypassing ON2 despite a
    // complete acquired surface. Independent IN1 must derive O rather than trust these hashes.
    let raw = fs::read(f.root.path().join(APPROVAL)).unwrap();
    let entry = crate::isolated_maintenance::registry(f.root.path())
        .unwrap()
        .find("depth_fixture")
        .unwrap()
        .clone();
    let approval = onboarding::approval::read(&raw, APPROVAL, &entry).unwrap();
    let mut record = crate::support_depth::FreezePlan::onboarding(
        &mut GitInputs::new(f.root.path()).unwrap(),
        &approval,
        None,
    )
    .unwrap()
    .record;
    record.obligations = None;
    let mut state = life::load_lifecycle_state(f.root.path(), STATE).unwrap();
    state.support_depth = Some(
        life::Preserved::new(life::LifecycleDepth {
            policy_identity: record.policy.identity.clone(),
            obligations_identity: None,
        })
        .unwrap(),
    );
    write(
        f.root.path(),
        RECORD,
        serde_json::to_vec_pretty(&record).unwrap(),
    );
    write(
        f.root.path(),
        STATE,
        serde_json::to_vec_pretty(&state).unwrap(),
    );
    candidate(&f, &parent);
    let error = check_integration(f.root.path(), &parent).unwrap_err();
    assert!(error.contains("independently derived P/O"), "{error}");
    assert_eq!(
        git(f.root.path(), &["rev-parse", "refs/remotes/origin/staging"]),
        parent
    );
}

#[test]
fn public_on2_in1_requires_parent_acquisition_and_intact_binding() {
    let f = public_on1::declared_chain();
    let _installed = f.install();
    let parent = head(&f);
    crate::runtime_follow_on::run_in_workspace(f.root.path(), args(false), &mut Vec::new())
        .unwrap();
    let outputs: Vec<_> = [
        RECORD,
        STATE,
        crate::support_matrix::JSON_OUTPUT_PATH,
        crate::support_matrix::MARKDOWN_OUTPUT_PATH,
        crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
    ]
    .into_iter()
    .map(|p| (p, fs::read(f.root.path().join(p)).unwrap()))
    .collect();
    // Reuse genuine ON2 output against two invalid parents: candidate-only acquisition
    // provenance, then an orphan object. Neither candidate may heal the missing parent binding.
    let statement = crate::depth_record::acquisition::statement_path(PACK, "0.2.0");
    let statement_bytes = fs::read(f.root.path().join(&statement)).unwrap();
    for orphan in [false, true] {
        git(f.root.path(), &["reset", "--hard", &parent]);
        // The freeze record was untracked; reset only restores tracked files.
        if f.root.path().join(RECORD).exists() {
            fs::remove_file(f.root.path().join(RECORD)).unwrap();
        }
        if orphan {
            write(
                f.root.path(),
                STATE,
                &outputs.iter().find(|(p, _)| *p == STATE).unwrap().1,
            );
        } else {
            fs::remove_file(f.root.path().join(&statement)).unwrap();
        }
        f.commit();
        let parent = head(&f);
        local(&f, &parent);
        for (path, bytes) in &outputs {
            write(f.root.path(), path, bytes);
        }
        write(f.root.path(), &statement, &statement_bytes);
        candidate(&f, &parent);
        let error = check_integration(f.root.path(), &parent).unwrap_err();
        let expected = if orphan {
            "deleted binding"
        } else {
            "acquisition statement missing at integration ref"
        };
        assert!(error.contains(expected), "{error}");
        assert_eq!(
            git(f.root.path(), &["rev-parse", "refs/remotes/origin/staging"]),
            parent
        );
    }
}
