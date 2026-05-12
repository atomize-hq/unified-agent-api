# PLAN - Prove The Packet PR Maintenance Lane Can Actually Land

Status: ready for implementation  
Date: 2026-05-12  
Working branch: `staging`  
Plan revision baseline: `d6b86cdc`  
Design input: `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-design-20260511-185442.md`  
Supersedes: the prior repo-root `PLAN.md` for dry-run-only packet proof readiness

## Executive Summary

The repo already proved the front half of the automated maintenance lane for `opencode`.

The shared watcher can detect stale state. The generic `packet_pr` opener can materialize a
truthful maintenance packet. The live request already freezes the executor, prompt digest,
writable surfaces, green gates, and the exact manual closeout command.

The remaining proof is the landing path:

1. prepare a fresh dry-run packet from the live request
2. execute `execute-agent-maintenance --write` with that exact frozen `run_id`
3. pass the exact packet-declared green gates
4. author a truthful `maintenance-closeout.json`
5. run `close-agent-maintenance`
6. commit replayable proof artifacts showing another maintainer can follow the same flow

No new workflow family. No new transport topology. No second archive system. One honest,
operator-faithful live run that lands and closes.

## Objective

Make the repository able to truthfully claim:

> the live `opencode` automated maintenance packet can drive a real bounded maintenance write,
> survive the exact green gates it declares, and be explicitly closed with the repo-owned
> closeout command, with committed evidence that another maintainer can replay.

## Success Criteria

1. The proof starts from
   `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`.
2. A fresh `execute-agent-maintenance --dry-run` succeeds and emits a reusable `run_id`.
3. `execute-agent-maintenance --write --run-id <run_id>` succeeds against that same frozen packet.
4. The write-mode run records `status = "write_validated"` and `validation_passed = true` in
   `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/run-status.json`.
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
    `opencode-maintenance` lane.
11. `docs/agents/lifecycle/opencode-maintenance/governance/proof/` contains enough structured
    evidence to replay the full operator flow without rediscovering command order.
12. `make preflight` passes on the final proof-bearing branch head.

## Locked Decisions

1. The proof target remains the current live `opencode` automated request. Do not switch agents.
2. The proof is incomplete unless it includes both `execute-agent-maintenance --write` and manual
   `close-agent-maintenance`.
3. Another dry-run-only archive does not count.
4. The canonical proof archive root stays
   `docs/agents/lifecycle/opencode-maintenance/governance/proof/`.
5. If the request, rendered prompt, writable surfaces, green gates, target version, branch name,
   or request SHA drift after dry-run, the prepared packet is invalid. Rerun dry-run before write.
6. If write mode succeeds but exposes any contract gap inside the bounded surfaces, that is not
   "proof with follow-up". Fix the gap, rerender if needed, rerun dry-run, rerun write, then
   archive only the final successful run.
7. Manual closeout remains manual by design. Do not automate it inside
   `execute-agent-maintenance`.
8. Raw `codex-stdout.txt` and `codex-stderr.txt` remain temp evidence under
   `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`. Commit structured summaries and JSON
   evidence by default. Promote raw logs only if a failure requires them for diagnosis.

## Step 0 Scope Contract

### Premise Challenge

| Premise | Assessment | Decision |
| --- | --- | --- |
| The shared watcher or PR opener still needs redesign. | Rejected. The repo already proved queue emission and `packet_pr` opening for the live `opencode` lane. | Reuse the current watcher and generic opener unchanged. |
| Another dry-run archive would finish the job. | Rejected. The missing claim is that the packet can land the bounded write path and explicit closeout. | Write mode and closeout are mandatory. |
| We should widen scope to another agent to make the proof stronger. | Rejected. That turns a proof milestone into a rollout milestone. | `opencode` only. |
| We can treat non-critical packet rough edges discovered after write as success. | Rejected. This milestone is about truthful replay, not almost-truthful replay. | Fix the gap and rerun. |
| The closeout step can be implied if write mode passes. | Rejected. The contract deliberately keeps closeout manual. | Author `maintenance-closeout.json` and run `close-agent-maintenance`. |
| We should archive only prose notes, not machine-readable evidence. | Rejected. Later maintainers need replayable proof, not a memory of the proof. | Commit structured JSON and command evidence. |

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| Canonical automated request | `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml` | Reuse as the proof input. Refresh only if the request drifted from branch truth. |
| Canonical operator contract | `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md` and `docs/cli-agent-onboarding-factory-operator-guide.md` | Reuse as the command order and ownership source of truth. |
| Local relay execution | `crates/xtask/src/agent_maintenance/execute/workflow.rs`, `runtime.rs`, `validate.rs`, `packet.rs` | Reuse. This already validates preflight, packet continuity, writable surfaces, runtime writes, and green gates. |
| Manual closeout writer | `crates/xtask/src/agent_maintenance/closeout.rs`, `closeout/write.rs`, `closeout/validate.rs` | Reuse. This already validates request linkage, live drift truth, and owned output writes. |
| Relay and closeout regression coverage | `crates/xtask/tests/agent_maintenance_execute.rs`, `crates/xtask/tests/agent_maintenance_closeout/**` | Reuse and extend only if the live run exposes a missing guardrail. |
| Existing proof archive root | `docs/agents/lifecycle/opencode-maintenance/governance/proof/` | Reuse. Append execution and closeout evidence here. |
| Existing proof state | `proof-notes.md`, `execute-dry-run.txt`, `request-sha256.txt`, queue and workflow evidence | Reuse if still truthful. Replace any artifact that no longer matches the final successful run. |

### NOT In Scope

1. Enrolling new agents in release watch.
2. Redesigning `packet_pr` versus worker-dispatch topology.
3. Adding a new workflow YAML family.
4. Creating a second proof archive subsystem.
5. Automating closeout inside `execute-agent-maintenance`.
6. Promoting raw `codex-stdout.txt` and `codex-stderr.txt` into committed artifacts unless a real
   failure requires them.
7. Expanding the proof to `codex` or `claude_code`.

### Minimum Complete Change

The minimum complete implementation is:

1. verify the live `opencode` request is still proof-stable for the current branch head
2. refresh the request packet only if that truth audit fails
3. prepare a fresh dry-run packet and record its `run_id`
4. execute write mode against that exact `run_id`
5. inspect the bounded diff and fix any contract breakage inside the declared surfaces
6. rerun dry-run and write if any packet-affecting file changed
7. author `maintenance-closeout.json`
8. run `close-agent-maintenance`
9. archive structured execution and closeout evidence under the canonical proof root
10. rerun final verification on the finished closeout state

Anything smaller still leaves the landing claim unproven.

### Complexity, Search, Completeness, And Distribution Checks

**Complexity smell**

This plan may touch more than eight files if the live run exposes a real gap. That is acceptable.
The repo already owns the machinery. The goal is to finish the live proof, not to keep the diff
artificially tiny.

**Search check**

- **[Layer 1]** Reuse `execute-agent-maintenance` for write mode. The relay contract already
  exists and is tested.
- **[Layer 1]** Reuse `close-agent-maintenance` for manual closeout. The closeout contract already
  exists and is tested.
- **[Layer 1]** Reuse the existing proof archive root instead of inventing another evidence lane.
- **[EUREKA]** The hidden landmine is packet continuity, not the write engine. The prepared packet
  is invalid if request SHA, prompt contents, prompt digest, target version, branch name,
  writable surfaces, green gates, closeout path, or closeout command drift after dry-run. Ignore
  that and the proof becomes fake.

**Completeness rule**

Do the whole lake:

- fresh dry-run
- write mode
- exact green gates
- manual closeout
- structured proof archive
- final `make preflight`

Do not stop at "write mode produced a diff."

**Distribution check**

There is no new user-facing binary or package here.

The shipped artifact is committed operational truth:

- a closed `opencode` maintenance run
- a bounded proof archive
- a branch head that later maintainers can inspect and replay

## Architecture Contract

### System Boundary

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
committed proof archive + final preflight
```

Do not move closeout into write mode.

Do not replace the request-owned green gates with a handwritten checklist.

Do not turn this into a second packet generation project.

### Dependency Graph

```text
docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml
  ├── detected_release
  ├── execution_contract
  │    ├── prompt_template_path
  │    ├── prompt_sha256
  │    ├── writable_surfaces
  │    ├── ordered_commands
  │    ├── green_gates
  │    └── closeout_path
  └── closeout_path
          |
          v
docs/agents/lifecycle/opencode-maintenance/HANDOFF.md
          |
          v
crates/xtask/src/agent_maintenance/execute/workflow.rs
  ├── runtime.rs      (Codex preflight + green gate execution)
  ├── validate.rs     (prepared packet continuity + writable-surface jail + no-op rejection)
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

### Packet Invalidation Rules

The prepared packet remains valid only until one of these changes:

| Change after dry-run | Effect | Required response |
| --- | --- | --- |
| `maintenance-request.toml` contents or SHA | Prepared packet is invalid. | Rerun dry-run before write. |
| Rendered prompt contents or prompt digest | Prepared packet is invalid. | Rerun dry-run before write. |
| `target_version` or `branch_name` | Prepared packet is invalid. | Rerun dry-run before write. |
| `writable_surfaces`, `green_gates`, `closeout_path`, or closeout command | Prepared packet is invalid. | Rerun dry-run before write. |
| Any fix inside repo code or docs that changes packet-owned truth before write | Prepared packet may be invalid. | Re-evaluate. If the fix touched request truth, rerun dry-run. |
| Manual closeout authoring after a successful write | Prepared packet is no longer needed. | Do not rerun write unless the write proof itself changed. |
| Proof archive note edits after a successful write | Does not invalidate the successful write. | No rerun, unless the archive text misstates request or run truth. |

### Proof Archive Contract

The canonical proof root is:

```text
docs/agents/lifecycle/opencode-maintenance/governance/proof/
```

It must end this milestone containing:

- existing queue and PR-open evidence, if still truthful
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

Raw `codex-stdout.txt` and `codex-stderr.txt` stay temp-only unless a real failure forces them
into the archive for diagnosis.

## Failure Modes Registry

| Codepath | Real failure | Test coverage | Planned handling | Maintainer signal | Critical gap |
| --- | --- | --- | --- | --- | --- |
| Dry-run preflight | Codex binary or auth state is broken | Covered in `crates/xtask/tests/agent_maintenance_execute.rs` | Dry-run fails closed before write mode. Fix host state and rerun. | Explicit CLI failure with preflight message | No |
| Prepared packet continuity | Request SHA, prompt, target version, branch name, writable surfaces, green gates, closeout path, or closeout command drift after dry-run | Covered in `workflow.rs` and `validate.rs`; prompt mismatch and drift protections are tested | Write mode blocks. Refresh request truth if needed, rerun dry-run, then rerun write. | Explicit CLI failure listing mismatches | No |
| Write boundary | Maintained agent writes outside `writable_surfaces` or touches closeout path | Covered in `crates/xtask/tests/agent_maintenance_execute.rs` | Write mode fails closed and records the boundary violation. | Explicit CLI failure and run packet evidence | No |
| Runtime no-op | Write mode exits cleanly but produces no runtime-owned changes | Covered in `crates/xtask/tests/agent_maintenance_execute.rs` | Treat as failure. The lane does not get credit for doing nothing. | Explicit CLI failure and validation report | No |
| Green gates | One declared gate fails after write | Covered by gate execution flow and live run | Stop the run. Fix the underlying issue. If the fix touched packet truth, restart from dry-run. | Explicit gate failure in validation report | No |
| Closeout truth | `resolved_findings` still match live drift, `explicit_none_reason` is used while drift exists, or deferred findings are incomplete | Covered in `crates/xtask/tests/agent_maintenance_closeout/live_drift_validation.rs` | `close-agent-maintenance` fails closed. Fix the closeout JSON or fix the remaining drift. | Explicit closeout validation error | No |
| Proof archive truth | `proof-notes.md`, `request-sha256.txt`, or `run-id.txt` describe the wrong successful run | Not runtime-tested; operationally enforced by this plan | Treat as proof failure. Rewrite the archive from the actual successful run before claiming success. | Review-time mismatch between archive and run packet | Yes, if left uncorrected |

There are no silent-failure paths in the base plan. Every meaningful failure must end in an
explicit command error or an explicit archive review failure before the milestone can be marked
done.

## Test Review

### Test Framework And Commands

This is a Rust workspace. The relevant verification surfaces are:

- unit and integration tests via `cargo test -p xtask`
- command-level regression coverage in `crates/xtask/tests/agent_maintenance_execute.rs`
- closeout truth coverage in `crates/xtask/tests/agent_maintenance_closeout/**`
- contract and repo-wide gates via the packet-declared green commands and final `make preflight`

### Code Path Coverage

```text
CODE PATH COVERAGE
===========================
[+] docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml
    │
    ├── request truth audit
    │   ├── [LIVE PROOF REQUIRED] request fields still match branch truth
    │   └── [LIVE PROOF REQUIRED] request SHA recorded into archive
    │
    └── HANDOFF.md alignment
        ├── [★★★ TESTED] renderer owns command, surface, and closeout wording
        └── [LIVE PROOF REQUIRED] current handoff still matches the frozen request

[+] execute-agent-maintenance --dry-run
    │
    ├── [★★★ TESTED] preflight pass path
    ├── [★★★ TESTED] preflight failure path
    ├── [★★★ TESTED] dry-run writes only under temp run root
    └── [LIVE PROOF REQUIRED] real opencode dry-run on current staging head

[+] prepared packet continuity
    │
    ├── [★★★ TESTED] prompt mismatch rejection
    ├── [★★★ TESTED] request and contract drift rejection
    └── [LIVE PROOF REQUIRED] run_id reused only against unchanged packet truth

[+] execute-agent-maintenance --write
    │
    ├── [★★★ TESTED] boundary violation rejection
    ├── [★★★ TESTED] no-op rejection
    ├── [★★★ TESTED] green gates run in order
    ├── [★★★ TESTED] closeout remains manual
    └── [LIVE PROOF REQUIRED] real opencode write succeeds on the frozen packet

[+] close-agent-maintenance
    │
    ├── [★★★ TESTED] resolved findings cannot still match live drift
    ├── [★★★ TESTED] explicit-none is rejected when live drift exists
    ├── [★★★ TESTED] deferred findings must account for live drift
    ├── [★★★ TESTED] deferred findings are rejected when live report is clean
    ├── [★★★ TESTED] live drift re-check errors block closeout
    └── [LIVE PROOF REQUIRED] real opencode closeout succeeds after the bounded write

[+] proof archive
    │
    ├── [LIVE PROOF REQUIRED] archive copies match the final successful run packet
    └── [LIVE PROOF REQUIRED] proof notes describe the actual request SHA, run_id, and closeout result

─────────────────────────────────
COVERAGE: implementation guardrails are automated; milestone proof remains operational
AUTOMATED COVERAGE: strong on dry-run, write, boundary, and closeout truth
LIVE GAPS TO SATISFY: 6 operational proof points, all closed by this milestone
REGRESSION RULE: if the live run exposes a missing guardrail, add the regression test before claiming success
─────────────────────────────────
```

### Test Requirements

1. Run a focused regression pass before the live write:
   `cargo test -p xtask agent_maintenance_execute -- --nocapture` and
   `cargo test -p xtask agent_maintenance_closeout -- --nocapture`,
   or fall back to `cargo test -p xtask`.
2. Treat the live dry-run and live write as proof tests, not just operator steps. Archive their
   outputs exactly.
3. If the live run exposes any missing invariant in `workflow.rs`, `validate.rs`, or closeout
   validation, add or update the matching `crates/xtask/tests/**` regression before archiving the
   final proof.
4. Finish with the exact green gates and final `make preflight` on the final proof-bearing head.

## Implementation Plan

### Phase 1: Freeze Request Truth

**Goal:** start from one stable request and one stable proof boundary.

**Inputs**

- `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`
- `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md`
- `docs/agents/lifecycle/opencode-maintenance/governance/proof/proof-notes.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

**Actions**

1. Verify the request still points at the intended `target_version`, `branch_name`, and
   `request_commit`.
2. Verify `HANDOFF.md` still matches the request's execution contract.
3. Verify the rendered prompt digest in the request still matches the current prompt template.
4. Decide whether the existing queue and PR-open proof artifacts are still truthful enough to keep.
5. If request truth drifted, run:

```sh
cargo run -p xtask -- refresh-agent --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --write
```

6. If Phase 1 changed any request-owned truth, commit those fixes before starting the dry-run
   proof so the request SHA and archive evidence stay crisp.

**Outputs**

- one canonical live request
- one matching `HANDOFF.md`
- proof archive notes that no longer claim stale request truth

**Exit criteria**

- a maintainer can point at one canonical request and say "this is the packet we are proving"

### Phase 2: Prepare A Fresh Dry-Run Packet

**Goal:** freeze one prepared `run_id` for the actual write attempt.

**Command**

```sh
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --dry-run
```

**Expected outcomes**

- exit code `0`
- CLI output includes `run_id`, `run_dir`, and the exact closeout command
- run packet written under `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`
- `validation-report.json` records `status = "pass"`
- `run-status.json` records `status = "dry_run_ready"`

**Archive outputs**

- `request-sha256.txt`
- `run-id.txt`
- `execute-dry-run.txt`
- `validation-report-dry-run.json`
- `run-status-dry-run.json`

**Exit criteria**

- there is one fresh prepared run packet and one committed `run-id.txt` pointing at it

### Phase 3: Execute Write Mode Against The Frozen Packet

**Goal:** prove that the packet can land the bounded maintenance update.

**Command**

```sh
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --write --run-id <run_id>
```

**Expected outcomes**

- exit code `0`
- `validation-report.json` records `status = "pass"`
- `run-status.json` records `status = "write_validated"`
- `run-status.json` records `validation_passed = true`
- `written-paths.json` is non-empty
- every written path matches the frozen `writable_surfaces`
- the closeout path stays untouched
- the exact `green_gates` run in order and all pass

**Archive outputs**

- `execute-write.txt`
- `validation-report-write.json`
- `run-status-write.json`
- `written-paths-write.json`

**Exit criteria**

- the repo has one truthful write-mode diff produced by the live packet

### Phase 4: Resolve Any Surfaced Gap And Rerun Cleanly

**Goal:** prevent false proof if the first write exposes a hidden assumption.

**Loop rule**

If Phase 3 fails because of packet truth, writable surfaces, a green gate, a no-op write, or a
missing guardrail:

1. fix the issue inside the bounded blast radius
2. add or update the matching regression test if the failure exposed a missing invariant
3. rerun Phase 2 from a fresh dry-run if the fix changed packet-owned truth
4. rerun Phase 3
5. archive only the final successful run

Do not keep a proof archive that mixes evidence from a failed prepared packet and a later
successful packet.

### Phase 5: Author Closeout And Run Manual Closeout

**Goal:** prove the explicit maintainer closeout step the operator guide requires.

**Closeout file**

`docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json`

**Required fields**

- `request_ref`
- `request_sha256`
- `resolved_findings`
- exactly one of:
  - `deferred_findings`
  - `explicit_none_reason`
- `preflight_passed`
- `recorded_at`
- `commit`

**Command**

```sh
cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json
```

**Expected outcomes**

- exit code `0`
- refreshed `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md`
- refreshed `docs/agents/lifecycle/opencode-maintenance/governance/remediation-log.md`
- written `maintenance-closeout.json`
- live drift validation passes for all resolved findings

**Archive output**

- `closeout-summary.md`

**Exit criteria**

- the maintenance lane is explicitly closed, not implicitly assumed closed

### Phase 6: Finalize The Proof Archive And Re-Run Final Verification

**Goal:** leave one replayable, reviewable closed proof at the final branch head.

**Actions**

1. Update `proof-notes.md` so it names the final request SHA, final `run_id`, whether a rerun was
   required, and the final closeout result.
2. Verify every committed proof artifact matches the final successful temp run packet.
3. Run the final verification sequence from the final closeout state:

```sh
cargo fmt --all
cargo run -p xtask -- codex-validate --root cli_manifests/opencode
cargo run -p xtask -- support-matrix --check
cargo run -p xtask -- capability-matrix --check
cargo run -p xtask -- capability-matrix-audit
make preflight
```

4. Review the final diff to confirm it still reads as one bounded maintenance run.

**Exit criteria**

- proof archive is truthful
- final branch head is green
- the diff stays inside the expected maintenance blast radius

## Worktree Parallelization Strategy

This milestone needs a parallelization section, but the honest answer is that the happy path is
mostly sequential.

The same live request, the same frozen `run_id`, the same temp run packet, and the same proof root
all form one chain of custody. Splitting that chain across worktrees before the first successful
write is how you create stale packets and merge noise.

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Freeze request truth | `docs/agents/lifecycle/opencode-maintenance/`, `docs/cli-agent-onboarding-factory-operator-guide.md` | — |
| Prepare dry-run packet | `docs/agents/.uaa-temp/agent-maintenance/`, `docs/agents/lifecycle/opencode-maintenance/governance/proof/` | Freeze request truth |
| Execute write mode | `crates/opencode/`, `crates/agent_api/`, `cli_manifests/opencode/`, `docs/specs/unified-agent-api/`, `docs/agents/.uaa-temp/agent-maintenance/` | Prepare dry-run packet |
| Fix surfaced gap, if any | Usually `crates/xtask/`, `crates/opencode/`, `crates/agent_api/`, `docs/agents/lifecycle/opencode-maintenance/`, `cli_manifests/opencode/` | Execute write mode |
| Author closeout and close lane | `docs/agents/lifecycle/opencode-maintenance/` | Successful write mode |
| Final archive + preflight | `docs/agents/lifecycle/opencode-maintenance/governance/proof/`, repo-wide checks | Successful closeout |

### Parallel Lanes

Base case:

- Lane A: Freeze request truth -> Prepare dry-run packet -> Execute write mode -> Closeout ->
  Final archive

That is sequential because each step consumes truth created by the prior step.

Conditional case, only if Phase 3 exposes a real code gap:

- Lane A: preserve failing run evidence and update proof notes draft
- Lane B: fix the bounded code or contract issue and add the matching regression test

Even then, merge Lane B back before rerunning dry-run. Do not let two worktrees create competing
request truth or competing proof artifacts.

### Execution Order

1. Run the happy path in one worktree.
2. Only split into a second worktree if the first live write exposes a bounded code gap that can be
   fixed independently.
3. Merge the fix worktree back.
4. Rerun dry-run and write from the merged truth in the primary worktree.

### Conflict Flags

- `docs/agents/lifecycle/opencode-maintenance/**` is shared by nearly every phase. Parallel edits
  there are likely to conflict.
- `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/` is run-specific and should remain owned
  by the active proof worktree.
- `docs/agents/lifecycle/opencode-maintenance/governance/proof/**` must describe one final
  successful run. Do not let multiple worktrees write competing proof narratives.

**Parallelization verdict:** sequential implementation on the happy path. A second worktree is
justified only for a bounded repair after a failed live write.

## Completion Summary

- Step 0: Scope Challenge, complete. Scope stays on the live `opencode` packet.
- Architecture Review: complete. No new architecture required.
- Code Quality Review: complete. Reuse existing executor and closeout surfaces. Do not add a new
  workflow family or archive system.
- Test Review: complete. Guardrails are already strong; live dry-run, live write, and live closeout
  remain the proof obligations.
- Performance Review: complete. This milestone is bounded by command execution and repo validation,
  not by a new hot path.
- NOT in scope: written.
- What already exists: written.
- Failure modes: written. The only critical gap is proof-archive truth if it is left stale.
- Parallelization: written. Happy path is sequential; conditional repair lane only after a failed
  live write.
- Lake score: the complete option wins. This plan proves the whole lane, not just packet opening.
