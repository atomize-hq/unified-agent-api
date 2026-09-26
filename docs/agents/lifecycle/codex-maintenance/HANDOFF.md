<!-- generated-by: xtask close-agent-maintenance; owner: maintenance-control-plane -->

# Handoff

This packet records the closed maintenance run for `codex`.

Manual closeout remained an explicit maintainer action recorded with `close-agent-maintenance`; relay execution does not finalize it automatically.

## Request linkage

- request ref: `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
- request sha256: `92961ae8c983df38f1d6dce287bf0761088e517d3567b133070a6c1f4e3aea6b`
- trigger kind: `upstream_release_detected`
- basis ref: `cli_manifests/codex/latest_validated.txt`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- requested control-plane actions:
- `packet_doc_refresh`

## Trigger context

- detected_by: `.github/workflows/agent-maintenance-release-watch.yml`
- current_validated: `0.125.0`
- target_version: `0.156.1`
- latest_stable: `0.157.0`
- version_policy: `latest_stable_minus_one`
- source_kind: `github_releases`
- source_ref: `openai/codex`
- dispatch_kind: `packet_pr`
- dispatch_workflow: `agent-maintenance-open-pr.yml`
- branch_name: `automation/codex-maintenance-0.156.1`

## Closeout

- closeout metadata: `docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json`
- preflight passed: `true`
- recorded at: `2026-09-25T21:25:15Z`
- commit: `aacd785fe6a3920ab016b6ce7986a4c7c5831f18`

## Resolved findings

- [registry_manifest_drift] The codex 0.156.1 packet materialized the version-scoped manifest artifacts required by the live maintenance request.
  surfaces:
  - cli_manifests/codex/snapshots/0.156.1/aarch64-apple-darwin.json
  - cli_manifests/codex/snapshots/0.156.1/aarch64-unknown-linux-musl.json
  - cli_manifests/codex/snapshots/0.156.1/union.json
  - cli_manifests/codex/snapshots/0.156.1/x86_64-pc-windows-msvc.json
  - cli_manifests/codex/snapshots/0.156.1/x86_64-unknown-linux-musl.json
  - cli_manifests/codex/reports/0.156.1/coverage.aarch64-apple-darwin.json
  - cli_manifests/codex/reports/0.156.1/coverage.aarch64-unknown-linux-musl.json
  - cli_manifests/codex/reports/0.156.1/coverage.all.json
  - cli_manifests/codex/reports/0.156.1/coverage.any.json
  - cli_manifests/codex/reports/0.156.1/coverage.x86_64-pc-windows-msvc.json
  - cli_manifests/codex/reports/0.156.1/coverage.x86_64-unknown-linux-musl.json
  - cli_manifests/codex/versions/0.156.1.json
  - cli_manifests/codex/wrapper_coverage.json
  - cli_manifests/codex/artifacts.lock.json
- [support_publication_drift] Support-matrix publication was regenerated to match the landed codex 0.156.1 manifest truth.
  surfaces:
  - cli_manifests/support_matrix/current.json
  - docs/specs/unified-agent-api/support-matrix.md

## Deferred findings

- No deferred findings remain: `check-agent-drift --agent codex` reports status: clean, so no maintenance drift finding remains deferred. Derived from the live report at closeout time, not carried forward from the request.

## Runtime follow-up

- No runtime follow-up is currently required.
