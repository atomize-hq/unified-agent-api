# SEAM-2 — Session handle facet emission (`agent_api.session.handle.v1`) (uaa-0015)

## Seam Brief (Restated)

- **Seam ID**: SEAM-2
- **Name**: Session handle facet emission (events + completion)
- **Goal / value**: Make “resume-by-id” ergonomic by surfacing the backend-defined session/thread id in a stable, bounded metadata facet, without changing the public `agent_api` Rust type shapes.
- **Type**: integration
- **Scope**
  - In:
    - Implement capability id `agent_api.session.handle.v1` (SA-C02) in built-in `agent_api` backends:
      - Capture the id from typed parsed backend events (Codex `thread_id`, Claude `session_id`) via `SA-C01` (SEAM-1).
      - Emit **exactly one** early `AgentWrapperEventKind::Status` event carrying:
        - `data = { "schema": "agent_api.session.handle.v1", "session": { "id": "<opaque>" } }`
      - Attach the same facet to `AgentWrapperCompletion.data` whenever a completion is produced and the id is known and within bounds.
    - Enforce facet-level id bounds (per `docs/specs/unified-agent-api/event-envelope-schema-spec.md`):
      - `session.id` MUST be non-empty after trimming (whitespace-only ids are treated as “not known”).
      - `len(session.id) <= 1024` bytes (UTF-8) or else omit (MUST NOT truncate) and SHOULD emit a safe warning `Status`.
    - Advertise `agent_api.session.handle.v1` in `AgentWrapperCapabilities.ids` only once the above is implemented and tested per backend.
    - Tests pinning: “exactly once” placement, completion attachment, and bounds behavior for both backends.
  - Out:
    - Any spec changes (rules are authoritative in `docs/specs/unified-agent-api/event-envelope-schema-spec.md`).
    - Any new required fields on `AgentWrapperEvent` or `AgentWrapperCompletion`.
- **Touch surface**:
  - `crates/agent_api/src/backends/codex.rs`
  - `crates/agent_api/src/backends/codex/mapping.rs`
  - `crates/agent_api/src/backends/claude_code.rs`
  - `crates/agent_api/src/bounds.rs` (optional helper extraction only)
  - `crates/agent_api/tests/**` and/or `crates/agent_api/src/backends/**/tests.rs` (new tests)
- **Verification**:
  - For each built-in backend that advertises `agent_api.session.handle.v1`:
    - Exactly one early `Status` event includes the facet in `AgentWrapperEvent.data`.
    - `AgentWrapperCompletion.data` includes the facet when a completion is produced and the id was observed and valid.
    - Oversize ids are omitted (not truncated) and produce only safe/bounded warnings.
    - Whitespace-only ids are treated as invalid (trim-to-empty) and do not emit the facet.
- **Threading constraints**
  - Upstream blockers: SEAM-1 / `SA-C01 typed id accessor helpers`
  - Downstream blocked seams: none (but `SA-C02` is an input to SEAM-3 resume-by-id UX)
  - Contracts produced (owned):
    - `SA-C02 session handle facet (handle.v1)`
  - Contracts consumed:
    - `SA-C01 typed id accessor helpers`

## Slice index

- `S1` → `slice-1-claude-handle-facet-emission.md`: Claude backend: capture + emit handle facet (events + completion), advertise capability, pin tests.
- `S2` → `slice-2-codex-handle-facet-emission.md`: Codex backend: capture + emit handle facet (events + completion), advertise capability, pin tests.

## Threading Alignment (mandatory)

- **Contracts produced (owned)**:
  - `SA-C02 session handle facet (handle.v1)`:
    - Definition (per `threading.md` + normative spec): when a backend advertises `agent_api.session.handle.v1`, it emits exactly one early `Status` event whose `data` is the handle facet and attaches the same facet to `completion.data` when known and in-bounds.
    - Where it lives (implementation):
      - Claude: `crates/agent_api/src/backends/claude_code.rs`
      - Codex: `crates/agent_api/src/backends/codex.rs` (+ `codex/mapping.rs` mapping touch surface)
    - Produced by:
      - `S1` implements + advertises SA-C02 for the Claude backend.
      - `S2` implements + advertises SA-C02 for the Codex backend.
- **Contracts consumed**:
  - `SA-C01 typed id accessor helpers` (SEAM-1):
    - Claude id source MUST be `claude_code::ClaudeStreamJsonEvent::session_id() -> Option<&str>`.
    - Codex id source MUST be `codex::ThreadEvent::thread_id() -> Option<&str>`.
  - `docs/specs/unified-agent-api/event-envelope-schema-spec.md`:
    - Handle facet schema, “exactly once” event rule, completion attachment, and facet-level bounds are normative.
- **Dependency edges honored**:
  - `SEAM-1 blocks SEAM-2`: both `S1` and `S2` depend on `SA-C01` being present for their backend before implementing extraction.
  - `SEAM-2 + SEAM-3 jointly unblock resume-by-id UX`: `SA-C02` is a prerequisite for an orchestrator to discover and persist the id needed for `agent_api.session.resume.v1` selector `"id"`.
- **Parallelization notes**:
  - Conflict-safe split aligns to the threading workstreams:
    - `S1` (Claude) aligns with WS-B.
    - `S2` (Codex) aligns with WS-C.
  - Shared touch surfaces:
    - Both slices may touch `crates/agent_api/tests/**`; prefer backend-scoped tests (or separate files) to reduce merge conflicts.
  - Integration follow-up (WS-INT, out-of-scope for this seam decomposition):
    - After new capability ids ship, regenerate and commit `docs/specs/unified-agent-api/capability-matrix.md` via `cargo run -p xtask -- capability-matrix`.

