use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use serde_json::Value;
use xtask::recommend_next_agent_research;

use crate::harness::{fixture_root, repo_root, snapshot_files, write_text, HarnessOutput};

pub const RESEARCH_PACKET_ROOT: &str = "docs/agents/.uaa-temp/recommend-next-agent/research-runs";
pub const EVAL_RUNS_ROOT: &str = "docs/agents/.uaa-temp/recommend-next-agent/runs";
pub const PASS1_RUN_ID: &str = "rna-pass1";
pub const PASS2_RUN_ID: &str = "rna-pass2";
pub const FAKE_CODEX_SCENARIO_FILE: &str = "fake-rna-codex-scenario.txt";

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Project automation tasks")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    RecommendNextAgentResearch(recommend_next_agent_research::Args),
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
                Command::RecommendNextAgentResearch(args) => {
                    match recommend_next_agent_research::run_in_workspace(
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

pub fn prepare_recommendation_fixture(prefix: &str) -> PathBuf {
    let fixture = fixture_root(prefix);
    seed_recommendation_touchpoints(&fixture);
    fixture
}

pub fn recommend_args(
    mode_flag: &str,
    pass: &str,
    run_id: &str,
    codex_binary: &Path,
) -> Vec<String> {
    let mut args = vec![
        "xtask".to_string(),
        "recommend-next-agent-research".to_string(),
        mode_flag.to_string(),
        "--pass".to_string(),
        pass.to_string(),
        "--run-id".to_string(),
        run_id.to_string(),
    ];
    if mode_flag == "--write" {
        args.extend([
            "--codex-binary".to_string(),
            codex_binary.display().to_string(),
        ]);
    }
    args
}

pub fn pass2_args(
    mode_flag: &str,
    run_id: &str,
    prior_run_dir: &str,
    codex_binary: Option<&Path>,
) -> Vec<String> {
    let mut args = vec![
        "xtask".to_string(),
        "recommend-next-agent-research".to_string(),
        mode_flag.to_string(),
        "--pass".to_string(),
        "pass2".to_string(),
        "--prior-run-dir".to_string(),
        prior_run_dir.to_string(),
        "--run-id".to_string(),
        run_id.to_string(),
    ];
    if let Some(codex_binary) = codex_binary {
        args.extend([
            "--codex-binary".to_string(),
            codex_binary.display().to_string(),
        ]);
    }
    args
}

pub fn seed_prior_insufficiency_run(root: &Path, run_id: &str, zero_survivors: bool) -> String {
    let run_dir = root.join(EVAL_RUNS_ROOT).join(run_id);
    fs::create_dir_all(&run_dir).expect("create prior run dir");
    let candidates = if zero_survivors {
        vec![
            serde_json::json!({"agent_id":"alpha","status":"candidate_rejected","recommended":false,"shortlisted":false,"rejection_reasons":["rejected"]}),
            serde_json::json!({"agent_id":"beta","status":"candidate_error","recommended":false,"shortlisted":false,"error_reasons":["error"]}),
        ]
    } else {
        vec![
            serde_json::json!({"agent_id":"alpha","status":"eligible","recommended":true,"shortlisted":true,"rejection_reasons":[],"error_reasons":[]}),
            serde_json::json!({"agent_id":"beta","status":"candidate_rejected","recommended":false,"shortlisted":false,"rejection_reasons":["rejected"],"error_reasons":[]}),
            serde_json::json!({"agent_id":"gamma","status":"candidate_error","recommended":false,"shortlisted":false,"rejection_reasons":[],"error_reasons":["error"]}),
        ]
    };
    write_json(
        &run_dir.join("candidate-pool.json"),
        &serde_json::json!({
            "run_id": run_id,
            "candidates": candidates,
        }),
    );
    write_json(
        &run_dir.join("run-status.json"),
        &serde_json::json!({
            "run_id": run_id,
            "status": "insufficient_eligible_candidates",
            "next_action": "expand_discovery",
            "recommended_agent_id": if zero_survivors { Value::Null } else { Value::String("alpha".to_string()) },
        }),
    );
    format!("{EVAL_RUNS_ROOT}/{run_id}")
}

pub fn fake_codex_binary(root: &Path) -> PathBuf {
    let binary = root.join("fake-rna-codex.sh");
    if !binary.is_file() {
        write_text(&binary, FAKE_CODEX_SCRIPT);
        mark_executable(&binary);
    }
    binary
}

pub fn write_fake_codex_scenario(root: &Path, scenario: &str) {
    write_text(
        &root.join(FAKE_CODEX_SCENARIO_FILE),
        &format!("{scenario}\n"),
    );
}

pub fn force_freeze_discovery_failure(root: &Path) {
    write_text(
        &root.join("scripts/recommend_next_agent.py"),
        concat!(
            "#!/usr/bin/env python3\n",
            "import sys\n",
            "print('ERROR: forced freeze-discovery failure')\n",
            "raise SystemExit(1)\n",
        ),
    );
}

pub fn packet_dir(root: &Path, run_id: &str) -> PathBuf {
    root.join(RESEARCH_PACKET_ROOT).join(run_id)
}

pub fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read json")).expect("parse json")
}

pub fn write_json(path: &Path, value: &Value) {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize json");
    bytes.push(b'\n');
    fs::write(path, bytes).expect("write json");
}

pub fn snapshot_without_packet_runs(root: &Path) -> BTreeMap<String, Vec<u8>> {
    snapshot_files(root)
        .into_iter()
        .filter(|(path, _)| !path.starts_with(RESEARCH_PACKET_ROOT))
        .collect()
}

fn seed_recommendation_touchpoints(root: &Path) {
    let repo = repo_root();
    let script_source = repo.join("scripts/recommend_next_agent.py");
    let contract_source = repo.join("docs/specs/cli-agent-recommendation-dossier-contract.md");
    fs::create_dir_all(root.join("scripts")).expect("create scripts dir");
    fs::copy(script_source, root.join("scripts/recommend_next_agent.py"))
        .expect("copy recommend_next_agent.py");
    fs::create_dir_all(root.join("docs/specs")).expect("create specs dir");
    fs::copy(
        contract_source,
        root.join("docs/specs/cli-agent-recommendation-dossier-contract.md"),
    )
    .expect("copy dossier contract");
    write_text(
        &root.join("docs/agents/selection/candidate-seed.toml"),
        seed_text(),
    );
    write_text(
        &root.join("docs/agents/selection/discovery-hints.json"),
        "{\n  \"exclude_candidates\": [\"codex\"],\n  \"include_candidates\": [\"alpha\"]\n}\n",
    );
    fs::create_dir_all(root.join(EVAL_RUNS_ROOT)).expect("create eval runs root");
}

fn seed_text() -> &'static str {
    concat!(
        "[defaults.descriptor]\n",
        "canonical_targets = [\"darwin-arm64\"]\n",
        "wrapper_coverage_binding_kind = \"generated_from_wrapper_crate\"\n",
        "always_on_capabilities = [\"agent_api.run\", \"agent_api.events\"]\n",
        "target_gated_capabilities = []\n",
        "config_gated_capabilities = []\n",
        "backend_extensions = []\n",
        "support_matrix_enabled = true\n",
        "capability_matrix_enabled = true\n",
        "capability_matrix_target = \"\"\n",
        "docs_release_track = \"crates-io\"\n",
        "\n",
        "[candidate.alpha]\n",
        "display_name = \"Alpha CLI\"\n",
        "research_urls = [\"https://research.local/alpha/repo\", \"https://research.local/alpha/docs\"]\n",
        "install_channels = [\"brew install alpha\", \"npm install -g alpha\"]\n",
        "auth_notes = \"Alpha auth notes\"\n",
        "\n",
        "[candidate.beta]\n",
        "display_name = \"Beta CLI\"\n",
        "research_urls = [\"https://research.local/beta/repo\", \"https://research.local/beta/docs\"]\n",
        "install_channels = [\"brew install beta\", \"npm install -g beta\"]\n",
        "auth_notes = \"Beta auth notes\"\n",
        "\n",
        "[candidate.gamma]\n",
        "display_name = \"Gamma CLI\"\n",
        "research_urls = [\"https://research.local/gamma/repo\", \"https://research.local/gamma/docs\"]\n",
        "install_channels = [\"brew install gamma\", \"npm install -g gamma\"]\n",
        "auth_notes = \"Gamma auth notes\"\n",
        "\n",
        "[candidate.delta]\n",
        "display_name = \"Delta CLI\"\n",
        "research_urls = [\"https://research.local/delta/repo\", \"https://research.local/delta/docs\"]\n",
        "install_channels = [\"brew install delta\", \"npm install -g delta\"]\n",
        "auth_notes = \"Delta auth notes\"\n",
    )
}

#[cfg(unix)]
fn mark_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path).expect("stat fake codex").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("chmod fake codex");
}

#[cfg(not(unix))]
fn mark_executable(_path: &Path) {}

const FAKE_CODEX_SCRIPT: &str = r###"#!/usr/bin/env bash
set -euo pipefail

prompt_field() {
  local prompt="$1"
  local label="$2"
  printf '%s\n' "$prompt" | sed -n "s/^${label}: \`\([^\\\`]*\)\`.*/\1/p" | head -n 1
}

workspace_root="$PWD"
while (($#)); do
  case "$1" in
    exec) shift ;;
    --cd) workspace_root="${2:-}"; shift 2 ;;
    --skip-git-repo-check|--dangerously-bypass-approvals-and-sandbox) shift ;;
    *) echo "fake-rna-codex: unsupported args: $*" >&2; exit 2 ;;
  esac
done

prompt="$(cat)"
scenario="success"
if [[ -f "$workspace_root/fake-rna-codex-scenario.txt" ]]; then
  scenario="$(tr -d '\r\n' < "$workspace_root/fake-rna-codex-scenario.txt")"
fi

phase="discovery"
if [[ "$prompt" == *"Recommendation Research Dossier Prompt"* ]]; then
  phase="research"
fi

run_id="$(prompt_field "$prompt" "Run id")"
allowed_root="$(prompt_field "$prompt" "Allowed output root")"
seed_snapshot="$(prompt_field "$prompt" "Frozen seed snapshot")"
packet_root="$(prompt_field "$prompt" "Execution packet root")"
log_file="$workspace_root/${packet_root}/fake-rna-codex-invocations.log"
mkdir -p "$(dirname "$log_file")"
printf '%s:%s:%s\n' "$phase" "$scenario" "$run_id" >> "$log_file"

if [[ "$phase" == "discovery" ]]; then
  python3 - "$workspace_root" "$allowed_root" "$run_id" "$scenario" <<'PY'
import hashlib
import json
import pathlib
import sys

workspace_root = pathlib.Path(sys.argv[1])
allowed_root = workspace_root / sys.argv[2]
run_id = sys.argv[3]
scenario = sys.argv[4]
allowed_root.mkdir(parents=True, exist_ok=True)

candidates = [
    ("alpha", "Alpha CLI"),
    ("beta", "Beta CLI"),
    ("gamma", "Gamma CLI"),
    ("delta", "Delta CLI"),
]
if scenario == "too_few_candidates":
    candidates = candidates[:2]

seed_lines = [
    "[defaults.descriptor]",
    'canonical_targets = ["darwin-arm64"]',
    'wrapper_coverage_binding_kind = "generated_from_wrapper_crate"',
    'always_on_capabilities = ["agent_api.run", "agent_api.events"]',
    'target_gated_capabilities = []',
    'config_gated_capabilities = []',
    'backend_extensions = []',
    'support_matrix_enabled = true',
    'capability_matrix_enabled = true',
    'capability_matrix_target = ""',
    'docs_release_track = "crates-io"',
]
for agent_id, display_name in candidates:
    seed_lines.extend([
        "",
        f"[candidate.{agent_id}]",
        f'display_name = "{display_name}"',
        f'research_urls = ["https://research.local/{agent_id}/repo", "https://research.local/{agent_id}/docs"]',
        f'install_channels = ["brew install {agent_id}", "npm install -g {agent_id}"]',
        f'auth_notes = "{display_name} auth notes"',
    ])
(allowed_root / "candidate-seed.generated.toml").write_text("\n".join(seed_lines) + "\n", encoding="utf-8")

if scenario == "freeze_fail":
    summary_text = "\n".join([
        f"# Discovery Summary {run_id}",
        "",
        f"Discovery run id: {run_id}",
        "Discovery pass number: 1",
        "Queries used: best AI coding CLI; AI agent CLI tools; developer agent command line",
        "",
        "## alpha - Alpha CLI",
        "Alpha CLI entered the pool for run {run_id}.",
        "",
        "## beta - Beta CLI",
        "Beta CLI entered the pool for run {run_id}.",
        "",
        "## gamma - Gamma CLI",
        "Gamma CLI entered the pool for run {run_id}.",
        "",
        "## delta - Delta CLI",
        "Delta CLI entered the pool for run {run_id}.",
        "",
    ]).replace("{run_id}", run_id)
else:
    lines = [
        f"# Discovery Summary {run_id}",
        "",
        f"Discovery run id: {run_id}",
        "Discovery pass number: 1",
        "Queries used: best AI coding CLI; AI agent CLI tools; developer agent command line",
        "",
    ]
    for agent_id, display_name in candidates:
        heading = f"## {agent_id} - {display_name}"
        if scenario == "summary_missing_display_name":
            heading = f"## {agent_id}"
        body = f"{display_name} entered the pool for run {run_id}."
        if scenario == "summary_missing_display_name":
            body = f"Candidate {agent_id} entered the pool for run {run_id}."
        lines.extend([
            heading,
            body,
            "",
        ])
    summary_text = "\n".join(lines)
(allowed_root / "discovery-summary.md").write_text(summary_text, encoding="utf-8")

def canonical_entry(entry):
    base = {
        "candidate_id": entry["candidate_id"],
        "captured_at": entry["captured_at"],
        "role": entry["role"],
        "source_kind": entry["source_kind"],
        "title": entry["title"],
        "url": entry["url"],
    }
    if entry["source_kind"] == "web_search_result":
        base["query"] = entry["query"]
        base["rank"] = entry["rank"]
    return base

sources = []
for rank, (agent_id, display_name) in enumerate(candidates, start=1):
    entry = {
        "candidate_id": agent_id,
        "source_kind": "web_search_result",
        "url": f"https://search.example.test/{agent_id}",
        "title": f"{display_name} search result",
        "captured_at": "2026-05-04T00:00:00Z",
        "role": "frontier_signal",
        "query": "best AI coding CLI",
        "rank": rank,
    }
    entry["sha256"] = hashlib.sha256(
        json.dumps(canonical_entry(entry), sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    ).hexdigest()
    if scenario == "freeze_fail" and rank == 1:
        entry["sha256"] = "0" * 64
    sources.append(entry)
payload = {"run_id": run_id, "sources": sources}
if scenario == "invalid_sources_lock_keys":
    payload = {
        "workflow_version": "recommend_next_agent_research_v1",
        "run_id": run_id,
        "sources": [
            {
                **{k: v for k, v in entry.items() if k != "source_kind"},
                "kind": entry["source_kind"],
            }
            for entry in sources
        ],
    }
(allowed_root / "sources.lock.json").write_text(
    json.dumps(payload, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)

if scenario == "out_of_bounds":
    bad = workspace_root / "docs" / "unowned.md"
    bad.parent.mkdir(parents=True, exist_ok=True)
    bad.write_text("not allowed\n", encoding="utf-8")
PY
  exit 0
fi

python3 - "$workspace_root" "$allowed_root" "$seed_snapshot" "$scenario" <<'PY'
import hashlib
import json
import pathlib
import re
import sys

workspace_root = pathlib.Path(sys.argv[1])
allowed_root = workspace_root / sys.argv[2]
seed_snapshot = workspace_root / sys.argv[3]
scenario = sys.argv[4]
allowed_root.mkdir(parents=True, exist_ok=True)
(allowed_root / "dossiers").mkdir(parents=True, exist_ok=True)
(allowed_root / "research-summary.md").write_text("# Frozen research summary\n", encoding="utf-8")
run_id = allowed_root.name
(allowed_root / "research-metadata.json").write_text(
    json.dumps({"run_id": run_id, "evidence_collection_time_seconds": 12, "fetched_source_count": 8}, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)

seed_text = seed_snapshot.read_text(encoding="utf-8")
seed_sha = hashlib.sha256(seed_snapshot.read_bytes()).hexdigest()
candidate_ids = re.findall(r"^\[candidate\.([A-Za-z0-9_-]+)\]\s*$", seed_text, flags=re.MULTILINE)
for index, agent_id in enumerate(candidate_ids):
    display_name = agent_id.replace("_", " ").title()
    evidence = [
        {
            "evidence_id": f"{agent_id}-doc",
            "kind": "official_doc",
            "url": f"https://research.local/{agent_id}/docs",
            "title": f"{display_name} docs",
            "captured_at": "2026-05-04T00:00:00Z",
            "sha256": hashlib.sha256(f"{agent_id}:doc".encode("utf-8")).hexdigest(),
            "excerpt": f"{display_name} official docs",
        },
        {
            "evidence_id": f"{agent_id}-pkg",
            "kind": "package_registry",
            "url": f"https://research.local/{agent_id}/pkg",
            "title": f"{display_name} package registry",
            "captured_at": "2026-05-04T00:00:00Z",
            "sha256": hashlib.sha256(f"{agent_id}:pkg".encode("utf-8")).hexdigest(),
            "excerpt": f"{display_name} package registry",
        },
        {
            "evidence_id": f"{agent_id}-gh",
            "kind": "github",
            "url": f"https://research.local/{agent_id}/repo",
            "title": f"{display_name} repository",
            "captured_at": "2026-05-04T00:00:00Z",
            "sha256": hashlib.sha256(f"{agent_id}:gh".encode("utf-8")).hexdigest(),
            "excerpt": f"{display_name} repository",
        },
    ]
    claims = {}
    for claim_key in [
        "non_interactive_execution",
        "offline_strategy",
        "observable_cli_surface",
        "redaction_fit",
        "crate_first_fit",
        "reproducibility",
        "future_leverage",
    ]:
        claims[claim_key] = {
            "state": "verified" if claim_key in {"non_interactive_execution", "observable_cli_surface"} else "inferred",
            "summary": f"{display_name} {claim_key} summary",
            "evidence_ids": [f"{agent_id}-doc", f"{agent_id}-pkg", f"{agent_id}-gh"],
        }
    payload = {
        "schema_version": "1.0.0",
        "agent_id": agent_id,
        "display_name": display_name,
        "generated_at": "2026-05-04T00:00:00Z",
        "seed_snapshot_sha256": seed_sha,
        "official_links": [
            f"https://research.local/{agent_id}/docs",
            f"https://research.local/{agent_id}/repo",
        ],
        "install_channels": [
            f"brew install {agent_id}",
            f"npm install -g {agent_id}",
        ],
        "auth_prerequisites": [f"{display_name} auth notes"],
        "claims": claims,
        "probe_requests": [
            {
                "probe_kind": "help",
                "binary": agent_id.replace("_", "-"),
                "required_for_gate": False,
            }
        ],
        "blocked_steps": [],
        "normalized_caveats": [],
        "evidence": evidence,
    }
    if scenario == "identity_mismatch" and index == 0:
        payload["seed_snapshot_sha256"] = "0" * 64
    if scenario == "invalid_research_schema" and index == 0:
        payload["official_links"] = [{"label": "Docs", "url": f"https://research.local/{agent_id}/docs"}]
    (allowed_root / "dossiers" / f"{agent_id}.json").write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
PY
"###;
