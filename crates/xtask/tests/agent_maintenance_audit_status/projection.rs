use super::*;

// The `--emit-json` projection's lifecycle: what it contains, that it is byte-reproducible, and
// which failures must leave it behind. The last of those is the contract `uaa-0025` sharpens, so
// the cases live together rather than scattered through the exit-code tests.

#[rustfmt::skip]
#[test]
fn failing_fresh_emit_json_write_removes_stale_projection() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-stale-emit-json", &clean_report(TARGET_VERSION)); let emit_path = force_write_failure_emit_path(&fixture, "status"); write_text(&emit_path, "{\n  \"stale\": true\n}\n");
    let (result, stderr) = run_emit_json_with_stderr(&fixture, &emit_path); let outcome = result.expect("computed clean outcome should win fresh write failures");
    assert_eq!(outcome, AuditStatusOutcome::Clean); assert_eq!(outcome.exit_code(), 0); assert_projection_write_warning(&stderr, &emit_path, &["forced maintenance audit status projection write failure"]); assert!(!emit_path.exists(), "fresh write failures must remove stale projections so the advisory file is never stale");
}

#[test]
fn emitted_json_is_byte_identical_across_identical_runs() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-byte-identical",
        &discovery_report(TARGET_VERSION),
    );
    let emit_path = fixture.join("_ci_tmp/audit/status.json");

    let (first, first_bytes) = emit_projection(&fixture, &emit_path);
    assert_eq!(first.exit_code(), EXIT_UPLIFTS_REQUIRED);

    let (second, second_bytes) = emit_projection(&fixture, &emit_path);
    assert_eq!(second.exit_code(), EXIT_UPLIFTS_REQUIRED);

    assert_eq!(first_bytes, second_bytes);
    assert!(
        first_bytes.ends_with(b"\n"),
        "emitted json should preserve the trailing newline contract"
    );
}

#[rustfmt::skip]
#[test]
fn failing_emit_json_post_derivation_failure_removes_existing_projection() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-preserve-emit-json-after-derivation", &clean_report(TARGET_VERSION)); let emit_path = fixture.join("_ci_tmp/audit/status.json"); let original = "{\n  \"stale\": true\n}\n";
    write_text(&emit_path, original); replace_in_request(&fixture, "pre_run_debt_count = 0", "pre_run_debt_count = 1");
    let (result, stderr) = run_emit_json_with_stderr(&fixture, &emit_path); let err = result.expect_err("post-derivation validation failures must remove the prior projection");
    assert_validation(&err, &["support_surface_audit"]); assert!(stderr.is_empty(), "derivation failures should not emit projection-write warnings"); assert!(!emit_path.exists(), "post-derivation failures must remove stale emit-json projections");
}

#[rustfmt::skip]
#[test]
fn failing_emit_json_pre_derivation_preserves_existing_projection() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-preserve-emit-json", &clean_report(TARGET_VERSION)); let emit_path = fixture.join("_ci_tmp/audit/status.json"); let original = "{\n  \"stale\": true\n}\n"; write_text(&emit_path, original);
    replace_in_request(&fixture, "agent_id = \"codex\"", "agent_id = \"unknown_agent\"");
    let err = audit_status::run_in_workspace(&fixture, audit_args(REQUEST_PATH, Some(emit_path.clone())), &mut Vec::new()).expect_err("pre-derivation validation failures must keep the existing projection");
    assert!(matches!(err, AuditStatusError::Validation(_))); assert_eq!(err.exit_code(), 2); assert_eq!(fs::read_to_string(&emit_path).expect("read preserved emit-json target"), original);
}

#[test]
fn long_emit_json_target_succeeds() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-long-emit-json",
        &clean_report(TARGET_VERSION),
    );
    let emit_path = fixture
        .join("_ci_tmp")
        .join(format!("{}.json", "a".repeat(245)));

    let (outcome, bytes) = emit_projection(&fixture, &emit_path);

    assert_eq!(outcome, AuditStatusOutcome::Clean);
    assert!(
        emit_path.is_file(),
        "long emit-json target should be written"
    );
    assert_eq!(parse_json(&bytes)["target_version"], json!(TARGET_VERSION));
}

#[test]
fn schema_version_is_first_key_and_stable_across_runs() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-schema-version",
        &clean_report(TARGET_VERSION),
    );
    let emit_path = fixture.join("_ci_tmp/audit/status.json");

    let (first, first_bytes) = emit_projection(&fixture, &emit_path);
    assert_eq!(first, AuditStatusOutcome::Clean);
    let (second, second_bytes) = emit_projection(&fixture, &emit_path);
    assert_eq!(second, AuditStatusOutcome::Clean);

    assert_eq!(parse_json(&first_bytes)["schema_version"], json!(1));
    assert_eq!(first_bytes, second_bytes);
    assert!(
        std::str::from_utf8(&first_bytes)
            .expect("schema-version json must be utf-8")
            .starts_with("{\n  \"schema_version\": 1,\n"),
        "schema_version should be emitted first for machine consumers"
    );
}

#[test]
fn request_sha256_is_present_after_schema_version_and_stable_across_runs() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-request-sha",
        &clean_report(TARGET_VERSION),
    );
    let emit_path = fixture.join("_ci_tmp/audit/status.json");
    let expected_request_sha256 = request_sha256(&fixture);
    let (first, first_bytes) = emit_projection(&fixture, &emit_path);
    assert_eq!(first, AuditStatusOutcome::Clean);
    let (second, second_bytes) = emit_projection(&fixture, &emit_path);
    assert_eq!(second, AuditStatusOutcome::Clean);

    assert_eq!(
        parse_json(&first_bytes)["request_sha256"],
        json!(expected_request_sha256)
    );
    assert_eq!(first_bytes, second_bytes);
    assert!(
        std::str::from_utf8(&first_bytes)
            .expect("request-sha json must be utf-8")
            .starts_with(&format!(
                "{{\n  \"schema_version\": 1,\n  \"request_sha256\": \"{}\",\n",
                expected_request_sha256
            )),
        "request_sha256 should follow schema_version for stale-file detection"
    );
}

// uaa-0025. These four pin the rule on `Args::emit_json` by demonstrating the hazard it names: a
// projection that outlives a failed run is indistinguishable from a current one, so a consumer may
// trust the exit code and nothing else. They are not assertions that staleness was eliminated —
// removing an existing file the run does not own is `uaa-0028`'s question, not this one's.

#[rustfmt::skip]
#[test]
fn a_pre_derivation_failure_leaves_a_projection_that_contradicts_the_run() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-stale-contradicts-run", &clean_report(TARGET_VERSION));
    write_live_coverage_report(&fixture, &discovery_report(TARGET_VERSION));
    let emit_path = fixture.join("_ci_tmp/audit/status.json");
    write_text(&emit_path, "{\n  \"schema_version\": 1,\n  \"uplifts_required\": false\n}\n");
    replace_in_request(&fixture, &format!("request_commit = \"{REQUEST_COMMIT}\""), "request_commit = \"NOT A COMMIT AT ALL\"");
    let (result, _stderr) = run_emit_json_with_stderr(&fixture, &emit_path);
    assert_eq!(result.expect_err("an invalid request_commit must fail validation").exit_code(), 2);
    // The live evidence required uplifts and the run then failed, yet the seeded file still says
    // the opposite. Existence is not currency, and the contents do not even have to agree.
    let surviving = fs::read(&emit_path).expect("the pre-derivation route must preserve the file");
    assert_eq!(parse_json(&surviving)["uplifts_required"], json!(false));
}

#[rustfmt::skip]
#[test]
fn a_corrupt_live_coverage_report_reaches_the_same_preserve_path() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-stale-corrupt-coverage", &clean_report(TARGET_VERSION));
    let emit_path = fixture.join("_ci_tmp/audit/status.json");
    write_text(&emit_path, "{\n  \"stale\": true\n}\n");
    write_text(&coverage_report_path(&fixture), "{\"inputs\":{\"upstream\":{\"semantic_ve");
    let (result, _stderr) = run_emit_json_with_stderr(&fixture, &emit_path);
    assert_eq!(result.expect_err("a corrupt live coverage report must fail validation").exit_code(), 2);
    assert!(emit_path.exists(), "a corrupt report is classified before live evidence is read, so it preserves too");
}

#[rustfmt::skip]
#[test]
fn a_clean_run_followed_by_a_failing_run_leaves_the_clean_projection_in_place() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-clean-then-stale", &clean_report(TARGET_VERSION));
    let emit_path = fixture.join("_ci_tmp/audit/status.json");
    let (first, clean_bytes) = emit_projection(&fixture, &emit_path);
    assert_eq!(first, AuditStatusOutcome::Clean);
    replace_in_request(&fixture, &format!("request_commit = \"{REQUEST_COMMIT}\""), "request_commit = \"NOT A COMMIT AT ALL\"");
    let (result, _stderr) = run_emit_json_with_stderr(&fixture, &emit_path);
    assert_eq!(result.expect_err("the second run must fail validation").exit_code(), 2);
    // Byte-identical to the clean run, request_sha256 included, so no field in the file tells a
    // consumer which run wrote it. This is why the rule has to be the invocation's exit code.
    assert_eq!(fs::read(&emit_path).expect("the clean projection must still be there"), clean_bytes);
}

#[rustfmt::skip]
#[test]
fn a_failed_post_derivation_cleanup_warns_instead_of_passing_silently() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-cleanup-warns", &clean_report(TARGET_VERSION));
    let emit_path = fixture.join("_ci_tmp/audit").join("status.force-remove-failure.json");
    write_text(&emit_path, "{\n  \"stale\": true\n}\n");
    replace_in_request(&fixture, "pre_run_debt_count = 0", "pre_run_debt_count = 1");
    let (result, stderr) = run_emit_json_with_stderr(&fixture, &emit_path);
    assert_eq!(result.expect_err("post-derivation validation failures must fail").exit_code(), 2);
    assert!(stderr.contains("could not remove stale advisory projection"), "a failed cleanup must be audible: {stderr}");
    assert!(stderr.contains(emit_path.to_string_lossy().as_ref()), "the warning must name the projection it could not remove: {stderr}");
}
