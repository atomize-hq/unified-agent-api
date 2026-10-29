use super::*;
#[test]
fn retention_reserves_attempt_and_never_overwrites_or_escapes() {
    let tmp = tempfile::tempdir().unwrap();
    let prefix = "cli_manifests/codex/reports/0.156.1/evidence/native-v1";
    std::fs::create_dir_all(tmp.path().join(prefix)).unwrap();
    let mut files = Retained::reserve(tmp.path(), &format!("{prefix}/attempt-1")).unwrap();
    let file = files.put("intent.json", b"finite intent").unwrap();
    assert_eq!(
        std::fs::read(tmp.path().join(file.path)).unwrap(),
        b"finite intent"
    );
    assert!(files.put("intent.json", b"replacement").is_err());
    assert!(Retained::reserve(tmp.path(), &format!("{prefix}/attempt-1")).is_err());
    assert!(files.put("../escape", b"outside").is_err());
    assert!(files.put("bundle.json", b"cycle").is_err());
    assert!(files.put("too-large", &vec![0; 1_048_577]).is_err());
}
#[cfg(unix)]
#[test]
fn retention_rejects_symlink_ancestors_and_changed_attempt_directory() {
    use std::os::unix::fs::symlink;
    let tmp = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    symlink(outside.path(), tmp.path().join("alias")).unwrap();
    assert!(Retained::reserve(tmp.path(), "alias/attempt").is_err());
    let mut files = Retained::reserve(tmp.path(), "attempt").unwrap();
    std::fs::rename(tmp.path().join("attempt"), tmp.path().join("prior")).unwrap();
    symlink(outside.path(), tmp.path().join("attempt")).unwrap();
    assert!(files.put("data.json", b"must not escape").is_err());
    assert!(!outside.path().join("data.json").exists());
}
#[test]
fn interrupted_child_keeps_completed_failure_and_full_requested_population() {
    use crate::depth_evidence::{
        native::{NativeFailure, NativeObservation, NativeStep},
        CaseOutcome,
    };
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join("execution")).unwrap();
    let requested = execution::requested(&"a".repeat(64));
    let step = NativeStep {
        case: requested[0].clone(),
        observation: NativeObservation::default(),
        outcome: CaseOutcome::Failed,
        elapsed_ms: 3,
        failure: Some(NativeFailure::RequestRefused),
    };
    std::fs::write(
        tmp.path().join("execution/case-0.json"),
        canonical(&step).unwrap(),
    )
    .unwrap();
    let cases = execution::journal(tmp.path(), "execution", &requested).unwrap();
    assert_eq!(cases.requested, requested);
    assert_eq!(cases.executed.len(), 1);
    assert_eq!(cases.executed[0].outcome, CaseOutcome::Failed);
    std::fs::write(
        tmp.path().join("execution/case-1.json"),
        br#"{"secret":"sk-must-not-be-retained"}"#,
    )
    .unwrap();
    assert!(execution::journal(tmp.path(), "execution", &requested).is_err());
}
#[test]
fn uninstalled_profile_refuses_before_workspace_or_native_io() {
    let args = form::CollectArgs {
        live: true,
        register: true,
        binary: "/never-probed".into(),
        working_dir: "/never-read".into(),
        codex_home: None,
        attempt: "attempt".into(),
    };
    assert!(form::collect(Path::new("/not-a-workspace"), args)
        .unwrap_err()
        .contains("profile is not installed"));
    let args = form::PrepareFinishArgs {
        attempt: "attempt".into(),
        evidence_commit: "a".repeat(40),
    };
    assert!(form::prepare_finish(Path::new("/not-a-workspace"), args)
        .unwrap_err()
        .contains("profile is not installed"));
}
