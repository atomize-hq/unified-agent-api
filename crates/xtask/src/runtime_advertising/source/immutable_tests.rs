use super::*;
use std::{io::Write, process::Stdio};

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    let source = Source::workspace(root.path()).unwrap();
    source.git(&["init", "-q"]).unwrap();
    fs::create_dir_all(root.path().join("dir/sub")).unwrap();
    for (path, bytes) in [
        ("bound", &b"regular"[..]),
        ("dir/a", &b"binary\0\xff\n"[..]),
        ("dir/sub/child", &b"child"[..]),
        ("dir.ext", &b"sibling"[..]),
        ("space ü\n\tline", &b"special"[..]),
        (":(glob)*", &b"literal"[..]),
        ("empty", &b""[..]),
    ] {
        fs::write(root.path().join(path), bytes).unwrap();
    }
    source.git(&["add", "."]).unwrap();
    commit(&source);
    root
}

fn commit(source: &Source) {
    source
        .git(&[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-qm",
            "fixture",
        ])
        .unwrap();
}

fn compare(root: &Path, revision: &str, paths: &[&str]) {
    let cached = Source::tip(root, revision).unwrap();
    let mut single = cached.clone();
    single.immutable = None; // The original commands remain the independent differential oracle.
    for path in paths {
        assert_eq!(
            cached.optional_bytes(path),
            single.optional_bytes(path),
            "bytes {path:?}"
        );
        assert_eq!(
            cached.optional(path),
            single.optional(path),
            "text {path:?}"
        );
        assert_eq!(
            cached.executable(path),
            single.executable(path),
            "mode {path:?}"
        );
        assert_eq!(cached.paths(path), single.paths(path), "paths {path:?}");
        assert_eq!(
            cached.committed_files(path),
            single.committed_files(path),
            "files {path:?}"
        );
    }
}

#[test]
fn immutable_results_errors_order_and_special_paths_match_single_commands() {
    let root = fixture();
    fs::write(
        root.path().join("bound"),
        "workspace is not the committed view",
    )
    .unwrap();
    compare(
        root.path(),
        "HEAD",
        &[
            "bound",
            "dir",
            "dir/a",
            "dir/sub",
            "dir/sub/child",
            "dir/missing",
            "dir/sub/missing/deeper",
            "dir.ext",
            "space ü\n\tline",
            ":(glob)*",
            "empty",
            "absent",
            "absent/child",
            "bound/child",
            "",
            ".",
            "..",
            "../bound",
            "dir//a",
            "dir/",
            "dir/./a",
            ".git/config",
            "dir\\a",
            "bad\0path",
            "bad\0parent/child",
        ],
    );
}

#[cfg(unix)]
#[test]
fn immutable_symlinks_gitlinks_modes_and_non_utf8_paths_match() {
    use std::{
        ffi::OsStr,
        os::unix::{
            ffi::OsStrExt,
            fs::{symlink, PermissionsExt},
        },
    };
    let root = fixture();
    let source = Source::workspace(root.path()).unwrap();
    fs::set_permissions(root.path().join("bound"), fs::Permissions::from_mode(0o755)).unwrap();
    symlink("bound", root.path().join("link")).unwrap();
    symlink("dir", root.path().join("parent")).unwrap();
    fs::write(
        root.path().join("dir").join(OsStr::from_bytes(b"\xff")),
        "opaque",
    )
    .unwrap();
    source.git(&["add", "."]).unwrap();
    let head = source.git(&["rev-parse", "HEAD"]).unwrap();
    source
        .git(&[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{},gitlink", head.trim()),
        ])
        .unwrap();
    commit(&source);
    compare(
        root.path(),
        "HEAD",
        &[
            "bound",
            "link",
            "link/child",
            "parent/a",
            "parent",
            "gitlink",
            "gitlink/child",
            "dir",
            "absent",
        ],
    );
}

fn remove_object(source: &Source, revision: &str) {
    let object = source.git(&["rev-parse", revision]).unwrap();
    let object = object.trim();
    fs::remove_file(
        source
            .root
            .join(format!(".git/objects/{}/{}", &object[..2], &object[2..])),
    )
    .unwrap();
}

#[test]
fn immutable_missing_blob_subtree_and_commit_keep_bounded_errors() {
    let root = fixture();
    let source = Source::workspace(root.path()).unwrap();
    remove_object(&source, "HEAD:bound");
    compare(
        root.path(),
        "HEAD",
        &["bound", "dir/a", "absent", "bound/child"],
    );
    remove_object(&source, "HEAD:dir/sub");
    compare(
        root.path(),
        "HEAD",
        &[
            "bound",
            "dir",
            "dir/a",
            "dir/sub",
            "dir/sub/child",
            "absent",
        ],
    );
    let cached = Source::tip(root.path(), "HEAD").unwrap();
    let mut single = cached.clone();
    single.immutable = None;
    remove_object(&source, "HEAD");
    assert_eq!(
        cached.optional_bytes("bound"),
        single.optional_bytes("bound")
    );
    assert_eq!(
        cached.optional_bytes("../bad"),
        single.optional_bytes("../bad")
    );
}

#[test]
fn immutable_non_blob_objects_in_regular_entries_keep_cat_file_errors() {
    let root = fixture();
    let source = Source::workspace(root.path()).unwrap();
    let tree = source.git(&["rev-parse", "HEAD^{tree}"]).unwrap();
    let head = source.git(&["rev-parse", "HEAD"]).unwrap();
    let mut raw = Vec::new();
    for (name, object) in [
        ("commit", head.trim()),
        ("missing", &"1".repeat(40)),
        ("tree", tree.trim()),
    ] {
        raw.extend_from_slice(format!("100644 {name}\0").as_bytes());
        raw.extend(hex::decode(object).unwrap());
    }
    let revision = raw_commit(&source, &raw);
    compare(
        root.path(),
        &revision,
        &["commit", "missing", "tree", "absent"],
    );
}

fn raw_commit(source: &Source, raw: &[u8]) -> String {
    let mut child = crate::git_inputs::command(&source.root)
        .args(["hash-object", "-t", "tree", "--literally", "-w", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(raw).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let tree = String::from_utf8(output.stdout).unwrap();
    let commit = source
        .git(&[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit-tree",
            tree.trim(),
            "-m",
            "malformed fixture",
        ])
        .unwrap();
    commit.trim().into()
}

#[test]
fn malformed_duplicate_tree_paths_keep_the_original_multi_row_rejection() {
    let root = fixture();
    let source = Source::workspace(root.path()).unwrap();
    let object = source.git(&["rev-parse", "HEAD:bound"]).unwrap();
    let mut raw = Vec::new();
    for _ in 0..2 {
        raw.extend_from_slice(b"100644 duplicate\0");
        raw.extend(hex::decode(object.trim()).unwrap());
    }
    let revision = raw_commit(&source, &raw);
    assert!(Source::tip(root.path(), &revision)
        .unwrap()
        .optional_bytes("duplicate")
        .is_err());
    compare(root.path(), &revision, &["duplicate", "absent"]);
}

#[test]
fn malformed_unsorted_tree_paths_keep_the_original_pathspec_results() {
    let root = fixture();
    let source = Source::workspace(root.path()).unwrap();
    let object = source.git(&["rev-parse", "HEAD:bound"]).unwrap();
    let mut raw = Vec::new();
    for name in ["z", "a"] {
        raw.extend_from_slice(format!("100644 {name}\0").as_bytes());
        raw.extend(hex::decode(object.trim()).unwrap());
    }
    let revision = raw_commit(&source, &raw);
    compare(root.path(), &revision, &["a", "z", "absent"]);
}

#[test]
fn immutable_clones_stay_on_resolved_commit_after_head_moves() {
    let root = fixture();
    let cached = Source::tip(root.path(), "HEAD").unwrap();
    let mut single = cached.clone();
    single.immutable = None;
    assert_eq!(cached.read("bound").unwrap(), "regular");
    fs::write(root.path().join("bound"), "new commit").unwrap();
    let workspace = Source::workspace(root.path()).unwrap();
    workspace.git(&["add", "."]).unwrap();
    commit(&workspace);
    assert_eq!(
        cached.clone().optional_bytes("bound"),
        single.optional_bytes("bound")
    );
    assert_eq!(cached.paths("dir"), single.paths("dir"));
}

#[test]
fn warm_immutable_reads_observe_removed_and_corrupt_blobs() {
    for corrupt in [false, true] {
        let root = fixture();
        let cached = Source::tip(root.path(), "HEAD").unwrap();
        let mut single = cached.clone();
        single.immutable = None;
        assert_eq!(cached.read("bound").unwrap(), "regular");
        let object = single.git(&["rev-parse", "HEAD:bound"]).unwrap();
        let object = object.trim();
        let loose = root
            .path()
            .join(format!(".git/objects/{}/{}", &object[..2], &object[2..]));
        fs::remove_file(&loose).unwrap();
        if corrupt {
            fs::write(loose, b"corrupt loose object").unwrap();
        }
        let expected = single.optional_bytes("bound");
        assert!(expected.is_err());
        assert_eq!(cached.clone().optional_bytes("bound"), expected);
        // A failed stream must allow subsequent reads of other available blobs.
        assert_eq!(
            cached.optional_bytes("dir/a"),
            single.optional_bytes("dir/a")
        );
    }
}
