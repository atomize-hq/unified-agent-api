# Claude Code CLI Manifests (`cli_manifests/claude_code`)

This directory is the home for **Claude Code CLI snapshot artifacts** used to maintain wrapper parity over time.

## Ops Playbook

Maintainer runbook (Release Watch triage, snapshot update + review checklist, intentionally-unwrapped policy, and promotion criteria): [OPS_PLAYBOOK.md](OPS_PLAYBOOK.md)

## CI Plan

CI triggers + binary acquisition + PR automation plan (v1 target state): [CI_WORKFLOWS_PLAN.md](CI_WORKFLOWS_PLAN.md)

## Generate / Update (`current.json`)

Canonical generator command:

`cargo run -p xtask -- claude-snapshot --claude-binary <PATH_TO_CLAUDE> --out-file cli_manifests/claude_code/snapshots/<version>/<target>.json --capture-raw-help --raw-help-target <target> --supplement cli_manifests/claude_code/supplement/commands.json`

Notes:
- The generator must be pointed at a local `claude` binary; it does not download binaries and should not use the network.

## Normative Spec (Schema + Rules)

The source of truth for artifact shapes and merge/compare semantics is:
- `SCHEMA.json` (JSON Schema)
- `RULES.json` (merge + comparison rules)
- `VALIDATOR_SPEC.md` (deterministic RULES enforcement spec)

CI should hard-fail if any committed artifact does not validate against `SCHEMA.json`.

Versioning/timestamps:
- `binary.semantic_version` is the upstream Claude Code version (SemVer-like); promotion pointers accept stable `MAJOR.MINOR.PATCH` only.
- CI should set committed timestamps deterministically (or omit them where allowed) to avoid churn across reruns.

## Support Publication Contract

- `docs/specs/unified-agent-api/support-matrix.md` is the canonical support-publication contract.
- `docs/specs/unified-agent-api/capability-matrix.md` remains a separate backend inventory and
  should not be read as the support policy contract.

## On-disk Layout (v1)

- `min_supported.txt` — minimum supported Claude Code CLI version (single semver line).
- `latest_validated.txt` — latest Claude Code CLI version that passed the validation matrix (single semver line).
- `current.json` — generated snapshot for `latest_validated.txt`.

Optional/generated:

- `raw_help/<semantic_version>/**` — raw `--help` captures (for debugging help parser drift; stored as CI artifacts, not committed):
  - Root help: `raw_help/<semantic_version>/help.txt`
  - Per-command help: `raw_help/<semantic_version>/commands/<token1>/<token2>/help.txt`
- `supplement/commands.json` — small, explicit hand-maintained supplements for known “help omissions”.

## Conventions

- Keep JSON deterministic: stable sort order, avoid timestamps in fields that would churn diffs unnecessarily (use `collected_at` only).
- Treat `min_supported.txt` and `latest_validated.txt` as the only authoritative promotion pointers.
- Retention: keep snapshots + reports for the last 3 validated versions (sliding window), plus the versions referenced by `min_supported.txt` and `latest_validated.txt`.

## Snapshot Schema (v1)

`current.json` is a single JSON object with:
- `snapshot_schema_version` (int): must be `1`.
- `tool` (string): must be `claude-code-cli`.
- `collected_at` (RFC3339 string): snapshot generation timestamp.
- `binary` (object):
  - `sha256` (string)
  - `size_bytes` (int)
  - `platform` (object): `os` (string), `arch` (string)
  - `target_triple` (string): required
  - `version_output` (string)
  - `semantic_version` (string): required
  - `channel` (string, optional): `stable|beta|nightly|unknown` (when derivable)
  - `commit` (string, optional)
- `commands` (array; stable-sorted):
  - `path` (array of strings): command/subcommand tokens; the root `claude` command is represented as `[]`
  - `about` (string, optional)
  - `usage` (string, optional)
  - `stability` (string, optional): `stable|experimental|beta|deprecated|unknown` (when derivable)
  - `platforms` (array of strings, optional): `linux|macos|windows|wsl`
  - `args` (array, optional): help-derived positional args (when discoverable)
  - `flags` (array, optional): help-derived flags/options:
    - `long` (string, optional): includes leading `--` (example: `--json`)
    - `short` (string, optional): includes leading `-` (example: `-j`)
    - `takes_value` (bool)
    - `value_name` (string, optional): as shown in help (example: `PATH`)
    - `repeatable` (bool, optional)
    - `stability` (string, optional)
    - `platforms` (array, optional): `linux|macos|windows|wsl`
- `features` (object, optional): reserved for future metadata; unused by v1 Claude snapshots.
- `known_omissions` (array of strings, optional): records applied supplements for review visibility.

Note: per-target snapshots are schema v1 (`snapshots/<version>/<target_triple>.json`). The multi-platform union snapshot is schema v2 (`snapshots/<version>/union.json`). When present, `current.json` is the union snapshot for `latest_validated.txt`.

## Deterministic Ordering Rules (v1)

- `commands` are sorted lexicographically by `path` tokens (token-by-token; shorter prefix first).
- `flags` are sorted by `(long, short)`:
  - missing `long` sorts after present
  - missing `short` sorts after present
  - ties preserve original discovery order (stable sort)

## Help Supplements (v1)

`supplement/commands.json` exists to make known help gaps explicit and reviewable (without heuristics).

An example file is provided at `supplement/commands.example.json` — keep `supplement/commands.json` for real, discovered omissions only.

Schema:
- `version` (int): must be `1`
- `commands` (array):
  - `path` (array of strings): command tokens
  - `platforms` (array of strings, optional): `linux|macos|windows|wsl`
  - `note` (string): why this supplement exists

Application semantics:
- If a supplemented `path` is missing from `claude --help` discovery, the generator inserts a `commands[]` entry with that `path`.
- If `platforms` is present, it sets/overrides `commands[].platforms`.
- Help-derived fields (`about`, `usage`, `flags`, `args`) are never fabricated by supplements.
- For each applied supplement, `known_omissions` appends: `supplement/commands.json:v1:<path-joined-by-space>`.

Multi-platform note:
- When generating a union snapshot, apply supplements per-target before merging so `available_on` correctly reflects platform availability.

## Coverage Reports (planned)

Coverage reports are generated by comparing:
- upstream union snapshot (`current.json` / `snapshots/<version>/union.json`)
- wrapper coverage manifest (`wrapper_coverage.json`)

Standard report outputs (committed):
- `reports/<version>/coverage.any.json`
- `reports/<version>/coverage.all.json` (only when union `complete=true`)
- `reports/<version>/coverage.<target_triple>.json`

## Version Status Metadata (planned)

To avoid ambiguity in terms like “validated”, we track per-version workflow status:
- `versions/<version>.json` (schema: `VERSION_METADATA_SCHEMA.json`)

Statuses:
- `snapshotted`: snapshots generated and schema-valid; no wrapper claims.
- `reported`: coverage report generated; work queue available; no wrapper validation claim.
- `validated`: passed the validation matrix and is promotion-grade for `latest_validated.txt` / `current.json`.
- `supported`: wrapper coverage meets the support-publication policy for this version (stronger than validated). Concretely, all surfaced commands/flags/args on all expected targets are covered as `explicit|passthrough|intentionally_unsupported` (no missing/unknown/unsupported), and any `intentionally_unsupported` requires a rationale note.

## Per-Target Pointers (planned)

Pointers per target triple (committed):
- `pointers/latest_supported/<target_triple>.txt`
- `pointers/latest_validated/<target_triple>.txt`

Compatibility:
- `latest_validated.txt` remains the canonical promotion pointer for the required target (`x86_64-unknown-linux-musl`) and must match `pointers/latest_validated/x86_64-unknown-linux-musl.txt`.

Promotion (v1):
- Linux-first promotion is allowed even when the union snapshot is incomplete (`complete=false`), as long as Linux passed validation and is supported by coverage.
- macOS/Windows pointers advance independently when their signals are available.

Pointer file format:
- Each pointer file contains a single line (`<semver>` or `none`) and must end with a newline.
- Pointer files should always exist for every expected target; use `none` until a value is known.

Retention enforcement:
- Pruning snapshots/reports outside the sliding window should be done mechanically/deterministically by CI tooling (no LLM decisions).
