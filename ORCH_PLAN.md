# Enclose Create-Mode Closeout Without Ad Hoc Authoring Orchestration Plan

## Summary

- Execute from the current checked-out branch `codex/recommend-next-agent`.
- Treat HEAD `f6023e1` on `2026-05-03` as the live execution baseline.
- Keep the critical semantic path local to the parent agent:
  - freeze the prepared-vs-closed contract first
  - keep `close-proving-run` finalization and historical backfill reuse on the parent lane
  - keep the parent as the only integrator and final verifier
- Use dedicated worktrees under:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/unified-agent-api-closeout-prep/{handoff,packet-docs,tests-fixtures}`
- Use workstream branches:
  - `codex/recommend-next-agent-closeout-handoff`
  - `codex/recommend-next-agent-closeout-packet-docs`
  - `codex/recommend-next-agent-closeout-tests-fixtures`
- Use GPT-5.4 with `reasoning_effort=high` for all worker lanes. Cap concurrency at `3`.
- Keep orchestration state in one local source of truth:
  - queue: `.runs/closeout-prep/tasks.json`
  - freeze: `.runs/closeout-prep/freeze.json`
  - session log: `.runs/closeout-prep/session-log.md`
  - acceptance log: `.runs/closeout-prep/acceptance.md`
- Treat `.runs/closeout-prep/**` as run artifacts and execution state, not authored product surfaces.

## Hard Guards

- Scope is locked to the current `PLAN.md` milestone only:
  - `Enclose Create-Mode Closeout Without Ad Hoc Authoring`
- Do not reopen the previous milestone:
  - `Make The Published State Honest In The Lifecycle Model`
- Do not invent a new lifecycle stage.
- Do not invent a second closeout artifact path.
- The canonical create-mode closeout file remains:
  - `docs/agents/lifecycle/<prefix>/governance/proving-run-closeout.json`
- The new command name is locked exactly as:
  - `prepare-proving-run-closeout --approval <path> --check|--write`
- `prepare-proving-run-closeout` must not accept `--closeout`.
- `refresh-publication --write` must hand off to:
  - `prepare-proving-run-closeout --approval <path> --write`
- `prepare-proving-run-closeout --write` must keep `lifecycle_stage = published`.
- `close-proving-run --write` remains the only writer of `closed_baseline`.
- The prepared-vs-closed contract is the central freeze gate. No downstream lane launches before it is frozen.
- `onboard-agent` preview must never render a prepared draft as a closed proving run.
- Historical backfill must migrate to the shared create-mode closeout builder/serializer. No new bespoke inline closeout JSON.
- Maintenance closeout behavior and schema stay out of scope.
- Parent agent is the only merge authority.
- No worker may edit files outside its assigned ownership set.
- If the parent changes any frozen prepared/closed semantics after worker launch, all launched workers are stale by default.

## Orchestration State

Parent-owned execution state lives under:

- `RUN_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.runs/closeout-prep`

State files:

- `baseline.json`
  - branch
  - head sha
  - launch timestamp
  - dirty-state summary
  - plan title
- `freeze.json`
  - freeze sha
  - frozen command names
  - frozen closeout state enum
  - frozen machine-owned vs human-owned field map
  - frozen placeholder vocabulary
  - frozen preview phase names
  - worker ownership map
  - stale-lane triggers
- `tasks.json`
  - task id
  - owner
  - status
  - dependencies
  - worktree path
  - launch sha
  - restart count
- `session-log.md`
  - parent decisions
  - worker launch/return summaries
  - stale-lane invalidations
  - merge sequence
- `acceptance.md`
  - command matrix
  - exit codes
  - final acceptance checklist

Per-task sentinels:

- `.runs/closeout-prep/task-closeout-00-baseline/`
- `.runs/closeout-prep/task-closeout-10-root-contract/`
- `.runs/closeout-prep/task-closeout-20-freeze/`
- `.runs/closeout-prep/task-closeout-30-handoff/`
- `.runs/closeout-prep/task-closeout-40-packet-docs/`
- `.runs/closeout-prep/task-closeout-50-tests-fixtures/`
- `.runs/closeout-prep/task-closeout-60-parent-final/`
- `.runs/closeout-prep/task-closeout-70-final-verify/`

Sentinel rules:

- Parent writes `started.json` before each task begins.
- Parent writes exactly one terminal marker per task:
  - `done.json`
  - `blocked.json`
- Workers do not write orchestration state.
- Worker results are summarized by the parent into `session-log.md`, not copied wholesale.

## Worker Model

- Parent lane:
  - owns the root contract freeze
  - owns final closeout finalization
  - owns historical backfill reuse
  - owns merge decisions
  - owns final verification
- Worker lanes:
  - launch only from the recorded freeze sha
  - edit only assigned files
  - do not merge, rebase, or resolve cross-lane conflicts
  - return only:
    - changed files
    - commands run
    - exit codes
    - blockers or unresolved assumptions
- Maximum worker concurrency: `3`
- Fixed worker lane set for this milestone:
  - Lane H: published-to-prepared handoff
  - Lane P: preview and docs
  - Lane T: tests and fixtures

## Context-Control Rules

- Parent working context stays limited to:
  - `PLAN.md`
  - `ORCH_PLAN.md`
  - `freeze.json`
  - `tasks.json`
  - parent-owned source files
  - latest integration diff summary
- Worker prompts contain only:
  - owned file set
  - exact relevant `PLAN.md` excerpt
  - frozen prepared/closed contract summary
  - required commands
  - forbidden touch surfaces
  - acceptance checks for that lane
- Parent reviews worker summaries and narrow diffs only. Do not pull whole worker transcripts into the main context.
- Workers are disposable:
  - if a lane goes stale, discard it
  - relaunch from the newest freeze sha
  - do not manually salvage partial stale work into integration
- Close a worker immediately after merge or rejection.

## Parent vs Worker Ownership Model

### Parent-only ownership before freeze

The parent owns the root contract lane in:

- `crates/xtask/src/proving_run_closeout.rs`
- `crates/xtask/src/agent_lifecycle.rs`
- `crates/xtask/src/main.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/prepare_proving_run_closeout.rs` only for skeleton/scaffold creation if required to keep the CLI contract compile-clean before freeze

Parent root-lane responsibilities:

- lock the create-mode closeout state model:
  - `prepared`
  - `closed`
- centralize create-mode closeout builder and serializer behavior
- centralize placeholder detection and machine-owned field rules
- lock the exact command spelling and lifecycle helper names
- scaffold any minimal CLI/module wiring needed so post-freeze worker lanes can stay disjoint
- record the prepared-vs-closed contract in `freeze.json`

### Worker ownership after freeze

Lane H: published-to-prepared handoff

- `crates/xtask/src/prepare_proving_run_closeout.rs`
- `crates/xtask/src/publication_refresh.rs`

Lane H responsibilities:

- implement `prepare-proving-run-closeout --check|--write`
- validate published continuity before draft generation
- derive the canonical closeout path from approval truth
- write the prepared draft and update next-command provenance
- make `refresh-publication --write` point to prepare-closeout instead of directly to closeout

Lane P: preview and docs

- `crates/xtask/src/onboard_agent/preview/render.rs`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

Lane P responsibilities:

- add `closeout_prepared` preview semantics
- ensure prepared packet surfaces never present as closed
- update operator and normative docs to reflect the new post-publication handoff

Lane T: tests and fixtures

- `crates/xtask/tests/refresh_publication_entrypoint.rs`
- `crates/xtask/tests/prepare_proving_run_closeout_entrypoint.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/preview_states.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/closeout_schema_validation.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_paths.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`
- `crates/xtask/tests/historical_lifecycle_backfill_entrypoint.rs`
- committed lifecycle/closeout fixture surfaces only if required to keep shared serializer parity:
  - `docs/agents/lifecycle/**/governance/proving-run-closeout.json`

Lane T responsibilities:

- add the new entrypoint test target
- update preview-state, schema, path, and write-mode regression coverage
- add historical backfill coverage against the shared serializer
- adjust committed closeout fixtures only if the shared serializer makes that necessary

### Parent-only ownership after freeze

The parent keeps exclusive ownership of the final semantic lane in:

- `crates/xtask/src/close_proving_run.rs`
- `crates/xtask/src/historical_lifecycle_backfill.rs`

Parent final-lane responsibilities:

- make `close-proving-run` consume prepared drafts on the normal path
- reject unresolved placeholders
- rewrite machine-owned fields from current truth before final write
- keep `close-proving-run` the only writer of `closed_baseline`
- migrate historical backfill to the shared builder/serializer

Workers never own:

- `PLAN.md`
- `ORCH_PLAN.md`
- `.runs/**`
- parent-owned semantic files listed above

## Worktree/Branch Plan With Concrete Names

Canonical paths:

- `REPO_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api`
- `WORKTREE_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/unified-agent-api-closeout-prep`

Integration lane:

- branch: `codex/recommend-next-agent`
- worktree: `REPO_ROOT`
- owner: parent

Worker lanes, created only from the recorded freeze sha:

- Lane H
  - branch: `codex/recommend-next-agent-closeout-handoff`
  - worktree: `$WORKTREE_ROOT/handoff`
- Lane P
  - branch: `codex/recommend-next-agent-closeout-packet-docs`
  - worktree: `$WORKTREE_ROOT/packet-docs`
- Lane T
  - branch: `codex/recommend-next-agent-closeout-tests-fixtures`
  - worktree: `$WORKTREE_ROOT/tests-fixtures`

Creation pattern after freeze:

```bash
mkdir -p "$WORKTREE_ROOT" "$RUN_ROOT"
FREEZE_SHA=$(jq -r '.freeze_sha' "$RUN_ROOT/freeze.json")

git worktree add -b codex/recommend-next-agent-closeout-handoff \
  "$WORKTREE_ROOT/handoff" \
  "$FREEZE_SHA"

git worktree add -b codex/recommend-next-agent-closeout-packet-docs \
  "$WORKTREE_ROOT/packet-docs" \
  "$FREEZE_SHA"

git worktree add -b codex/recommend-next-agent-closeout-tests-fixtures \
  "$WORKTREE_ROOT/tests-fixtures" \
  "$FREEZE_SHA"
```

## Freeze / Restart / Stale-Lane Invalidation Rules

Freeze point:

- Workers launch only after the parent completes the root contract lane and records `freeze_sha`.
- Freeze means these statements are locked for downstream lanes:
  - the new command name is exactly `prepare-proving-run-closeout`
  - the closeout state enum is exactly `prepared` / `closed`
  - the canonical artifact path stays `proving-run-closeout.json`
  - machine-owned vs human-owned field boundaries are frozen
  - placeholder vocabulary and rejection rules are frozen
  - preview phase naming is frozen
  - `close-proving-run` remains the sole `closed_baseline` writer

Stale-lane invalidation triggers:

- The parent edits any frozen root-contract file after worker launch:
  - `crates/xtask/src/proving_run_closeout.rs`
  - `crates/xtask/src/agent_lifecycle.rs`
  - `crates/xtask/src/main.rs`
  - `crates/xtask/src/lib.rs`
- The parent changes the frozen command spelling, field ownership map, placeholder vocabulary, or preview phase name.
- A worker edits outside its ownership set.
- A worker introduces:
  - a second closeout artifact path
  - a new lifecycle stage
  - `--closeout` on `prepare-proving-run-closeout`
- Parent final-lane integration discovers that the frozen contract itself must change.

Restart rules:

- If one worker lane is stale, discard and relaunch only that lane from the newest freeze sha.
- If the root contract changes after freeze, all worker lanes are stale by default.
- If parent final-lane work requires edits back in frozen root-contract files, stop, write a new freeze, and relaunch every affected worker lane.
- Do not rebase stale worker lanes. Relaunch cleanly.

## Merge Policy

- The parent is the only merge authority.
- Merge preconditions for any worker lane:
  - ownership boundaries are clean
  - lane-local validation passed
  - no stale trigger fired since lane launch
- Merge order is fixed:
  1. Lane H: handoff
  2. Lane P: packet/docs
  3. Parent final lane: closeout finalization + historical backfill reuse
  4. Lane T: tests/fixtures
- Reason for this order:
  - handoff wiring must exist before prepared packet behavior is meaningful
  - preview/docs should land against the same prepared-draft semantics
  - parent final lane must make `close-proving-run` and historical backfill honor the frozen contract before test assertions are treated as final
  - tests/fixtures merge last so they validate the near-final tree
- If a worker diff conflicts with parent-owned semantic files, reject and relaunch the lane.
- If integration reveals a gap inside worker-owned files but the frozen contract is still correct, bounce the fix back to that worker lane instead of hand-editing it on the parent branch.

## Task Graph

```text
CLOSEOUT-00 Baseline
  -> CLOSEOUT-10 Parent Root Contract
  -> CLOSEOUT-20 Freeze
       -> CLOSEOUT-30 Handoff Lane
       -> CLOSEOUT-40 Packet/Docs Lane
       -> CLOSEOUT-50 Tests/Fixtures Lane
  -> CLOSEOUT-60 Parent Final Lane
  -> CLOSEOUT-70 Final Verification
```

Critical path:

- `CLOSEOUT-10 -> CLOSEOUT-20 -> CLOSEOUT-30 -> CLOSEOUT-60 -> CLOSEOUT-70`

Parallel window:

- `CLOSEOUT-30`, `CLOSEOUT-40`, and `CLOSEOUT-50` may run concurrently only after `CLOSEOUT-20`

## Workstream Plan

### CLOSEOUT-00 Baseline

Owner: parent
Depends on: none

Actions:

- record branch, head sha, launch timestamp, and dirty state
- confirm `PLAN.md` title and baseline
- confirm the stale `ORCH_PLAN.md` has been superseded
- initialize `tasks.json`, `session-log.md`, and `acceptance.md`

Exit:

- baseline recorded
- orchestration state initialized

### CLOSEOUT-10 Parent Root Contract

Owner: parent
Depends on: `CLOSEOUT-00`

Files:

- `crates/xtask/src/proving_run_closeout.rs`
- `crates/xtask/src/agent_lifecycle.rs`
- `crates/xtask/src/main.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/prepare_proving_run_closeout.rs` only for skeleton/scaffold if needed

Required outcomes:

- create-mode closeout state model is locked to `prepared` and `closed`
- create-mode closeout serializer/builder path is centralized
- machine-owned and human-owned fields are explicit in code
- placeholder detection is explicit in code
- CLI/module contract for `prepare-proving-run-closeout` is frozen enough that downstream lanes can build against it

Verification:

- `cargo test -p xtask --no-run`

Acceptance for `CLOSEOUT-10`:

- no second artifact path exists
- no new lifecycle stage exists
- the parent can describe the exact frozen contract in one page of `freeze.json`

### CLOSEOUT-20 Freeze

Owner: parent
Depends on: `CLOSEOUT-10`

Actions:

- write `freeze.json`
- record:
  - freeze sha
  - command names
  - state enum
  - field ownership
  - placeholder vocabulary
  - preview phase names
  - worker ownership boundaries
  - stale triggers

Exit:

- workers may fork from the freeze sha

### CLOSEOUT-30 Handoff Lane

Owner: worker
Depends on: `CLOSEOUT-20`

Files:

- `crates/xtask/src/prepare_proving_run_closeout.rs`
- `crates/xtask/src/publication_refresh.rs`

Required outcomes:

- implement `prepare-proving-run-closeout --check|--write`
- derive the closeout path from approval truth
- validate published continuity before draft generation
- write a prepared draft without changing `lifecycle_stage = published`
- update lifecycle next-command provenance to point at `close-proving-run`
- update `refresh-publication --write` to point to `prepare-proving-run-closeout`

Lane-local validation:

- `cargo test -p xtask --test refresh_publication_entrypoint --no-run`
- `cargo test -p xtask --no-run`

Exit:

- published-to-prepared handoff wiring is implemented without touching final-closeout semantics

### CLOSEOUT-40 Packet/Docs Lane

Owner: worker
Depends on: `CLOSEOUT-20`

Files:

- `crates/xtask/src/onboard_agent/preview/render.rs`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

Required outcomes:

- add `closeout_prepared` preview behavior
- ensure prepared draft copy says the run is not yet closed
- update docs so post-publication flow is:
  - `refresh-publication`
  - `prepare-proving-run-closeout`
  - human bounded edits
  - `close-proving-run`

Lane-local validation:

- presence checks:

```bash
rg -n 'prepare-proving-run-closeout|closeout_prepared|prepared|closed_proving_run' \
  crates/xtask/src/onboard_agent/preview/render.rs \
  docs/specs/cli-agent-onboarding-charter.md \
  docs/specs/unified-agent-api/capabilities-schema-spec.md \
  docs/cli-agent-onboarding-factory-operator-guide.md
```

- absence checks:

```bash
! rg -n 'proving-run-closeout\\.draft|published -> close-proving-run' \
  docs/specs/cli-agent-onboarding-charter.md \
  docs/specs/unified-agent-api/capabilities-schema-spec.md \
  docs/cli-agent-onboarding-factory-operator-guide.md
```

Exit:

- packet preview and operator docs tell one prepared-vs-closed story

### CLOSEOUT-50 Tests/Fixtures Lane

Owner: worker
Depends on: `CLOSEOUT-20`

Files:

- `crates/xtask/tests/refresh_publication_entrypoint.rs`
- `crates/xtask/tests/prepare_proving_run_closeout_entrypoint.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/preview_states.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/closeout_schema_validation.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_paths.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`
- `crates/xtask/tests/historical_lifecycle_backfill_entrypoint.rs`
- committed fixture surfaces only if required:
  - `docs/agents/lifecycle/**/governance/proving-run-closeout.json`

Required outcomes:

- add the new entrypoint test target
- assert refresh now points at prepare-closeout
- assert prepared drafts parse and serialize correctly
- assert prepared drafts do not preview as closed
- assert close-proving-run finalizes prepared drafts and rejects unresolved placeholders
- assert historical backfill uses the shared serializer path

Lane-local validation:

- `cargo test -p xtask --test onboard_agent_closeout_preview --no-run`
- `cargo test -p xtask --test historical_lifecycle_backfill_entrypoint --no-run`
- `cargo test -p xtask --test prepare_proving_run_closeout_entrypoint --no-run`

Exit:

- regression surfaces exist for every new seam even if full green proof waits for parent final integration

### CLOSEOUT-60 Parent Final Lane

Owner: parent
Depends on: `CLOSEOUT-30`, `CLOSEOUT-40`, `CLOSEOUT-50`

Files:

- `crates/xtask/src/close_proving_run.rs`
- `crates/xtask/src/historical_lifecycle_backfill.rs`

Actions:

- merge Lane H
- merge Lane P
- implement close-proving-run finalization on the parent lane
- migrate historical backfill to the shared serializer/builder path
- merge Lane T only after the final-closeout path matches the frozen contract

Parent-only semantic outcomes:

- prepared drafts are the normal create-mode closeout input
- unresolved placeholders block closeout
- machine-owned fields are rewritten from current truth before final write
- `close-proving-run` alone writes `closed_baseline`
- historical backfill no longer emits bespoke inline closeout JSON

Post-merge smoke:

- after Lane H:
  - `cargo test -p xtask --test refresh_publication_entrypoint --no-run`
- after Lane P:
  - `cargo test -p xtask --test onboard_agent_closeout_preview --no-run`
- after parent final-lane edits:
  - `cargo test -p xtask --test historical_lifecycle_backfill_entrypoint --no-run`
- after Lane T:
  - `cargo test -p xtask --no-run`

Exit:

- the full milestone implementation is integrated on the parent branch

### CLOSEOUT-70 Final Verification

Owner: parent
Depends on: `CLOSEOUT-60`

Required command matrix:

```bash
cargo test -p xtask --test refresh_publication_entrypoint
cargo test -p xtask --test prepare_proving_run_closeout_entrypoint
cargo test -p xtask --test onboard_agent_closeout_preview
cargo test -p xtask --test historical_lifecycle_backfill_entrypoint
make test
make preflight
```

If any command fails:

- write `.runs/closeout-prep/task-closeout-70-final-verify/blocked.json`
- stop
- do not silently weaken acceptance

## Tests and Acceptance

### Root Contract

- `proving_run_closeout.rs` owns the shared create-mode closeout state and serializer path.
- `prepared` and `closed` are the only accepted create-mode closeout states.
- The canonical artifact path remains `proving-run-closeout.json`.

### Handoff

- `refresh-publication --write` records `prepare-proving-run-closeout --approval <path> --write` as the next command.
- `prepare-proving-run-closeout --write` derives the closeout path from approval truth.
- `prepare-proving-run-closeout --write` writes a prepared draft without advancing lifecycle stage beyond `published`.

### Preview and Docs

- prepared packet surfaces render as prepared, not closed
- operator and normative docs describe the new post-publication handoff sequence consistently

### Final Closeout and Historical Backfill

- `close-proving-run` consumes prepared drafts on the normal path
- unresolved placeholders are rejected
- machine-owned fields are rewritten at finalization time
- `close-proving-run` remains the sole `closed_baseline` writer
- `historical_lifecycle_backfill` uses the shared create-mode closeout builder/serializer

### Verification Matrix

- `cargo test -p xtask --test refresh_publication_entrypoint`
  - proves refresh points at prepare-closeout and preserves published semantics
- `cargo test -p xtask --test prepare_proving_run_closeout_entrypoint`
  - proves the new command exists and materializes prepared drafts correctly
- `cargo test -p xtask --test onboard_agent_closeout_preview`
  - proves prepared drafts render as prepared and close-proving-run finalizes them correctly
- `cargo test -p xtask --test historical_lifecycle_backfill_entrypoint`
  - proves historical backfill stays truthful through the shared serializer path
- `make test`
  - repo-wide regression gate
- `make preflight`
  - final integration gate

## Assumptions

- The current branch `codex/recommend-next-agent` remains the integration lane for this session.
- Worker lanes are allowed to use dedicated git worktrees under the absolute path named above.
- GPT-5.4 with `reasoning_effort=high` is available for all worker lanes.
- The parent can create a compile-clean skeleton for `prepare_proving_run_closeout.rs` before freeze if needed, then stop touching that file after freeze.
- Committed `docs/agents/lifecycle/**/governance/proving-run-closeout.json` files may not need changes if the shared serializer is byte-compatible with current historical examples. The tests/fixtures lane owns those files only if updates become necessary.
- There is no intentional human approval gate inside this milestone. The only pauses are:
  - freeze creation
  - stale-lane invalidation
  - blocked final verification
