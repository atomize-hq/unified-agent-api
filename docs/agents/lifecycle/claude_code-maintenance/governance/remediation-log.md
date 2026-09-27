<!-- generated-by: xtask close-agent-maintenance; owner: maintenance-control-plane -->

# Remediation log

## Request

- request ref: `docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml`
- request sha256: `c2a50ed7a20fb815f32d0d86d20d9deea8d962b5f6b8724d571c9dee100a6f66`
- request recorded at: `2026-09-25T08:49:43Z`
- request commit: `9e4a06ad0a9494fcc8d3fbb10ef57f35b55c42aa`

## Trigger context

- detected_by: `.github/workflows/agent-maintenance-release-watch.yml`
- current_validated: `2.1.29`
- target_version: `2.1.274`
- latest_stable: `2.1.274`
- version_policy: `upstream_stable_pointer`
- source_kind: `npm_dist_tag`
- source_ref: `@anthropic-ai/claude-code#stable`
- dispatch_kind: `packet_pr`
- dispatch_workflow: `agent-maintenance-open-pr.yml`
- branch_name: `automation/claude_code-maintenance-2.1.274`

## Resolved findings

- [registry_manifest_drift] The claude_code 2.1.274 packet materialized the version-scoped manifest artifacts required by the live maintenance request.
  surfaces:
  - cli_manifests/claude_code/snapshots/2.1.274/darwin-arm64.json
  - cli_manifests/claude_code/snapshots/2.1.274/linux-x64.json
  - cli_manifests/claude_code/snapshots/2.1.274/union.json
  - cli_manifests/claude_code/snapshots/2.1.274/win32-x64.json
  - cli_manifests/claude_code/reports/2.1.274/coverage.all.json
  - cli_manifests/claude_code/reports/2.1.274/coverage.any.json
  - cli_manifests/claude_code/reports/2.1.274/coverage.darwin-arm64.json
  - cli_manifests/claude_code/reports/2.1.274/coverage.linux-x64.json
  - cli_manifests/claude_code/reports/2.1.274/coverage.win32-x64.json
  - cli_manifests/claude_code/versions/2.1.274.json
  - cli_manifests/claude_code/wrapper_coverage.json
  - cli_manifests/claude_code/artifacts.lock.json
- [support_publication_drift] Support-matrix publication was regenerated to match the landed claude_code 2.1.274 manifest truth.
  surfaces:
  - cli_manifests/support_matrix/current.json
  - docs/specs/unified-agent-api/support-matrix.md

## Deferred findings

- No deferred findings remain: `check-agent-drift --agent claude_code` reports status: clean, so no maintenance drift finding remains deferred. Derived from the live report at closeout time, not carried forward from the request.
