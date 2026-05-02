<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/unified-agent-api/PLAN-autoplan-restore-20260502-151831.md -->
# PLAN - Generic Capability Publication Foundation

Status: planned  
Date: 2026-05-02  
Branch: `codex/recommend-next-agent`  
Base branch: `main`  
Repo: `atomize-hq/unified-agent-api`  
Work item: `Land The Generic Capability Publication Foundation`  
Plan commit baseline: `9daea9f`

Separate design doc: not required for this slice. This is a backend-only control-plane change,
and this `PLAN.md` is the design record.

## Objective

Make capability publication truthful, generic, and boring.

After this plan lands:

1. A newly onboarded agent that has reached `runtime_integrated` or later can participate in
   capability publication without any new hardcoded backend `match` arm in
   `crates/xtask/src/capability_matrix.rs`.
2. `capability-matrix`, `capability-matrix-audit`, `prepare-publication`,
   `check-agent-drift`, and `close-proving-run` all reason from the same publication truth model.
3. The repo has one pinned answer to "why is this agent allowed to publish these capability ids on
   this target?"
4. The capability-matrix specs and generated markdown stop describing publication as built-in
   backend inventory when the repo actually wants lifecycle-backed control-plane truth.

## Implementation Notes

- Capability publication must stop discovering truth from hardcoded runtime backend construction in
  `xtask`.
- The shared publication model must be metadata-driven. It must reuse committed registry,
  lifecycle, approval, and manifest artifacts that already exist in the repo.
- Lifecycle gates publication eligibility. Registry enrollment alone is not sufficient.
- The shared projection contract in `crates/xtask/src/capability_projection.rs` remains the only
  place that decides which capability ids are advertised for a target.
- This slice does not introduce plugin discovery, runtime reflection, or any new artifact family.

## Source Inputs

- Backlog source:
  - `TODOS.md`
  - `docs/backlog/cli-agent-onboarding-lifecycle-unification-gap-memo.md`
- Normative contracts:
  - `docs/specs/agent-registry-contract.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/cli-agent-onboarding-charter.md`
- Current implementation surfaces:
  - `crates/xtask/src/capability_matrix.rs`
  - `crates/xtask/src/capability_matrix_audit.rs`
  - `crates/xtask/src/capability_projection.rs`
  - `crates/xtask/src/prepare_publication.rs`
  - `crates/xtask/src/close_proving_run.rs`
  - `crates/xtask/src/agent_maintenance/drift/shared.rs`
  - `crates/xtask/src/agent_maintenance/drift/publication.rs`
  - `crates/xtask/data/agent_registry.toml`
- Current tests:
  - `crates/xtask/tests/c8_capability_matrix_unit.rs`
  - `crates/xtask/tests/c8_spec_capability_matrix_paths.rs`
  - `crates/xtask/tests/agent_maintenance_drift.rs`
  - `crates/xtask/tests/prepare_publication_entrypoint.rs`
  - `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`

## Verified Current State

These facts are verified from the current branch, not inferred:

1. `crates/xtask/src/capability_matrix.rs` imports concrete backend types from `agent_api` and
   hardcodes runtime inventory in `runtime_backend_capabilities(agent_id)`.
2. `collect_builtin_backend_inventory()` and
   `collect_builtin_backend_inventory_from_registry(...)` still combine registry entries with
   built-in backend constructor truth before rendering markdown.
3. `validate_agent_publication_continuity(...)` in `capability_matrix.rs` validates manifest and
   projection continuity today, but it still depends on modeled runtime truth derived from the same
   hardcoded backend inventory path.
4. `crates/xtask/src/capability_matrix_audit.rs` calls
   `crate::capability_matrix::collect_builtin_backend_capabilities()` and therefore inherits the
   same hardcoded inventory boundary.
5. `crates/xtask/src/close_proving_run.rs` duplicates the audit allowlist and audit logic locally
   in `validate_capability_matrix_audit_green()` instead of reusing the CLI audit implementation.
6. `crates/xtask/src/prepare_publication.rs` currently calls
   `capability_matrix::validate_agent_publication_continuity(...)`, so the publication continuity
   gate is still anchored to the generator module instead of a dedicated shared source.
7. `crates/xtask/src/agent_maintenance/drift/shared.rs` already derives capability truth from
   registry declaration plus manifest projection via `project_advertised_capabilities(...)`, which
   is closer to the desired model than the generator path.
8. `docs/specs/unified-agent-api/capability-matrix.md` still publishes the header
   "`opencode`, `gemini_cli`, `aider` use the default built-in backend config", which is the
   wrong control-plane explanation for a lifecycle-backed publication lane.

## Problem Statement

Capability publication has split brain.

The repo already has the right control-plane ingredients:

- registry-owned capability declarations
- lifecycle-owned eligibility and continuity evidence
- lifecycle-linked approval descriptors
- manifest-root target availability
- shared projection logic for advertised capabilities

But the publication lane still asks a second, wrong question:

- "Can `xtask` instantiate this backend from a hardcoded built-in list?"

That creates three concrete failures:

1. A newly enrolled agent still needs hidden Rust edits before publication becomes truthful.
2. Different consumers reason about capability truth differently:
   - generator and closeout gate use compiled backend inventory
   - drift already leans on registry plus manifest projection
3. The specs describe built-in backend semantics even though the onboarding lifecycle wants
   publication to follow committed lifecycle and approval artifacts.

## Step 0 Scope Challenge

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| capability declaration ownership | `crates/xtask/data/agent_registry.toml`, `docs/specs/agent-registry-contract.md` | Reuse directly. Registry remains the committed declaration source. |
| frozen onboarding declaration | `approved-agent.toml`, `crates/xtask/src/approval_artifact.rs` | Reuse directly. Publication truth must validate registry against this frozen descriptor. |
| lifecycle eligibility | `crates/xtask/src/agent_lifecycle.rs` | Reuse directly. Lifecycle stage decides whether an agent can enter publication inventory. |
| target-scoped capability projection | `crates/xtask/src/capability_projection.rs` | Reuse directly. Do not rebuild projection rules in consumers. |
| per-agent manifest availability | `cli_manifests/<agent>/current.json` | Reuse directly. This remains the manifest availability input. |
| drift reporting framework | `crates/xtask/src/agent_maintenance/drift/*` | Reuse, but point it at the same shared publication source used by generation and closeout. |
| closeout gate wiring | `crates/xtask/src/close_proving_run.rs` | Reuse, but remove the duplicated audit semantics. |
| capability matrix rendering | `crates/xtask/src/capability_matrix.rs` | Reuse render and check/write flow. Replace only the inventory builder and header semantics. |

### Minimum Complete Change Set

The smallest complete version of this milestone is:

1. add one shared publication-capability source module
2. move publication continuity validation into that module
3. make generator inventory come from that module
4. make `capability-matrix-audit` call a shared audit over that same inventory
5. make `close-proving-run` call that same shared audit instead of cloning it
6. make drift capability truth and `prepare-publication` reuse the same source module
7. update specs and generated wording so the repo no longer describes the old built-in-only model

Anything smaller leaves split truth in place.

### Complexity Check

This plan touches more than 8 files. That is still the minimal complete version because the bug
spans:

- shared inventory construction
- generator
- audit
- closeout gate
- prepare-publication continuity
- drift truth
- specs and generated wording
- tests

Complexity control for this slice:

- one new source module only
- no new lifecycle stage
- no new artifact type
- no new plugin or reflection system
- no support-matrix refactor

### Search / Build Decision

This is a Layer 1 reuse problem, not a new architecture problem.

- **[Layer 1]** Reuse `approval_artifact.rs` as the frozen capability declaration surface.
- **[Layer 1]** Reuse `agent_lifecycle.rs` for publication eligibility and lifecycle lookup.
- **[Layer 1]** Reuse `capability_projection.rs` for advertised capability derivation.
- **[Layer 1]** Reuse `cli_manifests/<agent>/current.json` as the target/command availability
  surface instead of inventing a second target registry.
- **[Layer 3]** Stop asking compiled backend constructors for publication truth. Lifecycle,
  approval, registry, and manifest artifacts are already the right control-plane abstraction.

### TODOS Cross-Reference

This plan closes exactly one pending TODO:

- `Land The Generic Capability Publication Foundation`

It explicitly unblocks, but does not implement:

- `Enclose The Publication Lane End To End`
- `Make The Published State Honest In The Lifecycle Model`
- `Decide Whether Capability Matrix Markdown Stays Canonical After M5`

### Completeness Decision

The shortcut version would keep the hardcoded runtime backend inventory and just spread a slightly
cleaner helper around the same false model. That is not acceptable.

The complete version is still a boilable lake:

- one metadata-driven source of publication truth
- one shared audit implementation
- one lifecycle-based eligibility rule
- one set of tests that prove the generic lane works

### Distribution Check

No new binary, package, container, or artifact family is introduced here. Distribution pipeline
changes are not part of this slice.

## Locked Decisions

These decisions are locked for this plan.

1. Add one shared source module:
   - `crates/xtask/src/capability_publication.rs`
2. Publication eligibility is lifecycle-driven. Include an agent only when:
   - `publication.capability_matrix_enabled = true`
   - lifecycle state exists for `scaffold.onboarding_pack_prefix`
   - lifecycle stage is one of:
     - `runtime_integrated`
     - `publication_ready`
     - `published`
     - `closed_baseline`
3. Capability declaration truth for publication comes from the lifecycle-linked approval artifact,
   but current registry truth must still match it exactly. Any drift is a validation failure.
4. Advertised capability projection remains owned by the existing projection contract:
   - `crates/xtask/src/capability_projection.rs`
   - `cli_manifests/<agent>/current.json`
5. `capability-matrix-audit` and `close-proving-run` must call one shared audit function over one
   shared inventory. No duplicated allowlists.
6. `capability-matrix` generation must omit pre-runtime agents instead of failing on them.
7. This slice does not add runtime plugin loading, backend reflection, or dynamic factory
   discovery. The generic source is metadata-driven, not constructor-driven.

## Architecture Review

### Current Split-Brain Flow

```text
agent_registry.toml
        +
cli_manifests/<agent>/current.json
        |
        +--> capability_projection.rs
        |       |
        |       +--> drift modeled truth
        |
        +--> capability_matrix.rs
                |
                +--> runtime_backend_capabilities(agent_id)
                        |
                        +--> hardcoded built-in backend inventory
                                |
                                +--> capability-matrix markdown
                                +--> capability-matrix-audit
                                +--> prepare-publication continuity
                                +--> close-proving-run audit clone
```

What is broken:

- drift truth is already artifact-driven
- generator, audit, continuity, and closeout are still constructor-driven
- new agent onboarding has to satisfy both worlds

### Target Architecture

```text
agent_registry.toml
        +
lifecycle-state.json
        +
approved-agent.toml
        +
cli_manifests/<agent>/current.json
        |
        v
capability_publication.rs
  - load eligible entries
  - resolve lifecycle state
  - validate approval <-> registry continuity
  - resolve publication target
  - project advertised capabilities
  - build shared publication inventory
  - audit orthogonality over that inventory
        |
        +--> capability-matrix render/check
        +--> capability-matrix-audit
        +--> prepare-publication continuity
        +--> check-agent-drift capability truth
        +--> close-proving-run gate
```

### Shared Source Contract

`capability_publication.rs` becomes the only place allowed to answer:

- is this agent publication-eligible right now?
- which target is publication truth scoped to?
- which capability ids are advertised for publication?
- does registry truth still match the frozen approval descriptor?
- does the union manifest satisfy the publication target contract?

Recommended core types:

```text
PublicationCapabilityRecord
  - agent_id
  - display_name
  - backend_module
  - manifest_root
  - onboarding_pack_prefix
  - lifecycle_stage
  - approval_artifact_path
  - approval_artifact_sha256
  - publication_target
  - canonical_targets
  - advertised_capability_ids

PublicationInventory
  - records[]
  - header_profiles[]

CapabilityAuditViolation
  - capability_id
  - supported_by[]
```

Recommended public entrypoints:

1. `collect_publication_inventory(workspace_root) -> Result<PublicationInventory, String>`
2. `collect_publication_capabilities(workspace_root) -> Result<BTreeMap<String, AgentWrapperCapabilities>, String>`
3. `validate_agent_publication_continuity(workspace_root, entry) -> Result<(), String>`
4. `audit_publication_capabilities(inventory) -> Result<(), String>`

The generator can keep its current render model if the shared module returns
`BTreeMap<String, AgentWrapperCapabilities>`, but the source of that map must move out of
`capability_matrix.rs`.

### Required Validation Pipeline

For each `registry.capability_matrix_entries()` candidate:

1. Read the registry entry.
2. Resolve the lifecycle-state path from `scaffold.onboarding_pack_prefix`.
3. If lifecycle state is missing, fail validation for direct continuity checks and skip the entry
   when building generated publication inventory.
4. If lifecycle stage is earlier than `runtime_integrated`, exclude the entry from publication
   inventory without error.
5. Load the lifecycle-linked `approved-agent.toml`.
6. Require exact continuity for:
   - `agent_id`
   - `display_name`
   - `backend_module`
   - `manifest_root`
   - capability declarations
   - publication flags
   - `capability_matrix_target`
7. Load `cli_manifests/<agent>/current.json`.
8. Resolve publication target with `resolve_capability_publication_target(entry)`.
9. Derive advertised capability ids only through `project_advertised_capabilities(...)`.
10. Emit one shared record used by all publication consumers.

### Consumer Migration Rules

#### `capability_matrix.rs`

- Keep CLI args, output path handling, stale-check flow, bucketing, and markdown rendering here.
- Delete the concrete backend imports from `agent_api`.
- Delete `runtime_backend_capabilities(...)`.
- Delete `collect_builtin_backend_inventory()` and replace it with a call into the shared module.
- Replace header text so it explains lifecycle-backed publication targets, not built-in config.

#### `capability_matrix_audit.rs`

- Keep only the CLI entrypoint and report rendering here.
- Move the orthogonality allowlist out of this file.
- Replace direct calls to `collect_builtin_backend_capabilities()` with the shared audit function.

#### `prepare_publication.rs`

- Keep publication packet and lifecycle transition logic untouched.
- Replace the `capability_matrix::validate_agent_publication_continuity(...)` dependency with
  `capability_publication::validate_agent_publication_continuity(...)`.
- Do not duplicate capability projection or continuity rules here.

#### `agent_maintenance/drift/shared.rs` and `publication.rs`

- Keep support-matrix logic unchanged.
- Replace local capability truth derivation with the same publication record or shared capability
  set used by the generator.
- Drift must compare published markdown against the exact same modeled truth as generation.

#### `close_proving_run.rs`

- Delete the local orthogonality allowlist constant.
- Delete local `supported_backends(...)` audit cloning.
- Replace `validate_capability_matrix_audit_green()` internals with a call into the shared audit.
- Keep closeout write flow and docs regeneration untouched.

### Architecture Invariants

These are non-negotiable:

1. No agent-name `match` is allowed anywhere in publication truth derivation.
2. No consumer may restate capability projection, lifecycle eligibility, or approval continuity
   independently.
3. Pre-runtime agents must be invisible to generated capability publication.
4. A publication-eligible agent must be able to enter the matrix with no code changes outside
   registry, lifecycle, approval, manifest, and tests.
5. Generated docs may explain publication target shape, but they may not claim built-in backend
   construction is the source of truth.

### Inline Diagram Targets

If implementation adds or restructures these files, keep local ASCII diagrams accurate:

- `crates/xtask/src/capability_publication.rs`
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/close_proving_run.rs`

## Code Quality Review

### Module Ownership

| Area | Files | Responsibility |
| --- | --- | --- |
| shared publication source | `crates/xtask/src/capability_publication.rs` | eligibility, continuity, inventory construction, audit entrypoint |
| capability-matrix render entrypoint | `crates/xtask/src/capability_matrix.rs` | CLI args, render, check/write path handling |
| semantic audit entrypoint | `crates/xtask/src/capability_matrix_audit.rs` | CLI surface only, no truth derivation |
| publication continuity | `crates/xtask/src/prepare_publication.rs` | validate one agent's readiness through the shared source |
| drift capability truth | `crates/xtask/src/agent_maintenance/drift/shared.rs`, `publication.rs` | compare published matrix against shared modeled truth |
| closeout gate | `crates/xtask/src/close_proving_run.rs` | reuse shared audit result, stop cloning audit semantics |
| docs/specs | `docs/specs/**`, `docs/cli-agent-onboarding-factory-operator-guide.md` | explain lifecycle-backed publication truth clearly |

### Code Quality Rules

1. Rendering stays dumb. Truth lives in `capability_publication.rs`, not in render or CLI files.
2. `capability-matrix-audit` and `close-proving-run` must share one allowlist constant and one
   audit implementation.
3. `prepare-publication` must not depend on the generator module for publication truth.
4. Drift must not model capabilities independently once the shared source exists.
5. No runtime backend constructors may remain in publication logic after this slice.
6. Keep the diff small: one new module, surgical rewires, no new abstraction stack beyond the
   shared source.

## Detailed File Plan

### New File

- `crates/xtask/src/capability_publication.rs`

### Updated Files

- `crates/xtask/src/lib.rs`
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/capability_matrix_audit.rs`
- `crates/xtask/src/prepare_publication.rs`
- `crates/xtask/src/close_proving_run.rs`
- `crates/xtask/src/agent_maintenance/drift/shared.rs`
- `crates/xtask/src/agent_maintenance/drift/publication.rs`
- `crates/xtask/tests/c8_capability_matrix_unit.rs`
- `crates/xtask/tests/c8_spec_capability_matrix_paths.rs`
- `crates/xtask/tests/agent_maintenance_drift.rs`
- `crates/xtask/tests/prepare_publication_entrypoint.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`
- `crates/xtask/tests/support/agent_maintenance_drift_harness.rs`
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/specs/unified-agent-api/README.md`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/specs/unified-agent-api/capability-matrix.md`

## Implementation Steps

### Step 1 - Add the shared publication source

1. Create `crates/xtask/src/capability_publication.rs`.
2. Move these responsibilities into it:
   - lifecycle eligibility lookup
   - lifecycle-linked approval loading
   - approval-to-registry continuity validation
   - publication target resolution
   - manifest projection
   - shared inventory collection
   - orthogonality audit over that inventory
3. Export the module from `crates/xtask/src/lib.rs`.

Definition of done:

- the new module can build publication records without importing any concrete backend type
- the module exposes a reusable audit path
- the module has unit tests for eligibility and continuity edge cases

### Step 2 - Migrate capability-matrix generation

1. Remove concrete `agent_api` backend imports from `capability_matrix.rs`.
2. Remove `runtime_backend_capabilities(...)`.
3. Replace `collect_builtin_backend_inventory()` with a call into the shared module.
4. Keep `render_matrix(...)`, bucket logic, and file output flow in place.
5. Update the generated header text so it describes lifecycle-backed publication targets instead of
   "default built-in backend config".

Definition of done:

- `cargo run -p xtask -- capability-matrix --check` passes using only shared publication truth
- no publication inventory path in `capability_matrix.rs` instantiates a backend

### Step 3 - Centralize the semantic audit

1. Move the orthogonality allowlist into the shared module.
2. Expose one shared `audit_publication_capabilities(...)` entrypoint.
3. Make `capability_matrix_audit.rs` call it.
4. Make `close_proving_run.rs` call it instead of its local duplicate.

Definition of done:

- `capability-matrix-audit` and `close-proving-run` fail on the same violation set
- there is exactly one allowlist definition in the codebase for this audit

### Step 4 - Align prepare-publication and drift

1. Replace `prepare_publication` capability continuity wiring with the shared module.
2. Replace drift capability truth derivation with the shared module.
3. Keep support-matrix and runtime-evidence logic untouched.

Definition of done:

- drift, continuity validation, and generation all talk about the same published capability set
- no publication consumer in scope computes capability truth independently

### Step 5 - Update tests to prove generic behavior

1. Add a synthetic publication-eligible agent fixture by extending seeded registry, lifecycle,
   approval, and manifest inputs inside test harnesses.
2. Assert the synthetic agent appears in capability publication without editing any hardcoded
   backend inventory list.
3. Assert the same synthetic agent is excluded before `runtime_integrated`.
4. Assert closeout and drift consume the same truth.

Definition of done:

- each regression in the test review section is covered
- the synthetic fixture proves generic publication behavior instead of another built-in case

### Step 6 - Update specs and operator docs

1. Rewrite capability-matrix semantics in `capabilities-schema-spec.md`.
2. Update `docs/specs/unified-agent-api/README.md`.
3. Update `docs/specs/cli-agent-onboarding-charter.md` wherever it still frames capability
   publication as built-in-only.
4. Update `docs/cli-agent-onboarding-factory-operator-guide.md` so the create lane says
   publication eligibility is lifecycle-backed.
5. Regenerate `docs/specs/unified-agent-api/capability-matrix.md`.

Definition of done:

- docs and generated markdown describe the same truth model the code enforces
- no spec in scope claims backend constructor inventory is canonical

## Test Review

### Test Framework

This repo is Rust-first. The relevant coverage lives in `cargo test -p xtask --test ...`
integration tests plus the capability unit/spec tests under `crates/xtask/tests/`.

### Code Path Coverage Diagram

```text
GENERIC CAPABILITY PUBLICATION
==============================
[+] capability_publication.rs
    ├── [GAP] lifecycle gate excludes enrolled / approved agents
    ├── [GAP] runtime_integrated agent is included without hardcoded match arm
    ├── [GAP] approval <-> registry capability drift fails fast
    ├── [GAP] manifest missing required publication target fails fast
    ├── [GAP] publication target resolves through capability_projection.rs only
    └── [GAP] shared audit reports orthogonality violations over shared inventory

[+] capability-matrix
    ├── [TESTED] render path / output path handling
    ├── [GAP] inventory comes from shared source, not runtime_backend_capabilities()
    └── [GAP] header text reflects lifecycle-backed publication semantics

[+] capability-matrix-audit
    ├── [TESTED] orthogonality semantics
    └── [GAP] uses shared audit implementation and shared inventory

[+] prepare-publication
    ├── [TESTED] approval / lifecycle continuity checks
    └── [GAP] capability publication continuity comes from shared source module

[+] check-agent-drift
    ├── [TESTED] published capability mismatch detection
    └── [GAP] compares published markdown against the same shared modeled truth as generation

[+] close-proving-run
    ├── [TESTED] publication gate path exists
    └── [GAP] reuses shared audit instead of local clone
```

### Required Tests

| Flow | Existing coverage | Required new coverage | Planned test file |
| --- | --- | --- | --- |
| synthetic runtime-integrated agent publishes without a new `xtask` match arm | none | yes | `crates/xtask/tests/c8_capability_matrix_unit.rs` |
| pre-runtime enrolled agent is excluded from generated inventory | none | yes | `c8_capability_matrix_unit.rs` |
| approval capability declaration drift fails shared source construction | none | yes | `c8_capability_matrix_unit.rs` |
| lifecycle-linked approval path mismatch fails continuity validation | none | yes | `c8_capability_matrix_unit.rs` |
| `capability-matrix-audit` uses shared audit and shared inventory | partial | yes | `c8_capability_matrix_unit.rs` or `c8_spec_capability_matrix_paths.rs` |
| `prepare-publication` continuity uses shared source | partial | yes | `prepare_publication_entrypoint.rs` |
| drift truth matches generator truth | partial | yes | `agent_maintenance_drift.rs` |
| `close-proving-run` audit gate reuses shared logic | partial | yes | `onboard_agent_closeout_preview/close_proving_run_write.rs` |

### Regression Rules

These regressions are mandatory to cover:

- adding a new publication-eligible agent must not require editing a hardcoded backend list
- changing capability publication semantics in one consumer must not silently diverge from the
  other consumers
- pre-runtime agents must not start appearing in the generated matrix
- approval, registry, and manifest drift must fail at the shared continuity boundary before any
  consumer can publish contradictory truth

### Required Test Commands

```bash
cargo test -p xtask --test c8_capability_matrix_unit
cargo test -p xtask --test c8_spec_capability_matrix_paths
cargo test -p xtask --test prepare_publication_entrypoint
cargo test -p xtask --test agent_maintenance_drift
cargo test -p xtask --test onboard_agent_closeout_preview
make check
```

### Verification Commands

```bash
cargo run -p xtask -- capability-matrix --check
cargo run -p xtask -- capability-matrix-audit
cargo run -p xtask -- check-agent-drift --agent codex
cargo run -p xtask -- prepare-publication --approval docs/agents/lifecycle/codex-cli-onboarding/governance/approved-agent.toml --check
```

## Failure Modes Registry

| Failure mode | Detection | Handling | Test required | Critical gap today |
| --- | --- | --- | --- | --- |
| newly enrolled runtime-integrated agent is omitted until `xtask` code is hand-edited | synthetic publication fixture | fail current branch, then fix shared source | yes | yes |
| pre-runtime agent appears in matrix too early | lifecycle-gating test | exclude from publication inventory | yes | yes |
| registry and frozen approval capability declarations drift | shared source continuity validation | fail fast, no publication truth | yes | yes |
| lifecycle state exists, but approval path does not match that pack prefix | shared continuity validation | fail fast, no publication truth | yes | yes |
| `current.json` misses the required publication target | shared continuity validation | fail fast, no publication truth | yes | no |
| drift compares against a different truth source than generator | drift parity test | route drift through shared source | yes | yes |
| closeout passes while CLI audit would fail | shared audit reuse test | remove duplicate audit code | yes | yes |
| spec wording still claims built-in-inventory semantics after code changes | doc diff + review | update spec in the same change | yes | yes |

Critical gap definition for this slice:

- any state where one publication consumer says an agent/capability set is valid and another says
  it is invalid, without a real artifact change

## Performance Review

Performance is not the product risk here. Consistency is.

Still, the shared source must stay bounded:

1. one registry scan over `capability_matrix_entries()` per command
2. one lifecycle load, one approval load, and one `current.json` load per eligible candidate
3. no runtime backend constructor instantiation
4. no shelling out between publication consumers

Expected runtime cost is trivial relative to current `xtask` integration tests.

## Worktree Parallelization Strategy

This plan has useful parallelization only after the shared source API is frozen. Before that,
everything is stacked on the same seam.

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| shared publication source | `crates/xtask/src/` | — |
| generator migration | `crates/xtask/src/` | shared publication source |
| audit + closeout reuse | `crates/xtask/src/`, `crates/xtask/tests/onboard_agent_closeout_preview/` | shared publication source |
| drift + prepare-publication alignment | `crates/xtask/src/agent_maintenance/drift/`, `crates/xtask/src/prepare_publication.rs` | shared publication source |
| docs/spec updates | `docs/specs/`, `docs/` | shared publication source semantics stable |
| tests and final verification | `crates/xtask/tests/` | generator migration, audit + closeout reuse, drift + prepare-publication alignment |

### Parallel Lanes

- Lane A: shared source module -> generator migration
  - sequential, shared `crates/xtask/src/`
- Lane B: audit + closeout reuse
  - starts after Lane A freezes the shared API
- Lane C: drift + prepare-publication alignment
  - starts after Lane A freezes the shared API
- Lane D: docs/spec updates
  - can start after Lane A, but should finish after behavior is locked
- Lane E: tests and verification
  - starts after B and C merge because it exercises final semantics

### Execution Order

1. Launch Lane A first.
2. Once the shared source API is stable, launch Lanes B, C, and D in parallel.
3. Merge B and C before starting Lane E.
4. Run Lane E and the verification commands.
5. Regenerate and re-check docs last.

### Conflict Flags

- Lanes A and B both touch `crates/xtask/src/`. Do not run them in parallel.
- Lanes A and C both touch shared source imports and signatures. Do not run them in parallel.
- Lanes B and C are the best true parallel split.
- Lane D will churn if semantics are still moving. Start it only after Lane A decisions are frozen.

## NOT In Scope

- new publication write commands
- support-matrix refactors
- published-stage lifecycle redesign
- backend plugin discovery
- dynamic runtime reflection
- changing the orthogonality rule itself
- changing capability declarations for existing agents

## Acceptance Criteria

1. `cargo run -p xtask -- capability-matrix --check` succeeds without any hardcoded
   `runtime_backend_capabilities()` agent inventory.
2. `cargo run -p xtask -- capability-matrix-audit` and `close-proving-run` use the same audit
   implementation.
3. A synthetic publication-eligible agent fixture can enter capability publication without adding a
   new `xtask` backend `match` arm.
4. A synthetic pre-runtime agent fixture is excluded from publication output.
5. `check-agent-drift` compares published capability truth against the same shared modeled truth as
   the generator.
6. The capability-matrix spec and generated header no longer describe the publication surface as a
   built-in backend inventory.
7. No consumer in scope restates publication-capability derivation independently.

## What Success Looks Like

When this lands, adding the next agent to capability publication should look like this:

1. registry entry is enrolled with `capability_matrix_enabled = true`
2. lifecycle reaches `runtime_integrated`
3. approval artifact and `current.json` are present and internally consistent
4. shared publication source picks the agent up automatically
5. generator, audit, drift, prepare-publication, and closeout all agree on the result

No hidden Rust edit. No second truth system. No surprise audit failure from a different model.

## TODO Relation

`TODOS.md` does not need a new item for this slice.

This plan is the implementation plan for:

- `Land The Generic Capability Publication Foundation`

The remaining pending TODOs stay deferred as written.

## Completion Summary

- Step 0: Scope Challenge — scope accepted as-is; this is the minimum complete slice
- Architecture Review: one shared source module, five publication consumers rewired
- Code Quality Review: duplicate truth derivation removed from generator, audit, closeout, and drift
- Test Review: coverage diagram produced, 12 concrete gaps enumerated
- Performance Review: bounded metadata reads, zero runtime backend construction
- NOT in scope: written
- What already exists: written
- Failure modes: 8 failure modes listed, 7 critical gaps flagged on the current branch
- Parallelization: 5 lanes total, 3 launchable after Lane A, 2 strictly sequential gates
- Lake Score: complete option chosen for every major decision in this slice
