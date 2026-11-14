//! Small raw-Git proofs use one repository and explicit object identities per test.
use super::*;
use std::{fs, io::Write, process::Stdio};

fn git(root: &Path, args: &[&str]) -> String {
    let output = crate::git_inputs::command(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
fn hash(root: &Path, kind: &str, bytes: &[u8]) -> String {
    let mut child = crate::git_inputs::command(root)
        .args(["hash-object", "-w", "-t", kind, "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().into()
}
fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "--quiet", "--template="]);
    git(root.path(), &["config", "user.name", "Fixture"]);
    git(
        root.path(),
        &["config", "user.email", "fixture@example.invalid"],
    );
    fs::write(root.path().join("Cargo.toml"), "[workspace]\n").unwrap();
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "--quiet", "-m", "predecessor"]);
    root
}
fn read(root: &Path, revision: &str) -> crate::freeze_transition::Tree {
    let mut reader = Objects::new(root);
    let object = reader.commit(revision).unwrap().object;
    objects::read(&mut reader, &object).unwrap()
}
#[test]
fn authenticated_tree_reader_compares_regular_executable_symlink_and_gitlink() {
    let root = repository();
    let parent = git(root.path(), &["rev-parse", "HEAD"]);
    let before = read(root.path(), &parent);
    let expected = crate::freeze_transition::operations(
        &before,
        &crate::freeze_transition::ValidatedAcquisitionDelta::empty(),
        [],
    )
    .unwrap();
    let blob = git(root.path(), &["rev-parse", "HEAD:Cargo.toml"]);
    for (mode, object) in [
        ("100755", blob.as_str()),
        ("120000", blob.as_str()),
        ("160000", parent.as_str()),
    ] {
        git(root.path(), &["read-tree", &parent]);
        git(
            root.path(),
            &[
                "update-index",
                "--cacheinfo",
                &format!("{mode},{object},Cargo.toml"),
            ],
        );
        let tree = git(root.path(), &["write-tree"]);
        let commit = git(
            root.path(),
            &["commit-tree", &tree, "-p", &parent, "-m", "kind change"],
        );
        let candidate = read(root.path(), &commit);
        assert_eq!(candidate["Cargo.toml"].mode, mode);
        assert!(expected.require_equal(&candidate).is_err());
    }
}
#[test]
fn authenticated_tree_reader_refuses_hidden_empty_tree_object() {
    let root = repository();
    let empty = hash(root.path(), "tree", &[]);
    let mut bytes = b"40000 hidden\0".to_vec();
    bytes.extend(hex::decode(empty).unwrap());
    let tree = hash(root.path(), "tree", &bytes);
    let commit = git(
        root.path(),
        &["commit-tree", &tree, "-m", "empty directory"],
    );
    let mut reader = Objects::new(root.path());
    let object = reader.commit(&commit).unwrap().object;
    let failure = objects::read(&mut reader, &object).unwrap_err();
    assert!(matches!(
        &failure,
        PlanningFailure::Ineligible(Reason::FreezePlanningRefused, _)
    ));
    assert!(failure.message().contains("empty directory"));
}

#[test]
fn transient_authenticated_blob_read_failure_is_incomplete_and_recovers() {
    let root = repository();
    let commit = git(root.path(), &["rev-parse", "HEAD"]);
    let expected = read(root.path(), &commit);
    let blob = git(root.path(), &["rev-parse", "HEAD:Cargo.toml"]);
    let blob_path = root
        .path()
        .join(".git/objects")
        .join(&blob[..2])
        .join(&blob[2..]);
    let saved = fs::read(&blob_path).unwrap();
    fs::remove_file(&blob_path).unwrap();
    let mut reader = Objects::new(root.path());
    let object = reader.commit(&commit).unwrap().object;
    assert!(matches!(
        objects::read(&mut reader, &object),
        Err(PlanningFailure::Incomplete(_))
    ));
    fs::write(blob_path, saved).unwrap();
    assert_eq!(objects::read(&mut reader, &object).unwrap(), expected);
}
