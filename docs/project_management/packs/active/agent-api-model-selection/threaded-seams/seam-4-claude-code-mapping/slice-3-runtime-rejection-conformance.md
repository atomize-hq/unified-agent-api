---
slice_id: S3
seam_id: SEAM-4
slice_kind: delivery
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
contracts_produced:
  - C-07
contracts_consumed:
  - C-04
  - C-09
open_remediations: []
candidate_subslices: []
---
### S3 - Runtime rejection conformance (Claude)

- **User/system value**: ensures syntactically-valid but runtime-rejected model ids fail safely and consistently (completion + terminal Error event parity) even when the stream is already open.
- **Acceptance criteria**:
  - completion error message and terminal Error event message match byte-for-byte
  - no raw model ids or stderr leaks into consumer-visible errors
- **Verification**: use a deterministic fake-claude scenario that fails after the stream begins.

