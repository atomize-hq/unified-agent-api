# OpenCode Onboarding Evidence Contract (v1)

Status: Normative  
Scope: prerequisite, smoke, replay, and reopen rules for treating the OpenCode v1 runtime surface
as current input

## Normative language

This document uses RFC 2119 requirement keywords (`MUST`, `MUST NOT`, `SHOULD`).

## Purpose

Define the evidence posture required to trust the canonical OpenCode v1 runtime surface without
confusing live maintainer smoke, provider-specific behavior, and later support-publication
requirements.

## Normative references

- `docs/project_management/next/cli-agent-onboarding-charter.md`
- `docs/specs/opencode-wrapper-run-contract.md`
- `docs/specs/unified-agent-api/run-protocol-spec.md`

## Preconditions

Before downstream seams may treat the OpenCode v1 basis as current, the evidence posture MUST make
all of these explicit:

- at least one supported install path
- provider or account prerequisites needed to run a real authenticated prompt on the canonical
  surface
- how the maintainer confirms the canonical runtime surface exists and can route at least one
  model-backed prompt successfully
- which later evidence must be committed so wrapper and backend support do not depend on a live
  provider account

The packet-level evaluation recipe remains admissible as baseline evidence for this seam, but only
if it is grounded in the canonical `opencode run --format json` path and the provider/auth/model
setup needed to exercise that path.

## Baseline prerequisite record

The current accepted planning basis for this seam MUST remain pinned to the following baseline
record unless a reopen trigger fires:

- install path used for the accepted basis: `npm install -g opencode-ai`
- auth prerequisite used for the accepted basis: configure provider-backed credentials via
  `opencode auth login`
- model-routing confirmation path for the accepted basis:
  - run `opencode models` or `opencode models --refresh` successfully so the chosen provider/model
    inventory is visible
  - confirm the canonical runtime surface on `opencode/gpt-5-nano`
- maintainer smoke path for the accepted basis:
  - `opencode run --format json -m opencode/gpt-5-nano "Reply with the word OK."`
  - `opencode run --format json -m opencode/gpt-5-nano "Summarize this repository structure in 5 bullets."`
  - `opencode run --format json -m opencode/gpt-5-nano --session <session_id> "Reply with CONTINUED."`
  - `opencode run --format json -m opencode/gpt-5-nano --session <session_id> --fork "Reply with FORKED."`
  - `opencode run --format json -m opencode/gpt-5-nano --dir . "Reply with DIR_OK."`

Downstream seams MUST use this baseline prerequisite record as the self-sufficient canonical source
for install, auth, model-routing, and smoke-path facts. Packet prose MAY provide supporting
context, but it MUST NOT be required to recover these baseline prerequisites.

## Planning-grade maintainer smoke

The initial seam lock MAY rely on authenticated maintainer smoke, but that smoke MUST demonstrate
all of the following on `opencode run --format json`:

- structured machine-parseable events emitted directly from the run
- a distinct completion marker or completion handoff after streamed output
- at least one non-trivial prompt against the local repository
- at least one successful model-routed run on the canonical surface
- session reuse or continuation on the canonical surface
- session fork on the canonical surface
- explicit working-directory control on the canonical surface

This live smoke is planning-grade only. It proves the canonical run path is credible now, but it
does not by itself prove wrapper support is reproducible without a live provider account.

Helper-surface probes for `serve` or `acp` MAY be recorded only to support their deferred
classification. They MUST NOT redefine the canonical wrapper boundary.

## Publication-grade deterministic evidence

Live maintainer smoke is sufficient to lock the planning seam, but it is not sufficient by itself
to claim long-term wrapper or backend support.

Before later seams publish support, the repo MUST also carry deterministic evidence that does not
depend on an active provider-backed account. That later evidence MUST include:

- committed transcript or protocol evidence for `run --format json`
- a replay or fake-binary strategy for deterministic tests that can exercise the canonical surface
  without a live provider account
- committed wrapper-coverage evidence for the help and flag surface

These evidence classes are separate and complementary:

- wrapper coverage proves the CLI surface shape
- transcript or protocol evidence proves the canonical runtime exchange
- replay or fake-binary evidence proves the support claim can be tested deterministically later

Later seams MAY choose the exact artifact layout, but they MUST preserve this distinction:

- live smoke proves the runtime choice is credible now
- committed replay evidence proves the support claim is reproducible later

## Failure interpretation

- Provider-specific failures that occur outside the chosen canonical success path MUST be treated as
  evidence posture, not as implicit wrapper semantics.
- A provider or auth failure invalidates the current basis only when it shows that the canonical
  `run --format json` path is no longer a credible way to obtain the required planning-grade smoke.
- Failures on helper surfaces do not, by themselves, widen v1 scope.
- A missing helper-surface success path does not imply a wrapper defect if the canonical run path
  still satisfies the planning-grade smoke checklist.

## Reopen triggers

This contract MUST be reopened if any of the following become true:

- provider or auth posture changes invalidate the baseline prerequisite record or observed
  canonical success path
- the canonical run surface stops emitting the structured events needed for the planning-grade
  smoke checklist
- later support work tries to rely on live smoke alone instead of committed replay evidence
- helper surfaces become necessary to satisfy the planning-grade smoke checklist
- new evidence shows that the deferred-surface policy or deterministic replay posture is ambiguous
- the separation between planning-grade smoke and publication-grade replay is no longer clear

## Downstream obligations

- Later wrapper, backend, and promotion planning MUST cite this document when deciding whether the
  OpenCode basis is still current.
- If any reopen trigger fires, downstream work MUST stop and reopen the runtime and evidence
  contract rather than normalizing the drift locally.

## Baseline verification checklist

Before this contract is treated as current input, the repo SHOULD confirm:

- one supported install path is named
- provider or account prerequisites are named
- the maintainer smoke path runs on `opencode run --format json`
- model routing is exercised on the canonical surface
- the live-smoke boundary is distinct from later committed replay evidence
- helper-surface probes, if recorded, are only evidence for deferred classification
