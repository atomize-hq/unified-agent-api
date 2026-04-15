---
slice_id: S3
seam_id: SEAM-3
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - row-model behavior diverges between JSON and Markdown
    - handoff evidence omits the consumer-facing row-model contract
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
### S3 - Support-matrix conformance and handoff

- **User/system value**: the seam exits with proof that publication stays deterministic and with a crisp handoff for validator and fixture consumers.
- **Scope (in/out)**:
  - In: targeted derivation/rendering verification plus the downstream handoff notes needed for closeout.
  - Out: full contradiction enforcement policy and synthetic future-agent fixture expansion.
- **Acceptance criteria**:
  - targeted tests or verification prove JSON and Markdown stay projections of the same row model.
  - closeout evidence names the derived row model, publication outputs, and verification commands explicitly.
  - downstream seams can tell which changes to row fields, ordering, or projection rules force revalidation.
- **Verification**:
  - run targeted support-matrix derivation/publication tests
  - map every closeout evidence item to a file path or command output
  - confirm future-agent-shaped neutrality remains a handoff rule even though full synthetic-fixture coverage belongs to `SEAM-5`

Checklist:
- Implement: capture support-matrix conformance and downstream handoff evidence
- Test: run targeted derivation and publication verification
- Validate: confirm downstream stale triggers stay concrete and seam-owned
