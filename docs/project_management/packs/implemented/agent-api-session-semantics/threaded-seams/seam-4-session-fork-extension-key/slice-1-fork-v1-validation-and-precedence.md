### S1 — Fork v1 validation + staged-rollout precedence pinning

- **User/system value**: A single, reusable parser/validator for `agent_api.session.fork.v1` that enforces the closed `.v1` schema and pins the R0 precedence rules so backend mappings remain deterministic and contradiction-free during staged rollout.
- **Scope (in/out)**:
  - In:
    - Implement a shared parser for `extensions["agent_api.session.fork.v1"]` (per `extensions-spec.md`):
      - type object, closed schema,
      - `selector` is `"last"` or `"id"`,
      - `"id"` requires `id` present and non-empty after trimming,
      - `"last"` forbids `id`.
    - Add tests pinning:
      - schema/type errors + closed-schema behavior,
      - staged-rollout precedence from `extensions-spec.md` R0:
        - unsupported key(s) yield `UnsupportedCapability` *before* contradiction rules,
        - contradiction (`resume` + `fork`) yields `InvalidRequest` only when **both** keys are supported.
  - Out:
    - Backend-specific CLI / app-server mapping (that’s `S2` / `S4`).
    - Implementing `resume.v1` behavior (SEAM-3), except for referencing the key in mutual-exclusivity tests.
- **Acceptance criteria**:
  - Parser rejects:
    - non-object values,
    - missing `selector`,
    - unknown object keys,
    - invalid selectors,
    - `selector == "id"` with missing/empty/whitespace-only `id`,
    - `selector == "last"` with any `id` present.
  - Parser returns a typed selector (`Last` | `Id(String)`), preserving the raw `id` string (no normalization beyond trim-to-empty validation).
  - Tests pin the precedence rules named in `docs/specs/unified-agent-api/extensions-spec.md` for the `resume.v1`/`fork.v1` mutual-exclusivity pair.
- **Dependencies**:
  - Normative: `docs/specs/unified-agent-api/extensions-spec.md` (`fork.v1` schema + R0 precedence + contradiction rules).
- **Verification**:
  - `cargo test -p agent_api --all-features` (or `make test`).
- **Rollout/safety**:
  - Additive internal helpers + tests; no backend capability advertisement occurs in this slice.

#### S1.T1 — Add shared `agent_api.session.fork.v1` selector parser (closed schema)

- **Outcome**: A tiny, reusable parser that enforces the `.v1` fork schema and returns a typed selector consumed by both built-in backends.
- **Inputs/outputs**:
  - Input: `serde_json::Value` for `extensions["agent_api.session.fork.v1"]`.
  - Output: a typed selector (reuse the same selector type used by `resume.v1`, since schemas match) and `AgentWrapperError::InvalidRequest` on schema violations.
  - Files:
    - `crates/agent_api/src/backends/mod.rs` (register shared selector module if not already present)
    - `crates/agent_api/src/backends/session_selectors.rs` (extend with `parse_session_fork_v1(...)`)
- **Implementation notes**:
  - Enforce closed schema by rejecting any keys other than `selector` and `id`.
  - Validate `id` via `trim().is_empty()` but preserve the original `id` string when non-empty after trimming.
  - Error messages do not need to be user-pretty, but MUST be deterministic and MUST NOT embed large JSON values.
- **Acceptance criteria**:
  - Both built-in backends use this helper (no duplicated ad-hoc fork JSON parsing in `claude_code.rs` / `codex.rs`).

Checklist:
- Implement:
  - Add `parse_session_fork_v1(value: &Value) -> Result<SessionSelectorV1, AgentWrapperError>` (or equivalent) next to the existing resume parser.
- Validate:
  - Unknown keys fail with `InvalidRequest` (closed schema).
- Cleanup:
  - Keep helper `pub(crate)`; do not add new public API surface.

#### S1.T2 — Pin unit tests for `fork.v1` schema validation (type + selector + id rules)

- **Outcome**: Regression tests that fail loudly on schema drift and enforce the `.v1` closed-schema contract.
- **Inputs/outputs**:
  - Inputs: representative `serde_json::Value` cases for valid/invalid objects.
  - Outputs: new unit tests co-located with the parser module.
  - Files:
    - `crates/agent_api/src/backends/session_selectors.rs`
- **Test cases (pinned)**:
  - Valid:
    - `{ "selector": "last" }`
    - `{ "selector": "id", "id": "abc" }`
  - Invalid:
    - non-object value (e.g., string, null, array),
    - missing `selector`,
    - `selector` not in `"last" | "id"`,
    - `selector == "id"` with `id: ""` / `"   "`,
    - `selector == "last"` with any `id`,
    - unknown extra key(s) (e.g., `{ "selector": "last", "extra": true }`).
- **Verification**:
  - `cargo test -p agent_api --all-features`

Checklist:
- Implement:
  - Add table-driven tests for valid and invalid cases.
- Validate:
  - Ensure error values do not contain the full raw JSON payload.

#### S1.T3 — Pin R0 precedence (staged rollout): fork-only support yields `UnsupportedCapability` (resume key)

- **Outcome**: Tests that pin the R0 precedence behavior for the specific `fork.v1`/`resume.v1` pair so staged rollout remains deterministic.
- **Inputs/outputs**:
  - Inputs: a dummy harness adapter with configurable `supported_extension_keys()` and a `validate_and_extract_policy` that (when reached) applies the resume↔fork contradiction check.
  - Outputs: tests that demonstrate:
    - only fork supported + both keys present → `UnsupportedCapability` (resume key),
    - both keys supported + both present → `InvalidRequest`.
  - Files (preferred to minimize merge conflicts):
    - `crates/agent_api/src/backend_harness/normalize/tests.rs`
- **Implementation notes**:
  - Extend (do not duplicate) the existing staged-rollout precedence tests introduced for SEAM-3 so both directions of the resume↔fork pair are pinned.
  - Ensure the “unsupported resume” case proves the contradiction validator is not called (R0 gate wins).
- **Verification**:
  - `cargo test -p agent_api --all-features`

Checklist:
- Implement:
  - Add two focused tests using a tiny in-test adapter (fork-only supported vs both supported).
- Validate:
  - Error variants and precedence match `docs/specs/unified-agent-api/extensions-spec.md` R0.

