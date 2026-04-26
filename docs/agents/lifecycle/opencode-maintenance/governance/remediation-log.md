<!-- generated-by: xtask close-agent-maintenance; owner: maintenance-control-plane -->

# Remediation log

## Request

- request ref: `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`
- request sha256: `43143c88d2ed79e80f7e0f843a7e9a6e4ad60e8261522a62f3695763cf1e89ea`
- request recorded at: `2026-04-22T00:48:19Z`
- request commit: `8e77b59`

## Resolved findings

- [governance_doc_drift] The stale OpenCode SEAM-2 capability claim is now superseded by the maintenance packet and linked back to the current backend contract and generated capability publication.
  surfaces:
  - docs/integrations/opencode/governance/seam-2-closeout.md
  - crates/agent_api/src/backends/opencode/backend.rs
  - docs/specs/opencode-agent-api-backend-contract.md
  - docs/specs/unified-agent-api/capability-matrix.md
  - docs/agents/lifecycle/opencode-maintenance/HANDOFF.md
  - docs/agents/lifecycle/opencode-maintenance/review_surfaces.md

## Deferred findings

- No deferred findings remain: No deferred maintenance findings remain after the maintenance packet refresh. Repo-level preflight is still blocked by unrelated pre-existing loc-cap debt outside the M4 maintenance lane.
