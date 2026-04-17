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

## Event and completion boundary

The backend MUST preserve the wrapper-owned event and completion semantics while exposing only the
bounded Unified Agent API surface.

- The backend MUST emit only backend-owned event kinds and completion state that fit the universal
  event/completion envelope rules.
- The backend MUST preserve completion finality from the wrapper contract.
- The backend MUST NOT expose raw backend stdout, stderr, or provider-specific diagnostics as a
  public event or completion payload.
- The backend MAY retain raw backend output internally for debugging, replay, or evidence purposes
  only if that retention stays outside the public Unified Agent API surface and remains consistent
  with the wrapper and evidence contracts.

## Bounded payload and redaction baseline

The backend MUST keep public payloads bounded and redacted.

- Any backend-owned event or completion data MUST stay within the bounds defined by the universal
  event-envelope contract and related Unified Agent API rules.
- The backend MUST redact provider secrets, raw backend lines, and provider-specific diagnostics
  before anything reaches the public backend surface.
- The backend MUST treat redaction as a backend-owned responsibility for its own mapping layer; it
  MUST NOT rely on downstream consumers to scrub backend-private content.
- If a backend condition cannot be represented safely, the backend MUST fail closed rather than
  widening the public payload shape.

## Capability posture

The backend MUST advertise only the capabilities it can deterministically honor for the exposed
OpenCode backend behavior.

- The backend MUST fail closed on unsupported capability requests and MUST NOT over-advertise
  support.
- Capability support MUST remain aligned with the concrete backend behavior and the universal
  capability registry.
- This document does not define new universal capability ids.

## Backend-specific extension ownership

The backend MUST keep OpenCode-specific backend extensions under the backend-owned namespace.

- Any backend-specific extension keys introduced for OpenCode MUST remain under
  `backend.opencode.*`.
- The backend MUST treat unsupported backend-specific extension keys as fail-closed inputs.
- The backend MUST NOT promote backend-specific extension semantics into universal extension keys
  unless a later seam explicitly justifies that change in the canonical specs.
- This document does not invent concrete backend-specific keys; it only establishes the ownership
  boundary and the fail-closed posture.

## Scope guards and non-goals

This contract intentionally does not:

- change the canonical OpenCode wrapper transport surface
- define helper-surface behavior outside the canonical wrapper contract
- widen universal capability ids or extension semantics
- publish backend promotion decisions
- define fixture, replay, or fake-binary policy beyond the evidence posture already owned by the
  OpenCode evidence contract
- move wrapper-private parsing, normalization, or redaction duties out of the wrapper seam

## Baseline verification checklist

Before this contract is treated as settled, the repo SHOULD confirm:

- the contract stays aligned with the canonical wrapper run contract and the OpenCode manifest
  contract
- request mapping is described only at the backend boundary and does not reopen wrapper semantics
- event and completion handling stays bounded and redacted
- backend-specific extension ownership remains under `backend.opencode.*`
- unsupported capability or extension requests fail closed
- the contract is concrete enough for later backend mapping and capability review work without
  depending on planning prose
