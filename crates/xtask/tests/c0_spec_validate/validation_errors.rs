use super::*;

#[test]
fn c0_validate_reports_wrapper_overlap_errors_with_required_fields_and_is_deterministic() {
    let temp = make_temp_dir("ccm-c0-validate-overlap");
    let codex_dir = materialize_minimal_valid_workspace(&temp);

    let wrapper_coverage = json!({
        "schema_version": 1,
        "generated_at": TS,
        "wrapper_version": "0.0.0-test",
        "coverage": [
            {
                "path": ["exec"],
                "level": "explicit",
                "scope": { "platforms": ["linux"] }
            },
            {
                "path": ["exec"],
                "level": "explicit",
                "scope": { "target_triples": [REQUIRED_TARGET] }
            }
        ]
    });
    write_json(&codex_dir.join("wrapper_coverage.json"), &wrapper_coverage);

    let a = run_xtask_validate(&codex_dir);
    let b = run_xtask_validate(&codex_dir);
    assert!(
        !a.status.success(),
        "expected overlap failure:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        a.status,
        String::from_utf8_lossy(&a.stdout),
        String::from_utf8_lossy(&a.stderr)
    );
    assert_eq!(
        a.stderr, b.stderr,
        "validator output must be deterministic for identical inputs"
    );

    let stderr = String::from_utf8_lossy(&a.stderr);
    assert!(
        stderr.contains("wrapper_coverage.json"),
        "expected wrapper_coverage.json path in errors, got:\n{stderr}"
    );
    assert!(
        stderr.contains(REQUIRED_TARGET),
        "expected target triple in overlap errors, got:\n{stderr}"
    );
    assert!(
        stderr.contains("exec"),
        "expected unit key (command path) in overlap errors, got:\n{stderr}"
    );
    assert!(
        stderr.contains("0") && stderr.contains("1"),
        "expected matching entry indexes mentioned in overlap errors, got:\n{stderr}"
    );
}

#[test]
fn c0_validate_rejects_pointer_files_without_trailing_newline() {
    let temp = make_temp_dir("ccm-c0-validate-pointer-newline");
    let codex_dir = materialize_minimal_valid_workspace(&temp);

    write_text(&codex_dir.join("latest_validated.txt"), VERSION);

    let output = run_xtask_validate(&codex_dir);
    assert!(
        !output.status.success(),
        "expected pointer format failure:\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("latest_validated.txt"),
        "expected latest_validated.txt referenced in errors, got:\n{stderr}"
    );
}
