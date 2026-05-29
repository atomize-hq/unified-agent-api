<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# claude_code maintenance

This packet tracks automated upstream-release maintenance for `claude_code`.

## Request

- request artifact: `docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml`
- trigger kind: `upstream_release_detected`
- basis ref: `cli_manifests/claude_code/latest_validated.txt`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- recorded at: `2026-05-29T07:03:35Z`
- request commit: `773981d81242c7353b89097baaa21e0891c191db`

## Trigger context

- detected_by: `.github/workflows/agent-maintenance-release-watch.yml`
- current_validated: `2.1.29`
- target_version: `2.1.154`
- latest_stable: `2.1.156`
- version_policy: `latest_stable_minus_one`
- source_kind: `gcs_object_listing`
- source_ref: `claude-code-dist-86c565f3-f756-42ad-8dfa-d59b1c096819/claude-code-releases`
- dispatch_kind: `packet_pr`
- dispatch_workflow: `agent-maintenance-open-pr.yml`
- branch_name: `automation/claude_code-maintenance-2.1.154`

## Support-surface audit

- required: `true`
- pre-run debt count: `2`
- expected post-run debt count: `2`
- discovered upstream surface rows: `0`
- preexisting unsupported rows: `2`
- required uplifts this run:
- none
- deferred preexisting gaps:
- `claude install` `install` via `requires_new_architectural_seam` (TODOS.md#close-claude-code-install-maintenance-gap)
- `claude install` `--force` via `requires_new_architectural_seam` (TODOS.md#close-claude-code-install-maintenance-gap)


## Canonical execution contract

Use `docs/agents/lifecycle/claude_code-maintenance/HANDOFF.md` as the exact contributor execution contract for this lane. The PR body summary under `docs/agents/lifecycle/claude_code-maintenance/governance/pr-summary.md` is derivative only.
