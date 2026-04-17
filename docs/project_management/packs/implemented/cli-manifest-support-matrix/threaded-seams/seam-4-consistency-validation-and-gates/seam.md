---
seam_id: SEAM-4
seam_slug: consistency-validation-and-gates
status: closed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-4-consistency-validation-and-gates.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-3-closeout.md
  required_threads:
    - THR-01
    - THR-03
    - THR-04
  stale_triggers:
    - row fields or ordering change after landing
    - evidence-note rules change after landing
    - Markdown and JSON diverge after publication
    - repo-gate cost or participation changes after landing
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S99
  status: passed
open_remediations: []
---
# SEAM-4 - Consistency validation and repo-gate enforcement

- **Current planning posture**: closed. The contradiction-validation boundary, repo-gate integration, and seam-exit closeout are all landed and recorded in `governance/seam-4-closeout.md`.

## Seam Brief (Restated)

- **Goal / value**: land one deterministic validation seam so support publication fails loudly instead of drifting through pointer/status contradictions or stale Markdown.
- **Type**: conformance
- **Scope**
  - In:
    - extend validator ownership where it already exists
    - add generator-level contradiction checks against the published row model
    - detect Markdown staleness against `cli_manifests/support_matrix/current.json`
    - decide and record how support-matrix generation participates in `make preflight`
  - Out:
    - redefining the published support row model
    - synthetic future-agent fixture expansion
- **Touch surface**:
  - `crates/xtask/src/codex_validate.rs`
  - `crates/xtask/src/support_matrix.rs`
  - `crates/xtask/tests/*.rs`
  - `Makefile`
- **Verification**:
  - contradiction cases fail deterministically
  - stale Markdown is detected automatically
  - repo-gate participation is explicit and exercised
  - the validator consumes the landed row model from `SEAM-3` rather than re-deriving truth
- **Basis posture**:
  - Currentness: current
  - Upstream closeouts assumed: `../../governance/seam-3-closeout.md`
  - Required threads: `THR-01`, `THR-03`, `THR-04`
  - Stale triggers:
    - row fields or ordering change after landing
    - evidence-note rules change after landing
    - Markdown and JSON diverge after publication
    - repo-gate cost or participation changes after landing
- **Threading constraints**
  - Upstream blockers: none
  - Downstream blocked seams: `SEAM-5`
  - Contracts produced: `C-06`
  - Contracts consumed: `C-01`, `C-04`, `C-05`
  - Canonical contract refs: `docs/specs/unified-agent-api/support-matrix.md`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99`
- **Why this seam needs an explicit exit gate**: downstream fixture work depends on one landed contradiction contract, explicit repo-gate posture, and a recorded Markdown-staleness enforcement boundary before it can freeze future-agent conformance coverage.
- **Expected contracts to publish**: `C-06`
- **Expected threads to publish / advance**: `THR-04`
- **Likely downstream stale triggers**:
  - contradiction classes change
  - repo-gate participation changes
  - Markdown freshness stops consuming the same derived model
  - row-model ownership drifts back into validator-only logic
- **Expected closeout evidence**:
  - landed contradiction checks in `crates/xtask/src/support_matrix.rs` and `crates/xtask/src/codex_validate.rs`
  - landed tests covering contradiction and Markdown-staleness behavior
  - explicit repo-gate integration decision in `Makefile`

## Slice index

- `S1` -> `slice-1-row-model-consistency-checks.md`
- `S2` -> `slice-2-markdown-staleness-and-preflight-integration.md`
- `S3` -> `slice-3-validator-conformance-and-handoff.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-4-closeout.md`
