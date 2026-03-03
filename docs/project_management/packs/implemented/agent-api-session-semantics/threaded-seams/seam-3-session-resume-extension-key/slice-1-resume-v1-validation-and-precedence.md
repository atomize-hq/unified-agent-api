### S1 — Resume v1 validation + staged-rollout precedence pinning

- **User/system value**: A single, reusable parser/validator for `agent_api.session.resume.v1` that enforces the closed `.v1` schema and pins the R0 precedence rules so backend mappings stay deterministic and contradiction-free.
- **Scope (in/out)**:
  - In:
    - Implement a shared parser for `extensions["agent_api.session.resume.v1"]`:
      - type object, closed schema,
      - `selector` is `"last"` or `"id"`,
      - `"id"` requires `id` present and non-empty after trimming,
      - `"last"` forbids `id`.
    - Provide a small helper for the resume↔fork contradiction check that backends can call *after* the R0 key gate has passed.
    - Add tests that pin:
      - schema/type errors + closed-schema behavior,
      - staged-rollout precedence from `extensions-spec.md` R0:
        - unsupported key(s) yield `UnsupportedCapability` *before* contradiction rules,
        - contradiction (`resume` + `fork`) yields `InvalidRequest` only when both keys are supported.
  - Out:
    - Any backend-specific CLI mapping/spawn wiring (that’s `S2`/`S3`).
    - Implementing `agent_api.session.fork.v1` (SEAM-4).
- **Acceptance criteria**:
  - Parser rejects:
    - non-object values,
    - missing `selector`,
    - unknown object keys,
    - invalid selectors,
    - `selector == "id"` with missing/empty/whitespace-only `id`,
    - `selector == "last"` with any `id` present.
  - Parser returns a typed selector (`Last` | `Id(String)`), preserving the raw `id` string (no normalization beyond trim-to-empty validation).
  - Tests pin the precedence rules named in `seam-3-session-resume-extension-key.md` and `docs/specs/universal-agent-api/extensions-spec.md`.
- **Dependencies**:
  - Normative: `docs/specs/universal-agent-api/extensions-spec.md` (`resume.v1` schema + R0 precedence + contradiction rules).
- **Verification**:
  - `cargo test -p agent_api --all-features` (or `make test`).
- **Rollout/safety**:
  - Additive internal helpers; no backend capability advertisement occurs in this slice.

#### S1.T1 — Add shared `agent_api.session.resume.v1` selector parser (closed schema)

- **Outcome**: A tiny, reusable parser that enforces the `.v1` resume schema and returns a typed selector used by both built-in backends.
- **Inputs/outputs**:
  - Input: `serde_json::Value` for `extensions["agent_api.session.resume.v1"]`.
  - Output: a typed selector (e.g., `SessionSelectorV1::{Last, Id { id: String }}`) and `AgentWrapperError::InvalidRequest` on schema violations.
  - Files:
    - `crates/agent_api/src/backends/mod.rs` (register helper module)
    - `crates/agent_api/src/backends/session_selectors.rs` (new; shared parsing helpers)
- **Implementation notes**:
  - Enforce closed schema by rejecting any keys other than `selector` and `id`.
  - Validate `id` via `trim().is_empty()` but preserve the original `id` string when non-empty after trimming.
  - Error messages do not need to be user-pretty, but MUST be deterministic and MUST NOT embed large JSON values.
- **Acceptance criteria**:
  - Parser is used as a single-source-of-truth by both backends in `S2`/`S3` (no duplicated ad-hoc parsing in each backend file).
- **Test notes**: Covered by `S1.T2`.
- **Risk/rollback notes**: none (internal helper only).

Checklist:
- Implement:
  - Add a typed selector enum and a `parse_session_resume_v1(value: &Value) -> Result<..., AgentWrapperError>` helper.
- Test:
  - `cargo test -p agent_api --all-features`
- Validate:
  - Unknown keys fail with `InvalidRequest` (closed schema).
- Cleanup:
  - Keep helper `pub(crate)`; do not add public API surface.

#### S1.T2 — Pin unit tests for `resume.v1` schema validation (type + selector + id rules)

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
- **Acceptance criteria**:
  - Tests cover both structural errors and contradiction errors (`last` + `id` present).
- **Verification**:
  - `cargo test -p agent_api --all-features`

Checklist:
- Implement:
  - Add table-driven tests for valid and invalid cases.
- Test:
  - `cargo test -p agent_api --all-features`
- Validate:
  - Ensure error values do not contain the full raw JSON payload.

#### S1.T3 — Pin R0 precedence (staged rollout): `UnsupportedCapability` beats contradiction rules

- **Outcome**: Tests that pin the R0 precedence behavior for the specific `resume.v1`/`fork.v1` pair so staged rollout remains deterministic.
- **Inputs/outputs**:
  - Inputs: a dummy harness adapter with configurable `supported_extension_keys()` and a `validate_and_extract_policy` that (when reached) applies the resume↔fork contradiction check.
  - Outputs: tests that demonstrate:
    - only resume supported + both keys present → `UnsupportedCapability` (fork key),
    - both keys supported + both present → `InvalidRequest`.
  - Files (preferred to minimize merge conflicts):
    - `crates/agent_api/src/backend_harness/normalize/tests.rs`
- **Implementation notes**:
  - Keep this test harness-only: do not advertise fork support on real built-in backends before SEAM-4.
  - Ensure the “unsupported fork” case proves `validate_and_extract_policy` is not called (R0 gate wins).
- **Verification**:
  - `cargo test -p agent_api --all-features`

Checklist:
- Implement:
  - Add two focused tests using a tiny in-test adapter.
- Test:
  - `cargo test -p agent_api --all-features`
- Validate:
  - Error variants and precedence match `docs/specs/universal-agent-api/extensions-spec.md` R0.

