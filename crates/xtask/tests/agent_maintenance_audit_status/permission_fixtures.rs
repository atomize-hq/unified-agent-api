use super::*;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(unix)]
struct PermissionRestoreGuard {
    path: PathBuf,
    original_permissions: fs::Permissions,
}

#[cfg(unix)]
impl Drop for PermissionRestoreGuard {
    fn drop(&mut self) {
        let _ = fs::set_permissions(&self.path, self.original_permissions.clone());
    }
}

#[cfg(unix)]
fn block_permissions(path: &Path, mode: u32) -> PermissionRestoreGuard {
    let guard_path = path.to_path_buf();
    let original_permissions = fs::metadata(path)
        .unwrap_or_else(|error| panic!("stat permission fixture {}: {error}", path.display()))
        .permissions();
    let mut blocked_permissions = original_permissions.clone();
    blocked_permissions.set_mode(mode);
    fs::set_permissions(path, blocked_permissions)
        .unwrap_or_else(|error| panic!("block permission fixture {}: {error}", path.display()));
    PermissionRestoreGuard {
        path: guard_path,
        original_permissions,
    }
}

#[cfg(unix)]
fn assert_permission_denied<T>(result: std::io::Result<T>, operation: &str) {
    match result {
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {}
        Err(error) => panic!(
            "permission fixture probe `{operation}` failed with {:?}, expected PermissionDenied: {error}",
            error.kind()
        ),
        Ok(_) => panic!(
            "permission fixture probe `{operation}` succeeded; this environment bypasses the configured permissions"
        ),
    }
}

#[cfg(unix)]
#[test]
fn permission_guard_restores_mode_after_panic() {
    let fixture = fixture_root("agent-maintenance-audit-status-permission-guard-panic");
    let guarded_dir = fixture.join("guarded");
    fs::create_dir_all(&guarded_dir).expect("create guarded fixture directory");
    let original_mode = fs::metadata(&guarded_dir)
        .expect("stat guarded fixture directory")
        .permissions()
        .mode();

    let panic = std::panic::catch_unwind(|| {
        let _restore = block_permissions(&guarded_dir, 0o000);
        panic!("intentional panic while permissions are blocked");
    });

    assert!(panic.is_err(), "guard test must exercise panic unwinding");
    let restored_mode = fs::metadata(&guarded_dir)
        .expect("stat restored fixture directory")
        .permissions()
        .mode();
    assert_eq!(restored_mode, original_mode);
}

#[cfg(unix)]
#[test]
fn unreadable_live_evidence_is_internal_error_not_validation() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-evidence-io-failure",
        &clean_report(TARGET_VERSION),
    );
    let err = run_with_unreadable_evidence(&fixture);
    assert!(matches!(err, AuditStatusError::Internal(_)));
    assert_eq!(err.exit_code(), 1);
    assert_ne!(err.exit_code(), 2);
}

#[cfg(unix)]
#[test]
fn unreadable_live_evidence_has_same_exit_code_with_and_without_discovery_work() {
    let clean_fixture = prepared_fixture(
        "agent-maintenance-audit-status-unreadable-clean",
        &clean_report(TARGET_VERSION),
    );
    let discovery_fixture = prepared_fixture(
        "agent-maintenance-audit-status-unreadable-discovery",
        &discovery_report(TARGET_VERSION),
    );

    let clean_err = run_with_unreadable_evidence(&clean_fixture);
    let discovery_err = run_with_unreadable_evidence(&discovery_fixture);

    assert!(matches!(clean_err, AuditStatusError::Internal(_)));
    assert!(matches!(discovery_err, AuditStatusError::Internal(_)));
    assert_eq!(clean_err.exit_code(), 1);
    assert_eq!(discovery_err.exit_code(), clean_err.exit_code());
    assert_ne!(discovery_err.exit_code(), 2);
}

#[cfg(unix)]
#[rustfmt::skip]
#[test]
fn uplift_outcome_survives_unwritable_emit_json_target_and_warns() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-uplifts-unwritable-emit-json", &discovery_report(TARGET_VERSION)); let emit_path = fixture.join("_ci_tmp/audit/status.json");
    let (result, stderr) = with_unwritable_emit_parent(&emit_path, || run_emit_json_with_stderr(&fixture, &emit_path)); let outcome = result.expect("computed uplift outcome should win write failures");
    assert_eq!(outcome, AuditStatusOutcome::UpliftsRequired); assert_eq!(outcome.exit_code(), EXIT_UPLIFTS_REQUIRED); assert_projection_write_warning(&stderr, &emit_path, &["could not write advisory projection", "Permission denied"]);
}

#[cfg(unix)]
#[rustfmt::skip]
#[test]
fn clean_outcome_survives_unwritable_emit_json_target() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-clean-unwritable-emit-json", &clean_report(TARGET_VERSION)); let emit_path = fixture.join("_ci_tmp/audit/status.json");
    let (result, stderr) = with_unwritable_emit_parent(&emit_path, || run_emit_json_with_stderr(&fixture, &emit_path)); let outcome = result.expect("computed clean outcome should win write failures");
    assert_eq!(outcome, AuditStatusOutcome::Clean); assert_eq!(outcome.exit_code(), 0); assert_projection_write_warning(&stderr, &emit_path, &["could not write advisory projection", "Permission denied"]);
}

#[cfg(unix)]
fn with_unwritable_emit_parent<T>(emit_path: &Path, action: impl FnOnce() -> T) -> T {
    let emit_parent = emit_path.parent().expect("emit parent");
    fs::create_dir_all(emit_parent).expect("create emit parent");
    let _restore = block_permissions(emit_parent, 0o555);
    let probe_path = emit_parent.join("permission-probe");
    let probe_result = fs::write(&probe_path, b"probe");
    if probe_result.is_ok() {
        let _ = fs::remove_file(&probe_path);
    }
    assert_permission_denied(probe_result, "write to unwritable emit parent");
    action()
}

#[cfg(unix)]
fn run_with_unreadable_evidence(root: &Path) -> AuditStatusError {
    let report_dir = coverage_report_dir(root);
    let _restore = block_permissions(&report_dir, 0o000);
    assert_permission_denied(
        fs::read_dir(&report_dir),
        "read unreadable coverage report directory",
    );
    audit_status::run_in_workspace(root, audit_args(REQUEST_PATH, None), &mut Vec::new())
        .expect_err("unreadable evidence must fail")
}
