use super::*;
use crate::candidate_execution::{finalized_tree, TestHolder};
use std::fs;

#[test]
fn successful_diagnostics_rebase_only_known_absolute_path_fields() {
    let caller = tempfile::Builder::new()
        .prefix("caller with spaces ")
        .tempdir()
        .unwrap();
    fs::write(caller.path().join("source file.txt"), "source").unwrap();
    let candidate = Candidate::prepare(caller.path(), "run", vec!["withheld.txt".into()]).unwrap();
    let _holder = TestHolder::new(&candidate.root);
    let holder = candidate.root.parent().unwrap();
    let relative = format!("crates/codex/src{}/note.txt", holder.display());
    let non_boundary = format!("{}D/candidate/source file.txt", holder.display());
    let prose = format!("executor mentioned {}", candidate.root.display());
    let execution = serde_json::json!({
        "prompt_path": candidate.root.join("source file.txt"),
        "stdout_path": candidate.root.join("run/codex-stdout.log"),
        "stderr_path": candidate.root.join("missing.txt"),
        "binary": candidate.root.join("withheld.txt"),
        "argv": ["--cd", candidate.root, "--literal", candidate.root, "--other", non_boundary],
        "unknown": candidate.root,
    });
    let report = serde_json::json!({
        "codex_execution": execution,
        "preflight": {"binary": candidate.root.join("source file.txt"),
            "argv": ["--cd", candidate.root], "stdout": prose, "stderr": non_boundary},
        "checks": [{"message": prose}],
        "written_paths": [relative],
    });
    let mut diagnostics = Tree::new();
    for (name, value) in [
        ("codex-execution.json", execution),
        ("validation-report.json", report.clone()),
        (
            "run-status.json",
            serde_json::json!({"run_dir": candidate.root.join("run"), "written_paths": [relative]}),
        ),
        ("written-paths.json", serde_json::json!([relative])),
        ("other.json", report.clone()),
    ] {
        diagnostics.insert(
            format!("run/{name}"),
            Node::File(serde_json::to_vec_pretty(&value).unwrap(), 0o644),
        );
    }
    diagnostics.insert(
        "run/codex-stdout.log".into(),
        Node::File(prose.clone().into_bytes(), 0o644),
    );
    diagnostics.insert(
        "run/codex-stderr.log".into(),
        Node::File(non_boundary.clone().into_bytes(), 0o644),
    );
    let mut observed = candidate.before.clone();
    for path in [&relative, "withheld.txt"] {
        observed.insert(path.into(), Node::File(b"generated".to_vec(), 0o644));
    }
    let delivery = finalized_tree(
        &candidate,
        caller.path(),
        Some(&observed),
        true,
        diagnostics,
    )
    .unwrap();
    candidate
        .deliver(caller.path(), &delivery, true, None)
        .unwrap();
    let holder = holder.to_owned();
    candidate.discard();
    assert!(!holder.exists());
    let load = |name: &str| -> serde_json::Value {
        serde_json::from_slice(&fs::read(caller.path().join("run").join(name)).unwrap()).unwrap()
    };
    let execution = load("codex-execution.json");
    for field in ["prompt_path", "stdout_path"] {
        assert!(Path::new(execution[field].as_str().unwrap()).is_file());
    }
    for field in ["stderr_path", "binary"] {
        assert_eq!(execution[field], "[not retained]");
    }
    assert_eq!(execution["argv"][1], caller.path().to_str().unwrap());
    assert_eq!(
        execution["argv"][3],
        holder.join("candidate").to_str().unwrap()
    );
    assert_eq!(execution["argv"][5], non_boundary);
    assert_eq!(
        execution["unknown"],
        holder.join("candidate").to_str().unwrap()
    );
    let delivered_report = load("validation-report.json");
    assert_eq!(delivered_report["codex_execution"], execution);
    assert!(Path::new(delivered_report["preflight"]["binary"].as_str().unwrap()).is_file());
    assert_eq!(
        delivered_report["preflight"]["argv"][1],
        caller.path().to_str().unwrap()
    );
    assert_eq!(delivered_report["preflight"]["stdout"], prose);
    assert_eq!(delivered_report["preflight"]["stderr"], non_boundary);
    assert_eq!(delivered_report["checks"], report["checks"]);
    assert_eq!(
        delivered_report["written_paths"],
        serde_json::json!([relative])
    );
    assert_eq!(load("written-paths.json"), serde_json::json!([relative]));
    assert_eq!(load("other.json"), report);
    assert_eq!(
        load("run-status.json")["run_dir"],
        caller.path().join("run").to_str().unwrap()
    );
    assert_eq!(
        fs::read_to_string(caller.path().join("run/codex-stdout.log")).unwrap(),
        prose
    );
    assert_eq!(
        fs::read_to_string(caller.path().join("run/codex-stderr.log")).unwrap(),
        non_boundary
    );
    assert!(caller.path().join(relative).is_file());
    assert!(!caller.path().join("withheld.txt").exists());
}

#[test]
fn candidate_absolute_paths_require_an_exact_component_boundary() {
    let holder = "/tmp/uaa-relay-ABC";
    let mappings = vec![(format!("{holder}/candidate"), "/caller".into())];
    let mut value = serde_json::json!({
        "prompt_path": "/tmp/uaa-relay-ABCD/candidate/prompt",
        "stdout_path": "/tmp/uaa-relay-ABC",
        "stderr_path": "/tmp/uaa-relay-ABC/candidate/missing",
        "binary": "crates/codex/src/tmp/uaa-relay-ABC/note.txt",
        "argv": ["--cd", "/tmp/uaa-relay-ABC/candidate"],
    });
    rebase_execution(&mut value, holder, &mappings);
    assert_eq!(value["prompt_path"], "/tmp/uaa-relay-ABCD/candidate/prompt");
    assert_eq!(value["stdout_path"], "[not retained]");
    assert_eq!(value["stderr_path"], "[not retained]");
    assert_eq!(
        value["binary"],
        "crates/codex/src/tmp/uaa-relay-ABC/note.txt"
    );
    assert_eq!(value["argv"][1], "/caller");
}

#[test]
fn candidate_absolute_paths_are_compared_without_normalization() {
    let holder = "/tmp/uaa-relay-ABC";
    let mappings = vec![(format!("{holder}/candidate/file"), "/caller/file".into())];
    for path in [
        "/tmp/uaa-relay-ABC/candidate//file",
        "/tmp/uaa-relay-ABC/candidate/./file",
        "/tmp/uaa-relay-ABC/candidate/../candidate/file",
    ] {
        let mut value = serde_json::json!(path);
        rebase_path(&mut value, holder, &mappings);
        assert_eq!(value, "[not retained]");
    }
    for path in [
        "/tmp/uaa-relay-ABCD/candidate/file",
        "crates/codex/src/tmp/uaa-relay-ABC/note.txt",
        "message mentions /tmp/uaa-relay-ABC/candidate/file",
    ] {
        let mut value = serde_json::json!(path);
        rebase_path(&mut value, holder, &mappings);
        assert_eq!(value, path);
    }
}
