<!-- generated-by: xtask close-agent-maintenance; owner: maintenance-control-plane -->

# Handoff

This packet records the closed maintenance run for `opencode`.

## Request linkage

- request ref: `docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml`
- request sha256: `74d0de0365a5007ab6c3223d5641d1d6fcee2a4a16452a438f8f5ba5cf7cad1a`
- trigger kind: `drift_detected`
- basis ref: `docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md`
- opened from: `docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md`
- requested control-plane actions:
- `packet_doc_refresh`

## Closeout

- closeout metadata: `docs/project_management/next/opencode-maintenance/governance/maintenance-closeout.json`
- preflight passed: `false`
- recorded at: `2026-04-22T01:04:31Z`
- commit: `8e77b59`

## Resolved findings

- [governance_doc_drift] The stale OpenCode SEAM-2 capability claim is now superseded by the maintenance packet and linked back to the current backend contract and generated capability publication.
  surfaces:
  - docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md
  - crates/agent_api/src/backends/opencode/backend.rs
  - docs/specs/opencode-agent-api-backend-contract.md
  - docs/specs/unified-agent-api/capability-matrix.md
  - docs/project_management/next/opencode-maintenance/HANDOFF.md
  - docs/project_management/next/opencode-maintenance/review_surfaces.md

## Deferred findings

- No deferred findings remain: No deferred maintenance findings remain after the maintenance packet refresh. Repo-level preflight is still blocked by unrelated pre-existing loc-cap debt outside the M4 maintenance lane.

## Runtime follow-up

- No runtime follow-up is currently required.
