---
seam_id: SEAM-2
seam_slug: shared-wrapper-normalization
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-2-shared-wrapper-normalization.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - support publication contract changes after landing
    - manifest root file layout changes
    - wrapper coverage semantics diverge between Codex and Claude roots
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S99
  status: pending
open_remediations: []
---
# SEAM-2 - Shared wrapper normalization and agent-root intake

## Seam Brief (Restated)

- **Goal / value**: land one neutral shared normalization and root-intake seam so publication and validation work can reuse the same cross-agent core instead of copying Codex- and Claude-specific logic.
- **Type**: integration
- **Scope**
  - In:
    - extract shared normalization helpers into `crates/xtask/src/wrapper_coverage_shared.rs`
    - define a neutral root-intake contract for versions, pointers, current metadata, and coverage reports
    - keep `crates/xtask/src/codex_wrapper_coverage.rs` and `crates/xtask/src/claude_wrapper_coverage.rs` as thin adapters
  - Out:
    - final support row derivation
    - JSON or Markdown rendering
    - contradiction validation and repo-gate enforcement
- **Touch surface**:
  - `crates/xtask/src/codex_wrapper_coverage.rs`
  - `crates/xtask/src/claude_wrapper_coverage.rs`
  - `crates/xtask/src/wrapper_coverage_shared.rs`
  - `crates/xtask/tests/c2_spec_wrapper_coverage.rs`
  - `cli_manifests/codex/**`
  - `cli_manifests/claude_code/**`
- **Verification**:
  - the shared module preserves current normalization behavior for Codex and Claude roots
  - the root-intake contract is shape-driven rather than agent-name-driven
  - adapter modules own only root-specific defaults, imports, and loading glue after extraction
  - the owned seam contract is concrete enough for `SEAM-3` through `SEAM-5` to plan against without reopening the shared-vs-adapter boundary
- **Basis posture**:
  - Currentness: current
  - Upstream closeouts assumed: `../../governance/seam-1-closeout.md`
  - Required threads: `THR-01`, `THR-02`
  - Stale triggers:
    - support publication contract changes after landing
    - manifest root file layout changes
    - wrapper coverage semantics diverge between Codex and Claude roots
- **Threading constraints**
  - Upstream blockers: none
  - Downstream blocked seams: `SEAM-3`, `SEAM-4`, `SEAM-5`
  - Contracts produced: `C-02`, `C-03`
  - Contracts consumed: `C-01`
  - Canonical contract refs: `docs/specs/codex-wrapper-coverage-generator-contract.md`, `docs/specs/unified-agent-api/support-matrix.md`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99`
- **Why this seam needs an explicit exit gate**: downstream publication, validation, and fixture seams all depend on the shared normalization boundary and neutral root-intake contract being explicit before they freeze their own models.
- **Expected contracts to publish**: `C-02`, `C-03`
- **Expected threads to publish / advance**: `THR-02`
- **Likely downstream stale triggers**:
  - shared normalization responsibilities move back into agent-specific modules
  - root-intake shapes for versions, pointers, current metadata, or coverage reports change
  - shared helpers stop being future-agent-shaped and reintroduce agent-name branching
- **Expected closeout evidence**:
  - landed `wrapper_coverage_shared.rs` extraction
  - thin Codex and Claude adapter updates
  - targeted wrapper-coverage verification evidence
  - explicit downstream revalidation notes for `SEAM-3` through `SEAM-5`

## Slice index

- `S00` -> `slice-00-shared-normalization-and-root-intake-contract-definition.md`
- `S1` -> `slice-1-shared-normalization-extraction.md`
- `S2` -> `slice-2-neutral-root-intake-and-adapter-slimming.md`
- `S3` -> `slice-3-wrapper-coverage-conformance-and-handoff.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
