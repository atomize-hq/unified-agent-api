# Packet PR Maintenance Prompt (`{{VERSION}}`)

This template renders the exact maintained-agent prompt for `claude_code` packet execution.
`docs/agents/lifecycle/claude_code-maintenance/HANDOFF.md` remains canonical and `governance/pr-summary.md` is derivative.

@codex

## Goal

Execute the automated maintenance packet for `claude_code` target `{{VERSION}}`.

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

1. Compare the current validated baseline from `cli_manifests/claude_code/latest_validated.txt` against the target `{{VERSION}}` artifacts.
2. Use `support_surface_audit` to classify newly discovered non-TUI surface, preexisting non-TUI debt, required uplifts, and allowed deferrals.
3. Land bounded wrapper/backend/manifest/publication updates for every row in `required_uplifts_this_run`.
4. Refresh or create version-scoped manifest artifacts under `cli_manifests/claude_code/snapshots/{{VERSION}}/`, `cli_manifests/claude_code/reports/{{VERSION}}/`, and `cli_manifests/claude_code/versions/{{VERSION}}.json` as required by the packet.
5. Leave closeout manual; record it only with `close-agent-maintenance` after the declared green gates pass.

## Done criteria

- Changes stay within the writable surfaces frozen in `docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml`.
- No newly discovered non-TUI surface remains unresolved unless the packet records one allowed deferral.
- `cargo run -p xtask -- codex-validate --root cli_manifests/claude_code` passes.
- The remaining ordered commands and green gates from `docs/agents/lifecycle/claude_code-maintenance/HANDOFF.md` pass or are captured in maintainer follow-up notes.
