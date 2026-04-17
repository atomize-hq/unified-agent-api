---
slice_id: S2
seam_id: SEAM-3
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
  - THR-04
contracts_produced:
  - C-06
contracts_consumed:
  - C-04
  - C-09
open_remediations: []
candidate_subslices: []
---
### S2 - Fork model rejection before app-server transport

- **User/system value**: keeps Codex deterministic across exposed flows by rejecting model override requests on fork flows before any app-server request is issued.
- **Scope (in/out)**:
  - In:
    - consume the same typed `Option<String>` handoff as exec/resume
    - reject fork flows when `model.is_some()` before `thread/list`, `thread/fork`, or `turn/start`
    - preserve the pinned safe backend message `model override unsupported for codex fork`
  - Out:
    - runtime rejection after run start (S3)
- **Acceptance criteria**:
  - fork flows with model override fail before any outbound app-server call
  - surfaced error message is safe and exact
- **Dependencies**: S1 typed plumbing, `C-09`
- **Verification**: fork-focused tests assert zero outbound JSON-RPC calls plus exact message.
- **Rollout/safety**: earlier failure is strictly safer than partial fork execution.

#### S2.T1 - Thread typed model selection into fork request path

- **Outcome**: fork code knows whether an accepted model override was requested without re-parsing extensions.
- **Thread/contract refs**: `THR-04`, `C-09`

#### S2.T2 - Enforce pinned pre-handle safe rejection

- **Outcome**: `model.is_some()` short-circuits fork before selector resolution triggers app-server calls.
- **Thread/contract refs**: `THR-04`, `C-04`, `C-06`
