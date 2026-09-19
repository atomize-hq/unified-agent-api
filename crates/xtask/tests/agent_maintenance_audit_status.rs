#![allow(dead_code, unused_imports)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::{json, Value};

#[path = "agent_maintenance_audit_status/permission_fixtures.rs"]
mod permission_fixtures;

#[path = "support/onboard_agent_harness.rs"]
mod harness;

mod agent_lifecycle {
    pub use xtask::agent_lifecycle::*;
}
mod agent_registry {
    pub use xtask::agent_registry::*;
}
#[path = "../src/agent_maintenance/audit_status.rs"]
mod audit_status;
#[path = "../src/agent_maintenance/contract_policy.rs"]
mod contract_policy;
#[path = "../src/agent_maintenance/docs.rs"]
mod docs;
#[path = "../src/agent_maintenance/prepare.rs"]
mod prepare;
#[path = "../src/agent_maintenance/request.rs"]
mod request;
#[path = "../src/agent_maintenance/support_audit.rs"]
mod support_audit;
#[path = "../src/workspace_mutation.rs"]
mod workspace_mutation;

use audit_status::{
    Args as AuditStatusArgs, AuditStatusOutcome, Error as AuditStatusError,
    EXIT_INCOMPLETE_ACQUISITION, EXIT_UPLIFTS_REQUIRED,
};
use harness::{fixture_root, write_text};
use prepare::{apply_prepare_plan, build_prepare_plan, Args as PrepareArgs};

const SEEDED_REGISTRY: &str = include_str!("../data/agent_registry.toml");
const REQUEST_PATH: &str =
    "docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml";
const TARGET_VERSION: &str = "0.98.0";
const REQUEST_COMMIT: &str = "abcdef1";

#[rustfmt::skip]
#[test]
fn empty_required_uplifts_reports_clean_exit_code_and_false_flag() {
    let (outcome, json) = run_success(&prepared_fixture("agent-maintenance-audit-status-clean", &clean_report(TARGET_VERSION)), "clean audit status");
    assert_eq!(outcome, AuditStatusOutcome::Clean); assert_eq!(outcome.exit_code(), 0); assert_eq!(json["agent_id"], json!("codex")); assert_eq!(json["target_version"], json!(TARGET_VERSION)); assert_projection(&json, false, "exact", 0, 0, 0, 0);
}

#[rustfmt::skip]
#[test]
fn expect_target_version_match_proceeds_normally() {
    let (outcome, json) = run_success_with_args(
        &prepared_fixture("agent-maintenance-audit-status-expect-target-version-match", &clean_report(TARGET_VERSION)),
        audit_args_with_expected(REQUEST_PATH, Some(TARGET_VERSION), None),
        "matching expected target version should proceed",
    );
    assert_eq!(outcome, AuditStatusOutcome::Clean); assert_eq!(outcome.exit_code(), 0); assert_eq!(json["target_version"], json!(TARGET_VERSION)); assert_projection(&json, false, "exact", 0, 0, 0, 0);
}

#[rustfmt::skip]
#[test]
fn expect_target_version_mismatch_is_typed_before_malformed_evidence_work() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-expect-target-version-mismatch", &clean_report(TARGET_VERSION)); write_text(&coverage_report_path(&fixture), "{not valid json\n");
    let err = audit_status::run_in_workspace(&fixture, audit_args_with_expected(REQUEST_PATH, Some("0.99.0"), None), &mut Vec::new()).expect_err("mismatched expected target version must fail");
    assert!(matches!(err, AuditStatusError::TargetVersionMismatch(_))); assert_eq!(err.exit_code(), 5); for needle in ["0.99.0", TARGET_VERSION, "does not describe the version under acquisition"] { assert!(err.to_string().contains(needle)); } assert!(!err.to_string().contains("parse"), "target-version mismatch must win over malformed evidence");
}

#[rustfmt::skip]
#[test]
fn omitting_expect_target_version_preserves_today_behavior_exactly() {
    let clean_fixture = prepared_fixture("agent-maintenance-audit-status-expect-target-version-clean-parity", &clean_report(TARGET_VERSION));
    let uplift_fixture = prepared_fixture("agent-maintenance-audit-status-expect-target-version-uplift-parity", &discovery_report(TARGET_VERSION));

    let (clean_without_flag_outcome, clean_without_flag_bytes) = run_success_bytes_with_args(&clean_fixture, audit_args(REQUEST_PATH, None), "clean audit status without expected target version");
    let (clean_with_flag_outcome, clean_with_flag_bytes) = run_success_bytes_with_args(&clean_fixture, audit_args_with_expected(REQUEST_PATH, Some(TARGET_VERSION), None), "clean audit status with matching expected target version");
    assert_eq!(clean_with_flag_outcome, clean_without_flag_outcome); assert_eq!(clean_with_flag_bytes, clean_without_flag_bytes);

    let (uplift_without_flag_outcome, uplift_without_flag_bytes) = run_success_bytes_with_args(&uplift_fixture, audit_args(REQUEST_PATH, None), "uplift audit status without expected target version");
    let (uplift_with_flag_outcome, uplift_with_flag_bytes) = run_success_bytes_with_args(&uplift_fixture, audit_args_with_expected(REQUEST_PATH, Some(TARGET_VERSION), None), "uplift audit status with matching expected target version");
    assert_eq!(uplift_with_flag_outcome, uplift_without_flag_outcome); assert_eq!(uplift_with_flag_bytes, uplift_without_flag_bytes);
}

#[rustfmt::skip]
#[test]
fn non_empty_required_uplifts_reports_exit_three_and_true_flag() {
    let (outcome, json) = run_success(&prepared_fixture("agent-maintenance-audit-status-uplifts", &discovery_report(TARGET_VERSION)), "uplift audit status");
    assert_eq!(outcome, AuditStatusOutcome::UpliftsRequired); assert_eq!(outcome.exit_code(), EXIT_UPLIFTS_REQUIRED); assert_projection(&json, true, "exact", 1, 0, 1, 1); assert_eq!(json["required_uplifts"], expected_required_uplifts_json());
}

#[rustfmt::skip]
#[test]
fn frozen_clean_packet_live_dirty_returns_exit_three_and_drifted_reconciliation() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-clean-frozen-live-dirty", &clean_report(TARGET_VERSION)); write_text(&coverage_report_path(&fixture), &discovery_report(TARGET_VERSION));
    let (outcome, json) = run_success(&fixture, "frozen-clean/live-dirty audit status");
    assert_eq!(outcome, AuditStatusOutcome::UpliftsRequired); assert_eq!(outcome.exit_code(), EXIT_UPLIFTS_REQUIRED); assert_projection(&json, true, "drifted", 1, 0, 1, 1); assert_eq!(json["required_uplifts"], expected_required_uplifts_json());
}

#[rustfmt::skip]
#[test]
fn missing_live_coverage_report_is_validation_error_not_clean() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-missing-coverage", &clean_report(TARGET_VERSION)); fs::remove_dir_all(coverage_report_dir(&fixture)).expect("remove seeded coverage report dir");
    let mut stdout = Vec::new(); let err = audit_status::run_in_workspace(&fixture, audit_args(REQUEST_PATH, None), &mut stdout).expect_err("missing live coverage report must fail");
    assert!(matches!(err, AuditStatusError::Validation(_)), "unexpected error: {err:?}"); assert_eq!(err.exit_code(), 2); assert!(stdout.is_empty(), "validation failures must not emit a clean projection"); assert!(err.to_string().contains("cli_manifests/codex/reports/0.98.0"), "error should name the expected target-version report directory");
}

#[rustfmt::skip]
#[test]
fn drifted_reconciliation_without_uplifts_is_validation_error() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-drifted-reconciliation", &clean_report(TARGET_VERSION)); replace_in_request(&fixture, "pre_run_debt_count = 0", "pre_run_debt_count = 1");
    let err = audit_status::run_in_workspace(&fixture, audit_args(REQUEST_PATH, None), &mut Vec::new()).expect_err("drifted reconciliation without uplifts must fail");
    assert!(matches!(err, AuditStatusError::Validation(_)), "unexpected error: {err:?}"); assert_eq!(err.exit_code(), 2); assert!(err.to_string().contains("support_surface_audit"), "drifted reconciliation should surface the packet inconsistency");
}

#[rustfmt::skip]
#[test]
fn failing_fresh_emit_json_write_removes_stale_projection() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-stale-emit-json", &clean_report(TARGET_VERSION)); let emit_path = force_write_failure_emit_path(&fixture, "status"); write_text(&emit_path, "{\n  \"stale\": true\n}\n");
    let (result, stderr) = run_emit_json_with_stderr(&fixture, &emit_path); let outcome = result.expect("computed clean outcome should win fresh write failures");
    assert_eq!(outcome, AuditStatusOutcome::Clean); assert_eq!(outcome.exit_code(), 0); assert_projection_write_warning(&stderr, &emit_path, &["forced maintenance audit status projection write failure"]); assert!(!emit_path.exists(), "fresh write failures must remove stale projections so the advisory file is never stale");
}

#[rustfmt::skip]
#[test]
fn malformed_or_unresolvable_request_is_validation_error_not_exit_three() {
    let fixture = fixture_root("agent-maintenance-audit-status-invalid-request"); seed_registry(&fixture);
    let err = audit_status::run_in_workspace(&fixture, audit_args("docs/agents/lifecycle/codex-maintenance/governance/missing-request.toml", None), &mut Vec::new()).expect_err("missing request must fail");
    assert!(matches!(err, AuditStatusError::Validation(_))); assert_eq!(err.exit_code(), 2); assert_ne!(err.exit_code(), EXIT_UPLIFTS_REQUIRED);
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
fn drifted_packet_with_invalid_request_commit_is_validation_error_not_exit_three() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-drifted-invalid-request-commit", &clean_report(TARGET_VERSION)); write_text(&coverage_report_path(&fixture), &discovery_report(TARGET_VERSION)); replace_in_request(&fixture, &format!("request_commit = \"{REQUEST_COMMIT}\""), "request_commit = \"NOT A COMMIT AT ALL\"");
    let err = audit_status::run_in_workspace(&fixture, audit_args(REQUEST_PATH, None), &mut Vec::new()).expect_err("invalid post-reconciliation fields must fail validation");
    assert!(matches!(err, AuditStatusError::Validation(_))); assert_eq!(err.exit_code(), 2); assert_ne!(err.exit_code(), EXIT_UPLIFTS_REQUIRED); assert!(err.to_string().contains("request_commit"), "post-reconciliation validation failures should name the invalid field");
}

#[test]
fn wrong_version_live_coverage_report_is_rejected() {
    let err = run_failure(
        &prepared_fixture(
            "agent-maintenance-audit-status-wrong-version-report",
            &clean_report("0.97.0"),
        ),
        "wrong-version coverage evidence must fail",
    );
    assert_validation(&err, &[TARGET_VERSION, "0.97.0"]);
    assert_ne!(err.exit_code(), EXIT_INCOMPLETE_ACQUISITION);
    assert_ne!(err.exit_code(), 0);
}

#[test]
fn mixed_version_evidence_directory_is_rejected_even_when_selected_report_matches() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-mixed-version-evidence",
        &clean_report(TARGET_VERSION),
    );
    replace_named_coverage_reports(
        &fixture,
        &[
            ("coverage.darwin-arm64.json", &clean_report(TARGET_VERSION)),
            ("coverage.linux-arm64.json", &clean_report("0.97.0")),
            ("coverage.linux-x64.json", &clean_report("0.97.0")),
            ("coverage.win32-x64.json", &clean_report("0.97.0")),
        ],
    );

    let mut stdout = Vec::new();
    let err = audit_status::run_in_workspace(&fixture, audit_args(REQUEST_PATH, None), &mut stdout)
        .expect_err("mixed-version evidence must not report clean");
    assert!(stdout.is_empty());
    assert_validation(
        &err,
        &["coverage.linux-arm64.json", TARGET_VERSION, "0.97.0"],
    );
}

#[rustfmt::skip]
#[test]
fn incomplete_union_snapshot_exits_four_and_names_missing_targets() {
    let (fixture, missing_targets) = incomplete_union_fixture("agent-maintenance-audit-status-incomplete-union-clean", &clean_report(TARGET_VERSION), 2);
    assert_incomplete_acquisition(&run_failure(&fixture, "incomplete union snapshot must exit four"), &targets(&missing_targets));
}

#[rustfmt::skip]
#[test]
fn incomplete_union_snapshot_blocks_uplift_exit_three_with_exit_four() {
    let (fixture, missing_targets) = incomplete_union_fixture("agent-maintenance-audit-status-incomplete-union-uplifts", &discovery_report(TARGET_VERSION), 1);
    let err = run_failure(&fixture, "incomplete union snapshot must block exit three"); assert_incomplete_acquisition(&err, &targets(&missing_targets)); assert_ne!(err.exit_code(), EXIT_UPLIFTS_REQUIRED);
}

#[rustfmt::skip]
#[test]
fn missing_union_snapshot_is_validation_error() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-missing-union", &clean_report(TARGET_VERSION)); fs::remove_file(union_snapshot_path(&fixture)).expect("remove seeded union snapshot");
    assert_validation(&run_failure(&fixture, "missing union snapshot must fail validation"), &["cli_manifests/codex/snapshots/0.98.0/union.json"]);
}

#[rustfmt::skip]
#[test]
fn malformed_union_snapshot_is_validation_error() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-malformed-union", &clean_report(TARGET_VERSION)); write_text(&union_snapshot_path(&fixture), "{\"complete\": false");
    assert_validation(&run_failure(&fixture, "malformed union snapshot must fail validation"), &["valid JSON with bool `complete`"]);
}

#[rustfmt::skip]
#[test]
fn union_snapshot_complete_must_be_present_and_bool_or_exit_two() {
    let missing_fixture = prepared_fixture("agent-maintenance-audit-status-union-missing-complete", &clean_report(TARGET_VERSION));
    rewrite_union_snapshot(&missing_fixture, |union| { union.as_object_mut().expect("union snapshot object").remove("complete"); });
    assert_validation(&run_failure(&missing_fixture, "missing complete must fail validation"), &["declare bool `complete`"]);

    let wrong_type_fixture = prepared_fixture("agent-maintenance-audit-status-union-non-bool-complete", &clean_report(TARGET_VERSION));
    rewrite_union_snapshot(&wrong_type_fixture, |union| { union["complete"] = json!("false"); });
    assert_validation(&run_failure(&wrong_type_fixture, "non-bool complete must fail validation"), &["valid JSON with bool `complete`"]);
}

#[rustfmt::skip]
#[test]
fn complete_union_snapshot_without_missing_targets_key_is_accepted() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-complete-union-no-missing-targets", &clean_report(TARGET_VERSION));
    rewrite_union_snapshot(&fixture, |union| { union["complete"] = json!(true); union.as_object_mut().expect("union snapshot object").remove("missing_targets"); });
    let (outcome, json) = run_success(&fixture, "complete union snapshot without missing_targets should be accepted");
    assert_eq!(outcome, AuditStatusOutcome::Clean); assert_eq!(outcome.exit_code(), 0); assert_eq!(json["uplifts_required"], json!(false));
}

#[rustfmt::skip]
#[test]
fn wrong_version_uplift_evidence_is_validation_error_not_exit_three() {
    let err = run_failure(&prepared_fixture("agent-maintenance-audit-status-wrong-version-uplifts", &discovery_report("0.97.0")), "wrong-version uplift evidence must fail validation");
    assert_validation(&err, &[TARGET_VERSION, "0.97.0"]); assert_ne!(err.exit_code(), EXIT_INCOMPLETE_ACQUISITION); assert_ne!(err.exit_code(), EXIT_UPLIFTS_REQUIRED);
}

#[rustfmt::skip]
#[test]
fn malformed_live_coverage_report_is_validation_error_not_internal() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-malformed-coverage", &clean_report(TARGET_VERSION)); write_text(&coverage_report_path(&fixture), "{\"inputs\":{\"upstream\":{\"semantic_ve");
    let err = run_failure(&fixture, "malformed coverage report must fail validation"); assert_validation(&err, &["parse"]); assert_ne!(err.exit_code(), 1);
}

#[rustfmt::skip]
#[test]
fn coverage_named_directory_is_validation_error_not_internal() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-coverage-directory", &clean_report(TARGET_VERSION)); fs::create_dir(coverage_report_dir(&fixture).join("coverage.d.json")).expect("create coverage-shaped directory");
    assert_validation(&run_failure(&fixture, "coverage-shaped directory must fail validation"), &["coverage.d.json"]);
}

#[rustfmt::skip]
#[test]
fn case_differing_coverage_report_is_bound_by_directory_walk() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-case-differing-report", &clean_report(TARGET_VERSION)); write_named_coverage_report(&fixture, "Coverage.linux-arm64.json", &clean_report("0.97.0"));
    assert_validation(&run_failure(&fixture, "case-differing coverage report must be bound by directory walk"), &["Coverage.linux-arm64.json", TARGET_VERSION, "0.97.0"]);
}

#[test]
fn validator_and_derivation_share_selected_report_path_for_per_os_only_directories() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-shared-selector",
        &discovery_report(TARGET_VERSION),
    );
    replace_named_coverage_reports(
        &fixture,
        &[
            (
                "coverage.darwin-arm64.json",
                &discovery_report(TARGET_VERSION),
            ),
            ("coverage.linux-x64.json", &discovery_report(TARGET_VERSION)),
        ],
    );

    let version_dir = coverage_report_dir(&fixture);
    let validator_selected =
        audit_status::selected_coverage_report_path(&version_dir).expect("validator selection");
    let derivation_selected =
        support_audit::select_report_path(&version_dir).expect("derivation selection");
    assert_eq!(validator_selected, derivation_selected);
    assert_eq!(
        validator_selected
            .file_name()
            .and_then(|name| name.to_str()),
        Some("coverage.darwin-arm64.json")
    );

    let live_audit = live_audit(&fixture);

    assert_eq!(live_audit.unbaselined_gap_surface.len(), 1);
    assert_eq!(
        live_audit.unbaselined_gap_surface[0].evidence_ref,
        validator_selected
            .strip_prefix(&fixture)
            .expect("repo-relative evidence path")
            .to_string_lossy()
            .to_string()
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

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("parse audit status json")
}

fn expected_required_uplifts_json() -> Value {
    json!([{
        "surface_kind": "commands",
        "command_path": "codex status",
        "surface_id": "status",
        "reason": "unbaselined_gap",
        "required_writes": ["backend", "manifest", "packet_docs", "publication", "wrapper"]
    }])
}

fn assert_projection(
    json: &Value,
    uplifts_required: bool,
    reconciliation: &str,
    discovered: u64,
    preexisting: u64,
    missing_wrapper: u64,
    missing_backend: u64,
) {
    assert_eq!(json["uplifts_required"], json!(uplifts_required));
    assert_eq!(
        json["required_uplifts"],
        if uplifts_required {
            expected_required_uplifts_json()
        } else {
            json!([])
        }
    );
    assert_eq!(json["reconciliation"], json!(reconciliation));
    assert_eq!(json["unbaselined_gap_surface"], json!(discovered));
    assert_eq!(json["preexisting_unsupported_surface"], json!(preexisting));
    assert_eq!(json["missing_wrapper_support"], json!(missing_wrapper));
    assert_eq!(json["missing_backend_support"], json!(missing_backend));
}

fn assert_validation(err: &AuditStatusError, contains: &[&str]) {
    assert!(
        matches!(err, AuditStatusError::Validation(_)),
        "unexpected error: {err:?}"
    );
    assert_eq!(err.exit_code(), 2);
    for needle in contains {
        assert!(
            err.to_string().contains(needle),
            "error should contain `{needle}`"
        );
    }
}

#[rustfmt::skip]
fn assert_incomplete_acquisition(err: &AuditStatusError, contains: &[&str]) {
    assert!(matches!(err, AuditStatusError::IncompleteAcquisition(_)), "unexpected error: {err:?}"); assert_eq!(err.exit_code(), EXIT_INCOMPLETE_ACQUISITION);
    for needle in contains { assert!(err.to_string().contains(needle), "error should contain `{needle}`"); }
}

#[rustfmt::skip]
fn assert_projection_write_warning(stderr: &str, emit_path: &Path, contains: &[&str]) {
    assert!(stderr.contains("warning:"), "projection write failures must warn on stderr"); assert!(stderr.contains(emit_path.to_string_lossy().as_ref()), "warning should name the advisory projection path");
    for needle in contains { assert!(stderr.contains(needle), "warning should contain `{needle}`"); }
}

fn run_success(root: &Path, context: &str) -> (AuditStatusOutcome, Value) {
    let (outcome, stdout) =
        run_success_bytes_with_args(root, audit_args(REQUEST_PATH, None), context);
    (outcome, parse_json(&stdout))
}

fn run_success_with_args(
    root: &Path,
    args: AuditStatusArgs,
    context: &str,
) -> (AuditStatusOutcome, Value) {
    let (outcome, stdout) = run_success_bytes_with_args(root, args, context);
    (outcome, parse_json(&stdout))
}

fn run_success_bytes_with_args(
    root: &Path,
    args: AuditStatusArgs,
    context: &str,
) -> (AuditStatusOutcome, Vec<u8>) {
    let mut stdout = Vec::new();
    let outcome = audit_status::run_in_workspace(root, args, &mut stdout).expect(context);
    (outcome, stdout)
}

fn run_failure(root: &Path, context: &str) -> AuditStatusError {
    audit_status::run_in_workspace(root, audit_args(REQUEST_PATH, None), &mut Vec::new())
        .expect_err(context)
}

fn incomplete_union_fixture(
    prefix: &str,
    coverage_report: &str,
    skip: usize,
) -> (PathBuf, Vec<String>) {
    let fixture = prepared_fixture(prefix, coverage_report);
    let missing_targets = union_expected_targets(&fixture)
        .into_iter()
        .skip(skip)
        .collect::<Vec<_>>();
    rewrite_union_snapshot(&fixture, |union| {
        union["complete"] = json!(false);
        union["missing_targets"] = json!(missing_targets.clone());
    });
    (fixture, missing_targets)
}

fn targets(values: &[String]) -> Vec<&str> {
    values.iter().map(String::as_str).collect()
}

#[rustfmt::skip]
fn emit_projection(root: &Path, emit_path: &Path) -> (AuditStatusOutcome, Vec<u8>) {
    let mut stdout = Vec::new(); let outcome = audit_status::run_in_workspace(root, audit_args(REQUEST_PATH, Some(emit_path.to_path_buf())), &mut stdout).expect("emit projection");
    assert!(stdout.is_empty(), "emit-json should not write stdout"); (outcome, fs::read(emit_path).expect("read emitted projection"))
}

#[rustfmt::skip]
fn run_emit_json_with_stderr(root: &Path, emit_path: &Path) -> (Result<AuditStatusOutcome, AuditStatusError>, String) {
    let mut stdout = Vec::new(); let mut stderr = Vec::new();
    let result = audit_status::run_in_workspace_with_stderr(root, audit_args(REQUEST_PATH, Some(emit_path.to_path_buf())), &mut stdout, &mut stderr);
    assert!(stdout.is_empty(), "emit-json should not write stdout"); (result, String::from_utf8(stderr).expect("captured stderr must be utf-8"))
}

#[rustfmt::skip]
fn live_audit(root: &Path) -> support_audit::SupportSurfaceAudit {
    let validated_request = request::load_request_envelope_validated_with_policy(root, Path::new(REQUEST_PATH), request::AuditDriftPolicy::Tolerate).expect("load validated request");
    let registry = agent_registry::AgentRegistry::load(root).expect("load registry");
    let detected_release = validated_request.envelope.request.detected_release.as_ref().expect("detected release");
    support_audit::derive_support_surface_audit(root, registry.find("codex").expect("codex entry"), detected_release).expect("derive live audit")
}

fn request_sha256(root: &Path) -> String {
    request::load_request_envelope_validated(root, Path::new(REQUEST_PATH))
        .expect("load validated request")
        .envelope
        .request
        .sha256
}

fn audit_args(request: &str, emit_json: Option<PathBuf>) -> AuditStatusArgs {
    audit_args_with_expected(request, None, emit_json)
}

fn audit_args_with_expected(
    request: &str,
    expect_target_version: Option<&str>,
    emit_json: Option<PathBuf>,
) -> AuditStatusArgs {
    AuditStatusArgs {
        request: PathBuf::from(request),
        expect_target_version: expect_target_version.map(str::to_owned),
        emit_json,
        workspace_root: None,
    }
}

fn prepared_fixture(prefix: &str, coverage_report: &str) -> PathBuf {
    let fixture = fixture_root(prefix);
    seed_registry(&fixture);
    seed_support_files(&fixture, coverage_report);
    let plan = build_prepare_plan(&fixture, &prepare_args()).expect("build prepare plan");
    apply_prepare_plan(&fixture, &plan).expect("apply prepare plan");
    fixture
}

#[rustfmt::skip]
fn prepare_args() -> PrepareArgs { PrepareArgs {
    agent: "codex".to_string(), current_version: "0.97.0".to_string(), latest_stable: "0.99.0".to_string(),
    target_version: TARGET_VERSION.to_string(), opened_from: Path::new(".github/workflows/agent-maintenance-open-pr.yml").to_path_buf(),
    detected_by: ".github/workflows/agent-maintenance-release-watch.yml".to_string(), dispatch_kind: "packet_pr".to_string(),
    dispatch_workflow: None, branch_name: "automation/codex-maintenance-0.98.0".to_string(),
    request_recorded_at: "2026-05-05T15:00:00Z".to_string(), request_commit: REQUEST_COMMIT.to_string(), dry_run: true, write: false,
} }

fn seed_registry(root: &Path) {
    write_text(
        &root.join("crates/xtask/data/agent_registry.toml"),
        SEEDED_REGISTRY,
    );
}

#[rustfmt::skip]
fn seed_support_files(root: &Path, coverage_report: &str) {
    let registry = agent_registry::AgentRegistry::parse(SEEDED_REGISTRY).expect("parse registry"); let entry = registry.find("codex").expect("codex entry");
    for (path, contents) in [
        (".github/workflows/agent-maintenance-open-pr.yml", "name: Packet PR worker\n"),
        ("cli_manifests/codex/PR_BODY_TEMPLATE.md", "@codex\n\n## Goal\n\nFollow the maintained PR template for {{VERSION}}.\n"),
        ("cli_manifests/codex/OPS_PLAYBOOK.md", "# Codex ops\n"), ("cli_manifests/codex/CI_WORKFLOWS_PLAN.md", "# Codex CI workflows\n"),
        ("docs/agents/lifecycle/codex-maintenance/OPS_PLAYBOOK.md", "# Packet ops\n"),
        ("docs/agents/lifecycle/codex-maintenance/CI_WORKFLOWS_PLAN.md", "# Packet workflow plan\n"),
        ("cli_manifests/codex/latest_validated.txt", "0.97.0\n"), ("docs/specs/unified-agent-api/non-tui-support-debt.md", "# Non-TUI Support Debt Inventory\n\n## Inventory\n"),
    ] { write_text(&root.join(path), contents); }
    write_text(&root.join("docs/agents/lifecycle/codex-maintenance/governance/execute-agent-maintenance-prompt.md"), &contract_policy::packet_pr_prompt_template(entry, "docs/agents/lifecycle/codex-maintenance"));
    write_text(&root.join(&entry.manifest_root).join("snapshots").join(TARGET_VERSION).join("union.json"), &complete_union_snapshot(&entry.canonical_targets));
    write_text(&coverage_report_path(root), coverage_report);
}

fn coverage_report_dir(root: &Path) -> PathBuf {
    root.join("cli_manifests/codex/reports")
        .join(TARGET_VERSION)
}
fn coverage_report_path(root: &Path) -> PathBuf {
    coverage_report_dir(root).join("coverage.any.json")
}
fn force_write_failure_emit_path(root: &Path, stem: &str) -> PathBuf {
    root.join("_ci_tmp/audit")
        .join(format!("{stem}.force-write-failure.json"))
}
fn union_snapshot_path(root: &Path) -> PathBuf {
    root.join("cli_manifests/codex/snapshots")
        .join(TARGET_VERSION)
        .join("union.json")
}

fn replace_named_coverage_reports(root: &Path, reports: &[(&str, &str)]) {
    fs::remove_file(coverage_report_path(root)).expect("remove preferred coverage report");
    for (name, contents) in reports {
        write_named_coverage_report(root, name, contents);
    }
}

fn write_named_coverage_report(root: &Path, file_name: &str, contents: &str) {
    write_text(&coverage_report_dir(root).join(file_name), contents);
}

fn rewrite_union_snapshot(root: &Path, mutate: impl FnOnce(&mut Value)) {
    let path = union_snapshot_path(root);
    let mut union = read_json_value(&path);
    mutate(&mut union);
    write_json_value(&path, &union);
}

fn union_expected_targets(root: &Path) -> Vec<String> {
    read_json_value(&union_snapshot_path(root))["expected_targets"]
        .as_array()
        .expect("union expected_targets array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("expected_targets entry should be a string")
                .to_string()
        })
        .collect()
}

fn replace_in_request(root: &Path, before: &str, after: &str) {
    let path = root.join(REQUEST_PATH);
    let original = fs::read_to_string(&path).expect("read generated request");
    let updated = original.replacen(before, after, 1);
    assert_ne!(
        updated, original,
        "expected generated request to contain the replacement marker `{before}`"
    );
    write_text(&path, &updated);
}

fn read_json_value(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("read json fixture"))
        .expect("parse json fixture")
}
fn write_json_value(path: &Path, value: &Value) {
    write_text(
        path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(value).expect("serialize json fixture")
        ),
    );
}

fn complete_union_snapshot(expected_targets: &[String]) -> String {
    format!("{}\n", serde_json::to_string_pretty(&json!({
        "expected_targets": expected_targets, "complete": true,
        "inputs": expected_targets.iter().map(|target| json!({ "target_triple": target })).collect::<Vec<_>>()
    })).expect("serialize union snapshot"))
}

fn clean_report(semantic_version: &str) -> String {
    coverage_report(semantic_version, false)
}
fn discovery_report(semantic_version: &str) -> String {
    coverage_report(semantic_version, true)
}

#[rustfmt::skip]
fn coverage_report(semantic_version: &str, includes_discovery: bool) -> String {
    let missing_commands = if includes_discovery { json!([{ "path": ["status"] }]) } else { json!([]) };
    format!("{}\n", serde_json::to_string_pretty(&json!({
        "schema_version": 1, "generated_at": "2026-05-05T15:00:00Z",
        "inputs": { "upstream": { "semantic_version": semantic_version, "mode": "union", "targets": ["x86_64-unknown-linux-musl"] }, "wrapper": { "schema_version": 1, "wrapper_version": "0.1.0" }, "rules": { "rules_schema_version": 1 } },
        "platform_filter": { "mode": "any" }, "deltas": { "missing_commands": missing_commands, "missing_flags": [], "missing_args": [], "intentionally_unsupported": [] }
    })).expect("serialize coverage report"))
}
