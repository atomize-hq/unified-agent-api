<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# codex maintenance

This packet tracks automated upstream-release maintenance for `codex`.

## Request

- request artifact: `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
- trigger kind: `upstream_release_detected`
- basis ref: `cli_manifests/codex/latest_validated.txt`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- recorded at: `2026-05-14T18:37:34Z`
- request commit: `4a6073bc7b7500441d8db170d5e5e3c9c9942366`

## Trigger context

- detected_by: `.github/workflows/agent-maintenance-release-watch.yml`
- current_validated: `0.125.0`
- target_version: `0.129.0`
- latest_stable: `0.130.0`
- version_policy: `latest_stable_minus_one`
- source_kind: `github_releases`
- source_ref: `openai/codex`
- dispatch_kind: `packet_pr`
- dispatch_workflow: `agent-maintenance-open-pr.yml`
- branch_name: `automation/codex-maintenance-0.129.0`

## Support-surface audit

- required: `true`
- pre-run debt count: `2`
- expected post-run debt count: `2`
- discovered upstream surface rows: `10`
- preexisting unsupported rows: `2`
- required uplifts this run:
- `codex update` `update` via `new_upstream_surface`
- `codex exec-server` `--executor-id` via `new_upstream_surface`
- `codex exec-server` `--name` via `new_upstream_surface`
- `codex login` `--with-access-token` via `new_upstream_surface`
- `codex sandbox linux` `--include-managed-config` via `new_upstream_surface`
- `codex sandbox linux` `--permissions-profile` via `new_upstream_surface`
- `codex sandbox macos` `--include-managed-config` via `new_upstream_surface`
- `codex sandbox macos` `--permissions-profile` via `new_upstream_surface`
- `codex sandbox windows` `--include-managed-config` via `new_upstream_surface`
- `codex sandbox windows` `--permissions-profile` via `new_upstream_surface`
- deferred preexisting gaps:
- `codex completion` `completion` via `requires_new_architectural_seam` (TODOS.md#close-codex-completion-maintenance-gap)
- `codex completion` `SHELL` via `requires_new_architectural_seam` (TODOS.md#close-codex-completion-maintenance-gap)


## Canonical execution contract

Use `docs/agents/lifecycle/codex-maintenance/HANDOFF.md` as the exact contributor execution contract for this lane. The PR body summary under `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md` is derivative only.
