use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use serde_json::Value;
use xtask::{
    approval_artifact::load_approval_artifact,
    prepare_publication::validate_runtime_evidence_run_for_approval, repair_runtime_evidence,
};

#[path = "support/onboard_agent_harness.rs"]
mod harness;

use harness::{fixture_root, seed_gemini_approval_artifact, sha256_hex, write_text, HarnessOutput};

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Project automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    RepairRuntimeEvidence(repair_runtime_evidence::Args),
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
                Command::RepairRuntimeEvidence(args) => {
                    match repair_runtime_evidence::run_in_workspace(
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
            "lifecycle_stage": "runtime_integrated",
            "support_tier": "baseline_runtime",
            "side_states": [],
            "current_owner_command": "runtime-follow-on --write",
            "expected_next_command": "prepare-publication --approval docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml --write",
            "last_transition_at": "2026-05-01T00:00:00Z",
            "last_transition_by": "repair-runtime-evidence-entrypoint-test",
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
            "active_runtime_evidence_run_id": "gemini-cli-runtime-follow-on-rerun",
            "implementation_summary": {
                "requested_runtime_profile": "default",
                "achieved_runtime_profile": "default",
                "primary_template": "opencode",
                "template_lineage": ["opencode"],
                "landed_surfaces": [
                    "wrapper_runtime",
                    "backend_harness",
                    "wrapper_coverage_source",
                    "runtime_manifest_evidence",
                    "agent_api_onboarding_test"
                ],
                "deferred_surfaces": [],
                "minimal_profile_justification": Value::Null
            },
            "publication_packet_path": Value::Null,
            "publication_packet_sha256": Value::Null,
            "closeout_baseline_path": Value::Null
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
        &fixture.join("crates/agent_api/tests/c1_gemini_cli_runtime_follow_on.rs"),
        "#[test]\nfn smoke() {}\n",
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
        "# Supplement\n",
    );
    write_text(
        &fixture.join("cli_manifests/gemini_cli/supplement/commands.md"),
        "# Commands\n",
    );

    (fixture, approval_path)
}

fn repair_args(mode_flag: &str, approval_path: &str) -> Vec<String> {
    vec![
        "xtask".to_string(),
        "repair-runtime-evidence".to_string(),
        mode_flag.to_string(),
        "--approval".to_string(),
        approval_path.to_string(),
    ]
}

fn repair_run_root(fixture: &Path) -> PathBuf {
    fixture.join("docs/agents/.uaa-temp/runtime-follow-on/runs/repair-gemini_cli-runtime-follow-on")
}

#[test]
fn repair_runtime_evidence_command_requires_one_mode() {
    let fixture = fixture_root("repair-runtime-evidence-mode");
    let output = run_cli(
        [
            "xtask",
            "repair-runtime-evidence",
            "--approval",
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml",
        ],
        &fixture,
    );
    assert_ne!(output.exit_code, 0);
    assert!(output.stderr.contains("--check"));
    assert!(output.stderr.contains("--write"));
}

#[test]
fn repair_runtime_evidence_check_reports_repairable_bundle() {
    let (fixture, approval_path) = prepare_fixture("repair-runtime-evidence-check");

    let output = run_cli(repair_args("--check", &approval_path), &fixture);
    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert!(output
        .stdout
        .contains("OK: repair-runtime-evidence check passed."));
    assert!(output
        .stdout
        .contains("run_id: repair-gemini_cli-runtime-follow-on"));
    assert!(output.stdout.contains("written_paths: 12"));
    assert_no_staging_dirs(&fixture);
}

#[test]
fn repair_runtime_evidence_write_emits_bundle_without_advancing_lifecycle() {
    let (fixture, approval_path) = prepare_fixture("repair-runtime-evidence-write");
    let lifecycle_path =
        fixture.join("docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json");
    let before_lifecycle = read_json(&lifecycle_path);
    let expected_prepare_publication_handoff =
        "prepare-publication --approval docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml --write";

    let output = run_cli(repair_args("--write", &approval_path), &fixture);
    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);

    let repair_root = repair_run_root(&fixture);
    for name in [
        "input-contract.json",
        "run-status.json",
        "validation-report.json",
        "handoff.json",
        "written-paths.json",
        "run-summary.md",
    ] {
        assert!(repair_root.join(name).is_file(), "missing {name}");
    }

    let written_paths: Vec<String> = serde_json::from_slice(
        &fs::read(repair_root.join("written-paths.json")).expect("read written paths"),
    )
    .expect("parse written paths");
    assert_eq!(written_paths.len(), 12);
    assert!(written_paths.contains(&"cli_manifests/gemini_cli/snapshots/default.json".to_string()));
    assert!(written_paths.contains(&"cli_manifests/gemini_cli/snapshots/union.json".to_string()));
    assert!(written_paths.contains(&"cli_manifests/gemini_cli/supplement/notes.md".to_string()));
    assert!(written_paths.contains(&"cli_manifests/gemini_cli/supplement/commands.md".to_string()));
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

    let run_status: Value = serde_json::from_slice(
        &fs::read(repair_root.join("run-status.json")).expect("read run status"),
    )
    .expect("parse run status");
    assert_eq!(
        run_status.get("run_dir").and_then(Value::as_str),
        Some(repair_root.to_string_lossy().as_ref())
    );

    let after_lifecycle = read_json(&lifecycle_path);
    assert_eq!(
        after_lifecycle
            .get("lifecycle_stage")
            .and_then(Value::as_str),
        before_lifecycle
            .get("lifecycle_stage")
            .and_then(Value::as_str),
        "repair should not advance lifecycle stage"
    );
    assert_eq!(
        after_lifecycle.get("support_tier").and_then(Value::as_str),
        before_lifecycle.get("support_tier").and_then(Value::as_str),
    );
    assert_eq!(
        after_lifecycle
            .get("expected_next_command")
            .and_then(Value::as_str),
        Some(expected_prepare_publication_handoff),
    );
    assert_eq!(
        after_lifecycle
            .get("active_runtime_evidence_run_id")
            .and_then(Value::as_str),
        Some("repair-gemini_cli-runtime-follow-on")
    );
    assert_eq!(
        after_lifecycle
            .get("current_owner_command")
            .and_then(Value::as_str),
        Some("repair-runtime-evidence --write")
    );
    assert_eq!(
        after_lifecycle
            .get("last_transition_by")
            .and_then(Value::as_str),
        Some("xtask repair-runtime-evidence --write")
    );
    assert_ne!(
        after_lifecycle
            .get("last_transition_at")
            .and_then(Value::as_str),
        before_lifecycle
            .get("last_transition_at")
            .and_then(Value::as_str),
    );

    let approval =
        load_approval_artifact(&fixture, &approval_path).expect("load approval after repair");
    let discovered = validate_runtime_evidence_run_for_approval(
        &fixture,
        &approval,
        "repair-gemini_cli-runtime-follow-on",
    )
    .expect("validate repaired runtime evidence");
    assert_eq!(discovered.run_id, "repair-gemini_cli-runtime-follow-on");
    assert_eq!(discovered.runtime_evidence_paths.len(), 6);
}

#[test]
fn repair_runtime_evidence_write_replaces_existing_canonical_bundle_without_staging_leaks() {
    let (fixture, approval_path) = prepare_fixture("repair-runtime-evidence-replace-existing");
    let repair_root = repair_run_root(&fixture);
    write_text(
        &repair_root.join("run-summary.md"),
        "# Old Repair Bundle\n\nThis should be replaced.\n",
    );
    write_json(
        &repair_root.join("written-paths.json"),
        &serde_json::json!(["stale/path.txt"]),
    );

    let output = run_cli(repair_args("--write", &approval_path), &fixture);
    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);

    let summary = fs::read_to_string(repair_root.join("run-summary.md")).expect("read summary");
    assert!(summary.contains("Runtime Evidence Repair"));
    assert!(!summary.contains("Old Repair Bundle"));

    let runs_root = fixture.join("docs/agents/.uaa-temp/runtime-follow-on/runs");
    let leaked = fs::read_dir(&runs_root)
        .expect("read runs root")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| {
            name.starts_with(".tmp-repair-gemini_cli") || name.starts_with(".bak-repair-gemini_cli")
        })
        .collect::<Vec<_>>();
    assert!(leaked.is_empty(), "unexpected staging dirs: {leaked:?}");
}

#[test]
fn repair_runtime_evidence_write_rolls_back_bundle_when_lifecycle_persist_fails() {
    fn fail_lifecycle_persist(
        _workspace_root: &Path,
        _relative_path: &str,
        _state: &xtask::agent_lifecycle::LifecycleState,
    ) -> Result<(), String> {
        Err("simulated lifecycle persist failure".to_string())
    }

    let (fixture, approval_path) = prepare_fixture("repair-runtime-evidence-lifecycle-failure");
    let lifecycle_path =
        fixture.join("docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json");
    let before_lifecycle = fs::read(&lifecycle_path).expect("read lifecycle before failure");

    let repair_root = repair_run_root(&fixture);
    write_text(
        &repair_root.join("run-summary.md"),
        "# Existing Repair Bundle\n\nMust survive rollback.\n",
    );
    write_json(
        &repair_root.join("written-paths.json"),
        &serde_json::json!(["preserved/path.txt"]),
    );

    repair_runtime_evidence::set_test_lifecycle_persist_mutator(Some(fail_lifecycle_persist));
    let output = run_cli(repair_args("--write", &approval_path), &fixture);
    repair_runtime_evidence::set_test_lifecycle_persist_mutator(None);

    assert_eq!(output.exit_code, 1, "stderr:\n{}", output.stderr);
    assert!(output
        .stderr
        .contains("simulated lifecycle persist failure"));

    let after_lifecycle = fs::read(&lifecycle_path).expect("read lifecycle after failure");
    assert_eq!(
        after_lifecycle, before_lifecycle,
        "lifecycle should be restored"
    );

    let summary = fs::read_to_string(repair_root.join("run-summary.md")).expect("read summary");
    assert!(summary.contains("Existing Repair Bundle"));
    let written_paths: Vec<String> = serde_json::from_slice(
        &fs::read(repair_root.join("written-paths.json")).expect("read written paths"),
    )
    .expect("parse written paths");
    assert_eq!(written_paths, vec!["preserved/path.txt".to_string()]);
    assert_no_staging_dirs(&fixture);
}

#[test]
fn repair_runtime_evidence_check_uses_runtime_evidence_validator_and_cleans_up_staging() {
    fn corrupt_run_dir(run_root: &Path) -> Result<(), String> {
        let run_status_path = run_root.join("run-status.json");
        let mut run_status = read_json(&run_status_path);
        run_status["run_dir"] = Value::String("/tmp/not-the-real-run-root".to_string());
        write_json(&run_status_path, &run_status);
        Ok(())
    }

    let (fixture, approval_path) = prepare_fixture("repair-runtime-evidence-check-validator");
    repair_runtime_evidence::set_test_stage_mutator(Some(corrupt_run_dir));
    let output = run_cli(repair_args("--check", &approval_path), &fixture);
    repair_runtime_evidence::set_test_stage_mutator(None);

    assert_eq!(output.exit_code, 2, "stderr:\n{}", output.stderr);
    assert!(output
        .stderr
        .contains("recorded run_dir `/tmp/not-the-real-run-root`"));
    assert_no_staging_dirs(&fixture);
}

#[test]
fn repair_runtime_evidence_check_fails_when_runtime_outputs_cannot_be_derived() {
    let (fixture, approval_path) = prepare_fixture("repair-runtime-evidence-missing-outputs");
    for path in [
        "crates/gemini_cli/src/lib.rs",
        "crates/agent_api/src/backends/gemini_cli/backend.rs",
        "crates/agent_api/src/backends/gemini_cli/mod.rs",
        "crates/agent_api/src/backends/gemini_cli/harness.rs",
        "crates/agent_api/src/backends/gemini_cli/mapping.rs",
        "crates/agent_api/src/backends/gemini_cli/util.rs",
        "crates/gemini_cli/src/wrapper_coverage_manifest.rs",
        "crates/agent_api/tests/c1_gemini_cli_runtime_follow_on.rs",
        "cli_manifests/gemini_cli/snapshots/default.json",
        "cli_manifests/gemini_cli/snapshots/union.json",
        "cli_manifests/gemini_cli/supplement/notes.md",
        "cli_manifests/gemini_cli/supplement/commands.md",
    ] {
        fs::remove_file(fixture.join(path)).expect("remove runtime-owned output");
    }

    let output = run_cli(repair_args("--check", &approval_path), &fixture);
    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("could not derive any committed runtime-owned outputs"));
}

fn assert_no_staging_dirs(fixture: &Path) {
    let runs_root = fixture.join("docs/agents/.uaa-temp/runtime-follow-on/runs");
    let leaked = fs::read_dir(&runs_root)
        .expect("read runs root")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| {
            name.starts_with(".tmp-repair-gemini_cli") || name.starts_with(".bak-repair-gemini_cli")
        })
        .collect::<Vec<_>>();
    assert!(leaked.is_empty(), "unexpected staging dirs: {leaked:?}");
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read json")).expect("parse json")
}

fn write_json(path: &Path, value: &Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize json");
    bytes.push(b'\n');
    fs::write(path, bytes).expect("write json");
}
