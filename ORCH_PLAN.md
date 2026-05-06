# ORCH_PLAN - Prove The Real Codex Stale-Maintenance Path

## Summary

Authoritative milestone source: `PLAN.md` in the repo root. This file is an execution aid only.

This orchestration replaces the stale relay-implementation plan. The current session must prove the
real path described in `PLAN.md`:

```text
main scheduled workflow definition
  -> scheduled watcher run fires from main
  -> watcher checks out staging
  -> watcher emits stale codex queue entry using runtime truth
  -> watcher dispatches the Codex worker with frozen queue fields
  -> worker checks out staging
  -> worker opens automation/codex-maintenance-<target_version>
  -> parent checks out that generated PR branch
  -> parent runs execute-agent-maintenance --dry-run
  -> parent reruns execute-agent-maintenance --write --run-id <prepared_run_id>
  -> parent captures evidence and leaves close-agent-maintenance manual
  -> parent reverts the temporary cron acceleration on main
```

This is primarily an operational proof, not a feature-build milestone. The parent agent is the
only integrator and the only actor allowed to push `main`, push `staging`, observe the scheduled
run, inspect the generated PR, revert the temporary cron change, run the maintainer proof, and
declare acceptance.

Concurrency policy:

- Live cap: `2` worker lanes, plus the parent.
- Reason: only `staging` readiness and `main` cron prep are worth parallelizing up front. The real
  bottleneck is serialized branch coordination plus GitHub schedule latency. Adding more workers
  before the generated PR exists increases drift and merge overhead without shortening the critical
  path.

## Completion Definition

This session is done only when all of the following are true:

1. Local preflight passed and the local watcher output recorded a stale `codex` entry with
   `dispatch_workflow = codex-cli-update-snapshot.yml`.
2. `staging` was proven or updated to the exact watcher/worker baseline that the scheduled proof is
   meant to exercise.
3. `main` temporarily carried the accelerated cron and then had that acceleration reverted after
   the scheduled proof succeeded.
4. One real `schedule` run fired from `.github/workflows/agent-maintenance-release-watch.yml` on
   `main`.
5. That scheduled watcher checked out `staging`, emitted the stale `codex` queue entry, and
   dispatched the Codex worker using queue-owned data.
6. The worker opened `automation/codex-maintenance-<target_version>`, where `target_version` is
   the runtime watcher value recorded for this session.
7. The generated PR body came from
   `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md` and the required packet files
   exist in the PR.
8. The parent ran `execute-agent-maintenance --dry-run` on that generated PR branch, then reran
   `execute-agent-maintenance --write --run-id <prepared_run_id>`.
9. Dry-run and write-mode evidence was captured under the session `.runs/...` root, and write mode
   stayed inside its declared envelope.
10. `close-agent-maintenance` remained manual and untouched by write mode.
11. The parent recorded watcher evidence, worker evidence, PR evidence, maintainer-proof evidence,
    and the cron-revert reference, then marked acceptance against `PLAN.md`.

## Hard Guards

- `PLAN.md` is the sole milestone authority. If this file conflicts with `PLAN.md`, `PLAN.md` wins.
- `main` owns the scheduled workflow definition, including the temporary cron acceleration and its
  revert.
- `staging` owns the code actually executed by both the watcher and the worker.
- Runtime watcher output is truth for `target_version`. The current expected value is advisory
  only; the emitted queue value wins.
- The generated PR branch name must be `automation/codex-maintenance-<target_version>`.
- Success requires one real `schedule` event. `workflow_dispatch` is debug-only and does not count
  as success.
- Success requires the maintainer proof on the generated PR branch:
  `execute-agent-maintenance --dry-run` first, then
  `execute-agent-maintenance --write --run-id <prepared_run_id>`.
- `close-agent-maintenance` remains manual and out of scope for write mode.
- The temporary cron acceleration must be reverted on `main` after the scheduled watcher succeeds
  and the PR exists, even if local maintainer proof continues afterward.
- The parent is the only integrator. Workers do not merge, do not push `main`, do not push
  `staging`, do not touch the generated automation branch, and do not write orchestration state.
- No worker or parent step may assume unrelated modified files are safe to edit. Only files needed
  for the active lane may be touched.
- Stop immediately if the scheduled watcher does not check out `staging`, if the worker is
  dispatched with fabricated inputs, if the PR body does not come from
  `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md`, or if write mode crosses its
  declared envelope.

## Authority Model

Parent-only authority:

- interprets `PLAN.md` for this session and owns this orchestration file
- creates and maintains `.runs/prove-real-codex-stale-maintenance/**`
- decides lane launch order, lane freeze SHAs, and any relaunch
- is the only actor allowed to push `main`
- is the only actor allowed to push `staging`
- is the only actor allowed to observe GitHub Actions runs and record watcher/worker evidence
- is the only actor allowed to inspect the generated automation PR and decide whether it satisfies
  the packet contract
- is the only actor allowed to check out the generated
  `automation/codex-maintenance-<target_version>` branch locally
- is the only actor allowed to run `execute-agent-maintenance --dry-run`
- is the only actor allowed to run `execute-agent-maintenance --write --run-id <prepared_run_id>`
- is the only actor allowed to revert the temporary cron acceleration on `main`
- is the only actor allowed to declare acceptance or failure for the proof

Worker authority:

- may prepare lane-scoped changes only on the assigned worker branch and worktree
- may run only the lane-scoped validation commands assigned to that lane
- may report `ready-for-parent`, `blocked`, or `no-op`
- may not merge
- may not push `main`
- may not push `staging`
- may not push or touch the generated automation branch
- may not observe or interpret live GitHub run evidence as acceptance proof
- may not inspect the generated PR as the authoritative acceptance actor
- may not run the maintainer proof commands
- may not write `.runs/**`
- may not widen scope beyond the active lane contract

## Run-State Source Of Truth

Parent-owned orchestration root:

```text
/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.runs/prove-real-codex-stale-maintenance
```

Parent initializes it once:

```bash
mkdir -p .runs/prove-real-codex-stale-maintenance/artifacts
```

Required parent-owned records:

- `baseline.json`
  - parent branch, parent SHA, dirty-state summary, `PLAN.md` hash, timestamp
- `freeze.json`
  - lane table, worktree paths, launch SHAs, stop conditions, acceptance gates
- `lane-status.json`
  - lane status: `pending|running|blocked|ready-for-parent|merged|no-op|aborted`
- `artifacts/local-watch.json`
  - local `maintenance-watch` output captured during baseline
- `artifacts/watcher-run.md`
  - scheduled run URL, event type, checkout ref proof, queue excerpt
- `artifacts/worker-run.md`
  - worker run URL, dispatch inputs, result
- `artifacts/pr.md`
  - PR URL, branch name, required files, PR body source verification
- `artifacts/maintainer-proof.md`
  - exact dry-run command, `run_id`, exact write command, result, diff summary
- `artifacts/cron-revert.md`
  - revert commit/PR reference on `main`
- `acceptance.md`
  - success criteria checklist mapped directly to `PLAN.md`

Workers never write under `.runs/prove-real-codex-stale-maintenance`.

## Workstream Plan

### Branch And Worktree Layout

Repo root:

```text
/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api
```

Shared worktree root:

```text
/Users/spensermcconnell/__Active_Code/atomize-hq/wt
```

Parent live worktrees:

- `main`: `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-main-live`
- `staging`: `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-staging-live`
- generated PR branch:
  `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-proof-pr-<target_version>`

Worker lanes:

| Lane | Purpose | Base ref | Worker branch | Worktree |
| --- | --- | --- | --- | --- |
| A | prove or land the exact watcher/worker baseline onto `staging` | `origin/staging` | `codex/recommend-next-agent-orch-a-staging-proof` | `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-orch-a-staging-proof` |
| B | prepare the temporary `main` cron acceleration and paired revert | `origin/main` | `codex/recommend-next-agent-orch-b-main-cron` | `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-orch-b-main-cron` |

Valid parent live-worktree setup forms:

- If local branches `main` and `staging` already exist and track the intended remotes:

```bash
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-staging-live staging
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-main-live main
```

- If local branches do not yet exist, create them explicitly from the remote-tracking refs:

```bash
git worktree add -b staging /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-staging-live origin/staging
git worktree add -b main /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-main-live origin/main
```

Worker-lane setup commands:

```bash
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-orch-a-staging-proof -b codex/recommend-next-agent-orch-a-staging-proof origin/staging
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-orch-b-main-cron -b codex/recommend-next-agent-orch-b-main-cron origin/main
```

When the PR exists, parent adds the proof branch worktree:

```bash
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-proof-pr-<target_version> automation/codex-maintenance-<target_version>
```

### Lane Breakdown

#### Parent P0 - Baseline Capture And Freeze

Parent-only. No workers yet.

Actions:

1. Record current baseline in `.runs/.../baseline.json`.
2. Hash `PLAN.md` and record it in `.runs/.../baseline.json`.
3. Record that the old relay-implementation `ORCH_PLAN.md` is rejected as scope authority.
4. Run the local preflight that `PLAN.md` requires before any branch choreography:

```bash
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_execute
cargo run -p xtask -- maintenance-watch --emit-json _ci_tmp/maintenance-watch.json
```

5. Copy `_ci_tmp/maintenance-watch.json` into
   `.runs/prove-real-codex-stale-maintenance/artifacts/local-watch.json`.
6. Freeze the emitted `current_validated`, `dispatch_workflow`, and runtime `target_version` in
   `.runs/.../freeze.json`.

P0 stopping conditions:

- the local watcher output has no stale `codex` entry
- the output does not point at `codex-cli-update-snapshot.yml`
- the output branch name does not match `automation/codex-maintenance-<target_version>`
- any of the three required `xtask` suites fail

#### Lane A - `staging` Proof Baseline

Delegatable. Parent integrates.

Mission:

- prove that `staging` already contains the required proof baseline from `PLAN.md`, or
- land the minimum missing commits onto a worker branch rooted from `origin/staging` so the parent
  can integrate them onto `staging`

Owned surfaces:

- `crates/xtask/src/agent_maintenance/watch.rs`
- `crates/xtask/src/agent_maintenance/request.rs`
- `crates/xtask/src/agent_maintenance/prepare.rs`
- `crates/xtask/src/agent_maintenance/execute.rs`
- `.github/workflows/codex-cli-update-snapshot.yml`
- `.github/workflows/agent-maintenance-open-pr.yml`

Explicitly forbidden:

- `.github/workflows/agent-maintenance-release-watch.yml`
- `PLAN.md`
- `ORCH_PLAN.md`
- `.runs/**`
- the generated automation PR branch

Lane A required validation:

```bash
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_execute
```

Lane A handoff must include:

- lane status: `ready-for-parent`, `blocked`, or `no-op`
- lane base SHA from `origin/staging` at launch time
- whether `staging` was already sufficient or required a patch
- exact commits needed from `codex/recommend-next-agent` if cherry-picks are required
- exact files changed
- exact commands run
- exact tests run
- unresolved risks or assumptions, if any

Acceptance gate for Lane A:

- parent can state, with SHA evidence, that the scheduled watcher and dispatched worker will
  execute the intended maintenance code from `staging`

#### Lane B - `main` Cron Acceleration And Revert Prep

Delegatable. Parent integrates only after Lane A is live on `staging`.

Mission:

- prepare the temporary schedule acceleration on `main`
- prepare the paired revert so cleanup is immediate after the scheduled proof succeeds

Owned surface:

- `.github/workflows/agent-maintenance-release-watch.yml`

Exact allowed change:

- replace the normal cron with a temporary every-5-minutes off-peak schedule such as
  `3-58/5 * * * *`

Explicitly forbidden:

- changing checkout from `staging`
- changing dispatch logic
- changing queue shape
- changing any worker workflow
- touching `PLAN.md`, `ORCH_PLAN.md`, or `.runs/**`

Lane B required validation:

- inspect the workflow diff and prove only the schedule changed

Lane B handoff must include:

- lane status: `ready-for-parent`, `blocked`, or `no-op`
- lane base SHA from `origin/main` at launch time
- the acceleration commit
- the revert commit or exact reverse patch prepared from the same worker branch
- exact files changed
- exact commands run
- confirmation that `workflow_dispatch` remains present only for debugging
- unresolved risks or assumptions, if any

Acceptance gate for Lane B:

- parent has a minimal `main` patch ready to accelerate the real schedule and a clean, immediate
  revert path

### Worker Handoff Contract

Every worker lane returns one compact handoff packet to the parent containing exactly:

- lane id
- lane status: `ready-for-parent`, `blocked`, or `no-op`
- launch base ref and exact base SHA
- changed files, or explicit statement that nothing changed
- exact commands run
- exact validation result summary
- commit SHA to review, if changes exist
- unresolved risks, assumptions, or follow-up notes

Parent live-context rule:

- the parent retains only the compact worker handoff summary in active context
- the parent drops worker scratch reasoning after extracting branch SHA, changed files, commands
  run, status, and unresolved risks
- the authoritative durable record is the parent-written `.runs/...` state root, not the worker’s
  local notes

#### Parent O1 - Live Branch Integration Gate

Parent-only.

Actions:

1. Integrate Lane A onto `staging` or mark it `no-op` if `staging` was already correct.
2. Push `staging` live.
3. Re-verify the `staging` worktree at the pushed SHA.
4. Only then integrate Lane B onto `main`.
5. Push the temporary cron acceleration live on `main`.
6. Record both pushed SHAs in `.runs/.../freeze.json`.

Parent O1 stopping conditions:

- `staging` is not ready when the `main` cron acceleration would go live
- the `main` patch includes anything beyond the temporary schedule edit
- another actor changes `main` or `staging` in a way that affects the maintenance path before the
  parent pushes

#### Parent O2 - Scheduled Watcher Observation

Parent-only. This is the first real proof gate.

Actions:

1. Wait for a real `schedule` event on
   `.github/workflows/agent-maintenance-release-watch.yml`.
2. Observe the watcher run and capture evidence.
3. Confirm the queue contains the stale `codex` entry with runtime truth.
4. Confirm the dispatch job calls `codex-cli-update-snapshot.yml` against `staging`.

Helpful commands:

```bash
gh run list --workflow agent-maintenance-release-watch.yml --event schedule --branch main --limit 5
gh run view <watcher_run_id> --log
```

Required assertions:

- event is `schedule`
- workflow definition is from `main`
- checkout ref is `staging`
- emitted `current_validated` is `0.97.0`
- emitted `version_policy` is `latest_stable_minus_one`
- emitted `dispatch_workflow` is `codex-cli-update-snapshot.yml`
- emitted branch name is `automation/codex-maintenance-<target_version>`
- runtime emitted `target_version` is recorded as truth for the rest of the session

Parent O2 stopping conditions:

- no scheduled run appears within the bounded retry window
- the queue does not include stale `codex`
- dispatch is attempted with hand-fabricated inputs or the wrong ref

#### Parent O3 - Worker Run, PR Inspection, And Cron Revert

Parent-only.

Actions:

1. Observe the dispatched worker run.
2. Confirm the PR exists and the branch matches the emitted queue value.
3. Confirm the PR body comes from
   `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md`.
4. Confirm the PR includes:
   - `docs/agents/lifecycle/codex-maintenance/HANDOFF.md`
   - `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
   - `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md`
5. As soon as the watcher succeeded, the worker succeeded, and the PR exists, revert the temporary
   cron acceleration on `main`.

Helpful commands:

```bash
gh run view <worker_run_id> --log
gh pr list --head automation/codex-maintenance-<target_version> --state open
gh pr view <pr_number> --json url,headRefName,body,files
```

Required assertions:

- PR head branch is `automation/codex-maintenance-<target_version>`
- request + packet agree on target version and manual closeout boundary
- `close-agent-maintenance` has not run
- the revert reaches `main` before the session proceeds to the local maintainer proof

Parent O3 stopping conditions:

- worker run fails before PR creation
- PR body source is wrong
- required packet files are missing
- cron revert cannot be applied cleanly on `main`

#### Parent O4 - Maintainer Proof On The Generated PR Branch

Parent-only.

Actions:

1. Add the generated automation branch as its own worktree.
2. Run the exact dry-run command from repo root in that worktree:

```bash
cargo run -p xtask -- execute-agent-maintenance \
  --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml \
  --dry-run
```

3. Discover the newest `run_id` under
   `docs/agents/.uaa-temp/agent-maintenance/runs/`.
4. Re-run with the same `run_id`:

```bash
cargo run -p xtask -- execute-agent-maintenance \
  --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml \
  --write \
  --run-id "$RUN_ID"
```

5. Capture the resulting diff summary and artifact paths in `.runs/.../artifacts/maintainer-proof.md`.

Required assertions:

- dry-run writes only under `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`
- the prepared packet includes the expected run artifacts from `PLAN.md`
- write mode succeeds using the prepared `run_id`
- write mode stays inside the declared write envelope
- `maintenance-closeout.json` is not created or mutated
- the flow stops before `close-agent-maintenance`

Parent O4 stopping conditions:

- dry-run fails
- `run_id` cannot be identified cleanly
- write mode reconstructs state instead of reusing the prepared run
- any write escapes the request-owned boundary

#### Parent O5 - Evidence And Acceptance

Parent-only.

Actions:

1. Finalize all records under `.runs/prove-real-codex-stale-maintenance`.
2. Write `acceptance.md` against the numbered success criteria in `PLAN.md`.
3. Record whether the generated PR remains open for normal maintainer follow-through or is being
   treated as validation-only.
4. Leave `close-agent-maintenance` untouched.

Exit condition:

- cron acceleration is reverted on `main`
- watcher evidence, worker evidence, PR evidence, and maintainer-proof evidence are all captured
- the manual closeout boundary is still intact

## Parent Critical Path

Serialized gates the parent must enforce:

```text
P0 baseline capture and local preflight
-> launch Lane A and Lane B in parallel
-> Gate 1: Lane A integrated or proven no-op on staging
-> Gate 2: only after Gate 1, Lane B integrated onto main
-> Gate 3: wait for one real scheduled watcher run from main
-> Gate 4: confirm worker run and generated PR
-> Gate 5: revert temporary cron on main immediately
-> Gate 6: check out automation/codex-maintenance-<target_version>
-> Gate 7: run --dry-run, then --write --run-id <prepared_run_id>
-> Gate 8: capture evidence and declare acceptance
```

Critical path rules:

- Lane B can be prepared early, but it cannot be merged until Lane A is live on `staging`.
- No worker lane is launched for the generated PR branch before the branch actually exists.
- Parent does not start the maintainer proof until the cron revert is already pushed to `main`.
- If the scheduled run fails for environmental reasons, parent captures evidence first, then opens
  a new defect loop from the failing branch. The failed run still does not count as success.

## Merge And Integration Policy

- Workers branch from the lane base ref listed above and commit only lane-owned changes.
- Parent is the only integrator and uses one of:
  - `git cherry-pick -x <worker_commit>` into the live `main` or `staging` worktree
  - manual reapplication by the parent if drift makes cherry-pick unsafe
- Workers never merge each other.
- Lane A integrates only into `staging`.
- Lane B integrates only into `main`.
- The generated branch `automation/codex-maintenance-<target_version>` is never hand-authored by a
  worker. It must come from the real worker run.
- If a lane becomes unnecessary because the target branch already matches the proof baseline, the
  parent marks it `no-op` in `lane-status.json` instead of forcing a cosmetic merge.
- If Lane A is `no-op`, the parent records the inspected `staging` SHA in `lane-status.json` and
  proceeds directly to the `staging` push/re-verify gate without creating a cosmetic branch merge.
- If Lane B is `no-op` because there is no safe or necessary cron patch to land yet, the parent
  records that status in `lane-status.json` and does not advance to scheduled proof until a valid
  temporary acceleration patch exists.
- If the scheduled run or maintainer proof uncovers a real defect, parent creates a new repair lane
  from the branch that actually failed, updates `.runs/.../freeze.json`, and relaunches only the
  minimal necessary repair scope.

## Relaunch / Restart Rules

- If `staging` changes in a way that affects the proof baseline after Lane A launches but before the
  parent pushes or marks it `no-op`, Lane A is stale. Parent records the new `staging` SHA in
  `.runs/.../freeze.json`, marks Lane A `aborted`, and relaunches Lane A from the new `origin/staging`
  base.
- If `main` changes in a way that affects `.github/workflows/agent-maintenance-release-watch.yml`
  after Lane B launches but before the parent pushes, Lane B is stale. Parent records the new
  `main` SHA, marks Lane B `aborted`, and relaunches Lane B from the new `origin/main` base.
- If the scheduled run fails for environmental reasons, such as GitHub schedule delay, transient
  runner failure, or temporary Actions platform issues, parent does not relaunch Lane A or Lane B
  automatically. Parent records the failed run, keeps the same branch freeze if still valid, and
  retries the operational wait window or reruns only the minimum parent-owned operational step.
- If the scheduled run fails for contract reasons, such as wrong checkout ref, wrong queue fields,
  wrong dispatch workflow, or missing PR artifacts, parent records the failing run as proof of a
  real defect, freezes the failing SHA pair from `main` and `staging`, and launches the minimum
  repair lane from the branch that actually owns the defect.
- If the generated PR branch target version differs from the earlier expected advisory value, the
  runtime watcher value becomes the new session truth immediately. No existing worker lane is
  relaunched solely because the advisory expected value changed, because Lane A and Lane B are not
  allowed to hard-code the target version.
- If the generated PR branch already exists before the current scheduled proof and the new worker
  run reuses it with the runtime watcher target, parent records that fact in `.runs/.../artifacts/pr.md`
  and continues only if the run evidence still proves the current session path end-to-end.

## Context-Control Rules

- Parent context stays anchored to `PLAN.md`, this orchestration file, the `.runs/...` state root,
  and the currently pushed SHAs for `main`, `staging`, and the generated automation branch.
- Workers receive only:
  - the relevant `PLAN.md` success criteria
  - the branch topology
  - the lane-owned file list
  - the lane base ref and worktree path
  - the exact validation commands
  - the hard forbidden surfaces
- Workers are not given the stale prior milestone as design input except as a warning that it is
  obsolete.
- Parent strips worker-local reasoning after handoff and keeps only: branch SHA, files touched,
  commands run, and unresolved risks.
- No worker may hard-code a target version. All target-version references must be phrased as
  `automation/codex-maintenance-<target_version>` until the parent records the runtime watcher
  output.
- Parent-only data:
  - GitHub run URLs
  - PR URL
  - pushed SHAs on `main` and `staging`
  - maintainer `run_id`
  - `.runs/**`

## Tests And Acceptance

### Required Local Preflight Before Live Branch Changes

Run from the repo root on the parent branch:

```bash
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_execute
cargo run -p xtask -- maintenance-watch --emit-json _ci_tmp/maintenance-watch.json
```

Acceptance:

- the three suites pass
- local watcher output contains stale `codex`
- emitted branch name matches `automation/codex-maintenance-<target_version>`

### `staging` Readiness Acceptance

- the pushed `staging` SHA contains the proof baseline from `PLAN.md` or a deliberate descendant
- watcher and worker surfaces on `staging` match what local preflight validated

### `main` Schedule Acceptance

- only the cron schedule changed
- `workflow_dispatch` remains debug-only
- checkout remains `staging`

### Scheduled Proof Acceptance

- one `schedule` run fired from the `main` workflow definition
- watcher checked out `staging`
- emitted queue entry matches `PLAN.md`
- dispatched worker was `codex-cli-update-snapshot.yml`
- runtime watcher output is recorded as target-version truth

### Generated PR Acceptance

- PR head branch is `automation/codex-maintenance-<target_version>`
- PR body comes from `governance/pr-summary.md`
- required packet files are present
- manual closeout boundary is still visible and intact

### Maintainer Proof Acceptance

- `execute-agent-maintenance --dry-run` succeeds first
- `execute-agent-maintenance --write --run-id <prepared_run_id>` succeeds second
- dry-run artifact root is under `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`
- write mode stays inside the declared envelope
- `close-agent-maintenance` is not run

### Final Session Acceptance

- `main` temporary cron acceleration is reverted
- `.runs/prove-real-codex-stale-maintenance` contains enough evidence for replay
- success criteria `1` through `11` from `PLAN.md` are checked off explicitly in `acceptance.md`

## Assumptions

- the parent can push to `main` and `staging`, or can obtain the required approvals without
  changing the branch topology described here
- `gh` CLI is authenticated for viewing Actions runs and PR metadata; GitHub UI is acceptable as a
  fallback, but the same evidence must still be copied into `.runs/...`
- `staging` and `main` will not receive conflicting workflow or maintenance-path edits during the
  proof window; if they do, the parent pauses and refreshes the freeze state before continuing
- the scheduled watcher is allowed a bounded delay window by GitHub Actions; this is a wait state,
  not a reason to widen scope
- the generated PR branch remains open after proof unless a human maintainer explicitly decides it
  is validation-only
