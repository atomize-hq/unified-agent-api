# OpenCode Parity Validator Spec (`xtask codex-validate`)

This document defines the deterministic validator contract for `cli_manifests/opencode/`.

## Command

`cargo run -p xtask -- codex-validate --root cli_manifests/opencode`

## Inputs

- `RULES.json`
- `SCHEMA.json`
- `VERSION_METADATA_SCHEMA.json`
- committed files under `cli_manifests/opencode/`

## Current bootstrap expectations

- `min_supported.txt` and `latest_validated.txt` exist and may be `none`
- per-target pointer files exist for `linux-x64`, `darwin-arm64`, and `win32-x64`
- `versions/1.4.9.json` exists with `status: snapshotted`
- `snapshots/1.4.9/union.json` and the observed per-target snapshot exist
- `wrapper_coverage.json` exists and remains bounded to the canonical `run --format json` wrapper surface

## Scope boundaries

- The validator checks manifest-root evidence only.
- It does not publish backend support.
- It does not publish unified support.

## Pointer posture

- Root pointers stay `none` until Linux-first validation and support evidence are committed.
- Target pointer files must always exist even when the value is unknown.

## Coverage posture

- `wrapper_coverage.json` records committed wrapper-owned coverage declarations.
- Missing reports are acceptable while a version remains `snapshotted`.
- Reports become required only when version metadata advances into reported, validated, or supported states.
