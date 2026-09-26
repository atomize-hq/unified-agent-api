<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# Handoff

This file is the canonical contributor execution contract for `opencode` maintenance.

## Before you start: freeze this packet

The nightly watcher regenerates this packet every night for as long as this agent's
validated pointer trails upstream. Regeneration rewrites the request and force-pushes this
branch back to base, which destroys work committed to the branch and invalidates a closeout
bound to the previous request even when that closeout was never committed. Declare the freeze
**before your first adjudication**, not before the closeout command.

Commit exactly one file, to `staging` and never to this packet branch: the branch is inside the
tree the force-push replaces, so a marker carried there is destroyed by the operation it exists
to block.

```sh
git switch staging && git pull --ff-only
mkdir -p docs/agents/lifecycle/opencode-maintenance/governance/automation-stand-down
cat > docs/agents/lifecycle/opencode-maintenance/governance/automation-stand-down/1.18.31.toml <<'TOML'
schema_version = 1
agent_id = "opencode"
target_version = "1.18.31"
reason = "closeout in progress"
request_recorded_at = "2026-09-26T08:36:03Z"
TOML
git add docs/agents/lifecycle/opencode-maintenance/governance/automation-stand-down/1.18.31.toml
git commit docs/agents/lifecycle/opencode-maintenance/governance/automation-stand-down/1.18.31.toml -m "chore(opencode): stand automation down for 1.18.31"
git push origin staging
git switch -
```

The `git add` is required because the marker is always a new file, and the path on `git commit` is
what keeps everything else out of the commit. If `git switch` refuses, your tree is dirty: this
step runs before any packet work, so commit or stash that work first.

Confirm the freeze is live, from this branch:

```sh
cargo run -p xtask -- maintenance-stand-down-check --agent opencode --target-version 1.18.31 --from-ref origin/staging
```

Nothing releases the freeze but retirement. Closing this PR, pushing to it, approving it and
merging it all leave it in force; the promotion PR for `1.18.31` removes the marker.

## Packet origin

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
- discovered upstream surface rows: `481`
- preexisting unsupported rows: `8`
- required uplifts this run:
- `opencode acp` `acp` via `unbaselined_gap`
- `opencode agent` `agent` via `unbaselined_gap`
- `opencode attach` `attach` via `unbaselined_gap`
- `opencode completion` `completion` via `unbaselined_gap`
- `opencode db` `db` via `unbaselined_gap`
- `opencode debug` `debug` via `unbaselined_gap`
- `opencode export` `export` via `unbaselined_gap`
- `opencode github` `github` via `unbaselined_gap`
- `opencode import` `import` via `unbaselined_gap`
- `opencode mcp` `mcp` via `unbaselined_gap`
- `opencode models` `models` via `unbaselined_gap`
- `opencode plugin` `plugin` via `unbaselined_gap`
- `opencode pr` `pr` via `unbaselined_gap`
- `opencode providers` `providers` via `unbaselined_gap`
- `opencode serve` `serve` via `unbaselined_gap`
- `opencode session` `session` via `unbaselined_gap`
- `opencode stats` `stats` via `unbaselined_gap`
- `opencode uninstall` `uninstall` via `unbaselined_gap`
- `opencode upgrade` `upgrade` via `unbaselined_gap`
- `opencode web` `web` via `unbaselined_gap`
- `opencode acp` `--cors` via `unbaselined_gap`
- `opencode acp` `--cwd` via `unbaselined_gap`
- `opencode acp` `--help` via `unbaselined_gap`
- `opencode acp` `--hostname` via `unbaselined_gap`
- `opencode acp` `--log-level` via `unbaselined_gap`
- `opencode acp` `--mdns` via `unbaselined_gap`
- `opencode acp` `--mdns-domain` via `unbaselined_gap`
- `opencode acp` `--port` via `unbaselined_gap`
- `opencode acp` `--print-logs` via `unbaselined_gap`
- `opencode acp` `--pure` via `unbaselined_gap`
- `opencode acp` `--version` via `unbaselined_gap`
- `opencode agent` `--help` via `unbaselined_gap`
- `opencode agent` `--log-level` via `unbaselined_gap`
- `opencode agent` `--print-logs` via `unbaselined_gap`
- `opencode agent` `--pure` via `unbaselined_gap`
- `opencode agent` `--version` via `unbaselined_gap`
- `opencode agent create` `--description` via `unbaselined_gap`
- `opencode agent create` `--help` via `unbaselined_gap`
- `opencode agent create` `--log-level` via `unbaselined_gap`
- `opencode agent create` `--mode` via `unbaselined_gap`
- `opencode agent create` `--model` via `unbaselined_gap`
- `opencode agent create` `--path` via `unbaselined_gap`
- `opencode agent create` `--print-logs` via `unbaselined_gap`
- `opencode agent create` `--pure` via `unbaselined_gap`
- `opencode agent create` `--tools` via `unbaselined_gap`
- `opencode agent create` `--version` via `unbaselined_gap`
- `opencode agent list` `--help` via `unbaselined_gap`
- `opencode agent list` `--log-level` via `unbaselined_gap`
- `opencode agent list` `--print-logs` via `unbaselined_gap`
- `opencode agent list` `--pure` via `unbaselined_gap`
- `opencode agent list` `--version` via `unbaselined_gap`
- `opencode attach` `--continue` via `unbaselined_gap`
- `opencode attach` `--dir` via `unbaselined_gap`
- `opencode attach` `--fork` via `unbaselined_gap`
- `opencode attach` `--help` via `unbaselined_gap`
- `opencode attach` `--log-level` via `unbaselined_gap`
- `opencode attach` `--mini` via `unbaselined_gap`
- `opencode attach` `--no-replay` via `unbaselined_gap`
- `opencode attach` `--password` via `unbaselined_gap`
- `opencode attach` `--print-logs` via `unbaselined_gap`
- `opencode attach` `--pure` via `unbaselined_gap`
- `opencode attach` `--replay-limit` via `unbaselined_gap`
- `opencode attach` `--session` via `unbaselined_gap`
- `opencode attach` `--username` via `unbaselined_gap`
- `opencode attach` `--version` via `unbaselined_gap`
- `opencode completion` `--agent` via `unbaselined_gap`
- `opencode completion` `--auto` via `unbaselined_gap`
- `opencode completion` `--continue` via `unbaselined_gap`
- `opencode completion` `--cors` via `unbaselined_gap`
- `opencode completion` `--fork` via `unbaselined_gap`
- `opencode completion` `--help` via `unbaselined_gap`
- `opencode completion` `--hostname` via `unbaselined_gap`
- `opencode completion` `--log-level` via `unbaselined_gap`
- `opencode completion` `--mdns` via `unbaselined_gap`
- `opencode completion` `--mdns-domain` via `unbaselined_gap`
- `opencode completion` `--mini` via `unbaselined_gap`
- `opencode completion` `--model` via `unbaselined_gap`
- `opencode completion` `--no-replay` via `unbaselined_gap`
- `opencode completion` `--port` via `unbaselined_gap`
- `opencode completion` `--print-logs` via `unbaselined_gap`
- `opencode completion` `--prompt` via `unbaselined_gap`
- `opencode completion` `--pure` via `unbaselined_gap`
- `opencode completion` `--replay-limit` via `unbaselined_gap`
- `opencode completion` `--session` via `unbaselined_gap`
- `opencode completion` `--version` via `unbaselined_gap`
- `opencode db` `--format` via `unbaselined_gap`
- `opencode db` `--help` via `unbaselined_gap`
- `opencode db` `--log-level` via `unbaselined_gap`
- `opencode db` `--print-logs` via `unbaselined_gap`
- `opencode db` `--pure` via `unbaselined_gap`
- `opencode db` `--version` via `unbaselined_gap`
- `opencode db path` `--help` via `unbaselined_gap`
- `opencode db path` `--log-level` via `unbaselined_gap`
- `opencode db path` `--print-logs` via `unbaselined_gap`
- `opencode db path` `--pure` via `unbaselined_gap`
- `opencode db path` `--version` via `unbaselined_gap`
- `opencode debug` `--help` via `unbaselined_gap`
- `opencode debug` `--log-level` via `unbaselined_gap`
- `opencode debug` `--print-logs` via `unbaselined_gap`
- `opencode debug` `--pure` via `unbaselined_gap`
- `opencode debug` `--version` via `unbaselined_gap`
- `opencode debug agent` `--help` via `unbaselined_gap`
- `opencode debug agent` `--log-level` via `unbaselined_gap`
- `opencode debug agent` `--params` via `unbaselined_gap`
- `opencode debug agent` `--print-logs` via `unbaselined_gap`
- `opencode debug agent` `--pure` via `unbaselined_gap`
- `opencode debug agent` `--tool` via `unbaselined_gap`
- `opencode debug agent` `--version` via `unbaselined_gap`
- `opencode debug config` `--help` via `unbaselined_gap`
- `opencode debug config` `--log-level` via `unbaselined_gap`
- `opencode debug config` `--print-logs` via `unbaselined_gap`
- `opencode debug config` `--pure` via `unbaselined_gap`
- `opencode debug config` `--version` via `unbaselined_gap`
- `opencode debug file` `--help` via `unbaselined_gap`
- `opencode debug file` `--log-level` via `unbaselined_gap`
- `opencode debug file` `--print-logs` via `unbaselined_gap`
- `opencode debug file` `--pure` via `unbaselined_gap`
- `opencode debug file` `--version` via `unbaselined_gap`
- `opencode debug file list` `--help` via `unbaselined_gap`
- `opencode debug file list` `--log-level` via `unbaselined_gap`
- `opencode debug file list` `--print-logs` via `unbaselined_gap`
- `opencode debug file list` `--pure` via `unbaselined_gap`
- `opencode debug file list` `--version` via `unbaselined_gap`
- `opencode debug file read` `--help` via `unbaselined_gap`
- `opencode debug file read` `--log-level` via `unbaselined_gap`
- `opencode debug file read` `--print-logs` via `unbaselined_gap`
- `opencode debug file read` `--pure` via `unbaselined_gap`
- `opencode debug file read` `--version` via `unbaselined_gap`
- `opencode debug file search` `--help` via `unbaselined_gap`
- `opencode debug file search` `--log-level` via `unbaselined_gap`
- `opencode debug file search` `--print-logs` via `unbaselined_gap`
- `opencode debug file search` `--pure` via `unbaselined_gap`
- `opencode debug file search` `--version` via `unbaselined_gap`
- `opencode debug info` `--help` via `unbaselined_gap`
- `opencode debug info` `--log-level` via `unbaselined_gap`
- `opencode debug info` `--print-logs` via `unbaselined_gap`
- `opencode debug info` `--pure` via `unbaselined_gap`
- `opencode debug info` `--version` via `unbaselined_gap`
- `opencode debug lsp` `--help` via `unbaselined_gap`
- `opencode debug lsp` `--log-level` via `unbaselined_gap`
- `opencode debug lsp` `--print-logs` via `unbaselined_gap`
- `opencode debug lsp` `--pure` via `unbaselined_gap`
- `opencode debug lsp` `--version` via `unbaselined_gap`
- `opencode debug lsp diagnostics` `--help` via `unbaselined_gap`
- `opencode debug lsp diagnostics` `--log-level` via `unbaselined_gap`
- `opencode debug lsp diagnostics` `--print-logs` via `unbaselined_gap`
- `opencode debug lsp diagnostics` `--pure` via `unbaselined_gap`
- `opencode debug lsp diagnostics` `--version` via `unbaselined_gap`
- `opencode debug lsp document-symbols` `--help` via `unbaselined_gap`
- `opencode debug lsp document-symbols` `--log-level` via `unbaselined_gap`
- `opencode debug lsp document-symbols` `--print-logs` via `unbaselined_gap`
- `opencode debug lsp document-symbols` `--pure` via `unbaselined_gap`
- `opencode debug lsp document-symbols` `--version` via `unbaselined_gap`
- `opencode debug lsp symbols` `--help` via `unbaselined_gap`
- `opencode debug lsp symbols` `--log-level` via `unbaselined_gap`
- `opencode debug lsp symbols` `--print-logs` via `unbaselined_gap`
- `opencode debug lsp symbols` `--pure` via `unbaselined_gap`
- `opencode debug lsp symbols` `--version` via `unbaselined_gap`
- `opencode debug paths` `--help` via `unbaselined_gap`
- `opencode debug paths` `--log-level` via `unbaselined_gap`
- `opencode debug paths` `--print-logs` via `unbaselined_gap`
- `opencode debug paths` `--pure` via `unbaselined_gap`
- `opencode debug paths` `--version` via `unbaselined_gap`
- `opencode debug rg` `--help` via `unbaselined_gap`
- `opencode debug rg` `--log-level` via `unbaselined_gap`
- `opencode debug rg` `--print-logs` via `unbaselined_gap`
- `opencode debug rg` `--pure` via `unbaselined_gap`
- `opencode debug rg` `--version` via `unbaselined_gap`
- `opencode debug rg files` `--glob` via `unbaselined_gap`
- `opencode debug rg files` `--help` via `unbaselined_gap`
- `opencode debug rg files` `--limit` via `unbaselined_gap`
- `opencode debug rg files` `--log-level` via `unbaselined_gap`
- `opencode debug rg files` `--print-logs` via `unbaselined_gap`
- `opencode debug rg files` `--pure` via `unbaselined_gap`
- `opencode debug rg files` `--query` via `unbaselined_gap`
- `opencode debug rg files` `--version` via `unbaselined_gap`
- `opencode debug rg search` `--glob` via `unbaselined_gap`
- `opencode debug rg search` `--help` via `unbaselined_gap`
- `opencode debug rg search` `--limit` via `unbaselined_gap`
- `opencode debug rg search` `--log-level` via `unbaselined_gap`
- `opencode debug rg search` `--print-logs` via `unbaselined_gap`
- `opencode debug rg search` `--pure` via `unbaselined_gap`
- `opencode debug rg search` `--version` via `unbaselined_gap`
- `opencode debug scrap` `--help` via `unbaselined_gap`
- `opencode debug scrap` `--log-level` via `unbaselined_gap`
- `opencode debug scrap` `--print-logs` via `unbaselined_gap`
- `opencode debug scrap` `--pure` via `unbaselined_gap`
- `opencode debug scrap` `--version` via `unbaselined_gap`
- `opencode debug skill` `--help` via `unbaselined_gap`
- `opencode debug skill` `--log-level` via `unbaselined_gap`
- `opencode debug skill` `--print-logs` via `unbaselined_gap`
- `opencode debug skill` `--pure` via `unbaselined_gap`
- `opencode debug skill` `--version` via `unbaselined_gap`
- `opencode debug snapshot` `--help` via `unbaselined_gap`
- `opencode debug snapshot` `--log-level` via `unbaselined_gap`
- `opencode debug snapshot` `--print-logs` via `unbaselined_gap`
- `opencode debug snapshot` `--pure` via `unbaselined_gap`
- `opencode debug snapshot` `--version` via `unbaselined_gap`
- `opencode debug snapshot diff` `--help` via `unbaselined_gap`
- `opencode debug snapshot diff` `--log-level` via `unbaselined_gap`
- `opencode debug snapshot diff` `--print-logs` via `unbaselined_gap`
- `opencode debug snapshot diff` `--pure` via `unbaselined_gap`
- `opencode debug snapshot diff` `--version` via `unbaselined_gap`
- `opencode debug snapshot patch` `--help` via `unbaselined_gap`
- `opencode debug snapshot patch` `--log-level` via `unbaselined_gap`
- `opencode debug snapshot patch` `--print-logs` via `unbaselined_gap`
- `opencode debug snapshot patch` `--pure` via `unbaselined_gap`
- `opencode debug snapshot patch` `--version` via `unbaselined_gap`
- `opencode debug snapshot track` `--help` via `unbaselined_gap`
- `opencode debug snapshot track` `--log-level` via `unbaselined_gap`
- `opencode debug snapshot track` `--print-logs` via `unbaselined_gap`
- `opencode debug snapshot track` `--pure` via `unbaselined_gap`
- `opencode debug snapshot track` `--version` via `unbaselined_gap`
- `opencode debug startup` `--help` via `unbaselined_gap`
- `opencode debug startup` `--log-level` via `unbaselined_gap`
- `opencode debug startup` `--print-logs` via `unbaselined_gap`
- `opencode debug startup` `--pure` via `unbaselined_gap`
- `opencode debug startup` `--version` via `unbaselined_gap`
- `opencode debug v2` `--help` via `unbaselined_gap`
- `opencode debug v2` `--log-level` via `unbaselined_gap`
- `opencode debug v2` `--print-logs` via `unbaselined_gap`
- `opencode debug v2` `--pure` via `unbaselined_gap`
- `opencode debug v2` `--version` via `unbaselined_gap`
- `opencode debug wait` `--help` via `unbaselined_gap`
- `opencode debug wait` `--log-level` via `unbaselined_gap`
- `opencode debug wait` `--print-logs` via `unbaselined_gap`
- `opencode debug wait` `--pure` via `unbaselined_gap`
- `opencode debug wait` `--version` via `unbaselined_gap`
- `opencode export` `--help` via `unbaselined_gap`
- `opencode export` `--log-level` via `unbaselined_gap`
- `opencode export` `--print-logs` via `unbaselined_gap`
- `opencode export` `--pure` via `unbaselined_gap`
- `opencode export` `--sanitize` via `unbaselined_gap`
- `opencode export` `--version` via `unbaselined_gap`
- `opencode github` `--help` via `unbaselined_gap`
- `opencode github` `--log-level` via `unbaselined_gap`
- `opencode github` `--print-logs` via `unbaselined_gap`
- `opencode github` `--pure` via `unbaselined_gap`
- `opencode github` `--version` via `unbaselined_gap`
- `opencode github install` `--help` via `unbaselined_gap`
- `opencode github install` `--log-level` via `unbaselined_gap`
- `opencode github install` `--print-logs` via `unbaselined_gap`
- `opencode github install` `--pure` via `unbaselined_gap`
- `opencode github install` `--version` via `unbaselined_gap`
- `opencode github run` `--event` via `unbaselined_gap`
- `opencode github run` `--help` via `unbaselined_gap`
- `opencode github run` `--log-level` via `unbaselined_gap`
- `opencode github run` `--print-logs` via `unbaselined_gap`
- `opencode github run` `--pure` via `unbaselined_gap`
- `opencode github run` `--token` via `unbaselined_gap`
- `opencode github run` `--version` via `unbaselined_gap`
- `opencode import` `--help` via `unbaselined_gap`
- `opencode import` `--log-level` via `unbaselined_gap`
- `opencode import` `--print-logs` via `unbaselined_gap`
- `opencode import` `--pure` via `unbaselined_gap`
- `opencode import` `--version` via `unbaselined_gap`
- `opencode mcp` `--help` via `unbaselined_gap`
- `opencode mcp` `--log-level` via `unbaselined_gap`
- `opencode mcp` `--print-logs` via `unbaselined_gap`
- `opencode mcp` `--pure` via `unbaselined_gap`
- `opencode mcp` `--version` via `unbaselined_gap`
- `opencode mcp add` `--env` via `unbaselined_gap`
- `opencode mcp add` `--header` via `unbaselined_gap`
- `opencode mcp add` `--help` via `unbaselined_gap`
- `opencode mcp add` `--log-level` via `unbaselined_gap`
- `opencode mcp add` `--print-logs` via `unbaselined_gap`
- `opencode mcp add` `--pure` via `unbaselined_gap`
- `opencode mcp add` `--url` via `unbaselined_gap`
- `opencode mcp add` `--version` via `unbaselined_gap`
- `opencode mcp auth` `--help` via `unbaselined_gap`
- `opencode mcp auth` `--log-level` via `unbaselined_gap`
- `opencode mcp auth` `--print-logs` via `unbaselined_gap`
- `opencode mcp auth` `--pure` via `unbaselined_gap`
- `opencode mcp auth` `--version` via `unbaselined_gap`
- `opencode mcp auth list` `--help` via `unbaselined_gap`
- `opencode mcp auth list` `--log-level` via `unbaselined_gap`
- `opencode mcp auth list` `--print-logs` via `unbaselined_gap`
- `opencode mcp auth list` `--pure` via `unbaselined_gap`
- `opencode mcp auth list` `--version` via `unbaselined_gap`
- `opencode mcp debug` `--help` via `unbaselined_gap`
- `opencode mcp debug` `--log-level` via `unbaselined_gap`
- `opencode mcp debug` `--print-logs` via `unbaselined_gap`
- `opencode mcp debug` `--pure` via `unbaselined_gap`
- `opencode mcp debug` `--version` via `unbaselined_gap`
- `opencode mcp list` `--help` via `unbaselined_gap`
- `opencode mcp list` `--log-level` via `unbaselined_gap`
- `opencode mcp list` `--print-logs` via `unbaselined_gap`
- `opencode mcp list` `--pure` via `unbaselined_gap`
- `opencode mcp list` `--version` via `unbaselined_gap`
- `opencode mcp logout` `--help` via `unbaselined_gap`
- `opencode mcp logout` `--log-level` via `unbaselined_gap`
- `opencode mcp logout` `--print-logs` via `unbaselined_gap`
- `opencode mcp logout` `--pure` via `unbaselined_gap`
- `opencode mcp logout` `--version` via `unbaselined_gap`
- `opencode models` `--help` via `unbaselined_gap`
- `opencode models` `--log-level` via `unbaselined_gap`
- `opencode models` `--print-logs` via `unbaselined_gap`
- `opencode models` `--pure` via `unbaselined_gap`
- `opencode models` `--refresh` via `unbaselined_gap`
- `opencode models` `--verbose` via `unbaselined_gap`
- `opencode models` `--version` via `unbaselined_gap`
- `opencode plugin` `--force` via `unbaselined_gap`
- `opencode plugin` `--global` via `unbaselined_gap`
- `opencode plugin` `--help` via `unbaselined_gap`
- `opencode plugin` `--log-level` via `unbaselined_gap`
- `opencode plugin` `--print-logs` via `unbaselined_gap`
- `opencode plugin` `--pure` via `unbaselined_gap`
- `opencode plugin` `--version` via `unbaselined_gap`
- `opencode pr` `--help` via `unbaselined_gap`
- `opencode pr` `--log-level` via `unbaselined_gap`
- `opencode pr` `--print-logs` via `unbaselined_gap`
- `opencode pr` `--pure` via `unbaselined_gap`
- `opencode pr` `--version` via `unbaselined_gap`
- `opencode providers` `--help` via `unbaselined_gap`
- `opencode providers` `--log-level` via `unbaselined_gap`
- `opencode providers` `--print-logs` via `unbaselined_gap`
- `opencode providers` `--pure` via `unbaselined_gap`
- `opencode providers` `--version` via `unbaselined_gap`
- `opencode providers list` `--help` via `unbaselined_gap`
- `opencode providers list` `--log-level` via `unbaselined_gap`
- `opencode providers list` `--print-logs` via `unbaselined_gap`
- `opencode providers list` `--pure` via `unbaselined_gap`
- `opencode providers list` `--version` via `unbaselined_gap`
- `opencode providers login` `--help` via `unbaselined_gap`
- `opencode providers login` `--log-level` via `unbaselined_gap`
- `opencode providers login` `--method` via `unbaselined_gap`
- `opencode providers login` `--print-logs` via `unbaselined_gap`
- `opencode providers login` `--provider` via `unbaselined_gap`
- `opencode providers login` `--pure` via `unbaselined_gap`
- `opencode providers login` `--version` via `unbaselined_gap`
- `opencode providers logout` `--help` via `unbaselined_gap`
- `opencode providers logout` `--log-level` via `unbaselined_gap`
- `opencode providers logout` `--print-logs` via `unbaselined_gap`
- `opencode providers logout` `--pure` via `unbaselined_gap`
- `opencode providers logout` `--version` via `unbaselined_gap`
- `opencode run` `--agent` via `unbaselined_gap`
- `opencode run` `--attach` via `unbaselined_gap`
- `opencode run` `--auto` via `unbaselined_gap`
- `opencode run` `--command` via `unbaselined_gap`
- `opencode run` `--file` via `unbaselined_gap`
- `opencode run` `--help` via `unbaselined_gap`
- `opencode run` `--interactive` via `unbaselined_gap`
- `opencode run` `--log-level` via `unbaselined_gap`
- `opencode run` `--password` via `unbaselined_gap`
- `opencode run` `--port` via `unbaselined_gap`
- `opencode run` `--print-logs` via `unbaselined_gap`
- `opencode run` `--pure` via `unbaselined_gap`
- `opencode run` `--share` via `unbaselined_gap`
- `opencode run` `--thinking` via `unbaselined_gap`
- `opencode run` `--title` via `unbaselined_gap`
- `opencode run` `--username` via `unbaselined_gap`
- `opencode run` `--variant` via `unbaselined_gap`
- `opencode run` `--version` via `unbaselined_gap`
- `opencode serve` `--cors` via `unbaselined_gap`
- `opencode serve` `--help` via `unbaselined_gap`
- `opencode serve` `--hostname` via `unbaselined_gap`
- `opencode serve` `--log-level` via `unbaselined_gap`
- `opencode serve` `--mdns` via `unbaselined_gap`
- `opencode serve` `--mdns-domain` via `unbaselined_gap`
- `opencode serve` `--port` via `unbaselined_gap`
- `opencode serve` `--print-logs` via `unbaselined_gap`
- `opencode serve` `--pure` via `unbaselined_gap`
- `opencode serve` `--version` via `unbaselined_gap`
- `opencode session` `--help` via `unbaselined_gap`
- `opencode session` `--log-level` via `unbaselined_gap`
- `opencode session` `--print-logs` via `unbaselined_gap`
- `opencode session` `--pure` via `unbaselined_gap`
- `opencode session` `--version` via `unbaselined_gap`
- `opencode session delete` `--help` via `unbaselined_gap`
- `opencode session delete` `--log-level` via `unbaselined_gap`
- `opencode session delete` `--print-logs` via `unbaselined_gap`
- `opencode session delete` `--pure` via `unbaselined_gap`
- `opencode session delete` `--version` via `unbaselined_gap`
- `opencode session list` `--format` via `unbaselined_gap`
- `opencode session list` `--help` via `unbaselined_gap`
- `opencode session list` `--log-level` via `unbaselined_gap`
- `opencode session list` `--max-count` via `unbaselined_gap`
- `opencode session list` `--print-logs` via `unbaselined_gap`
- `opencode session list` `--pure` via `unbaselined_gap`
- `opencode session list` `--version` via `unbaselined_gap`
- `opencode stats` `--days` via `unbaselined_gap`
- `opencode stats` `--help` via `unbaselined_gap`
- `opencode stats` `--log-level` via `unbaselined_gap`
- `opencode stats` `--models` via `unbaselined_gap`
- `opencode stats` `--print-logs` via `unbaselined_gap`
- `opencode stats` `--project` via `unbaselined_gap`
- `opencode stats` `--pure` via `unbaselined_gap`
- `opencode stats` `--tools` via `unbaselined_gap`
- `opencode stats` `--version` via `unbaselined_gap`
- `opencode uninstall` `--dry-run` via `unbaselined_gap`
- `opencode uninstall` `--force` via `unbaselined_gap`
- `opencode uninstall` `--help` via `unbaselined_gap`
- `opencode uninstall` `--keep-config` via `unbaselined_gap`
- `opencode uninstall` `--keep-data` via `unbaselined_gap`
- `opencode uninstall` `--log-level` via `unbaselined_gap`
- `opencode uninstall` `--print-logs` via `unbaselined_gap`
- `opencode uninstall` `--pure` via `unbaselined_gap`
- `opencode uninstall` `--version` via `unbaselined_gap`
- `opencode upgrade` `--help` via `unbaselined_gap`
- `opencode upgrade` `--log-level` via `unbaselined_gap`
- `opencode upgrade` `--method` via `unbaselined_gap`
- `opencode upgrade` `--print-logs` via `unbaselined_gap`
- `opencode upgrade` `--pure` via `unbaselined_gap`
- `opencode upgrade` `--version` via `unbaselined_gap`
- `opencode web` `--cors` via `unbaselined_gap`
- `opencode web` `--help` via `unbaselined_gap`
- `opencode web` `--hostname` via `unbaselined_gap`
- `opencode web` `--log-level` via `unbaselined_gap`
- `opencode web` `--mdns` via `unbaselined_gap`
- `opencode web` `--mdns-domain` via `unbaselined_gap`
- `opencode web` `--port` via `unbaselined_gap`
- `opencode web` `--print-logs` via `unbaselined_gap`
- `opencode web` `--pure` via `unbaselined_gap`
- `opencode web` `--version` via `unbaselined_gap`
- `opencode attach` `url` via `unbaselined_gap`
- `opencode completion` `project` via `unbaselined_gap`
- `opencode db` `query` via `unbaselined_gap`
- `opencode debug agent` `name` via `unbaselined_gap`
- `opencode debug file list` `path` via `unbaselined_gap`
- `opencode debug file read` `path` via `unbaselined_gap`
- `opencode debug file search` `query` via `unbaselined_gap`
- `opencode debug lsp diagnostics` `file` via `unbaselined_gap`
- `opencode debug lsp document-symbols` `uri` via `unbaselined_gap`
- `opencode debug lsp symbols` `query` via `unbaselined_gap`
- `opencode debug rg search` `pattern` via `unbaselined_gap`
- `opencode debug snapshot diff` `hash` via `unbaselined_gap`
- `opencode debug snapshot patch` `hash` via `unbaselined_gap`
- `opencode export` `sessionID` via `unbaselined_gap`
- `opencode import` `file` via `unbaselined_gap`
- `opencode mcp add` `name` via `unbaselined_gap`
- `opencode mcp auth` `name` via `unbaselined_gap`
- `opencode mcp auth list` `name` via `unbaselined_gap`
- `opencode mcp debug` `name` via `unbaselined_gap`
- `opencode mcp logout` `name` via `unbaselined_gap`
- `opencode models` `provider` via `unbaselined_gap`
- `opencode plugin` `module` via `unbaselined_gap`
- `opencode pr` `number` via `unbaselined_gap`
- `opencode providers login` `url` via `unbaselined_gap`
- `opencode providers logout` `provider` via `unbaselined_gap`
- `opencode session delete` `sessionID` via `unbaselined_gap`
- `opencode upgrade` `target` via `unbaselined_gap`
- `opencode agent create` `create` via `unbaselined_gap`
- `opencode agent list` `list` via `unbaselined_gap`
- `opencode db path` `path` via `unbaselined_gap`
- `opencode debug agent` `agent` via `unbaselined_gap`
- `opencode debug config` `config` via `unbaselined_gap`
- `opencode debug file` `file` via `unbaselined_gap`
- `opencode debug file list` `list` via `unbaselined_gap`
- `opencode debug file read` `read` via `unbaselined_gap`
- `opencode debug file search` `search` via `unbaselined_gap`
- `opencode debug info` `info` via `unbaselined_gap`
- `opencode debug lsp` `lsp` via `unbaselined_gap`
- `opencode debug lsp diagnostics` `diagnostics` via `unbaselined_gap`
- `opencode debug lsp document-symbols` `document-symbols` via `unbaselined_gap`
- `opencode debug lsp symbols` `symbols` via `unbaselined_gap`
- `opencode debug paths` `paths` via `unbaselined_gap`
- `opencode debug rg` `rg` via `unbaselined_gap`
- `opencode debug rg files` `files` via `unbaselined_gap`
- `opencode debug rg search` `search` via `unbaselined_gap`
- `opencode debug scrap` `scrap` via `unbaselined_gap`
- `opencode debug skill` `skill` via `unbaselined_gap`
- `opencode debug snapshot` `snapshot` via `unbaselined_gap`
- `opencode debug snapshot diff` `diff` via `unbaselined_gap`
- `opencode debug snapshot patch` `patch` via `unbaselined_gap`
- `opencode debug snapshot track` `track` via `unbaselined_gap`
- `opencode debug startup` `startup` via `unbaselined_gap`
- `opencode debug v2` `v2` via `unbaselined_gap`
- `opencode debug wait` `wait` via `unbaselined_gap`
- `opencode github install` `install` via `unbaselined_gap`
- `opencode github run` `run` via `unbaselined_gap`
- `opencode mcp add` `add` via `unbaselined_gap`
- `opencode mcp auth` `auth` via `unbaselined_gap`
- `opencode mcp auth list` `list` via `unbaselined_gap`
- `opencode mcp debug` `debug` via `unbaselined_gap`
- `opencode mcp list` `list` via `unbaselined_gap`
- `opencode mcp logout` `logout` via `unbaselined_gap`
- `opencode providers list` `list` via `unbaselined_gap`
- `opencode providers login` `login` via `unbaselined_gap`
- `opencode providers logout` `logout` via `unbaselined_gap`
- `opencode session delete` `delete` via `unbaselined_gap`
- `opencode session list` `list` via `unbaselined_gap`
- deferred preexisting gaps:
- `opencode acp` `acp` via `requires_new_architectural_seam` (TODOS.md#close-opencode-non-tui-maintenance-gaps)
- `opencode attach` `attach` via `requires_new_architectural_seam` (TODOS.md#close-opencode-non-tui-maintenance-gaps)
- `opencode models` `models` via `requires_new_architectural_seam` (TODOS.md#close-opencode-non-tui-maintenance-gaps)
- `opencode providers` `providers` via `requires_new_architectural_seam` (TODOS.md#close-opencode-non-tui-maintenance-gaps)
- `opencode serve` `serve` via `requires_new_architectural_seam` (TODOS.md#close-opencode-non-tui-maintenance-gaps)
- `opencode web` `web` via `requires_new_architectural_seam` (TODOS.md#close-opencode-non-tui-maintenance-gaps)
- `opencode run` `--agent` via `requires_new_architectural_seam` (TODOS.md#close-opencode-non-tui-maintenance-gaps)
- `opencode run` `--attach` via `requires_new_architectural_seam` (TODOS.md#close-opencode-non-tui-maintenance-gaps)


## Relay contract

Guarded lifecycle command names appear below as contract metadata or as instructions for an actor outside the relay. An agent executing this packet inside a relay session must not invoke them.

- maintained agent packet: `opencode`
- local execution host: `local Codex CLI host via execute-agent-maintenance`
- executor surface: `execute-agent-maintenance`
- request artifact: `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`
- prompt template path: `docs/agents/lifecycle/opencode-maintenance/governance/execute-agent-maintenance-prompt.md`
- prompt sha256: `478131e737048e5124968e22d6085bd4af522fdda345010febd475a434dac7c7`
- canonical handoff: `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md`
- derivative pr summary: `docs/agents/lifecycle/opencode-maintenance/governance/pr-summary.md`
- exact closeout artifact: `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json`
- branch linkage: `automation/opencode-maintenance-1.18.31`
- manual closeout required: `true`

## Writable surfaces

- `docs/agents/lifecycle/opencode-maintenance/**`
- `crates/opencode/**`
- `crates/agent_api/**`
- `cli_manifests/opencode/artifacts.lock.json`
- `cli_manifests/opencode/snapshots/1.18.31/**`
- `cli_manifests/opencode/reports/1.18.31/**`
- `cli_manifests/opencode/versions/1.18.31.json`
- `cli_manifests/opencode/wrapper_coverage.json`
- `cli_manifests/support_matrix/current.json`
- `docs/specs/unified-agent-api/support-matrix.md`
- `crates/agent_api/src/runtime_support_data.rs`
- `docs/specs/unified-agent-api/non-tui-support-debt.md`

## Read-only inputs

- `docs/agents/lifecycle/opencode-maintenance/OPS_PLAYBOOK.md`
- `docs/agents/lifecycle/opencode-maintenance/CI_WORKFLOWS_PLAN.md`
- `docs/agents/lifecycle/opencode-maintenance/governance/execute-agent-maintenance-prompt.md`
- `.github/workflows/agent-maintenance-open-pr.yml`

## Ordered repo commands

- `cargo fmt --all`
- `cargo run -p xtask -- codex-validate --root cli_manifests/opencode`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix --check`
- `cargo run -p xtask -- capability-matrix-audit`
- `make preflight`

## Exact green gates

- `cargo fmt --all`
- `cargo run -p xtask -- codex-validate --root cli_manifests/opencode`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix --check`
- `cargo run -p xtask -- capability-matrix-audit`
- `make preflight`

## Recovery

Packet recovery is a maintainer action run from outside a relay session. An agent executing this packet inside a relay session must not run any command named in this Recovery section, including its notes.

- recreate packet command: `cargo run -p xtask -- refresh-agent --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --write`
- reopen pr body path: `docs/agents/lifecycle/opencode-maintenance/governance/pr-summary.md`
- reopen pr branch: `automation/opencode-maintenance-1.18.31`
- notes:
- If PR creation fails after packet generation, rerun packet regeneration from the frozen request and reopen the PR from the generated pr-summary path.
- If the local execution-host preflight (local Codex CLI host via execute-agent-maintenance) fails, fix the Codex binary/auth state and rerun `execute-agent-maintenance --dry-run` before write mode.

## Dry-run to write relay

Starting the relay is a maintainer action run from outside a relay session. An agent executing this packet inside a relay session must not run either command. The maintainer uses the `run_id` printed by the dry-run output, replacing `RUN_ID_FROM_DRY_RUN` before invoking write mode.

```sh
cargo run -p xtask -- execute-agent-maintenance --dry-run --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml
cargo run -p xtask -- execute-agent-maintenance --write --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --run-id RUN_ID_FROM_DRY_RUN
```

## Exact closeout command

After the declared green gates pass, the actor handed the packet PR records the closeout. An agent executing this packet inside a relay session must not run this command.

```sh
cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json
```

## Exact maintained-agent prompt

```md
# Packet PR Maintenance Prompt (`1.18.31`)

This template renders the exact maintained-agent prompt for `opencode` packet execution.
`docs/agents/lifecycle/opencode-maintenance/HANDOFF.md` remains canonical and `governance/pr-summary.md` is derivative.

@codex

## Goal

Execute the automated maintenance packet for `opencode` target `1.18.31`.

## Frozen request contract

- Read `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml` before changing code or docs.
- Read the packet-owned `support_surface_audit` block before deciding whether the run can succeed.
- Treat `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md` as canonical for writable surfaces, read-only inputs, ordered commands, green gates, and recovery.
- Treat `.github/workflows/agent-maintenance-open-pr.yml` as the opening workflow source.
- Never invoke `execute-agent-maintenance`, `prepare-agent-maintenance`, or `refresh-agent`. If this prompt was delivered by `execute-agent-maintenance`, that process is the executor and is already running; lifecycle queries remain available. `HANDOFF.md` is the agent's contract for writable surfaces, read-only inputs, ordered commands, green gates, and the freeze step; its relay and recovery sections describe maintainer actions that start or recreate a run, and its closeout section identifies the closeout actor.
- Do not write outside the execution contract frozen in the request packet.

## Manifest inputs

- `cli_manifests/opencode/README.md`
- `cli_manifests/opencode/VALIDATOR_SPEC.md`
- `cli_manifests/opencode/RULES.json`
- `cli_manifests/opencode/SCHEMA.json`
- `cli_manifests/opencode/current.json`
- `cli_manifests/opencode/latest_validated.txt`
- `cli_manifests/opencode/wrapper_coverage.json`

## Required workflow

1. Compare the current validated baseline from `cli_manifests/opencode/latest_validated.txt` against the target `1.18.31` artifacts.
2. Use `support_surface_audit` to classify newly discovered non-TUI surface, preexisting non-TUI debt, required uplifts, and allowed deferrals.
3. For each `deferred_preexisting_gaps` row that also appears in `required_uplifts_this_run`, decide whether its `defer_reason` still holds at `1.18.31`. If it does, re-authorize that identity's existing debt row or rows in `docs/specs/unified-agent-api/non-tui-support-debt.md` in place:
   - set `authorized_at_version` to `1.18.31`;
   - set `scope_target_triples` so the rows together cover exactly the targets whose `cli_manifests/opencode/reports/1.18.31/coverage.<target>.json` lists the surface, with no target in two rows;
   - set `authorization_evidence_ref` to `cli_manifests/opencode/reports/1.18.31/coverage.any.json`.

   Change no other field and add no row. If the blocker no longer holds, treat the row as an uplift.
4. Land bounded wrapper/backend/manifest/publication updates for every remaining row in `required_uplifts_this_run`. Newly discovered surface is never deferred (maintenance-request contract field invariant 3), and no debt row may be added.
5. Refresh or create version-scoped manifest artifacts under `cli_manifests/opencode/snapshots/1.18.31/`, `cli_manifests/opencode/reports/1.18.31/`, and `cli_manifests/opencode/versions/1.18.31.json` as required by the packet.
6. An agent executing this packet inside a relay session does not run `close-agent-maintenance` or `prepare-agent-closeout`; after the declared green gates pass, the actor handed the packet PR records the closeout.

## Done criteria

- Changes stay within the writable surfaces frozen in `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`.
- Every row in `required_uplifts_this_run` is uplifted, or is a preexisting debt row re-authorized at `1.18.31`; newly discovered surface is never deferred.
- `cargo run -p xtask -- codex-validate --root cli_manifests/opencode` passes.
- `cargo run -p xtask -- maintenance-audit-status --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml` exits 0.
- The remaining ordered commands and green gates from `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md` pass or are captured in maintainer follow-up notes.

```
