# OpenCode Agent API Backend Contract (v1)

Status: Normative  
Scope: backend-owned mapping baseline for the OpenCode backend surface inside Unified Agent API

## Normative language

This document uses RFC 2119 requirement keywords (`MUST`, `MUST NOT`, `SHOULD`).

## Purpose

Define the backend-owned contract baseline for mapping the canonical OpenCode wrapper surface into
the Unified Agent API backend envelope without leaking wrapper-private, provider-private, or raw
backend details into the public API.

This document is deliberately backend-owned and backend-bounded. It does not redefine the wrapper
runtime seam, widen universal capability semantics, or publish promotion decisions for other
backends.

## Normative references

- `docs/specs/opencode-wrapper-run-contract.md`
- `docs/specs/opencode-cli-manifest-contract.md`
- `docs/specs/opencode-onboarding-evidence-contract.md`
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/specs/unified-agent-api/extensions-spec.md`
- `docs/specs/unified-agent-api/run-protocol-spec.md`
- `docs/specs/unified-agent-api/event-envelope-schema-spec.md`

If this document conflicts with the wrapper contract or the universal Unified Agent API specs, the
wrapper and universal contracts control their respective domains.

## Backend-owned mapping boundary

The OpenCode backend MUST treat the canonical wrapper contract as the input boundary for backend
mapping work.

- The backend MUST consume the wrapper-owned request, event, and completion semantics described by
  `docs/specs/opencode-wrapper-run-contract.md`.
- The backend MUST translate those wrapper-owned inputs into the Unified Agent API backend envelope
  without inventing new wrapper behavior or bypassing wrapper-owned normalization rules.
- The backend MUST keep request mapping consumer-shaped: the backend contract may describe how the
  wrapper surface is mapped, but it MUST NOT broaden the wrapper surface or require helper surfaces
  that the wrapper contract excludes.
- The backend MUST fail closed if a requested input cannot be represented within the accepted
  backend mapping for the canonical wrapper path.

## Request mapping

The backend MUST map the wrapper-owned request into the backend envelope without introducing new
request semantics.

- The backend MUST treat `AgentWrapperRunRequest.prompt` as the required user prompt input for the
  canonical OpenCode run path.
- The backend MUST treat `AgentWrapperRunRequest.working_dir`, `timeout`, and `env` as backend-owned
  request context inputs only to the extent they are already accepted by the wrapper contract and
  can be represented without widening the public surface.
- The backend MUST consume wrapper-owned extension inputs only after the universal request
  validation path and backend-owned normalization have accepted them.
- The backend MUST NOT reinterpret a rejected or absent wrapper-owned input as a different backend
  capability.
- If an accepted request input has no deterministic representation in the backend mapping for the
  canonical wrapper path, the backend MUST fail closed rather than synthesizing alternate wrapper
  behavior.

## Event and completion boundary

The backend MUST preserve the wrapper-owned event and completion semantics while exposing only the
bounded Unified Agent API surface.

- The backend MUST map wrapper-owned structured output into stable `AgentWrapperEventKind` values
  rather than exposing wrapper-private event types directly.
- The backend MUST map assistant text output and streamed textual deltas to `TextOutput` events.
- The backend MUST map tool lifecycle notifications to `ToolCall` and `ToolResult` events only when
  the mapped payload is metadata-only and bounded.
- The backend MUST map wrapper-owned run lifecycle transitions and safe terminal notices to
  `Status` events.
- The backend MUST map unsupported or unclassifiable backend notifications to `Unknown` or a safe
  bounded status/error form, never to raw payload passthrough.
- The backend MUST preserve completion finality from the wrapper contract.
- The backend MUST keep `AgentWrapperCompletion.final_text` optional and deterministic; it MAY be
  populated only when the backend can derive a final assistant text response safely from the
  supported flow.
- The backend MUST NOT expose raw backend stdout, stderr, or provider-specific diagnostics as a
  public event or completion payload.
- The backend MAY retain raw backend output internally for debugging or evidence purposes only if
  that retention stays outside the public Unified Agent API surface and remains consistent with
  the wrapper and evidence contracts.

## Completion mapping

The backend MUST map completion results as a bounded, redacted summary of the supported wrapper run.

- The backend MUST surface the underlying run status in `AgentWrapperCompletion.status`.
- The backend MUST populate `AgentWrapperCompletion.final_text` only when the supported wrapper flow
  deterministically yields a final assistant message text that can be extracted without parsing raw
  backend lines into public API surface.
- The backend MUST leave `AgentWrapperCompletion.final_text` as `None` when no such deterministic
  text is available.
- The backend MUST keep `AgentWrapperCompletion.data` bounded, redacted, and metadata-only.
- The backend MUST NOT copy raw stdout, stderr, provider diagnostics, or unbounded transcript
  content into completion data.

## Bounded payload and redaction baseline

The backend MUST keep public payloads bounded and redacted.

- Any backend-owned event or completion data MUST stay within the bounds defined by the universal
  event-envelope contract and related Unified Agent API rules.
- The backend MUST redact provider secrets, raw backend lines, and provider-specific diagnostics
  before anything reaches the public backend surface.
- The backend MUST treat redaction as a backend-owned responsibility for its own mapping layer; it
  MUST NOT rely on downstream consumers to scrub backend-private content.
- Event `data` and completion `data` MAY carry only bounded metadata needed to describe the mapped
  backend state; they MUST NOT carry raw transcript lines, raw provider diagnostics, or other
  wrapper-private content.
- If a backend condition cannot be represented safely, the backend MUST fail closed rather than
  widening the public payload shape.

## Accepted runtime failure translation

When the backend has accepted a request input and later discovers at runtime that the canonical
wrapper path cannot honor it, the backend MUST translate that failure into the universal backend
error model.

- The backend MUST surface the failure as `AgentWrapperError::Backend { message }`.
- The backend MUST use a safe, redacted `message` that does not embed raw backend stdout, stderr, or
  provider-specific diagnostics.
- If the backend has already returned an `AgentWrapperRunHandle` and the consumer-visible events
  stream is still open, the backend MUST emit exactly one terminal `AgentWrapperEventKind::Error`
  event carrying the same safe, redacted `message`.
- The backend MUST close the stream after emitting that terminal error event.
- The backend MUST apply this translation only to accepted inputs that later become impossible to
  honor at runtime; malformed or unsupported inputs remain governed by the earlier fail-closed
  request validation path.
- Runtime rejection on the canonical wrapper path MUST NOT be converted into a new wrapper behavior
  or a widened public event shape.

## Validation and evidence posture

The backend MUST keep its validation posture fixture-first and reproducible by default.

- Backend validation MUST be expressible without requiring a live provider account by default.
- Replay and fake-binary paths MAY be used for deterministic validation support, but they MUST
  remain validation-only support paths and MUST NOT be described as new runtime transports or as a
  widening of supported wrapper behavior.
- The backend MUST keep redaction and bounded-payload obligations reviewable through these
  validation paths without relying on raw backend lines, provider-specific diagnostics, or public
  transcript passthrough.
- The backend MUST preserve the distinction between live runtime behavior and deterministic
  validation evidence; support claims MUST continue to depend on the canonical wrapper contract and
  published evidence posture, not on ad hoc local replay assumptions.
- If validation posture, replay posture, or redaction posture becomes ambiguous or conflicts with
  the wrapper, manifest, or evidence contracts, the backend MUST stop and reopen the upstream
  seam rather than normalizing the drift locally.
- The backend MUST treat drift in wrapper event or completion semantics, manifest inventory or
  pointer rules, capability or extension registry rules, or payload redaction boundaries as a
  reopen trigger for upstream seam work.

## Capability posture

The backend MUST advertise only the capabilities it can deterministically honor for the canonical
`opencode run --format json` surface and its wrapper-controlled inputs.

In v1, the backend MUST advertise the following universal capability ids when the implementation
can honor them end-to-end:

- `agent_api.run`
- `agent_api.events`
- `agent_api.events.live`
- `agent_api.config.model.v1`
- `agent_api.session.resume.v1`
- `agent_api.session.fork.v1`

The last three ids are grounded in the canonical wrapper controls already accepted by the
OpenCode v1 surface: `--model`, `--session` / `--continue`, and `--fork`.

The backend MUST NOT advertise `agent_api.exec.add_dirs.v1` in v1 because the canonical wrapper
contract explicitly marks multi-directory add-on behavior as out of scope and fail-closed.

The backend MUST fail closed on unsupported capability requests and MUST NOT over-advertise
support. Any universal capability id not listed above remains unclaimed by this contract revision
until a later seam adds concrete backend evidence for it.

## Backend-specific extension ownership

The backend MUST keep OpenCode-specific backend extensions under the backend-owned namespace.

- Any backend-specific extension keys introduced for OpenCode MUST remain under
  `backend.opencode.*`.
- This contract does not define any concrete `backend.opencode.*` keys in v1; the namespace is
  reserved for later backend-owned extension work only if a later seam explicitly justifies it.
- The backend MUST treat unsupported backend-specific extension keys as fail-closed inputs.
- If a request includes a key under `backend.opencode.*` that this backend does not recognize, the
  backend MUST fail closed with `AgentWrapperError::UnsupportedCapability` before spawn.
- The backend MUST NOT promote backend-specific extension semantics into universal extension keys
  unless a later seam explicitly justifies that change in the canonical specs.
- This document does not invent concrete backend-specific keys; it only establishes the ownership
  boundary and the fail-closed posture.

## Scope guards and non-goals

This contract intentionally does not:

- change the canonical OpenCode wrapper transport surface
- define helper-surface behavior outside the canonical wrapper contract
- widen universal capability ids or extension semantics
- claim support for any backend-specific extension key not explicitly defined here
- publish backend promotion decisions
- define evidence-policy details beyond the evidence posture already owned by the OpenCode
  evidence contract
- move wrapper-private parsing, normalization, or redaction duties out of the wrapper seam
- introduce any planning identifiers or seam-local process markers into canonical spec text

## Baseline verification checklist

Before this contract is treated as settled, the repo SHOULD confirm:

- the contract stays aligned with the canonical wrapper run contract and the OpenCode manifest
  contract
- request mapping is described only at the backend boundary and does not reopen wrapper semantics
- event mapping covers stable kinds, bounded metadata, and no raw backend payload passthrough
- completion mapping preserves finality while keeping `final_text` optional and deterministic
- accepted runtime failures translate to safe `AgentWrapperError::Backend` messages and terminal
  `Error` events when a stream is open
- public payloads stay bounded and redacted
- only the capabilities listed above are claimed for v1 backend advertisement
- backend-specific extension ownership remains under `backend.opencode.*`
