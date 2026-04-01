---
slice_id: S2
seam_id: SEAM-2
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
  - THR-02
  - THR-03
contracts_produced:
  - C-05
contracts_consumed:
  - C-09
open_remediations: []
candidate_subslices: []
---
### S2 - Backend exposure gates + advertising coupling

- **User/system value**: ensures capability advertising is truthful across all exposed run flows by coupling R0 admission, typed normalization, and backend mapping outcomes.
- **Scope (in/out)**:
  - In:
    - backend capability sets include `agent_api.config.model.v1` only when deterministic outcomes exist
    - adapter surfaces consume the shared typed handoff (`Option<String>`) without re-parsing
  - Out:
    - runtime rejection translation details (SEAM-3/4)
- **Acceptance criteria**:
  - advertising is enabled only when deterministic across exposed flows
  - no backend mapping seam needs to inspect raw request extensions
- **Dependencies**: S1, `C-05`, `C-09`
- **Verification**:
  - targeted tests prove R0 ordering + typed handoff adoption
  - diff review shows no second parser site appears
- **Rollout/safety**: if deterministic support is not yet present for any flow, keep advertising false until it is.
- **Review surface refs**: `../../review_surfaces.md` (R1, R2)

#### S2.T1 - Couple advertising to deterministic outcomes per backend

- **Outcome**: each backend's capability inventory is updated only when it can either apply the override or take a pinned safe rejection path for every exposed run flow.
- **Thread/contract refs**: `THR-03`, `C-05`
- **Acceptance criteria**: no "silent drop" behavior remains possible once advertising is true.

Checklist:
- Implement: capability set flip (as allowed)
- Test: capability inventory assertions
- Validate: confirm each exposed flow has a deterministic outcome

#### S2.T2 - Enforce "no second parser" adoption in adapter surfaces

- **Outcome**: all adapter/harness code consumes only the typed `Option<String>` handoff.
- **Thread/contract refs**: `THR-02`, `C-09`
- **Acceptance criteria**: `rg` shows no new parse/trim/validate logic outside `normalize.rs`.

Checklist:
- Implement: adapter wiring changes
- Test: targeted harness/backend tests
- Validate: repo search for raw parse sites
