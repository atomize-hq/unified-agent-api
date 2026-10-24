use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use xtask::{
    manifest_promotion::{self, Args},
    workspace_mutation::{apply_mutations, PlannedMutation},
};
fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn git(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{:?}: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().into()
}
fn write(root: &Path, path: &str, bytes: impl AsRef<[u8]>) {
    let full = root.join(path);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(full, bytes).unwrap();
}
fn fixture() -> (tempfile::TempDir, Args) {
    let root = tempfile::tempdir().unwrap();
    let source = repo();
    let list = git(&source, &["ls-files", "-z"]);
    for path in list.split('\0').filter(|p| !p.is_empty()) {
        write(root.path(), path, fs::read(source.join(path)).unwrap());
    }
    git(root.path(), &["init", "-q"]);
    git(root.path(), &["config", "user.name", "Fixture"]);
    git(
        root.path(),
        &["config", "user.email", "fixture@example.invalid"],
    );
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "baseline"]);
    git(
        root.path(),
        &["update-ref", "refs/remotes/origin/staging", "HEAD"],
    );
    let version = "0.155.0".to_owned();
    let args = Args {
        agent: "codex".into(),
        version,
        passed_targets: vec!["x86_64-unknown-linux-musl".into()],
        validation_commit: git(root.path(), &["rev-parse", "HEAD"]),
        write: false,
    };
    (root, args)
}
#[test]
fn ordinary_promotion_is_read_only_until_applied_and_preserves_unvalidated_target_pointers() {
    let (root, args) = fixture();
    let before = git(root.path(), &["status", "--porcelain"]);
    let plan = manifest_promotion::build_plan(root.path(), &args).unwrap();
    assert_eq!(git(root.path(), &["status", "--porcelain"]), before);
    let mac = "cli_manifests/codex/pointers/latest_validated/aarch64-apple-darwin.txt";
    let prior = fs::read(root.path().join(mac)).unwrap();
    assert!(plan
        .paths()
        .iter()
        .any(|p| p.ends_with("runtime_support_data.rs")));
    plan.apply(root.path()).unwrap();
    assert_eq!(fs::read(root.path().join(mac)).unwrap(), prior);
    let m: serde_json::Value = serde_json::from_slice(
        &fs::read(root.path().join(format!(
            "cli_manifests/codex/versions/{}.json",
            args.version
        )))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        m["validation"]["passed_targets"],
        serde_json::json!(args.passed_targets)
    );
}
#[test]
fn cached_promotion_refuses_after_input_or_integration_tip_changes() {
    let (root, args) = fixture();
    let plan = manifest_promotion::build_plan(root.path(), &args).unwrap();
    let prior = fs::read(root.path().join("cli_manifests/codex/latest_validated.txt")).unwrap();
    write(root.path(), "crates/new-input.rs", "changed");
    assert!(plan
        .apply(root.path())
        .unwrap_err()
        .contains("inputs differ"));
    assert_eq!(
        fs::read(root.path().join("cli_manifests/codex/latest_validated.txt")).unwrap(),
        prior
    );
    fs::remove_file(root.path().join("crates/new-input.rs")).unwrap();
    let plan = manifest_promotion::build_plan(root.path(), &args).unwrap();
    git(
        root.path(),
        &["commit", "--allow-empty", "-qm", "tip movement"],
    );
    git(
        root.path(),
        &["update-ref", "refs/remotes/origin/staging", "HEAD"],
    );
    assert!(plan.apply(root.path()).unwrap_err().contains("revision"));
}
#[test]
fn neutral_lower_batch_cannot_write_retained_depth_even_without_a_declaration() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "cli_manifests/codex/reports/1.2.3/depth-record.json",
        "retained malformed record still blocks",
    );
    let plan = PlannedMutation::create(
        "cli_manifests/codex/latest_validated.txt",
        b"1.2.3\n".to_vec(),
    );
    assert!(apply_mutations(root.path(), &[plan])
        .unwrap_err()
        .to_string()
        .contains("retained record"));
    assert!(!root
        .path()
        .join("cli_manifests/codex/latest_validated.txt")
        .exists());
    let args = xtask::manifest_version_metadata::Args {
        root: Some(root.path().join("cli_manifests/codex")),
        rules: None,
        version: "1.2.3".into(),
        status: xtask::manifest_version_metadata::Status::Validated,
        passed_targets: vec![],
        failed_targets: vec![],
        skipped_targets: vec![],
    };
    assert!(xtask::manifest_version_metadata::run(args)
        .unwrap_err()
        .to_string()
        .contains("retained record"));
}
#[test]
fn unknown_unacquired_or_duplicate_passed_targets_refuse_without_outputs() {
    let (root, mut args) = fixture();
    for targets in [
        vec!["invented-target".into()],
        vec![
            args.passed_targets[0].clone(),
            args.passed_targets[0].clone(),
        ],
    ] {
        args.passed_targets = targets;
        assert!(manifest_promotion::build_plan(root.path(), &args).is_err());
        assert!(git(root.path(), &["status", "--porcelain"]).is_empty());
    }
}
