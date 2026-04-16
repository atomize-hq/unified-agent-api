use super::*;

#[test]
fn c6_validator_emits_report_missing_includes_intentionally_unsupported() {
    let temp = make_temp_dir("ccm-c6-report-iu-validator");
    let codex_dir = temp.join("cli_manifests").join("codex");
    materialize_minimal_valid_codex_dir(&codex_dir);
    write_invalid_report_fixture(&codex_dir);

    assert_validation_failure(
        &codex_dir,
        "REPORT_MISSING_INCLUDES_INTENTIONALLY_UNSUPPORTED",
        "reports/0.61.0/coverage.any.json",
    );
}
