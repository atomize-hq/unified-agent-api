---
slice_id: S00
seam_id: SEAM-1
slice_kind: contract_definition
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - OpenCode CLI event-shape drift on `run --format json`
    - helper-surface promotion into the v1 wrapper boundary
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: inherited
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
### S00 - contract-baselines

- **User/system value**: give downstream seams canonical repo-owned baselines for the runtime and
  evidence contracts so wrapper work does not depend on packet-era wording.
- **Scope (in/out)**:
  - In: create `docs/specs/opencode-wrapper-run-contract.md`,
    `docs/specs/opencode-onboarding-evidence-contract.md`, and the verification checklist that
    keeps them aligned with the packet and charter
  - Out: wrapper implementation, manifest inventory definition, backend mapping details
- **Acceptance criteria**:
  - both spec files exist under `docs/specs/`
  - the runtime contract names the single canonical v1 surface and the explicit deferred surfaces
  - the evidence contract names prerequisites, replay versus live-smoke posture, and reopen
    triggers
  - neither spec leaks planning IDs
- **Dependencies**:
  - `../../seam-1-runtime-surface-and-evidence-lock.md`
  - `../../../cli-agent-onboarding-charter.md`
  - `../../../cli-agent-onboarding-third-agent-packet.md`
- **Verification**:
  - compare each normative decision against packet sections 9-13 and the charter's wrapper rules
  - confirm the docs live under `docs/specs/**`, which is this repo's normative contract surface
  - confirm no planning IDs appear in the new spec files
- **Rollout/safety**:
  - fail closed when the packet and charter do not justify widening scope
  - keep helper surfaces deferred until a later seam explicitly reopens them
- **Review surface refs**:
  - `review.md#r1---runtime-lock-and-downstream-handoff`
  - `review.md#r2---canonical-v1-boundary`

#### S00.T1 - Write the runtime-surface contract baseline

- **Outcome**: `docs/specs/opencode-wrapper-run-contract.md` pins the one allowed v1 wrapper
  surface and the deferred-surface list.
- **Inputs/outputs**:
  - Inputs: packet sections 9-12, charter wrapper rules
  - Outputs: `docs/specs/opencode-wrapper-run-contract.md`
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - keep the document descriptive and free of planning IDs
  - make helper-surface exclusions explicit
- **Acceptance criteria**:
  - `opencode run --format json` is the only canonical v1 wrapper seam
  - `serve`, `acp`, `run --attach`, and interactive TUI remain explicitly deferred
- **Test notes**:
  - cross-check the document against the packet's observed evidence and invalidation conditions
- **Risk/rollback notes**:
  - if new evidence contradicts the scope lock, reopen `SEAM-1` instead of silently weakening the
    contract

#### S00.T2 - Write the evidence-envelope contract baseline

- **Outcome**: `docs/specs/opencode-onboarding-evidence-contract.md` defines prerequisite classes,
  live-smoke posture, deterministic replay expectations, and reopen triggers.
- **Inputs/outputs**:
  - Inputs: packet sections 6, 9, 11, and 13
  - Outputs: `docs/specs/opencode-onboarding-evidence-contract.md`
- **Thread/contract refs**: `THR-01`, `C-02`
- **Implementation notes**:
  - distinguish live maintainer smoke from the committed replay evidence later seams must publish
  - keep provider-specific failures from widening v1 semantics
- **Acceptance criteria**:
  - the document names install, auth, model-routing, smoke, replay, and reopen rules
  - downstream seams can tell which evidence is sufficient for planning versus support publication
- **Test notes**:
  - verify every prerequisite class called out in the seam brief appears in the spec
- **Risk/rollback notes**:
  - if the evidence posture becomes ambiguous again, reopen `REM-001`

Checklist:
- Implement: done
- Test: done
- Validate: done
- Cleanup: done
