use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use clap::{CommandFactory, Parser, Subcommand};
use serde_json::Value;
use xtask::{
    agent_lifecycle, prepare_publication,
    publication_refresh::{self, CAPABILITY_MATRIX_OUTPUT_PATH},
};

#[path = "support/agent_maintenance_drift_harness.rs"]
mod drift_harness;
#[path = "support/onboard_agent_harness.rs"]
mod harness;

use drift_harness::{
    seed_gemini_runtime_integrated_state, seed_publication_inputs, seed_runtime_evidence_run,
    RuntimeEvidenceTruth,
};
use harness::{fixture_root, replace_text_once, sha256_hex, HarnessOutput};

const APPROVAL_PATH: &str =
    "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
const LIFECYCLE_STATE_PATH: &str =
    "docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json";
const PUBLICATION_PACKET_PATH: &str =
    "docs/agents/lifecycle/gemini-cli-onboarding/governance/publication-ready.json";
const REGISTRY_PATH: &str = "crates/xtask/data/agent_registry.toml";

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
    RefreshPublication(publication_refresh::Args),
}

#[derive(Clone, Copy)]
struct PublicationFlags {
    support_enabled: bool,
    capability_enabled: bool,
}

const SUPPORT_AND_CAPABILITY: PublicationFlags = PublicationFlags {
    support_enabled: true,
    capability_enabled: true,
};
const SUPPORT_ONLY: PublicationFlags = PublicationFlags {
    support_enabled: true,
    capability_enabled: false,
};
const CAPABILITY_ONLY: PublicationFlags = PublicationFlags {
    support_enabled: false,
    capability_enabled: true,
};

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
                Command::RefreshPublication(args) => {
                    match publication_refresh::run_in_workspace(workspace_root, args, &mut stdout) {
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

fn prepare_publication_ready_fixture(prefix: &str, flags: PublicationFlags) -> PathBuf {
    let fixture = fixture_root(prefix);
    seed_publication_inputs(&fixture);
    write_fixture_workspace_manifest(&fixture);
    apply_publication_flags(&fixture, flags);
    seed_gemini_runtime_integrated_state(&fixture);
    seed_runtime_evidence_run(&fixture, RuntimeEvidenceTruth::Truthful);

    let prepare_output = run_cli(
        [
            "xtask",
            "prepare-publication",
            "--write",
            "--approval",
            APPROVAL_PATH,
        ],
        &fixture,
    );
    assert_eq!(
        prepare_output.exit_code, 0,
        "stderr:\n{}",
        prepare_output.stderr
    );

    fixture
}

fn write_fixture_workspace_manifest(fixture: &Path) {
    fs::write(
        fixture.join("Cargo.toml"),
        concat!(
            "[workspace]\n",
            "members = [\n",
            "  \"crates/agent_api\",\n",
            "  \"crates/codex\",\n",
            "  \"crates/claude_code\",\n",
            "  \"crates/opencode\",\n",
            "  \"crates/gemini_cli\",\n",
            "  \"crates/wrapper_events\",\n",
            "  \"crates/xtask\",\n",
            "]\n",
            "resolver = \"2\"\n",
            "\n",
            "[workspace.package]\n",
            "version = \"0.3.0\"\n",
            "edition = \"2021\"\n",
            "rust-version = \"1.78\"\n",
            "license = \"MIT OR Apache-2.0\"\n",
            "authors = [\"Unified Agent API Contributors\"]\n",
        ),
    )
    .expect("write fixture workspace manifest");
    fs::write(fixture.join("crates/xtask/src/main.rs"), "fn main() {}\n")
        .expect("write fixture xtask binary");
}

fn apply_publication_flags(fixture: &Path, flags: PublicationFlags) {
    replace_text_once(
        &fixture.join(APPROVAL_PATH),
        "support_matrix_enabled = true\n",
        &format!("support_matrix_enabled = {}\n", flags.support_enabled),
    );
    replace_text_once(
        &fixture.join(APPROVAL_PATH),
        "capability_matrix_enabled = true\n",
        &format!("capability_matrix_enabled = {}\n", flags.capability_enabled),
    );

    let registry_path = fixture.join(REGISTRY_PATH);
    let registry = fs::read_to_string(&registry_path).expect("read registry");
    let gemini_block = concat!(
        "agent_id = \"gemini_cli\"\n",
        "display_name = \"Gemini CLI\"\n",
        "crate_path = \"crates/gemini_cli\"\n",
        "backend_module = \"crates/agent_api/src/backends/gemini_cli\"\n",
        "manifest_root = \"cli_manifests/gemini_cli\"\n",
        "package_name = \"unified-agent-api-gemini-cli\"\n",
        "canonical_targets = [\"darwin-arm64\"]\n",
        "\n",
        "[agents.wrapper_coverage]\n",
        "binding_kind = \"generated_from_wrapper_crate\"\n",
        "source_path = \"crates/gemini_cli\"\n",
        "\n",
        "[agents.capability_declaration]\n",
        "always_on = [\"agent_api.config.model.v1\", \"agent_api.events\", \"agent_api.events.live\", \"agent_api.run\"]\n",
        "backend_extensions = []\n",
        "[agents.publication]\n",
        "support_matrix_enabled = true\n",
        "capability_matrix_enabled = true\n",
    );
    let replacement = format!(
        concat!(
            "agent_id = \"gemini_cli\"\n",
            "display_name = \"Gemini CLI\"\n",
            "crate_path = \"crates/gemini_cli\"\n",
            "backend_module = \"crates/agent_api/src/backends/gemini_cli\"\n",
            "manifest_root = \"cli_manifests/gemini_cli\"\n",
            "package_name = \"unified-agent-api-gemini-cli\"\n",
            "canonical_targets = [\"darwin-arm64\"]\n",
            "\n",
            "[agents.wrapper_coverage]\n",
            "binding_kind = \"generated_from_wrapper_crate\"\n",
            "source_path = \"crates/gemini_cli\"\n",
            "\n",
            "[agents.capability_declaration]\n",
            "always_on = [\"agent_api.config.model.v1\", \"agent_api.events\", \"agent_api.events.live\", \"agent_api.run\"]\n",
            "backend_extensions = []\n",
            "[agents.publication]\n",
            "support_matrix_enabled = {support}\n",
            "capability_matrix_enabled = {capability}\n",
        ),
        support = flags.support_enabled,
        capability = flags.capability_enabled,
    );
    let matches = registry.matches(gemini_block).count();
    assert_eq!(matches, 1, "expected one gemini publication block");
    fs::write(
        &registry_path,
        registry.replacen(gemini_block, &replacement, 1),
    )
    .expect("write registry");
}

fn refresh_args(mode_flag: &str) -> Vec<String> {
    vec![
        "xtask".to_string(),
        "refresh-publication".to_string(),
        mode_flag.to_string(),
        "--approval".to_string(),
        APPROVAL_PATH.to_string(),
    ]
}

fn lifecycle_state_path(fixture: &Path) -> PathBuf {
    fixture.join(LIFECYCLE_STATE_PATH)
}

fn publication_packet_path(fixture: &Path) -> PathBuf {
    fixture.join(PUBLICATION_PACKET_PATH)
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read json")).expect("parse json")
}

fn expected_output_paths(flags: PublicationFlags) -> Vec<String> {
    publication_refresh::expected_publication_output_paths(
        flags.support_enabled,
        flags.capability_enabled,
    )
}

fn seed_stale_output(fixture: &Path, path: &str) {
    let absolute = fixture.join(path);
    if let Some(parent) = absolute.parent() {
        fs::create_dir_all(parent).expect("create output parent");
    }
    fs::write(absolute, "stale publication output\n").expect("write stale publication output");
}

fn seed_publication_output_state(fixture: &Path, flags: PublicationFlags) {
    if flags.support_enabled {
        seed_stale_output(
            fixture,
            publication_refresh::SUPPORT_MATRIX_JSON_OUTPUT_PATH,
        );
        seed_stale_output(
            fixture,
            publication_refresh::SUPPORT_MATRIX_MARKDOWN_OUTPUT_PATH,
        );
    }

    if flags.capability_enabled {
        seed_stale_output(fixture, CAPABILITY_MATRIX_OUTPUT_PATH);
    }
}

fn write_preflight_makefile(fixture: &Path, should_pass: bool) {
    let recipe = if should_pass {
        "preflight:\n\t@true\n"
    } else {
        "preflight:\n\t@false\n"
    };
    fs::write(fixture.join("Makefile"), recipe).expect("write Makefile");
}

#[test]
fn refresh_publication_help_text_includes_required_surface() {
    let top_help = Cli::command().render_help().to_string();
    assert!(top_help.contains("refresh-publication"));

    let err = Cli::try_parse_from(["xtask", "refresh-publication", "--help"])
        .expect_err("subcommand help should short-circuit parsing");
    assert_eq!(err.exit_code(), 0);
    let help_text = err.to_string();
    assert!(help_text.contains("--approval"));
    assert!(help_text.contains("--check"));
    assert!(help_text.contains("--write"));
}

#[test]
fn refresh_publication_write_advances_to_closeout_and_refreshes_outputs() {
    let fixture =
        prepare_publication_ready_fixture("refresh-publication-success", SUPPORT_AND_CAPABILITY);
    seed_publication_output_state(&fixture, SUPPORT_AND_CAPABILITY);
    write_preflight_makefile(&fixture, true);

    let output = run_cli(refresh_args("--write"), &fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert!(output
        .stdout
        .contains("OK: refresh-publication write complete."));
    let lifecycle_state = read_json(&lifecycle_state_path(&fixture));
    assert_eq!(
        lifecycle_state
            .get("lifecycle_stage")
            .and_then(Value::as_str),
        Some("published")
    );
    assert_eq!(
        lifecycle_state.get("support_tier").and_then(Value::as_str),
        Some("publication_backed")
    );
    assert_eq!(
        lifecycle_state
            .get("current_owner_command")
            .and_then(Value::as_str),
        Some("refresh-publication --write")
    );
    assert_eq!(
        lifecycle_state
            .get("expected_next_command")
            .and_then(Value::as_str),
        Some(
            agent_lifecycle::publication_ready_closeout_command(
                APPROVAL_PATH,
                "gemini-cli-onboarding",
            )
            .as_str(),
        )
    );
    assert_eq!(
        lifecycle_state
            .get("last_transition_by")
            .and_then(Value::as_str),
        Some("xtask refresh-publication --write")
    );
    assert_eq!(
        lifecycle_state
            .get("publication_packet_path")
            .and_then(Value::as_str),
        Some(PUBLICATION_PACKET_PATH)
    );
    assert_eq!(
        lifecycle_state.get("closeout_baseline_path"),
        Some(&Value::Null)
    );

    let packet_sha = sha256_hex(&publication_packet_path(&fixture));
    let packet = read_json(&publication_packet_path(&fixture));
    assert_eq!(
        lifecycle_state
            .get("publication_packet_sha256")
            .and_then(Value::as_str),
        Some(packet_sha.as_str())
    );
    assert_eq!(
        packet.get("lifecycle_stage").and_then(Value::as_str),
        Some("publication_ready")
    );
    assert_eq!(
        packet
            .get("required_publication_outputs")
            .and_then(Value::as_array)
            .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>()),
        Some(
            expected_output_paths(SUPPORT_AND_CAPABILITY)
                .iter()
                .map(String::as_str)
                .collect()
        )
    );

    for path in expected_output_paths(SUPPORT_AND_CAPABILITY) {
        let absolute = fixture.join(&path);
        assert!(absolute.is_file(), "missing {path}");
        assert_ne!(
            fs::read_to_string(&absolute).expect("read refreshed output"),
            "stale publication output\n"
        );
    }
}

#[test]
fn refresh_publication_check_reports_stale_outputs() {
    let fixture = prepare_publication_ready_fixture(
        "refresh-publication-stale-check",
        SUPPORT_AND_CAPABILITY,
    );
    seed_publication_output_state(&fixture, SUPPORT_AND_CAPABILITY);

    let output = run_cli(refresh_args("--check"), &fixture);

    assert_eq!(output.exit_code, 2, "stderr:\n{}", output.stderr);
    assert!(
        output
            .stderr
            .contains("publication-owned outputs are stale"),
        "stderr:\n{}",
        output.stderr
    );
    assert!(output
        .stderr
        .contains(publication_refresh::SUPPORT_MATRIX_JSON_OUTPUT_PATH));
    assert!(output.stderr.contains(CAPABILITY_MATRIX_OUTPUT_PATH));
    assert_eq!(
        read_json(&lifecycle_state_path(&fixture))
            .get("expected_next_command")
            .and_then(Value::as_str),
        Some(agent_lifecycle::publication_ready_refresh_command(APPROVAL_PATH).as_str())
    );
}

#[test]
fn refresh_publication_write_support_only_updates_only_support_outputs() {
    let fixture =
        prepare_publication_ready_fixture("refresh-publication-support-only", SUPPORT_ONLY);
    seed_publication_output_state(&fixture, SUPPORT_ONLY);
    let capability_before =
        fs::read(fixture.join(CAPABILITY_MATRIX_OUTPUT_PATH)).expect("read capability before");
    write_preflight_makefile(&fixture, true);

    let output = run_cli(refresh_args("--write"), &fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    for path in [
        publication_refresh::SUPPORT_MATRIX_JSON_OUTPUT_PATH,
        publication_refresh::SUPPORT_MATRIX_MARKDOWN_OUTPUT_PATH,
    ] {
        assert!(fixture.join(path).is_file(), "missing {path}");
    }
    assert_eq!(
        fs::read(fixture.join(CAPABILITY_MATRIX_OUTPUT_PATH)).expect("read capability after"),
        capability_before
    );
    let packet = read_json(&publication_packet_path(&fixture));
    assert_eq!(
        packet
            .get("required_publication_outputs")
            .and_then(Value::as_array)
            .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>()),
        Some(
            expected_output_paths(SUPPORT_ONLY)
                .iter()
                .map(String::as_str)
                .collect()
        )
    );
}

#[test]
fn refresh_publication_write_capability_only_updates_only_capability_output() {
    let fixture =
        prepare_publication_ready_fixture("refresh-publication-capability-only", CAPABILITY_ONLY);
    seed_publication_output_state(&fixture, CAPABILITY_ONLY);
    let support_json_before =
        fs::read(fixture.join(publication_refresh::SUPPORT_MATRIX_JSON_OUTPUT_PATH))
            .expect("read support json before");
    let support_markdown_before =
        fs::read(fixture.join(publication_refresh::SUPPORT_MATRIX_MARKDOWN_OUTPUT_PATH))
            .expect("read support markdown before");
    write_preflight_makefile(&fixture, true);

    let output = run_cli(refresh_args("--write"), &fixture);

    assert_eq!(output.exit_code, 0, "stderr:\n{}", output.stderr);
    assert!(fixture.join(CAPABILITY_MATRIX_OUTPUT_PATH).is_file());
    assert_eq!(
        fs::read(fixture.join(publication_refresh::SUPPORT_MATRIX_JSON_OUTPUT_PATH))
            .expect("read support json after"),
        support_json_before
    );
    assert_eq!(
        fs::read(fixture.join(publication_refresh::SUPPORT_MATRIX_MARKDOWN_OUTPUT_PATH))
            .expect("read support markdown after"),
        support_markdown_before
    );
    let packet = read_json(&publication_packet_path(&fixture));
    assert_eq!(
        packet
            .get("required_publication_outputs")
            .and_then(Value::as_array)
            .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>()),
        Some(
            expected_output_paths(CAPABILITY_ONLY)
                .iter()
                .map(String::as_str)
                .collect()
        )
    );
}

#[test]
fn refresh_publication_write_rolls_back_output_mutations_when_gate_fails() {
    let fixture = prepare_publication_ready_fixture(
        "refresh-publication-output-rollback",
        SUPPORT_AND_CAPABILITY,
    );
    seed_publication_output_state(&fixture, SUPPORT_AND_CAPABILITY);
    write_preflight_makefile(&fixture, false);
    let before = expected_output_paths(SUPPORT_AND_CAPABILITY)
        .into_iter()
        .map(|path| {
            let contents = fs::read(fixture.join(&path)).expect("read output before");
            (path, contents)
        })
        .collect::<BTreeMap<_, _>>();

    let output = run_cli(refresh_args("--write"), &fixture);

    assert_eq!(output.exit_code, 2);
    assert!(output.stderr.contains("publication gate command"));
    for (path, before_contents) in before {
        let after_contents = fs::read(fixture.join(&path)).expect("read output after");
        assert_eq!(
            after_contents, before_contents,
            "gate failure must roll back output writes for {path}"
        );
    }
}

#[test]
fn refresh_publication_write_rolls_back_lifecycle_updates_when_gate_fails() {
    let fixture = prepare_publication_ready_fixture(
        "refresh-publication-lifecycle-rollback",
        SUPPORT_AND_CAPABILITY,
    );
    seed_publication_output_state(&fixture, SUPPORT_AND_CAPABILITY);
    let lifecycle_before =
        fs::read_to_string(lifecycle_state_path(&fixture)).expect("read lifecycle before");
    let packet_before =
        fs::read_to_string(publication_packet_path(&fixture)).expect("read packet before");
    write_preflight_makefile(&fixture, false);

    let output = run_cli(refresh_args("--write"), &fixture);

    assert_eq!(output.exit_code, 2);
    assert_eq!(
        fs::read_to_string(lifecycle_state_path(&fixture)).expect("read lifecycle after"),
        lifecycle_before
    );
    assert_eq!(
        fs::read_to_string(publication_packet_path(&fixture)).expect("read packet after"),
        packet_before
    );
}

#[test]
fn refresh_publication_rejects_rerun_after_published_handoff_is_set() {
    let fixture =
        prepare_publication_ready_fixture("refresh-publication-idempotent", SUPPORT_AND_CAPABILITY);
    seed_publication_output_state(&fixture, SUPPORT_AND_CAPABILITY);
    write_preflight_makefile(&fixture, true);

    let first = run_cli(refresh_args("--write"), &fixture);
    assert_eq!(first.exit_code, 0, "stderr:\n{}", first.stderr);
    let outputs_after_first = expected_output_paths(SUPPORT_AND_CAPABILITY)
        .into_iter()
        .map(|path| {
            let contents = fs::read(fixture.join(&path)).expect("read first-pass output");
            (path, contents)
        })
        .collect::<BTreeMap<_, _>>();

    let second = run_cli(refresh_args("--write"), &fixture);

    assert_eq!(second.exit_code, 2, "stderr:\n{}", second.stderr);
    assert!(second
        .stderr
        .contains("requires lifecycle stage `publication_ready`"));
    let lifecycle_state = read_json(&lifecycle_state_path(&fixture));
    assert_eq!(
        lifecycle_state
            .get("expected_next_command")
            .and_then(Value::as_str),
        Some(
            agent_lifecycle::publication_ready_closeout_command(
                APPROVAL_PATH,
                "gemini-cli-onboarding",
            )
            .as_str(),
        )
    );
    for (path, first_contents) in outputs_after_first {
        let second_contents = fs::read(fixture.join(&path)).expect("read second-pass output");
        assert_eq!(
            second_contents, first_contents,
            "output drifted on rerun: {path}"
        );
    }
}
