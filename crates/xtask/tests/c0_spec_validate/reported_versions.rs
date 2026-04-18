use super::*;

#[test]
fn c0_validate_requires_reports_when_version_status_reported() {
    let temp = make_temp_dir("ccm-c0-validate-reports");
    let codex_dir = materialize_minimal_valid_workspace(&temp);

    materialize_reported_codex_version(&codex_dir, false);
    write_support_matrix_artifact(&temp);

    let output = run_xtask_validate(&codex_dir);
    assert!(
        !output.status.success(),
        "expected failure:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("reports") && stderr.contains("coverage.any.json"),
        "expected report requirement violation, got:\n{stderr}"
    );

    write_minimal_report_files(&codex_dir, REPORTED_VERSION, &[REQUIRED_TARGET], false);
    write_support_matrix_artifact(&temp);

    let output = run_xtask_validate(&codex_dir);
    assert!(
        output.status.success(),
        "expected success after adding required reports:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn c0_validate_requires_coverage_all_only_when_union_complete() {
    let temp = make_temp_dir("ccm-c0-validate-coverage-all");
    let codex_dir = materialize_minimal_valid_workspace(&temp);

    materialize_reported_codex_version(&codex_dir, true);
    write_minimal_report_files(&codex_dir, REPORTED_VERSION, &TARGETS, false);
    write_support_matrix_artifact(&temp);

    let output = run_xtask_validate(&codex_dir);
    assert!(
        !output.status.success(),
        "expected failure (missing coverage.all.json):\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("coverage.all.json"),
        "expected missing coverage.all.json violation, got:\n{stderr}"
    );

    write_minimal_report_files(&codex_dir, REPORTED_VERSION, &TARGETS, true);
    write_support_matrix_artifact(&temp);

    let output = run_xtask_validate(&codex_dir);
    assert!(
        output.status.success(),
        "expected success after adding coverage.all.json:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
