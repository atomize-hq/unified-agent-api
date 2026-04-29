use std::{collections::BTreeMap, fs, path::Path};

use clap::{CommandFactory, Parser, Subcommand};
use serde_json::json;
use xtask::runtime_follow_on;

#[path = "support/onboard_agent_harness.rs"]
mod harness;

use harness::{
    fixture_root, replace_text_once, run_xtask, seed_approval_artifact, seed_release_touchpoints,
    snapshot_files, wrapper_scaffold_args, HarnessOutput,
};

const RUNTIME_RUNS_ROOT: &str = "docs/agents/.uaa-temp/runtime-follow-on/runs";

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
    (fixture, approval_path)
}

fn runtime_args(mode_flag: &str, approval_path: &str) -> Vec<String> {
    let mut args = vec![
        "xtask".to_string(),
        "runtime-follow-on".to_string(),
        mode_flag.to_string(),
        "--approval".to_string(),
        approval_path.to_string(),
    ];
    if mode_flag == "--write" {
        args.extend(["--run-id".to_string(), "rtfo-write".to_string()]);
    }
    args
}

fn write_valid_handoff(fixture: &Path, run_id: &str) {
    let handoff = json!({
        "agent_id": "cursor",
        "manifest_root": "cli_manifests/cursor",
        "runtime_lane_complete": true,
        "publication_refresh_required": true,
        "required_commands": [
            "support-matrix --check",
            "capability-matrix --check",
            "capability-matrix-audit",
            "make preflight"
        ],
        "blockers": []
    });
    let path = fixture
        .join(RUNTIME_RUNS_ROOT)
        .join(run_id)
        .join("handoff.json");
    fs::write(
        &path,
        serde_json::to_vec_pretty(&handoff).expect("serialize handoff"),
    )
    .expect("write handoff");
}

fn write_required_test(fixture: &Path) {
    let test_path = fixture.join("crates/agent_api/tests/c1_cursor_runtime_follow_on.rs");
    if let Some(parent) = test_path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(&test_path, "#[test]\nfn runtime_follow_on_smoke() {}\n").expect("write test");
}

fn snapshot_without_runtime_runs(root: &Path) -> BTreeMap<String, Vec<u8>> {
    snapshot_files(root)
        .into_iter()
        .filter(|(path, _)| !path.starts_with(RUNTIME_RUNS_ROOT))
        .collect()
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
    assert!(prompt.contains("Do not edit generated `cli_manifests/cursor/wrapper_coverage.json`."));
    assert!(prompt.contains("crates/agent_api/tests/c1_cursor_runtime_follow_on.rs"));
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
            "rtfo-write",
        ],
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_valid_handoff(&fixture, "rtfo-write");
    write_required_test(&fixture);
    fs::write(fixture.join("docs/unowned.md"), "not allowed\n").expect("write unowned");

    let output = run_cli(runtime_args("--write", &approval_path), &fixture);

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
            "rtfo-write",
        ],
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_valid_handoff(&fixture, "rtfo-write");
    write_required_test(&fixture);
    fs::write(
        fixture.join("cli_manifests/cursor/wrapper_coverage.json"),
        "{\n  \"schema_version\": 1\n}\n",
    )
    .expect("write generated wrapper coverage");

    let output = run_cli(runtime_args("--write", &approval_path), &fixture);

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
            "rtfo-write",
        ],
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_required_test(&fixture);
    let handoff_path = fixture
        .join(RUNTIME_RUNS_ROOT)
        .join("rtfo-write")
        .join("handoff.json");
    fs::write(
        &handoff_path,
        serde_json::to_vec_pretty(&json!({
            "agent_id": "cursor",
            "manifest_root": "cli_manifests/cursor",
            "publication_refresh_required": true,
            "required_commands": ["support-matrix --check"],
            "blockers": []
        }))
        .expect("serialize malformed handoff"),
    )
    .expect("write malformed handoff");

    let output = run_cli(runtime_args("--write", &approval_path), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("handoff.json is missing required field `runtime_lane_complete`"));
}

#[test]
fn runtime_follow_on_write_succeeds_with_valid_handoff_and_allowed_paths() {
    let (fixture, approval_path) = prepare_runtime_fixture("runtime-follow-on-success");
    let dry_run = run_cli(
        [
            "xtask",
            "runtime-follow-on",
            "--dry-run",
            "--approval",
            &approval_path,
            "--run-id",
            "rtfo-write",
        ],
        &fixture,
    );
    assert_eq!(dry_run.exit_code, 0, "stderr:\n{}", dry_run.stderr);
    write_valid_handoff(&fixture, "rtfo-write");
    write_required_test(&fixture);
    let backend_path = fixture.join("crates/agent_api/src/backends/cursor/mod.rs");
    if let Some(parent) = backend_path.parent() {
        fs::create_dir_all(parent).expect("create backend parent");
    }
    fs::write(&backend_path, "pub fn runtime_follow_on() {}\n").expect("write backend");
    let fake_bin_path =
        fixture.join("crates/agent_api/src/bin/fake_cursor_stream_json_agent_api.rs");
    if let Some(parent) = fake_bin_path.parent() {
        fs::create_dir_all(parent).expect("create fake binary parent");
    }
    fs::write(&fake_bin_path, "fn main() {}\n").expect("write fake binary");
    fs::create_dir_all(fixture.join("cli_manifests/cursor/snapshots")).expect("create snapshots");
    fs::write(
        fixture.join("cli_manifests/cursor/snapshots/default.json"),
        "{\n  \"snapshot\": true\n}\n",
    )
    .expect("write snapshot");
    fs::write(fixture.join("Cargo.lock"), "# synthetic lockfile delta\n").expect("write lockfile");

    let output = run_cli(runtime_args("--write", &approval_path), &fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    let written_paths = fs::read_to_string(
        fixture
            .join(RUNTIME_RUNS_ROOT)
            .join("rtfo-write")
            .join("written-paths.json"),
    )
    .expect("read written paths");
    let _: Vec<String> = serde_json::from_str(&written_paths).expect("parse written paths");
}
