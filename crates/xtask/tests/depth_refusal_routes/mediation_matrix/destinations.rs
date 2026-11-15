//! NE2's explicit destination has its own scope, independent of the generating workspace.
use super::*;

#[test]
fn ne2_proposed_record_destinations_refuse_before_writing() {
    for destination in [RECORD.to_owned(), format!("nested/{RECORD}")] {
        let ws = Workspace::plain();
        ws.assert_refuses(
            &["capability-matrix", "--out", &destination],
            "capability-matrix: unsupported depth write (proposed depth record)",
        );
    }
}

#[test]
fn ne2_explicit_output_cannot_hide_a_nested_onboarding_workspace() {
    let ws = Workspace::plain();
    write(
        ws.root(),
        &format!("nested/{REGISTRY}"),
        include_str!("../../fixtures/depth-synthetic/onboarding/registry-entry-deferred.toml"),
    );
    ws.assert_refuses(
        &["capability-matrix", "--out", "nested/new/matrix.md"],
        "capability-matrix: unsupported depth write (declared enrollment)",
    );
    std::os::unix::fs::symlink("nested", ws.root().join("nested-alias")).unwrap();
    ws.assert_refuses(
        &["capability-matrix", "--out", "nested-alias/new/matrix.md"],
        "capability-matrix: unsupported depth write (declared enrollment)",
    );
    fs::create_dir_all(ws.root().join("nested/new")).unwrap();
    std::os::unix::fs::symlink("nested/new/matrix.md", ws.root().join("file-alias.md")).unwrap();
    ws.assert_refuses(
        &["capability-matrix", "--out", "file-alias.md"],
        "capability-matrix: unsupported depth write (declared enrollment)",
    );
}

#[test]
fn ne2_registry_output_refuses_before_replacing_bindings() {
    for destination in [REGISTRY.to_owned(), format!("nested/{REGISTRY}")] {
        let ws = Workspace::plain();
        ws.assert_refuses(
            &["capability-matrix", "--out", &destination],
            "parse agent registry TOML:",
        );
    }
}

#[test]
fn ne2_ordinary_explicit_output_keeps_caller_path_resolution() {
    let ws = Workspace::plain();
    fs::create_dir_all(ws.root().join("relative-output")).unwrap();
    fs::create_dir_all(ws.root().join("alias-target")).unwrap();
    std::os::unix::fs::symlink("alias-target", ws.root().join("linked-output")).unwrap();
    std::os::unix::fs::symlink(
        "ordinary-output/alias.md",
        ws.root().join("ordinary-alias.md"),
    )
    .unwrap();
    let absolute = ws.root().join("absolute-output/matrix.md");
    for destination in [
        "relative-output/../ordinary-output/matrix.md",
        "linked-output/nested/matrix.md",
        "ordinary-alias.md",
        absolute.to_str().unwrap(),
    ] {
        let output = ws.xtask(&["capability-matrix", "--out", destination]);
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{error}");
        let actual = fs::read_to_string(ws.root().join(destination)).unwrap();
        assert!(actual.contains("# Capability matrix"));
    }
}
