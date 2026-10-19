# OpenCode Parity Validator Spec (`xtask codex-validate`)

This document defines the deterministic validator contract for `cli_manifests/opencode/`.

## Command

`cargo run -p xtask -- codex-validate --root cli_manifests/opencode`

## Inputs

- `RULES.json`
- `SCHEMA.json`
- `VERSION_METADATA_SCHEMA.json`
- committed files under `cli_manifests/opencode/`
- inside a workspace, the inputs that [Depth records](#depth-records) lists

## Current committed expectations

- `min_supported.txt` exists and may be `none`
- `latest_validated.txt` records the committed root validated baseline `1.4.11`
- per-target pointer files exist for `linux-x64`, `darwin-arm64`, and `win32-x64`
- `versions/1.4.9.json` exists with `status: snapshotted`
- `versions/1.4.11.json` exists with `status: validated`
- `snapshots/1.4.9/union.json` and the observed per-target snapshot exist
- `wrapper_coverage.json` exists and remains bounded to the canonical `run --format json` wrapper surface

## Scope boundaries

- The validator checks manifest-root evidence only.
- It does not publish backend support.
- It does not publish unified support.

## Pointer posture

- The root compatibility pointer `latest_validated.txt` is promoted to `1.4.11`.
- Per-target pointer files must always exist even when the value is unknown.
- Per-target `latest_validated` promotion remains independent from the root compatibility pointer.

## Coverage posture

- `wrapper_coverage.json` records committed wrapper-owned coverage declarations.
- Missing reports are acceptable while a version remains `snapshotted`.
- Reports become required only when version metadata advances into reported, validated, or supported states.

## Depth records

This root adopts the depth record that Annex B of `docs/specs/support-depth-contract.md` defines.
A depth-enrolled version's record is `reports/<version>/depth-record.json`, and `RULES.json`
names that path as `storage.committed.reports.depth_record`. A root whose `RULES.json` names no
such path does not adopt Annex B. Annex B defines what the record states, its identities and its
acceptance entries, and the support-depth contract states the invariants a record must satisfy.
This spec restates neither. The checks below are the ones that contract's complete mediation
rule 7 requires this spec to state. Their numbers are those of
`docs/agents/lifecycle/support-depth-validation-test-list.md`.

### Inputs

Inside a workspace the validator also reads, for the agent whose registry entry has this
manifest root:

- each `reports/<version>/depth-record.json`, through the record reader of Annex B for that agent
  and version;
- the agent's standing maintenance request,
  `docs/agents/lifecycle/<agent_id>-maintenance/governance/maintenance-request.toml`, for its
  Event (`request_commit`, `request_recorded_at`, `trigger_kind` and `opened_from`),
  `detected_release.target_version` and `[support_depth]` table only. Nothing else in it is
  checked here, so a first-freeze placeholder passes;
- when the root holds a record, the agent's lifecycle record,
  `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/lifecycle-state.json`, the files it
  names, and `docs/agents/lifecycle/<agent_id>-maintenance/governance/maintenance-closeout.json`.

Outside a workspace no registry entry names the record's agent, so the validator cannot read a
record there.

### Checks (normative)

Violations:

- `DEPTH_RECORD_UNADOPTED`: the root holds a record and `RULES.json` names no depth record path
  (record invariant 1).
- `DEPTH_RECORD_UNANCHORED`: the root holds a record and no registry entry names its agent,
  because validation runs outside a workspace or no entry has this manifest root.
- `DEPTH_RECORD_PATH_INVALID`: `RULES.json` names a depth record path other than
  `<root>/reports/<version>/depth-record.json`, where `<root>` is this root's path in the
  workspace. Outside a workspace the root has no such path, so this is not checked there.
- `DEPTH_RECORD_INVALID` (check 13): anything is at a record's path that is not a readable record,
  or a record does not parse, states another revision, or states a P, O or E identity that is not
  the digest of its part.
- `DEPTH_RECORD_NOT_THE_REQUESTS` (check 3): the standing request is for a version whose record
  does not state the request's Event, the P identity of its `[support_depth]` table, and its O
  identity, or no O while the table states none. A request without the table froze no P, so no
  record of its version states what it froze.
- `DEPTH_RECORD_MISSING` (check 1): the standing request's `[support_depth]` table shows that its
  generation froze P, and its version has no record.
- `DEPTH_REQUEST_UNREADABLE`: a standing request is there and cannot be read as text, or is not
  TOML of the shape the inputs above read. An absent request is no request.
- `DEPTH_LIFECYCLE_DANGLING` (check 10): the root holds a record and the agent's lifecycle record
  does not load with the files it names, by path and, where it states one, content identity; or
  it states `proving_run_closeout_written` and names no proving-run closeout; or the proving-run
  closeout it names is not held; or it states `maintenance_closeout_written` and the agent's
  maintenance closeout is not held. A closeout is held when the file at its path is a closeout of
  its kind in form. The lifecycle record names the proving-run closeout by path alone, so this
  validator decides no more than that.

Checks 1 and 3 are decided against the maintenance request only. An onboarding generation has only
its approval's Event beside the record, and the onboarding charter names no second statement of its
P and O yet, so until it does neither check reaches the create lane.

Checks 5, 6, 7, 8 and 12 concern the support publication and are not this validator's. Checks 2, 4,
9 and 11 compare a merge result with the integration branch tip, which this validator does not
read.

### Present behavior

The checks above are implemented in `crates/xtask/src/manifest_validate/depth.rs`. No root holds a
depth record and no request carries a `[support_depth]` table, so none of them fails on the
committed tree. The relay's validating loader still rejects the table as an unknown key.
