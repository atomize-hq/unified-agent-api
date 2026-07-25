#![allow(dead_code, unused_imports)]

use std::{
    fs,
    path::{Path, PathBuf},
};

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
const CLEAN_REPORT: &str = concat!(
    "{\n",
    "  \"deltas\": {\n",
    "    \"missing_commands\": [],\n",
    "    \"missing_flags\": [],\n",
    "    \"missing_args\": [],\n",
    "    \"intentionally_unsupported\": []\n",
    "  }\n",
    "}\n"
);
const DISCOVERY_REPORT: &str = concat!(
    "{\n",
    "  \"deltas\": {\n",
    "    \"missing_commands\": [\n",
    "      {\n",
    "        \"path\": [\"status\"]\n",
    "      }\n",
    "    ],\n",
    "    \"missing_flags\": [],\n",
    "    \"missing_args\": [],\n",
    "    \"intentionally_unsupported\": []\n",
    "  }\n",
    "}\n"
);

#[test]
fn empty_required_uplifts_reports_clean_exit_code_and_false_flag() {
    let fixture = prepared_fixture("agent-maintenance-audit-status-clean", CLEAN_REPORT);

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
    let fixture = prepared_fixture("agent-maintenance-audit-status-uplifts", DISCOVERY_REPORT);

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
    assert_eq!(
        json["required_uplifts"],
        json!([{
            "surface_kind": "commands",
            "command_path": "codex status",
            "surface_id": "status",
            "reason": "new_upstream_surface",
            "required_writes": [
                "backend",
                "manifest",
                "packet_docs",
                "publication",
                "wrapper"
            ]
        }])
    );
}

#[test]
fn frozen_clean_packet_live_dirty_returns_exit_three_and_drifted_reconciliation() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-clean-frozen-live-dirty",
        CLEAN_REPORT,
    );
    write_text(&coverage_report_path(&fixture), DISCOVERY_REPORT);

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
    assert_eq!(
        json["required_uplifts"],
        json!([{
            "surface_kind": "commands",
            "command_path": "codex status",
            "surface_id": "status",
            "reason": "new_upstream_surface",
            "required_writes": [
                "backend",
                "manifest",
                "packet_docs",
                "publication",
                "wrapper"
            ]
        }])
    );
}

#[test]
fn missing_live_coverage_report_is_validation_error_not_clean() {
    let fixture = prepared_fixture(
        "agent-maintenance-audit-status-missing-coverage",
        CLEAN_REPORT,
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
        CLEAN_REPORT,
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
        CLEAN_REPORT,
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
        DISCOVERY_REPORT,
    );
    let emit_path = fixture.join("_ci_tmp/audit/status.json");

    let mut stdout = Vec::new();
    let first = audit_status::run_in_workspace(
        &fixture,
        audit_args(REQUEST_PATH, Some(emit_path.clone())),
        &mut stdout,
    )
    .expect("first emit");
    assert_eq!(first.exit_code(), EXIT_UPLIFTS_REQUIRED);
    assert!(stdout.is_empty(), "emit-json should not write stdout");
    let first_bytes = fs::read(&emit_path).expect("read first emitted json");

    let mut stdout = Vec::new();
    let second = audit_status::run_in_workspace(
        &fixture,
        audit_args(REQUEST_PATH, Some(emit_path.clone())),
        &mut stdout,
    )
    .expect("second emit");
    assert_eq!(second.exit_code(), EXIT_UPLIFTS_REQUIRED);
    assert!(stdout.is_empty(), "emit-json should not write stdout");
    let second_bytes = fs::read(&emit_path).expect("read second emitted json");

    assert_eq!(first_bytes, second_bytes);
    assert!(
        first_bytes.ends_with(b"\n"),
        "emitted json should preserve the trailing newline contract"
    );
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("parse audit status json")
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
        target_version: "0.98.0".to_string(),
        opened_from: Path::new(".github/workflows/agent-maintenance-open-pr.yml").to_path_buf(),
        detected_by: ".github/workflows/agent-maintenance-release-watch.yml".to_string(),
        dispatch_kind: "packet_pr".to_string(),
        dispatch_workflow: None,
        branch_name: "automation/codex-maintenance-0.98.0".to_string(),
        request_recorded_at: "2026-05-05T15:00:00Z".to_string(),
        request_commit: "abcdef1".to_string(),
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

    write_text(
        &root.join(".github/workflows/agent-maintenance-open-pr.yml"),
        "name: Packet PR worker\n",
    );
    write_text(
        &root.join("cli_manifests/codex/PR_BODY_TEMPLATE.md"),
        "@codex\n\n## Goal\n\nFollow the maintained PR template for {{VERSION}}.\n",
    );
    write_text(
        &root.join("cli_manifests/codex/OPS_PLAYBOOK.md"),
        "# Codex ops\n",
    );
    write_text(
        &root.join("cli_manifests/codex/CI_WORKFLOWS_PLAN.md"),
        "# Codex CI workflows\n",
    );
    write_text(
        &root.join(
            "docs/agents/lifecycle/codex-maintenance/governance/execute-agent-maintenance-prompt.md",
        ),
        &contract_policy::packet_pr_prompt_template(
            entry,
            "docs/agents/lifecycle/codex-maintenance",
        ),
    );
    write_text(
        &root.join("docs/agents/lifecycle/codex-maintenance/OPS_PLAYBOOK.md"),
        "# Packet ops\n",
    );
    write_text(
        &root.join("docs/agents/lifecycle/codex-maintenance/CI_WORKFLOWS_PLAN.md"),
        "# Packet workflow plan\n",
    );
    write_text(
        &root.join("cli_manifests/codex/latest_validated.txt"),
        "0.97.0\n",
    );
    write_text(&coverage_report_path(root), coverage_report);
    write_text(
        &root.join("docs/specs/unified-agent-api/non-tui-support-debt.md"),
        "# Non-TUI Support Debt Inventory\n\n## Inventory\n",
    );
}

fn coverage_report_dir(root: &Path) -> PathBuf {
    root.join("cli_manifests/codex/reports/0.98.0")
}

fn coverage_report_path(root: &Path) -> PathBuf {
    coverage_report_dir(root).join("coverage.any.json")
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
