---
slice_id: S2
seam_id: SEAM-4
slice_kind: adoption
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - Markdown update strategy changes
    - repo-gate participation changes
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-04
contracts_produced: []
contracts_consumed:
  - C-05
  - C-06
open_remediations: []
---
### S2 - Markdown staleness and preflight integration

- **User/system value**: stale Markdown or missing publication reruns fail before the repo lands misleading support claims.
- **Scope (in/out)**:
  - In: detect Markdown drift against the generated block and decide how support-matrix generation participates in `make preflight`.
  - Out: synthetic future-agent fixture expansion.
- **Acceptance criteria**:
  - Markdown freshness is checked against the same generated section strategy landed in `SEAM-3`.
  - repo-gate participation is explicit and deterministic.
  - publication enforcement does not mutate the support semantics above the generated section.
- **Verification**:
  - targeted tests catch stale Markdown and missing reruns
  - repo-gate integration is documented and exercised in the same ownership surface

Checklist:
- Implement: enforce Markdown freshness and repo-gate participation
- Test: prove stale Markdown fails deterministically
- Validate: confirm projection ownership stays downstream of the published row model
