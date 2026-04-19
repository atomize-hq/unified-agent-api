---
pack_id: cli-manifest-support-matrix
pack_version: v1
pack_status: extracted
source_ref: docs/project_management/next/cli-manifest-support-matrix/plan.md
execution_horizon:
  active_seam: null
  next_seam: null
---

# Scope Brief - CLI manifest support matrix

- **Goal**: turn existing committed CLI manifest evidence into one deterministic support publication pipeline that emits a machine-readable support artifact plus a Markdown projection without changing runtime `agent_api` behavior.
- **Why now**: the repo already carries enough manifest, coverage, pointer, and version metadata to publish support truth mechanically, but current docs and validators do not yet pin the semantics or prevent support-claim drift.
- **Primary user(s) + JTBD**:
  - repo maintainers: "publish deterministic support truth from committed manifest evidence instead of hand-maintained prose"
  - backend and tooling maintainers: "reuse one neutral derivation path across Codex, Claude Code, and future agents without duplicating wrapper-normalization logic"
  - reviewers and release operators: "fail fast when pointers, version metadata, and published support rows disagree"
- **In-scope**:
  - lock support semantics and naming in the UAA spec layer
  - extract shared wrapper-coverage normalization into a neutral seam
  - generate `cli_manifests/support_matrix/current.json`
  - generate `docs/specs/unified-agent-api/support-matrix.md` from the same derived model
  - harden validator/test coverage for pointer contradictions, Markdown staleness, and neutral future-agent-shaped fixtures
- **Out-of-scope**:
  - runtime support-metadata APIs in `agent_api`
  - replacing the current capability matrix
  - rebuilding snapshot/union/coverage-report pipelines
  - onboarding a real third CLI agent in phase 1
  - introducing a second mutable support ledger under `docs/specs/**`
- **Success criteria**:
  - `cargo run -p xtask -- support-matrix` deterministically writes the JSON artifact and Markdown projection
  - support rows derive only from committed manifest/version/pointer/report metadata
  - published support truth stays mechanically consistent with per-target pointers and `versions/<version>.json.status`
  - wrapper-coverage normalization lives in one shared neutral module with thin per-agent adapters
  - tests cover Codex, Claude Code, and a synthetic third-agent-shaped fixture
  - `make preflight` remains the repo integration gate
- **Constraints**:
  - `cli_manifests/**` stays the evidence layer
  - `docs/specs/unified-agent-api/**` stays the semantics/publication layer
  - target-scoped support truth is primary; version summaries are projections
  - backend-specific passthrough remains visible state but is not counted as UAA unified support
  - existing codex-root-specific commands stay codex-specific where behavior is genuinely codex-root-specific
- **External systems / dependencies**:
  - committed manifest roots under `cli_manifests/codex/**` and `cli_manifests/claude_code/**`
  - xtask surfaces in `crates/xtask/src/main.rs`, `capability_matrix.rs`, `codex_validate.rs`, `codex_wrapper_coverage.rs`, and `claude_wrapper_coverage.rs`
  - canonical UAA publication surfaces under `docs/specs/unified-agent-api/**`
  - repo integration gate `make preflight`
- **Known unknowns / risks**:
  - partial target coverage could collapse into a false version-global "supported" claim if row derivation is not target-first
  - backend support state could leak into UAA unified support if layer boundaries are not explicit
  - pointer movement can drift from published support rows unless the generator and validators share one truth model
  - a hidden Codex- or Claude-specific assumption in normalization could block future-agent onboarding
- **Assumptions**:
  - execution horizon is inferred from the critical path: semantics lock-in first, shared normalization second, publication and enforcement afterward
  - this repo's durable normative contract refs are the existing `docs/specs/**` documents rather than a separate `docs/contracts/` tree
  - the support-matrix generator remains cheap enough to stay inside repo-gate expectations once implemented
