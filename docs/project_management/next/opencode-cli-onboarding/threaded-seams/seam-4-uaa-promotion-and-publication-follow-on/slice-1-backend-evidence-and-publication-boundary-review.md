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
  guessing from backend-specific behavior or collapsing backend support into universal promotion.
- **Scope (in/out)**:
  - In: backend evidence intake, capability and extension review, backend-support versus
    universal-promotion boundary, capability-matrix-as-evidence-only wording
  - Out: canonical spec edits, capability-matrix edits, or backend remapping
- **Acceptance criteria**:
  - promotion review consumes the landed backend contract and closeout as its concrete input
  - backend-specific support remains distinct from any candidate universal promotion
  - missing cross-backend evidence forces an explicit no-promotion outcome rather than implied
    approval
  - no recommendation depends on unpublished or guessed backend behavior
  - the capability matrix is treated as supporting evidence only, not as runtime truth
- **Dependencies**:
  - `docs/project_management/next/opencode-cli-onboarding/governance/seam-3-closeout.md`
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
- **Verification**:
  - confirm the backend contract and closeout are sufficient to describe what stays backend-specific
  - confirm any candidate promotion path is grounded in the existing universal rules
  - confirm the capability matrix is read as a generated evidence artifact, not as runtime truth
- **Rollout/safety**:
  - keep backend support and universal support as separate publication states
  - treat missing cross-backend evidence as a no-promotion condition rather than implied approval
  - do not infer universal promotion from single-backend evidence or matrix presence alone
- **Review surface refs**:
  - `review.md#r2---backend-support-versus-uaa-promotion`

#### S1.T1 - Lock backend-evidence intake and publication boundary

- **Outcome**: the seam names one bounded input set for promotion review and one explicit boundary
  between backend-specific support and UAA promotion, with explicit no-promotion fallback when
  cross-backend evidence is absent.
- **Inputs/outputs**:
  - Inputs: `SEAM-3` closeout, backend mapping contract, universal promotion rules, generated
    capability-matrix evidence
  - Outputs: promotion-review seam-local planning and recommendation inputs
- **Thread/contract refs**: `THR-03`, `C-05`, `C-06`
- **Implementation notes**:
  - treat backend evidence as sufficient for review, not as automatic promotion
  - force ambiguous support claims back into explicit no-promotion or follow-on routing
  - read the capability matrix as a maintenance/evidence artifact only; it does not authorize
    runtime promotion decisions
- **Acceptance criteria**:
  - downstream work can tell exactly what evidence the promotion seam is allowed to consume
  - backend-only behavior stays visible when no promotion is justified
  - missing cross-backend evidence cannot be normalized into universal support
- **Test notes**:
  - compare the review boundary against the landed backend contract and universal registry rules
  - check that the matrix is referenced only as evidence for review, not as truth for runtime
- **Risk/rollback notes**:
  - if the boundary stays vague, later reviewers will confuse backend support with universal truth
  - if evidence is incomplete, rollback to explicit no-promotion rather than implicit approval

Checklist:
- Implement: define the concrete backend-evidence intake and publication boundary
- Test: compare the boundary against the landed backend contract and universal rules
- Validate: confirm promotion review can proceed without guessing about backend behavior
