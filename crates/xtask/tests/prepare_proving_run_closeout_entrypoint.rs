use std::{fs, path::Path, path::PathBuf, process::Command};

use clap::{CommandFactory, Parser, Subcommand};
use serde_json::{json, Value};
use xtask::{agent_lifecycle, prepare_proving_run_closeout};

#[path = "support/onboard_agent_harness.rs"]
mod harness;

use harness::{fixture_root, seed_gemini_approval_artifact, sha256_hex, write_text, HarnessOutput};

const APPROVAL_PATH: &str =
    "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
const CLOSEOUT_PATH: &str =
    "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json";
const HANDOFF_PATH: &str = "docs/agents/lifecycle/gemini-cli-onboarding/HANDOFF.md";
const README_PATH: &str = "docs/agents/lifecycle/gemini-cli-onboarding/README.md";
const LIFECYCLE_STATE_PATH: &str =
    "docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json";
const PUBLICATION_PACKET_PATH: &str =
    "docs/agents/lifecycle/gemini-cli-onboarding/governance/publication-ready.json";

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Project automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: CommandKind,
}

#[derive(Debug, Subcommand)]
enum CommandKind {
    PrepareProvingRunCloseout(prepare_proving_run_closeout::Args),
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
                CommandKind::PrepareProvingRunCloseout(args) => {
                    match prepare_proving_run_closeout::run_in_workspace(
                        workspace_root,
                        args,
                        &mut stdout,
                    ) {
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

fn prepare_args(mode_flag: &str) -> Vec<String> {
    vec![
        "xtask".to_string(),
        "prepare-proving-run-closeout".to_string(),
        mode_flag.to_string(),
        "--approval".to_string(),
        APPROVAL_PATH.to_string(),
    ]
}

fn prepare_published_fixture(prefix: &str) -> PathBuf {
    let fixture = fixture_root(prefix);
    let approval_path =
        seed_gemini_approval_artifact(&fixture, APPROVAL_PATH, "gemini-cli-onboarding");
    seed_published_baseline(&fixture, &approval_path);
    init_git_repo(&fixture);
    fixture
}

fn init_git_repo(root: &Path) {
    run_git(root, ["init"]);
    run_git(root, ["config", "user.name", "Codex Test"]);
    run_git(root, ["config", "user.email", "codex@example.com"]);
    run_git(root, ["add", "."]);
    run_git(root, ["commit", "--no-gpg-sign", "-m", "fixture"]);
}

fn run_git<const N: usize>(root: &Path, args: [&str; N]) {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {:?} failed:\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn seed_published_baseline(root: &Path, approval_path: &str) {
    let approval_sha256 = sha256_hex(&root.join(approval_path));
    let lifecycle_path = root.join(LIFECYCLE_STATE_PATH);
    let packet_path = root.join(PUBLICATION_PACKET_PATH);
    let packet_json = |lifecycle_sha256: &str| {
        json!({
            "schema_version": "1",
            "agent_id": "gemini_cli",
            "approval_artifact_path": approval_path,
            "approval_artifact_sha256": approval_sha256,
            "lifecycle_state_path": LIFECYCLE_STATE_PATH,
            "lifecycle_state_sha256": lifecycle_sha256,
            "lifecycle_stage": "publication_ready",
            "support_tier_at_emit": "baseline_runtime",
            "manifest_root": "cli_manifests/gemini_cli",
            "expected_targets": ["darwin-arm64"],
            "capability_publication_enabled": true,
            "support_publication_enabled": true,
            "capability_matrix_target": Value::Null,
            "required_commands": [
                "cargo run -p xtask -- support-matrix --check",
                "cargo run -p xtask -- capability-matrix --check",
                "cargo run -p xtask -- capability-matrix-audit",
                "make preflight"
            ],
            "required_publication_outputs": [
                "cli_manifests/support_matrix/current.json",
                "docs/specs/unified-agent-api/support-matrix.md",
                "docs/specs/unified-agent-api/capability-matrix.md"
            ],
            "runtime_evidence_paths": [
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/input-contract.json",
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/run-status.json",
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/run-summary.md",
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/validation-report.json",
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/written-paths.json",
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/handoff.json"
            ],
            "publication_owned_paths": [
                LIFECYCLE_STATE_PATH,
                PUBLICATION_PACKET_PATH
            ],
            "blocking_issues": [],
            "implementation_summary": {
                "requested_runtime_profile": "default",
                "achieved_runtime_profile": "default",
                "primary_template": "gemini_cli",
                "template_lineage": ["gemini_cli"],
                "landed_surfaces": [
                    "wrapper_runtime",
                    "backend_harness",
                    "runtime_manifest_evidence"
                ],
                "deferred_surfaces": [],
                "minimal_profile_justification": Value::Null
            }
        })
    };
    let lifecycle_json = |packet_sha256: &str| {
        json!({
            "schema_version": "1",
            "agent_id": "gemini_cli",
            "onboarding_pack_prefix": "gemini-cli-onboarding",
            "approval_artifact_path": approval_path,
            "approval_artifact_sha256": approval_sha256,
            "lifecycle_stage": "published",
            "support_tier": "publication_backed",
            "side_states": [],
            "current_owner_command": "refresh-publication --write",
            "expected_next_command": agent_lifecycle::published_prepare_closeout_command(
                approval_path
            ),
            "last_transition_at": "2026-05-01T00:00:00Z",
            "last_transition_by": "xtask refresh-publication --write",
            "required_evidence": required_evidence_json(agent_lifecycle::LifecycleStage::Published),
            "satisfied_evidence": required_evidence_json(agent_lifecycle::LifecycleStage::Published),
            "blocking_issues": [],
            "retryable_failures": [],
            "active_runtime_evidence_run_id": Value::Null,
            "implementation_summary": {
                "requested_runtime_profile": "default",
                "achieved_runtime_profile": "default",
                "primary_template": "gemini_cli",
                "template_lineage": ["gemini_cli"],
                "landed_surfaces": [
                    "wrapper_runtime",
                    "backend_harness",
                    "runtime_manifest_evidence"
                ],
                "deferred_surfaces": [],
                "minimal_profile_justification": Value::Null
            },
            "publication_packet_path": PUBLICATION_PACKET_PATH,
            "publication_packet_sha256": packet_sha256,
            "closeout_baseline_path": Value::Null
        })
    };

    write_text(
        &packet_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&packet_json(
                "0000000000000000000000000000000000000000000000000000000000000000",
            ))
            .expect("serialize publication-ready packet")
        ),
    );
    let mut packet_sha256 = sha256_hex(&packet_path);
    write_text(
        &lifecycle_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&lifecycle_json(&packet_sha256))
                .expect("serialize lifecycle state")
        ),
    );
    let lifecycle_sha256 = sha256_hex(&lifecycle_path);
    write_text(
        &packet_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&packet_json(&lifecycle_sha256))
                .expect("serialize publication-ready packet")
        ),
    );
    packet_sha256 = sha256_hex(&packet_path);
    write_text(
        &lifecycle_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&lifecycle_json(&packet_sha256))
                .expect("serialize lifecycle state")
        ),
    );
}

fn required_evidence_json(stage: agent_lifecycle::LifecycleStage) -> Value {
    Value::Array(
        agent_lifecycle::required_evidence_for_stage(stage)
            .iter()
            .map(|evidence| Value::String(evidence.as_str().to_string()))
            .collect(),
    )
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read json")).expect("parse json")
}

fn write_json(path: &Path, value: &Value) {
    write_text(
        path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(value).expect("serialize json")
        ),
    );
}

#[test]
fn prepare_proving_run_closeout_help_text_includes_required_surface() {
    let top_help = Cli::command().render_help().to_string();
    assert!(top_help.contains("prepare-proving-run-closeout"));

    let err = Cli::try_parse_from(["xtask", "prepare-proving-run-closeout", "--help"])
        .expect_err("subcommand help should short-circuit parsing");
    assert_eq!(err.exit_code(), 0);
    let help_text = err.to_string();
    assert!(help_text.contains("--approval"));
    assert!(help_text.contains("--check"));
    assert!(help_text.contains("--write"));
}

#[test]
fn prepare_proving_run_closeout_check_reports_closeout_path_without_writing_files() {
    let fixture = prepare_published_fixture("prepare-closeout-check");

    let output = run_cli(prepare_args("--check"), &fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert!(output
        .stdout
        .contains("OK: prepare-proving-run-closeout check passed."));
    assert!(output
        .stdout
        .contains(&format!("closeout_path: {CLOSEOUT_PATH}")));
    assert!(!fixture.join(CLOSEOUT_PATH).exists());
    assert!(!fixture.join(README_PATH).exists());
    assert!(!fixture.join(HANDOFF_PATH).exists());
}

#[test]
fn prepare_proving_run_closeout_write_refreshes_prepared_closeout_and_packet_docs() {
    let fixture = prepare_published_fixture("prepare-closeout-write");

    let output = run_cli(prepare_args("--write"), &fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert!(output
        .stdout
        .contains("OK: prepare-proving-run-closeout write complete."));

    let closeout = read_json(&fixture.join(CLOSEOUT_PATH));
    assert_eq!(
        closeout.get("state").and_then(Value::as_str),
        Some("prepared")
    );
    assert_eq!(
        closeout.get("approval_ref").and_then(Value::as_str),
        Some(APPROVAL_PATH)
    );
    assert_eq!(
        closeout.get("approval_source").and_then(Value::as_str),
        Some("governance-review")
    );
    assert_eq!(
        closeout.get("preflight_passed").and_then(Value::as_bool),
        Some(true)
    );

    let lifecycle = read_json(&fixture.join(LIFECYCLE_STATE_PATH));
    assert_eq!(
        lifecycle.get("lifecycle_stage").and_then(Value::as_str),
        Some("published")
    );
    assert_eq!(
        lifecycle
            .get("current_owner_command")
            .and_then(Value::as_str),
        Some("prepare-proving-run-closeout --write")
    );
    assert_eq!(
        lifecycle
            .get("expected_next_command")
            .and_then(Value::as_str),
        Some(
            agent_lifecycle::publication_ready_closeout_command(
                APPROVAL_PATH,
                "gemini-cli-onboarding",
            )
            .as_str()
        )
    );
    assert_eq!(
        lifecycle.get("last_transition_by").and_then(Value::as_str),
        Some("xtask prepare-proving-run-closeout --write")
    );

    let readme = fs::read_to_string(fixture.join(README_PATH)).expect("read README");
    let handoff = fs::read_to_string(fixture.join(HANDOFF_PATH)).expect("read HANDOFF");
    assert!(readme.contains("- Packet state: `closeout_prepared`"));
    assert!(readme.contains("the proving run is not yet closed"));
    assert!(handoff.contains("- state: `prepared`"));
    assert!(handoff.contains("The proving run is not yet closed."));
}

#[test]
fn prepare_proving_run_closeout_second_write_preserves_human_fields_and_repropagates_docs() {
    let fixture = prepare_published_fixture("prepare-closeout-second-write");
    let closeout_path = fixture.join(CLOSEOUT_PATH);

    let first = run_cli(prepare_args("--write"), &fixture);
    assert_eq!(first.exit_code, 0, "stderr:\n{}", first.stderr);

    let mut customized = read_json(&closeout_path);
    customized["manual_control_plane_edits"] = Value::from(5);
    customized["partial_write_incidents"] = Value::from(2);
    customized["ambiguous_ownership_incidents"] = Value::from(1);
    customized["duration_seconds"] = Value::from(37);
    customized["duration_missing_reason"] = Value::Null;
    customized["residual_friction"] = Value::Array(vec![Value::String(
        "Manual coordination still required.".to_string(),
    )]);
    customized["explicit_none_reason"] = Value::Null;
    write_json(&closeout_path, &customized);
    write_text(&fixture.join(HANDOFF_PATH), "stale handoff\n");

    let second = run_cli(prepare_args("--write"), &fixture);
    assert_eq!(second.exit_code, 0, "stderr:\n{}", second.stderr);

    let prepared = read_json(&closeout_path);
    assert_eq!(
        prepared.get("state").and_then(Value::as_str),
        Some("prepared")
    );
    assert_eq!(
        prepared
            .get("manual_control_plane_edits")
            .and_then(Value::as_u64),
        Some(5)
    );
    assert_eq!(
        prepared
            .get("partial_write_incidents")
            .and_then(Value::as_u64),
        Some(2)
    );
    assert_eq!(
        prepared
            .get("ambiguous_ownership_incidents")
            .and_then(Value::as_u64),
        Some(1)
    );
    assert_eq!(
        prepared.get("duration_seconds").and_then(Value::as_u64),
        Some(37)
    );
    assert_eq!(
        prepared
            .get("residual_friction")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(Value::as_str),
        Some("Manual coordination still required.")
    );

    let handoff = fs::read_to_string(fixture.join(HANDOFF_PATH)).expect("read HANDOFF");
    assert!(!handoff.contains("stale handoff"));
    assert!(handoff.contains("- manual control-plane file edits by maintainers: `5`"));
    assert!(handoff.contains("- partial-write incidents: `2`"));
    assert!(handoff.contains("- ambiguous ownership incidents: `1`"));
    assert!(handoff.contains("- approved-agent to repo-ready control-plane mutation time: `37s`"));
    assert!(handoff.contains("- Manual coordination still required."));
}

#[test]
fn prepare_proving_run_closeout_rejects_non_published_lifecycle_state_without_writing() {
    let fixture = prepare_published_fixture("prepare-closeout-reject-non-published");
    let lifecycle_path = fixture.join(LIFECYCLE_STATE_PATH);
    let mut lifecycle = read_json(&lifecycle_path);
    lifecycle["lifecycle_stage"] = Value::String("publication_ready".to_string());
    lifecycle["support_tier"] = Value::String("baseline_runtime".to_string());
    lifecycle["current_owner_command"] = Value::String("prepare-publication --write".to_string());
    lifecycle["expected_next_command"] = Value::String(
        agent_lifecycle::publication_ready_refresh_command(APPROVAL_PATH),
    );
    lifecycle["last_transition_by"] =
        Value::String("xtask prepare-publication --write".to_string());
    lifecycle["required_evidence"] =
        required_evidence_json(agent_lifecycle::LifecycleStage::PublicationReady);
    lifecycle["satisfied_evidence"] =
        required_evidence_json(agent_lifecycle::LifecycleStage::PublicationReady);
    write_json(&lifecycle_path, &lifecycle);
    write_text(&fixture.join(README_PATH), "sentinel readme\n");
    write_text(&fixture.join(HANDOFF_PATH), "sentinel handoff\n");

    let output = run_cli(prepare_args("--write"), &fixture);

    assert_eq!(output.exit_code, 2, "stdout:\n{}", output.stdout);
    assert!(output
        .stderr
        .contains("prepare-proving-run-closeout requires lifecycle stage `published`"));
    assert!(!fixture.join(CLOSEOUT_PATH).exists());
    assert_eq!(
        fs::read_to_string(fixture.join(README_PATH)).expect("read README"),
        "sentinel readme\n"
    );
    assert_eq!(
        fs::read_to_string(fixture.join(HANDOFF_PATH)).expect("read HANDOFF"),
        "sentinel handoff\n"
    );
}
