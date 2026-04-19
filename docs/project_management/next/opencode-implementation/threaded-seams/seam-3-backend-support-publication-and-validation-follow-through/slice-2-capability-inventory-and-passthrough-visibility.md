---
slice_id: S2
seam_id: SEAM-3
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - capability-inventory semantics drift
    - backend capability posture changes in a way that affects publication meaning
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
  - THR-07
contracts_produced:
  - C-04
contracts_consumed:
  - C-03
open_remediations: []
---
### S2 - capability-inventory-and-passthrough-visibility

- **User/system value**: keep backend capability inventory accurate while preserving an explicit
  explanation of what remains passthrough or backend-specific.
- **Scope (in/out)**:
  - In: OpenCode enrollment in capability inventory generation, capability-matrix doc updates, and
    passthrough-visibility wording needed so backend-specific behavior is not misread as universal
    support
  - Out: widening OpenCode capability advertisement beyond the landed backend or changing
    support-matrix semantics already owned by S1
- **Acceptance criteria**:
  - capability inventory reflects the landed OpenCode backend posture and no more
  - capability inventory wording stays separate from support publication wording
  - passthrough visibility stays explicit where support does not yet exist
- **Dependencies**:
  - `docs/specs/unified-agent-api/capability-matrix.md`
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `../../governance/seam-2-closeout.md`
  - current `crates/xtask/src/capability_matrix.rs`
- **Verification**:
  - run `cargo run -p xtask -- capability-matrix`
  - inspect the committed capability inventory and docs for fidelity to the landed backend
    contract and explicit passthrough visibility where applicable

Checklist:
- Implement: enroll OpenCode in capability inventory and clarify passthrough visibility
- Test: compare the inventory against the landed `agent_api` OpenCode backend contract
- Validate: confirm capability inventory does not imply new universal support
