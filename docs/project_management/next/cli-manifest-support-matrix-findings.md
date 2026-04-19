# CLI Manifest + Support Matrix Findings

Status: research briefing  
Date (UTC): 2026-04-15  
Prepared for: follow-on planning and `/plan-eng-review` work on CLI manifest automation, version onboarding, and support-matrix generation

## Purpose

This document consolidates the repo context gathered during the current session so a later planning or engineering review pass can start from one artifact instead of re-reading `cli_manifests/**`, `docs/specs/**`, `docs/specs/unified-agent-api/**`, and the relevant `xtask` tooling from scratch.

It is intentionally structured as a pre-plan briefing:

- what already exists
- what is authoritative
- what is currently true in the repo
- what is inconsistent or stale
- which questions must be resolved before automation work should begin
- what a minimal planning boundary should be

## Scope Of This Briefing

In scope:
- `cli_manifests/codex/**`
- `cli_manifests/claude_code/**`
- `docs/specs/**`
- `docs/specs/unified-agent-api/**`
- `docs/adr/**` entries that directly constrain parity or support-state semantics
- `crates/xtask/src/{capability_matrix,codex_report,codex_version_metadata,codex_validate,parity_triad_scaffold}.rs`
- supporting validation scripts under `scripts/`

Out of scope:
- making code or doc fixes
- deciding the final automation architecture
- running a full `/plan-eng-review` workflow in this turn

## Executive Summary

The repo already has a real parity pipeline. This is not a speculative design. Both CLI roots already contain:

- pinned upstream artifacts
- generated snapshots
- generated union snapshots
- generated coverage reports
- deterministic validators
- version metadata
- per-target pointers

The main problem is not missing machinery. The main problem is that the support story is split across two truth layers:

1. `cli_manifests/**` contains the richer, versioned truth about upstream CLI coverage, validation, target support, and promotion.
2. `docs/specs/unified-agent-api/**` contains a generated capability advertisement view that is useful, but much narrower.

That split is causing drift and ambiguity:

- stale prose that still says several artifacts are "planned" when they already exist
- inconsistent use of `validated` vs `supported`
- a capability matrix that answers "what do built-in backends advertise now?" but not "which upstream CLI versions are supported on which targets/configs/flows?"

The likely architectural direction is:

- keep `cli_manifests/**` as the evidence and version-tracking layer
- define support-matrix semantics more explicitly in `docs/specs/unified-agent-api/**`
- generate a first-class support-matrix artifact from manifest/version metadata, not from hand-maintained prose

## How To Use This With `/plan-eng-review`

Recommended use:

1. Treat this document as the branch-local research brief.
2. Use the "Scope Challenge" and "What Already Exists" sections as Step 0 inputs.
3. Use the "Findings" and "Open Questions" sections as the architecture-review issue list.
4. Use the "Proposed Planning Boundary" section as the initial candidate scope for a plan.

If a later plan is created, this file should be cited as background context rather than copied into the plan verbatim.

## Eng Review Decision Log

This section records `/plan-eng-review` decisions as they are made so the document stays current during the review instead of being patched at the end.

### Decision 1. Step 0 scope boundary

Date (UTC): 2026-04-15

Accepted option: `1A`

Decision:

- phase 1 is the complete first lake:
  - pin semantics for `validated` vs `supported`
  - clean stale manifest/spec prose
  - generate structured support-matrix JSON from existing manifest/version metadata
  - generate Markdown publication under `docs/specs/unified-agent-api/`
  - extend validation and tests so status/pointer drift cannot pass silently
- future-agent onboarding must shape the schema and generator boundaries now, but generic onboarding scaffolding is deferred to a later phase unless phase 1 proves it is required

Rationale:

- this achieves the real automation goal for Codex and Claude Code without spending an innovation token on premature frameworking
- it keeps the first slice explicit and testable while still forcing agent-neutral semantics
- it avoids the common failure mode of building a “future-proof” abstraction before the third agent exists

### Decision 2. Support model layers

Date (UTC): 2026-04-15

Accepted option: `3A`

Decision:

- the implementation plan must model three distinct support layers:
  - manifest / upstream support
  - backend-crate support
  - UAA unified support
- backend-specific passthrough remains a first-class state, but it does not count as UAA unified support
- promotion into UAA universal capability advertising must remain stricter than backend-crate support for a specific agent

Rationale:

- a CLI surface can be supported by `crates/codex` or `crates/claude_code` before it is safe to claim deterministic cross-agent behavior in UAA
- this prevents the support matrix from collapsing backend readiness and UAA readiness into one misleading status
- it preserves a clean path for continued CLI version onboarding plus later promotion into the unified API

### Decision 3. Support scoping primitive

Date (UTC): 2026-04-15

Accepted option: `2A`

Decision:

- support is target-scoped first at every layer:
  - manifest / upstream support
  - backend-crate support
  - UAA unified support
- version-level summaries are derived projections over target rows, not the canonical primitive
- this rule applies even when a human-readable summary later collapses rows into one per-version statement

Rationale:

- future CLI agents are likely to have uneven target maturity, so a version-global primitive would hide the real state
- target-scoped rows let the plan represent Linux-first, partial-union, and staggered promotion without inventing ambiguous exceptions
- version summaries remain useful, but only as projections over explicit per-target evidence

### Decision 4. Canonical artifact placement

Date (UTC): 2026-04-15

Accepted option: `4A`

Decision:

- the canonical generated support-matrix JSON lives under `cli_manifests/` as derived evidence
- the generated human-readable Markdown projection lives under `docs/specs/unified-agent-api/`
- `docs/specs/**` remains the semantics and publication layer, not the machine-truth storage layer

Rationale:

- the matrix JSON is derived directly from manifest/version metadata and should live next to that evidence chain
- this keeps the docs tree from becoming a second mutable ledger
- reader-facing documentation still belongs in the spec tree, but as a projection of committed machine truth

### Decision 5. Runtime API boundary

Date (UTC): 2026-04-15

Accepted option: `5A`

Decision:

- phase 1 does not add a runtime metadata API to UAA
- support-matrix generation remains an offline, committed, validator-enforced artifact flow
- runtime UAA behavior continues to rely on backend capability advertising and deterministic request handling, not on shipping support-matrix metadata to consumers

Rationale:

- this keeps the first slice inside the release-engineering and documentation boundary
- it avoids turning an internal maintenance truth source into a public runtime contract that would need long-term compatibility support
- it preserves the option to revisit runtime publication later if a real consumer need appears

### Decision 6. Neutral parity naming and cutover

Date (UTC): 2026-04-15

Accepted option: `6A`, with hard cutover

Decision:

- phase 1 introduces neutral shared parity/support naming and module boundaries
- existing codex-branded generic command names and documentation are removed rather than preserved as long-term aliases
- codex-specific naming remains only where behavior is truly Codex-specific

Rationale:

- the repo is effectively greenfield for this support-matrix and future-agent onboarding work, so carrying misleading compatibility shims adds noise without much value
- hard cutover avoids normalizing the idea that generic multi-agent tooling should remain branded as Codex forever
- this still keeps the implementation explicit: shared logic gets neutral names, genuinely agent-specific logic keeps agent-specific names

### Decision 7. Wrapper coverage normalization seam

Date (UTC): 2026-04-15

Accepted option: `7A`

Decision:

- phase 1 extracts a shared wrapper-coverage normalization engine
- each agent keeps only a thin adapter for:
  - loading its crate-specific wrapper coverage manifest
  - supplying default rules paths and output paths
  - any genuinely agent-specific manifest details

Rationale:

- the current Claude and Codex wrapper-coverage generators are near-copies and will become obvious technical debt once another CLI agent arrives
- this is a good DRY extraction because the duplicated logic is already structurally the same
- it stays engineered enough: shared normalization is centralized, but snapshot/report/validator systems are not prematurely forced into one giant abstraction

### Decision 8. Support-matrix module boundary

Date (UTC): 2026-04-15

Accepted option: `8A`

Decision:

- phase 1 adds one dedicated neutral support-matrix module in `xtask`
- the module owns:
  - structured support-matrix derivation from manifest/version inputs
  - machine-readable output shaping
  - Markdown projection helpers or rendering entrypoints
- validators and publication commands consume that module rather than re-implementing support-matrix logic inline

Rationale:

- support-matrix generation is a distinct concern and should have one obvious home in the codebase
- this avoids smearing the new logic across version metadata generation, validation, and capability publication files
- it is the smallest clean extraction that keeps the phase-1 diff understandable

### Decision 9. Future-agent test posture

Date (UTC): 2026-04-15

Accepted option: `9A`

Decision:

- phase 1 test coverage includes:
  - real Codex fixture/workspace cases
  - real Claude Code fixture/workspace cases
  - at least one synthetic third-agent-shaped fixture suite
- the synthetic fixture is required for:
  - support-matrix derivation
  - shared wrapper-coverage normalization seams
  - neutral command/documentation assumptions where applicable

Rationale:

- the project goal explicitly includes future CLI-agent onboarding, so phase 1 should verify that new neutral seams are actually neutral
- a synthetic third-agent fixture is much cheaper than onboarding a real third agent prematurely, while still catching Codex/Claude-specific assumptions
- this is the complete version of the test plan for the chosen scope, not a shortcut that hopes generalization will work later

### Decision 10. Real third-agent scope

Date (UTC): 2026-04-15

Accepted option: `10A`

Decision:

- phase 1 does not onboard a real third CLI agent
- phase 1 proves future-agent readiness through synthetic third-agent-shaped fixtures and neutral module/command boundaries only
- any real third-agent onboarding is deferred to a later phase or separate plan once a concrete agent target is chosen

Rationale:

- real third-agent onboarding is product expansion, not just validation of the support-system architecture
- adding a real integration now would mix infrastructure cleanup with new upstream semantics, larger test scope, and new promotion decisions
- synthetic fixtures are enough to verify that the new shared seams are not accidentally Codex/Claude-specific

### Decision 11. Deferred third-agent onboarding follow-up

Date (UTC): 2026-04-15

Accepted option: `11A`

Decision:

- capture a follow-on TODO to select the first real third CLI agent and prepare its onboarding packet after phase 1 lands
- do not expand phase 1 to perform the real onboarding work now

Rationale:

- this preserves the chosen phase-1 scope while keeping the next concrete expansion step visible
- the repo goal includes future CLI-agent onboarding, so the deferred work should be tracked explicitly rather than left as conversational context

### Decision 12. Version status vs published support truth

Date (UTC): 2026-04-15

Accepted option: `12A`

Decision:

- `versions/<v>.json.status` remains a workflow-stage summary field
- per-target pointers plus target-scoped support-matrix rows are the canonical published truth
- validator logic should enforce that published support rows and pointers are internally consistent, without forcing the single version-level `status` field to represent every published target outcome directly

Rationale:

- the review already chose target-scoped rows as the primitive, so one scalar status field should not be overloaded into a misleading multi-target truth source
- this preserves useful workflow state in version metadata while making published support truth explicit and target-scoped
- it avoids the current ambiguity where `validated` and `supported` are being used as both workflow labels and publication claims

## Eng Review Notes

### Performance review

Date (UTC): 2026-04-15

Observed local command timings during review:

- `cargo run -p xtask -- capability-matrix`: about 1.0s wall time
- `cargo run -p xtask -- codex-validate --root cli_manifests/codex`: about 0.56s wall time

Observed repository scale:

- `cli_manifests/codex/`: about 736K, 42 files
- `cli_manifests/claude_code/`: about 284K, 32 files
- `docs/specs/unified-agent-api/`: about 108K
- `crates/xtask/tests/`: about 196K, 22 files

Assessment:

- no blocking performance issues were identified for phase 1
- the dominant performance risk is duplicated derivation work, not raw corpus size
- phase 1 should keep support-matrix derivation single-pass inside the dedicated support-matrix module, then let validators and Markdown publication consume the same derived model rather than re-deriving it independently

## Phase 1 Implementation Plan

This section turns the resolved `/plan-eng-review` decisions into the concrete phase `1A` execution plan.

### Goal

Land a neutral, validator-enforced support publication pipeline that:

- continues to support ongoing Codex and Claude Code version onboarding
- keeps backend-crate support distinct from UAA unified support
- publishes target-scoped support truth from manifest evidence
- proves future-agent readiness structurally without onboarding a real third agent yet

### What already exists

The implementation must reuse the existing machinery rather than replacing it:

- per-agent manifest evidence under `cli_manifests/**`
- union snapshots, coverage reports, version metadata, and pointer files
- `xtask` generation and validator entrypoints
- checked-in generated Markdown pattern from `capability-matrix`
- `crates/xtask/tests/*.rs` subprocess-style fixture testing

### Planned workstreams

#### Workstream A. Semantic and naming cleanup

- pin neutral terminology in manifest/spec prose so `validated`, `supported`, backend support, UAA unified support, and passthrough are not conflated
- hard-cutover generic multi-agent `xtask` command and module naming away from Codex-branded generic names
- fix known stale prose and cross-pollinated target-name / flag-name references

Primary touchpoints:

- `docs/project_management/next/cli-manifest-support-matrix-findings.md`
- `cli_manifests/codex/README.md`
- `cli_manifests/claude_code/README.md`
- `cli_manifests/codex/VALIDATOR_SPEC.md`
- `cli_manifests/claude_code/VALIDATOR_SPEC.md`
- `cli_manifests/codex/CI_AGENT_RUNBOOK.md`
- `cli_manifests/claude_code/CI_AGENT_RUNBOOK.md`
- `cli_manifests/codex/RULES.json`
- `cli_manifests/claude_code/RULES.json`
- `crates/xtask/src/main.rs`

#### Workstream B. Shared parity/support extraction

- extract a shared wrapper-coverage normalization engine with thin per-agent adapters
- add a dedicated neutral support-matrix module in `xtask`
- keep support-matrix derivation single-pass and reusable by generator, validator, and Markdown publication

Primary touchpoints:

- `crates/xtask/src/claude_wrapper_coverage.rs`
- `crates/xtask/src/codex_wrapper_coverage.rs`
- new neutral shared module(s) under `crates/xtask/src/`
- neutralized CLI wiring in `crates/xtask/src/main.rs`

#### Workstream C. Generated support publication

- generate canonical support-matrix JSON under `cli_manifests/`
- generate Markdown projection under `docs/specs/unified-agent-api/`
- model support rows target-first and layer-first:
  - manifest / upstream support
  - backend-crate support
  - UAA unified support
  - backend-specific passthrough visibility where applicable

Primary touchpoints:

- new canonical JSON artifact path under `cli_manifests/`
- new Markdown projection path under `docs/specs/unified-agent-api/`
- `crates/xtask/src/capability_matrix.rs` only where publication patterns are reused, not where capability semantics are overloaded

#### Workstream D. Validation and test hardening

- extend validator rules to reject status/pointer/support-row contradictions
- add fixture-driven derivation tests for Codex, Claude Code, and a synthetic third-agent-shaped case
- add staleness/golden tests for checked-in Markdown projection

Primary touchpoints:

- validator module(s) under `crates/xtask/src/codex_validate/` or renamed neutral equivalents after cutover
- `crates/xtask/tests/*.rs`

## Failure Modes

For each new phase-1 codepath, these are the realistic production-maintainer failures that must be covered.

| Codepath | Likely failure | Test needed | Error handling needed | User-visible effect if missed |
|---|---|---|---|---|
| Support-matrix derivation | partial-union target rows collapse into false version-global support | yes | yes | published support doc lies |
| Layer modeling | backend support is promoted to UAA unified support too early | yes | yes | consumers believe cross-agent behavior is deterministic when it is not |
| Pointer consistency | `latest_supported/*` advances without target-row support evidence | yes | yes | published support pointers drift from machine truth |
| Markdown projection | checked-in docs stale after JSON changes | yes | yes | reviewers read stale support status |
| Neutral command cutover | docs/tests still call removed codex-branded generic commands | yes | yes | maintainer workflows fail unexpectedly |
| Shared wrapper normalization | hidden Codex/Claude assumptions reject future-agent-shaped inputs | yes | yes | future agent onboarding requires rework of “shared” seams |

Critical gap rule from this review:

- no phase-1 implementation is complete unless support-matrix derivation, pointer consistency, and Markdown staleness each have both automated tests and deterministic failure behavior

## Worktree Parallelization Strategy

| Step | Modules touched | Depends on |
|------|-----------------|------------|
| Semantic + naming cleanup | `cli_manifests/**`, `docs/specs/**`, `crates/xtask/src/main.rs` | — |
| Shared wrapper normalization extraction | `crates/xtask/src/*wrapper_coverage*` | semantic naming decisions |
| Support-matrix module + generation | `crates/xtask/src/` new neutral support module, `cli_manifests/**`, `docs/specs/unified-agent-api/**` | semantic decisions |
| Validator + tests | `crates/xtask/src/*validate*`, `crates/xtask/tests/*.rs` | support-matrix module shape, semantic decisions |

### Parallel lanes

- Lane A: semantic + naming cleanup
- Lane B: shared wrapper normalization extraction
- Lane C: support-matrix module + JSON/Markdown generation
- Lane D: validator + tests

### Execution order

- launch Lane A first
- once naming and semantic contracts are pinned, launch Lanes B and C in parallel
- start Lane D after the support-matrix module shape is stable enough to write fixtures against, while still allowing validator tests to proceed in parallel with final publication wiring

### Conflict flags

- Lanes A, C, and D all touch `crates/xtask/src/main.rs` or adjacent neutral command wiring
- Lanes C and D both touch validator-facing support-matrix model code
- do not split Lanes C and D too finely across multiple workers unless ownership is very explicit

## Completion Summary

- Step 0: Scope Challenge — scope accepted as phase `1A`
- Architecture Review: 5 issues found, all resolved
- Code Quality Review: 3 issues found, all resolved
- Test Review: diagram produced, 22 planned gaps identified and turned into required test scope
- Performance Review: 0 issues found
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 1 item added
- Failure modes: 3 critical publication/consistency gaps flagged as mandatory coverage areas
- Outside voice: skipped
- Parallelization: 4 lanes, 2 parallel after semantic lock-in / 2 dependent follow-on lanes
- Lake Score: 12/12 decisions chose the more complete option for the selected phase

## Unresolved Decisions That May Bite Later

None for phase `1A`. The major semantic and scope decisions required to begin implementation are now pinned in this document.

## Research Basis

This briefing is based on:

- direct repo inspection during this session
- three parallel `gpt-5.4` high-reasoning research passes:
  - `cli_manifests/**`
  - `docs/specs/**` outside `docs/specs/unified-agent-api/**`
  - `docs/specs/unified-agent-api/**`
- direct inspection of generated and committed artifacts
- local execution of `cargo run -p xtask -- capability-matrix`

## Authority Map

### Practical authority chain

```text
Upstream releases / distribution manifests
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
latest_validated.txt / pointers/latest_validated/*
latest_supported.txt / pointers/latest_supported/*
                       |
                       v
current.json
```

### Current spec split

```text
cli_manifests/**
  owns versioned parity evidence, validation, promotion pointers, target support

docs/specs/unified-agent-api/**
  owns backend capability ids, extension semantics, generated capability matrix

Current gap:
  no single first-class spec/artifact that merges
  version x target x config x flow x capability x support status
```

## What Already Exists

### Manifest-level parity system

Both CLI roots already contain the same broad model:

- `min_supported.txt`
- `latest_validated.txt`
- `current.json`
- `pointers/latest_validated/<target>.txt`
- `pointers/latest_supported/<target>.txt`
- `versions/<version>.json`
- `reports/<version>/coverage.*.json`
- `wrapper_coverage.json`
- `artifacts.lock.json`
- `supplement/commands.json`

Key references:

- `cli_manifests/codex/README.md`
- `cli_manifests/claude_code/README.md`
- `cli_manifests/codex/RULES.json`
- `cli_manifests/claude_code/RULES.json`
- `cli_manifests/codex/VALIDATOR_SPEC.md`
- `cli_manifests/claude_code/VALIDATOR_SPEC.md`

### Deterministic tooling

Relevant tooling already exists and is active:

- `crates/xtask/src/codex_report.rs`
- `crates/xtask/src/codex_version_metadata.rs`
- `crates/xtask/src/codex_validate.rs`
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/capability_matrix_audit.rs`
- `crates/xtask/src/parity_triad_scaffold.rs`

What these tools already do:

- build coverage reports from union snapshots plus wrapper coverage
- compute version metadata
- enforce pointer/current/report invariants
- generate the unified-agent-api capability matrix
- scaffold parity work queues from coverage reports

### Normative support/coverage contracts

The repo already has strong contracts for:

- wrapper coverage generation and classification:
  - `docs/specs/codex-wrapper-coverage-generator-contract.md`
  - `docs/specs/codex-wrapper-coverage-scenarios-v1.md`
- parser compatibility and fixture-backed acceptance:
  - `docs/specs/codex-thread-event-jsonl-parser-contract.md`
  - `docs/specs/codex-thread-event-jsonl-parser-scenarios-v1.md`
  - `docs/specs/claude-stream-json-parser-contract.md`
  - `docs/specs/claude-stream-json-parser-scenarios-v1.md`
- wrapper event ingestion:
  - `docs/specs/wrapper-events-ingestion-contract.md`
- Codex/Claude backend transport mapping:
  - `docs/specs/codex-app-server-jsonrpc-contract.md`
  - `docs/specs/codex-streaming-exec-contract.md`
  - `docs/specs/codex-external-sandbox-mapping-contract.md`
  - `docs/specs/claude-code-session-mapping-contract.md`

### Unified Agent API capability publication

The repo already generates a capability matrix:

- artifact: `docs/specs/unified-agent-api/capability-matrix.md`
- generator: `cargo run -p xtask -- capability-matrix`
- semantics owner: `docs/specs/unified-agent-api/capabilities-schema-spec.md`

This artifact is real and reproducible. It is not hand-written.

## Observed Current State

### Codex parity state

Observed facts:

- `cli_manifests/codex/current.json` is a schema-v2 union snapshot, not schema-v1.
- `current.json` has `complete: false`.
- required target is Linux only in current committed state.
- `latest_validated.txt` is `0.97.0`.
- `pointers/latest_validated/x86_64-unknown-linux-musl.txt` is `0.97.0`.
- macOS and Windows `latest_validated` pointers are `none`.
- `versions/0.61.0.json`, `versions/0.92.0.json`, and `versions/0.97.0.json` are all `validated`.
- later Codex versions claim Linux support in metadata.

Concrete file pointers:

- `cli_manifests/codex/current.json`
- `cli_manifests/codex/latest_validated.txt`
- `cli_manifests/codex/pointers/latest_validated/x86_64-unknown-linux-musl.txt`
- `cli_manifests/codex/pointers/latest_validated/aarch64-apple-darwin.txt`
- `cli_manifests/codex/pointers/latest_validated/x86_64-pc-windows-msvc.txt`
- `cli_manifests/codex/versions/0.61.0.json`
- `cli_manifests/codex/versions/0.92.0.json`
- `cli_manifests/codex/versions/0.97.0.json`

### Claude Code parity state

Observed facts:

- `cli_manifests/claude_code/current.json` is a complete union snapshot.
- `current.json` has `complete: true`.
- all three targets have `latest_validated` and `latest_supported` pointers at `2.1.29`.
- `versions/2.1.29.json` still says `status: "validated"`.
- metadata says all three targets are passed/supported.

Concrete file pointers:

- `cli_manifests/claude_code/current.json`
- `cli_manifests/claude_code/latest_validated.txt`
- `cli_manifests/claude_code/pointers/latest_validated/linux-x64.txt`
- `cli_manifests/claude_code/pointers/latest_validated/darwin-arm64.txt`
- `cli_manifests/claude_code/pointers/latest_validated/win32-x64.txt`
- `cli_manifests/claude_code/pointers/latest_supported/linux-x64.txt`
- `cli_manifests/claude_code/pointers/latest_supported/darwin-arm64.txt`
- `cli_manifests/claude_code/pointers/latest_supported/win32-x64.txt`
- `cli_manifests/claude_code/versions/2.1.29.json`

### Unified Agent API capability state

Observed facts:

- the capability matrix is generated from built-in backend capability advertising
- generation uses canonical Linux targets:
  - Codex: `x86_64-unknown-linux-musl`
  - Claude Code: `linux-x64`
- the artifact is intentionally not exhaustive
- it is a maintenance/overview artifact, not runtime truth

Key references:

- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/specs/unified-agent-api/capability-matrix.md`
- `crates/xtask/src/capability_matrix.rs`

## Step 0 Scope Challenge

### What existing code already solves most of the problem

The repo already solves most of the hard mechanics:

- upstream version acquisition and pinning
- snapshot generation
- union generation
- wrapper-coverage generation
- coverage diff generation
- per-version metadata materialization
- deterministic validation of pointer/current/report consistency
- human-readable capability publication

That means the next plan should not be a new system. It should be a consolidation plan.

### Minimum useful change set

The smallest high-value scope is:

1. define a single semantic model for `validated` vs `supported`
2. remove manifest/prose drift so committed state and docs agree
3. generate a structured support-matrix artifact from existing manifest/version metadata
4. publish a human-readable matrix from that structured artifact in the spec tree
5. add validation so status/pointer drift cannot recur silently

Anything bigger than that is likely overbuilt for a first pass.

### Complexity smell to avoid

Do not start with:

- a new runtime API
- a new storage system
- a rewrite of the parity pipeline
- a second parallel source of truth under `docs/specs/unified-agent-api/`

The repo already has one good evidence layer. Use it.

## Findings

### 1. The parity pipeline is real, but the prose still describes large parts of it as future work

This is the most obvious documentation problem.

Examples:

- `cli_manifests/codex/README.md` still labels coverage reports, version metadata, and per-target pointers as planned.
- `cli_manifests/claude_code/README.md` does the same.
- Codex README still contains language that implies the generator emits schema-v1 `current.json`, while the committed `current.json` is schema-v2 union.

Why it matters:

- maintainers and future agents will misread what is authoritative
- planning will overestimate how much net-new automation is needed
- support-matrix work may duplicate machinery that already exists

Key pointers:

- `cli_manifests/codex/README.md`
- `cli_manifests/claude_code/README.md`
- `cli_manifests/codex/current.json`

### 2. `validated` and `supported` are defined, but not used consistently

The manifest READMEs and metadata schemas distinguish:

- `validated`
- `supported`

But the committed data does not use those states consistently.

Codex inconsistency:

- Linux `latest_supported` still points to `0.61.0`
- newer versions already claim Linux support in `versions/<v>.json`

Claude inconsistency:

- all `latest_supported/*` pointers are advanced to `2.1.29`
- `versions/2.1.29.json` still says `status: "validated"`

Why it matters:

- a support matrix generated today would inherit contradictory semantics
- automation cannot safely decide when to advance `latest_supported`
- status reporting is not mechanically trustworthy yet

Key pointers:

- `cli_manifests/codex/pointers/latest_supported/x86_64-unknown-linux-musl.txt`
- `cli_manifests/codex/versions/0.92.0.json`
- `cli_manifests/codex/versions/0.97.0.json`
- `cli_manifests/claude_code/pointers/latest_supported/linux-x64.txt`
- `cli_manifests/claude_code/versions/2.1.29.json`

### 3. The unified-agent-api capability matrix is not a version-support matrix

Current matrix semantics:

- union of advertised capability ids
- built from built-in backend configs
- generated against canonical Linux targets
- not exhaustive

That is useful, but it does not answer:

- which upstream CLI versions are supported?
- on which targets?
- under which config gates?
- for which run flows?

This is the core architectural mismatch.

Why it matters:

- if support-matrix work is forced directly into the current capability matrix, it will become muddy fast
- capability presence and upstream-version support are related, but not the same dimension

Key pointers:

- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/specs/unified-agent-api/capability-matrix.md`
- `crates/xtask/src/capability_matrix.rs`

### 4. Manifest data already contains most of the raw inputs needed for an automated support matrix

The necessary inputs already exist:

- per-version workflow status
- per-version supported targets
- per-version validation result sets
- per-target latest supported pointers
- per-target latest validated pointers
- union completeness
- coverage reports that say whether a version is missing anything

This strongly suggests the support matrix should be generated from existing metadata, not manually authored.

Why it matters:

- the next plan can stay incremental
- no need to invent a new evidence format
- this can likely be done with a generator plus a validator extension

Key pointers:

- `cli_manifests/codex/versions/*.json`
- `cli_manifests/claude_code/versions/*.json`
- `cli_manifests/codex/pointers/**`
- `cli_manifests/claude_code/pointers/**`
- `crates/xtask/src/codex_version_metadata.rs`

### 5. Codex multi-target support is pinned at the artifact layer but not realized at the snapshot/report layer

Codex `artifacts.lock.json` contains macOS and Windows pins, but current committed snapshots/reports remain Linux-only and `current.json` is still incomplete.

Possible interpretations:

- intentional Linux-first policy
- unfinished CI realization
- blocked workflow coverage

This needs an explicit answer before matrix semantics are pinned.

Why it matters:

- support-matrix rows need to distinguish "not attempted yet" from "unsupported" from "intentionally best-effort only"

Key pointers:

- `cli_manifests/codex/artifacts.lock.json`
- `cli_manifests/codex/current.json`
- `cli_manifests/codex/CI_WORKFLOWS_PLAN.md`

### 6. There are concrete doc/config correctness bugs that should be fixed before a matrix generator is trusted

Examples gathered during this session:

- Claude `RULES.json` compatibility text still references the Codex Linux triple name instead of `linux-x64`.
- Codex docs disagree about validator CLI flag naming:
  - validator spec says `--codex-dir`
  - runbook uses `--root`

Why it matters:

- these are small bugs, but they show the current semantics are not tightly enforced at the prose layer
- support-matrix generation should not be layered on top of already-confused definitions

Key pointers:

- `cli_manifests/claude_code/RULES.json`
- `cli_manifests/codex/VALIDATOR_SPEC.md`
- `cli_manifests/codex/CI_AGENT_RUNBOOK.md`

### 7. The repo has strong deterministic contracts, but they are Codex-first and parity-first, not cross-agent support-matrix first

The best-specified parts of the repo are:

- Codex wrapper coverage
- parser contracts
- report generation
- pointer validation

The cross-agent support-matrix concept is present only indirectly.

The MCP management spec is the closest thing to a first-class support matrix model because it already discusses:

- target-dependent capability availability
- config-gated operations
- built-in default advertising posture

Why it matters:

- the next plan should reuse those patterns rather than inventing a new conceptual model from scratch

Key pointers:

- `docs/specs/unified-agent-api/mcp-management-spec.md`
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`

## Architecture Implications

### Recommendation: keep one evidence layer

Best current architectural boundary:

- `cli_manifests/**` remains the evidence and version-tracking layer
- `docs/specs/unified-agent-api/**` defines semantics and publishes generated views

Avoid:

- moving support truth into Markdown prose
- maintaining parallel support ledgers in both trees
- generating one matrix from runtime capability advertising and another from manifest state

### Recommendation: separate two artifacts

There should likely be two generated artifacts, not one overloaded one:

1. capability matrix
   - backend capability advertisement view
   - close to current artifact

2. support matrix
   - upstream version x target x config-state x support-state view
   - generated from manifest/version metadata

This keeps the concepts clean.

### Recommendation: support matrix should be structured first, Markdown second

Human-readable Markdown is not enough for this job.

Preferred flow:

```text
manifest/version metadata
        |
        v
generated JSON support matrix
        |
        +--> validator checks
        |
        +--> generated Markdown summary in docs/specs/unified-agent-api/
```

Why:

- JSON can carry status axes without ambiguity
- Markdown can remain the reviewer-friendly projection
- later automation can consume the JSON directly

## Test / Validation Implications

The repo already has the right instinct: deterministic generation plus validator enforcement.

The next plan should extend that model, not replace it.

Minimum new validation likely needed:

1. `versions/<v>.json.status` must agree with the pointer state semantics
2. `latest_supported/*` advancement must be mechanically derivable or mechanically blocked
3. support-matrix JSON must be generated, sorted, and schema-validated
4. generated Markdown must be reproducible from the JSON source
5. stale prose should not be allowed to contradict live artifact semantics

Possible test surfaces:

- xtask unit tests for support-state derivation
- fixture tests for generated support-matrix JSON
- golden test for generated Markdown output
- validator tests for inconsistent status/pointer combinations

## What Already Exists That Should Be Reused

Reuse these directly:

- manifest pointer model
- version metadata schema and generator
- report generation pipeline
- validator pattern and mechanical fix mode
- capability-matrix generator and publication pattern
- parity triad scaffolding pattern for future automation work

Concrete reuse targets:

- `crates/xtask/src/codex_version_metadata.rs`
- `crates/xtask/src/codex_validate.rs`
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/parity_triad_scaffold.rs`

## Open Questions Requiring Explicit Decisions

These should be answered in planning before implementation starts.

### Q1. What exactly does `supported` mean?

Candidates:

- `supported` means "required target only, no missing coverage gaps"
- `supported` means "all expected targets supported"
- `supported` means "safe to advertise in docs"

This must be pinned. The current repo state uses all three ideas interchangeably.

### Q2. What is the intended relationship between `versions/<v>.json.status` and `latest_supported/*` pointers?

Possible rules:

- advancing `latest_supported/*` requires `status == supported`
- pointer advancement derives `status == supported`
- `status` remains workflow-local and pointers are the only published truth

Current repo state does not make this deterministic.

### Q3. For Codex, are missing macOS/Windows snapshots policy or implementation debt?

Need a clear answer:

- intentionally Linux-first for now
- best-effort targets not yet automated
- broken/unrealized workflow that should be completed

Matrix semantics depend on this.

### Q4. Should support-matrix publication live under `docs/specs/unified-agent-api/` or under `cli_manifests/`?

Recommended answer:

- JSON source under generated/artifact tooling, derived from manifest layer
- published Markdown summary under `docs/specs/unified-agent-api/`

But this should still be chosen explicitly.

### Q5. Does the unified-agent-api contract need a runtime metadata API?

Only needed if consumers need live upstream-version coverage at runtime.

If the matrix is only for docs/release engineering:

- no runtime API is needed
- keep it as an offline/generated artifact

## Proposed Planning Boundary

Recommended first planning slice:

### Slice A. Semantic cleanup

- pin exact semantics of `validated` vs `supported`
- pin exact relationship between version metadata status and pointer advancement
- pin Codex multi-target posture for near-term planning

### Slice B. Source-of-truth cleanup

- remove stale "planned" language from manifest docs
- fix incorrect target-name and CLI-flag references
- make the docs reflect the current implemented pipeline

### Slice C. Generated support matrix

- define structured support-matrix schema
- generate it from existing manifest/version metadata
- generate Markdown projection in `docs/specs/unified-agent-api/`
- document its semantics and generation rules

### Slice D. Validation hardening

- add validator checks for status/pointer consistency
- add tests/goldens for support-matrix generation

## Parallelization Opportunities

This work splits cleanly into at least three lanes:

| Lane | Modules touched | Depends on |
|------|-----------------|------------|
| Semantics | `docs/specs/unified-agent-api/`, `cli_manifests/*/README.md`, `cli_manifests/*/RULES.json` | — |
| Generator | `crates/xtask/src/` | semantic decisions |
| Validation | `crates/xtask/src/codex_validate.rs`, tests | semantic decisions, partial generator output |

Suggested order:

- Launch semantics/documentation clarification first.
- Once `supported` semantics are pinned, run generator and validation lanes in parallel.

Conflict risks:

- `RULES.json` and spec docs are likely shared touchpoints.
- `crates/xtask/src/` will need coordination if generator and validation work are split too finely.

## NOT In Scope

The following should stay out of the first support-matrix planning slice unless a later decision pulls them in:

- changing runtime backend APIs
- new network/download behavior in wrapper crates
- redesigning the parity snapshot format
- replacing the existing coverage-report pipeline
- solving every multi-target Codex CI gap in the same change
- refactoring parser contracts unrelated to support-state publication

## Appendix A: Key File Pointers

### Manifest roots

- `cli_manifests/codex/README.md`
- `cli_manifests/claude_code/README.md`
- `cli_manifests/codex/RULES.json`
- `cli_manifests/claude_code/RULES.json`
- `cli_manifests/codex/VALIDATOR_SPEC.md`
- `cli_manifests/claude_code/VALIDATOR_SPEC.md`
- `cli_manifests/codex/OPS_PLAYBOOK.md`
- `cli_manifests/claude_code/OPS_PLAYBOOK.md`
- `cli_manifests/codex/CI_WORKFLOWS_PLAN.md`
- `cli_manifests/claude_code/CI_WORKFLOWS_PLAN.md`
- `cli_manifests/codex/CI_AGENT_RUNBOOK.md`
- `cli_manifests/claude_code/CI_AGENT_RUNBOOK.md`

### Live state

- `cli_manifests/codex/current.json`
- `cli_manifests/claude_code/current.json`
- `cli_manifests/codex/versions/*.json`
- `cli_manifests/claude_code/versions/*.json`
- `cli_manifests/codex/pointers/**`
- `cli_manifests/claude_code/pointers/**`
- `cli_manifests/codex/reports/**`
- `cli_manifests/claude_code/reports/**`

### Spec tree

- `docs/specs/unified-agent-api/README.md`
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/specs/unified-agent-api/capability-matrix.md`
- `docs/specs/unified-agent-api/mcp-management-spec.md`
- `docs/specs/codex-wrapper-coverage-generator-contract.md`
- `docs/specs/codex-wrapper-coverage-scenarios-v1.md`
- `docs/specs/wrapper-events-ingestion-contract.md`
- `docs/specs/codex-thread-event-jsonl-parser-contract.md`
- `docs/specs/codex-thread-event-jsonl-parser-scenarios-v1.md`
- `docs/specs/claude-stream-json-parser-contract.md`
- `docs/specs/claude-stream-json-parser-scenarios-v1.md`
- `docs/specs/codex-app-server-jsonrpc-contract.md`
- `docs/specs/codex-streaming-exec-contract.md`
- `docs/specs/codex-external-sandbox-mapping-contract.md`
- `docs/specs/claude-code-session-mapping-contract.md`

### ADRs with direct relevance

- `docs/adr/0001-codex-cli-parity-maintenance.md`
- `docs/adr/0003-wrapper-coverage-auto-generation.md`
- `docs/adr/0004-wrapper-coverage-iu-subtree-inheritance.md`
- `docs/adr/0023-cli-manifest-coverage-for-headless-automation.md`

### Tooling

- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/capability_matrix_audit.rs`
- `crates/xtask/src/codex_report.rs`
- `crates/xtask/src/codex_version_metadata.rs`
- `crates/xtask/src/codex_validate.rs`
- `crates/xtask/src/parity_triad_scaffold.rs`
- `scripts/check_publish_readiness.py`
- `scripts/validate_publish_versions.py`

## Appendix B: Session-Specific High-Signal Observations

- `cargo run -p xtask -- capability-matrix` succeeds locally and reproduces the committed matrix shape.
- Claude manifest docs contain at least one cross-pollinated Codex target-name reference.
- Both manifest roots already have enough structured data to support automated support-matrix generation without introducing a new evidence store.
- The main missing ingredient is semantic clarity, not raw data collection.
