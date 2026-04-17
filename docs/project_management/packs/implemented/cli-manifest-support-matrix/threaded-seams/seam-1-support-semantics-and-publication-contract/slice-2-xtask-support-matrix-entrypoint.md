---
slice_id: S2
seam_id: SEAM-1
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - neutral xtask entrypoint naming changes
    - support publication outputs change
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
contracts_produced: []
contracts_consumed:
  - C-01
open_remediations: []
---
### S2 - xtask support-matrix entrypoint

- **User/system value**: downstream implementation seams inherit one neutral command name and invocation contract instead of baking support publication behind capability-matrix naming.
- **Scope (in/out)**:
  - In: wire `support-matrix` into `crates/xtask/src/main.rs` and define the CLI boundary expected by later implementation seams.
  - Out: row derivation, JSON/Markdown rendering, contradiction validation, or fixture coverage.
- **Acceptance criteria**:
  - `xtask --help` exposes `support-matrix` as the neutral support publication entrypoint.
  - capability-matrix generation and audit commands remain unchanged.
  - the command contract points downstream work at the canonical JSON and Markdown publication targets rather than inventing new surfaces.
- **Dependencies**:
  - `S00` contract-definition output
  - existing xtask command layout in `crates/xtask/src/main.rs`
- **Verification**:
  - inspect the xtask command enum and dispatch logic to confirm the new name and help text match the contract
  - confirm there is no accidental behavior change to capability-matrix commands
  - confirm the help text uses support-matrix terminology consistent with the canonical spec
- **Rollout/safety**:
  - additive CLI tooling change only
  - no runtime `agent_api` behavior change
- **Review surface refs**:
  - `review.md#likely-mismatch-hotspots`
  - `../../seam-1-support-semantics-and-publication-contract.md`

#### S2.T1 - Reserve the neutral command name and help text

- **Outcome**: xtask exposes `support-matrix` with help text that matches the owned support-publication contract.
- **Inputs/outputs**:
  - Inputs: `crates/xtask/src/main.rs`, `S00` contract-definition slice
  - Outputs: xtask subcommand enum/help text updates and any required downstream placeholder wiring
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**: keep the boundary intentionally narrow so later seams can fill in derivation/rendering without renaming the public command.
- **Acceptance criteria**: the new command exists, is neutral, and does not reuse capability-matrix wording for support publication.
- **Test notes**: run xtask help or targeted CLI parsing tests after wiring the command.
- **Risk/rollback notes**: renaming the command later would invalidate downstream planning and stale the contract.

#### S2.T2 - Preserve existing capability-matrix behavior

- **Outcome**: adding the support-matrix entrypoint does not alter capability-matrix generation or audit semantics.
- **Inputs/outputs**:
  - Inputs: existing `CapabilityMatrix` and `CapabilityMatrixAudit` code paths
  - Outputs: unchanged capability-matrix contract with separate support-matrix path
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**: keep the support-matrix boundary additive and explicit in the CLI help/dispatch layout.
- **Acceptance criteria**: downstream users can still rely on the existing capability-matrix commands exactly as before.
- **Test notes**: spot-check help text and dispatch coverage for capability-matrix commands after the change.
- **Risk/rollback notes**: collapsing support and capability flows into one command surface would reintroduce the semantics drift this seam is meant to eliminate.

Checklist:
- Implement: add the neutral xtask entrypoint contract
- Test: verify help/dispatch behavior for new and existing xtask commands
- Validate: confirm command naming matches the canonical support-publication contract
