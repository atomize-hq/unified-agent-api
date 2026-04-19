---
slice_id: S1
seam_id: SEAM-1
slice_kind: documentation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - `--format json` stops being a stable structured event transport
    - a helper surface becomes required to get machine-parseable events
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
contracts_consumed: []
open_remediations: []
---
### S1 - runtime-surface-lock

- **User/system value**: freeze the exact OpenCode surface downstream code is allowed to build
  against.
- **Scope (in/out)**:
  - In: canonical transport choice, accepted run controls, deferred helper surfaces, explicit
    non-goals
  - Out: typed event taxonomy, manifest artifact layout, UAA promotion
- **Acceptance criteria**:
  - the runtime contract states that `opencode run --format json` is the only v1 wrapper surface
  - accepted controls are limited to prompt, model, session reuse, continue, fork, and directory
    selection on that same surface
  - helper surfaces stay explicitly deferred
- **Dependencies**:
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`
- **Verification**:
  - check that the contract reflects packet sections 9-12 without importing helper-surface scope
  - confirm later seams can cite one contract file instead of packet prose
- **Rollout/safety**:
  - fail closed on add-dir and helper-surface expansion
  - keep backend-only controls backend-specific until a later seam reopens them
- **Review surface refs**:
  - `review.md#r2---canonical-v1-boundary`

#### S1.T1 - Freeze the accepted control set

- **Outcome**: the contract enumerates the run controls treated as part of the v1 seam.
- **Inputs/outputs**:
  - Inputs: packet section 10 and 12
  - Outputs: accepted control list in `docs/specs/opencode-wrapper-run-contract.md`
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - keep the control set on the same `run --format json` surface
  - leave timeout semantics wrapper-owned
- **Acceptance criteria**:
  - model, session reuse, continue, fork, and directory selection are explicit
  - `run --attach` is still deferred
- **Test notes**:
  - compare the control list against observed packet evidence
- **Risk/rollback notes**:
  - if control support fragments across multiple transports, reopen the seam

Checklist:
- Implement: done
- Test: done
- Validate: done
- Cleanup: done
