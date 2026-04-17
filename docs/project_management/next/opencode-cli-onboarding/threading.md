# Threading - OpenCode CLI onboarding

This document makes the OpenCode onboarding control plane explicit: contracts, dependency edges,
workstreams, and revalidation triggers.

Ownership note: this repo's normative contract surfaces live under `docs/specs/**`. `SEAM-1` now
owns the initial OpenCode-specific canonical refs there, and downstream seam-local planning must
create or update the later seam-owned `docs/specs/**` artifacts before claiming those contracts are
passed.

## Execution horizon summary

- Active seam: `SEAM-3`
- Next seam: `SEAM-4`
- Future seams: none
- Only `SEAM-3` is eligible for authoritative downstream deep planning by default.
- `SEAM-4` may later receive seam-local review and slices, but any deeper work stays provisional
  until `SEAM-3` publishes its handoff.

## Contract registry

- **Contract ID**: `C-01`
  - **Type**: API
  - **Owner seam**: `SEAM-1`
  - **Direct consumers**: `SEAM-2`, `SEAM-3`, `SEAM-4`
  - **Derived consumers**: future OpenCode implementation and validation packs
  - **Thread IDs**: `THR-01`
  - **Definition**: the canonical v1 OpenCode wrapper seam is `opencode run --format json`,
    including headless request execution and line-delimited structured event output; helper surfaces
    such as `serve`, `acp`, `run --attach`, and direct interactive TUI mode remain explicitly
    deferred.
  - **Canonical contract ref**: `docs/specs/opencode-wrapper-run-contract.md`
  - **Versioning / compat**: v1 remains locked until a later seam explicitly reopens helper-surface
    scope with contradictory evidence or a new product need.

- **Contract ID**: `C-02`
  - **Type**: policy
  - **Owner seam**: `SEAM-1`
  - **Direct consumers**: `SEAM-2`, `SEAM-3`
  - **Derived consumers**: future smoke automation and fixture baselines
  - **Thread IDs**: `THR-01`
  - **Definition**: install paths, auth/provider prerequisites, maintainer smoke expectations,
    reproducibility constraints, and reopen triggers for the canonical v1 run surface must be
    explicit before downstream seams claim current input.
  - **Canonical contract ref**: `docs/specs/opencode-onboarding-evidence-contract.md`
  - **Versioning / compat**: evidence rules may become stricter, but they must continue to preserve
    fixture-first validation and fail-closed scope decisions.

- **Contract ID**: `C-03`
  - **Type**: event
  - **Owner seam**: `SEAM-2`
  - **Direct consumers**: `SEAM-3`, `SEAM-4`
  - **Derived consumers**: future wrapper parity and regression work
  - **Thread IDs**: `THR-02`
  - **Definition**: `crates/opencode/` owns the bounded spawn surface, typed event taxonomy,
    completion-finality handoff, offline parsing posture, and redaction boundary for the canonical
    OpenCode v1 run flow.
  - **Canonical contract ref**: `docs/specs/opencode-wrapper-run-contract.md`
  - **Versioning / compat**: downstream seams may extend mapping detail but must not invent wrapper
    semantics that bypass the wrapper-owned contract.

- **Contract ID**: `C-04`
  - **Type**: state
  - **Owner seam**: `SEAM-2`
  - **Direct consumers**: `SEAM-3`
  - **Derived consumers**: future manifest validators, release review, and support publication work
  - **Thread IDs**: `THR-02`
  - **Definition**: `cli_manifests/opencode/` owns the OpenCode artifact inventory, pointer/update
    rules, version metadata posture, and parity evidence layout needed to keep wrapper support
    auditable.
  - **Canonical contract ref**: `docs/specs/opencode-cli-manifest-contract.md`
  - **Versioning / compat**: future artifact additions must remain additive and compatible with the
    repo's existing manifest-evidence model.

- **Contract ID**: `C-05`
  - **Type**: schema
  - **Owner seam**: `SEAM-3`
  - **Direct consumers**: `SEAM-4`
  - **Derived consumers**: future OpenCode backend tests and public backend selection surfaces
  - **Thread IDs**: `THR-03`
  - **Definition**: the OpenCode backend maps wrapper-owned events and completions into the universal
    envelope without raw-line leakage, preserves completion finality, and advertises only the
    capability ids supported by the concrete backend behavior.
  - **Canonical contract ref**: `docs/specs/opencode-agent-api-backend-contract.md`
  - **Versioning / compat**: universal capability ids and envelopes remain governed by
    `docs/specs/unified-agent-api/**`; backend-specific mapping detail may evolve only inside that
    envelope.

- **Contract ID**: `C-06`
  - **Type**: config
  - **Owner seam**: `SEAM-3`
  - **Direct consumers**: `SEAM-4`
  - **Derived consumers**: future extension docs and capability-matrix review
  - **Thread IDs**: `THR-03`
  - **Definition**: backend-specific extension keys for OpenCode stay under `backend.opencode.*`
    until promotion is justified, and universal extension keys are accepted only when the backend
    can deterministically honor the owner-doc semantics.
  - **Canonical contract ref**: `docs/specs/unified-agent-api/extensions-spec.md`
  - **Versioning / compat**: any movement from backend-specific to universal ownership must satisfy
    the canonical promotion and registry rules.

- **Contract ID**: `C-07`
  - **Type**: conformance
  - **Owner seam**: `SEAM-4`
  - **Direct consumers**: pack closeout and future follow-on packs
  - **Derived consumers**: capability-matrix and canonical-spec maintenance
  - **Thread IDs**: `THR-04`
  - **Definition**: the promotion seam owns the explicit recommendation for what remains
    backend-specific, what is eligible for UAA promotion, and which canonical surfaces require a
    separate follow-on execution pack.
  - **Canonical contract ref**: `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - **Versioning / compat**: promotion decisions must preserve the built-in multi-backend rule and
    keep backend-only support visible when universal promotion is not justified.

## Thread registry

- **Thread ID**: `THR-01`
  - **Producer seam**: `SEAM-1`
  - **Consumer seam(s)**: `SEAM-2`, `SEAM-3`, `SEAM-4`
  - **Carried contract IDs**: `C-01`, `C-02`
  - **Purpose**: lock the canonical OpenCode v1 run surface and its evidence/reproducibility
    posture before wrapper or backend planning freezes downstream assumptions.
  - **State**: `revalidated`
  - **Revalidation trigger**: contradictory maintainer smoke, upstream CLI event-shape drift, or a
    decision to pull `serve`, `acp`, `run --attach`, or interactive TUI behavior into v1 scope
  - **Satisfied by**: the `SEAM-1` closeout record plus the canonical runtime and evidence
    contracts under `docs/specs/opencode-*.md`
  - **Notes**: `SEAM-2` has now revalidated against this thread; downstream seams should treat the
    published runtime/evidence handoff as current input instead of packet prose.

- **Thread ID**: `THR-02`
  - **Producer seam**: `SEAM-2`
  - **Consumer seam(s)**: `SEAM-3`
  - **Carried contract IDs**: `C-03`, `C-04`
  - **Purpose**: hand the backend seam one wrapper-owned event/completion contract plus one
    manifest-root artifact contract, so backend work stays consumer-shaped instead of redefining the
    wrapper.
  - **State**: `revalidated`
  - **Revalidation trigger**: any change in wrapper event taxonomy, completion semantics, manifest
    artifact inventory, or fixture/fake-binary strategy
  - **Satisfied by**: the `SEAM-2` closeout record plus the published wrapper and manifest
    contracts under `docs/specs/opencode-*.md`
  - **Notes**: `SEAM-3` has now revalidated against this thread; downstream backend planning should
    treat the published wrapper/manifest handoff as current input instead of provisional seam
    prose.

- **Thread ID**: `THR-03`
  - **Producer seam**: `SEAM-3`
  - **Consumer seam(s)**: `SEAM-4`
  - **Carried contract IDs**: `C-05`, `C-06`
  - **Purpose**: expose the actual OpenCode backend envelope, advertised capabilities, and
    extension-ownership posture for promotion review.
  - **State**: `identified`
  - **Revalidation trigger**: any change to wrapper contract inputs, capability advertisement, or
    backend-specific extension handling
  - **Satisfied by**: future seam-local planning and closeout for `SEAM-3`
  - **Notes**: this thread keeps the promotion seam from guessing about backend behavior.

- **Thread ID**: `THR-04`
  - **Producer seam**: `SEAM-4`
  - **Consumer seam(s)**: pack closeout and future follow-on packs
  - **Carried contract IDs**: `C-07`
  - **Purpose**: publish the authoritative backend-support versus UAA-promotion recommendation and
    name any required follow-on pack for canonical spec or matrix work.
  - **State**: `identified`
  - **Revalidation trigger**: new multi-backend evidence, spec-registry changes, or additional
    OpenCode behavior that meaningfully changes the promotion case
  - **Satisfied by**: future seam-local planning and closeout for `SEAM-4`
  - **Notes**: this thread must remain explicit even when the correct answer is "no promotion yet."

## Dependency graph

- `SEAM-1 -> SEAM-2`: wrapper and manifest planning must consume one explicit runtime/evidence
  contract instead of packet prose.
- `SEAM-2 -> SEAM-3`: backend planning must consume wrapper-owned event/completion and manifest
  inventory contracts rather than invent new wrapper semantics.
- `SEAM-3 -> SEAM-4`: promotion review is valid only once backend behavior, capability
  advertisement, and extension ownership are concrete.

## Critical path

`SEAM-1 (runtime surface + evidence lock)` -> `SEAM-2 (wrapper + manifest foundation)` ->
`SEAM-3 (agent_api backend mapping)` -> `SEAM-4 (UAA promotion review)`

## Workstreams

- `WS-RUNTIME-CONTRACT`: `SEAM-1`
- `WS-WRAPPER-MANIFEST`: `SEAM-2`
- `WS-BACKEND-MAPPING`: `SEAM-3`
- `WS-PROMOTION-REVIEW`: `SEAM-4`
