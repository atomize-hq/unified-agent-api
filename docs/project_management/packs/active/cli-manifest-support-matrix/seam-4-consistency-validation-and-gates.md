---
seam_id: SEAM-4
seam_slug: consistency-validation-and-gates
type: conformance
status: proposed
execution_horizon: next
plan_version: v1
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-01
    - THR-03
    - THR-04
  stale_triggers:
    - derived row fields or ordering change
    - repo-gate cost changes enough to alter `make preflight` integration
    - new contradiction classes appear between pointers, statuses, and published rows
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S99
  status: pending
open_remediations: []
---

# SEAM-4 - Consistency validation and repo-gate enforcement

- **Goal / value**: make support-claim drift fail deterministically instead of silently landing through stale Markdown, mismatched pointers, or contradictory status semantics.
- **Scope**
  - In:
    - extend validator ownership where it already exists
    - add generator-level contradiction checks
    - detect Markdown staleness automatically
    - decide and record how support-matrix generation participates in `make preflight`
  - Out:
    - inventing new runtime behavior
    - redefining the published support row model
- **Primary interfaces**
  - Inputs:
    - derived row model from `SEAM-3`
    - existing validator posture in `crates/xtask/src/codex_validate.rs`
    - repo gate `make preflight`
  - Outputs:
    - deterministic contradiction failures
    - enforced Markdown freshness
    - recorded repo-gate participation for support-matrix generation
- **Key invariants / rules**:
  - pointer advancement and published support rows must not disagree silently
  - Markdown cannot drift from JSON without failing automation
  - the repo gate remains the integration control plane, not ad hoc command bundles
- **Dependencies**
  - Direct blockers:
    - `SEAM-3`
  - Transitive blockers:
    - `SEAM-1`
    - `SEAM-2`
  - Direct consumers:
    - `SEAM-5`
  - Derived consumers:
    - release and review workflows
- **Touch surface**:
  - `crates/xtask/src/codex_validate.rs`
  - `crates/xtask/src/support_matrix.rs`
  - `crates/xtask/tests/*.rs`
  - `Makefile`
- **Verification**:
  - explicit contradiction cases fail deterministically
  - stale Markdown is caught automatically
  - repo-gate participation is documented and exercised in the same ownership surface
  - if this seam **consumes** an upstream contract, verification depends on the accepted row model from `SEAM-3`
- **Canonical contract refs**:
  - `docs/specs/unified-agent-api/support-matrix.md`
- **Risks / unknowns**:
  - Risk: validator logic could fork away from generator logic and recreate two competing truth models.
  - De-risk plan: require validator checks to consume the same shared derived model and make that a review gate.
- **Rollout / safety**:
  - fail-fast and deterministic only
  - repo-gate coupling should stay cheap enough for routine use
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `future` because validator enforcement depends on the final derived model and publication outputs.
  - Which threads matter most: `THR-03`, `THR-04`
  - What the first seam-local review should focus on: whether every contradiction class is phrased as a machine-checkable rule rather than a reviewer judgment call
  - Boundary slice intent: `S00` is unnecessary unless repo-gate ownership or validator authority is still ambiguous after `SEAM-3` lands
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-06`
  - Threads likely to advance: `THR-04`
  - Review-surface areas likely to shift after landing: evidence-to-validation flow and touch-surface map
  - Downstream seams most likely to require revalidation: `SEAM-5`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
