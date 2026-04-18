---
slice_id: S1
seam_id: SEAM-2
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - wrapper-owned event or completion semantics drift after `SEAM-1`
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
  - THR-05
  - THR-06
contracts_produced:
  - C-03
contracts_consumed:
  - C-01
open_remediations: []
---
### S1 - request-event-and-completion-mapping

- **User/system value**: keep backend implementation bounded by one explicit request, event, and
  completion mapping so the universal envelope stays small, stable, and redacted.
- **Scope (in/out)**:
  - In: run-request mapping, event bucketing, completion-finality preservation, bounded payload,
    and redaction rules
  - Out: capability promotion, manifest-root publication, and helper-surface expansion
- **Acceptance criteria**:
  - the backend seam consumes wrapper-owned typed events and completion semantics without inventing
    new wrapper behavior
  - bounded payload and redaction rules stay explicit
  - raw backend lines and provider-specific diagnostics remain out of the universal envelope
- **Dependencies**:
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `../../governance/seam-1-closeout.md`
- **Verification**:
  - confirm the backend contract answers what `crates/agent_api/**` must implement without
    reopening wrapper semantics
  - confirm completion finality remains explicit and the envelope stays bounded

Checklist:
- Implement: define request, event, completion, bounded payload, and redaction mapping rules
- Test: compare mapping decisions against wrapper-owned runtime and evidence contracts
- Validate: confirm backend work can consume the revalidated handoff without reopening `SEAM-1`
