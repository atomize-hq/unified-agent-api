# Generic Capability Publication Foundation Orchestration Plan

## Summary

- Execute only against the current milestone in repo-root `PLAN.md`: `PLAN - Generic Capability Publication Foundation`.
- Integration branch is the live implementation branch `codex/recommend-next-agent`. `main` is review base only, never the worker fork point.
- Parent agent is the only integrator, merger, rebase authority, API-freeze owner, and final verifier.
- Worker model for delegated lanes:
  - model: GPT-5.4
  - reasoning: high
- Keep the critical path local to the parent for:
  - baseline capture against current repo state
  - shared seam freeze in `crates/xtask/src/capability_publication.rs`
  - `crates/xtask/src/lib.rs` export wiring
  - generator migration in `crates/xtask/src/capability_matrix.rs`
  - final convergence
  - test lane
  - final generated `docs/specs/unified-agent-api/capability-matrix.md`
- Maximum worker concurrency is `3`, and only after the parent freezes Lane A. This matches the justified post-freeze lanes in `PLAN.md`:
  - Lane B: audit + closeout reuse
  - Lane C: drift + prepare-publication alignment
  - Lane D: docs/spec updates
- No human approval gates are planned for this run. Stop only for hard guards, stale-lane reopen, or verification failure.
- This document replaces the old runtime-evidence orchestration plan entirely. That older plan is not authoritative for this milestone.

## Hard Guards

- Scope is locked to the current `PLAN.md` milestone only.
- Do not widen into:
  - support-matrix refactors
  - lifecycle-stage redesign
  - new binary/package/container artifact families
  - plugin discovery
  - dynamic runtime reflection
  - backend constructor discovery
  - orthogonality-rule redesign
- Locked implementation decisions from `PLAN.md` must remain intact:
  - lifecycle-driven publication eligibility
  - approval-registry continuity validation
  - shared projection ownership stays in `crates/xtask/src/capability_projection.rs`
  - one shared audit implementation
  - pre-runtime agents omitted from generated publication
  - no plugin/reflection architecture
- No consumer in scope may derive publication capability truth independently after this run.
- Parent and workers must adjust to the repo’s actual state at launch time. Never assume a clean worktree.
- Never revert someone else’s edits.
- Stop the run immediately if any of the following become true:
  - `PLAN.md` conflicts with a normative contract in `docs/specs/**`
  - the shared publication seam requires a lifecycle-model redesign
  - the shared publication seam requires introducing plugin/reflection discovery
  - a worker needs to edit a parent-owned file to finish its lane
  - a worker lane can no longer stay disjoint after the freeze point

## Orchestration State

Parent-owned local orchestration state lives under:

- `RUN_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.runs/generic-capability-publication`

Canonical parent-owned state files:

- `baseline.json`
  - current branch
  - current HEAD SHA
  - review base branch
  - dirty-file summary at launch
  - lane ownership snapshot against actual dirty state
  - note of any lane delayed because of pre-existing dirt
- `tasks.json`
  - ordered task list
  - task owner
  - task status
  - dependencies
  - restart count if a lane is reopened
- `session-log.md`
  - chronological parent decisions
  - worker launch notes
  - relaunch reasons
  - deviations resolved without widening scope
- `freeze.json`
  - freeze commit SHA
  - frozen public functions and public types in `crates/xtask/src/capability_publication.rs`
  - expected `crates/xtask/src/capability_matrix.rs` header semantics
  - any signature changes in `crates/xtask/src/capability_projection.rs`
  - lane ownership map
  - forbidden touch surfaces by lane
- `merge-log.md`
  - merge order
  - merge commit / fast-forward result
  - post-merge smoke command results
  - relaunch reason if a lane is rejected
- `acceptance.md`
  - final command sequence
  - exit codes
  - pass/fail status against each acceptance criterion
  - unresolved unrelated repo-state blockers, if any

Per-task sentinels live under:

- `REPO_ROOT/.runs/<task-id>/started.json`
- `REPO_ROOT/.runs/<task-id>/status.json`
- `REPO_ROOT/.runs/<task-id>/done.json`
- `REPO_ROOT/.runs/<task-id>/blocked.json`

Sentinel rules:

- one directory per task ID
- parent creates `started.json` before work begins
- parent updates `status.json` during long-running work or after worker returns
- parent writes exactly one terminal sentinel: `done.json` or `blocked.json`
- workers do not edit orchestration state directly

## Worker Model

- Parent owns all integration, orchestration state, freeze decisions, merge sequencing, and acceptance decisions.
- Workers edit only their assigned lane surfaces.
- Workers do not merge branches.
- Workers do not rebase.
- Workers do not modify `PLAN.md`.
- Workers do not modify orchestration state under `.runs/**`.
- Workers do not touch any parent-owned surface.

Worker return contract is fixed. Each worker returns only:

- changed files
- commands run
- exit codes
- blockers or assumptions

If a worker needs a parent-owned file changed, or needs the freeze contract changed, it stops and returns a blocker instead of guessing.

## Context-Control Rules

Parent active context stays narrow. The parent keeps only:

- `PLAN.md`
- `ORCH_PLAN.md`
- `RUN_ROOT/baseline.json`
- `RUN_ROOT/tasks.json`
- `RUN_ROOT/freeze.json` after freeze
- `RUN_ROOT/session-log.md`
- the latest integration diff summary

Worker prompts contain only:

- the owned files
- forbidden touch surfaces
- the relevant `PLAN.md` excerpt
- the current `freeze.json` contract
- the required commands
- the acceptance target for that lane

Workers are closed immediately after merge or rejection. Do not keep idle worker lanes open after their result is consumed.

## Parent vs Worker Ownership Model

### Parent-only ownership

The parent keeps these surfaces local from kickoff through final verification:

- `ORCH_PLAN.md`
- `crates/xtask/src/capability_publication.rs`
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/capability_projection.rs` only if a signature change is required to support the shared publication seam
- all merge/rebase/conflict resolution
- all test-lane edits under `crates/xtask/tests/**`
- final generated `docs/specs/unified-agent-api/capability-matrix.md`
- `.runs/generic-capability-publication/**`

Parent responsibilities:

- capture baseline branch / SHA / dirty state
- snapshot lane ownership against actual dirty files before worker launch
- freeze the shared source contract before any worker starts
- launch workers from the freeze commit only
- reject stale worker output
- merge in order
- run all final commands and decide acceptance

### Worker lane ownership

Workers own only their disjoint lane files.

Lane B: audit + closeout reuse

- `crates/xtask/src/capability_matrix_audit.rs`
- `crates/xtask/src/close_proving_run.rs`

Lane C: drift + prepare-publication alignment

- `crates/xtask/src/prepare_publication.rs`
- `crates/xtask/src/agent_maintenance/drift/shared.rs`
- `crates/xtask/src/agent_maintenance/drift/publication.rs`

Lane D: docs/spec prose alignment

- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/specs/unified-agent-api/README.md`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

Workers do not own:

- `crates/xtask/src/capability_publication.rs`
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/tests/**`
- generated `docs/specs/unified-agent-api/capability-matrix.md`
- `.runs/**`

## Worktree/Branch Plan With Concrete Names

Canonical paths:

- `REPO_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api`
- `WORKTREE_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/unified-agent-api-generic-capability-publication`

Integration surface:

- branch: `codex/recommend-next-agent`
- worktree: `REPO_ROOT`
- owner: parent only

Worker branches and worktrees, created only after the parent records the freeze commit:

- Lane B:
  - branch: `codex/recommend-next-agent-publication-audit-closeout`
  - worktree: `$WORKTREE_ROOT/publication-audit-closeout`
- Lane C:
  - branch: `codex/recommend-next-agent-publication-drift-prepare`
  - worktree: `$WORKTREE_ROOT/publication-drift-prepare`
- Lane D:
  - branch: `codex/recommend-next-agent-publication-docs-specs`
  - worktree: `$WORKTREE_ROOT/publication-docs-specs`

Creation commands from `REPO_ROOT` after freeze:

```bash
mkdir -p "$WORKTREE_ROOT" "$RUN_ROOT"
FREEZE_SHA=$(jq -r '.freeze_commit_sha' "$RUN_ROOT/freeze.json")

git worktree add -b codex/recommend-next-agent-publication-audit-closeout \
  "$WORKTREE_ROOT/publication-audit-closeout" \
  "$FREEZE_SHA"

git worktree add -b codex/recommend-next-agent-publication-drift-prepare \
  "$WORKTREE_ROOT/publication-drift-prepare" \
  "$FREEZE_SHA"

git worktree add -b codex/recommend-next-agent-publication-docs-specs \
  "$WORKTREE_ROOT/publication-docs-specs" \
  "$FREEZE_SHA"
```

Worktree rules:

- never fork workers from `main`
- never fork workers before freeze completes
- never reuse a dirty worker worktree
- never let workers merge back into the integration branch themselves

## Freeze Artifact And Restart Rule

The freeze point is the first parent commit where these are simultaneously true:

- `crates/xtask/src/capability_publication.rs` exists and exposes the parent-approved contract
- `crates/xtask/src/capability_matrix.rs` reads publication truth from that shared module
- `crates/xtask/src/lib.rs` exports the shared module
- `RUN_ROOT/freeze.json` exists and is complete

`freeze.json` is the authoritative worker contract. The parent must record:

- `freeze_commit_sha`
- frozen public functions in `crates/xtask/src/capability_publication.rs`
- frozen public types in `crates/xtask/src/capability_publication.rs`
- expected `capability_matrix.rs` header semantics
- any signature changes in `capability_projection.rs`
- lane ownership map
- forbidden touch surfaces per lane

Shared-seam change definition after worker launch:

- any public signature, type, or error-contract change in `crates/xtask/src/capability_publication.rs`
- any export change in `crates/xtask/src/lib.rs` that affects worker imports
- any parent change in `crates/xtask/src/capability_projection.rs` required by the shared publication contract
- any semantic change in how `crates/xtask/src/capability_matrix.rs` expects the shared publication inventory to behave
- any lane ownership or forbidden-surface change in `freeze.json`

Mandatory stale-worker rule:

1. If the shared seam changes after any worker starts, all open worker lanes are stale.
2. Stale worker output must not be merged, cherry-picked, or manually reconciled.
3. Parent must:
   - update `freeze.json`
   - record the new freeze SHA
   - mark affected task sentinels stale / blocked
   - delete stale worker worktrees and branches
   - relaunch fresh worktrees from the new freeze commit
4. Restart from the first downstream task affected by the change.

Lane-local reopen rule:

- If only a worker-owned file changes after launch, only that lane is stale.
- Parent should avoid touching worker-owned files after launch except to reject and relaunch the lane.

## Merge Policy

- Parent is the only merger.
- Parent merges only into `codex/recommend-next-agent` in `REPO_ROOT`.
- Parent does not ask workers to rebase.
- Parent may do local mechanical conflict resolution during merge.
- Any conflict that changes the frozen shared seam is a stop-and-reopen event.

Merge order:

1. Parent completes the freeze task locally and records `freeze.json`.
2. Launch Lanes B, C, and D from `freeze_commit_sha`.
3. Merge Lane B first.
4. Rerun Lane B smoke commands locally before merging anything else.
5. Merge Lane C second.
6. Rerun Lane C smoke commands locally before merging anything else.
7. Merge Lane D third.
8. If B or C changed semantics after the first docs draft, reject or relaunch Lane D from the current freeze before continuing.
9. Parent runs the test lane locally.
10. Parent runs regeneration and final verification locally.

Reject a worker lane immediately if it:

- edits a file outside its ownership set
- depends on changing the shared seam after launch
- assumes a clean tree and drops unrelated local state
- returns results without the required command / exit-code accounting

## Task Graph

```text
task/gcp-00-baseline
  ->
task/gcp-01-freeze
  ->
parallel:
  task/gcp-02-audit-closeout
  task/gcp-03-drift-prepare
  task/gcp-04-docs-specs
  ->
task/gcp-05-converge-tests
  ->
task/gcp-06-final-verify
```

## Workstream Plan

### task/gcp-00-baseline

Owner:

- parent

Owned surfaces:

- `RUN_ROOT/baseline.json`
- `RUN_ROOT/tasks.json`
- `RUN_ROOT/session-log.md`
- `REPO_ROOT/.runs/task-gcp-00-baseline/**`

Required work:

- capture current branch, HEAD SHA, and dirty-state summary
- confirm `PLAN.md` is the current milestone source of truth
- snapshot lane ownership against actual dirty state
- detect whether any worker-owned surface is already dirty and must be absorbed or delayed

Required commands:

```bash
git branch --show-current
git rev-parse HEAD
git status --short
```

Acceptance:

- baseline reflects actual repo state at launch time
- lane ownership snapshot exists in `baseline.json`
- parent has identified whether any lane must be delayed because of pre-existing dirt

### task/gcp-01-freeze

Owner:

- parent

Owned surfaces:

- `crates/xtask/src/capability_publication.rs`
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/capability_projection.rs` only if required
- `RUN_ROOT/freeze.json`
- `REPO_ROOT/.runs/task-gcp-01-freeze/**`

Forbidden surfaces:

- worker-owned lane files
- `crates/xtask/tests/**`
- authored docs/specs except generated semantics review notes in `session-log.md`

Required work:

- add `crates/xtask/src/capability_publication.rs`
- export it from `crates/xtask/src/lib.rs`
- move publication eligibility, continuity validation, inventory construction, and shared audit entrypoints into the new module
- migrate `crates/xtask/src/capability_matrix.rs` to consume the shared publication source
- remove generator dependence on hardcoded runtime backend construction
- record the frozen contract in `freeze.json`

Required commands:

```bash
cargo test -p xtask --test c8_capability_matrix_unit
cargo run -p xtask -- capability-matrix --check
```

Acceptance:

- `capability_matrix.rs` no longer owns publication truth derivation
- the parent can state the frozen shared interface concretely in `freeze.json`
- freeze is stable enough that downstream lanes can proceed independently

### task/gcp-02-audit-closeout

Owner:

- worker B

Owned surfaces:

- `crates/xtask/src/capability_matrix_audit.rs`
- `crates/xtask/src/close_proving_run.rs`

Forbidden surfaces:

- `crates/xtask/src/capability_publication.rs`
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/tests/**`
- docs/specs
- `.runs/**`

Required work:

- move the audit CLI entrypoint in `crates/xtask/src/capability_matrix_audit.rs` onto the shared publication audit
- remove duplicated audit logic and allowlist ownership from `crates/xtask/src/close_proving_run.rs`
- keep closeout write flow unchanged except for shared-audit reuse

Required commands:

```bash
cargo test -p xtask --test onboard_agent_closeout_preview
cargo run -p xtask -- capability-matrix-audit
```

Acceptance:

- `capability-matrix-audit` and `close-proving-run` use one audit implementation
- no local allowlist clone remains in `close_proving_run.rs`
- no publication truth derivation is reintroduced in the lane

### task/gcp-03-drift-prepare

Owner:

- worker C

Owned surfaces:

- `crates/xtask/src/prepare_publication.rs`
- `crates/xtask/src/agent_maintenance/drift/shared.rs`
- `crates/xtask/src/agent_maintenance/drift/publication.rs`

Forbidden surfaces:

- `crates/xtask/src/capability_publication.rs`
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/tests/**`
- docs/specs
- `.runs/**`

Required work:

- replace `prepare_publication.rs` continuity dependency on `capability_matrix` with the shared publication source
- route drift capability truth through the shared publication source
- keep support-matrix logic unchanged

Required commands:

```bash
cargo test -p xtask --test prepare_publication_entrypoint
cargo test -p xtask --test agent_maintenance_drift
cargo run -p xtask -- prepare-publication --approval docs/agents/lifecycle/codex-cli-onboarding/governance/approved-agent.toml --check
```

Acceptance:

- `prepare-publication`, drift, and generator all reason from the same publication truth
- no local capability-truth derivation remains in drift or `prepare_publication.rs`

### task/gcp-04-docs-specs

Owner:

- worker D

Owned surfaces:

- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/specs/unified-agent-api/README.md`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

Forbidden surfaces:

- generated `docs/specs/unified-agent-api/capability-matrix.md`
- all `crates/xtask/src/**`
- all `crates/xtask/tests/**`
- `.runs/**`

Required work:

- update the authored docs/specs to describe lifecycle-backed publication truth
- remove built-in-backend-inventory wording from the in-scope docs
- align prose with the frozen Lane A semantics only

Required commands:

```bash
rg -n "built-in backend|built-in backend config|default built-in backend config" docs/specs/unified-agent-api docs/cli-agent-onboarding-factory-operator-guide.md
```

Acceptance:

- no in-scope authored doc claims hardcoded backend constructor inventory is the publication truth source
- prose matches the frozen code semantics
- lane is eligible for relaunch if B/C semantics changed after docs draft started

### task/gcp-05-converge-tests

Owner:

- parent

Owned surfaces:

- merged integration branch state
- `crates/xtask/tests/**`
- `RUN_ROOT/merge-log.md`
- `RUN_ROOT/session-log.md`
- `REPO_ROOT/.runs/task-gcp-05-converge-tests/**`

Required work:

- merge B, then rerun Lane B smoke locally
- merge C, then rerun Lane C smoke locally
- merge D only after B/C semantics are confirmed stable, otherwise relaunch D
- add or finish shared regression coverage in `crates/xtask/tests/**`
- prove the generic behavior with a synthetic publication-eligible fixture
- prove pre-runtime omission
- prove continuity drift failures
- prove shared audit reuse and drift/generator parity

Required commands:

```bash
cargo test -p xtask --test onboard_agent_closeout_preview
cargo run -p xtask -- capability-matrix-audit
cargo test -p xtask --test prepare_publication_entrypoint
cargo test -p xtask --test agent_maintenance_drift
cargo run -p xtask -- prepare-publication --approval docs/agents/lifecycle/codex-cli-onboarding/governance/approved-agent.toml --check
cargo test -p xtask --test c8_capability_matrix_unit
cargo test -p xtask --test c8_spec_capability_matrix_paths
cargo test -p xtask --test prepare_publication_entrypoint
cargo test -p xtask --test agent_maintenance_drift
cargo test -p xtask --test onboard_agent_closeout_preview
make check
```

Acceptance:

- required test files in `PLAN.md` cover the locked regressions
- no consumer in scope still restates publication-capability derivation independently
- merge checkpoints are recorded in `merge-log.md`

### task/gcp-06-final-verify

Owner:

- parent

Owned surfaces:

- merged integration branch state
- generated `docs/specs/unified-agent-api/capability-matrix.md`
- `RUN_ROOT/acceptance.md`
- `REPO_ROOT/.runs/task-gcp-06-final-verify/**`

Required work:

- regenerate `docs/specs/unified-agent-api/capability-matrix.md` from merged code
- rerun the full ordered final verification sequence
- record command results and acceptance in `acceptance.md`

Required final ordered sequence:

1. Regenerate the matrix:
   ```bash
   cargo run -p xtask -- capability-matrix
   ```
2. Verify the regenerated matrix is clean:
   ```bash
   cargo run -p xtask -- capability-matrix --check
   ```
3. Verify the shared audit:
   ```bash
   cargo run -p xtask -- capability-matrix-audit
   ```
4. Verify drift against the merged truth:
   ```bash
   cargo run -p xtask -- check-agent-drift --agent codex
   ```
5. Verify publication continuity through the shared source path:
   ```bash
   cargo run -p xtask -- prepare-publication --approval docs/agents/lifecycle/codex-cli-onboarding/governance/approved-agent.toml --check
   ```

Failure handling:

- If a failure is caused by files in touched scope for this milestone, patch within the milestone touch set and rerun from the first failed step.
- If a failure is caused by unrelated pre-existing repo state outside the touch set, record it in `acceptance.md`, leave scope unchanged, and stop instead of widening the run.

Acceptance:

- generated capability publication matches the merged shared source
- all final gates pass, or any unrelated blocker is explicitly recorded as out of scope
- the branch is ready for review without hidden follow-up edits

## Tests And Acceptance

The run is accepted only if all of the following are true:

1. `cargo run -p xtask -- capability-matrix --check` passes without any hardcoded runtime backend inventory path in `capability_matrix.rs`.
2. `capability-matrix-audit` and `close-proving-run` use the same shared audit implementation.
3. A synthetic publication-eligible agent fixture can enter publication truth without a new `xtask` hardcoded backend arm.
4. A synthetic pre-runtime fixture is omitted from generated publication output.
5. `check-agent-drift` compares against the same shared publication truth used by the generator.
6. `prepare-publication` uses the shared continuity path, not the generator module.
7. In-scope authored docs and the generated matrix header no longer describe publication as built-in backend inventory.
8. No consumer in scope restates publication-capability derivation independently.

## Assumptions

- `PLAN.md` remains the current milestone source of truth for the full run.
- No human approval gate is required before merge; ordinary review happens after branch completion.
- Parent can create local worker worktrees under `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/`.
- The shared seam can be frozen with one parent-owned source module and surgical rewires, not a broader architecture rewrite.
- The generated `docs/specs/unified-agent-api/capability-matrix.md` should be refreshed only after the merged code is stable.
- Current repo dirt may overlap with lane-owned files; the parent must snapshot and honor that actual dirty state before launching workers, even if it forces a lane delay or ownership rebalance.
- If repo state changes under active work, the parent will re-evaluate ownership and relaunch stale lanes rather than forcing merges across moving semantics.
