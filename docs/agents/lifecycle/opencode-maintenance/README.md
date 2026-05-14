<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# opencode maintenance

This packet tracks automated upstream-release maintenance for `opencode`.

## Request

- request artifact: `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`
- trigger kind: `upstream_release_detected`
- basis ref: `cli_manifests/opencode/latest_validated.txt`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- recorded at: `2026-05-14T06:32:36Z`
- request commit: `b5ba0d73784f4c260737c958b4a9e0cc83c399e3`

## Trigger context

- detected_by: `.github/workflows/agent-maintenance-release-watch.yml`
- current_validated: `1.4.11`
- target_version: `1.14.49`
- latest_stable: `1.14.50`
- version_policy: `latest_stable_minus_one`
- source_kind: `github_releases`
- source_ref: `anomalyco/opencode`
- dispatch_kind: `packet_pr`
- dispatch_workflow: `agent-maintenance-open-pr.yml`
- branch_name: `automation/opencode-maintenance-1.14.49`

## Canonical execution contract

Use `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md` as the exact contributor execution contract for this lane. The PR body summary under `docs/agents/lifecycle/opencode-maintenance/governance/pr-summary.md` is derivative only.
