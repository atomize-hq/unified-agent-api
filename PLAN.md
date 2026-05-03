# PLAN - Make The Published State Honest In The Lifecycle Model

Status: planned  
Date: 2026-05-03  
Branch: `codex/recommend-next-agent`  
Base branch: `main`  
Repo: `atomize-hq/unified-agent-api`  
Work item: `Make The Published State Honest In The Lifecycle Model`  
Plan commit baseline: `9aa348a`

Separate design doc: not required for this slice. This is a backend-only control-plane and
repository workflow correction. `PLAN.md` is the design record.

## Objective

Make `published` an honest committed lifecycle stage instead of a schema-only ghost.

After this plan lands:

1. The canonical create-mode path becomes:
   `runtime_integrated -> publication_ready -> published -> closed_baseline`.
2. `refresh-publication --write` becomes the only writer for
   `LifecycleStage::Published`.
3. Green publication stops being an implied condition hidden inside
   `publication_ready` and becomes an explicit committed state with explicit evidence.
4. `close-proving-run` consumes `published` in the normal path and advances only from
   publication truth to closeout truth.
5. Specs, operator docs, lifecycle validation, and maintenance logic all describe the
   same machine.

This matters because the current model asks operators and future automation to reason about a
state that exists in the schema, evidence model, support-tier rules, and validators, but has no
real producer. That is bad control-plane design. A lifecycle stage should either have one owner
that writes it or it should not exist.

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
- Current implementation surfaces:
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
- Live lifecycle fixtures:
  - `docs/agents/lifecycle/aider-onboarding/governance/lifecycle-state.json`
  - `docs/agents/lifecycle/*/governance/publication-ready.json`

## Verified Current State

These facts are verified from the current branch, not inferred.

1. `LifecycleStage` still declares `Published` in
   `crates/xtask/src/agent_lifecycle.rs`.
2. The evidence model already encodes `published` as a distinct stage.
   `PUBLISHED_MINIMUM_EVIDENCE` requires:
   - `publication_packet_written`
   - `support_matrix_check_green`
   - `capability_matrix_check_green`
   - `capability_matrix_audit_green`
   - `preflight_green`
3. `validate_stage_support_tier(...)` already treats `published` and
   `closed_baseline` as the only stages allowed to carry
   `publication_backed` or `first_class`.
4. `refresh-publication --write` currently validates the `publication_ready` seam,
   writes publication outputs, runs the gate, and then only updates:
   - `current_owner_command = "refresh-publication --write"`
   - `expected_next_command = close-proving-run ...`
   - transition provenance fields
   It does not set `lifecycle_stage = published`, does not promote the support tier,
   and does not mark published-stage evidence as satisfied.
5. `prepare-publication --write` is the only stage writer before refresh. It advances
   `runtime_integrated -> publication_ready`, clears
   `active_runtime_evidence_run_id`, and writes `publication-ready.json`.
6. `close-proving-run` currently accepts `publication_ready` or legacy/manual
   `published`, then writes `closed_baseline`.
7. Publication eligibility checks already allow `published` in
   `capability_publication::is_publication_eligible_stage(...)`.
8. Maintenance governance drift treats `published` or `closed_baseline` as valid
   maintenance baselines.
9. `publication-ready.json` is intentionally specialized. Its validator requires:
   - `lifecycle_stage = publication_ready`
   - `support_tier_at_emit = baseline_runtime`
   That means the frozen packet is a handoff artifact, not the durable record of the
   post-refresh state.
10. The live branch currently has an agent at `publication_ready`:
    `docs/agents/lifecycle/aider-onboarding/governance/lifecycle-state.json`
    still points to `refresh-publication --write`.
11. No code path on this branch assigns `LifecycleStage::Published`. The only stage
    writers are:
    - `onboard_agent` -> `enrolled`
    - `runtime_follow_on` -> `runtime_integrated`
    - `prepare_publication` -> `publication_ready`
    - `close_proving_run` -> `closed_baseline`
12. No branch-local design doc exists under `~/.gstack/projects/unified-agent-api/`
    for `codex-recommend-next-agent`. That is acceptable here because this slice is
    backend-only and the plan itself is the design artifact.

## Problem Statement

The machine says there is a `published` stage. The live lane does not.

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

That creates four concrete problems:

1. The state machine and the implementation disagree about when publication becomes
   real.
2. `publication_ready` is overloaded. It means both "handoff packet exists" and
   "publication gate already passed," depending on hidden context.
3. `published` appears in validators and docs but has no single command owner.
4. Future automation cannot answer "has publication completed?" from lifecycle truth
   alone. It has to reverse-engineer evidence, next-command text, or historical
   compatibility logic.

Target shape:

```text
runtime-follow-on --write
  -> prepare-publication --write
       writes publication-ready.json
       writes lifecycle_stage = publication_ready
       expected_next_command = refresh-publication --write
  -> refresh-publication --write
       refreshes publication outputs
       runs green publication gate
       writes lifecycle_stage = published
       support_tier = publication_backed | first_class
       expected_next_command = close-proving-run --write
  -> close-proving-run --write
       writes lifecycle_stage = closed_baseline
```

This is the honest machine:

- `publication_ready` means the committed handoff packet exists and refresh is next.
- `published` means publication-owned surfaces are green and closeout is next.
- `closed_baseline` means closeout truth is committed and maintenance may start.

## Step 0 Scope Challenge

### Premise Check

The repo does not need a new lifecycle concept here. It already has the right concept.
It just fails to commit it.

The real decision is not "invent a better name." The real decision is:

1. make `published` real by giving it one writer, or
2. delete it and push more meaning into `publication_ready`.

Recommendation: choose option 1. It is the more explicit machine, the smaller semantic
diff, and it aligns with the evidence model already in the code.

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| lifecycle stage enum | `agent_lifecycle::LifecycleStage` | Reuse directly. `Published` already exists. |
| published-stage evidence contract | `PUBLISHED_MINIMUM_EVIDENCE` | Reuse directly. Do not invent a second "green publication" marker. |
| publication writer and gate | `publication_refresh.rs` | Reuse directly as the stage owner. This is the natural `published` writer. |
| handoff packet | `publication-ready.json` + `PublicationReadyPacket` | Reuse directly. Keep it as the pre-refresh packet artifact. |
| closeout consumer | `close_proving_run.rs` | Reuse directly. Tighten it around `published` as the normal input stage. |
| support-tier semantics | `validate_stage_support_tier(...)` | Reuse directly. The stage/tier mapping already expects `published`. |
| publication eligibility | `capability_publication.rs` | Reuse directly, but align comments and callers to the now-real stage. |
| maintenance baseline semantics | `agent_maintenance/drift/governance.rs` | Reuse directly, with clarified compatibility rules. |

### Implementation Alternatives

| Option | Summary | Pros | Cons | Recommendation |
| --- | --- | --- | --- | --- |
| A | Make `refresh-publication --write` advance to `published` | Reuses existing schema, evidence, and support-tier rules. Makes the machine explicit. | Requires lifecycle/test/doc updates across the refresh and closeout seam. | **Choose this.** |
| B | Remove `published` from schema and treat green publication as evidence inside `publication_ready` | Avoids adding a new committed transition writer. | Leaves `publication_ready` overloaded, increases ambiguity, and forces more validator/spec churn. | Reject. |
| C | Rename `published` to a new stage name | Could match wording more tightly. | Pure naming churn, no functional gain, bigger migration surface. | Reject. |

### Minimum Complete Change Set

The smallest complete version of this milestone is:

1. make `refresh-publication --write` write `LifecycleStage::Published`
2. promote support tier during refresh to `publication_backed` unless already
   `first_class`
3. populate published-stage required and satisfied evidence during refresh
4. record `publication_packet_path` and `publication_packet_sha256` at refresh time
5. make `close-proving-run` treat `published` as the canonical input stage while
   keeping a bounded compatibility path for existing `publication_ready` fixtures and
   in-flight branches
6. update lifecycle/spec/operator docs so the stage sequence is stated exactly once
7. update tests and seeded fixtures so `published` is observable in the refresh seam

Anything smaller keeps the stage machine dishonest.

### Complexity Check

This slice will touch more than 8 files. That is still the minimal complete version because the
truth boundary spans:

- lifecycle schema and validation
- publication refresh writer logic
- closeout input-stage validation
- maintenance baseline semantics
- spec wording
- operator-guide wording
- refresh and closeout fixtures/tests

Complexity control:

- no new command
- no new artifact type
- no packet rename
- no support/capability publication redesign
- no maintenance workflow redesign
- no closeout schema redesign

### Search / Build Decision

This is mostly a Layer 1 reuse correction with a small Layer 3 truth decision.

- **[Layer 1]** Reuse `LifecycleStage::Published`.
- **[Layer 1]** Reuse `PUBLISHED_MINIMUM_EVIDENCE`.
- **[Layer 1]** Reuse `refresh-publication --write` as the stage owner.
- **[Layer 1]** Reuse `publication-ready.json` as the committed handoff packet.
- **[Layer 1]** Reuse `close-proving-run` as the consumer of published truth.
- **[Layer 3]** Stop treating "green publication" as hidden prose and make it a real
  committed transition.

### TODOS Cross-Reference

This plan closes exactly one pending TODO:

- `Make The Published State Honest In The Lifecycle Model`

It explicitly unblocks, but does not implement:

- `Enclose Create-Mode Closeout Without Ad Hoc Authoring`

### Completeness Decision

The shortcut version would only reword the docs and keep `publication_ready` doing double duty.
That saves almost nothing and preserves the bug.

The complete version is still a boilable lake:

- one existing writer upgraded
- one existing stage made real
- one existing closeout seam tightened
- one documentation pass to remove ambiguity

### Distribution Check

No new binary, package, or published artifact type is introduced.

## Locked Decisions

1. The canonical create-mode stage sequence becomes:
   `approved -> enrolled -> runtime_integrated -> publication_ready -> published -> closed_baseline`.
2. `refresh-publication --write` is the only command allowed to write
   `LifecycleStage::Published`.
3. `publication-ready.json` remains the pre-refresh handoff packet. Its name does not
   change in this milestone.
4. `published` is a committed lifecycle state, not just a validated condition.
5. `publication_ready` means "packet emitted, refresh not yet committed." It no longer
   means "refresh may already be done."
6. Successful refresh promotes `support_tier` to `publication_backed` unless the agent
   is already `first_class`.
7. Successful refresh records `publication_packet_path` and
   `publication_packet_sha256` in `lifecycle-state.json`.
8. `close-proving-run` treats `published` as the normal input stage. It may keep a
   temporary compatibility branch for `publication_ready` only where needed to support
   existing fixtures or in-flight repos.
9. Maintenance drift semantics remain "publication truth exists at `published` and
   later."
10. This milestone does not redesign closeout authoring. It only makes the stage model
    truthful.

## Architecture Review

### State Machine

The lifecycle machine after this change is:

```text
approved
  -> enrolled
  -> runtime_integrated
  -> publication_ready
       owner: prepare-publication --write
       meaning: frozen packet exists, refresh is next
  -> published
       owner: refresh-publication --write
       meaning: publication outputs are written and green
  -> closed_baseline
       owner: close-proving-run --write
       meaning: closeout baseline is committed, maintenance may begin
```

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
    published support/capability outputs
    lifecycle_stage = published
    publication_packet_path / sha
    published-stage evidence
  verifies:
    support-matrix --check
    capability-matrix --check
    capability-matrix-audit
    make preflight

close-proving-run --write
  reads:
    lifecycle_stage = published
    coherent publication-ready.json
    green publication truth
  writes:
    lifecycle_stage = closed_baseline
    closeout_baseline_path
```

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

### Concrete Design Changes

1. `publication_refresh::build_next_lifecycle_state(...)`
   becomes the place that commits `LifecycleStage::Published`.
2. Refresh updates:
   - `lifecycle_stage = published`
   - `support_tier = publication_backed | first_class`
   - `required_evidence = required_evidence_for_stage(Published)`
   - `satisfied_evidence = required_evidence_for_stage(Published)`
   - `publication_packet_path = Some(...)`
   - `publication_packet_sha256 = Some(...)`
   - `expected_next_command = close-proving-run ...`
3. `publication-ready.json` remains valid as a publication handoff packet. Refresh may
   continue rewriting its `lifecycle_state_sha256` to point at the post-refresh
   lifecycle snapshot, but the packet's own `lifecycle_stage` stays
   `publication_ready` because it documents the emit seam, not the current stage.
4. `close-proving-run` validation is tightened so the normal path is
   `published -> closed_baseline`.
5. Compatibility path:
   - allow `publication_ready` only when the refresh-era artifacts indicate a
     pre-migration or partially migrated repository state
   - keep the compatibility branch isolated and documented as transitional
   - do not leave a second ambiguous steady-state path

### Realistic Failure Scenario Per New Or Changed Path

| Codepath | Production failure | Accounted for in plan? |
| --- | --- | --- |
| refresh writes published state before gate completes | lifecycle claims publication is green even though the gate later fails | Yes. State promotion must occur only after output writes and gate success, under the same rollback boundary. |
| refresh writes outputs but forgets packet hash/path | maintenance and closeout cannot prove publication continuity | Yes. Refresh now owns packet path/hash recording. |
| closeout still accepts raw publication_ready silently | operators bypass the published seam and the state machine stays ambiguous | Yes. Compatibility path is explicit and temporary, not the default. |
| docs updated but tests not updated | regressions reintroduce publication_ready as the post-refresh steady state | Yes. Refresh/closeout fixture updates are mandatory. |

## Code Quality Review

### DRY and Ownership

The repo already has the right abstractions. The bug is ownership drift, not missing
abstraction.

Code quality direction:

1. Do not add a second "publication complete" helper type.
2. Do not add a second post-refresh packet format.
3. Keep the stage transition logic localized to:
   - `prepare_publication.rs`
   - `publication_refresh.rs`
   - `close_proving_run.rs`
4. Keep stage-specific validation centralized in `agent_lifecycle.rs` and
   `agent_lifecycle/validation.rs`.

### Explicit Over Clever

Preferred implementation shape:

- update the existing refresh state builder
- update existing validation branches
- update existing tests and fixtures

Avoid:

- hidden "derived published" booleans
- packet-name aliases
- inferring publication completion from `expected_next_command`
- compatibility logic spread across multiple modules

### ASCII Diagram Maintenance

This slice changes lifecycle semantics and should add or update inline ASCII diagrams in the
high-signal lifecycle modules if nearby comments exist or become necessary:

- `crates/xtask/src/agent_lifecycle.rs`
- `crates/xtask/src/publication_refresh.rs`
- `crates/xtask/src/close_proving_run.rs`

At minimum, `PLAN.md` carries the canonical state-machine diagram for this change.

## Test Review

100 percent coverage is the goal for the changed lifecycle seam.

### Affected Codepaths

```text
CODE PATH COVERAGE
===========================
[+] crates/xtask/src/publication_refresh.rs
    │
    ├── run_in_workspace(--check)
    │   ├── validates publication_ready seam
    │   ├── rejects stale publication outputs
    │   └── [GAP] published-stage compatibility not relevant here
    │
    └── run_in_workspace(--write)
        ├── validates publication_ready seam
        ├── writes publication outputs
        ├── runs green gate
        ├── [GAP] writes lifecycle_stage = published
        ├── [GAP] promotes support_tier
        ├── [GAP] writes publication_packet_path / sha
        ├── [GAP] writes published-stage evidence
        └── [GAP] rolls lifecycle mutation back if gate fails after state mutation planning

[+] crates/xtask/src/close_proving_run.rs
    │
    ├── validate_closeout_inputs(...)
    │   ├── accepts published
    │   ├── [GAP] treats published as canonical normal path
    │   └── [GAP] isolates publication_ready as compatibility only
    │
    └── write_closed_baseline(...)
        ├── consumes publication_packet_path / sha
        └── preserves support_tier = first_class when already set

[+] crates/xtask/src/agent_lifecycle.rs
    │
    ├── required_evidence_for_stage(Published)
    ├── validate_stage_support_tier(Published)
    ├── is_resting_stage_v1(Published = false)
    └── [GAP] sample/fixture coverage must prove published is now a real reachable state
```

### User And Operator Flow Coverage

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
    ├── [GAP] existing publication_ready fixture remains closable only through explicit compatibility branch
    └── [GAP] compatibility branch is rejected once published-state prerequisites are expected

[+] Maintenance and publication truth
    │
    ├── [GAP] published is accepted as maintenance baseline
    └── [GAP] publication_ready pre-refresh is not treated as maintenance baseline

─────────────────────────────────
COVERAGE TARGET: all changed paths
  Critical path tests to add/update: 8
  E2E-style CLI seam tests: 4
  Unit/schema validation tests: 4
QUALITY TARGET: no stage-transition branch without a direct test
─────────────────────────────────
```

### Test Requirements To Add Or Update

1. `crates/xtask/tests/refresh_publication_entrypoint.rs`
   - assert `refresh-publication --write` sets `lifecycle_stage = "published"`
   - assert `support_tier = "publication_backed"` unless already `first_class`
   - assert `publication_packet_path` and `publication_packet_sha256` are present
   - assert `required_evidence` and `satisfied_evidence` equal the published-stage set
2. `crates/xtask/tests/refresh_publication_entrypoint.rs`
   - add rollback test proving a gate failure does not leave a persisted published
     lifecycle state
3. `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`
   - seed a canonical `published` lifecycle state and assert closeout succeeds
4. `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_paths.rs`
   - verify canonical next-command flow is `published -> closed_baseline`
5. `crates/xtask/tests/agent_lifecycle_state.rs`
   - add direct coverage for published-stage validation, packet path/hash
     requirements, and support-tier pairing
6. `crates/xtask/tests/support/agent_maintenance_drift_harness.rs`
   - ensure maintenance accepts `published` as publication truth and rejects
     pre-refresh `publication_ready`
7. `crates/xtask/tests/prepare_publication_entrypoint.rs`
   - keep the pre-refresh seam explicit: prepare writes `publication_ready`, not
     `published`
8. Live lifecycle fixtures under `docs/agents/lifecycle/**`
   - update any seeded post-refresh assumptions so the fixture story matches the
     new machine

### Regression Rule

This is a lifecycle regression fix. The regression test is mandatory:

- `refresh-publication --write` must produce a committed `published` state.

No deferral. No TODO. This test is the proof that the control-plane lie was removed.

## Performance Review

This slice is not performance-sensitive in the runtime-user sense. It is a control-plane mutation
path. The relevant performance concern is repeated gate work and mutation ordering.

Findings:

1. Do not introduce extra publication-gate passes beyond the existing refresh checks.
   This is already a heavy path because `make preflight` runs.
2. Do not add redundant packet rebuilds in hot loops. One pre-write validation and one
   post-state packet serialization is enough.
3. Keep rollback file snapshots scoped to the exact publication-owned outputs plus the
   lifecycle surfaces already mutated by refresh.

No new caching, concurrency, or data-volume work is required in this milestone.

## Implementation Plan

### Slice 1. Commit `published` in the refresh writer

Change `publication_refresh.rs` so a successful write path:

1. upgrades lifecycle stage to `published`
2. upgrades support tier to `publication_backed` unless already `first_class`
3. writes published-stage evidence
4. records `publication_packet_path` and `publication_packet_sha256`
5. preserves `expected_next_command = close-proving-run ...`
6. keeps the refresh rollback boundary honest

Exit criteria:

- refresh success leaves a committed published lifecycle state
- refresh failure leaves no fake published state behind

### Slice 2. Tighten lifecycle and closeout validation

Update:

- `agent_lifecycle.rs`
- `agent_lifecycle/validation.rs`
- `close_proving_run.rs`
- `agent_maintenance/drift/governance.rs`
- `capability_publication.rs`

Goals:

1. codify `published` as reachable, required, and canonical after refresh
2. narrow `publication_ready` back to pre-refresh semantics
3. isolate compatibility handling for older or in-flight states

Exit criteria:

- canonical post-refresh stage is published everywhere
- compatibility path is explicit and bounded

### Slice 3. Update docs and seeded lifecycle fixtures

Update:

- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- any seeded lifecycle JSON fixtures that represent refresh-complete states

Goals:

1. one written story across spec, operator guide, and live fixture examples
2. no remaining prose that treats `published` as hypothetical while refresh stays at
   `publication_ready`

Exit criteria:

- docs and fixtures match the new machine exactly

## NOT In Scope

- replacing `publication-ready.json` with a renamed packet artifact
  - rationale: naming churn without fixing the control-plane bug
- removing `published` from the schema
  - rationale: rejected in this milestone because it produces a less explicit machine
- redesigning `close-proving-run` artifact authoring
  - rationale: separate pending milestone
- narrowing or redesigning the `make preflight` publication gate
  - rationale: gate semantics are already pinned and should remain stable in this slice
- changing capability/support publication output ownership
  - rationale: that lane already landed; this slice only makes the lifecycle truthful

## What Already Exists

| Existing code or flow | Role in this plan | Reuse vs change |
| --- | --- | --- |
| `prepare-publication --write` | writes the frozen handoff into `publication_ready` | Reuse with no semantic expansion |
| `refresh-publication --write` | already owns publication writes and green gate | Change to also own stage promotion |
| `publication-ready.json` | committed handoff packet | Reuse as-is |
| `PUBLISHED_MINIMUM_EVIDENCE` | already defines what publication completion means | Reuse directly |
| `validate_stage_support_tier(...)` | already expects publication-backed tiers after publication | Reuse directly |
| `close-proving-run --write` | current consumer of the post-publication seam | Tighten to canonical `published` input |
| maintenance governance drift | already treats `published` as a maintenance-capable state | Clarify and preserve |

## Failure Modes Registry

| Failure mode | Test covers it? | Error handling exists? | User-visible outcome | Critical? |
| --- | --- | --- | --- | --- |
| refresh gate fails after staging lifecycle mutation | Planned | Must exist | command failure, no persisted fake published state | Yes |
| refresh succeeds but support tier stays `baseline_runtime` | Planned | validation should catch via stage/tier mismatch | silent semantic corruption without test | Yes |
| published state lacks packet path/hash | Planned | validation should fail | closeout or maintenance breaks later | Yes |
| closeout still accepts ordinary `publication_ready` as steady state | Planned | partial compatibility branch only | hidden ambiguity persists | Yes |
| docs still describe refresh as staying in `publication_ready` | Planned | doc review only | operator confusion | No |

Critical gap rule:

Any path that can leave `published` committed without published-stage evidence,
support-tier promotion, or packet continuity is a release blocker for this slice.

## Worktree Parallelization Strategy

This plan has parallelization opportunities because the implementation splits cleanly
across runtime code, validation/tests, and docs.

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Refresh stage writer | `crates/xtask/src/` lifecycle + refresh modules | — |
| Closeout and drift alignment | `crates/xtask/src/` closeout + maintenance drift modules | Refresh stage writer |
| Lifecycle and seam tests | `crates/xtask/tests/` | Refresh stage writer |
| Spec and operator docs | `docs/specs/`, `docs/` lifecycle docs | Refresh stage writer |
| Seeded lifecycle fixture updates | `docs/agents/lifecycle/` | Refresh stage writer |

### Parallel Lanes

Lane A: Refresh stage writer -> Closeout and drift alignment  
Sequential, shared `crates/xtask/src/` lifecycle modules.

Lane B: Lifecycle and seam tests  
Starts after Lane A's public interfaces are stable. Sequential inside `crates/xtask/tests/`,
independent from docs.

Lane C: Spec and operator docs -> Seeded lifecycle fixture updates  
Sequential, shared lifecycle documentation surfaces. Can run in parallel with Lane B after
Lane A defines the final semantics.

### Execution Order

1. Launch Lane A first. It defines the real state transition contract.
2. Once Lane A compiles and the semantics are stable, launch Lane B and Lane C in
   parallel worktrees.
3. Merge Lane B and Lane C.
4. Run the full xtask test set plus targeted doc/spec validation.

### Conflict Flags

- Lane A and Lane B both depend on lifecycle semantics but do not need to touch the
  same directories if ownership stays clean.
- Lane A and Lane C both touch lifecycle terminology. The merge risk is low if Lane C
  waits until Lane A locks the exact wording.
- Sequential implementation is required inside `crates/xtask/src/` because refresh,
  closeout, and lifecycle validation share the same module boundary.

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
2. refresh failure rolls back to pre-refresh state
3. closeout consumes `published` as the normal path
4. compatibility path is explicit and narrow
5. docs/spec text matches the new stage sequence

## Completion Summary

- Step 0: Scope Challenge — choose the complete fix, make `published` real
- Architecture Review: 4 core lifecycle issues identified, 1 recommended design locked
- Code Quality Review: no new abstraction required, ownership correction only
- Test Review: diagram produced, 8 concrete coverage updates required
- Performance Review: no hot-path risk, keep gate and rollback bounded
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 new TODOs proposed, this plan consumes an existing TODO directly
- Failure modes: 4 critical gaps flagged
- Outside voice: not run for this drafting pass
- Parallelization: 3 lanes, 2 parallel / 1 sequential root lane
- Lake Score: 1/1 recommendations chose the complete option

## Ready-To-Implement Outcome

When this plan is done, a maintainer should be able to inspect
`lifecycle-state.json` after `refresh-publication --write` and see one truthful answer:

`"lifecycle_stage": "published"`

That is the whole game for this slice.
