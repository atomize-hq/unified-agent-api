---
slice_id: S99
seam_id: SEAM-3
slice_kind: seam_exit_gate
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - wrapper contract drift after landing
    - capability advertisement or extension ownership drift after landing
    - validation or redaction posture drift after landing
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-03
contracts_produced:
  - C-05
  - C-06
contracts_consumed:
  - C-03
  - C-04
open_remediations: []
---
### S99 - seam-exit-gate

- **Purpose**: convert landed backend-planning execution into downstream-consumable closeout and
  promotion readiness for `SEAM-4`.
- **Scope (in/out)**:
  - In: landed evidence capture, contract and thread publication record, review-surface delta,
    stale-trigger emission, remediation disposition, promotion-readiness statement
  - Out: net-new backend implementation, wrapper contract changes, or UAA promotion decisions
- **Acceptance criteria**:
  - closeout can record the landed backend mapping and extension contract artifacts without
    ambiguity
  - outbound publication of `THR-03` is explicit
  - downstream stale triggers are explicit
  - promotion blockers, if any, are explicit
  - promotion readiness can be stated as `ready` or `blocked`
- **Dependencies**:
  - `../../governance/seam-3-closeout.md`
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
- **Verification**:
  - confirm closeout records landed backend mapping, capability advertisement posture, validation
    evidence, and final redaction boundary
  - confirm any downstream stale trigger matches seam-local basis metadata
- **Canonical contract refs**:
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
- **Review surface refs**:
  - `review.md#planned-seam-exit-gate-focus`
