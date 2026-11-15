//! Validator applicability survives removal of every candidate prerequisite.
use super::*;

#[test]
fn readonly_gates_validator_refuses_committed_prerequisite_deletion() {
    let (f, host) = setup();
    let root = f.root.path();
    let _route = f.install();
    let _reader = host.install();
    super::super::ma5_closeout::prepare(root).unwrap();
    super::super::ma5_closeout::advance(&f, &host);
    for name in ["RULES.json", "SCHEMA.json", "VERSION_METADATA_SCHEMA.json"] {
        fs::remove_file(root.join("cli_manifests/codex").join(name)).unwrap();
    }
    super::super::ma5_closeout::advance(&f, &host);
    let before = super::super::ma5_closeout::snapshot(root);
    let error = gates::with_committed_evaluation(root, || gates::check_shared(root)).unwrap_err();
    assert!(
        error.contains("cli_manifests/codex: I/O error: No such file or directory"),
        "{error}"
    );
    assert_eq!(super::super::ma5_closeout::snapshot(root), before);
}
