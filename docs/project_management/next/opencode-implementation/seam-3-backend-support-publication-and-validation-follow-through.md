---
seam_id: SEAM-3
seam_slug: backend-support-publication-and-validation-follow-through
type: conformance
status: closed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - ../opencode-cli-onboarding/governance/seam-4-closeout.md
  required_threads:
    - THR-04
    - THR-05
    - THR-06
    - THR-07
  stale_triggers:
    - any `THR-04` revalidation trigger fires
    - support-matrix semantics, capability-inventory semantics, or committed root-set assumptions drift
    - OpenCode backend evidence starts implying new universal support or promotion pressure
    - passthrough visibility is no longer explicit or support rows collapse multiple layers together
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

# SEAM-3 - Backend support publication and validation follow-through

- **Goal / value**: finish the bounded publication work required after OpenCode code lands so the
  repo can validate and publish OpenCode support truth without implying UAA promotion.
- **Scope**
  - In:
    - extend committed root or backend enumerations so OpenCode participates in support and
      capability inventory generation
    - keep support-matrix, capability-matrix, and root validation flows aligned with OpenCode
      evidence
    - preserve the four support layers explicitly: manifest support, backend support, UAA unified
      support, and passthrough visibility
    - publish the explicit no-promotion posture unless inherited stale triggers reopen that boundary
  - Out:
    - adding new universal capabilities for OpenCode
    - reopening UAA promotion as active work under the current evidence basis
    - turning publication follow-through into generic future-agent framework work
- **Primary interfaces**
  - Inputs:
    - `THR-04`
    - `THR-05`
    - `THR-06`
    - `docs/specs/unified-agent-api/support-matrix.md`
    - `docs/specs/opencode-cli-manifest-contract.md`
    - `docs/specs/opencode-agent-api-backend-contract.md`
    - current `crates/xtask/src/support_matrix.rs` and `crates/xtask/src/capability_matrix.rs`
  - Outputs:
    - bounded OpenCode participation in support publication and capability inventory outputs
    - explicit evidence that backend support and passthrough visibility do not imply UAA promotion
    - `THR-07`
- **Key invariants / rules**:
  - manifest support, backend support, UAA unified support, and passthrough visibility stay
    distinct
  - `support-matrix` remains the canonical publication truth; `capability-matrix` remains a
    separate backend inventory
  - hard-coded current-agent or built-in-backend sets may be extended for OpenCode, but those
    changes must stay bounded to OpenCode landing rather than generic future-agent machinery
  - stale triggers from the closed onboarding pack remain the only normal route back into UAA
    promotion work
- **Dependencies**
  - Direct blockers:
    - `SEAM-2`
  - Transitive blockers:
    - `SEAM-1`, `THR-04`
  - Direct consumers:
    - pack closeout and future stale-trigger revalidation packs
  - Derived consumers:
    - future OpenCode publication maintenance and promotion review if stale triggers later fire
- **Touch surface**:
  - `crates/xtask/src/support_matrix.rs`
  - `crates/xtask/src/support_matrix/**`
  - `crates/xtask/src/capability_matrix.rs`
  - `cli_manifests/support_matrix/current.json`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- **Verification**:
  - support publication drift checks must pass under `cargo run -p xtask -- support-matrix --check`
  - capability inventory regeneration must include OpenCode without changing the meaning of support
    publication
  - OpenCode root validation remains required via `cargo run -p xtask -- codex-validate --root cli_manifests/opencode`
  - if this seam **consumes** upstream contracts, verification depends on landed `SEAM-1` and
    `SEAM-2` evidence rather than planning-only claims
- **Canonical contract refs**:
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- **Risks / unknowns**:
  - Risk: current publication tooling remains hard-coded to Codex and Claude, making OpenCode drift
    invisible unless this seam owns the bounded update.
  - De-risk plan: keep OpenCode root and backend enumeration work explicit inside this seam.
  - Risk: support rows or evidence notes may accidentally imply UAA support because backend support
    exists.
  - De-risk plan: keep the four support layers explicit and preserve passthrough visibility as a
    separate explanation surface.
  - Risk: capability inventory updates may be misread as promotion work.
  - De-risk plan: keep capability inventory and support publication clearly separated in both code
    and docs.
- **Rollout / safety**:
  - treat publication work as evidence projection, not as runtime truth creation
  - no active promotion lane exists here unless inherited stale triggers fire
  - keep all changes deterministic and rooted in committed manifest/backend evidence
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is now `active` because `SEAM-2` has
    published `THR-06`, no queued seam remains behind it in this pack, and the only forward work
    left is the bounded publication follow-through this seam owns.
  - Which threads matter most: `THR-04`, `THR-06`, `THR-07`
  - What the first seam-local review should focus on: committed root/backend enumeration, support
    row derivation, passthrough visibility notes, explicit non-promotion posture, and keeping
    support publication separate from capability inventory
  - Boundary slice intent: `S00` now defines the publication contract and support-layer baseline
    before conformance slices update the committed outputs
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-04`
  - Threads likely to advance: `THR-07`
  - Review-surface areas likely to shift after landing: the support-layer publication boundary and
    repo touch-surface map
  - Downstream seams most likely to require revalidation: future stale-trigger-driven promotion
    packs only
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in
    pre-exec verification for the producing seam.
