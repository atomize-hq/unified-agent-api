<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# Handoff

This file is the canonical contributor execution contract for `codex` maintenance.

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
mkdir -p docs/agents/lifecycle/codex-maintenance/governance/automation-stand-down
cat > docs/agents/lifecycle/codex-maintenance/governance/automation-stand-down/0.156.1.toml <<'TOML'
schema_version = 1
agent_id = "codex"
target_version = "0.156.1"
reason = "closeout in progress"
request_recorded_at = "2026-09-25T08:49:50Z"
TOML
git add docs/agents/lifecycle/codex-maintenance/governance/automation-stand-down/0.156.1.toml
git commit docs/agents/lifecycle/codex-maintenance/governance/automation-stand-down/0.156.1.toml -m "chore(codex): stand automation down for 0.156.1"
git push origin staging
git switch -
```

The `git add` is required because the marker is always a new file, and the path on `git commit` is
what keeps everything else out of the commit. If `git switch` refuses, your tree is dirty: this
step runs before any packet work, so commit or stash that work first.

Confirm the freeze is live, from this branch:

```sh
cargo run -p xtask -- maintenance-stand-down-check --agent codex --target-version 0.156.1 --from-ref origin/staging
```

Nothing releases the freeze but retirement. Closing this PR, pushing to it, approving it and
merging it all leave it in force; the promotion PR for `0.156.1` removes the marker.

## Packet origin

- detected_by: `.github/workflows/agent-maintenance-release-watch.yml`
- current_validated: `0.125.0`
- target_version: `0.156.1`
- latest_stable: `0.157.0`
- version_policy: `latest_stable_minus_one`
- source_kind: `github_releases`
- source_ref: `openai/codex`
- dispatch_kind: `packet_pr`
- dispatch_workflow: `agent-maintenance-open-pr.yml`
- branch_name: `automation/codex-maintenance-0.156.1`

## Support-surface audit

- required: `true`
- pre-run debt count: `2`
- expected post-run debt count: `2`
- discovered upstream surface rows: `56`
- preexisting unsupported rows: `2`
- required uplifts this run:
- `codex agents` `agents` via `unbaselined_gap`
- `codex completion` `completion` via `unbaselined_gap`
- `codex migrate-rollouts` `migrate-rollouts` via `unbaselined_gap`
- `codex queue` `queue` via `unbaselined_gap`
- `codex app-server` `--code-mode-host` via `unbaselined_gap`
- `codex app-server daemon update` `--from-cli` via `unbaselined_gap`
- `codex app-server daemon update` `--yes` via `unbaselined_gap`
- `codex exec` `--thread-source` via `unbaselined_gap`
- `codex exec fork` `--ephemeral` via `unbaselined_gap`
- `codex exec fork` `--ignore-rules` via `unbaselined_gap`
- `codex exec fork` `--ignore-user-config` via `unbaselined_gap`
- `codex exec fork` `--json` via `unbaselined_gap`
- `codex exec fork` `--output-last-message` via `unbaselined_gap`
- `codex exec fork` `--output-schema` via `unbaselined_gap`
- `codex exec fork` `--skip-git-repo-check` via `unbaselined_gap`
- `codex exec fork` `--thread-source` via `unbaselined_gap`
- `codex exec resume` `--thread-source` via `unbaselined_gap`
- `codex exec review` `--thread-source` via `unbaselined_gap`
- `codex exec-server` `--aws-profile` via `unbaselined_gap`
- `codex exec-server` `--aws-region` via `unbaselined_gap`
- `codex exec-server` `--aws-service` via `unbaselined_gap`
- `codex exec-server` `--aws-sigv4` via `unbaselined_gap`
- `codex exec-server` `--concurrent-requests` via `unbaselined_gap`
- `codex exec-server` `--exit-on-stdin-close` via `unbaselined_gap`
- `codex exec-server` `--remote-transport` via `unbaselined_gap`
- `codex exec-server forward` `--aws-profile` via `unbaselined_gap`
- `codex exec-server forward` `--aws-region` via `unbaselined_gap`
- `codex exec-server forward` `--aws-service` via `unbaselined_gap`
- `codex exec-server forward` `--aws-sigv4` via `unbaselined_gap`
- `codex exec-server forward` `--connect` via `unbaselined_gap`
- `codex exec-server forward` `--environment-id` via `unbaselined_gap`
- `codex exec-server forward` `--exit-on-stdin-close` via `unbaselined_gap`
- `codex exec-server forward` `--name` via `unbaselined_gap`
- `codex exec-server forward` `--remote-transport` via `unbaselined_gap`
- `codex exec-server forward` `--use-agent-identity-auth` via `unbaselined_gap`
- `codex mcp add` `--oauth-client-registration` via `unbaselined_gap`
- `codex mcp login` `--no-browser` via `unbaselined_gap`
- `codex mcp login` `--oauth-client-registration` via `unbaselined_gap`
- `codex migrate-rollouts` `--apply` via `unbaselined_gap`
- `codex migrate-rollouts` `--json` via `unbaselined_gap`
- `codex migrate-rollouts` `--max-mib-per-second` via `unbaselined_gap`
- `codex migrate-rollouts` `--thread` via `unbaselined_gap`
- `codex migrate-rollouts` `--verbose` via `unbaselined_gap`
- `codex queue` `--message` via `unbaselined_gap`
- `codex queue` `--thread` via `unbaselined_gap`
- `codex` `--approve-for-me` via `unbaselined_gap`
- `codex` `--no-daemon` via `unbaselined_gap`
- `codex` `--worktree` via `unbaselined_gap`
- `codex completion` `SHELL` via `unbaselined_gap`
- `codex exec fork` `PROMPT` via `unbaselined_gap`
- `codex exec fork` `SESSION_ID` via `unbaselined_gap`
- `codex exec-server help` `COMMAND` via `unbaselined_gap`
- `codex app-server daemon update` `update` via `unbaselined_gap`
- `codex exec fork` `fork` via `unbaselined_gap`
- `codex exec-server forward` `forward` via `unbaselined_gap`
- `codex exec-server help` `help` via `unbaselined_gap`
- deferred preexisting gaps:
- `codex completion` `completion` via `requires_new_architectural_seam` (TODOS.md#close-codex-completion-maintenance-gap)
- `codex completion` `SHELL` via `requires_new_architectural_seam` (TODOS.md#close-codex-completion-maintenance-gap)


## Relay contract

- maintained agent packet: `codex`
- local execution host: `local Codex CLI host via execute-agent-maintenance`
- executor surface: `execute-agent-maintenance`
- request artifact: `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
- prompt template path: `docs/agents/lifecycle/codex-maintenance/governance/execute-agent-maintenance-prompt.md`
- prompt sha256: `2498dea39ad568795819b81b7fee2f728f91904d2a79720bbadc88b3a4a3e963`
- canonical handoff: `docs/agents/lifecycle/codex-maintenance/HANDOFF.md`
- derivative pr summary: `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md`
- exact closeout artifact: `docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json`
- branch linkage: `automation/codex-maintenance-0.156.1`
- manual closeout required: `true`

## Writable surfaces

- `docs/agents/lifecycle/codex-maintenance/**`
- `crates/codex/**`
- `crates/agent_api/**`
- `cli_manifests/codex/artifacts.lock.json`
- `cli_manifests/codex/snapshots/0.156.1/**`
- `cli_manifests/codex/reports/0.156.1/**`
- `cli_manifests/codex/versions/0.156.1.json`
- `cli_manifests/codex/wrapper_coverage.json`
- `cli_manifests/support_matrix/current.json`
- `docs/specs/unified-agent-api/support-matrix.md`
- `crates/agent_api/src/runtime_support_data.rs`
- `docs/specs/unified-agent-api/non-tui-support-debt.md`
- `docs/specs/codex-wrapper-coverage-scenarios-v1.md`

## Read-only inputs

- `docs/agents/lifecycle/codex-maintenance/OPS_PLAYBOOK.md`
- `docs/agents/lifecycle/codex-maintenance/CI_WORKFLOWS_PLAN.md`
- `docs/agents/lifecycle/codex-maintenance/governance/execute-agent-maintenance-prompt.md`
- `.github/workflows/agent-maintenance-open-pr.yml`

## Ordered repo commands

- `cargo fmt --all`
- `cargo run -p xtask -- codex-validate --root cli_manifests/codex`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix --check`
- `cargo run -p xtask -- capability-matrix-audit`
- `make preflight`

## Exact green gates

- `cargo fmt --all`
- `cargo run -p xtask -- codex-validate --root cli_manifests/codex`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix --check`
- `cargo run -p xtask -- capability-matrix-audit`
- `make preflight`

## Recovery

- recreate packet command: `cargo run -p xtask -- refresh-agent --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --write`
- reopen pr body path: `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md`
- reopen pr branch: `automation/codex-maintenance-0.156.1`
- notes:
- If PR creation fails after packet generation, rerun packet regeneration from the frozen request and reopen the PR from the generated pr-summary path.
- If the local execution-host preflight (local Codex CLI host via execute-agent-maintenance) fails, fix the Codex binary/auth state and rerun `execute-agent-maintenance --dry-run` before write mode.

## Dry-run to write relay

Use the `run_id` printed by the dry-run output, replacing `RUN_ID_FROM_DRY_RUN` before invoking write mode.

```sh
cargo run -p xtask -- execute-agent-maintenance --dry-run --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml
cargo run -p xtask -- execute-agent-maintenance --write --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --run-id RUN_ID_FROM_DRY_RUN
```

## Exact closeout command

```sh
cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json
```

## Exact maintained-agent prompt

```md
# Packet PR Maintenance Prompt (`0.156.1`)

This template renders the exact maintained-agent prompt for `codex` packet execution.
`docs/agents/lifecycle/codex-maintenance/HANDOFF.md` remains canonical and `governance/pr-summary.md` is derivative.

@codex

## Goal

Execute the automated maintenance packet for `codex` target `0.156.1`.

## Frozen request contract

- Read `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml` before changing code or docs.
- Read the packet-owned `support_surface_audit` block before deciding whether the run can succeed.
- Treat `docs/agents/lifecycle/codex-maintenance/HANDOFF.md` as canonical for writable surfaces, read-only inputs, ordered commands, green gates, and recovery.
- Treat `.github/workflows/agent-maintenance-open-pr.yml` as the opening workflow source.
- Do not write outside the execution contract frozen in the request packet.

## Manifest inputs

- `cli_manifests/codex/README.md`
- `cli_manifests/codex/VALIDATOR_SPEC.md`
- `cli_manifests/codex/RULES.json`
- `cli_manifests/codex/SCHEMA.json`
- `cli_manifests/codex/current.json`
- `cli_manifests/codex/latest_validated.txt`
- `cli_manifests/codex/wrapper_coverage.json`

## Required workflow

1. Compare the current validated baseline from `cli_manifests/codex/latest_validated.txt` against the target `0.156.1` artifacts.
2. Use `support_surface_audit` to classify newly discovered non-TUI surface, preexisting non-TUI debt, required uplifts, and allowed deferrals.
3. For each `deferred_preexisting_gaps` row that also appears in `required_uplifts_this_run`, decide whether its `defer_reason` still holds at `0.156.1`. If it does, re-authorize that identity's existing debt row or rows in `docs/specs/unified-agent-api/non-tui-support-debt.md` in place:
   - set `authorized_at_version` to `0.156.1`;
   - set `scope_target_triples` so the rows together cover exactly the targets whose `cli_manifests/codex/reports/0.156.1/coverage.<target>.json` lists the surface, with no target in two rows;
   - set `authorization_evidence_ref` to `cli_manifests/codex/reports/0.156.1/coverage.any.json`.

   Change no other field and add no row. If the blocker no longer holds, treat the row as an uplift.
4. Land bounded wrapper/backend/manifest/publication updates for every remaining row in `required_uplifts_this_run`. Newly discovered surface is never deferred (maintenance-request contract field invariant 3), and no debt row may be added.
5. Refresh or create version-scoped manifest artifacts under `cli_manifests/codex/snapshots/0.156.1/`, `cli_manifests/codex/reports/0.156.1/`, and `cli_manifests/codex/versions/0.156.1.json` as required by the packet.
6. Leave closeout manual; record it only with `close-agent-maintenance` after the declared green gates pass.

## Done criteria

- Changes stay within the writable surfaces frozen in `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`.
- Every row in `required_uplifts_this_run` is uplifted, or is a preexisting debt row re-authorized at `0.156.1`; newly discovered surface is never deferred.
- `cargo run -p xtask -- codex-validate --root cli_manifests/codex` passes.
- `cargo run -p xtask -- maintenance-audit-status --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml` exits 0.
- The remaining ordered commands and green gates from `docs/agents/lifecycle/codex-maintenance/HANDOFF.md` pass or are captured in maintainer follow-up notes.

```
