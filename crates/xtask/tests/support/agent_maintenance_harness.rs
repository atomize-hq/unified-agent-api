#![allow(dead_code)]

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use serde_json::Value;
use sha2::{Digest, Sha256};
use xtask::agent_maintenance::execute;

use crate::{
    harness::{
        fixture_root, seed_gemini_approval_artifact, seed_release_touchpoints, write_text,
        HarnessOutput,
    },
    release_doc, support_matrix,
};

pub const EXECUTE_RUNS_ROOT: &str = "docs/agents/.uaa-temp/agent-maintenance/runs";
pub const EXECUTE_WRITE_RUN_ID: &str = "am-write";
pub const FAKE_EXECUTE_CODEX_SCENARIO_FILE: &str = "fake-agent-maintenance-codex-scenario.txt";
pub const FAKE_EXECUTE_CODEX_LOG_FILE: &str = "fake-agent-maintenance-codex-invocations.log";
pub const GATE_ORDER_LOG_FILE: &str = "gate-order.log";

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Project automation tasks")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    ExecuteAgentMaintenance(execute::Args),
}

pub fn seed_opencode_basis(root: &Path) {
    seed_release_touchpoints(root);
    write_text(
        &root.join("docs/specs/unified-agent-api/support-matrix.md"),
        "# Support matrix\n\nManual contract text.\n",
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/non-tui-support-debt.md"),
        "# Non-TUI Support Debt Inventory\n\n## Inventory\n",
    );
    seed_publishable_workspace_member(root, "crates/gemini_cli", "unified-agent-api-gemini-cli");
    seed_cli_manifest_root(
        root,
        "cli_manifests/codex",
        &["x86_64-unknown-linux-musl"],
        &[(&["mcp", "list"], &["x86_64-unknown-linux-musl"])],
    );
    seed_cli_manifest_root(
        root,
        "cli_manifests/claude_code",
        &["linux-x64"],
        &[(&["mcp", "list"], &["linux-x64"])],
    );
    seed_cli_manifest_root(
        root,
        "cli_manifests/opencode",
        &["linux-x64", "darwin-arm64", "win32-x64"],
        &[],
    );
    write_text(
        &root.join("cli_manifests/opencode/latest_validated.txt"),
        "1.4.11\n",
    );
    write_text(
        &root.join("cli_manifests/opencode/artifacts.lock.json"),
        "{\n  \"schema_version\": 1\n}\n",
    );
    write_text(
        &root.join("cli_manifests/opencode/wrapper_coverage.json"),
        "{\n  \"schema_version\": 1\n}\n",
    );
    write_text(
        &root.join("cli_manifests/opencode/versions/1.14.47.json"),
        "{\n  \"semantic_version\": \"1.14.47\"\n}\n",
    );
    write_text(
        &root.join("cli_manifests/opencode/reports/1.14.47/coverage.any.json"),
        "{\n  \"deltas\": {\n    \"missing_commands\": [],\n    \"missing_flags\": [],\n    \"missing_args\": [],\n    \"intentionally_unsupported\": []\n  }\n}\n",
    );
    seed_cli_manifest_root(root, "cli_manifests/gemini_cli", &["darwin-arm64"], &[]);
    seed_cli_manifest_root(root, "cli_manifests/aider", &["darwin-arm64"], &[]);

    let support_bundle =
        support_matrix::generate_publication_artifacts(root).expect("generate support publication");
    write_text(
        &root.join("cli_manifests/support_matrix/current.json"),
        &support_bundle.json,
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/support-matrix.md"),
        &support_bundle.markdown,
    );
    write_text(
        &root.join("crates/agent_api/src/runtime_support_data.rs"),
        &support_bundle.runtime_support_data,
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/capability-matrix.md"),
        &default_capability_matrix_markdown(),
    );

    seed_gemini_approval_artifact(
        root,
        "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml",
        "gemini-cli-onboarding",
    );
    seed_clean_governance_closeouts(root);

    let release_doc = release_doc::render_release_doc(root).expect("render release doc");
    write_text(&root.join(release_doc::RELEASE_DOC_PATH), &release_doc);
}

pub fn overwrite_opencode_governance_with_stale_claim(root: &Path) {
    write_text(
        &root.join(
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
        "# Closeout\n\n- capability advertisement is intentionally conservative and now matches the landed backend contract and generated capability inventory:\n  <!-- xtask-governance-check:opencode-capabilities:start -->\n  `agent_api.events`, `agent_api.events.live`, `agent_api.run`\n  <!-- xtask-governance-check:opencode-capabilities:end -->\n  are the claimed OpenCode v1 capability ids under the current runtime evidence\n",
    );
}

fn seed_clean_governance_closeouts(root: &Path) {
    write_text(
        &root.join(
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
        "# Closeout\n\n- capability advertisement is intentionally conservative and now matches the landed backend contract and generated capability inventory:\n  <!-- xtask-governance-check:opencode-capabilities:start -->\n  `agent_api.config.model.v1`, `agent_api.events`, `agent_api.events.live`, `agent_api.run`, `agent_api.session.fork.v1`, `agent_api.session.resume.v1`\n  <!-- xtask-governance-check:opencode-capabilities:end -->\n  are the claimed OpenCode v1 capability ids under the current runtime evidence\n",
    );
    write_text(
        &root.join(
            "docs/integrations/opencode/governance/seam-3-closeout.md",
        ),
        "# Closeout\n\n- the support publication artifacts now show OpenCode as manifest-supported only where committed root evidence justifies it, while\n  <!-- xtask-governance-check:opencode-support:start -->\n  backend_support = supported\n  uaa_support = supported\n  <!-- xtask-governance-check:opencode-support:end -->\n  under the current backend evidence and pointer posture\n",
    );
}

fn seed_publishable_workspace_member(root: &Path, member_path: &str, package_name: &str) {
    write_text(
        &root.join(member_path).join("Cargo.toml"),
        &format!("[package]\nname = \"{package_name}\"\nversion = \"0.2.3\"\nedition = \"2021\"\n"),
    );
}

fn seed_cli_manifest_root(
    root: &Path,
    manifest_root: &str,
    canonical_targets: &[&str],
    commands: &[(&[&str], &[&str])],
) {
    let current = serde_json::json!({
        "expected_targets": canonical_targets,
        "inputs": canonical_targets
            .iter()
            .map(|target| serde_json::json!({
                "target_triple": target,
                "binary": { "semantic_version": "1.0.0" }
            }))
            .collect::<Vec<_>>(),
        "commands": commands
            .iter()
            .map(|(path, available_on)| serde_json::json!({
                "path": path,
                "available_on": available_on,
            }))
            .collect::<Vec<_>>(),
    });
    write_text(
        &root.join(manifest_root).join("current.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&current).expect("serialize current manifest")
        ),
    );

    let version = serde_json::json!({
        "semantic_version": "1.0.0",
        "status": "supported",
        "coverage": { "supported_targets": canonical_targets },
    });
    write_text(
        &root.join(manifest_root).join("versions/1.0.0.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&version).expect("serialize version metadata")
        ),
    );

    for target in canonical_targets {
        write_text(
            &root
                .join(manifest_root)
                .join(format!("pointers/latest_supported/{target}.txt")),
            "1.0.0\n",
        );
        write_text(
            &root
                .join(manifest_root)
                .join(format!("pointers/latest_validated/{target}.txt")),
            "1.0.0\n",
        );
        let report = serde_json::json!({
            "inputs": { "upstream": { "targets": [target] } },
            "deltas": {
                "missing_commands": [],
                "missing_flags": [],
                "missing_args": [],
                "intentionally_unsupported": [],
                "wrapper_only_commands": [],
                "wrapper_only_flags": [],
                "wrapper_only_args": [],
            }
        });
        write_text(
            &root
                .join(manifest_root)
                .join(format!("reports/1.0.0/coverage.{target}.json")),
            &format!(
                "{}\n",
                serde_json::to_string_pretty(&report).expect("serialize report")
            ),
        );
    }
}

fn default_capability_matrix_markdown() -> String {
    include_str!("../../../../docs/specs/unified-agent-api/capability-matrix.md").to_string()
}

pub fn run_execute_cli<I, S>(argv: I, workspace_root: &Path) -> HarnessOutput
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
                Command::ExecuteAgentMaintenance(args) => {
                    match execute::run_in_workspace(workspace_root, args, &mut stdout) {
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

pub fn prepare_execute_fixture(prefix: &str) -> PathBuf {
    let fixture = fixture_root(prefix);
    seed_registry(&fixture);
    seed_execute_support_files(&fixture);
    write_text(
        &fixture
            .join("docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml"),
        &execution_request_toml(EXECUTE_WRITE_RUN_ID),
    );
    write_text(&fixture.join("Cargo.toml"), "[workspace]\nmembers = []\n");
    write_text(&fixture.join("gate-command.sh"), &gate_command_script());
    mark_executable(&fixture.join("gate-command.sh"));
    fixture
}

pub fn execute_args(mode_flag: &str, codex_binary: Option<&Path>) -> Vec<String> {
    let mut args = vec![
        "xtask".to_string(),
        "execute-agent-maintenance".to_string(),
        mode_flag.to_string(),
        "--request".to_string(),
        "docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml".to_string(),
    ];
    args.extend(["--run-id".to_string(), EXECUTE_WRITE_RUN_ID.to_string()]);
    if let Some(binary) = codex_binary {
        args.extend(["--codex-binary".to_string(), binary.display().to_string()]);
    }
    args
}

pub fn fake_execute_codex_binary(fixture: &Path) -> PathBuf {
    let binary = fixture.join("fake-agent-maintenance-codex.sh");
    if !binary.is_file() {
        write_text(&binary, &fake_execute_codex_script());
        mark_executable(&binary);
    }
    binary
}

pub fn write_fake_execute_codex_scenario(fixture: &Path, scenario: &str) {
    write_text(
        &fixture
            .join(EXECUTE_RUNS_ROOT)
            .join(EXECUTE_WRITE_RUN_ID)
            .join(FAKE_EXECUTE_CODEX_SCENARIO_FILE),
        &format!("{scenario}\n"),
    );
}

pub fn write_fake_execute_codex_preflight_scenario(fixture: &Path, scenario: &str) {
    write_text(
        &fixture.join(FAKE_EXECUTE_CODEX_SCENARIO_FILE),
        &format!("{scenario}\n"),
    );
}

pub fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read json")).expect("parse json")
}

pub fn snapshot_without_execute_runs(root: &Path) -> BTreeMap<String, Vec<u8>> {
    crate::harness::snapshot_files(root)
        .into_iter()
        .filter(|(path, _)| !path.starts_with(EXECUTE_RUNS_ROOT))
        .collect()
}

fn seed_registry(root: &Path) {
    write_text(
        &root.join("crates/xtask/data/agent_registry.toml"),
        include_str!("../../data/agent_registry.toml"),
    );
}

fn seed_execute_support_files(root: &Path) {
    let registry =
        xtask::agent_registry::AgentRegistry::parse(include_str!("../../data/agent_registry.toml"))
            .expect("parse seeded registry");
    let entry = registry.find("codex").expect("codex registry entry");
    write_text(
        &root.join(".github/workflows/agent-maintenance-open-pr.yml"),
        "name: Packet PR worker\n",
    );
    write_text(
        &root.join("docs/agents/lifecycle/codex-maintenance/governance/execute-agent-maintenance-prompt.md"),
        &xtask::agent_maintenance::contract_policy::packet_pr_prompt_template(
            entry,
            "docs/agents/lifecycle/codex-maintenance",
        ),
    );
    write_text(
        &root.join("docs/agents/lifecycle/codex-maintenance/OPS_PLAYBOOK.md"),
        "# Packet ops\n",
    );
    write_text(
        &root.join("docs/agents/lifecycle/codex-maintenance/CI_WORKFLOWS_PLAN.md"),
        "# Packet workflow plan\n",
    );
    write_text(
        &root.join("cli_manifests/codex/latest_validated.txt"),
        "0.97.0\n",
    );
    write_text(
        &root.join("cli_manifests/codex/artifacts.lock.json"),
        "{\n  \"schema_version\": 1\n}\n",
    );
    write_text(
        &root.join("cli_manifests/codex/wrapper_coverage.json"),
        "{\n  \"schema_version\": 1\n}\n",
    );
    write_text(
        &root.join("cli_manifests/codex/versions/0.98.0.json"),
        "{\n  \"semantic_version\": \"0.98.0\"\n}\n",
    );
    write_text(
        &root.join("cli_manifests/codex/reports/0.98.0/coverage.any.json"),
        "{\n  \"deltas\": {\n    \"missing_commands\": [],\n    \"missing_flags\": [],\n    \"missing_args\": [],\n    \"intentionally_unsupported\": []\n  }\n}\n",
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/non-tui-support-debt.md"),
        "# Non-TUI Support Debt Inventory\n\n## Inventory\n",
    );
}

fn execution_request_toml(run_id: &str) -> String {
    let registry =
        xtask::agent_registry::AgentRegistry::parse(include_str!("../../data/agent_registry.toml"))
            .expect("parse seeded registry");
    let entry = registry.find("codex").expect("codex registry entry");
    let prompt = xtask::agent_maintenance::contract_policy::packet_pr_prompt_template(
        entry,
        "docs/agents/lifecycle/codex-maintenance",
    )
    .replace("{{VERSION}}", "0.98.0");
    let prompt_sha256 = hex::encode(Sha256::digest(prompt.as_bytes()));
    let gate_one = format!(
        "sh ./gate-command.sh gate-1 docs/agents/.uaa-temp/agent-maintenance/runs/{run_id}/{GATE_ORDER_LOG_FILE}"
    );
    let gate_two = format!(
        "sh ./gate-command.sh gate-2 docs/agents/.uaa-temp/agent-maintenance/runs/{run_id}/{GATE_ORDER_LOG_FILE}"
    );

    format!(
        concat!(
            "artifact_version = \"2\"\n",
            "agent_id = \"codex\"\n",
            "trigger_kind = \"upstream_release_detected\"\n",
            "basis_ref = \"cli_manifests/codex/latest_validated.txt\"\n",
            "opened_from = \".github/workflows/agent-maintenance-open-pr.yml\"\n",
            "requested_control_plane_actions = [\"packet_doc_refresh\"]\n",
            "request_recorded_at = \"2026-05-05T15:00:00Z\"\n",
            "request_commit = \"abcdef1\"\n",
            "\n",
            "[runtime_followup_required]\n",
            "required = false\n",
            "items = []\n",
            "\n",
            "[detected_release]\n",
            "detected_by = \".github/workflows/agent-maintenance-release-watch.yml\"\n",
            "current_validated = \"0.97.0\"\n",
            "target_version = \"0.98.0\"\n",
            "latest_stable = \"0.99.0\"\n",
            "version_policy = \"latest_stable_minus_one\"\n",
            "source_kind = \"github_releases\"\n",
            "source_ref = \"openai/codex\"\n",
            "dispatch_kind = \"packet_pr\"\n",
            "dispatch_workflow = \"agent-maintenance-open-pr.yml\"\n",
            "branch_name = \"automation/codex-maintenance-0.98.0\"\n",
            "\n",
            "[support_surface_audit]\n",
            "required = true\n",
            "surface_kinds = [\"commands\", \"subcommands\", \"flags\", \"global_flags\", \"positional_args\"]\n",
            "excluded_surface_kinds = [\"tui_only\"]\n",
            "allowed_deferrals = [\n",
            "  \"upstream_not_machine_exposed\",\n",
            "  \"platform_evidence_missing\",\n",
            "  \"requires_new_infra\",\n",
            "  \"requires_new_architectural_seam\",\n",
            "  \"outside_registry_maintenance_write_envelope\",\n",
            "]\n",
            "pre_run_debt_count = 0\n",
            "expected_post_run_debt_count = 0\n",
            "\n",
            "[execution_contract]\n",
            "executor = \"codex\"\n",
            "prompt_template_path = \"docs/agents/lifecycle/codex-maintenance/governance/execute-agent-maintenance-prompt.md\"\n",
            "prompt_sha256 = \"{prompt_sha256}\"\n",
            "pr_summary_path = \"docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md\"\n",
            "closeout_path = \"docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json\"\n",
            "requires_manual_closeout = true\n",
            "writable_surfaces = [\n",
            "  \"docs/agents/lifecycle/codex-maintenance/**\",\n",
            "  \"cli_manifests/codex/versions/0.98.0.json\",\n",
            "]\n",
            "read_only_inputs = [\n",
            "  \"docs/agents/lifecycle/codex-maintenance/OPS_PLAYBOOK.md\",\n",
            "  \"docs/agents/lifecycle/codex-maintenance/CI_WORKFLOWS_PLAN.md\",\n",
            "  \"docs/agents/lifecycle/codex-maintenance/governance/execute-agent-maintenance-prompt.md\",\n",
            "  \".github/workflows/agent-maintenance-open-pr.yml\",\n",
            "]\n",
            "ordered_commands = [\n",
            "  \"{gate_one}\",\n",
            "  \"{gate_two}\",\n",
            "]\n",
            "green_gates = [\n",
            "  \"{gate_one}\",\n",
            "  \"{gate_two}\",\n",
            "]\n",
            "\n",
            "[execution_contract.recovery]\n",
            "recreate_packet_command = \"cargo run -p xtask -- refresh-agent --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --write\"\n",
            "reopen_pr_body_path = \"docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md\"\n",
            "reopen_pr_branch = \"automation/codex-maintenance-0.98.0\"\n",
            "notes = [\n",
            "  \"If PR creation fails after packet generation, rerun packet regeneration from the frozen request and reopen the PR from the generated pr-summary path.\",\n",
            "  \"If the local execution-host preflight (local Codex CLI host via execute-agent-maintenance) fails, fix the Codex binary/auth state and rerun `execute-agent-maintenance --dry-run` before write mode.\",\n",
            "]\n"
        ),
        prompt_sha256 = prompt_sha256,
        gate_one = gate_one,
        gate_two = gate_two,
    )
}

fn fake_execute_codex_script() -> String {
    format!(
        r#"#!/usr/bin/env bash
set -euo pipefail

scenario="success"
if [[ -n "${{XTASK_AGENT_MAINTENANCE_RUN_DIR:-}}" && -f "${{XTASK_AGENT_MAINTENANCE_RUN_DIR}}/{scenario_file}" ]]; then
  scenario="$(tr -d '\r\n' < "${{XTASK_AGENT_MAINTENANCE_RUN_DIR}}/{scenario_file}")"
elif [[ -f "$PWD/{scenario_file}" ]]; then
  scenario="$(tr -d '\r\n' < "$PWD/{scenario_file}")"
fi

if [[ "${{1:-}}" == "--version" ]]; then
  echo "codex 0.99.0"
  exit 0
fi

if [[ "${{1:-}}" == "--help" || "${{1:-}}" == "-h" ]]; then
  cat <<'EOF'
codex 0.99.0

Usage: codex [OPTIONS] <COMMAND>

Commands:
  exec     Execute commands
  help     Print this message
EOF
  exit 0
fi

if [[ "${{1:-}}" != "exec" ]]; then
  echo "fake-agent-maintenance-codex: unsupported invocation: $*" >&2
  exit 2
fi
shift

workspace_root="$PWD"
argv=()
while (($#)); do
  case "$1" in
    --cd)
      workspace_root="${{2:-}}"
      argv+=("$1" "${{2:-}}")
      shift 2
      ;;
    --skip-git-repo-check|--dangerously-bypass-approvals-and-sandbox)
      argv+=("$1")
      shift
      ;;
    *)
      echo "fake-agent-maintenance-codex: unsupported exec invocation: $*" >&2
      exit 2
      ;;
  esac
done

prompt="$(cat)"
if [[ "$prompt" == *"Repository preflight for execute-agent-maintenance."* ]]; then
  case "$scenario" in
    preflight_fail)
      echo "auth failure" >&2
      exit 17
      ;;
    preflight_write)
      mkdir -p "$workspace_root/docs"
      printf 'oops\n' > "$workspace_root/docs/unowned-preflight.md"
      printf '{sentinel}\n'
      exit 0
      ;;
    *)
      printf '{sentinel}\n'
      exit 0
      ;;
  esac
fi

run_dir="${{XTASK_AGENT_MAINTENANCE_RUN_DIR:-}}"
if [[ -z "$run_dir" ]]; then
  echo "fake-agent-maintenance-codex: missing run dir" >&2
  exit 2
fi
mkdir -p "$run_dir"
printf 'exec %s\n' "${{argv[*]}}" >> "$run_dir/{log_file}"

case "$scenario" in
  success)
    mkdir -p "$workspace_root/docs/agents/lifecycle/codex-maintenance"
    cat > "$workspace_root/docs/agents/lifecycle/codex-maintenance/runtime-note.md" <<'EOF'
# Runtime note

Relay write completed.
EOF
    cat > "$workspace_root/cli_manifests/codex/versions/0.98.0.json" <<'EOF'
{{
  "semantic_version": "0.98.0",
  "status": "validated"
}}
EOF
    ;;
  success_with_pycache)
    mkdir -p "$workspace_root/docs/agents/lifecycle/codex-maintenance"
    cat > "$workspace_root/docs/agents/lifecycle/codex-maintenance/runtime-note.md" <<'EOF'
# Runtime note

Relay write completed.
EOF
    cat > "$workspace_root/cli_manifests/codex/versions/0.98.0.json" <<'EOF'
{{
  "semantic_version": "0.98.0",
  "status": "validated"
}}
EOF
    mkdir -p "$workspace_root/scripts/__pycache__"
    printf 'bytecode\n' > "$workspace_root/scripts/__pycache__/publish_planner.cpython-313.pyc"
    ;;
  out_of_bounds)
    mkdir -p "$workspace_root/docs"
    printf 'not allowed\n' > "$workspace_root/docs/unowned.md"
    ;;
  noop)
    :
    ;;
  exec_fail)
    echo "forced exec failure" >&2
    exit 23
    ;;
  *)
    echo "unknown scenario: $scenario" >&2
    exit 2
    ;;
esac

echo "fake execute codex completed"
"#,
        scenario_file = FAKE_EXECUTE_CODEX_SCENARIO_FILE,
        sentinel = "UAA_AGENT_MAINTENANCE_PREFLIGHT_OK",
        log_file = FAKE_EXECUTE_CODEX_LOG_FILE,
    )
}

fn gate_command_script() -> String {
    "#!/usr/bin/env sh\nset -eu\nlabel=\"$1\"\nlog_path=\"$2\"\nmkdir -p \"$(dirname \"$log_path\")\"\nprintf '%s\\n' \"$label\" >> \"$log_path\"\n".to_string()
}

#[cfg(unix)]
fn mark_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path).expect("stat executable").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("chmod executable");
}

#[cfg(not(unix))]
fn mark_executable(_path: &Path) {}
