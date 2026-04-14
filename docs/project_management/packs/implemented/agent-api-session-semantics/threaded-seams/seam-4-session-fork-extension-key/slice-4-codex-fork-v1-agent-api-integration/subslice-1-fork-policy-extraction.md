### S4a — Codex fork policy extraction (parse/validate + policy plumbing)

- **User/system value**: Establish the Codex backend’s pre-spawn fork selector plumbing so later sub-slices can implement behavior without re-threading policy.
- **Scope (in/out)**:
  - In:
    - Add Codex backend policy fields for `agent_api.session.fork.v1`.
    - Parse/validate `extensions["agent_api.session.fork.v1"]` via the shared helper from `S1`.
    - Isolate Codex fork-specific wiring in `crates/agent_api/src/backends/codex/fork.rs` (module-only; behavior lands in later sub-slices).
  - Out:
    - Enabling runtime support (do not add the key to `supported_extension_keys()` yet).
    - Any `codex app-server` JSON-RPC calls.
    - Capability advertisement (that is `S4e`).
- **Acceptance criteria**:
  - Codex backend policy extraction can represent an optional fork selector (`None` / `"last"` / `"id"`) using the shared `S1` selector type.
  - No behavior change occurs for production users: requests containing `agent_api.session.fork.v1` are still rejected as `UnsupportedCapability` until `S4b` enables support.
- **Dependencies**:
  - `S1` shared fork selector parser (`docs/specs/unified-agent-api/extensions-spec.md` schema).
- **Verification**:
  - `cargo test -p agent_api --features codex`
- **Rollout/safety**:
  - Keep `supported_extension_keys()` and capability ids unchanged in this sub-slice to avoid partial, incorrect support landing.

#### S4.T1 — Codex backend: parse/validate `fork.v1` and extract fork policy (pre-spawn)

- **Outcome**: Codex backend policy extraction supports an optional fork selector and enforces `.v1` closed-schema validation pre-spawn.
- **Inputs/outputs**:
  - Inputs: `AgentWrapperRunRequest.extensions`.
  - Outputs:
    - Extended Codex policy including `fork: Option<...>` (typed selector),
    - `InvalidRequest` for schema violations (once the key is enabled in `supported_extension_keys()` in `S4b`),
    - resume↔fork contradiction check wired for when both keys are supported (R0 precedence respected).
  - Files (preferred for isolation):
    - `crates/agent_api/src/backends/codex.rs` (thread policy through backend harness)
    - `crates/agent_api/src/backends/codex/fork.rs` (new; isolate fork policy + app-server wiring)
- **Implementation notes**:
  - Use the shared parser from `S1` (no duplicated JSON parsing).
  - Keep the fork codepath isolated from the existing `codex exec` streaming mapping to reduce churn.

Checklist:
- Implement:
  - Add `EXT_SESSION_FORK_V1` constant and parse it into Codex policy when present.
- Defer (to `S4b`):
  - Add the fork key to `supported_extension_keys()` only once the app-server flow is implemented.
