use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use serde_json::Value;
use xtask::runtime_follow_on;

use crate::harness::{
    fixture_root, run_xtask, seed_approval_artifact, seed_release_touchpoints, sha256_hex,
    snapshot_files, wrapper_scaffold_args, HarnessOutput,
};

pub const RUNTIME_RUNS_ROOT: &str = "docs/agents/.uaa-temp/runtime-follow-on/runs";
pub const WRITE_RUN_ID: &str = "rtfo-write";
pub const FAKE_CODEX_SCENARIO_FILE: &str = "fake-codex-scenario.txt";
pub const FAKE_CODEX_LOG_FILE: &str = "fake-codex-invocations.log";

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Project automation tasks")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    RuntimeFollowOn(runtime_follow_on::Args),
}

pub fn run_cli<I, S>(argv: I, workspace_root: &Path) -> HarnessOutput
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

pub fn prepare_runtime_fixture(prefix: &str) -> (PathBuf, String) {
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
            "active_runtime_evidence_run_id": Value::Null,
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

pub fn fake_codex_binary_with_exact_commands(fixture: &Path) -> PathBuf {
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

pub fn runtime_args(mode_flag: &str, approval_path: &str, fixture: &Path) -> Vec<String> {
    runtime_args_with_binary(mode_flag, approval_path, &fake_codex_binary(fixture))
}

pub fn runtime_args_with_binary(
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

pub fn write_fake_codex_scenario(fixture: &Path, scenario: &str) {
    fs::write(
        fixture
            .join(RUNTIME_RUNS_ROOT)
            .join(WRITE_RUN_ID)
            .join(FAKE_CODEX_SCENARIO_FILE),
        format!("{scenario}\n"),
    )
    .expect("write fake codex scenario");
}

pub fn read_codex_execution(fixture: &Path, run_id: &str) -> Value {
    let execution_path = fixture
        .join(RUNTIME_RUNS_ROOT)
        .join(run_id)
        .join("codex-execution.json");
    serde_json::from_slice(&fs::read(execution_path).expect("read codex execution"))
        .expect("parse codex execution")
}

pub fn snapshot_without_runtime_runs(root: &Path) -> BTreeMap<String, Vec<u8>> {
    snapshot_files(root)
        .into_iter()
        .filter(|(path, _)| !path.starts_with(RUNTIME_RUNS_ROOT))
        .collect()
}

pub fn lifecycle_state_path(fixture: &Path) -> PathBuf {
    fixture.join("docs/agents/lifecycle/cursor-cli-onboarding/governance/lifecycle-state.json")
}

pub fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read json")).expect("parse json")
}

pub fn write_json(path: &Path, value: &Value) {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize json");
    bytes.push(b'\n');
    fs::write(path, bytes).expect("write json");
}
