### S2 — Claude `ClaudeStreamJsonEvent::session_id()` (and wrapper_events adoption)

- **User/system value**: A single, typed, best-effort way to read the Claude session id from parsed Claude stream-json events; downstream crates stop duplicating per-variant match logic and correctly surface ids even for `Unknown { session_id: Some(...) }`.
- **Scope (in/out)**:
  - In:
    - Implement `claude_code::ClaudeStreamJsonEvent::session_id() -> Option<&str>` with the pinned variant coverage and “no normalization” rule.
    - Add unit tests proving variant coverage and borrowed return.
    - Adopt the accessor in `crates/wrapper_events/src/claude_code_adapter.rs`.
  - Out:
    - Any changes to Claude stream-json parsing contracts (this reads already-typed fields only).
    - Any `agent_api` handle facet emission logic (SEAM-2).
- **Acceptance criteria**:
  - `ClaudeStreamJsonEvent::session_id()` exists and returns:
    - `Some(session_id.as_str())` for: `SystemInit`, `SystemOther`, `UserMessage`, `AssistantMessage`, `ResultSuccess`, `ResultError`, `StreamEvent`.
    - `Unknown { session_id: Some(id), .. }` → `Some(id.as_str())`
    - `Unknown { session_id: None, .. }` → `None`
  - The accessor returns the stored field as-is (no trimming/normalization).
  - Tests pin the variant set and confirm the returned `&str` is borrowed (no allocation required).
  - `crates/wrapper_events` Claude adapter calls `.session_id()` rather than duplicating match logic.
- **Dependencies**:
  - `S1` is independent; this slice publishes the Claude half of `SA-C01` and completes that contract.
- **Verification**:
  - `cargo test -p claude_code`
  - `cargo test -p wrapper_events`
- **Rollout/safety**:
  - Additive API (`Option<&str>`); absence remains valid and consumers must not treat it as always-present.

#### S2.T1 — Add `ClaudeStreamJsonEvent::session_id()` accessor

- **Outcome**: A tiny, well-tested helper on `claude_code::ClaudeStreamJsonEvent` returning `Option<&str>` with pinned variant coverage, including the `Unknown` escape hatch.
- **Inputs/outputs**:
  - Input: existing `claude_code::ClaudeStreamJsonEvent` variants (already-typed).
  - Output: `pub fn session_id(&self) -> Option<&str>` on `ClaudeStreamJsonEvent`.
  - Files:
    - `crates/claude_code/src/stream_json.rs`
- **Implementation notes**:
  - Implement as a `match self { ... }` returning `Some(session_id.as_str())` where the variant carries `session_id`.
  - For `Unknown`, branch on `session_id: Option<String>` (or equivalent) and return `session_id.as_deref()`.
  - MUST NOT parse raw stdout/stderr lines; only read typed fields.
- **Acceptance criteria**:
  - Signature matches `threading.md` (`Option<&str>`).
  - Variant coverage matches the pinned set in `seam-1-typed-session-thread-id-accessors.md`.
- **Test notes**: covered by `S2.T2`.
- **Risk/rollback notes**: none (additive; compile-time API only).

Checklist:
- Implement:
  - Add method on `ClaudeStreamJsonEvent`.
- Test:
  - Build + run `cargo test -p claude_code`.
- Validate:
  - Confirm match arms cover exactly the pinned variants (no “default” branch).
- Cleanup:
  - Keep any helper private; do not add new public types.

#### S2.T2 — Claude accessor unit tests (pinned coverage + borrowed return)

- **Outcome**: Deterministic tests that pin the variant set and validate `session_id()` returns borrowed `&str`.
- **Inputs/outputs**:
  - Inputs: representative `ClaudeStreamJsonEvent` values for each pinned variant, plus `Unknown` with both `session_id: Some(...)` and `None`.
  - Outputs: new tests in `crates/claude_code/tests/**` (or `#[cfg(test)]` module in `stream_json.rs` if that’s the crate convention).
- **Implementation notes**:
  - For each id-bearing variant, assert `session_id() == Some(<expected>)`.
  - For `Unknown { session_id: None, .. }`, assert `session_id().is_none()`.
  - If fields are accessible, add a “borrowed return” assertion by comparing `as_ptr()`/`len()` against the underlying `String` (or by proving the returned `&str` points into the event-owned buffer).
- **Acceptance criteria**:
  - Tests enumerate the full pinned variant set (explicitly), including the `Unknown` cases.
  - Tests fail if a new variant is introduced and not consciously added to the accessor.
- **Test notes**:
  - Run `cargo test -p claude_code` (and keep tests fast; no integration/system dependencies).
- **Risk/rollback notes**: none.

Checklist:
- Implement:
  - Add one test per pinned variant category (id-bearing + unknown-none).
- Test:
  - `cargo test -p claude_code`
- Validate:
  - Ensure no allocations are introduced (API returns `&str`).
- Cleanup:
  - Prefer table-driven tests to keep LOC small.

#### S2.T3 — Adopt `ClaudeStreamJsonEvent::session_id()` in `wrapper_events` (Claude adapter)

- **Outcome**: `wrapper_events` reads the Claude session id via the typed accessor and no longer duplicates per-variant match logic (including correct handling of `Unknown` events).
- **Inputs/outputs**:
  - Inputs: `claude_code::ClaudeStreamJsonEvent::session_id()` (from `S2.T1`).
  - Outputs: simplified Claude adapter id extraction logic.
  - Files:
    - `crates/wrapper_events/src/claude_code_adapter.rs`
- **Implementation notes**:
  - Replace duplicated `match ClaudeStreamJsonEvent::{...}` branches with a call to `event.session_id()`.
  - Preserve existing behavior for variants that never carried an id.
- **Acceptance criteria**:
  - No per-variant id match logic remains for Claude session ids in the adapter.
  - Existing wrapper_events tests continue to pass; add/adjust tests only if needed to preserve determinism.
- **Test notes**:
  - Run `cargo test -p wrapper_events`.
- **Risk/rollback notes**:
  - Low risk; rollback is reverting to the previous match logic (but prefer keeping single-source-of-truth).

Checklist:
- Implement:
  - Update adapter to call `.session_id()`.
- Test:
  - `cargo test -p wrapper_events`
- Validate:
  - `rg \"ClaudeStreamJsonEvent::\" crates/wrapper_events/src/claude_code_adapter.rs` no longer shows id-extraction match logic.
- Cleanup:
  - Keep any remaining matches focused on non-id mapping only.

