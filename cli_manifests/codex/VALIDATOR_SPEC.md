# Codex Parity Validator Spec (`xtask codex-validate`)

This document specifies the deterministic validator used to enforce `cli_manifests/codex/RULES.json` invariants in CI.
The support-publication contract itself lives in `docs/specs/unified-agent-api/support-matrix.md`;
this validator only enforces the parity artifacts that feed validation and support state.

## Goals

- Deterministically enforce semantic invariants that JSON Schema cannot express (cross-file consistency, pointer invariants, rationale requirements).
- Produce stable, actionable error output (sorted, path-addressable) suitable for CI logs.
- Support an optional, purely mechanical `fix` mode for safe automations (no LLM decisions, no network).

## Non-Goals

- No upstream downloads.
- No “best effort” guessing: violations fail fast (unless explicitly suppressed via flags).
- No interactive prompts.

## Command Interface

Binary: `xtask`  
Subcommand: `codex-validate`

### Usage

`cargo run -p xtask -- codex-validate [OPTIONS]`

### Options

- `--codex-dir <PATH>` (default: `cli_manifests/codex`)
  - Root directory containing `SCHEMA.json`, `RULES.json`, pointer files, snapshots, reports, and version metadata.
- `--mode <check|fix>` (default: `check`)
  - `check`: validate only; exit non-zero on any violation.
  - `fix`: apply **purely mechanical** fixes (see “Fix Mode”), then re-run `check`.
- `--rules <PATH>` (default: `<codex-dir>/RULES.json`)
- `--schema <PATH>` (default: `<codex-dir>/SCHEMA.json`)
- `--version-metadata-schema <PATH>` (default: `<codex-dir>/VERSION_METADATA_SCHEMA.json`)
- `--json` (default: off)
  - Emit a machine-readable JSON report to stdout in addition to human text.

## Inputs (authoritative)

The validator reads:
- `RULES.json` as the semantic contract (expected targets, required target, pointer invariants, etc.)
- `SCHEMA.json` and `VERSION_METADATA_SCHEMA.json` as shape contracts
- The workspace files under `--codex-dir` (snapshots, reports, pointers, metadata)

No other sources are consulted for standalone root validation.

When `--root` resolves inside a discoverable Cargo workspace (an ancestor `Cargo.toml` contains
`[workspace]`), the validator also enforces the support-matrix publication artifact under
`cli_manifests/support_matrix/current.json` against the committed workspace root set, and reads
the inputs that [Depth records](#depth-records) lists. Outside a
discoverable workspace layout, `codex-validate` reads only files under the supplied root.

## Output

### Human text output (default)

- One line summary:
  - `OK: manifest-validate` on success (the retained `codex-validate` alias prints the same string)
  - `FAIL: <n> violations` on failure
- Then one violation per line, stable-sorted:
  - `<CODE> <SEVERITY> <FILEPATH> <JSON_POINTER?> <MESSAGE>`

Sorting:
1. `CODE` (lexicographic)
2. `FILEPATH` (lexicographic)
3. `JSON_POINTER` (lexicographic; empty last)
4. `MESSAGE` (lexicographic)

### JSON output (`--json`)

Top-level object:
- `ok` (bool)
- `violations` (array of objects), each:
  - `code` (string)
  - `severity` (`error` only in v1)
  - `path` (string; file path)
  - `json_pointer` (string, optional)
  - `message` (string)
  - `details` (object, optional)

## Exit Codes

- `0`: success (no violations)
- `2`: violations found
- `3`: validator error (I/O, parse error, invalid RULES/schema files)

## Determinism/Safety

- No network access.
- No nondeterministic ordering.
- `fix` performs only mechanical actions defined below.

## Checks (normative)

### `iu_notes`: `intentionally_unsupported` requires rationale

File: `wrapper_coverage.json`

For each unit with `level == "intentionally_unsupported"`:
- Require `note` exists and `trim().len() > 0`.

Applies to:
- command entries (`coverage[]`)
- nested flags (`coverage[].flags[]`)
- nested args (`coverage[].args[]`)

Violation:
- `IU_NOTE_MISSING`
  - Include unit type, `path`, and `key`/`name` where applicable.

### `report_intentionally_unsupported`: IU must not be reported as missing

Files:
- `reports/<version>/coverage.*.json`

Policy:
- A unit classified as `intentionally_unsupported` (explicitly, or via IU subtree inheritance per ADR 0004) must be:
  - absent from `deltas.missing_commands`, `deltas.missing_flags`, and `deltas.missing_args`, and
  - present under `deltas.intentionally_unsupported` with a non-empty `note`.

Rules:
- No `missing_*` entry may have `wrapper_level == "intentionally_unsupported"`.
- Every entry in `deltas.intentionally_unsupported` MUST include:
  - `wrapper_level == "intentionally_unsupported"`, and
  - a non-empty `note` (after trimming whitespace).

Sorting (deterministic):
- If `deltas.intentionally_unsupported` is present, it MUST be stable-sorted by:
  1. unit kind order: commands (no `key` and no `name`), then flags (`key`), then args (`name`)
  2. `path` (lexicographic token compare, then length)
  3. `key` (flags only) or `name` (args only)

Violations:
- `REPORT_MISSING_INCLUDES_INTENTIONALLY_UNSUPPORTED`
- `REPORT_IU_NOTE_MISSING`
- `REPORT_IU_NOT_SORTED`

### `parity_exclusions`: excluded units must not become parity work

Files:
- `RULES.json` (`parity_exclusions`)
- `wrapper_coverage.json`
- `reports/<version>/coverage.*.json`

Policy:
- Interactive TUI mode and TUI-only help-surface units are excluded from parity work-queue deltas.

Rules config invariants (RULES.json):
- `parity_exclusions.schema_version` must be `1`.
- `parity_exclusions.units[]` entries must be unique by identity:
  - command: `path`
  - flag: `path` + `key`
  - arg: `path` + `name`
- `parity_exclusions.units[]` entries must be stable-sorted by `(unit, path, key_or_name)`.
- Every `parity_exclusions.units[]` entry must include a non-empty `note`.

Wrapper coverage invariant:
- `wrapper_coverage.json` must not contain any identity listed in `parity_exclusions.units[]` (no wrapper claim for excluded TUI surfaces).

Report invariant:
- No report may list an excluded identity under:
  - `deltas.missing_commands`
  - `deltas.missing_flags`
  - `deltas.missing_args`

Violations:
- `PARITY_EXCLUSIONS_SCHEMA_VERSION`
- `PARITY_EXCLUSIONS_MISSING_UNITS`
- `PARITY_EXCLUSIONS_NOTE_MISSING`
- `PARITY_EXCLUSIONS_INVALID_ENTRY`
- `PARITY_EXCLUSIONS_DUPLICATE`
- `PARITY_EXCLUSIONS_NOT_SORTED`
- `WRAPPER_COVERAGE_INCLUDES_EXCLUDED`
- `REPORT_MISSING_INCLUDES_EXCLUDED`

### `pointers`: pointer materialization, format, and consistency

Pointer files are defined by `RULES.json.storage.pointers.*`.

#### Materialization

For every `target_triple` in `RULES.json.union.expected_targets` require existence of:
- `pointers/latest_supported/<target_triple>.txt`
- `pointers/latest_validated/<target_triple>.txt`

Violation:
- `POINTER_MISSING_FILE`

#### Format

Each pointer file must:
- be UTF-8 text
- contain exactly one line and end with a trailing newline
- value is either `none` or a stable semver matching `RULES.json.versioning.pointers.stable_semver_pattern`

Violations:
- `POINTER_INVALID_FORMAT`
- `POINTER_INVALID_VALUE`

#### Linux compatibility invariants

- `latest_validated.txt` must exist, must be a stable semver (not `none`), and must equal:
  - `pointers/latest_validated/<required_target>.txt`
- `current.json` must exist and be byte-for-byte identical to:
  - `snapshots/<latest_validated>/union.json`

Violations:
- `POINTER_LATEST_VALIDATED_MISMATCH`
- `CURRENT_JSON_NOT_EQUAL_UNION`

#### Pointer → metadata/snapshot consistency

If a pointer value is `<semver>`:
- Require `versions/<semver>.json` exists.
- Require `snapshots/<semver>/union.json` exists.
- Require `snapshots/<semver>/<target_triple>.json` exists (for that pointer’s target).

If `latest_supported/<target_triple>.txt` is `<semver>`:
- Require `versions/<semver>.json.coverage.supported_targets` contains `<target_triple>`.

If `latest_validated/<target_triple>.txt` is `<semver>`:
- Require `versions/<semver>.json.coverage.supported_targets` contains `<target_triple>`.
- Require `versions/<semver>.json.validation.passed_targets` contains `<target_triple>`.

Violations:
- `POINTER_REFERENCES_MISSING_VERSION_METADATA`
- `POINTER_REFERENCES_MISSING_SNAPSHOT`
- `POINTER_INCONSISTENT_WITH_VERSION_METADATA`

### `version_metadata_validation_sets`: validation sets are explicit and disjoint

File: `versions/<version>.json`

If `validation` is present:
- `passed_targets`, `failed_targets`, and `skipped_targets` must be pairwise disjoint.
- All targets in those sets must be members of `RULES.json.union.expected_targets`.
- The required target must appear in **exactly one** of the three sets.

Violations:
- `VALIDATION_TARGET_SETS_OVERLAP`
- `VALIDATION_TARGET_NOT_EXPECTED`
- `VALIDATION_REQUIRED_TARGET_NOT_EXPLICIT`

### `current_json_identity`: `current.json` corresponds to the latest validated promotion snapshot

Using `latest_validated.txt`:
- Parse union snapshot `snapshots/<latest_validated>/union.json`
- Ensure `current.json` is byte-for-byte identical to that union snapshot
- Ensure `current.json.inputs[]` includes the required target (Linux-first v1)

Violations:
- `CURRENT_JSON_NOT_EQUAL_UNION` (same as pointer check)
- `CURRENT_JSON_MISSING_REQUIRED_TARGET`

### `support_matrix_publication`: committed support publication is present and readable

File:
- `../support_matrix/current.json`

Policy:
- `xtask codex-validate` treats the committed support-matrix JSON as a required publication
  artifact when the workspace root can be resolved from `--root`.
- The artifact must exist before publication consistency checks run.
- Its `schema_version` is the revision its rows are written in: 2 when a row carries depth facts
  and 1 otherwise (see [Depth records](#depth-records)).

Violations:
- `SUPPORT_MATRIX_ARTIFACT_MISSING`
- `SUPPORT_MATRIX_INVALID_JSON`
- `SUPPORT_MATRIX_SCHEMA_INVALID`
- any `SUPPORT_MATRIX_*` publication-consistency violation emitted from the shared
  support-matrix consistency validator

### `schemas` (optional convenience)

Validate JSON files against:
- `SCHEMA.json` for snapshots/reports/wrapper coverage
- `VERSION_METADATA_SCHEMA.json` for version metadata

Violations:
- `SCHEMA_INVALID_JSON`
- `SCHEMA_VALIDATION_FAILED`

## Fix Mode (mechanical)

`--mode fix` may apply only these mechanical changes:

1. **Create missing pointer files** for every expected target (if absent) with content:
   - `none\n`
2. **Normalize pointer file formatting**:
   - ensure exactly one line + trailing newline; rewrite if necessary
3. **Normalize `current.json`** to match `snapshots/<latest_validated>/union.json` byte-for-byte:
   - overwrite `current.json` with the union snapshot content

`fix` must not:
- delete arbitrary files (retention pruning is a separate mechanical command)
- modify snapshots, reports, or version metadata (except `current.json` per above)
- invent values other than `none`

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
  names, and `docs/agents/lifecycle/<agent_id>-maintenance/governance/maintenance-closeout.json`;
- through the support publication check, the depth records of every root the support matrix
  publishes, the names of the record files of every other registry root, which the check
  refuses, and each file a record binds that git tracks as a regular file, read for its content
  identity and nothing else.

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
  is there and does not load with the files it names, by path and, where it states one, content
  identity; or it states `proving_run_closeout_written` and names no proving-run closeout; or the
  proving-run closeout it names is not held; or it states `maintenance_closeout_written` and the
  agent's maintenance closeout is not held. An absent lifecycle record is no lifecycle record. A
  closeout is held when the file at its path is a closeout of its kind in form. The lifecycle
  record names the proving-run closeout by path alone, so this validator decides no more than that.

Checks 1 and 3 are decided against the maintenance request only. An onboarding generation has only
its approval's Event beside the record, and the onboarding charter names no second statement of its
P and O yet, so until it does neither check reaches the create lane.

Checks 5, 6, 7, 8 and 12 are decided through the support publication. Inside a workspace the
validator requires each row of `cli_manifests/support_matrix/current.json` to be the row the
support matrix derives from the committed tree, depth facts included: for a depth-enrolled row, the
facts that its record and the files it binds decide under "Support depth" of
`docs/specs/unified-agent-api/support-matrix.md`. Violations:

- `SUPPORT_MATRIX_DEPTH_MISMATCH` (checks 5, 6, 7, 8 and 12): a row publishes depth facts other
  than those, publishes none while its record makes it depth-enrolled, or publishes some while it
  is not depth-enrolled.
- `SUPPORT_MATRIX_SCHEMA_INVALID`: the artifact's `schema_version` is not the revision its rows are
  written in, 2 when a row carries depth facts and 1 otherwise.
- `SUPPORT_MATRIX_ROOT_READ_ERROR`: the support matrix cannot derive a root's rows: it cannot read
  a record, a root holds one without adopting Annex B, a registry root it does not publish holds
  one, a record's obligations part states, for an operation, an obligation set other than the
  one its policy part resolves, or a record binds evidence and git cannot list the files it
  tracks in the workspace. Publication fails rather than publishing that version's rows as rows
  without depth facts.

Checks 2, 4, 9 and 11 compare a merge result with the integration branch tip, which this
validator does not read.

### Present behavior

The checks above are implemented in `crates/xtask/src/manifest_validate/depth.rs`, except checks
5, 6, 7, 8 and 12: the support matrix derives depth facts in `crates/xtask/src/support_matrix/`,
and its consistency check compares each published row with the derived one. No root holds a
depth record and no request carries a `[support_depth]` table, so none of them fails on the
committed tree. The relay's validating loader still rejects the table as an unknown key.
