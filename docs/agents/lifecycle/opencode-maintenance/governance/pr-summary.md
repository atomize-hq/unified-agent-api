<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# PR summary

Automated maintenance packet for `opencode` target `1.18.31`.

- canonical execution contract: `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md`
- request artifact: `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`
- branch: `automation/opencode-maintenance-1.18.31`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- prompt sha256: `478131e737048e5124968e22d6085bd4af522fdda345010febd475a434dac7c7`

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


## Next step

Follow `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md` exactly. This PR summary is derivative from the same execution-packet renderer.

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
