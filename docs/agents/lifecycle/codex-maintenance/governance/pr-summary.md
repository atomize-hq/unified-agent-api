<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# PR summary

Automated maintenance packet for `codex` target `0.156.1`.

- canonical execution contract: `docs/agents/lifecycle/codex-maintenance/HANDOFF.md`
- request artifact: `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
- branch: `automation/codex-maintenance-0.156.1`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- prompt sha256: `b215b743126c352c23e57fd45c3c32468abf0574a1f9a48b20a76ab56588c0f8`

## Support-surface audit

- required: `true`
- pre-run debt count: `2`
- expected post-run debt count: `2`
- discovered upstream surface rows: `56`
- preexisting unsupported rows: `2`
- required uplifts this run:
- `codex agents` `agents` via `unbaselined_gap`
- `codex completion` `completion` via `unbaselined_gap`
- `codex migrate-rollouts` `migrate-rollouts` via `unbaselined_gap`
- `codex queue` `queue` via `unbaselined_gap`
- `codex app-server` `--code-mode-host` via `unbaselined_gap`
- `codex app-server daemon update` `--from-cli` via `unbaselined_gap`
- `codex app-server daemon update` `--yes` via `unbaselined_gap`
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
- `codex mcp login` `--no-browser` via `unbaselined_gap`
- `codex mcp login` `--oauth-client-registration` via `unbaselined_gap`
- `codex migrate-rollouts` `--apply` via `unbaselined_gap`
- `codex migrate-rollouts` `--json` via `unbaselined_gap`
- `codex migrate-rollouts` `--max-mib-per-second` via `unbaselined_gap`
- `codex migrate-rollouts` `--thread` via `unbaselined_gap`
- `codex migrate-rollouts` `--verbose` via `unbaselined_gap`
- `codex queue` `--message` via `unbaselined_gap`
- `codex queue` `--thread` via `unbaselined_gap`
- `codex` `--approve-for-me` via `unbaselined_gap`
- `codex` `--no-daemon` via `unbaselined_gap`
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


## Next step

Follow `docs/agents/lifecycle/codex-maintenance/HANDOFF.md` exactly. This PR summary is derivative from the same execution-packet renderer.

## Exact maintained-agent prompt

```md
# Packet PR Maintenance Prompt (`0.156.1`)

This template renders the exact maintained-agent prompt for `codex` packet execution.
`docs/agents/lifecycle/codex-maintenance/HANDOFF.md` remains canonical and `governance/pr-summary.md` is derivative.

@codex

## Goal

Execute the automated maintenance packet for `codex` target `0.156.1`.

## Frozen request contract

- Read `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml` before changing code or docs.
- Read the packet-owned `support_surface_audit` block before deciding whether the run can succeed.
- Treat `docs/agents/lifecycle/codex-maintenance/HANDOFF.md` as canonical for writable surfaces, read-only inputs, ordered commands, green gates, and recovery.
- Treat `.github/workflows/agent-maintenance-open-pr.yml` as the opening workflow source.
- Never invoke `execute-agent-maintenance`, `prepare-agent-maintenance`, or `refresh-agent`. If this prompt was delivered by `execute-agent-maintenance`, that process is the executor and is already running; lifecycle queries remain available. `HANDOFF.md` is the agent's contract for writable surfaces, read-only inputs, ordered commands, green gates, and the freeze step; its relay and recovery sections describe maintainer actions that start or recreate a run, and its closeout section identifies the closeout actor.
- Do not write outside the execution contract frozen in the request packet.

## Manifest inputs

- `cli_manifests/codex/README.md`
- `cli_manifests/codex/VALIDATOR_SPEC.md`
- `cli_manifests/codex/RULES.json`
- `cli_manifests/codex/SCHEMA.json`
- `cli_manifests/codex/current.json`
- `cli_manifests/codex/latest_validated.txt`
- `cli_manifests/codex/wrapper_coverage.json`

## Required workflow

1. Compare the current validated baseline from `cli_manifests/codex/latest_validated.txt` against the target `0.156.1` artifacts.
2. Use `support_surface_audit` to classify newly discovered non-TUI surface, preexisting non-TUI debt, required uplifts, and allowed deferrals.
3. For each `deferred_preexisting_gaps` row that also appears in `required_uplifts_this_run`, decide whether its `defer_reason` still holds at `0.156.1`. If it does, re-authorize that identity's existing debt row or rows in `docs/specs/unified-agent-api/non-tui-support-debt.md` in place:
   - set `authorized_at_version` to `0.156.1`;
   - set `scope_target_triples` so the rows together cover exactly the targets whose `cli_manifests/codex/reports/0.156.1/coverage.<target>.json` lists the surface, with no target in two rows;
   - set `authorization_evidence_ref` to `cli_manifests/codex/reports/0.156.1/coverage.any.json`.

   Change no other field and add no row. If the blocker no longer holds, treat the row as an uplift.
4. Land bounded wrapper/backend/manifest/publication updates for every remaining row in `required_uplifts_this_run`. Newly discovered surface is never deferred (maintenance-request contract field invariant 3), and no debt row may be added.
5. Refresh or create version-scoped manifest artifacts under `cli_manifests/codex/snapshots/0.156.1/`, `cli_manifests/codex/reports/0.156.1/`, and `cli_manifests/codex/versions/0.156.1.json` as required by the packet.
6. An agent executing this packet inside a relay session does not run `close-agent-maintenance` or `prepare-agent-closeout`; after the declared green gates pass, the actor handed the packet PR records the closeout.

## Done criteria

- Changes stay within the writable surfaces frozen in `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`.
- Every row in `required_uplifts_this_run` is uplifted, or is a preexisting debt row re-authorized at `0.156.1`; newly discovered surface is never deferred.
- `cargo run -p xtask -- codex-validate --root cli_manifests/codex` passes.
- `cargo run -p xtask -- maintenance-audit-status --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml` exits 0.
- The remaining ordered commands and green gates from `docs/agents/lifecycle/codex-maintenance/HANDOFF.md` pass or are captured in maintainer follow-up notes.

```
