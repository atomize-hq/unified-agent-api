<!-- generated-by: xtask close-agent-maintenance; owner: maintenance-control-plane -->

# Handoff

This packet records the closed maintenance run for `opencode`.

Manual closeout remained an explicit maintainer action recorded with `close-agent-maintenance`; relay execution does not finalize it automatically.

## Request linkage

- request ref: `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`
- request sha256: `f8fa17dc42ca05bf3ec09e7f01423240234db0fdf2553a45e39b98b90c71f570`
- trigger kind: `upstream_release_detected`
- basis ref: `cli_manifests/opencode/latest_validated.txt`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- requested control-plane actions:
- `packet_doc_refresh`

## Trigger context

- detected_by: `.github/workflows/agent-maintenance-release-watch.yml`
- current_validated: `1.4.11`
- target_version: `1.14.47`
- latest_stable: `1.14.48`
- version_policy: `latest_stable_minus_one`
- source_kind: `github_releases`
- source_ref: `anomalyco/opencode`
- dispatch_kind: `packet_pr`
- dispatch_workflow: `agent-maintenance-open-pr.yml`
- branch_name: `automation/opencode-maintenance-1.14.47`

## Closeout

- closeout metadata: `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json`
- preflight passed: `true`
- recorded at: `2026-05-13T00:20:45Z`
- commit: `dc809949`

## Resolved findings

- [registry_manifest_drift] The opencode 1.14.47 packet refreshed the version-scoped manifest snapshots, coverage reports, and lockfile outputs required by the live maintenance request.
  surfaces:
  - cli_manifests/opencode/artifacts.lock.json
  - cli_manifests/opencode/reports/1.14.47/coverage.any.json
  - cli_manifests/opencode/reports/1.14.47/coverage.darwin-arm64.json
  - cli_manifests/opencode/snapshots/1.14.47/darwin-arm64.json
  - cli_manifests/opencode/snapshots/1.14.47/union.json
  - cli_manifests/opencode/versions/1.14.47.json
- [support_publication_drift] Support-matrix publication was refreshed to match the landed opencode 1.14.47 manifest truth after the successful write run.
  surfaces:
  - cli_manifests/support_matrix/current.json
  - docs/specs/unified-agent-api/support-matrix.md

## Deferred findings

- No deferred findings remain: No deferred maintenance findings remain after the successful opencode 1.14.47 packet write and green-gate validation; `check-agent-drift --agent opencode` is clean.

## Runtime follow-up

- No runtime follow-up is currently required.
