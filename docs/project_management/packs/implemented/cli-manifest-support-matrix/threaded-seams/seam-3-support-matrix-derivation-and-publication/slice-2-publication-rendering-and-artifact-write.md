---
slice_id: S2
seam_id: SEAM-3
slice_kind: adoption
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - publication paths change
    - JSON and Markdown drift away from the same row model
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
contracts_produced: []
contracts_consumed:
  - C-04
  - C-05
open_remediations: []
---
### S2 - Publication rendering and artifact write

- **User/system value**: the seam writes both publication surfaces from one row model so support truth stays deterministic and reviewable.
- **Scope (in/out)**:
  - In: render `cli_manifests/support_matrix/current.json` and the Markdown projection from the same derived model.
  - Out: contradiction enforcement and fixture/golden coverage beyond what is required to stabilize the renderer boundary.
- **Acceptance criteria**:
  - JSON and Markdown consume the same derived row model.
  - publication writes the canonical phase-1 output paths named in `SEAM-1`.
  - Markdown remains a projection and not a second truth source.
- **Verification**:
  - compare JSON and Markdown outputs against the owned row-model contract
  - confirm the publication paths match the canonical support publication contract
  - confirm no capability-matrix behavior is altered

Checklist:
- Implement: render the machine-readable and Markdown publication surfaces
- Test: verify both outputs consume the same row model
- Validate: confirm projection ownership stays explicit
