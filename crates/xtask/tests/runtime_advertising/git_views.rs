use super::*;
use fixtures::*;

#[test]
fn git_view_is_immutable_and_bad_or_pre_bridge_objects_never_become_empty_sets() {
    let temp = fixture();
    git(temp.path(), &["init", "-q"]);
    let tip = commit(temp.path());
    let before = Revision::tip(temp.path(), &tip).unwrap();
    edit(
        temp.path(),
        "crates/agent_api/Cargo.toml",
        "default = []",
        "default = [\"codex\"]",
    );
    assert!(Revision::workspace(temp.path())
        .unwrap()
        .additions(&before)
        .unwrap()
        .iter()
        .any(|a| a.requested == 0));
    assert!(Revision::tip(temp.path(), &tip)
        .unwrap()
        .additions(&before)
        .unwrap()
        .is_empty());
    for bad in ["--help", "missing-ref", "HEAD:Cargo.toml"] {
        assert!(Revision::tip(temp.path(), bad).is_err());
    }
    git(
        temp.path(),
        &[
            "rm",
            "-q",
            "crates/agent_api/src/runtime_advertising/data.rs",
        ],
    );
    let missing = commit(temp.path());
    assert!(Revision::tip(temp.path(), &missing)
        .err()
        .unwrap()
        .contains("missing advertising input"));
}

#[cfg(unix)]
#[test]
fn symlink_substitution_is_refused_in_working_and_git_inputs_including_optional_ancestors() {
    use std::os::unix::fs::symlink;
    let temp = fixture();
    let original = temp.path().join(xtask::runtime_advertising::DATA);
    std::fs::rename(&original, temp.path().join("saved.rs")).unwrap();
    symlink(temp.path().join("saved.rs"), &original).unwrap();
    assert!(Revision::workspace(temp.path()).is_err());
    git(temp.path(), &["init", "-q"]);
    let tip = commit(temp.path());
    assert!(Revision::tip(temp.path(), &tip).is_err());

    let temp = fixture();
    std::fs::create_dir(temp.path().join("settings")).unwrap();
    std::fs::write(
        temp.path().join("settings/config.toml"),
        "[build]\nrustflags=['--cfg','feature=\"codex\"']\n",
    )
    .unwrap();
    symlink("settings", temp.path().join(".cargo")).unwrap();
    git(temp.path(), &["init", "-q"]);
    let tip = commit(temp.path());
    assert!(
        Revision::tip(temp.path(), &tip).is_err(),
        "optional config ancestor must not vanish from Git view"
    );
}
