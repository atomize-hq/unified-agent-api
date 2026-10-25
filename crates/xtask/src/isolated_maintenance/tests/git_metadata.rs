//! IB-DEF-1: nested external Git storage must not acquire or retain a caller binding.
use super::*;

#[derive(Clone, Copy, Debug)]
enum Namespace {
    Fanout,
    Pack,
    Refs,
}

fn nested_path(fixture: &Fixture, namespace: Namespace) -> PathBuf {
    match namespace {
        Namespace::Fanout => {
            let head = git(fixture.root.path(), &["rev-parse", "HEAD"]);
            fixture.root.path().join(".git/objects").join(&head[..2])
        }
        Namespace::Pack => {
            git(fixture.root.path(), &["repack", "-a", "-d"]);
            let path = fixture.root.path().join(".git/objects/pack");
            assert!(fs::read_dir(&path).unwrap().next().is_some());
            path
        }
        Namespace::Refs => fixture.root.path().join(".git/refs/heads"),
    }
}

fn refuses_externalized_namespace(namespace: Namespace, after_capture: bool) {
    let fixture = Fixture::new();
    let path = nested_path(&fixture, namespace);
    // This genuine positive control also covers packed storage before it is externalized.
    let context = fixture.resolve().unwrap();
    context.recheck(fixture.root.path()).unwrap();
    let external = tempfile::tempdir().unwrap();
    let storage = external.path().join("storage");
    fs::rename(&path, &storage).unwrap();
    std::os::unix::fs::symlink(&storage, &path).unwrap();
    // Git still consumes the same commit and branch; the ownership predicate must refuse it.
    assert!(!git(fixture.root.path(), &["cat-file", "-p", "HEAD"]).is_empty());
    assert_eq!(
        git(fixture.root.path(), &["symbolic-ref", "HEAD"]),
        "refs/heads/staging"
    );
    let result = if after_capture {
        context.recheck(fixture.root.path())
    } else {
        fixture.resolve().map(drop)
    };
    assert!(
        result.is_err(),
        "IB-DEF-1: {namespace:?}, after_capture={after_capture}"
    );
    assert!(result
        .unwrap_err()
        .contains("symlinked isolated Git metadata descendant"));
    assert!(fs::symlink_metadata(&path)
        .unwrap()
        .file_type()
        .is_symlink());
    assert!(fs::read_dir(storage).unwrap().next().is_some());
}

#[test]
fn ib_def_1_fanout_acquire() {
    refuses_externalized_namespace(Namespace::Fanout, false);
}
#[test]
fn ib_def_1_fanout_recheck() {
    refuses_externalized_namespace(Namespace::Fanout, true);
}
#[test]
fn ib_def_1_pack_acquire() {
    refuses_externalized_namespace(Namespace::Pack, false);
}
#[test]
fn ib_def_1_pack_recheck() {
    refuses_externalized_namespace(Namespace::Pack, true);
}
#[test]
fn ib_def_1_refs_acquire() {
    refuses_externalized_namespace(Namespace::Refs, false);
}
#[test]
fn ib_def_1_refs_recheck() {
    refuses_externalized_namespace(Namespace::Refs, true);
}
