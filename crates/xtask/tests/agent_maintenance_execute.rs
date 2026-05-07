use std::fs;

use clap::{CommandFactory, Parser};
use serde_json::Value;

mod release_doc {
    pub use xtask::release_doc::*;
}
mod support_matrix {
    pub use xtask::support_matrix::*;
}

#[path = "support/onboard_agent_harness.rs"]
mod harness;
#[path = "support/agent_maintenance_harness.rs"]
mod maintenance_harness;

use maintenance_harness::{
    execute_args, fake_execute_codex_binary, prepare_execute_fixture, read_json, run_execute_cli,
    snapshot_without_execute_runs, write_fake_execute_codex_scenario, Cli, EXECUTE_RUNS_ROOT,
    EXECUTE_WRITE_RUN_ID, FAKE_EXECUTE_CODEX_LOG_FILE, GATE_ORDER_LOG_FILE,
};

#[test]
fn execute_agent_maintenance_help_text_includes_required_surface() {
    let top_help = Cli::command().render_help().to_string();
    assert!(top_help.contains("execute-agent-maintenance"));

    let err = Cli::try_parse_from(["xtask", "execute-agent-maintenance", "--help"])
        .expect_err("subcommand help should short-circuit parsing");
    assert_eq!(err.exit_code(), 0);
    let help_text = err.to_string();
    assert!(help_text.contains("--request"));
    assert!(help_text.contains("--dry-run"));
    assert!(help_text.contains("--write"));
    assert!(help_text.contains("--run-id"));
    assert!(help_text.contains("--codex-binary"));
}

#[test]
fn execute_agent_maintenance_dry_run_writes_frozen_packet_only_under_run_root() {
    let fixture = prepare_execute_fixture("agent-maintenance-execute-dry-run");
    let codex_binary = fake_execute_codex_binary(&fixture);
    let before = snapshot_without_execute_runs(&fixture);
    let output = run_execute_cli(execute_args("--dry-run", Some(&codex_binary)), &fixture);
    let after = snapshot_without_execute_runs(&fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert_eq!(before, after, "dry-run must only write the temp run packet");

    let run_dir = fixture.join(EXECUTE_RUNS_ROOT).join(EXECUTE_WRITE_RUN_ID);
    for name in [
        "input-contract.json",
        "codex-prompt.md",
        "run-status.json",
        "run-summary.md",
        "validation-report.json",
        "written-paths.json",
    ] {
        assert!(run_dir.join(name).is_file(), "missing {name}");
    }
    let prompt = fs::read_to_string(run_dir.join("codex-prompt.md")).expect("read prompt");
    assert!(prompt.contains("Execute maintenance target 0.98.0."));
    assert!(output.stdout.contains("closeout remains manual"));
}

#[test]
fn execute_agent_maintenance_write_requires_run_id() {
    let fixture = prepare_execute_fixture("agent-maintenance-execute-run-id");
    let codex_binary = fake_execute_codex_binary(&fixture);
    let output = run_execute_cli(
        [
            "xtask",
            "execute-agent-maintenance",
            "--write",
            "--request",
            "docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml",
            "--codex-binary",
            codex_binary.to_string_lossy().as_ref(),
        ],
        &fixture,
    );

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("--run-id is required"));
}

#[test]
fn execute_agent_maintenance_write_rejects_out_of_bounds_paths() {
    let fixture = prepare_execute_fixture("agent-maintenance-execute-boundary");
    let codex_binary = fake_execute_codex_binary(&fixture);
    let dry_run = run_execute_cli(execute_args("--dry-run", Some(&codex_binary)), &fixture);
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_execute_codex_scenario(&fixture, "out_of_bounds");

    let output = run_execute_cli(execute_args("--write", Some(&codex_binary)), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("write boundary violation"));
    assert!(output.stderr.contains("docs/unowned.md"));
}

#[test]
fn execute_agent_maintenance_write_rejects_noop_runtime_execution() {
    let fixture = prepare_execute_fixture("agent-maintenance-execute-noop");
    let codex_binary = fake_execute_codex_binary(&fixture);
    let dry_run = run_execute_cli(execute_args("--dry-run", Some(&codex_binary)), &fixture);
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_execute_codex_scenario(&fixture, "noop");

    let output = run_execute_cli(execute_args("--write", Some(&codex_binary)), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("no runtime-owned output changes from the prepared baseline"));
}

#[test]
fn execute_agent_maintenance_write_fails_closed_on_prompt_mismatch() {
    let fixture = prepare_execute_fixture("agent-maintenance-execute-prompt-mismatch");
    let codex_binary = fake_execute_codex_binary(&fixture);
    let dry_run = run_execute_cli(execute_args("--dry-run", Some(&codex_binary)), &fixture);
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    harness::write_text(
        &fixture.join("cli_manifests/codex/PR_BODY_TEMPLATE.md"),
        "@codex\n\n## Goal\n\nChanged prompt target {{VERSION}}.\n",
    );

    let output = run_execute_cli(execute_args("--write", Some(&codex_binary)), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("prompt_sha256"));
    assert!(output
        .stderr
        .contains("must match the rendered prompt template digest"));
}

#[test]
fn execute_agent_maintenance_write_reuses_prepared_baseline_runs_gates_and_keeps_closeout_manual() {
    let fixture = prepare_execute_fixture("agent-maintenance-execute-success");
    let codex_binary = fake_execute_codex_binary(&fixture);
    let dry_run = run_execute_cli(execute_args("--dry-run", Some(&codex_binary)), &fixture);
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_execute_codex_scenario(&fixture, "success");

    let output = run_execute_cli(execute_args("--write", Some(&codex_binary)), &fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert!(output.stdout.contains("closeout remains manual"));
    let run_dir = fixture.join(EXECUTE_RUNS_ROOT).join(EXECUTE_WRITE_RUN_ID);
    let written_paths: Vec<String> = serde_json::from_slice(
        &fs::read(run_dir.join("written-paths.json")).expect("read written paths"),
    )
    .expect("parse written paths");
    assert!(written_paths
        .iter()
        .any(|path| { path == "docs/agents/lifecycle/codex-maintenance/runtime-note.md" }));
    assert!(written_paths
        .iter()
        .any(|path| path == "cli_manifests/codex/versions/0.98.0.json"));
    let gate_order = fs::read_to_string(run_dir.join(GATE_ORDER_LOG_FILE)).expect("read gate log");
    assert_eq!(gate_order, "gate-1\ngate-2\n");
    let invocation_log =
        fs::read_to_string(run_dir.join(FAKE_EXECUTE_CODEX_LOG_FILE)).expect("read invocation log");
    assert!(invocation_log.contains("--skip-git-repo-check"));
    assert!(invocation_log.contains("--cd"));
    assert!(!invocation_log.contains("--quiet"));
    assert!(!fixture
        .join("docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json")
        .exists());

    let execution = read_json(&run_dir.join("codex-execution.json"));
    assert_eq!(execution.get("exit_code").and_then(Value::as_i64), Some(0));
    let argv = execution
        .get("argv")
        .and_then(Value::as_array)
        .expect("argv array");
    assert_eq!(argv.first().and_then(Value::as_str), Some("exec"));
    assert!(!argv
        .iter()
        .filter_map(Value::as_str)
        .any(|arg| arg == "--quiet"));
    let report = read_json(&run_dir.join("validation-report.json"));
    assert_eq!(report.get("status").and_then(Value::as_str), Some("pass"));
}


#[test]
fn execute_agent_maintenance_write_ignores_generated_python_bytecode_caches() {
    let fixture = prepare_execute_fixture("agent-maintenance-execute-pyc");
    let codex_binary = fake_execute_codex_binary(&fixture);
    let dry_run = run_execute_cli(execute_args("--dry-run", Some(&codex_binary)), &fixture);
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_fake_execute_codex_scenario(&fixture, "success_with_pycache");

    let output = run_execute_cli(execute_args("--write", Some(&codex_binary)), &fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    let run_dir = fixture.join(EXECUTE_RUNS_ROOT).join(EXECUTE_WRITE_RUN_ID);
    let written_paths: Vec<String> = serde_json::from_slice(
        &fs::read(run_dir.join("written-paths.json")).expect("read written paths"),
    )
    .expect("parse written paths");
    assert!(!written_paths
        .iter()
        .any(|path| path.ends_with(".pyc") || path.contains("__pycache__")));
    let report = read_json(&run_dir.join("validation-report.json"));
    assert_eq!(report.get("status").and_then(Value::as_str), Some("pass"));
}
