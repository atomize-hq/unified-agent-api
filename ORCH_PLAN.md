# ORCH_PLAN - Execute Shared Packet-PR Support-Uplift Convergence

Status: ready for implementation  
Date: 2026-05-13  
Working branch: `staging`  
Plan revision baseline: `b5ba0d73`  
Primary design input:
- `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-design-20260513-112453.md`

Supersedes:
- the prior repo-root `ORCH_PLAN.md` for the `opencode`-only proof session

## Summary

- Branch context: the parent agent integrates on the existing `staging` branch. No long-lived
  alternate integration branch is introduced for this session.
- Objective: execute the current `PLAN.md` to completion by freezing the support-aware maintenance
  contract, converging `codex` and `claude_code` onto shared `packet_pr`, aligning packet
  derivation plus relay semantics, refreshing committed lifecycle and publication surfaces, and
  finishing with shared regression coverage plus one real migrated `codex` proof.
- Parent role: the parent agent is the sole orchestrator, sole integrator, sole checkpoint owner,
  and sole final gate owner.
- Parallelization model:
  - Phase 1 is fully serial and ends with one frozen contract baseline commit.
  - After that freeze, three disjoint code lanes may run in parallel:
    - Lane A: packet derivation plus packet-owned docs
    - Lane B: relay validation and execution enforcement
    - Lane C: transport convergence
  - Integration doc refresh is serial after A, B, and C merge.
  - Regression coverage plus the migrated `codex` proof is last.
- Worktree model: lane worktrees branch from the exact Phase 1 freeze commit under local `wt/`
  roots and return bounded diffs only. The parent merges them one at a time back into `staging`.
- Proof model: `codex` is the required first migrated proof. `claude_code` must reach transport and
  contract parity in code and tests in the same milestone, but it does not need to be the first
  live proof lane.
- Hard truth: the support-audit schema frozen in `PLAN.md` Phase 1 is not negotiable downstream.
  If that schema changes, parallelization stops and every downstream lane re-bases from a new
  contract-freeze commit.

## Run-State Sources Of Truth

- current plan intent: `PLAN.md`
- current orchestration contract: `ORCH_PLAN.md`
- serial-to-parallel baseline: the recorded Phase 1 freeze SHA on `staging`
- active lane map: parent-maintained worktree path -> branch -> owned-surface mapping for A, B,
  and C
- canonical migrated proof request: `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
- canonical migrated proof root: `docs/agents/lifecycle/codex-maintenance/governance/proof/`
- local parallel scratch root: `wt/` only; never committed

## Definition Of Done

The session is complete only when all of the following are true:

1. Phase 1 leaves one frozen normative contract across:
   - `docs/specs/maintenance-request-contract-v1.md`
   - `docs/specs/agent-registry-contract.md`
   - `docs/specs/cli-agent-onboarding-charter.md`
   - `docs/specs/unified-agent-api/support-matrix.md`
   - `docs/specs/unified-agent-api/non-tui-support-debt.md`
2. `docs/cli-agent-maintenance-steady-state-plan.md` is retired as a live planning surface and the
   operator guide plus workflow atlas point back to the same normative contract.
3. Shared packet derivation emits the frozen `support_surface_audit` schema and shared packet-owned
   docs describe bounded non-TUI support uplift rather than artifact refresh only.
4. Shared relay validation fails closed when the support-aware contract is missing, inconsistent,
   under-scoped, or launders unsupported gaps through invalid deferrals.
5. `crates/xtask/data/agent_registry.toml`, watcher code, and workflow YAML all treat shared
   `packet_pr` plus `agent-maintenance-open-pr.yml` as the live enrolled transport for `codex`,
   `claude_code`, and `opencode`.
6. Worker-specific steady-state transport for `codex` and `claude_code` is retired or explicitly
   demoted to unscheduled historical/manual-only status with no registry-driven role.
7. Integration doc refresh lands after the merged code lanes and materializes any now-required
   `claude_code` maintenance packet surface at the shared renderer's `agent_id`-derived
   maintenance root, for example `docs/agents/lifecycle/claude_code-maintenance/**`, if transport
   convergence makes it a steady-state lane.
8. Shared regression coverage protects packet derivation, relay enforcement, registry/watcher
   convergence, blocker taxonomy, debt-count ratchet, support publication contraction, and the
   migrated lane.
9. One real `codex` maintenance run succeeds through the shared watcher, shared opener, shared
   packet, shared relay, and manual closeout path, producing committed proof artifacts.
10. Final verification is green, including `make preflight`.

## Hard Guards

1. The parent agent is the only checkpoint authority and the only agent allowed to declare a phase
   complete.
2. The existing `staging` branch remains the primary integration branch for the full session.
3. Phase 1 is serial. No downstream lane starts before the parent records the exact contract-freeze
   commit SHA.
4. The Phase 1 freeze locks:
   - `support_surface_audit` field names
   - debt inventory row shape
   - allowed deferral taxonomy
   - eligibility reasons
   - the meaning of `required_uplifts_this_run[]`
5. The frozen support-audit schema is not renegotiated in packet prose, relay logic, workflow YAML,
   test fixtures, or downstream lane-local docs.
6. Workflow YAML owns transport only. It must not become a second policy store for support audit,
   writable-surface narrowing, ratchet rules, or green-gate semantics.
7. Lane A may not invent relay-only validation rules. Lane B may not rename packet schema or write
   new policy prose. Lane C may not encode agent-specific maintenance semantics in workflow YAML.
8. Phase 5 is serial by design. Do not materialize `claude_code` maintenance packet roots under
   the shared renderer's `agent_id`-derived maintenance path, for example
   `docs/agents/lifecycle/claude_code-maintenance/**`, or rewrite support publication while A, B,
   and C are still moving independently.
9. `claude_code` packet-surface materialization happens only if transport convergence makes
   steady-state committed maintenance surfaces necessary and they do not already exist at that
   `agent_id`-derived maintenance root.
10. `opencode` does not keep a special carveout. Historical proof artifacts stay intact, but its
    steady-state support posture must align with the same ratchet contract as every other agent.
11. No worker lane may run the authoritative live proof flow:
    - shared watcher proof capture
    - shared opener packet materialization for the final proof packet
    - `execute-agent-maintenance --dry-run`
    - `execute-agent-maintenance --write`
    - `close-agent-maintenance`
    - final `make preflight`
12. If the contract freeze changes after A, B, or C starts, the parent stops parallel work, records
    the new freeze commit, and re-bases or reruns all affected lanes from that new baseline.
13. Any proof failure that exposes a missing invariant must add or tighten regression coverage
    before the proof is rerun.
14. Do not revert unrelated user or teammate changes. Integrate around the current repo state and
    keep lane diffs bounded to their declared surfaces.
15. The parent owns the final merged diff review and must reject any lane that spills into TUI
    parity, unrelated runtime work, or ad hoc publication policy.

## Run-State / Checkpoint Model

The session is controlled through explicit checkpoints. If a later step invalidates an earlier
checkpoint, execution returns to the earliest invalidated checkpoint and reruns forward from there.

### Checkpoint C0 - Baseline Captured

State frozen:

- current `staging` head
- current `PLAN.md`
- current stale `ORCH_PLAN.md` replacement intent
- current status of:
  - `docs/specs/**`
  - `docs/agents/lifecycle/codex-maintenance/**`
  - `docs/agents/lifecycle/opencode-maintenance/**`
  - `docs/agents/lifecycle/claude-code-cli-onboarding/**`
  - `.github/workflows/*.yml`
  - `crates/xtask/src/agent_maintenance/**`
  - `crates/xtask/src/agent_registry/**`

Invalidated by:

- unexpected branch movement before the Phase 1 freeze is recorded
- unrelated repo churn that changes owned surfaces before C1

Rerun implication:

- repeat baseline capture before claiming any later checkpoint

### Checkpoint C1 - Contract Freeze Commit Recorded

State frozen:

- exact Phase 1 baseline commit SHA
- exact support-audit schema
- exact debt inventory row shape
- exact blocker taxonomy
- explicit ownership map across specs and explanatory docs

Invalidated by:

- any post-freeze rename to support-audit fields
- any blocker taxonomy drift
- any debt-row schema drift
- any new live policy inserted into workflow YAML or packet prose

Rerun implication:

- redo the Phase 1 freeze, capture the new SHA, and rebase or restart A, B, and C from that commit

### Checkpoint C2 - Parallel Lanes Launched From One Baseline

State frozen:

- three lane branches created from the exact C1 SHA
- lane ownership boundaries acknowledged
- lane-local test targets agreed

Invalidated by:

- a lane branching from the wrong commit
- scope creep that crosses lane ownership without parent approval

Rerun implication:

- respawn the affected lane from C1 and discard mismatched branch work

### Checkpoint C3 - Lanes A, B, And C Merged

State frozen:

- packet derivation plus packet docs lane merged
- relay enforcement lane merged
- transport convergence lane merged
- merged `staging` head carries one stable steady-state implementation baseline

Invalidated by:

- merge conflict resolution that changes frozen policy without returning to C1
- post-merge discovery that any lane skipped its required acceptance

Rerun implication:

- repair the affected lane, rerun its tests, and re-merge before Phase 5 starts

### Checkpoint C4 - Integration Docs And Publication Refreshed

State frozen:

- committed lifecycle maintenance docs aligned to merged code
- support publication aligned to debt truth
- any required `claude_code` maintenance packet root materialized

Invalidated by:

- lifecycle docs still describing worker transport as steady state
- support publication caveats lacking debt rows or concrete blockers
- generated packet surfaces still encoding pre-freeze semantics

Rerun implication:

- rerun the integration doc refresh pass before starting Phase 6

### Checkpoint C5 - Regression Baseline Green

State frozen:

- shared tests for prepare, execute, registry/watcher, and CI wiring are green
- lane-local invariants are covered
- proof can start without known untested contract gaps

Invalidated by:

- any code or doc fix touching covered behavior after tests pass
- any proof failure revealing a missing invariant

Rerun implication:

- add or update the test first, rerun the targeted suites, then rerun forward

### Checkpoint C6 - Migrated `codex` Proof Green

State frozen:

- shared watcher resolves `codex` to shared opener
- shared opener or equivalent packet materialization yields a fresh migrated `codex` packet
- one active `run_id`
- successful dry-run and write on that exact packet
- successful manual closeout
- committed proof artifacts under the canonical `codex` proof root

Invalidated by:

- request truth drift after packet generation
- dry-run or write mismatch against proof claims
- proof artifacts mixing evidence from different runs

Rerun implication:

- preserve failed evidence separately, fix the cause, then rerun from the earliest invalidated
  proof-prep checkpoint

### Checkpoint C7 - Final Head Green

State frozen:

- final proof-bearing head remains bounded and reviewable
- final repo verification is green
- no open conflict remains between contract, code, docs, publication truth, and proof artifacts

Invalidated by:

- any final gate failure
- any bounded-scope violation discovered in final diff review

Rerun implication:

- fix only the invalidated surface, rerun affected gates, and restore final head green

## Worktree And Branch Strategy

### Primary Worktree

The parent stays in the primary repository worktree at:

`/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api`

The parent owns:

- baseline capture
- Phase 1 contract freeze
- recording the freeze SHA
- lane launch and merge sequencing
- Phase 5 integration doc refresh
- all proof preparation and proof execution
- final verification and closeout

### Parallel Worktree Root

All optional lane worktrees live under an uncommitted local root:

`/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/wt/orch-maintenance-convergence/`

This root is scratch only. Nothing under `wt/` is committed.

### Lane Branch Names

Branch names are fixed up front so another agent can execute without guessing:

- Phase 1 baseline note only, no long-lived branch:
  - record the exact freeze SHA from `staging`
- Lane A:
  - worktree path: `wt/orch-maintenance-convergence/lane-a-packet-derivation`
  - branch: `codex/orch-lane-a-packet-derivation`
- Lane B:
  - worktree path: `wt/orch-maintenance-convergence/lane-b-relay-enforcement`
  - branch: `codex/orch-lane-b-relay-enforcement`
- Lane C:
  - worktree path: `wt/orch-maintenance-convergence/lane-c-transport-convergence`
  - branch: `codex/orch-lane-c-transport-convergence`
- Optional bounded repair:
  - worktree path: `wt/orch-maintenance-convergence/fix-<scope>`
  - branch: `codex/orch-fix-<scope>`

### Integration Rules

1. Every lane branches from the exact recorded C1 SHA, not from a moving `staging`.
2. The parent merges or cherry-picks lane work back into `staging` one lane at a time after lane
   acceptance is satisfied.
3. No lane may merge directly into another lane worktree. Cross-lane integration happens only in
   the parent worktree.
4. Phase 5 and later run only on merged `staging`, never in a parallel lane worktree.
5. If the parent needs a local merge rehearsal before integrating into `staging`, it may create a
   disposable local branch, but `staging` remains the authoritative session branch.

## Ownership Model

## Parent

Role: sole orchestrator, sole integrator, sole final gate owner.

Parent-only responsibilities:

- declare checkpoint passage
- own the contract freeze and baseline SHA
- approve lane scope and launch
- merge lane work into `staging`
- decide whether `claude_code` maintenance roots must be materialized in Phase 5
- run shared watcher or opener proof capture for the final `codex` packet
- run authoritative dry-run, write, closeout, and final verification commands
- promote final proof artifacts
- declare the session complete or failed

## Optional Worker Lanes

Preferred worker configuration:

- model: `GPT-5.4`
- `reasoning_effort=high`

Allowed worker roles:

- bounded implementation on one declared lane
- read-only audit of one declared surface
- bounded repair after a surfaced failure

Prohibited worker roles:

- contract-freeze authority
- final proof authority
- lane merge authority
- final publication truth authority
- final gate authority

## Workstream Plan

## WS0 - Baseline Capture And Scope Lock

- ID: `WS0`
- Owner: `parent`
- Launch gate: none
- Owned surfaces:
  - `PLAN.md`
  - `ORCH_PLAN.md`
  - current `staging` branch state
  - existing lifecycle, spec, workflow, and xtask surfaces listed in `C0`
- Actions:
  1. Capture current `staging` head and current dirty-state context.
  2. Verify the old orchestration plan is stale relative to the current `PLAN.md`.
  3. Identify the exact serial-to-parallel boundary: end of Phase 1 contract freeze.
- Stop conditions:
  - ambiguity about the milestone or branch context
  - ambiguity about whether Phase 1 is still open
- Acceptance:
  - parent can name the serial critical path, parallel lanes, and final proof target
  - `C0` passes

## WS1 - Phase 1 Contract Freeze

- ID: `WS1`
- Owner: `parent`
- Launch gate: `C0`
- Owned surfaces:
  - `docs/specs/maintenance-request-contract-v1.md`
  - `docs/specs/agent-registry-contract.md`
  - `docs/specs/cli-agent-onboarding-charter.md`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/non-tui-support-debt.md`
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
  - `docs/cli-agent-onboarding-factory-workflow-atlas.md`
  - `docs/cli-agent-maintenance-steady-state-plan.md`
- Actions:
  1. Freeze one normative success definition: support audit first, bounded support uplift second,
     green gates third, manual closeout last.
  2. Make the exact `support_surface_audit` schema normative, including:
     - field names
     - enum values
     - eligibility reasons
     - allowed deferrals
     - debt-count invariants
  3. Create or align `non-tui-support-debt.md` as the machine-checkable baseline inventory.
  4. Demote `docs/cli-agent-maintenance-steady-state-plan.md` to archived-pointer status.
  5. Align the operator guide and atlas to explanatory-only status with no shadow policy.
  6. Record the exact resulting `staging` commit SHA as the only valid baseline for A, B, and C.
- Stop conditions:
  - unresolved disagreement across the normative specs
  - debt inventory row shape still ambiguous
  - packet success semantics still split between specs and explanatory docs
- Acceptance:
  - one frozen contract story exists
  - one exact baseline SHA is recorded
  - `C1` passes

## WS2 - Lane A Packet Derivation Plus Packet-Owned Docs

- ID: `WS2`
- Owner: `worker lane A` or `parent`
- Launch gate: `C1`
- Branch: `codex/orch-lane-a-packet-derivation`
- Owned surfaces:
  - `crates/xtask/src/agent_maintenance/contract_policy.rs`
  - `crates/xtask/src/agent_maintenance/prepare.rs`
  - `crates/xtask/src/agent_maintenance/docs.rs`
  - `crates/xtask/src/agent_maintenance/request.rs`
  - `crates/xtask/src/agent_maintenance/request/automation.rs`
  - `crates/xtask/tests/agent_maintenance_prepare.rs`
  - committed `codex` and `opencode` maintenance packet surfaces generated by the shared renderer
- Actions:
  1. Add one shared typed support-audit representation in the prepare/request layer.
  2. Derive the audit block from upstream evidence, wrapper coverage, backend truth, support
     publication truth, and the debt inventory.
  3. Rewrite prompt, `HANDOFF.md`, and `pr-summary.md` semantics around support uplift.
  4. Keep lane A out of relay validation and watcher transport logic.
- Stop conditions:
  - lane A needs to rename the frozen support-audit schema
  - lane A starts inventing relay-only policy or transport logic
  - generated docs still describe artifact-refresh-only success
- Acceptance:
  - prepare-layer schema and generated docs match the Phase 1 freeze exactly
  - focused prepare tests are green
  - lane returns a bounded diff for merge

## WS3 - Lane B Relay Enforcement

- ID: `WS3`
- Owner: `worker lane B` or `parent`
- Launch gate: `C1`
- Branch: `codex/orch-lane-b-relay-enforcement`
- Owned surfaces:
  - `crates/xtask/src/agent_maintenance/execute.rs`
  - `crates/xtask/src/agent_maintenance/execute/runtime.rs`
  - `crates/xtask/src/agent_maintenance/execute/validate.rs`
  - `crates/xtask/src/agent_maintenance/execute/workflow.rs`
  - `crates/xtask/src/agent_maintenance/execute/packet.rs`
  - `crates/xtask/src/agent_maintenance/execute/types.rs`
  - `crates/xtask/tests/agent_maintenance_execute.rs`
- Actions:
  1. Enforce required presence and continuity of the support-audit block before write mode.
  2. Fail closed on:
     - missing newly discovered uplift rows
     - invalid eligibility on preexisting gaps
     - invalid blocker taxonomy
     - repo-owned deferrals with no tracked follow-on seam, owner, and milestone
     - debt-count regression
  3. Keep closeout manual and keep the writable envelope bounded to non-TUI surfaces only.
  4. Keep lane B out of packet rendering semantics except where needed to validate frozen fields.
- Stop conditions:
  - lane B attempts to rename or redefine the frozen schema
  - lane B widens write scope into TUI or unrelated runtime work
  - lane B relies on agent-specific hidden policy
- Acceptance:
  - relay invariants are shared and fail closed
  - focused execute tests are green
  - lane returns a bounded diff for merge

## WS4 - Lane C Transport Convergence

- ID: `WS4`
- Owner: `worker lane C` or `parent`
- Launch gate: `C1`
- Branch: `codex/orch-lane-c-transport-convergence`
- Owned surfaces:
  - `crates/xtask/data/agent_registry.toml`
  - `crates/xtask/src/agent_registry.rs`
  - `crates/xtask/src/agent_registry/release_watch.rs`
  - `crates/xtask/src/agent_maintenance/watch.rs`
  - `.github/workflows/agent-maintenance-release-watch.yml`
  - `.github/workflows/agent-maintenance-open-pr.yml`
  - `.github/workflows/codex-cli-update-snapshot.yml`
  - `.github/workflows/claude-code-update-snapshot.yml`
  - `crates/xtask/tests/agent_maintenance_watch.rs`
  - `crates/xtask/tests/agent_registry.rs`
  - `crates/xtask/tests/c4_spec_ci_wiring.rs`
- Actions:
  1. Migrate `codex` and `claude_code` registry truth to `dispatch_kind = "packet_pr"`.
  2. Make the shared watcher fan out to the shared opener for all enrolled automated lanes.
  3. Retire worker-specific steady-state transport:
     - default: delete the worker snapshot workflows
     - fallback: keep them unscheduled and clearly historical/manual-only if deletion is blocked
  4. Keep lane C out of support-policy prose and relay semantics.
- Stop conditions:
  - workflow YAML becomes a second policy store
  - `codex` or `claude_code` still resolve to worker transport after migration
  - lane C starts materializing lifecycle docs that belong to Phase 5
- Acceptance:
  - registry, watcher, and workflow tests are green
  - no live scheduled or registry-driven path still depends on worker-specific transport
  - lane returns a bounded diff for merge

## WS5 - Parent Merge Of A, B, And C

- ID: `WS5`
- Owner: `parent`
- Launch gate: `WS2`, `WS3`, and `WS4` accepted individually
- Owned surfaces:
  - merged `staging` head only
- Actions:
  1. Merge or cherry-pick lane A into `staging`.
  2. Merge or cherry-pick lane B into `staging`.
  3. Merge or cherry-pick lane C into `staging`.
  4. Resolve conflicts without rewriting frozen policy.
  5. Run the lane-level targeted tests again on merged `staging` if any conflict touched shared
     files.
- Stop conditions:
  - conflict resolution changes frozen schema or blocker semantics
  - merged head no longer cleanly reflects lane ownership boundaries
- Acceptance:
  - one merged steady-state code baseline exists on `staging`
  - `C3` passes

## WS6 - Phase 5 Integration Doc Refresh

- ID: `WS6`
- Owner: `parent`
- Launch gate: `C3`
- Owned surfaces:
  - `docs/agents/lifecycle/codex-maintenance/**`
  - `docs/agents/lifecycle/opencode-maintenance/**`
  - `docs/agents/lifecycle/claude-code-cli-onboarding/**`
  - if needed, new `docs/agents/lifecycle/claude_code-maintenance/**`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/non-tui-support-debt.md`
  - `cli_manifests/support_matrix/current.json`
- Actions:
  1. Regenerate or refresh committed `codex` and `opencode` maintenance packet surfaces from the
     merged shared renderer.
  2. Materialize `claude_code` committed maintenance packet surfaces if transport convergence now
     makes them a steady-state requirement and they do not already exist at the shared renderer's
     `agent_id`-derived maintenance root, for example
     `docs/agents/lifecycle/claude_code-maintenance/**`.
  3. Update support publication so every remaining non-TUI caveat points to a debt row or concrete
     blocker instead of normalizing deliberate unsupported posture.
  4. Remove worker-transport worldview from packet-owned playbooks and workflow plans.
  5. Preserve already-landed `opencode` proof artifacts as historical evidence while removing stale
     explanatory carveouts around them.
- Stop conditions:
  - `claude_code` packet materialization path is still ambiguous after reviewing the merged code
  - support publication still contradicts debt inventory truth
  - lifecycle docs still describe worker flows as steady state
- Acceptance:
  - lifecycle docs and support publication match the merged code and frozen contract
  - any required `claude_code` maintenance root is present
  - `C4` passes

## WS7 - Regression Consolidation And Pre-Proof Green

- ID: `WS7`
- Owner: `parent`
- Launch gate: `C4`
- Owned surfaces:
  - `crates/xtask/tests/agent_maintenance_prepare.rs`
  - `crates/xtask/tests/agent_maintenance_execute.rs`
  - `crates/xtask/tests/agent_maintenance_watch.rs`
  - `crates/xtask/tests/agent_registry.rs`
  - `crates/xtask/tests/c4_spec_ci_wiring.rs`
  - any newly required shared regression fixture updates
- Actions:
  1. Land any remaining regression additions required by the merged steady state.
  2. Run targeted xtask suites until green.
  3. Run broader `cargo test -p xtask` if targeted suites expose cross-surface drift.
  4. Do not start the live `codex` proof until the merged regression baseline is trustworthy.
- Stop conditions:
  - missing invariant coverage remains
  - merged behavior still depends on manual reviewer interpretation
- Acceptance:
  - targeted regressions for prepare, execute, transport, and spec wiring are green
  - shared proof lane can start without known test debt
  - `C5` passes

## WS8 - Migrated `codex` Proof

- ID: `WS8`
- Owner: `parent`
- Launch gate: `C5`
- Owned surfaces:
  - `docs/agents/lifecycle/codex-maintenance/**`
  - `docs/agents/lifecycle/codex-maintenance/governance/proof/**`
  - temp run state under `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`
  - exact `codex` request-owned writable surfaces declared by the migrated packet
- Actions:
  1. Capture shared watcher evidence proving `codex` now resolves to shared `packet_pr`.
  2. Materialize the actual proof packet through the shared opener path, not by hand-authoring a
     fake request. Use the shared `prepare-agent-maintenance` shape with:
     - `--agent codex`
     - `--opened-from .github/workflows/agent-maintenance-open-pr.yml`
     - `--detected-by .github/workflows/agent-maintenance-release-watch.yml`
     - `--dispatch-kind packet_pr`
     - `--write`
     - release/version arguments taken from the actual watcher-detected release being proved
  3. Freeze the resulting request SHA and run:
     ```sh
     cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --dry-run
     ```
  4. Reuse that exact `run_id` for:
     ```sh
     cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --write --run-id <run_id>
     ```
  5. Author truthful manual closeout and run:
     ```sh
     cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json
     ```
  6. Promote only the final successful structured evidence into the canonical `codex` proof root.
- Stop conditions:
  - packet truth still points to worker transport
  - dry-run or write fails
  - write escapes the packet-declared writable envelope
  - proof artifacts mix evidence from different proof attempts
  - closeout is written automatically by write mode or is otherwise untruthful
- Acceptance:
  - one migrated `codex` packet proves shared watcher -> shared opener -> shared relay -> manual
    closeout end to end
  - proof artifacts are complete and replayable
  - `C6` passes

## WS9 - Final Verification And Session Close

- ID: `WS9`
- Owner: `parent`
- Launch gate: `C6`
- Owned surfaces:
  - final merged `staging` head
  - final proof-bearing surfaces
- Actions:
  1. Run final repository gates.
  2. Review the final diff for bounded scope and absence of shadow policy.
  3. Confirm the final story is consistent across:
     - specs
     - packet rendering
     - relay validation
     - watcher/open-pr transport
     - lifecycle docs
     - support publication
     - proof artifacts
- Stop conditions:
  - final repo gate failure
  - proof-bearing head no longer reads as one bounded maintenance convergence session
- Acceptance:
  - final repo gates are green
  - final diff is bounded and reviewable
  - `C7` passes

## WS10 - Conditional Repair Lane

- ID: `WS10`
- Owner: `bounded worker lane` for patching, `parent` for integration and rerun
- Launch gate: failure in `WS2` through `WS9`
- Owned surfaces:
  - only the specific failing scope assigned by the parent
- Actions:
  1. Fix one bounded failure only.
  2. Run only the narrow local tests approved by the parent.
  3. Return a concise summary that states whether C1, C3, C4, or C5 may have been invalidated.
- Stop conditions:
  - repair scope expands beyond the assigned issue
  - worker tries to run the authoritative live proof path
  - repair quietly rewrites frozen policy
- Acceptance:
  - parent receives a bounded diff and integrates it into `staging`
  - reruns restart from the earliest invalidated checkpoint

## Context-Control Rules

1. The parent retains:
   - the exact C1 freeze SHA
   - lane ownership boundaries
   - checkpoint state
   - final proof inputs and outputs
   - final acceptance truth
2. Each worker lane receives only:
   - the exact baseline SHA
   - its declared branch name
   - its allowed file surfaces
   - its target tests
   - its stop conditions
3. Workers must not receive "fix anything necessary" authority.
4. Workers must not decide that Phase 1 truth is wrong. They may only report a conflict back to the
   parent.
5. Workers must not author or promote final proof artifacts.
6. Workers must not run the final live watcher/open-pr/proof path.
7. Worker outputs must include:
   - touched files
   - commands run
   - pass/fail status of those commands
   - unresolved risks or assumptions
   - explicit statement if the work appears to invalidate C1, C3, C4, or C5
8. The parent verifies worker claims locally before merging.
9. The parent promotes only the final successful proof evidence. Failed or superseded proof runs
   stay outside the canonical proof root.
10. If a worker discovers that `claude_code` packet-surface materialization needs a different path
    than the shared renderer's `agent_id`-derived maintenance root, the worker reports it and
    stops. The parent resolves that naming/path question in Phase 5.

## Failure Handling And Rerun Policy

### If Phase 1 Freeze Changes After Lanes Start

- stop all downstream lane work
- record the new freeze commit
- rebase or respawn A, B, and C from that exact commit
- do not hand-wave the drift as a small rename

### If A, B, Or C Fails Its Lane Acceptance

- keep the lane bounded
- repair only the failing surface
- rerun the lane-local targeted tests
- merge only after the lane returns to green

### If Phase 5 Exposes Missing `claude_code` Maintenance Surfaces

- materialize them in Phase 5 from merged shared renderer truth
- do not retroactively reopen lane C unless transport behavior itself was wrong

### If Regression Coverage Is Missing After Merge

- add or update the missing shared tests before starting the live proof
- keep test additions in merged `staging`, not back in lane worktrees unless the parent decides
  otherwise

### If Migrated `codex` Proof Fails

- preserve failed evidence outside the final proof root
- classify the failure:
  - packet generation drift
  - relay validation gap
  - writable-surface escape
  - transport mismatch
  - publication mismatch
  - closeout truth gap
  - missing regression invariant
- add or tighten regression coverage first if the failure revealed a missing invariant
- rerun from the earliest invalidated checkpoint only

### If Final Gates Fail

- fix only the failing surface
- rerun the affected gates
- keep the proof root truthful to the final successful run only

## Tests And Acceptance

## Contract Freeze

Required checks:

```sh
rg -n 'support_surface_audit|non-tui-support-debt|packet_pr|workflow_dispatch|deliberately unsupported' docs/specs docs/cli-agent-*
```

Acceptance targets:

- the three normative specs use identical support-audit field names and blocker semantics
- `docs/cli-agent-maintenance-steady-state-plan.md` no longer acts as live policy
- the debt inventory covers all currently published non-TUI caveats

## Lane A

Required checks:

```sh
cargo test -p xtask agent_maintenance_prepare -- --nocapture
```

Acceptance targets:

- generated packet/request/prompt docs serialize the frozen support-audit schema exactly
- `HANDOFF.md`, `pr-summary.md`, and prompt text describe support-aware bounded uplift
- no per-agent renderer logic invents extra schema fields

## Lane B

Required checks:

```sh
cargo test -p xtask agent_maintenance_execute -- --nocapture
```

Acceptance targets:

- packets missing support-audit truth fail closed
- invalid blockers such as `deliberately_unsupported` and `not_part_of_v1` fail validation
- debt-count ratchet and repo-owned deferral rules are enforced

## Lane C

Required checks:

```sh
cargo test -p xtask agent_maintenance_watch -- --nocapture
cargo test -p xtask agent_registry -- --nocapture
cargo test -p xtask c4_spec_ci_wiring -- --nocapture
```

Acceptance targets:

- `codex` resolves to shared `packet_pr`
- `claude_code` resolves to shared `packet_pr`
- no active registry-driven path still points at worker-specific transport

## Integration Doc Refresh

Required checks:

```sh
cargo run -p xtask -- support-matrix --check
```

Acceptance targets:

- lifecycle maintenance docs match the merged shared renderer and merged transport reality
- support publication no longer normalizes deliberate unsupported non-TUI posture
- every remaining non-TUI caveat points to debt truth or a concrete blocker

## Regression Baseline

Required checks:

```sh
cargo test -p xtask agent_maintenance_prepare -- --nocapture
cargo test -p xtask agent_maintenance_execute -- --nocapture
cargo test -p xtask agent_maintenance_watch -- --nocapture
cargo test -p xtask agent_registry -- --nocapture
cargo test -p xtask c4_spec_ci_wiring -- --nocapture
```

Fallback:

```sh
cargo test -p xtask
```

Acceptance targets:

- all merged shared-lane invariants are covered and green
- no known proof-critical gap remains untested

## Migrated `codex` Proof

Required commands:

```sh
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --dry-run
cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --write --run-id <run_id>
cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json
```

Additional required gate:

```sh
cargo run -p xtask -- codex-validate --root cli_manifests/codex
```

Acceptance targets:

- the `codex` request packet was materialized from shared opener semantics, not left on worker
  transport truth
- one frozen `run_id` is used for dry-run and write
- write stays inside exact packet-declared writable surfaces
- manual closeout remains explicit
- the canonical proof root contains only final successful structured evidence

## Final Repo Gates

Required commands:

```sh
cargo fmt --all
cargo run -p xtask -- codex-validate --root cli_manifests/codex
cargo run -p xtask -- support-matrix --check
cargo run -p xtask -- capability-matrix --check
cargo run -p xtask -- capability-matrix-audit
make preflight
```

Acceptance targets:

- all final commands pass
- final diff remains bounded to the maintenance convergence lane
- final proof-bearing head is the truthful final session head

## Assumptions

1. The working branch remains `staging` for the entire orchestration session.
2. The parent can create local scratch worktrees under `wt/` without committing any `wt/`
   contents.
3. `codex` remains the required first migrated live proof lane.
4. `claude_code` must reach transport and contract parity in code and tests during this milestone,
   even if its first live proof is deferred.
5. The current `codex` maintenance packet surfaces are transitional and will be regenerated to the
   shared packet-pr contract before the proof runs.
6. `docs/agents/lifecycle/claude-code-cli-onboarding/**` is not yet the steady-state maintenance
   packet root. If a new maintenance root is required, Phase 5 materializes it at the shared
   renderer's `agent_id`-derived maintenance path, for example
   `docs/agents/lifecycle/claude_code-maintenance/**`, and makes that root canonical there.
7. Existing `opencode` proof artifacts remain historical evidence and should not be rewritten
   except where surrounding explanatory docs need to stop telling a stale steady-state story.
8. Shared watcher/open-pr transport and the local `execute-agent-maintenance` plus
   `close-agent-maintenance` relay flow remain the intended steady-state architecture.

## Session Close Criteria

The parent may close the session only after:

1. `C0` through `C7` pass in order.
2. The C1 freeze SHA is recorded and every lane is traceable back to it.
3. A, B, and C are merged before any integration doc refresh starts.
4. Phase 5 completes before any live proof begins.
5. The migrated `codex` proof artifacts are committed under the canonical proof root.
6. Final repository verification is green on the proof-bearing `staging` head.

Until then, the maintenance convergence milestone is not complete.
