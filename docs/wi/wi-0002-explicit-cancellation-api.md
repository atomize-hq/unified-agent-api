# WI-0002 — Implement explicit cancellation API for `agent_api` runs

## Status
- Status: Proposed
- Date (UTC): 2026-02-24
- Owner(s): spensermcconnell

## Summary

Implement ADR-0014 by adding an explicit cancellation API to `agent_api`:

- `AgentWrapperGateway::run_control(...) -> AgentWrapperRunControl`
- `AgentWrapperCancelHandle::cancel()`

## Inputs

- ADR: `docs/adr/0014-agent-api-explicit-cancellation.md`
- Planning pack:
  - `docs/project_management/packs/active/agent-api-explicit-cancellation/`
- Normative specs:
  - `docs/specs/universal-agent-api/contract.md`
  - `docs/specs/universal-agent-api/run-protocol-spec.md`

## Acceptance criteria

- A caller can start a run with `run_control(...)` and call `cancel()` to request best-effort
  termination.
- On cancellation, completion resolves to `AgentWrapperError::Backend { message: "cancelled" }`.
- Cancellation is idempotent and safe to call after completion resolves.
- Capability gating:
  - backends that support explicit cancellation advertise `agent_api.control.cancel.v1`
  - `run_control(...)` fails-closed when the capability is absent
- Drop semantics remain best-effort and do not deadlock under the backend harness drain-on-drop
  posture.

## Verification

- `make fmt-check`
- `make clippy`
- `make test`
