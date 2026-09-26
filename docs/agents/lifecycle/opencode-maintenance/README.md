<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# opencode maintenance

This packet tracks automated upstream-release maintenance for `opencode`.

## Request

- request artifact: `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`
- trigger kind: `upstream_release_detected`
- basis ref: `cli_manifests/opencode/latest_validated.txt`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- recorded at: `2026-09-26T08:36:03Z`
- request commit: `74b32f696dcaeb8a836946230682c264a9b72d7d`

## Trigger context

- detected_by: `.github/workflows/agent-maintenance-release-watch.yml`
- current_validated: `1.4.11`
- target_version: `1.18.31`
- latest_stable: `1.18.32`
- version_policy: `latest_stable_minus_one`
- source_kind: `github_releases`
- source_ref: `anomalyco/opencode`
- dispatch_kind: `packet_pr`
- dispatch_workflow: `agent-maintenance-open-pr.yml`
- branch_name: `automation/opencode-maintenance-1.18.31`

## Support-surface audit

- required: `true`
- pre-run debt count: `8`
- expected post-run debt count: `8`
- discovered upstream surface rows: `8`
- preexisting unsupported rows: `8`
- required uplifts this run:
- `opencode acp` `acp` via `unbaselined_gap`
- `opencode attach` `attach` via `unbaselined_gap`
- `opencode models` `models` via `unbaselined_gap`
- `opencode providers` `providers` via `unbaselined_gap`
- `opencode serve` `serve` via `unbaselined_gap`
- `opencode web` `web` via `unbaselined_gap`
- `opencode run` `--agent` via `unbaselined_gap`
- `opencode run` `--attach` via `unbaselined_gap`
- deferred preexisting gaps:
- `opencode acp` `acp` via `requires_new_architectural_seam` (TODOS.md#close-opencode-non-tui-maintenance-gaps)
- `opencode attach` `attach` via `requires_new_architectural_seam` (TODOS.md#close-opencode-non-tui-maintenance-gaps)
- `opencode models` `models` via `requires_new_architectural_seam` (TODOS.md#close-opencode-non-tui-maintenance-gaps)
- `opencode providers` `providers` via `requires_new_architectural_seam` (TODOS.md#close-opencode-non-tui-maintenance-gaps)
- `opencode serve` `serve` via `requires_new_architectural_seam` (TODOS.md#close-opencode-non-tui-maintenance-gaps)
- `opencode web` `web` via `requires_new_architectural_seam` (TODOS.md#close-opencode-non-tui-maintenance-gaps)
- `opencode run` `--agent` via `requires_new_architectural_seam` (TODOS.md#close-opencode-non-tui-maintenance-gaps)
- `opencode run` `--attach` via `requires_new_architectural_seam` (TODOS.md#close-opencode-non-tui-maintenance-gaps)


## Canonical execution contract

Use `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md` as the exact contributor execution contract for this lane. The PR body summary under `docs/agents/lifecycle/opencode-maintenance/governance/pr-summary.md` is derivative only.
