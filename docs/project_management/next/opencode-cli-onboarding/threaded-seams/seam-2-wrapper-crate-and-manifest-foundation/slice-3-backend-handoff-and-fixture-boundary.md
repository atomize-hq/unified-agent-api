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
    - downstream seams infer wrapper semantics from implementation detail instead of contracts
    - fixture or fake-binary posture drifts from the landed evidence contract
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
contracts_produced:
  - C-03
  - C-04
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
---
### S3 - backend-handoff-and-fixture-boundary

- **User/system value**: make sure `THR-02` is concrete enough that `SEAM-3` consumes one
  wrapper-owned and manifest-owned truth without reopening `SEAM-1`.
- **Scope (in/out)**:
  - In: downstream handoff wording, stale-trigger inventory, fixture/fake-binary boundary, explicit
    backend-consumer rules
  - Out: publishing `THR-02` as landed post-exec truth
- **Acceptance criteria**:
  - downstream consumers can cite `THR-02` plus the wrapper and manifest contracts directly
  - stale triggers are explicit and tied to real wrapper, manifest, or validation drift
  - no later seam needs to infer whether helper-surface creep or evidence drift reopens upstream
    scope
- **Dependencies**:
  - `threading.md`
  - `review.md`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
- **Verification**:
  - confirm `THR-02` matches contract ownership in `threading.md`
  - confirm `SEAM-3` can rely on the canonical spec files directly instead of packet prose
- **Rollout/safety**:
  - do not publish the thread early; keep publication for post-exec closeout
  - make stale-trigger rules explicit now so `SEAM-3` does not guess
- **Review surface refs**:
  - `review.md#r1---wrapper-and-manifest-handoff`

#### S3.T1 - Lock the downstream wrapper and manifest handoff

- **Outcome**: the handoff explicitly names which changes force backend revalidation and which
  fixture/fake-binary boundaries the backend seam must preserve.
- **Inputs/outputs**:
  - Inputs: seam brief, `threading.md`, review bundle, wrapper contract, manifest contract, evidence
    contract
  - Outputs: stale-trigger and handoff language across the seam-local plan
- **Thread/contract refs**: `THR-02`, `C-03`, `C-04`
- **Implementation notes**:
  - keep the triggers tied to wrapper contract drift, manifest inventory drift, or evidence drift
  - avoid overloading downstream seams with speculative blockers
- **Acceptance criteria**:
  - each stale trigger corresponds to a real contract or evidence change
  - backend work can tell when to stop and reopen `SEAM-2`
- **Test notes**:
  - compare the trigger list against seam brief basis metadata, review bundle hotspots, and the
    canonical contract refs
- **Risk/rollback notes**:
  - if downstream seams discover new hidden inputs, reopen this slice before execution

Checklist:
- Implement: make the backend handoff and fixture/fake-binary boundary explicit for `THR-02`
- Test: compare the handoff against threading, review, and canonical contract refs
- Validate: confirm `SEAM-3` can revalidate against published wrapper and manifest truths later
