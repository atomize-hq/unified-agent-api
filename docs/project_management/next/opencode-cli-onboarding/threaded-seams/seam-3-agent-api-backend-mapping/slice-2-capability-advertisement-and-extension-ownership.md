---
slice_id: S2
seam_id: SEAM-3
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - capability or extension registry changes under `docs/specs/unified-agent-api/**`
    - wrapper or manifest contract changes that alter supported backend behavior
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-03
contracts_produced:
  - C-05
  - C-06
contracts_consumed:
  - C-03
  - C-04
open_remediations: []
---
### S2 - capability-advertisement-and-extension-ownership

- **User/system value**: keep capability advertisement and extension handling concrete enough that
  backend support stays reviewable and universal promotion pressure fails closed.
- **Scope (in/out)**:
  - In: supported capability ids, fail-closed advertisement posture, backend-specific extension
    ownership, unsupported-key handling
  - Out: UAA promotion decisions, manifest-root evidence publication, or wrapper contract changes
- **Acceptance criteria**:
  - the backend seam advertises only capabilities the concrete backend behavior can honor
  - backend-specific extension keys stay under `backend.opencode.*` unless explicitly promoted later
  - unsupported capabilities and extension keys fail closed and remain explicit
- **Dependencies**:
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
  - `governance/seam-2-closeout.md`
- **Verification**:
  - compare capability and extension decisions against the universal registry rules and published
    wrapper/manifest inputs
  - confirm later promotion review can tell what remains backend-specific without guessing
- **Rollout/safety**:
  - fail closed on unsupported or unstable behaviors
  - keep backend-specific extension ownership explicit for downstream review
- **Review surface refs**:
  - `review.md#r1---backend-handoff`

#### S2.T1 - Lock capability advertisement and extension ownership

- **Outcome**: the backend contract explicitly names which capability ids are supported and which
  extension keys remain backend-owned.
- **Inputs/outputs**:
  - Inputs: published wrapper/manifest handoff, universal capability and extension rules
  - Outputs: capability and extension sections in `docs/specs/opencode-agent-api-backend-contract.md`
- **Thread/contract refs**: `THR-03`, `C-05`, `C-06`
- **Implementation notes**:
  - keep advertisement tied to concrete backend behavior
  - keep backend-specific keys explicit rather than implied
- **Acceptance criteria**:
  - later implementation can advertise only what the owner docs justify
  - later promotion review can distinguish backend-only behavior from candidate promotion work
- **Test notes**:
  - compare the owner rules against the universal capability and extension registries
- **Risk/rollback notes**:
  - vague advertisement posture will force later seams to reopen backend scope before they can
    review promotion

Checklist:
- Implement: define supported capability advertisement and backend-specific extension ownership
- Test: compare the posture against the universal capability and extension rules
- Validate: confirm `SEAM-4` can later consume one explicit backend-owned truth
