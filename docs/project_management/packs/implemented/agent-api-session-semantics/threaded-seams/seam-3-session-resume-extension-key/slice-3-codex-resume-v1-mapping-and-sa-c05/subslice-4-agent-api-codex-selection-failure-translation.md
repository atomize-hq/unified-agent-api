### S3d — `agent_api` Codex backend: resume selection-failure translation (pinned messages + terminal `Error`)

- **User/system value**: Resume selection failures surface as pinned safe messages (`"no session found"` / `"session not found"`) and are observable both via stream events and the completion result.
- **Scope (in/out)**:
  - In:
    - Implement safe “not found” classification for Codex resume outcomes.
    - Map selection failures to pinned `AgentWrapperError::Backend` messages per `extensions-spec.md`.
    - Ensure exactly one terminal `AgentWrapperEventKind::Error` event is emitted when a stream exists.
  - Out:
    - Fake-binary integration tests and capability advertisement (see `S3e`).
- **Acceptance criteria**:
  - Messages are exactly:
    - `"no session found"` for selector `"last"`,
    - `"session not found"` for selector `"id"`.
  - No raw Codex stdout/stderr/JSONL content is embedded in surfaced messages or event payloads.
  - When a stream exists, exactly one terminal `Error` event is emitted and the completion error matches its message.
- **Dependencies**:
  - `S3c` (resume selector is plumbed into Codex spawn wiring).
  - Normative: `docs/specs/universal-agent-api/extensions-spec.md` (pinned messages + terminal `Error` event rule).
- **Verification**:
  - `cargo test -p agent_api --features codex`
- **Rollout/safety**:
  - Capability advertisement remains gated until `S3e` tests pass.

#### S3.T4 — Codex selection failure translation (pinned messages + terminal `Error` event rule)

- **Outcome**: Selection failures for resume surface as pinned safe `Backend` errors and emit exactly one terminal `Error` event when a stream exists.
- **Inputs/outputs**:
  - Inputs: resume selector (`last` vs `id`) and the Codex wrapper’s typed outcomes.
  - Outputs:
    - Completion resolves to `Err(AgentWrapperError::Backend { message: <pinned> })` for selection failures.
    - Event stream emits exactly one terminal `Error` event with `message == <pinned>` before closing.
  - Files:
    - `crates/agent_api/src/backends/codex.rs`
- **Implementation notes**:
  - Do not embed raw Codex stderr/stdout/JSONL lines in the surfaced messages.
  - Prefer classifying “not found” via a stable typed signal (e.g., a known error event type or wrapper error kind), falling back to a safe generic backend error message for other failures.
- **Acceptance criteria**:
  - Messages are exactly:
    - `"no session found"` for selector `"last"`,
    - `"session not found"` for selector `"id"`.

Checklist:
- Implement:
  - Add a safe “not found” classification path and map it to pinned messages.
  - Ensure the terminal error event is emitted exactly once in the stream.
- Test:
  - `cargo test -p agent_api --features codex`

