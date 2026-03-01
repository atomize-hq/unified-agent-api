### S4e — Advertise `agent_api.session.fork.v1` capability id (Codex)

- **User/system value**: Make Codex fork discoverable only after behavior and tests are in place, avoiding partial rollout.
- **Scope (in/out)**:
  - In:
    - Add `"agent_api.session.fork.v1"` to Codex backend capability ids after `S4a`–`S4d` have landed and tests pass.
  - Out:
    - Any behavior changes or refactors beyond the capability id toggle.
- **Acceptance criteria**:
  - Codex backend `capabilities().ids` includes `"agent_api.session.fork.v1"` only once the fork flow + mapping + safety + tests are complete.
- **Dependencies**:
  - `S4d` must be green in CI.
- **Verification**:
  - `cargo test -p agent_api --features codex`
- **Rollout/safety**:
  - This is the only point where `agent_api.session.fork.v1` becomes discoverable via capabilities.

#### S4.T7 — Advertise `agent_api.session.fork.v1` capability id (Codex)

- **Outcome**: Codex backend capabilities include `"agent_api.session.fork.v1"` only after conformance is implemented and tested.
- **Inputs/outputs**:
  - Output: Codex `capabilities().ids` includes `"agent_api.session.fork.v1"`.
  - Files:
    - `crates/agent_api/src/backends/codex.rs`

Checklist:
- Implement:
  - Add the capability id after `S4.T1`–`S4.T6` land.
