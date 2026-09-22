<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# codex maintenance

This packet tracks automated upstream-release maintenance for `codex`.

## Request

- request artifact: `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
- trigger kind: `upstream_release_detected`
- basis ref: `cli_manifests/codex/latest_validated.txt`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- recorded at: `2026-09-22T08:33:08Z`
- request commit: `be776222acb977b7efc12057c76eaed533d9e8a4`

## Trigger context

- detected_by: `.github/workflows/agent-maintenance-release-watch.yml`
- current_validated: `0.125.0`
- target_version: `0.155.0`
- latest_stable: `0.155.1`
- version_policy: `latest_stable_minus_one`
- source_kind: `github_releases`
- source_ref: `openai/codex`
- dispatch_kind: `packet_pr`
- dispatch_workflow: `agent-maintenance-open-pr.yml`
- branch_name: `automation/codex-maintenance-0.155.0`

## Support-surface audit

- required: `true`
- pre-run debt count: `2`
- expected post-run debt count: `2`
- discovered upstream surface rows: `52`
- preexisting unsupported rows: `2`
- required uplifts this run:
- `codex agents` `agents` via `unbaselined_gap`
- `codex completion` `completion` via `unbaselined_gap`
- `codex migrate-rollouts` `migrate-rollouts` via `unbaselined_gap`
- `codex queue` `queue` via `unbaselined_gap`
- `codex app-server` `--code-mode-host` via `unbaselined_gap`
- `codex exec` `--thread-source` via `unbaselined_gap`
- `codex exec fork` `--ephemeral` via `unbaselined_gap`
- `codex exec fork` `--ignore-rules` via `unbaselined_gap`
- `codex exec fork` `--ignore-user-config` via `unbaselined_gap`
- `codex exec fork` `--json` via `unbaselined_gap`
- `codex exec fork` `--output-last-message` via `unbaselined_gap`
- `codex exec fork` `--output-schema` via `unbaselined_gap`
- `codex exec fork` `--skip-git-repo-check` via `unbaselined_gap`
- `codex exec fork` `--thread-source` via `unbaselined_gap`
- `codex exec resume` `--thread-source` via `unbaselined_gap`
- `codex exec review` `--thread-source` via `unbaselined_gap`
- `codex exec-server` `--aws-profile` via `unbaselined_gap`
- `codex exec-server` `--aws-region` via `unbaselined_gap`
- `codex exec-server` `--aws-service` via `unbaselined_gap`
- `codex exec-server` `--aws-sigv4` via `unbaselined_gap`
- `codex exec-server` `--concurrent-requests` via `unbaselined_gap`
- `codex exec-server` `--exit-on-stdin-close` via `unbaselined_gap`
- `codex exec-server` `--remote-transport` via `unbaselined_gap`
- `codex exec-server forward` `--aws-profile` via `unbaselined_gap`
- `codex exec-server forward` `--aws-region` via `unbaselined_gap`
- `codex exec-server forward` `--aws-service` via `unbaselined_gap`
- `codex exec-server forward` `--aws-sigv4` via `unbaselined_gap`
- `codex exec-server forward` `--connect` via `unbaselined_gap`
- `codex exec-server forward` `--environment-id` via `unbaselined_gap`
- `codex exec-server forward` `--exit-on-stdin-close` via `unbaselined_gap`
- `codex exec-server forward` `--name` via `unbaselined_gap`
- `codex exec-server forward` `--remote-transport` via `unbaselined_gap`
- `codex exec-server forward` `--use-agent-identity-auth` via `unbaselined_gap`
- `codex mcp add` `--oauth-client-registration` via `unbaselined_gap`
- `codex mcp login` `--oauth-client-registration` via `unbaselined_gap`
- `codex migrate-rollouts` `--apply` via `unbaselined_gap`
- `codex migrate-rollouts` `--json` via `unbaselined_gap`
- `codex migrate-rollouts` `--max-mib-per-second` via `unbaselined_gap`
- `codex migrate-rollouts` `--thread` via `unbaselined_gap`
- `codex migrate-rollouts` `--verbose` via `unbaselined_gap`
- `codex queue` `--message` via `unbaselined_gap`
- `codex queue` `--thread` via `unbaselined_gap`
- `codex` `--approve-for-me` via `unbaselined_gap`
- `codex` `--worktree` via `unbaselined_gap`
- `codex completion` `SHELL` via `unbaselined_gap`
- `codex exec fork` `PROMPT` via `unbaselined_gap`
- `codex exec fork` `SESSION_ID` via `unbaselined_gap`
- `codex exec-server help` `COMMAND` via `unbaselined_gap`
- `codex app-server daemon update` `update` via `unbaselined_gap`
- `codex exec fork` `fork` via `unbaselined_gap`
- `codex exec-server forward` `forward` via `unbaselined_gap`
- `codex exec-server help` `help` via `unbaselined_gap`
- deferred preexisting gaps:
- `codex completion` `completion` via `requires_new_architectural_seam` (TODOS.md#close-codex-completion-maintenance-gap)
- `codex completion` `SHELL` via `requires_new_architectural_seam` (TODOS.md#close-codex-completion-maintenance-gap)


## Canonical execution contract

Use `docs/agents/lifecycle/codex-maintenance/HANDOFF.md` as the exact contributor execution contract for this lane. The PR body summary under `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md` is derivative only.
