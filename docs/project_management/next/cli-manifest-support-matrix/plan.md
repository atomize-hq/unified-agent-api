# CLI_MANIFEST_SUPPORT_MATRIX – Plan

Source: `docs/project_management/next/cli-manifest-support-matrix-findings.md`  
Status: Ready for implementation  
Last reviewed (UTC): 2026-04-15

## Purpose
Turn the existing manifest parity evidence in `cli_manifests/codex` and `cli_manifests/claude_code` into a first-class support publication pipeline. Phase 1 does not build a new system. It locks support semantics, extracts the small neutral seams the repo is already asking for, generates a structured support-matrix artifact from committed manifest/version metadata, publishes a Markdown projection in the UAA spec tree, and hardens validation so support claims cannot drift from machine truth.

## Scope Lock
- Keep `cli_manifests/**` as the only evidence layer.
- Keep `docs/specs/unified-agent-api/**` as the semantics and publication layer.
- Keep the existing generated capability matrix separate from the new support matrix.
- Keep runtime `agent_api` behavior unchanged in phase 1.
- Keep support truth target-scoped first, then derive per-version summaries from those rows.
- Model support in three layers:
  - manifest / upstream support
  - backend-crate support
  - UAA unified support
- Preserve backend-specific passthrough as visible state, but do not count it as UAA unified support.

## Success Criteria
- `cargo run -p xtask -- support-matrix` deterministically writes one machine-readable support artifact and one Markdown projection.
- Support rows are derived from committed manifest/version metadata, not hand-edited prose.
- `versions/<v>.json.status`, per-target pointers, and published support rows are mechanically consistent.
- Manifest docs stop describing already-shipped artifacts as "planned".
- Shared wrapper-coverage normalization logic exists in one neutral place with thin Codex and Claude adapters.
- Tests cover Codex fixtures, Claude fixtures, and at least one synthetic third-agent-shaped fixture.
- `make preflight` remains the repo gate for integration.

## What Already Exists
The plan must reuse these surfaces instead of rebuilding them:

- Manifest evidence:
  - `cli_manifests/codex/**`
  - `cli_manifests/claude_code/**`
- Existing generators and validators:
  - [main.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/main.rs)
  - [capability_matrix.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/capability_matrix.rs)
  - [codex_report.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/codex_report.rs)
  - [codex_version_metadata.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/codex_version_metadata.rs)
  - [codex_validate.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/codex_validate.rs)
  - [codex_wrapper_coverage.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/codex_wrapper_coverage.rs)
  - [claude_wrapper_coverage.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/claude_wrapper_coverage.rs)
- Existing generated publication pattern:
  - `docs/specs/unified-agent-api/capability-matrix.md`
- Existing validator/test posture:
  - `crates/xtask/tests/*.rs`
  - `make preflight`

## Not In Scope
- Adding a runtime support-metadata API to `agent_api`.
- Replacing the current capability matrix artifact.
- Rewriting the snapshot, union, or coverage-report pipelines.
- Onboarding a real third CLI agent in phase 1.
- Solving every Codex multi-target CI gap in the same slice.
- Introducing a second mutable support ledger under `docs/specs/**`.

## Authority And Publication Flow
```text
upstream release pins
        |
        v
cli_manifests/<agent>/artifacts.lock.json
        |
        v
snapshots/<version>/<target>.json
        |
        v
snapshots/<version>/union.json
        |
        +------------------------------+
        |                              |
        v                              v
wrapper_coverage.json          versions/<version>.json
        |                              |
        +--------------+---------------+
                       |
                       v
reports/<version>/coverage.*.json
                       |
                       v
latest_validated / latest_supported pointers
                       |
                       v
cli_manifests/support_matrix/current.json
                       |
                       v
docs/specs/unified-agent-api/support-matrix.md
```

The capability matrix stays separate:

```text
built-in backend capability advertising
                |
                v
docs/specs/unified-agent-api/capability-matrix.md
```

## Concrete Artifact Decisions
These choices are now part of the implementation plan:

| Concern | Decision |
|---|---|
| Machine-readable support artifact | `cli_manifests/support_matrix/current.json` |
| Human-readable projection | `docs/specs/unified-agent-api/support-matrix.md` |
| New neutral xtask entrypoint | `cargo run -p xtask -- support-matrix` |
| New primary module | `crates/xtask/src/support_matrix.rs` |
| Shared wrapper normalization seam | `crates/xtask/src/wrapper_coverage_shared.rs` |
| Existing capability matrix | stays as-is, separate concern |
| Existing codex-specific commands | remain codex-specific where behavior is truly codex-root-specific |

## Semantic Model
Support publication is target-scoped first. Version summaries are projections, not the primitive.

```text
agent x version x target
        |
        +--> manifest_support
        |      "what the committed manifest evidence says"
        |
        +--> backend_support
        |      "what crates/codex or crates/claude_code safely support"
        |
        +--> uaa_support
        |      "what UAA can claim as deterministic cross-agent behavior"
        |
        +--> passthrough_visibility
               "backend-specific surface visible, but not unified"
```

Implementation rule:

- `versions/<v>.json.status` remains workflow-stage metadata.
- Per-target `latest_supported/*` pointers plus support-matrix rows are the published truth.
- The generator must reject or flag cases where those truths disagree.

## Suggested Branching
- Orchestration branch: `feat/cli-manifest-support-matrix`
- Prefix: `cmsm`
- Worktree pattern: `wt/<branch>`

This plan intentionally stops short of scaffolding `tasks.json`, `session_log.md`, and kickoff prompts. Those should be generated from the phases below once the branch is created.

## Phase Overview
### C0 – Semantic Lock-In And Naming Cleanup
Goal: remove ambiguity before adding more automation.

Primary work:
- Add a new semantics owner doc for support publication:
  - `docs/specs/unified-agent-api/support-matrix.md`
- Update UAA spec index/linkage where needed:
  - `docs/specs/unified-agent-api/README.md`
- Remove stale "planned" language and incorrect references from:
  - `cli_manifests/codex/README.md`
  - `cli_manifests/claude_code/README.md`
  - `cli_manifests/codex/VALIDATOR_SPEC.md`
  - `cli_manifests/claude_code/VALIDATOR_SPEC.md`
  - `cli_manifests/codex/CI_AGENT_RUNBOOK.md`
  - `cli_manifests/claude_code/CI_AGENT_RUNBOOK.md`
  - `cli_manifests/codex/RULES.json`
  - `cli_manifests/claude_code/RULES.json`
- Add the neutral xtask command wiring in [main.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/main.rs).

Acceptance:
- `validated` vs `supported` semantics are written once, explicitly.
- Docs no longer contradict committed artifacts.
- Generic cross-agent naming is neutral. Truly codex-specific commands stay codex-specific.

### C1 – Shared Wrapper Normalization And Support-Matrix Core
Goal: extract the duplicated normalization seam and build the derivation model once.

Primary work:
- Extract shared normalization helpers from:
  - [codex_wrapper_coverage.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/codex_wrapper_coverage.rs)
  - [claude_wrapper_coverage.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/claude_wrapper_coverage.rs)
- Add:
  - `crates/xtask/src/wrapper_coverage_shared.rs`
  - `crates/xtask/src/support_matrix.rs`
- Keep per-agent adapters thin:
  - manifest loading
  - default rules path selection
  - crate-specific manifest imports

Acceptance:
- Shared normalization rules live in one module.
- Support-matrix derivation is single-pass and reusable.
- No giant generic parity framework appears in phase 1.

### C2 – Generated Support Publication
Goal: derive support truth from the existing evidence layer and publish it in two forms.

Primary work:
- Implement `xtask support-matrix` with deterministic defaults:
  - JSON out: `cli_manifests/support_matrix/current.json`
  - Markdown out: `docs/specs/unified-agent-api/support-matrix.md`
- Reuse publication patterns from [capability_matrix.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/capability_matrix.rs), but keep semantics separate.
- Derive rows from:
  - `cli_manifests/*/versions/*.json`
  - `cli_manifests/*/pointers/**`
  - `cli_manifests/*/reports/**`
  - `cli_manifests/*/current.json`
- Emit explicit row fields for:
  - agent
  - version
  - target
  - manifest support state
  - backend support state
  - UAA support state
  - pointer promotion state
  - evidence notes when a row is intentionally partial

Acceptance:
- JSON and Markdown are generated from the same derived model.
- The support matrix does not overload or replace the capability matrix.
- The generator fails loudly on contradictory manifest inputs.

### C3 – Validation Hardening And Fixture Coverage
Goal: make drift impossible to land quietly.

Primary work:
- Extend existing validation logic where it already owns codex-root invariants:
  - [codex_validate.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/codex_validate.rs)
- Add generator-level checks for cross-root contradictions before publication.
- Add or update `crates/xtask/tests/*.rs` for:
  - support-state derivation fixtures
  - JSON golden output
  - Markdown golden output
  - pointer/status contradiction cases
  - synthetic third-agent-shaped fixture coverage
- Wire support-matrix generation into the repo gate if it remains cheap and deterministic.

Acceptance:
- Pointer advancement and support publication cannot silently disagree.
- Markdown staleness is caught automatically.
- The shared seam is verified against a non-Codex/non-Claude-shaped fixture.

## Minimal Implementation Sequence
```text
C0 semantics/docs
    |
    +--> C1 shared normalization
    |
    +--> C2 support-matrix generator
               |
               v
            C3 validation + goldens + gate wiring
```

Do not reverse this. If semantics are not pinned first, the generator will just freeze today's contradictions into a prettier format.

## Test Strategy
This feature is tooling-heavy, so the test plan is command- and artifact-driven.

### Code Path Coverage
```text
[+] support_matrix::load_agent_roots()
    |- codex fixture root
    |- claude fixture root
    `- synthetic third-agent-shaped root

[+] support_matrix::derive_rows()
    |- target-scoped manifest rows
    |- backend support rows
    |- UAA support rows
    `- passthrough visibility rows

[+] support_matrix::validate_consistency()
    |- status vs latest_supported pointer mismatch
    |- incomplete union vs supported claim mismatch
    `- missing evidence vs intentionally partial mismatch

[+] support_matrix::render_json()
    `- deterministic ordering / schema-valid output

[+] support_matrix::render_markdown()
    `- deterministic projection from JSON model
```

### Required Test Surfaces
- Unit tests for row derivation and ordering.
- Fixture tests for Codex and Claude roots using checked-in manifest samples.
- A synthetic third-agent-shaped fixture to prove the neutral seam is actually neutral.
- Golden tests for:
  - `cli_manifests/support_matrix/current.json`
  - `docs/specs/unified-agent-api/support-matrix.md`
- Validator tests for pointer/status contradictions and stale Markdown.

### Commands
- Fast local loop:
  - `cargo test -p xtask support_matrix`
  - `cargo test -p xtask wrapper_coverage`
- Integration gate:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - relevant `cargo test -p xtask ...`
  - `make preflight`

Prior learning applied: `uaa-use-make-preflight` (confidence 9/10). The repo gate here is `make preflight`, not an ad hoc command bundle.

## Failure Modes
| Codepath | Failure | Guardrail |
|---|---|---|
| support-matrix derivation | partial target coverage collapses into a false version-global "supported" claim | target-first row model + derivation tests |
| layer promotion | backend-crate support leaks into UAA unified support too early | explicit layer fields + contradiction checks |
| pointer interpretation | `latest_supported/*` moves without matching row evidence | generator preflight checks + validator tests |
| Markdown projection | checked-in docs drift from JSON | golden test + reproducible renderer |
| shared normalization | hidden Codex or Claude assumptions reject future-agent-shaped inputs | synthetic fixture suite |
| naming cutover | maintainers lose workflows because truly codex-specific commands were renamed unnecessarily | rename only neutral seams; keep codex-root commands codex-root-specific |

Critical gaps from the review that phase 1 must close:
- support-matrix derivation must have automated tests
- pointer consistency must have deterministic failure behavior
- Markdown staleness must be mechanically checked

## Parallelization Strategy
| Lane | Modules touched | Depends on |
|---|---|---|
| A. semantics/docs | `docs/specs/unified-agent-api/**`, `cli_manifests/**`, `crates/xtask/src/main.rs` | — |
| B. shared normalization | `crates/xtask/src/*wrapper_coverage*`, new shared helper | A |
| C. support-matrix generator | `crates/xtask/src/support_matrix.rs`, `cli_manifests/support_matrix/**`, `docs/specs/unified-agent-api/support-matrix.md` | A |
| D. validation + goldens | `crates/xtask/src/codex_validate.rs`, `crates/xtask/tests/*.rs` | C, partial B |

Execution order:
- Launch Lane A first.
- Launch Lanes B and C in parallel after semantic lock-in.
- Launch Lane D once the support-matrix model is stable enough to fixture against.

Conflict flags:
- Lanes A and C both touch `crates/xtask/src/main.rs`.
- Lanes C and D both touch support-matrix model shape and output ordering.
- Do not split Lane C across multiple workers unless one owns JSON/model and the other owns Markdown only.

## Follow-On After This Plan
Once the feature branch exists, scaffold the normal planning-pack artifacts from this plan:
- `tasks.json`
- `session_log.md`
- phase-local kickoff prompts

That scaffolding should mirror the repo's existing `docs/project_management/next/<feature>/` packs and break C0-C3 into code/test/integration tasks.

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 0 | — | — |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | — | — |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 1 | CLEAR | Phase 1 scope locked, neutral seams chosen, support publication artifacts pinned, failure modes and test obligations explicit |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | — | — |

**UNRESOLVED:** 0  
**VERDICT:** ENG CLEARED — ready to scaffold tasks and implement.
