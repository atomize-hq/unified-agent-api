//! Actual ON1/ON2 output continues through public repair and IN1.
#![cfg(unix)]
use super::*;
use clap::Parser;

fn repair(f: &Fixture, write: bool) -> Result<Vec<u8>, crate::repair_runtime_evidence::Error> {
    let args = crate::repair_runtime_evidence::Args::parse_from([
        "repair-runtime-evidence",
        if write { "--write" } else { "--check" },
        "--approval",
        APPROVAL,
    ]);
    let mut output = Vec::new();
    crate::repair_runtime_evidence::run_in_workspace(f.root.path(), args, &mut output)?;
    Ok(output)
}

#[test]
fn public_repair_refuses_forged_success_when_protected_generation_is_enrolled() {
    let started = std::time::Instant::now();
    let f = public_on1::declared_chain();
    let _installed = f.install();
    let args = |write| {
        crate::runtime_follow_on::Args::parse_from([
            "runtime-follow-on",
            if write { "--write" } else { "--dry-run" },
            "--approval",
            APPROVAL,
            "--run-id",
            "public-on2-unprotected-success",
        ])
    };
    let parent = head(&f);
    crate::runtime_follow_on::run_in_workspace(f.root.path(), args(false), &mut Vec::new())
        .unwrap();
    integrate(&f, &parent);
    let protected = head(&f);
    assert_eq!(
        life::load_lifecycle_state(f.root.path(), STATE)
            .unwrap()
            .lifecycle_stage,
        life::LifecycleStage::Enrolled
    );
    let executor = public_on2::executor("", 0);
    let mut implementation = args(true);
    implementation.codex_binary =
        Some(executor.path().join("executor.sh").to_string_lossy().into());
    crate::runtime_follow_on::run_in_workspace(f.root.path(), implementation, &mut Vec::new())
        .unwrap();
    let implemented = candidate(&f, &protected);
    assert_ne!(implemented, protected);
    let mut state = life::load_lifecycle_state(f.root.path(), STATE).unwrap();
    assert_eq!(
        state.lifecycle_stage,
        life::LifecycleStage::RuntimeIntegrated
    );
    let runs = "docs/agents/.uaa-temp/runtime-follow-on/runs";
    let contract = fs::read(
        f.root
            .path()
            .join(runs)
            .join(state.active_runtime_evidence_run_id.as_ref().unwrap())
            .join("input-contract.json"),
    )
    .unwrap();
    let forged_run = "forged-success";
    let mut contract: Value = serde_json::from_slice(&contract).unwrap();
    contract["run_id"] = json!(forged_run);
    write(
        f.root.path(),
        &format!("{runs}/{forged_run}/input-contract.json"),
        serde_json::to_vec_pretty(&contract).unwrap(),
    );
    let inventory = serde_json::to_vec(&json!(["Cargo.toml"])).unwrap();
    write(
        f.root.path(),
        &format!("{runs}/{forged_run}/written-paths.json"),
        &inventory,
    );
    // Preserve the raw P/O object bytes while forging the selector and matching digest.
    state.active_runtime_evidence_run_id = Some(forged_run.into());
    state.written_paths_sha256 =
        Some(life::Preserved::new(hex::encode(Sha256::digest(&inventory))).unwrap());
    write(
        f.root.path(),
        STATE,
        serde_json::to_vec_pretty(&state).unwrap(),
    );
    candidate(&f, &protected);
    assert_eq!(
        git(f.root.path(), &["rev-parse", "refs/remotes/origin/staging"]),
        protected
    );
    let before =
        crate::candidate_execution::tree::capture(f.root.path(), &Default::default()).unwrap();
    for write in [false, true] {
        let error = repair(&f, write).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("protected runtime_integrated generation"),
            "{error}"
        );
        assert_eq!(
            crate::candidate_execution::tree::capture(f.root.path(), &Default::default()).unwrap(),
            before
        );
    }
    eprintln!(
        "public repair unprotected success refusal elapsed {:?}",
        started.elapsed()
    );
}

#[test]
fn public_repair_sibling_chain_preserves_frozen_generation_and_written_paths() {
    let started = std::time::Instant::now();
    let f = public_on1::declared_chain();
    let _installed = f.install();
    let args = |write| {
        crate::runtime_follow_on::Args::parse_from([
            "runtime-follow-on",
            if write { "--write" } else { "--dry-run" },
            "--approval",
            APPROVAL,
            "--run-id",
            "public-on2-repair-sibling",
        ])
    };
    assert_refused(&f);
    let parent = head(&f);
    crate::runtime_follow_on::run_in_workspace(f.root.path(), args(false), &mut Vec::new())
        .unwrap();
    integrate(&f, &parent);
    assert_refused(&f);
    let executor = public_on2::executor("", 0);
    let mut implementation = args(true);
    implementation.codex_binary =
        Some(executor.path().join("executor.sh").to_string_lossy().into());
    let parent = head(&f);
    crate::runtime_follow_on::run_in_workspace(f.root.path(), implementation, &mut Vec::new())
        .unwrap();
    integrate(&f, &parent);
    let state = life::load_lifecycle_state(f.root.path(), STATE).unwrap();
    let record = fs::read(f.root.path().join(RECORD)).unwrap();
    let parsed: Value = serde_json::from_slice(&record).unwrap();
    assert!(parsed["execution"].is_null());
    assert_eq!(parsed["acceptance_entries"], json!([]));
    let run = format!(
        "docs/agents/.uaa-temp/runtime-follow-on/runs/{}",
        state.active_runtime_evidence_run_id.as_ref().unwrap()
    );
    let written = fs::read(f.root.path().join(&run).join("written-paths.json")).unwrap();
    refusal_matrix(&f, &run);
    // Break only the repairable reporting, retaining the lifecycle-authenticated inventory.
    fs::remove_file(f.root.path().join(&run).join("handoff.json")).unwrap();
    let parent = head(&f);
    integrate(&f, &parent);
    let parent = head(&f);
    assert!(String::from_utf8(repair(&f, false).unwrap())
        .unwrap()
        .contains("check passed"));
    delivery_failure(&f);
    let output = String::from_utf8(repair(&f, true).unwrap()).unwrap();
    assert!(output.contains("write complete"));
    let repaired = life::load_lifecycle_state(f.root.path(), STATE).unwrap();
    assert_eq!(repaired.lifecycle_stage, state.lifecycle_stage);
    assert_eq!(repaired.support_depth, state.support_depth);
    assert_eq!(repaired.written_paths_sha256, state.written_paths_sha256);
    assert_eq!(fs::read(f.root.path().join(RECORD)).unwrap(), record);
    let run = format!(
        "docs/agents/.uaa-temp/runtime-follow-on/runs/{}",
        repaired.active_runtime_evidence_run_id.as_ref().unwrap()
    );
    assert_eq!(
        fs::read(f.root.path().join(&run).join("written-paths.json")).unwrap(),
        written
    );
    integrate(&f, &parent);
    let captured = crate::isolated_maintenance::route(f.root.path())
        .unwrap()
        .unwrap()
        .capture(f.root.path())
        .unwrap();
    let approval = crate::runtime_follow_on::depth::approval::execution(
        f.root.path(),
        &fs::read(f.root.path().join(APPROVAL)).unwrap(),
        APPROVAL,
        &captured,
    )
    .unwrap();
    crate::prepare_publication::validate_runtime_evidence_run_for_approval(
        f.root.path(),
        &approval,
        repaired.active_runtime_evidence_run_id.as_deref().unwrap(),
    )
    .unwrap();
    assert!(output.contains(repaired.active_runtime_evidence_run_id.as_deref().unwrap()));
    delivery_failure(&f);
    // The production call still refuses the same depth workspace after the guard drops.
    drop(_installed);
    assert_refused(&f);
    eprintln!(
        "public repair sibling chain elapsed {:?}",
        started.elapsed()
    );
}

fn assert_refused(f: &Fixture) {
    let previous = head(f);
    let dirty = !git(f.root.path(), &["status", "--porcelain=v1"]).is_empty();
    if dirty {
        let tip = git(f.root.path(), &["rev-parse", "refs/remotes/origin/staging"]);
        candidate(f, &tip);
    }
    let before =
        crate::candidate_execution::tree::capture(f.root.path(), &Default::default()).unwrap();
    let error = repair(f, true).unwrap_err();
    assert!(
        !error.to_string().contains("uncommitted or untracked"),
        "{error}"
    );
    assert_eq!(
        crate::candidate_execution::tree::capture(f.root.path(), &Default::default()).unwrap(),
        before
    );
    if dirty {
        git(f.root.path(), &["reset", "--mixed", &previous]);
    }
}

fn replace_json(f: &Fixture, path: &str, edit: impl FnOnce(&mut Value)) -> Vec<u8> {
    let original = fs::read(f.root.path().join(path)).unwrap();
    let mut value: Value = serde_json::from_slice(&original).unwrap();
    edit(&mut value);
    write(
        f.root.path(),
        path,
        serde_json::to_vec_pretty(&value).unwrap(),
    );
    original
}

fn refusal_matrix(f: &Fixture, run: &str) {
    let started = std::time::Instant::now();
    for field in [
        "active_runtime_evidence_run_id",
        "written_paths_sha256",
        "support_depth",
    ] {
        let original = replace_json(f, STATE, |v| match field {
            "support_depth" => v[field]["policy_identity"] = json!("a".repeat(64)),
            "written_paths_sha256" => v[field] = json!("a".repeat(64)),
            _ => v[field] = json!("stale-run"),
        });
        assert_refused(f);
        write(f.root.path(), STATE, original);
    }
    let original = replace_json(f, STATE, |v| {
        v.as_object_mut().unwrap().remove("support_depth");
    });
    assert_refused(f);
    write(f.root.path(), STATE, original);
    let original = fs::read(f.root.path().join(RECORD)).unwrap();
    fs::remove_file(f.root.path().join(RECORD)).unwrap();
    assert_refused(f);
    write(f.root.path(), RECORD, original);
    let contract = format!("{run}/input-contract.json");
    for field in ["run_id", "support_depth", "approval_artifact_sha256"] {
        let original = replace_json(f, &contract, |v| {
            if field == "support_depth" {
                v[field]["obligations_identity"] = json!("a".repeat(64));
            } else {
                v[field] = json!("stale");
            }
        });
        assert_refused(f);
        write(f.root.path(), &contract, original);
    }
    let paths = format!("{run}/written-paths.json");
    let original = fs::read(f.root.path().join(&paths)).unwrap();
    let mut changed = original.clone();
    changed.push(b'\n');
    write(f.root.path(), &paths, changed);
    assert_refused(f);
    write(f.root.path(), &paths, original);
    let coverage = format!("{ROOT}/wrapper_coverage.json");
    let original = fs::read(f.root.path().join(&coverage)).unwrap();
    let mut changed = original.clone();
    changed.push(b'\n');
    write(f.root.path(), &coverage, changed);
    assert_refused(f);
    write(f.root.path(), &coverage, original);
    // A current lifecycle rollback is also refused against the later protected generation.
    for stage in [
        "enrolled",
        "publication_ready",
        "published",
        "closed_baseline",
    ] {
        let original = replace_json(f, STATE, |v| v["lifecycle_stage"] = json!(stage));
        assert_refused(f);
        write(f.root.path(), STATE, original);
    }
    for mode in 0..10 {
        let _hook = StageHook::install(mode);
        assert_refused(f);
    }
    eprintln!(
        "public repair refusal matrix elapsed {:?}",
        started.elapsed()
    );
}

thread_local! { static STAGE_EDIT: std::cell::Cell<u8> = const { std::cell::Cell::new(0) }; }
struct StageHook;
impl StageHook {
    fn install(edit: u8) -> Self {
        STAGE_EDIT.with(|slot| slot.set(edit));
        crate::repair_runtime_evidence::set_test_stage_mutator(Some(alter_proposal));
        Self
    }
}
impl Drop for StageHook {
    fn drop(&mut self) {
        crate::repair_runtime_evidence::set_test_stage_mutator(None);
    }
}
fn alter_proposal(run: &Path) -> Result<(), String> {
    let root = run.ancestors().nth(6).unwrap();
    let edit = STAGE_EDIT.with(|slot| slot.get());
    let (path, field) = match edit {
        0 => (root.join(STATE), "support_depth"),
        1 => (root.join(RECORD), "policy"),
        2 => (root.join(RECORD), "obligations"),
        3 => (root.join(RECORD), "execution"),
        4 => (root.join(RECORD), "acceptance_entries"),
        5 => (
            root.join(format!("{ROOT}/wrapper_coverage.json")),
            "schema_version",
        ),
        6 => {
            write(root, &format!("{ROOT}/latest_validated.txt"), "0.2.0\n");
            return Ok(());
        }
        7 => (run.join("written-paths.json"), ""),
        8 => (run.join("input-contract.json"), "support_depth"),
        _ => (root.join(STATE), "lifecycle_stage"),
    };
    let mut v: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    if edit == 7 {
        v = json!(["crates/forged/src/lib.rs"]);
    } else {
        v[field] = json!("forged");
    }
    fs::write(path, serde_json::to_vec_pretty(&v).unwrap()).map_err(|e| e.to_string())
}

struct DeliveryHook;
impl Drop for DeliveryHook {
    fn drop(&mut self) {
        crate::repair_runtime_evidence::routed::DELIVERY_FAILURE.with(|slot| slot.set(None));
    }
}
fn delivery_failure(f: &Fixture) {
    let before =
        crate::candidate_execution::tree::capture(f.root.path(), &Default::default()).unwrap();
    crate::repair_runtime_evidence::routed::DELIVERY_FAILURE.with(|slot| slot.set(Some(1)));
    let _hook = DeliveryHook;
    let error = repair(f, true).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("injected repair delivery failure"),
        "{error}"
    );
    assert_eq!(
        crate::candidate_execution::tree::capture(f.root.path(), &Default::default()).unwrap(),
        before
    );
}
