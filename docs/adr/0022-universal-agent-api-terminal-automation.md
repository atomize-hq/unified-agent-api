# ADR-0022 — Headless terminal automation in backend crates
#
# Note: Run `make adr-fix ADR=docs/adr/0022-universal-agent-api-terminal-automation.md`
# after editing to update the ADR_BODY_SHA256 drift guard.

## Status
- Status: Draft
- Date (UTC): 2026-04-06
- Owner(s): spensermcconnell

## Scope

- Add optional, backend-owned headless terminal automation transports in:
  - `crates/claude_code`
  - `crates/codex`
- Keep automation background-only:
  - no foregrounded TUI,
  - no operator console,
  - no change to the headless posture of the wrappers.
- Keep `agent_api` unchanged for now:
  - no new Universal Agent API session tiers,
  - no new `agent_api` public transport abstraction in this ADR.

## Related Docs

- Workspace + parity-lane structure:
  - `docs/adr/0006-agent-wrappers-workspace.md`
- Universal Agent API scope and non-goals:
  - `docs/adr/0009-universal-agent-api.md`
  - `docs/specs/universal-agent-api/contract.md`
  - `docs/specs/universal-agent-api/run-protocol-spec.md`
  - `docs/specs/universal-agent-api/event-envelope-schema-spec.md`
- Current backend-owned transport contracts:
  - `docs/specs/claude-code-session-mapping-contract.md`
  - `docs/specs/codex-streaming-exec-contract.md`
  - `docs/specs/codex-app-server-jsonrpc-contract.md`
- Existing PTY precedent:
  - `crates/claude_code/src/client/setup_token/pty.rs`
- Parity-lane evidence / rollout companion:
  - `docs/adr/0023-cli-manifest-coverage-for-headless-automation.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 2581291fa9a1623cbc88963cf2914c1c2282c0e793775b21cad736b7dc3930f2

### Decision (draft)

- Add optional **headless** terminal automation support at the individual backend-crate level,
  not in `agent_api` first.
- The automation transport exists to recover capabilities that are not reachable through the
  backend's primary structured transport:
  - Claude Code primary transport remains `claude --print --output-format stream-json`
  - Codex primary transports remain:
    - streaming exec/resume for exec-like flows
    - app-server JSON-RPC for fork/session flows
- PTY-backed automation is a **secondary transport**, not the new default.
- `agent_api` remains a headless, structured run contract in this ADR; future promotion requires
  proven convergence across backend crates.

### Why

- The repo is already organized around backend-owned transports and parity lanes, not around a
  single monolithic execution model.
- Some CLI behavior is materially different under a TTY even when the caller remains fully
  headless and unattended.
- Adding PTY/background automation inside `claude_code` and `codex` preserves the current
  architecture:
  backend crates own transport truth first, and `agent_api` only universalizes behavior after it
  is proven stable across backends.

## Problem / Context

The original `0022` draft correctly identified a real limitation: not all upstream agent behavior
is reachable through "spawn process, consume structured stdout, exit" flows.

However, the initial framing pushed that concern into the Universal Agent API itself. That was not
well grounded in the repo's current contracts:

- `agent_api` is explicitly a headless universal run surface:
  - `docs/adr/0009-universal-agent-api.md`
  - `docs/specs/universal-agent-api/contract.md`
- the built-in backends already expose backend-specific primary transports:
  - Claude Code headless print/stream-json
  - Codex streaming exec/resume
  - Codex app-server JSON-RPC for fork/session flows
- the repo already has a crate-per-agent and parity-lane model:
  - transport logic belongs in `crates/<agent>`
  - coverage/reporting evidence belongs in `cli_manifests/<agent>`

The right correction is:

- keep the feature **headless**,
- move it down to the backend crates,
- treat it as a fallback transport for capability gaps,
- and only consider universalization later if the backends converge on a genuinely shared model.

## Goals

- Recover backend capabilities that require a TTY or PTY while keeping callers fully headless.
- Preserve the current structured/headless primary transports wherever they already work well.
- Keep transport choice backend-owned and flow-specific.
- Avoid forcing `agent_api` into a broader public contract before the backend semantics are proven.

## Non-Goals

- Foregrounding a TUI for human use.
- Building an operator console.
- Replacing the current primary structured transports.
- Defining a new universal terminal/session trait in `agent_api` in this ADR.
- Claiming that every Codex or Claude Code flow should use PTY automation.

## Decision

### 1. Backend crates own the automation transport

The implementation home for headless terminal automation is:

- `crates/claude_code`
- `crates/codex`

The automation transport is backend-private or backend-specific public API first. It is not a new
`agent_api` transport layer in this ADR.

### 2. Primary structured transports remain preferred

Automation is a fallback, not the new baseline.

#### Claude Code

Primary transport remains:

- `claude --print --output-format stream-json`

Pinned behavior for that surface continues to live in:

- `docs/specs/claude-code-session-mapping-contract.md`

Headless PTY automation may be added only for flows or capabilities that cannot be expressed
honestly through that structured print surface.

#### Codex

Primary transport remains flow-dependent:

- exec/resume-like flows:
  - `docs/specs/codex-streaming-exec-contract.md`
- fork/session flows:
  - `docs/specs/codex-app-server-jsonrpc-contract.md`

Codex automation decisions must remain flow-specific:

- PTY automation may make sense for some exec/resume gaps.
- app-server JSON-RPC remains the preferred transport for fork/session flows unless a specific
  capability gap justifies a backend-owned fallback and a new contract says exactly when that
  fallback is used.

This ADR does not authorize "switch all Codex flows to PTY."

### 3. Headless remains a hard requirement

The new automation transport must remain suitable for unattended execution:

- no interactive foreground UI requirement,
- no human-in-the-loop assumption,
- deterministic startup, timeout, termination, and error translation,
- safe/redacted outputs at the wrapper boundary.

If a backend automates prompts or confirmations internally, that behavior must still be explicit in
the backend contract. Silent approval automation is not acceptable.

### 4. Backend contracts must pin transport selection policy

Each backend crate that adopts headless terminal automation must define, in backend-owned docs:

- which capabilities/flows still use the primary structured transport,
- which capabilities/flows use PTY-backed automation,
- the precedence rules when both transports are theoretically possible,
- the safe failure mode when the automation transport cannot be established.

This repo should not rely on implicit "if structured mode failed, retry with PTY" behavior.

### 5. `agent_api` is unchanged in this ADR

This ADR does not change:

- `AgentWrapperBackend`
- `AgentWrapperRunHandle`
- `AgentWrapperRunControl`
- the `agent_api` capability model
- the universal event envelope

If backend-level automation later converges strongly enough to justify a shared capability or
shared public surface, that must be proposed in a separate ADR after real backend evidence exists.

## Implementation Preferences

### Rust-native PTY transport is acceptable

Where a PTY is needed, Rust-native PTY management is preferred.

`portable_pty` is an acceptable implementation choice and already has a narrow in-repo precedent at:

- `crates/claude_code/src/client/setup_token/pty.rs`

### No TUI framework as a control plane

This ADR does not justify adopting a TUI framework such as Ratatui as the control plane for
backend automation.

If the repo later wants an operator-facing UI, that can be built above the backend automation
layer. It is not part of this decision.

### Safety posture remains unchanged

Even when a PTY is involved, the wrapper boundary must preserve current repo safety rules:

- no raw backend line leakage through universal event payloads,
- safe/redacted errors,
- bounded capture and bounded reporting,
- explicit timeout and termination behavior.

## Consequences

### Positive

- The repo can recover headless capabilities trapped behind TTY-sensitive behavior without forcing
  `agent_api` to universalize too early.
- Transport truth stays close to the backend crates that already own it.
- Claude Code and Codex can evolve independently where their transport realities differ.

### Negative

- The backend crates become more complex.
- Some capabilities may be covered differently across backends for a while.
- Additional parity-lane and report work is required so new automation-backed support is visible and
  auditable.

## Rollout Plan

1. Write backend-owned transport-selection contracts for the first adopted flows.
2. Implement one high-value headless automation path in `crates/claude_code`.
3. Evaluate Codex per flow instead of adopting PTY globally.
4. Update parity-lane coverage/reporting so automation-backed support is represented in
   `cli_manifests/*`.
5. Reassess whether any universal `agent_api` promotion is warranted only after the backend
   transports have shipped and stabilized.

## Validation

- Backend-crate tests must pin:
  - transport selection,
  - deterministic startup behavior,
  - timeout handling,
  - best-effort termination,
  - safe error translation.
- Existing structured transport tests must remain green.
- Parity-lane reports must stop flagging automation-covered surfaces as missing once the companion
  manifest/coverage work from ADR-0023 is implemented.
