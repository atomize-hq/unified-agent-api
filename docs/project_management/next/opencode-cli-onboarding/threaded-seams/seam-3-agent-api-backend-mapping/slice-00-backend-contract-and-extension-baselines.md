---
slice_id: S00
seam_id: SEAM-3
slice_kind: contract_definition
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - wrapper-owned event or completion semantics drift
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
  - THR-02
  - THR-03
contracts_produced:
  - C-05
  - C-06
contracts_consumed:
  - C-03
  - C-04
open_remediations: []
---
### S00 - backend-contract-and-extension-baselines

- **User/system value**: give backend implementation one explicit contract baseline for mapping,
  capability advertisement, and extension ownership so later work does not invent backend rules
  opportunistically.
- **Scope (in/out)**:
  - In: define the backend-owned mapping contract baseline and the backend-specific extension
    ownership baseline under the canonical planning surface
  - Out: landing backend code, widening universal capabilities, or reopening wrapper semantics
- **Acceptance criteria**:
  - `docs/specs/opencode-agent-api-backend-contract.md` is concrete enough about request, event,
    completion, bounded payload, and redaction ownership that later implementation can proceed
    without guessing
  - backend-specific extension keys remain under `backend.opencode.*` unless a later seam justifies
    promotion
  - the baseline stays bounded by the landed wrapper and manifest contracts
- **Dependencies**:
  - `governance/seam-2-closeout.md`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
- **Verification**:
  - compare each backend-owned contract decision against the published wrapper and manifest
    contracts plus the universal capability and extension rules
  - confirm the baselines live under the canonical contract surfaces named for this seam
- **Rollout/safety**:
  - fail closed on wrapper-boundary drift
  - keep backend-specific capability and extension behavior out of universal promotion decisions
- **Review surface refs**:
  - `review.md#r1---backend-handoff`
  - `review.md#r2---backend-owned-boundary`

#### S00.T1 - Define the backend-owned mapping contract baseline

- **Outcome**: one backend contract names request mapping, event bucketing, completion-finality,
  bounded payload, and redaction obligations for OpenCode backend work.
- **Inputs/outputs**:
  - Inputs: landed wrapper and manifest contracts, universal envelope rules
  - Outputs: `docs/specs/opencode-agent-api-backend-contract.md`
- **Thread/contract refs**: `THR-03`, `C-05`
- **Implementation notes**:
  - keep the mapping consumer-shaped for later backend implementation
  - keep raw backend lines and provider-specific diagnostics out of the public envelope
- **Acceptance criteria**:
  - later seams can cite one backend-owned contract for mapping detail
  - completion finality and bounded payload obligations remain explicit and testable
- **Test notes**:
  - compare the contract against the published wrapper handoff and universal schema constraints
- **Risk/rollback notes**:
  - if backend mapping remains ambiguous, implementation will either leak payloads or overclaim
    support

#### S00.T2 - Define backend-specific extension ownership

- **Outcome**: one baseline states which OpenCode-specific extension keys stay backend-owned and how
  unsupported keys fail closed.
- **Inputs/outputs**:
  - Inputs: extension registry rules, published wrapper/manifest handoff
  - Outputs: backend-owned extension language across `docs/specs/opencode-agent-api-backend-contract.md`
    and seam-local planning
- **Thread/contract refs**: `THR-03`, `C-06`
- **Implementation notes**:
  - keep extension ownership under `backend.opencode.*` unless a later seam explicitly promotes it
  - make unsupported or unstable keys explicit rather than implied
- **Acceptance criteria**:
  - later implementation can advertise only supported capabilities and extensions
  - extension ownership stays concrete without widening universal spec scope
- **Test notes**:
  - compare the owner rules against `docs/specs/unified-agent-api/extensions-spec.md`
- **Risk/rollback notes**:
  - if extension ownership stays vague, backend work will drift into unofficial universal behavior

Checklist:
- Implement: define the backend-owned mapping and extension baselines
- Test: cross-check contract decisions against published wrapper/manifest inputs and universal rules
- Validate: confirm `C-05` and `C-06` are concrete enough for backend implementation planning
