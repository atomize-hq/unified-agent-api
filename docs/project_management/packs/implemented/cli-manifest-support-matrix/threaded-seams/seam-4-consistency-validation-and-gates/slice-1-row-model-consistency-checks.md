---
slice_id: S1
seam_id: SEAM-4
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - row fields or ordering change
    - pointer/status rules change
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-03
  - THR-04
contracts_produced:
  - C-06
contracts_consumed:
  - C-04
  - C-05
open_remediations: []
---
### S1 - Row-model consistency checks

- **User/system value**: support publication stops lying quietly when pointers, version status, and published rows disagree.
- **Scope (in/out)**:
  - In: add deterministic contradiction checks that consume the landed shared row model and current pointer/status evidence.
  - Out: Markdown freshness wiring and repo-gate adoption details.
- **Acceptance criteria**:
  - the validator consumes the published row model instead of re-deriving support truth.
  - explicit contradiction cases fail deterministically.
  - pointer promotion state never upgrades support truth silently.
- **Verification**:
  - targeted contradiction tests fail on mismatched pointers, version status, and row-model outputs
  - checks stay anchored to the landed row fields and ordering contract

Checklist:
- Implement: add row-model contradiction checks
- Test: exercise deterministic contradiction failures
- Validate: confirm `C-06` is concrete enough for downstream fixture planning
