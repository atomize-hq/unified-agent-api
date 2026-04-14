### S2 — Claude Code `agent_api.session.resume.v1` mapping + selection-failure translation

- **User/system value**: Orchestrators can resume Claude sessions using a backend-neutral extension key (`resume.v1`) with deterministic CLI mapping and safe, pinned selection-failure behavior.
- **Scope (in/out)**:
  - In:
    - Add `agent_api.session.resume.v1` to Claude backend:
      - `supported_extension_keys()` allowlist (fail-closed gate),
      - `AgentWrapperCapabilities.ids` (runtime discovery) only once behavior + tests land.
    - Parse and validate `extensions["agent_api.session.resume.v1"]` via the shared helper from `S1` (closed schema).
    - Map to pinned Claude CLI subsequences (per `docs/specs/claude-code-session-mapping-contract.md`):
      - selector `"last"` → `--continue`
      - selector `"id"` → `--resume <ID>`
      - keep headless print mode: `--print --output-format stream-json ... --verbose PROMPT`
      - when `agent_api.exec.non_interactive == true` (default), include `--permission-mode bypassPermissions` (no fallback).
    - Implement selection-failure translation per `extensions-spec.md`:
      - selector `"last"` empty scope → `AgentWrapperError::Backend("no session found")`
      - selector `"id"` unknown/unresumable → `AgentWrapperError::Backend("session not found")`
      - if failure occurs after returning a run handle, emit exactly one terminal `Error` event with the same pinned message.
  - Out:
    - Fork mapping (`agent_api.session.fork.v1`) (SEAM-4).
    - Handle facet emission (`agent_api.session.handle.v1`) (SEAM-2).
- **Acceptance criteria**:
  - When Claude advertises `agent_api.session.resume.v1`:
    - CLI argv contains the ordered subsequence pinned by `claude-code-session-mapping-contract.md` for the chosen selector.
    - Prompt is a single positional argv token and is the final argv token.
    - Invalid schemas fail pre-spawn with `AgentWrapperError::InvalidRequest`.
    - Selection failures surface as `AgentWrapperError::Backend` with pinned messages and satisfy the terminal `Error` event rule.
- **Dependencies**:
  - `S1` shared resume selector parser.
  - Normative: `docs/specs/unified-agent-api/extensions-spec.md` (schema + selection failure + contradiction rules).
  - Normative: `docs/specs/claude-code-session-mapping-contract.md` (argv subsequences + safe error translation).
- **Verification**:
  - `cargo test -p agent_api --features claude_code`
- **Rollout/safety**:
  - Capability-gated: do not advertise `agent_api.session.resume.v1` until mapping + selection-failure behavior + tests pass.

#### S2.T1 — Claude backend: parse/validate `resume.v1` and plumb into spawn policy

- **Outcome**: Claude policy extraction supports an optional resume selector and enforces `.v1` closed-schema validation pre-spawn.
- **Inputs/outputs**:
  - Inputs: `AgentWrapperRunRequest.extensions`.
  - Outputs:
    - Extended Claude policy including `resume: Option<...>` (typed selector),
    - `InvalidRequest` for schema violations,
    - resume↔fork contradiction check wired for when both keys are supported (the check itself is pinned by `extensions-spec.md`).
  - Files:
    - `crates/agent_api/src/backends/claude_code.rs`
- **Implementation notes**:
  - Use the shared parser from `S1` (no duplicated JSON parsing).
  - Absence of `resume.v1` means “new session per backend defaults” (no `--continue`/`--resume` flags).
- **Acceptance criteria**:
  - Validation runs pre-spawn (inside `validate_and_extract_policy`).
  - The policy is purely derived from supported keys (R0 key gate is harness-owned).

Checklist:
- Implement:
  - Add `EXT_SESSION_RESUME_V1` constant and parse it into policy when present.
  - Thread the parsed selector into spawn logic (used by `S2.T2`).
- Test:
  - `cargo test -p agent_api --features claude_code`
- Validate:
  - Schema errors return `InvalidRequest` and do not attempt to spawn `claude`.

#### S2.T2 — Claude backend: implement pinned CLI mapping for selector `"last"` / `"id"`

- **Outcome**: Claude spawn wiring maps the typed selector into the pinned `claude --print --output-format stream-json ...` argv subsequences.
- **Inputs/outputs**:
  - Inputs: typed selector from `S2.T1`, `AgentWrapperRunRequest.prompt`, `agent_api.exec.non_interactive`.
  - Outputs: a `claude_code::ClaudePrintRequest` configured to produce a stream-json event stream for resume semantics.
  - Files:
    - `crates/agent_api/src/backends/claude_code.rs`
- **Implementation notes**:
  - selector `"last"` → set `continue_session = true` on `ClaudePrintRequest`.
  - selector `"id"` → set `resume_value = <ID>` on `ClaudePrintRequest` (not `--continue`).
  - Keep `--verbose` behavior as enforced by `crates/claude_code` for stream-json output.
  - Preserve existing non-interactive permission-mode mapping (`bypassPermissions`).
- **Acceptance criteria**:
  - For each selector, the spawned argv contains the ordered subsequence pinned by `docs/specs/claude-code-session-mapping-contract.md`.

Checklist:
- Implement:
  - Add the selector→request mapping in `spawn`.
- Test:
  - `cargo test -p agent_api --features claude_code`
- Validate:
  - Prompt remains the final argv token (no extra trailing args after it).

#### S2.T3 — Claude selection failure translation: pinned `Backend` error + terminal `Error` event

- **Outcome**: Selection failures map to pinned safe messages and are observable both via the event stream and the completion future.
- **Inputs/outputs**:
  - Inputs: typed selector, typed Claude stream-json events + completion status.
  - Outputs:
    - `AgentWrapperError::Backend { message: <pinned> }` on completion for selection failure,
    - Exactly one terminal `AgentWrapperEventKind::Error` event with `message == <pinned>` before stream finality.
  - Files:
    - `crates/agent_api/src/backends/claude_code.rs`
- **Implementation notes**:
  - Prefer translating selection failure from structured, typed information (e.g., a recognizable `ResultError` payload) and/or a stable “not found” exit condition, but MUST NOT embed raw backend output in the surfaced message.
  - Ensure “terminal Error event” is emitted even if the backend produces no stream-json events on failure (use a tail/synthetic error event pattern if needed, similar to how the Codex backend emits a tail non-zero error event today).
- **Acceptance criteria**:
  - Messages match `extensions-spec.md` exactly for the two selection-failure cases.
  - No raw Claude stdout/stderr content is embedded in `AgentWrapperEvent.data` or error messages.

Checklist:
- Implement:
  - Add run-local state needed to decide whether failure is selection-failure vs generic backend failure.
  - Emit exactly one terminal `Error` event for the selection-failure case.
- Test:
  - `cargo test -p agent_api --features claude_code`
- Validate:
  - Error event and completion error agree on the pinned message.

#### S2.T4 — Pin integration tests with `fake_claude_stream_json_agent_api` (argv + failure behavior)

- **Outcome**: Deterministic tests that pin Claude resume argv mapping and selection-failure semantics without relying on the real `claude` binary.
- **Inputs/outputs**:
  - Inputs:
    - `fake_claude_stream_json_agent_api` scenarios gated by env vars,
    - `AgentWrapperRunRequest.extensions["agent_api.session.resume.v1"]`.
  - Outputs:
    - New tests for selector `"last"`/`"id"` argv mapping (ordered subsequence) and prompt placement,
    - New tests for selection failure pinned messages and terminal `Error` event rule.
  - Files:
    - `crates/agent_api/src/bin/fake_claude_stream_json_agent_api.rs` (extend to assert resume flags + simulate not-found)
    - `crates/agent_api/tests/**` (new focused test file to reduce conflicts)
- **Verification**:
  - `cargo test -p agent_api --features claude_code`

Checklist:
- Implement:
  - Extend fake binary to:
    - assert `--continue` or `--resume <ID>` based on scenario,
    - assert `--verbose` is present,
    - simulate selection failures in a way the adapter can translate safely.
  - Add integration tests covering:
    - valid `"last"` and `"id"` mapping,
    - invalid schema (`InvalidRequest`),
    - selection failure pinned messages + terminal `Error` event.
- Test:
  - `cargo test -p agent_api --features claude_code`

#### S2.T5 — Advertise `agent_api.session.resume.v1` capability id (Claude)

- **Outcome**: Claude backend capabilities include `"agent_api.session.resume.v1"` only after conformance is implemented and tested.
- **Inputs/outputs**:
  - Output: Claude `capabilities().ids` includes `"agent_api.session.resume.v1"`.
  - Files:
    - `crates/agent_api/src/backends/claude_code.rs`
- **Acceptance criteria**:
  - Capability advertisement matches behavior: no “advertise without mapping + tests”.

Checklist:
- Implement:
  - Add the capability id after `S2.T1`–`S2.T4` land.
- Test:
  - `cargo test -p agent_api --features claude_code`

