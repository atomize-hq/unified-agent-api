<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# claude_code maintenance

This packet tracks automated upstream-release maintenance for `claude_code`.

## Request

- request artifact: `docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml`
- trigger kind: `upstream_release_detected`
- basis ref: `cli_manifests/claude_code/latest_validated.txt`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- recorded at: `2026-10-03T09:09:21Z`
- request commit: `f61534be004305a32e36f9631f218f8094572b59`

## Trigger context

- detected_by: `.github/workflows/agent-maintenance-release-watch.yml`
- current_validated: `2.1.274`
- target_version: `2.1.285`
- latest_stable: `2.1.285`
- version_policy: `upstream_stable_pointer`
- source_kind: `npm_dist_tag`
- source_ref: `@anthropic-ai/claude-code#stable`
- dispatch_kind: `packet_pr`
- dispatch_workflow: `agent-maintenance-open-pr.yml`
- branch_name: `automation/claude_code-maintenance-2.1.285`

## Support-surface audit

- required: `true`
- pre-run debt count: `2`
- expected post-run debt count: `2`
- discovered upstream surface rows: `10`
- preexisting unsupported rows: `2`
- required uplifts this run:
- `claude_code install` `install` via `unbaselined_gap`
- `claude_code install` `--force` via `unbaselined_gap`
- `claude_code plugin configure` `--json` via `unbaselined_gap`
- `claude_code plugin configure` `--values-stdin` via `unbaselined_gap`
- `claude_code plugin install` `--registry` via `unbaselined_gap`
- `claude_code plugin list` `--data-size` via `unbaselined_gap`
- `claude_code` `--client-data-url` via `unbaselined_gap`
- `claude_code` `--desktop` via `unbaselined_gap`
- `claude_code plugin configure` `plugin` via `unbaselined_gap`
- `claude_code plugin configure` `configure` via `unbaselined_gap`
- deferred preexisting gaps:
- `claude_code install` `install` via `requires_new_architectural_seam` (TODOS.md#close-claude-code-install-maintenance-gap)
- `claude_code install` `--force` via `requires_new_architectural_seam` (TODOS.md#close-claude-code-install-maintenance-gap)


## Canonical execution contract

Use `docs/agents/lifecycle/claude_code-maintenance/HANDOFF.md` as the exact contributor execution contract for this lane. The PR body summary under `docs/agents/lifecycle/claude_code-maintenance/governance/pr-summary.md` is derivative only.
