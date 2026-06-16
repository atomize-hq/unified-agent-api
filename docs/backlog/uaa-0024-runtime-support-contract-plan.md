# PLAN — UAA Generic Runtime Support Contract (Codex First)

Status: Proposed  
Scope: library-level runtime-support metadata surface plus automation integration

This plan implements the draft contract in:

- `docs/specs/unified-agent-api/runtime-support-contract.md`

It assumes the following locked product decisions:

- generic contract, Codex first implementation
- public API exposed from `unified-agent-api`
- v1 publishes only the `latest_validated` projection
- consumers own downloads, checksums, extraction, and install layout
- target identity is full `target_triple`
- the runtime-support payload is a library-only embedded artifact, not a consumer-time manifest reader
- maintenance alignment stays inside existing packet/publication machinery without widening `requested_control_plane_actions`
- onboarding, publication/promotion, and maintenance automation must all stay aligned

## Objective

Add a library-level UAA support metadata surface that publishes UAA-owned runtime support truth
without forcing consumers to read repo files directly, while keeping runtime acquisition and install
ownership outside the generic contract.

This plan specifically implements a **validated-runtime projection**.
It does not attempt to republish both pointer tiers and does not redefine the support-matrix contract's
`validated` vs `supported` semantics.

## Implementation spine

### Phase 1 — Contract and API freeze

Freeze the public semantic model before touching implementation:

- runtime family
- target triple
- validated runtime version
- resolve/list operations
- typed fail-closed errors

Deliverables:

- draft or updated UAA spec documents
- crate-root API placement decision
- success/failure semantics documented in tests/specs

### Phase 2 — Embedded metadata pipeline

Build the generic internal pipeline that converts committed manifest truth into embedded crate-owned
runtime-support data.

Inputs:

- per-target `latest_validated` pointer files
- per-version metadata
- publication enrollment/support truth needed to know which runtime families participate

Deliverables:

- generic derivation model
- Codex-first generated payload
- tests that reject drift or contradictory tuples

### Phase 3 — Public umbrella API

Expose the generic support surface from `crates/agent_api` without leaking backend wrapper types.

Likely pieces:

- `RuntimeSupportRecord`
- resolve function(s)
- list function(s)
- typed error path for unknown runtime / unsupported target / missing validated tuple

Deliverables:

- public API implementation
- crate-level docs/examples
- unit tests

### Phase 4 — Codex-first implementation wiring

Wire the generic pipeline to Codex committed truth first.

Deliverables:

- embedded Codex metadata
- tests proving Linux/Codex tuple resolution from the current committed `latest_validated` pointer
- no runtime dependency on repo-relative file loading

### Phase 5 — Automation integration

Thread the new support surface through the existing factory and maintenance flows so it is not a sidecar
that drifts from publication truth.

This integration MUST preserve the current maintenance packet contract:

- automated release-watch requests remain `requested_control_plane_actions = ["packet_doc_refresh"]`
- runtime-support regeneration/validation is owned by existing publication and packet refresh lanes
- the feature MUST NOT introduce a second maintenance action queue for runtime-support work

Required touchpoints:

- `onboard-agent` / approval and registry publication enablement
- `prepare-publication`
- `refresh-publication`
- `prepare-proving-run-closeout` / `close-proving-run` continuity expectations where needed
- `prepare-agent-maintenance`
- `refresh-agent`
- support/publication validation paths
- operator documentation:
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
  - `docs/cli-agent-onboarding-factory-workflow-atlas.md`
  - `docs/cli-agent-maintenance-steady-state-plan.md` (pointer or authority notes if impacted)

Recommended packetization for implementation review:

- Packet 1: contract/API/data seam
- Packet 2: automation/doc reconciliation seam

## Dependency order

1. Contract freeze
2. Embedded metadata derivation design
3. Public API scaffolding in `agent_api`
4. Codex-first data implementation
5. Automation and docs integration
6. Final validation and green gates

This ordering is required because the automation flows should consume a stable API/data model, not one
that is still moving while docs and tests are being written.

## Parallelism

Can be parallel after Phase 1:

- embedded-data derivation work
- public API scaffolding
- automation/doc impact audit

Must stay sequential:

- final API stabilization before downstream docs/tests are declared done
- final automation integration after the data model exists
- green-gate closeout after all touched surfaces are aligned

## Risks and mitigations

### Risk 1 — Reintroducing repo-file runtime coupling

If the first implementation just teaches consumers to parse `cli_manifests/**`, the public contract fails.

Mitigation:

- require embedded crate-owned data for runtime consumption
- add tests that exercise the API without repo-path assumptions
- reject designs that require consumer-time JSON/pointer parsing or a repo checkout

### Risk 2 — Smuggling download policy into the support contract

There is a natural temptation to bundle URLs/checksums because they live adjacent to existing Codex manifest truth.

Mitigation:

- keep the contract version-only
- document that consumers own acquisition/integrity
- reject API proposals that expose download URLs or checksums

### Risk 3 — Drift between publication truth and embedded support data

If support-matrix publication changes but embedded support data is not refreshed, UAA will advertise stale support truth.

Mitigation:

- add regeneration/validation to publication and maintenance flows
- add drift tests in xtask and lifecycle checks
- keep maintenance integration inside the existing packet/publication lanes rather than introducing
  new `requested_control_plane_actions`

### Risk 4 — Codex-specific logic leaking into the generic API

A Codex-first implementation can accidentally hardcode Codex naming or target assumptions into shared code.

Mitigation:

- make the internal model runtime-family keyed from the start
- keep Codex-specific mapping isolated to its first data source implementation

### Risk 5 — Public API contract conflict with existing umbrella rules

`agent_api` currently forbids publicly re-exporting backend crate types.

Mitigation:

- use serde-friendly umbrella-owned types only
- update the canonical contract docs in the same implementation change if crate-root API additions are required

## Verification checkpoints

### Checkpoint A — Spec review

Reviewers confirm:

- ownership split is clear
- target triple semantics are explicit
- validated-only projection semantics are explicit
- automation integration is in scope

### Checkpoint B — Data model review

Reviewers confirm:

- generic embedded model exists
- Codex-first derivation works from committed truth
- no download/install details entered the generic contract

### Checkpoint C — Public API review

Reviewers confirm:

- API is exposed from `unified-agent-api`
- backend wrapper types are not leaked
- resolve/list and error semantics are stable

### Checkpoint D — Automation review

Reviewers confirm:

- onboarding/publication/maintenance flows regenerate or validate the support metadata surface
- operator docs explain the new ownership boundary

### Checkpoint E — Green gate

Reviewers confirm:

- relevant unit/integration tests pass
- support/capability publication checks pass
- `make preflight` passes

## Done definition

This plan is complete when:

- the draft contract has been implemented without reopening the locked product decisions
- `unified-agent-api` publishes generic validated runtime support truth for Codex by target triple
- consumers no longer need repo-file reads for that support truth
- existing factory and maintenance automation surfaces keep the embedded support metadata aligned with committed publication truth
