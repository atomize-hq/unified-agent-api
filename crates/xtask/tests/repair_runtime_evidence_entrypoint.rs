use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use serde_json::Value;
use xtask::{
    approval_artifact::load_approval_artifact,
    prepare_publication::discover_runtime_evidence_for_approval, repair_runtime_evidence,
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
        &fixture.join("cli_manifests/gemini_cli/supplement/notes.md"),
        "# Supplement\n",
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
    assert!(output.stdout.contains("written_paths: 6"));
}

#[test]
fn repair_runtime_evidence_write_emits_bundle_without_advancing_lifecycle() {
    let (fixture, approval_path) = prepare_fixture("repair-runtime-evidence-write");
    let lifecycle_path =
        fixture.join("docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json");
    let before_lifecycle = fs::read(&lifecycle_path).expect("read lifecycle before write");

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
    assert!(!written_paths.is_empty());

    let after_lifecycle = fs::read(&lifecycle_path).expect("read lifecycle after write");
    assert_eq!(
        after_lifecycle, before_lifecycle,
        "repair should not mutate lifecycle state"
    );

    let approval =
        load_approval_artifact(&fixture, &approval_path).expect("load approval after repair");
    let discovered = discover_runtime_evidence_for_approval(&fixture, &approval)
        .expect("discover repaired runtime evidence");
    assert_eq!(discovered.run_id, "repair-gemini_cli-runtime-follow-on");
    assert_eq!(discovered.runtime_evidence_paths.len(), 6);
}

#[test]
fn repair_runtime_evidence_check_fails_when_runtime_outputs_cannot_be_derived() {
    let (fixture, approval_path) = prepare_fixture("repair-runtime-evidence-missing-outputs");
    for path in [
        "crates/gemini_cli/src/lib.rs",
        "crates/agent_api/src/backends/gemini_cli/backend.rs",
        "crates/gemini_cli/src/wrapper_coverage_manifest.rs",
        "crates/agent_api/tests/c1_gemini_cli_runtime_follow_on.rs",
        "cli_manifests/gemini_cli/snapshots/default.json",
        "cli_manifests/gemini_cli/supplement/notes.md",
    ] {
        fs::remove_file(fixture.join(path)).expect("remove runtime-owned output");
    }

    let output = run_cli(repair_args("--check", &approval_path), &fixture);
    assert_eq!(output.exit_code, 2);
    assert!(output
        .stderr
        .contains("could not derive any committed runtime-owned outputs"));
}

fn write_json(path: &Path, value: &Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize json");
    bytes.push(b'\n');
    fs::write(path, bytes).expect("write json");
}
