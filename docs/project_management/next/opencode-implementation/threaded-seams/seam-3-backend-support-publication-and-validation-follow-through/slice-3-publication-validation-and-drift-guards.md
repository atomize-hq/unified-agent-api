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
    - support or capability publication drift checks stop covering OpenCode
    - committed support outputs or OpenCode root validation drift from the landed evidence basis
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
### S3 - publication-validation-and-drift-guards

- **User/system value**: make OpenCode publication changes mechanically reviewable and resistant to
  future drift.
- **Scope (in/out)**:
  - In: deterministic publication verification commands, drift-check alignment, committed output
    regeneration, and evidence capture needed for closeout
  - Out: inventing new runtime evidence or turning validation into live-provider smoke by default
- **Acceptance criteria**:
  - OpenCode publication is covered by the repo's deterministic support and capability check flows
  - OpenCode root validation remains part of the seam's proof set
  - closeout can cite concrete command evidence and committed outputs instead of interpretation
- **Dependencies**:
  - current `crates/xtask` support and capability matrix commands
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
  - `cli_manifests/opencode/**`
- **Verification**:
  - run `cargo run -p xtask -- support-matrix --check`
  - run `cargo run -p xtask -- capability-matrix`
  - run `cargo run -p xtask -- codex-validate --root cli_manifests/opencode`
  - confirm closeout can point at deterministic command results and committed outputs

Checklist:
- Implement: align publication drift checks and committed outputs with OpenCode enrollment
- Test: run the deterministic support, capability, and root-validation commands
- Validate: confirm closeout evidence will be concrete and committed-evidence-first
