#!/usr/bin/env bash
# Run a bounded Codex task in an assigned worktree with an explicit sandbox and a time limit.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/run-codex-worker.sh --worktree DIR --packet FILE \
  [--sandbox read-only|workspace-write] [--profile NAME] [--timeout SECONDS] \
  [--output-last-message FILE] [--dry-run]

The packet must be outside the target worktree. Without --profile, Codex uses
its default configuration; with --profile, the overlay $CODEX_HOME/NAME.config.toml
must exist. --sandbox is always passed to Codex (default workspace-write), so the
global sandbox_mode never applies. Codex is stopped after --timeout seconds
(default 3600) and the launcher exits 124.
EOF
}

profile=""
worktree=""
packet=""
sandbox="workspace-write"
timeout_seconds=3600
output_last_message=""
dry_run=0

while (($#)); do
  case "$1" in
    --profile) profile=${2:?missing value for --profile}; shift 2 ;;
    --worktree) worktree=${2:?missing value for --worktree}; shift 2 ;;
    --packet) packet=${2:?missing value for --packet}; shift 2 ;;
    --sandbox) sandbox=${2:?missing value for --sandbox}; shift 2 ;;
    --timeout) timeout_seconds=${2:?missing value for --timeout}; shift 2 ;;
    --output-last-message) output_last_message=${2:?missing value for --output-last-message}; shift 2 ;;
    --dry-run) dry_run=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) printf 'error: unknown argument: %s\n' "$1" >&2; usage >&2; exit 2 ;;
  esac
done

[[ -n "$worktree" && -n "$packet" ]] || {
  printf '%s\n' 'error: --worktree and --packet are required' >&2
  usage >&2
  exit 2
}
case "$sandbox" in read-only|workspace-write) ;; *)
  printf '%s\n' 'error: --sandbox must be read-only or workspace-write' >&2
  exit 2 ;;
esac
[[ "$timeout_seconds" =~ ^[1-9][0-9]*$ ]] || {
  printf '%s\n' 'error: --timeout must be a positive whole number of seconds' >&2
  exit 2
}

worktree=$(cd "$worktree" && pwd -P)
packet=$(cd "$(dirname "$packet")" && pwd -P)/$(basename "$packet")
[[ -d "$worktree/.git" || -f "$worktree/.git" ]] || {
  printf 'error: worktree is not a Git worktree: %s\n' "$worktree" >&2
  exit 2
}
[[ -r "$packet" ]] || { printf 'error: packet is not readable: %s\n' "$packet" >&2; exit 2; }
case "$packet" in "$worktree"/*)
  printf '%s\n' 'error: packet must be outside the target worktree' >&2; exit 2 ;;
esac

cmd=(codex exec --sandbox "$sandbox" --cd "$worktree")
if [[ -n "$profile" ]]; then
  [[ "$profile" =~ ^[A-Za-z0-9_-]+$ ]] || {
    printf '%s\n' 'error: profile may contain only letters, digits, _ and -' >&2
    exit 2
  }
  profile_file="${CODEX_HOME:-"$HOME/.codex"}/$profile.config.toml"
  [[ -f "$profile_file" ]] || {
    printf 'error: Codex profile overlay does not exist: %s\n' "$profile_file" >&2
    exit 2
  }
  cmd+=(--profile "$profile")
fi
if [[ -n "$output_last_message" ]]; then
  output_last_message=$(cd "$(dirname "$output_last_message")" && pwd -P)/$(basename "$output_last_message")
  cmd+=(--output-last-message "$output_last_message")
fi
cmd+=(-)

command -v codex >/dev/null || { printf '%s\n' 'error: codex is not on PATH' >&2; exit 127; }
timeout_bin=$(command -v timeout || command -v gtimeout) || {
  printf '%s\n' 'error: timeout (GNU coreutils) is not on PATH' >&2
  exit 127
}
cmd=("$timeout_bin" -k 30 "$timeout_seconds" "${cmd[@]}")

if ((dry_run)); then
  printf 'profile: %s\n' "${profile:-(default configuration)}"
  printf 'packet: %s\n' "$packet"
  printf 'command:'
  printf ' %q' "${cmd[@]}"
  printf '\n'
  exit 0
fi

set +e
"${cmd[@]}" < "$packet"
status=$?
set -e
if ((status == 124 || status == 137)); then
  printf 'error: codex did not finish within %s seconds and was stopped\n' "$timeout_seconds" >&2
fi
exit "$status"
