# SEAM-1 — Typed session/thread id accessors (uaa-0017)

## Seam Brief (Restated)

- **Seam ID**: SEAM-1
- **Name**: Typed id accessors on backend event models (Codex + Claude Code)
- **Goal / value**: Provide tiny, reusable accessors so “what is the current session/thread id?” can be answered without duplicating per-variant match logic across `wrapper_events`, `agent_api`, and future consumers.
- **Type**: integration
- **Scope**
  - In:
    - Codex: `codex::ThreadEvent::thread_id() -> Option<&str>` returning the best-effort thread id when the variant contains one (pinned variant set).
    - Claude Code: `claude_code::ClaudeStreamJsonEvent::session_id() -> Option<&str>` returning the best-effort session id (including `Unknown { session_id: Some(...) }`) (pinned variant set).
    - Unit tests proving pinned variant coverage and that accessors return borrowed `&str` (no allocation required).
    - Adopt the accessors in `crates/wrapper_events` so id extraction match logic is not duplicated.
  - Out:
    - Any Unified Agent API spec changes (code-only ergonomics).
    - Any new public session handle surfaces in `crates/agent_api` (SEAM-2 owns that work).
- **Touch surface**:
  - `crates/codex/src/events.rs`
  - `crates/codex/tests/**`
  - `crates/claude_code/src/stream_json.rs`
  - `crates/claude_code/tests/**`
  - `crates/wrapper_events/src/codex_adapter.rs`
  - `crates/wrapper_events/src/claude_code_adapter.rs`
- **Verification**:
  - `thread_id()` returns `Some(...)` exactly for the pinned Codex variant set and `None` for `ThreadEvent::Error`.
  - `session_id()` returns `Some(...)` exactly for the pinned Claude variant set (including `Unknown { session_id: Some(...) }`) and `None` for `Unknown { session_id: None, .. }`.
  - `wrapper_events` adapters call the accessors (no duplicated per-variant match logic for ids).
- **Threading constraints**
  - Upstream blockers: none
  - Downstream blocked seams: SEAM-2 (session handle facet emission)
  - Contracts produced (owned):
    - `SA-C01 typed id accessor helpers`
  - Contracts consumed: none

## Slice index

- `S1` → `slice-1-codex-thread-id-accessor.md`: Publish Codex `ThreadEvent::thread_id()` and adopt it in `wrapper_events` (Codex path).
- `S2` → `slice-2-claude-session-id-accessor.md`: Publish Claude `ClaudeStreamJsonEvent::session_id()` and adopt it in `wrapper_events` (Claude path).

## Threading Alignment (mandatory)

- **Contracts produced (owned)**:
  - `SA-C01 typed id accessor helpers`:
    - Definition (per `threading.md`): `codex::ThreadEvent::thread_id() -> Option<&str>` and `claude_code::ClaudeStreamJsonEvent::session_id() -> Option<&str>`.
    - Where it lives:
      - Codex: `crates/codex/src/events.rs`
      - Claude Code: `crates/claude_code/src/stream_json.rs`
    - Produced by:
      - `S1` publishes the Codex half of SA-C01.
      - `S2` publishes the Claude half of SA-C01 (SA-C01 is fully complete after `S2`).
- **Contracts consumed**:
  - None (this seam only reads already-typed event fields; it MUST NOT parse raw stdout/stderr lines).
- **Dependency edges honored**:
  - `SEAM-1 blocks SEAM-2`: `S1`/`S2` land before SEAM-2 so SEAM-2 can source ids via SA-C01 instead of duplicating match logic.
- **Parallelization notes**:
  - What can proceed now:
    - `S1` and `S2` are independent beyond shared reviewer context; each is a small, single-owner PR sequence.
  - What must wait:
    - SEAM-2 backend handle facet emission should wait to merge until SA-C01 exists for the backend(s) it is touching (Codex → after `S1`, Claude → after `S2`).

