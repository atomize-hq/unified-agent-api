---
slice_id: S3
seam_id: SEAM-3
slice_kind: delivery
execution_horizon: next
status: decomposed
plan_version: v1
basis:
  currentness: provisional
  basis_ref: seam.md#basis
  stale_triggers: []
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-04
contracts_produced:
  - C-06
contracts_consumed:
  - C-04
  - C-09
open_remediations: []
candidate_subslices: []
---
### S3 - Runtime rejection conformance

- **User/system value**: ensures syntactically-valid but runtime-rejected model ids fail safely and consistently (completion + terminal Error event parity) even when the stream is already open.
- **Scope (in/out)**:
  - In:
    - runtime rejection classification + safe message translation
    - scenario coverage for "stream already open then reject"
  - Out:
    - capability advertising ownership (SEAM-2)
- **Acceptance criteria**:
  - completion error message and terminal Error event message match
  - no raw model ids or stderr leaks into consumer-visible errors
- **Dependencies**: S1/S2, `C-04`
- **Verification**: use the fake-codex "midstream runtime rejection" scenario and assert parity.
- **Rollout/safety**: treat unsafe translation as a merge blocker.

#### S3.T1 - Implement/validate runtime rejection translation and parity

- **Outcome**: safe `Backend` translation is used for runtime model rejection, with event/completion parity.
- **Thread/contract refs**: `THR-04`, `C-04`

#### S3.T2 - Add focused regression tests for stream-open failure

- **Outcome**: tests lock the parity and redaction posture.
- **Thread/contract refs**: `THR-04`, `C-04`
