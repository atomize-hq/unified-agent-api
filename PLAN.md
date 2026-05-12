# PLAN - Prove The Packet PR Maintenance Lane Can Actually Land

Status: ready for implementation  
Date: 2026-05-11  
Working branch: `staging`  
Plan revision baseline: `91a3cb1c`  
Design input: `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-design-20260511-185442.md`  
Supersedes: the prior repo-root `PLAN.md` for packet-open and dry-run-only proof readiness

## Executive Summary

The repo already proved the front half of this lane.

`opencode` is enrolled. The shared watcher can detect it as stale. The generic `packet_pr` opener
can materialize a truthful maintenance packet. The live request already carries the relay contract
for `execute-agent-maintenance`, exact writable surfaces, exact green gates, and the explicit
manual closeout command.

The missing proof is the back half:

1. prepare a fresh dry-run packet from the live request
2. execute `execute-agent-maintenance --write` with that same frozen `run_id`
3. pass the packet-declared green gates
4. author a valid `maintenance-closeout.json`
5. run `close-agent-maintenance`
6. commit replayable evidence showing the lane really lands

No new architecture. No new workflow family. One honest operator-faithful run.

## Objective

Make the repository able to truthfully claim:

> the live `opencode` automated maintenance packet can drive a real bounded maintenance write,
> survive the exact green gates it declares, and be explicitly closed with the repo-owned closeout
> command, with committed evidence that another maintainer can replay.

## Success Criteria

1. The proof starts from `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`.
2. A fresh `execute-agent-maintenance --dry-run` succeeds and emits a reusable `run_id`.
3. `execute-agent-maintenance --write --run-id <run_id>` succeeds against that same frozen packet.
4. The write-mode run records `status = "write_validated"` and `validation_passed = true` in the
   run packet under `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`.
5. All changed paths recorded by write mode stay inside the request's `writable_surfaces`.
6. `execute-agent-maintenance --write` does not write
   `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json`.
7. The exact `green_gates` from the live request pass in order, without a substituted command list.
8. The resulting diff is reviewable as one bounded maintenance run, not a spill into unrelated
   repo surfaces.
9. A valid `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json`
   exists with truthful request linkage, resolved findings, and either deferred findings or an
   explicit none reason.
10. `close-agent-maintenance` succeeds and refreshes the owned closeout outputs for the
    `opencode-maintenance` packet.
11. `docs/agents/lifecycle/opencode-maintenance/governance/proof/` contains enough structured
    evidence to replay the full operator flow without rediscovering command order.
12. `make preflight` passes on the final proof-bearing branch head.

## Locked Decisions

1. The milestone target remains the current live `opencode` automated request. Do not switch proof
   targets.
2. The proof is complete only if it includes both `execute-agent-maintenance --write` and manual
   `close-agent-maintenance`.
3. Another dry-run-only archive does not count. The new proof must extend to write mode and
   closeout.
4. The existing proof archive root stays canonical:
   `docs/agents/lifecycle/opencode-maintenance/governance/proof/`.
5. If the live request, rendered prompt, or writable surfaces drift after dry-run, the prepared
   packet is invalid. Rerun dry-run before write mode. Do not force stale `run_id` reuse.
6. If write mode succeeds but exposes any contract gap or packet rough edge inside the bounded
   surfaces, that is not "proof with follow-up." Fix it, rerender if needed, rerun dry-run, rerun
   write, then archive the final successful run only.
7. Manual closeout remains manual by design. Do not automate it inside `execute-agent-maintenance`.
8. Raw `codex-stdout.txt` and `codex-stderr.txt` remain temp evidence under
   `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`. The committed proof archive should
   store structured summaries and JSON evidence, not raw noisy logs, unless debugging demands it.
9. Do not reopen already-solved scope:
   - no new release-watch enrollment work
   - no new dispatch kind
   - no new workflow YAML family
   - no new source kind
   - no second proof archive subsystem

## Step 0 Scope Contract

### Premise Challenge

| Premise | Assessment | Decision |
| --- | --- | --- |
| The shared watcher or PR opener still needs redesign. | Rejected. The repo already proved queue emission and `packet_pr` opening on the live `opencode` lane. | Reuse the current watcher and generic opener unchanged. |
| Another dry-run archive would finish the job. | Rejected. The missing claim is that the packet can actually land the bounded write path and closeout. | Write mode and closeout are mandatory. |
| We should widen scope to another agent to make the proof more impressive. | Rejected. That turns a proof milestone into a rollout milestone. | `opencode` only. |
| We can treat non-critical packet rough edges discovered after write as success. | Rejected. The proof is about truthful operator replay, not almost-truthful operator replay. | Fix any surfaced contract issue and rerun the proof. |
| The closeout step can be implied if write mode passes. | Rejected. The contract and operator guide explicitly keep closeout manual. | Author `maintenance-closeout.json` and run `close-agent-maintenance`. |
| We should archive only prose notes, not machine-readable evidence. | Rejected. Later maintainers need replayable proof, not a memory of the proof. | Commit structured JSON and command outputs. |

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| Canonical automated request | `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml` | Reuse as the proof input. Regenerate only if it drifted from branch truth. |
| Canonical operator contract | `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md` and `docs/cli-agent-onboarding-factory-operator-guide.md` | Reuse as the command order and ownership source of truth. |
| Local relay execution | `crates/xtask/src/agent_maintenance/execute/workflow.rs`, `runtime.rs`, `validate.rs`, `packet.rs` | Reuse. This already validates preflight, packet continuity, writable surfaces, and green gates. |
| Manual closeout writer | `crates/xtask/src/agent_maintenance/closeout.rs`, `closeout/write.rs`, `closeout/validate.rs` | Reuse. This already validates request linkage, live drift truth, and owned output writes. |
| Relay/closeout regression coverage | `crates/xtask/tests/agent_maintenance_execute.rs`, `crates/xtask/tests/agent_maintenance_closeout/**` | Reuse and extend only if the live run exposes a missing guardrail. |
| Existing proof archive root | `docs/agents/lifecycle/opencode-maintenance/governance/proof/` | Reuse. Append execution and closeout evidence here. |
| Existing proof state | `proof-notes.md`, `execute-dry-run.txt`, earlier queue/opener evidence | Reuse if still truthful. Regenerate only if the request or release truth moved. |

### Minimum Complete Change

The minimum complete implementation is:

1. verify the live `opencode` request is still proof-stable for the current branch head
2. prepare a fresh dry-run packet and record its `run_id`
3. execute write mode against that exact `run_id`
4. review the bounded diff and fix any contract breakage inside the declared surfaces
5. rerun dry-run and write if any packet-affecting file changed
6. author `maintenance-closeout.json`
7. run `close-agent-maintenance`
8. archive structured execution and closeout evidence under the canonical proof root
9. rerun final verification on the finished closeout state

Anything smaller still leaves the landing claim unproven.

### Complexity, Search, Completeness, And Distribution Checks

**Complexity smell**

This plan may touch more than eight files if the live run exposes a real gap. That is acceptable.
The repo already owns the machinery. The goal is to finish the live proof, not to keep the diff
artificially tiny.

**Search check**

- **[Layer 1]** Reuse `execute-agent-maintenance` for write mode. The relay contract is already
  implemented and tested.
- **[Layer 1]** Reuse `close-agent-maintenance` for manual closeout. The contract already writes
  the owned closeout outputs and validates live drift truth.
- **[Layer 1]** Reuse the existing proof archive root instead of inventing a second evidence lane.
- **[EUREKA]** The hidden landmine is not the write engine. It is packet continuity.
  `validate_prepared_packet()` fail-closes when request SHA, rendered prompt, target version,
  branch name, writable surfaces, green gates, or closeout command drift after dry-run. That means
  any packet-affecting fix between dry-run and write forces a fresh dry-run. Ignore that and the
  proof becomes fake.

**Completeness rule**

Do the whole lake:

- fresh dry-run
- write mode
- exact green gates
- manual closeout
- structured proof archive
- final preflight

Do not stop at "write mode produced a diff."

**Distribution check**

There is no new user-facing binary or package here.

The shipped artifact is committed operational truth:

- a closed `opencode` maintenance run
- a bounded proof archive
- a branch head that later maintainers can inspect and replay

## Final Contract To Land

### Proof Input Continuity Contract

The proof input remains:

- request path: `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`
- canonical handoff: `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md`
- closeout path: `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json`

Before dry-run:

1. confirm `maintenance-request.toml` still reflects the intended proof base
2. confirm `HANDOFF.md` still matches the request's execution contract
3. confirm the detected release fields still describe the proof target
4. confirm the writable surfaces still match the intended maintenance blast radius

If any of those are stale, refresh the packet truth first, then start the proof from a new dry-run.

### Dry-Run Packet Contract

Command:

```sh
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --dry-run
```

Expected outcomes:

- exit code `0`
- CLI output includes `run_id`, `run_dir`, and the exact closeout command
- run packet written under `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`
- validation report status = `pass`
- run status = `dry_run_ready`

Required temp artifacts:

- `input-contract.json`
- `codex-prompt.md`
- `validation-report.json`
- `run-status.json`
- `written-paths.json`
- `run-summary.md`

### Write-Mode Contract

Command:

```sh
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --write --run-id <run_id>
```

Expected outcomes:

- exit code `0`
- validation report status = `pass`
- run status = `write_validated`
- `written_paths.json` is non-empty
- every written path matches the frozen `writable_surfaces`
- `maintenance-closeout.json` is not written by write mode
- the exact `green_gates` run in order and all pass

Required temp artifacts after a successful write:

- `validation-report.json`
- `run-status.json`
- `written-paths.json`
- `run-summary.md`
- `codex-execution.json`
- `codex-stdout.txt`
- `codex-stderr.txt`

If any packet-affecting file changes after dry-run and before write mode, discard the prepared
packet and rerun dry-run.

### Closeout Contract

Before closeout:

1. inspect the write-mode diff
2. decide the resolved findings list truthfully
3. decide whether deferred findings exist; if not, write `explicit_none_reason`

Required JSON fields in `maintenance-closeout.json`:

- `request_ref`
- `request_sha256`
- `resolved_findings`
- exactly one of:
  - `deferred_findings`
  - `explicit_none_reason`
- `preflight_passed`
- `recorded_at`
- `commit`

Command:

```sh
cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json
```

Expected outcomes:

- exit code `0`
- refreshed `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md`
- refreshed `docs/agents/lifecycle/opencode-maintenance/governance/remediation-log.md`
- written `maintenance-closeout.json`
- live drift validation passes for all resolved findings

### Proof Archive Contract

The canonical proof root is:

```text
docs/agents/lifecycle/opencode-maintenance/governance/proof/
```

It must end the milestone containing:

- existing queue/opener evidence, if still truthful
- `request-sha256.txt`
- `run-id.txt`
- `execute-dry-run.txt`
- `execute-write.txt`
- `validation-report-dry-run.json`
- `validation-report-write.json`
- `run-status-dry-run.json`
- `run-status-write.json`
- `written-paths-write.json`
- `closeout-summary.md`
- `proof-notes.md`

Raw `codex-stdout.txt` and `codex-stderr.txt` stay temp-only unless a debugging failure forces
them into the archive.

## Architecture Review

### Boundary Decision

The boundary stays:

```text
live maintenance request
        |
        v
execute-agent-maintenance --dry-run
        |
        v
prepared run packet with frozen run_id
        |
        v
execute-agent-maintenance --write
        |
        v
bounded diff inside writable_surfaces
        |
        v
manual maintenance-closeout.json authoring
        |
        v
close-agent-maintenance
        |
        v
committed proof archive
```

Do not move closeout into write mode.

Do not replace the request-owned green gates with a handwritten checklist.

Do not turn this into a second packet generation project.

### Dependency Graph

```text
docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml
  ├── detected_release
  ├── execution_contract
  └── closeout_path
          |
          v
crates/xtask/src/agent_maintenance/execute/workflow.rs
  ├── runtime.rs      (Codex preflight + green gates)
  ├── validate.rs     (prepared packet continuity + writable surface jail)
  └── packet.rs       (run packet persistence)
          |
          v
docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/
          |
          v
maintainer diff review + maintenance-closeout.json
          |
          v
crates/xtask/src/agent_maintenance/closeout/validate.rs
crates/xtask/src/agent_maintenance/closeout/write.rs
          |
          v
docs/agents/lifecycle/opencode-maintenance/HANDOFF.md
docs/agents/lifecycle/opencode-maintenance/governance/remediation-log.md
docs/agents/lifecycle/opencode-maintenance/governance/proof/**
```

### File-Level Ownership

| Area | Files | Purpose |
| --- | --- | --- |
| Live proof input | `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`, `HANDOFF.md` | Freeze the request and execution contract the proof must obey. |
| Relay execution | `crates/xtask/src/agent_maintenance/execute/**` | Enforce preflight, continuity, writable surfaces, and green gates. |
| Manual closeout | `crates/xtask/src/agent_maintenance/closeout/**` | Validate closeout JSON and write owned closeout outputs. |
| Live temp evidence | `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/**` | Capture machine-readable run evidence for dry-run and write mode. |
| Committed proof archive | `docs/agents/lifecycle/opencode-maintenance/governance/proof/**` | Store replayable evidence for later maintainers. |
| Operator docs | `docs/cli-agent-onboarding-factory-operator-guide.md` | Update only if the live proof exposes wording drift. |
| Regression coverage | `crates/xtask/tests/agent_maintenance_execute.rs`, `crates/xtask/tests/agent_maintenance_closeout/**` | Lock any newly discovered failure closed. |

### Production Failure Scenario Per Path

| Codepath | Real failure | Planned handling |
| --- | --- | --- |
| Dry-run preflight | Codex binary/auth state is broken or preflight mutates the repo | Dry-run fails closed before write mode. Fix host state and rerun. |
| Packet continuity | Request SHA, rendered prompt, or writable surfaces drift after dry-run | `validate_prepared_packet()` rejects the write. Rerun dry-run after the fix. |
| Write boundary | Maintained agent writes outside `writable_surfaces` or touches closeout path | Write mode fails closed and records the violation in the run packet. |
| Runtime no-op | Write mode produces no runtime-owned diff | Write mode fails closed. The lane does not get credit for doing nothing. |
| Green gates | One declared gate fails after codex write | Write mode stops and records the failing gate. Fix the underlying issue, then restart from dry-run. |
| Closeout truth | Resolved findings still appear in live drift | `close-agent-maintenance` fails closed. Fix or defer honestly. |
| Proof archive drift | Committed proof notes reference the wrong request SHA or run_id | Treat as a proof failure. Rewrite the archive from the actual successful run. |

## Implementation Plan

### Phase 1: Freeze Proof Inputs

**Goal:** start from one stable request and one stable proof boundary.

**Files**

- `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`
- `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md`
- `docs/agents/lifecycle/opencode-maintenance/governance/proof/proof-notes.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md` only if wording drift is already obvious

**Concrete changes**

1. Verify the request still points at the intended `target_version`, `branch_name`, and
   `request_commit`.
2. Verify `HANDOFF.md` still matches the request's execution contract.
3. Decide whether existing queue/opener proof artifacts are still truthful enough to keep.
4. If request or packet docs drifted, refresh them before any new dry-run.

**Verification**

- request and handoff tell one story
- no stale packet wording survives into the live proof run

**Exit criteria**

- a maintainer can point at one canonical request and say "this is the packet we are proving"

### Phase 2: Prepare A Fresh Dry-Run Packet

**Goal:** freeze one prepared `run_id` for the actual write attempt.

**Files**

- `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/**`
- `docs/agents/lifecycle/opencode-maintenance/governance/proof/execute-dry-run.txt`
- `docs/agents/lifecycle/opencode-maintenance/governance/proof/request-sha256.txt`
- `docs/agents/lifecycle/opencode-maintenance/governance/proof/run-id.txt`
- `docs/agents/lifecycle/opencode-maintenance/governance/proof/validation-report-dry-run.json`
- `docs/agents/lifecycle/opencode-maintenance/governance/proof/run-status-dry-run.json`

**Concrete changes**

1. Run fresh dry-run from the live request.
2. Capture the emitted `run_id`.
3. Copy the dry-run summary and structured JSON evidence into the committed proof root.
4. Record the exact request SHA used for the proof.

**Verification**

- dry-run exits `0`
- preflight passes
- run status is `dry_run_ready`

**Exit criteria**

- there is one fresh prepared run packet and one committed `run_id` file pointing at it

### Phase 3: Execute Write Mode And Review The Bounded Diff

**Goal:** prove that the packet can actually land the bounded maintenance update.

**Files**

- writable surfaces declared by the live request
- `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/**`
- `docs/agents/lifecycle/opencode-maintenance/governance/proof/execute-write.txt`
- `docs/agents/lifecycle/opencode-maintenance/governance/proof/validation-report-write.json`
- `docs/agents/lifecycle/opencode-maintenance/governance/proof/run-status-write.json`
- `docs/agents/lifecycle/opencode-maintenance/governance/proof/written-paths-write.json`

**Concrete changes**

1. Run write mode with the exact prepared `run_id`.
2. Inspect `written-paths.json` and the repo diff for scope, truth, and packet compliance.
3. If write mode fails because the packet or executor still has a hidden assumption:
   - fix the issue inside the bounded blast radius
   - rerun dry-run
   - rerun write mode
4. Archive only the final successful write evidence.

**Verification**

- write exits `0`
- run status is `write_validated`
- all green gates pass in order
- no boundary violation occurs
- closeout path stays untouched

**Exit criteria**

- the repo has one truthful write-mode diff produced by the live packet

### Phase 4: Author Closeout And Run Manual Closeout

**Goal:** prove the explicit maintainer closeout step the operator guide requires.

**Files**

- `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json`
- `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md`
- `docs/agents/lifecycle/opencode-maintenance/governance/remediation-log.md`
- `docs/agents/lifecycle/opencode-maintenance/governance/proof/closeout-summary.md`

**Concrete changes**

1. Author `maintenance-closeout.json` from the actual write result.
2. Record resolved findings truthfully.
3. Use deferred findings only if live drift still exists and is intentionally left open.
4. Run `close-agent-maintenance`.
5. Archive a short closeout summary describing what was resolved and why any deferred item remains.

**Verification**

- closeout JSON parses and validates
- live drift check passes for resolved findings
- closeout write refreshes the owned outputs

**Exit criteria**

- the maintenance lane is explicitly closed, not implicitly assumed closed

### Phase 5: Archive Final Proof And Operator Notes

**Goal:** leave behind replayable evidence, not just a passing branch.

**Files**

- `docs/agents/lifecycle/opencode-maintenance/governance/proof/**`
- `docs/cli-agent-onboarding-factory-operator-guide.md` only if the live proof disproves current wording

**Concrete changes**

1. Refresh `proof-notes.md` so it describes the final successful run, not just dry-run readiness.
2. Keep existing queue/opener evidence if still truthful; regenerate only if the request or target
   moved.
3. Add the final write and closeout evidence files from the successful run.
4. Update the operator guide only if the live proof exposed wording drift, command-order drift, or
   artifact-shape drift.

**Verification**

- proof root contains a complete replay trail
- operator wording matches the proof that just landed

**Exit criteria**

- another maintainer can replay the lane from committed artifacts without asking "what actually happened?"

### Phase 6: Final Verification

**Goal:** make the finished closeout state green.

**Final verification**

```sh
cargo fmt --all
cargo run -p xtask -- codex-validate --root cli_manifests/opencode
cargo run -p xtask -- support-matrix --check
cargo run -p xtask -- capability-matrix --check
cargo run -p xtask -- capability-matrix-audit
make preflight
```

**Exit criteria**

- the exact packet-declared gates pass on the final proof-bearing commit

## Code Quality Review

### DRY Rules

1. Do not add a second execution surface for the same maintenance lane.
2. Do not add a second closeout writer.
3. Do not create a new archive format if the existing proof root can hold the evidence.
4. Do not duplicate green gates in ad hoc docs when the request already owns them.

### Explicit Over Clever

Prefer:

- one request
- one fresh run id
- one successful write packet
- one closeout artifact
- one proof archive

Avoid:

- partial successes dressed up as proof
- magic reruns that skip dry-run continuity
- extra automation that hides the explicit maintainer closeout step

### Minimal-Diff Rules

The best outcome is:

- no core code changes at all
- one live proof run
- docs and evidence refreshed to match it

If the live run exposes a hidden assumption, fix only the implicated relay, closeout, or packet
surface. No opportunistic cleanup.

## Test Review

### Test Framework

Rust integration tests under `crates/xtask/tests/` remain the primary harness.

### Code Path Coverage

```text
CODE PATH COVERAGE
===========================
[+] maintenance-request.toml continuity
    ├── [VERIFY] request sha, target_version, branch_name stay frozen for proof
    └── [VERIFY] execution_contract matches HANDOFF.md

[+] crates/xtask/src/agent_maintenance/execute/workflow.rs
    ├── [TESTED] dry-run prepares packet and records closeout command
    ├── [TESTED] write mode requires matching dry-run packet
    ├── [TESTED] green gates run in order and stop on failure
    └── [VERIFY] live opencode request reaches write_validated

[+] crates/xtask/src/agent_maintenance/execute/validate.rs
    ├── [TESTED] prompt/request drift fail closed
    ├── [TESTED] out-of-bounds writes fail closed
    ├── [TESTED] closeout path write is forbidden
    └── [TESTED] noop runtime execution fails closed

[+] crates/xtask/src/agent_maintenance/closeout/validate.rs
    ├── [TESTED] request_ref and request_sha256 must match
    ├── [TESTED] exactly one of deferred_findings or explicit_none_reason is required
    └── [VERIFY] live opencode closeout truth matches actual resolved findings

[+] crates/xtask/src/agent_maintenance/closeout/write.rs
    ├── [TESTED] only owned closeout surfaces are written
    ├── [TESTED] automated-request closeout preserves trigger truth
    └── [VERIFY] final opencode closeout refreshes the maintained handoff and remediation log
```

### User / Operator Flow Coverage

```text
USER FLOW COVERAGE
===========================
[+] Maintainer freezes the live packet
    ├── [VERIFY] request + handoff are in sync
    └── [VERIFY] proof notes reference the actual request sha

[+] Maintainer prepares dry-run packet
    ├── [VERIFY] dry-run exits 0
    ├── [VERIFY] run_id is captured
    └── [VERIFY] structured run packet exists under .uaa-temp

[+] Maintainer executes write mode
    ├── [VERIFY] exact green gates pass
    ├── [VERIFY] written paths stay within writable_surfaces
    ├── [VERIFY] resulting diff is non-empty and bounded
    └── [VERIFY] maintenance-closeout.json remains untouched by write mode

[+] Maintainer performs manual closeout
    ├── [VERIFY] maintenance-closeout.json is truthful
    ├── [VERIFY] close-agent-maintenance exits 0
    └── [VERIFY] refreshed closeout outputs match the request and live drift truth

[+] Future maintainer replays the proof
    ├── [VERIFY] proof archive has run_id + request sha + run statuses
    └── [VERIFY] proof notes explain the command order and rerun rule when packet drift occurs
```

### Exact Test Files To Extend

Extend only if the live proof reveals a missing guardrail:

- `crates/xtask/tests/agent_maintenance_execute.rs`
- `crates/xtask/tests/agent_maintenance_closeout/write_outputs.rs`
- `crates/xtask/tests/agent_maintenance_closeout/request_and_schema.rs`
- `crates/xtask/tests/agent_maintenance_closeout/live_drift_validation.rs`

If the live run does not reveal a gap, do not create speculative new tests.

### Regression Rule

This is a regression if any of these happen:

1. a stale prepared packet can still reach write mode after request or prompt drift
2. write mode can touch `maintenance-closeout.json`
3. write mode can exit successfully with no bounded repo-owned changes
4. closeout accepts resolved findings that still appear in live drift
5. the proof archive can be updated without recording the actual request SHA and run ID
6. the operator guide still tells a different command order than the successful proof used

Any live regression found during this milestone gets a test before the work is called done.

## Performance Review

This is operational work, not a hot path.

The real performance constraints are:

1. avoid unnecessary reruns by freezing packet truth before dry-run
2. do not add new steady-state automation just to archive one proof
3. accept that `make preflight` dominates runtime; that is the price of truthful proof

## Failure Modes Registry

| Failure mode | Test required | Error handling | User-visible outcome |
| --- | --- | --- | --- |
| Codex preflight fails | Already covered | dry-run fails closed | explicit stop before write mode |
| Request/prompt drift after dry-run | Already covered | write fails closed | explicit stop, rerun dry-run required |
| Write escapes `writable_surfaces` | Already covered | write fails closed | explicit stop with violating paths |
| Write produces no runtime diff | Already covered | write fails closed | explicit stop, no fake success |
| Green gate fails after write | Already covered | write fails closed | explicit stop with failing command |
| Closeout JSON mismatches request SHA | Already covered | closeout fails closed | explicit stop before closeout outputs refresh |
| Resolved findings still match live drift | Already covered | closeout fails closed | explicit stop, deferred/resolved truth must be fixed |
| Proof archive records stale run metadata | New check if missing | human review + archive rewrite | explicit proof failure, not silent rot |

**Critical gap definition**

Any path that claims:

- the packet can land
- the packet passed write mode
- or the run was closed

without structured evidence for that exact request SHA and run ID is a critical gap.

## NOT In Scope

1. Re-enrolling `opencode` or any second agent.
2. Changing the shared watcher topology.
3. Introducing a new source kind or new dispatch kind.
4. Automating closeout inside `execute-agent-maintenance`.
5. Building a generic long-term proof archival subsystem.
6. Promoting `opencode` to a new `latest_supported.txt` posture unless the bounded write itself
   requires it.
7. Opportunistic cleanup outside the request's writable surfaces.

## TODOS.md Impact

No new repo-wide TODO is required up front.

If the live proof exposes a genuinely separate follow-on, record it only after the proof lands.
Do not smuggle a second milestone into this one.

## Worktree Parallelization Strategy

This plan has limited parallelism. The live dry-run -> write -> closeout chain is sequential by
design because `run_id` continuity is the contract.

There is still useful pre-proof parallelism if the live run exposes code gaps.

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| 1. Freeze proof inputs | `docs/agents/lifecycle/opencode-maintenance/`, `docs/cli-agent-onboarding-factory-operator-guide.md` | — |
| 2. Relay hardening, if needed | `crates/xtask/src/agent_maintenance/execute/`, `crates/xtask/tests/agent_maintenance_execute.rs` | 1 |
| 3. Closeout hardening, if needed | `crates/xtask/src/agent_maintenance/closeout/`, `crates/xtask/tests/agent_maintenance_closeout/` | 1 |
| 4. Proof archive scaffolding | `docs/agents/lifecycle/opencode-maintenance/governance/proof/` | 1 |
| 5. Live dry-run -> write -> closeout | `.uaa-temp/agent-maintenance/runs/`, request writable surfaces, `docs/agents/lifecycle/opencode-maintenance/` | 2, 3, 4 |
| 6. Final verification and doc polish | workspace-wide green gates, operator docs if needed | 5 |

### Parallel Lanes

- Lane A: Step 2  
  Independent relay hardening if the live write path exposes a missing guardrail.
- Lane B: Step 3  
  Independent closeout hardening if the manual closeout validation or rendering is the problem.
- Lane C: Step 4  
  Proof archive scaffolding and filename conventions before the final evidence copy.
- Lane D: Step 5  
  Sequential live execution. No parallelism once dry-run starts.
- Lane E: Step 6  
  Sequential final verification after the live run and closeout are complete.

### Execution Order

1. Freeze the proof inputs first.
2. If no code gaps are visible, skip straight to Lane C and then Lane D.
3. If code gaps appear, launch Lane A and Lane B in parallel while Lane C prepares the proof root.
4. Merge A, B, and C.
5. Run Lane D as one uninterrupted chain:
   - dry-run
   - write
   - diff review
   - closeout
6. Run Lane E last.

### Conflict Flags

- Lane A and Lane D both depend on the rendered prompt and execution contract. Do not begin dry-run
  until Lane A is merged.
- Lane B and Lane D both touch closeout-owned surfaces under
  `docs/agents/lifecycle/opencode-maintenance/`. Do not author the final closeout JSON until Lane
  B is merged.
- Lane C and Lane D both touch the proof root. Reserve final filenames for the successful run and
  avoid committing placeholder artifacts with real names.

## Completion Summary

- Step 0: scope reduced to the real missing proof, not prior enrollment work
- Architecture review: existing shared packet lane remains intact; only the live landing proof is missing
- Code quality review: one request, one run ID, one explicit closeout, one proof archive
- Test review: existing relay and closeout tests are mostly in place; extend only if the live proof finds a missing guardrail
- Performance review: sequential by contract once dry-run starts
- NOT in scope: written
- What already exists: written
- TODOS.md impact: no new TODO required by default
- Failure modes: all critical gaps must be driven to zero
- Parallelization: 5 useful steps, but the live proof chain itself is strictly sequential
- Lake Score: choose the complete option on write mode, closeout, structured evidence, and final verification
