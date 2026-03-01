### S2 — Codex handle facet emission (`agent_api.session.handle.v1`)

- **User/system value**: Orchestrators can discover and persist a stable Codex thread id (surfaced as a session handle facet) via a bounded metadata facet (events + completion) without parsing raw backend lines.
- **Scope (in/out)**:
  - In:
    - Capture Codex `thread_id` from typed parsed events via `SA-C01` (`ThreadEvent::thread_id()`).
    - Emit exactly one early `Status` event with `AgentWrapperEvent.data = handle_facet` once the id is known and valid.
    - Attach `handle_facet` to `AgentWrapperCompletion.data` when completion exists and id is valid.
    - Enforce facet-level bounds (trim-to-empty invalid; `<= 1024` bytes; omit not truncate; safe warning).
    - Advertise `agent_api.session.handle.v1` in Codex backend capabilities only once behavior + tests land.
  - Out:
    - Any spec changes (implementation MUST follow `docs/specs/universal-agent-api/event-envelope-schema-spec.md`).
    - Any raw stdout/stderr parsing to derive ids.
- **Acceptance criteria**:
  - When the Codex backend advertises `agent_api.session.handle.v1`:
    - Exactly one early `AgentWrapperEventKind::Status` event contains:
      - `data.schema == "agent_api.session.handle.v1"`
      - `data.session.id == <opaque thread id>`
    - If a completion is produced and the id is known + valid, then `completion.data` equals the same facet schema and id.
  - Bounds rules are pinned:
    - If `thread_id.trim().is_empty()`: treat as “not known” and do not emit the facet.
    - If `thread_id.as_bytes().len() > 1024`: do not emit the facet (MUST NOT truncate) and SHOULD emit a safe warning `Status`.
  - Id source is typed-only: `ThreadEvent::thread_id()`; no raw-line parsing.
- **Dependencies**:
  - `SA-C01 typed id accessor helpers` (SEAM-1) — Codex `thread_id()` accessor.
  - Normative: `docs/specs/universal-agent-api/event-envelope-schema-spec.md` (handle.v1 rules).
- **Verification**:
  - `cargo test -p agent_api` (plus targeted backend tests as added in `S2.T5`).
- **Rollout/safety**:
  - Capability-gated: do not advertise `agent_api.session.handle.v1` until the emission + completion attachment + bounds tests pass.

#### S2.T1 — Add run-local handle state and capture Codex `thread_id` (typed-only)

- **Outcome**: A run-scoped state cell that records the first valid Codex `thread_id` (within bounds) and tracks whether the handle facet has been emitted and whether an oversize warning has been emitted.
- **Inputs/outputs**:
  - Input: `codex::ThreadEvent` values as they stream through the harness adapter.
  - Output: run-local state in the Codex harness adapter (no global/static state).
  - Files:
    - `crates/agent_api/src/backends/codex.rs`
- **Implementation notes**:
  - Use `SA-C01`: call `event.thread_id()` on the typed event (never parse raw JSONL lines).
  - Validate the candidate id:
    - Reject if `trim().is_empty()`.
    - Reject if `as_bytes().len() > 1024` (mark “oversize seen” for one-time warning).
    - If accepted, store the id (owned `String`) for later event emission + completion attachment.
- **Acceptance criteria**:
  - State is per-run (adapter instance), threadsafe if needed.
  - Oversize ids are treated as “not known” for both event and completion emission points.

Checklist:
- Implement:
  - Add a small handle-facet state struct to the adapter.
  - Capture + validate candidate ids via `ThreadEvent::thread_id()`.
- Test:
  - `cargo test -p agent_api`
- Validate:
  - No raw-line parsing; extraction is strictly typed-event driven.

#### S2.T2 — Emit the handle facet exactly once as an early `Status` event (Codex)

- **Outcome**: The Codex backend emits exactly one early `Status` event whose `data` is the handle facet once the id is known and valid.
- **Inputs/outputs**:
  - Inputs:
    - Run-local stored `thread_id` from `S2.T1`.
    - Mapped wrapper events produced from Codex thread events.
  - Outputs:
    - One `AgentWrapperEventKind::Status` event with `data = handle_facet`.
- **Implementation notes**:
  - Preferred: attach the facet to an existing early `Status` event (`thread.started` is a natural attachment point).
  - Fallback (normative): if the id becomes known on an event that does not map to `Status`, emit a synthetic `Status` event immediately after capturing the id.
  - Ensure the facet is emitted **exactly once** per run:
    - After emission, mark `handle_facet_emitted = true` in run-local state.
    - Do not re-attach on later events even if they carry the same id.
- **Acceptance criteria**:
  - Exactly one `Status` event has `data.schema == "agent_api.session.handle.v1"`.
  - Emission is as-soon-as-known: no delay beyond the event that first reveals the valid id.

Checklist:
- Implement:
  - Post-process mapped events to inject `data` into the chosen `Status` event.
  - Add synthetic `Status` emission path if needed.
- Test:
  - `cargo test -p agent_api`
- Validate:
  - Exactly-once invariant is enforced by state, not by “best effort” heuristics.

#### S2.T3 — Attach handle facet to `AgentWrapperCompletion.data` (Codex)

- **Outcome**: When a completion is produced and the id is known + valid, `completion.data` contains the handle facet.
- **Inputs/outputs**:
  - Input: stored valid `thread_id` from run-local state.
  - Output: `AgentWrapperCompletion.data = Some(handle_facet)` when valid.
  - Files:
    - `crates/agent_api/src/backends/codex.rs`
- **Implementation notes**:
  - Attach the same facet schema + id used for event emission.
  - Do not attach if id is unknown/invalid/oversize.
- **Acceptance criteria**:
  - `completion.data.schema == "agent_api.session.handle.v1"` exactly when id was validly observed.

Checklist:
- Implement:
  - Update `map_completion` to read run-local state and set `data` accordingly.
- Test:
  - `cargo test -p agent_api`

#### S2.T4 — Advertise `agent_api.session.handle.v1` capability id (Codex)

- **Outcome**: The Codex backend advertises `agent_api.session.handle.v1` only once the behavior is implemented and tested.
- **Inputs/outputs**:
  - Output: Codex backend capabilities include `"agent_api.session.handle.v1"`.
  - Files:
    - `crates/agent_api/src/backends/codex.rs`
- **Acceptance criteria**:
  - Capability id is present in `AgentWrapperCapabilities.ids`.
  - Facet emission logic is consistent with capability advertisement (no “advertise without behavior”).

Checklist:
- Implement:
  - Add capability id to `capabilities()` set after the previous tasks land.
- Test:
  - `cargo test -p agent_api`

#### S2.T5 — Pin tests: placement, exactly-once, completion attachment, bounds (Codex)

- **Outcome**: Regression tests that pin SA-C02 behavior for Codex.
- **Inputs/outputs**:
  - Inputs: representative typed thread events that carry (a) a valid thread id, (b) whitespace-only id, (c) oversize id.
  - Outputs: tests that exercise multi-event sequences and completion attachment.
  - Files (choose one to reduce conflicts):
    - `crates/agent_api/src/backends/codex/tests.rs` (unit-style, direct adapter + mapping calls), and/or
    - `crates/agent_api/tests/**` (integration-style harness tests).
- **Test assertions (pinned)**:
  - Exactly one `Status` event includes the facet in `data`.
  - The facet appears “early” (no later than the first event after the id becomes known).
  - `completion.data` includes the facet when completion exists and id is valid.
  - Oversize id:
    - No facet emission (event or completion).
    - A safe warning `Status` is emitted (and does not include the facet).
  - Whitespace-only id:
    - No facet emission (event or completion).
- **Verification**:
  - `cargo test -p agent_api`

Checklist:
- Implement:
  - Add a multi-event test harness that calls adapter mapping sequentially.
  - Add an oversize thread id test case (e.g., 1025 bytes).
- Test:
  - `cargo test -p agent_api`
- Validate:
  - Tests do not rely on spawning the real `codex` binary; they should be deterministic and fast.

