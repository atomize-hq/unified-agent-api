# OpenCode CLI Manifests (`cli_manifests/opencode`)

This directory is the canonical manifest root for committed OpenCode CLI evidence in this repo.

## Scope

- Manifest-root evidence only.
- No backend support claim.
- No unified `agent_api` support claim.

## Current posture

- Bootstrap root with two committed evidence bundles:
  - `1.4.11` `linux-x64` validation/report evidence already present in the root
  - `1.4.9` `darwin-arm64` snapshot evidence added as the first local macOS bootstrap
- Promotion pointers remain `none` until Linux-first validation and support publication land.
- `wrapper_coverage.json` is limited to the canonical `opencode run --format json` wrapper surface.

## Update flow

1. Capture or regenerate per-target snapshot evidence under `snapshots/<version>/`.
2. Update `versions/<version>.json` to reflect the workflow state.
3. Advance pointers only when the committed evidence satisfies the promotion rules.
4. Re-run `cargo run -p xtask -- codex-validate --root cli_manifests/opencode`.

## Normative files

- `SCHEMA.json`
- `RULES.json`
- `VALIDATOR_SPEC.md`
- `VERSION_METADATA_SCHEMA.json`

## Notes

- `current.json` is a bootstrap union snapshot for the current local `darwin-arm64` observation.
- `supplement/commands.json` stays empty until a real help omission is observed.
- Raw help captures remain non-authoritative and are not committed here.
