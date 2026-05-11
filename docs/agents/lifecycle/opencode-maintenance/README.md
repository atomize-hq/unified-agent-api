<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# opencode maintenance

This packet tracks automated upstream-release maintenance for `opencode`.

## Request

- request artifact: `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`
- trigger kind: `upstream_release_detected`
- basis ref: `cli_manifests/opencode/latest_validated.txt`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- recorded at: `2026-05-11T19:44:54Z`
- request commit: `2af2890d044254877e10e73c08b7c4e1359d4a46`

## Trigger context

- detected_by: `.github/workflows/agent-maintenance-release-watch.yml`
- current_validated: `1.4.11`
- target_version: `1.14.40`
- latest_stable: `1.14.41`
- version_policy: `latest_stable_minus_one`
- source_kind: `github_releases`
- source_ref: `anomalyco/opencode`
- dispatch_kind: `packet_pr`
- dispatch_workflow: `agent-maintenance-open-pr.yml`
- branch_name: `automation/opencode-maintenance-1.14.40`

## Canonical execution contract

Use `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md` as the exact contributor execution contract for this lane. The PR body summary under `docs/agents/lifecycle/opencode-maintenance/governance/pr-summary.md` is derivative only.
