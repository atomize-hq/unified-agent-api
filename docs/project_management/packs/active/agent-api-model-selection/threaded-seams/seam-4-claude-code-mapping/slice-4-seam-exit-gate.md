---
slice_id: S4
seam_id: SEAM-4
slice_kind: seam_exit_gate
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers: []
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
contracts_produced: []
contracts_consumed:
  - C-07
open_remediations: []
candidate_subslices: []
---
### S4 - Seam-exit gate (SEAM-4)

This is the dedicated final seam-exit slice for SEAM-4. It does not hide unfinished feature delivery work.

- **Purpose**: record landed Claude Code mapping truth and publish the signal SEAM-5 and downstream promotion will consume.
- **Planned landed evidence**:
  - mapping commit/PR link
  - links to argv ordering + fallback-exclusion tests
  - runtime rejection parity tests (completion + terminal Error event)
- **Contracts expected to publish or change**: `C-07` (and any Claude Code contract doc updates)
- **Threads expected to advance**: `THR-05`
- **Promotion readiness statement**:
  - downstream promotion is blocked unless SEAM-4 closeout records `seam_exit_gate.status: passed` and `promotion_readiness: ready`

Checklist:
- Validate: closeout file updated: `../../governance/seam-4-closeout.md`
- Validate: remediation log updated if needed: `../../governance/remediation-log.md`

