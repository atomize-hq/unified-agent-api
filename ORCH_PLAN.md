# Make The Published State Honest In The Lifecycle Model Orchestration Plan

## Summary

- This orchestration plan executes only the current `PLAN.md` milestone: `Make The Published State Honest In The Lifecycle Model`.
- Integration branch: `codex/recommend-next-agent`.
- Execution baseline: HEAD `07a0ce9259b8` on `2026-05-03`.
- The parent agent keeps the semantic authority lane local:
  - `refresh-publication --write` becomes the only committed writer of `LifecycleStage::Published`
  - the normal path becomes `publication_ready -> published -> closed_baseline`
  - `close-proving-run` consumes `published` on the normal path
  - the legacy `publication_ready` compatibility branch stays narrow and transitional
- After the parent freezes the semantics, three worker lanes may run in parallel:
  - tests
  - narrative docs
  - seeded lifecycle fixtures
- Final integration, merge decisions, and full verification return to the parent.

## Hard Guards

- Scope is locked to the current milestone only.
- Do not preserve or reintroduce any publication-lane orchestration from the prior milestone.
- Do not add or redesign commands in this slice.
- Do not invent a new lifecycle stage, packet type, or post-refresh artifact model.
- `refresh-publication --write` is the only allowed writer for `LifecycleStage::Published`.
- `prepare-publication --write` remains the writer for `publication_ready` only.
- `publication_ready` must mean only: the handoff packet exists and refresh is next.
- `close-proving-run` must treat `published` as the normal closeout input.
- Compatibility for `publication_ready` exists only for legacy post-refresh states that were never lifecycle-promoted.
- Docs, fixtures, and tests do not start until the parent freeze is recorded.
- The parent is the only merge authority and the only lane allowed to change semantic code after freeze.
- No worker may touch files outside its assigned ownership set.
- No lane may widen into closeout redesign, maintenance workflow redesign, or support/capability publication redesign beyond what `PLAN.md` already requires for lifecycle honesty.

## Orchestration State

Parent-owned orchestration state lives under:

- `RUN_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.runs/published-state-honesty`

State files:

- `baseline.json`
  - branch
  - head sha
  - dirty-state summary
  - launch timestamp
- `freeze.json`
  - freeze sha
  - frozen stage sequence
  - frozen refresh writer contract
  - frozen closeout compatibility rule
  - worker ownership map
  - stale-lane triggers
- `tasks.json`
  - task id
  - owner
  - status
  - dependencies
  - restart count
- `merge-log.md`
  - merge order
  - lane validation result
  - post-merge smoke result
- `acceptance.md`
  - final command matrix
  - exit codes
  - acceptance checklist

Task sentinels:

- `.runs/task-honest-00-baseline/`
- `.runs/task-honest-10-parent-semantics/`
- `.runs/task-honest-20-freeze/`
- `.runs/task-honest-30-tests/`
- `.runs/task-honest-40-docs/`
- `.runs/task-honest-50-fixtures/`
- `.runs/task-honest-60-merge/`
- `.runs/task-honest-70-verify/`

Sentinel rules:

- The parent writes `started.json` before each task begins.
- The parent writes exactly one terminal file per task: `done.json` or `blocked.json`.
- Workers do not write orchestration state.

## Worker Model

- Parent lane:
  - owns all semantic code
  - defines freeze
  - launches workers
  - invalidates stale lanes
  - merges completed lanes
  - runs final verification
- Worker lanes:
  - launch only from the recorded freeze sha
  - edit only assigned files
  - do not merge, rebase, or resolve cross-lane conflicts
  - return changed files, commands run, exit codes, and blockers only
- Maximum worker concurrency: `3`
- Worker lane set is fixed for this milestone:
  - Lane T: tests
  - Lane D: docs
  - Lane F: seeded fixtures

## Context-Control Rules

- Parent context stays limited to:
  - `PLAN.md`
  - `ORCH_PLAN.md`
  - current integration diff
  - parent-owned semantic files
  - `freeze.json`
- Worker prompts contain only:
  - owned files
  - the relevant `PLAN.md` excerpt
  - the frozen lifecycle sequence
  - the frozen compatibility rule
  - lane-local acceptance checks
  - forbidden touch surfaces
- Workers are disposable:
  - if a lane goes stale, discard it and relaunch from the newest freeze sha
  - do not manually salvage stale output into the integration branch

## Parent vs Worker Ownership Model

### Parent-only ownership

The parent owns the root semantic lane and any lifecycle-contract fallout in:

- `crates/xtask/src/publication_refresh.rs`
- `crates/xtask/src/agent_lifecycle.rs`
- `crates/xtask/src/agent_lifecycle/validation.rs`
- `crates/xtask/src/close_proving_run.rs`
- `crates/xtask/src/capability_publication.rs`
- `crates/xtask/src/agent_maintenance/drift/governance.rs`
- `crates/xtask/src/prepare_publication.rs` only if needed to keep `publication_ready` explicitly pre-refresh

Parent responsibilities:

- make refresh commit `published`
- write published-stage evidence and continuity fields during refresh
- promote support tier during refresh to `publication_backed` unless already `first_class`
- keep `publication_ready` pre-refresh only
- implement the narrow compatibility branch for legacy post-refresh `publication_ready`
- make closeout consume `published` on the normal path
- align maintenance and capability publication semantics with the honest post-refresh stage
- freeze the contract before any worker launches
- integrate all follow-on lanes and run final verification

### Worker ownership

Lane T: tests

- `crates/xtask/tests/agent_lifecycle_state.rs`
- `crates/xtask/tests/refresh_publication_entrypoint.rs`
- `crates/xtask/tests/prepare_publication_entrypoint.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_paths.rs`
- `crates/xtask/tests/support/agent_maintenance_drift_harness.rs`

Lane D: narrative docs

- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

Lane F: seeded lifecycle fixtures

- `docs/agents/lifecycle/**`

Workers never own:

- any `crates/xtask/src/**` file
- `PLAN.md`
- `ORCH_PLAN.md`
- `.runs/**`

## Worktree/Branch Plan With Concrete Names

Canonical paths:

- `REPO_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api`
- `WORKTREE_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/unified-agent-api-published-state-honesty`

Integration lane:

- branch: `codex/recommend-next-agent`
- worktree: `REPO_ROOT`
- owner: parent

Worker lanes, created only from the recorded freeze sha:

- Lane T
  - branch: `codex/recommend-next-agent-honest-published-tests`
  - worktree: `$WORKTREE_ROOT/tests`
- Lane D
  - branch: `codex/recommend-next-agent-honest-published-docs`
  - worktree: `$WORKTREE_ROOT/docs`
- Lane F
  - branch: `codex/recommend-next-agent-honest-published-fixtures`
  - worktree: `$WORKTREE_ROOT/fixtures`

Creation pattern after freeze:

```bash
mkdir -p "$WORKTREE_ROOT" "$RUN_ROOT"
FREEZE_SHA=$(jq -r '.freeze_sha' "$RUN_ROOT/freeze.json")

git worktree add -b codex/recommend-next-agent-honest-published-tests \
  "$WORKTREE_ROOT/tests" \
  "$FREEZE_SHA"

git worktree add -b codex/recommend-next-agent-honest-published-docs \
  "$WORKTREE_ROOT/docs" \
  "$FREEZE_SHA"

git worktree add -b codex/recommend-next-agent-honest-published-fixtures \
  "$WORKTREE_ROOT/fixtures" \
  "$FREEZE_SHA"
```

## Freeze / Restart / Stale-Lane Invalidation Rules

Freeze point:

- Workers launch only after the parent completes the root semantic lane and records `freeze_sha`.
- Freeze means these statements are locked for downstream lanes:
  - refresh is the only writer of `published`
  - the normal path is `publication_ready -> published -> closed_baseline`
  - closeout consumes `published` on the normal path
  - the legacy `publication_ready` compatibility branch definition is stable enough for docs, fixtures, and tests

Stale-lane invalidation triggers:

- The parent edits any of these files after worker launch:
  - `crates/xtask/src/publication_refresh.rs`
  - `crates/xtask/src/agent_lifecycle.rs`
  - `crates/xtask/src/agent_lifecycle/validation.rs`
  - `crates/xtask/src/close_proving_run.rs`
  - `crates/xtask/src/capability_publication.rs`
  - `crates/xtask/src/agent_maintenance/drift/governance.rs`
  - `crates/xtask/src/prepare_publication.rs` if it was part of the freeze
- The parent changes frozen wording that docs or fixtures were told to mirror.
- A worker edits outside its ownership set.
- A worker returns assumptions that conflict with `freeze.json`.

Restart rules:

- If one worker lane is stale, discard and relaunch only that lane from the newest freeze sha.
- If the parent semantic lane changes after freeze, all worker lanes are stale by default.
- Do not rebase stale lanes for this milestone; relaunch them cleanly.

## Merge Policy

- The parent is the only merge authority.
- Merge preconditions for any worker lane:
  - ownership boundaries are clean
  - lane-local validation passed
  - no stale trigger fired since lane launch
- Merge order is fixed:
  1. Lane F: seeded fixtures
  2. Lane D: docs
  3. Lane T: tests
- Reason for this order:
  - fixtures must reflect the frozen lifecycle truth before final test integration
  - docs should match the same frozen lifecycle story before the final verification sweep
  - tests land last so they validate the near-final tree
- If a worker diff conflicts with parent-owned semantic files, reject and relaunch the lane.
- If integration reveals new semantic work, return to the parent lane, write a new freeze sha, and relaunch affected workers.

## Task Graph

```text
HONEST-00 Baseline
  -> HONEST-10 Parent Semantic Authority Lane
  -> HONEST-20 Freeze
       -> HONEST-30 Tests Lane
       -> HONEST-40 Docs Lane
       -> HONEST-50 Fixtures Lane
  -> HONEST-60 Parent Integration
  -> HONEST-70 Final Verification
```

Critical path:

- `HONEST-10 -> HONEST-20 -> HONEST-60 -> HONEST-70`

Parallel window:

- `HONEST-30`, `HONEST-40`, and `HONEST-50` only, after `HONEST-20`

## Workstream Plan

### HONEST-00 Baseline

Owner: parent
Depends on: none

Actions:

- record branch, head sha, dirty state, and launch timestamp
- confirm the milestone title in `PLAN.md`
- confirm no unrelated in-flight edits make the parent semantic lane ambiguous

Exit:

- baseline recorded
- task registry initialized

### HONEST-10 Parent Semantic Authority Lane

Owner: parent
Depends on: `HONEST-00`

Files:

- `crates/xtask/src/publication_refresh.rs`
- `crates/xtask/src/agent_lifecycle.rs`
- `crates/xtask/src/agent_lifecycle/validation.rs`
- `crates/xtask/src/close_proving_run.rs`
- `crates/xtask/src/capability_publication.rs`
- `crates/xtask/src/agent_maintenance/drift/governance.rs`
- `crates/xtask/src/prepare_publication.rs` if required

Required outcomes:

- successful refresh commits `lifecycle_stage = published`
- refresh becomes the only committed writer of `published`
- refresh writes published-stage evidence and continuity fields
- refresh promotes support tier correctly
- closeout consumes `published` on the normal path
- legacy post-refresh `publication_ready` compatibility is explicit and narrow
- plain prepare-time `publication_ready` is not closable

Freeze gate:

- the parent must finish the semantic code and any immediate lifecycle validation adjustments before worker launch
- the parent must capture the final compatibility rule in `freeze.json`
- workers do not start while the semantic contract is still moving

Exit:

- semantic contract is frozen enough for downstream lanes

### HONEST-20 Freeze

Owner: parent
Depends on: `HONEST-10`

Actions:

- write `freeze.json`
- record:
  - freeze sha
  - frozen stage sequence
  - frozen refresh writer contract
  - frozen closeout compatibility rule
  - worker ownership boundaries
  - stale triggers

Exit:

- workers may fork from the freeze sha

### HONEST-30 Tests Lane

Owner: worker
Depends on: `HONEST-20`

Files:

- `crates/xtask/tests/agent_lifecycle_state.rs`
- `crates/xtask/tests/refresh_publication_entrypoint.rs`
- `crates/xtask/tests/prepare_publication_entrypoint.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_paths.rs`
- `crates/xtask/tests/support/agent_maintenance_drift_harness.rs`

Required assertions:

- `refresh-publication --write` leaves `lifecycle_stage = "published"`
- refresh writes published-stage evidence and continuity fields
- refresh promotes support tier to `publication_backed` unless already `first_class`
- refresh failure does not leave fake committed `published`
- `prepare-publication --write` still leaves `publication_ready`, never `published`
- closeout succeeds from canonical `published`
- ordinary prepare-time `publication_ready` is rejected
- only the explicit legacy compatibility shape may close from `publication_ready`
- maintenance accepts `published` and rejects pre-refresh `publication_ready` as a post-publication baseline

Lane-local validation:

- `cargo test -p xtask --test refresh_publication_entrypoint`
- `cargo test -p xtask --test agent_lifecycle_state`
- `cargo test -p xtask --test prepare_publication_entrypoint`
- `cargo test -p xtask --test onboard_agent_closeout_preview -- --nocapture`
- `cargo test -p xtask --test agent_maintenance_drift`

Exit:

- every changed branch in the lifecycle seam has direct regression coverage

### HONEST-40 Docs Lane

Owner: worker
Depends on: `HONEST-20`

Files:

- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

Required changes:

- restate the canonical path as `publication_ready -> published -> closed_baseline`
- describe `publication_ready` as pre-refresh only
- describe refresh success as committing `published`
- describe closeout as consuming `published` on the normal path
- describe the compatibility branch as transitional, not a second steady-state meaning

Lane-local validation:

- absence check:

```bash
! rg -n 'refresh-publication.*publication_ready|close-proving-run.*publication_ready' \
  docs/specs/cli-agent-onboarding-charter.md \
  docs/specs/unified-agent-api/capabilities-schema-spec.md \
  docs/cli-agent-onboarding-factory-operator-guide.md
```

- presence check:

```bash
rg -n 'publication_ready -> published -> closed_baseline|refresh-publication.*published|close-proving-run.*published' \
  docs/specs/cli-agent-onboarding-charter.md \
  docs/specs/unified-agent-api/capabilities-schema-spec.md \
  docs/cli-agent-onboarding-factory-operator-guide.md
```

Exit:

- normative and operator docs tell one lifecycle story

### HONEST-50 Fixtures Lane

Owner: worker
Depends on: `HONEST-20`

Files:

- `docs/agents/lifecycle/**`

Required changes:

- update only those seeded examples that represent post-refresh truth
- keep pre-refresh examples at `publication_ready` when they are intentionally pre-refresh
- ensure seeded lifecycle examples reflect `published` after successful refresh

Lane-local validation:

- absence check for closable or post-refresh `publication_ready` lifecycle fixtures:

```bash
! rg -n --multiline -P '"lifecycle_stage": "publication_ready"[\\s\\S]*"expected_next_command": "close-proving-run' \
  docs/agents/lifecycle/**/governance/lifecycle-state.json
```

- presence check for legitimate pre-refresh examples:

```bash
rg -n --multiline -P '"lifecycle_stage": "publication_ready"[\\s\\S]*"expected_next_command": "refresh-publication' \
  docs/agents/lifecycle/**/governance/lifecycle-state.json
```

- presence check for post-refresh truth:

```bash
rg -n --multiline -P '"lifecycle_stage": "published"[\\s\\S]*"expected_next_command": "close-proving-run' \
  docs/agents/lifecycle/**/governance/lifecycle-state.json
```

Exit:

- seeded lifecycle fixtures match the frozen lifecycle model

### HONEST-60 Parent Integration

Owner: parent
Depends on: `HONEST-30`, `HONEST-40`, `HONEST-50`

Actions:

- verify worker ownership boundaries
- merge Lane F, then Lane D, then Lane T
- run post-merge smoke checks after each merge
- reject and relaunch any stale or cross-boundary lane instead of hand-correcting it

Post-merge smoke:

- after Lane F: lifecycle fixture consistency sweep
- after Lane D: doc wording sweep
- after Lane T:
  - `cargo test -p xtask --test refresh_publication_entrypoint`
  - `cargo test -p xtask --test agent_lifecycle_state`
  - `cargo test -p xtask --test agent_maintenance_drift`

Exit:

- follow-on work is integrated on the parent branch

### HONEST-70 Final Verification

Owner: parent
Depends on: `HONEST-60`

Required command matrix:

```bash
cargo test -p xtask --test refresh_publication_entrypoint
cargo test -p xtask --test agent_lifecycle_state
cargo test -p xtask --test prepare_publication_entrypoint
cargo test -p xtask --test onboard_agent_closeout_preview -- --nocapture
cargo test -p xtask --test agent_maintenance_drift
make test
make preflight
```

Targeted proof mapping for the four milestone verification outcomes:

- refresh success writes `published`:
  - `cargo test -p xtask --test refresh_publication_entrypoint`
- refresh failure rolls back to the pre-refresh state:
  - `cargo test -p xtask --test refresh_publication_entrypoint`
- closeout consumes `published` on the normal path:
  - `cargo test -p xtask --test onboard_agent_closeout_preview -- --nocapture`
- compatibility `publication_ready` is explicit and narrow:
  - `cargo test -p xtask --test onboard_agent_closeout_preview -- --nocapture`
- maintenance-drift post-publication truth remains aligned with the honest lifecycle stage:
  - `cargo test -p xtask --test agent_maintenance_drift`

Required acceptance checks:

- a successful refresh leaves:

```json
{ "lifecycle_stage": "published" }
```

- `refresh-publication --write` is the only committed writer of `published`
- closeout consumes `published` on the normal path
- legacy `publication_ready` compatibility remains narrow and transitional
- the named targeted tests above are the direct proof that:
  - refresh success writes `published`
  - refresh failure rolls back to the pre-refresh state
  - closeout consumes `published` on the normal path
  - compatibility `publication_ready` remains explicit and narrow
- published-stage evidence and continuity fields are present when `published` is committed
- docs, fixtures, and tests all tell the same lifecycle story

Exit:

- acceptance log is complete
- the milestone is ready for branch closeout

## Tests And Acceptance

Minimum verification commands:

```bash
cargo test -p xtask --test refresh_publication_entrypoint
cargo test -p xtask --test agent_lifecycle_state
cargo test -p xtask --test prepare_publication_entrypoint
cargo test -p xtask --test onboard_agent_closeout_preview -- --nocapture
cargo test -p xtask --test agent_maintenance_drift
make test
make preflight
```

Acceptance fails if any of these remain false:

- `published` is not a reachable committed stage
- refresh can still finish normally while leaving steady-state `publication_ready`
- closeout still accepts ordinary prepare-time `publication_ready` as normal input
- the compatibility branch is broader than the frozen rule
- docs or seeded fixtures contradict the new stage sequence
- test coverage misses a changed lifecycle branch

## Assumptions

- `PLAN.md` remains the canonical milestone design for this slice.
- The current integration branch remains `codex/recommend-next-agent`.
- The current execution baseline remains HEAD `07a0ce9259b8` until work begins.
- The parent semantic lane can be contained to the named `crates/xtask/src/**` files.
- Narrative docs, seeded fixtures, and regression tests can safely wait until after freeze.
- Three follow-on worker lanes are the maximum useful concurrency for this milestone; more would add merge risk without reducing the true critical path.
