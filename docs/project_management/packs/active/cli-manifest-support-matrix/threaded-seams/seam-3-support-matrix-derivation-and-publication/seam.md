---
seam_id: SEAM-3
seam_slug: support-matrix-derivation-and-publication
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-3-support-matrix-derivation-and-publication.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
  stale_triggers:
    - support publication semantics change after landing
    - neutral root-intake interfaces change after landing
    - row-field or evidence-note requirements change before execution starts
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
# SEAM-3 - Support-matrix derivation and publication

## Seam Brief (Restated)

- **Goal / value**: land one deterministic support-matrix derivation seam so the repo can publish target-scoped support truth from committed evidence without re-deriving Codex- and Claude-specific logic in multiple places.
- **Type**: capability
- **Scope**
  - In:
    - implement `crates/xtask/src/support_matrix.rs`
    - derive one shared target-scoped row model from versions, pointers, reports, and current metadata
    - render `cli_manifests/support_matrix/current.json`
    - render `docs/specs/unified-agent-api/support-matrix.md` from the same row model
  - Out:
    - contradiction enforcement policy details
    - fixture and golden conformance beyond what is required to stabilize the row model
- **Touch surface**:
  - `crates/xtask/src/support_matrix.rs`
  - `crates/xtask/src/capability_matrix.rs`
  - `cli_manifests/support_matrix/current.json`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `crates/xtask/tests/*.rs`
- **Verification**:
  - one shared row model feeds both JSON and Markdown publication outputs
  - target rows remain primary and do not collapse partial target truth into version-global claims
  - the seam consumes the landed support semantics and shared root-intake handoff without reopening them
  - the owned row-model and projection contract is concrete enough for `SEAM-4` and `SEAM-5` to plan against
- **Basis posture**:
  - Currentness: current
  - Upstream closeouts assumed: `../../governance/seam-1-closeout.md`, `../../governance/seam-2-closeout.md`
  - Required threads: `THR-01`, `THR-02`, `THR-03`
  - Stale triggers:
    - support publication semantics change after landing
    - neutral root-intake interfaces change after landing
    - row-field or evidence-note requirements change before execution starts
- **Threading constraints**
  - Upstream blockers: none
  - Downstream blocked seams: `SEAM-4`, `SEAM-5`
  - Contracts produced: `C-04`, `C-05`
  - Contracts consumed: `C-01`, `C-02`, `C-03`
  - Canonical contract refs: `docs/specs/unified-agent-api/support-matrix.md`, `docs/specs/unified-agent-api/README.md`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99`
- **Why this seam needs an explicit exit gate**: downstream validation and fixture seams both depend on one landed row-model and projection contract before they can freeze contradiction rules, staleness checks, and future-agent conformance coverage.
- **Expected contracts to publish**: `C-04`, `C-05`
- **Expected threads to publish / advance**: `THR-03`
- **Likely downstream stale triggers**:
  - row fields or ordering change
  - evidence-note rules change
  - JSON and Markdown stop consuming the same derived model
  - support-matrix publication paths or projection rules change
- **Expected closeout evidence**:
  - landed `crates/xtask/src/support_matrix.rs`
  - landed `cli_manifests/support_matrix/current.json`
  - landed Markdown projection in `docs/specs/unified-agent-api/support-matrix.md`
  - targeted derivation and publication verification evidence

## Slice index

- `S00` -> `slice-00-row-model-and-projection-contract-definition.md`
- `S1` -> `slice-1-shared-row-derivation.md`
- `S2` -> `slice-2-publication-rendering-and-artifact-write.md`
- `S3` -> `slice-3-support-matrix-conformance-and-handoff.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-3-closeout.md`
