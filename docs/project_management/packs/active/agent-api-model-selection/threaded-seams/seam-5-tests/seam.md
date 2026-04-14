---
seam_id: SEAM-5
seam_slug: tests
status: closed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-5-tests.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
    - ../../governance/seam-3-closeout.md
    - ../../governance/seam-4-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
    - THR-04
    - THR-05
  stale_triggers:
    - capability matrix regeneration is deferred from advertising changes
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S3
  status: passed
open_remediations: []
---
# SEAM-5 - Tests (Activated)

## Seam brief (source of truth)

- See `../../seam-5-tests.md`.

## Promotion basis
- **Seam ID**: SEAM-5
- **Name**: regression coverage for `agent_api.config.model.v1`
- **Goal / value**: lock the universal model-selection contract in place so backend or spec churn
  cannot silently regress R0 ordering, trim-before-map semantics, absence behavior, backend argv
  placement, safe runtime rejection, or capability publication.
- **Type**: risk (contract conformance + regression)
- **Scope**
  - In:
    - `SEAM-5A` coverage for unsupported-before-`InvalidRequest` ordering and the pinned safe
      invalid template `invalid agent_api.config.model.v1`.
    - `SEAM-5B` coverage for trim-before-map semantics, absence/default preservation, Codex and
      Claude backend mapping, runtime rejection translation, terminal `Error` event emission, and
      capability-matrix freshness after advertising changes.
    - Regression checks that backend seams consume the shared normalized value rather than
      re-reading the raw extension payload.
  - Out:
    - live upstream catalog/e2e compatibility tests against real model availability.
    - speculative tests for future universal keys such as `fallback-model`.
- **Touch surface**:
  - `crates/agent_api/src/backend_harness/normalize/tests.rs`
  - `crates/agent_api/src/backend_harness/runtime/tests.rs`
  - `crates/agent_api/src/backend_harness/runtime/tests/**`
  - `crates/agent_api/src/backends/codex/tests/capabilities.rs`
  - `crates/agent_api/src/backends/codex/tests/mapping.rs`
  - `crates/agent_api/src/backends/codex/tests/app_server.rs`
  - `crates/agent_api/src/backends/codex/tests/backend_contract.rs`
  - `crates/agent_api/src/backends/claude_code/tests/capabilities.rs`
  - `crates/agent_api/src/backends/claude_code/tests/mapping.rs`
  - `crates/agent_api/src/backends/claude_code/tests/backend_contract.rs`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- **Verification**:
  - `cargo test -p agent_api backend_harness::normalize`
  - `cargo test -p agent_api codex`
  - `cargo test -p agent_api claude_code`
  - `cargo run -p xtask -- capability-matrix`
  - `make test`
- **Threading constraints**
  - Upstream blockers:
    - `SEAM-1` unlocks the early `SEAM-5A` ordering and invalid-template tests.
    - `SEAM-2` unlocks shared-normalizer handoff coverage and capability-matrix freshness
      assertions.
    - `SEAM-3` unlocks Codex mapping and Codex runtime-rejection coverage.
    - `SEAM-4` unlocks Claude mapping and Claude runtime-rejection coverage.
  - Downstream blocked seams:
    - none; this seam is the terminal regression gate for the feature.
  - Contracts produced (owned):
    - none. SEAM-5 only pins conformance to contracts owned by SEAM-1 through SEAM-4.
  - Contracts consumed:
    - `MS-C03`, `MS-C04`, `MS-C05`, `MS-C06`, `MS-C07`, `MS-C08`, `MS-C09`

- Upstream seam exit: `../../governance/seam-4-closeout.md` (seam-exit gate passed; promotion readiness ready).
- Required threads: `THR-01..THR-05` are published per `../../threading.md`.

## Next planning step

- Execute `slice-*.md` sequentially (S1..S3), then complete the dedicated `seam-exit-gate` slice.
