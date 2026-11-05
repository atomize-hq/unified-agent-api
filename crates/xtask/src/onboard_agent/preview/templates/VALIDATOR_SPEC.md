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

- The expected target is the registry entry's one canonical target, `{{REQUIRED_TARGET}}`
  (platform `{{REQUIRED_PLATFORM}}`), which is also the required target. Onboarding refuses a
  root with more than one canonical target until a required-target policy exists.
- Snapshots declare `"tool": "{{TOOL_NAME}}"`, which `SCHEMA.json` requires.
- Onboarding installs the bootstrap state: `min_supported.txt`, `latest_validated.txt` and both
  per-target pointer files set to `none`, a `wrapper_coverage.json` that claims no coverage, and a
  bootstrap `current.json` that lists the expected target and no inputs. While
  `latest_validated.txt` is `none`, the validator accepts that bootstrap `current.json`; promotion
  replaces it with the promoted union snapshot. The root validates as onboarding writes it.
- Per-version metadata (`versions/<version>.json`) is written by `xtask manifest-version-metadata`
  once a version is acquired. Onboarding writes none: no version has been acquired.
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
