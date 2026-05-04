# Enclose The Recommendation Research Host Surface Orchestration Plan

## Summary

- Execute from the current checked-out branch `codex/recommend-next-agent` in `/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api`.
- Treat live workspace HEAD `8e097f4` as the launch baseline.
- Treat the current working copy of `PLAN.md` as the authoritative milestone contract for orchestration.
- Keep the parent agent as the only contract freezer, merge authority, integrator, proving-run operator, and final verifier.
- Use subagents only after the parent freezes the command contract and ownership boundaries.
- Use dedicated worker worktrees under `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/unified-agent-api-recommend-next-agent-research/`.
- Use workstream branches:
  - `codex/recommend-next-agent-research-core`
  - `codex/recommend-next-agent-research-docs-skill`
  - `codex/recommend-next-agent-research-python-gap`
- Use GPT-5.4 with `reasoning_effort=high` for every worker lane.
- Cap worker concurrency at `2` by default, with a third lane launched only if a real Python-side contract gap is proven.
- Treat `.runs/**` and `docs/agents/.uaa-temp/**` as derived/run surfaces, not committed deliverables, unless this plan explicitly promotes outputs during the final proving phase.
- Parent critical path:
  `RNA-00 Baseline -> RNA-10 Lock Command And Contract -> RNA-20 Freeze And Launch Gate -> RNA-60 Parent Integration Gate -> RNA-70 Parent Proving And Review Gate -> RNA-75 Parent Promote And Handoff Gate -> RNA-80 Final Verification`

## Hard Guards

- Scope is locked to the current `PLAN.md` milestone only:
  `Enclose The Recommendation Research Host Surface`
- Do not redesign recommendation scoring, shortlist dimensions, packet templates, approval artifacts, or create-lane behavior.
- The new command is locked exactly as:
  `cargo run -p xtask -- recommend-next-agent-research --dry-run|--write`
- Pass support is locked to `pass1` and `pass2`.
- `pass2` requires prior insufficiency input and a fresh `run_id`.
- The Python runner remains post-research only:
  `freeze-discovery`, `generate`, and `promote` stay in `scripts/recommend_next_agent.py`.
- The repo, not Codex, owns prompt rendering, dry-run packet creation, bounded Codex execution, `freeze-discovery` invocation, validation, and execution evidence.
- Codex write roots are limited to:
  `docs/agents/.uaa-temp/recommend-next-agent/discovery/<run_id>/`
  `docs/agents/.uaa-temp/recommend-next-agent/research/<run_id>/`
- The execution packet root is locked to:
  `docs/agents/.uaa-temp/recommend-next-agent/research-runs/<run_id>/`
- `--write` is invalid without a preexisting dry-run packet for the same `run_id`.
- `generate` and `promote` keep their public CLI shape unchanged.
- The proving run is last. It validates the sealed contract and must not define the contract.
- If pass1 and pass2 both fail to produce a sufficient run, the milestone halts. No `promote` occurs.
- If the proving flow reveals a contract defect, halt, reopen freeze, invalidate downstream lanes, and return to implementation.
- No worker may touch:
  `PLAN.md`
  `ORCH_PLAN.md`
  `.runs/**`
  `docs/agents/.uaa-temp/**`
  `docs/agents/selection/runs/**`
  `docs/agents/lifecycle/**/governance/approved-agent.toml`
- Parent is the only merge authority and the only lane allowed to create real proving artifacts.

## Orchestration State

Parent-owned orchestration state lives under:

- `RUN_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.runs/recommend-next-agent-research`

State files:

- `baseline.json`
- `freeze.json`
- `tasks.json`
- `session-log.md`
- `acceptance.md`

Required contents:

- `baseline.json`
  - branch
  - head sha
  - dirty-state summary
  - launch timestamp
  - plan title
- `freeze.json`
  - frozen CLI spelling
  - frozen flag rules
  - frozen pass rules
  - frozen allowed roots
  - frozen packet filenames
  - frozen proving-run ordering
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
  - worker launch summaries
  - stale-lane invalidations
  - merge outcomes
  - proving review notes
- `acceptance.md`
  - command matrix
  - gate results
  - completion checklist
  - final verification status

Per-task sentinels:

- `.runs/recommend-next-agent-research/task-rna-00-baseline/`
- `.runs/recommend-next-agent-research/task-rna-10-parent-contract/`
- `.runs/recommend-next-agent-research/task-rna-20-freeze/`
- `.runs/recommend-next-agent-research/task-rna-30-core-xtask/`
- `.runs/recommend-next-agent-research/task-rna-40-docs-skill/`
- `.runs/recommend-next-agent-research/task-rna-50-python-gap/`
- `.runs/recommend-next-agent-research/task-rna-60-parent-integration/`
- `.runs/recommend-next-agent-research/task-rna-70-parent-proving-review/`
- `.runs/recommend-next-agent-research/task-rna-75-parent-promote-handoff/`
- `.runs/recommend-next-agent-research/task-rna-80-final-verify/`

Sentinel rules:

- Parent writes `started.json` before each task begins.
- Parent writes exactly one terminal marker per task:
  `done.json`
  `blocked.json`
  `skipped.json`
- Workers do not write orchestration state.
- Worker results are summarized by the parent into `session-log.md`, not copied wholesale.

## Worker Model

- Parent lane owns:
  contract freeze, worker launch, merge policy, integration, proving execution, proving review gates, final acceptance.
- Worker lanes launch only from the recorded `freeze_sha`.
- Worker lanes return only:
  changed files, commands run, exit codes, blockers, unresolved assumptions.
- Worker lanes do not merge, rebase, or resolve cross-lane conflicts.
- Worker lanes do not run the real proving flow.
- Worker lanes use GPT-5.4 with `reasoning_effort=high`.

Fixed lane set for this milestone:

- Lane C
  - purpose: core xtask implementation, harness, entrypoint tests
- Lane D
  - purpose: docs/spec/operator-guide/skill rewrite against frozen contract
- Lane P
  - purpose: Python-side regression adjustment only if a genuine downstream contract gap appears
- Parent proving lane
  - purpose: final pass1/pass2 execution, run review, promotion, handoff, final verification

## Context-Control Rules

- Parent working context stays limited to:
  `PLAN.md`, this orchestration plan, `.runs/recommend-next-agent-research/**`, frozen contract summaries, targeted diffs, and final integration results.
- Worker prompts contain only:
  owned file set, exact `PLAN.md` excerpts, the frozen contract summary, required commands, forbidden surfaces, and lane-local acceptance checks.
- Workers are disposable:
  if a lane goes stale, discard it and relaunch from the newest freeze sha.
- Do not pull whole worker transcripts into the parent context.
- Lane D may restate the frozen contract but may not invent or reinterpret it.
- Lane P may adjust tests only; it may not widen Python runtime behavior without a new parent freeze.

## Parent vs Worker Ownership Model

### Parent-only ownership before freeze

- `crates/xtask/src/main.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/recommend_next_agent_research.rs` for CLI skeleton, argument model, and frozen contract constants only
- `.runs/recommend-next-agent-research/**`

Parent pre-worker responsibilities:

- lock the command spelling and argument rules
- lock pass1 and pass2 semantics
- lock allowed write roots and packet root
- lock packet/evidence filenames
- lock the rule that `--write` requires a matching dry-run packet
- lock the proving-run ordering
- lock the rule that docs/spec/skill rewrite depends on the frozen command contract
- record all of the above in `freeze.json`

### Lane C ownership after freeze

- `crates/xtask/src/recommend_next_agent_research.rs`
- `crates/xtask/src/recommend_next_agent_research/codex_exec.rs` only if extraction is structurally necessary
- `crates/xtask/src/recommend_next_agent_research/models.rs` only if extraction is structurally necessary
- `crates/xtask/src/recommend_next_agent_research/render.rs` only if extraction is structurally necessary
- `crates/xtask/src/recommend_next_agent_research/validation.rs` only if extraction is structurally necessary
- `crates/xtask/tests/recommend_next_agent_research_entrypoint.rs`
- `crates/xtask/tests/support/recommend_next_agent_research_harness.rs`

Lane C responsibilities:

- implement PLAN Step 2 dry-run packet rendering
- implement PLAN Step 3 discovery execution and repo-owned `freeze-discovery` handoff
- implement PLAN Step 4 research execution and validation
- implement PLAN Step 6 xtask entrypoint and harness coverage
- preserve the frozen CLI and proving-order semantics

### Lane D ownership after freeze

- `docs/specs/cli-agent-recommendation-dossier-contract.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `.codex/skills/recommend-next-agent/SKILL.md`

Lane D responsibilities:

- implement PLAN Step 5 docs/spec/skill rewrite
- rewrite the normative contract to describe the repo-owned host surface
- rewrite the operator guide to the same exact flow
- reduce the skill to a thin wrapper over the repo-owned procedure
- treat Lane C semantics in `freeze.json` as authoritative

### Lane P ownership after trigger only

- `scripts/test_recommend_next_agent.py`

Lane P responsibilities:

- implement only the minimum PLAN Step 6 Python regression coverage required if a real downstream contract gap appears
- keep Python runtime behavior unchanged unless the parent halts and issues a new freeze

### Parent-only ownership after freeze

- merge decisions
- `.runs/recommend-next-agent-research/**`
- proving-run execution under `.uaa-temp`
- run inspection and promotion decisions
- committed proving outputs under `docs/agents/selection/runs/**`
- lifecycle handoff surfaces under `docs/agents/lifecycle/**/governance/approved-agent.toml`

## Worktree And Branch Plan

Canonical paths:

- `REPO_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api`
- `WORKTREE_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/unified-agent-api-recommend-next-agent-research`

Integration lane:

- branch: `codex/recommend-next-agent`
- workspace: `REPO_ROOT`
- owner: parent

Worker lanes, created only from the recorded `freeze_sha`:

- Lane C
  - branch: `codex/recommend-next-agent-research-core`
  - worktree: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/unified-agent-api-recommend-next-agent-research/core`
- Lane D
  - branch: `codex/recommend-next-agent-research-docs-skill`
  - worktree: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/unified-agent-api-recommend-next-agent-research/docs-skill`
- Lane P
  - branch: `codex/recommend-next-agent-research-python-gap`
  - worktree: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/unified-agent-api-recommend-next-agent-research/python-gap`

Creation pattern after freeze:

```bash
mkdir -p "$WORKTREE_ROOT" "$RUN_ROOT"
FREEZE_SHA=$(jq -r '.freeze_sha' "$RUN_ROOT/freeze.json")

git worktree add -b codex/recommend-next-agent-research-core \
  "$WORKTREE_ROOT/core" \
  "$FREEZE_SHA"

git worktree add -b codex/recommend-next-agent-research-docs-skill \
  "$WORKTREE_ROOT/docs-skill" \
  "$FREEZE_SHA"
```

Conditional Lane P creation:

```bash
git worktree add -b codex/recommend-next-agent-research-python-gap \
  "$WORKTREE_ROOT/python-gap" \
  "$FREEZE_SHA"
```

## Freeze / Restart / Stale-Lane Invalidation Rules

Freeze point:

- Workers launch only after `RNA-20` records `freeze_sha`.
- Freeze means these statements are locked for downstream lanes:
  - command spelling
  - flag rules
  - pass rules
  - allowed write roots
  - execution packet root
  - packet/evidence filenames
  - proving-run ordering
  - ownership boundaries
  - acceptance gates

Stale-lane invalidation triggers:

- Parent edits any frozen contract item after worker launch.
- Parent changes any of these files after worker launch in a way that changes command semantics:
  - `crates/xtask/src/main.rs`
  - `crates/xtask/src/lib.rs`
  - `crates/xtask/src/recommend_next_agent_research.rs`
- Lane C discovers a required semantic change to packet filenames, pass rules, allowed roots, or proof ordering.
- Lane D writes behavior not present in `freeze.json`.
- Lane P attempts to change Python runtime behavior rather than narrow test coverage.
- Any worker edits outside its ownership set.
- Any worker relies on committed `.uaa-temp` or `docs/agents/selection/runs/**` outputs before the proving phase.
- The proving flow reveals a contract defect.

Restart rules:

- If one worker lane is stale, discard and relaunch only that lane from the newest freeze sha.
- If the parent changes the frozen contract, all active worker lanes are stale by default.
- If the proving flow reveals a contract defect, halt, reopen freeze, and invalidate all downstream lanes and docs that depended on the prior contract.
- Do not salvage stale work into integration.
- Do not rebase stale lanes. Relaunch cleanly.

## Conflict And Escalation Policy

- If Lane C and Lane D disagree on command semantics, Lane C wins because executable behavior is authoritative. Lane D is discarded and relaunched against the latest freeze.
- If Lane P starts surfacing product-behavior pressure instead of a test-only adjustment, halt and reopen freeze rather than letting Lane P drift scope.
- If integration finds a defect in worker-owned files, prefer bounce-back or relaunch over ad hoc parent patching.
- If worker-owned fixes would force a frozen contract change, halt, reopen freeze, and invalidate affected downstream lanes.
- If a lane cannot satisfy its acceptance checks without touching forbidden surfaces, it is blocked, not widened.

## Merge Policy

- Parent is the only merge authority.
- Merge preconditions for any worker lane:
  - lane-local validation passed
  - ownership boundaries are clean
  - no stale trigger fired since lane launch
  - the lane still matches `freeze.json`
- Merge order is fixed:
  1. Lane C
  2. Lane P if activated
  3. Lane D
  4. Parent proving lane
- Reason for this order:
  - Lane C defines the executable command surface
  - Lane P is allowed only as a narrow downstream test adjustment after Lane C proves it is needed
  - Lane D must land against the latest frozen executable semantics
  - proving happens only after code, tests, docs, and skill all agree
- If a worker diff conflicts with parent-owned or frozen semantic files, reject and relaunch the lane.
- If integration reveals a gap inside worker-owned files but the frozen contract is still correct, bounce the fix back to the worker lane instead of hand-editing it on the parent branch.

## Task Graph

```text
RNA-00 Baseline
  -> RNA-10 Parent Lock Command And Contract
  -> RNA-20 Parent Freeze And Worker Launch Gate
       -> RNA-30 Core Xtask Lane
       -> RNA-40 Docs/Spec/Skill Lane
       -> RNA-50 Optional Python Gap Lane
  -> RNA-60 Parent Integration Gate
  -> RNA-70 Parent Proving And Review Gate
  -> RNA-75 Parent Promote And Handoff Gate
  -> RNA-80 Final Verification
```

Critical path:

- `RNA-10 -> RNA-20 -> RNA-30 -> RNA-60 -> RNA-70 -> RNA-75 -> RNA-80`

Parallel window:

- `RNA-30` and `RNA-40` may run concurrently only after `RNA-20`.
- `RNA-50` may launch only if `RNA-30` or `RNA-60` proves a genuine Python-side contract gap.

## Serialized Parent Lane

### Pre-worker serialized gate

The parent must complete these in order before any worker launches:

1. `RNA-00` baseline capture
2. `RNA-10` command and contract lock
3. `RNA-20` freeze write, worker ownership freeze, and launch gate acceptance

No worker starts before all three pass.

### Post-worker serialized gate

The parent must complete these in order after worker execution:

1. `RNA-60` integration gate
2. `RNA-70` proving and review gate
3. `RNA-75` promote and handoff gate
4. `RNA-80` final verification

No promotion occurs before `RNA-70` review acceptance. No completion is claimed before `RNA-80`.

## Workstream Plan

### RNA-00 Baseline

Owner: parent  
Maps to PLAN: setup for all steps

Actions:

- record branch, head sha, launch timestamp, and dirty-state summary
- record that `PLAN.md` is locally modified and treat the working copy as authoritative
- initialize `tasks.json`, `session-log.md`, and `acceptance.md`
- confirm that `.runs/**` and `.uaa-temp/**` are derived surfaces, not default deliverables

Acceptance:

- baseline recorded
- orchestration state initialized
- parent has one source of truth for frozen decisions and later proving evidence

### RNA-10 Parent Lock Command And Contract

Owner: parent  
Maps to PLAN Step 1: lock command and contract shape

Files:

- `crates/xtask/src/main.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/recommend_next_agent_research.rs`

Required outcomes:

- add and parse the new xtask subcommand
- freeze `pass1|pass2` argument rules
- freeze `--write` precondition rules
- freeze the execution packet root and allowed Codex write roots
- freeze required packet/evidence filenames
- freeze proving-run ordering
- freeze the dependency that Step 5 docs/skill work follows this contract rather than defining it

Verification:

- `cargo test -p xtask --no-run`

Acceptance:

- no later lane needs to reinterpret command semantics
- the parent can restate the full contract in `freeze.json` without ambiguity
- Step 1 is considered complete enough for downstream lanes to build against

### RNA-20 Parent Freeze And Worker Launch Gate

Owner: parent  
Maps to PLAN Step 1 completion and worker launch gate

Actions:

- write `freeze.json`
- record worker ownership boundaries
- record stale-lane triggers
- record `freeze_sha`
- create Lane C and Lane D worktrees from `freeze_sha`
- do not create Lane P unless triggered later

Launch gate acceptance:

- frozen contract is recorded
- worker ownership boundaries are recorded
- worker branches start from the same `freeze_sha`
- parent signs off that Step 1 is closed and worker-safe

### RNA-30 Core Xtask Lane

Owner: worker  
Maps to PLAN Steps 2, 3, 4, and core Step 6

Files:

- `crates/xtask/src/recommend_next_agent_research.rs`
- optional helper modules only if structurally necessary
- `crates/xtask/tests/recommend_next_agent_research_entrypoint.rs`
- `crates/xtask/tests/support/recommend_next_agent_research_harness.rs`

Execution sequence inside Lane C:

1. Implement PLAN Step 2 dry-run packet rendering.
2. Implement PLAN Step 3 discovery execution and repo-owned `freeze-discovery` handoff.
3. Implement PLAN Step 4 research execution, validation, and identity checks.
4. Implement PLAN Step 6 xtask entrypoint and harness coverage for the new seam.

Lane-local validation:

- `cargo test -p xtask --test recommend_next_agent_research_entrypoint`
- `cargo test -p xtask --test recommend_next_agent_approval_artifact`
- `cargo test -p xtask --no-run`

Acceptance:

- `--dry-run` writes a complete packet without invoking Codex
- `--write` rejects missing dry-run baselines
- discovery and research writes are bounded to the frozen roots
- repo-owned `freeze-discovery` runs between discovery and research
- research validation enforces seed and dossier identity
- Step 2, Step 3, Step 4, and core Step 6 are implementation-complete

### RNA-40 Docs / Spec / Skill Lane

Owner: worker  
Maps to PLAN Step 5

Files:

- `docs/specs/cli-agent-recommendation-dossier-contract.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `.codex/skills/recommend-next-agent/SKILL.md`

Execution sequence inside Lane D:

1. Rewrite the normative spec to describe the repo-owned host command and packet root.
2. Rewrite the operator guide to the same command order.
3. Rewrite the skill as a thin wrapper over the repo-owned flow.
4. Remove any remaining documented freehand discovery or dossier-authoring path outside the xtask host command.

Lane-local validation:

```bash
rg -n 'recommend-next-agent-research|freeze-discovery|pass1|pass2|research-runs|approved-agent.toml' \
  docs/specs/cli-agent-recommendation-dossier-contract.md \
  docs/cli-agent-onboarding-factory-operator-guide.md \
  .codex/skills/recommend-next-agent/SKILL.md
```

Acceptance:

- docs, operator guide, and skill all match the frozen command contract
- no doc surface defines behavior that is not present in Lane C or `freeze.json`
- Step 5 is complete

### RNA-50 Optional Python Gap Lane

Owner: worker  
Maps to PLAN Step 6 only if needed  
Trigger: only if `RNA-30` or `RNA-60` proves a genuine Python-side contract gap

Files:

- `scripts/test_recommend_next_agent.py`

Execution sequence inside Lane P:

1. add or adjust only the minimum downstream regression coverage needed
2. keep Python CLI shape unchanged
3. stop and escalate if runtime-behavior changes appear necessary

Lane-local validation:

- `python3 -m unittest scripts/test_recommend_next_agent.py`

Acceptance:

- either the lane is skipped or a narrow test-only adjustment lands
- Lane P does not widen product scope
- conditional Step 6 coverage is complete only if truly required

### RNA-60 Parent Integration Gate

Owner: parent  
Maps to PLAN Step 6 completion and pre-proving gate

Actions:

- merge Lane C
- decide whether Lane P is necessary
- merge Lane P if activated
- merge Lane D last so docs align to the integrated executable semantics
- rerun targeted validation on the integrated branch
- reject or relaunch worker lanes instead of patching worker-owned defects ad hoc

Integration gate verification:

- `cargo test -p xtask --test recommend_next_agent_research_entrypoint`
- `cargo test -p xtask --test recommend_next_agent_approval_artifact`
- `python3 -m unittest scripts/test_recommend_next_agent.py`
- `make check`

Acceptance:

- Step 2, Step 3, Step 4, Step 5, and Step 6 are integrated on the parent branch
- code, tests, spec, operator guide, and skill agree on one sealed contract
- no proving commands have run yet
- the parent explicitly approves the tree as proving-ready

### RNA-70 Parent Proving And Review Gate

Owner: parent  
Maps to PLAN Step 7 proving flow through deterministic evaluation and review, before promotion

Rules:

- proving is last
- proving validates the sealed contract
- proving must not be used to define missing behavior
- the parent must inspect the chosen sufficient run before promotion

Execution sequence:

1. choose explicit `PASS1_RUN_ID`
2. run `cargo run -p xtask -- recommend-next-agent-research --dry-run --pass pass1 --run-id <PASS1_RUN_ID>`
3. run `cargo run -p xtask -- recommend-next-agent-research --write --pass pass1 --run-id <PASS1_RUN_ID>`
4. run `python3 scripts/recommend_next_agent.py generate --research-dir docs/agents/.uaa-temp/recommend-next-agent/research/<PASS1_RUN_ID> --run-id <PASS1_RUN_ID> --scratch-root docs/agents/.uaa-temp/recommend-next-agent/runs`
5. inspect the resulting research tree and deterministic run output
6. if pass1 is sufficient, choose `PASS1_RUN_ID` as the candidate promotion run
7. if pass1 is insufficient, choose fresh `PASS2_RUN_ID`
8. if pass1 is insufficient, run `cargo run -p xtask -- recommend-next-agent-research --dry-run --pass pass2 --prior-run-dir docs/agents/.uaa-temp/recommend-next-agent/runs/<PASS1_RUN_ID> --run-id <PASS2_RUN_ID>`
9. if pass1 is insufficient, run `cargo run -p xtask -- recommend-next-agent-research --write --pass pass2 --prior-run-dir docs/agents/.uaa-temp/recommend-next-agent/runs/<PASS1_RUN_ID> --run-id <PASS2_RUN_ID>`
10. if pass2 runs, run `python3 scripts/recommend_next_agent.py generate --research-dir docs/agents/.uaa-temp/recommend-next-agent/research/<PASS2_RUN_ID> --run-id <PASS2_RUN_ID> --scratch-root docs/agents/.uaa-temp/recommend-next-agent/runs`
11. inspect the pass2 research tree and deterministic run output
12. choose exactly one sufficient run id or halt

Parent review gate before promotion:

- inspect the chosen run’s `run-status.json`
- inspect the chosen run’s deterministic comparison/selection output
- confirm the associated research tree is contract-valid and acceptable for promotion
- confirm no contract defect surfaced during the real run
- confirm the run is sufficient, not merely syntactically valid

Hard stops:

- if pass1 and pass2 both fail to produce a sufficient run, halt the milestone and do not promote
- if the proving flow reveals a contract defect, halt, reopen freeze, invalidate downstream lanes, and return to implementation
- if the run is valid but not acceptable for promotion, halt rather than forcing `promote`

Acceptance:

- at least one sufficient run exists and is explicitly accepted by the parent for promotion
- or the milestone is explicitly halted with no partial success claimed
- Step 7 proving-and-review is complete only when the parent records a chosen run id or a hard stop

### RNA-75 Parent Promote And Handoff Gate

Owner: parent  
Maps to PLAN Step 7 promotion and handoff completion

Actions:

- run `python3 scripts/recommend_next_agent.py promote --run-dir docs/agents/.uaa-temp/recommend-next-agent/runs/<CHOSEN_RUN_ID> --repo-run-root docs/agents/selection/runs --approved-agent-id <agent_id> --onboarding-pack-prefix <prefix>`
- verify the committed recommendation run output under `docs/agents/selection/runs/<CHOSEN_RUN_ID>/`
- verify `docs/agents/lifecycle/<prefix>/governance/approved-agent.toml`
- run `cargo run -p xtask -- onboard-agent --approval docs/agents/lifecycle/<prefix>/governance/approved-agent.toml --dry-run`

Acceptance:

- exactly one chosen sufficient run is promoted
- the canonical promoted output exists
- the approved-agent handoff exists
- no `.runs/**` or `.uaa-temp/**` surfaces are treated as committed deliverables

### RNA-80 Final Verification

Owner: parent  
Maps to PLAN final acceptance closeout

Required command matrix:

- `cargo test -p xtask --test recommend_next_agent_research_entrypoint`
- `cargo test -p xtask --test recommend_next_agent_approval_artifact`
- `python3 -m unittest scripts/test_recommend_next_agent.py`
- `make test`
- `make preflight`

Failure handling:

- if any command fails, write `blocked.json` for `RNA-80`
- stop
- do not weaken acceptance or backfill success through docs-only updates

Acceptance:

- all milestone acceptance gates pass on the integrated tree
- proving outputs are present and scoped correctly
- the parent records completion only after this gate passes

## Tests And Acceptance

### Command / Acceptance Matrix

| Surface | Command or Evidence | Expected acceptance |
| --- | --- | --- |
| New host command dry-run | `cargo run -p xtask -- recommend-next-agent-research --dry-run --pass pass1 --run-id <id>` | Writes a complete execution packet and does not invoke Codex |
| New host command write | `cargo run -p xtask -- recommend-next-agent-research --write --pass pass1 --run-id <id>` | Requires matching dry-run packet, runs bounded discovery and research, fails closed on contract violations |
| Repo-owned freeze handoff | packet evidence plus write-mode test coverage | `freeze-discovery` is invoked by xtask between discovery and research, not by operator choreography |
| Pass2 support | `cargo run -p xtask -- recommend-next-agent-research --dry-run\|--write --pass pass2 --prior-run-dir <run_dir> --run-id <fresh_id>` | Requires prior insufficiency input, uses a fresh `run_id`, does not mutate pass1 artifacts |
| Generate CLI stability | `python3 scripts/recommend_next_agent.py generate --research-dir ... --run-id ... --scratch-root ...` | CLI shape unchanged and works against the new research tree |
| Promote CLI stability | `python3 scripts/recommend_next_agent.py promote --run-dir ... --repo-run-root docs/agents/selection/runs --approved-agent-id <agent_id> --onboarding-pack-prefix <prefix>` | CLI shape unchanged and still renders promoted output plus final approval artifact |
| Skill/operator alignment | `docs/specs/cli-agent-recommendation-dossier-contract.md`, `docs/cli-agent-onboarding-factory-operator-guide.md`, `.codex/skills/recommend-next-agent/SKILL.md` | All describe the same repo-owned procedure and none instruct freehand discovery/research outside xtask |
| Proving review gate | parent review notes in `session-log.md` and chosen run evidence | Parent inspects the sufficient run and explicitly approves it before `promote` |
| Proving output and handoff | promoted run under `docs/agents/selection/runs/<run_id>/` and `docs/agents/lifecycle/<prefix>/governance/approved-agent.toml` | One proving run created through the new host surface produces promoted output and approved-agent handoff |
| Derived-surface discipline | git diff / parent verification | No committed `.runs/**` or `.uaa-temp/**` surfaces except intentionally promoted outputs outside those derived roots |

### Completion Checklist

- `PLAN.md` Step 1 is frozen before worker launch.
- `PLAN.md` Step 2 is implemented in Lane C and integrated.
- `PLAN.md` Step 3 is implemented in Lane C and integrated.
- `PLAN.md` Step 4 is implemented in Lane C and integrated.
- `PLAN.md` Step 5 is implemented in Lane D and integrated.
- `PLAN.md` Step 6 is complete before any proving commands run.
- `PLAN.md` Step 7 runs only after the integrated tree is proving-ready.
- The parent inspects the chosen sufficient run before `promote`.
- If both pass1 and pass2 are insufficient, the milestone halts with no promotion.
- If the proving flow exposes a contract defect, freeze is reopened and downstream lanes are invalidated.
- `generate` and `promote` CLI shapes remain unchanged.
- `freeze-discovery` is repo-owned and invoked from xtask.
- `pass2` support is present and requires prior insufficiency input.
- docs/spec/skill surfaces align with executable behavior.
- one proving run produces promoted output and the approved-agent handoff.
- no committed `.runs/**` or `docs/agents/.uaa-temp/**` surfaces are treated as deliverables.

## Success Metric

The milestone is complete only when the repo owns the AI research host surface end to end and the parent has proven that ownership through one reviewed, promoted recommendation run without widening behavior beyond `PLAN.md`.
