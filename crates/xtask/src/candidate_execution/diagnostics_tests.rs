use super::*;
use crate::candidate_execution::{finalized_tree, TestHolder};
use std::fs;

#[test]
fn successful_diagnostics_retain_only_delivered_paths_in_json_and_text() {
    let caller = tempfile::Builder::new()
        .prefix("caller with spaces ")
        .tempdir()
        .unwrap();
    fs::write(caller.path().join("source file.txt"), "source").unwrap();
    let candidate = Candidate::prepare(caller.path(), "run", vec!["withheld.txt".into()]).unwrap();
    let _holder = TestHolder::new(&candidate.root);
    let mut observed = candidate.before.clone();
    for path in ["generated.txt", "withheld.txt"] {
        observed.insert(path.into(), Node::File(b"generated".to_vec(), 0o644));
    }
    let paths = [
        candidate.root.join("source file.txt"),
        candidate.root.join("generated.txt"),
        candidate.root.join("run/log.txt"),
        candidate.root.clone(),
        candidate.root.join("source file.txt.missing"),
        candidate.root.join("withheld.txt"),
        candidate.root.join(".git/HEAD"),
        candidate.root.parent().unwrap().join("scratch"),
    ];
    let log = paths
        .iter()
        .map(|path| format!("`{}`\n", path.display()))
        .collect::<String>();
    let diagnostics = Tree::from([
        (
            "run/log.txt".into(),
            Node::File(log.clone().into_bytes(), 0o644),
        ),
        (
            "run/report.json".into(),
            Node::File(
                serde_json::to_vec(&serde_json::json!({
                    "paths": paths, "nested": {"message": log}
                }))
                .unwrap(),
                0o644,
            ),
        ),
    ]);
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
    let holder = candidate.root.parent().unwrap().to_path_buf();
    candidate.discard();
    assert!(!holder.exists());
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(caller.path().join("run/report.json")).unwrap()).unwrap();
    for value in report["paths"].as_array().unwrap().iter().take(4) {
        assert!(Path::new(value.as_str().unwrap()).exists(), "{value}");
    }
    for value in report["paths"].as_array().unwrap().iter().skip(4) {
        assert_eq!(value, "[not retained]");
    }
    let log = fs::read_to_string(caller.path().join("run/log.txt")).unwrap();
    assert!(!log.contains(holder.to_str().unwrap()));
    assert_eq!(report["nested"]["message"], log);
    assert_eq!(log.matches("[not retained]").count(), 4);
    assert!(!caller.path().join("withheld.txt").exists());
    assert!(caller.path().join("generated.txt").is_file());
}
