use super::*;
use crate::git_inputs::{immutable::Immutable, listing::Listing};

#[test]
fn borrowed_single_reads_share_objects_and_recheck_working_files() {
    let caller = fixture();
    let scratch = tempfile::tempdir().unwrap();
    for i in 0..64 {
        let name = format!("file-{i}");
        let bytes = format!("bytes {i}\0\n");
        fs::write(caller.path().join(&name), &bytes).unwrap();
        fs::write(scratch.path().join(name), bytes).unwrap();
    }
    git(caller.path(), &["add", "."]).unwrap();
    let borrowed = batch::commit(caller.path());
    let cache = Immutable::new(&borrowed.0, &borrowed.1);
    GIT_COMMANDS.with(|count| count.set(0));
    for i in 0..64 {
        let path = format!("file-{i}");
        assert_eq!(
            read_cached(scratch.path(), &path, Some(&borrowed), Some(&cache.clone()))
                .unwrap()
                .bytes,
            Some(format!("bytes {i}\0\n").into_bytes())
        );
    }
    assert_eq!(GIT_COMMANDS.with(|count| count.get()), 3);
    fs::write(scratch.path().join("file-0"), "changed").unwrap();
    fs::remove_file(scratch.path().join("file-1")).unwrap();
    fs::remove_file(scratch.path().join("file-2")).unwrap();
    fs::create_dir(scratch.path().join("file-2")).unwrap();
    fs::write(scratch.path().join("untracked"), "untracked").unwrap();
    for path in [
        "file-0",
        "file-1",
        "file-2",
        "file-3",
        "untracked",
        "absent",
        "../bad",
    ] {
        assert_eq!(
            read_cached(scratch.path(), path, Some(&borrowed), Some(&cache)),
            read(scratch.path(), path, Some(&borrowed)),
            "{path}"
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{symlink, PermissionsExt};
        fs::set_permissions(
            scratch.path().join("file-3"),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        fs::remove_file(scratch.path().join("file-4")).unwrap();
        symlink("file-5", scratch.path().join("file-4")).unwrap();
        for path in ["file-3", "file-4"] {
            assert_eq!(
                read_cached(scratch.path(), path, Some(&borrowed), Some(&cache)),
                read(scratch.path(), path, Some(&borrowed))
            );
        }
    }
}

#[test]
fn nested_directory_listing_and_verification_use_two_git_snapshots() {
    let root = fixture();
    fs::create_dir(root.path().join("scope")).unwrap();
    for i in 0..64 {
        fs::create_dir_all(root.path().join(format!("scope/{i}/sub"))).unwrap();
    }
    let mut view = GitInputs::new(root.path()).unwrap();
    GIT_COMMANDS.with(|count| count.set(0));
    assert!(view.files_under("scope").unwrap().is_empty());
    assert_eq!(view.directories.len(), 129);
    assert_eq!(GIT_COMMANDS.with(|count| count.get()), 2);
    GIT_COMMANDS.with(|count| count.set(0));
    view.verify().unwrap();
    assert_eq!(GIT_COMMANDS.with(|count| count.get()), 2);
    fs::create_dir(root.path().join("scope/0/new")).unwrap();
    assert_eq!(
        view.verify(),
        Err("acquisition directory changed during freeze: scope/0".into())
    );
    equivalent_traversal(&GitInputs::new(root.path()).unwrap());
}

#[test]
fn cached_directory_names_and_partial_freeze_match_per_directory_commands() {
    let root = fixture();
    for path in [
        "scope/a",
        "scope/dir/b",
        "scope/dir/sub/c",
        "scope/space ü\n/d",
        "scope/:(glob)*/e",
        "unrelated/a",
    ] {
        let full = root.path().join(path);
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(full, path).unwrap();
    }
    git(root.path(), &["add", "."]).unwrap();
    // Include tracked children missing from the filesystem, and an untracked child.
    fs::remove_file(root.path().join("scope/dir/sub/c")).unwrap();
    fs::remove_dir(root.path().join("scope/dir/sub")).unwrap();
    fs::write(root.path().join("scope/dir/untracked"), "untracked").unwrap();
    let listing = Listing::default();
    for path in [
        "scope",
        "scope/dir",
        "scope/dir/sub",
        "scope/space ü\n",
        "scope/:(glob)*",
        "scope/a",
        "absent",
        "../bad",
    ] {
        assert_eq!(
            list_cached(root.path(), path, None, Some(&listing)),
            list(root.path(), path, None),
            "{path:?}"
        );
    }
    equivalent_traversal(&GitInputs::new(root.path()).unwrap());
    fs::remove_file(root.path().join("scope/dir/untracked")).unwrap();
    fs::create_dir_all(root.path().join("scope/dir/sub")).unwrap();
    fs::write(root.path().join("scope/dir/sub/c"), "scope/dir/sub/c").unwrap();
    let mut frozen = equivalent_traversal(&GitInputs::new(root.path()).unwrap());
    // Each new traversal/recheck must observe the current index, never a stale global listing.
    fs::write(root.path().join("scope/dir/new"), "new").unwrap();
    git(root.path(), &["add", "scope/dir/new"]).unwrap();
    fs::remove_file(root.path().join("scope/dir/new")).unwrap();
    assert_eq!(
        frozen.verify(),
        Err("acquisition directory changed during freeze: scope/dir".into())
    );
    assert!(frozen.files_under("scope").is_err());
    equivalent_traversal(&GitInputs::new(root.path()).unwrap());
}

#[cfg(unix)]
#[test]
fn cached_listing_keeps_unrelated_non_utf8_names_opaque() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
    let root = fixture();
    fs::create_dir(root.path().join("scope")).unwrap();
    fs::write(root.path().join("scope/a"), "a").unwrap();
    fs::write(root.path().join(OsStr::from_bytes(b"\xff")), "opaque").unwrap();
    git(root.path(), &["add", "."]).unwrap();
    equivalent_traversal(&GitInputs::new(root.path()).unwrap());
    fs::write(
        root.path().join("scope").join(OsStr::from_bytes(b"\xff")),
        "invalid name",
    )
    .unwrap();
    git(root.path(), &["add", "."]).unwrap();
    equivalent_traversal(&GitInputs::new(root.path()).unwrap());
    fs::remove_file(root.path().join("scope").join(OsStr::from_bytes(b"\xff"))).unwrap();
    equivalent_traversal(&GitInputs::new(root.path()).unwrap());
}

#[test]
fn malformed_borrowed_trees_and_non_blob_objects_match_single_reads() {
    let root = fixture();
    let original = batch::commit(root.path());
    let object =
        String::from_utf8(git(root.path(), &["rev-parse", "HEAD:bound"]).unwrap()).unwrap();
    let tree = String::from_utf8(git(root.path(), &["rev-parse", "HEAD^{tree}"]).unwrap()).unwrap();
    let missing = "1".repeat(40);
    fs::create_dir(root.path().join("scope")).unwrap();
    for path in ["scope/bound", "scope/z"] {
        fs::write(root.path().join(path), "initial").unwrap();
    }
    for entries in [
        vec![("bound", object.trim()), ("bound", object.trim())],
        vec![("z", object.trim()), ("bound", object.trim())],
        vec![("bound", tree.trim())],
        vec![("bound", original.1.as_str())],
        vec![("bound", missing.as_str())],
    ] {
        let mut raw = Vec::new();
        for (name, object) in entries {
            raw.extend_from_slice(format!("100644 {name}\0").as_bytes());
            raw.extend(hex::decode(object).unwrap());
        }
        let tree = literal_tree(root.path(), &raw);
        let borrowed = tree_commit(root.path(), &tree);
        let cache = Immutable::new(&borrowed.0, &borrowed.1);
        for path in ["bound", "absent", "bound/child", "../bad"] {
            assert_eq!(
                batch::read_many(root.path(), &[path], Some(&borrowed))[0],
                read(root.path(), path, Some(&borrowed)),
                "batch {path}"
            );
            assert_eq!(
                read_cached(root.path(), path, Some(&borrowed), Some(&cache)),
                read(root.path(), path, Some(&borrowed)),
                "{path}"
            );
        }

        // Exercise traversal's batched read against the old optional/required path, including
        // partial recording and error precedence. A captured proposal avoids requiring a live
        // isolated-registry binding for this reader-level malformed-object fixture.
        let mut raw = b"40000 scope\0".to_vec();
        raw.extend(hex::decode(&tree).unwrap());
        let borrowed = tree_commit(root.path(), &literal_tree(root.path(), &raw));
        let mut view = GitInputs::new(root.path()).unwrap();
        view.immutable = Some(Immutable::new(&borrowed.0, &borrowed.1));
        view.borrowed = Some(borrowed);
        view.proposed = Some(
            crate::runtime_advertising::source::Source::captured(
                root.path(),
                ["scope/bound", "scope/z"]
                    .into_iter()
                    .map(|path| (path.into(), (b"initial".to_vec(), false)))
                    .collect(),
            )
            .unwrap(),
        );
        equivalent_traversal(&view);
    }
}

fn literal_tree(root: &Path, raw: &[u8]) -> String {
    let mut child = command(root)
        .args(["hash-object", "-t", "tree", "--literally", "-w", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(raw).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().into()
}

fn tree_commit(root: &Path, tree: &str) -> (PathBuf, String) {
    let commit = git(
        root,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit-tree",
            tree,
            "-m",
            "malformed fixture",
        ],
    )
    .unwrap();
    (
        root.to_owned(),
        String::from_utf8(commit).unwrap().trim().into(),
    )
}
