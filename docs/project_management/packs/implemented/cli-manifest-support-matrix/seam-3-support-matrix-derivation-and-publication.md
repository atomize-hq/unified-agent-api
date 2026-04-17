---
seam_id: SEAM-3
seam_slug: support-matrix-derivation-and-publication
type: capability
status: closed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
    - governance/seam-2-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
  stale_triggers:
    - SEAM-1 changes support-layer semantics
    - SEAM-2 changes neutral root-intake interfaces
    - publication row fields need to distinguish new support states
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

# SEAM-3 - Support-matrix derivation and publication

- **Current planning posture**: closed. The row-model contract, derivation path, publication outputs, conformance evidence, and seam-exit closeout are all landed and recorded in `governance/seam-3-closeout.md`.
- **Goal / value**: publish support truth from committed evidence using one shared derived row model that feeds both the JSON artifact and the Markdown projection.
- **Scope**
  - In:
    - implement `crates/xtask/src/support_matrix.rs`
    - derive target-scoped rows from versions, pointers, reports, and current metadata
    - render `cli_manifests/support_matrix/current.json`
    - render `docs/specs/unified-agent-api/support-matrix.md`
  - Out:
    - contradiction enforcement policy details
    - fixture/golden conformance suites beyond what is needed to stabilize the model
- **Primary interfaces**
  - Inputs:
    - shared root-intake outputs from `SEAM-2`
    - publication semantics from `SEAM-1`
    - committed manifest roots under `cli_manifests/*`
  - Outputs:
    - deterministic target rows
    - machine-readable current support artifact
    - Markdown projection from the same row model
- **Key invariants / rules**:
  - target rows remain the primitive
  - JSON and Markdown consume the same derived model
  - generator fails loudly on contradictory manifest inputs instead of projecting a misleading matrix
  - evidence notes explicitly call out intentionally partial rows
- **Dependencies**
  - Direct blockers:
    - `SEAM-1`
    - `SEAM-2`
  - Transitive blockers:
    - none
  - Direct consumers:
    - `SEAM-4`
    - `SEAM-5`
  - Derived consumers:
    - maintainers reading support publication
- **Touch surface**:
  - `crates/xtask/src/support_matrix.rs`
  - `crates/xtask/src/capability_matrix.rs`
  - `cli_manifests/support_matrix/current.json`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `cli_manifests/*/versions/*.json`
  - `cli_manifests/*/pointers/**`
  - `cli_manifests/*/reports/**`
  - `cli_manifests/*/current.json`
- **Verification**:
  - JSON and Markdown outputs are deterministic projections from the same row model
  - target rows do not collapse partial target truth into version-global claims
  - if this seam **consumes** an upstream contract, verification depends on accepted upstream support semantics and shared root-intake behavior
  - if this seam **produces** an owned contract, verification is the derivation/rendering model becoming concrete enough for validator and conformance planning rather than requiring final published acceptance up front
- **Canonical contract refs**:
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/README.md`
- **Risks / unknowns**:
  - Risk: the derived row model could accidentally encode current agent quirks instead of a neutral schema.
  - De-risk plan: keep derivation single-pass and hand it to SEAM-5 for future-agent-shaped fixture verification.
- **Rollout / safety**:
  - additive publication only
  - capability matrix remains untouched except for reused rendering patterns
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `future` because the seam has landed and closed, and downstream promotion has moved validator work into the active horizon.
  - Which threads matter most: `THR-02`, `THR-03`
  - What the first seam-local review should focus on: whether the row fields, ordering, and evidence-note rules are enough for both publication and validator consumers
  - Boundary slice intent: reserve `S00` for row-model and projection contract-definition work before the derivation and renderer slices start changing production artifacts
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-04`, `C-05`
  - Threads likely to advance: `THR-03`
  - Review-surface areas likely to shift after landing: support publication workflow and evidence-to-validation flow
  - Downstream seams most likely to require revalidation: `SEAM-4`, `SEAM-5`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
