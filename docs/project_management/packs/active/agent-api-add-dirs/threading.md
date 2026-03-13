# Threading — Universal extra context roots (`agent_api.exec.add_dirs.v1`)

This section makes coupling explicit: contracts/interfaces, dependency edges, and sequencing.

## Contract registry

Ownership note: within this pack, **Owner seam** refers to workstream ownership for implementing
and validating behavior. Normative ownership for `agent_api.*` extension-key semantics is always
`docs/specs/universal-agent-api/extensions-spec.md` (and foundational contract terms live in
`docs/specs/universal-agent-api/contract.md`). If any pack text conflicts with the normative specs,
the specs win.

- **AD-C01 — Core add-dir extension key**
  - **Type**: schema
  - **Owner seam (pack)**: SEAM-1
  - **Normative owner doc**: `docs/specs/universal-agent-api/extensions-spec.md`
  - **Consumers**: SEAM-2/3/4/5
  - **Definition**: `agent_api.exec.add_dirs.v1` is a closed object schema with required
    `dirs: string[]`, `dirs.len()` in `1..=16`, and per-entry trimmed byte bound `<= 1024`. Trimming
    is leading/trailing Unicode whitespace per the owner doc.

- **AD-C02 — Effective add-dir set algorithm**
  - **Type**: config
  - **Owner seam (pack)**: SEAM-2
  - **Consumers**: SEAM-3/4/5
  - **Definition**: the wrapper computes one effective directory list by trimming leading/trailing
    Unicode whitespace (per the owner doc),
    resolving relatives against the run's effective working directory (per
    `docs/specs/universal-agent-api/contract.md` "Working directory resolution (effective working directory)"),
    lexically normalizing,
    verifying `exists && is_dir`, and deduplicating while preserving first occurrence order. This
    list is exported as `Vec<PathBuf>` from
    `backend_harness::normalize::normalize_add_dirs_v1(...)`.
    The backend adapter layer MUST determine the effective working directory per `contract.md` and
    pass it into `normalize_add_dirs_v1(...)` unchanged.

- **AD-C03 — Safe error posture**
  - **Type**: policy
  - **Owner seam (pack)**: SEAM-1
  - **Consumers**: SEAM-2/3/4/5
  - **Definition**: `InvalidRequest` messages for this key MUST use one of the exact safe
    templates `invalid agent_api.exec.add_dirs.v1`,
    `invalid agent_api.exec.add_dirs.v1.dirs`, or
    `invalid agent_api.exec.add_dirs.v1.dirs[<i>]`, where `<i>` is the zero-based failing entry
    index. Backends MUST NOT invent any other `InvalidRequest` message shape for this key.
    Runtime failures surface as safe/redacted backend errors (and MUST NOT embed raw backend
    stdout/stderr) per the owner doc.

- **AD-C04 — Session-flow parity**
  - **Type**: integration
  - **Owner seam (pack)**: SEAM-1
  - **Consumers**: SEAM-3/4/5
  - **Definition**: `agent_api.exec.add_dirs.v1` is orthogonal to the session selector keys
    `agent_api.session.resume.v1` and `agent_api.session.fork.v1`; session flows are selected via
    `AgentWrapperRunRequest.extensions` (not separate request fields). Backends MUST preserve the
    same accepted effective add-dir set across new-session, resume, and fork decision-making, and
    MUST NOT silently ignore accepted inputs. Claude applies the accepted set on fork flows. The
    current Codex fork contract uses the pinned pre-handle backend rejection path from
    `docs/specs/codex-app-server-jsonrpc-contract.md`. (Normative: see
    `docs/specs/universal-agent-api/extensions-spec.md` `agent_api.exec.add_dirs.v1`.)

- **AD-C05 — Codex argv mapping**
  - **Type**: integration
  - **Owner seam**: SEAM-3
  - **Consumers**: SEAM-5
  - **Definition**: Codex receives one repeated `--add-dir <DIR>` pair per normalized unique
    directory, in order, on exec/resume flows. Any accepted `--model` pair stays earlier in argv per
    `docs/specs/codex-streaming-exec-contract.md`.

- **AD-C06 — Claude Code argv mapping**
  - **Type**: integration
  - **Owner seam**: SEAM-4
  - **Consumers**: SEAM-5
  - **Definition**: Claude Code receives one variadic `--add-dir <DIR...>` group containing the
    normalized unique directories, in order, after any accepted `--model` pair and before
    `--continue` / `--fork-session` / `--resume` and the final `--verbose` token.

- **AD-C07 — Absence semantics**
  - **Type**: policy
  - **Owner seam**: SEAM-1
  - **Consumers**: SEAM-2/3/4/5
  - **Definition**: when the key is absent, no backend synthesizes extra directories and no
    `--add-dir` argv is emitted.

## Dependency graph (text)

- `SEAM-1 blocks SEAM-2` because: the shared normalizer must implement the already-pinned v1
  schema, normalization, and safe-error rules.
- `SEAM-2 blocks SEAM-3` because: Codex support should consume the shared normalized directory set
  exported from `backend_harness::normalize::normalize_add_dirs_v1(...)` and the pinned Codex fork
  rejection contract instead of inventing backend-local path semantics.
- `SEAM-2 blocks SEAM-4` because: Claude Code support should consume the same shared normalized
  `Vec<PathBuf>` output from `backend_harness::normalize::normalize_add_dirs_v1(...)` instead of
  inventing backend-local path semantics.
- `SEAM-3 blocks SEAM-5` because: tests must pin Codex capability advertising, argv order, and
  session-flow behavior, including the fork rejection boundary.
- `SEAM-4 blocks SEAM-5` because: tests must pin Claude Code capability advertising, argv order,
  add-dir placement, and session-flow behavior.

## Critical path

`SEAM-1 (contract)` → `SEAM-2 (shared normalizer)` → `SEAM-3 (Codex mapping + fork rejection)` /
`SEAM-4 (Claude mapping + argv placement)` → `SEAM-5 (tests + capability artifact)`

## Integration points

- **Run extension gate**: `backend_harness::normalize_request()` must fail closed on unsupported
  keys before any add-dir value parsing happens.
- **Effective working directory handoff**: the shared normalizer and each backend’s spawn path
  must agree on the same working directory source. Effective working directory is defined in
  `docs/specs/universal-agent-api/contract.md` ("Working directory resolution (effective working directory)"),
  and backend adapters MUST pass that selected value into `normalize_add_dirs_v1(...)` unchanged.
- **Session selectors**: resume/fork parsing stays orthogonal, but accepted add-dir inputs must
  survive into those flows. The one pinned exception is Codex fork, which rejects before any
  app-server request using the backend-owned safe message.
- **Wrapper crate parity**: `codex::CodexClientBuilder` and `claude_code::ClaudePrintRequest`
  already expose add-dir surfaces; backend seams wire the normalized list into them.
- **Shared normalizer anchor**: SEAM-2 owns `backend_harness::normalize::normalize_add_dirs_v1(...)`
  and the `Vec<PathBuf>` output consumed by both backend policy layers.
- **Canonical backend docs**: SEAM-3 and SEAM-4 are not done until
  `docs/specs/codex-streaming-exec-contract.md`,
  `docs/specs/codex-app-server-jsonrpc-contract.md` and
  `docs/specs/claude-code-session-mapping-contract.md` reflect the exact mapping/rejection truth.
- **Capability publication**: SEAM-5 must regenerate
  `docs/specs/universal-agent-api/capability-matrix.md` with
  `cargo run -p xtask -- capability-matrix` once the backend capability ids change.

## Parallelization notes / conflict-safe workstreams

- **WS-CONTRACT**: SEAM-1 (`extensions-spec.md` confirmation + pack contract).
- **WS-NORMALIZE**: SEAM-2 (shared normalizer + reusable validation/resolution helpers).
- **WS-CODEX**: SEAM-3 (Codex capability + policy + exec/resume/fork mapping).
- **WS-CLAUDE**: SEAM-4 (Claude capability + policy + print/resume/fork mapping).
- **WS-TESTS**: SEAM-5 (shared normalizer tests plus backend capability/mapping/session tests and
  capability-matrix regeneration).
- **WS-INT (Integration)**: end-to-end validation and `make preflight` after the seams land.

## Pinned decisions / resolved threads

- **Trimming is leading/trailing Unicode whitespace** (per the owner doc).
- **Relative paths are allowed** and resolve against the effective working directory.
- **No containment rule** is imposed for v1.
- **Lexical normalization only**: no shell expansion, env expansion, canonicalization, or symlink
  resolution requirement.
- **Dedup is not an error**: duplicates collapse after normalization while preserving order.
