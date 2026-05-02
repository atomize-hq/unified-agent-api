use std::fs;

use clap::{CommandFactory, Parser};
use serde_json::Value;

#[path = "support/onboard_agent_harness.rs"]
mod harness;
#[path = "support/runtime_follow_on_harness.rs"]
mod runtime_harness;

use harness::replace_text_once;
use runtime_harness::{
    fake_codex_binary_with_exact_commands, lifecycle_state_path, prepare_runtime_fixture,
    read_codex_execution, read_json, run_cli, runtime_args, runtime_args_with_binary,
    snapshot_without_runtime_runs, write_fake_codex_scenario, write_json, Cli, FAKE_CODEX_LOG_FILE,
    RUNTIME_RUNS_ROOT, WRITE_RUN_ID,
};

#[test]
fn runtime_follow_on_help_text_includes_required_surface() {
    let top_help = Cli::command().render_help().to_string();
    assert!(top_help.contains("runtime-follow-on"));

    let err = Cli::try_parse_from(["xtask", "runtime-follow-on", "--help"])
        .expect_err("subcommand help should short-circuit parsing");
    assert_eq!(err.exit_code(), 0);
    let help_text = err.to_string();
    assert!(help_text.contains("--dry-run"));
    assert!(help_text.contains("--write"));
    assert!(help_text.contains("--approval"));
    assert!(help_text.contains("--requested-tier"));
    assert!(help_text.contains("--minimal-justification-file"));
    assert!(help_text.contains("--allow-rich-surface"));
    assert!(help_text.contains("--run-id"));
    assert!(help_text.contains("--codex-binary"));
}

#[test]
fn runtime_follow_on_dry_run_writes_full_packet_without_touching_other_paths() {
    let (fixture, approval_path) = prepare_runtime_fixture("runtime-follow-on-dry-run");
    let before = snapshot_without_runtime_runs(&fixture);
    let output = run_cli(
        [
            "xtask",
            "runtime-follow-on",
            "--dry-run",
            "--approval",
            &approval_path,
            "--run-id",
            "rtfo-dry-run",
        ],
        &fixture,
    );
    let after = snapshot_without_runtime_runs(&fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert_eq!(
        before, after,
        "dry-run must only write scratch packet artifacts"
    );

    let run_dir = fixture.join(RUNTIME_RUNS_ROOT).join("rtfo-dry-run");
    for name in [
        "input-contract.json",
        "codex-prompt.md",
        "run-status.json",
        "run-summary.md",
        "validation-report.json",
        "written-paths.json",
        "handoff.json",
    ] {
        assert!(run_dir.join(name).is_file(), "missing {name}");
    }
    let prompt = fs::read_to_string(run_dir.join("codex-prompt.md")).expect("read prompt");
    let input_contract = read_json(&run_dir.join("input-contract.json"));
    assert!(prompt.contains("Do not edit generated `cli_manifests/cursor/wrapper_coverage.json`."));
    assert!(prompt.contains("crates/agent_api/tests/c1_cursor_runtime_follow_on.rs"));
    assert!(prompt.contains("canonical targets:"));
    assert!(prompt.contains("linux-x64"));
    assert!(prompt.contains("agent_api.tools.mcp.list.v1:linux-x64"));
    assert!(prompt.contains("agent_api.exec.external_sandbox.v1:allow_external_sandbox_exec"));
    assert!(prompt.contains("cargo run -p xtask -- support-matrix --check"));
    assert_eq!(
        input_contract
            .get("canonical_targets")
            .and_then(Value::as_array)
            .and_then(|values| values.first())
            .and_then(Value::as_str),
        Some("linux-x64")
    );
    assert_eq!(
        input_contract
            .get("support_matrix_enabled")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        input_contract
            .get("capability_matrix_enabled")
            .and_then(Value::as_bool),
        Some(true)
    );
}

#[test]
fn runtime_follow_on_rejects_minimal_without_justification() {
    let (fixture, approval_path) = prepare_runtime_fixture("runtime-follow-on-minimal");
    let output = run_cli(
        [
            "xtask",
            "runtime-follow-on",
            "--dry-run",
            "--approval",
            &approval_path,
            "--requested-tier",
            "minimal",
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("--minimal-justification-file"));
}

#[test]
fn runtime_follow_on_rejects_approval_registry_mismatch() {
    let (fixture, approval_path) = prepare_runtime_fixture("runtime-follow-on-registry-mismatch");
    replace_text_once(
        &fixture.join("crates/xtask/data/agent_registry.toml"),
        "source_path = \"crates/cursor\"\n",
        "source_path = \"crates/not_cursor\"\n",
    );

    let output = run_cli(
        [
            "xtask",
            "runtime-follow-on",
            "--dry-run",
            "--approval",
            &approval_path,
            "--run-id",
            "rtfo-mismatch",
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("approval/registry mismatch"));
}

#[test]
fn runtime_follow_on_requires_enrolled_lifecycle_state() {
    let (fixture, approval_path) = prepare_runtime_fixture("runtime-follow-on-lifecycle-stage");
    let lifecycle_path = lifecycle_state_path(&fixture);
    let mut lifecycle_state = read_json(&lifecycle_path);
    lifecycle_state["lifecycle_stage"] = Value::String("approved".to_string());
    write_json(&lifecycle_path, &lifecycle_state);

    let output = run_cli(
        [
            "xtask",
            "runtime-follow-on",
            "--dry-run",
            "--approval",
            &approval_path,
            "--run-id",
            "rtfo-stage-check",
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("requires lifecycle stage `enrolled`"));
}

#[test]
fn runtime_follow_on_write_rejects_out_of_bounds_paths() {
    let (fixture, approval_path) = prepare_runtime_fixture("runtime-follow-on-boundary");
    let dry_run = run_cli(
        [
            "xtask",
            "runtime-follow-on",
            "--dry-run",
            "--approval",
            &approval_path,
            "--run-id",
            WRITE_RUN_ID,
        ],
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_codex_scenario(&fixture, "success");
    fs::write(fixture.join("docs/unowned.md"), "not allowed\n").expect("write unowned");

    let output = run_cli(runtime_args("--write", &approval_path, &fixture), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("write boundary violation"));
    assert!(output.stderr.contains("docs/unowned.md"));
}

#[test]
fn runtime_follow_on_write_rejects_generated_wrapper_coverage_json_edit() {
    let (fixture, approval_path) = prepare_runtime_fixture("runtime-follow-on-wrapper-coverage");
    let dry_run = run_cli(
        [
            "xtask",
            "runtime-follow-on",
            "--dry-run",
            "--approval",
            &approval_path,
            "--run-id",
            WRITE_RUN_ID,
        ],
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_codex_scenario(&fixture, "success");
    fs::write(
        fixture.join("cli_manifests/cursor/wrapper_coverage.json"),
        "{\n  \"schema_version\": 1\n}\n",
    )
    .expect("write generated wrapper coverage");

    let output = run_cli(runtime_args("--write", &approval_path, &fixture), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("generated wrapper coverage edits are forbidden"));
}

#[test]
fn runtime_follow_on_write_requires_semantically_valid_handoff() {
    let (fixture, approval_path) = prepare_runtime_fixture("runtime-follow-on-handoff");
    let dry_run = run_cli(
        [
            "xtask",
            "runtime-follow-on",
            "--dry-run",
            "--approval",
            &approval_path,
            "--run-id",
            WRITE_RUN_ID,
        ],
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_codex_scenario(&fixture, "invalid_handoff");

    let output = run_cli(runtime_args("--write", &approval_path, &fixture), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("handoff.json is missing required field `runtime_lane_complete`"));
    let lifecycle_state = read_json(&lifecycle_state_path(&fixture));
    assert_eq!(
        lifecycle_state
            .get("lifecycle_stage")
            .and_then(Value::as_str),
        Some("enrolled")
    );
    assert_eq!(
        lifecycle_state
            .get("side_states")
            .and_then(Value::as_array)
            .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>()),
        Some(vec!["failed_retryable"])
    );
    assert!(lifecycle_state
        .get("retryable_failures")
        .and_then(Value::as_array)
        .expect("retryable_failures array")
        .iter()
        .any(|value| value.as_str()
            == Some("handoff.json is missing required field `runtime_lane_complete`")));
    assert!(lifecycle_state
        .get("implementation_summary")
        .is_some_and(Value::is_null));
}

#[test]
fn runtime_follow_on_write_rejects_noop_runtime_execution() {
    let (fixture, approval_path) = prepare_runtime_fixture("runtime-follow-on-noop");
    let required_test = fixture.join("crates/agent_api/tests/c1_cursor_runtime_follow_on.rs");
    if let Some(parent) = required_test.parent() {
        fs::create_dir_all(parent).expect("create required test parent");
    }
    fs::write(&required_test, "#[test]\nfn runtime_follow_on_smoke() {}\n")
        .expect("write baseline required test");
    let dry_run = run_cli(
        [
            "xtask",
            "runtime-follow-on",
            "--dry-run",
            "--approval",
            &approval_path,
            "--run-id",
            WRITE_RUN_ID,
        ],
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_codex_scenario(&fixture, "handoff_only");

    let output = run_cli(runtime_args("--write", &approval_path, &fixture), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("produced no runtime-owned output changes from the prepared baseline"));
}

#[test]
fn runtime_follow_on_write_rejects_legacy_short_form_publication_commands() {
    let (fixture, approval_path) = prepare_runtime_fixture("runtime-follow-on-legacy-commands");
    let dry_run = run_cli(
        [
            "xtask",
            "runtime-follow-on",
            "--dry-run",
            "--approval",
            &approval_path,
            "--run-id",
            WRITE_RUN_ID,
        ],
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_codex_scenario(&fixture, "success");

    let output = run_cli(runtime_args("--write", &approval_path, &fixture), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains(
        "handoff.json required_commands must match the frozen publication command set exactly"
    ));
    let lifecycle_state = read_json(&lifecycle_state_path(&fixture));
    assert_eq!(
        lifecycle_state
            .get("lifecycle_stage")
            .and_then(Value::as_str),
        Some("enrolled")
    );
    assert_eq!(
        lifecycle_state
            .get("side_states")
            .and_then(Value::as_array)
            .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>()),
        Some(vec!["failed_retryable"])
    );
}

#[test]
fn runtime_follow_on_write_spawns_configured_codex_binary_and_requires_real_writes() {
    let (fixture, approval_path) = prepare_runtime_fixture("runtime-follow-on-success");
    let dry_run = run_cli(
        [
            "xtask",
            "runtime-follow-on",
            "--dry-run",
            "--approval",
            &approval_path,
            "--run-id",
            WRITE_RUN_ID,
        ],
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_codex_scenario(&fixture, "success");

    let exact_command_binary = fake_codex_binary_with_exact_commands(&fixture);
    let output = run_cli(
        runtime_args_with_binary("--write", &approval_path, &exact_command_binary),
        &fixture,
    );

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    let execution = read_codex_execution(&fixture, WRITE_RUN_ID);
    assert_eq!(
        execution.get("binary").and_then(Value::as_str),
        Some(exact_command_binary.to_string_lossy().as_ref())
    );
    assert_eq!(execution.get("exit_code").and_then(Value::as_i64), Some(0));
    let argv = execution
        .get("argv")
        .and_then(Value::as_array)
        .expect("argv array");
    assert_eq!(argv.first().and_then(Value::as_str), Some("exec"));
    assert!(argv.iter().any(|value| value.as_str() == Some("--cd")));
    assert!(argv
        .iter()
        .any(|value| value.as_str() == Some("--dangerously-bypass-approvals-and-sandbox")));
    let invocation_log = fs::read_to_string(
        fixture
            .join(RUNTIME_RUNS_ROOT)
            .join(WRITE_RUN_ID)
            .join(FAKE_CODEX_LOG_FILE),
    )
    .expect("read fake codex invocation log");
    assert!(invocation_log.contains("--skip-git-repo-check"));
    assert!(invocation_log.contains("--cd"));
    let written_paths = fs::read_to_string(
        fixture
            .join(RUNTIME_RUNS_ROOT)
            .join(WRITE_RUN_ID)
            .join("written-paths.json"),
    )
    .expect("read written paths");
    let parsed: Vec<String> = serde_json::from_str(&written_paths).expect("parse written paths");
    assert!(
        !parsed.is_empty(),
        "success path must record runtime-owned writes"
    );
    let lifecycle_state = read_json(&lifecycle_state_path(&fixture));
    assert_eq!(
        lifecycle_state
            .get("lifecycle_stage")
            .and_then(Value::as_str),
        Some("runtime_integrated")
    );
    assert_eq!(
        lifecycle_state.get("support_tier").and_then(Value::as_str),
        Some("baseline_runtime")
    );
    assert_eq!(
        lifecycle_state
            .get("expected_next_command")
            .and_then(Value::as_str),
        Some(
            "prepare-publication --approval docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml --write"
        )
    );
    assert!(lifecycle_state
        .get("satisfied_evidence")
        .and_then(Value::as_array)
        .expect("satisfied_evidence array")
        .iter()
        .any(|value| value.as_str() == Some("runtime_write_complete")));
    assert!(lifecycle_state
        .get("satisfied_evidence")
        .and_then(Value::as_array)
        .expect("satisfied_evidence array")
        .iter()
        .any(|value| value.as_str() == Some("implementation_summary_present")));
    assert!(lifecycle_state
        .get("implementation_summary")
        .and_then(Value::as_object)
        .is_some());
    assert_eq!(
        lifecycle_state
            .get("active_runtime_evidence_run_id")
            .and_then(Value::as_str),
        Some(WRITE_RUN_ID)
    );
    assert!(!fixture
        .join("docs/agents/lifecycle/cursor-cli-onboarding/governance/publication-ready.json")
        .exists());
}

#[test]
fn runtime_follow_on_write_marks_blocked_when_handoff_reports_exact_blockers() {
    let (fixture, approval_path) = prepare_runtime_fixture("runtime-follow-on-blocked");
    let dry_run = run_cli(
        [
            "xtask",
            "runtime-follow-on",
            "--dry-run",
            "--approval",
            &approval_path,
            "--run-id",
            WRITE_RUN_ID,
        ],
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_codex_scenario(&fixture, "exec_fail");
    write_json(
        &fixture
            .join(RUNTIME_RUNS_ROOT)
            .join(WRITE_RUN_ID)
            .join("handoff.json"),
        &serde_json::json!({
            "agent_id": "cursor",
            "manifest_root": "cli_manifests/cursor",
            "runtime_lane_complete": false,
            "publication_refresh_required": true,
            "required_commands": [
                "cargo run -p xtask -- support-matrix --check",
                "cargo run -p xtask -- capability-matrix --check",
                "cargo run -p xtask -- capability-matrix-audit",
                "make preflight"
            ],
            "blockers": ["Upstream runtime dependency is not yet available."]
        }),
    );

    let output = run_cli(runtime_args("--write", &approval_path, &fixture), &fixture);

    assert_eq!(output.exit_code, 2);
    let lifecycle_state = read_json(&lifecycle_state_path(&fixture));
    assert_eq!(
        lifecycle_state
            .get("lifecycle_stage")
            .and_then(Value::as_str),
        Some("enrolled")
    );
    assert_eq!(
        lifecycle_state
            .get("side_states")
            .and_then(Value::as_array)
            .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>()),
        Some(vec!["blocked"])
    );
    assert!(lifecycle_state
        .get("blocking_issues")
        .and_then(Value::as_array)
        .expect("blocking_issues array")
        .iter()
        .any(|value| value.as_str() == Some("Upstream runtime dependency is not yet available.")));
}
