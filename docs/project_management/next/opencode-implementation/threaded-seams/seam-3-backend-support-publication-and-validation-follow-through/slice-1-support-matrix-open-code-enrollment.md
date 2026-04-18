---
slice_id: S1
seam_id: SEAM-3
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - support-matrix semantics or committed root-set assumptions drift
    - manifest-root publication evidence changes in a way that affects support-row meaning
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
  - THR-07
contracts_produced:
  - C-04
contracts_consumed:
  - C-02
  - C-03
open_remediations: []
---
### S1 - support-matrix-open-code-enrollment

- **User/system value**: make OpenCode show up in committed support publication without changing
  what support publication is allowed to claim.
- **Scope (in/out)**:
  - In: OpenCode enrollment in support-matrix generation, committed support output updates, root or
    backend enumeration changes, and explicit support-layer wording that stays below UAA promotion
  - Out: capability-inventory meaning changes beyond what publication must consume or any runtime
    backend changes
- **Acceptance criteria**:
  - support publication includes OpenCode in the correct layer or layers derived from landed
    manifest and backend evidence
  - committed support outputs and docs stay explicit about what is backend support versus UAA
    unified support versus passthrough visibility
  - OpenCode publication remains bounded to landed evidence instead of previewing future promotion
- **Dependencies**:
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
  - current `crates/xtask/src/support_matrix.rs`
  - current `cli_manifests/support_matrix/current.json`
- **Verification**:
  - run `cargo run -p xtask -- support-matrix --check`
  - inspect the committed output and doc wording for layer separation and explicit non-promotion
    posture

Checklist:
- Implement: enroll OpenCode in support publication and update committed support outputs
- Test: compare generated support rows against landed OpenCode manifest/backend evidence
- Validate: confirm support rows keep backend support and UAA support separate
