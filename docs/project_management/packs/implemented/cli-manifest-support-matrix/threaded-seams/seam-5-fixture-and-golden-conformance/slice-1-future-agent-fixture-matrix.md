---
slice_id: S1
seam_id: SEAM-5
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - shared normalization starts branching on known agent names
    - fixture root shape changes without test updates
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
  - C-02
  - C-03
open_remediations: []
---
### S1 - Future-agent fixture matrix

- **User/system value**: the neutral core stays shape-driven instead of collapsing into Codex/Claude special cases.
- **Scope (in/out)**:
  - In: Codex fixture coverage, Claude fixture coverage, one synthetic future-agent-shaped root, and shared fixture-loading coverage.
  - Out: golden output drift checks and seam-exit handoff bookkeeping.
- **Acceptance criteria**:
  - Codex and Claude fixtures both exercise the shared core through equivalent paths.
  - a synthetic future-agent-shaped root passes without any shared-core agent-name branching.
  - fixture coverage distinguishes "not attempted", "unsupported", and "intentionally partial" states.
- **Verification**:
  - targeted fixture tests cover all three fixture shapes
  - shared-core tests remain shape-driven and do not special-case known agent names
  - the fixture surface is broad enough to protect downstream neutrality work

Checklist:
- Implement: add or refresh the fixture matrix and synthetic root coverage
- Test: prove Codex, Claude, and synthetic fixtures reach the same shared path
- Validate: confirm the shared core still reads as future-agent-shaped
