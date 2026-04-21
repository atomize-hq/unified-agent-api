use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use xtask::onboard_agent;

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Project automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    OnboardAgent(onboard_agent::Args),
}

#[derive(Debug)]
struct HarnessOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

#[test]
fn onboard_agent_branch_real_gemini_closeout_packet_matches_committed_preview() {
    let workspace_root = repo_root();
    let output = run_cli(gemini_dry_run_args(), &workspace_root);
    let metrics_path = workspace_root.join(
        "docs/project_management/next/gemini-cli-onboarding/governance/proving-run-metrics.json",
    );
    let metrics_text = fs::read_to_string(&metrics_path).expect("read committed metrics");
    let metrics: serde_json::Value =
        serde_json::from_str(&metrics_text).expect("parse committed metrics");
    let recorded_at = metrics["recorded_at"].as_str().expect("recorded_at string");
    let commit = metrics["commit"].as_str().expect("commit string");

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert!(output
        .stdout
        .contains("Shared onboarding plan preview; no filesystem writes performed."));
    assert!(output
        .stdout
        .contains("This packet records the closed proving run for `gemini_cli`."));
    assert!(output.stdout.contains(recorded_at));
    assert!(output.stdout.contains(commit));
    assert_stdout_contains_file_preview(
        &output.stdout,
        &workspace_root,
        "docs/project_management/next/gemini-cli-onboarding/README.md",
    );
    assert_stdout_contains_file_preview(
        &output.stdout,
        &workspace_root,
        "docs/project_management/next/gemini-cli-onboarding/scope_brief.md",
    );
    assert_stdout_contains_file_preview(
        &output.stdout,
        &workspace_root,
        "docs/project_management/next/gemini-cli-onboarding/seam_map.md",
    );
    assert_stdout_contains_file_preview(
        &output.stdout,
        &workspace_root,
        "docs/project_management/next/gemini-cli-onboarding/threading.md",
    );
    assert_stdout_contains_file_preview(
        &output.stdout,
        &workspace_root,
        "docs/project_management/next/gemini-cli-onboarding/review_surfaces.md",
    );
    assert_stdout_contains_file_preview(
        &output.stdout,
        &workspace_root,
        "docs/project_management/next/gemini-cli-onboarding/governance/remediation-log.md",
    );
    assert_stdout_contains_file_preview(
        &output.stdout,
        &workspace_root,
        "docs/project_management/next/gemini-cli-onboarding/HANDOFF.md",
    );
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
                Command::OnboardAgent(args) => {
                    match onboard_agent::run_in_workspace(workspace_root, args, &mut stdout) {
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

fn gemini_dry_run_args() -> Vec<String> {
    vec![
        "xtask".to_string(),
        "onboard-agent".to_string(),
        "--dry-run".to_string(),
        "--agent-id".to_string(),
        "gemini_cli".to_string(),
        "--display-name".to_string(),
        "Gemini CLI".to_string(),
        "--crate-path".to_string(),
        "crates/gemini_cli".to_string(),
        "--backend-module".to_string(),
        "crates/agent_api/src/backends/gemini_cli".to_string(),
        "--manifest-root".to_string(),
        "cli_manifests/gemini_cli".to_string(),
        "--package-name".to_string(),
        "unified-agent-api-gemini-cli".to_string(),
        "--canonical-target".to_string(),
        "darwin-arm64".to_string(),
        "--wrapper-coverage-binding-kind".to_string(),
        "generated_from_wrapper_crate".to_string(),
        "--wrapper-coverage-source-path".to_string(),
        "crates/gemini_cli".to_string(),
        "--always-on-capability".to_string(),
        "agent_api.config.model.v1".to_string(),
        "--always-on-capability".to_string(),
        "agent_api.events".to_string(),
        "--always-on-capability".to_string(),
        "agent_api.events.live".to_string(),
        "--always-on-capability".to_string(),
        "agent_api.run".to_string(),
        "--support-matrix-enabled".to_string(),
        "true".to_string(),
        "--capability-matrix-enabled".to_string(),
        "true".to_string(),
        "--docs-release-track".to_string(),
        "crates-io".to_string(),
        "--onboarding-pack-prefix".to_string(),
        "gemini-cli-onboarding".to_string(),
    ]
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn assert_stdout_contains_file_preview(stdout: &str, root: &Path, relative_path: &str) {
    let contents = fs::read_to_string(root.join(relative_path)).expect("read committed file");
    let expected = format!("Path: {relative_path}\n```md\n{contents}```");
    assert!(
        stdout.contains(&expected),
        "stdout did not contain committed preview for {relative_path}:\n{stdout}"
    );
}
