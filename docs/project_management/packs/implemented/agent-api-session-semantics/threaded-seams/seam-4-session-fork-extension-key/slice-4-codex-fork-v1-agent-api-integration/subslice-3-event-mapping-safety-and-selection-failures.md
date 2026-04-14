### S4c — Bounded event mapping + safety (approval fail-fast, cancellation, selection failures)

- **User/system value**: Make Codex fork runs safe and spec-conformant by bounding notifications into Unified Agent API events, enforcing non-interactive behavior, and translating selection failures deterministically.
- **Scope (in/out)**:
  - In:
    - Map app-server notifications into bounded `AgentWrapperEvent`s (no raw payloads in `data`).
    - Enforce non-interactive safety (`agent_api.exec.non_interactive`, default `true`) by:
      - setting `approvalPolicy="never"`,
      - detecting approval requests and failing fast with pinned `"approval required"`,
      - sending `$ /cancelRequest` best-effort before failing.
    - Implement selection-failure translation with pinned messages:
      - `"no session found"` (empty `"last"` scope),
      - `"session not found"` (unknown `"id"`).
    - Wire explicit cancellation via `$ /cancelRequest` for in-flight `turn/start` request ids and enforce cancellation precedence from `run-protocol-spec.md`.
  - Out:
    - Capability advertisement (`S4e`).
- **Acceptance criteria**:
  - Notifications map to `AgentWrapperEvent` kinds per `docs/specs/codex-app-server-jsonrpc-contract.md`, with bounds enforced by `event-envelope-schema-spec.md`.
  - In non-interactive mode, approval prompts never hang: `"approval required"` is surfaced consistently via terminal `Error` event (when a stream exists) and completion error.
  - Selection failures use pinned messages and obey the terminal `Error` event rule when a stream exists.
  - `run_control.cancel()` sends `$ /cancelRequest` for the in-flight `turn/start` id and produces the pinned `"cancelled"` outcome per `run-protocol-spec.md`.
- **Dependencies**:
  - Normative: `docs/specs/codex-app-server-jsonrpc-contract.md` (notification mapping + approval request definition + `$ /cancelRequest`).
  - Normative: `docs/specs/unified-agent-api/extensions-spec.md` (selection-failure messages + terminal `Error` rule).
  - Normative: `docs/specs/unified-agent-api/run-protocol-spec.md` (cancellation semantics/precedence).
  - Normative: `docs/specs/unified-agent-api/event-envelope-schema-spec.md` (bounds + redaction rules).
- **Verification**:
  - `cargo test -p agent_api --features codex`
- **Rollout/safety**:
  - Still do not advertise `agent_api.session.fork.v1` until `S4d` lands.

#### S4.T3 — Implement bounded notification → `AgentWrapperEvent` mapping (no raw payloads in `data`)

- **Outcome**: App-server notifications are surfaced to callers as bounded Unified Agent API events without leaking raw backend payloads.
- **Inputs/outputs**:
  - Inputs: JSON-RPC notification method + params.
  - Outputs: `AgentWrapperEvent` stream items:
    - `TextOutput` for deltas,
    - `ToolCall`/`ToolResult` for item lifecycle (metadata-only),
    - `Status` for turn lifecycle,
    - `Error` for errors.
  - Files:
    - `crates/agent_api/src/backends/codex/fork.rs` (or a new `codex/app_server_mapping.rs` module)
- **Implementation notes**:
  - Enforce text bounds per `event-envelope-schema-spec.md` (split if needed).
  - Do not forward raw params blobs in `data`; only include safe metadata fields when necessary.

Checklist:
- Implement:
  - Add a mapping function that matches the pinned method set from `codex-app-server-jsonrpc-contract.md`.
- Validate:
  - `AgentWrapperEvent.data` is `None` (or bounded metadata-only) for app-server-derived events.

#### S4.T4 — Non-interactive safety + approval required fail-fast (`"approval required"`)

- **Outcome**: The Codex fork flow never hangs in unattended mode; approval requests trigger a deterministic, safe failure.
- **Inputs/outputs**:
  - Inputs: `agent_api.exec.non_interactive` (default `true`), JSON-RPC notifications.
  - Outputs:
    - If approval request observed:
      - send `$ /cancelRequest` for the in-flight `turn/start` id (best-effort),
      - emit exactly one terminal `Error` event with `message == "approval required"` if the stream exists,
      - resolve completion with `Err(AgentWrapperError::Backend { message: "approval required" })`.
  - Files:
    - `crates/agent_api/src/backends/codex/fork.rs`
- **Dependencies**:
  - Normative: `docs/specs/codex-app-server-jsonrpc-contract.md` (approval request definition + fail-fast ordering + cancellation).

Checklist:
- Implement:
  - Set `approvalPolicy="never"` on both `thread/fork` and `turn/start` when non-interactive is true.
  - Detect approval requests:
    - `method == "codex/event"` and payload `type` is `"approval_required"` or `"approval"` (directly or under `params.msg`).
  - On detection, cancel `turn/start` and fail fast with the pinned message.

#### S4.T5 — Selection-failure translation (pinned messages + terminal `Error` event rule)

- **Outcome**: Fork selection failures are translated into pinned safe Unified Agent API errors.
- **Inputs/outputs**:
  - Inputs: selector `"last"`/`"id"`, thread/list result, app-server errors.
  - Outputs:
    - `"no session found"` (empty `"last"` scope),
    - `"session not found"` (unknown id),
    - terminal `Error` event emission when a stream exists (per `extensions-spec.md`).
  - Files:
    - `crates/agent_api/src/backends/codex/fork.rs`
- **Implementation notes**:
  - `"last"` empty scope can be determined deterministically from `thread/list` (post-spawn but before `turn/start`).
  - `"id"` unknown/unforkable may surface as a JSON-RPC error from `thread/fork`; translate safely without embedding raw backend details.

Checklist:
- Implement:
  - Translate the two selection-failure cases into pinned `AgentWrapperError::Backend` messages.
  - Emit exactly one terminal `Error` event when the consumer-visible stream exists.
