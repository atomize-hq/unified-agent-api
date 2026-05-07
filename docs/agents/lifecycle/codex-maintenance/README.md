<!-- generated-by: xtask refresh-agent; owner: control-plane -->

# codex maintenance

This packet tracks automated upstream-release maintenance for `codex`.

## Request

- request artifact: `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
- trigger kind: `upstream_release_detected`
- basis ref: `cli_manifests/codex/latest_validated.txt`
- opened from: `.github/workflows/codex-cli-update-snapshot.yml`
- recorded at: `2026-05-07T06:24:24Z`
- request commit: `1e44a63ca3d2b0de4686725ca7a79793b90f8b57`

## Trigger context

- detected_by: `.github/workflows/agent-maintenance-release-watch.yml`
- current_validated: `0.97.0`
- target_version: `0.125.0`
- latest_stable: `0.128.0`
- version_policy: `latest_stable_minus_one`
- source_kind: `github_releases`
- source_ref: `openai/codex`
- dispatch_kind: `workflow_dispatch`
- dispatch_workflow: `codex-cli-update-snapshot.yml`
- branch_name: `automation/codex-maintenance-0.125.0`

## Canonical execution contract

Use `docs/agents/lifecycle/codex-maintenance/HANDOFF.md` as the exact contributor execution contract for this lane. The PR body summary under `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md` is derivative only.
