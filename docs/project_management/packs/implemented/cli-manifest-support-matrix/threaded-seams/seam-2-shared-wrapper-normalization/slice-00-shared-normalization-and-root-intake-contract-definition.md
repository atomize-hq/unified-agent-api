---
slice_id: S00
seam_id: SEAM-2
slice_kind: contract_definition
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - shared-vs-adapter ownership shifts
    - root-intake input shapes change
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
contracts_produced:
  - C-02
  - C-03
contracts_consumed:
  - C-01
open_remediations: []
---
### S00 - Shared normalization and root-intake contract definition

- **User/system value**: downstream seams get one execution-grade definition of the shared module boundary and neutral root-intake shape before the repo starts moving duplicated logic.
- **Scope (in/out)**:
  - In: define shared-vs-adapter ownership, the neutral root-intake inputs, and the verification checklist for `C-02` and `C-03`.
  - Out: actual helper extraction, adapter rewrites, and publication/rendering work.
- **Acceptance criteria**:
  - the contract names `crates/xtask/src/wrapper_coverage_shared.rs` as the shared normalization home.
  - the contract lists versions, pointers, current metadata, and coverage reports as the root-intake inputs that future support-matrix work will consume.
  - the contract states that shared code is shape-driven and future-agent-shaped rather than agent-name-driven.
  - the contract leaves publication semantics owned by `SEAM-1` and publication/rendering owned by `SEAM-3`.
- **Dependencies**:
  - `../../governance/seam-1-closeout.md`
  - `crates/xtask/src/codex_wrapper_coverage.rs`
  - `crates/xtask/src/claude_wrapper_coverage.rs`
  - `docs/specs/unified-agent-api/support-matrix.md`
- **Verification**:
  - compare the contract baseline against the current duplicated helpers and confirm the shared-vs-adapter split is concrete enough to implement
  - confirm the root-intake list is explicit enough for `SEAM-3` to consume without reopening path or ownership decisions
  - confirm the contract does not reopen `SEAM-1` support semantics
- **Rollout/safety**:
  - planning and contract-definition only
  - no behavior change hidden in this slice
- **Review surface refs**:
  - `review.md#likely-mismatch-hotspots`
  - `../../threading.md`

#### S00.T1 - Freeze the shared-vs-adapter boundary

- **Outcome**: one contract states exactly what belongs in the shared module and what remains in root-specific adapters.
- **Inputs/outputs**:
  - Inputs: current Codex and Claude wrapper-coverage modules plus `SEAM-1` closeout evidence
  - Outputs: contract-definition updates embodied in seam-local planning
- **Thread/contract refs**: `THR-01`, `THR-02`, `C-02`
- **Implementation notes**: make the boundary explicit enough that extraction does not drift into publication or validator ownership.
- **Acceptance criteria**: a reviewer can point to each duplicated helper and tell whether it moves into the shared seam or stays in an adapter.
- **Test notes**: compare the final boundary against the duplicated helper set currently present in both wrapper-coverage modules.
- **Risk/rollback notes**: a fuzzy boundary will leak agent-name assumptions into the shared core.

#### S00.T2 - Lock the neutral root-intake contract

- **Outcome**: the seam records the root-shaped inputs future support-matrix work will consume from each agent root.
- **Inputs/outputs**:
  - Inputs: manifest roots, `RULES.json`, versions metadata, pointer files, current snapshots, and coverage reports
  - Outputs: contract-definition notes for `C-03`
- **Thread/contract refs**: `THR-01`, `THR-02`, `C-03`
- **Implementation notes**: keep the intake contract shape-driven; do not invent a generic framework or future renderer APIs here.
- **Acceptance criteria**: the root-intake list is concrete enough for downstream derivation planning and thin-adapter adoption.
- **Test notes**: verify the listed inputs map to real repo surfaces under both current manifest roots.
- **Risk/rollback notes**: ambiguous intake rules will force `SEAM-3` to reopen the contract while implementing publication.

Checklist:
- Implement: define the shared boundary and neutral root-intake contract
- Test: map each claimed input or helper boundary to a real repo surface
- Validate: confirm `C-02` and `C-03` are concrete enough for downstream planning
