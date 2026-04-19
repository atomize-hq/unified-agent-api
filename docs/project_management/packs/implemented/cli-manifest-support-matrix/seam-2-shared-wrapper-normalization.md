---
seam_id: SEAM-2
seam_slug: shared-wrapper-normalization
type: integration
status: closed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - SEAM-1 changes support-layer field names
    - manifest root file layout changes
    - wrapper coverage semantics diverge between Codex and Claude roots
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

# SEAM-2 - Shared wrapper normalization and agent-root intake

- **Current planning posture**: closed. The shared normalization boundary, root-intake contract, conformance evidence, and seam-exit closeout are recorded in `governance/seam-2-closeout.md`.
- **Goal / value**: extract one neutral shared seam from the existing wrapper-coverage loaders so support publication and validation can operate on a reusable cross-agent core.
- **Scope**
  - In:
    - extract shared normalization helpers into `crates/xtask/src/wrapper_coverage_shared.rs`
    - keep Codex and Claude modules thin and root-specific
    - define neutral loading of versions, pointers, current metadata, and coverage reports from each agent root
  - Out:
    - final support row derivation
    - JSON/Markdown rendering
    - validator contradiction rules
- **Primary interfaces**
  - Inputs:
    - `cli_manifests/codex/**`
    - `cli_manifests/claude_code/**`
    - existing loaders in `crates/xtask/src/codex_wrapper_coverage.rs` and `crates/xtask/src/claude_wrapper_coverage.rs`
  - Outputs:
    - shared normalization helpers
    - neutral root-intake interface consumed by the future support-matrix module
    - thin agent adapters that preserve current root-specific behavior
- **Key invariants / rules**:
  - no giant generic parity framework appears in phase 1
  - shared logic owns normalization rules; adapters own only manifest loading, default path selection, and agent-specific imports
  - shared core must be future-agent-shaped rather than hard-coded to current agent names
- **Dependencies**
  - Direct blockers:
    - `SEAM-1`
  - Transitive blockers:
    - none
  - Direct consumers:
    - `SEAM-3`, `SEAM-4`, `SEAM-5`
  - Derived consumers:
    - future agent-manifest onboarding work
- **Touch surface**:
  - `crates/xtask/src/codex_wrapper_coverage.rs`
  - `crates/xtask/src/claude_wrapper_coverage.rs`
  - `crates/xtask/src/wrapper_coverage_shared.rs`
  - `cli_manifests/codex/**`
  - `cli_manifests/claude_code/**`
- **Verification**:
  - Codex and Claude coverage roots still normalize through equivalent behavior after extraction
  - the shared seam accepts root-shaped inputs rather than special-casing current agent names
  - if this seam **consumes** an upstream contract, verification depends on `SEAM-1` having pinned the publication semantics and authority model
  - if this seam **produces** an owned contract, verification is the shared seam becoming concrete enough for SEAM-3/4/5 planning and implementation rather than requiring final publication artifacts to exist
- **Canonical contract refs**:
  - `docs/specs/codex-wrapper-coverage-generator-contract.md`
  - `docs/specs/unified-agent-api/support-matrix.md`
- **Risks / unknowns**:
  - Risk: hidden Codex-first assumptions in current wrapper-coverage code could leak into the shared seam.
  - De-risk plan: pin shared-vs-adapter responsibilities and require a future-agent-shaped fixture in downstream conformance.
- **Rollout / safety**:
  - behavior-preserving extraction only
  - no publication claims change until downstream seams land
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `future` because it has left the forward planning window after publishing the shared normalization and root-intake handoff for downstream seams.
  - Which threads matter most: `THR-01`, `THR-02`
  - What the first seam-local review should focus on: whether the proposed shared interface truly removes duplicated normalization rules without smuggling in agent-name assumptions
  - Boundary slice intent: reserve `S00` for shared interface freezing if seam-local planning discovers unresolved ambiguity in the shared-vs-adapter boundary
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-02`, `C-03`
  - Threads likely to advance: `THR-02`
  - Review-surface areas likely to shift after landing: evidence-to-validation flow and touch-surface map
  - Downstream seams most likely to require revalidation: `SEAM-3`, `SEAM-4`, `SEAM-5`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
