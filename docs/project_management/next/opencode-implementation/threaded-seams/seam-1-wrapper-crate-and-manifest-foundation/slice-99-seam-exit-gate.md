---
slice_id: S99
seam_id: SEAM-1
slice_kind: seam_exit_gate
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - wrapper-owned runtime semantics drift after landing
    - manifest-root inventory or validator rules drift after landing
    - deterministic replay or evidence posture weakens after landing
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
  - C-01
  - C-02
contracts_consumed:
  - C-07
open_remediations: []
---
### S99 - seam-exit-gate

- **Purpose**: convert landed wrapper and manifest execution into a downstream-consumable closeout
  record with explicit `THR-05` publication and promotion readiness.
- **Scope (in/out)**:
  - In: landed evidence capture, contract publication record, thread publication record,
    review-surface delta, stale-trigger emission, remediation disposition, and promotion-readiness
    statement
  - Out: net-new wrapper, manifest, backend, or publication implementation
- **Acceptance criteria**:
  - closeout can record landed `crates/opencode/**`, `cli_manifests/opencode/**`, and
    OpenCode-root validation evidence without ambiguity
  - outbound publication of `THR-05` is explicit
  - downstream stale triggers are explicit and tied to real landed surfaces
  - promotion blockers, if any, are explicit
  - promotion readiness can be stated as `ready` or `blocked`
- **Dependencies**:
  - `../../governance/seam-1-closeout.md`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
- **Verification**:
  - confirm closeout records landed wrapper behavior, manifest inventory posture, validator evidence,
    and final deterministic-evidence boundary
  - confirm downstream stale triggers match seam-local basis metadata
- **Canonical contract refs**:
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
- **Review surface refs**:
  - `review.md#planned-seam-exit-gate-focus`
