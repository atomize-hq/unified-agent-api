---
slice_id: S1
seam_id: SEAM-3
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - wrapper-owned event or completion semantics drift
    - new evidence that backend payload bounding or redaction must differ from current assumptions
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
  - C-01
  - C-03
open_remediations: []
---
### S1 - request-event-and-completion-mapping

- **User/system value**: keep backend implementation bounded by one explicit request/event/
  completion mapping so the universal envelope stays small, stable, and redacted.
- **Scope (in/out)**:
  - In: run-request mapping, event bucketing, completion-finality preservation, bounded payload and
    redaction rules
  - Out: capability promotion, manifest-root publication, and helper-surface expansion
- **Acceptance criteria**:
  - the backend seam consumes wrapper-owned typed events and completion semantics without inventing
    new wrapper behavior
  - bounded payload and redaction rules stay explicit
  - raw backend lines and provider-specific diagnostics remain out of the universal envelope
- **Dependencies**:
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - `governance/seam-2-closeout.md`
- **Verification**:
  - confirm the backend contract answers what `crates/agent_api/` must implement without reopening
    wrapper semantics
  - confirm completion finality remains explicit and the envelope stays bounded
- **Rollout/safety**:
  - keep the backend fail closed on unsupported inputs
  - preserve wrapper-owned redaction boundaries while enforcing backend-owned payload limits
- **Review surface refs**:
  - `review.md#r2---backend-owned-boundary`

#### S1.T1 - Lock request and event mapping rules

- **Outcome**: the backend contract explicitly names how wrapper-owned request and event inputs map
  into the universal backend envelope.
- **Inputs/outputs**:
  - Inputs: published wrapper handoff, universal backend envelope rules
  - Outputs: mapping sections in `docs/specs/opencode-agent-api-backend-contract.md`
- **Thread/contract refs**: `THR-03`, `C-05`
- **Implementation notes**:
  - keep the mapping consumer-shaped and bounded
  - avoid leaking raw lines or unstable backend detail into the universal envelope
- **Acceptance criteria**:
  - later backend implementation can cite one mapping contract for request/event behavior
  - unsupported payload shapes fail closed instead of silently widening the backend contract
- **Test notes**:
  - compare the resulting mapping sections against wrapper-owned event and completion rules
- **Risk/rollback notes**:
  - if mapping detail stays ambiguous, backend code will either over-advertise or leak raw payloads

Checklist:
- Implement: define request, event, completion, bounded payload, and redaction mapping rules
- Test: compare mapping decisions against wrapper-owned runtime and evidence contracts
- Validate: confirm backend work can consume the published handoff without reopening `SEAM-2`
