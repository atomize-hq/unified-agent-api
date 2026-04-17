# C0 Spec - Packet Closeout + Runtime Lock

Source docs:
- `docs/project_management/next/cli-agent-onboarding-charter.md`
- `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`

## Decisions (no ambiguity)

- C0 is planning-only. It must not edit `crates/opencode/`, `cli_manifests/opencode/`, or
  `crates/agent_api/`.
- C0 confirms one already-locked decision: the canonical v1 OpenCode wrapper runtime surface is
  `opencode run --format json` unless contradictory evidence is found.
- Deferred surfaces that must remain explicit in the pack:
  - `opencode serve`
  - `opencode acp`
  - `opencode run --attach ...`
  - direct interactive CLI/stdout posture
- The C0 output must capture:
  - install paths and platform posture
  - auth/provider prerequisites
  - maintainer smoke evidence expectations
  - blocked unknowns
  - explicit deferred surfaces
- No downstream planning may assume a wrapper runtime surface until C0-integ is complete.

## Task Breakdown (no ambiguity)

- `C0-code`:
  - preserve the closed packet decisions inside this planning pack
  - state the exact v1 wrapper seam and why the deferred seams stay out of v1
- `C0-test`:
  - define the evidence checklist, reproducibility constraints, and blocker list for the chosen seam
- `C0-integ`:
  - reconcile the runtime-lock proposal and validation obligations into one execution-ready C0
    decision packet for C1

## Scope

- confirm the crate-first order remains authoritative
- preserve the locked runtime choice and turn it into execution-safe planning inputs
- define the minimum maintainer evidence required to move into wrapper planning
- state which OpenCode surfaces are out of scope for v1 even if they remain attractive later

## Acceptance Criteria

- this pack preserves one canonical OpenCode runtime surface for v1
- the pack names install/auth/provider prerequisites and maintainer smoke evidence required before
  implementation
- the pack lists deferred surfaces and why they are deferred
- C1 receives explicit locked inputs rather than packet-era candidate language

## Out of Scope

- editing any crate, manifest, CI workflow, or canonical spec outside this directory
- designing wrapper crate APIs in detail
- designing `agent_api` event mapping
- making UAA promotion decisions
