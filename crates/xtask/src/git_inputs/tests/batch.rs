use super::*;

pub(super) fn read_many(
    root: &Path,
    paths: &[&str],
    borrowed: Option<&(PathBuf, String)>,
) -> Vec<Result<Input, String>> {
    crate::git_inputs::batch::read_many(root, paths, borrowed).collect()
}

fn equivalent(root: &Path, paths: &[&str], borrowed: Option<&(PathBuf, String)>) {
    let actual = read_many(root, paths, borrowed);
    let cache =
        borrowed.map(|(caller, commit)| super::super::immutable::Immutable::new(caller, commit));
    assert_eq!(actual.len(), paths.len());
    for (path, actual) in paths.iter().zip(actual) {
        let expected = read(root, path, borrowed);
        if cache.is_some() {
            assert_eq!(
                read_cached(root, path, borrowed, cache.as_ref()),
                expected,
                "{path}"
            );
        }
        match (actual, expected) {
            (Ok(actual), Ok(expected)) => {
                assert_eq!(actual.index, expected.index, "{path}");
                assert_eq!(actual.bytes, expected.bytes, "{path}");
            }
            (Err(actual), Err(expected)) => assert_eq!(actual, expected, "{path}"),
            _ => panic!("different result for {path}"),
        }
    }
}

pub(super) fn commit(root: &Path) -> (PathBuf, String) {
    git(
        root,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-qm",
            "fixture",
        ],
    )
    .unwrap();
    let head = String::from_utf8(git(root, &["rev-parse", "HEAD"]).unwrap()).unwrap();
    (root.to_owned(), head.trim().to_owned())
}

#[test]
fn batch_matches_single_for_index_and_borrowed_entries() {
    let root = fixture();
    fs::create_dir(root.path().join("dir")).unwrap();
    fs::create_dir(root.path().join("one")).unwrap();
    for path in [
        "dir/a",
        "dir/b",
        "one/child",
        "space ü\nline",
        ":(glob)*",
        "empty",
    ] {
        fs::write(
            root.path().join(path),
            if path == "empty" {
                &b""[..]
            } else {
                &b"\0bytes\n"[..]
            },
        )
        .unwrap();
    }
    git(root.path(), &["add", "."]).unwrap();
    let borrowed = commit(root.path());
    fs::write(root.path().join("untracked"), "extra").unwrap();
    let paths = [
        "bound",
        "absent",
        "untracked",
        "dir",
        "dir/a",
        "one",
        "one/child",
        "space ü\nline",
        ":(glob)*",
        "empty",
        "bound",
        "../bad",
        ".git/config",
        "bound/child",
    ];
    equivalent(root.path(), &paths, None);
    equivalent(root.path(), &paths, Some(&borrowed));
    fs::write(root.path().join("bound"), "changed").unwrap();
    fs::remove_file(root.path().join("dir/a")).unwrap();
    equivalent(root.path(), &paths, None);
    equivalent(root.path(), &paths, Some(&borrowed));
}

#[cfg(unix)]
#[test]
fn batch_matches_single_for_modes_and_symlinks() {
    use std::os::unix::{fs::symlink, fs::PermissionsExt};
    let root = fixture();
    fs::write(root.path().join("exec"), "executable").unwrap();
    fs::set_permissions(root.path().join("exec"), fs::Permissions::from_mode(0o755)).unwrap();
    symlink("bound", root.path().join("link")).unwrap();
    symlink(".", root.path().join("parent")).unwrap();
    git(root.path(), &["add", "."]).unwrap();
    let borrowed = commit(root.path());
    let paths = ["bound", "exec", "link", "parent/bound"];
    equivalent(root.path(), &paths, None);
    equivalent(root.path(), &paths, Some(&borrowed));
    fs::set_permissions(root.path().join("bound"), fs::Permissions::from_mode(0o755)).unwrap();
    fs::set_permissions(root.path().join("exec"), fs::Permissions::from_mode(0o644)).unwrap();
    equivalent(root.path(), &paths, None);
    equivalent(root.path(), &paths, Some(&borrowed));
    fs::remove_file(root.path().join("link")).unwrap();
    fs::write(root.path().join("link"), "bound").unwrap();
    equivalent(root.path(), &paths, None);
    equivalent(root.path(), &paths, Some(&borrowed));
}

#[test]
fn batch_matches_single_for_unmerged_and_invalid_objects() {
    use std::{io::Write, process::Stdio};
    let root = fixture();
    let borrowed = commit(root.path());
    let object = String::from_utf8(git(root.path(), &["rev-parse", ":bound"]).unwrap()).unwrap();
    let mut child = command(root.path())
        .args(["update-index", "--index-info"])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    write!(
        child.stdin.take().unwrap(),
        "0 {}\tbound\n100644 {} 1\tbound\n100644 {} 2\tbound\n",
        "0".repeat(40),
        object.trim(),
        object.trim()
    )
    .unwrap();
    assert!(child.wait().unwrap().success());
    equivalent(root.path(), &["absent", "bound"], None);
    equivalent(root.path(), &["absent", "bound"], Some(&borrowed));
    git(root.path(), &["update-index", "--force-remove", "bound"]).unwrap();
    for oid in [
        "1".repeat(40),
        String::from_utf8(git(root.path(), &["rev-parse", "HEAD^{tree}"]).unwrap())
            .unwrap()
            .trim()
            .into(),
    ] {
        git(
            root.path(),
            &[
                "update-index",
                "--add",
                "--cacheinfo",
                &format!("100644,{oid},bound"),
            ],
        )
        .unwrap();
        equivalent(root.path(), &["bound", "absent"], None);
    }
}

#[test]
fn batch_uses_constant_git_processes() {
    let root = fixture();
    let names: Vec<_> = (0..2048).map(|i| format!("file-{i}")).collect();
    for name in &names {
        fs::write(root.path().join(name), vec![42; 4096]).unwrap();
    }
    git(root.path(), &["add", "."]).unwrap();
    let borrowed = commit(root.path());
    let paths: Vec<_> = names.iter().map(String::as_str).collect();
    for borrowed in [None, Some(&borrowed)] {
        GIT_COMMANDS.with(|count| count.set(0));
        assert!(read_many(root.path(), &paths, borrowed)
            .iter()
            .all(Result::is_ok));
        assert_eq!(GIT_COMMANDS.with(|count| count.get()), 3);
    }
}

#[test]
fn borrowed_batch_reads_the_callers_objects_without_a_scratch_repository() {
    let caller = fixture();
    let borrowed = commit(caller.path());
    let scratch = tempfile::tempdir().unwrap();
    fs::write(scratch.path().join("bound"), "initial").unwrap();
    equivalent(scratch.path(), &["bound", "absent"], Some(&borrowed));
    // An indexed file replaced by a directory retains the single reader's non-file error.
    fs::remove_file(scratch.path().join("bound")).unwrap();
    fs::create_dir(scratch.path().join("bound")).unwrap();
    equivalent(scratch.path(), &["bound", "absent"], Some(&borrowed));
}

#[test]
fn exceptional_tree_listing_and_blob_failures_preserve_single_results() {
    let root = fixture();
    fs::create_dir(root.path().join("dir")).unwrap();
    fs::write(root.path().join("dir/child"), "child").unwrap();
    git(root.path(), &["add", "."]).unwrap();
    let borrowed = commit(root.path());
    let blob = String::from_utf8(git(root.path(), &["rev-parse", ":bound"]).unwrap()).unwrap();
    let blob = blob.trim();
    fs::remove_file(
        root.path()
            .join(format!(".git/objects/{}/{}", &blob[..2], &blob[2..])),
    )
    .unwrap();
    equivalent(root.path(), &["bound", "dir/child", "absent"], None);
    equivalent(
        root.path(),
        &["bound", "dir/child", "absent"],
        Some(&borrowed),
    );
    let tree = String::from_utf8(git(root.path(), &["rev-parse", "HEAD:dir"]).unwrap()).unwrap();
    let tree = tree.trim();
    fs::remove_file(
        root.path()
            .join(format!(".git/objects/{}/{}", &tree[..2], &tree[2..])),
    )
    .unwrap();
    equivalent(
        root.path(),
        &["dir", "absent", "bound", "dir/child"],
        Some(&borrowed),
    );
}

#[cfg(unix)]
#[test]
fn unrelated_non_utf8_paths_remain_opaque_and_requested_descendants_match() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
    let root = fixture();
    fs::create_dir(root.path().join("dir")).unwrap();
    fs::write(root.path().join(OsStr::from_bytes(b"dir/\xff")), "bytes").unwrap();
    git(root.path(), &["add", "."]).unwrap();
    let borrowed = commit(root.path());
    equivalent(root.path(), &["bound", "dir", "absent"], None);
    equivalent(root.path(), &["bound", "dir", "absent"], Some(&borrowed));
}

#[test]
fn empty_or_invalid_batches_do_not_spawn_git() {
    let root = fixture();
    GIT_COMMANDS.with(|count| count.set(0));
    assert!(read_many(root.path(), &[], None).is_empty());
    let paths = [
        "",
        "../bound",
        "./bound",
        "dir//bound",
        "dir\\bound",
        ".git/index",
    ];
    assert!(read_many(root.path(), &paths, None)
        .iter()
        .all(Result::is_err));
    assert_eq!(GIT_COMMANDS.with(|count| count.get()), 0);
    equivalent(root.path(), &paths, None);
}

#[test]
fn single_unmerged_stage_and_gitlink_modes_match_the_single_reader() {
    let root = fixture();
    let borrowed = commit(root.path());
    let object = String::from_utf8(git(root.path(), &["rev-parse", ":bound"]).unwrap()).unwrap();
    let mut child = command(root.path())
        .args(["update-index", "--index-info"])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    write!(
        child.stdin.take().unwrap(),
        "0 {}\tbound\n100644 {} 2\tbound\n",
        "0".repeat(40),
        object.trim()
    )
    .unwrap();
    assert!(child.wait().unwrap().success());
    equivalent(root.path(), &["bound"], None);
    git(root.path(), &["update-index", "--force-remove", "bound"]).unwrap();
    git(
        root.path(),
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{},bound", borrowed.1),
        ],
    )
    .unwrap();
    equivalent(root.path(), &["bound", "bound/child"], None);
    let borrowed = commit(root.path());
    equivalent(root.path(), &["bound", "bound/child"], Some(&borrowed));
}

#[test]
fn streamed_batches_fetch_each_blob_only_when_consumed() {
    for borrowed in [false, true] {
        let root = fixture();
        fs::write(root.path().join("second"), "initial").unwrap();
        git(root.path(), &["add", "."]).unwrap();
        let commit = commit(root.path());
        let borrowed = borrowed.then_some(&commit);
        let object =
            String::from_utf8(git(root.path(), &["rev-parse", ":bound"]).unwrap()).unwrap();
        let object = object.trim();
        let mut inputs =
            crate::git_inputs::batch::read_many(root.path(), &["bound", "second"], borrowed);
        assert!(inputs.next().unwrap().is_ok());
        fs::remove_file(root.path().join(format!(
            ".git/objects/{}/{}",
            &object[..2],
            &object[2..]
        )))
        .unwrap();
        let expected = read(root.path(), "second", borrowed);
        assert!(expected.is_err());
        assert_eq!(inputs.next().unwrap(), expected);
        assert!(inputs.next().is_none());
    }
}
