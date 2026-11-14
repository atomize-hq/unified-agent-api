//! Coherent runtime substitutions and unchanged-E checkpoint attacks through public ON3 and IN1.
use super::*;

const PACKET: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/publication-ready.json";
const RUNS: &str = crate::runtime_evidence_run::RUNTIME_RUNS_ROOT;
const CONTRACT_REFUSAL: &str = "input contract differs from frozen success state";
const PUBLIC_SELECTION_REFUSAL: &str = "ON3 runtime state differs from protected generation";
const IN1_SELECTION_REFUSAL: &str =
    "ON3 packet does not select its predecessor's active runtime evidence";

fn edit(root: &Path, path: &str, change: impl FnOnce(&mut Value)) {
    let mut value: Value = serde_json::from_slice(&fs::read(root.join(path)).unwrap()).unwrap();
    change(&mut value);
    write(root, path, serde_json::to_vec_pretty(&value).unwrap());
}

fn refuse_public(root: &Path, write: bool, expected: &str) {
    let before = crate::candidate_execution::tree::capture(root, &Default::default()).unwrap();
    let error = run(root, write).unwrap_err().to_string();
    assert!(error.contains(expected), "expected {expected}, got {error}");
    assert_eq!(
        crate::candidate_execution::tree::capture(root, &Default::default()).unwrap(),
        before
    );
}

fn selected(root: &Path) -> String {
    let packet = life::load_publication_ready_packet(root, PACKET).unwrap();
    crate::runtime_evidence_run::validate_runtime_evidence_paths_shape(
        &packet.runtime_evidence_paths,
    )
    .unwrap()
}

fn alternate(root: &Path, selected: &str) -> String {
    let alternate = if selected == "fr2-freeze" {
        "review-alternate"
    } else {
        "fr2-freeze"
    };
    let existing = root
        .join(format!("{RUNS}/{alternate}/input-contract.json"))
        .exists();
    // Retain older evidence when present. Repair's missing neutral handoff is restored from the
    // valid selected run. Every other file in the older bundle retains its original contents.
    for name in crate::runtime_evidence_run::REQUIRED_RUNTIME_EVIDENCE_FILES {
        let target = format!("{RUNS}/{alternate}/{name}");
        if !root.join(&target).exists() {
            write(
                root,
                &target,
                fs::read(root.join(format!("{RUNS}/{selected}/{name}"))).unwrap(),
            );
        }
    }
    if !existing {
        edit(
            root,
            &format!("{RUNS}/{alternate}/input-contract.json"),
            |v| {
                v["run_id"] = json!(alternate);
            },
        );
        edit(root, &format!("{RUNS}/{alternate}/run-status.json"), |v| {
            v["run_id"] = json!(alternate);
            v["run_dir"] = json!(format!("{RUNS}/{alternate}"));
        });
    }
    alternate.into()
}

fn prove_valid_run(root: &Path, run: &str) {
    let captured = crate::isolated_maintenance::route(root)
        .unwrap()
        .unwrap()
        .capture(root)
        .unwrap();
    let raw = fs::read(root.join(APPROVAL)).unwrap();
    let approval =
        crate::runtime_follow_on::depth::approval::execution(root, &raw, APPROVAL, &captured)
            .unwrap();
    let bundle = crate::prepare_publication::validate_runtime_evidence_run_for_approval(
        root, &approval, run,
    )
    .unwrap();
    assert_eq!(bundle.run_id, run);
}

fn contract_changes() -> [(&'static str, Value); 6] {
    [
        ("/support_depth/version", json!("0.3.0")),
        ("/support_depth/policy_identity", json!("a".repeat(64))),
        ("/support_depth/obligations_identity", json!("a".repeat(64))),
        ("/run_id", json!("another-valid-run")),
        ("/workflow_version", json!("runtime_follow_on_v1")),
        ("/support_depth", Value::Null),
    ]
}

pub(super) fn runtime_selection(
    f: &Fixture,
    root: &Path,
    api: &super::super::super::fixture::Fake,
) {
    let parent = git(root, &["rev-parse", "HEAD"]);
    let state = life::load_lifecycle_state(root, STATE).unwrap();
    let run = state.active_runtime_evidence_run_id.as_deref().unwrap();
    let status: Value = serde_json::from_slice(
        &fs::read(root.join(format!("{RUNS}/{run}/run-status.json"))).unwrap(),
    )
    .unwrap();
    // The immutable template is cloned into multiple sibling repositories. Its evidence must
    // remain portable without rewriting any committed runtime bundle in those clones.
    assert_eq!(status["run_dir"], format!("{RUNS}/{run}"));
    for (pointer, value) in contract_changes() {
        edit(root, &format!("{RUNS}/{run}/input-contract.json"), |v| {
            *v.pointer_mut(pointer).unwrap() = value;
        });
        commit(root, api, &f.baseline, &parent);
        for write in [false, true] {
            refuse_public(root, write, CONTRACT_REFUSAL);
        }
        git(root, &["reset", "--hard", &parent]);
    }
    let other = alternate(root, run);
    edit(root, STATE, |v| {
        v["active_runtime_evidence_run_id"] = json!(other)
    });
    commit(root, api, &f.baseline, &parent);
    prove_valid_run(root, &other);
    for write in [false, true] {
        refuse_public(root, write, PUBLIC_SELECTION_REFUSAL);
    }
    git(root, &["reset", "--hard", &parent]);
}

pub(super) fn reporting_candidate(
    f: &Fixture,
    root: &Path,
    api: &super::super::super::fixture::Fake,
    parent: &str,
    head: &str,
) {
    let run = selected(root);
    for (pointer, value) in contract_changes() {
        edit(root, &format!("{RUNS}/{run}/input-contract.json"), |v| {
            *v.pointer_mut(pointer).unwrap() = value;
        });
        git(root, &["reset", "--soft", parent]);
        commit(root, api, &f.baseline, parent);
        refuse_public(root, false, CONTRACT_REFUSAL);
        let error = check_integration(root, parent).unwrap_err();
        assert!(error.contains(CONTRACT_REFUSAL), "{error}");
        git(root, &["reset", "--hard", head]);
    }
    substitute_packet(root, &run);
    git(root, &["reset", "--soft", parent]);
    commit(root, api, &f.baseline, parent);
    prove_valid_run(root, &selected(root));
    refuse_public(root, false, PUBLIC_SELECTION_REFUSAL);
    let error = check_integration(root, parent).unwrap_err();
    assert!(error.contains(IN1_SELECTION_REFUSAL), "{error}");
    git(root, &["reset", "--hard", head]);
}

fn substitute_packet(root: &Path, run: &str) {
    let other = alternate(root, run);
    let paths = crate::runtime_evidence_run::runtime_evidence_paths_for_run(&other).unwrap();
    edit(root, PACKET, |v| v["runtime_evidence_paths"] = json!(paths));
}

pub(super) fn publication_checkpoint(
    f: &Fixture,
    root: &Path,
    api: &super::super::super::fixture::Fake,
    head: &str,
) {
    let record = fs::read(root.join(RECORD)).unwrap();
    for remove in [true, false] {
        if remove {
            fs::remove_file(root.join(PACKET)).unwrap();
        } else {
            edit(root, PACKET, |v| {
                v["blocking_issues"] = json!(["forged checkpoint"])
            });
        }
        commit(root, api, &f.baseline, head);
        assert_eq!(fs::read(root.join(RECORD)).unwrap(), record);
        let error = check_integration(root, head).unwrap_err();
        let expected = if remove {
            "read "
        } else {
            "is stale relative to lifecycle state and runtime evidence"
        };
        assert!(error.contains(expected), "{error}");
        assert!(error.contains("publication-ready.json"), "{error}");
        git(root, &["reset", "--hard", head]);
    }
    let run = selected(root);
    substitute_packet(root, &run);
    commit(root, api, &f.baseline, head);
    assert_eq!(fs::read(root.join(RECORD)).unwrap(), record);
    prove_valid_run(root, &selected(root));
    refuse_public(root, false, PUBLIC_SELECTION_REFUSAL);
    let error = check_integration(root, head).unwrap_err();
    assert!(error.contains(IN1_SELECTION_REFUSAL), "{error}");
    git(root, &["reset", "--hard", head]);
}
