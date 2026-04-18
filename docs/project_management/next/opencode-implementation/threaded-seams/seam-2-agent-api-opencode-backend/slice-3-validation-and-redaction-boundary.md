---
slice_id: S3
seam_id: SEAM-2
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - new evidence that replay, fake-binary, or fixture posture must differ
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
  - THR-06
contracts_produced:
  - C-03
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
---
### S3 - validation-and-redaction-boundary

- **User/system value**: make sure backend validation and redaction posture is concrete enough that
  support claims do not depend on live-provider luck and payload leakage remains reviewable.
- **Scope (in/out)**:
  - In: fixture-first validation posture, replay/fake-binary boundary, redaction and bounded
    payload verification, explicit reopen triggers
  - Out: live-provider-only support claims, promotion review, or manifest-root policy changes
- **Acceptance criteria**:
  - backend validation remains fixture-first and reproducible by default without requiring a live
    provider account
  - replay and fake-binary posture stay explicit as validation-only support paths and do not
    become new runtime transports
  - redaction and bounded-payload obligations are concrete enough to verify without guessing
- **Dependencies**:
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
- **Verification**:
  - confirm backend validation can consume the revalidated wrapper/manifest handoff without
    requiring a live provider account by default
  - confirm redaction and payload-boundary obligations remain explicit and testable through
    deterministic validation posture only

Checklist:
- Implement: make validation, replay/fake-binary, and redaction boundaries explicit for `THR-06`
- Test: compare the posture against wrapper, manifest, and evidence contracts
- Validate: confirm backend work can revalidate against published upstream truth without guessing
