use std::{collections::BTreeMap, process::Command};

use clap::Parser;
use serde_json::json;

use super::*;

const REQUEST_PATH: &str =
    "docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml";

#[test]
fn from_request_refreezes_against_live_reports_and_is_idempotent() {
    let fixture = fixture_root("prepare-agent-maintenance-from-request");
    seed_registry(&fixture);
    seed_support_files(&fixture);
    fs::remove_dir_all(fixture.join("cli_manifests/codex/reports/0.98.0"))
        .expect("remove target report");
    seed_debt(&fixture, false);

    let initial = build_prepare_plan(&fixture, &args()).expect("build placeholder plan");
    apply_prepare_plan(&fixture, &initial).expect("freeze placeholder");
    let frozen = initial
        .request
        .support_surface_audit
        .as_ref()
        .expect("placeholder audit");
    assert_eq!(frozen.required_uplifts_this_run.len(), 2);
    assert!(frozen
        .unbaselined_gap_surface
        .iter()
        .all(|row| row.evidence_ref == support_audit::NON_TUI_SUPPORT_DEBT_PATH));

    write_target_reports(&fixture, &["alpha", "zeta", "new-surface"]);
    let drift = request::load_request_envelope_validated(&fixture, Path::new(REQUEST_PATH))
        .expect_err("post-acquisition placeholder must drift");
    let drift = drift.to_string();
    assert!(
        drift.contains("no longer matches the live derived maintenance contract"),
        "unexpected strict-load refusal: {drift}"
    );

    let recorded_at = initial.request.request_recorded_at.clone();
    let request_commit = initial.request.request_commit.clone();
    let refreeze_args =
        prepare::args_from_request_in_workspace(&fixture, Path::new(REQUEST_PATH), false, true)
            .expect("recover recorded inputs");
    let mut first_output = Vec::new();
    prepare::run_in_workspace(&fixture, refreeze_args, &mut first_output).expect("re-freeze");

    let validated = request::load_request_envelope_validated(&fixture, Path::new(REQUEST_PATH))
        .expect("strict load after re-freeze");
    assert_eq!(
        validated.support_surface_audit_reconciliation,
        Some(request::AuditReconciliation::Exact)
    );
    assert_eq!(validated.envelope.request.request_recorded_at, recorded_at);
    assert_eq!(validated.envelope.request.request_commit, request_commit);
    let audit = validated
        .envelope
        .request
        .support_surface_audit
        .expect("support audit");
    let report_ref = "cli_manifests/codex/reports/0.98.0/coverage.any.json";
    assert!(audit.required_uplifts_this_run.iter().all(|uplift| audit
        .unbaselined_gap_surface
        .iter()
        .any(|row| row.identity() == uplift.identity() && row.evidence_ref == report_ref)));

    let before = snapshot_plan_files(&fixture, &initial);
    let second_args =
        prepare::args_from_request_in_workspace(&fixture, Path::new(REQUEST_PATH), false, true)
            .expect("recover inputs again");
    let mut second_output = Vec::new();
    prepare::run_in_workspace(&fixture, second_args, &mut second_output)
        .expect("idempotent re-freeze");
    assert_eq!(snapshot_plan_files(&fixture, &initial), before);
    let output = String::from_utf8(second_output).expect("utf8 output");
    assert!(output.contains(&format!(
        "applied {} files (written 0, identical {})",
        initial.files.len(),
        initial.files.len()
    )));
}

#[test]
fn from_request_refuses_wrong_trigger_missing_field_and_wrong_path() {
    for case in ["trigger", "field", "path"] {
        let fixture = fixture_root(&format!("prepare-agent-maintenance-refusal-{case}"));
        seed_registry(&fixture);
        seed_support_files(&fixture);
        let plan = build_prepare_plan(&fixture, &args()).expect("build request");
        apply_prepare_plan(&fixture, &plan).expect("write request");
        let request_path = fixture.join(REQUEST_PATH);
        let mut relative = Path::new(REQUEST_PATH).to_path_buf();
        let expected_field = match case {
            "trigger" => {
                let text = fs::read_to_string(&request_path)
                    .expect("read request")
                    .replace(
                        "trigger_kind = \"upstream_release_detected\"",
                        "trigger_kind = \"manual\"",
                    );
                write_text(&request_path, &text);
                "trigger_kind"
            }
            "field" => {
                let text = fs::read_to_string(&request_path)
                    .expect("read request")
                    .replace("target_version = \"0.98.0\"\n", "");
                write_text(&request_path, &text);
                "detected_release.target_version"
            }
            "path" => {
                relative = Path::new("wrong/request.toml").to_path_buf();
                write_text(
                    &fixture.join(&relative),
                    &fs::read_to_string(&request_path).expect("read request"),
                );
                "from_request"
            }
            _ => unreachable!(),
        };
        let error = prepare::args_from_request_in_workspace(&fixture, &relative, true, false)
            .expect_err(case);
        let message = error.to_string();
        assert!(message.contains(expected_field), "{case}: {message}");
        assert!(
            message.contains(&relative.display().to_string()),
            "{case}: {message}"
        );
    }
}

#[test]
fn explicit_cli_conversion_preserves_every_field() {
    let parsed = prepare::Cli::try_parse_from([
        "prepare-agent-maintenance",
        "--agent",
        "agent-value",
        "--current-version",
        "current-version-value",
        "--latest-stable",
        "latest-stable-value",
        "--target-version",
        "target-version-value",
        "--opened-from",
        "opened/from/value.yml",
        "--detected-by",
        "detected-by-value",
        "--dispatch-kind",
        "dispatch-kind-value",
        "--dispatch-workflow",
        "dispatch-workflow-value",
        "--branch-name",
        "branch-name-value",
        "--request-recorded-at",
        "request-recorded-at-value",
        "--request-commit",
        "request-commit-value",
        "--dry-run",
    ])
    .expect("parse explicit CLI")
    .into_args()
    .expect("convert explicit CLI");

    assert_eq!(
        parsed,
        Args {
            agent: "agent-value".to_string(),
            current_version: "current-version-value".to_string(),
            latest_stable: "latest-stable-value".to_string(),
            target_version: "target-version-value".to_string(),
            opened_from: Path::new("opened/from/value.yml").to_path_buf(),
            detected_by: "detected-by-value".to_string(),
            dispatch_kind: "dispatch-kind-value".to_string(),
            dispatch_workflow: Some("dispatch-workflow-value".to_string()),
            branch_name: "branch-name-value".to_string(),
            request_recorded_at: "request-recorded-at-value".to_string(),
            request_commit: "request-commit-value".to_string(),
            dry_run: true,
            write: false,
        }
    );
}

#[test]
fn explicit_cli_matches_the_existing_plan_and_invalid_modes_are_rejected() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root");
    let mut explicit = args();
    explicit.target_version = "999.0.0".to_string();
    explicit.latest_stable = "999.0.1".to_string();
    let plan = build_prepare_plan(root, &explicit).expect("existing explicit plan");
    let expected = format!(
        "request: {}\n{}dry_run: true\n",
        plan.request.relative_path,
        plan.planned_paths()
            .into_iter()
            .map(|path| format!("planned: {path}\n"))
            .collect::<String>()
    );
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(explicit_cli_args())
        .output()
        .expect("run xtask");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("utf8 stdout"),
        expected
    );

    let mixed = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args([
            "prepare-agent-maintenance",
            "--from-request",
            REQUEST_PATH,
            "--agent",
            "codex",
            "--dry-run",
        ])
        .output()
        .expect("run mixed CLI");
    assert_eq!(mixed.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&mixed.stderr).contains("cannot be used with"));

    let missing_mode = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["prepare-agent-maintenance", "--from-request", REQUEST_PATH])
        .output()
        .expect("run from-request without mode");
    assert_eq!(missing_mode.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&missing_mode.stderr).contains("<--dry-run|--write>"));
}

#[test]
fn packet_prompt_and_contract_pin_debt_reauthorization_semantics() {
    let fixture = fixture_root("prepare-agent-maintenance-prompt-semantics");
    seed_registry(&fixture);
    seed_support_files(&fixture);
    let registry = agent_registry::AgentRegistry::parse(SEEDED_REGISTRY).expect("registry");
    let entry = registry.find("codex").expect("codex");
    let maintenance_root = "docs/agents/lifecycle/codex-maintenance";
    let request_path =
        "docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml";
    let prompt = contract_policy::packet_pr_prompt_template(entry, maintenance_root)
        .replace("{{VERSION}}", "0.98.0");

    for clause in [
        "re-authorize",
        "in place",
        "authorized_at_version",
        "no target in two rows",
        "authorization_evidence_ref` to `cli_manifests/codex/reports/0.98.0/coverage.any.json",
        "add no row",
        "Newly discovered surface is never deferred",
        "maintenance-audit-status --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml` exits 0",
    ] {
        assert!(prompt.contains(clause), "prompt must contain `{clause}`");
    }

    let contract = contract_policy::build_execution_contract(
        &fixture,
        entry,
        request_path,
        maintenance_root,
        ".github/workflows/agent-maintenance-open-pr.yml",
        "0.98.0",
        "automation/codex-maintenance-0.98.0",
    )
    .expect("build packet-PR execution contract");
    let debt_path = support_audit::NON_TUI_SUPPORT_DEBT_PATH;
    assert!(!contract
        .read_only_inputs
        .iter()
        .any(|path| path == debt_path));
    assert!(contract
        .writable_surfaces
        .iter()
        .any(|path| path == debt_path));
}

#[test]
fn placeholder_is_identity_ordered_distinct_and_can_reconcile_satisfied() {
    let fixture = fixture_root("prepare-agent-maintenance-placeholder-canonical");
    seed_registry(&fixture);
    seed_opencode_support_files(&fixture);
    seed_opencode_debt(&fixture, "1.18.30", "coverage.authorization.json");
    write_agent_report(
        &fixture,
        "opencode",
        "1.18.30",
        "coverage.authorization.json",
        &["alpha", "zeta"],
        false,
    );

    let initial = build_prepare_plan(&fixture, &opencode_args()).expect("build placeholder");
    let audit = initial
        .request
        .support_surface_audit
        .as_ref()
        .expect("audit");
    let identities = audit
        .required_uplifts_this_run
        .iter()
        .map(|row| row.surface_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(identities, vec!["alpha", "zeta"]);
    assert_eq!(audit.pre_run_debt_count, 2);
    assert_eq!(audit.expected_post_run_debt_count, 2);
    assert_eq!(audit.deferred_preexisting_gaps.len(), 2);
    apply_prepare_plan(&fixture, &initial).expect("freeze placeholder");

    seed_opencode_debt(&fixture, "1.18.31", "coverage.any.json");
    write_agent_report(
        &fixture,
        "opencode",
        "1.18.31",
        "coverage.any.json",
        &["alpha", "zeta"],
        true,
    );
    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    let loaded = request::load_request_envelope_validated(&fixture, Path::new(request_path))
        .expect("perfect relay reconciles");
    assert_eq!(
        loaded.support_surface_audit_reconciliation,
        Some(request::AuditReconciliation::Satisfied)
    );
}

fn opencode_args() -> Args {
    Args {
        agent: "opencode".to_string(),
        current_version: "1.18.30".to_string(),
        latest_stable: "1.18.32".to_string(),
        target_version: "1.18.31".to_string(),
        opened_from: Path::new(".github/workflows/agent-maintenance-open-pr.yml").to_path_buf(),
        detected_by: ".github/workflows/agent-maintenance-release-watch.yml".to_string(),
        dispatch_kind: "packet_pr".to_string(),
        dispatch_workflow: None,
        branch_name: "automation/opencode-maintenance-1.18.31".to_string(),
        request_recorded_at: "2026-09-24T08:00:00Z".to_string(),
        request_commit: "abcdef1".to_string(),
        dry_run: true,
        write: false,
    }
}

fn explicit_cli_args() -> Vec<&'static str> {
    vec![
        "prepare-agent-maintenance",
        "--agent",
        "codex",
        "--current-version",
        "0.97.0",
        "--latest-stable",
        "999.0.1",
        "--target-version",
        "999.0.0",
        "--opened-from",
        ".github/workflows/agent-maintenance-open-pr.yml",
        "--detected-by",
        ".github/workflows/agent-maintenance-release-watch.yml",
        "--dispatch-kind",
        "packet_pr",
        "--branch-name",
        "automation/codex-maintenance-0.98.0",
        "--request-recorded-at",
        "2026-05-05T15:00:00Z",
        "--request-commit",
        "abcdef1",
        "--dry-run",
    ]
}

fn snapshot_plan_files(root: &Path, plan: &prepare::PreparePlan) -> BTreeMap<String, Vec<u8>> {
    plan.files
        .iter()
        .map(|file| {
            (
                file.relative_path.clone(),
                fs::read(root.join(&file.relative_path)).expect("read planned file"),
            )
        })
        .collect()
}

fn seed_debt(root: &Path, split_zeta: bool) {
    seed_debt_for_version(root, split_zeta, "0.97.0", "coverage.authorization.json");
    write_authorization_report(root, "0.97.0", "coverage.authorization.json");
}

fn seed_debt_for_version(root: &Path, split_zeta: bool, version: &str, report: &str) {
    let registry = agent_registry::AgentRegistry::parse(SEEDED_REGISTRY).expect("registry");
    let targets = &registry.find("codex").expect("codex").canonical_targets;
    let all = targets.join(", ");
    let (first, rest) = targets.split_first().expect("codex targets");
    let evidence = format!("cli_manifests/codex/reports/{version}/{report}");
    let mut rows = vec![debt_row("zeta-b", "zeta", &all, version, &evidence)];
    if split_zeta {
        rows = vec![
            debt_row("zeta-b", "zeta", &rest.join(", "), version, &evidence),
            debt_row("alpha", "alpha", &all, version, &evidence),
            debt_row("zeta-a", "zeta", first, version, &evidence),
        ];
    } else {
        rows.push(debt_row("alpha", "alpha", &all, version, &evidence));
    }
    write_text(
        &root.join(support_audit::NON_TUI_SUPPORT_DEBT_PATH),
        &format!(
            "# Non-TUI Support Debt Inventory\n\n### `support-debt-authorization-contract-target-version-v1`\n\n## Inventory\n\n{}",
            rows.concat()
        ),
    );
}

fn seed_opencode_support_files(root: &Path) {
    let registry = agent_registry::AgentRegistry::parse(SEEDED_REGISTRY).expect("registry");
    let targets = &registry
        .find("opencode")
        .expect("opencode")
        .canonical_targets;
    write_text(
        &root.join(".github/workflows/agent-maintenance-open-pr.yml"),
        "name: Packet PR worker\n",
    );
    write_text(
        &root.join("cli_manifests/opencode/latest_validated.txt"),
        "1.18.30\n",
    );
    write_text(
        &root.join("cli_manifests/opencode/RULES.json"),
        &json!({"union": {"expected_targets": targets}}).to_string(),
    );
}

fn seed_opencode_debt(root: &Path, version: &str, report: &str) {
    let registry = agent_registry::AgentRegistry::parse(SEEDED_REGISTRY).expect("registry");
    let targets = &registry
        .find("opencode")
        .expect("opencode")
        .canonical_targets;
    let evidence = format!("cli_manifests/opencode/reports/{version}/{report}");
    let rows = [
        agent_debt_row(
            "opencode",
            "zeta-b",
            "zeta",
            &targets[1..].join(", "),
            version,
            &evidence,
        ),
        agent_debt_row(
            "opencode",
            "alpha",
            "alpha",
            &targets.join(", "),
            version,
            &evidence,
        ),
        agent_debt_row(
            "opencode",
            "zeta-a",
            "zeta",
            &targets[0],
            version,
            &evidence,
        ),
    ];
    write_text(
        &root.join(support_audit::NON_TUI_SUPPORT_DEBT_PATH),
        &format!(
            "# Non-TUI Support Debt Inventory\n\n### `support-debt-authorization-contract-target-version-v1`\n\n## Inventory\n\n{}",
            rows.concat()
        ),
    );
}

fn debt_row(row_id: &str, surface: &str, scope: &str, version: &str, evidence: &str) -> String {
    agent_debt_row("codex", row_id, surface, scope, version, evidence)
}

fn agent_debt_row(
    agent: &str,
    row_id: &str,
    surface: &str,
    scope: &str,
    version: &str,
    evidence: &str,
) -> String {
    format!(
        concat!(
            "### `{row_id}`\n\n",
            "- `agent_id`: `{agent}`\n",
            "- `surface_kind`: `commands`\n",
            "- `command_path`: `{agent} {surface}`\n",
            "- `surface_id`: `{surface}`\n",
            "- `current_reason`: `test`\n",
            "- `blocker_class`: `requires_new_architectural_seam`\n",
            "- `owner`: `test`\n",
            "- `milestone`: `test`\n",
            "- `follow_on`: `TODOS.md#test`\n",
            "- `evidence_ref`: `test`\n",
            "- `scope_target_triples`: `{scope}`\n",
            "- `authorized_at_version`: `{version}`\n",
            "- `authorization_evidence_ref`: `{evidence}`\n\n"
        ),
        row_id = row_id,
        agent = agent,
        surface = surface,
        scope = scope,
        version = version,
        evidence = evidence,
    )
}

fn write_authorization_report(root: &Path, version: &str, name: &str) {
    let registry = agent_registry::AgentRegistry::parse(SEEDED_REGISTRY).expect("registry");
    let targets = &registry.find("codex").expect("codex").canonical_targets;
    let rows = ["alpha", "zeta"]
        .into_iter()
        .map(|surface| json!({"path": [surface], "upstream_available_on": targets}))
        .collect::<Vec<_>>();
    let report = report_json(version, targets, "any", None, &rows);
    write_text(
        &root.join(format!("cli_manifests/codex/reports/{version}/{name}")),
        &report.to_string(),
    );
}

fn write_target_reports(root: &Path, surfaces: &[&str]) {
    write_agent_report(root, "codex", "0.98.0", "coverage.any.json", surfaces, true);
}

fn write_agent_report(
    root: &Path,
    agent: &str,
    version: &str,
    any_name: &str,
    surfaces: &[&str],
    write_exact: bool,
) {
    let registry = agent_registry::AgentRegistry::parse(SEEDED_REGISTRY).expect("registry");
    let targets = &registry.find(agent).expect("agent").canonical_targets;
    let rows = surfaces
        .iter()
        .map(|surface| json!({"path": [surface], "upstream_available_on": targets}))
        .collect::<Vec<_>>();
    let dir = root.join(format!("cli_manifests/{agent}/reports/{version}"));
    write_text(
        &dir.join(any_name),
        &report_json(version, targets, "any", None, &rows).to_string(),
    );
    if !write_exact {
        return;
    }
    for target in targets {
        write_text(
            &dir.join(format!("coverage.{target}.json")),
            &report_json(
                version,
                std::slice::from_ref(target),
                "exact_target",
                Some(target),
                &rows,
            )
            .to_string(),
        );
    }
}

fn report_json(
    version: &str,
    targets: &[String],
    mode: &str,
    target: Option<&String>,
    rows: &[serde_json::Value],
) -> serde_json::Value {
    let mut report = json!({
        "inputs": {"upstream": {"semantic_version": version, "targets": targets}},
        "platform_filter": {"mode": mode},
        "deltas": {
            "missing_commands": rows,
            "missing_flags": [],
            "missing_args": [],
            "intentionally_unsupported": [],
        },
    });
    if let Some(target) = target {
        report["platform_filter"]["target_triple"] = json!(target);
    }
    report
}
