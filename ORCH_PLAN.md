# UAA-0022 Orchestration Plan

## Summary

- Execute against the current implementation branch `codex/recommend-next-agent`. Treat `main` only as the review base branch, not the worker fork point.
- Plan authority is `/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/PLAN.md`. If `PLAN.md`, current code, and older notes disagree, `PLAN.md` wins for this run unless it conflicts with `docs/specs/**`; spec conflict is a stop-and-escalate condition.
- Keep the critical path local to the parent agent for:
  - kickoff and baseline capture
  - contract/schema freeze
  - all edits in the runtime-follow-on module cluster
  - final integration
  - final verification
- Use dedicated worktrees under `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/unified-agent-api-uaa-0022/{int,code,tests,docs}` with branches:
  - `codex/uaa-0022-int`
  - `codex/uaa-0022-code`
  - `codex/uaa-0022-tests`
  - `codex/uaa-0022-docs`
- Worker model for delegated lanes:
  - model: GPT-5.4
  - reasoning: high
- Cap safe concurrency at 2 workers after the freeze commit.
  - Justification: `runtime_follow_on.rs`, `models.rs`, `render.rs`, and the prompt template share one validator vocabulary and one artifact contract; splitting that cluster earlier raises merge risk more than it buys speed.
- Keep one canonical orchestration state root owned only by the parent agent:
  - `REPO_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api`
  - `WORKTREE_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/unified-agent-api-uaa-0022`
  - `ORCH_RUN_ROOT=$REPO_ROOT/.runs/uaa-0022-runtime-follow-on`
  - `RUNTIME_RUNS_ROOT=$REPO_ROOT/docs/agents/.uaa-temp/runtime-follow-on/runs`
- Treat `$ORCH_RUN_ROOT/*` as orchestration state only, and `$RUNTIME_RUNS_ROOT/*` as generated validation evidence only.
- The only authored source-of-truth surfaces for this run are:
  - `crates/xtask/src/runtime_follow_on.rs`
  - `crates/xtask/src/runtime_follow_on/models.rs`
  - `crates/xtask/src/runtime_follow_on/render.rs`
  - `crates/xtask/templates/runtime_follow_on_codex_prompt.md`
  - `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
  - `crates/xtask/tests/fixtures/fake_codex.sh`
  - `docs/cli-agent-onboarding-factory-operator-guide.md`

## Approval Gates

- UAA-0022 has zero human approval gates.
- There is no planned maintainer approval pause between kickoff and closeout.
- The only intentional pauses in this run are:
  - hard stop conditions
  - worker bounce-backs
  - explicit contract-lane reopen and restart handling

## Worker Model

- Parent agent is the only integrator, rebaser, and verifier.
- Workers may edit only their assigned surfaces and must return:
  - changed files
  - commands run
  - exit codes
  - blockers or assumptions
- Workers do not merge branches, do not write orchestration state, and do not hand-author generated runtime-follow-on run artifacts.
- Any worker that needs a change in a parent-owned file must stop and bounce the request back instead of widening scope.

## Hard Guards

- Scope is locked to `PLAN.md` intent:
  - keep one runtime lane
  - keep one handoff artifact
  - keep one typed summary model
  - do not add a new crate
  - do not add a new `xtask` command
  - do not add a second machine-readable artifact family
- Module ownership facts are locked:
  - `InputContract`, `RunStatus`, and `HandoffContract` live in `models.rs`
  - dry-run preparation and write validation live in `runtime_follow_on.rs`
  - prompt rendering and markdown/status rendering live in `render.rs`
- The required publication command set is fixed and must remain exactly:
  - `support-matrix --check`
  - `capability-matrix --check`
  - `capability-matrix-audit`
  - `make preflight`
- `handoff.json` remains the only machine-readable handoff artifact.
  - No `implementation-summary.json`
  - No new sidecar schema
  - No new queue artifact family
- Publication refresh stays a later lane.
  - This run may strengthen `publication_refresh_ready`
  - This run may not implement publication refresh
  - This run may not edit publication-owned manifest outputs as if refresh already happened
- Generated runtime-follow-on run artifacts under `docs/agents/.uaa-temp/runtime-follow-on/runs/<run_id>/` are evidence, not source.
  - Inspect them during verification
  - Do not hand-edit them
  - Do not treat them as canonical authored inputs
- Do not revert concurrent user or maintainer edits.
  - If a worktree encounters unrelated dirt outside its ownership, ignore it
  - If a worktree encounters conflicting edits inside its ownership that change semantics after freeze, stop and escalate to the parent
- Stop conditions for the full run:
  - `PLAN.md` authority is unclear or contradicted by `docs/specs/**`
  - the fixed publication command set in code or docs drifts from the plan
  - a worker needs to touch a non-owned file to finish
  - `make clippy` or test failures require changes outside the locked touch set
  - runtime-follow-on generated artifacts imply a second handoff artifact or widened lane ownership

## Orchestration State

Canonical parent-owned files under `$ORCH_RUN_ROOT`:

- `baseline.json`
  - captured before any product edits
  - stores current branch, current sha, base branch, plan authority checks, and target touch-set confirmation
- `tasks.json`
  - one record per task
  - includes owner, branch, worktree, status, and sentinel path
- `session-log.md`
  - terse timeline of launches, freezes, merges, retries, and stop decisions
- `contract-freeze.json`
  - written only after the parent freezes the runtime-follow-on contract
  - stores the freeze commit sha, owned enum/field vocabulary, and required publication commands
- `merge-log.md`
  - ordered merge history into integration
- `acceptance.md`
  - final checklist and command outcomes

Per-task sentinels under `$REPO_ROOT/.runs/<TASK_ID>/`:

- required minimum files:
  - `started.json`
  - `status.json`
  - `done.json` or `blocked.json`
- workers write only their own sentinel directory
- parent consumes sentinels instead of polling chat transcripts

Generated validation evidence to inspect but not author:

- `$RUNTIME_RUNS_ROOT/<run_id>/input-contract.json`
- `$RUNTIME_RUNS_ROOT/<run_id>/handoff.json`
- `$RUNTIME_RUNS_ROOT/<run_id>/run-status.json`
- `$RUNTIME_RUNS_ROOT/<run_id>/run-summary.md`
- `$RUNTIME_RUNS_ROOT/<run_id>/validation-report.json`
- `$RUNTIME_RUNS_ROOT/<run_id>/written-paths.json`

## Worktree Plan

Branches and worktrees:

- integration:
  - branch: `codex/uaa-0022-int`
  - worktree: `$WORKTREE_ROOT/int`
- parent code lane:
  - branch: `codex/uaa-0022-code`
  - worktree: `$WORKTREE_ROOT/code`
- test worker:
  - branch: `codex/uaa-0022-tests`
  - worktree: `$WORKTREE_ROOT/tests`
- docs worker:
  - branch: `codex/uaa-0022-docs`
  - worktree: `$WORKTREE_ROOT/docs`

Creation commands from `$REPO_ROOT`:

```bash
mkdir -p "$WORKTREE_ROOT" "$ORCH_RUN_ROOT"
BASE_BRANCH=$(git rev-parse --abbrev-ref HEAD)
BASE_SHA=$(git rev-parse HEAD)

git worktree add -b codex/uaa-0022-int "$WORKTREE_ROOT/int" "$BASE_SHA"
git worktree add -b codex/uaa-0022-code "$WORKTREE_ROOT/code" "$BASE_SHA"
```

Create worker worktrees only after `contract-freeze.json` exists:

```bash
FREEZE_SHA=$(jq -r '.freeze_commit' "$ORCH_RUN_ROOT/contract-freeze.json")
git worktree add -b codex/uaa-0022-tests "$WORKTREE_ROOT/tests" "$FREEZE_SHA"
git worktree add -b codex/uaa-0022-docs "$WORKTREE_ROOT/docs" "$FREEZE_SHA"
```

Worktree rules:

- never reuse a dirty worktree
- never let workers branch from anything earlier than `freeze_commit`
- never let workers merge back to integration directly
- if `freeze_commit` changes because the parent reopens the contract lane, destroy and recreate worker worktrees from the new freeze commit

## Restart And Reopen Rule

- If the parent changes the code-freeze lane after either worker has launched, the contract lane has reopened.
- Reopen handling is mandatory:
  - close and discard the active `codex/uaa-0022-tests` and `codex/uaa-0022-docs` branches/worktrees
  - invalidate the prior worker sentinels for:
    - `$REPO_ROOT/.runs/task-uaa-0022-b-tests/`
    - `$REPO_ROOT/.runs/task-uaa-0022-c-docs/`
  - write stale or blocked terminal status into those sentinel roots
  - regenerate `contract-freeze.json` from the new freeze commit
  - recreate both worker worktrees from the new `freeze_commit`
  - relaunch both workers only from that new `freeze_commit`
- Never merge stale worker output after a freeze change.
  - do not hand-merge it
  - do not cherry-pick it
  - do not salvage partial hunks from it into integration

## Merge Policy

- Integration always merges from lane heads.
  - merge `codex/uaa-0022-code`, `codex/uaa-0022-tests`, and `codex/uaa-0022-docs` into `codex/uaa-0022-int`
  - do not cherry-pick unless a worker has already been discarded as stale and the parent is applying a replacement locally
- Workers never rebase themselves after launch.
  - only the parent decides whether a worker remains mergeable or must be recreated
- If the parent patches `task/uaa-0022-a-code-freeze` after tests/docs workers have launched, those workers are stale immediately.
  - discard them under the restart rule
  - do not attempt a hand-merge against the patched freeze

## Task Graph

Critical path:

1. `task/uaa-0022-00-baseline`
2. `task/uaa-0022-a-code-freeze`
3. launch in parallel:
  - `task/uaa-0022-b-tests`
  - `task/uaa-0022-c-docs`
4. `task/uaa-0022-d-integrate`

Parallel-safe tasks:

- `task/uaa-0022-b-tests`
- `task/uaa-0022-c-docs`

Deliberately serialized tasks:

- `task/uaa-0022-00-baseline`
- `task/uaa-0022-a-code-freeze`
- `task/uaa-0022-d-integrate`

## Workstream Plan

### WS-BASELINE

#### `task/uaa-0022-00-baseline` — parent agent only

Owned files:

- `$ORCH_RUN_ROOT/baseline.json`
- `$ORCH_RUN_ROOT/tasks.json`
- `$ORCH_RUN_ROOT/session-log.md`
- `$REPO_ROOT/.runs/task-uaa-0022-00-baseline/*`

Forbidden files:

- all product source files

Required commands:

```bash
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git status --short
test -f PLAN.md
test -f crates/xtask/src/runtime_follow_on.rs
test -f crates/xtask/src/runtime_follow_on/models.rs
test -f crates/xtask/src/runtime_follow_on/render.rs
test -f crates/xtask/templates/runtime_follow_on_codex_prompt.md
test -f crates/xtask/tests/runtime_follow_on_entrypoint.rs
test -f crates/xtask/tests/fixtures/fake_codex.sh
test -f docs/cli-agent-onboarding-factory-operator-guide.md
rg -n "struct InputContract|struct RunStatus|struct HandoffContract" crates/xtask/src/runtime_follow_on/models.rs
rg -n "fn validate_write_mode|fn validate_handoff" crates/xtask/src/runtime_follow_on.rs
rg -n "render_prompt|render_run_summary|render_run_status" crates/xtask/src/runtime_follow_on/render.rs
```

Acceptance:

- current branch is `codex/recommend-next-agent`
- base review branch remains `main`
- `PLAN.md` is present and is the run authority
- the named runtime-follow-on ownership facts still match the repo
- the locked touch set still matches the plan scope
- any unrelated dirty files are recorded, not reverted

### WS-CODE

#### `task/uaa-0022-a-code-freeze` — parent agent only

Owned files:

- `crates/xtask/src/runtime_follow_on.rs`
- `crates/xtask/src/runtime_follow_on/models.rs`
- `crates/xtask/src/runtime_follow_on/render.rs`
- `crates/xtask/templates/runtime_follow_on_codex_prompt.md`
- `$ORCH_RUN_ROOT/contract-freeze.json`
- `$ORCH_RUN_ROOT/session-log.md`
- `$REPO_ROOT/.runs/task-uaa-0022-a-code-freeze/*`

Forbidden files:

- `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
- `crates/xtask/tests/fixtures/fake_codex.sh`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- any new crate or new artifact-family path

Required work:

1. Freeze the typed summary schema in `models.rs`.
  - add the enums and structs required by `PLAN.md`
  - extend `InputContract`, `RunStatus`, and `HandoffContract`
2. Freeze the validator vocabulary in `runtime_follow_on.rs`.
  - seed `expected_default_surfaces`
  - seed `known_template_ids`
  - seed `known_rich_surfaces`
  - strengthen dry-run placeholder emission
  - enforce the semantic handoff rules
3. Freeze the render and prompt contract.
  - make `run-summary.md` a projection of validated structured data
  - mirror only the approved status fields into `run-status.json`
  - expose the exact enum vocabulary and tier/surface rules in the prompt template
4. Keep the implementation inside the existing module cluster.
  - no new command
  - no new artifact family
  - no second validator surface

Required commands:

```bash
cargo test -p xtask runtime_follow_on
cargo test -p xtask --test runtime_follow_on_entrypoint --no-run
```

Acceptance:

- one typed implementation summary model exists and is the only schema expansion
- `handoff.json` remains the sole machine-readable handoff artifact
- the fixed publication command set is encoded and normalized exactly once
- dry-run placeholder output remains parseable but intentionally not write-success-valid
- `validate_handoff` owns semantic enforcement
- `render_run_summary` and `render_run_status` consume validated data, not re-parsed prose
- prompt contract, model contract, and validator contract use the same vocabulary
- `contract-freeze.json` records:
  - `freeze_commit`
  - `required_publication_commands`
  - `expected_default_surfaces`
  - `known_rich_surfaces`
  - `known_template_ids`

Freeze rule:

- do not launch worker lanes until `task/uaa-0022-a-code-freeze` is merged into `codex/uaa-0022-int` and `contract-freeze.json` is written from that exact merged commit

### WS-TESTS

#### `task/uaa-0022-b-tests` — worker 1

Owned files:

- `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
- `crates/xtask/tests/fixtures/fake_codex.sh`

Forbidden files:

- `crates/xtask/src/runtime_follow_on.rs`
- `crates/xtask/src/runtime_follow_on/models.rs`
- `crates/xtask/src/runtime_follow_on/render.rs`
- `crates/xtask/templates/runtime_follow_on_codex_prompt.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `$ORCH_RUN_ROOT/*`

Required work:

1. Extend the fake Codex fixture to emit:
  - the valid richer success path
  - dry-run placeholder behavior
  - explicit invalid handoff scenarios driven by the frozen vocabulary
2. Lock the required success and failure semantics in entrypoint tests.
3. Keep the harness aligned to the parent-owned contract exactly as frozen.

Required commands:

```bash
cargo test -p xtask --test runtime_follow_on_entrypoint
```

Acceptance:

- success path proves a rich `handoff.json`, `run-status.json`, and `run-summary.md`
- dry-run path proves placeholder `implementation_summary`
- regression coverage exists for:
  - missing implementation summary
  - publication-ready with blockers
  - tier mismatch
  - minimal without justification
  - required-command-set mismatch
  - duplicate surface entries
  - unapproved rich surface
  - unaccounted requested rich surface
  - status projection from validated summary
  - rendered markdown semantic sections
- no product-code file changes are made in this lane

Bounce-back rules:

- if the worker needs a product-code change to make a test coherent, stop and return:
  - the failing test name
  - the expected contract
  - the required parent-owned file

### WS-DOCS

#### `task/uaa-0022-c-docs` — worker 2

Owned files:

- `docs/cli-agent-onboarding-factory-operator-guide.md`

Forbidden files:

- all `crates/xtask/src/runtime_follow_on/*`
- `crates/xtask/templates/runtime_follow_on_codex_prompt.md`
- `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
- `crates/xtask/tests/fixtures/fake_codex.sh`
- `$ORCH_RUN_ROOT/*`

Required work:

1. Update the operator guide to describe the stronger runtime-follow-on handoff.
2. Document:
  - `implementation_summary`
  - `publication_refresh_ready`
  - the same handoff artifact continuing into the later publication lane
  - runtime scratch artifacts as generated evidence
3. Keep the procedure aligned to the frozen vocabulary and command set.

Suggested review commands:

```bash
rg -n "runtime-follow-on|publication_refresh_ready|handoff.json" docs/cli-agent-onboarding-factory-operator-guide.md
```

Acceptance:

- operator guide matches the shipped schema and validation semantics
- guide does not introduce a new command or a second handoff artifact
- guide states that publication refresh is still a separate later lane
- guide does not promote generated scratch artifacts into source-of-truth status

Bounce-back rules:

- if the worker concludes the frozen field names or semantics are unclear, stop and return the exact ambiguous phrase instead of editorially inventing procedure

### WS-INT

#### `task/uaa-0022-d-integrate` — parent agent only

Owned files:

- the full locked touch set
- `$ORCH_RUN_ROOT/merge-log.md`
- `$ORCH_RUN_ROOT/acceptance.md`
- `$ORCH_RUN_ROOT/session-log.md`
- `$REPO_ROOT/.runs/task-uaa-0022-d-integrate/*`

Required integration sequence:

1. Merge `codex/uaa-0022-code` into `codex/uaa-0022-int`.
2. Write `contract-freeze.json` from the integration commit if it does not already reflect the merged tree.
3. Create `codex/uaa-0022-tests` and `codex/uaa-0022-docs` from `freeze_commit`.
4. Merge worker lanes one at a time into integration.
  - merge tests first
  - merge docs second
  - merge from branch heads only
  - do not cherry-pick active worker commits into integration
5. Resolve only mechanical conflicts locally.
  - if a worker changed a forbidden file, reject the lane and bounce it back
  - if a worker’s assumptions contradict `contract-freeze.json`, reject the lane and bounce it back
6. Run the final verification loop from the integration worktree.

Exact verification loop:

```bash
cargo test -p xtask --test runtime_follow_on_entrypoint
cargo test -p xtask runtime_follow_on
make fmt-check
make clippy
```

Concrete runtime-artifact verification substep:

```bash
cargo run -p xtask -- runtime-follow-on \
  --dry-run \
  --approval docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml \
  --run-id uaa-0022-orch-verify-dry-run

test -f "$RUNTIME_RUNS_ROOT/uaa-0022-orch-verify-dry-run/input-contract.json"
test -f "$RUNTIME_RUNS_ROOT/uaa-0022-orch-verify-dry-run/handoff.json"
test -f "$RUNTIME_RUNS_ROOT/uaa-0022-orch-verify-dry-run/run-status.json"
test -f "$RUNTIME_RUNS_ROOT/uaa-0022-orch-verify-dry-run/run-summary.md"
test -f "$RUNTIME_RUNS_ROOT/uaa-0022-orch-verify-dry-run/validation-report.json"
```

Inspect exactly these artifact paths:

- `$RUNTIME_RUNS_ROOT/uaa-0022-orch-verify-dry-run/input-contract.json`
- `$RUNTIME_RUNS_ROOT/uaa-0022-orch-verify-dry-run/handoff.json`
- `$RUNTIME_RUNS_ROOT/uaa-0022-orch-verify-dry-run/run-status.json`
- `$RUNTIME_RUNS_ROOT/uaa-0022-orch-verify-dry-run/run-summary.md`
- `$RUNTIME_RUNS_ROOT/uaa-0022-orch-verify-dry-run/validation-report.json`

Verification loop rules:

- run commands in the listed order
- if a command fails because of this plan’s touched surfaces, patch only inside the locked touch set and rerun from the first failed command
- if a command fails because of unrelated pre-existing repo state outside the touch set, record the failure in `acceptance.md` and stop instead of widening scope
- use the deterministic dry-run verification packet above for manual artifact inspection because it is repo-owned and non-mutating
- rely on `cargo test -p xtask --test runtime_follow_on_entrypoint` for write-mode success and failure artifact semantics
- do not invent an ad hoc write-mode closeout run against the live repo

Acceptance:

- integration branch contains only the locked touch set plus orchestration-state artifacts
- worker branches were forked from `freeze_commit`
- all final verification commands pass
- generated `handoff.json`, `run-status.json`, and `run-summary.md` reflect the strengthened contract without any second machine-readable artifact
- publication refresh remains represented only as downstream readiness data, not as implemented refresh behavior
- closeout orchestration state exists under `$ORCH_RUN_ROOT`:
  - `baseline.json`
  - `tasks.json`
  - `session-log.md`
  - `contract-freeze.json`
  - `merge-log.md`
  - `acceptance.md`

Stop conditions:

- merge conflict requires inventing new semantics not present in `PLAN.md` or `contract-freeze.json`
- `make clippy` requires touching files outside the locked touch set
- runtime-follow-on tests show artifact behavior that contradicts the frozen command set or tier/surface rules

## Context-Control Rules

- Parent agent keeps only these materials live in working context:
  - `PLAN.md`
  - `ORCH_PLAN.md`
  - `$ORCH_RUN_ROOT/tasks.json`
  - `$ORCH_RUN_ROOT/contract-freeze.json` once it exists
  - the latest integration diff summary
- Worker prompts contain only:
  - owned file list
  - forbidden file list
  - the exact relevant excerpt from `PLAN.md`
  - the frozen vocabulary from `contract-freeze.json`
  - required commands
  - bounce-back rules
- Workers return narrow summaries only.
  - no full transcript ingestion into parent context
  - no generated artifact dumps unless a failure requires a specific excerpt
- The parent records decisions in `$ORCH_RUN_ROOT/session-log.md`, not in ad hoc scratch files across worktrees.
- Close worker lanes immediately after merge or rejection.
- Use sentinel files and git state for progress tracking rather than repeated manual inspection of worker trees.

## Tests And Acceptance

- Parent code-freeze gate:
  - `cargo test -p xtask runtime_follow_on`
  - `cargo test -p xtask --test runtime_follow_on_entrypoint --no-run`
- Worker gate:
  - `cargo test -p xtask --test runtime_follow_on_entrypoint`
- Final integration gate:
  - `cargo test -p xtask --test runtime_follow_on_entrypoint`
  - `cargo test -p xtask runtime_follow_on`
  - `make fmt-check`
  - `make clippy`

Final acceptance checklist:

- `models.rs` is the single source of typed summary vocabulary and the mirrored status fields
- `runtime_follow_on.rs` owns dry-run placeholder emission, write validation, and semantic handoff checks
- `render.rs` renders deterministic operator-facing markdown and status fields from validated data
- `runtime_follow_on_codex_prompt.md` tells Codex to write the richer contract directly and forbids unauthorized rich surfaces
- `runtime_follow_on_entrypoint.rs` covers the success path and the enumerated failure modes from `PLAN.md`
- `fake_codex.sh` can emit the valid richer handoff and the invalid scenarios needed by the regression suite
- `docs/cli-agent-onboarding-factory-operator-guide.md` matches the shipped contract and keeps publication refresh in the later lane
- generated runtime-follow-on run artifacts are inspected as evidence only and are not treated as authored source
- `$ORCH_RUN_ROOT` contains:
  - `baseline.json`
  - `tasks.json`
  - `session-log.md`
  - `contract-freeze.json`
  - `merge-log.md`
  - `acceptance.md`

## Assumptions

- The current branch `codex/recommend-next-agent` is the correct live baseline for the work, and `main` remains the review base branch.
- The runtime-follow-on module cluster remains small enough that parent-owned sequential delivery is faster and safer than splitting production code across workers.
- The repo’s existing test surfaces are sufficient; no new test harness command is needed beyond the commands already named in `PLAN.md`.
- `docs/specs/**` remains normative. If a normative spec conflicts with `PLAN.md` during execution, the parent stops and resolves authority before continuing.
- Worktree creation under `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/unified-agent-api-uaa-0022` is available and does not need a different repo-local convention.
