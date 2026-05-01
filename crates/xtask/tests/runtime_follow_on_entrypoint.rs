use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use clap::{CommandFactory, Parser, Subcommand};
use serde_json::Value;
use xtask::runtime_follow_on;

#[path = "support/onboard_agent_harness.rs"]
mod harness;

use harness::{
    fixture_root, replace_text_once, run_xtask, seed_approval_artifact, seed_release_touchpoints,
    sha256_hex, snapshot_files, wrapper_scaffold_args, HarnessOutput,
};

const RUNTIME_RUNS_ROOT: &str = "docs/agents/.uaa-temp/runtime-follow-on/runs";
const WRITE_RUN_ID: &str = "rtfo-write";
const FAKE_CODEX_SCENARIO_FILE: &str = "fake-codex-scenario.txt";
const FAKE_CODEX_LOG_FILE: &str = "fake-codex-invocations.log";

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Project automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    RuntimeFollowOn(runtime_follow_on::Args),
}

fn run_cli<I, S>(argv: I, workspace_root: &Path) -> HarnessOutput
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = argv
        .into_iter()
        .map(|arg| arg.as_ref().to_string())
        .collect::<Vec<_>>();

    match Cli::try_parse_from(args) {
        Ok(cli) => {
            let mut stdout = Vec::new();
            let mut stderr = String::new();
            let exit_code = match cli.command {
                Command::RuntimeFollowOn(args) => {
                    match runtime_follow_on::run_in_workspace(workspace_root, args, &mut stdout) {
                        Ok(()) => 0,
                        Err(err) => {
                            stderr = format!("{err}\n");
                            err.exit_code()
                        }
                    }
                }
            };
            HarnessOutput {
                exit_code,
                stdout: String::from_utf8(stdout).expect("stdout must be utf-8"),
                stderr,
            }
        }
        Err(err) => HarnessOutput {
            exit_code: err.exit_code(),
            stdout: String::new(),
            stderr: err.to_string(),
        },
    }
}

fn prepare_runtime_fixture(prefix: &str) -> (std::path::PathBuf, String) {
    let fixture = fixture_root(prefix);
    seed_release_touchpoints(&fixture);
    let approval_path = seed_approval_artifact(
        &fixture,
        "docs/agents/lifecycle/cursor-cli-onboarding/governance/approved-agent.toml",
        "cursor",
        "cursor",
        None,
    );
    let onboard = run_xtask(
        &fixture,
        [
            "onboard-agent".to_string(),
            "--write".to_string(),
            "--approval".to_string(),
            approval_path.clone(),
        ],
    );
    assert_eq!(onboard.exit_code, 0, "stderr:\n{}", onboard.stderr);
    let scaffold = run_xtask(&fixture, wrapper_scaffold_args("--write", "cursor"));
    assert_eq!(scaffold.exit_code, 0, "stderr:\n{}", scaffold.stderr);
    write_json(
        &fixture
            .join("docs/agents/lifecycle/cursor-cli-onboarding/governance/lifecycle-state.json"),
        &serde_json::json!({
            "schema_version": "1",
            "agent_id": "cursor",
            "onboarding_pack_prefix": "cursor-cli-onboarding",
            "approval_artifact_path": approval_path,
            "approval_artifact_sha256": sha256_hex(&fixture.join(&approval_path)),
            "lifecycle_stage": "enrolled",
            "support_tier": "bootstrap",
            "side_states": [],
            "current_owner_command": "runtime-follow-on --write",
            "expected_next_command": format!("runtime-follow-on --approval {approval_path} --dry-run"),
            "last_transition_at": "2026-05-01T00:00:00Z",
            "last_transition_by": "runtime-follow-on-entrypoint-test",
            "required_evidence": [
                "registry_entry",
                "docs_pack",
                "manifest_root_skeleton"
            ],
            "satisfied_evidence": [
                "registry_entry",
                "docs_pack",
                "manifest_root_skeleton"
            ],
            "blocking_issues": [],
            "retryable_failures": [],
            "implementation_summary": Value::Null,
            "publication_packet_path": Value::Null,
            "publication_packet_sha256": Value::Null,
            "closeout_baseline_path": Value::Null
        }),
    );
    (fixture, approval_path)
}

fn fake_codex_fixture_source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake_codex.sh")
}

fn fake_codex_binary(fixture: &Path) -> PathBuf {
    let binary = fixture
        .join(RUNTIME_RUNS_ROOT)
        .join(WRITE_RUN_ID)
        .join("fake_codex.sh");
    if !binary.is_file() {
        fs::copy(fake_codex_fixture_source(), &binary).expect("copy fake codex fixture");
        mark_fixture_executable(&binary);
    }
    binary
}

fn fake_codex_binary_with_exact_commands(fixture: &Path) -> PathBuf {
    let binary = fixture
        .join(RUNTIME_RUNS_ROOT)
        .join(WRITE_RUN_ID)
        .join("fake_codex-exact-commands.sh");
    if !binary.is_file() {
        fs::copy(fake_codex_fixture_source(), &binary).expect("copy exact-command fake codex");
        let script = fs::read_to_string(&binary).expect("read exact-command fake codex");
        let script = script
            .replace(
                "\"support-matrix --check\"",
                "\"cargo run -p xtask -- support-matrix --check\"",
            )
            .replace(
                "\"capability-matrix --check\"",
                "\"cargo run -p xtask -- capability-matrix --check\"",
            )
            .replace(
                "\"capability-matrix-audit\"",
                "\"cargo run -p xtask -- capability-matrix-audit\"",
            );
        fs::write(&binary, script).expect("write exact-command fake codex");
        mark_fixture_executable(&binary);
    }
    binary
}

#[cfg(unix)]
fn mark_fixture_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .expect("stat fake codex fixture")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("chmod fake codex fixture");
}

#[cfg(not(unix))]
fn mark_fixture_executable(_path: &Path) {}

fn runtime_args(mode_flag: &str, approval_path: &str, fixture: &Path) -> Vec<String> {
    runtime_args_with_binary(mode_flag, approval_path, &fake_codex_binary(fixture))
}

fn runtime_args_with_binary(
    mode_flag: &str,
    approval_path: &str,
    codex_binary: &Path,
) -> Vec<String> {
    let mut args = vec![
        "xtask".to_string(),
        "runtime-follow-on".to_string(),
        mode_flag.to_string(),
        "--approval".to_string(),
        approval_path.to_string(),
    ];
    if mode_flag == "--write" {
        args.extend([
            "--run-id".to_string(),
            WRITE_RUN_ID.to_string(),
            "--codex-binary".to_string(),
            codex_binary.display().to_string(),
        ]);
    }
    args
}

fn write_fake_codex_scenario(fixture: &Path, scenario: &str) {
    fs::write(
        fixture
            .join(RUNTIME_RUNS_ROOT)
            .join(WRITE_RUN_ID)
            .join(FAKE_CODEX_SCENARIO_FILE),
        format!("{scenario}\n"),
    )
    .expect("write fake codex scenario");
}

fn read_codex_execution(fixture: &Path, run_id: &str) -> Value {
    let execution_path = fixture
        .join(RUNTIME_RUNS_ROOT)
        .join(run_id)
        .join("codex-execution.json");
    serde_json::from_slice(&fs::read(execution_path).expect("read codex execution"))
        .expect("parse codex execution")
}

fn snapshot_without_runtime_runs(root: &Path) -> BTreeMap<String, Vec<u8>> {
    snapshot_files(root)
        .into_iter()
        .filter(|(path, _)| !path.starts_with(RUNTIME_RUNS_ROOT))
        .collect()
}

fn lifecycle_state_path(fixture: &Path) -> PathBuf {
    fixture.join("docs/agents/lifecycle/cursor-cli-onboarding/governance/lifecycle-state.json")
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read json")).expect("parse json")
}

fn write_json(path: &Path, value: &Value) {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize json");
    bytes.push(b'\n');
    fs::write(path, bytes).expect("write json");
}

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
