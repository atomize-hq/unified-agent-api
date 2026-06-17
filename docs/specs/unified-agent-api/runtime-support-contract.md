# Runtime Support Contract — Unified Agent API

Status: Approved  
Approved (UTC): 2026-06-16  
Date (UTC): 2026-06-16  
Canonical location: `docs/specs/unified-agent-api/runtime-support-contract.md`

This document defines the generic runtime-support metadata contract for the Unified Agent API.
It is the normative contract for the library-level surface that publishes UAA-owned support truth
without making consumers read repo files directly.

Normative language: this contract uses RFC 2119 requirement keywords (`MUST`, `MUST NOT`, `SHOULD`).

## Locked contract assumptions

1. The public consumer is the published `unified-agent-api` library crate, not repo-local file readers.
2. This is a **generic runtime support contract, Codex first implementation**.
3. This surface is a **library-only validated-runtime projection**, not a replacement for the support
   matrix contract.
4. The first contract revision publishes one resolved `latest_validated` version per
   `(runtime_family, target_triple)` tuple rather than exposing both pointer tiers.
5. Consuming crates and hosts own binary acquisition, checksum verification, extraction, caching, and install layout.
6. Target identity is the full `target_triple`, not an OS-only label.
7. The surface MUST participate in onboarding, publication/promotion, and maintenance automation flows.

## Objective

Expose a stable, library-level support metadata surface from `unified-agent-api` so downstream consumers
can ask UAA which validated runtime versions and target triples it publishes without reading
`cli_manifests/**` directly.

This contract defines a **validated-runtime projection** only.
It MUST NOT redefine the published support semantics in
`docs/specs/unified-agent-api/support-matrix.md`, which distinguishes `validated` from `supported`
and treats pointer state as input evidence rather than support truth by itself.

Primary user stories:

- As a consuming crate, I can resolve the UAA-published validated runtime version for
  `(runtime_family, target_triple)`.
- As a consuming crate, I can enumerate the target triples and resolved validated versions for one
  runtime family.
- As an onboarding/publication/maintenance automation path, I can regenerate or validate the embedded support
  metadata from committed repo truth and fail closed when that truth is missing or contradictory.
- As Substrate or another host, I can use UAA as the authority for version/target support while still owning
  the runtime binary sourcing and install mechanics.

## Tech Stack

- Rust 2021 workspace (`rust-version = 1.78`)
- Public crate: `crates/agent_api` (`unified-agent-api`)
- Runtime-specific wrapper crates such as `crates/codex`
- Repo automation and publication generation in `crates/xtask`
- Committed manifest evidence under `cli_manifests/<agent>/`
- Serde-friendly public types only; no backend-crate types in the public API

## Commands

Verify or refresh the affected public and automation surfaces:

```sh
cargo test -p unified-agent-api --features codex
cargo test -p xtask --all-targets
cargo run -p xtask -- support-matrix --check
cargo run -p xtask -- capability-matrix --check
cargo run -p xtask -- capability-matrix-audit
make preflight
```

Refresh commands for the committed projection:

```sh
cargo run -p xtask -- support-matrix
cargo run -p xtask -- prepare-publication --approval <approval_path> --write
cargo run -p xtask -- refresh-publication --approval <approval_path> --check
cargo run -p xtask -- prepare-agent-maintenance --agent <agent_id> ... --write
cargo run -p xtask -- refresh-agent --request <request_path> --check
```

## Project Structure

Relevant source-of-truth and consumer paths for this change:

```text
crates/agent_api/                         → public umbrella crate; consumer-facing API lives here
crates/codex/                             → Codex-first implementation details or embedded support data helpers
crates/xtask/                             → generation, validation, onboarding/publication/maintenance automation
cli_manifests/<agent>/                    → committed manifest truth (versions, pointers, reports, current.json)
docs/specs/unified-agent-api/             → normative UAA contracts
```

## Code Style

Public API additions MUST remain agent-agnostic, serde-friendly, and fail closed.
A representative style target:

```rust
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuntimeSupportRecord {
    pub runtime_family: String,
    pub target_triple: String,
    pub version: String,
}

pub fn resolve_runtime_support(
    runtime_family: &str,
    target_triple: &str,
) -> Result<RuntimeSupportRecord, AgentWrapperError> {
    // Validate inputs, consult embedded metadata, and fail closed on unknown or unsupported tuples.
}
```

Key conventions:

- prefer `snake_case` module and function names
- expose owned string data, not wrapper-crate types or borrowed manifest internals
- use umbrella-owned records plus typed errors for unsupported/unknown tuples
- do not embed binary download policy into the UAA support contract

## Testing Strategy

Testing must cover three layers:

1. **Library API tests**
   - in `crates/agent_api/**`
   - verify resolve/list behavior, validated-projection semantics, unknown-runtime failures,
     unsupported-target failures
2. **Generation/validation tests**
   - in `crates/xtask/**`
   - verify embedded support metadata derives only from committed repo truth and fails on drift
3. **Automation integration tests**
   - onboarding/publication/maintenance command tests
   - verify the new support metadata surface is included anywhere publication or maintenance support truth is refreshed or audited

Coverage expectations:

- every public API success path has a deterministic unit test
- every public API failure mode has a deterministic unit test
- every automation touchpoint that consumes or validates support truth has at least one regression test

## Boundaries

- Always:
  - keep the contract generic across runtime families
  - keep Codex as the first implementation only, not the only supported shape
  - use full `target_triple` identity
  - derive runtime support metadata from committed repo truth
  - keep this library surface limited to the `latest_validated` projection for v1
  - make the runtime-support payload crate-owned and usable without a repo checkout
  - keep consumer installs fail-closed when the tuple is unknown or unsupported
  - include onboarding, promotion/publication, and maintenance automation touchpoints in the implementation plan
- Ask first:
  - changing the existing `agent_api` crate-root public API contract shape
  - introducing new dependencies for embedded data generation or runtime loading
  - changing support-tier semantics in `docs/specs/unified-agent-api/support-matrix.md`
  - changing maintenance packet schema or registry schema beyond what is needed to route this surface through existing flows
- Never:
  - make consuming crates read `cli_manifests/**` directly at runtime
  - put download URLs, checksums, or install paths into the generic UAA support contract
  - duplicate version-selection policy in downstream crates
  - expose backend-wrapper crate types in the umbrella public API

## Contract Shape

### Public semantics

The public contract MUST support two consumer operations:

1. **Resolve** one exact validated runtime version for:
   - `runtime_family`
   - `target_triple`
2. **List** the supported target/version tuples for one runtime family

The public contract for this revision MUST NOT require callers to choose between pointer tiers.
It publishes one validated-runtime projection only.

### Validated projection model

The first contract revision MUST expose only:

- `latest_validated`

This value is a library-level projection chosen because it matches current maintenance and publication
operational truth most closely.
It MUST be documented as a validated-runtime baseline, not as a redefinition of the support matrix.

### Data ownership split

UAA runtime-support metadata MUST contain:

- runtime family
- target triple
- resolved semantic version

UAA runtime-support metadata MUST NOT contain:

- download URL
- checksum
- asset name
- extraction/install layout
- cache location

### Packaging model

The published library surface MUST be usable without a repo checkout.
Therefore the runtime-support metadata MUST be embedded in published crate artifacts or otherwise
materialized as crate-owned runtime data, not loaded from repo-relative paths at consumer runtime.

This contract is therefore a **library-only artifact**:

- consumer runtime code MUST NOT read `cli_manifests/**`
- consumer runtime code MUST NOT parse committed JSON or pointer files directly
- any manifest-derived processing MUST happen before publication or as part of crate-owned generation,
  never as a repo checkout requirement for downstream users

### Source-of-truth derivation

The embedded metadata MUST be derived from committed repo truth, initially:

- `cli_manifests/<agent>/pointers/latest_validated/<target>.txt`
- `cli_manifests/<agent>/versions/*.json`
- any registry/publication gate needed to determine whether the runtime family is enrolled for publication

Codex is the first implementation target.
The contract and derivation code MUST remain generic enough to onboard later runtime families without
redefining the API.

### Automation integration

The following flows MUST remain aligned with the new runtime-support surface:

- onboarding control-plane enrollment and publication enablement
- publication preparation and refresh
- proving-run closeout where publication truth becomes part of the committed baseline
- automated maintenance packet preparation, refresh, and closeout checks
- operator-facing docs that describe the onboarding and maintenance factory

The automation contract MUST ensure that when support publication truth changes, the embedded runtime-support
surface is regenerated or validated in the same lane rather than drifting separately.

For automated release-watch maintenance, this integration MUST stay within the existing packet/publication
machinery.
It MUST NOT widen `requested_control_plane_actions` beyond the current normative
`["packet_doc_refresh"]` contract.

## Conformance Criteria

This contract is satisfied when all of the following are true:

1. `unified-agent-api` exposes a library-level generic runtime-support API.
2. The public API is generic by runtime family and target triple, with Codex as the first implemented family.
3. The surface exposes only the `latest_validated` projection for this contract revision.
4. The surface returns versions only; binary sourcing and integrity remain consumer-owned.
5. Consumers do not need a repo checkout or direct manifest file access to use the API.
6. The implementation derives embedded metadata from committed repo truth and fails closed on drift.
7. Onboarding, publication/promotion, and maintenance automation flows are updated so this support surface is regenerated or validated alongside existing support publication truth without expanding automated maintenance `requested_control_plane_actions`.
8. Operator docs clearly describe this ownership split so future agents do not reintroduce repo-file runtime coupling.

## Open Questions

None.

Implementation details remain downstream design choices, for example:

- exact crate-root module path for the public API
- exact embedded-data generation mechanism
- exact xtask command or helper module that materializes the embedded support payload
