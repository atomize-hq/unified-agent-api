### S1 — Codex `ThreadEvent::thread_id()` (and wrapper_events adoption)

- **User/system value**: A single, typed, best-effort way to read the Codex thread id from parsed Codex events; downstream crates stop duplicating per-variant match logic.
- **Scope (in/out)**:
  - In:
    - Implement `codex::ThreadEvent::thread_id() -> Option<&str>` with the pinned variant coverage and “no normalization” rule.
    - Add unit tests proving variant coverage and borrowed return.
    - Adopt the accessor in `crates/wrapper_events/src/codex_adapter.rs`.
  - Out:
    - Any changes to Codex JSONL parsing contracts (this reads already-typed fields only).
    - Any `agent_api` handle facet emission logic (SEAM-2).
- **Acceptance criteria**:
  - `ThreadEvent::thread_id()` exists and returns:
    - `Some(thread_id.as_str())` for: `ThreadStarted`, `TurnStarted`, `TurnCompleted`, `TurnFailed`, `ItemStarted`, `ItemDelta`, `ItemCompleted`, `ItemFailed`.
    - `None` for: `Error`.
  - The accessor returns the stored field as-is (no trimming/normalization).
  - Tests pin the variant set and confirm the returned `&str` is borrowed (no allocation required).
  - `crates/wrapper_events` Codex adapter calls `.thread_id()` rather than duplicating match logic.
- **Dependencies**: none (this slice publishes the Codex half of `SA-C01`).
- **Verification**:
  - `cargo test -p codex`
  - `cargo test -p wrapper_events`
- **Rollout/safety**:
  - Additive API (`Option<&str>`); absence remains valid and consumers must not treat it as always-present.

#### S1.T1 — Add `ThreadEvent::thread_id()` accessor

- **Outcome**: A tiny, well-tested helper on `codex::ThreadEvent` returning `Option<&str>` with pinned variant coverage.
- **Inputs/outputs**:
  - Input: existing `codex::ThreadEvent` variants (already-typed).
  - Output: `pub fn thread_id(&self) -> Option<&str>` on `ThreadEvent`.
  - Files:
    - `crates/codex/src/events.rs`
- **Implementation notes**:
  - Implement as a `match self { ... }` returning `Some(thread_id.as_str())` where the variant carries `thread_id`.
  - MUST NOT parse raw stdout/stderr lines; only read typed fields.
- **Acceptance criteria**:
  - Signature matches `threading.md` (`Option<&str>`).
  - Variant coverage matches the pinned set in `seam-1-typed-session-thread-id-accessors.md`.
- **Test notes**: covered by `S1.T2`.
- **Risk/rollback notes**: none (additive; compile-time API only).

Checklist:
- Implement:
  - Add method on `ThreadEvent`.
- Test:
  - Build + run `cargo test -p codex`.
- Validate:
  - Confirm match arms cover exactly the pinned variants (no “default” branch).
- Cleanup:
  - Keep any helper private; do not add new public types.

#### S1.T2 — Codex accessor unit tests (pinned coverage + borrowed return)

- **Outcome**: Deterministic tests that pin the variant set and validate `thread_id()` returns borrowed `&str`.
- **Inputs/outputs**:
  - Inputs: representative `ThreadEvent` values for each pinned variant.
  - Outputs: new tests in `crates/codex/tests/**` (or `#[cfg(test)]` module in `events.rs` if that’s the crate convention).
- **Implementation notes**:
  - For each id-bearing variant, assert `thread_id() == Some(<expected>)`.
  - For `ThreadEvent::Error`, assert `thread_id().is_none()`.
  - If fields are accessible, add a “borrowed return” assertion by comparing `as_ptr()`/`len()` against the underlying `String` (or by proving the returned `&str` points into the event-owned buffer).
- **Acceptance criteria**:
  - Tests enumerate the full pinned variant set (explicitly).
  - Tests fail if a new variant is introduced and not consciously added to the accessor.
- **Test notes**:
  - Run `cargo test -p codex` (and keep tests fast; no integration/system dependencies).
- **Risk/rollback notes**: none.

Checklist:
- Implement:
  - Add one test per pinned variant category (id-bearing + error).
- Test:
  - `cargo test -p codex`
- Validate:
  - Ensure no allocations are introduced (API returns `&str`).
- Cleanup:
  - Prefer table-driven tests to keep LOC small.

#### S1.T3 — Adopt `ThreadEvent::thread_id()` in `wrapper_events` (Codex adapter)

- **Outcome**: `wrapper_events` reads the Codex thread id via the typed accessor and no longer duplicates per-variant match logic.
- **Inputs/outputs**:
  - Inputs: `codex::ThreadEvent::thread_id()` (from `S1.T1`).
  - Outputs: simplified Codex adapter id extraction logic.
  - Files:
    - `crates/wrapper_events/src/codex_adapter.rs`
- **Implementation notes**:
  - Replace duplicated `match ThreadEvent::{...}` branches with a call to `event.thread_id()`.
  - Preserve existing behavior for variants that never carried an id.
- **Acceptance criteria**:
  - No per-variant id match logic remains for Codex thread ids in the adapter.
  - Existing wrapper_events tests continue to pass; add/adjust tests only if needed to preserve determinism.
- **Test notes**:
  - Run `cargo test -p wrapper_events` (and the workspace preflight if already part of local workflow).
- **Risk/rollback notes**:
  - Low risk; rollback is reverting to the previous match logic (but prefer keeping single-source-of-truth).

Checklist:
- Implement:
  - Update adapter to call `.thread_id()`.
- Test:
  - `cargo test -p wrapper_events`
- Validate:
  - `rg \"ThreadEvent::\" crates/wrapper_events/src/codex_adapter.rs` no longer shows id-extraction match logic.
- Cleanup:
  - Keep any remaining matches focused on non-id mapping only.

