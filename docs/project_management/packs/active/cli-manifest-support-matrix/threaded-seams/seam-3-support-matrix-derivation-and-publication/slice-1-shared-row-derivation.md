---
slice_id: S1
seam_id: SEAM-3
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - row-field requirements drift from the contract
    - shared root-intake semantics change before derivation lands
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-02
  - THR-03
contracts_produced: []
contracts_consumed:
  - C-04
  - C-05
open_remediations: []
---
### S1 - Shared row derivation

- **User/system value**: committed manifest evidence becomes one neutral support row model that future seams can consume without re-deriving support truth.
- **Scope (in/out)**:
  - In: implement the shared derivation path in `crates/xtask/src/support_matrix.rs` and consume the landed root-intake contract from `SEAM-2`.
  - Out: final Markdown rendering details, contradiction enforcement policy, or fixture/golden expansion.
- **Acceptance criteria**:
  - one derived row model consumes versions, pointers, reports, and current metadata from both current agent roots.
  - the derivation path stays target-scoped first and does not collapse partial target truth into version-global claims.
  - the derived model remains future-agent-shaped rather than special-casing Codex or Claude by name.
- **Verification**:
  - run targeted support-matrix derivation checks
  - inspect the derived row fields against the owned contract baseline
  - confirm the derivation consumes the neutral root-intake seam rather than rebuilding root-specific logic

Checklist:
- Implement: derive the shared support row model
- Test: verify target-scoped truth and evidence-note rules
- Validate: confirm the derivation boundary is reusable for rendering and validation seams
