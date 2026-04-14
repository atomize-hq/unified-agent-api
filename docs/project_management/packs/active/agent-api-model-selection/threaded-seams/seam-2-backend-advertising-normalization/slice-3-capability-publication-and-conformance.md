---
slice_id: S3
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
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
    - regenerate `docs/specs/unified-agent-api/capability-matrix.md` in the same change as advertising flips
    - treat stale matrix diffs and second-parser drift as merge blockers
  - Out:
    - backend runtime rejection fixtures and argv-order tests (SEAM-3/4/5)
- **Acceptance criteria**:
  - `docs/specs/unified-agent-api/capability-matrix.md` is regenerated in the same change that flips `agent_api.config.model.v1` advertising.
  - The generated matrix posture matches the final built-in `capabilities()` posture for Codex and Claude Code.
  - Merge validation includes a focused review that raw parsing of `agent_api.config.model.v1` still exists only in `crates/agent_api/src/backend_harness/normalize.rs`.
  - SEAM-5 can consume the published matrix and the final capability posture without special-case interpretation.
- **Dependencies**:
  - S2
  - `THR-03`
  - `C-08`
  - `MS-C08`
  - the deterministic mapping outputs from `MS-C06` and `MS-C07` must already be present in the integration change that lands this slice
- **Verification**:
  - `cargo run -p xtask -- capability-matrix`
  - `rg -n "agent_api\\.config\\.model\\.v1" crates/agent_api/src` classification
- **Rollout/safety**: never hand-edit the matrix; commit the xtask output with the advertising flip.
- **Review surface refs**: `../../review_surfaces.md` (R3)

#### S3.T1 - Regenerate and review the capability matrix in the same advertising-flip change

- **Outcome**: The generated matrix publishes the final built-in model-selection posture without drift from runtime capability code.
- **Thread/contract refs**: `THR-03`, `C-08`, `C-05`
- **Inputs/outputs**:
  - Input:
    - `docs/specs/unified-agent-api/capability-matrix.md`
    - `crates/agent_api/src/backends/codex/backend.rs`
    - `crates/agent_api/src/backends/claude_code/backend.rs`
  - Output:
    - `docs/specs/unified-agent-api/capability-matrix.md`
- **Implementation notes**:
  - Run `cargo run -p xtask -- capability-matrix` in the same branch/PR that flips built-in advertising.
  - Review the diff specifically for the `agent_api.config.*` bucket and the `agent_api.config.model.v1` row.
  - Do not queue matrix regeneration as a later cleanup task; stale output blocks merge by contract.
- **Acceptance criteria**:
  - The generated matrix diff matches the final `capabilities()` posture of both built-in backends.
  - No manual edits are required after the xtask run.
- **Test notes**:
  - Pair with `cargo test -p agent_api --features codex,claude_code` before merge.
- **Risk/rollback notes**:
  - Low: generated artifact only, but required for truthful publication.

Checklist:
- Implement: run `cargo run -p xtask -- capability-matrix`
- Validate: review diff for the `agent_api.config.*` bucket
- Cleanup: commit generated changes with advertising flip

#### S3.T2 - Capture conformance evidence (single-parser + truthful advertising)

- **Outcome**: The final integration review has explicit, repeatable checks for the two seam-critical invariants: one raw parser and one truthful published capability posture.
- **Thread/contract refs**: `THR-03`, `THR-02`, `C-09`
- **Inputs/outputs**:
  - Input:
    - `crates/agent_api/src/backend_harness/normalize.rs`
    - `crates/agent_api/src/backends/codex/`
    - `crates/agent_api/src/backends/claude_code/`
    - `docs/project_management/packs/active/agent-api-model-selection/threading.md`
  - Output:
    - merge validation evidence for the change set (review + command output), plus any follow-up test updates if the check fails
- **Implementation notes**:
  - Review the repo search for `agent_api.config.model.v1` and classify every hit:
    - allowed: shared constant definitions, capability advertising, tests, docs
    - forbidden: a second raw parse/validation site outside `crates/agent_api/src/backend_harness/normalize.rs`
  - Confirm the public advertising flip matches the deterministic-support evidence from SEAM-3 / SEAM-4:
    - Codex: exec/resume map, fork safe rejection
    - Claude Code: print exec/resume/fork map and no `--fallback-model` implication
  - If review reveals a mismatch, fix the code or keep advertising false; do not waive the invariant.
- **Acceptance criteria**:
  - No forbidden raw parser sites remain.
  - Public advertising is enabled only when the downstream mapping evidence is already in the same integration stack.
- **Test notes**:
  - Suggested validation command:
    - `rg -n "agent_api\\.config\\.model\\.v1" crates/agent_api/src docs/specs/unified-agent-api`
- **Risk/rollback notes**:
  - High if skipped: a second parser or early advertising flip would create spec-visible drift.

Checklist:
- Validate: `rg` and classify every match
- Validate: confirm deterministic support across run flows
