<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# Handoff

This file is the canonical contributor execution contract for `claude_code` maintenance.

## Before you start: freeze this packet

The nightly watcher regenerates this packet every night for as long as this agent's
validated pointer trails upstream. Regeneration rewrites the request and force-pushes this
branch back to base, which destroys work committed to the branch and invalidates a closeout
bound to the previous request even when that closeout was never committed. Declare the freeze
**before your first adjudication**, not before the closeout command.

Commit exactly one file, to `staging` and never to this packet branch: the branch is inside the
tree the force-push replaces, so a marker carried there is destroyed by the operation it exists
to block.

```sh
git switch staging && git pull --ff-only
mkdir -p docs/agents/lifecycle/claude_code-maintenance/governance/automation-stand-down
cat > docs/agents/lifecycle/claude_code-maintenance/governance/automation-stand-down/2.1.267.toml <<'TOML'
schema_version = 1
agent_id = "claude_code"
target_version = "2.1.267"
reason = "closeout in progress"
request_recorded_at = "2026-09-23T08:34:47Z"
TOML
git add docs/agents/lifecycle/claude_code-maintenance/governance/automation-stand-down/2.1.267.toml
git commit docs/agents/lifecycle/claude_code-maintenance/governance/automation-stand-down/2.1.267.toml -m "chore(claude_code): stand automation down for 2.1.267"
git push origin staging
git switch -
```

The `git add` is required because the marker is always a new file, and the path on `git commit` is
what keeps everything else out of the commit. If `git switch` refuses, your tree is dirty: this
step runs before any packet work, so commit or stash that work first.

Confirm the freeze is live, from this branch:

```sh
cargo run -p xtask -- maintenance-stand-down-check --agent claude_code --target-version 2.1.267 --from-ref origin/staging
```

Nothing releases the freeze but retirement. Closing this PR, pushing to it, approving it and
merging it all leave it in force; the promotion PR for `2.1.267` removes the marker.

## Packet origin

- detected_by: `.github/workflows/agent-maintenance-release-watch.yml`
- current_validated: `2.1.29`
- target_version: `2.1.267`
- latest_stable: `2.1.267`
- version_policy: `upstream_stable_pointer`
- source_kind: `npm_dist_tag`
- source_ref: `@anthropic-ai/claude-code#stable`
- dispatch_kind: `packet_pr`
- dispatch_workflow: `agent-maintenance-open-pr.yml`
- branch_name: `automation/claude_code-maintenance-2.1.267`

## Support-surface audit

- required: `true`
- pre-run debt count: `2`
- expected post-run debt count: `2`
- discovered upstream surface rows: `2`
- preexisting unsupported rows: `2`
- required uplifts this run:
- `claude_code install` `install` via `unbaselined_gap`
- `claude_code install` `--force` via `unbaselined_gap`
- deferred preexisting gaps:
- `claude_code install` `install` via `requires_new_architectural_seam` (TODOS.md#close-claude-code-install-maintenance-gap)
- `claude_code install` `--force` via `requires_new_architectural_seam` (TODOS.md#close-claude-code-install-maintenance-gap)


## Relay contract

- maintained agent packet: `claude_code`
- local execution host: `local Codex CLI host via execute-agent-maintenance`
- executor surface: `execute-agent-maintenance`
- request artifact: `docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml`
- prompt template path: `docs/agents/lifecycle/claude_code-maintenance/governance/execute-agent-maintenance-prompt.md`
- prompt sha256: `297f0c7fa33ef3deb0d907006a2d098e77316d461dc7a46802841ce8b9289f19`
- canonical handoff: `docs/agents/lifecycle/claude_code-maintenance/HANDOFF.md`
- derivative pr summary: `docs/agents/lifecycle/claude_code-maintenance/governance/pr-summary.md`
- exact closeout artifact: `docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-closeout.json`
- branch linkage: `automation/claude_code-maintenance-2.1.267`
- manual closeout required: `true`

## Writable surfaces

- `docs/agents/lifecycle/claude_code-maintenance/**`
- `crates/claude_code/**`
- `crates/agent_api/**`
- `cli_manifests/claude_code/artifacts.lock.json`
- `cli_manifests/claude_code/snapshots/2.1.267/**`
- `cli_manifests/claude_code/reports/2.1.267/**`
- `cli_manifests/claude_code/versions/2.1.267.json`
- `cli_manifests/claude_code/wrapper_coverage.json`
- `cli_manifests/support_matrix/current.json`
- `docs/specs/unified-agent-api/support-matrix.md`
- `crates/agent_api/src/runtime_support_data.rs`
- `docs/specs/unified-agent-api/non-tui-support-debt.md`

## Read-only inputs

- `docs/agents/lifecycle/claude_code-maintenance/OPS_PLAYBOOK.md`
- `docs/agents/lifecycle/claude_code-maintenance/CI_WORKFLOWS_PLAN.md`
- `docs/agents/lifecycle/claude_code-maintenance/governance/execute-agent-maintenance-prompt.md`
- `.github/workflows/agent-maintenance-open-pr.yml`
- `docs/specs/unified-agent-api/non-tui-support-debt.md`

## Ordered repo commands

- `cargo fmt --all`
- `cargo run -p xtask -- codex-validate --root cli_manifests/claude_code`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix --check`
- `cargo run -p xtask -- capability-matrix-audit`
- `make preflight`

## Exact green gates

- `cargo fmt --all`
- `cargo run -p xtask -- codex-validate --root cli_manifests/claude_code`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix --check`
- `cargo run -p xtask -- capability-matrix-audit`
- `make preflight`

## Recovery

- recreate packet command: `cargo run -p xtask -- refresh-agent --request docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml --write`
- reopen pr body path: `docs/agents/lifecycle/claude_code-maintenance/governance/pr-summary.md`
- reopen pr branch: `automation/claude_code-maintenance-2.1.267`
- notes:
- If PR creation fails after packet generation, rerun packet regeneration from the frozen request and reopen the PR from the generated pr-summary path.
- If the local execution-host preflight (local Codex CLI host via execute-agent-maintenance) fails, fix the Codex binary/auth state and rerun `execute-agent-maintenance --dry-run` before write mode.

## Dry-run to write relay

Use the `run_id` printed by the dry-run output, replacing `RUN_ID_FROM_DRY_RUN` before invoking write mode.

```sh
cargo run -p xtask -- execute-agent-maintenance --dry-run --request docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml
cargo run -p xtask -- execute-agent-maintenance --write --request docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml --run-id RUN_ID_FROM_DRY_RUN
```

## Exact closeout command

```sh
cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-closeout.json
```

## Exact maintained-agent prompt

```md
# Packet PR Maintenance Prompt (`2.1.267`)

This template renders the exact maintained-agent prompt for `claude_code` packet execution.
`docs/agents/lifecycle/claude_code-maintenance/HANDOFF.md` remains canonical and `governance/pr-summary.md` is derivative.

@codex

## Goal

Execute the automated maintenance packet for `claude_code` target `2.1.267`.

## Frozen request contract

- Read `docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml` before changing code or docs.
- Read the packet-owned `support_surface_audit` block before deciding whether the run can succeed.
- Treat `docs/agents/lifecycle/claude_code-maintenance/HANDOFF.md` as canonical for writable surfaces, read-only inputs, ordered commands, green gates, and recovery.
- Treat `.github/workflows/agent-maintenance-open-pr.yml` as the opening workflow source.
- Do not write outside the execution contract frozen in the request packet.

## Manifest inputs

- `cli_manifests/claude_code/README.md`
- `cli_manifests/claude_code/VALIDATOR_SPEC.md`
- `cli_manifests/claude_code/RULES.json`
- `cli_manifests/claude_code/SCHEMA.json`
- `cli_manifests/claude_code/current.json`
- `cli_manifests/claude_code/latest_validated.txt`
- `cli_manifests/claude_code/wrapper_coverage.json`

## Required workflow

1. Compare the current validated baseline from `cli_manifests/claude_code/latest_validated.txt` against the target `2.1.267` artifacts.
2. Use `support_surface_audit` to classify newly discovered non-TUI surface, preexisting non-TUI debt, required uplifts, and allowed deferrals.
3. Land bounded wrapper/backend/manifest/publication updates for every row in `required_uplifts_this_run`.
4. Refresh or create version-scoped manifest artifacts under `cli_manifests/claude_code/snapshots/2.1.267/`, `cli_manifests/claude_code/reports/2.1.267/`, and `cli_manifests/claude_code/versions/2.1.267.json` as required by the packet.
5. Leave closeout manual; record it only with `close-agent-maintenance` after the declared green gates pass.

## Done criteria

- Changes stay within the writable surfaces frozen in `docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml`.
- No newly discovered non-TUI surface remains unresolved unless the packet records one allowed deferral.
- `cargo run -p xtask -- codex-validate --root cli_manifests/claude_code` passes.
- The remaining ordered commands and green gates from `docs/agents/lifecycle/claude_code-maintenance/HANDOFF.md` pass or are captured in maintainer follow-up notes.

```
