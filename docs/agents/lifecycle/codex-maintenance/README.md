<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# codex maintenance

This packet tracks automated upstream-release maintenance for `codex`.

## Request

- request artifact: `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
- trigger kind: `upstream_release_detected`
- basis ref: `cli_manifests/codex/latest_validated.txt`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- recorded at: `2026-09-25T08:49:50Z`
- request commit: `9e4a06ad0a9494fcc8d3fbb10ef57f35b55c42aa`

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

## Support-surface audit

- required: `true`
- pre-run debt count: `2`
- expected post-run debt count: `2`
- discovered upstream surface rows: `2`
- preexisting unsupported rows: `2`
- required uplifts this run:
- `codex completion` `completion` via `unbaselined_gap`
- `codex completion` `SHELL` via `unbaselined_gap`
- deferred preexisting gaps:
- `codex completion` `completion` via `requires_new_architectural_seam` (TODOS.md#close-codex-completion-maintenance-gap)
- `codex completion` `SHELL` via `requires_new_architectural_seam` (TODOS.md#close-codex-completion-maintenance-gap)


## Canonical execution contract

Use `docs/agents/lifecycle/codex-maintenance/HANDOFF.md` as the exact contributor execution contract for this lane. The PR body summary under `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md` is derivative only.
