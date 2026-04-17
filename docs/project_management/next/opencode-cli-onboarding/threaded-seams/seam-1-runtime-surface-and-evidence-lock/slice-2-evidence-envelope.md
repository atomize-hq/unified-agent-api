---
slice_id: S2
seam_id: SEAM-1
slice_kind: documentation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - provider/auth posture changes invalidate the current smoke assumptions
    - downstream support publication tries to rely on live smoke alone
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
  - C-02
contracts_consumed: []
open_remediations: []
---
### S2 - evidence-envelope

- **User/system value**: tell downstream seams exactly what evidence makes the runtime lock
  trustworthy now and what extra replay evidence later support publication still needs.
- **Scope (in/out)**:
  - In: install and auth prerequisites, maintainer smoke expectations, deterministic replay
    expectations, reopen triggers
  - Out: wrapper implementation, support-matrix publication, capability promotion
- **Acceptance criteria**:
  - live maintainer smoke and deterministic replay obligations are explicitly separated
  - provider-specific failures are treated as evidence posture, not as implicit wrapper semantics
  - reopen triggers are concrete enough to invalidate `basis.currentness` when reality drifts
- **Dependencies**:
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`
- **Verification**:
  - cross-check every prerequisite class from the seam brief against the evidence contract
  - verify the contract states that later support publication needs committed replay artifacts
- **Rollout/safety**:
  - preserve fixture-first downstream testing expectations
  - keep provider-backed smoke necessary for orientation but insufficient for long-term support
- **Review surface refs**:
  - `review.md#likely-mismatch-hotspots`

#### S2.T1 - Separate live smoke from deterministic replay

- **Outcome**: the evidence contract records which live proofs lock the seam now and which replay
  artifacts later seams must publish before claiming support.
- **Inputs/outputs**:
  - Inputs: packet sections 6, 9, 11, and 13
  - Outputs: evidence sections in `docs/specs/opencode-onboarding-evidence-contract.md`
- **Thread/contract refs**: `THR-01`, `C-02`
- **Implementation notes**:
  - keep the future wrapper protocol evidence artifact explicit
  - do not let provider-account success become a permanent support requirement
- **Acceptance criteria**:
  - the contract names both planning-grade smoke and publication-grade replay evidence
  - the contract records clear reopen triggers
- **Test notes**:
  - compare the evidence classes against packet risks and acceptance gates
- **Risk/rollback notes**:
  - if replay requirements stop being explicit, reopen `REM-001`

Checklist:
- Implement: done
- Test: done
- Validate: done
- Cleanup: done
