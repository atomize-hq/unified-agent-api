# Codex CLI Parity Ops Playbook

This document is the maintainer runbook for keeping this repo’s Rust wrapper in parity with upstream Codex CLI releases using the “snapshot → diff → update” workflow.

Key policy references:
- Source-of-truth policy/architecture: `docs/adr/0001-codex-cli-parity-maintenance.md`
- Snapshot artifacts home: `cli_manifests/codex/README.md`
- CI/automation plan (triggers, binary acquisition, PR loop): `cli_manifests/codex/CI_WORKFLOWS_PLAN.md`
- CI agent runbook (how to use reports to update wrapper): `cli_manifests/codex/CI_AGENT_RUNBOOK.md`
- PR body template (for automation PRs): `cli_manifests/codex/PR_BODY_TEMPLATE.md`

## Core Policies (read before operating)

- **No runtime downloads.** The crate must not download or update Codex binaries at runtime. Downloads happen only in CI/workflows (per ADR).
- **Promotion safety.** We only promote snapshots/pointers for upstream releases that are **non-prerelease** and **non-draft**, unless a human explicitly decides otherwise and records that decision in the ops log / PR notes.
- **Authoritative pointers.** Only these files are authoritative:
  - `cli_manifests/codex/min_supported.txt`
  - `cli_manifests/codex/latest_validated.txt`
  `cli_manifests/codex/current.json` is generated and should correspond to `latest_validated.txt`.
- **Help/real-binary reality wins.** Snapshot diffs and real-binary validations are the primary source of truth; “additional signals” (release notes mining, doc cross-checks) are alerts only.

## Live Upstream-Release Flow

The shipped maintenance path for Codex parity is:

1. `.github/workflows/agent-maintenance-release-watch.yml` detects stale `codex` parity from registry truth and dispatches `.github/workflows/codex-cli-update-snapshot.yml`.
2. The worker refreshes the Codex parity artifacts, runs `prepare-agent-maintenance --write`, and opens branch `automation/codex-maintenance-<target_version>` with PR body `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md`.
3. The maintainer reviews `docs/agents/lifecycle/codex-maintenance/HANDOFF.md` as canonical, `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md` as derivative, and `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml` as the surface that owns relay/write envelope/gates/recovery, then runs:

```bash
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --dry-run
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --write --run-id <prepared_run_id>
```

4. `execute-agent-maintenance --dry-run` is the required trust step before write mode. It validates the local execution host, prints the exact writable surfaces and green gates, and prepares the frozen run packet under `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`.
5. `execute-agent-maintenance --write` reuses that prepared baseline, enforces the request-owned write envelope, runs the request-owned green gates, and stops before closeout.
6. The maintainer reviews the diff and runs `close-agent-maintenance` explicitly. Closeout is never performed by the relay.

Boundaries:
- Promotion-only pointer changes such as `latest_validated.txt` and `min_supported.txt` remain separate maintainer actions and are not part of the upstream-release relay.
- Packet-only agents remain deferred; do not widen non-relay maintenance packets into `execute-agent-maintenance --write`.

## Release Watch: Triage Checklist

When the shared Release Watch workflow runs, or when you manually inspect a queued Codex maintenance item:

1. Confirm the alert is for a **stable** upstream release (not draft/prerelease).
2. Record:
   - upstream latest stable version
   - computed candidate version (per workflow policy)
   - release URLs (and any notable release-note callouts)
3. Decide whether to run the Update Snapshot workflow now:
   - If the candidate is new and we’re within normal maintenance cadence, proceed.
   - If the release is risky/large, proceed but expect follow-on triads (don’t “quick fix” in the ops PR).

## Run Update Snapshot (workflow_dispatch)

Normal operation is the shared watcher dispatch above. Use this workflow manually only to replay or repair the worker step for a known target version.

Preferred path: run the GitHub Actions workflow:
- `.github/workflows/codex-cli-update-snapshot.yml`

Required replay inputs:
- `agent_id`: `codex`
- `current_version`: the current validated Codex version from registry truth
- `latest_stable`: the latest stable upstream version seen by the watcher
- `target_version`: the worker target version to validate
- `opened_from`: the repo-relative worker workflow path, `.github/workflows/codex-cli-update-snapshot.yml`
- `detected_by`: `.github/workflows/agent-maintenance-release-watch.yml`
- `dispatch_kind`: `workflow_dispatch`
- `branch_name`: `automation/codex-maintenance-<target_version>`

Optional inputs:
- `update_min_supported`: keep `false`. `min_supported.txt` is a maintainer-only policy decision and must be updated in a separate PR.

Notes:
- The workflow is responsible for downloading/extracting the upstream release artifact(s) and updating `cli_manifests/codex/artifacts.lock.json`.
- The workflow should regenerate `cli_manifests/codex/current.json` (and optionally `cli_manifests/codex/raw_help/<version>/**`) using `xtask`.
- If the org/repo disables workflow write permissions for `GITHUB_TOKEN`, the workflow will not be able to open a PR; either:
  - configure a repo secret `CODEX_AUTOMATION_TOKEN` (PAT or GitHub App token with repo write) for PR creation, or
  - download the workflow artifact bundle and open a PR manually with the regenerated files.

## Automation PRs: Agent Instructions

The Update Snapshot workflow opens a PR branch like `automation/codex-maintenance-<target_version>`.

That PR is intentionally *not* self-closing. It is the maintainer handoff point for the local relay. The implementation work happens by:
- reviewing the generated request and canonical `HANDOFF.md`
- running `execute-agent-maintenance --dry-run`
- running `execute-agent-maintenance --write --run-id <prepared_run_id>`
- reviewing the resulting diff, then recording closeout explicitly with `close-agent-maintenance`

Use:
- `cli_manifests/codex/PR_BODY_TEMPLATE.md` as the PR description template (starts with `@codex`).
- `cli_manifests/codex/CI_AGENT_RUNBOOK.md` as the detailed operating instructions for the agent.

Recovery wording is frozen:
- If PR creation fails after packet generation, rerun packet regeneration from the frozen request and reopen the PR from the generated pr-summary path.
- If the local execution-host preflight (local Codex CLI host via execute-agent-maintenance) fails, fix the Codex binary/auth state and rerun `execute-agent-maintenance --dry-run` before write mode.

Use the machine-derived recovery instructions rendered in:
- `docs/agents/lifecycle/codex-maintenance/HANDOFF.md`
- `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`

### Target end state (v1)

See `cli_manifests/codex/CI_WORKFLOWS_PLAN.md` for the target end state:
- Release Watch dispatches Update Snapshot automatically for stable releases.
- Update Snapshot acquires binaries from GitHub Releases for Linux/macOS/Windows, generates per-target snapshots, merges a union, generates reports, and opens/updates a dedicated per-version PR branch for automation.

## Review Snapshot Diff (treat as a checklist)

In the snapshot PR, review diffs in `cli_manifests/codex/current.json` (and any `raw_help/**`) as a checklist:

- **Additions:** new commands/flags/config toggles
  - Decide: wrap now vs intentionally unwrapped (see below)
  - If wrap: plan follow-on triads (code/test/integ) for the new surface(s)
- **Removals/renames:** anything disappearing from help is high-signal
  - Confirm against the real binary and release notes
  - Plan compatibility shims or breaking-change handling as needed
- **Stability markers:** new “experimental/beta/deprecated” labels
  - Capture in snapshot (`stability`) and reassess whether the wrapper should expose it
- **Schema drift signals:** JSON/notification schema hints in help output or release notes
  - Use as prompts for review, but treat fixtures + real-binary outputs as the arbiter

### Intentionally Unwrapped Surfaces (explicit policy)

These surfaces are intentionally *not* wrapped unless/until they meet promotion criteria:

- Interactive TUI mode (`codex` with no args)
- Shell completion generation (`codex completion …`)
- `codex cloud exec` (experimental/setup-time utility unless it becomes core to embedding)
- Experimental MCP management commands (`codex mcp list/get/add/remove/login/logout`) unless they become stable and necessary

Operational representation (ADR 0004):
- Represent intentionally unwrapped command families as **IU subtree roots** in `crates/codex/src/wrapper_coverage_manifest.rs` (ADR 0004):
  - add a command entry for the parent path (e.g. `["completion"]`, `["cloud"]`, or `["mcp"]`) with `level: intentionally_unsupported` and a stable, non-empty `note`.
  - do **not** enumerate every descendant flag/arg; descendants are classified as IU by inheritance unless explicitly overridden.
- Reports will keep these surfaces visible under `deltas.intentionally_unsupported` (audit-friendly) while keeping `missing_*` actionable.

### Promotion Criteria (unwrapped → wrapped)

Promote an intentionally unwrapped surface to “wrapped” only when it meets all of the following:

1. **Stable enough:** marked stable (not experimental/beta) for at least **2 stable releases**.
2. **Embedding need:** needed for headless embedding (not primarily interactive UX).
3. **Testable:** exercisable via non-interactive tests (or explicitly gated live probes).
4. **Deterministic failures:** clear exit codes and/or machine-readable JSON errors.
5. **No new supply-chain behavior:** does not require new network/download behavior in the core crate.

## Update Pointers Safely (`min_supported.txt` / `latest_validated.txt`)

Use these rules when updating pointers:

- Only update `latest_validated.txt` after validations succeed (see next section) for the exact version you’re promoting.
- Only update `min_supported.txt` intentionally (policy change). Prefer leaving it stable until we have explicit intent + coverage.
- Ensure `cli_manifests/codex/current.json` corresponds to `latest_validated.txt` (same semantic version, generated from the validated binary).

## “Validated” Commands (Linux, isolated home)

Run the “validated” commands with an isolated home directory by setting **both** `CODEX_E2E_HOME` and `CODEX_HOME` to the same temporary directory.

Example (from repo root):

```bash
CODEX_E2E_HOME="$(mktemp -d)"
export CODEX_E2E_HOME
export CODEX_HOME="$CODEX_E2E_HOME"
export CODEX_E2E_BINARY="./codex-x86_64-unknown-linux-musl"

cargo test -p codex
cargo test -p codex --examples
cargo test -p codex --test cli_e2e -- --nocapture
```

Notes:
- Live/credentialed probes (if any) must remain opt-in and explicitly gated (do not enable by default in CI).
- If you are validating a different binary path, update `CODEX_E2E_BINARY` accordingly.

## Additional Signals (warn/alert only)

Use these signals to prompt human review; they are not the source of truth:

- **Release notes mining:** scan upstream release notes for backticked commands and `--flag` tokens; treat as “verify in snapshot/help/binary”.
- **Optional docs/reference cross-check:** compare to official docs as prompts for discrepancies; confirm via snapshot + real-binary behavior.

## Trial Run: 0.61.0 → 0.77.0 (Linux)

This is a one-time (or occasional) checklist to validate the operational loop on Linux using the known gap described in ADR 0001.

### Prereqs

- A locally available **0.61.0** Linux musl binary placed at:
  - `./codex-x86_64-unknown-linux-musl`
  This is a gitignored workspace artifact. CI obtains it by downloading/extracting the upstream asset `codex-x86_64-unknown-linux-musl.tar.gz`.
- A locally available **0.77.0** binary, one of:
  - `codex` on `PATH`, or
  - obtained via the Update Snapshot workflow, or
  - stored at `./.codex-bins/0.77.0/codex-x86_64-unknown-linux-musl` and symlinked into `./codex-x86_64-unknown-linux-musl` when running validations.

### Checklist

1. Snapshot 0.61.0 (baseline):
   - `cargo run -p xtask -- codex-snapshot --codex-binary ./codex-x86_64-unknown-linux-musl --out-dir cli_manifests/codex --capture-raw-help --supplement cli_manifests/codex/supplement/commands.json`
2. Snapshot 0.77.0 (candidate):
   - Point the generator at the 0.77.0 binary path (or update the symlink) and rerun the same command.
3. Review diffs as a checklist:
   - additions / removals / renames / stability markers
   - any “help omissions” that should be tracked via `cli_manifests/codex/supplement/commands.json`
4. Run Linux validations with isolated home:
   - Set `CODEX_E2E_HOME` and `CODEX_HOME` to the same `mktemp -d` directory
   - Set `CODEX_E2E_BINARY=./codex-x86_64-unknown-linux-musl` (pointing at the candidate binary)
   - Run: `cargo test -p codex`, `cargo test -p codex --examples`, `cargo test -p codex --test cli_e2e -- --nocapture`
5. Promote pointers (only after validation):
   - Update `cli_manifests/codex/latest_validated.txt` to `0.77.0`
   - Ensure `cli_manifests/codex/current.json` corresponds to `0.77.0`
   - Leave `min_supported.txt` at `0.61.0` unless you are explicitly changing policy
