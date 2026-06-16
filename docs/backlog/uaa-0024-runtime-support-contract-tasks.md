# TASKS — UAA Generic Runtime Support Contract (Codex First)

Status: Proposed

- [ ] Task: Freeze the runtime-support contract and umbrella API placement
  - Acceptance: The spec states the generic contract, Codex-first implementation scope, validated-only projection semantics, target-triple identity, library-only embedded-artifact packaging, and the ownership split between UAA support truth and consumer-managed binary acquisition.
  - Verify: Human review of `docs/specs/unified-agent-api/runtime-support-contract.md` and any related spec index updates.
  - Files: `docs/specs/unified-agent-api/runtime-support-contract.md`, `docs/specs/unified-agent-api/README.md`

- [ ] Task: Define the generic embedded metadata model
  - Acceptance: A shared internal model exists for `(runtime_family, target_triple) -> latest_validated version` and is explicitly derived from committed manifest truth rather than runtime repo-file access.
  - Verify: Targeted unit tests for the derivation layer; review confirms no URLs/checksums/install paths were added to the generic model.
  - Files: `crates/agent_api/src/**` or shared support module location, `crates/xtask/src/**` derivation helpers, related tests

- [ ] Task: Implement Codex-first support metadata derivation
  - Acceptance: Codex committed pointer/version truth is converted into the generic embedded metadata model using the `latest_validated` target tuples only.
  - Verify: `cargo test -p xtask --all-targets` plus targeted derivation tests covering current Codex target triples.
  - Files: `crates/xtask/src/**`, `cli_manifests/codex/**` readers/generators as needed, related tests

- [ ] Task: Expose the public runtime-support API from `unified-agent-api`
  - Acceptance: Downstream consumers can resolve one validated runtime tuple and list validated runtime tuples via `agent_api` public types/functions without depending on backend wrapper crate types.
  - Verify: `cargo test -p unified-agent-api --features codex`; compile-only or doc examples for the public API path.
  - Files: `crates/agent_api/src/lib.rs`, new `crates/agent_api/src/**` support modules, public API tests

- [ ] Task: Wire Codex-first data into the public API without repo runtime coupling
  - Acceptance: The runtime-support API works from embedded or crate-owned packaged data and does not require consumers to read `cli_manifests/**` at runtime.
  - Verify: Unit/integration test proving resolution works without repo-path lookup assumptions; review confirms no runtime manifest-path dependency remains.
  - Files: `crates/agent_api/src/**`, `crates/codex/src/**` only if needed for data packaging helpers, related tests

- [ ] Task: Integrate the support surface into onboarding and publication automation
  - Acceptance: The onboarding/publication flow regenerates or validates the runtime-support metadata surface anywhere support publication truth is refreshed, and the relevant specs/docs mention the new validated-runtime projection and library-only packaging rule.
  - Verify: Targeted lifecycle tests plus `cargo run -p xtask -- support-matrix --check` and any publication checks touched by the implementation.
  - Files: `crates/xtask/src/onboard_agent.rs`, `crates/xtask/src/prepare_publication.rs`, `crates/xtask/src/publication_refresh.rs`, related tests, `docs/specs/cli-agent-onboarding-charter.md`

- [ ] Task: Integrate the support surface into maintenance automation
  - Acceptance: Maintenance preparation/refresh/drift paths validate or regenerate the runtime-support metadata surface alongside existing support publication truth while keeping automated release-watch requests on `requested_control_plane_actions = ["packet_doc_refresh"]`, and operator-facing maintenance docs explain the ownership split.
  - Verify: Targeted maintenance tests plus `cargo test -p xtask --all-targets` for touched maintenance suites.
  - Files: `crates/xtask/src/agent_maintenance/**`, related tests, `docs/cli-agent-onboarding-factory-operator-guide.md`, `docs/cli-agent-onboarding-factory-workflow-atlas.md`, `docs/cli-agent-maintenance-steady-state-plan.md`

- [ ] Task: Close documentation and green-gate drift
  - Acceptance: Spec indexes and operator docs point to the new runtime-support contract, and the repo green gate is clean after all touched surfaces are aligned.
  - Verify: `cargo run -p xtask -- support-matrix --check`, `cargo run -p xtask -- capability-matrix --check`, `cargo run -p xtask -- capability-matrix-audit`, `make preflight`
  - Files: `docs/specs/unified-agent-api/README.md`, `docs/README.md` if needed, any final touched specs/docs/tests
