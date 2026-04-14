### S3 — SA-C06: Codex app-server fork RPC surface (typed client + protocol tests)

- **User/system value**: Establish a pinned, testable, headless Codex fork substrate (`codex app-server` JSON-RPC) before integrating it into `agent_api`, de-risking protocol drift and ensuring cancellation + selection behavior can be validated in isolation.
- **Scope (in/out)**:
  - In:
    - Implement the SA-C06 typed app-server RPC surface in `crates/codex` per `docs/specs/codex-app-server-jsonrpc-contract.md`:
      - `thread/list` request/response types (including pagination cursors) and deterministic `"last"` selection helper,
      - `thread/fork` request/response types (`result.thread.id`),
      - `turn/start` request type supporting the pinned prompt→`input[]` mapping (`text_elements: []`) and `approvalPolicy`,
      - `$ /cancelRequest` support for in-flight requests (already supported by transport; tests pin usage).
    - Add protocol-level tests (fake JSON-RPC server) pinning:
      - `thread/list` paging + deterministic `"last"` selection,
      - `thread/fork` response parsing,
      - `turn/start` prompt mapping request shape, and
      - cancellation (`$ /cancelRequest`) request shape + cancellation response handling.
    - Ensure `initialize` can be sent with `capabilities.experimentalApi=true` for deterministic method availability (per contract).
  - Out:
    - `agent_api` Codex backend fork integration and Unified Agent API event mapping (`S4`).
    - Any capability advertisement in `agent_api` (this slice is wrapper-only).
- **Acceptance criteria**:
  - Wrapper can call:
    - `thread/list` with the pinned param subset (`cwd`, `cursor`, `limit`, `sortKey`),
    - `thread/fork` and read the forked id from `result.thread.id`,
    - `turn/start` with the pinned `input[]` shape (`text_elements: []`),
    - `$ /cancelRequest` for an in-flight `turn/start` request id.
  - Tests pin:
    - request field names and values per contract (`cwd` filtering, `sortKey="updated_at"`, paging),
    - deterministic `"last"` selection algorithm: max `(updatedAt, createdAt, id)` across all pages.
- **Dependencies**:
  - Normative: `docs/specs/codex-app-server-jsonrpc-contract.md` (method names, field names, selection algorithm, cancellation, non-interactive requirements).
- **Verification**:
  - `cargo test -p codex --all-features` (or `make test`).
- **Rollout/safety**:
  - Wrapper-only, additive surfaces; no behavior change for `agent_api` until `S4` adopts it.

#### S3.T1 — Add typed protocol models for `thread/list`, `thread/fork`, and fork-oriented `turn/start` params

- **Outcome**: The codex crate has concrete, serde-backed Rust types matching the pinned app-server wire contract, making request construction and response parsing deterministic.
- **Inputs/outputs**:
  - Inputs: pinned wire shapes in `docs/specs/codex-app-server-jsonrpc-contract.md`.
  - Outputs: new/extended protocol structs/enums with serde renames matching wire fields.
  - Files:
    - `crates/codex/src/mcp/protocol.rs`
- **Implementation notes**:
  - Prefer additive “v2” structs where existing “thread/start/resume + task/notification” flows would otherwise be destabilized.
  - Model the contract’s camelCase fields (`threadId`, `createdAt`, `updatedAt`, `nextCursor`) via serde renames.
  - For the pinned prompt mapping:
    - extend the input model so serialized JSON includes `text_elements: []` when `type=="text"`.

Checklist:
- Implement:
  - Add method constants:
    - `METHOD_THREAD_LIST` (`"thread/list"`)
    - `METHOD_THREAD_FORK` (`"thread/fork"`)
  - Add typed params/results:
    - `ThreadListParams` + `ThreadListResponse` + `ThreadSummary`
    - `ThreadForkParams` + `ThreadForkResponse` (`thread.id`)
    - fork-oriented `TurnStartParamsV2` (or an additive extension to `TurnStartParams`) that can carry `approvalPolicy` and the pinned `input[]` text shape.
- Validate:
  - Serialization matches the contract’s field names exactly.

#### S3.T2 — Extend `CodexAppServer` client with typed RPC helpers + experimentalApi handshake

- **Outcome**: Call sites can issue fork RPCs without hand-rolling JSON values, and fork flows can reliably opt into the experimental API set during `initialize`.
- **Inputs/outputs**:
  - Inputs: new protocol types from `S3.T1`.
  - Outputs: typed helper methods on `CodexAppServer`:
    - `thread_list(...)`,
    - `thread_fork(...)`,
    - `turn_start_v2(...)` (or an upgraded `turn_start` that supports the pinned fork shape),
    - `start_experimental(...)` (or equivalent) that sends `capabilities.experimentalApi=true` in `initialize`.
  - Files:
    - `crates/codex/src/mcp/client.rs`
- **Implementation notes**:
  - Keep the existing `start(...)` behavior stable; prefer a new entrypoint for the experimental handshake so non-fork users remain unaffected.
  - Prefer mapping JSON-RPC `result` directly into typed response structs using `map_response::<T>(...)`.

Checklist:
- Implement:
  - Add `CodexAppServer` methods for new RPCs.
  - Add an “experimental API” start/handshake helper used by fork flows/tests.
- Test:
  - Covered by `S3.T4`–`S3.T5`.

#### S3.T3 — Implement deterministic `"last"` selection helper (paging + max tuple)

- **Outcome**: A single, reusable function that implements the contract’s `"last"` selection algorithm without embedding `agent_api` concerns.
- **Inputs/outputs**:
  - Inputs: `ThreadListResponse` pages.
  - Output: selected source thread id or `None` when no threads exist.
  - Files:
    - `crates/codex/src/mcp/client.rs` (or a small helper module under `crates/codex/src/mcp/`)
- **Algorithm (pinned)**:
  - Aggregate all `thread/list` pages (follow `nextCursor` until `null`).
  - Select the thread with max `(updatedAt, createdAt, id)` using lexicographic ordering (largest wins).

Checklist:
- Implement:
  - Add a helper that:
    - calls `thread/list` with `sortKey="updated_at"` and `limit=100`,
    - follows pagination,
    - selects max tuple deterministically.
- Test:
  - Unit-test the tuple ordering separately from paging where possible.

#### S3.T4 — Add fake app-server harness that supports fork RPCs without breaking existing tests

- **Outcome**: A deterministic fake JSON-RPC server capable of exercising `thread/list`/`thread/fork`/`turn/start`/`$/cancelRequest` without destabilizing the existing “task/notification” app-server fixtures.
- **Inputs/outputs**:
  - Inputs: existing fake app-server (`write_fake_app_server`) and tests in `tests_core/app_server_rpc_flows.rs`.
  - Outputs: a *new* fake server implementation (or a mode flag) for fork RPC tests.
  - Files:
    - `crates/codex/src/mcp/test_support.rs`
    - (if needed) `crates/codex/src/mcp/tests_core.rs` (register new test module)

Checklist:
- Implement:
  - Add `write_fake_app_server_fork_v1()` that:
    - responds to `initialize`,
    - implements `thread/list` with paging + stable data shapes,
    - implements `thread/fork` returning `{"thread": {"id": "..."} }` in `result`,
    - implements `turn/start` and records the received params for assertions,
    - supports `$ /cancelRequest` by returning JSON-RPC error `code=-32800`.
  - Add `start_fake_app_server_fork_v1()` helper that launches the new fake server.

#### S3.T5 — Pin protocol flow tests for fork RPCs (list/select, fork, turn/start shape, cancel)

- **Outcome**: Wrapper-level tests that validate request/response shapes and the deterministic selection algorithm, independent of `agent_api`.
- **Inputs/outputs**:
  - Inputs: fake server from `S3.T4`.
  - Outputs: new `codex` crate tests for fork RPCs.
  - Files (prefer new file to reduce conflicts):
    - `crates/codex/src/mcp/tests_core/app_server_fork_rpc_flows.rs`
    - `crates/codex/src/mcp/tests_core.rs` (module include)
- **Pinned assertions**:
  - `thread/list` is called with:
    - `cwd == <provided cwd>`,
    - `sortKey == "updated_at"`,
    - `limit == 100`,
    - pagination follows `nextCursor` until null.
  - `"last"` selection chooses max `(updatedAt, createdAt, id)` across pages.
  - `thread/fork` parses forked id from `result.thread.id`.
  - `turn/start` request contains:
    - `input == [{\"type\":\"text\",\"text\":<prompt>,\"text_elements\":[]}]`,
    - `approvalPolicy` present when provided by the caller.
  - Cancellation:
    - calling `cancel(request_id)` sends `$ /cancelRequest` with `params.id == request_id` and surfaces `McpError::Cancelled`.

Checklist:
- Implement:
  - Add a focused test per method + one integrated “last → fork → start” flow.
- Test:
  - `cargo test -p codex --all-features`

