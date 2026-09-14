use std::{fs, path::Path};

use tempfile::TempDir;
use xtask::agent_maintenance::audit_status::{self, Args, Error, EXIT_TARGET_VERSION_MISMATCH};

const REQUEST_PATH: &str =
    "docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml";
const REQUEST_VERSION: &str = "0.98.0";
const EXPECTED_VERSION: &str = "0.99.0";

#[test]
fn mismatch_wins_over_an_invalid_request_commit() {
    let request = fs::read_to_string(repo_root().join(REQUEST_PATH)).expect("read real request");
    let request = request
        .lines()
        .map(|line| {
            if line.starts_with("request_commit = ") {
                "request_commit = \"NOT A COMMIT\""
            } else if line.starts_with("target_version = ") {
                "target_version = \"0.98.0\""
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let root = fixture(&request);

    assert_mismatch(run(&root, None));
}

#[test]
fn mismatch_does_not_replace_a_preseeded_projection() {
    let root = fixture(
        r#"[detected_release]
target_version = "0.98.0"
"#,
    );
    let projection = root.path().join("_ci_tmp/audit/status.json");
    fs::create_dir_all(projection.parent().expect("projection parent")).expect("create parent");
    let seeded = b"{\n  \"preseeded\": true\n}\n";
    fs::write(&projection, seeded).expect("seed projection");

    assert_mismatch(run(&root, Some(projection.clone())));
    assert_eq!(fs::read(projection).expect("read projection"), seeded);
}

#[test]
fn an_unparseable_request_falls_through_to_the_validated_loader() {
    let mismatch_root = fixture(mismatch_only_request());
    assert_mismatch(run(&mismatch_root, None));
    let root = fixture("not = [valid toml\n");
    let err = run(&root, None).expect_err("invalid TOML must fail validation");

    assert!(
        matches!(err, Error::Validation(_)),
        "unexpected error: {err:?}"
    );
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("parse maintenance request"));
}

#[test]
#[cfg(unix)]
fn an_outside_request_is_not_read_by_the_precheck() {
    let mismatch_root = fixture(mismatch_only_request());
    assert_mismatch(run(&mismatch_root, None));
    let root = TempDir::new().expect("workspace root");
    let outside = TempDir::new().expect("outside root");
    let outside_request = outside.path().join("maintenance-request.toml");
    fs::write(
        &outside_request,
        "[detected_release]\ntarget_version = \"0.98.0\"\n",
    )
    .expect("write outside request");
    let request = root.path().join(REQUEST_PATH);
    fs::create_dir_all(request.parent().expect("request parent")).expect("create request parent");
    std::os::unix::fs::symlink(&outside_request, request).expect("symlink outside request");
    let err = run(&root, None).expect_err("outside request must be rejected by validated loader");

    assert!(
        matches!(err, Error::Validation(_)),
        "unexpected error: {err:?}"
    );
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("resolves outside workspace root"));
    assert!(!err
        .to_string()
        .contains("does not describe the version under acquisition"));
}

fn fixture(request: &str) -> TempDir {
    let root = TempDir::new().expect("workspace root");
    let path = root.path().join(REQUEST_PATH);
    fs::create_dir_all(path.parent().expect("request parent")).expect("create request parent");
    fs::write(path, request).expect("write request");
    root
}

fn mismatch_only_request() -> &'static str {
    "[detected_release]\ntarget_version = \"0.98.0\"\n"
}

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

fn run(
    root: &TempDir,
    emit_json: Option<std::path::PathBuf>,
) -> Result<audit_status::AuditStatusOutcome, Error> {
    audit_status::run_in_workspace(
        root.path(),
        args(Path::new(REQUEST_PATH), emit_json),
        &mut Vec::new(),
    )
}

fn args(request: &Path, emit_json: Option<std::path::PathBuf>) -> Args {
    Args {
        request: request.to_path_buf(),
        expect_target_version: Some(EXPECTED_VERSION.to_owned()),
        emit_json,
        workspace_root: None,
    }
}

fn assert_mismatch(result: Result<audit_status::AuditStatusOutcome, Error>) {
    let err = result.expect_err("target version mismatch must fail");
    assert!(
        matches!(err, Error::TargetVersionMismatch(_)),
        "unexpected error: {err:?}"
    );
    assert_eq!(err.exit_code(), EXIT_TARGET_VERSION_MISMATCH);
    for needle in [
        EXPECTED_VERSION,
        REQUEST_VERSION,
        "does not describe the version under acquisition",
    ] {
        assert!(err.to_string().contains(needle), "missing `{needle}`");
    }
}
