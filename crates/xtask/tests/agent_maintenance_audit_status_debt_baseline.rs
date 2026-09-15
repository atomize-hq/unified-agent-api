use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use xtask::agent_maintenance::{
    audit_status::{self, Args, AuditStatusOutcome, Error, EXIT_UPLIFTS_REQUIRED},
    contract_policy, prepare,
};
use xtask::agent_registry::AgentRegistry;

#[path = "support/onboard_agent_harness.rs"]
mod harness;

use harness::{fixture_root, write_text};

const SEEDED_REGISTRY: &str = include_str!("../data/agent_registry.toml");
const REQUEST_PATH: &str =
    "docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml";
const DEBT_PATH: &str = "docs/specs/unified-agent-api/non-tui-support-debt.md";
const TARGET_VERSION: &str = "0.98.0";

#[test]
fn a_debt_row_the_wrapper_covers_fails_the_gate_even_with_uplifts() {
    // Frozen before acquisition, as the nightly packet is, and frozen with the evidence present,
    // which also round-trips a rendered `unmatched_debt_surface` row through the request loader.
    for (prefix, freeze_with_evidence) in [
        ("audit-status-unmatched-debt-nightly", false),
        ("audit-status-unmatched-debt-refreshed", true),
    ] {
        let root = seeded_workspace(prefix, "requires_new_architectural_seam");
        if freeze_with_evidence {
            write_evidence(&root, json!([]));
        }
        freeze(&root);
        write_evidence(&root, json!([]));
        let frozen = std::fs::read_to_string(root.join(REQUEST_PATH)).expect("read request");
        assert_eq!(
            frozen.contains("[[support_surface_audit.unmatched_debt_surface]]"),
            freeze_with_evidence
        );

        let err = run_gate(&root).expect_err("an unmatched debt row must fail the gate");
        assert!(matches!(err, Error::Validation(_)), "unexpected: {err:?}");
        assert_eq!(err.exit_code(), 2);
        let message = err.to_string();
        for needle in [
            "surface_kind=flags command_path=codex exec surface_id=--legacy",
            "observation=covered_by_wrapper",
            "#codex-exec-legacy-flag",
        ] {
            assert!(message.contains(needle), "`{needle}` missing: {message}");
        }
    }
}

#[test]
fn uplifts_with_an_unchanged_debt_baseline_still_exit_three() {
    let root = seeded_workspace(
        "audit-status-debt-baseline-unchanged",
        "requires_new_architectural_seam",
    );
    freeze(&root);
    write_evidence(&root, json!([{"path": ["exec"], "key": "--legacy"}]));

    let (outcome, projection) = run_gate(&root).expect("matching debt keeps exit 3");
    assert_eq!(outcome, AuditStatusOutcome::UpliftsRequired);
    assert_eq!(outcome.exit_code(), EXIT_UPLIFTS_REQUIRED);
    // The request was frozen before acquisition, so the uplift alone drifts it.
    assert_eq!(projection["reconciliation"], json!("drifted"));
    assert_eq!(projection["preexisting_unsupported_surface"], json!(1));
}

#[test]
fn a_debt_baseline_changed_since_the_freeze_fails_the_gate_even_with_uplifts() {
    let root = seeded_workspace(
        "audit-status-debt-baseline-changed",
        "requires_new_architectural_seam",
    );
    freeze(&root);
    write_evidence(&root, json!([{"path": ["exec"], "key": "--legacy"}]));
    write_text(&root.join(DEBT_PATH), &debt_inventory("requires_new_infra"));

    let err = run_gate(&root).expect_err("a changed debt baseline must fail the gate");
    assert_eq!(err.exit_code(), 2);
    let message = err.to_string();
    assert!(
        message.contains("`support_surface_audit.deferred_preexisting_gaps` no longer matches"),
        "unexpected: {message}"
    );
}

/// A codex workspace whose debt inventory holds one row, `codex exec --legacy`.
#[rustfmt::skip]
fn seeded_workspace(prefix: &str, blocker_class: &str) -> PathBuf {
    let root = fixture_root(prefix);
    let registry = AgentRegistry::parse(SEEDED_REGISTRY).expect("parse registry");
    let entry = registry.find("codex").expect("codex entry");
    let debt = debt_inventory(blocker_class);
    for (path, contents) in [
        ("crates/xtask/data/agent_registry.toml", SEEDED_REGISTRY),
        (".github/workflows/agent-maintenance-open-pr.yml", "name: Packet PR worker\n"),
        ("cli_manifests/codex/PR_BODY_TEMPLATE.md", "@codex\n\nFollow the PR template for {{VERSION}}.\n"),
        ("cli_manifests/codex/OPS_PLAYBOOK.md", "# Codex ops\n"),
        ("cli_manifests/codex/CI_WORKFLOWS_PLAN.md", "# Codex CI workflows\n"),
        ("docs/agents/lifecycle/codex-maintenance/OPS_PLAYBOOK.md", "# Packet ops\n"),
        ("docs/agents/lifecycle/codex-maintenance/CI_WORKFLOWS_PLAN.md", "# Packet workflow plan\n"),
        ("cli_manifests/codex/latest_validated.txt", "0.97.0\n"),
        (DEBT_PATH, debt.as_str()),
    ] {
        write_text(&root.join(path), contents);
    }
    write_text(
        &root.join("docs/agents/lifecycle/codex-maintenance/governance/execute-agent-maintenance-prompt.md"),
        &contract_policy::packet_pr_prompt_template(entry, "docs/agents/lifecycle/codex-maintenance"),
    );
    root
}

#[rustfmt::skip]
fn debt_inventory(blocker_class: &str) -> String {
    format!(
        concat!(
            "# Non-TUI Support Debt Inventory\n\n## Inventory\n\n### `codex-exec-legacy-flag`\n\n",
            "- `agent_id`: `codex`\n- `surface_kind`: `flags`\n- `command_path`: `codex exec`\n",
            "- `surface_id`: `--legacy`\n- `current_reason`: `The legacy flag stays outside the seam.`\n",
            "- `blocker_class`: `{}`\n- `owner`: `wrappers team`\n- `milestone`: `test`\n",
            "- `follow_on`: `TODOS.md#close-codex-legacy-gap`\n",
            "- `evidence_ref`: `cli_manifests/codex/reports/0.97.0/coverage.any.json`\n",
        ),
        blocker_class
    )
}

#[rustfmt::skip]
fn freeze(root: &Path) {
    let args = prepare::Args {
        agent: "codex".to_string(), current_version: "0.97.0".to_string(), latest_stable: "0.99.0".to_string(),
        target_version: TARGET_VERSION.to_string(), opened_from: PathBuf::from(".github/workflows/agent-maintenance-open-pr.yml"),
        detected_by: ".github/workflows/agent-maintenance-release-watch.yml".to_string(), dispatch_kind: "packet_pr".to_string(),
        dispatch_workflow: None, branch_name: "automation/codex-maintenance-0.98.0".to_string(),
        request_recorded_at: "2026-09-15T08:37:00Z".to_string(), request_commit: "abcdef1".to_string(), dry_run: true, write: false,
    };
    let plan = prepare::build_prepare_plan(root, &args).expect("build prepare plan");
    prepare::apply_prepare_plan(root, &plan).expect("apply prepare plan");
}

/// Writes acquisition evidence: a complete union that lists `codex status` and `codex exec
/// --legacy`, and a coverage report whose gaps are `codex status` plus `missing_flags`.
#[rustfmt::skip]
fn write_evidence(root: &Path, missing_flags: Value) {
    let registry = AgentRegistry::parse(SEEDED_REGISTRY).expect("parse registry");
    let targets = &registry.find("codex").expect("codex entry").canonical_targets;
    let union = json!({
        "expected_targets": targets, "complete": true,
        "inputs": targets.iter().map(|target| json!({"target_triple": target})).collect::<Vec<_>>(),
        "commands": [{"path": ["exec"], "flags": [{"key": "--legacy"}]}, {"path": ["status"]}],
    });
    let report = json!({
        "schema_version": 1, "generated_at": "2026-09-15T08:37:00Z",
        "inputs": {"upstream": {"semantic_version": TARGET_VERSION, "mode": "union", "targets": targets}},
        "platform_filter": {"mode": "any"},
        "deltas": {"missing_commands": [{"path": ["status"]}], "missing_flags": missing_flags, "missing_args": []},
    });
    let manifest_root = root.join("cli_manifests/codex");
    write_text(&manifest_root.join("snapshots").join(TARGET_VERSION).join("union.json"), &union.to_string());
    write_text(&manifest_root.join("reports").join(TARGET_VERSION).join("coverage.any.json"), &report.to_string());
}

fn run_gate(root: &Path) -> Result<(AuditStatusOutcome, Value), Error> {
    let args = Args {
        request: PathBuf::from(REQUEST_PATH),
        expect_target_version: None,
        emit_json: None,
        workspace_root: None,
    };
    let mut stdout = Vec::new();
    let outcome = audit_status::run_in_workspace(root, args, &mut stdout)?;
    let projection = serde_json::from_slice(&stdout).expect("parse audit status projection");
    Ok((outcome, projection))
}
