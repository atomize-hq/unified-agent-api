<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/restores/codex-recommend-next-agent-autoplan-restore-20260503-104405.md -->
# PLAN - Make The Published State Honest In The Lifecycle Model

Status: planned  
Date: 2026-05-03  
Branch: `codex/recommend-next-agent`  
Base branch: `main`  
Repo: `atomize-hq/unified-agent-api`  
Work item: `Make The Published State Honest In The Lifecycle Model`  
Plan commit baseline: `07a0ce9`

Separate design doc: not required for this slice. This is a backend-only lifecycle and
control-plane correction. `PLAN.md` is the canonical design and execution record.

## Objective

Make `published` a real committed lifecycle stage with one writer, one meaning, and one
normal downstream consumer.

After this plan lands:

1. the canonical create-mode path becomes
   `approved -> enrolled -> runtime_integrated -> publication_ready -> published -> closed_baseline`
2. `refresh-publication --write` becomes the only command that can commit
   `LifecycleStage::Published`
3. `publication_ready` goes back to meaning exactly one thing: the frozen handoff packet
   exists and refresh is next
4. `close-proving-run` consumes `published` on the normal path and treats
   `publication_ready` as compatibility-only
5. specs, operator docs, lifecycle validation, seeded fixtures, and maintenance logic all
   tell the same story

The non-negotiable outcome is simple:

```json
{ "lifecycle_stage": "published" }
```

That must be true immediately after a successful `refresh-publication --write`.

## Source Inputs

- Backlog source:
  - `TODOS.md`
  - `docs/backlog/cli-agent-onboarding-lifecycle-unification-gap-memo.md`
- Normative contracts:
  - `docs/specs/cli-agent-onboarding-charter.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/support-matrix.md`
- Procedure source:
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
- Implementation surfaces:
  - `crates/xtask/src/agent_lifecycle.rs`
  - `crates/xtask/src/agent_lifecycle/validation.rs`
  - `crates/xtask/src/prepare_publication.rs`
  - `crates/xtask/src/publication_refresh.rs`
  - `crates/xtask/src/close_proving_run.rs`
  - `crates/xtask/src/capability_publication.rs`
  - `crates/xtask/src/agent_maintenance/drift/governance.rs`
- Current tests and fixtures:
  - `crates/xtask/tests/agent_lifecycle_state.rs`
  - `crates/xtask/tests/refresh_publication_entrypoint.rs`
  - `crates/xtask/tests/prepare_publication_entrypoint.rs`
  - `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`
  - `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_paths.rs`
  - `crates/xtask/tests/support/agent_maintenance_drift_harness.rs`
  - `docs/agents/lifecycle/**`

## Verified Current State

These facts were re-verified from the current branch before this rewrite.

1. `LifecycleStage::Published` already exists in `crates/xtask/src/agent_lifecycle.rs`.
2. `required_evidence_for_stage(Published)` already maps to `PUBLISHED_MINIMUM_EVIDENCE`.
3. `validate_stage_support_tier(...)` already requires `publication_backed` or
   `first_class` for `published` and `closed_baseline`.
4. `publication_refresh::build_next_lifecycle_state(...)` currently updates transition
   metadata but does not set `lifecycle_stage = published`, does not promote
   `support_tier`, and does not record published continuity fields.
5. `prepare-publication --write` currently owns the transition into
   `publication_ready` and points the next command at `refresh-publication --write`.
6. `close-proving-run` currently accepts `publication_ready` or legacy/manual
   `published`, which leaves two possible interpretations of "post-publication."
7. `capability_publication` and maintenance governance already treat `published` as a
   valid post-publication state.
8. `publication-ready.json` is intentionally pinned to `lifecycle_stage = publication_ready`
   and `support_tier_at_emit = baseline_runtime`. It is a pre-refresh handoff packet, not a
   durable post-refresh state snapshot.
9. The live `aider-onboarding` fixture is still at `publication_ready` with
   `expected_next_command = refresh-publication --write`, so the repo still contains the
   honest pre-refresh seam.
10. The operator guide and charter already describe refresh as the sole publication
    consumer, but closeout wording still leaves `publication_ready` in the normal input path.

## Problem Statement

The schema says `published` exists. The live write path does not make it real.

Current shape:

```text
runtime-follow-on --write
  -> prepare-publication --write
       writes publication-ready.json
       writes lifecycle_stage = publication_ready
  -> refresh-publication --write
       writes publication outputs
       runs green publication gate
       keeps lifecycle_stage = publication_ready
  -> close-proving-run --write
       writes lifecycle_stage = closed_baseline
```

That creates four real problems:

1. the lifecycle machine and the actual write path disagree
2. `publication_ready` is overloaded and means different things depending on hidden context
3. `published` appears in validators and evidence contracts but has no committed producer
4. future automation cannot answer "has publication completed?" from lifecycle truth alone

Target shape:

```text
runtime-follow-on --write
  -> prepare-publication --write
       writes publication-ready.json
       writes lifecycle_stage = publication_ready
       expected_next_command = refresh-publication --write
  -> refresh-publication --write
       writes publication outputs
       runs green publication gate
       writes lifecycle_stage = published
       writes support_tier = publication_backed | first_class
       writes publication_packet_path / publication_packet_sha256
       writes published-stage evidence
       expected_next_command = close-proving-run --write
  -> close-proving-run --write
       consumes published on the normal path
       writes lifecycle_stage = closed_baseline
```

That machine is honest:

- `publication_ready` means refresh has not been committed yet
- `published` means publication-owned outputs are written and green
- `closed_baseline` means create-mode closeout truth is committed

## Step 0 Scope Challenge

### Premise Check

The repo does not need a new lifecycle concept. It already has the right one. It just
fails to commit it.

The real decision is:

1. make `published` real by giving it one writer, or
2. delete it and push more hidden meaning into `publication_ready`

Recommendation: choose option 1. It is the smaller semantic diff, the more explicit
state machine, and it reuses contracts that already exist in code.

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| lifecycle stage enum | `agent_lifecycle::LifecycleStage` | Reuse directly. `Published` already exists. |
| published evidence contract | `PUBLISHED_MINIMUM_EVIDENCE` | Reuse directly. Do not invent a second "publication complete" marker. |
| publication writer and gate | `publication_refresh.rs` | Reuse directly as the sole `published` writer. |
| pre-refresh handoff packet | `publication-ready.json` + `PublicationReadyPacket` | Reuse directly. Keep it as the handoff artifact. |
| closeout consumer | `close_proving_run.rs` | Reuse directly. Tighten it around `published` as the normal input. |
| stage/tier semantics | `validate_stage_support_tier(...)` | Reuse directly. The contract already expects `published`. |
| publication eligibility | `capability_publication.rs` | Reuse directly, then align wording and tests. |
| maintenance baseline semantics | `agent_maintenance/drift/governance.rs` | Reuse directly, but make the post-publication truth path explicit. |

### Alternatives Considered

| Option | Summary | Pros | Cons | Decision |
| --- | --- | --- | --- | --- |
| A | Make `refresh-publication --write` commit `published` | Reuses the existing schema, evidence model, support-tier rules, and command boundaries. | Requires lifecycle, test, and docs updates across the refresh and closeout seam. | **Chosen.** |
| B | Delete `published` and keep green publication hidden inside `publication_ready` | Avoids a new committed transition in the write path. | Makes the machine less explicit, increases validator churn, and preserves ambiguity. | Rejected. |
| C | Rename `published` to something else | Could tune wording. | Pure naming churn, zero control-plane gain, wider migration surface. | Rejected. |

### Minimum Complete Change Set

Anything smaller than this is a shortcut that leaves the lifecycle dishonest:

1. make `refresh-publication --write` write `LifecycleStage::Published`
2. promote `support_tier` during refresh to `publication_backed` unless already
   `first_class`
3. write published-stage required and satisfied evidence during refresh
4. record `publication_packet_path` and `publication_packet_sha256` at refresh time
5. make `close-proving-run` consume `published` as the canonical input stage
6. keep one bounded compatibility branch for pre-migration `publication_ready` states
7. update lifecycle/spec/operator docs and seeded fixtures so the state machine is
   described exactly once
8. add direct regression coverage proving that refresh now commits `published`

### Complexity Check

This slice touches more than 8 files, but that is still the minimum complete version.
The truth boundary spans:

- lifecycle schema and validation
- refresh writer logic
- closeout input-stage validation
- maintenance baseline semantics
- operator/spec docs
- refresh and closeout tests

Complexity controls:

- no new command
- no new artifact type
- no packet rename
- no support/capability publication redesign
- no closeout schema redesign
- no maintenance workflow redesign

### Search / Build Decision

This is mostly a Layer 1 reuse correction with one Layer 3 truth decision.

- **[Layer 1]** Reuse `LifecycleStage::Published`
- **[Layer 1]** Reuse `PUBLISHED_MINIMUM_EVIDENCE`
- **[Layer 1]** Reuse `refresh-publication --write` as the stage owner
- **[Layer 1]** Reuse `publication-ready.json` as the pre-refresh packet
- **[Layer 1]** Reuse `close-proving-run` as the post-publication consumer
- **[Layer 3]** Stop encoding "green publication" as hidden prose and make it a committed stage

### TODOS Cross-Reference

This plan closes:

- `Make The Published State Honest In The Lifecycle Model`

This plan unblocks, but does not implement:

- `Enclose Create-Mode Closeout Without Ad Hoc Authoring`

### Distribution Check

No new binary, package, container, or published artifact type is introduced.

## Locked Decisions

1. The canonical create-mode stage sequence is
   `approved -> enrolled -> runtime_integrated -> publication_ready -> published -> closed_baseline`.
2. `refresh-publication --write` is the only command allowed to write
   `LifecycleStage::Published`.
3. `publication-ready.json` remains the pre-refresh handoff packet. It is not renamed in
   this milestone.
4. `published` is a committed stage, not just a validated condition.
5. `publication_ready` means "packet emitted, refresh not yet committed." It no longer
   doubles as a post-refresh steady state.
6. Successful refresh promotes `support_tier` to `publication_backed` unless the agent is
   already `first_class`.
7. Successful refresh records `publication_packet_path` and `publication_packet_sha256`
   into `lifecycle-state.json`.
8. `close-proving-run` treats `published` as the normal input stage.
9. Compatibility for `publication_ready` is transitional and explicitly gated. It exists
   only to support pre-migration fixtures and in-flight repositories that already ran
   refresh before this lifecycle correction.
10. Maintenance drift may continue to treat `published` and `closed_baseline` as valid
    post-publication baselines. This plan does not narrow that maintenance capability.
11. `closed_baseline` remains the create-mode done state. `published` is truthful
    publication completion, not closeout completion.
12. This milestone does not redesign closeout artifact authoring.

## Architecture Review

### Canonical State Machine

```text
approved
  -> enrolled
  -> runtime_integrated
  -> publication_ready
       owner: prepare-publication --write
       meaning: frozen handoff packet exists, refresh is next
  -> published
       owner: refresh-publication --write
       meaning: publication-owned outputs are written and green
  -> closed_baseline
       owner: close-proving-run --write
       meaning: closeout truth is committed
```

### Stage Contract

| Stage | Written by | Required truth at commit time | Expected next command |
| --- | --- | --- | --- |
| `publication_ready` | `prepare-publication --write` | runtime evidence is pinned, `publication-ready.json` is written, support tier remains `baseline_runtime` | `refresh-publication --approval <path> --write` |
| `published` | `refresh-publication --write` | publication outputs are refreshed, publication gate is green, support tier is `publication_backed` or `first_class`, packet continuity fields are recorded | `close-proving-run --approval <path> --closeout <path>` |
| `closed_baseline` | `close-proving-run --write` | closeout inputs are valid, publication continuity still holds, baseline paths are written | maintenance or no-op follow-on |

### Command Ownership

```text
prepare-publication --write
  writes:
    lifecycle_stage = publication_ready
    publication-ready.json
  does not write:
    support/capability publication outputs

refresh-publication --write
  writes:
    publication-owned support/capability outputs
    lifecycle_stage = published
    support_tier = publication_backed | first_class
    publication_packet_path
    publication_packet_sha256
    required_evidence = required_evidence_for_stage(Published)
    satisfied_evidence = required_evidence_for_stage(Published)
  verifies:
    support-matrix --check
    capability-matrix --check
    capability-matrix-audit
    make preflight

close-proving-run --write
  reads:
    lifecycle_stage = published on the normal path
    publication-ready.json continuity
    green publication truth
  writes:
    lifecycle_stage = closed_baseline
    closeout_baseline_path
```

### Compatibility Rule

This is the ambiguity killer for the slice.

Normal path:

- `prepare-publication --write` ends at `publication_ready`
- `refresh-publication --write` ends at `published`
- `close-proving-run --write` consumes `published`

Compatibility-only path:

- `close-proving-run` may still accept `publication_ready` only when the state clearly
  represents a pre-migration "refresh already happened, lifecycle was never promoted"
  shape
- that means all of the following must be true:
  - `lifecycle_stage == publication_ready`
  - `expected_next_command` already points at `close-proving-run`, not refresh
  - `last_transition_by` reflects refresh ownership, not prepare ownership
  - publication outputs and publication audits are already green
- a plain prepare-time `publication_ready` state with
  `expected_next_command = refresh-publication --write` is never closable

This keeps old fixtures working without leaving two steady-state meanings for
`publication_ready`.

### Module / Responsibility Map

| Module | Responsibility in this slice |
| --- | --- |
| `crates/xtask/src/publication_refresh.rs` | Commit `published`, promote support tier, write continuity fields, keep rollback honest |
| `crates/xtask/src/agent_lifecycle.rs` | Stage contract, evidence minima, helper commands, packet continuity fields |
| `crates/xtask/src/agent_lifecycle/validation.rs` | Stage-specific validation and lifecycle invariants |
| `crates/xtask/src/close_proving_run.rs` | Canonical `published` input path and bounded `publication_ready` compatibility |
| `crates/xtask/src/agent_maintenance/drift/governance.rs` | Preserve explicit post-publication maintenance semantics |
| `crates/xtask/src/capability_publication.rs` | Align comments, selectors, and expectations to the real post-refresh stage |
| `crates/xtask/tests/**` | Lock the stage machine with direct regression coverage |
| `docs/specs/**` and `docs/cli-agent-onboarding-factory-operator-guide.md` | Restate one canonical lifecycle story |

### Failure-Aware Mutation Ordering

Refresh must not commit `published` until the whole publication seam is green.

Required ordering:

1. validate `publication_ready` seam and packet continuity
2. plan output mutations
3. write publication-owned outputs
4. run the publication gate
5. build and persist the next lifecycle state as `published`
6. persist any packet continuity updates tied to the new lifecycle snapshot
7. report success

Rollback rule:

- if any step before lifecycle persistence fails, nothing may leave a fake `published`
  state behind
- if lifecycle or packet persistence fails after output writes, rollback must restore the
  pre-refresh bytes for publication-owned outputs and lifecycle-owned files

### Dependency Graph

```text
prepare_publication.rs
  -> agent_lifecycle.rs
  -> publication-ready.json

publication_refresh.rs
  -> agent_lifecycle.rs
  -> publication-ready.json
  -> support_matrix.rs
  -> capability_matrix.rs
  -> capability_publication.rs
  -> workspace_mutation.rs

close_proving_run.rs
  -> agent_lifecycle.rs
  -> publication-ready.json
  -> agent_maintenance::drift
  -> capability_publication.rs
```

## Code Quality Review

The repo already has the right abstractions. This is an ownership correction, not a
"build a new model" job.

Implementation rules:

1. do not add a second "publication complete" helper type
2. do not add a second post-refresh packet format
3. keep stage transition logic localized to:
   - `prepare_publication.rs`
   - `publication_refresh.rs`
   - `close_proving_run.rs`
4. keep stage-specific invariants centralized in:
   - `agent_lifecycle.rs`
   - `agent_lifecycle/validation.rs`
5. do not infer publication completion from `expected_next_command`
6. do not spread compatibility logic across multiple modules

ASCII diagram maintenance:

- if nearby lifecycle comments exist in `agent_lifecycle.rs`, `publication_refresh.rs`, or
  `close_proving_run.rs`, update them in the same change
- `PLAN.md` remains the canonical cross-file state-machine diagram for this slice

## Test Review

This slice is not done until the changed lifecycle seam is directly provable in tests.
The regression is the whole point.

### Code Path Coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/xtask/src/publication_refresh.rs
    │
    ├── run_in_workspace(--check)
    │   ├── validates publication_ready seam
    │   └── preserves pre-refresh semantics
    │
    └── run_in_workspace(--write)
        ├── validates publication_ready seam
        ├── writes publication outputs
        ├── runs green gate
        ├── [GAP] writes lifecycle_stage = published
        ├── [GAP] promotes support_tier
        ├── [GAP] writes publication_packet_path / publication_packet_sha256
        ├── [GAP] writes published-stage evidence
        └── [GAP] rolls back cleanly if failure occurs before commit completion

[+] crates/xtask/src/close_proving_run.rs
    │
    ├── validate_closeout_inputs(...)
    │   ├── [GAP] treats published as canonical normal path
    │   ├── [GAP] accepts compatibility publication_ready only when explicitly eligible
    │   └── [GAP] rejects ordinary prepare-time publication_ready
    │
    └── write_closed_baseline(...)
        ├── consumes publication continuity fields
        └── preserves first_class when already set

[+] crates/xtask/src/agent_lifecycle.rs
    │
    ├── required_evidence_for_stage(Published)
    ├── validate_stage_support_tier(Published)
    └── [GAP] direct fixture coverage that published is now a reachable committed state
```

### Operator Flow Coverage

```text
USER / OPERATOR FLOW COVERAGE
=============================
[+] Create-mode publication flow
    │
    ├── [★★ TESTED] prepare-publication -> publication_ready
    ├── [GAP] refresh-publication -> published
    ├── [GAP] published -> close-proving-run -> closed_baseline
    └── [GAP] refresh failure rolls back without leaving fake published state

[+] Compatibility flow
    │
    ├── [GAP] legacy post-refresh publication_ready fixture closes only through the explicit compatibility branch
    └── [GAP] ordinary prepare-time publication_ready remains non-closable

[+] Maintenance and publication truth
    │
    ├── [GAP] published is accepted as a post-publication maintenance baseline
    └── [GAP] pre-refresh publication_ready is not treated as a maintenance baseline

─────────────────────────────────
COVERAGE TARGET: all changed paths
  Critical path tests to add/update: 8
  CLI seam tests: 5
  Validation/unit tests: 3
QUALITY TARGET: no stage-transition branch without a direct test
─────────────────────────────────
```

### Test Requirements To Add Or Update

| File | Required assertions |
| --- | --- |
| `crates/xtask/tests/refresh_publication_entrypoint.rs` | assert `refresh-publication --write` sets `lifecycle_stage = "published"` |
| `crates/xtask/tests/refresh_publication_entrypoint.rs` | assert support tier becomes `publication_backed` unless already `first_class` |
| `crates/xtask/tests/refresh_publication_entrypoint.rs` | assert `publication_packet_path` and `publication_packet_sha256` are present and coherent |
| `crates/xtask/tests/refresh_publication_entrypoint.rs` | assert `required_evidence` and `satisfied_evidence` equal the published-stage set |
| `crates/xtask/tests/refresh_publication_entrypoint.rs` | add rollback coverage proving a failed write path does not persist `published` |
| `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs` | seed canonical `published` and assert closeout succeeds |
| `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_paths.rs` | prove canonical next-command flow is `published -> closed_baseline` and that plain prepare-time `publication_ready` is rejected |
| `crates/xtask/tests/agent_lifecycle_state.rs` | add direct published-stage validation for support tier and continuity fields |
| `crates/xtask/tests/support/agent_maintenance_drift_harness.rs` | ensure maintenance accepts `published` and rejects pre-refresh `publication_ready` |
| `crates/xtask/tests/prepare_publication_entrypoint.rs` | keep prepare explicit: it writes `publication_ready`, never `published` |
| `docs/agents/lifecycle/**` fixtures | update seeded post-refresh examples so fixtures match the new machine |

### Regression Rule

This plan fixes a lifecycle regression. The regression test is mandatory:

- `refresh-publication --write` must leave a committed `published` lifecycle state

No deferral. No TODO. That test is the proof that the control-plane lie is gone.

## Performance Review

This is a control-plane write path, not a runtime hot path. The performance risk is wasted
gate work or a sloppy rollback boundary.

Constraints:

1. do not add extra publication-gate passes beyond the existing refresh checks
2. do not rebuild the packet multiple times in one write path without a reason
3. keep rollback snapshots scoped to the publication-owned outputs and lifecycle-owned
   files that refresh mutates
4. do not add concurrency to this seam; determinism matters more than speed here

No caching work, batching, or infra changes are required.

## Implementation Plan

### Slice 1. Commit `published` in the refresh writer

Primary files:

- `crates/xtask/src/publication_refresh.rs`
- `crates/xtask/src/agent_lifecycle.rs`

Exact changes:

1. update `build_next_lifecycle_state(...)` so successful refresh writes
   `lifecycle_stage = published`
2. promote `support_tier` to `publication_backed` unless already `first_class`
3. write `required_evidence` and `satisfied_evidence` for `Published`
4. record `publication_packet_path` and `publication_packet_sha256`
5. preserve refresh ownership metadata and `expected_next_command = close-proving-run ...`
6. keep lifecycle persistence inside the same honest rollback boundary as publication output writes

Exit criteria:

- refresh success leaves a committed `published` lifecycle state
- refresh failure leaves no fake `published` state behind

### Slice 2. Tighten lifecycle, closeout, and maintenance validation

Primary files:

- `crates/xtask/src/agent_lifecycle.rs`
- `crates/xtask/src/agent_lifecycle/validation.rs`
- `crates/xtask/src/close_proving_run.rs`
- `crates/xtask/src/agent_maintenance/drift/governance.rs`
- `crates/xtask/src/capability_publication.rs`

Exact changes:

1. codify `published` as the canonical post-refresh stage everywhere lifecycle truth is validated
2. narrow `publication_ready` back to pre-refresh meaning
3. implement one explicit compatibility branch for legacy post-refresh `publication_ready`
4. require closeout to reject ordinary prepare-time `publication_ready`
5. preserve the rule that post-publication maintenance truth begins at `published` and later

Exit criteria:

- canonical post-refresh stage is `published` everywhere
- compatibility path is explicit, narrow, and testable

### Slice 3. Update docs and seeded fixtures

Primary files:

- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/agents/lifecycle/**`

Exact changes:

1. restate the lifecycle sequence once, consistently
2. update refresh success semantics to say it commits `published`
3. update closeout semantics so `published` is the normal input stage
4. keep `publication-ready.json` described as the pre-refresh handoff packet
5. update only those fixtures that represent post-refresh truth

Exit criteria:

- specs, operator docs, and fixtures match the same machine with no contradictory prose

## NOT In Scope

- renaming `publication-ready.json`
  - rationale: naming churn without fixing the lifecycle lie
- removing `published` from the schema
  - rationale: rejected because it makes the machine less explicit
- redesigning `close-proving-run` artifact authoring
  - rationale: separate pending milestone
- redesigning the publication gate or shrinking `make preflight`
  - rationale: gate semantics are already pinned
- changing support/capability publication ownership
  - rationale: that lane already landed; this slice only makes lifecycle truth match it

## Failure Modes Registry

| Failure mode | Test covers it? | Error handling exists? | User-visible outcome | Critical? |
| --- | --- | --- | --- | --- |
| refresh gate fails after output writes but before lifecycle commit | Planned | Must exist | command failure, no persisted fake published state | Yes |
| refresh succeeds but support tier stays `baseline_runtime` | Planned | lifecycle validation should fail | silent semantic corruption without test | Yes |
| published state lacks packet path or packet sha | Planned | lifecycle validation should fail | closeout or maintenance breaks later | Yes |
| closeout still accepts ordinary `publication_ready` as a steady state | Planned | compatibility branch must be narrow | hidden ambiguity persists | Yes |
| docs still describe refresh as staying in `publication_ready` | Planned | doc review only | operator confusion | No |

Critical gap rule:

Any path that can leave `published` committed without published-stage evidence, correct
support-tier promotion, or packet continuity is a release blocker for this slice.

## Worktree Parallelization Strategy

This plan has real parallelization value because the implementation splits cleanly across
runtime code, test surfaces, and documentation once the core semantics are locked.

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Refresh stage writer | `crates/xtask/src/` lifecycle + refresh modules | — |
| Closeout and validation alignment | `crates/xtask/src/` closeout + lifecycle validation modules | Refresh stage writer |
| Lifecycle and seam tests | `crates/xtask/tests/` | Refresh stage writer |
| Specs and operator docs | `docs/specs/`, `docs/` | Refresh stage writer |
| Seeded lifecycle fixture updates | `docs/agents/lifecycle/` | Refresh stage writer |

### Parallel Lanes

Lane A: Refresh stage writer -> Closeout and validation alignment
Sequential. Shared `crates/xtask/src/` ownership, one semantic authority lane.

Lane B: Lifecycle and seam tests  
Starts after Lane A has locked the public semantics. Independent from docs, but should not
start before the stage contract is stable.

Lane C: Specs, operator docs, and seeded fixture updates
Starts after Lane A locks the exact lifecycle wording. Sequential inside docs because the
same terms repeat across multiple surfaces.

### Execution Order

1. Launch Lane A first.
2. Once Lane A compiles and the lifecycle contract is stable, launch Lane B and Lane C in
   parallel worktrees.
3. Merge Lane B and Lane C.
4. Run the targeted xtask suites and then the full repo gate.

### Conflict Flags

- Lane A and Lane B share semantics but not directories if ownership stays clean
- Lane A and Lane C share terminology, so Lane C must wait until Lane A locks the final wording
- sequential implementation is required inside `crates/xtask/src/`; trying to split refresh
  and closeout semantics into parallel code lanes is just merge-conflict farming

## Verification Matrix

Run at minimum:

```sh
cargo test -p xtask --test refresh_publication_entrypoint
cargo test -p xtask --test agent_lifecycle_state
cargo test -p xtask --test prepare_publication_entrypoint
cargo test -p xtask --test onboard_agent_closeout_preview -- --nocapture
make test
make preflight
```

Targeted assertions:

1. refresh success writes `published`
2. refresh failure rolls back to the pre-refresh state
3. closeout consumes `published` on the normal path
4. compatibility `publication_ready` is explicit and narrow
5. docs and fixtures match the new stage sequence exactly

## Completion Summary

- Step 0: Scope Challenge
  - make `published` real, do not hide publication truth inside `publication_ready`
- Architecture Review
  - one explicit writer, one explicit meaning, one explicit downstream consumer
- Code Quality Review
  - no new abstraction layer, only ownership repair and narrower compatibility
- Test Review
  - direct regression coverage for refresh, closeout, validation, and maintenance seams
- Performance Review
  - keep the gate count stable and the rollback boundary tight
- NOT in scope
  - written
- What already exists
  - written
- Failure modes
  - 4 critical gaps identified and all tied to mandatory tests
- Parallelization
  - 3 lanes, 2 parallel follow-on lanes after 1 root semantic lane

## Ready-To-Implement Outcome

When this plan is done, the repo has one truthful answer to "has publication completed?"

Look at `lifecycle-state.json` after `refresh-publication --write`.

If the answer is not:

```json
{ "lifecycle_stage": "published" }
```

the slice is not done.
