use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use clap::{CommandFactory, Parser, Subcommand};
use serde_json::Value;
use xtask::{agent_registry::AgentRegistry, prepare_publication};

#[path = "support/onboard_agent_harness.rs"]
mod harness;

use harness::{
    fixture_root, seed_gemini_approval_artifact, sha256_hex, snapshot_files, write_text,
    HarnessOutput,
};

const RUNTIME_RUNS_ROOT: &str = "docs/agents/.uaa-temp/runtime-follow-on/runs";
const RUN_ID: &str = "rtfo-publication";
const LEGACY_REQUIRED_PUBLICATION_COMMANDS: [&str; 4] = [
    "support-matrix --check",
    "capability-matrix --check",
    "capability-matrix-audit",
    "make preflight",
];

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Project automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    PreparePublication(prepare_publication::Args),
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
                Command::PreparePublication(args) => {
                    match prepare_publication::run_in_workspace(workspace_root, args, &mut stdout) {
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
    let registry = AgentRegistry::load(&fixture).expect("load seeded registry");
    let entry = registry
        .find("gemini_cli")
        .expect("gemini_cli entry present");

    write_json(
        &fixture
            .join("docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json"),
        &serde_json::json!({
            "schema_version": "1",
            "agent_id": "gemini_cli",
            "onboarding_pack_prefix": "gemini-cli-onboarding",
            "approval_artifact_path": approval_path,
            "approval_artifact_sha256": approval_sha,
            "lifecycle_stage": "runtime_integrated",
            "support_tier": "baseline_runtime",
            "side_states": [],
            "current_owner_command": "runtime-follow-on --write",
            "expected_next_command": "prepare-publication --approval docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml --write",
            "last_transition_at": "2026-05-01T00:00:00Z",
            "last_transition_by": "prepare-publication-entrypoint-test",
            "required_evidence": [
                "registry_entry",
                "docs_pack",
                "manifest_root_skeleton",
                "runtime_write_complete",
                "implementation_summary_present"
            ],
            "satisfied_evidence": [
                "registry_entry",
                "docs_pack",
                "manifest_root_skeleton",
                "runtime_write_complete",
                "implementation_summary_present"
            ],
            "blocking_issues": [],
            "retryable_failures": [],
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
            "publication_packet_path": Value::Null,
            "publication_packet_sha256": Value::Null,
            "closeout_baseline_path": Value::Null
        }),
    );
    write_json(
        &fixture.join("cli_manifests/gemini_cli/current.json"),
        &serde_json::json!({
            "expected_targets": entry.canonical_targets,
            "commands": []
        }),
    );

    let run_root = fixture.join(RUNTIME_RUNS_ROOT).join(RUN_ID);
    let run_root_display = run_root.display().to_string();
    write_json(
        &run_root.join("input-contract.json"),
        &serde_json::json!({
            "approval_artifact_path": "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml",
            "approval_artifact_sha256": approval_sha,
            "agent_id": "gemini_cli",
            "manifest_root": "cli_manifests/gemini_cli",
            "required_handoff_commands": [
                "cargo run -p xtask -- support-matrix --check",
                "cargo run -p xtask -- capability-matrix --check",
                "cargo run -p xtask -- capability-matrix-audit",
                "make preflight"
            ]
        }),
    );
    write_json(
        &run_root.join("run-status.json"),
        &serde_json::json!({
            "approval_artifact_path": "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml",
            "agent_id": "gemini_cli",
            "status": "write_validated",
            "validation_passed": true,
            "handoff_ready": true,
            "run_dir": run_root_display
        }),
    );
    write_json(
        &run_root.join("validation-report.json"),
        &serde_json::json!({
            "status": "pass"
        }),
    );
    write_json(
        &run_root.join("handoff.json"),
        &serde_json::json!({
            "agent_id": "gemini_cli",
            "manifest_root": "cli_manifests/gemini_cli",
            "runtime_lane_complete": true,
            "publication_refresh_required": true,
            "required_commands": [
                "cargo run -p xtask -- support-matrix --check",
                "cargo run -p xtask -- capability-matrix --check",
                "cargo run -p xtask -- capability-matrix-audit",
                "make preflight"
            ],
            "blockers": []
        }),
    );
    write_json(
        &run_root.join("written-paths.json"),
        &serde_json::json!([
            "crates/gemini_cli/src/lib.rs",
            "cli_manifests/gemini_cli/snapshots/default.json"
        ]),
    );
    write_text(
        &run_root.join("run-summary.md"),
        "# Runtime Summary\n\nValidated runtime follow-on output.\n",
    );

    (fixture, approval_path)
}

fn publication_args(mode_flag: &str, approval_path: &str) -> Vec<String> {
    vec![
        "xtask".to_string(),
        "prepare-publication".to_string(),
        mode_flag.to_string(),
        "--approval".to_string(),
        approval_path.to_string(),
    ]
}

fn publication_packet_path(fixture: &Path) -> PathBuf {
    fixture.join("docs/agents/lifecycle/gemini-cli-onboarding/governance/publication-ready.json")
}

fn lifecycle_state_path(fixture: &Path) -> PathBuf {
    fixture.join("docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json")
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read json")).expect("parse json")
}

fn write_json(path: &Path, value: &Value) {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize json");
    bytes.push(b'\n');
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, bytes).expect("write json");
}

fn snapshot_without_runtime_runs(root: &Path) -> BTreeMap<String, Vec<u8>> {
    snapshot_files(root)
        .into_iter()
        .filter(|(path, _)| !path.starts_with(RUNTIME_RUNS_ROOT))
        .collect()
}

#[test]
fn prepare_publication_help_text_includes_required_surface() {
    let top_help = Cli::command().render_help().to_string();
    assert!(top_help.contains("prepare-publication"));

    let err = Cli::try_parse_from(["xtask", "prepare-publication", "--help"])
        .expect_err("subcommand help should short-circuit parsing");
    assert_eq!(err.exit_code(), 0);
    let help_text = err.to_string();
    assert!(help_text.contains("--approval"));
    assert!(help_text.contains("--check"));
    assert!(help_text.contains("--write"));
}

#[test]
fn prepare_publication_write_emits_packet_and_advances_lifecycle() {
    let (fixture, approval_path) = prepare_fixture("prepare-publication-success");

    let output = run_cli(publication_args("--write", &approval_path), &fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    let lifecycle_state = read_json(&lifecycle_state_path(&fixture));
    assert_eq!(
        lifecycle_state
            .get("lifecycle_stage")
            .and_then(Value::as_str),
        Some("publication_ready")
    );
    assert_eq!(
        lifecycle_state.get("support_tier").and_then(Value::as_str),
        Some("baseline_runtime")
    );
    assert_eq!(
        lifecycle_state
            .get("expected_next_command")
            .and_then(Value::as_str),
        Some("support-matrix --check && capability-matrix --check && capability-matrix-audit && make preflight && close-proving-run --write")
    );
    assert!(lifecycle_state
        .get("satisfied_evidence")
        .and_then(Value::as_array)
        .expect("satisfied_evidence array")
        .iter()
        .any(|value| value.as_str() == Some("publication_packet_written")));
    assert!(lifecycle_state
        .get("publication_packet_path")
        .is_some_and(Value::is_null));

    let packet_path = publication_packet_path(&fixture);
    assert!(packet_path.is_file(), "publication-ready packet must exist");
    let packet = read_json(&packet_path);
    assert_eq!(
        packet.get("lifecycle_stage").and_then(Value::as_str),
        Some("publication_ready")
    );
    assert_eq!(
        packet
            .get("required_commands")
            .and_then(Value::as_array)
            .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>()),
        Some(vec![
            "cargo run -p xtask -- support-matrix --check",
            "cargo run -p xtask -- capability-matrix --check",
            "cargo run -p xtask -- capability-matrix-audit",
            "make preflight"
        ])
    );
    assert_eq!(
        packet
            .get("required_publication_outputs")
            .and_then(Value::as_array)
            .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>()),
        Some(vec![
            "cli_manifests/support_matrix/current.json",
            "docs/specs/unified-agent-api/support-matrix.md",
            "docs/specs/unified-agent-api/capability-matrix.md"
        ])
    );
    assert!(packet
        .get("runtime_evidence_paths")
        .and_then(Value::as_array)
        .expect("runtime_evidence_paths array")
        .iter()
        .any(|value| value.as_str()
            == Some("docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/handoff.json")));

    let lifecycle_sha = sha256_hex(&lifecycle_state_path(&fixture));
    assert_eq!(
        packet.get("lifecycle_state_sha256").and_then(Value::as_str),
        Some(lifecycle_sha.as_str())
    );
    assert!(!fixture
        .join("cli_manifests/support_matrix/current.json")
        .exists());
    assert!(!fixture
        .join("docs/specs/unified-agent-api/support-matrix.md")
        .exists());
    assert!(!fixture
        .join("docs/specs/unified-agent-api/capability-matrix.md")
        .exists());
}

#[test]
fn prepare_publication_check_revalidates_existing_packet_without_rewriting() {
    let (fixture, approval_path) = prepare_fixture("prepare-publication-check");
    let write_output = run_cli(publication_args("--write", &approval_path), &fixture);
    assert_eq!(
        write_output.exit_code, 0,
        "stderr:\n{}",
        write_output.stderr
    );
    let before = snapshot_without_runtime_runs(&fixture);

    let output = run_cli(publication_args("--check", &approval_path), &fixture);

    let after = snapshot_without_runtime_runs(&fixture);
    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert_eq!(
        before, after,
        "check mode must not rewrite committed surfaces"
    );
}

#[test]
fn prepare_publication_rejects_missing_runtime_evidence() {
    let (fixture, approval_path) = prepare_fixture("prepare-publication-missing-evidence");
    fs::remove_file(
        fixture
            .join(RUNTIME_RUNS_ROOT)
            .join(RUN_ID)
            .join("run-status.json"),
    )
    .expect("remove run-status");

    let output = run_cli(publication_args("--write", &approval_path), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("no successful runtime-follow-on evidence run"));
    assert_eq!(
        read_json(&lifecycle_state_path(&fixture))
            .get("lifecycle_stage")
            .and_then(Value::as_str),
        Some("runtime_integrated")
    );
    assert!(!publication_packet_path(&fixture).exists());
}

#[test]
fn prepare_publication_rejects_legacy_runtime_input_commands_with_repair_guidance() {
    let (fixture, approval_path) = prepare_fixture("prepare-publication-legacy-input-commands");
    let input_contract_path = fixture
        .join(RUNTIME_RUNS_ROOT)
        .join(RUN_ID)
        .join("input-contract.json");
    let mut input_contract = read_json(&input_contract_path);
    input_contract["required_handoff_commands"] = Value::Array(
        LEGACY_REQUIRED_PUBLICATION_COMMANDS
            .iter()
            .map(|value| Value::String((*value).to_string()))
            .collect(),
    );
    write_json(&input_contract_path, &input_contract);

    let output = run_cli(publication_args("--write", &approval_path), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains(
        "runtime input-contract required_handoff_commands must match the frozen publication command set exactly"
    ));
    assert!(output.stderr.contains(
        "cargo run -p xtask -- repair-runtime-evidence --approval docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml --write"
    ));
    assert_eq!(
        read_json(&lifecycle_state_path(&fixture))
            .get("lifecycle_stage")
            .and_then(Value::as_str),
        Some("runtime_integrated")
    );
    assert!(!publication_packet_path(&fixture).exists());
}

#[test]
fn prepare_publication_rejects_legacy_runtime_handoff_commands_with_repair_guidance() {
    let (fixture, approval_path) = prepare_fixture("prepare-publication-legacy-handoff-commands");
    let handoff_path = fixture
        .join(RUNTIME_RUNS_ROOT)
        .join(RUN_ID)
        .join("handoff.json");
    let mut handoff = read_json(&handoff_path);
    handoff["required_commands"] = Value::Array(
        LEGACY_REQUIRED_PUBLICATION_COMMANDS
            .iter()
            .map(|value| Value::String((*value).to_string()))
            .collect(),
    );
    write_json(&handoff_path, &handoff);

    let output = run_cli(publication_args("--write", &approval_path), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains(
        "runtime handoff required_commands must match the frozen publication command set exactly"
    ));
    assert!(output.stderr.contains(
        "cargo run -p xtask -- repair-runtime-evidence --approval docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml --write"
    ));
    assert_eq!(
        read_json(&lifecycle_state_path(&fixture))
            .get("lifecycle_stage")
            .and_then(Value::as_str),
        Some("runtime_integrated")
    );
    assert!(!publication_packet_path(&fixture).exists());
}

#[test]
fn validate_runtime_evidence_run_rejects_path_sensitive_run_dir_mismatch() {
    let (fixture, approval_path) = prepare_fixture("prepare-publication-run-dir-mismatch");
    let run_status_path = fixture
        .join(RUNTIME_RUNS_ROOT)
        .join(RUN_ID)
        .join("run-status.json");
    let mut run_status = read_json(&run_status_path);
    run_status["run_dir"] = Value::String("/tmp/not-the-real-run-root".to_string());
    write_json(&run_status_path, &run_status);

    let approval = xtask::approval_artifact::load_approval_artifact(&fixture, &approval_path)
        .expect("approval");
    let err = prepare_publication::validate_runtime_evidence_run_for_approval(
        &fixture, &approval, RUN_ID,
    )
    .expect_err("run_dir mismatch should fail");
    assert!(err
        .to_string()
        .contains("recorded run_dir `/tmp/not-the-real-run-root`"));
}

#[test]
fn prepare_publication_rejects_capability_continuity_drift() {
    let (fixture, approval_path) = prepare_fixture("prepare-publication-capability-drift");
    write_json(
        &fixture.join("cli_manifests/gemini_cli/current.json"),
        &serde_json::json!({
            "expected_targets": [],
            "commands": []
        }),
    );

    let output = run_cli(publication_args("--write", &approval_path), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("must declare at least one expected target"));
    assert_eq!(
        read_json(&lifecycle_state_path(&fixture))
            .get("lifecycle_stage")
            .and_then(Value::as_str),
        Some("runtime_integrated")
    );
    assert!(!publication_packet_path(&fixture).exists());
}
