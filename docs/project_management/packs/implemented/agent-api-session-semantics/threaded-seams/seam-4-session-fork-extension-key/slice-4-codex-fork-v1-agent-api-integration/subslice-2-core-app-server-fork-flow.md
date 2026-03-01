### S4b — Core Codex fork flow via app-server (`initialize` → list/select → fork → `turn/start`)

- **User/system value**: Make `agent_api.session.fork.v1` actually work for Codex by driving the pinned `codex app-server` JSON-RPC sequence and returning a live run handle.
- **Scope (in/out)**:
  - In:
    - Enable the fork key in the Codex backend `supported_extension_keys()` allowlist (fail-closed gate).
    - Implement the core JSON-RPC call order using SA-C06 (`S3`) typed RPC helpers:
      - `initialize` (experimentalApi handshake),
      - `thread/list` (only for selector `"last"`),
      - `thread/fork`,
      - `turn/start` on the forked thread with the pinned `input[]` prompt mapping.
    - Produce an `AgentWrapperRunHandle` that:
      - yields an event stream (may be minimally mapped; full mapping is `S4c`),
      - completes based on the `turn/start` lifecycle.
  - Out:
    - Bounded notification→event mapping details (`S4c`).
    - Approval-required fail-fast and explicit cancellation semantics (`S4c`).
    - Selection-failure pinned message translation and terminal `Error` event rule (`S4c`).
- **Acceptance criteria**:
  - With `agent_api.session.fork.v1` present:
    - selector `"id"` calls `thread/fork` with `threadId == <id>` and then `turn/start` on the forked id.
    - selector `"last"` calls `thread/list` with `cwd == <effective working dir>`, applies the pinned deterministic selection algorithm, then forks that source id.
  - `turn/start` request uses the pinned prompt mapping: `input == [{\"type\":\"text\",\"text\":<prompt>,\"text_elements\":[]}]`.
- **Dependencies**:
  - `S1` shared fork selector parser.
  - `S3` SA-C06 typed Codex app-server RPC support.
  - Normative: `docs/specs/codex-app-server-jsonrpc-contract.md` (method shapes + selection algorithm + prompt mapping).
  - Normative: `docs/specs/universal-agent-api/contract.md` (effective working directory).
- **Verification**:
  - `cargo test -p agent_api --features codex`
- **Rollout/safety**:
  - Still do not advertise `agent_api.session.fork.v1` capability id until `S4d` tests pass (`S4e`).

#### S4.T2 — Implement Codex fork flow via app-server: select source → fork → `turn/start`

- **Outcome**: The Codex backend can fork from `"last"` or `"id"` using SA-C06 and stream events from the follow-up `turn/start`.
- **Inputs/outputs**:
  - Inputs: typed fork selector; `AgentWrapperRunRequest.prompt`; effective working dir; non-interactive policy.
  - Outputs:
    - `AgentWrapperRunHandle` whose `events` stream reflects mapped notifications from `turn/start`,
    - completion resolves to `Ok(ExitStatus)` on success or a safe `AgentWrapperError` on failure.
  - Files:
    - `crates/agent_api/src/backends/codex/fork.rs`
- **Implementation notes**:
  - selector `"last"`:
    - compute effective working directory (`contract.md`) and pass it into `thread/list` filtering,
    - apply the pinned selection algorithm (use `S3` helper if provided).
  - selector `"id"`: use the validated id as `threadId`.
  - Call order (pinned):
    1) `initialize` (with experimentalApi)
    2) `thread/list` (only for `"last"`)
    3) `thread/fork`
    4) `turn/start`

Checklist:
- Implement:
  - Add an async task that drives the JSON-RPC sequence and forwards mapped notifications into the consumer-visible event channel.
  - Ensure selection failure short-circuits before `thread/fork` / `turn/start`.
