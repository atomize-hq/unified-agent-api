# Threading - CLI manifest support matrix

This document makes the support-matrix control plane explicit: contracts, dependency edges, workstreams, and revalidation triggers.

Ownership note: this repo's normative contract surfaces live under `docs/specs/**`. If any planning artifact in this pack conflicts with the normative specs or committed manifest evidence, the repo-owned spec/evidence surfaces win.

## Execution horizon summary

- Active seam: none
- Next seam: none
- Future seams: none
- `SEAM-2` has landed and closed.
- `SEAM-3` has landed and closed.
- `SEAM-4` has landed and closed because validator enforcement consumed the published row model from `SEAM-3` and closed out the contradiction contract.
- `SEAM-5` has now landed and closed because fixture and golden conformance consumed the landed contradiction rules from `SEAM-4` and published the future-agent neutrality handoff.

## Contract registry

- **Contract ID**: `C-01`
  - **Type**: schema
  - **Owner seam**: `SEAM-1`
  - **Direct consumers**: `SEAM-2`, `SEAM-3`, `SEAM-4`, `SEAM-5`
  - **Derived consumers**: pack closeout and future manifest-onboarding work
  - **Thread IDs**: `THR-01`
  - **Definition**: support publication is target-scoped first and exposes four distinct layers per row: manifest support, backend-crate support, UAA unified support, and passthrough visibility. `versions/<version>.json.status` remains workflow-stage metadata rather than published support truth.
  - **Canonical contract ref**: `docs/specs/unified-agent-api/support-matrix.md`
  - **Versioning / compat**: phase-1 stable once SEAM-1 lands; downstream seams must treat field and layer naming as locked input.

- **Contract ID**: `C-02`
  - **Type**: integration
  - **Owner seam**: `SEAM-2`
  - **Direct consumers**: `SEAM-3`, `SEAM-4`, `SEAM-5`
  - **Derived consumers**: future agent-manifest onboarding work
  - **Thread IDs**: `THR-02`
  - **Definition**: shared wrapper-coverage normalization lives in one neutral module with thin Codex and Claude adapters. The neutral seam owns common normalization rules; per-agent modules own only root-specific loading and defaults.
  - **Canonical contract ref**: `docs/specs/codex-wrapper-coverage-generator-contract.md`
  - **Versioning / compat**: additive extraction only; the shared seam must preserve current agent behavior while becoming reusable.

- **Contract ID**: `C-03`
  - **Type**: config
  - **Owner seam**: `SEAM-2`
  - **Direct consumers**: `SEAM-3`, `SEAM-4`, `SEAM-5`
  - **Derived consumers**: future support-matrix validators and future agent roots
  - **Thread IDs**: `THR-02`
  - **Definition**: the support-matrix pipeline loads committed evidence from each agent root using versions, current pointers, latest pointers, and coverage reports rather than inventing a second evidence store.
  - **Canonical contract ref**: `docs/specs/unified-agent-api/support-matrix.md`
  - **Versioning / compat**: future agents may add roots, but the root intake contract must stay shape-driven rather than agent-name-driven.

- **Contract ID**: `C-04`
  - **Type**: state
  - **Owner seam**: `SEAM-3`
  - **Direct consumers**: `SEAM-4`, `SEAM-5`
  - **Derived consumers**: generated JSON and Markdown publication
  - **Thread IDs**: `THR-03`
  - **Definition**: support rows derive in a single pass with explicit row fields for agent, version, target, manifest support state, backend support state, UAA support state, pointer promotion state, and evidence notes for intentional partials.
  - **Canonical contract ref**: `docs/specs/unified-agent-api/support-matrix.md`
  - **Versioning / compat**: renderer and validator surfaces must consume the same derived row model; no re-derivation divergence allowed.

- **Contract ID**: `C-05`
  - **Type**: UX affordance
  - **Owner seam**: `SEAM-3`
  - **Direct consumers**: `SEAM-4`, `SEAM-5`
  - **Derived consumers**: maintainers reading published support docs
  - **Thread IDs**: `THR-03`
  - **Definition**: `docs/specs/unified-agent-api/support-matrix.md` is a Markdown projection of the same derived JSON model. It is not a second truth source and must fail staleness checks when it drifts from the JSON artifact.
  - **Canonical contract ref**: `docs/specs/unified-agent-api/support-matrix.md`
  - **Versioning / compat**: projection layout may evolve, but it must remain a deterministic render from the shared row model.

- **Contract ID**: `C-06`
  - **Type**: policy
  - **Owner seam**: `SEAM-4`
  - **Direct consumers**: `SEAM-5`
  - **Derived consumers**: repo gate and release workflow
  - **Thread IDs**: `THR-04`
  - **Definition**: pointer promotion state, `versions/<version>.json.status`, and published support rows must not silently disagree. Contradictions fail deterministically before stale Markdown or incorrect support claims land.
  - **Canonical contract ref**: `docs/specs/unified-agent-api/support-matrix.md`
  - **Versioning / compat**: exact contradiction behavior becomes part of the validator contract once SEAM-4 lands.

- **Contract ID**: `C-07`
  - **Type**: conformance
  - **Owner seam**: `SEAM-5`
  - **Direct consumers**: none
  - **Derived consumers**: future agent-onboarding work and drift-guard maintenance
  - **Thread IDs**: `THR-05`
  - **Definition**: the neutral support-matrix seam must prove it handles Codex fixtures, Claude fixtures, and at least one synthetic third-agent-shaped fixture without agent-name-specific branching in the shared core.
  - **Canonical contract ref**: `docs/specs/unified-agent-api/support-matrix.md`
  - **Versioning / compat**: fixture coverage must expand with future agents without weakening neutrality guarantees.

## Thread registry

- **Thread ID**: `THR-01`
  - **Producer seam**: `SEAM-1`
  - **Consumer seam(s)**: `SEAM-2`, `SEAM-3`, `SEAM-4`, `SEAM-5`
  - **Carried contract IDs**: `C-01`
  - **Purpose**: lock naming, authority, publication targets, and layer semantics before downstream implementation freezes output shapes.
  - **State**: `revalidated`
  - **Revalidation trigger**: any change to support-layer vocabulary, canonical output locations, or the published meaning of `validated` vs `supported`
  - **Satisfied by**: landed spec text in `docs/specs/unified-agent-api/support-matrix.md`, aligned manifest docs, neutral `xtask support-matrix` command wiring, and the `SEAM-1` closeout record
  - **Notes**: `SEAM-2` and `SEAM-3` have now revalidated against this thread; downstream seams should treat the support-layer names, publication targets, and `validated` versus `supported` meaning as current inputs.

- **Thread ID**: `THR-02`
  - **Producer seam**: `SEAM-2`
  - **Consumer seam(s)**: `SEAM-3`, `SEAM-4`, `SEAM-5`
  - **Carried contract IDs**: `C-02`, `C-03`
  - **Purpose**: hand off one neutral normalization + root-intake seam so publication and validator layers do not duplicate Codex/Claude logic.
  - **State**: `revalidated`
  - **Revalidation trigger**: any shift in root file layout, coverage report shape, or extraction boundaries between shared and per-agent modules
  - **Satisfied by**: shared normalization module plus thin adapters with fixture coverage against current Codex and Claude roots
  - **Notes**: this thread is now current input for the active seam and the main handoff surface for `SEAM-3`, `SEAM-4`, and `SEAM-5`.

- **Thread ID**: `THR-03`
  - **Producer seam**: `SEAM-3`
  - **Consumer seam(s)**: `SEAM-4`, `SEAM-5`
  - **Carried contract IDs**: `C-04`, `C-05`
  - **Purpose**: expose one derived row model consumed by both JSON/Markdown publication and later validator checks.
  - **State**: `revalidated`
  - **Revalidation trigger**: any change to row fields, output ordering, or evidence-note rules
  - **Satisfied by**: shared derivation model plus deterministic JSON and Markdown renders from that model
  - **Notes**: `SEAM-3` has now landed and closed, and `SEAM-4` has revalidated against the published row-model and Markdown-projection contract surfaces recorded in the seam-3 closeout.

- **Thread ID**: `THR-04`
  - **Producer seam**: `SEAM-4`
  - **Consumer seam(s)**: `SEAM-5`
  - **Carried contract IDs**: `C-06`
  - **Purpose**: convert semantic contradictions into deterministic failures before stale support claims land.
  - **State**: `revalidated`
  - **Revalidation trigger**: any repo-gate decision that changes whether support-matrix generation runs in `make preflight`, or any new contradiction class
  - **Satisfied by**: validator coverage for pointer/status/row mismatches and stale Markdown
  - **Notes**: `SEAM-5` has revalidated against this thread and treats the contradiction contract as current input; if repo-gate cost changes, the boundary must be recorded explicitly.

- **Thread ID**: `THR-05`
  - **Producer seam**: `SEAM-5`
  - **Consumer seam(s)**: pack closeout and future agent-onboarding seams
  - **Carried contract IDs**: `C-07`
  - **Purpose**: keep the shared support-matrix core future-agent-shaped rather than Codex/Claude hard-coded.
  - **State**: `published`
  - **Revalidation trigger**: any new agent root, fixture schema change, or refactor that introduces agent-name branching into the shared core
  - **Satisfied by**: Codex + Claude + synthetic third-agent-shaped fixture suites, plus generated JSON and Markdown publication outputs that remain tied to the same shared row model
  - **Notes**: the thread is now published through `SEAM-5` closeout and must be revalidated by any future agent-onboarding seam that changes root shape, row ordering, evidence-note rules, or shared-core neutrality.

## Dependency graph

- `SEAM-1 -> SEAM-2`: semantics and authority must be explicit before extracting a shared neutral seam.
- `SEAM-2 -> SEAM-3`: derivation/publication should consume one neutral normalization path, not duplicate per-agent logic.
- `SEAM-3 -> SEAM-4`: validator and staleness checks must consume the final derived row model.
- `SEAM-3 -> SEAM-5`: fixture and golden coverage need the final renderer/model shape.
- `SEAM-4 -> SEAM-5`: conformance coverage must prove contradiction failures and stale-Markdown enforcement.

## Critical path

`SEAM-1 (semantics + authority)` -> `SEAM-2 (shared normalization + root intake)` -> `SEAM-3 (row derivation + publication)` -> `SEAM-4 (validator enforcement)` -> `SEAM-5 (neutral fixtures + goldens)`

## Workstreams

- `WS-CONTRACT`: `SEAM-1`
- `WS-SHARED-CORE`: `SEAM-2`
- `WS-PUBLICATION`: `SEAM-3`
- `WS-VALIDATION`: `SEAM-4`
- `WS-CONFORMANCE`: `SEAM-5`
