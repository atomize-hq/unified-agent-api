---
slice_id: S2
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
  - C-02
  - C-07
  - C-09
open_remediations: []
candidate_subslices: []
---
### S2 - Print/session argv conformance (ordering + fallback exclusion)

- **User/system value**: prevents drift by pinning argv ordering and explicitly proving that the universal key does not map to `--fallback-model`.
- **Acceptance criteria**:
  - `--model <trimmed-id>` appears in the root-flags region, before any `--add-dir` group, session-selector flags, `--fallback-model`, and the final prompt token
  - the universal key never maps to `--fallback-model`
- **Verification**: focused tests that inspect the emitted argv shape for both print + session flows.

