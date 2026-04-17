---
slice_id: S2
seam_id: SEAM-4
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - any change in built-in backend support that affects promotion eligibility
    - new multi-backend evidence that changes the promotion case
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
  - THR-04
contracts_produced:
  - C-07
contracts_consumed:
  - C-05
  - C-06
open_remediations: []
---
### S2 - promotion-recommendation-and-no-promotion-routing

- **User/system value**: force one explicit answer about what stays backend-specific, what if
  anything is eligible for UAA promotion, and when the right answer is no promotion yet.
- **Scope (in/out)**:
  - In: recommendation criteria, no-promotion path, follow-on pack trigger conditions
  - Out: executing spec edits or capability-matrix edits in this seam
- **Acceptance criteria**:
  - the seam can produce an explicit promotion recommendation or explicit no-promotion answer
  - backend-specific and unstable behavior remain visible when promotion is not justified
  - any required spec or matrix work is routed to a separate follow-on pack instead of hidden in
    this seam
- **Dependencies**:
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- **Verification**:
  - confirm every recommendation is grounded in concrete backend behavior and the multi-backend
    promotion rules
  - confirm the no-promotion answer is explicit when universal promotion is premature
- **Rollout/safety**:
  - route canonical spec or matrix edits to a separate approved follow-on pack
  - fail closed on promotion ambiguity
- **Review surface refs**:
  - `review.md#falsification-questions`

#### S2.T1 - Lock the promotion decision and no-promotion routing

- **Outcome**: one seam-local recommendation path says whether OpenCode remains backend-specific,
  what if anything is promotion-eligible, and when a follow-on pack is required.
- **Inputs/outputs**:
  - Inputs: backend evidence, capability and extension registries, promotion rules
  - Outputs: `C-07` recommendation posture and explicit no-promotion routing
- **Thread/contract refs**: `THR-03`, `THR-04`, `C-07`
- **Implementation notes**:
  - keep no-promotion a first-class outcome rather than an implicit fallback
  - require a separate follow-on pack when the answer depends on canonical spec or matrix edits
- **Acceptance criteria**:
  - later closeout can publish one explicit recommendation without ambiguity
  - pack closeout can tell whether new work is needed or not
- **Test notes**:
  - compare the recommendation rules against the multi-backend promotion constraints
- **Risk/rollback notes**:
  - if the recommendation path stays vague, backend-only behavior will drift into unofficial
    universal claims

Checklist:
- Implement: define the recommendation path and explicit no-promotion routing
- Test: compare the route against backend evidence and universal promotion rules
- Validate: confirm `C-07` is concrete enough for exit and closeout publication
