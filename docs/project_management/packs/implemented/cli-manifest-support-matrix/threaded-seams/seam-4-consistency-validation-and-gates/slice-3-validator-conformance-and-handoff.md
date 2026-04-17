---
slice_id: S3
seam_id: SEAM-4
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - contradiction behavior diverges from the published row model
    - repo-gate evidence is missing from closeout
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
  - C-06
open_remediations: []
---
### S3 - Validator conformance and handoff

- **User/system value**: the validator seam exits with proof that contradiction and freshness checks are deterministic and with a clear handoff for fixture/golden conformance.
- **Scope (in/out)**:
  - In: targeted contradiction/freshness verification plus closeout-ready handoff evidence.
  - Out: future-agent fixture/golden expansion.
- **Acceptance criteria**:
  - targeted tests prove contradiction and Markdown-freshness behavior.
  - closeout evidence names the exact validator, repo-gate, and publication surfaces.
  - downstream seams can tell which validator or repo-gate changes force revalidation.
- **Verification**:
  - run targeted validator and freshness tests
  - map every closeout evidence item to a real file path or command output
  - confirm `SEAM-5` can consume the contradiction contract without reopening `SEAM-3`

Checklist:
- Implement: capture validator conformance and downstream handoff evidence
- Test: run targeted contradiction and freshness verification
- Validate: confirm downstream stale triggers stay concrete and seam-owned
