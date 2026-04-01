---
slice_id: S3
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: next
status: decomposed
plan_version: v1
basis:
  currentness: provisional
  basis_ref: seam.md#basis
  stale_triggers: []
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-03
contracts_produced:
  - C-08
contracts_consumed:
  - C-05
open_remediations: []
candidate_subslices: []
---
### S3 - Capability publication + conformance gate

- **User/system value**: makes the public capability inventory truthful and reviewable by regenerating the capability matrix in lockstep with advertising changes.
- **Scope (in/out)**:
  - In:
    - regenerate `docs/specs/universal-agent-api/capability-matrix.md` in the same change as advertising flips
    - treat stale matrix diffs and second-parser drift as merge blockers
  - Out:
    - backend runtime rejection fixtures and argv-order tests (SEAM-3/4/5)
- **Acceptance criteria**:
  - the generated matrix posture matches runtime advertising for Codex + Claude Code
  - single-parser invariant remains true
- **Dependencies**: S2, `THR-03`, `C-08`
- **Verification**:
  - `cargo run -p xtask -- capability-matrix`
  - `rg -n "agent_api\\.config\\.model\\.v1" crates/agent_api/src` classification
- **Rollout/safety**: never hand-edit the matrix; commit the xtask output with the advertising flip.
- **Review surface refs**: `../../review_surfaces.md` (R3)

#### S3.T1 - Regenerate and review the capability matrix in the same change

- **Outcome**: published docs reflect the exact backend capability posture.
- **Thread/contract refs**: `THR-03`, `C-08`, `C-05`
- **Acceptance criteria**: the `agent_api.config.model.v1` row matches both backends' `capabilities()` output.

Checklist:
- Implement: run `cargo run -p xtask -- capability-matrix`
- Validate: review diff for the `agent_api.config.*` bucket
- Cleanup: commit generated changes with advertising flip

#### S3.T2 - Capture conformance evidence (single-parser + truthful advertising)

- **Outcome**: merge review has explicit evidence for the two seam-critical invariants.
- **Thread/contract refs**: `THR-03`, `THR-02`, `C-09`
- **Acceptance criteria**: no new parse sites; advertising only true when deterministic outcomes exist.

Checklist:
- Validate: `rg` and classify every match
- Validate: confirm deterministic support across run flows
