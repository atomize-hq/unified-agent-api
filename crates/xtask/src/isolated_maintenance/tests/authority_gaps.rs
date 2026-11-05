//! Host and Git ownership refusals kept separate from the route fixture inventory.
use super::*;

#[test]
fn authority_gap_governing_blob_requires_independent_host_identity() {
    let f = Fixture::new();
    let head = git(f.root.path(), &["rev-parse", "HEAD"]);
    f.api.put(
        &format!("repos/fixture/isolated/contents/{REGISTRY_RELATIVE_PATH}?ref={head}"),
        json!({"path":REGISTRY_RELATIVE_PATH,"type":"file","sha":"0".repeat(40)}),
    );
    assert!(
        f.resolve().is_err(),
        "local protected objects alone authorized governing data"
    );
}
#[cfg(unix)]
#[test]
fn authority_gap_shared_object_metadata_requires_owned_directory() {
    let f = Fixture::new();
    let external = tempfile::tempdir().unwrap();
    fs::rename(
        f.root.path().join(".git/objects"),
        external.path().join("objects"),
    )
    .unwrap();
    std::os::unix::fs::symlink(
        external.path().join("objects"),
        f.root.path().join(".git/objects"),
    )
    .unwrap();
    assert!(
        f.resolve().is_err(),
        "shared symlinked object directory accepted"
    );
}
