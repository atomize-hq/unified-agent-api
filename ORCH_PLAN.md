# ORCH_PLAN - First Honest Maintenance Proof Run

## Summary

Current checked-out branch context: `staging`  
Execution branch: `codex/first-honest-maintenance-proof-run`  
Workflow checkout ref: `staging`  
Authoritative milestone: repo-root `PLAN.md`, milestone `First Honest Maintenance Proof Run`  
Plan revision baseline: `bbd15f6`  
Convergence baseline: `ee8249a`  
Authoritative decision sink: existing transport-topology item in `TODOS.md`  
Proof target: `codex` only  
Workflow entrypoint: `.github/workflows/agent-maintenance-release-watch.yml`  
Remote discipline: every live workflow run must exercise pushed `origin/staging` truth  
Parent role: sole integrator, sole live-workflow operator, sole writer of final decision and closeout  
Worker model for launched lanes: `GPT-5.4` with `reasoning_effort=high`  
Worker concurrency cap: `0` before `C2`; maximum `2` conditional lanes after `C2` and only when ownership is disjoint

This plan is intentionally parent-serialized through the honest proof path. The shared watcher,
downstream worker, generated packet, local relay, recovery path, and manual closeout form one
evidence chain. Parallelism becomes honest only after that chain is complete or blocked and the
parent has frozen what actually failed.

### Orchestration Source Of Truth

Authoritative orchestration state:

- `.runs/first-honest-maintenance-proof-run/tasks.json`
- `.runs/first-honest-maintenance-proof-run/freeze.json`
- `.runs/first-honest-maintenance-proof-run/lane-status.json`
- `.runs/first-honest-maintenance-proof-run/session-log.md`

Most important derived evidence:

- `.runs/first-honest-maintenance-proof-run/queue-preflight-summary.md`
- `.runs/first-honest-maintenance-proof-run/live-runs.md`
- `.runs/first-honest-maintenance-proof-run/packet-review.md`
- `.runs/first-honest-maintenance-proof-run/failure-ledger.md`
- `.runs/first-honest-maintenance-proof-run/final-gates.md`
- `.runs/first-honest-maintenance-proof-run/decision.md`

## Hard Guards

- `PLAN.md` wins over this file on any conflict.
- The only live proof target in this milestone is `codex`.
- The proof must start at `.github/workflows/agent-maintenance-release-watch.yml`.
- Directly opening with `.github/workflows/codex-cli-update-snapshot.yml` is forbidden.
- No synthetic stale target is allowed. If `maintenance-watch` does not emit a real stale `codex`
  item, stop with `blocked_no_live_stale_codex`.
- The queue-emitted `target_version` and `branch_name` are authoritative over any pre-existing
  committed maintenance packet under `docs/agents/lifecycle/codex-maintenance/**`.
- The parent is the only actor allowed to:
  - trigger or rerun GitHub workflows
  - inspect and classify live proof evidence
  - create or reopen the maintenance PR
  - populate `maintenance-closeout.json`
  - run `close-agent-maintenance`
  - edit `TODOS.md`
  - decide keep-vs-cutover
- Workers may not edit `PLAN.md`, `ORCH_PLAN.md`, `TODOS.md`, `docs/specs/**`, or
  `docs/agents/lifecycle/codex-maintenance/**`.
- No proof-only xtask commands, workflow families, packet schema, or closeout format may be
  introduced.
- Generated maintenance packet docs must be regenerated through the real worker or the documented
  refresh path. Hand edits to rescue packet truth are forbidden.
- If a fix would require a new workflow family, a new schema, or a redesign across the shared
  watch -> worker -> packet -> relay boundary, classify it as `topology_issue` and stop widening
  scope inside this milestone.

## Worktree Strategy

Use the existing repo checkout only for reading and final comparison. Run the proof from dedicated
worktrees so parent integration and any later repair lanes stay isolated.

### Roots

- Worktree root:
  `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-first-honest-maintenance-proof-run`
- Run-state root:
  `/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.runs/first-honest-maintenance-proof-run`

### Required Worktrees

1. `staging-live`
   - branch: `staging`
   - path:
     `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-first-honest-maintenance-proof-run/staging-live`
   - purpose: inspect remote-aligned truth and final landing smoke check
2. `parent-proof`
   - branch: `codex/first-honest-maintenance-proof-run`
   - base: `origin/staging`
   - path:
     `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-first-honest-maintenance-proof-run/parent-proof`
   - purpose: parent critical path, integration, reruns, final validation

### Conditional Worker Worktrees

Create only after `C2_SHA` exists and the parent classifies a localized repair worth parallelizing.

1. `ws-b-watch-packet`
   - branch: `codex/maintenance-proof-watch-packet`
   - base: `C2_SHA`
2. `ws-c-relay-closeout`
   - branch: `codex/maintenance-proof-relay-closeout`
   - base: `C2_SHA`

### Creation Commands

```bash
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-first-honest-maintenance-proof-run/staging-live staging
git worktree add -b codex/first-honest-maintenance-proof-run /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-first-honest-maintenance-proof-run/parent-proof origin/staging
```

After `C2_SHA` freeze:

```bash
git worktree add -b codex/maintenance-proof-watch-packet /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-first-honest-maintenance-proof-run/ws-b-watch-packet "$C2_SHA"
git worktree add -b codex/maintenance-proof-relay-closeout /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-first-honest-maintenance-proof-run/ws-c-relay-closeout "$C2_SHA"
```

## Run-State And Freeze Artifacts

The parent owns every run-state artifact. Workers do not write under `.runs/**`.

### Required Files

- `.runs/first-honest-maintenance-proof-run/tasks.json`
- `.runs/first-honest-maintenance-proof-run/baseline.json`
- `.runs/first-honest-maintenance-proof-run/remote-alignment.md`
- `.runs/first-honest-maintenance-proof-run/queue-preflight.json`
- `.runs/first-honest-maintenance-proof-run/queue-preflight-summary.md`
- `.runs/first-honest-maintenance-proof-run/session-log.md`
- `.runs/first-honest-maintenance-proof-run/live-runs.md`
- `.runs/first-honest-maintenance-proof-run/packet-review.md`
- `.runs/first-honest-maintenance-proof-run/recovery-log.md`
- `.runs/first-honest-maintenance-proof-run/closeout-log.md`
- `.runs/first-honest-maintenance-proof-run/failure-ledger.md`
- `.runs/first-honest-maintenance-proof-run/freeze.json`
- `.runs/first-honest-maintenance-proof-run/lane-status.json`
- `.runs/first-honest-maintenance-proof-run/merge-log.md`
- `.runs/first-honest-maintenance-proof-run/final-gates.md`
- `.runs/first-honest-maintenance-proof-run/decision.md`
- `.runs/first-honest-maintenance-proof-run/acceptance.md`

### Queue And Session-Log Contract

`tasks.json` is the lightweight orchestration queue. It is authoritative for workstream and task
state. Minimum entry shape:

```json
{
  "id": "task/p2.3",
  "workstream": "WS-P2",
  "title": "Capture downstream worker run URL",
  "status": "pending|in_progress|blocked|completed",
  "owner": "parent|ws-b|ws-c",
  "depends_on": ["task/p2.2"],
  "checkpoint": "C1|C2|C3|null",
  "notes": "short current state"
}
```

`session-log.md` is the authoritative chronological operator log. The parent records:

- worktree creation
- run-state initialization
- every live workflow trigger
- every freeze event
- every worker launch and return
- every rerun decision
- final verdict timing

Authoritative orchestration-state artifacts:

- `tasks.json`
- `freeze.json`
- `lane-status.json`
- `session-log.md`

Derived evidence artifacts:

- `queue-preflight.json`
- `queue-preflight-summary.md`
- `live-runs.md`
- `packet-review.md`
- `recovery-log.md`
- `closeout-log.md`
- `failure-ledger.md`
- `merge-log.md`
- `final-gates.md`
- `decision.md`
- `acceptance.md`

### Freeze Points

1. `C0`
   - remote truth frozen
   - required fields:
     - local `staging` SHA
     - `origin/staging` SHA
     - cleanliness snapshot
     - plan baseline SHA
2. `C1`
   - local queue truth frozen
   - required fields:
     - `agent_id`
     - `current_validated`
     - `latest_stable`
     - `target_version`
     - `dispatch_workflow`
     - `branch_name`
     - `detected_by`
3. `C2`
   - live proof evidence frozen after packet review, relay attempt, and closeout/recovery branch
     classification
   - required fields:
     - watcher run URL
     - worker run URL
     - exercised remote commit SHA
     - branch / PR outcome
     - prepared `run_id` if produced
     - failure classification ledger
4. `C3`
   - post-repair integration frozen
   - required fields:
     - integrated commit SHA
     - merged lane SHAs
     - rerun scope
     - final gate status

## Workstream Plan

### WS-K0 - Kickoff, Worktrees, And Run-State Bootstrap

Type: parent-only  
Goal: create the execution surfaces and initialize authoritative orchestration state before any
proof task begins

Task queue:

- `task/k0.1` create `staging-live` worktree from `staging`
- `task/k0.2` create `parent-proof` worktree from `origin/staging`
- `task/k0.3` create `.runs/first-honest-maintenance-proof-run/`
- `task/k0.4` initialize `tasks.json`, `freeze.json`, `lane-status.json`, `session-log.md`
- `task/k0.5` record exact paths and branch mapping in `session-log.md`

Required commands:

```bash
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-first-honest-maintenance-proof-run/staging-live staging
git worktree add -b codex/first-honest-maintenance-proof-run /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-first-honest-maintenance-proof-run/parent-proof origin/staging
```

Execution:

1. Create the two required worktrees.
2. Initialize the run-state root and empty authoritative orchestration files.
3. Seed `tasks.json` with all `WS-K0` through `WS-P6` tasks in `pending` state.
4. Seed `lane-status.json` with `ws-b = not_created`, `ws-c = not_created`.
5. Append a kickoff entry to `session-log.md`.

Acceptance:

- worktree layout exists exactly once
- authoritative orchestration files exist before `WS-P0`
- the task queue can represent parent-only serialization and later conditional worker launch

### WS-P0 - Baseline And Preconditions Lock

Type: parent-only  
Goal: freeze the exact proving target, remote truth, and stop conditions before any live run

Task queue:

- `task/p0.1` read `PLAN.md` and dependent proof surfaces
- `task/p0.2` fetch `origin` and record branch cleanliness
- `task/p0.3` compare `staging` against `origin/staging`
- `task/p0.4` confirm `TODOS.md` decision sink and no-synthetic-target rule
- `task/p0.5` freeze `C0`

Required reads:

- `PLAN.md`
- `TODOS.md`
- `.github/workflows/agent-maintenance-release-watch.yml`
- `.github/workflows/codex-cli-update-snapshot.yml`
- `crates/xtask/data/agent_registry.toml`
- `docs/specs/maintenance-request-contract-v1.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `cli_manifests/codex/OPS_PLAYBOOK.md`

Required commands:

```bash
git fetch origin
git status --short --branch
git rev-parse staging
git rev-parse origin/staging
```

Execution:

1. Record baseline state in `baseline.json`.
2. Compare local `staging` to `origin/staging`.
3. If they differ, push `staging` before any live proof step.
4. Confirm the existing transport-topology TODO item is the only final verdict destination.
5. Update `tasks.json` and `session-log.md` as each task completes.
6. Freeze `C0`.

Acceptance:

- `origin/staging` alignment is known and documented.
- The parent can state the exact block condition for “no live stale codex target.”
- No live workflow has run yet.

### WS-P1 - Local Queue Preflight

Type: parent-only  
Goal: prove the shared watcher would honestly emit `codex` before spending Actions time

Task queue:

- `task/p1.1` run local `maintenance-watch --check`
- `task/p1.2` emit `_ci_tmp/maintenance-watch.json`
- `task/p1.3` copy queue artifact into run-state
- `task/p1.4` extract canonical queue fields into summary
- `task/p1.5` freeze `C1` or stop as `blocked_no_live_stale_codex`

Required commands:

```bash
cargo run -p xtask -- maintenance-watch --check
cargo run -p xtask -- maintenance-watch --emit-json _ci_tmp/maintenance-watch.json
```

Execution:

1. Confirm the queue contains a real stale `codex` item.
2. Copy `_ci_tmp/maintenance-watch.json` into run-state as `queue-preflight.json`.
3. Record the emitted queue item in `queue-preflight-summary.md`.
4. Update `tasks.json` and `session-log.md` with queue outcome and target version.
5. Freeze `C1`.
6. If `codex` is not emitted, stop with `blocked_no_live_stale_codex`. Do not fabricate worker
   inputs.

Acceptance:

- `C1` captures the exact `target_version` and canonical
  `automation/codex-maintenance-<target_version>` branch shape.
- The parent can name the downstream worker before touching GitHub Actions.

### WS-P2 - Shared Watcher Live Proof

Type: parent-only  
Goal: prove the real entrypoint dispatches the right worker with unchanged queue truth

Task queue:

- `task/p2.1` trigger `agent-maintenance-release-watch.yml`
- `task/p2.2` capture watcher run URL and exercised remote SHA
- `task/p2.3` capture downstream worker run URL
- `task/p2.4` compare live dispatch fields against `C1`
- `task/p2.5` record branch / PR outcome or replay-path outcome
- `task/p2.6` restart from `WS-P1` if target drift invalidates `C1`

Required command path:

```bash
gh workflow run agent-maintenance-release-watch.yml --ref staging
gh run list --workflow agent-maintenance-release-watch.yml --branch staging --limit 1
gh run list --workflow codex-cli-update-snapshot.yml --branch staging --limit 1
```

GitHub UI is acceptable if CLI auth is unavailable, but the same evidence must still be captured.

Execution:

1. Trigger `agent-maintenance-release-watch.yml` from `staging`.
2. Capture the watcher run URL and exercised `origin/staging` commit SHA.
3. Capture the downstream `codex-cli-update-snapshot.yml` run URL.
4. Compare live watcher/worker inputs against `C1`:
   - `agent_id`
   - `current_validated`
   - `latest_stable`
   - `target_version`
   - `dispatch_workflow`
   - `branch_name`
5. If the live queue sees a newer release and the target changes, discard `C1` and restart at
   `WS-P1`.
6. Record whether the worker:
   - opened the expected maintenance branch and PR, or
   - failed at PR creation with a documented replay path
7. Update `live-runs.md`, `tasks.json`, and `session-log.md`.

Acceptance:

- The proof starts at the shared watcher, not the worker.
- Queue truth survives watcher -> worker handoff unchanged, or the run is honestly restarted.
- The generated `docs/agents/lifecycle/codex-maintenance/**` surfaces now belong to the live run.

### WS-P3 - Packet Review And Relay Proof

Type: parent-only  
Goal: prove the generated packet is sufficient for a maintainer to execute the local lane

Task queue:

- `task/p3.1` review `HANDOFF.md`
- `task/p3.2` review `maintenance-request.toml`
- `task/p3.3` review `pr-summary.md`
- `task/p3.4` record packet field agreement
- `task/p3.5` run relay dry-run
- `task/p3.6` record prepared `run_id` and temp evidence root if produced
- `task/p3.7` run relay write mode on the frozen `run_id`
- `task/p3.8` classify packet / relay failures into `failure-ledger.md`

Review order:

1. `docs/agents/lifecycle/codex-maintenance/HANDOFF.md`
2. `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
3. `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md`

Required field agreement:

- `target_version`
- `branch_name`
- `detected_by`
- `dispatch_workflow`
- `executor = "execute-agent-maintenance"`
- `closeout_path`

Required commands:

```bash
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --dry-run
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --write --run-id <prepared_run_id>
```

Execution:

1. Record packet consistency findings in `packet-review.md`.
2. Run dry-run first.
3. If dry-run does not emit a prepared `run_id`, stop and classify the failure before attempting
   write mode.
4. Record the prepared `run_id` and temp evidence root under
   `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`.
5. Run write mode with the same `run_id`.
6. Record all results in `failure-ledger.md`.
7. Update `tasks.json` and `session-log.md`.

Acceptance:

- A maintainer can move from packet to relay without undocumented steps, or the exact missing step
  is frozen as evidence.
- `HANDOFF.md` remains canonical and `pr-summary.md` remains derivative.

### WS-P4 - Recovery And Closeout Proof

Type: parent-only  
Goal: prove recovery is sufficient when needed and closeout is explicit in all cases

Task queue:

- `task/p4.1` decide whether recovery is required
- `task/p4.2` run `refresh-agent --request ... --write` if required
- `task/p4.3` reopen PR via normal branch if required
- `task/p4.4` populate `maintenance-closeout.json`
- `task/p4.5` run `close-agent-maintenance`
- `task/p4.6` verify post-closeout surfaces still agree
- `task/p4.7` freeze `C2`

Recovery path, only if PR creation failed earlier:

```bash
cargo run -p xtask -- refresh-agent --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --write
gh pr create --base staging --head "automation/codex-maintenance-<target_version>" --title "..." --body-file docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md
```

Closeout path:

```bash
cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json
```

Execution:

1. If PR creation already succeeded, record recovery as `not_exercised_initial_pr_succeeded`.
2. If recovery is needed, prove it does not require packet surgery, branch renaming, or ad hoc
   prose edits.
3. Populate `maintenance-closeout.json` truthfully with:
   - `resolved_findings`
   - `deferred_findings` or `explicit_none_reason`
   - `preflight_passed`
   - `recorded_at`
   - `commit`
4. Run `close-agent-maintenance`.
5. Verify post-closeout surfaces still tell the same story.
6. Update `recovery-log.md`, `closeout-log.md`, `tasks.json`, and `session-log.md`.
7. Freeze `C2`.

Acceptance:

- Recovery is either proven sufficient when needed or explicitly marked unexercised because initial
  PR creation succeeded.
- Closeout uses the real request and the real closeout artifact, not a proof-only substitute.

### WS-P5 - Parent Decision And Optional Repair Launch

Type: parent-only  
Goal: decide whether the proof passed, is blocked, needs localized repair, or reveals topology
pressure

Task queue:

- `task/p5.1` classify every finding into one failure class
- `task/p5.2` decide whether the result is pass, block, localized repair, or topology issue
- `task/p5.3` update `lane-status.json`
- `task/p5.4` launch `WS-B` and/or `WS-C` only if ownership is disjoint
- `task/p5.5` record worker launch decisions and payloads in `session-log.md`

Execution:

1. Classify every issue in `failure-ledger.md` as one of:
   - `host_env`
   - `local_bug`
   - `doc_drift`
   - `target_specific`
   - `topology_issue`
2. Decide whether any localized repair remains both honest and parallelizable.
3. Launch zero, one, or two worker lanes according to the rules below.
4. If the result is `host_env` only and the documented host-repair path is sufficient, the parent
   repairs locally and reruns the same phase without launching workers.
5. If any issue is `topology_issue`, do not launch repair workers for a redesign. Move directly to
   final decision recording after parent validation.
6. Update `tasks.json`, `lane-status.json`, and `session-log.md`.

Acceptance:

- No worker is launched on speculation.
- Any worker that launches has a frozen `C2_SHA`, a bounded file map, and a single proven issue
  class to address.

### WS-B - Shared Watch / Worker / Packet Repair

Type: conditional worker  
Launch gate: `C2_SHA` exists and the parent has proven a localized issue in watcher truth,
workflow dispatch, packet preparation, or packet rendering

Owned surfaces:

- `.github/workflows/agent-maintenance-release-watch.yml`
- `.github/workflows/codex-cli-update-snapshot.yml`
- `crates/xtask/src/agent_maintenance/watch.rs`
- `crates/xtask/src/agent_maintenance/prepare.rs`
- `crates/xtask/src/agent_maintenance/docs.rs`
- `crates/xtask/src/agent_maintenance/contract_policy.rs`
- `crates/xtask/src/agent_maintenance/request.rs`
- `crates/xtask/src/agent_maintenance/request/automation.rs`
- `crates/xtask/tests/agent_maintenance_watch.rs`
- `crates/xtask/tests/agent_maintenance_prepare.rs`
- `crates/xtask/tests/c4_spec_ci_wiring.rs`

Required validations:

```bash
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test c4_spec_ci_wiring
```

Stop conditions:

- change requires `docs/specs/**`
- change requires hand edits under `docs/agents/lifecycle/codex-maintenance/**`
- change requires a new workflow family
- change requires a new packet schema
- change crosses into relay / closeout ownership

Acceptance:

- the smallest honest repair is implemented
- the narrowest regression lock is added
- the lane returns a clean handoff for parent integration

### WS-B Launch Payload

Ready-to-send worker brief:

```text
Workstream: WS-B
Base: <C2_SHA>
Model: GPT-5.4
Reasoning: high
Mission: fix one localized watcher / worker / packet issue without changing topology
Issue class: <local_bug|doc_drift|target_specific>
Owned files:
- .github/workflows/agent-maintenance-release-watch.yml
- .github/workflows/codex-cli-update-snapshot.yml
- crates/xtask/src/agent_maintenance/watch.rs
- crates/xtask/src/agent_maintenance/prepare.rs
- crates/xtask/src/agent_maintenance/docs.rs
- crates/xtask/src/agent_maintenance/contract_policy.rs
- crates/xtask/src/agent_maintenance/request.rs
- crates/xtask/src/agent_maintenance/request/automation.rs
- crates/xtask/tests/agent_maintenance_watch.rs
- crates/xtask/tests/agent_maintenance_prepare.rs
- crates/xtask/tests/c4_spec_ci_wiring.rs
Allowed run-state inputs:
- .runs/first-honest-maintenance-proof-run/failure-ledger.md
- .runs/first-honest-maintenance-proof-run/live-runs.md
- .runs/first-honest-maintenance-proof-run/queue-preflight-summary.md
- .runs/first-honest-maintenance-proof-run/packet-review.md
Mandatory validations:
- cargo test -p xtask --test agent_maintenance_watch
- cargo test -p xtask --test agent_maintenance_prepare
- cargo test -p xtask --test c4_spec_ci_wiring
Forbidden:
- editing PLAN.md, ORCH_PLAN.md, TODOS.md, docs/specs/**, docs/agents/lifecycle/codex-maintenance/**
- hand-editing generated packet surfaces
- adding a new workflow family or schema
- crossing into relay / closeout ownership
Return:
- status
- changed files
- commands with exit codes
- validation results
- commit SHA
- residual risks
```

### WS-C - Relay / Recovery / Closeout Repair

Type: conditional worker  
Launch gate: `C2_SHA` exists and the parent has proven a localized issue in relay execution,
recovery wording, operator guidance, or closeout behavior

Owned surfaces:

- `crates/xtask/src/agent_maintenance/execute.rs`
- `crates/xtask/src/agent_maintenance/execute/**`
- `crates/xtask/src/agent_maintenance/closeout.rs`
- `crates/xtask/src/agent_maintenance/closeout/**`
- `cli_manifests/codex/OPS_PLAYBOOK.md`
- `crates/xtask/tests/agent_maintenance_execute.rs`
- `crates/xtask/tests/agent_maintenance_closeout.rs`

Required validations:

```bash
cargo test -p xtask --test agent_maintenance_execute
cargo test -p xtask --test agent_maintenance_closeout
```

Stop conditions:

- change requires `PLAN.md`, `ORCH_PLAN.md`, `TODOS.md`, or `docs/specs/**`
- change requires hand edits under `docs/agents/lifecycle/codex-maintenance/**`
- change is actually topology redesign rather than localized repair
- change crosses into watcher / packet ownership

Acceptance:

- the smallest honest repair is implemented
- the narrowest regression lock is added
- the lane returns a clean handoff for parent integration

### WS-C Launch Payload

Ready-to-send worker brief:

```text
Workstream: WS-C
Base: <C2_SHA>
Model: GPT-5.4
Reasoning: high
Mission: fix one localized relay / recovery / closeout issue without changing topology
Issue class: <local_bug|doc_drift|target_specific>
Owned files:
- crates/xtask/src/agent_maintenance/execute.rs
- crates/xtask/src/agent_maintenance/execute/**
- crates/xtask/src/agent_maintenance/closeout.rs
- crates/xtask/src/agent_maintenance/closeout/**
- cli_manifests/codex/OPS_PLAYBOOK.md
- crates/xtask/tests/agent_maintenance_execute.rs
- crates/xtask/tests/agent_maintenance_closeout.rs
Allowed run-state inputs:
- .runs/first-honest-maintenance-proof-run/failure-ledger.md
- .runs/first-honest-maintenance-proof-run/packet-review.md
- .runs/first-honest-maintenance-proof-run/recovery-log.md
- .runs/first-honest-maintenance-proof-run/closeout-log.md
Mandatory validations:
- cargo test -p xtask --test agent_maintenance_execute
- cargo test -p xtask --test agent_maintenance_closeout
Forbidden:
- editing PLAN.md, ORCH_PLAN.md, TODOS.md, docs/specs/**, docs/agents/lifecycle/codex-maintenance/**
- hand-editing generated packet surfaces
- changing topology instead of making a localized repair
- crossing into watcher / packet ownership
Return:
- status
- changed files
- commands with exit codes
- validation results
- commit SHA
- residual risks
```

### WS-P6 - Parent Integration, Honest Reruns, And Final Verdict

Type: parent-only  
Goal: integrate any localized repair, rerun from the earliest impacted honest phase, and write the
final milestone decision

Task queue:

- `task/p6.1` integrate `WS-B` if present
- `task/p6.2` integrate `WS-C` if present
- `task/p6.3` regenerate stale parent-owned derived surfaces honestly after merges
- `task/p6.4` freeze `C3`
- `task/p6.5` rerun from the earliest impacted proof phase
- `task/p6.6` run final validations
- `task/p6.7` verify verdict prerequisites are satisfied
- `task/p6.8` write dated verdict note to `TODOS.md`
- `task/p6.9` move `staging-live` to the reviewed parent result and inspect smoke diff
- `task/p6.10` land reviewed result onto `staging`

Execution:

1. Integrate `WS-B` first if it exists.
2. Integrate `WS-C` second if it exists.
3. Record merge outcomes in `merge-log.md`.
4. Regenerate parent-owned derived surfaces only through honest upstream commands:
   - rerun worker / prepare path if request-owned truth changed
   - use `refresh-agent --request ... --write` only when the request is already current and only
     derived docs need regeneration
5. Never manually edit `docs/agents/lifecycle/codex-maintenance/**`.
6. Freeze `C3`.
7. Rerun from the earliest impacted honest phase.
8. Run final validations and record them in `final-gates.md`.
9. Confirm all verdict prerequisites are satisfied before touching `TODOS.md`:
   - `C3` exists if any repair merged
   - all required proof phases are either green or explicitly classified and accepted
   - all required regression locks are green
   - all stale generated maintenance surfaces have been honestly regenerated
   - final decision evidence is summarized in `decision.md`
10. Write the dated verdict note under the existing transport-topology TODO in `TODOS.md`.
11. Move `staging-live` to the reviewed parent result and inspect the smoke diff there before
    touching `staging`.
12. Record acceptance results in `acceptance.md`.
13. Append final landing steps and result to `session-log.md`.

Acceptance:

- Every localized fix is rerun from the earliest impacted phase.
- The milestone ends with one evidence-backed keep-vs-cutover decision.

## Launch Order

1. `WS-K0`
2. `WS-P0`
3. `WS-P1`
4. `WS-P2`
5. `WS-P3`
6. `WS-P4`
7. `WS-P5`
8. `WS-B` and/or `WS-C` only if `WS-P5` proves localized repair work
9. `WS-P6`

Parallelism rule:

- No concurrency before `C2`.
- Maximum post-`C2` worker concurrency is `2`, and only if `WS-B` and `WS-C` have disjoint
  ownership.
- If either lane discovers overlap or cross-lane coupling, it must stop and return `blocked`.

## Worker Prompt Contract

Every launched worker prompt must contain only:

- workstream id
- `C2_SHA`
- issue class
- owned files
- allowed run-state files
- required validations
- stop conditions
- return contract

### Allowed Run-State Inputs

- `WS-B`
  - `failure-ledger.md`
  - `live-runs.md`
  - `queue-preflight-summary.md`
  - `packet-review.md` if packet disagreement is the issue
- `WS-C`
  - `failure-ledger.md`
  - `packet-review.md`
  - `recovery-log.md`
  - `closeout-log.md`

Workers may not read:

- full `.runs/**` beyond their allowed summaries
- full GitHub logs unless the parent excerpts the relevant lines into run-state
- unrelated docs trees
- unrelated crates
- top-level planning docs beyond what the parent summarizes into the prompt

### Worker Return Contract

Every worker must return:

- workstream id
- status: `ready-for-parent`, `blocked`, or `no-op`
- `C2_SHA`
- issue class addressed
- changed files
- commands run
- exit code for every command run
- validation results
- resulting commit SHA
- residual risks or assumptions

## Failure Classification And Response Rules

| Class | Meaning | Required response | Allowed scope |
| --- | --- | --- | --- |
| `host_env` | local Codex CLI, auth, or host setup failed while packet truth stayed clear | parent repairs host using the documented path and reruns the same proof segment | no worker lane |
| `local_bug` | code or contract bug inside the existing topology | fix if boilable, add regression lock, rerun earliest impacted phase | localized repair only |
| `doc_drift` | instructions are incomplete or wrong | fix the doc plus a matching lock when possible, rerun impacted phase | localized repair only |
| `target_specific` | `codex`-specific issue that does not imply topology redesign | fix locally if boilable and keep decision narrow | localized repair only |
| `topology_issue` | the split watcher -> worker -> packet -> relay model itself is the problem | stop redesign work in this milestone and record trigger evidence | no architecture change here |

Rules:

1. `host_env` is not topology failure unless the documented repair path is itself unclear or
   broken.
2. `topology_issue` is the only class that can justify opening transport-topology convergence next.
3. If a change stops being boilable, reclassify it as topology evidence instead of sneaking in a
   redesign.

## Merge Rules And Honest Reruns

Parent-only integration order:

1. merge `WS-B` if present
2. merge `WS-C` if present
3. regenerate stale parent-owned derived surfaces through honest commands only
4. rerun from the earliest impacted proof phase
5. run final validations
6. refresh `decision.md`, `final-gates.md`, `acceptance.md`, and `session-log.md`
7. update `TODOS.md` only after verdict prerequisites are satisfied

Rerun matrix:

- watcher logic, registry truth, or shared watcher workflow changed:
  rerun `WS-P1`, `WS-P2`, and matching watch/dispatch tests
- downstream worker or packet generation changed:
  rerun `WS-P2`, `WS-P3`, and matching packet tests
- relay behavior or operator guidance changed without request-contract changes:
  rerun `WS-P3` and matching relay tests
- closeout behavior changed:
  rerun `WS-P4` and matching closeout tests
- request-owned fields changed:
  regenerate through the honest upstream path first; do not use direct refresh as a shortcut

Remote rerun rule:

- after any workflow-affecting repair, push `staging` before the next live watcher rerun and
  record the exercised remote commit SHA

## Context-Control Rules

- The parent keeps these artifacts current and authoritative:
  - `PLAN.md`
  - `TODOS.md`
  - `freeze.json`
  - `failure-ledger.md`
  - `live-runs.md`
  - `lane-status.json`
- Workers get summarized evidence, not the whole milestone transcript.
- If a worker needs more context than its prompt and allowed run-state files provide, it must stop
  as `blocked` rather than expanding scope on its own.
- Packet truth mismatches must be summarized by the parent in `packet-review.md`; workers should
  not reverse-engineer the entire proof from raw artifacts.

## Tests And Acceptance

### Required Proof Commands

```bash
cargo run -p xtask -- maintenance-watch --check
cargo run -p xtask -- maintenance-watch --emit-json _ci_tmp/maintenance-watch.json
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --dry-run
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --write --run-id <prepared_run_id>
cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json
```

### Required Regression Locks

```bash
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test c4_spec_ci_wiring
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_execute
cargo test -p xtask --test agent_maintenance_closeout
```

If localized fixes land, also run the impacted suite plus repo gates the packet still depends on:

```bash
cargo run -p xtask -- support-matrix --check
cargo run -p xtask -- capability-matrix --check
cargo run -p xtask -- capability-matrix-audit
make preflight
```

### Acceptance Checklist

Queue and watcher truth:

- local `maintenance-watch` preflight identified a real stale `codex` target
- `C1` records the authoritative queue item
- the live proof started at the shared watcher from `staging`
- any target-version drift triggered an honest restart

Worker and packet truth:

- the downstream `codex` worker was dispatched from the shared watcher
- `HANDOFF.md` remained canonical
- `maintenance-request.toml` remained the frozen relay contract
- `pr-summary.md` remained derivative
- packet surfaces agreed on version, branch, dispatcher, executor, and closeout path

Relay and recovery truth:

- `execute-agent-maintenance --dry-run` was exercised
- `execute-agent-maintenance --write --run-id <prepared_run_id>` was exercised if dry-run produced
  a prepared run
- recovery was sufficient when needed, and explicitly unexercised when not needed
- no packet surgery or branch renaming was required

Closeout truth:

- `maintenance-closeout.json` was populated explicitly
- `close-agent-maintenance` ran against the same frozen request
- post-closeout surfaces still told the same story

Decision truth:

- every issue was classified explicitly
- every localized fix gained a regression lock
- `TODOS.md` received one dated verdict note under the existing topology item
- the verdict cites proof evidence, not architecture taste

## Final Gate Logic

Before `TODOS.md` is updated, all of the following must be true:

- any merged repair lanes are integrated and recorded in `merge-log.md`
- stale generated maintenance surfaces have been regenerated honestly, never by manual edit
- the earliest impacted proof phase has been rerun
- required proof commands and regression locks are green, or any non-green result is explicitly
  classified and accepted in `decision.md`
- the parent has a final evidence summary with exercised SHA, target version, watcher run URL,
  worker run URL, and final classification outcome

Landing order is fixed:

1. integrate worker lanes
2. regenerate honest derived surfaces
3. rerun the earliest impacted proof phase
4. run final validations
5. write the verdict note in `TODOS.md`
6. move `staging-live` to the reviewed result and inspect the smoke diff
7. land the reviewed result onto `staging`

Keep the current split topology for now only if all encountered failures were `host_env`,
`local_bug`, `doc_drift`, or `target_specific`, and every required fix stayed localized.

Open transport-topology convergence next only if one or more of these are proven:

- the shared watcher and worker were individually correct, but the worker -> packet -> relay split
  still forced repeated undocumented handoffs
- the same failure pattern would hit both `codex` and `claude_code` because it lives in the shared
  architecture rather than target-specific acquisition
- the smallest honest fix requires a new workflow family, a new schema, or a cross-module redesign
  larger than a localized repair

The final verdict must be written as a dated evidence note under the existing transport-topology
TODO in `TODOS.md`, including:

- execution date
- exercised commit SHA
- target version
- watcher run URL
- worker run URL
- verdict
- short evidence summary

## Stop Conditions

Stop and re-plan immediately if any of the following occur:

- `codex` is not stale and no live queue item exists
- `staging` cannot be aligned with `origin/staging`
- target-version drift occurs mid-run and the proof cannot be restarted cleanly
- generated packet docs cannot be regenerated honestly
- recovery requires packet surgery or branch renaming
- a needed fix becomes topology redesign
- a worker needs a parent-owned file
- `PLAN.md` changes milestone boundaries after `C1`

## Assumptions

- `codex` remains enrolled in the shared maintenance watcher during execution.
- `docs/agents/lifecycle/codex-maintenance/**` is the only committed maintenance root required for
  this milestone.
- `claude_code` is not a second live target; it matters only if the final decision needs to reason
  about whether a failure pattern is shared or target-specific.
- The parent has enough GitHub access to trigger workflows and inspect runs, whether through `gh`
  or the GitHub UI.
- The local execution host can either run the documented Codex lane or fail clearly enough to be
  classified.
