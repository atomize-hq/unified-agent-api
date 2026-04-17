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
    - downstream seams cite packet prose instead of the canonical specs
    - downstream seams widen scope without a `SEAM-1` reopen signal
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
contracts_produced:
  - C-01
  - C-02
contracts_consumed: []
open_remediations: []
---
### S3 - downstream-handoff-check

- **User/system value**: make sure `THR-01` is concrete enough that `SEAM-2` and `SEAM-3`
  consume the same runtime and evidence truth.
- **Scope (in/out)**:
  - In: downstream handoff wording for direct `THR-01` consumers, stale-trigger inventory,
    explicit boundary rules
  - Out: publishing `THR-01` as landed post-exec truth
- **Acceptance criteria**:
  - direct downstream consumers can cite `THR-01` plus the two canonical specs
  - stale triggers are explicit and tied to real changes in runtime or evidence posture
  - no later seam needs to infer whether helper-surface drift reopens `SEAM-1`
- **Dependencies**:
  - `threading.md`
  - `review.md`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
- **Verification**:
  - confirm `THR-01` still matches contract ownership in `threading.md`
  - confirm direct downstream seams can rely on the canonical spec files directly instead of packet
    prose
- **Rollout/safety**:
  - do not publish the thread early; keep publication for post-exec closeout
  - make stale-trigger rules explicit now so `SEAM-2` and `SEAM-3` do not guess
- **Review surface refs**:
  - `review.md#planned-seam-exit-gate-focus`

#### S3.T1 - Lock the downstream revalidation rules

- **Outcome**: the handoff explicitly names which changes force downstream revalidation for
  `THR-01` consumers.
- **Inputs/outputs**:
  - Inputs: seam brief, `threading.md`, review bundle,
    `docs/specs/opencode-wrapper-run-contract.md`,
    `docs/specs/opencode-onboarding-evidence-contract.md`
  - Outputs: stale-trigger and handoff language across the seam-local plan
- **Thread/contract refs**: `THR-01`, `C-01`, `C-02`
- **Implementation notes**:
  - keep the triggers tied to contract drift, helper-surface creep, or evidence drift
  - avoid overloading downstream seams with speculative blockers
  - make the handoff concrete enough that `SEAM-2` and `SEAM-3` can reopen `SEAM-1` without
    re-reading packet prose
- **Acceptance criteria**:
  - each stale trigger corresponds to a real contract or evidence change
  - downstream seams can tell when to stop and reopen `SEAM-1`
- **Test notes**:
  - compare the trigger list against seam brief basis metadata, review bundle hotspots, and the
    canonical contract refs
- **Risk/rollback notes**:
  - if downstream seams discover new hidden inputs, reopen this slice before execution

Checklist:
- Implement: done
- Test: done
- Validate: done
- Cleanup: done
