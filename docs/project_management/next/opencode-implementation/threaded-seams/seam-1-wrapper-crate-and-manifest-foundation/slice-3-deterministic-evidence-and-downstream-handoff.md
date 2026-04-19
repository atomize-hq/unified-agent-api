---
slice_id: S3
seam_id: SEAM-1
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - deterministic proof paths diverge from landed wrapper or manifest behavior
    - downstream seams need to infer `THR-05` inputs from planning prose instead of landed evidence
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
contracts_produced:
  - C-01
  - C-02
contracts_consumed:
  - C-07
open_remediations: []
---
### S3 - deterministic-evidence-and-downstream-handoff

- **User/system value**: make the wrapper and manifest landing evidence concrete enough that
  downstream seams consume one closeout-backed handoff instead of re-deriving OpenCode truth from
  tests, fixtures, or planning notes.
- **Scope (in/out)**:
  - In: deterministic proof boundary alignment, transcript or fake-binary evidence posture,
    explicit downstream handoff rules for `THR-05`, and scope guardrails that keep backend and
    publication work consumer-shaped
  - Out: publishing `THR-05` before landing, backend implementation, or support-matrix updates
- **Acceptance criteria**:
  - `THR-05` handoff expectations are explicit before implementation starts
  - deterministic proof paths align with the wrapper and manifest contracts
  - downstream seams know exactly which landed changes trigger revalidation instead of guessing
- **Dependencies**:
  - `threading.md`
  - `review.md`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
- **Verification**:
  - compare the downstream handoff expectations against `threading.md` contract ownership and
    thread directionality
  - confirm deterministic proof remains the default done-ness path for wrapper and manifest work
  - confirm `SEAM-2` and `SEAM-3` can cite landed artifacts and `THR-05` instead of this plan
- **Rollout/safety**:
  - keep live smoke basis-lock only
  - keep the downstream handoff bounded to wrapper and manifest truth
  - treat new generic publication or promotion work as out of scope here
- **Review surface refs**:
  - `review.md#r1---foundation-handoff-flow`

#### S3.T1 - Lock the deterministic evidence boundary

- **Outcome**: implementation has one explicit rule set for how transcript, fake-binary,
  offline-parser, and root-validation evidence combine without depending on live provider access.
- **Inputs/outputs**:
  - Inputs: evidence contract, wrapper contract, manifest contract
  - Outputs: concrete execution expectations across wrapper tests, manifest reports, and closeout
    evidence
- **Thread/contract refs**: `THR-05`, `C-01`, `C-02`
- **Implementation notes**:
  - keep wrapper runtime proof, manifest-root proof, and live smoke as separate evidence classes
  - do not let transcript or fixture handling become a new runtime surface
- **Acceptance criteria**:
  - execution can prove wrapper and manifest behavior deterministically
  - later seams can tell when live smoke is relevant and when it is not
- **Test notes**:
  - compare planned proof classes against the evidence contract and root-validator expectations
- **Risk/rollback notes**:
  - if deterministic proof classes blur together, downstream seams will consume the wrong evidence

#### S3.T2 - Lock the downstream handoff boundary for `THR-05`

- **Outcome**: the seam-local plan names exactly what `SEAM-2` and `SEAM-3` may consume after
  `SEAM-1` lands and which changes must force revalidation.
- **Inputs/outputs**:
  - Inputs: seam brief, `threading.md`, review bundle, wrapper and manifest contracts
  - Outputs: explicit handoff and stale-trigger expectations for closeout and `THR-05`
- **Thread/contract refs**: `THR-05`, `C-01`, `C-02`
- **Implementation notes**:
  - keep downstream consumers dependent on landed wrapper and manifest truth, not on seam-local
    prose
  - keep stale triggers tied to real wrapper, manifest, or evidence changes
- **Acceptance criteria**:
  - downstream seams know which landed changes reopen wrapper or manifest planning
  - `THR-05` can be published without ambiguity at seam exit
- **Test notes**:
  - compare the trigger list against `threading.md`, the seam brief, and canonical spec refs
- **Risk/rollback notes**:
  - if the handoff stays vague, `SEAM-2` and `SEAM-3` will each invent different upstream truth

Checklist:
- Implement: define the deterministic evidence boundary and explicit `THR-05` downstream handoff
- Test: compare the handoff and proof classes against threading and canonical contracts
- Validate: confirm downstream seams can revalidate against landed `SEAM-1` evidence later
