---
slice_id: S00
seam_id: SEAM-2
slice_kind: contract_definition
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - wrapper-owned event or completion semantics drift after `SEAM-1`
    - capability or extension registry changes under `docs/specs/unified-agent-api/**`
gates:
  pre_exec:
    review: inherited
    contract: passed
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
  - C-07
open_remediations: []
---
### S00 - backend-contract-and-registration-baselines

- **User/system value**: give backend implementation one explicit contract and registration
  baseline so `crates/agent_api/**` work does not invent wrapper or publication rules
  opportunistically.
- **Scope (in/out)**:
  - In: define backend-owned request, event, completion, capability, extension, registration, and
    fail-closed baselines under the canonical planning surface
  - Out: landing backend code, widening universal capabilities, or reopening wrapper semantics
- **Acceptance criteria**:
  - `docs/specs/opencode-agent-api-backend-contract.md` is concrete enough about request, event,
    completion, bounded payload, redaction, capability, and extension ownership that later
    implementation can proceed without guessing
  - backend registration and feature gating surfaces are explicit enough that implementation can
    stay inside `crates/agent_api/**`
  - the baseline stays bounded by the landed wrapper and manifest contracts plus the no-promotion
    posture carried by `THR-04`
- **Dependencies**:
  - `../../governance/seam-1-closeout.md`
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
  - `docs/specs/unified-agent-api/run-protocol-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
- **Verification**:
  - compare each backend-owned contract and registration decision against the published wrapper and
    manifest handoff plus the universal protocol and extension rules
  - confirm the baseline stays concrete without waiting on post-exec publication
- **Rollout/safety**:
  - fail closed on wrapper-boundary drift
  - keep backend-specific behavior out of universal promotion decisions
- **Review surface refs**:
  - `review.md#r1---backend-handoff`
  - `review.md#r2---backend-owned-boundary`

#### S00.T1 - Define the backend-owned contract baseline

- **Outcome**: one backend contract names request mapping, event bucketing, completion-finality,
  bounded payload, redaction, capability posture, and extension ownership obligations for OpenCode
  backend work.
- **Inputs/outputs**:
  - Inputs: landed wrapper and manifest contracts, universal envelope rules
  - Outputs: `docs/specs/opencode-agent-api-backend-contract.md`
- **Thread/contract refs**: `THR-06`, `C-03`
- **Implementation notes**:
  - keep the contract consumer-shaped for later backend implementation
  - keep raw backend lines and provider-specific diagnostics out of the public envelope
- **Acceptance criteria**:
  - later seams can cite one backend-owned contract for mapping and capability detail
  - completion finality, bounded payload, and fail-closed obligations remain explicit and testable

Checklist:
- Implement: define the backend-owned contract and registration baselines
- Test: cross-check contract decisions against published wrapper/manifest inputs and universal rules
- Validate: confirm `C-03` is concrete enough for backend implementation planning
