---
slice_id: S2
seam_id: SEAM-1
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - manifest-root artifact inventory drifts from current repo norms
    - root validation posture stops matching the committed OpenCode artifact classes
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-05
contracts_produced:
  - C-02
contracts_consumed: []
open_remediations: []
---
### S2 - manifest-root-artifacts-and-validator-scope

- **User/system value**: establish a real `cli_manifests/opencode/` truth store so OpenCode
  support evidence lands in one auditable root instead of mixed planning notes or ad hoc outputs.
- **Scope (in/out)**:
  - In: committed root inventory, pointer files, schemas, rules, version metadata posture,
    snapshots, reports, wrapper coverage, current snapshot, and OpenCode-specific validator support
  - Out: backend support publication, UAA promotion, and generic validator abstractions for future
    agents
- **Acceptance criteria**:
  - `cli_manifests/opencode/` contains the committed artifact classes required by the manifest
    contract
  - `cargo run -p xtask -- codex-validate --root cli_manifests/opencode` can validate the root
  - validator changes stay bounded to OpenCode root needs
- **Dependencies**:
  - `docs/specs/opencode-cli-manifest-contract.md`
  - current manifest roots under `cli_manifests/**`
  - current root-validation behavior under `crates/xtask/**`
- **Verification**:
  - compare the landed root inventory against current codex and claude manifest roots
  - validate pointer, schema, rules, report, and current snapshot posture mechanically
  - confirm manifest evidence remains distinct from backend support or unified support claims
- **Rollout/safety**:
  - preserve one root-local truth store
  - keep validator work root-specific and deterministic
  - avoid adding future-agent template machinery
- **Review surface refs**:
  - `review.md#r1---foundation-handoff-flow`
  - `review.md#r2---repo-implementation-boundary`

#### S2.T1 - Materialize the OpenCode manifest root inventory

- **Outcome**: the repo has a committed `cli_manifests/opencode/` root with explicit inventory,
  pointers, reports, snapshots, and wrapper coverage artifacts.
- **Inputs/outputs**:
  - Inputs: manifest contract, current manifest-root patterns, S00 baseline
  - Outputs: `cli_manifests/opencode/**`
- **Thread/contract refs**: `THR-05`, `C-02`
- **Implementation notes**:
  - keep `current.json`, pointer files, report artifacts, and coverage declarations explicit
  - keep any raw or debug-only captures non-authoritative
- **Acceptance criteria**:
  - the committed root can answer the inventory and validation questions named by the contract
  - artifact ownership stays clear enough for later support publication work
- **Test notes**:
  - compare generated or committed files against existing manifest-root structures
- **Risk/rollback notes**:
  - if the inventory is incomplete, downstream publication will misread what OpenCode actually supports

#### S2.T2 - Land bounded OpenCode root validation support

- **Outcome**: root validation recognizes the OpenCode manifest root without widening into generic
  future-agent scaffolding.
- **Inputs/outputs**:
  - Inputs: existing `xtask` validator code, OpenCode manifest contract, landed root inventory
  - Outputs: bounded OpenCode-specific validation behavior under `crates/xtask/**`
- **Thread/contract refs**: `THR-05`, `C-02`
- **Implementation notes**:
  - keep validation tied to this root's schema, rules, and artifact inventory
  - avoid introducing reusable abstractions unless existing repo patterns already require them
- **Acceptance criteria**:
  - `codex-validate --root cli_manifests/opencode` succeeds once the root inventory is present
  - validator semantics still distinguish manifest support from backend or unified support
- **Test notes**:
  - run the root validator against the OpenCode root and compare failure modes with current roots
- **Risk/rollback notes**:
  - if validation becomes generic scaffolding, narrow it back to OpenCode-specific needs

Checklist:
- Implement: materialize the OpenCode manifest root and bounded validator support
- Test: validate the root inventory mechanically with the existing `xtask` flow
- Validate: confirm the root can serve as the authoritative manifest-support evidence surface
