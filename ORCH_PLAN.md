# Enclose The Publication Lane End To End Orchestration Plan

## Summary

- Execute only against the current repo-root milestone in `PLAN.md`: `Enclose The Publication Lane End To End`.
- Integration branch is the live implementation branch `codex/recommend-next-agent`.
- Execution baseline for this orchestration is current HEAD `000aa69` on `2026-05-02`.
- `PLAN.md` still records `Plan commit baseline: bfd6fd4`; treat that as milestone-history metadata only, never as the worker fork point.
- Parent agent is the only integrator, merger, rebase authority, canonical stale-sweep owner, committed artifact normalizer, and final verifier.
- Worker model for delegated lanes:
  - model: GPT-5.4
  - reasoning_effort: high
- Maximum worker concurrency is `3`, and only after the parent freezes the command contract and completes the stale-string/hash normalization sweep.
- No human approval pause is planned. Stop only for hard guards, stale-lane invalidation, or verification failure.
- This document supersedes the prior branch-local orchestration draft for this milestone.

## Hard Guards

- Scope is locked to the `PLAN.md` slice only.
- Do not widen into:
  - `LifecycleStage::Published` redesign
  - new lifecycle stages
  - support-matrix semantics changes
  - capability-matrix semantics changes
  - maintenance request schema changes
  - runtime-follow-on redesign
  - repair-runtime-evidence redesign
  - new artifact families beyond the required command and its lifecycle/docs/test fallout
  - generic JSON-driven command execution
- Locked implementation decisions from `PLAN.md` must remain intact:
  - add `refresh-publication --approval <path> --check|--write`
  - keep `prepare-publication` as handoff writer only
  - consume `publication-ready.json` as the only create-mode publication packet
  - keep `PublicationReadyPacket.required_publication_outputs` as the authoritative write set
  - share support/capability publication planning between create-mode and maintenance-mode
  - make `refresh-publication --write` transactional for publication-owned committed surfaces
  - keep `make preflight` inside the green gate
  - do not solve `LifecycleStage::Published` in this slice
- Any fixture or committed lifecycle artifact whose `expected_next_command` or reconstructed packet content depends on `PUBLICATION_READY_NEXT_COMMAND` must be swept and normalized before the downstream test worker launches.
- Workers do not touch parent-owned files.
- Workers do not merge, rebase, regenerate final publication outputs, or run the live aider verification flow.
- Never revert someone else’s edits. Baseline task must capture actual dirty state before worker launch.

## Orchestration State

Parent-owned orchestration state lives under:

- `RUN_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.runs/enclose-publication-lane-e2e`

Canonical parent-owned state files:

- `baseline.json`
  - branch, HEAD SHA, dirty-state summary, launch timestamp
  - note that execution baseline is `000aa69`
  - note that `PLAN.md` historical baseline is `bfd6fd4`
  - note of any lane delayed because of pre-existing dirt
- `tasks.json`
  - ordered tasks
  - owner
  - status
  - dependencies
  - restart count
- `session-log.md`
  - launch notes
  - freeze decisions
  - stale-sweep findings
  - worker prompts issued
  - relaunch reasons
  - merge and verification notes
- `freeze.json`
  - freeze commit SHA
  - frozen CLI contract for `refresh-publication`
  - frozen lifecycle next-command templates
  - frozen shared publication planning seam
  - stale-sweep invariants
  - lane ownership map
  - forbidden touch surfaces
- `merge-log.md`
  - merge order
  - smoke-command results after each merge
  - rejection/relaunch reasons
- `acceptance.md`
  - final ordered command sequence
  - exit codes
  - pass/fail per acceptance criterion
  - stale-string sweep results
  - unrelated blockers, if any

Per-task sentinels live under:

- `.runs/task-epl-00-baseline/**`
- `.runs/task-epl-01-freeze-core/**`
- `.runs/task-epl-02-parent-stale-sweep/**`
- `.runs/task-epl-03-maintenance-alignment/**`
- `.runs/task-epl-04-authored-docs/**`
- `.runs/task-epl-05-tests-fixtures/**`
- `.runs/task-epl-06-converge/**`
- `.runs/task-epl-07-final-verify/**`

Sentinel rules:

- parent creates `started.json` before work begins
- parent updates `status.json` during long tasks or after worker return
- parent writes exactly one terminal sentinel: `done.json` or `blocked.json`
- workers do not write orchestration state

## Worker Model

- Parent owns all orchestration, integration, freeze decisions, stale-string normalization, hash fallout normalization, merge sequencing, live verification, and acceptance decisions.
- Workers edit only their assigned lane files.
- Workers do not merge branches.
- Workers do not rebase.
- Workers do not edit `PLAN.md` or `.runs/**`.
- Workers do not regenerate final publication outputs or committed lifecycle docs.
- Workers return only:
  - changed files
  - commands run
  - exit codes
  - blockers or assumptions
- If a worker needs a parent-owned file changed, it stops and returns a blocker.

## Context-Control Rules

Parent active context stays narrow:

- `PLAN.md`
- this orchestration plan
- `freeze.json`
- `tasks.json`
- `session-log.md`
- latest integration diff summary
- only the files for the current task

Worker prompts contain only:

- owned files
- forbidden touch surfaces
- relevant `PLAN.md` excerpt
- frozen command/lifecycle templates from `freeze.json`
- the stale-sweep findings relevant to that lane
- required commands
- lane-local acceptance target

Workers are closed immediately after merge or rejection. No idle worker remains open after its result is consumed.

## Parent vs Worker Ownership Model

### Parent-only ownership

The parent keeps these surfaces local from kickoff through final verification:

- `crates/xtask/src/publication_refresh.rs`
- `crates/xtask/src/main.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/agent_lifecycle.rs`
- `crates/xtask/src/prepare_publication.rs`
- `crates/xtask/src/workspace_mutation.rs` only if rollback support needs a shared helper
- all merge/rebase/conflict resolution
- all stale-string sweep edits across authored docs, lifecycle artifacts, and impacted tests
- all fixture/hash normalization caused by `PUBLICATION_READY_NEXT_COMMAND` changes
- final generated publication outputs:
  - `cli_manifests/support_matrix/current.json`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- committed lifecycle and packet artifacts affected by next-command or reconstructed-hash fallout:
  - `docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json`
  - `docs/agents/lifecycle/codex-cli-onboarding/governance/publication-ready.json`
  - `docs/agents/lifecycle/claude-code-cli-onboarding/governance/lifecycle-state.json`
  - `docs/agents/lifecycle/claude-code-cli-onboarding/governance/publication-ready.json`
  - `docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json`
  - `docs/agents/lifecycle/gemini-cli-onboarding/governance/publication-ready.json`
  - `docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json`
  - `docs/agents/lifecycle/opencode-cli-onboarding/governance/publication-ready.json`
  - `docs/agents/lifecycle/aider-onboarding/governance/lifecycle-state.json`
- live create-mode verification artifacts for aider:
  - `docs/agents/lifecycle/aider-onboarding/governance/publication-ready.json`
- `.runs/enclose-publication-lane-e2e/**`

Parent responsibilities:

- capture baseline and actual dirty state
- freeze the command contract and shared planning seam
- run the authored-doc/test/lifecycle stale-string sweep immediately after freeze
- normalize any historical or fixture hash fallout before test worker launch
- launch workers only from the post-sweep parent commit
- reject stale worker output
- merge in order
- run targeted tests, search sweeps, live verification, and final acceptance

### Worker lane ownership

Lane B: maintenance alignment

- `crates/xtask/src/agent_maintenance/refresh.rs`
- `crates/xtask/tests/agent_maintenance_refresh.rs`

Lane C: authored docs

- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/specs/cli-agent-onboarding-charter.md`

Lane D: tests and fixture-facing regression coverage

- `crates/xtask/tests/refresh_publication_entrypoint.rs`
- `crates/xtask/tests/prepare_publication_entrypoint.rs`
- `crates/xtask/tests/agent_lifecycle_state.rs`
- `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
- `crates/xtask/tests/repair_runtime_evidence_entrypoint.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_paths.rs`
- `crates/xtask/tests/support/agent_maintenance_drift_harness.rs`
- `crates/xtask/tests/support/onboard_agent_harness.rs` only if required by the new entrypoint tests

Workers do not own:

- `crates/xtask/src/publication_refresh.rs`
- `crates/xtask/src/main.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/agent_lifecycle.rs`
- `crates/xtask/src/prepare_publication.rs`
- committed lifecycle docs under `docs/agents/lifecycle/**`
- generated publication outputs
- `.runs/**`

## Worktree/Branch Plan With Concrete Names

Canonical paths:

- `REPO_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api`
- `WORKTREE_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/unified-agent-api-enclose-publication-lane`

Integration surface:

- branch: `codex/recommend-next-agent`
- worktree: `REPO_ROOT`
- owner: parent only

Worker branches and worktrees, created only after the parent records the post-sweep freeze commit:

- Lane B:
  - branch: `codex/recommend-next-agent-maintenance-publication-align`
  - worktree: `$WORKTREE_ROOT/maintenance-publication-align`
- Lane C:
  - branch: `codex/recommend-next-agent-publication-docs`
  - worktree: `$WORKTREE_ROOT/publication-docs`
- Lane D:
  - branch: `codex/recommend-next-agent-publication-tests`
  - worktree: `$WORKTREE_ROOT/publication-tests`

Creation commands after `task/epl-02-parent-stale-sweep`:

```bash
mkdir -p "$WORKTREE_ROOT" "$RUN_ROOT"
POST_SWEEP_SHA=$(jq -r '.post_sweep_commit_sha' "$RUN_ROOT/freeze.json")

git worktree add -b codex/recommend-next-agent-maintenance-publication-align \
  "$WORKTREE_ROOT/maintenance-publication-align" \
  "$POST_SWEEP_SHA"

git worktree add -b codex/recommend-next-agent-publication-docs \
  "$WORKTREE_ROOT/publication-docs" \
  "$POST_SWEEP_SHA"

git worktree add -b codex/recommend-next-agent-publication-tests \
  "$WORKTREE_ROOT/publication-tests" \
  "$POST_SWEEP_SHA"
```

Worktree rules:

- never fork workers from `main`
- never fork workers before the parent stale sweep and hash normalization complete
- never reuse a stale worker worktree
- workers never merge back themselves

## Freeze Artifact And Restart Rule

The initial freeze point is the first parent commit where these are simultaneously true:

- `refresh-publication` exists and parses `--approval <path> --check|--write`
- `main.rs` wires the new subcommand
- `lib.rs` exports the new module if needed by tests
- `prepare_publication.rs` points `publication_ready` next-step semantics at `refresh-publication`
- `agent_lifecycle.rs` contains the frozen pre-refresh and post-refresh next-command templates
- the shared publication planning seam used by create-mode is callable and parent-approved
- `freeze.json` exists and is complete

`freeze.json` must record:

- `freeze_commit_sha`
- `post_sweep_commit_sha`
- `refresh_publication_cli`:
  - `approval` required
  - mutually exclusive `check` / `write`
- `prepare_publication_next_command_template`:
  - `refresh-publication --approval <path> --write`
- `post_refresh_next_command_template`:
  - `close-proving-run --approval <path> --closeout docs/agents/lifecycle/<prefix>/governance/proving-run-closeout.json`
- `required_publication_commands`:
  - still the four green-gate commands ending in `make preflight`
- frozen shared planning seam:
  - exact function names/signatures exposed for maintenance reuse
  - exact file owning the seam
- stale-sweep invariants:
  - exact raw shell-chain string that must no longer remain canonical in authored workflow docs, committed lifecycle artifacts, or impacted `crates/xtask/tests/**` fixtures
  - valid contexts where `prepare-publication --approval .* --write` still remains canonical
  - contexts where `refresh-publication` must now appear
- lane ownership map
- forbidden surfaces per lane

A worker becomes stale if any of these change after launch:

- CLI flags or their meaning
- lifecycle next-command template strings
- shared publication planning function signatures or semantics
- rollback contract over publication-owned files
- stale-sweep invariants
- lane ownership map

Stale-worker rule:

1. stale output is not merged or hand-reconciled
2. parent updates `freeze.json`
3. parent deletes stale worker worktree and branch
4. parent relaunches from the new post-sweep freeze commit
5. restart from the first downstream affected task

## Merge Policy

- Parent is the only merger.
- Parent merges only into `codex/recommend-next-agent` in `REPO_ROOT`.
- Parent does not ask workers to rebase.
- Parent may perform local mechanical conflict resolution only if it does not alter the frozen seam.
- Any conflict that changes the frozen seam or stale-sweep invariants is a stop-and-relaunch event.

Merge order:

1. parent completes `task/epl-01-freeze-core`
2. parent completes `task/epl-02-parent-stale-sweep`
3. launch and merge Lane B first
4. rerun Lane B smoke commands locally
5. merge Lane C second
6. rerun docs/search smoke locally
7. merge Lane D third
8. parent runs convergence and final verification locally

## Task Graph

```text
task/epl-00-baseline
  ->
task/epl-01-freeze-core
  ->
task/epl-02-parent-stale-sweep
  ->
parallel:
  task/epl-03-maintenance-alignment
  task/epl-04-authored-docs
  task/epl-05-tests-fixtures
  ->
task/epl-06-converge
  ->
task/epl-07-final-verify
```

## Workstream Plan

### task/epl-00-baseline

Owner:

- parent

Required work:

- capture current branch, HEAD SHA, and dirty-state summary
- record that live execution baseline is `000aa69`
- note that `PLAN.md` still cites `bfd6fd4`
- snapshot lane ownership against actual dirty files
- block worker launch if any worker-owned surface is already dirty

Required commands:

```bash
git branch --show-current
git rev-parse --short HEAD
git status --short
```

Acceptance:

- `baseline.json` reflects actual launch state
- any overlap between existing dirt and lane ownership is resolved before freeze

### task/epl-01-freeze-core

Owner:

- parent

Owned surfaces:

- `crates/xtask/src/publication_refresh.rs`
- `crates/xtask/src/main.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/agent_lifecycle.rs`
- `crates/xtask/src/prepare_publication.rs`
- `crates/xtask/src/workspace_mutation.rs` only if required

Required work:

- add the new `refresh-publication` command
- wire CLI registration and public exports
- freeze lifecycle next-command semantics
- keep `prepare-publication` as handoff writer only
- implement the parent-owned shared publication planning seam that maintenance will reuse
- implement transactional write + gate + rollback behavior for create-mode publication refresh
- freeze command output expectations enough for docs/tests/search normalization to proceed

Required commands:

```bash
cargo run -p xtask -- refresh-publication --help
cargo test -p xtask --test prepare_publication_entrypoint
cargo test -p xtask --test agent_lifecycle_state
```

Acceptance:

- new command is wired and runnable
- frozen lifecycle strings are recorded in `freeze.json`
- shared publication planning seam is stable enough for maintenance reuse without further parent edits

### task/epl-02-parent-stale-sweep

Owner:

- parent

Owned surfaces:

- all parent-owned committed lifecycle artifacts
- any test/doc/fixture file requiring purely mechanical stale-string or hash normalization before worker launch

Required work:

- run a targeted sweep for command-string fallout across:
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
  - `docs/specs/cli-agent-onboarding-charter.md`
  - `docs/agents/lifecycle/**/*.json`
  - impacted `crates/xtask/tests/**` fixtures
- replace stale `publication_ready` next-step shell-chain references with the new `refresh-publication` canonical form where required
- preserve valid `prepare-publication --approval .* --write` references in runtime-integrated contexts only
- normalize any committed lifecycle artifact, packet, or test fixture whose expected hash or reconstructed publication-ready content changed because `PUBLICATION_READY_NEXT_COMMAND` changed
- ensure this normalization is complete before any worker launches

Required search commands:

```bash
rg -n 'support-matrix --check && capability-matrix --check && capability-matrix-audit && make preflight && close-proving-run --write' \
  docs/cli-agent-onboarding-factory-operator-guide.md \
  docs/specs/cli-agent-onboarding-charter.md \
  docs/agents/lifecycle \
  crates/xtask/tests
rg -n 'prepare-publication --approval .* --write' \
  docs/cli-agent-onboarding-factory-operator-guide.md \
  docs/specs/cli-agent-onboarding-charter.md \
  docs/agents/lifecycle \
  crates/xtask/tests
rg -n 'refresh-publication' \
  docs/cli-agent-onboarding-factory-operator-guide.md \
  docs/specs/cli-agent-onboarding-charter.md \
  docs/agents/lifecycle \
  crates/xtask/tests \
  crates/xtask/src
rg -n 'PUBLICATION_READY_NEXT_COMMAND|expected_next_command|publication-ready.json' \
  crates/xtask/tests \
  docs/agents/lifecycle \
  crates/xtask/src/agent_lifecycle.rs
```

Normalization rules:

- raw shell-chain matches are invalid in the authored docs, committed lifecycle artifacts, and impacted tests whenever they are serving as the canonical `publication_ready` next step
- raw shell-chain matches remain valid only as the green-gate command inventory in code or packet `required_commands`, not as the lifecycle’s single next command
- `prepare-publication --approval .* --write` remains valid in runtime-integrated contexts, runtime-follow-on expectations, repair-runtime-evidence expectations, and docs describing the pre-refresh step
- `prepare-publication --approval .* --write` is invalid as the canonical next command once lifecycle stage is `publication_ready`
- `refresh-publication` must appear in the new CLI, in publication-ready lifecycle expectations, in authored docs for the create lane, and in the affected regression tests

Required commands:

```bash
cargo test -p xtask --test agent_lifecycle_state
cargo test -p xtask --test runtime_follow_on_entrypoint
cargo test -p xtask --test repair_runtime_evidence_entrypoint
```

Acceptance:

- no stale canonical raw shell-chain references remain in the swept scopes
- all historical/fixture hash fallout tied to `PUBLICATION_READY_NEXT_COMMAND` is normalized
- `freeze.json` records `post_sweep_commit_sha`
- downstream workers can launch from the normalized parent state without inheriting mechanical fallout

### task/epl-03-maintenance-alignment

Owner:

- worker B

Owned surfaces:

- `crates/xtask/src/agent_maintenance/refresh.rs`
- `crates/xtask/tests/agent_maintenance_refresh.rs`

Required work:

- replace maintenance-local support/capability publication planning with the frozen shared seam
- keep `refresh-agent` behavior unchanged except for shared planner reuse
- add parity/regression coverage proving maintenance refresh and create-mode publication derive the same publication bytes for the same surfaces

Forbidden surfaces:

- all parent-owned files
- docs
- committed lifecycle docs
- `.runs/**`

Required commands:

```bash
cargo test -p xtask --test agent_maintenance_refresh
cargo run -p xtask -- refresh-agent --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --dry-run
```

Acceptance:

- `refresh-agent` no longer owns a duplicate support/capability publication plan
- maintenance refresh tests prove parity against the frozen shared seam

### task/epl-04-authored-docs

Owner:

- worker C

Owned surfaces:

- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/specs/cli-agent-onboarding-charter.md`

Required work:

- replace the manual publication command choreography with the one-command refresh story
- document that `prepare-publication` writes only the handoff packet
- document that `refresh-publication` owns publication output writes, green gate, and rollback
- keep charter/operator wording aligned to the frozen command strings exactly

Forbidden surfaces:

- code
- tests
- committed lifecycle docs under `docs/agents/lifecycle/**`
- generated publication outputs
- `.runs/**`

Required commands:

```bash
rg -n 'prepare-publication|refresh-publication|support-matrix --check && capability-matrix --check && capability-matrix-audit && make preflight && close-proving-run --write' \
  docs/cli-agent-onboarding-factory-operator-guide.md \
  docs/specs/cli-agent-onboarding-charter.md
```

Acceptance:

- authored docs describe one publication consumer command, not a manual checklist
- wording matches the frozen CLI and lifecycle templates exactly

### task/epl-05-tests-fixtures

Owner:

- worker D

Owned surfaces:

- `crates/xtask/tests/refresh_publication_entrypoint.rs`
- `crates/xtask/tests/prepare_publication_entrypoint.rs`
- `crates/xtask/tests/agent_lifecycle_state.rs`
- `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
- `crates/xtask/tests/repair_runtime_evidence_entrypoint.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_paths.rs`
- `crates/xtask/tests/support/agent_maintenance_drift_harness.rs`
- `crates/xtask/tests/support/onboard_agent_harness.rs` only if necessary

Required work:

- add new entrypoint coverage for `refresh-publication`
- cover happy path, stale check behavior, support-only, capability-only, support+capability, rollback-on-gate-failure, lifecycle-update rollback, and idempotent rerun
- update `prepare-publication --check` coverage for both allowed `publication_ready` next-command states
- update runtime-follow-on and repair-runtime-evidence coverage where they assert the prepare-publication handoff string
- keep `close-proving-run` coverage proving unchanged behavior against post-refresh `publication_ready`
- update harness expectations if they validate `required_commands`, `expected_next_command`, or committed lifecycle packet linkage
- assume the parent already normalized any mechanical next-command/hash fallout before this lane starts; this lane owns only behavior-focused regressions after that point

Forbidden surfaces:

- all code under `crates/xtask/src/**`
- docs
- committed lifecycle docs
- generated publication outputs
- `.runs/**`

Required commands:

```bash
cargo test -p xtask --test refresh_publication_entrypoint
cargo test -p xtask --test prepare_publication_entrypoint
cargo test -p xtask --test agent_lifecycle_state
cargo test -p xtask --test runtime_follow_on_entrypoint
cargo test -p xtask --test repair_runtime_evidence_entrypoint
cargo test -p xtask --test onboard_agent_closeout_preview
cargo test -p xtask --test agent_maintenance_drift
```

Acceptance:

- the new command boundary and rollback path are covered
- `prepare-publication` check-mode compatibility is covered
- runtime-follow-on and repair-runtime-evidence remain aligned with the frozen handoff contract
- `close-proving-run` remains valid against the post-refresh baseline

### task/epl-06-converge

Owner:

- parent

Required work:

- merge Lane B, rerun its smoke commands locally
- merge Lane C, rerun docs/search smoke locally
- merge Lane D, rerun the widened regression stack locally
- rerun the stale-string sweep and prove no newly merged stale references reappeared
- run the full targeted verification stack before the live aider flow

Required commands:

```bash
cargo test -p xtask --test agent_maintenance_refresh
cargo test -p xtask --test refresh_publication_entrypoint
cargo test -p xtask --test prepare_publication_entrypoint
cargo test -p xtask --test agent_lifecycle_state
cargo test -p xtask --test runtime_follow_on_entrypoint
cargo test -p xtask --test repair_runtime_evidence_entrypoint
cargo test -p xtask --test onboard_agent_closeout_preview
cargo test -p xtask --test agent_maintenance_drift
make check
rg -n 'support-matrix --check && capability-matrix --check && capability-matrix-audit && make preflight && close-proving-run --write' \
  docs/cli-agent-onboarding-factory-operator-guide.md \
  docs/specs/cli-agent-onboarding-charter.md \
  docs/agents/lifecycle \
  crates/xtask/tests
rg -n 'prepare-publication --approval .* --write' \
  docs/cli-agent-onboarding-factory-operator-guide.md \
  docs/specs/cli-agent-onboarding-charter.md \
  docs/agents/lifecycle \
  crates/xtask/tests
rg -n 'refresh-publication' \
  docs/cli-agent-onboarding-factory-operator-guide.md \
  docs/specs/cli-agent-onboarding-charter.md \
  docs/agents/lifecycle \
  crates/xtask/tests \
  crates/xtask/src
```

Acceptance:

- merged branch is internally green before live repo-state verification
- no worker-owned duplicate logic survives in maintenance refresh
- no stale canonical raw shell-chain references reappear in the swept scopes

### task/epl-07-final-verify

Owner:

- parent

Owned surfaces:

- merged integration branch
- live aider lifecycle docs
- final generated publication outputs
- `acceptance.md`

Required work:

- run the real create-mode publication flow against aider
- allow `refresh-publication --check` to act as the pre-write stale detector, then prove `--write` makes the lane green
- keep `close-proving-run` verification at the test level only; do not widen into live closeout authoring for aider
- run the final targeted search sweeps and classify remaining matches as valid or invalid

Required final ordered sequence:

1. Verify pre-refresh search state:
   ```bash
   rg -n 'support-matrix --check && capability-matrix --check && capability-matrix-audit && make preflight && close-proving-run --write' \
     docs/cli-agent-onboarding-factory-operator-guide.md \
     docs/specs/cli-agent-onboarding-charter.md \
     docs/agents/lifecycle \
     crates/xtask/tests
   rg -n 'prepare-publication --approval .* --write' \
     docs/cli-agent-onboarding-factory-operator-guide.md \
     docs/specs/cli-agent-onboarding-charter.md \
     docs/agents/lifecycle \
     crates/xtask/tests
   rg -n 'refresh-publication' \
     docs/cli-agent-onboarding-factory-operator-guide.md \
     docs/specs/cli-agent-onboarding-charter.md \
     docs/agents/lifecycle \
     crates/xtask/tests \
     crates/xtask/src
   ```
2. Write the committed handoff:
   ```bash
   cargo run -p xtask -- prepare-publication --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --write
   ```
3. Run the stale/green detector:
   ```bash
   cargo run -p xtask -- refresh-publication --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --check
   ```
4. If step 3 reports stale publication outputs, repair them:
   ```bash
   cargo run -p xtask -- refresh-publication --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --write
   ```
5. Prove the post-write state is green:
   ```bash
   cargo run -p xtask -- refresh-publication --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --check
   ```
6. Verify semantic publication truth:
   ```bash
   cargo run -p xtask -- capability-matrix-audit
   ```
7. Run the full repo gate:
   ```bash
   make preflight
   ```
8. Re-run the targeted search sweeps and classify matches:
   ```bash
   rg -n 'support-matrix --check && capability-matrix --check && capability-matrix-audit && make preflight && close-proving-run --write' \
     docs/cli-agent-onboarding-factory-operator-guide.md \
     docs/specs/cli-agent-onboarding-charter.md \
     docs/agents/lifecycle \
     crates/xtask/tests
   rg -n 'prepare-publication --approval .* --write' \
     docs/cli-agent-onboarding-factory-operator-guide.md \
     docs/specs/cli-agent-onboarding-charter.md \
     docs/agents/lifecycle \
     crates/xtask/tests
   rg -n 'refresh-publication' \
     docs/cli-agent-onboarding-factory-operator-guide.md \
     docs/specs/cli-agent-onboarding-charter.md \
     docs/agents/lifecycle \
     crates/xtask/tests \
     crates/xtask/src
   ```

Final search classification rules:

- raw shell-chain matches:
  - invalid anywhere in the authored docs, committed lifecycle artifacts, or impacted tests when used as the lifecycle’s canonical next command
  - valid only in code or packet contexts that are intentionally describing the green-gate command inventory itself
- `prepare-publication --approval .* --write` matches:
  - valid in runtime-integrated contexts, runtime-follow-on expectations, repair-runtime-evidence expectations, and docs describing the pre-refresh step
  - invalid as the canonical next step after `publication_ready`
- `refresh-publication` matches:
  - must exist in `crates/xtask/src/**`, in publication-ready workflow docs, and in the updated regression tests
  - absence from those canonical contexts is a blocker

Failure handling:

- if step 3 or step 5 fails because surfaces are stale or miscomputed, fix within touched scope and rerun from step 3
- if step 4 fails, the new command must restore publication-owned committed surfaces; rollback failure is a hard blocker
- if `make preflight` fails for a touched-scope regression, fix within scope and rerun from the first failed step
- if `make preflight` fails for an unrelated pre-existing blocker, record it in `acceptance.md` and stop without widening

Acceptance:

- aider transitions from `runtime_integrated` to `publication_ready` through the new command path
- `refresh-publication --write` leaves committed publication outputs green and checkable
- final search sweeps show no stale canonical shell-chain references in the swept scopes
- final publication surfaces and lifecycle docs are review-ready on `codex/recommend-next-agent`

## Tests And Acceptance

The run is accepted only if all of the following are true:

1. `prepare-publication --write` points to `refresh-publication --approval ... --write`.
2. `prepare-publication --check` accepts both valid `publication_ready` next-command states:
  - pre-refresh `refresh-publication --approval ... --write`
  - post-refresh `close-proving-run --approval ... --closeout ...`
3. `refresh-publication --write` consumes `publication-ready.json` and writes only the packet’s required publication outputs.
4. `refresh-publication --write` rolls back publication-owned committed surfaces on gate failure.
5. maintenance refresh and create-mode publication use the same support/capability planning seam.
6. runtime-follow-on and repair-runtime-evidence remain aligned with the frozen pre-publication handoff contract.
7. `close-proving-run` works unchanged against a post-refresh `publication_ready` baseline.
8. authored docs describe one publication consumer command, not a manual checklist.
9. all historical or fixture hash fallout tied to `PUBLICATION_READY_NEXT_COMMAND` is normalized before the test worker launches.
10. final ordered verification through aider plus `make preflight` passes.
11. targeted `rg` sweeps prove no stale canonical raw shell-chain references remain in the authored docs, committed lifecycle artifacts, or impacted `crates/xtask/tests/**` scopes.

## Assumptions

- current live execution baseline remains `000aa69` until the run starts
- worker launch happens only after the parent has frozen CLI strings and completed the stale-string/hash normalization sweep
- `REQUIRED_PUBLICATION_COMMANDS` remains the four-command green gate and does not become `refresh-publication`
- historical closed-baseline packet hashes may need committed doc refresh because `reconstruct_publication_ready_state_from_closed_baseline()` uses `PUBLICATION_READY_NEXT_COMMAND`
- test and fixture fallout is broader than the first narrow lane draft, so the widened test lane plus parent stale sweep are both required
- live final verification is allowed to leave aider at `publication_ready` on the branch
- no unrelated repo dirt overlaps with worker-owned files at launch; if it does, the parent rebalance happens before worker creation
