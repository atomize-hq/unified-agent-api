use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use serde_json::Value;
use xtask::{
    agent_lifecycle::{load_lifecycle_state, LifecycleStage, PublicationReadyPacket},
    approval_artifact::load_approval_artifact,
    prepare_publication::validate_runtime_evidence_run_for_approval,
    proving_run_closeout,
};

#[allow(dead_code)]
#[path = "../src/historical_lifecycle_backfill.rs"]
mod historical_lifecycle_backfill;

#[path = "support/onboard_agent_harness.rs"]
mod harness;

use harness::{fixture_root, seed_gemini_approval_artifact, sha256_hex, HarnessOutput};

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Project automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    HistoricalLifecycleBackfill(historical_lifecycle_backfill::Args),
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
                Command::HistoricalLifecycleBackfill(args) => {
                    match historical_lifecycle_backfill::run_in_workspace(
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

fn prepare_fixture(prefix: &str) -> (PathBuf, String) {
    let fixture = fixture_root(prefix);
    let approval_path = seed_gemini_approval_artifact(
        &fixture,
        "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml",
        "gemini-cli-onboarding",
    );
    let approval_sha = sha256_hex(&fixture.join(&approval_path));

    write_json(
        &fixture
            .join("docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json"),
        &serde_json::json!({
            "schema_version": "1",
            "agent_id": "gemini_cli",
            "onboarding_pack_prefix": "gemini-cli-onboarding",
            "approval_artifact_path": approval_path,
            "approval_artifact_sha256": approval_sha,
            "lifecycle_stage": "closed_baseline",
            "support_tier": "publication_backed",
            "side_states": [],
            "current_owner_command": "historical-lifecycle-backfill",
            "expected_next_command": "none",
            "last_transition_at": "2026-04-21T11:23:09Z",
            "last_transition_by": "historical-lifecycle-backfill-entrypoint-test",
            "required_evidence": [
                "registry_entry",
                "docs_pack",
                "manifest_root_skeleton",
                "runtime_write_complete",
                "implementation_summary_present",
                "publication_packet_written",
                "proving_run_closeout_written"
            ],
            "satisfied_evidence": [
                "registry_entry",
                "docs_pack",
                "manifest_root_skeleton",
                "runtime_write_complete",
                "implementation_summary_present",
                "publication_packet_written",
                "proving_run_closeout_written"
            ],
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
            "publication_packet_path": "docs/agents/lifecycle/gemini-cli-onboarding/governance/publication-ready.json",
            "publication_packet_sha256": "deadbeef",
            "closeout_baseline_path": "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json"
        }),
    );

    write_text(
        &fixture.join("crates/gemini_cli/src/lib.rs"),
        "#![forbid(unsafe_code)]\n",
    );
    write_text(
        &fixture.join("crates/agent_api/src/backends/gemini_cli/backend.rs"),
        "#![forbid(unsafe_code)]\n",
    );
    write_text(
        &fixture.join("crates/agent_api/src/backends/gemini_cli/mod.rs"),
        "#![forbid(unsafe_code)]\n",
    );
    write_text(
        &fixture.join("crates/agent_api/src/backends/gemini_cli/harness.rs"),
        "#![forbid(unsafe_code)]\n",
    );
    write_text(
        &fixture.join("crates/agent_api/src/backends/gemini_cli/mapping.rs"),
        "#![forbid(unsafe_code)]\n",
    );
    write_text(
        &fixture.join("crates/agent_api/src/backends/gemini_cli/util.rs"),
        "#![forbid(unsafe_code)]\n",
    );
    write_text(
        &fixture.join("crates/agent_api/src/backends/gemini_cli/internal/ignored.rs"),
        "#![forbid(unsafe_code)]\n",
    );
    write_text(
        &fixture.join("crates/gemini_cli/src/wrapper_coverage_manifest.rs"),
        "#![forbid(unsafe_code)]\n",
    );
    write_text(
        &fixture.join("cli_manifests/gemini_cli/snapshots/default.json"),
        "{}\n",
    );
    write_text(
        &fixture.join("cli_manifests/gemini_cli/snapshots/union.json"),
        "{}\n",
    );
    write_text(
        &fixture.join("cli_manifests/gemini_cli/supplement/notes.md"),
        "# Notes\n",
    );
    write_text(
        &fixture.join("cli_manifests/gemini_cli/supplement/commands.md"),
        "# Commands\n",
    );

    (fixture, approval_path)
}

#[test]
fn historical_lifecycle_backfill_rebuilds_multi_file_runtime_evidence_and_downstream_packet() {
    let (fixture, approval_path) = prepare_fixture("historical-lifecycle-backfill-multi-file");
    let output = run_cli(
        [
            "xtask",
            "historical-lifecycle-backfill",
            "--agent",
            "gemini_cli",
        ],
        &fixture,
    );
    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);

    let approval =
        load_approval_artifact(&fixture, &approval_path).expect("load approval after backfill");
    let runtime = validate_runtime_evidence_run_for_approval(
        &fixture,
        &approval,
        "historical-gemini_cli-runtime-follow-on",
    )
    .expect("validate rebuilt runtime evidence");
    assert_eq!(runtime.run_id, "historical-gemini_cli-runtime-follow-on");

    let run_root = fixture.join(
        "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-gemini_cli-runtime-follow-on",
    );
    let written_paths: Vec<String> = serde_json::from_slice(
        &fs::read(run_root.join("written-paths.json")).expect("read written paths"),
    )
    .expect("parse written paths");
    assert!(
        written_paths.contains(&"crates/agent_api/src/backends/gemini_cli/backend.rs".to_string())
    );
    assert!(written_paths.contains(&"crates/agent_api/src/backends/gemini_cli/mod.rs".to_string()));
    assert!(
        written_paths.contains(&"crates/agent_api/src/backends/gemini_cli/harness.rs".to_string())
    );
    assert!(
        written_paths.contains(&"crates/agent_api/src/backends/gemini_cli/mapping.rs".to_string())
    );
    assert!(written_paths.contains(&"crates/agent_api/src/backends/gemini_cli/util.rs".to_string()));
    assert!(!written_paths
        .contains(&"crates/agent_api/src/backends/gemini_cli/internal/ignored.rs".to_string()));
    assert!(written_paths.contains(&"cli_manifests/gemini_cli/snapshots/default.json".to_string()));
    assert!(written_paths.contains(&"cli_manifests/gemini_cli/snapshots/union.json".to_string()));
    assert!(written_paths.contains(&"cli_manifests/gemini_cli/supplement/notes.md".to_string()));
    assert!(written_paths.contains(&"cli_manifests/gemini_cli/supplement/commands.md".to_string()));

    let lifecycle = load_lifecycle_state(
        &fixture,
        "docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json",
    )
    .expect("load lifecycle");
    assert_eq!(lifecycle.lifecycle_stage, LifecycleStage::ClosedBaseline);

    let packet_bytes = fs::read(
        fixture
            .join("docs/agents/lifecycle/gemini-cli-onboarding/governance/publication-ready.json"),
    )
    .expect("read publication packet");
    let packet: PublicationReadyPacket =
        serde_json::from_slice(&packet_bytes).expect("parse publication packet");
    packet
        .validate()
        .expect("validate publication packet schema");
    assert_eq!(
        packet.runtime_evidence_paths,
        runtime.runtime_evidence_paths
    );
    assert_eq!(
        lifecycle.publication_packet_path.as_deref(),
        Some("docs/agents/lifecycle/gemini-cli-onboarding/governance/publication-ready.json")
    );
    let closeout_rel =
        "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json";
    let closeout_path = fixture.join(closeout_rel);
    let closeout = proving_run_closeout::load_validated_closeout_with_states(
        &fixture,
        Path::new(closeout_rel),
        &closeout_path,
        proving_run_closeout::ProvingRunCloseoutExpected {
            approval_path: Some(Path::new(&approval_path)),
            onboarding_pack_prefix: "gemini-cli-onboarding",
        },
        proving_run_closeout::ProvingRunCloseoutState::all(),
    )
    .expect("load closeout through shared parser");
    assert_eq!(closeout.state.as_str(), "closed");
    assert_eq!(
        fs::read_to_string(&closeout_path).expect("read closeout"),
        proving_run_closeout::render_closeout_json(&closeout)
            .expect("render closeout through shared serializer")
    );
}

fn write_json(path: &Path, value: &Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize json");
    bytes.push(b'\n');
    fs::write(path, bytes).expect("write json");
}

fn write_text(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write text");
}
