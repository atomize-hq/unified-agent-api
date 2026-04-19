---
slice_id: S2
seam_id: SEAM-2
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
  - THR-05
  - THR-06
contracts_produced:
  - C-03
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
---
### S2 - capability-advertisement-and-extension-ownership

- **User/system value**: keep capability advertisement and extension handling concrete enough that
  backend support stays reviewable and promotion pressure fails closed.
- **Scope (in/out)**:
  - In: supported capability ids, fail-closed advertisement posture, backend-specific extension
    ownership, unsupported-key handling, and backend registration touch points
  - Out: UAA promotion decisions, manifest-root evidence publication, or wrapper contract changes
- **Acceptance criteria**:
  - the backend seam advertises only capabilities the concrete backend behavior can honor
  - backend-specific extension keys stay backend-owned unless explicitly promoted later
  - unsupported capabilities and extension keys fail closed and remain explicit
- **Dependencies**:
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
  - `../../governance/seam-1-closeout.md`
- **Verification**:
  - compare capability and extension decisions against the universal registry rules and revalidated
    wrapper/manifest inputs
  - confirm later publication work can tell what remains backend-specific without guessing

Checklist:
- Implement: define supported capability advertisement, extension ownership, and registration posture
- Test: compare the posture against the universal capability and extension rules
- Validate: confirm `SEAM-3` can later consume one explicit backend-owned truth
