#!/usr/bin/env bash
set -euo pipefail

RUNTIME_RUNS_ROOT="docs/agents/.uaa-temp/runtime-follow-on/runs"
FAKE_CODEX_LOG_FILE="fake-codex-invocations.log"
FAKE_CODEX_SCENARIO_FILE="fake-codex-scenario.txt"

enabled_features=()
while [[ "${1:-}" == "--enable" ]]; do
  enabled_features+=("${2:-}")
  shift 2
done

root_help() {
  local extra_line=""
  if printf '%s\n' "${enabled_features[@]}" | grep -qx "extra_feature"; then
    extra_line=$'  extra    Extra command (feature gated)\n'
  fi

  cat <<EOF
codex 0.77.0

Usage: codex [OPTIONS] <COMMAND>

Commands:
  zed      Zed command with a wrapped description
           that should not be interpreted as a new command token
${extra_line}  features  Inspect feature flags
  exec     Execute commands
  alpha    Alpha command
  help     Print this message or the help of the given subcommand(s)

Options:
  -q, --quiet        Suppress output
  --json             Emit JSON
  -v                 Verbose output
EOF
}

prompt_field() {
  local prompt="$1"
  local label="$2"
  printf '%s\n' "$prompt" | sed -n "s/^${label}: \`\\([^\\\`]*\\)\`.*/\\1/p" | head -n 1
}

write_required_test() {
  local workspace_root="$1"
  local required_test="$2"
  mkdir -p "$(dirname "$workspace_root/$required_test")"
  cat > "$workspace_root/$required_test" <<'EOF'
#[test]
fn runtime_follow_on_smoke() {}
EOF
}

write_valid_handoff() {
  local handoff_path="$1"
  local agent_id="$2"
  local manifest_root="$3"
  cat > "$handoff_path" <<EOF
{
  "agent_id": "$agent_id",
  "manifest_root": "$manifest_root",
  "runtime_lane_complete": true,
  "publication_refresh_required": true,
  "required_commands": [
    "support-matrix --check",
    "capability-matrix --check",
    "capability-matrix-audit",
    "make preflight"
  ],
  "blockers": []
}
EOF
}

write_invalid_handoff() {
  local handoff_path="$1"
  local agent_id="$2"
  local manifest_root="$3"
  cat > "$handoff_path" <<EOF
{
  "agent_id": "$agent_id",
  "manifest_root": "$manifest_root",
  "publication_refresh_required": true,
  "required_commands": [
    "support-matrix --check"
  ],
  "blockers": []
}
EOF
}

write_success_outputs() {
  local workspace_root="$1"
  local backend_module="$2"
  local manifest_root="$3"
  local agent_id="$4"

  mkdir -p "$workspace_root/$backend_module"
  cat > "$workspace_root/$backend_module/mod.rs" <<'EOF'
pub fn runtime_follow_on() {}
EOF

  mkdir -p "$workspace_root/crates/agent_api/src/bin"
  cat > "$workspace_root/crates/agent_api/src/bin/fake_${agent_id}_stream_json_agent_api.rs" <<'EOF'
fn main() {}
EOF

  mkdir -p "$workspace_root/$manifest_root/snapshots"
  cat > "$workspace_root/$manifest_root/snapshots/default.json" <<'EOF'
{
  "snapshot": true
}
EOF

  cat > "$workspace_root/Cargo.lock" <<'EOF'
# synthetic lockfile delta
EOF
}

run_exec() {
  local workspace_root="$PWD"
  local argv=()

  while (($#)); do
    case "$1" in
      --cd)
        workspace_root="${2:-}"
        argv+=("$1" "${2:-}")
        shift 2
        ;;
      --skip-git-repo-check|--dangerously-bypass-approvals-and-sandbox|--json|--quiet)
        argv+=("$1")
        shift
        ;;
      *)
        echo "fake_codex: unsupported exec invocation: $*" >&2
        exit 2
        ;;
    esac
  done

  local prompt
  prompt="$(cat)"
  local agent_id manifest_root backend_module required_test
  agent_id="$(prompt_field "$prompt" "Agent")"
  manifest_root="$(prompt_field "$prompt" "- manifest root")"
  backend_module="$(prompt_field "$prompt" "- backend module")"
  required_test="$(prompt_field "$prompt" "- required agent_api onboarding test")"
  local run_dir
  run_dir="$(find "$workspace_root/$RUNTIME_RUNS_ROOT" -mindepth 1 -maxdepth 1 -type d | sort | tail -n 1)"
  if [[ -z "$run_dir" ]]; then
    echo "fake_codex: missing runtime-follow-on run dir" >&2
    exit 2
  fi

  printf 'exec %s\n' "${argv[*]}" >> "$run_dir/$FAKE_CODEX_LOG_FILE"

  local scenario="success"
  if [[ -f "$run_dir/$FAKE_CODEX_SCENARIO_FILE" ]]; then
    scenario="$(tr -d '\r\n' < "$run_dir/$FAKE_CODEX_SCENARIO_FILE")"
  fi

  case "$scenario" in
    success)
      write_required_test "$workspace_root" "$required_test"
      write_success_outputs "$workspace_root" "$backend_module" "$manifest_root" "$agent_id"
      write_valid_handoff "$run_dir/handoff.json" "$agent_id" "$manifest_root"
      ;;
    handoff_only)
      write_valid_handoff "$run_dir/handoff.json" "$agent_id" "$manifest_root"
      ;;
    invalid_handoff)
      write_required_test "$workspace_root" "$required_test"
      write_invalid_handoff "$run_dir/handoff.json" "$agent_id" "$manifest_root"
      ;;
    exec_fail)
      echo "fake_codex: forced exec failure" >&2
      exit 17
      ;;
    *)
      echo "fake_codex: unknown scenario \`$scenario\`" >&2
      exit 2
      ;;
  esac

  echo "fake codex exec completed"
}

if [[ "${1:-}" == "--version" ]]; then
  echo "codex 0.77.0"
  exit 0
fi

if [[ "${1:-}" == "features" && "${2:-}" == "list" ]]; then
  cat <<'EOF'
base_feature stable true
extra_feature experimental false
EOF
  exit 0
fi

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  root_help
  exit 0
fi

if [[ "${1:-}" == "exec" ]]; then
  shift
  run_exec "$@"
  exit 0
fi

if [[ "${1:-}" != "help" ]]; then
  echo "fake_codex: unsupported invocation: $*" >&2
  exit 2
fi
shift

case "$*" in
  "")
    root_help
    ;;
  "help")
    cat <<'EOF'
Usage: codex help [COMMAND]...

Print this message or the help of the given subcommand(s)
EOF
    ;;
  "features")
    cat <<'EOF'
Inspect feature flags

Usage: codex features [OPTIONS] <COMMAND>

Commands:
  list  List known features
  help  Print this message or the help of the given subcommand(s)

Options:
  --enable <FEATURE>   Enable a feature (repeatable)
  --disable <FEATURE>  Disable a feature (repeatable)
  -h, --help           Print help
EOF
    ;;
  "features list")
    cat <<'EOF'
List known features

Usage: codex features list [OPTIONS]

Options:
  -h, --help  Print help
EOF
    ;;
  "features help")
    cat <<'EOF'
Print this message or the help of the given subcommand(s)

Usage: codex features help [COMMAND]...

Arguments:
  [COMMAND]...  Print help for the subcommand(s)
EOF
    ;;
  "exec")
    cat <<'EOF'
Usage: codex exec [OPTIONS] [PROMPT] [COMMAND]

Commands:
  start    Start execution
  resume   Resume execution

Options:
  --beta            Beta option (long only)

  -a, --alpha       Alpha option
  -c               Short-only option
EOF
    ;;
  "exec start")
    cat <<'EOF'
Usage: codex exec start [OPTIONS] <PROMPT>

Arguments:
  <PROMPT>
          First line of prompt description
          Second line of prompt description (wrapped)

Options:
  --zulu            Zulu mode (long only)
  -b                Short-only option
  -a, --alpha       Alpha option
  -d, --delta PATH  Delta path (takes value)
EOF
    ;;
  "exec resume")
    cat <<'EOF'
Usage: codex exec resume [OPTIONS]

Options:
  --json            Emit JSON
  -q, --quiet       Suppress output
EOF
    ;;
  "alpha")
    cat <<'EOF'
Usage: codex alpha [OPTIONS]

Options:
  -x, --xray        Xray mode
EOF
    ;;
  "zed")
    cat <<'EOF'
Usage: codex zed [OPTIONS]

Options:
  --zebra           Zebra mode
EOF
    ;;
  "sandbox")
    cat <<'EOF'
Usage: codex sandbox [OPTIONS]

Options:
  --linux-only      Linux only
EOF
    ;;
  "extra")
    cat <<'EOF'
Extra command (feature gated)

Usage: codex extra [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input to process

Options:
  -h, --help  Print help
EOF
    ;;
  *)
    echo "fake_codex: unsupported help path: $*" >&2
    exit 2
    ;;
esac
