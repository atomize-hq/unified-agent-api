# PLAN - Prove The Real Codex Stale-Maintenance Path

Status: proposed
Date: 2026-05-06
Branch: `codex/recommend-next-agent`
Base branch: `main`
Repo: `atomize-hq/unified-agent-api`
Work item: `Validate the real shared-watcher -> Codex maintenance PR -> maintainer relay path`
Plan commit baseline: `75aa237`
Design input: `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-design-20260506-091624.md`
Review addendum: `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-eng-review-test-plan-20260506-092335.md`
Supersedes: the 2026-05-05 relay-implementation `PLAN.md`. That plan mostly described work that is already landed on this branch. This plan is the honest next seam: prove the live path.

## Objective

Prove one boring, real, end-to-end Codex maintenance path:

```text
GitHub schedule on main
  -> shared watcher runs from default-branch workflow definition
  -> watcher checks out staging
  -> maintenance-watch emits stale codex queue entry
  -> watcher dispatches codex worker with frozen queue fields
  -> worker checks out staging
  -> worker opens automation/codex-maintenance-<target_version> PR from generated packet docs
  -> maintainer opens the PR, reads HANDOFF.md, runs execute-agent-maintenance --dry-run
  -> maintainer reruns execute-agent-maintenance --write --run-id <prepared_run_id>
  -> maintainer reviews the diff
  -> flow stops before close-agent-maintenance
```

Success is not "the code looks ready." Success is one real scheduled proof with captured evidence, with the manual closeout boundary still intact.

## Success Criteria

1. A `schedule` event on `.github/workflows/agent-maintenance-release-watch.yml` runs from the default branch workflow definition.
2. That scheduled watcher run checks out `staging`, not the triggering ref.
3. The queue job emits a stale `codex` entry with:
   - `current_validated = 0.97.0`
   - `version_policy = latest_stable_minus_one`
   - `dispatch_workflow = codex-cli-update-snapshot.yml`
   - `branch_name = automation/codex-maintenance-<target_version>`
4. The emitted `target_version` matches runtime release truth on the day of the run. The current expected value is `0.127.0`, but runtime watcher output wins.
5. The dispatch job calls the Codex worker without hand-fabricated downstream inputs.
6. The Codex worker opens a PR whose body comes from `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md`.
7. That PR contains:
   - `docs/agents/lifecycle/codex-maintenance/HANDOFF.md`
   - `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
   - `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md`
8. A maintainer can run `execute-agent-maintenance --dry-run` from repo root on the PR branch and obtain a prepared run packet under `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`.
9. A maintainer can run `execute-agent-maintenance --write --run-id <prepared_run_id>` and the relay stays inside its declared write envelope.
10. No automatic closeout occurs. `close-agent-maintenance` remains manual and untouched.
11. The temporary cron acceleration on `main` is reverted after the scheduled proof succeeds and the PR exists.

## Current Truth This Plan Locks In

These are the facts this plan treats as authoritative:

1. Scheduled GitHub Actions workflows run from the default branch workflow definition, not this feature branch. A cron-only tweak on `codex/recommend-next-agent` does not prove the real path.
2. `.github/workflows/agent-maintenance-release-watch.yml` currently checks out `staging`.
3. `.github/workflows/codex-cli-update-snapshot.yml` also checks out `staging`, even when dispatched from another ref.
4. `cli_manifests/codex/latest_validated.txt` is currently `0.97.0`.
5. `crates/xtask/data/agent_registry.toml` records Codex maintenance policy as `version_policy = "latest_stable_minus_one"` with `dispatch_workflow = "codex-cli-update-snapshot.yml"`.
6. On 2026-05-06, the preflight release check saw stable upstream tag `rust-v0.128.0`, so the current expected target is `0.127.0`. That value is advisory only. The scheduled watcher output is the runtime truth.
7. Manual closeout is still the trust boundary. `execute-agent-maintenance --write` must stop before `close-agent-maintenance`.

## Step 0 Scope Challenge

### What This Plan Is

- one execution plan for the real scheduled watcher path
- one explicit branch-coordination story across `main`, `staging`, and the generated automation branch
- one proof that the generated PR and the local maintainer relay compose correctly
- one cleanup path that removes the temporary schedule acceleration as soon as the scheduled proof is done

### What This Plan Is Not

- another relay feature implementation plan
- a `workflow_dispatch` success story disguised as scheduled proof
- a worker-only proof that skips the stale watcher
- a version-policy redesign
- an automatic closeout plan
- a new maintenance architecture

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| stale-agent detection | `crates/xtask/src/agent_maintenance/watch.rs` | Reuse exactly. This plan proves it, it does not redesign it. |
| queue field freezing | `crates/xtask/tests/agent_maintenance_watch.rs` | Reuse as local proof that expected queue fields are already encoded. |
| request + execution contract truth | `crates/xtask/src/agent_maintenance/request.rs` | Reuse exactly. The live proof must consume this request surface, not a synthetic packet. |
| packet generation | `crates/xtask/src/agent_maintenance/prepare.rs` | Reuse exactly. The worker already writes the packet and request from repo-owned truth. |
| packet rendering consistency | `crates/xtask/tests/agent_maintenance_prepare.rs` | Reuse as fail-closed proof for `HANDOFF.md`, `pr-summary.md`, and prompt digest lockstep. |
| maintainer relay | `crates/xtask/src/agent_maintenance/execute.rs` | Reuse exactly. The proof must run this command, not an ad hoc substitute. |
| relay boundary + gate coverage | `crates/xtask/tests/agent_maintenance_execute.rs` | Reuse as local proof that path jail, prompt digest, and manual closeout boundary already exist. |
| scheduled watcher | `.github/workflows/agent-maintenance-release-watch.yml` | Reuse with one temporary cron acceleration only. |
| worker PR creation | `.github/workflows/codex-cli-update-snapshot.yml` and `.github/workflows/agent-maintenance-open-pr.yml` | Reuse exactly. The live proof must dispatch the existing workflow path. |
| maintainer operating contract | `cli_manifests/codex/OPS_PLAYBOOK.md` | Reuse as the human-facing policy and replay surface. |

### Minimum Complete Change

The minimum complete plan is:

1. run local automated preflight against the already-landed maintenance code
2. make the proof baseline real on `staging`
3. temporarily accelerate the watcher cron on `main`
4. wait for one real scheduled run
5. inspect the generated Codex maintenance PR
6. run maintainer `--dry-run` and `--write` on that PR branch
7. capture evidence and revert the temporary cron tweak

Anything smaller proves less than the repo claims.

### Complexity Check

This plan is operationally multi-branch, but permanent code-touch complexity is low:

- one temporary workflow schedule change on `main`
- no new services
- no new long-lived abstractions
- no new permanent automation unless the live proof exposes a defect

That is the right shape. Boring by default.

### Search / Build Decision

- **[Layer 1]** Accept GitHub's native `schedule` semantics. Do not invent a branch-local workaround.
- **[Layer 1]** Reuse the shared watcher queue and worker dispatch path. Do not create a proof-only trigger.
- **[Layer 1]** Reuse `execute-agent-maintenance --dry-run|--write` as the only maintainer execution surface.
- **[Layer 3]** The real bug was not missing relay code. It was reasoning about branch-local cron edits as if scheduled workflows honored them. They do not.

### Distribution Check

No new distributable artifact is introduced here.

The deliverables are:

- one scheduled GitHub Actions watcher run
- one real Codex maintenance PR
- one maintainer relay evidence run under `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`

## Locked Decisions

1. The success path is a real `schedule` event, not `workflow_dispatch`.
2. `workflow_dispatch` is allowed only for debugging after a scheduled failure. It does not satisfy success criteria.
3. The proof must respect the repo's real branch topology:
   - `main` owns the scheduled workflow definition
   - `staging` is the code the watcher and worker actually execute
   - `automation/codex-maintenance-<target_version>` is the generated maintainer branch
4. The target version is captured from live queue output at execution time. The plan does not hard-code `0.120.0` or any other stale design value.
5. The shared watcher and Codex worker remain unchanged unless the proof exposes a real defect.
6. The generated maintenance PR is the maintainer entrypoint. No synthetic hand-authored packet is allowed.
7. `execute-agent-maintenance --dry-run` is mandatory before `--write`.
8. `execute-agent-maintenance --write` must reuse one prepared `run_id`.
9. The relay stops before `close-agent-maintenance`.
10. The temporary cron acceleration is reverted immediately after the scheduled watcher and worker proof are complete and the PR exists. The local maintainer proof continues after the revert.
11. The generated PR stays open by default after proof. It is a real maintenance branch unless a maintainer explicitly declares it validation-only.

## Architecture

### End-To-End Proof Flow

```text
main
  -> .github/workflows/agent-maintenance-release-watch.yml
  -> schedule event fires
  -> watcher job checks out staging
  -> cargo run -p xtask -- maintenance-watch --emit-json _ci_tmp/maintenance-watch.json
  -> queue entry for codex
       current_validated = 0.97.0
       latest_stable = <runtime truth>
       target_version = <runtime truth - 1>
       dispatch_workflow = codex-cli-update-snapshot.yml
  -> actions.createWorkflowDispatch(workflow_id = codex-cli-update-snapshot.yml, ref = staging)
  -> worker job checks out staging
  -> prepare-agent-maintenance --write
  -> writes maintenance-request.toml + HANDOFF.md + pr-summary.md
  -> opens automation/codex-maintenance-<target_version> PR
  -> maintainer checks out PR branch locally
  -> execute-agent-maintenance --dry-run
  -> capture run_id + prepared packet
  -> execute-agent-maintenance --write --run-id <same run_id>
  -> green gates pass
  -> maintainer reviews diff
  -> stop before close-agent-maintenance
```

### Branch Topology

```text
main
  owns:
    - scheduled workflow definition
    - temporary cron acceleration commit
    - cron revert commit

staging
  owns:
    - watcher code actually executed by the scheduled run
    - worker code actually executed by the dispatched run
    - proof baseline commit or deliberate descendant

automation/codex-maintenance-<target_version>
  owns:
    - generated maintenance packet
    - generated PR body
    - maintainer relay branch
```

If `staging` does not contain the proof baseline, the scheduled run is the wrong proof. Full stop.

### Control Boundaries

| Surface | Allowed responsibility | Must not do |
| --- | --- | --- |
| watcher workflow | trigger on schedule, build queue, dispatch worker | bypass queue generation, invent worker-only data |
| `maintenance-watch` | compute stale queue from registry + upstream truth | fabricate proof-only target versions |
| worker workflow | refresh parity artifacts, render packet docs, open PR | execute maintainer relay or closeout |
| `prepare-agent-maintenance` | write request truth and packet docs | mutate maintainer-owned change surfaces |
| `execute-agent-maintenance` | validate preflight, prepare dry-run packet, perform bounded write run | perform automatic closeout |
| `close-agent-maintenance` | explicit post-review human attestation | be folded into write mode |

## Execution Plan

### Phase 1. Local Preflight And Runtime Truth Capture

Purpose: prove the landed maintenance surfaces are green locally before touching GitHub scheduling.

Owner branch/worktree: current feature branch or a throwaway local worktree.

Commands:

```bash
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_execute
cargo run -p xtask -- maintenance-watch --emit-json _ci_tmp/maintenance-watch.json
```

Required outputs:

1. `_ci_tmp/maintenance-watch.json`
2. local proof that the three maintenance suites are green
3. captured runtime watcher target from emitted queue JSON

Acceptance:

1. `_ci_tmp/maintenance-watch.json` contains a `codex` stale-agent entry.
2. `current_validated` is `0.97.0`.
3. `dispatch_workflow` is `codex-cli-update-snapshot.yml`.
4. `branch_name` is `automation/codex-maintenance-<target_version>`.
5. `target_version` is recorded from runtime watcher output. If it differs from `0.127.0`, the emitted value becomes plan truth for the rest of the run.
6. No test failures exist in the three suites above.

Failure handling:

- If tests fail, stop and fix the repo-owned blocker first.
- If the watcher emits a different live target than expected, update the proof record and continue. Do not change code just to match the stale design doc.
- If no stale `codex` entry appears, stop and inspect `latest_validated.txt`, registry metadata, and live upstream release truth before touching GitHub workflows.

### Phase 2. Make The Proof Baseline Real On `staging`

Purpose: ensure the scheduled run will execute the code we actually mean to prove.

Owner branch/worktree: dedicated `staging` prep worktree.

Required actions:

1. Treat `75aa237` as the baseline commit for this proof.
2. Ensure `staging` contains that baseline or a deliberate descendant with the same maintenance surfaces.
3. Verify on `staging`:
   - `crates/xtask/src/agent_maintenance/watch.rs`
   - `crates/xtask/src/agent_maintenance/request.rs`
   - `crates/xtask/src/agent_maintenance/prepare.rs`
   - `crates/xtask/src/agent_maintenance/execute.rs`
   - `.github/workflows/codex-cli-update-snapshot.yml`
   - `.github/workflows/agent-maintenance-open-pr.yml`

Acceptance:

1. `staging` contains the exact proof baseline or an intentional descendant.
2. The code and workflow surfaces above match what Phase 1 validated locally.
3. There is no remaining ambiguity about which ref the watcher and worker will execute.

Failure handling:

- If `staging` is missing proof code, land the minimum required baseline there before attempting the scheduled run.
- If `staging` contains unrelated drift that changes maintenance behavior, stop and restate the proof baseline explicitly before continuing.

### Phase 3. Temporarily Accelerate The Scheduled Watcher On `main`

Purpose: get one real scheduled run quickly, without waiting for the normal daily cron.

Owner branch/worktree: dedicated `main` cron-tweak worktree.

Touch surface:

- `.github/workflows/agent-maintenance-release-watch.yml`

Required change:

- replace the current cron `17 3 * * *`
- with a temporary off-peak every-5-minutes schedule such as `3-58/5 * * * *`

Rules:

1. Merge this change to `main`, not the feature branch.
2. Keep `workflow_dispatch` enabled for debugging, but do not use it as the success path.
3. Do not change `actions/checkout` ref. `staging` checkout is part of the proof.
4. Do not change queue-shape or dispatch logic while doing the cron acceleration.

Acceptance:

1. `main` now hosts the temporary accelerated schedule.
2. The next scheduled run should happen within 5 minutes, subject to GitHub delay.
3. `staging` is already ready before this merge happens.

Failure handling:

- If cron acceleration is merged before `staging` is ready, back out and restart with proper branch sequencing.
- If GitHub scheduling is delayed, keep the temporary cron in place for one bounded retry window before investigating platform delay.

### Phase 4. Observe The Real Scheduled Watcher Run

Purpose: capture proof that the alarm rang by itself.

Evidence to capture from the watcher run:

1. watcher workflow run URL
2. queue job logs
3. emitted queue JSON excerpt for the `codex` entry
4. dispatch job logs showing:
   - `agent_id = codex`
   - `dispatch_workflow = codex-cli-update-snapshot.yml`
   - `dispatch_ref = staging`
   - `target_version = <runtime watcher value>`
   - `branch_name = automation/codex-maintenance-<target_version>`

Required assertions:

1. the run was triggered by `schedule`
2. the workflow definition came from `main`
3. the job checked out `staging`
4. the watcher chose the live `latest_stable_minus_one` target
5. the dispatch job succeeded

Failure handling:

- If no scheduled run appears, leave the accelerated cron in place for one bounded retry window, then inspect GitHub scheduling delay.
- If the queue is empty for `codex`, stop and inspect live release truth plus `cli_manifests/codex/latest_validated.txt`.
- If dispatch fails, treat that as the first real defect. Capture logs before changing anything.

### Phase 5. Inspect The Generated Codex Maintenance PR

Purpose: verify the worker opened the PR from repo-owned packet truth.

Evidence to capture:

1. worker workflow run URL
2. PR URL
3. PR head branch
4. file list and body source

Required assertions:

1. the PR branch is `automation/codex-maintenance-<target_version>`
2. the PR body matches `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md`
3. the PR contains:
   - `docs/agents/lifecycle/codex-maintenance/HANDOFF.md`
   - `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
   - `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md`
4. the request includes an `[execution_contract]` table
5. the request and packet agree on:
   - target version
   - writable surfaces
   - ordered commands
   - manual closeout boundary

Success handling:

- As soon as the watcher run succeeded, the worker run succeeded, and the PR exists, revert the temporary cron acceleration on `main`.
- After that revert lands, continue with the local maintainer proof.

Failure handling:

- If packet generation succeeded but PR creation failed, follow the repo-owned recovery path already encoded in `.github/workflows/agent-maintenance-open-pr.yml`.
- That recovery path is a valid repair path for this phase. It is not a substitute for the scheduled watcher proof in Phase 4.

### Phase 6. Prove The Maintainer Path On The PR Branch

Purpose: prove the human handoff the PR is actually for.

Owner branch/worktree: dedicated local checkout of `automation/codex-maintenance-<target_version>`.

Required commands from repo root on the generated PR branch:

```bash
cargo run -p xtask -- execute-agent-maintenance \
  --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml \
  --dry-run
```

Then:

1. find the newest directory under `docs/agents/.uaa-temp/agent-maintenance/runs/`
2. set `RUN_ID` to that directory name
3. rerun:

```bash
cargo run -p xtask -- execute-agent-maintenance \
  --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml \
  --write \
  --run-id "$RUN_ID"
```

Required assertions:

1. dry-run writes only under `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`
2. the prepared packet contains:
   - `input-contract.json`
   - `codex-prompt.md`
   - `run-status.json`
   - `run-summary.md`
   - `validation-report.json`
   - `written-paths.json`
3. write mode succeeds without boundary violations
4. the request-owned green gates pass
5. the resulting diff stays inside the declared write envelope
6. `maintenance-closeout.json` is not created or mutated by this step
7. no automatic `close-agent-maintenance` occurs

Failure handling:

- If local Codex preflight fails, fix local binary/auth and rerun dry-run. Do not force write mode.
- If write mode fails path validation or prompt digest validation, treat that as a real defect in the relay contract and capture the failure packet before retrying.

### Phase 7. Capture Final Evidence And Leave The Repo Clean

Purpose: leave behind proof, not operational debt.

Required outputs:

1. scheduled watcher run URL
2. worker run URL
3. PR URL
4. captured watcher queue excerpt
5. captured `run_id` path used for maintainer proof
6. final diff summary from the PR branch after `--write`
7. commit or PR reference that reverted the temporary cron acceleration on `main`
8. one explicit note stating whether the PR remains open for normal maintainer follow-through or is being treated as validation-only

Exit condition:

1. the temporary cron tweak is gone from `main`
2. the proof evidence above is recorded
3. the manual closeout boundary is still intact
4. no silent follow-up debt was created

## Engineering Review Consolidation

### Architecture Review

The architecture is sound if these boundaries stay intact:

1. GitHub schedule triggers the watcher. It does not bypass queue generation.
2. `maintenance-watch` computes staleness. It does not fabricate worker-only state.
3. `prepare-agent-maintenance` writes request truth and packet docs. It does not execute maintainer changes.
4. `execute-agent-maintenance` executes the maintainer relay. It does not perform closeout.
5. `close-agent-maintenance` remains explicit human attestation after diff review.

This matters because the user-facing product is not "we can generate files." The product is "the repo rings the bell, opens the right PR, and hands the maintainer one safe path."

### Code Quality Review

The main code-quality risk is not duplication. It is humans bypassing the repo-owned surfaces because the proof plan is vague.

This plan therefore chooses:

- one scheduled watcher path
- one generated PR path
- one canonical `HANDOFF.md`
- one `execute-agent-maintenance` relay
- one manual closeout boundary

That is explicit over clever. It also keeps the permanent diff small because no new architecture is allowed unless the live proof exposes a real defect.

### Performance Review

Performance is not the gating concern here. Timing and coordination are.

The only performance-sensitive choice in this plan is the temporary accelerated cron:

- use an off-peak every-5-minutes schedule
- avoid top-of-hour spikes
- revert immediately after the scheduled proof succeeds

No new repo hot path is introduced unless the live proof exposes a bug that requires a separate implementation task.

## Test Review

### Required Automated Suites

Run these before the live proof:

```bash
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_execute
```

These already cover the critical repo-owned behavior:

- queue freezing and target selection
- packet rendering and request truth lockstep
- dry-run packet generation
- write-envelope enforcement
- prompt digest fail-closed behavior
- manual closeout boundary

### Live Proof Coverage Diagram

```text
CODE PATH COVERAGE
===========================
[+] Shared watcher queue
    |
    ├── [★★★ TESTED] Frozen queue fields + latest_stable_minus_one selection
    │                 crates/xtask/tests/agent_maintenance_watch.rs
    └── [GAP]         Real GitHub schedule on main -> staging checkout
                      This plan proves it live

[+] Packet generation
    |
    ├── [★★★ TESTED] Request truth + execution_contract rendering
    │                 crates/xtask/tests/agent_maintenance_prepare.rs
    └── [GAP]         Real worker-opened PR from generated pr-summary.md
                      This plan proves it live

[+] Maintainer relay
    |
    ├── [★★★ TESTED] Dry-run packet writes only under temp run root
    ├── [★★★ TESTED] Write-mode boundary enforcement + prompt digest fail-closed
    └── [GAP]         Real maintainer run against a live worker-generated PR branch
                      This plan proves it live

USER FLOW COVERAGE
===========================
[+] Alarm rings by itself
    |
    └── [GAP] [->E2E] Scheduled watcher creates real downstream work

[+] Maintainer handoff
    |
    ├── [GAP] [->E2E] Open PR -> read HANDOFF.md -> dry-run -> write -> diff review
    └── [★★★ TESTED] Manual closeout remains outside write mode
                      crates/xtask/tests/agent_maintenance_execute.rs

─────────────────────────────────
COVERAGE: repo-owned behavior is well covered locally
Live gaps: 3 critical end-to-end proofs remain
  1. main schedule -> staging checkout
  2. real worker-opened PR from generated packet docs
  3. real maintainer dry-run/write path on that PR branch
─────────────────────────────────
```

### Missing Test Requirements Added By This Plan

This plan adds three required end-to-end validation steps, not new unit tests:

1. scheduled watcher proof on `main`
2. worker-opened PR proof from generated packet docs
3. maintainer dry-run/write proof on that PR branch

Those are mandatory because they are exactly the unproven product claims.

## Evidence Bundle

| Artifact | Where it comes from | Why it matters |
| --- | --- | --- |
| watcher run URL | scheduled run on `.github/workflows/agent-maintenance-release-watch.yml` | proves the alarm fired by itself |
| worker run URL | dispatched run on `.github/workflows/codex-cli-update-snapshot.yml` | proves watcher fanout reached the real worker |
| queue JSON excerpt | `_ci_tmp/maintenance-watch.json` or watcher logs | proves runtime target version and queue fields |
| PR URL | generated `automation/codex-maintenance-<target_version>` PR | proves packet generation and PR opening happened |
| request file path | `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml` | proves canonical request truth exists |
| handoff file path | `docs/agents/lifecycle/codex-maintenance/HANDOFF.md` | proves maintainer instructions are canonical |
| prepared run path | `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/` | proves dry-run packetization happened |
| final diff summary | local PR-branch review after `--write` | proves the maintainer relay mutated only allowed surfaces |
| cron revert reference | revert commit or PR on `main` | proves the temporary proof infrastructure was cleaned up |

## Failure Modes Registry

| Flow | Failure mode | Covered by test? | Error handling exists? | User-visible outcome | Critical gap? |
| --- | --- | --- | --- | --- | --- |
| schedule trigger | scheduled run never fires from the workflow definition that matters | no local test, yes by plan precondition | partial, bounded retry window only | silent non-proof unless checked explicitly | yes |
| watcher checkout | workflow runs, but the proof baseline is missing from `staging` | no | yes, by forcing Phase 2 before Phase 3 | wrong code gets "proven" | yes |
| queue emission | live release target differs from the stale design assumption | partial, local watch tests cover policy not live data | yes, runtime watcher output wins | expectation mismatch, not code failure | no |
| dispatch | watcher computes stale Codex entry but `createWorkflowDispatch` fails | not fully | partial, GitHub logs only | visible Actions failure | yes |
| worker PR creation | packet writes succeed but PR creation fails | partial | yes, explicit recovery path in packet-only PR workflow | visible worker failure | no |
| maintainer dry-run | local Codex preflight fails | yes | yes | visible CLI failure before mutation | no |
| maintainer write | relay writes outside declared surfaces | yes | yes, fail closed | visible CLI failure | no |
| closeout boundary | write mode performs closeout implicitly | yes | yes | silent trust-boundary violation if broken | yes |

Critical gaps are exactly the live proofs this plan closes.

## NOT In Scope

- redesigning `maintenance-watch`
- changing `latest_stable_minus_one`
- automating `close-agent-maintenance`
- updating `min_supported.txt`
- widening packet-only support for other agents
- broad CI cleanup unrelated to this proof
- speculative fixes for GitHub Actions behavior before a real failure exists

If the scheduled proof exposes a real defect, that defect becomes a separate implementation task with logs and a captured first-failing surface.

## TODOS.md Impact

No `TODOS.md` edits are part of this plan up front.

If the proof fails, create one precise follow-up item per failure. Each item must include:

- the first failing surface
- the exact log or artifact that proved the failure
- the affected branch or workflow
- the minimum next action

Do not dump vague "maintenance proof cleanup" debt into the backlog.

## Worktree Parallelization Strategy

This plan is partially parallelizable. Branch preparation and maintainer-environment readiness can move in parallel before the scheduled run happens. The scheduled proof itself stays sequential.

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| A. Local preflight + queue capture | `crates/xtask/src/agent_maintenance/`, `crates/xtask/tests/`, `crates/xtask/data/`, `cli_manifests/codex/` | — |
| B. Prepare `staging` proof baseline | `crates/xtask/src/agent_maintenance/`, `crates/xtask/tests/`, `.github/workflows/codex-cli-update-snapshot.yml`, `.github/workflows/agent-maintenance-open-pr.yml` | A |
| C. Temporary `main` cron acceleration | `.github/workflows/` | A |
| D. Observe scheduled watcher run | GitHub Actions watcher workflow + queue output | B, C |
| E. Inspect worker-opened PR | worker workflow outputs, `docs/agents/lifecycle/codex-maintenance/` | D |
| F. Maintainer dry-run/write proof | `docs/agents/.uaa-temp/agent-maintenance/`, generated maintenance packet, local relay run | E |
| G. Revert temporary cron | `.github/workflows/` on `main` | E |

### Parallel Lanes

- Lane 1: `A -> B`
  - sequential because `staging` readiness depends on a green local preflight
- Lane 2: `A -> C`
  - sequential inside the `main` workflow-definition lane
- Lane 3: maintainer environment preflight
  - independent local validation that Codex binary/auth is ready before `F`
- Lane 4: `D -> E -> G -> F`
  - sequential because each step consumes the real output of the previous one

### Execution Order

1. Run `A`.
2. After `A`, launch `B` and `C` in separate worktrees.
3. While `B` is finishing, run Lane 3 in parallel to confirm the maintainer environment is usable.
4. Once `B` and `C` are both green, wait for `D`.
5. After `D`, run `E`.
6. As soon as `E` proves the PR exists, run `G` immediately to remove the temporary cron.
7. Run `F` after `G`, using the generated PR branch and prepared `run_id`.

### Conflict Flags

- Lane A and Lane B do not share modules, but they do require cross-branch coordination.
- Lane B and Lane G both touch `.github/workflows/` on `main`. Treat them as one worktree lane.
- Lane D, E, G, and F must stay sequential. They all depend on one real generated PR and one real scheduled watcher run.
- Do not parallelize multiple scheduled proof attempts. One successful scheduled run is enough, and parallel retries create noisy duplicate automation branches.

## Completion Summary

- Step 0: Scope Challenge — scope accepted as-is, with one major correction: the proof must run through `main` and `staging`, not this feature branch alone
- Architecture Review: branch topology and control boundaries are now explicit
- Code Quality Review: no new abstractions or alternate proof paths added
- Test Review: coverage diagram produced, 3 live proof gaps identified and closed by execution steps
- Performance Review: no repo-code hot-path blocker, temporary schedule timing is the only tuning surface
- NOT in scope: written
- What already exists: written
- TODOS.md impact: no preemptive changes
- Failure modes: critical gaps enumerated with first-failure handling
- Parallelization: 4 lanes total, 2 meaningful pre-schedule parallel lanes, proof lane sequential
- Lake Score: complete proof path chosen over every shortcut

## Definition Of Done

This plan is done only when all of the following are true:

1. the scheduled watcher run happened from `main`
2. that watcher run checked out `staging`
3. the worker run happened from the watcher dispatch
4. the worker opened the Codex maintenance PR from generated packet docs
5. the maintainer completed `--dry-run` and `--write` on that PR branch
6. the resulting diff stayed inside the declared write envelope
7. `close-agent-maintenance` was still manual
8. the temporary cron acceleration was reverted
9. the evidence bundle is captured

If any one of those is missing, the repo has not yet proved the real path.
