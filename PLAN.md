# PLAN - Runtime Evidence Repair And Publication Seam Hardening

Status: implemented  
Date: 2026-05-02  
Branch: `codex/recommend-next-agent`  
Base branch: `main`  
Repo: `atomize-hq/unified-agent-api`  
Work item: `Repair stale runtime_integrated evidence and harden the runtime -> publication seam`

## Implementation Notes

- `repair-runtime-evidence --check` now stages a temporary runtime-evidence bundle and validates it through the same runtime-evidence semantic rules `prepare-publication` uses, via an explicit-run-root validator.
- `repair-runtime-evidence --write` remains on the canonical promote -> validate -> rollback path so the committed `run_dir` metadata stays truthful for the canonical repair run id.
- Shared runtime-owned backend derivation now includes all direct files under `descriptor.backend_module` and still excludes nested backend files.
- Historical refreshes are intentionally targeted per agent. Those commands primarily regenerate runtime-evidence run directories, but downstream governance artifacts may also change if the targeted backfill rewrites them mechanically.
- The lifecycle still lacks an explicit pointer to the active runtime-evidence run. That lifecycle-pointer redesign is a follow-on item, not part of this bounded fix.

## Objective

Land one repo-owned repair seam for already-committed `runtime_integrated` agents whose runtime evidence is stale or malformed, use it to repair `aider`, and close the validation gap that currently lets a bad runtime bundle look green until `prepare-publication` consumes it.

After this plan lands:

1. `repair-runtime-evidence --check` can tell an operator whether a `runtime_integrated` agent is repairable from committed runtime-owned outputs.
2. `repair-runtime-evidence --write` can reconstruct a truthful runtime evidence bundle without hand-editing JSON.
3. `prepare-publication --check` passes for `aider`.
4. `runtime-follow-on` no longer tolerates legacy short-form publication commands.
5. `check-agent-drift` surfaces stale runtime evidence as an explicit operator-facing failure, not a vague governance mismatch.

## Source Inputs

- Plan artifact:
  - `~/.gstack/projects/atomize-hq-unified-agent-api/ceo-plans/2026-05-01-runtime-integrated-evidence-repair-plan.md`
- Test artifact:
  - `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-test-plan-20260501-184500.md`
- Upstream design context:
  - `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-design-20260430-214712.md`
- Verified live failure:
  - `cargo run -p xtask -- prepare-publication --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --check`
  - current stderr: `runtime input-contract required_handoff_commands must match the frozen publication command set exactly`
- Relevant code owners:
  - `crates/xtask/src/main.rs`
  - `crates/xtask/src/lib.rs`
  - `crates/xtask/src/historical_lifecycle_backfill.rs`
  - `crates/xtask/src/runtime_follow_on.rs`
  - `crates/xtask/src/runtime_follow_on/lifecycle.rs`
  - `crates/xtask/src/prepare_publication.rs`
  - `crates/xtask/src/prepare_publication/runtime_evidence.rs`
  - `crates/xtask/src/agent_maintenance/drift/mod.rs`
  - `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
  - `crates/xtask/tests/prepare_publication_entrypoint.rs`
  - `crates/xtask/tests/agent_maintenance_drift.rs`
- Broken committed `aider` evidence:
  - `docs/agents/lifecycle/aider-onboarding/governance/lifecycle-state.json`
  - `docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/input-contract.json`
  - `docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/handoff.json`
  - `docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/run-status.json`
  - `docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/validation-report.json`
  - `docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/written-paths.json`
  - `docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/run-summary.md`

## Verified Current State

These facts are verified from the current branch, not inferred:

1. `docs/agents/lifecycle/aider-onboarding/governance/lifecycle-state.json` says `aider` is `runtime_integrated` with `baseline_runtime` support and `prepare-publication` as the next command.
2. `docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/input-contract.json` and `handoff.json` still use the legacy short-form command set:
   - `support-matrix --check`
   - `capability-matrix --check`
   - `capability-matrix-audit`
   - `make preflight`
3. `docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/written-paths.json` is empty.
4. `docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/run-status.json` still claims `write_validated`.
5. `prepare-publication` is stricter than the old runtime bundle and rejects it exactly as it should.
6. The forward runtime path is already partially hardened:
   - `crates/xtask/src/runtime_follow_on.rs` now rejects `written_paths.is_empty()`.
   - `crates/xtask/tests/runtime_follow_on_entrypoint.rs` already contains `runtime_follow_on_write_rejects_noop_runtime_execution`.
7. The remaining gap is historical and operator-facing:
   - stale committed runtime bundles can still exist on disk
   - `prepare-publication` rejects them late
   - `check-agent-drift --agent aider` reports generic governance drift, not a targeted runtime-evidence repair instruction

## Problem Statement

The branch has a truthful publication consumer and a stale committed producer artifact.

This is not a broad lifecycle redesign problem anymore. The lifecycle schema, `prepare-publication`, and runtime write validation are already in the repo. The bug is narrower and more dangerous:

- old runtime evidence can remain committed after the contract tightened
- the current repo has no explicit repair seam for that state
- the operator only learns the bundle is bad when the next lifecycle command rejects it

That is why `aider` looks close to done in lifecycle state but is still blocked in practice.

## Decision Summary

These choices are now locked. They are not open questions.

1. Add a new explicit command: `repair-runtime-evidence`.
2. Extract shared runtime evidence reconstruction and writing helpers from `historical_lifecycle_backfill.rs` instead of building a second ad hoc reconstruction path.
3. Keep `prepare-publication` strict. Do not widen legacy tolerance there.
4. Remove legacy short-form command tolerance from `runtime_follow_on/lifecycle.rs`.
5. Write repaired evidence to a deterministic repair run directory:
   - `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-<agent_id>-runtime-follow-on/`
   - for this slice: `repair-aider-runtime-follow-on`
6. Do not mutate lifecycle stage during repair. `repair-runtime-evidence` fixes runtime evidence only. `prepare-publication` remains the owner of the `runtime_integrated -> publication_ready` transition.
7. Surface stale runtime evidence in drift output as a first-class finding, not buried under generic governance text.

## Scope

### In Scope

- add `xtask repair-runtime-evidence --check/--write`
- add one shared runtime evidence bundle helper used by repair and historical backfill
- use the shared helper to derive truthful non-empty runtime-owned writes from committed outputs
- repair `aider` into a consumable runtime evidence bundle
- remove legacy short-form command tolerance from runtime handoff validation
- add explicit stale-runtime-evidence drift detection
- add targeted tests for repair, drift, and strict command continuity
- update operator docs for the new repair seam

### Out Of Scope

- redesigning the lifecycle schema
- changing `prepare-publication` packet shape
- changing the bounded runtime ownership model
- deleting every old stale runtime run directory in the repo
- turning repair into a generic maintenance framework
- expanding CI with new global workflows beyond existing test and `make preflight` coverage

## Step 0 Scope Challenge

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| lifecycle truth | `crates/xtask/src/agent_lifecycle.rs` | Reuse directly. No schema redesign. |
| runtime bundle discovery and strict consumption | `crates/xtask/src/prepare_publication/runtime_evidence.rs` | Reuse directly. Repair must produce bundles this code already accepts. |
| forward runtime write validation | `crates/xtask/src/runtime_follow_on.rs` | Reuse directly. Empty runtime writes are already rejected for new runs. |
| handoff semantic validation | `crates/xtask/src/runtime_follow_on/lifecycle.rs` | Reuse but tighten. Remove legacy command tolerance. |
| historical runtime bundle reconstruction | `crates/xtask/src/historical_lifecycle_backfill.rs` | Extract and reuse. This is the right starting point for repair logic. |
| drift reporting framework | `crates/xtask/src/agent_maintenance/drift/mod.rs` | Reuse and extend with a specific runtime evidence finding. |
| test harnesses | `crates/xtask/tests/*_entrypoint.rs` | Reuse and extend. No new harness family. |

### Minimum Complete Change Set

The smallest complete version of this work is:

1. add `repair-runtime-evidence`
2. extract shared bundle reconstruction and writer helpers
3. remove legacy short-form command tolerance in `runtime_follow_on/lifecycle.rs`
4. add runtime-evidence drift detection
5. repair `aider`
6. add regression tests for all of the above

Anything smaller leaves the repo with split truth again.

### Complexity Check

This work will touch more than 8 files, but it is still the minimal complete slice because the problem spans:

- command wiring
- shared helper extraction
- runtime handoff validation
- publication consumption
- drift reporting
- committed `aider` artifacts
- tests
- docs

The complexity control is not “touch fewer files.” The complexity control is “one shared reconstruction helper, one repair command, one deterministic repair directory, zero schema changes.”

### Search / Build Decision

This is a Layer 1 reuse problem, not a new architecture problem.

- **[Layer 1]** Reuse the existing `xtask` command pattern in `crates/xtask/src/main.rs`.
- **[Layer 1]** Reuse strict command validation from `agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS`.
- **[Layer 1]** Reuse discovery and consumption rules from `prepare_publication/runtime_evidence.rs`.
- **[Layer 1]** Reuse committed-output derivation from `historical_lifecycle_backfill.rs`.
- **[Layer 3]** Keep repair outside lifecycle-stage mutation. The repo needs a repair seam, not another stage owner.

### TODOS Cross-Reference

No new `TODOS.md` entry is required for this slice.

This plan closes an implementation gap inside the already-active lifecycle and publication follow-on work. A follow-up cleanup for pruning superseded stale run directories is intentionally deferred and does not block this repair milestone.

### Completeness Decision

The shortcut version would be:

- hand-edit the broken `aider` JSON files
- leave legacy tolerance in runtime handoff validation
- rely on `prepare-publication` to keep catching the problem late

That is not acceptable. The complete version is still a small lake. Build the real repair command and the real guardrail now.

## Architecture

### Current Failure Flow

```text
lifecycle-state.json
  stage = runtime_integrated
  next = prepare-publication
        |
        v
stale runtime bundle on disk
  - legacy short-form commands
  - empty written-paths.json
  - run-status still says write_validated
        |
        v
prepare-publication --check
  rejects exact command mismatch
```

### Target Flow

```text
runtime_integrated lifecycle state
        |
        v
repair-runtime-evidence --check
  - load approval + registry + lifecycle
  - derive runtime-owned writes from committed outputs
  - synthesize exact command set
  - validate bundle with publication consumer rules
        |
        v
repair-runtime-evidence --write
  - write deterministic repair run dir
  - revalidate the written bundle
  - leave lifecycle stage unchanged
        |
        v
prepare-publication --check
  passes
        |
        v
prepare-publication --write
  advances lifecycle to publication_ready
```

### Command Contract

New command surface:

```bash
cargo run -p xtask -- repair-runtime-evidence --approval <path> --check
cargo run -p xtask -- repair-runtime-evidence --approval <path> --write
```

Arguments:

- `--approval <repo-relative path>` required
- exactly one of `--check` or `--write`
- no `--run-id` in v1

Run directory:

- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-<agent_id>-runtime-follow-on/`
- example: `repair-aider-runtime-follow-on`
- `--write` replaces this deterministic repair directory atomically if it already exists

`--check` behavior:

1. require lifecycle stage `runtime_integrated`
2. require approval and registry continuity
3. derive committed runtime-owned writes from the repo, not from the stale runtime packet
4. fail if derived writes are empty
5. synthesize the exact frozen publication command set
6. validate the would-be bundle with the same semantic rules `prepare-publication` uses
7. print whether repair is needed and whether repair is possible

`--write` behavior:

1. run the same validations as `--check`
2. write these six files into the deterministic repair run directory:
   - `input-contract.json`
   - `run-status.json`
   - `validation-report.json`
   - `handoff.json`
   - `written-paths.json`
   - `run-summary.md`
3. write exact full publication commands, never short-form aliases
4. write non-empty `written-paths.json`
5. immediately re-read the written bundle through the shared validation path
6. exit non-zero if `prepare-publication --check` would still reject the repaired bundle
7. do not advance lifecycle stage

### Shared Helper Extraction

Add one shared helper module:

- new: `crates/xtask/src/runtime_evidence_bundle.rs`

This module owns:

- committed-output derivation for runtime-owned writes
- runtime evidence JSON writer helpers
- repair run directory naming
- `run-summary.md` rendering for reconstructed bundles
- semantic validation glue used by repair and historical backfill

Callers:

- `repair_runtime_evidence.rs` uses it directly
- `historical_lifecycle_backfill.rs` migrates to it
- `prepare_publication/runtime_evidence.rs` stays the read-side consumer

This avoids a second reconstruction system.

### Aider Repair Strategy

Repair `aider` by writing a new deterministic repair bundle, not by hand-editing the stale rerun directory.

Runtime-owned outputs to derive from committed repo state:

- `crates/aider/src/lib.rs`
- `crates/aider/src/wrapper_coverage_manifest.rs`
- `crates/agent_api/src/backends/aider/backend.rs`
- `crates/agent_api/src/backends/aider/harness.rs`
- `crates/agent_api/src/backends/aider/mapping.rs`
- `crates/agent_api/src/backends/aider/mod.rs`
- `crates/agent_api/tests/c1_aider_runtime_follow_on.rs`
- first file under `cli_manifests/aider/supplement/`
- first file under `cli_manifests/aider/snapshots/`

The repair command must derive these paths from approval and committed repo state, not from handwritten agent-specific code.

### Validation Hardening

Harden the seam in three places:

1. `runtime_follow_on/lifecycle.rs`
   - remove `LEGACY_REQUIRED_PUBLICATION_COMMANDS`
   - require exact equality with `agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS`
2. `prepare_publication/runtime_evidence.rs`
   - keep strict exact-match validation
   - improve error text to point operators at `repair-runtime-evidence` when stale bundle shape is detected
3. drift reporting
   - add a specific runtime-evidence drift finding for `runtime_integrated` agents whose committed bundle cannot pass publication consumption

## Code Quality Plan

### Module Ownership

| Area | Files | Responsibility |
| --- | --- | --- |
| command wiring | `crates/xtask/src/main.rs`, `crates/xtask/src/lib.rs` | expose the new subcommand cleanly |
| repair command | `crates/xtask/src/repair_runtime_evidence.rs` | CLI args, check/write flow, stdout |
| shared helper | `crates/xtask/src/runtime_evidence_bundle.rs` | bundle reconstruction, deterministic path derivation, bundle writing |
| forward runtime validation | `crates/xtask/src/runtime_follow_on/lifecycle.rs` | exact command set only |
| publication consumer | `crates/xtask/src/prepare_publication/runtime_evidence.rs` | consume repaired bundles without special cases |
| drift surfacing | `crates/xtask/src/agent_maintenance/drift/mod.rs`, `crates/xtask/src/agent_maintenance/drift/runtime_evidence.rs` | explicit stale-runtime-evidence finding |
| historical repair reuse | `crates/xtask/src/historical_lifecycle_backfill.rs` | call shared helper, stop owning private reconstruction logic |
| docs | `docs/cli-agent-onboarding-factory-operator-guide.md` | operator flow for repair |

### Code Quality Rules

1. No agent-specific branching for `aider`.
2. No second hardcoded command set. All writers and validators must use `agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS`.
3. No lifecycle stage mutation inside repair.
4. No workspace-wide diff or snapshot scan in repair. Use explicit path derivation from approval plus manifest child directories.
5. Keep the repair helper boring and explicit. This is state repair code. Cleverness is a bug source here.

## Detailed File Plan

### New Files

- `crates/xtask/src/repair_runtime_evidence.rs`
- `crates/xtask/src/runtime_evidence_bundle.rs`
- `crates/xtask/tests/repair_runtime_evidence_entrypoint.rs`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/input-contract.json`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/run-status.json`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/validation-report.json`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/handoff.json`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/written-paths.json`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/repair-aider-runtime-follow-on/run-summary.md`

### Updated Files

- `crates/xtask/src/main.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/historical_lifecycle_backfill.rs`
- `crates/xtask/src/runtime_follow_on/lifecycle.rs`
- `crates/xtask/src/prepare_publication/runtime_evidence.rs`
- `crates/xtask/src/agent_maintenance/drift/mod.rs`
- `crates/xtask/src/agent_maintenance/drift/runtime_evidence.rs`
- `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
- `crates/xtask/tests/prepare_publication_entrypoint.rs`
- `crates/xtask/tests/agent_maintenance_drift.rs`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

## Test Review

### Test Framework

This repo is Rust-first. The relevant test framework is the existing `cargo test -p xtask --test ...` integration harness under `crates/xtask/tests/`.

### Code Path Coverage Diagram

```text
RUNTIME EVIDENCE REPAIR
=======================
[+] xtask CLI surface
    ├── [GAP] repair-runtime-evidence help and arg validation
    └── [GAP] mutually exclusive --check / --write behavior

[+] repair-runtime-evidence --check
    ├── [GAP] rejects lifecycle stage != runtime_integrated
    ├── [GAP] rejects approval / registry / lifecycle continuity drift
    ├── [GAP] rejects empty derived written paths
    ├── [GAP] reports repair-needed when legacy short-form commands are present
    └── [GAP] reports repair-not-needed when bundle is already truthful

[+] repair-runtime-evidence --write
    ├── [GAP] writes all six runtime evidence files
    ├── [GAP] writes exact frozen command set
    ├── [GAP] writes non-empty written-paths.json
    ├── [GAP] remains lifecycle-stage neutral
    └── [GAP] repaired bundle is consumable by prepare-publication

[+] runtime-follow-on forward validation
    ├── [TESTED] rejects noop runtime execution
    └── [GAP] rejects legacy short-form handoff commands exactly

[+] prepare-publication consumption
    ├── [TESTED] rejects missing runtime evidence
    ├── [TESTED] rejects capability continuity drift
    └── [GAP] accepts repaired aider-style bundle end to end

[+] drift detection
    ├── [GAP] runtime_integrated stale evidence becomes explicit drift finding
    └── [GAP] clean repaired bundle no longer reports that finding
```

### Planned Tests

| Flow | Existing coverage | Required new coverage | Planned test file |
| --- | --- | --- | --- |
| CLI help and mode validation for repair command | none | yes | `crates/xtask/tests/repair_runtime_evidence_entrypoint.rs` |
| `--check` rejects non-`runtime_integrated` lifecycle | nearby coverage exists in runtime/publication entrypoint tests | yes | `repair_runtime_evidence_entrypoint.rs` |
| `--check` rejects empty derivation of committed runtime-owned writes | none | yes | `repair_runtime_evidence_entrypoint.rs` |
| `--check` detects legacy short-form command set as repair-needed | none | yes | `repair_runtime_evidence_entrypoint.rs` |
| `--write` emits full six-file bundle | none | yes | `repair_runtime_evidence_entrypoint.rs` |
| `--write` emits non-empty truthful written paths | none | yes | `repair_runtime_evidence_entrypoint.rs` |
| repaired bundle passes `prepare-publication --check` | no chained repair coverage | yes | `prepare_publication_entrypoint.rs` or `repair_runtime_evidence_entrypoint.rs` |
| runtime handoff validation rejects legacy short-form commands | current code still tolerates them in one path | yes | `runtime_follow_on_entrypoint.rs` |
| drift reports stale runtime evidence explicitly | no specific runtime-evidence drift coverage | yes | `agent_maintenance_drift.rs` |
| repaired `aider` artifacts stay consumable | live repo only | yes | one repository-level smoke command in verification section |

### Required Test Commands

```bash
cargo test -p xtask --test repair_runtime_evidence_entrypoint
cargo test -p xtask --test runtime_follow_on_entrypoint
cargo test -p xtask --test prepare_publication_entrypoint
cargo test -p xtask --test agent_maintenance_drift
make check
```

### Verification Commands

```bash
cargo run -p xtask -- repair-runtime-evidence --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --check
cargo run -p xtask -- repair-runtime-evidence --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --write
cargo run -p xtask -- prepare-publication --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --check
cargo run -p xtask -- check-agent-drift --agent aider
```

## Failure Modes Registry

| Failure mode | Detection | Handling | Test required | Critical gap today |
| --- | --- | --- | --- | --- |
| approval path or sha drift | repair and publication continuity validation | fail fast, no write | yes | no |
| lifecycle stage is not `runtime_integrated` | repair `--check` and `--write` | fail fast, no write | yes | no |
| committed runtime-owned outputs cannot produce a non-empty write set | repair derivation step | fail fast, no write | yes | yes |
| bundle still uses short-form commands | exact command validation | rewrite exact full commands, otherwise fail | yes | yes |
| bundle writes but still cannot pass publication consumption | post-write validation | fail command, keep lifecycle unchanged | yes | yes |
| drift remains invisible until publication | drift inspection | explicit runtime evidence finding | yes | yes |
| forward runtime path regresses and accepts legacy commands again | runtime handoff validation test | exact-match assertion in runtime tests | yes | yes |

Critical gap definition for this slice:

- any path that allows a `runtime_integrated` bundle to remain committed while `prepare-publication --check` would reject it is a critical gap

## Performance Review

This feature is small and repo-local. The real performance risk is accidental over-scanning.

Rules:

1. Repair derivation must stay bounded to known runtime-owned candidate paths plus `manifest_root/supplement` and `manifest_root/snapshots`.
2. Do not snapshot or hash the full workspace to reconstruct repair output.
3. Reuse existing JSON serializers and file writers. No extra process spawning inside repair except the human-run verification commands.
4. `prepare-publication` remains the only place that reasons about publication consumption. Repair should call the shared validator path, not shell out to a second `cargo run`.

Expected runtime cost is trivial compared to existing `xtask` integration tests.

## Implementation Steps

### Step 1 - Add shared runtime evidence helper

1. Create `crates/xtask/src/runtime_evidence_bundle.rs`.
2. Move committed-output derivation out of `historical_lifecycle_backfill.rs`.
3. Add shared writers for:
   - `input-contract.json`
   - `run-status.json`
   - `validation-report.json`
   - `handoff.json`
   - `written-paths.json`
   - `run-summary.md`
4. Keep the helper generic over approval, lifecycle state, run id, and host surface string.

### Step 2 - Add `repair-runtime-evidence`

1. Export the module from `crates/xtask/src/lib.rs`.
2. Wire the new subcommand in `crates/xtask/src/main.rs`.
3. Implement `Args`, `run`, and `run_in_workspace` in `crates/xtask/src/repair_runtime_evidence.rs`.
4. Add `--check` and `--write` flows exactly as specified above.

### Step 3 - Tighten forward validation

1. Remove `LEGACY_REQUIRED_PUBLICATION_COMMANDS` from `runtime_follow_on.rs` and `runtime_follow_on/lifecycle.rs`.
2. Make `validate_handoff` require exact command equality.
3. Improve repair guidance in publication-consumer errors when the bundle is stale.

### Step 4 - Add drift surfacing

1. Extend drift inspection for `runtime_integrated` agents.
2. If runtime evidence discovery or validation fails, emit an explicit runtime evidence finding that includes the repair command the operator should run.
3. Keep existing governance drift logic intact for other lifecycle states.

### Step 5 - Repair `aider`

1. Run the new repair command for `aider`.
2. Commit the deterministic repair bundle under `repair-aider-runtime-follow-on`.
3. Verify `prepare-publication --check` passes.
4. Leave the stale `aider-runtime-follow-on-rerun` directory untouched in this slice.

### Step 6 - Update docs

Update `docs/cli-agent-onboarding-factory-operator-guide.md` with:

- when to run `repair-runtime-evidence`
- expected failure message shape
- the `aider`-style stale-bundle scenario
- the sequence:
  - `repair-runtime-evidence --check`
  - `repair-runtime-evidence --write`
  - `prepare-publication --check`
  - `prepare-publication --write`

## Acceptance Criteria

1. `repair-runtime-evidence --check` fails clearly when repair is impossible.
2. `repair-runtime-evidence --write` emits a complete six-file bundle with non-empty written paths.
3. `repair-runtime-evidence --write` never advances lifecycle stage.
4. `prepare-publication --check` passes for `aider` after repair.
5. `runtime_follow_on/lifecycle.rs` no longer accepts legacy short-form command sets.
6. `check-agent-drift --agent aider` emits an explicit runtime-evidence finding before repair, and that specific finding disappears after repair even if non-runtime governance notes still remain until `prepare-publication --write`.
7. `historical_lifecycle_backfill.rs` uses the shared runtime evidence helper instead of owning private bundle reconstruction logic.

## Worktree Parallelization Strategy

This work has some parallelization opportunity, but only after the shared helper contract lands.

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| shared helper extraction | `crates/xtask/src/`, `crates/xtask/src/historical_lifecycle_backfill.rs` | — |
| repair command wiring | `crates/xtask/src/`, `crates/xtask/src/main.rs`, `crates/xtask/src/lib.rs` | shared helper extraction |
| forward validation hardening | `crates/xtask/src/runtime_follow_on/`, `crates/xtask/src/prepare_publication/` | shared helper extraction |
| drift surfacing | `crates/xtask/src/agent_maintenance/drift/` | shared helper extraction |
| tests | `crates/xtask/tests/` | repair command wiring, forward validation hardening, drift surfacing |
| aider artifact repair | `docs/agents/.uaa-temp/runtime-follow-on/runs/` | repair command wiring |
| docs update | `docs/` | repair command semantics stable |

### Parallel Lanes

- Lane A: shared helper extraction -> repair command wiring
  - sequential, shared `crates/xtask/src/`
- Lane B: forward validation hardening
  - can start after Lane A defines helper interfaces
- Lane C: drift surfacing
  - can start after Lane A defines helper interfaces
- Lane D: tests
  - starts after B and C, because tests need final behavior
- Lane E: aider artifact repair + operator docs
  - starts after A, but should finish after D so the committed artifacts reflect final behavior

### Execution Order

1. Launch Lane A first.
2. Once A lands, launch B and C in parallel.
3. Merge B and C.
4. Run D.
5. Run E last, then full verification.

### Conflict Flags

- Lanes A and B both touch `crates/xtask/src/`. They cannot safely run in parallel.
- Lanes B and C are mostly isolated.
- Lane D touches shared test harnesses and should stay sequential after behavior stabilizes.
- Lane E touches committed runtime artifacts and docs only, but it depends on the final semantics from A-D.

## NOT In Scope

- delete or rewrite `docs/agents/.uaa-temp/runtime-follow-on/runs/aider-runtime-follow-on-rerun/`
- change `prepare-publication` to accept legacy command aliases
- add a lifecycle field for “repaired by”
- make repair handle `closed_baseline` or `publication_ready` repair paths
- generalize repair into a multi-agent batch maintenance command
- update `TODOS.md` for stale-run cleanup

## What Already Exists

The new plan reuses these existing truths instead of rebuilding them:

| Need | Existing implementation | Plan action |
| --- | --- | --- |
| exact publication command contract | `agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS` | use everywhere |
| runtime bundle read-side validation | `prepare_publication/runtime_evidence.rs` | keep strict |
| forward rejection of empty runtime writes | `runtime_follow_on.rs` | keep and test |
| historical derivation of runtime-owned outputs | `historical_lifecycle_backfill.rs` | extract to shared helper |
| drift report rendering | `agent_maintenance/drift/mod.rs` | extend with explicit runtime evidence finding |
| integration test harnesses | existing `crates/xtask/tests/*_entrypoint.rs` | extend, do not replace |

## Completion Checklist

- [ ] `repair-runtime-evidence` command exists and is wired into `xtask`
- [ ] shared runtime evidence helper extracted and reused by historical backfill
- [ ] runtime handoff validation requires exact full publication commands
- [ ] drift inspection surfaces stale runtime evidence explicitly
- [ ] repaired `aider` runtime bundle committed under `repair-aider-runtime-follow-on`
- [ ] `prepare-publication --check` passes for `aider`
- [ ] targeted xtask integration tests added and passing
- [ ] operator guide updated

## Final Recommendation

Build the explicit repair seam. Do not patch `aider` by hand, and do not relax publication validation.

The repo already has the right model. What it lacks is the boring repair tool that turns old malformed runtime evidence back into truthful committed state and tells operators exactly what to do before they hit the next seam.
