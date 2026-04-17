---
slice_id: S1
seam_id: SEAM-2
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - duplicated normalization helpers diverge before extraction
    - shared helper signatures drift from the contract
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
  - THR-02
contracts_produced: []
contracts_consumed:
  - C-02
  - C-03
open_remediations: []
---
### S1 - Shared normalization extraction

- **User/system value**: duplicated Codex and Claude normalization logic moves into one neutral module that future seams can consume without re-implementing it.
- **Scope (in/out)**:
  - In: extract common rules parsing, platform inversion, scope normalization, and sorting helpers into `wrapper_coverage_shared.rs`.
  - Out: publication rendering, contradiction validation, or root-intake consumer code for later seams.
- **Acceptance criteria**:
  - `crates/xtask/src/wrapper_coverage_shared.rs` owns the shared normalization helpers currently duplicated across the two wrapper-coverage modules.
  - `crates/xtask/src/codex_wrapper_coverage.rs` and `crates/xtask/src/claude_wrapper_coverage.rs` preserve existing CLI behavior while delegating shared work.
  - the shared helper signatures stay neutral and do not branch on current agent names.
- **Dependencies**:
  - `S00` contract-definition output
  - current duplicate logic in `crates/xtask/src/codex_wrapper_coverage.rs` and `crates/xtask/src/claude_wrapper_coverage.rs`
- **Verification**:
  - run targeted wrapper-coverage tests after extraction
  - inspect the shared module surface to confirm it matches the seam contract
  - confirm the adapter modules are thinner without behavior drift
- **Rollout/safety**:
  - behavior-preserving extraction only
  - no published support artifact changes
- **Review surface refs**:
  - `review.md#likely-mismatch-hotspots`
  - `../../threading.md`

#### S1.T1 - Extract common normalization helpers

- **Outcome**: the duplicated helper set becomes one shared module.
- **Inputs/outputs**:
  - Inputs: current Codex and Claude wrapper-coverage modules
  - Outputs: `crates/xtask/src/wrapper_coverage_shared.rs`, slimmer adapter modules
- **Thread/contract refs**: `THR-02`, `C-02`
- **Implementation notes**: keep shared helpers focused on normalization and sorting only; do not pull report or validator concerns forward.
- **Acceptance criteria**: a reviewer can diff the old duplicate logic against the new shared module and see equivalent behavior.
- **Test notes**: run the targeted wrapper-coverage tests that currently guard the generator behavior.
- **Risk/rollback notes**: over-extracting here will make later seams undo the shared boundary.

#### S1.T2 - Preserve existing wrapper-coverage entrypoint behavior

- **Outcome**: the extraction stays invisible to current Codex and Claude wrapper-coverage commands.
- **Inputs/outputs**:
  - Inputs: existing CLI args and output paths in both wrapper-coverage modules
  - Outputs: thin adapters that still honor current command contracts
- **Thread/contract refs**: `THR-02`, `C-02`
- **Implementation notes**: adapters should keep root-specific defaults and crate imports only.
- **Acceptance criteria**: current wrapper-coverage commands still parse, normalize, and write output exactly as before.
- **Test notes**: spot-check command behavior and any targeted CLI or module tests already in the repo.
- **Risk/rollback notes**: changing the public wrapper-coverage contract here would break current evidence workflows and invalidate the seam scope.

Checklist:
- Implement: extract the duplicated normalization helpers into the shared module
- Test: re-run targeted wrapper-coverage verification
- Validate: confirm the adapters stay thin and behavior-preserving
