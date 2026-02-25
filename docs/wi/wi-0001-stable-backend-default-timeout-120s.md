# WI-0001 — Stable default timeout for built-in backends (120s)

## Status
- Status: Proposed
- Date (UTC): 2026-02-24
- Owner(s): spensermcconnell

## Summary

Set a stable, non-zero default timeout for the built-in `agent_api` backends (Codex + Claude Code)
to be safe-by-default, while preserving the existing “explicit disable” behavior when the caller
sets `timeout = Some(Duration::ZERO)`.

This work item intentionally does **not** introduce a universal/global default timeout in the
Universal Agent API spec. It makes a backend-specific default explicit and consistent across the
built-in backends.

## Motivation / Context

- The Universal Agent API contract pins that when `AgentWrapperRunRequest.timeout` is absent, a
  backend-specific default applies (the universal API MUST NOT invent a global default).
- Today, the built-in `agent_api` backends effectively default to “no timeout” when
  `default_timeout` is unset in backend config, because:
  - Codex maps `effective_timeout: None` to `Duration::ZERO` (disable) when configuring the Codex
    wrapper client.
  - Claude Code maps `effective_timeout: None` to `None` (disable) when configuring the Claude
    wrapper client.
- In orchestrators (e.g., Substrate), consumer-side timeouts typically do not guarantee backend
  process termination under the backend harness’s drain-on-drop posture. A safe default in the
  built-in backends reduces runaway processes and “hung forever” classes of failures.

## Proposed change

- Define a stable default timeout of **120 seconds** for built-in backends in `agent_api`:
  - Codex backend default timeout: 120s
  - Claude Code backend default timeout: 120s
- Preserve override semantics:
  - `request.timeout == Some(t)` overrides backend default when present.
  - `request.timeout == Some(Duration::ZERO)` explicitly disables timeout.
  - Backend config `default_timeout` continues to override the built-in default when set.

## Scope

In:
- Built-in backends only:
  - `crates/agent_api/src/backends/codex.rs`
  - `crates/agent_api/src/backends/claude_code.rs`
- Backend config defaults / how `BackendDefaults.default_timeout` is populated.

Out:
- Adding a new universal/core extension key for timeouts.
- Changing the Universal Agent API spec to define a global default timeout.
- Adding an explicit cancellation API (tracked by ADR-0014).

## Acceptance criteria

- With `CodexBackendConfig::default()` and `ClaudeCodeBackendConfig::default()`, a run with
  `AgentWrapperRunRequest.timeout == None` uses a 120s timeout by default.
- If the caller sets `AgentWrapperRunRequest.timeout == Some(Duration::ZERO)`, both backends
  disable timeouts.
- If the backend config sets `default_timeout: Some(t)`, that value is used when the request
  timeout is absent.
- Update or add tests proving:
  - request timeout overrides backend defaults
  - `Duration::ZERO` disables timeout for both built-in backends

## Implementation sketch

One of:

1) Set `default_timeout` in the backend config `Default` impls (preferred for predictability when
   callers rely on `::default()`).
2) Or, inject the default in `run()` when populating `BackendDefaults` if config omits it.

## Verification

- `make fmt-check`
- `make clippy`
- `make test`

