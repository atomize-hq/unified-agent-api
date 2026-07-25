#![allow(dead_code, unused_imports)]

use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use serde_json::{json, Value};

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
    Args as AuditStatusArgs, AuditStatusOutcome, Error as AuditStatusError, EXIT_UPLIFTS_REQUIRED,
};
use harness::{fixture_root, write_text};
use prepare::{apply_prepare_plan, build_prepare_plan, Args as PrepareArgs};

const SEEDED_REGISTRY: &str = include_str!("../data/agent_registry.toml");
const REQUEST_PATH: &str =
    "docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml";
const TARGET_VERSION: &str = "0.98.0";
const REQUEST_COMMIT: &str = "abcdef1";

#[test]
fn empty_required_uplifts_reports_clean_exit_code_and_false_flag() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-clean",
        &clean_report(TARGET_VERSION),
    );

    let mut stdout = Vec::new();
    let outcome =
        audit_status::run_in_workspace(&fixture, audit_args(REQUEST_PATH, None), &mut stdout)
            .expect("clean audit status");

    assert_eq!(outcome, AuditStatusOutcome::Clean);
    assert_eq!(outcome.exit_code(), 0);

    let json = parse_json(&stdout);
    assert_eq!(json["agent_id"], json!("codex"));
    assert_eq!(json["target_version"], json!("0.98.0"));
    assert_eq!(json["uplifts_required"], json!(false));
    assert_eq!(json["required_uplifts"], json!([]));
    assert_eq!(json["reconciliation"], json!("exact"));
    assert_eq!(json["discovered_upstream_surface"], json!(0));
    assert_eq!(json["preexisting_unsupported_surface"], json!(0));
    assert_eq!(json["missing_wrapper_support"], json!(0));
    assert_eq!(json["missing_backend_support"], json!(0));
}

#[test]
fn non_empty_required_uplifts_reports_exit_three_and_true_flag() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-uplifts",
        &discovery_report(TARGET_VERSION),
    );

    let mut stdout = Vec::new();
    let outcome =
        audit_status::run_in_workspace(&fixture, audit_args(REQUEST_PATH, None), &mut stdout)
            .expect("uplift audit status");

    assert_eq!(outcome, AuditStatusOutcome::UpliftsRequired);
    assert_eq!(outcome.exit_code(), EXIT_UPLIFTS_REQUIRED);

    let json = parse_json(&stdout);
    assert_eq!(json["uplifts_required"], json!(true));
    assert_eq!(json["reconciliation"], json!("exact"));
    assert_eq!(json["discovered_upstream_surface"], json!(1));
    assert_eq!(json["preexisting_unsupported_surface"], json!(0));
    assert_eq!(json["missing_wrapper_support"], json!(1));
    assert_eq!(json["missing_backend_support"], json!(1));
    assert_eq!(json["required_uplifts"], expected_required_uplifts_json());
}

#[test]
fn frozen_clean_packet_live_dirty_returns_exit_three_and_drifted_reconciliation() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-clean-frozen-live-dirty",
        &clean_report(TARGET_VERSION),
    );
    write_text(
        &coverage_report_path(&fixture),
        &discovery_report(TARGET_VERSION),
    );

    let mut stdout = Vec::new();
    let outcome =
        audit_status::run_in_workspace(&fixture, audit_args(REQUEST_PATH, None), &mut stdout)
            .expect("frozen-clean/live-dirty audit status");

    assert_eq!(outcome, AuditStatusOutcome::UpliftsRequired);
    assert_eq!(outcome.exit_code(), EXIT_UPLIFTS_REQUIRED);

    let json = parse_json(&stdout);
    assert_eq!(json["uplifts_required"], json!(true));
    assert_eq!(json["reconciliation"], json!("drifted"));
    assert_eq!(json["discovered_upstream_surface"], json!(1));
    assert_eq!(json["preexisting_unsupported_surface"], json!(0));
    assert_eq!(json["missing_wrapper_support"], json!(1));
    assert_eq!(json["missing_backend_support"], json!(1));
    assert_eq!(json["required_uplifts"], expected_required_uplifts_json());
}

#[test]
fn missing_live_coverage_report_is_validation_error_not_clean() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-missing-coverage",
        &clean_report(TARGET_VERSION),
    );
    fs::remove_dir_all(coverage_report_dir(&fixture)).expect("remove seeded coverage report dir");

    let mut stdout = Vec::new();
    let err = audit_status::run_in_workspace(&fixture, audit_args(REQUEST_PATH, None), &mut stdout)
        .expect_err("missing live coverage report must fail");

    assert!(matches!(err, AuditStatusError::Validation(_)));
    assert_eq!(err.exit_code(), 2);
    assert!(
        stdout.is_empty(),
        "validation failures must not emit a clean projection"
    );
    assert!(
        err.to_string()
            .contains("cli_manifests/codex/reports/0.98.0"),
        "error should name the expected target-version report directory"
    );
}

#[test]
fn drifted_reconciliation_without_uplifts_is_validation_error() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-drifted-reconciliation",
        &clean_report(TARGET_VERSION),
    );
    replace_in_request(&fixture, "pre_run_debt_count = 0", "pre_run_debt_count = 1");

    let err =
        audit_status::run_in_workspace(&fixture, audit_args(REQUEST_PATH, None), &mut Vec::new())
            .expect_err("drifted reconciliation without uplifts must fail");

    assert!(matches!(err, AuditStatusError::Validation(_)));
    assert_eq!(err.exit_code(), 2);
    assert!(
        err.to_string().contains("support_surface_audit"),
        "drifted reconciliation should surface the packet inconsistency"
    );
}

#[test]
fn failing_emit_json_run_does_not_preserve_stale_projection() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-stale-emit-json",
        &clean_report(TARGET_VERSION),
    );
    let emit_path = fixture.join("_ci_tmp/audit/status.json");

    let outcome = audit_status::run_in_workspace(
        &fixture,
        audit_args(REQUEST_PATH, Some(emit_path.clone())),
        &mut Vec::new(),
    )
    .expect("first emit");
    assert_eq!(outcome, AuditStatusOutcome::Clean);
    assert!(
        emit_path.is_file(),
        "successful emit-json runs should materialize the projection"
    );

    replace_in_request(&fixture, "pre_run_debt_count = 0", "pre_run_debt_count = 1");

    let err = audit_status::run_in_workspace(
        &fixture,
        audit_args(REQUEST_PATH, Some(emit_path.clone())),
        &mut Vec::new(),
    )
    .expect_err("second run must fail after request drift");
    assert!(matches!(err, AuditStatusError::Validation(_)));
    assert!(
        !emit_path.exists(),
        "failing emit-json runs must delete the stale projection before deriving"
    );
}

#[test]
fn malformed_or_unresolvable_request_is_validation_error_not_exit_three() {
    let fixture = fixture_root("agent-maintenance-audit-status-invalid-request");
    seed_registry(&fixture);

    let err = audit_status::run_in_workspace(
        &fixture,
        audit_args(
            "docs/agents/lifecycle/codex-maintenance/governance/missing-request.toml",
            None,
        ),
        &mut Vec::new(),
    )
    .expect_err("missing request must fail");

    assert!(matches!(err, AuditStatusError::Validation(_)));
    assert_eq!(err.exit_code(), 2);
    assert_ne!(err.exit_code(), EXIT_UPLIFTS_REQUIRED);
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

#[test]
fn drifted_packet_with_invalid_request_commit_is_validation_error_not_exit_three() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-drifted-invalid-request-commit",
        &clean_report(TARGET_VERSION),
    );
    write_text(
        &coverage_report_path(&fixture),
        &discovery_report(TARGET_VERSION),
    );
    replace_in_request(
        &fixture,
        &format!("request_commit = \"{REQUEST_COMMIT}\""),
        "request_commit = \"NOT A COMMIT AT ALL\"",
    );

    let err =
        audit_status::run_in_workspace(&fixture, audit_args(REQUEST_PATH, None), &mut Vec::new())
            .expect_err("invalid post-reconciliation fields must fail validation");

    assert!(matches!(err, AuditStatusError::Validation(_)));
    assert_eq!(err.exit_code(), 2);
    assert_ne!(err.exit_code(), EXIT_UPLIFTS_REQUIRED);
    assert!(
        err.to_string().contains("request_commit"),
        "post-reconciliation validation failures should name the invalid field"
    );
}

#[test]
fn wrong_version_live_coverage_report_is_rejected() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-wrong-version-report",
        &clean_report("0.97.0"),
    );

    let err =
        audit_status::run_in_workspace(&fixture, audit_args(REQUEST_PATH, None), &mut Vec::new())
            .expect_err("wrong-version coverage evidence must fail");

    assert!(matches!(err, AuditStatusError::Validation(_)));
    assert_eq!(err.exit_code(), 2);
    assert_ne!(err.exit_code(), 0);
    assert!(
        err.to_string().contains(TARGET_VERSION),
        "error should name the detected release target version"
    );
    assert!(
        err.to_string().contains("0.97.0"),
        "error should name the mismatched evidence version"
    );
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

    assert!(matches!(err, AuditStatusError::Validation(_)));
    assert_eq!(err.exit_code(), 2);
    assert!(stdout.is_empty());
    assert!(err.to_string().contains("coverage.linux-arm64.json"));
    assert!(err.to_string().contains(TARGET_VERSION));
    assert!(err.to_string().contains("0.97.0"));
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

    assert_eq!(live_audit.discovered_upstream_surface.len(), 1);
    assert_eq!(
        live_audit.discovered_upstream_surface[0].evidence_ref,
        validator_selected
            .strip_prefix(&fixture)
            .expect("repo-relative evidence path")
            .to_string_lossy()
            .to_string()
    );
}

#[test]
fn failing_emit_json_pre_derivation_preserves_existing_projection() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-preserve-emit-json",
        &clean_report(TARGET_VERSION),
    );
    let emit_path = fixture.join("_ci_tmp/audit/status.json");
    let original = "{\n  \"stale\": true\n}\n";
    write_text(&emit_path, original);

    replace_in_request(
        &fixture,
        "agent_id = \"codex\"",
        "agent_id = \"unknown_agent\"",
    );

    let err = audit_status::run_in_workspace(
        &fixture,
        audit_args(REQUEST_PATH, Some(emit_path.clone())),
        &mut Vec::new(),
    )
    .expect_err("pre-derivation validation failures must keep the existing projection");

    assert!(matches!(err, AuditStatusError::Validation(_)));
    assert_eq!(err.exit_code(), 2);
    assert_eq!(
        fs::read_to_string(&emit_path).expect("read preserved emit-json target"),
        original
    );
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
#[test]
fn failing_emit_json_cleanup_error_preserves_original_failure_and_exit_code() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-emit-json-cleanup-failure",
        &clean_report(TARGET_VERSION),
    );
    let emit_path = fixture.join("_ci_tmp/audit/status.json");

    let (first, _) = emit_projection(&fixture, &emit_path);
    assert_eq!(first, AuditStatusOutcome::Clean);

    write_text(&coverage_report_path(&fixture), &clean_report("0.97.0"));

    let emit_parent = emit_path.parent().expect("emit parent");
    let original_permissions = fs::metadata(emit_parent)
        .expect("stat emit parent")
        .permissions();
    let mut blocked_permissions = original_permissions.clone();
    blocked_permissions.set_mode(0o555);
    fs::set_permissions(emit_parent, blocked_permissions).expect("block emit cleanup");

    let err = audit_status::run_in_workspace(
        &fixture,
        audit_args(REQUEST_PATH, Some(emit_path.clone())),
        &mut Vec::new(),
    )
    .expect_err("cleanup failure must preserve original validation error");

    fs::set_permissions(emit_parent, original_permissions).expect("restore emit parent perms");

    assert!(matches!(err, AuditStatusError::Validation(_)));
    assert_eq!(err.exit_code(), 2);
    assert!(
        err.to_string().contains(TARGET_VERSION),
        "original error should still name the expected version"
    );
    assert!(
        err.to_string().contains("0.97.0"),
        "original error should still name the stale declared version"
    );
    assert!(
        !err.to_string()
            .contains("remove stale maintenance audit status projection"),
        "cleanup failures must not mask the original validation error"
    );
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
        "reason": "new_upstream_surface",
        "required_writes": ["backend", "manifest", "packet_docs", "publication", "wrapper"]
    }])
}

fn emit_projection(root: &Path, emit_path: &Path) -> (AuditStatusOutcome, Vec<u8>) {
    let mut stdout = Vec::new();
    let outcome = audit_status::run_in_workspace(
        root,
        audit_args(REQUEST_PATH, Some(emit_path.to_path_buf())),
        &mut stdout,
    )
    .expect("emit projection");
    assert!(stdout.is_empty(), "emit-json should not write stdout");
    (
        outcome,
        fs::read(emit_path).expect("read emitted projection"),
    )
}

fn live_audit(root: &Path) -> support_audit::SupportSurfaceAudit {
    let validated_request = request::load_request_envelope_validated_with_policy(
        root,
        Path::new(REQUEST_PATH),
        request::AuditDriftPolicy::Tolerate,
    )
    .expect("load validated request");
    let registry = agent_registry::AgentRegistry::load(root).expect("load registry");
    let detected_release = validated_request
        .envelope
        .request
        .detected_release
        .as_ref()
        .expect("detected release");
    support_audit::derive_support_surface_audit(
        root,
        registry.find("codex").expect("codex entry"),
        detected_release,
    )
    .expect("derive live audit")
}

fn request_sha256(root: &Path) -> String {
    request::load_request_envelope_validated(root, Path::new(REQUEST_PATH))
        .expect("load validated request")
        .envelope
        .request
        .sha256
}

fn audit_args(request: &str, emit_json: Option<PathBuf>) -> AuditStatusArgs {
    AuditStatusArgs {
        request: PathBuf::from(request),
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

fn prepare_args() -> PrepareArgs {
    PrepareArgs {
        agent: "codex".to_string(),
        current_version: "0.97.0".to_string(),
        latest_stable: "0.99.0".to_string(),
        target_version: TARGET_VERSION.to_string(),
        opened_from: Path::new(".github/workflows/agent-maintenance-open-pr.yml").to_path_buf(),
        detected_by: ".github/workflows/agent-maintenance-release-watch.yml".to_string(),
        dispatch_kind: "packet_pr".to_string(),
        dispatch_workflow: None,
        branch_name: "automation/codex-maintenance-0.98.0".to_string(),
        request_recorded_at: "2026-05-05T15:00:00Z".to_string(),
        request_commit: REQUEST_COMMIT.to_string(),
        dry_run: true,
        write: false,
    }
}

fn seed_registry(root: &Path) {
    write_text(
        &root.join("crates/xtask/data/agent_registry.toml"),
        SEEDED_REGISTRY,
    );
}

fn seed_support_files(root: &Path, coverage_report: &str) {
    let registry = agent_registry::AgentRegistry::parse(SEEDED_REGISTRY).expect("parse registry");
    let entry = registry.find("codex").expect("codex entry");

    for (path, contents) in [
        (
            ".github/workflows/agent-maintenance-open-pr.yml",
            "name: Packet PR worker\n",
        ),
        (
            "cli_manifests/codex/PR_BODY_TEMPLATE.md",
            "@codex\n\n## Goal\n\nFollow the maintained PR template for {{VERSION}}.\n",
        ),
        ("cli_manifests/codex/OPS_PLAYBOOK.md", "# Codex ops\n"),
        (
            "cli_manifests/codex/CI_WORKFLOWS_PLAN.md",
            "# Codex CI workflows\n",
        ),
        (
            "docs/agents/lifecycle/codex-maintenance/OPS_PLAYBOOK.md",
            "# Packet ops\n",
        ),
        (
            "docs/agents/lifecycle/codex-maintenance/CI_WORKFLOWS_PLAN.md",
            "# Packet workflow plan\n",
        ),
        ("cli_manifests/codex/latest_validated.txt", "0.97.0\n"),
        (
            "docs/specs/unified-agent-api/non-tui-support-debt.md",
            "# Non-TUI Support Debt Inventory\n\n## Inventory\n",
        ),
    ] {
        write_text(&root.join(path), contents);
    }
    write_text(
        &root.join(
            "docs/agents/lifecycle/codex-maintenance/governance/execute-agent-maintenance-prompt.md",
        ),
        &contract_policy::packet_pr_prompt_template(
            entry,
            "docs/agents/lifecycle/codex-maintenance",
        ),
    );
    write_text(&coverage_report_path(root), coverage_report);
}

fn coverage_report_dir(root: &Path) -> PathBuf {
    root.join("cli_manifests/codex/reports")
        .join(TARGET_VERSION)
}

fn coverage_report_path(root: &Path) -> PathBuf {
    coverage_report_dir(root).join("coverage.any.json")
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

#[cfg(unix)]
fn run_with_unreadable_evidence(root: &Path) -> AuditStatusError {
    let report_dir = coverage_report_dir(root);
    let original_permissions = fs::metadata(&report_dir)
        .expect("stat report dir")
        .permissions();
    let mut blocked_permissions = original_permissions.clone();
    blocked_permissions.set_mode(0o000);
    fs::set_permissions(&report_dir, blocked_permissions).expect("block coverage report dir");

    let err = audit_status::run_in_workspace(root, audit_args(REQUEST_PATH, None), &mut Vec::new())
        .expect_err("unreadable evidence must fail");

    fs::set_permissions(&report_dir, original_permissions).expect("restore report dir perms");
    err
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

fn clean_report(semantic_version: &str) -> String {
    coverage_report(semantic_version, false)
}

fn discovery_report(semantic_version: &str) -> String {
    coverage_report(semantic_version, true)
}

fn coverage_report(semantic_version: &str, includes_discovery: bool) -> String {
    let missing_commands = if includes_discovery {
        json!([{ "path": ["status"] }])
    } else {
        json!([])
    };
    format!(
        "{}\n",
        serde_json::to_string_pretty(&json!({
            "schema_version": 1,
            "generated_at": "2026-05-05T15:00:00Z",
            "inputs": {
                "upstream": {
                    "semantic_version": semantic_version,
                    "mode": "union",
                    "targets": ["x86_64-unknown-linux-musl"]
                },
                "wrapper": { "schema_version": 1, "wrapper_version": "0.1.0" },
                "rules": { "rules_schema_version": 1 }
            },
            "platform_filter": { "mode": "any" },
            "deltas": {
                "missing_commands": missing_commands,
                "missing_flags": [],
                "missing_args": [],
                "intentionally_unsupported": []
            }
        }))
        .expect("serialize coverage report")
    )
}
