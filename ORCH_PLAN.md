# ORCH_PLAN - Maintenance Settlement Must Gate `closed_baseline`

## Summary

| Item | Value |
| --- | --- |
| Current checked-out branch context | `staging` |
| Workspace cleanliness baseline | clean |
| Authoritative milestone | repo-root `PLAN.md`, dated `2026-05-11` |
| Milestone title | `Maintenance Settlement Must Gate closed_baseline` |
| Plan revision baseline | `35cf547` |
| Run slug | `maintenance-settlement-gates-closed-baseline` |
| Execution branch | `codex/maintenance-settlement-gates-closed-baseline` |
| Workflow checkout ref | `origin/staging` |
| Parent role | sole integrator, sole checkpoint owner, sole live `aider` proof operator, sole final decision-maker |
| Worker model | `GPT-5.4`, `reasoning_effort=high` |
| Worker concurrency cap | `0` before `C1`; maximum `2` active worker lanes after `C1`, and only when file ownership is disjoint |

This milestone is intentionally serialized through the shared contract seam. The parent lands Step 1 alone, freezes the reusable interfaces, and only then opens bounded worker lanes for disjoint ownership. The parent also owns every merge, every checkpoint, the real `aider` proof from `publication_ready` to `closed_baseline`, and the final acceptance gate.

## Hard Guards

- `PLAN.md` wins over this file on any conflict.
- Keep `artifact_version = "1"`.
- The only allowed approval-side maintenance modes in this milestone are `release_watch_enrolled` and `explicitly_deferred`.
- `crates/xtask/data/agent_registry.toml` remains the only release-watch enrollment store.
- Deferral is evidence, not enrollment.
- `close-proving-run` is the only command allowed to write final machine-owned maintenance settlement evidence.
- No new lifecycle stage, no new maintenance taxonomy, no watcher topology redesign, no packet redesign, and no workflow family redesign is in scope.
- Parent-only ownership covers Step 1 shared primitives, all merge decisions, all `.runs/**` artifacts, all checkpoint freezes, the `aider` continuity update, the live `aider` proof, and final acceptance.
- Workers may not edit `PLAN.md`, `ORCH_PLAN.md`, `.runs/**`, `docs/specs/**` unless they are the designated docs lane, or any `aider` governance artifact.
- Workers must stop immediately if their fix requires touching Step 1 shared primitive files, crossing into another lane's file map, or introducing a third maintenance branch.
- No worker may hand-edit machine-owned closeout settlement fields.
- The plan must fail closed on these known regressions:
  - missing `[descriptor.maintenance]`
  - deferred onboarding accidentally materializing registry enrollment
  - approval truth not matching registry truth
  - stale `maintenance_settlement` payload surviving closeout
  - stale `aider` approval continuity after approval artifact SHA changes
  - historical backfill updating lifecycle state without matching closeout evidence

## Worktree Strategy

### Roots

- Worktree root:
  `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-maintenance-settlement-gates-closed-baseline`
- Run-state root:
  `/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.runs/maintenance-settlement-gates-closed-baseline`

### Required Worktrees

| Worktree | Branch | Base | Purpose |
| --- | --- | --- | --- |
| `staging-readonly` | `staging` | local `staging` | read-only comparison surface and final smoke diff |
| `parent-integrator` | `codex/maintenance-settlement-gates-closed-baseline` | `origin/staging` | parent critical path, Step 1, integration, `aider` proof, final verification |

### Conditional Worker Worktrees

| Worktree | Branch | Launch gate | Purpose |
| --- | --- | --- | --- |
| `ws-a-approval-registry` | `codex/maintenance-settlement-approval-registry` | `C1_SHA` | Phase 2 approval-to-registry materialization |
| `ws-b-closeout-evidence` | `codex/maintenance-settlement-closeout-evidence` | `C1_SHA` | Phase 3 closeout predicate and lifecycle evidence |
| `ws-c-historical-backfill` | `codex/maintenance-settlement-historical-backfill` | `C3_SHA` | Phase 4 historical repair for `opencode` and `gemini_cli` |
| `ws-d-normative-docs` | `codex/maintenance-settlement-normative-docs` | `C4_SHA` | Phase 6 normative docs and operator guide truth |

### Creation Commands

```bash
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-maintenance-settlement-gates-closed-baseline/staging-readonly staging
git worktree add -b codex/maintenance-settlement-gates-closed-baseline /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-maintenance-settlement-gates-closed-baseline/parent-integrator origin/staging
```

After `C1_SHA` exists:

```bash
git worktree add -b codex/maintenance-settlement-approval-registry /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-maintenance-settlement-gates-closed-baseline/ws-a-approval-registry "$C1_SHA"
git worktree add -b codex/maintenance-settlement-closeout-evidence /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-maintenance-settlement-gates-closed-baseline/ws-b-closeout-evidence "$C1_SHA"
```

After `C3_SHA` exists:

```bash
git worktree add -b codex/maintenance-settlement-historical-backfill /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-maintenance-settlement-gates-closed-baseline/ws-c-historical-backfill "$C3_SHA"
```

After `C4_SHA` exists:

```bash
git worktree add -b codex/maintenance-settlement-normative-docs /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-maintenance-settlement-gates-closed-baseline/ws-d-normative-docs "$C4_SHA"
```

## Run-State And Freeze Artifacts

### Authoritative Orchestration Files

- `.runs/maintenance-settlement-gates-closed-baseline/tasks.json`
- `.runs/maintenance-settlement-gates-closed-baseline/freeze.json`
- `.runs/maintenance-settlement-gates-closed-baseline/lane-status.json`
- `.runs/maintenance-settlement-gates-closed-baseline/session-log.md`

### Required Derived Evidence Files

- `.runs/maintenance-settlement-gates-closed-baseline/baseline.json`
- `.runs/maintenance-settlement-gates-closed-baseline/foundation-summary.md`
- `.runs/maintenance-settlement-gates-closed-baseline/integration-notes.md`
- `.runs/maintenance-settlement-gates-closed-baseline/worker-briefs/ws-a.md`
- `.runs/maintenance-settlement-gates-closed-baseline/worker-briefs/ws-b.md`
- `.runs/maintenance-settlement-gates-closed-baseline/worker-briefs/ws-c.md`
- `.runs/maintenance-settlement-gates-closed-baseline/worker-briefs/ws-d.md`
- `.runs/maintenance-settlement-gates-closed-baseline/historical-backfill-audit.md`
- `.runs/maintenance-settlement-gates-closed-baseline/aider-proof.md`
- `.runs/maintenance-settlement-gates-closed-baseline/merge-log.md`
- `.runs/maintenance-settlement-gates-closed-baseline/final-gates.md`
- `.runs/maintenance-settlement-gates-closed-baseline/decision.md`
- `.runs/maintenance-settlement-gates-closed-baseline/acceptance.md`

### Queue Contract

`tasks.json` is the execution queue. Minimum entry shape:

```json
{
  "id": "task/p1.3",
  "workstream": "WS-P1",
  "title": "Freeze shared maintenance contract primitives",
  "status": "pending|in_progress|blocked|completed",
  "owner": "parent|ws-a|ws-b|ws-c|ws-d",
  "depends_on": ["task/p1.2"],
  "checkpoint": "C0|C1|C2|C3|C4|C5|null",
  "notes": "short current state"
}
```

### Freeze Points

| Checkpoint | Meaning | Required contents |
| --- | --- | --- |
| `C0` | baseline frozen before edits | local `staging` SHA, `origin/staging` SHA, clean workspace snapshot, worktree paths, plan baseline SHA |
| `C1` | Step 1 shared primitives frozen | `C1_SHA`, approved Step 1 file list, exported validator surface, maintenance normalization/hash surface, targeted test results |
| `C2` | parallel lane launch base frozen | worker file maps, launch order, open risks, parent review note confirming no Step 1 overlap |
| `C3` | closeout/evidence schema frozen | `C3_SHA`, `maintenance_readiness_settled` evidence id, final `maintenance_settlement` field names, mismatch predicate summary, stale-payload overwrite rule |
| `C4` | code integration base for docs and `aider` proof frozen | `C4_SHA`, merged WS-A and WS-B SHAs, final approval-mode names, registry materialization summary, closeout predicate summary |
| `C5` | final acceptance frozen | `aider` proof SHA, final test matrix, backfill audit summary, doc parity summary, `make preflight` result |

## Workstream Plan

### WS-K0 - Kickoff And Run-State Bootstrap

Type: parent-only  
Goal: establish clean execution surfaces before any code change.

Task queue:

- `task/k0.1` create `staging-readonly`
- `task/k0.2` create `parent-integrator`
- `task/k0.3` initialize `.runs/maintenance-settlement-gates-closed-baseline/`
- `task/k0.4` seed `tasks.json`, `freeze.json`, `lane-status.json`, `session-log.md`
- `task/k0.5` record `C0`

Acceptance:

- worktrees exist exactly once
- run-state root exists and is empty except for initialized artifacts
- `C0` is written before Step 1 begins

### WS-P1 - Shared Contract Primitives

Type: parent-only  
Goal: land Phase 1 completely before any parallelism.

Owned surfaces:

- `crates/xtask/src/agent_registry/release_watch.rs`
- `crates/xtask/src/approval_artifact.rs`
- `crates/xtask/tests/recommend_next_agent_approval_artifact.rs`

Task queue:

- `task/p1.1` expose one reusable release-watch validation path
- `task/p1.2` add approval-side maintenance model, normalization, and section hashing
- `task/p1.3` tighten forward-moving flows so missing maintenance truth cannot pass where this milestone requires it
- `task/p1.4` extend shared contract tests
- `task/p1.5` write `foundation-summary.md`
- `task/p1.6` freeze `C1`
- `task/p1.7` freeze `C2` and launch worker briefs

Required validation:

```bash
cargo test -p xtask recommend_next_agent_approval_artifact -- --nocapture
```

Acceptance:

- Step 1 owns all contract naming
- no duplicate release-watch validator exists
- one canonical maintenance normalization and hashing path exists
- `C1_SHA` is the only valid base for WS-A and WS-B

### WS-A - Approval To Registry Materialization

Type: conditional worker  
Launch gate: `C1_SHA`

Owned surfaces:

- `crates/xtask/src/onboard_agent.rs`
- `crates/xtask/src/onboard_agent/descriptor.rs`
- `crates/xtask/src/onboard_agent/approval.rs`
- `crates/xtask/src/onboard_agent/preview.rs`
- `crates/xtask/tests/onboard_agent_entrypoint/approval_mode.rs`
- relevant fixtures under `crates/xtask/tests/fixtures/**` only if required by the listed test

Mission:

- thread maintenance truth through approval-mode onboarding
- ensure `release_watch_enrolled` materializes registry truth
- ensure `explicitly_deferred` never materializes registry truth
- make preview output explicit and replay stable

Required validation:

```bash
cargo test -p xtask onboard_agent_entrypoint -- approval_mode
```

Stop conditions:

- needs edits to `approval_artifact.rs` or `release_watch.rs`
- needs lifecycle or closeout file changes
- writes release-watch enrollment for the deferred branch
- cannot prove byte-stable replay

Acceptance:

- deferred write never enrolls
- enrolled write cannot omit registry truth
- preview and write paths show the same approved maintenance decision

### WS-B - Closeout Predicate And Immutable Evidence

Type: conditional worker  
Launch gate: `C1_SHA`

Owned surfaces:

- `crates/xtask/src/close_proving_run.rs`
- `crates/xtask/src/proving_run_closeout.rs`
- `crates/xtask/src/agent_lifecycle.rs`
- `crates/xtask/src/prepare_proving_run_closeout.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`
- `crates/xtask/tests/prepare_proving_run_closeout_entrypoint.rs`
- `crates/xtask/tests/agent_lifecycle_state.rs`
- relevant fixtures under `crates/xtask/tests/fixtures/**` only if required by the listed tests

Mission:

- add `maintenance_readiness_settled`
- gate `ClosedBaseline` on it
- implement the exact two-branch approval/registry predicate
- make `close-proving-run` overwrite stale machine-owned settlement payloads

Required validation:

```bash
cargo test -p xtask onboard_agent_closeout_preview -- close_proving_run_write
cargo test -p xtask prepare_proving_run_closeout_entrypoint -- --nocapture
cargo test -p xtask agent_lifecycle_state -- --nocapture
```

Stop conditions:

- needs edits to onboarding files
- needs a third predicate branch
- needs `prepare-proving-run-closeout` to decide final settlement truth
- cannot overwrite stale prepared settlement payloads deterministically

Acceptance:

- no legal `closed_baseline` path exists without settled maintenance truth
- mismatched approval and registry truth fails closed
- prepared stale settlement payloads cannot survive final closeout

### WS-P2 - Parent Integration And Dependent Lane Launch

Type: parent-only  
Goal: preserve the real dependency graph while still exploiting safe overlap.

Execution order:

1. Review and integrate WS-B as soon as it is valid.
2. Freeze `C3` immediately after WS-B merge.
3. Launch WS-C from `C3_SHA` if WS-C ownership remains disjoint from any active lane.
4. Review and integrate WS-A when valid.
5. Freeze `C4` only after both WS-A and WS-B are merged.
6. Launch WS-D from `C4_SHA`.
7. Record every merge decision in `merge-log.md` and `integration-notes.md`.

Rules:

- WS-C may overlap with WS-A because Step 4 depends on Step 3, not Step 2.
- WS-D may not start until the names and closeout fields from WS-A and WS-B are both stable.
- If either WS-A or WS-B returns changes outside its file map, reject the lane and relaunch only after parent cleanup.
- If Step 1 contracts need to move after WS-A or WS-B launch, stop both lanes, discard their outputs, amend Step 1 on the parent branch, and freeze a new `C1`.

### WS-C - Historical Backfill For Closed Agents

Type: conditional worker  
Launch gate: `C3_SHA`

Owned surfaces:

- `crates/xtask/src/historical_lifecycle_backfill.rs`
- `crates/xtask/tests/historical_lifecycle_backfill_entrypoint.rs`
- `docs/agents/lifecycle/opencode-cli-onboarding/governance/approved-agent.toml`
- `docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json`
- `docs/agents/lifecycle/opencode-cli-onboarding/governance/publication-ready.json`
- `docs/agents/lifecycle/opencode-cli-onboarding/governance/proving-run-closeout.json`
- `docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml`
- `docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json`
- `docs/agents/lifecycle/gemini-cli-onboarding/governance/publication-ready.json`
- `docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json`

Mission:

- backfill `opencode` and `gemini_cli` to truthful `explicitly_deferred`
- regenerate lifecycle and closeout evidence together
- prevent half-migrated historical truth

Required validation:

```bash
cargo test -p xtask historical_lifecycle_backfill_entrypoint -- --nocapture
```

Stop conditions:

- tries to enroll either historical agent in the registry
- requires touching `aider` governance artifacts
- updates lifecycle state without matching closeout settlement evidence
- requires a second inventory outside the registry

Acceptance:

- both historical agents remain `closed_baseline`
- both approval artifacts declare `explicitly_deferred`
- both closeout artifacts carry machine-owned maintenance settlement evidence
- no half-migrated historical state remains

### WS-D - Normative Docs And Operator Truth

Type: conditional worker  
Launch gate: `C4_SHA`

Owned surfaces:

- `docs/specs/agent-registry-contract.md`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

Mission:

- document the exact approval-side maintenance table
- document the exact two-branch closeout predicate
- document deferred steady state truthfully
- keep docs aligned to the landed field names from `C4`

Required validation:

```bash
rg -n "release_watch_enrolled|explicitly_deferred|maintenance_settlement|maintenance_readiness_settled" docs/specs docs/cli-agent-onboarding-factory-operator-guide.md
```

Stop conditions:

- contract names still moving after `C4`
- needs code edits
- introduces broader taxonomy or alternate enrollment storage

Acceptance:

- normative docs and operator guide match the merged code exactly
- docs say registry omission is the only unenrolled state
- docs state that deferred agents remain ineligible for release-watch maintenance

### WS-P3 - Parent `aider` Proof And Continuity Update

Type: parent-only  
Launch gate: `C4_SHA`

Owned surfaces:

- `docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml`
- `docs/agents/lifecycle/aider-onboarding/governance/lifecycle-state.json`
- `docs/agents/lifecycle/aider-onboarding/governance/publication-ready.json`
- `docs/agents/lifecycle/aider-onboarding/governance/proving-run-closeout.json`

Execution:

1. Add `descriptor.maintenance` to `aider` approval with `mode = "explicitly_deferred"`.
2. Run:
   ```bash
   cargo run -p xtask -- refresh-publication --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --write
   ```
3. Confirm approval SHA continuity is refreshed in `lifecycle-state.json` and `publication-ready.json`.
4. Run:
   ```bash
   cargo run -p xtask -- prepare-proving-run-closeout --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --write
   ```
5. Complete bounded human fields in `proving-run-closeout.json`.
6. Run:
   ```bash
   cargo run -p xtask -- close-proving-run --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --closeout docs/agents/lifecycle/aider-onboarding/governance/proving-run-closeout.json
   ```
7. Record all proof evidence in `aider-proof.md`.

Acceptance:

- `aider` cannot close without explicit maintenance settlement
- `aider` closes successfully once the deferred branch is satisfied
- `aider` remains absent from registry release-watch enrollment in this milestone
- no stale continuity survives the approval SHA change

### WS-P4 - Final Verification, Landing, And Decision

Type: parent-only  
Goal: finish the milestone from integrated code to acceptance.

Execution:

1. Integrate WS-C if present.
2. Integrate WS-D if present.
3. Rerun the earliest impacted honest phase after each merged lane.
4. Run the final targeted suite.
5. Run `make preflight`.
6. Freeze `C5`.
7. Update `final-gates.md`, `decision.md`, and `acceptance.md`.
8. Move `staging-readonly` to the reviewed parent result and inspect the smoke diff.
9. Land the reviewed result onto `staging`.

Acceptance:

- all nine PLAN success criteria are evidenced
- final targeted tests and `make preflight` pass
- final decision is evidence-backed, not inferred

## Launch Order

1. `WS-K0`
2. `WS-P1`
3. freeze `C1`
4. freeze `C2`
5. launch `WS-A` and `WS-B`
6. merge `WS-B` when ready
7. freeze `C3`
8. launch `WS-C`
9. merge `WS-A` when ready
10. freeze `C4`
11. launch `WS-D`
12. run parent `WS-P3` `aider` proof
13. merge `WS-C`
14. merge `WS-D`
15. run `WS-P4`

Parallelism rules:

- no worker concurrency before `C1`
- maximum `2` active worker lanes total
- `WS-C` may overlap with `WS-A`
- `WS-D` may overlap with parent `aider` proof, but not with unstable contract naming before `C4`
- if any lane discovers overlap, it must stop as `blocked`

## Worker Prompt Contract

Every worker brief must contain only:

- workstream id
- base checkpoint SHA
- mission
- exact owned file map
- allowed run-state files
- required validations
- stop conditions
- return contract

### Allowed Run-State Inputs

- `WS-A`
  - `foundation-summary.md`
  - `worker-briefs/ws-a.md`
- `WS-B`
  - `foundation-summary.md`
  - `worker-briefs/ws-b.md`
- `WS-C`
  - `foundation-summary.md`
  - `integration-notes.md`
  - `worker-briefs/ws-c.md`
- `WS-D`
  - `foundation-summary.md`
  - `integration-notes.md`
  - `worker-briefs/ws-d.md`

Workers may not read:

- full `.runs/**` beyond their listed summaries
- unrelated governance trees
- `aider` proof artifacts
- planning docs beyond the brief the parent writes

### Worker Return Contract

Every worker must return:

- workstream id
- status: `ready-for-parent`, `blocked`, or `no-op`
- base checkpoint SHA
- changed files
- commands run
- exit code for every command
- validation results
- resulting commit SHA
- residual risks

## Merge Rules And Honest Reruns

- Parent-only merge order is fixed by dependency, not arrival time.
- `WS-B` merges before `WS-C` can launch.
- `WS-A` and `WS-B` must both merge before `WS-D` or the parent `aider` proof can rely on stable names.
- No worker rebases itself after launch. If base drift matters, the parent decides whether to relaunch.
- After any merge that changes closeout semantics, rerun the earliest impacted targeted suite before the next checkpoint.
- If a merged lane changes approval field names or settlement field names after a downstream lane already launched, the downstream lane is invalid and must be relaunched from the new frozen checkpoint.
- Parent regenerates only through the real xtask commands. No manual rescue edits to machine-owned lifecycle or closeout evidence.

Rerun matrix:

- Step 1 changes: discard and relaunch `WS-A` and `WS-B`
- onboarding changes: rerun `cargo test -p xtask onboard_agent_entrypoint -- approval_mode`
- closeout or lifecycle changes: rerun `cargo test -p xtask onboard_agent_closeout_preview -- close_proving_run_write`, `cargo test -p xtask prepare_proving_run_closeout_entrypoint -- --nocapture`, `cargo test -p xtask agent_lifecycle_state -- --nocapture`, and the `aider` proof if already exercised
- historical backfill changes: rerun `cargo test -p xtask historical_lifecycle_backfill_entrypoint -- --nocapture`
- docs-only changes: no targeted rerun beyond final verification unless code or generated governance truth moved

## Context-Control Rules

- The parent keeps `tasks.json`, `freeze.json`, `lane-status.json`, `session-log.md`, `merge-log.md`, and `final-gates.md` authoritative.
- Workers get summaries, not the whole milestone transcript.
- Each worker brief must state the one thing that can make the lane invalid.
- If a worker needs more context than its brief and allowed files provide, it stops as `blocked`.
- Parent writes all cross-lane coupling notes into `integration-notes.md` before launching any dependent lane.

## Tests And Acceptance

### Required Targeted Commands

```bash
cargo test -p xtask recommend_next_agent_approval_artifact -- --nocapture
cargo test -p xtask onboard_agent_entrypoint -- approval_mode
cargo test -p xtask onboard_agent_closeout_preview -- close_proving_run_write
cargo test -p xtask prepare_proving_run_closeout_entrypoint -- --nocapture
cargo test -p xtask agent_lifecycle_state -- --nocapture
cargo test -p xtask historical_lifecycle_backfill_entrypoint -- --nocapture
make preflight
```

### Required Regression Locks

- approval parsing rejects mixed or missing maintenance truth where this milestone requires it
- deferred onboarding cannot materialize registry release-watch enrollment
- enrolled onboarding cannot omit the registry block
- closeout rejects approval/registry mismatch
- closeout overwrites stale machine-owned settlement payloads
- `ClosedBaseline` requires `maintenance_readiness_settled`
- historical backfill rewrites lifecycle and closeout together
- `aider` continuity refresh blocks stale approval SHA drift

### Acceptance Checklist

- success criterion 1: approval artifact supports `[descriptor.maintenance]` under `artifact_version = "1"`
- success criterion 2: `onboard-agent --write` writes registry `maintenance.release_watch` only for `release_watch_enrolled`
- success criterion 3: deferred mode leaves registry release-watch absent
- success criterion 4: `close-proving-run` accepts only the two legal branches
- success criterion 5: closeout records immutable machine-owned maintenance settlement evidence
- success criterion 6: `LifecycleStage::ClosedBaseline` requires `maintenance_readiness_settled`
- success criterion 7: `opencode` and `gemini_cli` are backfilled truthfully without a second inventory
- success criterion 8: `aider` proves the tightened deferred branch from `publication_ready`
- success criterion 9: `make preflight` passes

## Final Gate Logic

Before the milestone is complete, all of the following must be true:

- `C5` exists
- Step 1 remained parent-owned and stable before any worker launch
- WS-A and WS-B both landed or were explicitly replaced by parent-owned equivalents
- `maintenance_settlement` field names and predicate rules are frozen and reflected in docs
- `maintenance_readiness_settled` is enforced in lifecycle evidence and covered by targeted tests
- `opencode` and `gemini_cli` both show `explicitly_deferred` in approval artifacts and matching closed-baseline evidence
- `aider` closes from the real integrated branch only after the deferred branch is satisfied
- no registry release-watch block exists for `opencode`, `gemini_cli`, or `aider`
- every required targeted command is green
- `make preflight` is green
- `decision.md` summarizes final evidence and any accepted residual risk

Completion verdicts:

- `complete`: all nine success criteria are proven
- `blocked`: a stop condition fired and the parent recorded exact blocker evidence
- `invalidated`: a checkpoint moved underneath an active lane and the lane output was discarded
- `out_of_scope`: the only path forward requires forbidden topology or taxonomy expansion

## Stop Conditions

- Step 1 cannot be isolated without changing `artifact_version`
- any lane requires a third maintenance mode or a second enrollment store
- deferred onboarding writes registry release-watch truth
- closeout cannot be expressed as the exact two allowed branches
- stale machine-owned settlement payload cannot be overwritten deterministically
- `aider` approval SHA changes but continuity cannot be refreshed truthfully in-repo
- historical backfill leaves lifecycle and closeout evidence out of sync
- a worker needs to cross into another lane's file map
- the only remaining fix path requires watcher, packet, workflow, or maintenance-taxonomy redesign
- `make preflight` fails for reasons introduced by this milestone and the parent cannot localize the fix without breaking scope

## Assumptions

- `.runs/maintenance-settlement-gates-closed-baseline/` is created fresh for this session.
- Worker lanes may add or update narrow test fixtures under existing xtask fixture trees when their owned tests require it.
- `historical-lifecycle-backfill` remains the intended repo-native mechanism for repairing already-closed governance artifacts rather than manual JSON surgery.
