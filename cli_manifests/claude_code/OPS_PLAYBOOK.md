# Claude Code CLI Parity Ops Playbook

This is the maintainer runbook for keeping `crates/claude_code` in parity with upstream **Claude Code CLI** releases using the same parity framework used by the Codex crate.

Key references:
- Parity framework ADRs: `docs/adr/0001-codex-cli-parity-maintenance.md`, `docs/adr/0003-wrapper-coverage-auto-generation.md`, `docs/adr/0004-wrapper-coverage-iu-subtree-inheritance.md`
- Parity root (schemas/rules): `cli_manifests/claude_code/README.md`
- CI plan: `cli_manifests/claude_code/CI_WORKFLOWS_PLAN.md`
- CI agent runbook: `cli_manifests/claude_code/CI_AGENT_RUNBOOK.md`
- PR body template: `cli_manifests/claude_code/PR_BODY_TEMPLATE.md`
- Workflows:
  - `.github/workflows/claude-code-release-watch.yml`
  - `.github/workflows/claude-code-update-snapshot.yml`
  - `.github/workflows/claude-code-promote.yml`

## Core Policies

- **No runtime downloads.** The wrapper crate must not download/update Claude Code at runtime. Downloads and pins happen in CI workflows only.
- **Channel policy:** automation tracks the upstream `stable` pointer.
- **Targets (v1):**
  - required: `linux-x64`
  - best-effort: `darwin-arm64`, `win32-x64`
- **Authoritative pointers:** `min_supported.txt` and `latest_validated.txt` (plus per-target pointers under `pointers/`). During bootstrap these pointers may be `none`.
- **Promotion safety:** only promote versions that have passed validation on the required target and meet the `RULES.json` gating rules.

## Live Upstream-Release Flow

The shipped maintenance path for Claude Code parity is:

1. `.github/workflows/agent-maintenance-release-watch.yml` detects stale `claude_code` parity from registry truth and dispatches `.github/workflows/claude-code-update-snapshot.yml`.
2. The worker refreshes the Claude Code parity artifacts, runs `prepare-agent-maintenance --write`, and opens branch `automation/claude_code-maintenance-<target_version>` with PR body `docs/agents/lifecycle/claude_code-maintenance/governance/pr-summary.md`.
3. The maintainer reviews `docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml` and `docs/agents/lifecycle/claude_code-maintenance/HANDOFF.md`, then runs:

```bash
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml --dry-run
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml --write --run-id <prepared_run_id>
```

4. `execute-agent-maintenance --dry-run` is the required trust step before write mode. It validates local Codex preflight, prints the exact writable surfaces and green gates, and prepares the frozen run packet under `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`.
5. `execute-agent-maintenance --write` reuses that prepared baseline, enforces the request-owned write envelope, runs the request-owned green gates, and stops before closeout.
6. The maintainer reviews the diff and runs `close-agent-maintenance` explicitly. Closeout is never performed by the relay.

Boundaries:
- Promotion-only pointer changes remain separate maintainer actions and are not part of the upstream-release relay.
- Packet-only agents remain deferred; do not widen non-relay maintenance packets into `execute-agent-maintenance --write`.

## Release Watch: Triage Checklist

When the shared Release Watch workflow runs, or when you manually inspect a queued Claude Code maintenance item:

1. Read the upstream `stable` pointer from the Claude Code distribution bucket.
2. Compare to `cli_manifests/claude_code/latest_validated.txt`.
3. If the candidate is strictly newer, run the Update Snapshot workflow for that version.

## Update Snapshot (workflow_dispatch)

Normal operation is the shared watcher dispatch above. Use this workflow manually only to replay or repair the worker step for a known target version.

Preferred path: run the GitHub Actions workflow:
- `.github/workflows/claude-code-update-snapshot.yml`

Required replay inputs:
- `agent_id`: `claude_code`
- `current_version`: the current validated Claude Code version from registry truth
- `latest_stable`: the latest stable upstream version seen by the watcher
- `target_version`: the worker target version to validate
- `opened_from`: the repo-relative worker workflow path, `.github/workflows/claude-code-update-snapshot.yml`
- `detected_by`: `.github/workflows/agent-maintenance-release-watch.yml`
- `dispatch_kind`: `workflow_dispatch`
- `branch_name`: `automation/claude_code-maintenance-<target_version>`

Responsibilities (high level):
- Download `manifest.json` and verify integrity (sha256 + size).
- Update `cli_manifests/claude_code/artifacts.lock.json`.
- Generate per-target help snapshots via `xtask claude-snapshot` (matrix).
- Generate a union snapshot via `xtask claude-union` (Linux).
- Generate wrapper coverage + reports + version metadata + validate:
  - `xtask claude-wrapper-coverage`
  - `xtask codex-report --root cli_manifests/claude_code`
  - `xtask codex-version-metadata --root cli_manifests/claude_code`
  - `xtask codex-validate --root cli_manifests/claude_code`
- Generate a triad scaffold under:
  - `.archived/project_management/next/claude-code-cli-parity-<version>/`
- Open a PR branch `automation/claude-code-<version>`.

After the worker PR exists, complete the maintainer-owned implementation step through the relay:

```bash
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml --dry-run
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml --write --run-id <prepared_run_id>
```

Use the recovery notes rendered in `HANDOFF.md` and `maintenance-request.toml` if PR creation or local relay preflight fails. Manual closeout remains explicit and outside relay write mode.

## Promotion (manual gate)

Promotion is a separate PR so it can be reviewed/approved independently.

Workflow:
- `.github/workflows/claude-code-promote.yml`

Responsibilities:
- Update `latest_validated.txt` and per-target pointers under `pointers/latest_validated/`.
- Update `current.json` to match the promoted union snapshot.
- Re-run `xtask codex-validate --root cli_manifests/claude_code`.
- Open a PR branch `automation/claude-code-promote-<version>`.

## Local Debugging Commands

- Validate committed artifacts:  
  `cargo run -p xtask -- codex-validate --root cli_manifests/claude_code`
- Regenerate wrapper coverage JSON:  
  `cargo run -p xtask -- claude-wrapper-coverage --out cli_manifests/claude_code/wrapper_coverage.json`
- Generate a per-target snapshot (no downloads; uses a local binary):  
  `cargo run -p xtask -- claude-snapshot --claude-binary <PATH> --out-file cli_manifests/claude_code/snapshots/<v>/<target>.json --capture-raw-help --raw-help-target <target> --supplement cli_manifests/claude_code/supplement/commands.json`
- Union snapshots:  
  `cargo run -p xtask -- claude-union --root cli_manifests/claude_code --version <v>`
