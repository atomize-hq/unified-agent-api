# {{DISPLAY_NAME}} Parity Validator Spec (`xtask manifest-validate`)

This document defines the deterministic validator contract for `{{MANIFEST_ROOT}}/`.
`xtask onboard-agent --write` scaffolded it for `{{AGENT_ID}}`, together with `RULES.json`,
`SCHEMA.json` and `VERSION_METADATA_SCHEMA.json`. Later changes to these files are reviewed edits.

## Command

`cargo run -p xtask -- manifest-validate --root {{MANIFEST_ROOT}}`

Optional `--tip <REV>` compares the workspace with the named integration-tip commit. It requires
Git and cannot be combined with `--mode fix` (exit 3 before any write).

## Inputs

- `RULES.json`
- `SCHEMA.json`
- `VERSION_METADATA_SCHEMA.json`
- committed files under `{{MANIFEST_ROOT}}/`

## Current committed expectations

- The expected targets are the registry entry's canonical targets, in registry order:
{{TARGET_LIST}}
- The required target is `{{REQUIRED_TARGET}}`.
- Snapshots declare `"tool": "{{TOOL_NAME}}"`, which `SCHEMA.json` requires.
- Onboarding writes no version metadata, snapshots, reports or pointer values. The bootstrap
  `current.json` lists the expected targets only, so the root does not pass this validator until
  the runtime lane has acquired and promoted a first version; per-version metadata
  (`versions/<version>.json`) is written by `xtask manifest-version-metadata` at that point.
- `RULES.json` declares no `parity_exclusions` and no `acquisition` block. Both are reviewed
  follow-up edits once the upstream surface and release source are known.

## Scope boundaries

- The validator checks manifest-root evidence only.
- It does not publish backend support or unified support.

## Pointer posture

- Per-target pointer files must always exist, even when the value is `none`.
- Per-target `latest_validated` promotion stays independent of the root compatibility pointer.

## Coverage posture

- Missing reports are acceptable while a version remains `snapshotted`.
- Reports become required only when version metadata advances into reported, validated, or
  supported states.
- `RULES.json` refuses promotion while the union is incomplete
  (`union.promotion_policy.allow_promote_when_incomplete = false`).

## Depth records

This root does not adopt the depth record that Annex B of `docs/specs/support-depth-contract.md`
defines. `RULES.json` names no `storage.committed.reports.depth_record`, so the root MUST NOT hold
a depth record, and `manifest-validate` reports `DEPTH_RECORD_UNADOPTED` for any record found
under it. Adopting Annex B is a reviewed change to this spec and to `RULES.json`; onboarding does
not make it.
