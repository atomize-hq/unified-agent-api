---
seam_id: SEAM-3
seam_slug: backend-support-publication-and-validation-follow-through
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-3-backend-support-publication-and-validation-follow-through.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
    - ../../../opencode-cli-onboarding/governance/seam-4-closeout.md
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
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S99
  status: pending
open_remediations: []
---
# SEAM-3 - Backend support publication and validation follow-through

## Seam Brief (Restated)

- **Goal / value**: finish the bounded publication work required after OpenCode code lands so the
  repo can validate and publish OpenCode support truth without implying UAA promotion.
- **Type**: conformance
- **Scope**
  - In:
    - extend committed root and backend enumerations so OpenCode participates in support and
      capability inventory generation
    - keep support-matrix, capability-matrix, and root validation flows aligned with landed
      OpenCode manifest and backend evidence
    - preserve the four support layers explicitly: manifest support, backend support, UAA unified
      support, and passthrough visibility
    - publish the explicit no-promotion posture unless inherited stale triggers reopen that
      boundary
  - Out:
    - adding new universal capabilities for OpenCode
    - reopening UAA promotion as active work under the current evidence basis
    - turning publication follow-through into generic future-agent framework work
- **Touch surface**:
  - `crates/xtask/src/support_matrix.rs`
  - `crates/xtask/src/support_matrix/**`
  - `crates/xtask/src/capability_matrix.rs`
  - `cli_manifests/support_matrix/current.json`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- **Verification**:
  - `review.md` is the authoritative pre-exec artifact and must show that support publication,
    capability inventory, passthrough visibility, and non-promotion posture are concrete
  - consumed wrapper, manifest, and backend contracts must remain grounded in the landed `SEAM-1`
    and `SEAM-2` closeouts plus revalidated `THR-04`, `THR-05`, and `THR-06`
  - publication planning stays deterministic and committed-evidence-first; live-provider smoke is
    not a default done-ness path for this seam
- **Canonical contract refs**:
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
  - `docs/specs/opencode-agent-api-backend-contract.md`
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
    - `../../governance/seam-2-closeout.md`
    - `../../../opencode-cli-onboarding/governance/seam-4-closeout.md`
  - Required threads: `THR-04`, `THR-05`, `THR-06`, `THR-07`
  - Stale triggers:
    - any inherited `THR-04` revalidation trigger fires
    - support-matrix semantics, capability-inventory semantics, or committed root-set assumptions drift
    - OpenCode backend evidence starts implying new universal support or promotion pressure
    - passthrough visibility is no longer explicit or support rows collapse multiple layers together
- **Threading constraints**
  - Upstream blockers: none; `SEAM-1` and `SEAM-2` closeouts plus `THR-04`, `THR-05`, and
    `THR-06` are now current inputs.
  - Downstream blocked seams: none inside this pack; pack closeout and stale-trigger-driven
    follow-on work depend on this seam's closeout.
  - Contracts produced: `C-04`
  - Contracts consumed: `C-01`, `C-02`, `C-03`, `C-07`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99`
- **Why this seam needs an explicit exit gate**: pack closeout and any later stale-trigger
  reopening need one closeout-backed record of what OpenCode publication changed, what remained
  backend-specific, and why those changes still do not imply UAA promotion.
- **Expected contracts to publish**: `C-04`
- **Expected threads to publish / advance**: `THR-07`
- **Likely downstream stale triggers**:
  - inherited `THR-04` revalidation fires
  - support-matrix semantics or capability-inventory semantics drift
  - publication evidence starts implying UAA promotion or collapses support layers together
- **Expected closeout evidence**:
  - landed OpenCode participation in support and capability publication flows
  - explicit support-layer and passthrough-visibility posture in committed outputs
  - validation evidence from support-matrix, capability-matrix, and OpenCode root checks

## Slice index

- `S00` -> `slice-00-publication-contract-and-layer-baselines.md`
- `S1` -> `slice-1-support-matrix-open-code-enrollment.md`
- `S2` -> `slice-2-capability-inventory-and-passthrough-visibility.md`
- `S3` -> `slice-3-publication-validation-and-drift-guards.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-3-closeout.md`
