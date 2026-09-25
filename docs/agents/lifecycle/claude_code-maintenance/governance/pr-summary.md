<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# PR summary

Automated maintenance packet for `claude_code` target `2.1.274`.

- canonical execution contract: `docs/agents/lifecycle/claude_code-maintenance/HANDOFF.md`
- request artifact: `docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml`
- branch: `automation/claude_code-maintenance-2.1.274`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- prompt sha256: `fb77405ce0eeab22c16d6777b82fa1f8390cacb6e96bce403cf270979634b3fe`

## Support-surface audit

- required: `true`
- pre-run debt count: `2`
- expected post-run debt count: `2`
- discovered upstream surface rows: `185`
- preexisting unsupported rows: `2`
- required uplifts this run:
- `claude_code agents` `agents` via `unbaselined_gap`
- `claude_code attach` `attach` via `unbaselined_gap`
- `claude_code auth` `auth` via `unbaselined_gap`
- `claude_code auto-mode` `auto-mode` via `unbaselined_gap`
- `claude_code gateway` `gateway` via `unbaselined_gap`
- `claude_code import` `import` via `unbaselined_gap`
- `claude_code install` `install` via `unbaselined_gap`
- `claude_code logs` `logs` via `unbaselined_gap`
- `claude_code project` `project` via `unbaselined_gap`
- `claude_code respawn` `respawn` via `unbaselined_gap`
- `claude_code rm` `rm` via `unbaselined_gap`
- `claude_code stop` `stop` via `unbaselined_gap`
- `claude_code ultrareview` `ultrareview` via `unbaselined_gap`
- `claude_code agents` `--all` via `unbaselined_gap`
- `claude_code agents` `--cwd` via `unbaselined_gap`
- `claude_code agents` `--json` via `unbaselined_gap`
- `claude_code auth login` `--claudeai` via `unbaselined_gap`
- `claude_code auth login` `--console` via `unbaselined_gap`
- `claude_code auth login` `--email` via `unbaselined_gap`
- `claude_code auth login` `--sso` via `unbaselined_gap`
- `claude_code auth status` `--json` via `unbaselined_gap`
- `claude_code auth status` `--text` via `unbaselined_gap`
- `claude_code auto-mode defaults` `--label` via `unbaselined_gap`
- `claude_code auto-mode reset` `--yes` via `unbaselined_gap`
- `claude_code gateway` `--config` via `unbaselined_gap`
- `claude_code import` `--dry-run` via `unbaselined_gap`
- `claude_code import` `--yes` via `unbaselined_gap`
- `claude_code install` `--force` via `unbaselined_gap`
- `claude_code mcp add` `--callback-port` via `unbaselined_gap`
- `claude_code mcp add` `--client-id` via `unbaselined_gap`
- `claude_code mcp add` `--client-secret` via `unbaselined_gap`
- `claude_code mcp add` `--env` via `unbaselined_gap`
- `claude_code mcp add` `--header` via `unbaselined_gap`
- `claude_code mcp add` `--scope` via `unbaselined_gap`
- `claude_code mcp add` `--transport` via `unbaselined_gap`
- `claude_code mcp add-from-claude-desktop` `--scope` via `unbaselined_gap`
- `claude_code mcp add-json` `--client-secret` via `unbaselined_gap`
- `claude_code mcp add-json` `--scope` via `unbaselined_gap`
- `claude_code mcp login` `--no-browser` via `unbaselined_gap`
- `claude_code mcp remove` `--scope` via `unbaselined_gap`
- `claude_code plugin disable` `--all` via `unbaselined_gap`
- `claude_code plugin disable` `--json` via `unbaselined_gap`
- `claude_code plugin disable` `--scope` via `unbaselined_gap`
- `claude_code plugin enable` `--json` via `unbaselined_gap`
- `claude_code plugin enable` `--scope` via `unbaselined_gap`
- `claude_code plugin eval` `--ablation` via `unbaselined_gap`
- `claude_code plugin eval` `--allow-real-servers` via `unbaselined_gap`
- `claude_code plugin eval` `--allow-tools` via `unbaselined_gap`
- `claude_code plugin eval` `--case` via `unbaselined_gap`
- `claude_code plugin eval` `--concurrency` via `unbaselined_gap`
- `claude_code plugin eval` `--eval-dir` via `unbaselined_gap`
- `claude_code plugin eval` `--json` via `unbaselined_gap`
- `claude_code plugin eval` `--judge-model` via `unbaselined_gap`
- `claude_code plugin eval` `--keep-temp` via `unbaselined_gap`
- `claude_code plugin eval` `--max-cost-usd` via `unbaselined_gap`
- `claude_code plugin eval` `--mocks` via `unbaselined_gap`
- `claude_code plugin eval` `--no-publish` via `unbaselined_gap`
- `claude_code plugin eval` `--no-scaffold` via `unbaselined_gap`
- `claude_code plugin eval` `--output-dir` via `unbaselined_gap`
- `claude_code plugin eval` `--publish-report` via `unbaselined_gap`
- `claude_code plugin eval` `--report` via `unbaselined_gap`
- `claude_code plugin eval` `--runs` via `unbaselined_gap`
- `claude_code plugin eval` `--scaffold` via `unbaselined_gap`
- `claude_code plugin eval` `--tag` via `unbaselined_gap`
- `claude_code plugin eval` `--threshold` via `unbaselined_gap`
- `claude_code plugin eval` `--trust-plugin` via `unbaselined_gap`
- `claude_code plugin eval init` `--eval-dir` via `unbaselined_gap`
- `claude_code plugin eval init` `--interactive` via `unbaselined_gap`
- `claude_code plugin init` `--author` via `unbaselined_gap`
- `claude_code plugin init` `--author-email` via `unbaselined_gap`
- `claude_code plugin init` `--description` via `unbaselined_gap`
- `claude_code plugin init` `--force` via `unbaselined_gap`
- `claude_code plugin init` `--with` via `unbaselined_gap`
- `claude_code plugin install` `--accept-command` via `unbaselined_gap`
- `claude_code plugin install` `--config` via `unbaselined_gap`
- `claude_code plugin install` `--json` via `unbaselined_gap`
- `claude_code plugin install` `--scope` via `unbaselined_gap`
- `claude_code plugin install` `--yes` via `unbaselined_gap`
- `claude_code plugin list` `--available` via `unbaselined_gap`
- `claude_code plugin list` `--json` via `unbaselined_gap`
- `claude_code plugin marketplace add` `--claudeai` via `unbaselined_gap`
- `claude_code plugin marketplace add` `--scope` via `unbaselined_gap`
- `claude_code plugin marketplace add` `--sparse` via `unbaselined_gap`
- `claude_code plugin marketplace list` `--json` via `unbaselined_gap`
- `claude_code plugin marketplace remove` `--scope` via `unbaselined_gap`
- `claude_code plugin prune` `--dry-run` via `unbaselined_gap`
- `claude_code plugin prune` `--scope` via `unbaselined_gap`
- `claude_code plugin prune` `--yes` via `unbaselined_gap`
- `claude_code plugin tag` `--dry-run` via `unbaselined_gap`
- `claude_code plugin tag` `--force` via `unbaselined_gap`
- `claude_code plugin tag` `--message` via `unbaselined_gap`
- `claude_code plugin tag` `--push` via `unbaselined_gap`
- `claude_code plugin tag` `--remote` via `unbaselined_gap`
- `claude_code plugin uninstall` `--json` via `unbaselined_gap`
- `claude_code plugin uninstall` `--keep-data` via `unbaselined_gap`
- `claude_code plugin uninstall` `--prune` via `unbaselined_gap`
- `claude_code plugin uninstall` `--scope` via `unbaselined_gap`
- `claude_code plugin uninstall` `--yes` via `unbaselined_gap`
- `claude_code plugin update` `--accept-command` via `unbaselined_gap`
- `claude_code plugin update` `--json` via `unbaselined_gap`
- `claude_code plugin update` `--scope` via `unbaselined_gap`
- `claude_code plugin update` `--yes` via `unbaselined_gap`
- `claude_code plugin validate` `--json` via `unbaselined_gap`
- `claude_code plugin validate` `--strict` via `unbaselined_gap`
- `claude_code project purge` `--all` via `unbaselined_gap`
- `claude_code project purge` `--dry-run` via `unbaselined_gap`
- `claude_code project purge` `--interactive` via `unbaselined_gap`
- `claude_code project purge` `--yes` via `unbaselined_gap`
- `claude_code ultrareview` `--json` via `unbaselined_gap`
- `claude_code ultrareview` `--no-post` via `unbaselined_gap`
- `claude_code ultrareview` `--post` via `unbaselined_gap`
- `claude_code ultrareview` `--timeout` via `unbaselined_gap`
- `claude_code` `--autocompact` via `unbaselined_gap`
- `claude_code` `--ax-screen-reader` via `unbaselined_gap`
- `claude_code` `--bare` via `unbaselined_gap`
- `claude_code` `--bg` via `unbaselined_gap`
- `claude_code` `--brief` via `unbaselined_gap`
- `claude_code` `--cloud` via `unbaselined_gap`
- `claude_code` `--effort` via `unbaselined_gap`
- `claude_code` `--environment` via `unbaselined_gap`
- `claude_code` `--exclude-dynamic-system-prompt-sections` via `unbaselined_gap`
- `claude_code` `--forward-subagent-text` via `unbaselined_gap`
- `claude_code` `--include-hook-events` via `unbaselined_gap`
- `claude_code` `--name` via `unbaselined_gap`
- `claude_code` `--permission-prompts` via `unbaselined_gap`
- `claude_code` `--plugin-url` via `unbaselined_gap`
- `claude_code` `--prompt-suggestions` via `unbaselined_gap`
- `claude_code` `--remote-control` via `unbaselined_gap`
- `claude_code` `--remote-control-session-name-prefix` via `unbaselined_gap`
- `claude_code` `--restricted` via `unbaselined_gap`
- `claude_code` `--safe-mode` via `unbaselined_gap`
- `claude_code` `--system-prompt-snapshot` via `unbaselined_gap`
- `claude_code` `--teleport` via `unbaselined_gap`
- `claude_code` `--tmux` via `unbaselined_gap`
- `claude_code` `--worktree` via `unbaselined_gap`
- `claude_code attach` `id` via `unbaselined_gap`
- `claude_code logs` `id` via `unbaselined_gap`
- `claude_code mcp add` `commandOrUrl` via `unbaselined_gap`
- `claude_code mcp add` `name` via `unbaselined_gap`
- `claude_code mcp add-json` `json` via `unbaselined_gap`
- `claude_code mcp add-json` `name` via `unbaselined_gap`
- `claude_code mcp get` `name` via `unbaselined_gap`
- `claude_code mcp login` `name` via `unbaselined_gap`
- `claude_code mcp logout` `name` via `unbaselined_gap`
- `claude_code mcp remove` `name` via `unbaselined_gap`
- `claude_code plugin details` `name` via `unbaselined_gap`
- `claude_code plugin enable` `plugin` via `unbaselined_gap`
- `claude_code plugin marketplace add` `source` via `unbaselined_gap`
- `claude_code plugin update` `plugin` via `unbaselined_gap`
- `claude_code plugin validate` `path` via `unbaselined_gap`
- `claude_code rm` `id` via `unbaselined_gap`
- `claude_code stop` `id` via `unbaselined_gap`
- `claude_code auth login` `login` via `unbaselined_gap`
- `claude_code auth logout` `logout` via `unbaselined_gap`
- `claude_code auth status` `status` via `unbaselined_gap`
- `claude_code auto-mode config` `config` via `unbaselined_gap`
- `claude_code auto-mode critique` `critique` via `unbaselined_gap`
- `claude_code auto-mode defaults` `defaults` via `unbaselined_gap`
- `claude_code auto-mode reset` `reset` via `unbaselined_gap`
- `claude_code mcp add` `add` via `unbaselined_gap`
- `claude_code mcp add-from-claude-desktop` `add-from-claude-desktop` via `unbaselined_gap`
- `claude_code mcp add-json` `add-json` via `unbaselined_gap`
- `claude_code mcp get` `get` via `unbaselined_gap`
- `claude_code mcp login` `login` via `unbaselined_gap`
- `claude_code mcp logout` `logout` via `unbaselined_gap`
- `claude_code mcp remove` `remove` via `unbaselined_gap`
- `claude_code mcp serve` `serve` via `unbaselined_gap`
- `claude_code plugin details` `details` via `unbaselined_gap`
- `claude_code plugin disable` `disable` via `unbaselined_gap`
- `claude_code plugin enable` `enable` via `unbaselined_gap`
- `claude_code plugin eval` `eval` via `unbaselined_gap`
- `claude_code plugin eval init` `init` via `unbaselined_gap`
- `claude_code plugin init` `init` via `unbaselined_gap`
- `claude_code plugin install` `install` via `unbaselined_gap`
- `claude_code plugin list` `list` via `unbaselined_gap`
- `claude_code plugin marketplace add` `add` via `unbaselined_gap`
- `claude_code plugin marketplace list` `list` via `unbaselined_gap`
- `claude_code plugin marketplace remove` `remove` via `unbaselined_gap`
- `claude_code plugin marketplace update` `update` via `unbaselined_gap`
- `claude_code plugin prune` `prune` via `unbaselined_gap`
- `claude_code plugin tag` `tag` via `unbaselined_gap`
- `claude_code plugin uninstall` `uninstall` via `unbaselined_gap`
- `claude_code plugin update` `update` via `unbaselined_gap`
- `claude_code plugin validate` `validate` via `unbaselined_gap`
- `claude_code project purge` `purge` via `unbaselined_gap`
- deferred preexisting gaps:
- `claude_code install` `install` via `requires_new_architectural_seam` (TODOS.md#close-claude-code-install-maintenance-gap)
- `claude_code install` `--force` via `requires_new_architectural_seam` (TODOS.md#close-claude-code-install-maintenance-gap)


## Next step

Follow `docs/agents/lifecycle/claude_code-maintenance/HANDOFF.md` exactly. This PR summary is derivative from the same execution-packet renderer.

## Exact maintained-agent prompt

```md
# Packet PR Maintenance Prompt (`2.1.274`)

This template renders the exact maintained-agent prompt for `claude_code` packet execution.
`docs/agents/lifecycle/claude_code-maintenance/HANDOFF.md` remains canonical and `governance/pr-summary.md` is derivative.

@codex

## Goal

Execute the automated maintenance packet for `claude_code` target `2.1.274`.

## Frozen request contract

- Read `docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml` before changing code or docs.
- Read the packet-owned `support_surface_audit` block before deciding whether the run can succeed.
- Treat `docs/agents/lifecycle/claude_code-maintenance/HANDOFF.md` as canonical for writable surfaces, read-only inputs, ordered commands, green gates, and recovery.
- Treat `.github/workflows/agent-maintenance-open-pr.yml` as the opening workflow source.
- Do not write outside the execution contract frozen in the request packet.

## Manifest inputs

- `cli_manifests/claude_code/README.md`
- `cli_manifests/claude_code/VALIDATOR_SPEC.md`
- `cli_manifests/claude_code/RULES.json`
- `cli_manifests/claude_code/SCHEMA.json`
- `cli_manifests/claude_code/current.json`
- `cli_manifests/claude_code/latest_validated.txt`
- `cli_manifests/claude_code/wrapper_coverage.json`

## Required workflow

1. Compare the current validated baseline from `cli_manifests/claude_code/latest_validated.txt` against the target `2.1.274` artifacts.
2. Use `support_surface_audit` to classify newly discovered non-TUI surface, preexisting non-TUI debt, required uplifts, and allowed deferrals.
3. For each `deferred_preexisting_gaps` row that also appears in `required_uplifts_this_run`, decide whether its `defer_reason` still holds at `2.1.274`. If it does, re-authorize that identity's existing debt row or rows in `docs/specs/unified-agent-api/non-tui-support-debt.md` in place:
   - set `authorized_at_version` to `2.1.274`;
   - set `scope_target_triples` so the rows together cover exactly the targets whose `cli_manifests/claude_code/reports/2.1.274/coverage.<target>.json` lists the surface, with no target in two rows;
   - set `authorization_evidence_ref` to `cli_manifests/claude_code/reports/2.1.274/coverage.any.json`.

   Change no other field and add no row. If the blocker no longer holds, treat the row as an uplift.
4. Land bounded wrapper/backend/manifest/publication updates for every remaining row in `required_uplifts_this_run`. Newly discovered surface is never deferred (maintenance-request contract field invariant 3), and no debt row may be added.
5. Refresh or create version-scoped manifest artifacts under `cli_manifests/claude_code/snapshots/2.1.274/`, `cli_manifests/claude_code/reports/2.1.274/`, and `cli_manifests/claude_code/versions/2.1.274.json` as required by the packet.
6. Leave closeout manual; record it only with `close-agent-maintenance` after the declared green gates pass.

## Done criteria

- Changes stay within the writable surfaces frozen in `docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml`.
- Every row in `required_uplifts_this_run` is uplifted, or is a preexisting debt row re-authorized at `2.1.274`; newly discovered surface is never deferred.
- `cargo run -p xtask -- codex-validate --root cli_manifests/claude_code` passes.
- `cargo run -p xtask -- maintenance-audit-status --request docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml` exits 0.
- The remaining ordered commands and green gates from `docs/agents/lifecycle/claude_code-maintenance/HANDOFF.md` pass or are captured in maintainer follow-up notes.

```
