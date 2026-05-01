# Runtime Evidence Repair Orchestration Plan

## Summary

- Execute on the live implementation branch `codex/recommend-next-agent`. Treat `main` only as the review base branch, never as the worker fork point.
- Plan authority is `PLAN.md` at the repo root. If `PLAN.md`, current code, and older orchestration notes disagree, `PLAN.md` wins unless it conflicts with `docs/specs/**`; a spec conflict is a stop-and-escalate condition.
- This run replaces the older broad lifecycle-maturity orchestration on this branch. The scope here is only the narrow runtime evidence repair milestone.
- Parent agent is the only integrator, merger, rebasing authority, repair operator, and final verifier.
- Current worktree dirt is expected. `PLAN.md` is already modified and must be recorded, not reverted.
- Worker model for delegated lanes:
  - model: GPT-5.4
  - reasoning: high
- Keep the critical path local to the parent for:
  - kickoff and dirty-state baseline capture
  - shared runtime evidence helper extraction
  - `repair-runtime-evidence` command wiring
  - `historical_lifecycle_backfill.rs` migration onto the shared helper
  - deterministic `aider` repair artifact generation
  - operator-guide updates
  - final integration and final verification
- Launch parallel workers only after the helper and command contract freeze commit exists.
- Use only 2 concurrent workers.
  - Worker 1: forward validation hardening
  - Worker 2: drift surfacing
- Do not launch a docs worker.
  - `docs/cli-agent-onboarding-factory-operator-guide.md` depends on final command semantics and should stay parent-owned.
- Do not launch an evidence worker.
  - `docs/agents/.uaa-temp/**` are generated evidence and committed run artifacts, not authored source.
- Integration surface is explicit and intentionally local:
  - the parent integrates directly onto `codex/recommend-next-agent` in the primary repo worktree at `REPO_ROOT`
  - there is no separate integration branch for this run
- No human approval gates are planned for this run.
- Canonical orchestration state owned only by the parent:
  - `REPO_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api`
  - `WORKTREE_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/unified-agent-api-runtime-evidence-repair`
  - `ORCH_RUN_ROOT=$REPO_ROOT/.runs/runtime-evidence-repair`
  - `RUNTIME_EVIDENCE_ROOT=$REPO_ROOT/docs/agents/.uaa-temp/runtime-follow-on/runs`
- Concrete live proof target:
  - `docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml`
  - `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/`

## Approval Gates

- This plan has zero human approval gates.
- The only intentional pauses are:
  - hard stop conditions
  - worker bounce-backs
  - helper-freeze reopen handling
  - final verification failures that require narrowing back to the touched scope

## Worker Model

- Parent agent owns all integration, orchestration state, freeze decisions, merge sequencing, repair execution, and acceptance decisions.
- Workers may edit only their assigned files and must return:
  - changed files
  - commands run
  - exit codes
  - blockers or assumptions
- Workers do not merge branches.
- Workers do not edit `$ORCH_RUN_ROOT/**`.
- Workers do not edit `PLAN.md`.
- Workers do not author or hand-edit `.uaa-temp` runtime evidence.
- Any worker that needs a parent-owned file must stop and bounce the request back.

## Hard Guards

- Scope is locked to the current `PLAN.md` milestone:
  - add repo-owned `repair-runtime-evidence --check/--write`
  - extract shared runtime evidence bundle helpers from `historical_lifecycle_backfill.rs`
  - repair stale `aider` runtime evidence
  - remove legacy short-form publication command tolerance from `runtime_follow_on/lifecycle.rs`
  - add explicit stale-runtime-evidence drift detection
  - add tests
  - update operator docs
- Do not widen scope into lifecycle redesign, publication packet redesign, or maintenance framework expansion.
- Do not add a new crate.
- Do not add a new service.
- Keep `prepare-publication` strict.
- Do not add legacy tolerance to `prepare-publication`.
- Do not add a second hardcoded publication command set.
- The required publication commands remain exactly:
  - `cargo run -p xtask -- support-matrix --check`
  - `cargo run -p xtask -- capability-matrix --check`
  - `cargo run -p xtask -- capability-matrix-audit`
  - `make preflight`
- `repair-runtime-evidence` must not advance lifecycle stage.
- The deterministic repair run directory is fixed:
  - `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-<agent_id>-runtime-follow-on/`
  - concrete target for this run: `repair-aider-runtime-follow-on`
- Leave `docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/` untouched in this milestone.
- Treat `docs/agents/.uaa-temp/**` as generated evidence and committed run artifacts, not authored source.
- Keep the helper extraction boring and explicit. No agent-specific `aider` branching is allowed in core logic.
- Do not overbuild a generic batch repair system.
- Stop conditions for the full run:
  - `PLAN.md` conflicts with `docs/specs/**`
  - helper extraction requires schema or lifecycle-stage redesign
  - `repair-runtime-evidence` cannot reuse a single shared helper path with `historical_lifecycle_backfill.rs`
  - a lane requires hand-authoring `.uaa-temp` JSON rather than producing it through command behavior
  - a lane requires relaxing `prepare-publication` strictness to pass

## Orchestration State

Canonical parent-owned files under `$ORCH_RUN_ROOT`:

- `baseline.json`
- `tasks.json`
- `session-log.md`
- `helper-freeze.json`
- `merge-log.md`
- `acceptance.md`

Per-task sentinels under `$REPO_ROOT/.runs/<task-id>/`:

- `started.json`
- `status.json`
- `done.json` or `blocked.json`

`baseline.json` must record:

- current branch
- current sha
- review base branch
- dirty-file summary
- explicit note that `PLAN.md` is modified before orchestration starts
- the currently failing `prepare-publication --check` condition for `aider`
- the currently empty `written-paths.json` fact for the old `aider` rerun bundle
- the current `check-agent-drift --agent aider` behavior before repair

`helper-freeze.json` must record:

- `freeze_commit`
- `repair_command_surface`
- `required_publication_commands`
- `repair_run_dir_pattern`
- `parent_owned_generated_surfaces`
- `worker_lane_file_ownership`
- `historical_backfill_shared_helper_rule`

Workers consume `helper-freeze.json` and do not redefine any of its contract locally.

## Worktree Plan

Integration surface:

- branch: `codex/recommend-next-agent`
- worktree: `REPO_ROOT`
- owner: parent only

Worker worktrees created only after helper freeze:

- validation worker:
  - branch: `codex/recommend-next-agent-validation-hardening`
  - worktree: `$WORKTREE_ROOT/validation-hardening`
- drift worker:
  - branch: `codex/recommend-next-agent-drift-surfacing`
  - worktree: `$WORKTREE_ROOT/drift-surfacing`

Creation commands from `REPO_ROOT` after `helper-freeze.json` exists:

```bash
mkdir -p "$WORKTREE_ROOT" "$ORCH_RUN_ROOT"
FREEZE_SHA=$(jq -r '.freeze_commit' "$ORCH_RUN_ROOT/helper-freeze.json")

git worktree add -b codex/recommend-next-agent-validation-hardening "$WORKTREE_ROOT/validation-hardening" "$FREEZE_SHA"
git worktree add -b codex/recommend-next-agent-drift-surfacing "$WORKTREE_ROOT/drift-surfacing" "$FREEZE_SHA"
```

Worktree rules:

- Never fork workers from `main`.
- Never fork workers from anything other than `freeze_commit`.
- Never reuse a dirty worker worktree.
- Parent integrates only in `REPO_ROOT` on `codex/recommend-next-agent`.
- There is no dedicated integration branch for this run.

## Restart And Reopen Rule

- If the parent changes any helper-freeze semantic surface after either worker launches, both workers are stale.
- Helper-freeze semantic surfaces are:
  - `crates/xtask/src/runtime_evidence_bundle.rs`
  - `crates/xtask/src/repair_runtime_evidence.rs`
  - `crates/xtask/src/historical_lifecycle_backfill.rs`
  - `crates/xtask/src/main.rs`
  - `crates/xtask/src/lib.rs`
  - the exact repair run directory convention
  - the exact publication command-string contract
- If the parent changes only validation-owned files after the validation worker launches, only that worker is stale.
- If the parent changes only drift-owned files after the drift worker launches, only that worker is stale.
- Stale worker handling is mandatory:
  1. Mark the worker sentinel as stale or blocked.
  2. Discard the worker branch and worktree.
  3. Regenerate `helper-freeze.json` if the freeze changed.
  4. Recreate the worker worktree from the new `freeze_commit`.
  5. Relaunch the worker with the new freeze context.
- Never merge stale worker output.
- Never cherry-pick stale worker commits into `codex/recommend-next-agent`.

## Merge Policy

- Parent merges worker branches only into the explicit integration surface:
  - `REPO_ROOT` on branch `codex/recommend-next-agent`
- Parent merges from branch heads only.
- Parent does not ask workers to rebase themselves.
- Parent merges the validation worker first.
- Parent reruns the validation gate after that merge.
- Parent merges the drift worker second.
- Parent reruns the combined gate after both merges.
- If a worker changed a forbidden file, reject the lane and relaunch or finish it locally.
- Mechanical conflict resolution is allowed locally.
- Semantic conflict resolution that changes the frozen repair contract is a stop-and-reopen event.

## Task Graph

Critical path:

1. `task/runtime-evidence-00-baseline`
2. `task/runtime-evidence-01-helper-freeze`
3. launch in parallel:
   - `task/runtime-evidence-02a-validation-hardening`
   - `task/runtime-evidence-02b-drift-surfacing`
4. `task/runtime-evidence-02c-converge`
5. `task/runtime-evidence-03-repair-aider-docs`
6. `task/runtime-evidence-04-final-verify`

Parallel-safe tasks:

- `task/runtime-evidence-02a-validation-hardening`
- `task/runtime-evidence-02b-drift-surfacing`

Deliberately serialized tasks:

- `task/runtime-evidence-00-baseline`
- `task/runtime-evidence-01-helper-freeze`
- `task/runtime-evidence-02c-converge`
- `task/runtime-evidence-03-repair-aider-docs`
- `task/runtime-evidence-04-final-verify`

## Workstream Plan

### WS-BASELINE

#### `task/runtime-evidence-00-baseline` — parent only

Existing required files that must already exist:

- `PLAN.md`
- `crates/xtask/src/main.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/historical_lifecycle_backfill.rs`
- `crates/xtask/src/runtime_follow_on/lifecycle.rs`
- `crates/xtask/src/prepare_publication/runtime_evidence.rs`
- `crates/xtask/src/agent_maintenance/drift/mod.rs`
- `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
- `crates/xtask/tests/prepare_publication_entrypoint.rs`
- `crates/xtask/tests/agent_maintenance_drift.rs`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/input-contract.json`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/handoff.json`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/written-paths.json`

Expected files that do not need to exist yet:

- `crates/xtask/src/repair_runtime_evidence.rs`
- `crates/xtask/src/runtime_evidence_bundle.rs`
- `crates/xtask/src/agent_maintenance/drift/runtime_evidence.rs`
- `crates/xtask/tests/repair_runtime_evidence_entrypoint.rs`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/input-contract.json`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/run-status.json`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/validation-report.json`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/handoff.json`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/written-paths.json`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/run-summary.md`

Owned files:

- `$ORCH_RUN_ROOT/baseline.json`
- `$ORCH_RUN_ROOT/tasks.json`
- `$ORCH_RUN_ROOT/session-log.md`
- `$REPO_ROOT/.runs/task-runtime-evidence-00-baseline/*`

Forbidden files:

- all product source files

Required commands:

```bash
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git status --short
test -f PLAN.md
test -f crates/xtask/src/main.rs
test -f crates/xtask/src/lib.rs
test -f crates/xtask/src/historical_lifecycle_backfill.rs
test -f crates/xtask/src/runtime_follow_on/lifecycle.rs
test -f crates/xtask/src/prepare_publication/runtime_evidence.rs
test -f crates/xtask/src/agent_maintenance/drift/mod.rs
test -f crates/xtask/tests/runtime_follow_on_entrypoint.rs
test -f crates/xtask/tests/prepare_publication_entrypoint.rs
test -f crates/xtask/tests/agent_maintenance_drift.rs
test -f docs/cli-agent-onboarding-factory-operator-guide.md
test -f docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml
test -f docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/input-contract.json
test -f docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/handoff.json
test -f docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/written-paths.json
```

Acceptance:

- current branch is `codex/recommend-next-agent`
- `PLAN.md` is present and is the run authority
- the dirty worktree summary is recorded, including the modified `PLAN.md`
- baseline captures real current failures and does not assume a clean tree
- unrelated dirt is recorded, not reverted

### WS-CORE

#### `task/runtime-evidence-01-helper-freeze` — parent only

Owned files:

- `crates/xtask/src/repair_runtime_evidence.rs`
- `crates/xtask/src/runtime_evidence_bundle.rs`
- `crates/xtask/src/historical_lifecycle_backfill.rs`
- `crates/xtask/src/main.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/tests/repair_runtime_evidence_entrypoint.rs`
- `$ORCH_RUN_ROOT/helper-freeze.json`
- `$ORCH_RUN_ROOT/session-log.md`
- `$REPO_ROOT/.runs/task-runtime-evidence-01-helper-freeze/*`

Forbidden files:

- `crates/xtask/src/runtime_follow_on/lifecycle.rs`
- `crates/xtask/src/prepare_publication/runtime_evidence.rs`
- `crates/xtask/src/agent_maintenance/drift/**`
- `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
- `crates/xtask/tests/prepare_publication_entrypoint.rs`
- `crates/xtask/tests/agent_maintenance_drift.rs`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/agents/.uaa-temp/**`

Required work:

1. Add the shared runtime evidence bundle helper module.
2. Move bundle reconstruction and writer helpers out of `historical_lifecycle_backfill.rs`.
3. Add `repair-runtime-evidence` command wiring in `main.rs` and `lib.rs`.
4. Implement `--check` and `--write` argument flow for the new command.
5. Keep repair lifecycle-stage neutral.
6. Keep the deterministic repair directory naming fixed.
7. Add focused entrypoint coverage for command surface and core repair behavior.
8. Write `helper-freeze.json` from the exact head commit that contains the finished parent-owned core.

Required commands:

```bash
cargo test -p xtask --test repair_runtime_evidence_entrypoint
```

Acceptance:

- shared helper exists and is the only reconstruction path for repair and historical backfill
- `repair-runtime-evidence` is wired into `xtask`
- helper-freeze records the frozen repair contract and launch commit
- worker lanes do not launch until this task is complete on `codex/recommend-next-agent`

### WS-VALIDATION

#### `task/runtime-evidence-02a-validation-hardening` — worker 1

Owned files:

- `crates/xtask/src/runtime_follow_on/lifecycle.rs`
- `crates/xtask/src/prepare_publication/runtime_evidence.rs`
- `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
- `crates/xtask/tests/prepare_publication_entrypoint.rs`

Forbidden files:

- `crates/xtask/src/repair_runtime_evidence.rs`
- `crates/xtask/src/runtime_evidence_bundle.rs`
- `crates/xtask/src/historical_lifecycle_backfill.rs`
- `crates/xtask/src/main.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/agent_maintenance/drift/**`
- `crates/xtask/tests/repair_runtime_evidence_entrypoint.rs`
- `crates/xtask/tests/agent_maintenance_drift.rs`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/agents/.uaa-temp/**`
- `$ORCH_RUN_ROOT/**`

Required work:

1. Remove legacy short-form publication command tolerance from `runtime_follow_on/lifecycle.rs`.
2. Require exact equality with `agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS`.
3. Keep `prepare-publication` strict.
4. Improve stale-bundle consumer errors so operators are pointed to `repair-runtime-evidence`.
5. Add regression coverage for legacy short-form rejection.
6. Add regression coverage for strict repaired-bundle consumption semantics.

Required commands:

```bash
cargo test -p xtask --test runtime_follow_on_entrypoint
cargo test -p xtask --test prepare_publication_entrypoint
```

Acceptance:

- forward runtime validation no longer accepts short-form command sets
- publication consumption remains strict and does not gain legacy aliases
- validation tests cover both rejection and truthful-bundle acceptance
- no drift, repair-core, docs, or `.uaa-temp` files are changed

Bounce-back rules:

- if the worker needs a new repair helper interface, stop and return the requested `helper-freeze.json` delta
- if the worker needs to touch `main.rs` or `lib.rs`, stop and bounce back

### WS-DRIFT

#### `task/runtime-evidence-02b-drift-surfacing` — worker 2

Owned files:

- `crates/xtask/src/agent_maintenance/drift/mod.rs`
- `crates/xtask/src/agent_maintenance/drift/runtime_evidence.rs`
- `crates/xtask/src/agent_maintenance/drift/shared.rs` when mechanically required
- `crates/xtask/tests/agent_maintenance_drift.rs`

Forbidden files:

- `crates/xtask/src/repair_runtime_evidence.rs`
- `crates/xtask/src/runtime_evidence_bundle.rs`
- `crates/xtask/src/historical_lifecycle_backfill.rs`
- `crates/xtask/src/main.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/runtime_follow_on/**`
- `crates/xtask/src/prepare_publication/**`
- `crates/xtask/tests/repair_runtime_evidence_entrypoint.rs`
- `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
- `crates/xtask/tests/prepare_publication_entrypoint.rs`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/agents/.uaa-temp/**`
- `$ORCH_RUN_ROOT/**`

Required work:

1. Add explicit stale-runtime-evidence drift detection for `runtime_integrated` agents.
2. Emit a targeted repair instruction that references `repair-runtime-evidence`.
3. Keep existing governance drift logic intact for other lifecycle states.
4. Add regression coverage proving the explicit runtime-evidence finding appears for stale bundles.
5. Add regression coverage proving that finding disappears once the runtime evidence is truthful.

Required commands:

```bash
cargo test -p xtask --test agent_maintenance_drift
```

Acceptance:

- drift output includes a first-class runtime-evidence finding instead of only vague governance drift
- the finding is scoped to the runtime evidence seam and includes the repair command
- no repair-core, validation, docs, or `.uaa-temp` files are changed

Bounce-back rules:

- if the worker needs a change to the frozen repair command contract, stop and return the exact freeze delta
- if the worker needs to touch `prepare_publication/runtime_evidence.rs`, stop and bounce back

### WS-CONVERGE

#### `task/runtime-evidence-02c-converge` — parent only

Owned files:

- `$ORCH_RUN_ROOT/merge-log.md`
- `$ORCH_RUN_ROOT/session-log.md`
- `$REPO_ROOT/.runs/task-runtime-evidence-02c-converge/*`

Required integration sequence:

1. Merge `codex/recommend-next-agent-validation-hardening` into `codex/recommend-next-agent`.
2. Run the validation gate.
3. Merge `codex/recommend-next-agent-drift-surfacing` into `codex/recommend-next-agent`.
4. Run the combined gate.
5. Do not reopen helper-freeze semantics here.

Required commands:

```bash
cargo test -p xtask --test repair_runtime_evidence_entrypoint
cargo test -p xtask --test runtime_follow_on_entrypoint
cargo test -p xtask --test prepare_publication_entrypoint
cargo test -p xtask --test agent_maintenance_drift
```

Acceptance:

- both delegated lanes are merged on the live integration surface
- frozen repair semantics remain intact
- no worker-authored `.uaa-temp` evidence exists

### WS-REPAIR

#### `task/runtime-evidence-03-repair-aider-docs` — parent only

Owned files:

- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/input-contract.json`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/run-status.json`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/validation-report.json`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/handoff.json`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/written-paths.json`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/run-summary.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `$ORCH_RUN_ROOT/session-log.md`
- `$REPO_ROOT/.runs/task-runtime-evidence-03-repair-aider-docs/*`

Forbidden files:

- `PLAN.md`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/**`

Required work:

1. Run the explicit drift check for `aider` before repair and record the result.
2. Run `repair-runtime-evidence --check` for `aider`.
3. Run `repair-runtime-evidence --write` for `aider`.
4. Commit only the deterministic repair bundle under `repair-aider-runtime-follow-on`.
5. Re-run `prepare-publication --check` for `aider`.
6. Re-run `check-agent-drift --agent aider` and verify the specific runtime-evidence finding is gone.
7. Update the operator guide with:
   - when to run `repair-runtime-evidence`
   - expected stale-bundle failure shape
   - the `aider` repair sequence
   - the fact that repair does not advance lifecycle stage

Required commands:

```bash
cargo run -p xtask -- check-agent-drift --agent aider
cargo run -p xtask -- repair-runtime-evidence --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --check
cargo run -p xtask -- repair-runtime-evidence --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --write
cargo run -p xtask -- prepare-publication --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --check
cargo run -p xtask -- check-agent-drift --agent aider
```

Acceptance:

- repaired `aider` bundle is generated through command behavior, not by hand editing JSON
- repaired `written-paths.json` is non-empty
- repair leaves lifecycle stage unchanged
- `prepare-publication --check` passes for `aider` after repair
- the explicit runtime-evidence drift finding exists before repair and is absent after repair
- the stale rerun directory remains untouched

### WS-VERIFY

#### `task/runtime-evidence-04-final-verify` — parent only

Owned files:

- the full milestone touch set
- `$ORCH_RUN_ROOT/acceptance.md`
- `$ORCH_RUN_ROOT/merge-log.md`
- `$ORCH_RUN_ROOT/session-log.md`
- `$REPO_ROOT/.runs/task-runtime-evidence-04-final-verify/*`

Final verification sequence:

1. Replay the repair command gate:
   ```bash
   cargo test -p xtask --test repair_runtime_evidence_entrypoint
   ```
2. Replay the forward validation gate:
   ```bash
   cargo test -p xtask --test runtime_follow_on_entrypoint
   cargo test -p xtask --test prepare_publication_entrypoint
   ```
3. Replay the drift gate:
   ```bash
   cargo test -p xtask --test agent_maintenance_drift
   ```
4. Replay the live `aider` proof sequence:
   ```bash
   cargo run -p xtask -- repair-runtime-evidence --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --check
   cargo run -p xtask -- repair-runtime-evidence --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --write
   cargo run -p xtask -- prepare-publication --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --check
   cargo run -p xtask -- check-agent-drift --agent aider
   ```
5. Run the workspace typecheck gate last:
   ```bash
   make check
   ```

Verification rules:

- Run the verification sequence in the listed order.
- Treat `make check` as the final whole-workspace gate only after the narrower xtask tests and live `aider` proof commands are green.
- If a failure is caused by this milestone’s files, patch inside the milestone touch set and rerun from the first failed step.
- If a failure is caused by unrelated pre-existing repo state outside the touch set, record it in `acceptance.md` and stop instead of widening scope.
- Do not author `.uaa-temp` evidence during final verification except through the repair command itself.
- Do not mutate lifecycle stage as part of verification.

Acceptance:

- the live branch `codex/recommend-next-agent` contains the full milestone
- the explicit integration surface remained `REPO_ROOT` on `codex/recommend-next-agent`
- the shared helper owns reconstruction logic used by repair and historical backfill
- `repair-runtime-evidence` exists and is repo-owned
- forward runtime validation rejects legacy short-form publication commands
- `prepare-publication` remains strict
- drift output gives an explicit repair path for stale runtime evidence
- repaired `aider` runtime evidence is committed under the deterministic repair directory
- `$ORCH_RUN_ROOT` contains:
  - `baseline.json`
  - `tasks.json`
  - `session-log.md`
  - `helper-freeze.json`
  - `merge-log.md`
  - `acceptance.md`

## Context-Control Rules

- Parent keeps only these materials active in working context:
  - `PLAN.md`
  - this `ORCH_PLAN.md`
  - `$ORCH_RUN_ROOT/tasks.json`
  - `$ORCH_RUN_ROOT/helper-freeze.json` after WS-CORE
  - the latest live-branch diff summary
- Worker prompts contain only:
  - owned files
  - forbidden files
  - the exact relevant `PLAN.md` excerpt
  - the frozen repair contract from `helper-freeze.json`
  - required commands
  - bounce-back rules
- Workers return narrow summaries only.
- Parent records decisions in `$ORCH_RUN_ROOT/session-log.md`.
- Close worker lanes immediately after merge or rejection.
- Do not ingest full worker transcripts back into parent context unless a specific failure excerpt is required.

## Tests And Acceptance

Parent gates by phase:

- WS-CORE:
  - `cargo test -p xtask --test repair_runtime_evidence_entrypoint`
- WS-VALIDATION:
  - `cargo test -p xtask --test runtime_follow_on_entrypoint`
  - `cargo test -p xtask --test prepare_publication_entrypoint`
- WS-DRIFT:
  - `cargo test -p xtask --test agent_maintenance_drift`
- WS-CONVERGE:
  - `cargo test -p xtask --test repair_runtime_evidence_entrypoint`
  - `cargo test -p xtask --test runtime_follow_on_entrypoint`
  - `cargo test -p xtask --test prepare_publication_entrypoint`
  - `cargo test -p xtask --test agent_maintenance_drift`
- WS-REPAIR:
  - `cargo run -p xtask -- check-agent-drift --agent aider`
  - `cargo run -p xtask -- repair-runtime-evidence --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --check`
  - `cargo run -p xtask -- repair-runtime-evidence --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --write`
  - `cargo run -p xtask -- prepare-publication --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --check`
  - `cargo run -p xtask -- check-agent-drift --agent aider`
- Final:
  - `make check`

Final acceptance checklist:

- `repair-runtime-evidence --check` clearly fails when repair is impossible
- `repair-runtime-evidence --write` emits the full six-file bundle with non-empty written paths
- repair never advances lifecycle stage
- `historical_lifecycle_backfill.rs` uses the shared helper instead of private reconstruction logic
- forward runtime validation no longer accepts legacy short-form publication commands
- `prepare-publication --check` passes for `aider` after repair
- `check-agent-drift --agent aider` surfaces an explicit runtime-evidence repair finding before repair and that specific finding disappears after repair
- `make check` runs last as the final whole-workspace gate

## Assumptions

- `codex/recommend-next-agent` is the correct live implementation branch for this run.
- `main` remains only the review base branch.
- The modified `PLAN.md` is intentional authority text and must not be reverted during orchestration.
- Existing xtask integration harnesses are sufficient; this milestone does not require a new test harness family beyond `repair_runtime_evidence_entrypoint.rs`.
- Helper extraction can freeze a small shared contract before workers launch.
- The `aider` repair can be derived from committed repo state without agent-specific special casing.
- Keeping docs and `.uaa-temp` repair artifacts parent-owned is faster and safer than splitting them into another lane.
- The parent can complete final integration and verification without widening scope beyond the PLAN milestone.
