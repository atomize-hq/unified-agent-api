# OpenCode Wrapper Run Contract (v1)

Status: Normative  
Scope: canonical OpenCode v1 wrapper runtime surface

## Normative language

This document uses RFC 2119 requirement keywords (`MUST`, `MUST NOT`, `SHOULD`).

## Purpose

Define the one OpenCode CLI surface this repo may treat as the canonical v1 wrapper boundary.
This contract freezes the runtime seam before wrapper implementation, backend mapping, or UAA
promotion work proceeds. The wrapper owns the spawn boundary, event normalization, completion
finality handling, parser behavior, and redaction behavior for that surface.

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
  MUST NOT require `serve`, `acp`, `run --attach`, direct interactive TUI behavior, or any other
  helper/session-management surface to obtain structured events or completion.
- Plain formatted stdout or stderr scraping MUST NOT be treated as the canonical wrapper transport.

## Wrapper-owned runtime boundaries

The wrapper implementation that consumes this contract MUST own the following responsibilities:

- process spawn and lifecycle management for the canonical `opencode run --format json` flow
- normalization of streamed output into typed wrapper events
- completion-finality detection and handoff
- parsing of the canonical structured output stream
- redaction of provider secrets and provider-specific diagnostics before any public wrapper surface

Backend-specific lines, debug text, and provider diagnostics MAY be captured as evidence, but they
MUST NOT be treated as the canonical wrapper API surface.

## Accepted v1 controls

On the canonical `run --format json` surface, the v1 wrapper boundary MAY rely on only the
following controls:

- prompt input
- explicit model selection via `--model`
- session reuse or continuation via `--session` and `--continue`
- session fork via `--fork`
- explicit working-directory selection via `--dir`

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
- any other helper surface that is needed only to recover structured events, completion, or run
  controls already covered by the canonical `run --format json` transport

These surfaces MAY be documented or probed as evidence, but they MUST remain helper or
backend-specific until a later seam explicitly reopens the boundary.

## Wrapper-owned safety posture

- Multi-directory add-on behavior comparable to Codex `add_dirs` is out of scope for v1 and MUST
  fail closed until a later contract defines it.
- Timeout behavior MAY remain wrapper-owned; the absence of a native CLI timeout flag does not
  invalidate the canonical surface.
- The wrapper and downstream backends MUST NOT treat raw backend lines, provider secrets, or
  provider-specific diagnostics as canonical public API surface by default.
- The wrapper MUST keep event typing, completion ownership, parsing, and redaction inside the
  wrapper seam rather than delegating those semantics to a backend seam.
- Any support claim that depends on helper-surface recovery of structured events or completion
  reopens this contract.

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
- Later seams MAY consume the wrapper-owned runtime boundary described here, but they MUST NOT
  invent new canonical event, completion, parser, or redaction semantics.

## Baseline verification checklist

Before a downstream seam treats this contract as settled, the repo SHOULD confirm:

- the only canonical v1 transport is `opencode run --format json`
- the accepted controls stay on that same surface
- `serve`, `acp`, `run --attach`, and interactive TUI remain deferred
- helper-surface probes are evidence only, not canonical transport
- raw stdout or stderr scraping is not treated as the wrapper boundary
