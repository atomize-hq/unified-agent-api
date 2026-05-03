# PLAN - Enclose The Publication Lane End To End

Status: planned  
Date: 2026-05-02  
Branch: `codex/recommend-next-agent`  
Base branch: `main`  
Repo: `atomize-hq/unified-agent-api`  
Work item: `Enclose The Publication Lane End To End`  
Plan commit baseline: `bfd6fd4`

Separate design doc: not required for this slice. This is a backend-only control-plane and CLI
workflow change. `PLAN.md` is the design record.

## Objective

Make publication refresh one repo-owned command instead of an operator choreography.

After this plan lands:

1. `prepare-publication --write` will still freeze runtime evidence and emit
   `publication-ready.json`, but its next-step contract will point to one publication consumer.
2. A new `xtask` command will consume `publication-ready.json`, materialize the publication-owned
   support and capability surfaces, run the green publication gate, and roll back on failure.
3. The operator will move from `prepare-publication` to one command, then to `close-proving-run`,
   instead of manually composing `support-matrix`, `capability-matrix`, `capability-matrix-audit`,
   and `make preflight`.
4. The repo will have one explicit answer to "what writes publication truth, what verifies it, and
   what happens if the gate is not green?"

This matters because the current lane is mechanically possible but operationally sloppy. The user
here is the maintainer running create-mode onboarding. Right now they get a frozen handoff packet,
then a loose shell checklist. That is where partial writes and archaeology creep in.

## Source Inputs

- Backlog source:
  - `TODOS.md`
  - `docs/backlog/cli-agent-onboarding-lifecycle-unification-gap-memo.md`
- Normative contracts:
  - `docs/specs/cli-agent-onboarding-charter.md`
  - `docs/specs/agent-registry-contract.md`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- Procedure source:
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
- Current implementation surfaces:
  - `crates/xtask/src/prepare_publication.rs`
  - `crates/xtask/src/agent_lifecycle.rs`
  - `crates/xtask/src/support_matrix.rs`
  - `crates/xtask/src/capability_matrix.rs`
  - `crates/xtask/src/capability_matrix_audit.rs`
  - `crates/xtask/src/capability_publication.rs`
  - `crates/xtask/src/close_proving_run.rs`
  - `crates/xtask/src/agent_maintenance/refresh.rs`
  - `crates/xtask/src/workspace_mutation.rs`
- Current tests and fixtures:
  - `crates/xtask/tests/prepare_publication_entrypoint.rs`
  - `crates/xtask/tests/agent_lifecycle_state.rs`
  - `crates/xtask/tests/agent_maintenance_refresh.rs`
  - `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`
  - `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_paths.rs`

## Verified Current State

These facts are verified from the current branch, not inferred.

1. `prepare_publication::build_publication_ready_packet(...)` already writes an explicit
   `required_publication_outputs` list derived from registry publication flags. Today that list is:
   - `cli_manifests/support_matrix/current.json`
   - `docs/specs/unified-agent-api/support-matrix.md`
   - `docs/specs/unified-agent-api/capability-matrix.md`
2. `prepare-publication --write` advances lifecycle state only to `publication_ready`, writes
   `publication-ready.json`, clears the active runtime evidence selector, and stops there.
3. `agent_lifecycle::PUBLICATION_READY_NEXT_COMMAND` is still the raw shell chain:
   `support-matrix --check && capability-matrix --check && capability-matrix-audit && make preflight && close-proving-run --write`.
4. The operator guide still tells maintainers to run:
   - `support-matrix`
   - `capability-matrix`
   - `support-matrix --check`
   - `capability-matrix --check`
   - `capability-matrix-audit`
   - `make preflight`
   as separate steps after `prepare-publication`.
5. `agent_maintenance::refresh::build_refresh_plan(...)` already knows how to render publication
   surfaces in memory using:
   - `support_matrix::generate_publication_artifacts(...)`
   - `capability_matrix::generate_markdown()`
   and then apply those writes through `workspace_mutation`.
6. `close-proving-run` already treats green publication as a prerequisite. It re-runs drift checks,
   re-runs the shared capability audit, and refuses closeout when published support/capability
   truth is stale.
7. `make preflight` is broader than publication freshness. It already includes:
   - `support-matrix --check`
   - `capability-matrix --check`
   - `capability-matrix-audit`
   plus hygiene, fmt, clippy, check, test, LOC, publish guards, and security.
8. No branch-local design doc was found under `~/.gstack/projects/unified-agent-api/` for
   `codex-recommend-next-agent`. That is acceptable here because this slice is backend-only and
   the plan itself is the design artifact.

## Problem Statement

The publication lane has a frozen handoff, but not a real owner.

Current shape:

```text
runtime-follow-on --write
  -> prepare-publication --write
       writes publication-ready.json
       sets lifecycle_stage = publication_ready
       points operator at a shell checklist
  -> operator manually runs publication writers
  -> operator manually runs publication checks
  -> close-proving-run --write
```

That creates three concrete problems:

1. The write set exists in the packet, but the repo does not own the act of materializing it.
2. Publication refresh is not transactional. A maintainer can update one published surface, fail
   later in the gate, and be left with a dirty repo plus an ambiguous next step.
3. The lifecycle says "publication_ready", but the next command is not really one command. It is a
   prose instruction disguised as lifecycle truth.

Target shape:

```text
runtime-follow-on --write
  -> prepare-publication --write
       writes publication-ready.json
       sets expected_next_command = refresh-publication --approval ... --write
  -> refresh-publication --write
       reads publication-ready.json
       writes required publication outputs
       runs the green publication gate
       rolls back on failure
       narrows next step to close-proving-run
  -> close-proving-run --write
```

## Step 0 Scope Challenge

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| frozen publication handoff | `prepare_publication.rs`, `publication-ready.json`, `agent_lifecycle.rs` | Reuse directly. Do not invent a second handoff packet. |
| required publication outputs | `PublicationReadyPacket.required_publication_outputs` | Reuse directly. This remains the write contract. |
| support publication rendering | `support_matrix::generate_publication_artifacts(...)` | Reuse directly. No shell-out for write generation. |
| capability publication rendering | `capability_matrix::generate_markdown()` | Reuse directly. No duplicate markdown builder. |
| capability publication audit | `capability_publication::audit_current_capability_publication(...)` | Reuse directly. Do not clone audit rules into the new command. |
| safe file writes and rollback-friendly snapshots | `workspace_mutation.rs` | Reuse for publication-owned file writes. |
| publication refresh planning | `agent_maintenance::refresh::build_refresh_plan(...)` | Reuse by extraction. Create-mode and maintenance should not diverge. |
| closeout publication validation | `close_proving_run.rs` | Reuse the same green-surface contract. Do not redefine closeout prerequisites. |

### Minimum Complete Change Set

The smallest complete version of this milestone is:

1. add one new `xtask` publication consumer command
2. extract one shared publication-output planning helper so create-mode and maintenance-mode use
   the same render/write set
3. repoint `prepare-publication` and lifecycle next-step text at the new command
4. teach the new command to write, verify, and roll back publication-owned surfaces
5. update docs and lifecycle fixtures so the new contract is the only create-mode story

Anything smaller leaves the repo with a packet that still delegates real publication ownership to
the operator.

### Complexity Check

This slice will touch more than 8 files. That is still the minimal complete version because the
seam spans:

- CLI wiring in `xtask`
- lifecycle next-command semantics
- shared publication rendering/planning
- operator docs and charter wording
- publication-ready fixtures and lifecycle tests

Complexity control:

- one new command only
- one shared planner extraction only
- no new lifecycle stage
- no new artifact family
- no support or capability semantics rewrite
- no closeout schema expansion

### Search / Build Decision

This is a Layer 1 reuse problem with one Layer 3 control-plane correction.

- **[Layer 1]** Reuse `publication-ready.json` as the publication contract.
- **[Layer 1]** Reuse `support_matrix::generate_publication_artifacts(...)`.
- **[Layer 1]** Reuse `capability_matrix::generate_markdown()`.
- **[Layer 1]** Reuse `capability_publication::audit_current_capability_publication(...)`.
- **[Layer 1]** Reuse `workspace_mutation` instead of inventing a second file-apply path.
- **[Layer 1]** Reuse maintenance refresh planning logic by extraction, not copy/paste.
- **[Layer 3]** Treat publication as a lifecycle-owned command boundary, not as prose plus four
  shell commands.

### TODOS Cross-Reference

This plan closes exactly one pending TODO:

- `Enclose The Publication Lane End To End`

It explicitly unblocks, but does not implement:

- `Make The Published State Honest In The Lifecycle Model`
- `Enclose Create-Mode Closeout Without Ad Hoc Authoring`

### Completeness Decision

The shortcut version would add a doc alias or shell wrapper and still leave the repo with split
write logic and non-transactional behavior. That is not good enough.

The complete version is still a boilable lake:

- one command
- one packet contract
- one write plan
- one green gate
- one rollback story

### Distribution Check

No new binary, package, container image, or release track is introduced here.

## Locked Decisions

1. Add a new `xtask` subcommand:  
   `cargo run -p xtask -- refresh-publication --approval <path> --check|--write`
2. `prepare-publication` stays a handoff writer. It does not start writing support/capability
   outputs directly.
3. `refresh-publication` consumes `publication-ready.json` as the only committed create-mode
   publication packet.
4. `PublicationReadyPacket.required_publication_outputs` remains the authoritative write set. The
   new command does not invent a second list.
5. Extract shared publication-output planning so create-mode `refresh-publication` and maintenance
   `refresh-agent` render the same support/capability surfaces from the same helper.
6. `refresh-publication --write` is transactional over publication-owned committed surfaces and any
   lifecycle metadata it updates. On gate failure, those committed files must be restored.
7. `make preflight` remains part of the publication green gate in this slice. Do not replace it
   with a narrower custom gate yet.
8. This slice does not resolve `LifecycleStage::Published`. Green publication remains a validated
   condition while lifecycle stage semantics stay on the current `publication_ready` path.
9. `refresh-publication --check|--write` requires lifecycle stage `publication_ready`. It consumes
   `publication-ready.json` and may update `lifecycle-state.json`, but it does not rewrite the
   packet itself.

## Architecture Review

### Command Boundary

The new command should be a true consumer, not a thin shell alias.

Recommended module shape:

| Responsibility | File |
| --- | --- |
| CLI args + top-level orchestration | `crates/xtask/src/publication_refresh.rs` |
| CLI registration | `crates/xtask/src/main.rs` |
| public exports | `crates/xtask/src/lib.rs` |
| shared publication output planning | extracted from `crates/xtask/src/agent_maintenance/refresh.rs` |
| lifecycle next-command helpers | `crates/xtask/src/agent_lifecycle.rs` |
| publication handoff producer alignment | `crates/xtask/src/prepare_publication.rs` |

### Target Flow

```text
publication-ready.json
  │
  ├── validate approval / lifecycle / packet continuity
  ├── derive publication output plan
  │     ├── support-matrix JSON + Markdown, when enabled
  │     └── capability-matrix Markdown, when enabled
  ├── snapshot current committed publication surfaces
  ├── write candidate surfaces
  ├── run green gate
  │     ├── support-matrix --check
  │     ├── capability-matrix --check
  │     ├── capability-matrix-audit
  │     └── make preflight
  ├── on failure: restore snapshots, keep lifecycle pre-refresh
  └── on success: keep written surfaces, update lifecycle next-step text
```

### Transaction Model

This is the main landmine. The repo already writes files atomically, but the gate is broader than
the file writes.

Implementation rule:

1. Build candidate publication outputs in memory.
2. Snapshot the current bytes for every `required_publication_output`.
3. Apply the candidate committed files.
4. Run the green gate.
5. If any gate step fails:
   - restore all snapped publication outputs
   - restore any lifecycle file touched by the command
   - surface the exact failing command and keep the repo in pre-refresh committed state
6. If all gate steps pass:
   - persist the new committed publication surfaces
   - narrow lifecycle `expected_next_command` to closeout

This is engineered enough. It does not require a temp worktree or a repo clone. It only needs
explicit snapshots for the small committed write set.

### Lifecycle Behavior

Before refresh success:

- `prepare-publication --write` should set:
  - `current_owner_command = "prepare-publication --write"`
  - `expected_next_command = "refresh-publication --approval <path> --write"`

After refresh success:

- lifecycle stage remains `publication_ready`
- `current_owner_command = "refresh-publication --write"`
- `expected_next_command = "close-proving-run --approval <path> --closeout docs/agents/lifecycle/<prefix>/governance/proving-run-closeout.json"`

Why keep the stage unchanged:

- the repo already has a separate TODO for honest `published` state semantics
- mixing that lifecycle redesign into this slice spends an innovation token for no reason
- `close-proving-run` already revalidates green publication truth, so we do not need a stage
  redesign just to own the refresh lane

### Prepare-Publication Check Compatibility

`prepare-publication --check` currently rejects any `publication_ready` lifecycle state whose
`expected_next_command` is not the old raw shell chain.

That must be updated.

New rule:

- at `runtime_integrated`, `prepare-publication --check` still expects the future next step to be
  `refresh-publication --approval ... --write`
- at `publication_ready`, it must accept both:
  - pre-refresh state: `refresh-publication --approval ... --write`
  - post-refresh state: `close-proving-run --approval ... --closeout ...`

This keeps the handoff checker useful without making it a second publication freshness gate.

## Code Quality Review

### DRY Boundaries

The repo already has two partial write paths for publication surfaces:

- create-mode intent in `prepare_publication.rs`
- maintenance-mode refresh planning in `agent_maintenance/refresh.rs`

Do not add a third.

Recommendation:

- extract a shared publication output planner that returns the support/capability files implied by
  a publication contract
- let `refresh-publication` call that helper directly
- make `refresh-agent` call the same helper for `support_matrix_refresh` and
  `capability_matrix_refresh`

That keeps maintenance and create-mode aligned. It also makes the next bug obvious instead of
duplicated.

### Explicit Over Clever

Do not build a generic "run arbitrary required commands from JSON" executor.

The packet's `required_commands` list is a validation contract, not a scripting engine. The new
command should call the known gate steps explicitly in Rust:

- `support_matrix::run(Args { check: true })`
- `capability_matrix::run(Args { check: true, out: None })`
- `capability_publication::audit_current_capability_publication(...)`
- a subprocess call for `make preflight`

That is boring and readable. Good.

### Minimal Diff Guardrails

Do not:

- teach `prepare-publication` to generate published outputs
- route create-mode publication through a synthetic maintenance request artifact
- add a new packet schema just to carry the same output list twice
- change support/capability semantics while solving ownership

## DX / Operator Experience

This is a developer-facing command, so the UX matters.

Success criteria for the maintainer:

1. After `prepare-publication --write`, the next step is exactly one command.
2. `refresh-publication --check` tells them whether current publication surfaces already satisfy
   the frozen packet.
3. `refresh-publication --write` prints:
   - approval path
   - packet path
   - agent id
   - planned publication outputs
   - gate steps it ran
   - whether rollback occurred
4. A failure message names the exact failing gate and leaves the committed repo state where it was
   before the command started.

Target operator time-to-green after runtime evidence exists:

- today: multiple commands plus manual sequencing
- target: one command to refresh and verify, one command to close out

## Test Review

100% coverage is the bar for the new command boundary and its failure paths.

### Code Path Coverage

```text
CODE PATH COVERAGE
===========================
[GAP] publication_refresh::run_in_workspace()
  ├── publication_ready happy path
  ├── packet / approval / lifecycle mismatch
  ├── support-only publication agent
  ├── capability-only publication agent
  ├── support+capability publication agent
  ├── gate failure after writes -> rollback
  ├── lifecycle update failure after green gate -> rollback
  └── idempotent re-run when outputs are already fresh

[GAP] prepare_publication::validate_check_mode()
  ├── pre-refresh publication_ready next command accepted
  └── post-refresh closeout next command accepted

[GAP] shared publication output planner
  ├── create-mode and maintenance-mode produce identical support outputs
  └── create-mode and maintenance-mode produce identical capability outputs
```

### User Flow Coverage

```text
USER FLOW COVERAGE
===========================
[GAP] prepare-publication --write
  -> refresh-publication --write
  -> close-proving-run --write

[GAP] refresh-publication --check
  -> detects stale support matrix only

[GAP] refresh-publication --check
  -> detects stale capability matrix only

[GAP] refresh-publication --write
  -> make preflight fails
  -> publication files restored
  -> lifecycle next step unchanged

[GAP] refresh-publication --write
  -> success
  -> close-proving-run sees green publication truth without extra manual steps
```

### Required Tests

Add or update these test surfaces:

1. New entrypoint suite:
   - `crates/xtask/tests/refresh_publication_entrypoint.rs`
2. Shared planner parity:
   - extend `crates/xtask/tests/agent_maintenance_refresh.rs`
3. Lifecycle contract updates:
   - extend `crates/xtask/tests/prepare_publication_entrypoint.rs`
   - extend `crates/xtask/tests/agent_lifecycle_state.rs`
4. Create-lane closeout integration:
   - extend `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`
   - extend `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_paths.rs`

### Regression Rules

These regressions are mandatory to cover:

- a publication-ready packet exists, but the repo still requires manual `support-matrix` and
  `capability-matrix` invocation to become green
- `refresh-publication --write` updates one published surface, then fails later and leaves the repo
  half-refreshed
- create-mode publication and maintenance refresh generate different bytes for the same support or
  capability surface
- `prepare-publication --check` starts failing legitimate post-refresh `publication_ready` states
  because lifecycle next-command text changed

### Required Test Commands

```bash
cargo test -p xtask --test prepare_publication_entrypoint
cargo test -p xtask --test refresh_publication_entrypoint
cargo test -p xtask --test agent_maintenance_refresh
cargo test -p xtask --test agent_lifecycle_state
cargo test -p xtask --test onboard_agent_closeout_preview
make check
```

### Verification Commands

```bash
cargo run -p xtask -- prepare-publication --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --write
cargo run -p xtask -- refresh-publication --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --check
cargo run -p xtask -- refresh-publication --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --write
cargo run -p xtask -- capability-matrix-audit
make preflight
```

## Failure Modes Registry

| Failure mode | Detection | Handling | Test required | Critical gap today |
| --- | --- | --- | --- | --- |
| `publication-ready.json` exists, but no repo-owned command consumes it | missing CLI entrypoint | add `refresh-publication` | yes | yes |
| support matrix writes succeed, but capability or preflight fails later | integration failure after file write | restore snapped publication outputs | yes | yes |
| lifecycle next command still points at raw shell chain after refresh lands | lifecycle test + operator guide diff | update lifecycle helper and docs together | yes | yes |
| create-mode refresh and maintenance refresh drift to different render logic | shared planner parity test | extract one helper | yes | yes |
| packet output list and actual write set diverge | packet-driven write-plan validation | fail before write | yes | yes |
| `make preflight` fails after publication files are updated | rollback integration test | restore files, surface failing gate | yes | yes |
| closeout path is still ambiguous after green publication | lifecycle next-step test | set exact closeout command template | yes | no |
| future `published` lifecycle redesign leaks into this slice | review + test boundary | defer to next TODO | no | no |

Critical gap definition for this slice:

- any state where publication surfaces were changed in commit-worthy files, but the publication gate
  did not go green and the repo did not restore pre-command committed publication truth

## Performance Review

Performance is not the product risk here. Correctness is.

Still, the command should stay boring:

1. generate publication files in process, not via shell nesting
2. touch only the packet-required publication outputs
3. run one `make preflight`, not a custom second copy of the preflight pipeline
4. avoid a temp repo clone or temp worktree unless rollback-on-failure proves insufficient

Expected cost:

- publication render time is trivial
- `make preflight` dominates runtime, and that is acceptable because the command is explicitly the
  green publication gate

## Worktree Parallelization Strategy

There is real parallelization here, but only after the command contract is frozen.

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| command contract + lifecycle next-step helpers | `crates/xtask/src/`, `docs/cli-agent-onboarding-factory-operator-guide.md`, `docs/specs/cli-agent-onboarding-charter.md` | — |
| shared publication output planner extraction | `crates/xtask/src/`, `crates/xtask/src/agent_maintenance/` | command contract |
| create-mode command implementation | `crates/xtask/src/`, `crates/xtask/tests/` | command contract, shared planner |
| docs and fixture updates | `docs/`, `docs/agents/lifecycle/**`, `crates/xtask/tests/` | command contract |
| final verification and closeout-path regression tests | `crates/xtask/tests/`, repo root `Makefile` gate usage | create-mode command, docs and fixture updates |

### Parallel Lanes

- Lane A: command contract + lifecycle next-step helpers
  - sequential, shared `crates/xtask/src/`
- Lane B: shared publication output planner extraction
  - starts after Lane A
- Lane C: create-mode command implementation and rollback logic
  - starts after Lane B
- Lane D: docs and fixture updates
  - starts after Lane A once command naming is frozen
- Lane E: final verification and regression tests
  - starts after C and D merge

### Execution Order

1. Launch Lane A first.
2. Once naming and lifecycle semantics are locked, launch Lanes B and D in parallel.
3. Launch Lane C after B freezes the shared planner API.
4. Merge C and D.
5. Run Lane E and the full verification chain.

### Conflict Flags

- Lanes A, B, and C all touch `crates/xtask/src/`. Do not run them in the same worktree.
- Lane D touches fixtures and operator docs that reference command names. Do not start it until
  Lane A freezes those names.
- Lane E is sequential. It depends on final behavior, not draft wiring.

## NOT In Scope

- redesigning `LifecycleStage::Published`
- adding new lifecycle evidence ids for green publication
- scaffolding `proving-run-closeout.json`
- teaching create-mode publication to refresh release docs
- replacing `make preflight` with a narrower custom publication gate
- rewriting support-matrix or capability-matrix semantics
- changing maintenance request scope beyond reusing the shared planner

## Acceptance Criteria

1. `prepare-publication --write` points to `refresh-publication --approval ... --write`, not to a
   raw shell chain.
2. `refresh-publication --write` consumes `publication-ready.json` and writes exactly the packet's
   `required_publication_outputs`.
3. `refresh-publication --write` runs the publication gate and restores committed publication files
   on failure.
4. `refresh-publication --check` can validate an already-green publication state without rewriting
   files.
5. Create-mode publication and maintenance refresh use the same support/capability render logic.
6. `close-proving-run` works unchanged against a post-refresh `publication_ready` baseline.
7. The operator guide and charter describe one publication consumer command, not a loose checklist.

## What Success Looks Like

When this lands, the maintainer flow becomes:

1. run `prepare-publication --write`
2. run `refresh-publication --approval <path> --write`
3. author `proving-run-closeout.json`
4. run `close-proving-run --approval <path> --closeout <path>`

No manual publication command choreography. No half-refreshed committed surfaces. No ambiguity
about who owns the publication write set.

## TODO Relation

`TODOS.md` does not need a new item for this slice.

This plan is the implementation plan for:

- `Enclose The Publication Lane End To End`

The remaining pending TODOs stay deferred as written.

## Review Summary

- Step 0: Scope Challenge — scope accepted as-is; this is the minimum complete slice
- Architecture Review: 1 new command, 1 shared planner extraction, no lifecycle-stage redesign
- Code Quality Review: duplicate publication render planning removed across create-mode and
  maintenance-mode
- Test Review: coverage diagram produced, 10 concrete gaps enumerated
- Performance Review: bounded file generation, preflight remains the dominant and intentional cost
- NOT in scope: written
- What already exists: written
- Failure modes: 8 failure modes listed, 6 critical gaps flagged on the current branch
- Parallelization: 5 steps, 2 useful parallel lanes after command naming is frozen
- Lake Score: the complete option won every major decision in this slice
