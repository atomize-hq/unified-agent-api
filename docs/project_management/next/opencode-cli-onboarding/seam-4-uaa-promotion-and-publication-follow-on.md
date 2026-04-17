---
seam_id: SEAM-4
seam_slug: uaa-promotion-and-publication-follow-on
type: conformance
status: closed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-3-closeout.md
  required_threads:
    - THR-03
    - THR-04
  stale_triggers:
    - any change in built-in backend support that affects promotion eligibility
    - capability-matrix or universal extension-registry rule changes
    - backend mapping changes that materially alter what OpenCode exposes
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

# SEAM-4 - UAA promotion and publication follow-on

- **Goal / value**: produce an explicit, reviewable answer to what OpenCode support remains
  backend-specific and what, if anything, is justified for UAA promotion once backend behavior is
  concrete.
- **Scope**
  - In:
    - review the backend seam's actual capability and extension surface
    - distinguish backend support, backend-specific extension coverage, and candidate
      `agent_api.*` promotion
    - identify any required follow-on pack for canonical spec or capability-matrix changes
    - preserve explicit non-promotion paths for backend-specific or unstable behavior
  - Out:
    - editing canonical specs or capability matrices in this extraction pass
    - reopening wrapper or backend scope except through explicit blocker escalation
    - treating backend completion as automatic universal promotion
- **Primary interfaces**
  - Inputs:
    - `THR-03`
    - `docs/project_management/next/opencode-cli-onboarding/governance/seam-3-closeout.md`
    - `docs/specs/opencode-agent-api-backend-contract.md`
    - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
    - `docs/specs/unified-agent-api/extensions-spec.md`
  - Outputs:
    - promotion-review recommendation
    - explicit follow-on pack requirement or no-promotion decision
    - `THR-04` handoff into pack closeout and future work
- **Key invariants / rules**:
  - backend support and universal support remain distinct publication states
  - new `agent_api.*` promotion must satisfy the canonical multi-backend rule unless allowlisted
  - review-only findings that imply spec or matrix edits must route to a separate execution pack
  - backend-specific fallback behavior must remain explicit even when no promotion is justified
- **Dependencies**
  - Direct blockers:
    - `SEAM-3`
  - Transitive blockers:
    - `SEAM-1`, `SEAM-2`
  - Direct consumers:
    - pack closeout and future execution packs
  - Derived consumers:
    - capability-matrix maintenance and future third-agent onboarding work
- **Touch surface**:
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
  - future `docs/project_management/**` follow-on packs
  - `docs/project_management/next/opencode-cli-onboarding/`
- **Verification**:
  - if this seam **consumes** upstream contracts, verification may depend on accepted backend
    closeout evidence from `SEAM-3`
  - seam-local review should prove every promotion recommendation is grounded in concrete backend
    behavior and the existing universal promotion rules
  - when no promotion is justified, verification should explicitly record the bounded non-promotion
    answer rather than leaving ambiguity for future reviewers
- **Canonical contract refs**:
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- **Risks / unknowns**:
  - Risk: the promotion seam becomes a cleanup bucket for backend uncertainty instead of a bounded
    review artifact.
  - De-risk plan: require concrete backend evidence via `THR-03` and route spec changes to a
    separate follow-on pack.
  - Risk: reviewers may read "OpenCode backend support" as "OpenCode is now universally promoted."
  - De-risk plan: make backend-specific fallback behavior and no-promotion outcomes first-class
    outputs.
- **Rollout / safety**:
  - review-only at extraction time
  - no canonical spec edits without a separate approved follow-on execution pack
  - preserve explicit backend-specific publication if universal promotion is premature
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `future` because the seam has landed,
    published its closeout-backed recommendation and `THR-04` handoff, and no additional queued
    seam remains in this pack.
  - Which threads matter most: `THR-03`, `THR-04`
  - What the first seam-local review should focus on: whether backend evidence is concrete, whether
    multi-backend promotion rules are satisfied, and whether the output is an explicit promotion
    recommendation or an explicit no-promotion follow-on
  - Boundary slice intent: reserve `S00` only if seam-local planning must define a contract-review
    boundary before promotion-analysis slices begin
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-07`
  - Threads likely to advance: `THR-04`
  - Review-surface areas likely to shift after landing: capability/publication expectations and the
    contract/dependency flow
  - Downstream seams most likely to require revalidation: future OpenCode follow-on packs and any
    later universal capability work
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in
    pre-exec verification for the producing seam.
