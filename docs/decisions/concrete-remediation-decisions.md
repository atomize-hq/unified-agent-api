# Concrete remediation decisions

Dates (UTC): 2026-02-24, 2026-02-28

Scope: Documentation-only remediation for `agent_api` concrete-audit gaps, based on:
- `docs/project_management/packs/active/agent-api-explicit-cancellation/audit-reports/concrete-audit.report.json`
- `concrete-audit.report.json` (session semantics pack + UAA specs)

This document records concrete decisions introduced where the audit required an explicit choice and
no single authoritative source fully pinned the behavior.

## CRD-0001 — Explicit cancellation is not an exception to completion gating

**Decision**

Explicit cancellation MUST still respect DR-0012 completion gating:
- `completion` MUST NOT resolve until the underlying backend process has exited, and
- unless the consumer drops `events` (opt-out), the consumer-visible `events` stream is final (`None`).

**Context**

CA-0001 required specifying whether explicit cancellation is an exception to the completion gating
rule, and how cancellation interacts with the “no late events after completion” guarantee.

**Chosen spec**

Pinned in `docs/specs/universal-agent-api/run-protocol-spec.md` under:
- “Relationship between `completion` and the event stream (DR-0012 / v1, normative)”
- “Explicit cancellation semantics (v1, normative)” → “Completion gating”

**Rationale**

- Keeps `completion` as the reliable “the run is fully done (process exit reached)” signal, which
  lets tests treat `completion` resolution as the termination observation point.
- Preserves the “no late events after completion” invariant for consumers that keep `events` alive.

**Implications**

- Implementations MUST request best-effort termination on `cancel()` and resolve completion only
  after process exit.
- Consumers may observe stream finality before completion in cancellation paths.

## CRD-0002 — Consumer-visible event stream closes on cancellation and buffered events are dropped

**Decision**

After cancellation is requested, the backend MUST:
- stop forwarding additional events to the consumer-visible `events` stream, and
- close the consumer-visible `events` stream (consumer can observe `None`).

If the backend buffers events for post-hoc emission (non-live), any buffered events not yet emitted
MUST be dropped (MUST NOT be flushed to the consumer after cancellation).

**Context**

CA-0001 required pinning consumer-visible event-stream behavior and buffered-event handling after
`cancel()`.

**Chosen spec**

Pinned in `docs/specs/universal-agent-api/run-protocol-spec.md` under:
- “Explicit cancellation semantics (v1, normative)” → “Consumer-visible event stream behavior after `cancel()`”

**Rationale**

- Aligns with the SEAM-2 driver plan (“stop forwarding” + “close stream”) and provides deterministic
  behavior for orchestrators.
- Avoids ambiguous “late events after completion” interactions in cancellation flows.

**Implications**

- Implementations MUST separate internal draining (to avoid deadlocks) from consumer-visible
  forwarding.
- Consumers must not rely on receiving post-hoc buffered events after requesting cancellation.

## CRD-0003 — Cancellation outcome precedence and tie-breaking

**Decision**

If cancellation is requested before `completion` resolves, the completion outcome MUST be the pinned
cancellation error:
- `Err(AgentWrapperError::Backend { message: "cancelled" })`

This MUST override any backend `Ok(...)` or `Err(...)` completion that would otherwise resolve after
cancellation is requested.

If `completion` resolves first, cancellation is a no-op and MUST NOT change the already-resolved
value.

If cancellation and completion become ready concurrently, cancellation wins.

**Context**

CA-0002 required precedence rules for cancellation vs backend completion outcomes and explicit
tie-breaking for simultaneous readiness.

**Chosen spec**

Pinned in:
- `docs/specs/universal-agent-api/run-protocol-spec.md` → “Explicit cancellation semantics” → “Completion outcome and precedence (pinned)”
- `docs/project_management/packs/active/agent-api-explicit-cancellation/seam-1-cancellation-contract.md` → “Completion outcome and precedence (pinned)”

**Rationale**

- Ensures deterministic cancellation behavior for orchestrators and tests.
- Avoids flake from race ambiguity and avoids exposing backend-specific kill/exit error shapes.

**Implications**

- Implementations should use cancellation-biased race resolution when cancellation and backend
  completion are simultaneously ready.

## CRD-0004 — SEAM-4 pinned timeouts and parameters

**Decision**

SEAM-4 tests use the following pinned parameters in v1:

- `FIRST_EVENT_TIMEOUT = 1s`
- `CANCEL_TERMINATION_TIMEOUT = 3s`
- `DROP_COMPLETION_TIMEOUT = 3s`
- `MANY_EVENTS_N = 200`

No platform-specific adjustment in v1 (same values on all supported platforms).

**Context**

CA-0007 required numeric timeout values, explicit pass/fail termination criteria, and pinned numeric
parameters for backpressure/drain regression tests.

**Chosen spec**

Pinned in:
- `docs/project_management/packs/active/agent-api-explicit-cancellation/threaded-seams/seam-4-tests/slice-1-explicit-cancel-integration.md`
- `docs/project_management/packs/active/agent-api-explicit-cancellation/threaded-seams/seam-4-tests/slice-2-drop-regression.md`

**Rationale**

- Keeps CI-friendly “seconds, not minutes” budgets while matching existing repo patterns (other
  `agent_api` integration tests commonly use ~1–3 second `tokio::time::timeout` windows).

**Implications**

- Cancellation/termination implementations must satisfy these timeouts in CI on supported platforms.

## CRD-0005 — Session selection failure is an error (no implicit start-new)

**Decision**

For `agent_api.session.resume.v1` and `agent_api.session.fork.v1`:
- If selection fails (`selector=="last"` with no prior session in scope; or `selector=="id"` not found),
  the backend MUST fail the run and MUST NOT start a new session implicitly.

**Context**

CA-0002 required a concrete error model for “no session” / “id not found” so implementers can write
deterministic tests and built-in backends don’t diverge.

**Chosen spec**

Pinned in `docs/specs/universal-agent-api/extensions-spec.md` under:
- `agent_api.session.resume.v1` → “Selection failure behavior (v1, normative)”
- `agent_api.session.fork.v1` → “Selection failure behavior (v1, normative)”

**Rationale**

Resume/fork keys are explicit user intent; silently starting a new session would violate caller
expectations and make failure-path tests non-deterministic.

**Implications**

- Built-in backends must translate backend-specific “not found” outcomes into a universal error.
- Orchestrators can treat selection failures as actionable errors rather than “maybe started new”.

## CRD-0006 — Pinned selection-failure error variant + messages

**Decision**

Selection failures MUST be surfaced as:
- `AgentWrapperError::Backend { message }`
- with `message` pinned as:
  - `"no session found"` for `selector=="last"`,
  - `"session not found"` for `selector=="id"`.

If selection failure occurs after an `AgentWrapperRunHandle` is returned, the backend MUST emit
exactly one terminal `AgentWrapperEventKind::Error` event with the same pinned message before
closing the stream.

**Context**

CA-0002 required choosing a concrete universal error variant/message and specifying stream emission
requirements when an events stream exists.

**Chosen spec**

Pinned in `docs/specs/universal-agent-api/extensions-spec.md` (same section as CRD-0005).

**Rationale**

- Keeps errors safe-by-default and testable without depending on backend stderr content.
- Provides a single universal behavior for both built-in backends and future adapters.

**Implications**

- Tests should assert the exact message strings, not backend-specific outputs.
- Backends must not embed raw backend lines in `AgentWrapperEvent.data` / error messages.

## CRD-0007 — Codex app-server “last” selection algorithm and tie-breaker

**Decision**

For Codex fork via app-server `thread/list` (selector `"last"`), “most recent” is selected by the
maximum tuple:
- `(updatedAt, createdAt, id)` (lexicographic; largest wins),
aggregated across all pages (follow `nextCursor` until `null`).

**Context**

CA-0001 required a concrete “last” selection algorithm (including tie-breaking) and a concrete
`thread/list` contract, rather than “likely use listing”.

**Chosen spec**

Pinned in `docs/specs/codex-app-server-jsonrpc-contract.md` under:
- `thread/list` → “Selection algorithm (pinned, deterministic)”

**Rationale**

- Avoids depending on unspecified server-side ordering semantics.
- Makes fake-server and integration tests deterministic and backend-agnostic.

**Implications**

- Implementations must page (or otherwise guarantee equivalent coverage) to avoid missing newer threads.

## CRD-0008 — Codex streaming resume requires a control + env-overrides entrypoint

**Decision**

`agent_api` Codex resume MUST use a control-capable `crates/codex` streaming entrypoint that:
- accepts per-run env overrides, and
- returns a termination handle (`ExecStreamControl.termination`).

Pinned API name/signature:
- `codex::CodexClient::stream_resume_with_env_overrides_control(request: codex::ResumeRequest, env_overrides: &BTreeMap<String, String>) -> Result<codex::ExecStreamControl, codex::ExecStreamError>`

**Context**

CA-0003 required pinning the concrete wrapper API + CLI prompt plumbing needed for deterministic
tests and to preserve `agent_api` cancellation/env semantics parity with `exec`.

**Chosen spec**

Pinned in:
- `docs/project_management/packs/active/agent-api-session-semantics/threading.md` → `SA-C05`
- `docs/project_management/packs/active/agent-api-session-semantics/seam-3-session-resume-extension-key.md`

**Rationale**

Existing `stream_resume` lacks a termination handle and per-run env overrides; `agent_api` requires
both to meet universal cancellation semantics and config precedence rules.

**Implications**

- A small additive API change is required in `crates/codex` before Codex resume can be shipped in `agent_api`.

## CRD-0009 — Codex app-server fork notification mapping is metadata-only and safety-first

**Decision**

For Codex fork via app-server (`thread/*` + `turn/start`), notification → `AgentWrapperEvent` mapping is pinned as:

- `agentMessage/delta` and reasoning delta notifications → `AgentWrapperEventKind::TextOutput` (bounded; split if needed).
- `item/started` → `AgentWrapperEventKind::ToolCall` (metadata-only).
- `item/completed` → `AgentWrapperEventKind::ToolResult` (metadata-only).
- `turn/started` / `turn/completed` → `AgentWrapperEventKind::Status` (safe message optional).
- `error` → `AgentWrapperEventKind::Error` with `message` derived from the bounded `error.message`.

Raw backend payloads MUST NOT be embedded in `AgentWrapperEvent.data`.

**Context**

CA-0001 required enumerating which app-server notifications must be mapped and defining the exact
mapping into universal event kinds/fields while preserving the Universal Agent API safety posture.

**Chosen spec**

Pinned in `docs/specs/codex-app-server-jsonrpc-contract.md` under:
- “Notifications → Universal Agent API events (pinned minimum)”

**Rationale**

- Matches the repo’s v1 safety posture (no raw backend lines/payloads in `data`).
- Keeps fork integration tests deterministic while still providing useful high-level event kinds.

**Implications**

- Tool details must be carried via the structured tools facet (`agent_api.tools.structured.v1`) in
  future work, not via raw payload embedding.
