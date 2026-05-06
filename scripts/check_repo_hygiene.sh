#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

fail=0

check_no_tracked_under() {
  local path="$1"
  if git ls-files --error-unmatch "$path" >/dev/null 2>&1; then
    # Allow removal commits to pass: a file may still be "tracked" in the index but already staged
    # for deletion.
    if git diff --cached --name-only --diff-filter=D -- "$path" | grep -q .; then
      return
    fi
    echo "ERROR: repo hygiene: path is tracked but should not be: $path"
    fail=1
  fi
}

check_no_tracked_glob() {
  local pattern="$1"
  local hits
  if command -v rg >/dev/null 2>&1; then
    hits="$(git ls-files | rg -n "$pattern" || true)"
  else
    hits="$(git ls-files | grep -En "$pattern" || true)"
  fi
  if [[ -n "$hits" ]]; then
    echo "ERROR: repo hygiene: tracked files match pattern: $pattern"
    echo "$hits"
    fail=1
  fi
}

check_tracked_content_pattern() {
  local pattern="$1"
  shift
  local hits
  hits="$(git grep -n -I -e "$pattern" -- . "$@" || true)"
  if [[ -n "$hits" ]]; then
    echo "ERROR: repo hygiene: tracked file contents match pattern: $pattern"
    echo "$hits"
    fail=1
  fi
}

check_duplicate_adr_numbers() {
  local duplicates
  duplicates="$(
    git ls-files 'docs/adr/*.md' \
      | while IFS= read -r path; do
          [[ -f "$path" ]] && printf '%s\n' "$path"
        done \
      | sed -E 's#docs/adr/([0-9]{4})-.*#\1#' \
      | sort \
      | uniq -d
  )"
  if [[ -n "$duplicates" ]]; then
    echo "ERROR: repo hygiene: duplicate ADR numbers detected under docs/adr"
    while IFS= read -r number; do
      [[ -z "$number" ]] && continue
      git ls-files "docs/adr/${number}-*.md"
    done <<< "$duplicates"
    fail=1
  fi
}

# Directories that should never be committed.
check_no_tracked_under "wt"
check_no_tracked_under "target"
check_no_tracked_under "_download"
check_no_tracked_under "_extract"
check_no_tracked_under ".codex-bins"
check_no_tracked_under "cli_manifests/codex/raw_help"

# Legacy/stray repo-root snapshot JSONs that must not be tracked.
check_no_tracked_under "aarch64-apple-darwin.json"
check_no_tracked_under "x86_64-unknown-linux-musl.json"
check_no_tracked_under "x86_64-pc-windows-msvc.json"

# Common artifact types that should not be committed.
check_no_tracked_glob "\\.log$"
check_no_tracked_glob "universal-agent-api-planning-pack_.*\\.zip$"

check_tracked_content_pattern "docs/project_management/next/" \
  ':(exclude).archived/**' \
  ':(exclude)scripts/check_repo_hygiene.sh' \
  ':(exclude)crates/xtask/tests/agent_maintenance_refresh.rs'
check_tracked_content_pattern "docs/specs/universal-agent-api/" \
  ':(exclude)scripts/check_repo_hygiene.sh'
check_no_tracked_under "docs/reports/agent-lifecycle"
check_no_tracked_under "docs/reports/verification"
check_tracked_content_pattern "docs/reports/agent-lifecycle" \
  ':(exclude).archived/**' \
  ':(exclude)DOCS_AGENTS_MIGRATION_PLAN.md' \
  ':(exclude)scripts/check_repo_hygiene.sh'
check_tracked_content_pattern "docs/reports/verification/cli-agent-selection/cli-agent-selection-packet.md" \
  ':(exclude).archived/**' \
  ':(exclude)DOCS_AGENTS_MIGRATION_PLAN.md' \
  ':(exclude)scripts/check_repo_hygiene.sh'
check_duplicate_adr_numbers

if [[ "$fail" -ne 0 ]]; then
  exit 1
fi

echo "repo hygiene: OK"
