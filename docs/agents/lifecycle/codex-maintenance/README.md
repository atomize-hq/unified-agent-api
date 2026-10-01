<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# codex maintenance

This packet tracks automated upstream-release maintenance for `codex`.

## Request

- request artifact: `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
- trigger kind: `upstream_release_detected`
- basis ref: `cli_manifests/codex/latest_validated.txt`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- recorded at: `2026-10-01T10:08:11Z`
- request commit: `f61534be004305a32e36f9631f218f8094572b59`

## Trigger context

- detected_by: `.github/workflows/agent-maintenance-release-watch.yml`
- current_validated: `0.156.1`
- target_version: `0.159.2`
- latest_stable: `0.159.3`
- version_policy: `latest_stable_minus_one`
- source_kind: `github_releases`
- source_ref: `openai/codex`
- dispatch_kind: `packet_pr`
- dispatch_workflow: `agent-maintenance-open-pr.yml`
- branch_name: `automation/codex-maintenance-0.159.2`

## Support-surface audit

- required: `true`
- pre-run debt count: `2`
- expected post-run debt count: `2`
- discovered upstream surface rows: `14`
- preexisting unsupported rows: `2`
- required uplifts this run:
- `codex completion` `completion` via `unbaselined_gap`
- `codex exec-server` `--linux-sandbox-pid-namespace` via `unbaselined_gap`
- `codex exec-server` `--proxy-private-ips-via-upstream` via `unbaselined_gap`
- `codex exec-server` `--ws-audience` via `unbaselined_gap`
- `codex exec-server` `--ws-auth` via `unbaselined_gap`
- `codex exec-server` `--ws-issuer` via `unbaselined_gap`
- `codex exec-server` `--ws-max-clock-skew-seconds` via `unbaselined_gap`
- `codex exec-server` `--ws-shared-secret-file` via `unbaselined_gap`
- `codex exec-server` `--ws-token-file` via `unbaselined_gap`
- `codex exec-server` `--ws-token-sha256` via `unbaselined_gap`
- `codex exec-server forward` `--linux-sandbox-pid-namespace` via `unbaselined_gap`
- `codex exec-server forward` `--proxy-private-ips-via-upstream` via `unbaselined_gap`
- `codex mcp add` `--oauth-client-secret` via `unbaselined_gap`
- `codex completion` `SHELL` via `unbaselined_gap`
- deferred preexisting gaps:
- `codex completion` `completion` via `requires_new_architectural_seam` (TODOS.md#close-codex-completion-maintenance-gap)
- `codex completion` `SHELL` via `requires_new_architectural_seam` (TODOS.md#close-codex-completion-maintenance-gap)


## Canonical execution contract

Use `docs/agents/lifecycle/codex-maintenance/HANDOFF.md` as the exact contributor execution contract for this lane. The PR body summary under `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md` is derivative only.
