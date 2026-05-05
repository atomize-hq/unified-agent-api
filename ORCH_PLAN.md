# ORCH_PLAN - Maintenance CI Registry-Driven Revamp

## Summary

This orchestration plan executes the live maintenance-CI milestone defined by `PLAN.md` on branch
`codex/recommend-next-agent` in
`/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api`.

This orchestration ends at maintenance-CI completion. It does not continue into goose execution,
goose proving, or any post-maintenance follow-on from `TODOS.md`. Goose remains a later milestone
gated on this one landing cleanly.

Frozen parent critical path:

```text
MCI-00 Baseline Capture
-> MCI-05 Scope Freeze From PLAN.md
-> MCI-10 Interface And Ownership Freeze
-> MCI-15 Worktree / Worker Launch Freeze
-> parallel worker phase
-> MCI-60 Parent Spine Integration
-> MCI-70 Parent Workflow/Worker Integration
-> MCI-80 Parent Docs And CI Contract Closeout
-> MCI-90 Final Proving And Acceptance
```

Milestone done state:

- `crates/xtask/data/agent_registry.toml` owns maintenance watch enrollment
- `maintenance-watch` exists and emits the stale-agent queue from repo truth
- `prepare-agent-maintenance` exists and writes automated maintenance request v2 plus packet docs
- one shared watcher workflow replaces the two legacy scheduled watcher workflows
- `codex` and `claude_code` worker workflows become shared-input worker-only entrypoints
- generic `packet_pr` support lands but no additional agents are enabled in milestone 1
- all explicit tests and gates from `PLAN.md` pass locally

## Worker Model

All workers use `GPT-5.4` with `reasoning_effort=high`.

Concurrency cap:

- Maximum concurrent workers: `4`
- Recommended live cap for this milestone: `4`
- Parent remains local and does not count toward the worker cap

Authority model:

- Parent agent is the only integrator
- Parent agent is the only merge authority
- Parent agent is the only rebase/relaunch authority
- Parent agent is the only final prover
- Parent agent is the only owner of orchestration state

Worker rules:

- Workers operate only on their frozen owned surfaces
- Workers do not merge
- Workers do not update orchestration state
- Workers do not widen scope
- Workers return diffs, commands run, exit codes, blockers, and unresolved assumptions only

## Hard Guards

- `PLAN.md` is the only authoritative plan for this milestone. The existing `ORCH_PLAN.md` is
  stale and is input only for stale-assumption rejection.
- This orchestration ends at maintenance-CI completion. Do not continue into goose execution,
  `scaffold-wrapper-crate`, `runtime-follow-on`, `prepare-publication`, `refresh-publication`,
  `prepare-proving-run-closeout`, or `close-proving-run`.
- Do not reintroduce stale goals from the current `ORCH_PLAN.md`, especially:
  - no `PLAN.md` replacement step
  - no promote-workflow edits
  - no goose lane
- Parent-only merge authority stays on `codex/recommend-next-agent`.
- Worker branches fork from `codex/recommend-next-agent`.
- Workflow product behavior remains frozen to `staging`:
  - shared watcher runs against `staging`
  - downstream workflow dispatches use `ref: staging`
  - generated maintenance PRs target base `staging`
- Milestone-1 watch enrollment is frozen:
  - `codex` enabled
  - `claude_code` enabled
  - all other agents not enabled
- Upstream source kinds are frozen:
  - `github_releases`
  - `gcs_object_listing`
- Request-contract freeze:
  - `artifact_version = "2"`
  - `trigger_kind = "upstream_release_detected"`
  - `[detected_release]` required for automated watch requests
- Workflow-topology freeze:
  - add `.github/workflows/agent-maintenance-release-watch.yml`
  - add `.github/workflows/agent-maintenance-open-pr.yml`
  - delete `.github/workflows/codex-cli-release-watch.yml`
  - delete `.github/workflows/claude-code-release-watch.yml`
  - retain and migrate `.github/workflows/codex-cli-update-snapshot.yml`
  - retain and migrate `.github/workflows/claude-code-update-snapshot.yml`
- Do not edit:
  - `.github/workflows/codex-cli-promote.yml`
  - `.github/workflows/claude-code-promote.yml`
  - `TODOS.md`
- Any change to frozen queue fields, request-v2 fields, workflow inputs, or branch rules
  invalidates affected worker lanes and requires relaunch from the new freeze SHA.

## Orchestration State

Parent-owned orchestration root:

```text
/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.runs/maintenance-ci-revamp
```

Parent-owned state files:

- `baseline.json`
- `freeze.json`
- `queue.json`
- `session-log.md`
- `acceptance.md`
- `tasks.json`

Required contents:

- `baseline.json`
  - branch
  - head sha
  - dirty-state summary
  - timestamp
  - workflow inventory
  - `PLAN.md` hash
  - stale-ORCH rejection notes
- `freeze.json`
  - frozen scope summary
  - exact command spellings
  - frozen queue schema
  - frozen request-v2 schema additions
  - frozen workflow filenames
  - frozen branch and PR rules
  - ownership split
  - stale-lane invalidation rules
- `queue.json`
  - ordered task queue
  - launch readiness per task
  - dependency satisfaction
  - active worker count
- `session-log.md`
  - parent decisions
  - worker launches
  - stale-lane invalidations
  - merge outcomes
  - proving notes
- `acceptance.md`
  - final checklist by area
  - command results
  - manual topology verification
- `tasks.json`
  - task id
  - title
  - owner
  - branch
  - worktree
  - launch sha
  - status
  - restart count
  - dependency ids

Per-task sentinel directories:

```text
.runs/maintenance-ci-revamp/task-mci-00-baseline/
.runs/maintenance-ci-revamp/task-mci-05-scope-freeze/
.runs/maintenance-ci-revamp/task-mci-10-interface-freeze/
.runs/maintenance-ci-revamp/task-mci-15-launch-freeze/
.runs/maintenance-ci-revamp/task-mci-20-registry-contract/
.runs/maintenance-ci-revamp/task-mci-30-watch-surface/
.runs/maintenance-ci-revamp/task-mci-40-packet-surface/
.runs/maintenance-ci-revamp/task-mci-50-shared-workflows/
.runs/maintenance-ci-revamp/task-mci-60-parent-spine-integration/
.runs/maintenance-ci-revamp/task-mci-70-worker-migrations/
.runs/maintenance-ci-revamp/task-mci-80-docs-ci-closeout/
.runs/maintenance-ci-revamp/task-mci-90-final-proving/
```

Each task directory uses parent-written markers only:

- `started.json`
- `done.json`
- `blocked.json`
- optional `skipped.json`

Workers never write orchestration state, markers, queue files, or logs.

## Branch And Worktree Layout

Repository root:

```text
/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api
```

Parent branch:

```text
codex/recommend-next-agent
```

Worker worktree root:

```text
/Users/spensermcconnell/__Active_Code/atomize-hq/wt
```

Frozen worker branches and worktrees:

| Task | Branch | Worktree |
| --- | --- | --- |
| `MCI-20` | `codex/recommend-next-agent-mci-20-registry` | `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-mci-20-registry` |
| `MCI-30` | `codex/recommend-next-agent-mci-30-watch` | `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-mci-30-watch` |
| `MCI-40` | `codex/recommend-next-agent-mci-40-packet` | `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-mci-40-packet` |
| `MCI-50` | `codex/recommend-next-agent-mci-50-shared-workflows` | `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-mci-50-shared-workflows` |
| `MCI-71` | `codex/recommend-next-agent-mci-71-codex-worker` | `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-mci-71-codex-worker` |
| `MCI-72` | `codex/recommend-next-agent-mci-72-claude-worker` | `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-mci-72-claude-worker` |

Creation pattern:

```sh
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-mci-20-registry -b codex/recommend-next-agent-mci-20-registry codex/recommend-next-agent
```

Use the same pattern for the remaining worker branches.

## Parent Vs Worker Ownership

### Parent-only surfaces

- `PLAN.md`
- `ORCH_PLAN.md`
- `TODOS.md`
- `.runs/maintenance-ci-revamp/**`
- `crates/xtask/src/agent_maintenance/mod.rs`
- `crates/xtask/src/main.rs`
- final integration edits in `crates/xtask/tests/c4_spec_ci_wiring.rs`
- final docs truth pass in `docs/cli-agent-onboarding-factory-operator-guide.md`

### Worker-owned surfaces

#### `MCI-20` Registry Contract

Owns:

- `crates/xtask/data/agent_registry.toml`
- `crates/xtask/src/agent_registry.rs`
- `docs/specs/agent-registry-contract.md`
- `crates/xtask/tests/agent_registry.rs`

Forbidden:

- `crates/xtask/src/agent_maintenance/**`
- `.github/workflows/**`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

#### `MCI-30` Watch Surface

Owns:

- `crates/xtask/src/agent_maintenance/watch.rs`
- `crates/xtask/tests/agent_maintenance_watch.rs`
- watch-only fixtures or harness support required by that test

Forbidden:

- `request.rs`
- `docs.rs`
- `prepare.rs`
- `closeout/**`
- `.github/workflows/**`

#### `MCI-40` Packet Surface

Owns:

- `crates/xtask/src/agent_maintenance/request.rs`
- `crates/xtask/src/agent_maintenance/docs.rs`
- `crates/xtask/src/agent_maintenance/prepare.rs`
- `crates/xtask/src/agent_maintenance/closeout/types.rs`
- `crates/xtask/src/agent_maintenance/closeout/write.rs`
- `docs/specs/cli-agent-onboarding-charter.md`
- `crates/xtask/tests/agent_maintenance_prepare.rs`
- required request/closeout test updates in:
  - `crates/xtask/tests/agent_maintenance_refresh.rs`
  - `crates/xtask/tests/agent_maintenance_closeout.rs`

Forbidden:

- `agent_registry.toml`
- `.github/workflows/**`

#### `MCI-50` Shared Workflow Topology

Owns:

- `.github/workflows/agent-maintenance-release-watch.yml`
- `.github/workflows/agent-maintenance-open-pr.yml`
- delete `.github/workflows/codex-cli-release-watch.yml`
- delete `.github/workflows/claude-code-release-watch.yml`
- first-pass updates to `crates/xtask/tests/c4_spec_ci_wiring.rs`
- minimal `ci.yml` edits only if needed for workflow contract coverage

Forbidden:

- `.github/workflows/codex-cli-update-snapshot.yml`
- `.github/workflows/claude-code-update-snapshot.yml`
- promote workflows

#### `MCI-71` Codex Worker Migration

Owns:

- `.github/workflows/codex-cli-update-snapshot.yml`

Forbidden:

- shared workflow files
- `c4_spec_ci_wiring.rs`
- promote workflows

#### `MCI-72` Claude Worker Migration

Owns:

- `.github/workflows/claude-code-update-snapshot.yml`

Forbidden:

- shared workflow files
- `c4_spec_ci_wiring.rs`
- promote workflows

## Workstream Plan

### Phase 1: Parent-only Freeze And Launch Control

#### `MCI-00` Baseline Capture

Scope:

- Capture the starting state from the real repo and live milestone inputs.

Owned files:

- `.runs/maintenance-ci-revamp/baseline.json`
- `.runs/maintenance-ci-revamp/session-log.md`
- `.runs/maintenance-ci-revamp/tasks.json`

Required commands:

```sh
git branch --show-current
git rev-parse HEAD
rg --files -g 'PLAN.md' -g 'ORCH_PLAN.md' -g '.github/workflows/*' -g 'crates/xtask/src/**'
```

Acceptance:

- branch confirmed as `codex/recommend-next-agent`
- workflow inventory captured
- stale `ORCH_PLAN.md` assumptions recorded
- `started.json` and `done.json` written for `task-mci-00-baseline`

#### `MCI-05` Scope Freeze From `PLAN.md`

Scope:

- Freeze the milestone strictly from `PLAN.md` and reject stale goals.

Owned files:

- `.runs/maintenance-ci-revamp/freeze.json`
- `.runs/maintenance-ci-revamp/session-log.md`

Required commands:

```sh
sed -n '1,260p' PLAN.md
sed -n '261,980p' PLAN.md
```

Acceptance:

- scope frozen to maintenance-CI only
- goose explicitly marked out of scope for this orchestration
- exact additions, deletions, and preserved worker workflows extracted from `PLAN.md`

#### `MCI-10` Interface And Ownership Freeze

Scope:

- Freeze interfaces before workers launch.

Owned files:

- `.runs/maintenance-ci-revamp/freeze.json`
- `.runs/maintenance-ci-revamp/queue.json`

Required commands:

```sh
sed -n '1,260p' crates/xtask/src/main.rs
sed -n '1,260p' crates/xtask/src/agent_registry.rs
sed -n '1,260p' crates/xtask/src/agent_maintenance/request.rs
sed -n '1,260p' crates/xtask/tests/c4_spec_ci_wiring.rs
```

Freeze items:

- xtask command spellings
- queue schema
- request-v2 fields
- workflow filenames
- branch naming
- PR base behavior
- exact owned/forbidden surfaces per task

Acceptance:

- workers can be launched from one stable contract
- stale-lane invalidation rules are explicit

#### `MCI-15` Worktree / Worker Launch Freeze

Scope:

- Create worktrees and launch the first worker wave from the frozen parent SHA.

Owned files:

- `.runs/maintenance-ci-revamp/queue.json`
- `.runs/maintenance-ci-revamp/tasks.json`
- task sentinel markers

Required commands:

```sh
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-mci-20-registry -b codex/recommend-next-agent-mci-20-registry codex/recommend-next-agent
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-mci-30-watch -b codex/recommend-next-agent-mci-30-watch codex/recommend-next-agent
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-mci-40-packet -b codex/recommend-next-agent-mci-40-packet codex/recommend-next-agent
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-mci-50-shared-workflows -b codex/recommend-next-agent-mci-50-shared-workflows codex/recommend-next-agent
```

Acceptance:

- worker launch SHA recorded
- no worker launched before `MCI-10` done
- active worker count does not exceed `4`

### Phase 2: Parallel Worker Phase

#### `MCI-20` Registry Contract

Scope:

- Add and validate `release_watch` metadata in registry truth.

Required commands:

```sh
cargo test -p xtask --test agent_registry
```

Acceptance:

- `release_watch` metadata parses and validates
- invalid dispatch/source combinations fail
- only `codex` and `claude_code` are enabled in milestone 1
- no second enrollment inventory exists

#### `MCI-30` Watch Surface

Scope:

- Add `maintenance-watch` detector and queue emitter.

Required commands:

```sh
cargo test -p xtask --test agent_maintenance_watch
```

Acceptance:

- `maintenance-watch --check` path exists
- `maintenance-watch --emit-json _ci_tmp/maintenance-watch.json` path exists
- queue entries include frozen fields from `PLAN.md`
- `github_releases` and `gcs_object_listing` both support `latest_stable_minus_one`
- no stale-detection logic remains in workflow JavaScript

#### `MCI-40` Packet Surface

Scope:

- Add request-v2 automated trigger support plus packet creation.

Required commands:

```sh
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_refresh
cargo test -p xtask --test agent_maintenance_closeout
```

Acceptance:

- request v2 parses
- `upstream_release_detected` validates
- `prepare-agent-maintenance --dry-run|--write` exists
- packet roots are created under `docs/agents/lifecycle/<agent_id>-maintenance/`
- generated packet content is contributor-ready and packet-first
- closeout remains truthful for automated triggers

#### `MCI-50` Shared Workflow Topology

Scope:

- Add shared watcher and generic packet-only PR workflow, and delete legacy scheduled watcher
  workflows.

Required commands:

```sh
cargo test -p xtask --test c4_spec_ci_wiring
```

Acceptance:

- one shared watcher workflow exists
- one generic packet-only workflow exists
- legacy scheduled watcher workflows are deleted
- shared workflow fans out from queue data
- generic packet-only workflow never performs artifact acquisition or snapshot generation

### Phase 3: Parent Spine Integration

#### `MCI-60` Parent Spine Integration

Scope:

- Integrate shared xtask entrypoints and module exports after worker phase 1 stabilizes.

Owned files:

- `crates/xtask/src/agent_maintenance/mod.rs`
- `crates/xtask/src/main.rs`

Required commands:

```sh
git merge --no-ff codex/recommend-next-agent-mci-20-registry
git merge --no-ff codex/recommend-next-agent-mci-30-watch
git merge --no-ff codex/recommend-next-agent-mci-40-packet
```

Acceptance:

- parent owns all integration edits in `mod.rs` and `main.rs`
- worker outputs land without write-surface collisions
- if frozen interfaces changed, stale lanes are invalidated before proceeding

### Phase 4: Worker Migration Phase

#### `MCI-71` Codex Worker Migration

Scope:

- Convert codex update workflow into a worker-only consumer of shared inputs.

Required commands:

```sh
cargo test -p xtask --test c4_spec_ci_wiring
```

Acceptance:

- accepts shared payload fields
- uses `target_version`
- runs `prepare-agent-maintenance --write` before PR creation
- preserves existing codex artifact/snapshot/union/report/validate path
- uses `automation/codex-maintenance-<target_version>` against `staging`

#### `MCI-72` Claude Worker Migration

Scope:

- Convert claude update workflow into a worker-only consumer of shared inputs.

Required commands:

```sh
cargo test -p xtask --test c4_spec_ci_wiring
```

Acceptance:

- accepts shared payload fields
- uses `target_version`
- runs `prepare-agent-maintenance --write` before PR creation
- preserves existing claude artifact/snapshot/union/report/validate path
- uses `automation/claude_code-maintenance-<target_version>` against `staging`

### Phase 5: Parent-only Integration / Proving / Closeout

#### `MCI-70` Parent Workflow/Worker Integration

Scope:

- Merge shared workflows first, then worker migrations.

Owned files:

- final conflict resolution in `crates/xtask/tests/c4_spec_ci_wiring.rs`

Required commands:

```sh
git merge --no-ff codex/recommend-next-agent-mci-50-shared-workflows
git merge --no-ff codex/recommend-next-agent-mci-71-codex-worker
git merge --no-ff codex/recommend-next-agent-mci-72-claude-worker
cargo test -p xtask --test c4_spec_ci_wiring
```

Acceptance:

- merge order preserved: shared workflows first, then worker migrations
- no promote workflows changed
- final CI wiring contract is truthful

#### `MCI-80` Docs And CI Contract Closeout

Scope:

- Final docs truth pass and any last CI contract cleanup.

Owned files:

- `docs/cli-agent-onboarding-factory-operator-guide.md`
- final touch-ups in docs/specs if parent integration exposed drift
- final cleanup in `crates/xtask/tests/c4_spec_ci_wiring.rs` if still needed

Required commands:

```sh
rg -n "maintenance-watch|prepare-agent-maintenance|release-watch|packet_pr|upstream_release_detected" docs/specs docs/cli-agent-onboarding-factory-operator-guide.md
```

Acceptance:

- operator guide matches live workflows and new command surfaces
- docs do not imply goose is in this milestone
- docs do not mention legacy watcher workflows as live entrypoints

#### `MCI-90` Final Proving And Acceptance

Scope:

- Run the full proving gate and complete final acceptance.

Required commands:

```sh
cargo test -p xtask --test agent_registry
cargo test -p xtask --test agent_maintenance_drift
cargo test -p xtask --test agent_maintenance_refresh
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test agent_maintenance_closeout
cargo test -p xtask --test c4_spec_ci_wiring
make preflight
```

Acceptance:

- all explicit commands pass
- final topology matches `PLAN.md`
- orchestration ends here with maintenance-CI complete
- no goose lane is launched

## Merge Order

Frozen merge order:

1. `MCI-20`
2. `MCI-30`
3. `MCI-40`
4. parent `MCI-60` spine integration
5. `MCI-50`
6. `MCI-71`
7. `MCI-72`
8. parent `MCI-80`
9. parent `MCI-90`

Relaunch rule:

- If `MCI-20`, `MCI-30`, or `MCI-40` changes frozen interfaces after another lane launched,
  affected lanes are stale and must be relaunched from the new parent HEAD.
- Rebase-in-place is allowed only for non-interface drift outside the worker’s frozen owned
  surfaces.

## Context-Control Rules

- Parent retains the full milestone context.
- Worker prompts include only:
  - frozen scope excerpt
  - owned files
  - forbidden files
  - exact acceptance
  - exact required commands
- Workers do not receive broad repo-edit authority.
- `MCI-30` and `MCI-40` may both touch `crates/xtask/src/agent_maintenance/`, but ownership is
  frozen by file, not by directory.
- `MCI-50`, `MCI-71`, and `MCI-72` may all affect workflow behavior, but only `MCI-50` may edit
  shared workflow files.
- Parent owns all final conflict resolution in `c4_spec_ci_wiring.rs`.

## Stale-Lane Invalidation Rules

Invalidate and relaunch a worker lane if any of the following changes after launch:

- `PLAN.md` hash
- frozen queue fields
- request-v2 field names
- workflow input names
- `dispatch_kind` semantics
- branch naming rule
- milestone-1 enrollment scope
- any write-surface boundary in this plan

Do not patch stale lanes informally. Relaunch them from the new frozen parent SHA.

## Tests And Acceptance

### Registry

- `release_watch` metadata exists in registry truth
- only `codex` and `claude_code` are enabled
- invalid source/dispatch combinations fail
- `cargo test -p xtask --test agent_registry` passes

### Watch

- `maintenance-watch --check` exists
- `maintenance-watch --emit-json _ci_tmp/maintenance-watch.json` exists
- queue schema matches `PLAN.md`
- both upstream source kinds are covered
- `cargo test -p xtask --test agent_maintenance_watch` passes

### Packet Creator

- automated request version is `2`
- trigger kind is `upstream_release_detected`
- `[detected_release]` is persisted
- maintenance packet roots are created on first write
- `cargo test -p xtask --test agent_maintenance_prepare` passes

### Request / Refresh / Closeout Compatibility

- `agent_maintenance_refresh` accepts request v2
- closeout preserves automated-trigger truth
- final `HANDOFF.md` remains truthful
- `cargo test -p xtask --test agent_maintenance_refresh` passes
- `cargo test -p xtask --test agent_maintenance_closeout` passes

### Shared Workflows

- `.github/workflows/agent-maintenance-release-watch.yml` exists
- `.github/workflows/agent-maintenance-open-pr.yml` exists
- `.github/workflows/codex-cli-release-watch.yml` is deleted
- `.github/workflows/claude-code-release-watch.yml` is deleted
- no promote workflow changes exist
- `cargo test -p xtask --test c4_spec_ci_wiring` passes

### Codex Worker

- `.github/workflows/codex-cli-update-snapshot.yml` accepts shared payload
- packet generation happens before PR creation
- codex artifact and validation pipeline remains intact
- branch naming and `staging` base rules hold

### Claude Worker

- `.github/workflows/claude-code-update-snapshot.yml` accepts shared payload
- packet generation happens before PR creation
- claude artifact and validation pipeline remains intact
- branch naming and `staging` base rules hold

### Final Proving

- all targeted xtask tests pass
- `make preflight` passes
- docs reflect the new live workflow
- maintenance-CI milestone is complete
- orchestration stops without entering goose execution
