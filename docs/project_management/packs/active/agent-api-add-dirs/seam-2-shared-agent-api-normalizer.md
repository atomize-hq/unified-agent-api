# SEAM-2 — Shared `agent_api` add-dir normalizer

- **Name**: shared add-dir request parsing / validation / resolution helper
- **Type**: integration
- **Goal / user value**: compute the effective add-dir set once so Codex and Claude Code consume
  the same validated directory list and error posture.

## Scope

- In:
  - Introduce shared code that:
    - parses `agent_api.exec.add_dirs.v1`,
    - enforces the closed schema and bounds,
    - resolves relative paths from the effective working directory (per
      `docs/specs/universal-agent-api/contract.md` "Working directory resolution (effective working directory)"),
    - lexically normalizes,
    - validates `exists && is_dir`,
    - deduplicates while preserving order,
    - returns safe/testable errors without path leaks.
  - Define one shared normalizer entrypoint in
    `crates/agent_api/src/backend_harness/normalize.rs`:
    `normalize_add_dirs_v1(...) -> Result<Vec<PathBuf>, AgentWrapperError>`.
  - Export exactly `Vec<PathBuf>` as the normalized unique directory list consumed by backend
    policy extraction and spawn layers.
- Out:
  - Backend capability advertising.
  - Backend-specific CLI argv emission.

## Primary interfaces (contracts)

- **Normalizer input contract**
  - **Inputs**:
    - raw extension value
    - effective working directory (per `docs/specs/universal-agent-api/contract.md` "Working directory resolution (effective working directory)")
  - **Outputs**:
    - normalized unique directory list or `InvalidRequest`

- **Backend-consumption contract**
  - **Inputs**:
    - normalized directory list
  - **Outputs**:
    - backend policy/spawn layers receive `Vec<PathBuf>` and map it without re-validating
      schema/path semantics

- **Shared helper entrypoint**
  - **Inputs**:
    - `extensions["agent_api.exec.add_dirs.v1"]`
    - effective working directory selected for the run (selected by the backend adapter layer per `contract.md`)
  - **Outputs**:
    - `Result<Vec<PathBuf>, AgentWrapperError>` from
      `backend_harness::normalize::normalize_add_dirs_v1(...)`

## Key invariants / rules

- The shared helper is the only place that decides trimming, path resolution, normalization, and
  dedup behavior.
- Backend-specific policy structs may carry the resulting `Vec<PathBuf>`, but they MUST treat it as
  an already-normalized value and MUST NOT duplicate schema or filesystem validation.
- Errors identify the failing field or index using the exact templates
  `invalid agent_api.exec.add_dirs.v1`, `invalid agent_api.exec.add_dirs.v1.dirs`, or
  `invalid agent_api.exec.add_dirs.v1.dirs[<i>]`, and never the raw path text.
- Backends MUST NOT invent any other `InvalidRequest` message shape for this key.
- The helper must not create a new working-directory precedence ladder that diverges from actual
  backend execution behavior.

## Dependencies

- Blocks: SEAM-3/4/5
- Blocked by: SEAM-1

## Touch surface

- `crates/agent_api/src/backend_harness/normalize.rs`
- `crates/agent_api/src/backend_harness/contract.rs`
- `crates/agent_api/src/backends/codex/policy.rs`
- `crates/agent_api/src/backends/claude_code/backend.rs`

## Verification

- Unit tests for:
  - non-object value rejection,
  - unknown key rejection,
  - missing `dirs`,
  - non-array `dirs`,
  - length bounds,
  - non-string entries,
  - trimmed empty entries,
  - byte-length bounds,
  - relative resolution,
  - lexical normalization,
  - dedup order preservation,
  - missing/non-directory path rejection,
  - exact safe InvalidRequest template selection with no raw path leakage in errors.

## Risks / unknowns

- **Risk**: resolving add-dirs too early could use the wrong working directory source.
- **De-risk plan**: thread the effective working directory explicitly into the helper from the
  backend adapter layer that already owns run defaults.

## Rollout / safety

- Land the shared helper in `backend_harness/normalize.rs` first so backend seams consume one pinned
  `Vec<PathBuf>` contract.
