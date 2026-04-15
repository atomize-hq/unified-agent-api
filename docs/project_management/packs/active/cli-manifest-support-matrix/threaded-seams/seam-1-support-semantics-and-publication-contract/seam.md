---
seam_id: SEAM-1
seam_slug: support-semantics-and-publication-contract
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-1-support-semantics-and-publication-contract.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - support layer vocabulary changes
    - canonical publication location changes
    - neutral xtask entrypoint naming changes
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
# SEAM-1 - Support semantics and publication contract

## Seam Brief (Restated)

- **Goal / value**: lock one target-first support publication contract before downstream seams implement shared normalization, rendering, and validator enforcement.
- **Type**: integration
- **Scope**
  - In:
    - define the canonical meaning of manifest support, backend support, UAA unified support, and passthrough visibility
    - establish `cli_manifests/support_matrix/current.json` and `docs/specs/unified-agent-api/support-matrix.md` as the phase-1 publication targets
    - align manifest, validator, and runbook prose so `validated` and `supported` stop drifting across surfaces
    - reserve the neutral `xtask support-matrix` command name without changing capability-matrix behavior
  - Out:
    - shared wrapper normalization extraction
    - row derivation and rendering
    - contradiction validators and fixture/golden suites
- **Touch surface**:
  - `docs/specs/unified-agent-api/README.md`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `cli_manifests/codex/README.md`
  - `cli_manifests/claude_code/README.md`
  - `cli_manifests/codex/VALIDATOR_SPEC.md`
  - `cli_manifests/claude_code/VALIDATOR_SPEC.md`
  - `cli_manifests/codex/CI_AGENT_RUNBOOK.md`
  - `cli_manifests/claude_code/CI_AGENT_RUNBOOK.md`
  - `cli_manifests/codex/RULES.json`
  - `cli_manifests/claude_code/RULES.json`
  - `crates/xtask/src/main.rs`
- **Verification**:
  - the contract definition slice makes the owned support-publication rules concrete enough to implement without waiting on post-exec publication evidence
  - the exact JSON and Markdown publication paths are pinned in the canonical spec layer
  - manifest docs, validator docs, and runbooks cite the same meaning for `validated`, `supported`, and target-scoped support truth
  - `xtask` exposes `support-matrix` as the neutral downstream command contract while leaving existing capability-matrix behavior intact
- **Canonical contract refs**:
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/README.md`
- **Basis posture**:
  - Currentness: current
  - Upstream closeouts assumed: none
  - Required threads: `THR-01`
  - Stale triggers:
    - support layer vocabulary changes
    - canonical publication location changes
    - neutral `xtask support-matrix` naming changes
- **Threading constraints**
  - Upstream blockers: none
  - Downstream blocked seams: `SEAM-2`, `SEAM-3`, `SEAM-4`, `SEAM-5`
  - Contracts produced: `C-01`
  - Contracts consumed: none
  - Canonical contract refs: `docs/specs/unified-agent-api/support-matrix.md`, `docs/specs/unified-agent-api/README.md`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99`
- **Why this seam needs an explicit exit gate**: every downstream seam consumes the support-layer vocabulary, publication targets, and command naming locked here; closeout must record the exact handoff shape.
- **Expected contracts to publish**: `C-01`
- **Expected threads to publish / advance**: `THR-01`
- **Likely downstream stale triggers**:
  - support-layer names change after downstream seams start coding
  - publication paths move away from `cli_manifests/support_matrix/current.json` or `docs/specs/unified-agent-api/support-matrix.md`
  - `xtask support-matrix` contract drifts from the documented surfaces
- **Expected closeout evidence**:
  - landed support-matrix spec text
  - aligned manifest and validator docs
  - neutral `xtask support-matrix` entrypoint wiring
  - explicit downstream revalidation notes for `SEAM-2` through `SEAM-5`

## Slice index

- `S00` -> `slice-00-support-publication-contract-definition.md`
- `S1` -> `slice-1-support-terminology-and-authority-alignment.md`
- `S2` -> `slice-2-xtask-support-matrix-entrypoint.md`
- `S3` -> `slice-3-support-publication-touch-surface-conformance.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
