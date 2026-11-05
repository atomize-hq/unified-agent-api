use super::*;

fn git(root: &Path, args: &[&str]) -> String {
    String::from_utf8(candidate_git::run(root, args).unwrap()).unwrap()
}
fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    fs::write(dir.path().join("tracked"), b"tracked").unwrap();
    git(dir.path(), &["add", "tracked"]);
    dir
}
fn bound(path: &str, bytes: &[u8]) -> Bound {
    Bound {
        path: path.into(),
        mode: "100644",
        bytes: bytes.to_vec(),
    }
}

#[test]
fn publish_indexes_the_validated_bytes_never_the_live_file() {
    let root = repo();
    let path = "evidence/receipt.json";
    fs::create_dir_all(root.path().join("evidence")).unwrap();
    // The live file was swapped after validation; only the validated bytes gain an identity.
    fs::write(root.path().join(path), b"swapped").unwrap();
    publish(root.path(), &[bound(path, b"validated")]).unwrap();
    assert_eq!(
        git(root.path(), &["show", &format!(":{path}")]),
        "validated"
    );
    // A path the index already holds is never overwritten.
    let error = publish(root.path(), &[bound(path, b"other")]).unwrap_err();
    assert!(error.to_string().contains("already holds"), "{error}");
    assert_eq!(
        git(root.path(), &["show", &format!(":{path}")]),
        "validated"
    );
}

#[test]
fn a_locked_index_fails_publication_and_leaves_the_index_unchanged() {
    let root = repo();
    let index = fs::read(root.path().join(".git/index")).unwrap();
    fs::write(root.path().join(".git/index.lock"), b"").unwrap();
    assert!(publish(root.path(), &[bound("evidence/r.json", b"r")]).is_err());
    fs::remove_file(root.path().join(".git/index.lock")).unwrap();
    assert_eq!(fs::read(root.path().join(".git/index")).unwrap(), index);
}

#[test]
fn unpublish_removes_only_owned_entries_and_reports_conflicts() {
    let root = repo();
    let before = git(root.path(), &["ls-files", "--stage"]);
    let published = publish(
        root.path(),
        &[
            bound("evidence/a.json", b"a"),
            bound("evidence/b.json", b"b"),
        ],
    )
    .unwrap();
    unpublish(root.path(), &published).unwrap();
    assert_eq!(git(root.path(), &["ls-files", "--stage"]), before);
    // A concurrent caller change to one owned entry is preserved and reported.
    let published = publish(
        root.path(),
        &[
            bound("evidence/a.json", b"a"),
            bound("evidence/b.json", b"b"),
        ],
    )
    .unwrap();
    let oid = hash_object(root.path(), b"concurrent").unwrap();
    git(
        root.path(),
        &[
            "update-index",
            "--cacheinfo",
            &format!("100644,{oid},evidence/b.json"),
        ],
    );
    let error = unpublish(root.path(), &published).unwrap_err();
    assert!(error.to_string().contains("evidence/b.json"), "{error}");
    let entries = git(root.path(), &["ls-files", "--stage"]);
    assert!(!entries.contains("evidence/a.json"), "{entries}");
    assert!(
        entries.contains(&format!("{oid} 0\tevidence/b.json")),
        "{entries}"
    );
}
