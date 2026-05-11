<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# PR summary

Automated maintenance packet for `opencode` target `1.14.40`.

- canonical execution contract: `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md`
- request artifact: `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`
- branch: `automation/opencode-maintenance-1.14.40`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- prompt sha256: `0060876bb5f17815a560ddee0f7f010ed32587b3fbecd5a47061b44be3d7327b`

## Next step

Follow `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md` exactly. This PR summary is derivative from the same execution-packet renderer.

## Exact maintained-agent prompt

```md
# Packet PR Maintenance Prompt (`1.14.40`)

This template renders the exact maintained-agent prompt for `opencode` packet execution.
`docs/agents/lifecycle/opencode-maintenance/HANDOFF.md` remains canonical and `governance/pr-summary.md` is derivative.

@codex

## Goal

Execute the automated maintenance packet for `opencode` target `1.14.40`.

## Frozen request contract

- Read `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml` before changing code or docs.
- Treat `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md` as canonical for writable surfaces, read-only inputs, ordered commands, green gates, and recovery.
- Treat `.github/workflows/agent-maintenance-open-pr.yml` as the opening workflow source.
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

1. Compare the current validated baseline from `cli_manifests/opencode/latest_validated.txt` against the target `1.14.40` artifacts.
2. Refresh or create version-scoped manifest artifacts under `cli_manifests/opencode/snapshots/1.14.40/`, `cli_manifests/opencode/reports/1.14.40/`, and `cli_manifests/opencode/versions/1.14.40.json` as required by the packet.
3. Update `crates/opencode/**` and `crates/agent_api/**` only when the target artifact delta requires wrapper or backend changes.
4. Leave closeout manual; record it only with `close-agent-maintenance` after the declared green gates pass.

## Done criteria

- Changes stay within the writable surfaces frozen in `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`.
- `cargo run -p xtask -- codex-validate --root cli_manifests/opencode` passes.
- The remaining ordered commands and green gates from `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md` pass or are captured in maintainer follow-up notes.

```
