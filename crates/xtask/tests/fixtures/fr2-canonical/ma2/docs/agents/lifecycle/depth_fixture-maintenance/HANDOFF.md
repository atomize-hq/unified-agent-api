<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# Handoff

This file is the canonical contributor execution contract for `depth_fixture` maintenance.

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
mkdir -p docs/agents/lifecycle/depth_fixture-maintenance/governance/automation-stand-down
cat > docs/agents/lifecycle/depth_fixture-maintenance/governance/automation-stand-down/0.2.0.toml <<'TOML'
schema_version = 1
agent_id = "depth_fixture"
target_version = "0.2.0"
reason = "closeout in progress"
request_recorded_at = "2026-11-03T00:00:00Z"
TOML
git add docs/agents/lifecycle/depth_fixture-maintenance/governance/automation-stand-down/0.2.0.toml
git commit docs/agents/lifecycle/depth_fixture-maintenance/governance/automation-stand-down/0.2.0.toml -m "chore(depth_fixture): stand automation down for 0.2.0"
git push origin staging
git switch -
```

The `git add` is required because the marker is always a new file, and the path on `git commit` is
what keeps everything else out of the commit. If `git switch` refuses, your tree is dirty: this
step runs before any packet work, so commit or stash that work first.

Confirm the freeze is live, from this branch:

```sh
cargo run -p xtask -- maintenance-stand-down-check --agent depth_fixture --target-version 0.2.0 --from-ref origin/staging
```

Nothing releases the freeze but retirement. Closing this PR, pushing to it, approving it and
merging it all leave it in force; the promotion PR for `0.2.0` removes the marker.

## Packet origin

- detected_by: `fixture`
- current_validated: `0.1.0`
- target_version: `0.2.0`
- latest_stable: `0.3.0`
- version_policy: `latest_stable_minus_one`
- source_kind: `github_releases`
- source_ref: `fixture/isolated`
- dispatch_kind: `packet_pr`
- dispatch_workflow: `agent-maintenance-open-pr.yml`
- branch_name: `automation/depth_fixture-maintenance-0.2.0`

## Support-surface audit

- required: `true`
- pre-run debt count: `0`
- expected post-run debt count: `0`
- discovered upstream surface rows: `0`
- preexisting unsupported rows: `0`
- required uplifts this run:
- none
- deferred preexisting gaps:
- none


## Relay contract

Guarded lifecycle command names appear below as contract metadata or as instructions for an actor outside the relay. An agent executing this packet inside a relay session must not invoke them.

- maintained agent packet: `depth_fixture`
- local execution host: `local Codex CLI host via execute-agent-maintenance`
- executor surface: `execute-agent-maintenance`
- request artifact: `docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-request.toml`
- prompt template path: `docs/agents/lifecycle/depth_fixture-maintenance/governance/execute-agent-maintenance-prompt.md`
- prompt sha256: `9df6511586dad42060921e6766dd1d3edae9307b16dc4d1cccb61e95340b44d8`
- canonical handoff: `docs/agents/lifecycle/depth_fixture-maintenance/HANDOFF.md`
- derivative pr summary: `docs/agents/lifecycle/depth_fixture-maintenance/governance/pr-summary.md`
- exact closeout artifact: `docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-closeout.json`
- branch linkage: `automation/depth_fixture-maintenance-0.2.0`
- manual closeout required: `true`

## Writable surfaces

- `docs/agents/lifecycle/depth_fixture-maintenance/**`
- `crates/depth_fixture/**`
- `crates/agent_api/**`
- `cli_manifests/depth_fixture/artifacts.lock.json`
- `cli_manifests/depth_fixture/snapshots/0.2.0/**`
- `cli_manifests/depth_fixture/reports/0.2.0/**`
- `cli_manifests/depth_fixture/versions/0.2.0.json`
- `cli_manifests/depth_fixture/wrapper_coverage.json`
- `cli_manifests/support_matrix/current.json`
- `docs/specs/unified-agent-api/support-matrix.md`
- `crates/agent_api/src/runtime_support_data.rs`
- `docs/specs/unified-agent-api/non-tui-support-debt.md`

## Read-only inputs

- `docs/agents/lifecycle/depth_fixture-maintenance/OPS_PLAYBOOK.md`
- `docs/agents/lifecycle/depth_fixture-maintenance/CI_WORKFLOWS_PLAN.md`
- `docs/agents/lifecycle/depth_fixture-maintenance/governance/execute-agent-maintenance-prompt.md`
- `.github/workflows/agent-maintenance-open-pr.yml`

## Ordered repo commands

- `cargo fmt --all`
- `cargo run -p xtask -- codex-validate --root cli_manifests/depth_fixture`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix --check`
- `cargo run -p xtask -- capability-matrix-audit`
- `make preflight`

## Exact green gates

- `cargo fmt --all`
- `cargo run -p xtask -- codex-validate --root cli_manifests/depth_fixture`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix --check`
- `cargo run -p xtask -- capability-matrix-audit`
- `make preflight`

## Recovery

Packet recovery is a maintainer action run from outside a relay session. An agent executing this packet inside a relay session must not run any command named in this Recovery section, including its notes.

- recreate packet command: `cargo run -p xtask -- refresh-agent --request docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-request.toml --write`
- reopen pr body path: `docs/agents/lifecycle/depth_fixture-maintenance/governance/pr-summary.md`
- reopen pr branch: `automation/depth_fixture-maintenance-0.2.0`
- notes:
- If PR creation fails after packet generation, rerun packet regeneration from the frozen request and reopen the PR from the generated pr-summary path.
- If the local execution-host preflight (local Codex CLI host via execute-agent-maintenance) fails, fix the Codex binary/auth state and rerun `execute-agent-maintenance --dry-run` before write mode.

## Dry-run to write relay

Starting the relay is a maintainer action run from outside a relay session. An agent executing this packet inside a relay session must not run either command. The maintainer uses the `run_id` printed by the dry-run output, replacing `RUN_ID_FROM_DRY_RUN` before invoking write mode.

```sh
cargo run -p xtask -- execute-agent-maintenance --dry-run --request docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-request.toml
cargo run -p xtask -- execute-agent-maintenance --write --request docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-request.toml --run-id RUN_ID_FROM_DRY_RUN
```

## Exact closeout command

After the declared green gates pass, the actor handed the packet PR records the closeout. An agent executing this packet inside a relay session must not run this command.

```sh
cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-closeout.json
```

## Exact maintained-agent prompt

```md
# Packet PR Maintenance Prompt (`0.2.0`)

This template renders the exact maintained-agent prompt for `depth_fixture` packet execution.
`docs/agents/lifecycle/depth_fixture-maintenance/HANDOFF.md` remains canonical and `governance/pr-summary.md` is derivative.

@codex

## Goal

Execute the automated maintenance packet for `depth_fixture` target `0.2.0`.

## Frozen request contract

- Read `docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-request.toml` before changing code or docs.
- Read the packet-owned `support_surface_audit` block before deciding whether the run can succeed.
- Treat `docs/agents/lifecycle/depth_fixture-maintenance/HANDOFF.md` as canonical for writable surfaces, read-only inputs, ordered commands, green gates, and recovery.
- Treat `.github/workflows/agent-maintenance-open-pr.yml` as the opening workflow source.
- Never invoke `execute-agent-maintenance`, `prepare-agent-maintenance`, or `refresh-agent`. If this prompt was delivered by `execute-agent-maintenance`, that process is the executor and is already running; lifecycle queries remain available. `HANDOFF.md` is the agent's contract for writable surfaces, read-only inputs, ordered commands, green gates, and the freeze step; its relay and recovery sections describe maintainer actions that start or recreate a run, and its closeout section identifies the closeout actor.
- Do not write outside the execution contract frozen in the request packet.

## Manifest inputs

- `cli_manifests/depth_fixture/README.md`
- `cli_manifests/depth_fixture/VALIDATOR_SPEC.md`
- `cli_manifests/depth_fixture/RULES.json`
- `cli_manifests/depth_fixture/SCHEMA.json`
- `cli_manifests/depth_fixture/current.json`
- `cli_manifests/depth_fixture/latest_validated.txt`
- `cli_manifests/depth_fixture/wrapper_coverage.json`

## Required workflow

1. Compare the current validated baseline from `cli_manifests/depth_fixture/latest_validated.txt` against the target `0.2.0` artifacts.
2. Use `support_surface_audit` to classify newly discovered non-TUI surface, preexisting non-TUI debt, required uplifts, and allowed deferrals.
3. For each `deferred_preexisting_gaps` row that also appears in `required_uplifts_this_run`, decide whether its `defer_reason` still holds at `0.2.0`. If it does, re-authorize that identity's existing debt row or rows in `docs/specs/unified-agent-api/non-tui-support-debt.md` in place:
   - set `authorized_at_version` to `0.2.0`;
   - set `scope_target_triples` so the rows together cover exactly the targets whose `cli_manifests/depth_fixture/reports/0.2.0/coverage.<target>.json` lists the surface, with no target in two rows;
   - set `authorization_evidence_ref` to `cli_manifests/depth_fixture/reports/0.2.0/coverage.any.json`.

   Change no other field and add no row. If the blocker no longer holds, treat the row as an uplift.
4. Land bounded wrapper/backend/manifest/publication updates for every remaining row in `required_uplifts_this_run`. Newly discovered surface is never deferred (maintenance-request contract field invariant 3), and no debt row may be added.
5. Refresh or create version-scoped manifest artifacts under `cli_manifests/depth_fixture/snapshots/0.2.0/`, `cli_manifests/depth_fixture/reports/0.2.0/`, and `cli_manifests/depth_fixture/versions/0.2.0.json` as required by the packet.
6. An agent executing this packet inside a relay session does not run `close-agent-maintenance` or `prepare-agent-closeout`; after the declared green gates pass, the actor handed the packet PR records the closeout.

## Done criteria

- Changes stay within the writable surfaces frozen in `docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-request.toml`.
- Every row in `required_uplifts_this_run` is uplifted, or is a preexisting debt row re-authorized at `0.2.0`; newly discovered surface is never deferred.
- `cargo run -p xtask -- codex-validate --root cli_manifests/depth_fixture` passes.
- `cargo run -p xtask -- maintenance-audit-status --request docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-request.toml` exits 0.
- The remaining ordered commands and green gates from `docs/agents/lifecycle/depth_fixture-maintenance/HANDOFF.md` pass or are captured in maintainer follow-up notes.

```
