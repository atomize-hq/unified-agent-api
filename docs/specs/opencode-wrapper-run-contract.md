# OpenCode Wrapper Run Contract (v1)

Status: Normative  
Scope: canonical OpenCode v1 wrapper runtime surface

## Normative language

This document uses RFC 2119 requirement keywords (`MUST`, `MUST NOT`, `SHOULD`).

## Purpose

Define the one OpenCode CLI surface this repo may treat as the canonical v1 wrapper boundary.
This contract freezes the runtime seam before wrapper implementation, backend mapping, or UAA
promotion work proceeds.

## Normative references

- `docs/project_management/next/cli-agent-onboarding-charter.md`
- `docs/specs/unified-agent-api/run-protocol-spec.md`
- `docs/specs/unified-agent-api/extensions-spec.md`
- `docs/specs/opencode-onboarding-evidence-contract.md`

If there is a conflict between planning prose and this document, this contract wins for the v1
OpenCode wrapper surface.

## Canonical v1 surface

- The OpenCode v1 wrapper surface MUST be `opencode run --format json`.
- The v1 wrapper MUST treat that command as the only canonical prompt-driven runtime transport.
- The canonical transport MUST be headless and machine-parseable from the run itself; the wrapper
  MUST NOT require `serve`, `acp`, `run --attach`, or interactive TUI behavior to obtain
  structured events.
- Plain formatted stdout or stderr scraping MUST NOT be treated as the canonical wrapper transport.

## Accepted v1 controls

On the canonical `run --format json` surface, the v1 wrapper boundary MAY rely on:

- prompt input
- explicit model selection
- session reuse or continuation
- session fork
- explicit working-directory selection

These controls MUST remain on the same canonical surface. A design that requires shifting one of
them onto a helper surface reopens this contract.

## Deferred and excluded surfaces

The following surfaces are explicitly outside the v1 wrapper boundary:

- `opencode serve`
- `opencode acp`
- `opencode run --attach`
- direct interactive TUI usage
- share, web, import/export, or other session-management flows not required for the canonical
  prompt-driven run path

These surfaces MAY be documented or probed as evidence, but they MUST remain helper or
backend-specific until a later seam explicitly reopens the boundary.

## Wrapper-owned safety posture

- Multi-directory add-on behavior comparable to Codex `add_dirs` is out of scope for v1 and MUST
  fail closed until a later contract defines it.
- Timeout behavior MAY remain wrapper-owned; the absence of a native CLI timeout flag does not
  invalidate the canonical surface.
- The wrapper and downstream backends MUST NOT treat raw backend lines, provider secrets, or
  provider-specific diagnostics as canonical public API surface by default.

## Reopen triggers

This contract MUST be reopened if any of the following become true:

- `--format json` is no longer a stable machine-parseable event transport
- a helper surface becomes required to obtain structured events or completion
- stdout or stderr mixes human text and structured output in a way that prevents robust parsing
- accepted v1 controls fragment across incompatible transports
- a user or product requirement makes `serve`, `acp`, `run --attach`, or interactive TUI behavior
  mandatory for the first wrapper release

## Downstream handoff

- Later wrapper, backend, and promotion planning MUST treat this document as the canonical source
  for what the OpenCode wrapper is allowed to implement.
- Later work MAY add bounded detail that stays inside this surface, but it MUST NOT expand the
  surface without reopening this contract.
