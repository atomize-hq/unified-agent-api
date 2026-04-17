---
slice_id: S1
seam_id: SEAM-4
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - backend mapping changes that materially alter what OpenCode exposes
    - capability or extension registry changes under `docs/specs/unified-agent-api/**`
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
contracts_produced: []
contracts_consumed:
  - C-05
  - C-06
open_remediations: []
---
### S1 - backend-evidence-and-publication-boundary-review

- **User/system value**: make sure promotion review consumes one concrete backend truth instead of
  guessing from backend-specific behavior or leaving support/publication states ambiguous.
- **Scope (in/out)**:
  - In: backend evidence intake, capability and extension review, backend-support versus
    universal-support boundary
  - Out: canonical spec edits, capability-matrix edits, or backend remapping
- **Acceptance criteria**:
  - promotion review consumes the landed backend contract and closeout as its concrete input
  - backend-specific support remains distinct from any candidate universal promotion
  - no recommendation depends on unpublished or guessed backend behavior
- **Dependencies**:
  - `docs/project_management/next/opencode-cli-onboarding/governance/seam-3-closeout.md`
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
- **Verification**:
  - confirm the backend contract and closeout are sufficient to describe what stays backend-specific
  - confirm any candidate promotion path is grounded in the existing universal rules
- **Rollout/safety**:
  - keep backend support and universal support as separate publication states
  - treat missing cross-backend evidence as a no-promotion condition rather than implied approval
- **Review surface refs**:
  - `review.md#r2---backend-support-versus-uaa-promotion`

#### S1.T1 - Lock backend-evidence intake and publication boundary

- **Outcome**: the seam names one bounded input set for promotion review and one explicit boundary
  between backend-specific support and UAA promotion.
- **Inputs/outputs**:
  - Inputs: `SEAM-3` closeout, backend mapping contract, universal promotion rules
  - Outputs: promotion-review seam-local planning and recommendation inputs
- **Thread/contract refs**: `THR-03`, `C-05`, `C-06`
- **Implementation notes**:
  - treat backend evidence as sufficient for review, not as automatic promotion
  - force ambiguous support claims back into explicit no-promotion or follow-on routing
- **Acceptance criteria**:
  - downstream work can tell exactly what evidence the promotion seam is allowed to consume
  - backend-only behavior stays visible when no promotion is justified
- **Test notes**:
  - compare the review boundary against the landed backend contract and universal registry rules
- **Risk/rollback notes**:
  - if the boundary stays vague, later reviewers will confuse backend support with universal truth

Checklist:
- Implement: define the concrete backend-evidence intake and publication boundary
- Test: compare the boundary against the landed backend contract and universal rules
- Validate: confirm promotion review can proceed without guessing about backend behavior
