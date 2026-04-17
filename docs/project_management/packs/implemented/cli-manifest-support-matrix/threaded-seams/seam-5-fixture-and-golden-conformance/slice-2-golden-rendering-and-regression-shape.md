---
slice_id: S2
seam_id: SEAM-5
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - JSON and Markdown goldens diverge
    - row ordering changes without golden updates
    - evidence-note rules change without regression updates
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
contracts_produced: []
contracts_consumed:
  - C-04
  - C-05
  - C-06
open_remediations: []
---
### S2 - Golden rendering and regression shape

- **User/system value**: the published JSON and Markdown surfaces stay deterministic and reject stale projections.
- **Scope (in/out)**:
  - In: golden coverage for JSON and Markdown outputs, ordering regressions, and stale-render detection.
  - Out: fixture-shape expansion and seam-exit handoff records.
- **Acceptance criteria**:
  - JSON and Markdown goldens derive from the same model and fail together when the model changes.
  - row-order or evidence-note regressions fail deterministically.
  - stale Markdown cannot survive a regression pass.
- **Verification**:
  - targeted golden tests compare the rendered outputs against the same shared model
  - stale-render tests catch block drift instead of rerendering silently
  - ordering regressions remain explicit and reproducible

Checklist:
- Implement: add or refresh the golden comparison coverage
- Test: prove block-scoped Markdown freshness and JSON parity stay coupled
- Validate: confirm row ordering and evidence notes remain a single derived truth
