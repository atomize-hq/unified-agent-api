# ADR-0014 — Explicit cancellation API for `agent_api` runs
#
# Note: Run `make adr-fix ADR=docs/adr/0014-agent-api-explicit-cancellation.md` after editing to update
# the ADR_BODY_SHA256 drift guard.

## Status
- Status: Draft
- Date (UTC): 2026-02-24
- Owner(s): spensermcconnell

## Scope

- Public API surface + run semantics:
  - `AgentWrapperGateway` and run handle cancellation behavior
  - run protocol semantics for cancellation vs drop
- Built-in backend harness integration (implementation target):
  - cancellation propagation to per-backend driver tasks and spawned CLI processes

## Related Docs

- Universal Agent API baseline:
  - `docs/adr/0009-universal-agent-api.md`
  - `docs/specs/universal-agent-api/contract.md`
  - `docs/specs/universal-agent-api/run-protocol-spec.md`
- Cancellation feature pack (planning spine; this ADR’s execution home):
  - `docs/project_management/packs/active/agent-api-explicit-cancellation/README.md`
  - `docs/project_management/packs/active/agent-api-explicit-cancellation/scope_brief.md`
  - `docs/project_management/packs/active/agent-api-explicit-cancellation/seam_map.md`
  - `docs/project_management/packs/active/agent-api-explicit-cancellation/threading.md`
- Backend harness (internal refactor context):
  - `docs/adr/0013-agent-api-backend-harness.md`
- Substrate integration posture (orchestrator context):
  - `docs/integrations/substrate.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 4e134673671dfd5faa909d50055a11afa792ac2c327f6eea67e278c678703125

### Changes (operator-facing)

- Add an explicit cancellation API alongside the existing run handle.
  - Existing: consumers can drop the run handle / stop awaiting completion, but that is not a
    reliable “stop the underlying process now” mechanism under the backend harness drain-on-drop posture.
  - New: consumers can invoke an explicit cancellation handle to request best-effort termination
    of the underlying backend process and driver tasks, with a pinned, stable completion error.
  - Why: orchestrators (e.g., Substrate) need a deterministic “stop” primitive independent of
    timeout budgets and independent of drop semantics.
  - Links:
    - `docs/specs/universal-agent-api/run-protocol-spec.md`
    - `docs/specs/universal-agent-api/contract.md`
    - `docs/project_management/packs/active/agent-api-explicit-cancellation/seam-1-cancellation-contract.md`

## Problem / Context

`agent_api` is intended to be consumed by other crates/applications that orchestrate many runs and
need predictable operational controls:

- deadlines and request budgets (timeouts),
- “stop” / user-driven cancellation, and
- shutdown behavior during application exit.

Today, consumers can wrap `run_handle.completion` in a `tokio::time::timeout`, but that only stops
the consumer from awaiting. It does not reliably terminate the underlying CLI process unless the
backend/wrapper itself enforces a timeout.

Additionally, the backend harness (ADR-0013) intentionally preserves “drain-on-drop” semantics to
avoid deadlocks/cancellation hazards when a consumer drops the events stream. That safety posture
increases the importance of a separate, explicit cancellation mechanism: dropping must not imply
cancel, but consumers still need a supported way to cancel intentionally.

## Goals

- Provide a first-class cancellation mechanism that:
  - is explicit (not implicit via drop),
  - is consistent across built-in backends, and
  - best-effort terminates the underlying CLI process and all associated driver tasks.
- Make the cancellation surface usable in orchestrators like Substrate without requiring intimate
  knowledge of backend-specific child-process handles.

## Non-Goals

- Redefining timeout semantics or introducing a universal/global default timeout in the spec.
- Making cancellation synchronous or guaranteeing immediate termination on all platforms.
- Exposing backend-specific process internals publicly.

## Proposed Design (Draft)

Introduce an explicit cancellation control as a first-class, opt-in run variant:

- Add `AgentWrapperGateway::run_control(...) -> AgentWrapperRunControl` which returns:
  - the existing `AgentWrapperRunHandle` (events + completion), and
  - a new `AgentWrapperCancelHandle` (explicit cancellation signal).

This is additive (does not change the existing `AgentWrapperGateway::run(...)` signature) and keeps
the base run handle shape stable while enabling orchestrator-grade cancellation.

## User Contract (Authoritative)

### Rust API surface

- The `agent_api` crate adds:
  - `AgentWrapperGateway::run_control(...) -> AgentWrapperRunControl`
  - `AgentWrapperRunControl { handle: AgentWrapperRunHandle, cancel: AgentWrapperCancelHandle }`
  - `AgentWrapperCancelHandle::cancel()`
- Existing `AgentWrapperGateway::run(...) -> AgentWrapperRunHandle` remains supported and unchanged.

### Cancellation semantics

- Explicit cancellation is best-effort and idempotent.
- Capability gating:
  - Backends MUST advertise `agent_api.control.cancel.v1` if and only if they support explicit cancellation.
  - If a backend does not support explicit cancellation, `run_control(...)` fails-closed as `UnsupportedCapability`.
- Cancellation completion error (pinned):
  - If cancellation occurs before success completion, `completion` resolves to:
    - `Err(AgentWrapperError::Backend { message: "cancelled" })`

## Architecture Shape

- `crates/agent_api`:
  - Add a new run entrypoint that returns a cancellation handle alongside the existing run handle.
  - Wire the cancellation signal into the backend harness driver tasks so cancellation can request:
    - backend process termination (best-effort), and
    - early completion resolution (error) when the run has not already completed.
- Built-in backends (Codex + Claude Code):
  - Must support best-effort termination of spawned CLI processes under cancellation.

## Sequencing / Dependencies

- This ADR is implemented via the execution pack:
  - `docs/project_management/packs/active/agent-api-explicit-cancellation/`
- Dependencies:
  - Backend harness structure (ADR-0013) is assumed for wiring cancellation without duplicating
    driver logic per backend.

## Security / Safety Posture

- Cancellation MUST NOT cause raw backend output to leak into events or errors.
- Cancellation errors are pinned to a safe, bounded message (`"cancelled"`).

## Semantics (to be pinned before implementation)

- Explicit cancellation is invoked only via the explicit cancellation handle.
- Drop semantics remain “best-effort cancellation” as currently specified by the run protocol, but
  are not required to be reliable.
- Explicit cancellation MUST:
  - request best-effort termination of the spawned backend process,
  - request best-effort termination of driver tasks (pump + completion sender), and
  - resolve completion to a stable error outcome.

## Rollout / Backwards Compatibility

- This is an additive public API change to `agent_api` (new types + a new gateway method).
- No backwards compatibility policy is required beyond:
  - preserving `AgentWrapperGateway::run(...)` behavior, and
  - keeping cancellation opt-in via `run_control(...)`.

## Validation Plan (Authoritative for this ADR once Accepted)

- Add unit/integration tests proving:
  - calling `cancel()` terminates a running fake backend,
  - completion resolves to the cancellation outcome,
  - cancellation does not violate event bounds/redaction rules, and
  - drop-without-cancel preserves current drain-on-drop semantics.

## Decision Summary

This ADR introduces multiple non-trivial decisions (API shape, completion error representation, and
backend-level cancellation responsibilities). Those decisions are tracked in:

- `docs/project_management/packs/active/agent-api-explicit-cancellation/decision_register.md`
