# ORCH_PLAN - Prove The Packet PR Maintenance Lane Can Actually Land

Status: ready for implementation  
Date: 2026-05-12  
Working branch: `staging`  
Plan revision baseline: `d6b86cdc`  
Design input: `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-design-20260511-185442.md`  
Supersedes: prior repo-root `PLAN.md` for dry-run-only packet proof readiness

## Summary

- Branch context: execute on the current `staging` branch only; the live packet itself declares maintenance branch `automation/opencode-maintenance-1.14.47`.
- Objective: prove the current live `opencode` automated maintenance packet can complete a real bounded write, pass the exact packet-declared green gates, remain inside the exact packet-declared writable surfaces, be manually closed with the repo-owned closeout command, and leave committed replayable proof artifacts.
- Parent role: the parent agent is the sole orchestrator, sole integrator, and sole owner of all serialized critical-path operations.
- Worker model: optional bounded worker lanes only; prefer `GPT-5.4` with `reasoning_effort=high`.
- Concurrency cap: one optional repair lane at a time, and only after a surfaced failure. Happy path is sequential.
- Worktree rule: the happy path stays in the primary `staging` worktree. Optional repair worktrees are allowed only for bounded remediation and may not run dry-run, write, or closeout.
- Run-state source of truth:
  - live request: `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`
  - canonical handoff: `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md`
  - prompt template: `docs/agents/lifecycle/opencode-maintenance/governance/execute-agent-maintenance-prompt.md`
  - active temp run state: `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`
  - canonical proof root: `docs/agents/lifecycle/opencode-maintenance/governance/proof/`
- Proof-artifact handling: `.uaa-temp/...` is run-state / derived evidence unless explicitly promoted into the proof root. Commit structured summaries and JSON by default. Raw `codex-stdout.txt` and `codex-stderr.txt` stay temp-only unless failure diagnosis requires promotion.
- Hard truth: another dry-run-only archive does not count. Proof is incomplete without both `execute-agent-maintenance --write` and manual `close-agent-maintenance`.

## Purpose

This orchestration plan executes the current live `PLAN.md` to completion for one target only:

`docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`

The milestone is not watcher detection, packet opening, or packet rendering. Those are already proven. The missing proof is the landing path:

1. fresh dry-run from the live request
2. one frozen reusable `run_id`
3. write mode against that exact frozen packet
4. exact green gates passing in order
5. manual closeout authored after write succeeds
6. repo-owned closeout command succeeding
7. replayable structured proof committed under the canonical proof root
8. final `make preflight` green on the proof-bearing head

## Definition Of Done

The session is complete only when all of the following are true:

1. `execute-agent-maintenance --dry-run` succeeds for the live request and yields a reusable `run_id`.
2. `execute-agent-maintenance --write --run-id <run_id>` succeeds against that exact frozen packet.
3. `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/run-status.json` records:
   - `status = "write_validated"`
   - `validation_passed = true`
4. All write-mode changes stay inside the exact request `writable_surfaces`.
5. Write mode does not create or modify `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json`.
6. The exact request `green_gates` pass in order:
   - `cargo fmt --all`
   - `cargo run -p xtask -- codex-validate --root cli_manifests/opencode`
   - `cargo run -p xtask -- support-matrix --check`
   - `cargo run -p xtask -- capability-matrix --check`
   - `cargo run -p xtask -- capability-matrix-audit`
   - `make preflight`
7. A truthful `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json` exists.
8. `cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json` succeeds.
9. `docs/agents/lifecycle/opencode-maintenance/governance/proof/` contains the final structured proof set:
   - existing queue / PR-open evidence if still truthful
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
10. `make preflight` is green on the final proof-bearing head.

## Locked Decisions And Hard Guards

1. Proof target is the current live `opencode` automated request only.
2. Proof is incomplete unless it includes both `execute-agent-maintenance --write` and manual `close-agent-maintenance`.
3. Another dry-run-only archive does not count.
4. Canonical proof archive root stays `docs/agents/lifecycle/opencode-maintenance/governance/proof/`.
5. Parent remains sole integrator and sole owner of serialized critical-path operations.
6. If request, prompt, writable surfaces, green gates, target version, branch name, closeout path, closeout command, or request SHA drift after dry-run, the prepared packet is invalid. Rerun dry-run before write.
7. If write succeeds but exposes a contract gap inside bounded surfaces, fix the gap, rerender if needed, rerun dry-run, rerun write, and archive only the final successful run.
8. Manual closeout remains manual by design; do not automate it inside `execute-agent-maintenance`.
9. Raw `codex-stdout.txt` and `codex-stderr.txt` remain temp evidence under `.uaa-temp`; do not commit them unless a real failure requires them.
10. No worker lane may run live dry-run, live write, or live closeout.
11. No proof artifact may describe a failed or superseded run as the final proof.

## Live Request Contract

### Canonical Request

`docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`

### Preserved Request Truth

- `agent_id = "opencode"`
- `executor = "execute-agent-maintenance"`
- `prompt_template_path = "docs/agents/lifecycle/opencode-maintenance/governance/execute-agent-maintenance-prompt.md"`
- `prompt_sha256 = "f68f4a5c6cc09a186256fe475e311bd4881e6dfeabd7852f1ed62cf659ce9685"`
- `closeout_path = "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json"`
- `requires_manual_closeout = true`
- `target_version = "1.14.47"`
- `branch_name = "automation/opencode-maintenance-1.14.47"`

### Exact Writable Surfaces

- `docs/agents/lifecycle/opencode-maintenance/**`
- `crates/opencode/**`
- `crates/agent_api/**`
- `cli_manifests/opencode/artifacts.lock.json`
- `cli_manifests/opencode/snapshots/1.14.47/**`
- `cli_manifests/opencode/reports/1.14.47/**`
- `cli_manifests/opencode/versions/1.14.47.json`
- `cli_manifests/opencode/wrapper_coverage.json`
- `cli_manifests/support_matrix/current.json`
- `docs/specs/unified-agent-api/support-matrix.md`

### Exact Green Gates

- `cargo fmt --all`
- `cargo run -p xtask -- codex-validate --root cli_manifests/opencode`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix --check`
- `cargo run -p xtask -- capability-matrix-audit`
- `make preflight`

## Run-State / Checkpoint Model

The happy path is serialized. Each checkpoint freezes truth for the next phase. If a later action invalidates an earlier checkpoint, execution returns to the earliest invalidated checkpoint and reruns forward from there.

### Checkpoint C0 - Baseline Captured

State frozen:

- current `staging` head
- current live request file
- current `HANDOFF.md`
- current proof root contents
- current regression baseline intent

Invalidated by:

- manual edits to request-owned surfaces before request freeze is completed
- branch/head movement that changes request-owned truth before `C1`

Rerun implication:

- repeat baseline audit before claiming any later checkpoint

### Checkpoint C1 - Request Truth Frozen

State frozen:

- canonical request contents
- canonical request SHA
- prompt continuity
- handoff alignment
- proof-root keep/replace decisions for existing artifacts

Invalidated by:

- any change to request contents
- any change to rendered prompt contents or prompt digest
- any change to `target_version`
- any change to `branch_name`
- any change to `writable_surfaces`
- any change to `green_gates`
- any change to `closeout_path`
- any change to the exact closeout command contract

Rerun implication:

- return to request freeze, then rerun regressions if affected, then rerun dry-run and everything after it

### Checkpoint C2 - Regression Baseline Green

State frozen:

- executor and closeout regression baseline is green enough to trust the live run

Invalidated by:

- changes to executor surfaces
- changes to closeout surfaces
- changes to matching regression tests
- any repair that touches `crates/xtask/src/agent_maintenance/**` or `crates/xtask/tests/**`

Rerun implication:

- rerun focused regressions before any dry-run or rerun attempt

### Checkpoint C3 - Dry-Run Packet Frozen

State frozen:

- one active `run_id`
- one active temp run directory
- one active request SHA
- one dry-run validation result
- one dry-run-ready packet eligible for write

Invalidated by:

- any packet-owned truth drift after dry-run
- any executor repair that affects packet semantics
- any ambiguity about which `run_id` is active

Rerun implication:

- discard the old packet for proof purposes and rerun dry-run to mint a new active `run_id`

### Checkpoint C4 - Write Proof Green

State frozen:

- one successful write on the frozen packet
- one bounded diff
- one validated run-status showing `write_validated`
- one successful ordered green-gate sequence
- one confirmed no-closeout-from-write proof

Invalidated by:

- discovery that write escaped `writable_surfaces`
- discovery that write transcript or run-status mismatches proof claims
- post-write repair to packet-owned truth or write-owned repo surfaces that changes what the proof run actually was

Rerun implication:

- repair, then rerun dry-run and write; archive only the final successful run

### Checkpoint C5 - Closeout Green

State frozen:

- truthful `maintenance-closeout.json`
- successful `close-agent-maintenance`
- refreshed owned closeout outputs

Invalidated by:

- closeout validation finding drift or unresolved findings
- edits to closeout surfaces that make the recorded closeout untruthful
- final proof review showing the closeout summary misstates the actual run

Rerun implication:

- fix closeout truth and rerun closeout; rerun write only if closeout truth depends on invalidating the write proof itself

### Checkpoint C6 - Final Proof Head Green

State frozen:

- proof root is truthful and complete
- final repo verification sequence is green
- final head remains a bounded maintenance diff

Invalidated by:

- any proof artifact mismatch
- any final gate failure
- any unrelated spillover diff that breaks bounded-scope reviewability

Rerun implication:

- fix only the invalidated phase, then rerun forward to final head green

## Worktree And Branch Strategy

### Primary Worktree

All happy-path execution stays in the primary repository worktree on branch `staging`.

The parent owns:

- all request freeze work
- all dry-run/write/closeout execution
- all proof promotion and proof editing
- final verification
- final integration

### Optional Repair Worktree Root

If a surfaced failure justifies isolated repair, create it under a bounded local root such as:

`wt/opencode-live-proof/`

Suggested repair branches:

- `codex/opencode-proof-fix-executor`
- `codex/opencode-proof-fix-closeout`
- `codex/opencode-proof-fix-proof-archive`

### Repair Worktree Rules

1. A repair worktree is optional, not default.
2. A repair worktree must own one bounded issue only.
3. A repair worktree may patch code, tests, or proof drafting surfaces within its assigned scope.
4. A repair worktree may not run:
   - `execute-agent-maintenance --dry-run`
   - `execute-agent-maintenance --write`
   - `close-agent-maintenance`
5. A repair worktree may not promote proof artifacts into the canonical proof root as final truth.
6. A repair worktree returns only:
   - narrow diff
   - test result summary if relevant
   - brief explanation of what changed and why
7. Parent merges repair worktree changes back into the primary worktree before any rerun.
8. All rerun commands happen only from the primary worktree after integration.

## Ownership Model

## Parent

Role: sole orchestrator, sole integrator, sole owner of critical-path serialized operations.

Parent-only responsibilities:

- baseline capture
- request truth audit and freeze
- regression gate decision
- active `run_id` selection
- live dry-run
- live write
- diff review for bounded scope
- closeout authoring/finalization
- live closeout command
- proof promotion
- final verification
- success/failure declaration for checkpoints and workstreams

## Optional Worker Lanes

Preferred configuration:

- model: `GPT-5.4`
- `reasoning_effort=high`

Allowed worker roles:

- read-only audit
- bounded repair
- closeout/proof draft support after final truth is already known

Prohibited worker roles:

- packet authority
- active run authority
- dry-run authority
- write authority
- closeout authority
- final proof authority
- final gate authority

## Workstream Plan

## WS0 - Baseline Capture

- ID: `WS0`
- Owner: `parent`
- Launch gate: none
- Owned file surfaces:
  - `PLAN.md`
  - `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`
  - `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md`
  - `docs/agents/lifecycle/opencode-maintenance/governance/proof/**`
  - `docs/agents/lifecycle/opencode-maintenance/governance/remediation-log.md`
- Required commands:
  - no mandatory mutation command
  - read-only inspection of live plan and contract surfaces
- Stop conditions:
  - ambiguity about current request truth
  - ambiguity about which prior proof artifacts remain truthful
- Acceptance:
  - baseline is captured
  - parent can identify the exact live request, handoff, proof root, and prior proof status
  - `C0` passes

## WS1 - Freeze Request Truth

- ID: `WS1`
- Owner: `parent`
- Launch gate: `C0`
- Owned file surfaces:
  - `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`
  - `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md`
  - `docs/agents/lifecycle/opencode-maintenance/governance/execute-agent-maintenance-prompt.md`
  - `docs/agents/lifecycle/opencode-maintenance/governance/proof/proof-notes.md`
  - existing proof artifacts retained or replaced for truth
- Required commands:
  - if truth drift exists:
    ```sh
    cargo run -p xtask -- refresh-agent --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --write
    ```
- Stop conditions:
  - request drift not resolved
  - prompt digest mismatch not resolved
  - handoff mismatch not resolved
  - uncertainty about retained prior proof artifacts
- Acceptance:
  - one canonical request SHA is frozen
  - handoff matches request truth
  - prompt continuity is valid
  - retained proof artifacts are explicitly known to still be truthful
  - `C1` passes

## WS2 - Regression Baseline

- ID: `WS2`
- Owner: `parent`
- Launch gate: `C1`
- Owned file surfaces:
  - `crates/xtask/src/agent_maintenance/execute/workflow.rs`
  - `crates/xtask/src/agent_maintenance/execute/runtime.rs`
  - `crates/xtask/src/agent_maintenance/execute/validate.rs`
  - `crates/xtask/src/agent_maintenance/execute/packet.rs`
  - `crates/xtask/src/agent_maintenance/closeout.rs`
  - `crates/xtask/src/agent_maintenance/closeout/write.rs`
  - `crates/xtask/src/agent_maintenance/closeout/validate.rs`
  - `crates/xtask/tests/agent_maintenance_execute.rs`
  - `crates/xtask/tests/agent_maintenance_closeout/**`
- Required commands:
  ```sh
  cargo test -p xtask agent_maintenance_execute -- --nocapture
  cargo test -p xtask agent_maintenance_closeout -- --nocapture
  ```
  - fallback:
    ```sh
    cargo test -p xtask
    ```
- Stop conditions:
  - focused regression failure
  - executor or closeout invariant gap surfaced before live write
- Acceptance:
  - regression baseline is green or repaired and re-green
  - if repairs touched these surfaces, the request-freeze implications are evaluated before proceeding
  - `C2` passes

## WS3 - Prepare Fresh Dry-Run Packet

- ID: `WS3`
- Owner: `parent`
- Launch gate: `C2`
- Owned file surfaces:
  - `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/**`
  - `docs/agents/lifecycle/opencode-maintenance/governance/proof/request-sha256.txt`
  - `docs/agents/lifecycle/opencode-maintenance/governance/proof/run-id.txt`
  - `docs/agents/lifecycle/opencode-maintenance/governance/proof/execute-dry-run.txt`
  - `docs/agents/lifecycle/opencode-maintenance/governance/proof/validation-report-dry-run.json`
  - `docs/agents/lifecycle/opencode-maintenance/governance/proof/run-status-dry-run.json`
- Required commands:
  ```sh
  cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --dry-run
  ```
- Stop conditions:
  - non-zero dry-run exit
  - preflight failure
  - missing or ambiguous `run_id`
  - dry-run output inconsistent with request truth
- Acceptance:
  - one active `run_id` exists
  - dry-run packet is ready and frozen
  - proof seed artifacts were promoted from the matching temp run
  - `C3` passes

## WS4 - Execute Write On Frozen Packet

- ID: `WS4`
- Owner: `parent`
- Launch gate: `C3`
- Owned file surfaces:
  - temp run dir for the active `run_id`
  - all packet-declared writable surfaces only
  - `docs/agents/lifecycle/opencode-maintenance/governance/proof/execute-write.txt`
  - `docs/agents/lifecycle/opencode-maintenance/governance/proof/validation-report-write.json`
  - `docs/agents/lifecycle/opencode-maintenance/governance/proof/run-status-write.json`
  - `docs/agents/lifecycle/opencode-maintenance/governance/proof/written-paths-write.json`
- Required commands:
  ```sh
  cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --write --run-id <run_id>
  ```
- Stop conditions:
  - non-zero write exit
  - packet continuity failure
  - boundary violation
  - no-op write rejection
  - any green gate failure
  - evidence that closeout path was written by write mode
  - ambiguous or unreviewable diff scope
- Acceptance:
  - write succeeded on the frozen packet
  - `run-status.json` reports `write_validated` and `validation_passed = true`
  - all written paths stay within exact writable surfaces
  - exact green gates passed in order
  - closeout path remained untouched
  - promoted write artifacts match the active successful temp run
  - `C4` passes

## WS5 - Conditional Repair Lane

- ID: `WS5`
- Owner: `optional worker lane` for patching, `parent` for integration and rerun decisions
- Launch gate: failure in `WS2`, `WS4`, `WS6`, or `WS7`
- Owned file surfaces:
  - only the bounded failing surface assigned by parent
  - examples:
    - executor surfaces under `crates/xtask/src/agent_maintenance/**`
    - closeout validation surfaces under `crates/xtask/src/agent_maintenance/closeout/**`
    - matching regressions under `crates/xtask/tests/**`
    - proof text under `docs/agents/lifecycle/opencode-maintenance/governance/proof/**`
- Required commands:
  - none required in worker lane
  - worker may run only narrow local tests approved by parent
  - parent reruns authoritative gates after integration
- Stop conditions:
  - repair scope expands beyond bounded assignment
  - worker attempts to run live dry-run/write/closeout
  - repair changes packet-owned truth without parent re-freeze
- Acceptance:
  - worker returns a narrow diff, concise explanation, and any relevant local test result
  - parent integrates repair into primary worktree
  - parent evaluates whether `C1`, `C2`, or `C3` was invalidated
  - reruns restart from the earliest invalidated checkpoint
  - failed run artifacts are not promoted as final proof

## WS6 - Manual Closeout

- ID: `WS6`
- Owner: `parent`
- Launch gate: `C4`
- Owned file surfaces:
  - `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json`
  - `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md`
  - `docs/agents/lifecycle/opencode-maintenance/governance/remediation-log.md`
  - `docs/agents/lifecycle/opencode-maintenance/governance/proof/closeout-summary.md`
- Required commands:
  ```sh
  cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json
  ```
- Stop conditions:
  - closeout JSON is untruthful or incomplete
  - resolved findings still match live drift
  - explicit none reason is used while drift still exists
  - closeout command fails validation
- Acceptance:
  - truthful closeout JSON exists
  - closeout command succeeds
  - owned closeout outputs refresh successfully
  - proof root contains truthful `closeout-summary.md`
  - `C5` passes

## WS7 - Final Proof Archive And Final Verification

- ID: `WS7`
- Owner: `parent`
- Launch gate: `C5`
- Owned file surfaces:
  - full proof root:
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
    - existing queue / PR-open evidence if still truthful
  - final repo head across packet-declared writable surfaces
- Required commands:
  ```sh
  cargo fmt --all
  cargo run -p xtask -- codex-validate --root cli_manifests/opencode
  cargo run -p xtask -- support-matrix --check
  cargo run -p xtask -- capability-matrix --check
  cargo run -p xtask -- capability-matrix-audit
  make preflight
  ```
- Stop conditions:
  - proof artifact mismatch with final successful run
  - final gate failure
  - final diff no longer reads as one bounded maintenance run
- Acceptance:
  - proof archive is complete and truthful
  - final repo gates are green
  - final head remains bounded and reviewable
  - `C6` passes

## Context-Control Rules

1. Parent context retains:
   - live request truth
   - current checkpoint status
   - active `run_id`
   - packet invalidation rules
   - proof archive promotion decisions
   - final gate status
2. Workers receive only the smallest context slice necessary for the assigned task.
3. Workers must not receive broad “fix the repo” authority.
4. Workers must not receive authority to decide checkpoint passage.
5. Workers must not run:
   - live dry-run
   - live write
   - live closeout
   - final repo verification sequence
6. Workers must return:
   - narrow diff or read-only finding set
   - concise explanation of what changed or what failed
   - any relevant local test output summary
   - explicit note if their change may invalidate `C1`, `C2`, or `C3`
7. Parent reviews only narrow worker diffs and concise summaries before integration.
8. Parent verifies every worker claim against local repo truth before merging or rerunning.
9. Parent promotes only final successful structured evidence into the proof root.
10. If a worker touches packet-owned truth, parent must treat the active packet as potentially invalid until request freeze is re-evaluated.
11. If a worker patch touches executor or closeout contract logic, parent must treat regression baseline as potentially invalid until rerun.

## Failure Handling And Rerun Policy

### If Dry-Run Fails

- fix local execution-host preflight or packet truth issue
- rerun from `WS1` or `WS2`, depending on whether request truth changed
- do not fabricate a `run_id`

### If Write Fails

- preserve temp run evidence
- determine whether failure is:
  - packet drift
  - boundary violation
  - no-op rejection
  - green gate failure
  - executor invariant gap
- repair only the bounded failing surface
- rerun from earliest invalidated checkpoint
- archive only the final successful run

### If Write Succeeds But Exposes A Contract Gap

- this is not “success with follow-up”
- fix the gap
- add or update matching regression if invariant was missing
- rerun dry-run and write if packet truth or write truth changed
- only final successful rerun is archived

### If Closeout Fails

- fix closeout truth or underlying unresolved drift
- rerun closeout
- rerun write only if the closeout fix invalidates what the write proof was claiming

### If Final Proof Review Fails

- correct the mismatched proof artifact
- rerun any invalidated gate
- do not leave mixed evidence from different runs in the proof root

## Tests And Acceptance

## Request Freeze

Acceptance targets:

- request still identifies `opencode`
- prompt path is exactly `docs/agents/lifecycle/opencode-maintenance/governance/execute-agent-maintenance-prompt.md`
- target version remains `1.14.47`
- packet branch remains `automation/opencode-maintenance-1.14.47`
- exact writable surfaces match live request
- exact green gates match live request and handoff
- handoff remains aligned with request
- request SHA used for proof is explicit and singular

Failure here means no dry-run is allowed yet.

## Regression Baseline

Required commands:

```sh
cargo test -p xtask agent_maintenance_execute -- --nocapture
cargo test -p xtask agent_maintenance_closeout -- --nocapture
```

Fallback:

```sh
cargo test -p xtask
```

Acceptance targets:

- executor regressions are green
- closeout regressions are green
- any newly exposed invariant gap is repaired and covered before live proof continues

## Dry-Run Proof

Required command:

```sh
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --dry-run
```

Acceptance targets:

- exit code `0`
- one active `run_id`
- one active temp run directory
- dry-run validation passes
- `run-status.json` records dry-run-ready state
- proof root contains matching dry-run artifacts

## Write Proof

Required command:

```sh
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --write --run-id <run_id>
```

Acceptance targets:

- exit code `0`
- exact frozen `run_id` reused
- `run-status.json` records `status = "write_validated"`
- `run-status.json` records `validation_passed = true`
- write is non-empty
- write stays inside exact writable surfaces
- closeout path remains untouched
- exact green gates pass in order
- proof root contains matching write artifacts

## Closeout

Required command:

```sh
cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json
```

Acceptance targets:

- truthful `maintenance-closeout.json`
- closeout validation succeeds
- refreshed handoff and remediation outputs are consistent
- proof root contains truthful `closeout-summary.md`

## Proof Archive

Acceptance targets:

- proof root contains required final structured files
- `run-id.txt` points to the final successful run only
- `request-sha256.txt` matches the final successful run’s request truth
- `proof-notes.md` names the final request SHA, final `run_id`, rerun status, and closeout result truthfully
- prior queue / PR-open evidence is retained only if still truthful
- raw temp logs stay uncommitted unless explicitly needed for failure diagnosis

## Final Repo Gates

Required commands:

```sh
cargo fmt --all
cargo run -p xtask -- codex-validate --root cli_manifests/opencode
cargo run -p xtask -- support-matrix --check
cargo run -p xtask -- capability-matrix --check
cargo run -p xtask -- capability-matrix-audit
make preflight
```

Acceptance targets:

- all final commands pass
- final diff remains bounded to the maintenance lane
- final head is the proof-bearing head for the completed session

## Assumptions

1. The live request at `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml` remains the canonical proof input.
2. The working branch remains `staging` for the full session.
3. The proof target remains `opencode` `1.14.47` unless legitimate request rerendering in request freeze changes the canonical truth.
4. Existing queue / PR-open proof artifacts may remain only if still truthful relative to the final successful run.
5. `.uaa-temp` run-state is derived evidence unless explicitly promoted into the proof root.
6. The executor and closeout implementations are expected to be largely correct; remaining work is live proof and bounded repair, not architecture redesign.
7. Happy-path orchestration is mostly sequential, and that is a real property of the milestone rather than a planning deficiency.

## Session Close Criteria

The parent may close the PLAN session only after:

1. `C0` through `C6` have passed in order.
2. The final successful `run_id` is recorded in the proof root.
3. The manual closeout command has succeeded.
4. The proof root contains the required replayable structured artifacts.
5. `make preflight` is green on the final proof-bearing head.

Until then, the lane is not proven landed.
