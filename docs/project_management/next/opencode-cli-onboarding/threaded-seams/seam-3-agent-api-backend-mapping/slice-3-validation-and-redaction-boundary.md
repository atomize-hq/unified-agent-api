---
slice_id: S3
seam_id: SEAM-3
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
  - THR-03
contracts_produced:
  - C-05
  - C-06
contracts_consumed:
  - C-02
  - C-03
  - C-04
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
  - backend validation remains fixture-first and reproducible by default
  - replay and fake-binary posture stay explicit without becoming a new runtime transport
  - redaction and bounded-payload obligations are concrete enough for later backend tests
- **Dependencies**:
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
- **Verification**:
  - confirm backend validation can consume the published wrapper/manifest handoff without requiring
    a live provider account by default
  - confirm redaction and payload-boundary obligations remain explicit and testable
- **Rollout/safety**:
  - keep fixture, replay, and fake-binary posture tied to deterministic validation only
  - force reopen of upstream seams rather than normalizing wrapper or manifest drift locally
- **Review surface refs**:
  - `review.md#r2---backend-owned-boundary`

#### S3.T1 - Lock validation posture and redaction boundary

- **Outcome**: one conformance slice states when backend work must stop and reopen upstream seams
  and how deterministic validation proves bounded payload and redaction behavior.
- **Inputs/outputs**:
  - Inputs: published wrapper/manifest handoff, evidence contract, backend-owned mapping contract
  - Outputs: validation and redaction boundary language across seam-local planning
- **Thread/contract refs**: `THR-03`, `C-05`, `C-06`
- **Implementation notes**:
  - keep replay and fake-binary posture tied to deterministic validation only
  - keep reopen triggers tied to real wrapper, manifest, capability, or redaction drift
- **Acceptance criteria**:
  - backend work can tell when to stop and reopen upstream seams
  - later tests can verify redaction and bounded payload behavior without guessing
- **Test notes**:
  - compare the boundary against the evidence contract and the published wrapper/manifest handoff
- **Risk/rollback notes**:
  - if validation posture stays vague, later backend support will drift into live-provider-only
    proof paths

Checklist:
- Implement: make validation, replay/fake-binary, and redaction boundaries explicit for `THR-03`
- Test: compare the posture against wrapper, manifest, and evidence contracts
- Validate: confirm backend work can revalidate against published upstream truth without guessing
