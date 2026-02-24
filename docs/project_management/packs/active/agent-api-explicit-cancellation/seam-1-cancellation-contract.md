# SEAM-1 — Cancellation contract (CA-C01)

This seam pins the public API surface and exact cancellation semantics.

## Public API surface (v1, normative for this pack)

Add a new gateway method and new public types:

- `AgentWrapperGateway::run_control(&self, agent_kind, request) -> ...`
- `AgentWrapperRunControl { handle: AgentWrapperRunHandle, cancel: AgentWrapperCancelHandle }`
- `AgentWrapperCancelHandle::cancel(&self)`

The existing `AgentWrapperGateway::run(...) -> AgentWrapperRunHandle` remains unchanged.

### Capability gating (exact)

- A backend supports explicit cancellation if and only if it advertises capability id:
  - `agent_api.control.cancel.v1`
- If the backend does not advertise `agent_api.control.cancel.v1`, `run_control(...)` MUST fail-closed
  with:
  - `AgentWrapperError::UnsupportedCapability { capability: "agent_api.control.cancel.v1" }`

## Cancellation semantics (exact)

- Calling `cancel()` MUST be idempotent.
- `cancel()` MUST be best-effort:
  - it requests that the underlying backend process terminate, and
  - it requests that harness driver tasks stop producing additional work.
- Completion outcome:
  - If cancellation occurs before the run completes normally, `completion` MUST resolve to:
    - `Err(AgentWrapperError::Backend { message })` where `message` is pinned to:
      - `"cancelled"`
  - If cancellation is called after completion already resolved, `cancel()` is a no-op and MUST NOT
    change the already-resolved completion value.

## Relationship to drop semantics

- Drop-based cancellation semantics remain as specified by `run-protocol-spec.md` (“best-effort”).
- Consumers requiring deterministic cancellation MUST use the explicit cancellation handle.
