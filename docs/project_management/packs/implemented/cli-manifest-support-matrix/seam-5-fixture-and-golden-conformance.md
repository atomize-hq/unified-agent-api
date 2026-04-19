---
seam_id: SEAM-5
seam_slug: fixture-and-golden-conformance
type: conformance
status: closed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-4-closeout.md
  required_threads:
    - THR-02
    - THR-03
    - THR-04
    - THR-05
  stale_triggers:
    - shared normalization starts branching on known agent names
    - row ordering changes without golden updates
    - contradiction rules expand without fixture coverage
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

# SEAM-5 - Neutral fixture and golden conformance

- **Current planning posture**: closed. The neutral fixture proof, coupled golden regressions, and seam-exit closeout are all landed and recorded in `governance/seam-5-closeout.md`.

- **Goal / value**: prove the support-matrix pipeline stays neutral, reproducible, and regression-resistant as current and future agent roots evolve.
- **Scope**
  - In:
    - Codex fixture coverage
    - Claude fixture coverage
    - synthetic third-agent-shaped fixture coverage
    - golden tests for JSON and Markdown outputs
    - ordering and evidence-note regression coverage
  - Out:
    - introducing a real third agent in phase 1
    - redefining support semantics or validator policy
- **Primary interfaces**
  - Inputs:
    - shared normalization contracts from `SEAM-2`
    - publication model from `SEAM-3`
    - contradiction rules from `SEAM-4`
  - Outputs:
    - durable regression suites
    - future-agent-shaped neutrality proof
    - golden outputs for JSON and Markdown publication
- **Key invariants / rules**:
  - tests must cover Codex, Claude, and at least one synthetic future-agent-shaped root
  - golden outputs must derive from the same model used in production code
  - fixture coverage must distinguish "not attempted", "unsupported", and "intentionally partial" states
- **Dependencies**
  - Direct blockers:
    - `SEAM-3`
    - `SEAM-4`
  - Transitive blockers:
    - `SEAM-1`
    - `SEAM-2`
  - Direct consumers:
    - none
  - Derived consumers:
    - future agent-onboarding work
- **Touch surface**:
  - `crates/xtask/tests/*.rs`
  - fixture roots under `cli_manifests/**` test surfaces
  - `cli_manifests/support_matrix/current.json`
  - `docs/specs/unified-agent-api/support-matrix.md`
- **Verification**:
  - Codex and Claude checked-in fixtures pass through the shared core
  - a synthetic third-agent-shaped root passes without shared-core agent-name branching
  - JSON and Markdown goldens catch ordering and stale-render regressions
  - if this seam **consumes** upstream contracts, verification may depend on accepted upstream evidence from `SEAM-3` and `SEAM-4`
- **Canonical contract refs**:
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- **Risks / unknowns**:
  - Risk: shared-core neutrality can erode slowly if tests only exercise current agent names and shapes.
  - De-risk plan: require at least one future-agent-shaped fixture in the same suite as Codex and Claude fixtures.
- **Rollout / safety**:
  - regression-only seam
  - should strengthen, not expand, publication surface area
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `future` because `SEAM-5` has now landed and closed and there are no remaining seams in the forward window for this pack.
  - Which threads matter most: `THR-05`
  - What the first seam-local review should focus on: whether the fixture matrix is broad enough to protect neutrality and not just current repo shapes
  - Boundary slice intent: `S00` is unnecessary unless conformance ownership between golden outputs and fixture suites is still ambiguous after `SEAM-4`
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-07`
  - Threads likely to advance: `THR-05`
  - Review-surface areas likely to shift after landing: evidence-to-validation flow and touch-surface map
  - Downstream seams most likely to require revalidation: future manifest-onboarding seams
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
