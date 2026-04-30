# PLAN - UAA-0022 Runtime Follow-On Review Contract Widening

Status: ready for implementation
Date: 2026-04-30
Branch: `codex/recommend-next-agent`
Base branch: `main`
Repo: `atomize-hq/unified-agent-api`
Work item: `uaa-0022`

## Source Inputs

- Existing design artifact, still sufficient:
  - `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-design-20260429-131949.md`
- Gap report driving this refresh:
  - `docs/backlog/uaa-0022-runtime-follow-on-narrowness-report.md`
- Original milestone definition and intent:
  - `docs/backlog/uaa-0022-runtime-follow-on-codex-runner.md`
- Live runtime-follow-on implementation:
  - `crates/xtask/src/runtime_follow_on.rs`
  - `crates/xtask/src/runtime_follow_on/models.rs`
  - `crates/xtask/src/runtime_follow_on/render.rs`
  - `crates/xtask/templates/runtime_follow_on_codex_prompt.md`
  - `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
  - `crates/xtask/tests/fixtures/fake_codex.sh`
- Canonical procedure and contracts:
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
  - `docs/specs/cli-agent-onboarding-charter.md`
  - `docs/adr/0013-agent-api-backend-harness.md`
- Existing queue context:
  - `TODOS.md`

## Outcome

Keep the shipped runtime-follow-on lane exactly where it is on control and ownership, then widen only the thin part:

1. add a machine-validated implementation summary that tells reviewers what actually landed
2. make the richer summary part of `handoff.json`, not just prose in `run-summary.md`
3. keep publication refresh as a separate later lane

This is not a new runtime runner. It is the missing reviewer-facing contract on the runner that already exists.

## Problem Statement

`xtask runtime-follow-on` now does the hard boring part correctly:

- it owns the Codex execution step
- it freezes the dry-run packet
- it rejects out-of-bounds writes
- it requires real runtime-owned output changes
- it validates a minimum handoff into publication refresh

That closed the execution seam.

What is still too thin is the review surface after a successful run. Today the repo can tell that Codex stayed in bounds and wrote something real. It still cannot tell, in a crisp machine-checked way:

- what tier actually landed
- which template lineage was used
- whether richer surfaces were intentionally deferred
- why a minimal result is acceptable when minimal was requested
- whether the publication lane is merely required or actually ready

The current artifacts prove control. They do not yet give maintainers the stronger summary language the plan originally called for.

## Scope Lock

### In scope

- Enrich the runtime lane's machine-readable summary contract.
- Extend `handoff.json` so publication refresh receives richer runtime context.
- Generate `run-summary.md` from validated structured data instead of thin ad hoc prose.
- Extend `run-status.json` only where it helps operators and review tooling.
- Tighten runner validation around tier reporting, template lineage, deferred rich surfaces, and publication readiness.
- Add regression tests for the new schema and failure modes.
- Update the operator guide so the shipped procedure matches the stronger contract.

### Out of scope

- Replacing `xtask runtime-follow-on` with a different host surface.
- Reopening the runtime/publication ownership boundary.
- Auto-grading backend quality by reading Rust semantics or AST shape.
- Folding publication refresh, support/capability publication, or proving-run closeout into this command.
- Introducing a new crate, new binary, or new external service.
- Creating a new design doc. The existing design artifact already covers the feature seam well enough.

## Step 0 Scope Challenge

### What already exists

| Sub-problem | Existing surface to reuse | Reuse decision |
| --- | --- | --- |
| Runner entrypoint and frozen packet | `crates/xtask/src/runtime_follow_on.rs` | Reuse directly. Do not create a second runtime lane. |
| Runtime contract models | `crates/xtask/src/runtime_follow_on/models.rs` | Extend in place. Keep one typed schema surface. |
| Human-readable run artifact | `crates/xtask/src/runtime_follow_on/render.rs` | Reuse, but drive it from validated structured fields. |
| Prompt ownership | `crates/xtask/templates/runtime_follow_on_codex_prompt.md` | Reuse. Expand the required summary/handoff instructions there. |
| Boundary and manifest validation | `validate_write_mode`, `validate_handoff` in `crates/xtask/src/runtime_follow_on.rs` | Reuse. Add semantic checks instead of parallel validators. |
| Procedural truth | `docs/cli-agent-onboarding-factory-operator-guide.md` | Reuse and update. The guide must match the stronger schema. |
| Regression harness | `crates/xtask/tests/runtime_follow_on_entrypoint.rs`, `crates/xtask/tests/fixtures/fake_codex.sh` | Reuse and extend. No new harness layer needed. |
| Deferred next lane | `TODOS.md` publication-refresh follow-on entry | Reuse. This plan should strengthen the handoff into that existing follow-on, not invent a new TODO. |

### Minimum change set

The smallest complete version of this work is:

1. add one new typed summary object to the runtime-follow-on models
2. embed that summary inside `handoff.json`
3. render `run-summary.md` from the validated summary object
4. add publication-readiness semantics beyond the current `publication_refresh_required = true`
5. add test coverage for success and schema failures
6. update the operator guide

No new subcommand. No new crate. No second summary file unless the existing artifacts prove too cramped during implementation.

### Complexity check

This should stay under the smell threshold for architecture churn even though it touches more than 8 files.

Expected file set:

- `PLAN.md`
- `crates/xtask/src/runtime_follow_on/models.rs`
- `crates/xtask/src/runtime_follow_on/render.rs`
- `crates/xtask/src/runtime_follow_on.rs`
- `crates/xtask/templates/runtime_follow_on_codex_prompt.md`
- `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
- `crates/xtask/tests/fixtures/fake_codex.sh`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

That is still one module cluster, one template, one test harness, one operator doc. Engineered enough, not sprawling.

### Search check

No new external framework or concurrency pattern is being introduced here. This is a schema-tightening pass over an internal Rust command.

- **[Layer 1]** Reuse existing `serde`-backed typed JSON models instead of inventing a looser dynamic schema layer.
- **[Layer 1]** Reuse the current dry-run/write packet flow instead of creating a second summary pipeline.
- **[Layer 1]** Reuse current entrypoint tests and fake Codex fixture instead of adding a new integration harness.

### TODOS cross-reference

`TODOS.md` already has the correct downstream follow-on:

- "Enclose The Publication Refresh Follow-On After The Runtime Runner"

This plan should make that TODO easier to execute by producing a better handoff. No new TODO is required if this plan lands as written.

### Completeness check

The shortcut version would be "add a few extra markdown bullets to `run-summary.md`."

That is not enough. It stays human-readable but still not machine-checkable. With the current runner architecture, the complete version is cheap:

- typed summary fields
- typed handoff fields
- generated markdown from the same source
- tests that lock the contract

That is a boilable lake.

### Distribution check

No new user-facing artifact type is being introduced.

This remains an internal repo workflow shipped through `xtask`. Distribution work is only:

- keep the CLI surface stable
- keep the operator guide accurate
- keep the tests green

## Decision Record

### 1. Keep one runtime lane

The normative host surface remains:

```sh
cargo run -p xtask -- runtime-follow-on --approval <path> --dry-run
cargo run -p xtask -- runtime-follow-on --approval <path> --write --run-id <id>
```

The plan strengthens artifacts emitted by that command. It does not replace the command.

### 2. Add one canonical implementation summary object

Introduce one typed summary object inside runtime-follow-on models. Do not maintain parallel freeform summary fields across files.

Proposed object:

```json
{
  "achieved_tier": "default | minimal | feature-rich",
  "primary_template": "opencode | gemini_cli | codex | claude_code",
  "template_lineage": ["opencode"],
  "landed_surfaces": [
    "wrapper_runtime",
    "backend_harness",
    "agent_api_onboarding_test",
    "wrapper_coverage_source",
    "runtime_manifest_evidence"
  ],
  "deferred_surfaces": [
    {
      "surface": "mcp_management",
      "reason": "not requested for this lane"
    }
  ],
  "minimal_tier_justification": null
}
```

This object is the review contract. `run-summary.md` should render from it, and `handoff.json` should carry it.

### 3. Strengthen `handoff.json`, do not create a second handoff format

The publication-refresh lane already expects `handoff.json`. Reuse that file and expand it.

Proposed minimum shape:

```json
{
  "agent_id": "example_agent",
  "manifest_root": "cli_manifests/example_agent",
  "runtime_lane_complete": true,
  "publication_refresh_required": true,
  "publication_refresh_ready": true,
  "required_commands": [
    "support-matrix --check",
    "capability-matrix --check",
    "capability-matrix-audit",
    "make preflight"
  ],
  "blockers": [],
  "implementation_summary": {
    "achieved_tier": "default",
    "primary_template": "opencode",
    "template_lineage": ["opencode"],
    "landed_surfaces": [
      "wrapper_runtime",
      "backend_harness",
      "agent_api_onboarding_test",
      "wrapper_coverage_source",
      "runtime_manifest_evidence"
    ],
    "deferred_surfaces": [],
    "minimal_tier_justification": null
  }
}
```

Why this shape:

- next-lane orchestration still reads one file
- maintainers still review one machine-readable handoff
- runtime semantics and publication readiness are locked together

### 4. `run-summary.md` becomes a render target, not a second source of truth

`run-summary.md` should be regenerated from the validated typed summary and validation report. Do not allow Codex-authored prose to become the canonical review summary.

That avoids drift between:

- what Codex claims
- what the validator proved
- what the next lane consumes

### 5. Semantic validation stays explicit and shallow on purpose

This increment should validate declared semantics, not pretend to infer them from Rust code shape.

The validator must check:

- `implementation_summary` exists
- `achieved_tier` is a known enum value
- `primary_template` is a known enum value
- `template_lineage` is non-empty and includes `primary_template`
- `achieved_tier = minimal` requires `minimal_tier_justification`
- every requested rich surface is either landed or explicitly deferred
- `publication_refresh_ready = true` only when blockers are empty and runtime lane completion checks passed

The validator should not try to prove that backend code is "truly default-tier" by analyzing source structure. That would spend an innovation token in the wrong place.

### 6. Default-tier and minimal-tier rules stay boring

- `requested_tier = default` continues to require the target onboarding test file.
- `requested_tier = minimal` continues to require explicit justification input.
- For this increment, require `implementation_summary.achieved_tier == requested_tier`.

That last rule is intentionally strict. It keeps the first semantic pass honest and simple. If the repo later wants downgrade reporting, that can be a separate follow-on.

## Architecture

### End-to-end flow

```text
approved-agent.toml + agent_registry.toml + dry-run packet
                │
                ▼
      runtime_follow_on::build_context
                │
                ├──────────────▶ input-contract.json
                ├──────────────▶ codex-prompt.md
                └──────────────▶ expected tier / template / rich-surface inputs
                                   │
                                   ▼
                            codex exec --write
                                   │
                     ┌─────────────┴─────────────┐
                     ▼                           ▼
              repo runtime writes         handoff.json
                                          implementation_summary{}
                     │                           │
                     └─────────────┬─────────────┘
                                   ▼
                         validate_write_mode()
                                   │
           ┌───────────────────────┼────────────────────────┐
           ▼                       ▼                        ▼
   boundary + manifest      summary semantics      test + handoff readiness
           │                       │                        │
           └───────────────────────┴──────────────┬─────────┘
                                                  ▼
                                 render_run_summary() from validated data
                                                  │
                                                  ▼
                    run-status.json + run-summary.md + validation-report.json
                                                  │
                                                  ▼
                               publication refresh lane consumes handoff.json
```

### Module plan

| Module | Change |
| --- | --- |
| `crates/xtask/src/runtime_follow_on/models.rs` | Add `ImplementationSummary`, `DeferredSurface`, and any minimal-justification summary shape needed in `handoff.json` / `run-status.json`. |
| `crates/xtask/src/runtime_follow_on.rs` | Extend dry-run defaults, parse richer handoff payload, validate semantic fields, compute publication readiness rules, and keep current boundary checks untouched. |
| `crates/xtask/src/runtime_follow_on/render.rs` | Render `run-summary.md` from the typed summary object plus the validation report. |
| `crates/xtask/templates/runtime_follow_on_codex_prompt.md` | Tell Codex exactly which summary fields and handoff semantics it must populate. |
| `crates/xtask/tests/runtime_follow_on_entrypoint.rs` | Add success/failure tests for richer summary semantics. |
| `crates/xtask/tests/fixtures/fake_codex.sh` | Emit the richer `handoff.json` for fixture scenarios. |
| `docs/cli-agent-onboarding-factory-operator-guide.md` | Update required scratch artifacts and handoff semantics. |

### Data contract details

#### `input-contract.json`

Extend the packet just enough to validate the richer output:

- `requested_tier`
- `allow_rich_surface`
- `required_agent_api_test`
- `expected_default_surfaces`
- `known_template_ids`

This keeps validation deterministic without introducing another config file.

#### `run-status.json`

Keep it operator-focused. Add only small mirrored fields that help dashboards and replay:

- `tier_achieved`
- `primary_template`
- `publication_refresh_ready`
- `deferred_surface_count`

Do not copy the full summary blob into `run-status.json` unless implementation pressure proves that duplication is worth it.

#### `run-summary.md`

Required sections:

1. Outcome
2. Implementation summary
3. Validation checks
4. Deferred surfaces
5. Publication refresh handoff
6. Written paths
7. Errors, when present

This should read like a maintainer review note, but every substantive claim comes from validated fields.

## Code Quality Review

### Opinionated recommendations

1. Keep one typed schema surface.
   Recommendation: add summary structs to `models.rs` and route every artifact through them.
   Why: DRY matters here. A second ad hoc JSON writer or markdown-only schema is how this plan regresses in two months.

2. Do not let `render.rs` invent semantics.
   Recommendation: `render.rs` formats validated data only.
   Why: explicit over clever. Validators decide truth, renderers only present it.

3. Avoid a new `implementation-summary.json` file unless forced.
   Recommendation: start with richer `handoff.json` plus generated `run-summary.md`.
   Why: minimal diff. One more artifact means one more thing to drift, test, and explain.

4. Keep enum vocabulary repo-owned and small.
   Recommendation: normalize template ids and surface ids in code, not freeform strings from Codex.
   Why: review tooling gets worse fast when prompt-authored strings become API surface.

## Test Review

100 percent coverage for the new contract is realistic here because the surface is a bounded Rust command with an existing fixture harness.

### CODE PATH COVERAGE

```text
CODE PATH COVERAGE
===========================
[+] crates/xtask/src/runtime_follow_on/models.rs
    │
    ├── ImplementationSummary serde round-trip
    │   ├── [GAP] valid default summary parses and serializes
    │   ├── [GAP] minimal summary requires justification payload
    │   └── [GAP] unknown template / surface values fail validation
    │
[+] crates/xtask/src/runtime_follow_on.rs
    │
    ├── validate_handoff()
    │   ├── [★★ TESTED] missing required legacy field fails
    │   ├── [GAP] missing implementation_summary fails
    │   ├── [GAP] publication_refresh_ready=true with blockers fails
    │   ├── [GAP] achieved_tier != requested_tier fails
    │   └── [GAP] requested rich surface neither landed nor deferred fails
    │
    ├── validate_write_mode()
    │   ├── [★★★ TESTED] out-of-bounds path rejection
    │   ├── [★★★ TESTED] generated wrapper_coverage.json rejection
    │   ├── [★★ TESTED] missing required default-tier test fails
    │   └── [GAP] validated summary fields feed run-summary render
    │
[+] crates/xtask/src/runtime_follow_on/render.rs
    │
    ├── render_run_summary()
    │   ├── [GAP] success summary shows achieved tier / template / deferred surfaces
    │   └── [GAP] failure summary still shows partial semantic context
```

### USER FLOW COVERAGE

```text
USER FLOW COVERAGE
===========================
[+] Dry-run to write success
    │
    ├── [★★ TESTED] packet prepares and write validates
    └── [GAP] [→INTEG] successful run emits rich handoff + rich markdown summary

[+] Minimal-tier exception run
    │
    ├── [★★ TESTED] minimal without justification is rejected at arg validation
    └── [GAP] [→INTEG] minimal with justification produces structured exception summary

[+] Feature-rich opt-in run
    │
    ├── [GAP] [→INTEG] requested rich surface marked as landed passes
    └── [GAP] [→INTEG] requested rich surface omitted without defer reason fails

[+] Publication handoff review
    │
    ├── [★★ TESTED] required_commands presence
    └── [GAP] publication_refresh_ready semantics are enforced

─────────────────────────────────
COVERAGE: 5/14 paths tested today
GAPS: 9 paths need tests
  - integration-style command tests: 7
  - render-focused unit tests: 2
─────────────────────────────────
```

### Required tests to add

1. `runtime_follow_on_write_accepts_rich_implementation_summary`
   - File: `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
   - Assert: success scenario writes `implementation_summary`, `publication_refresh_ready = true`, and `run-summary.md` includes achieved tier, template lineage, and deferred-surface section.

2. `runtime_follow_on_write_rejects_missing_implementation_summary`
   - File: `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
   - Assert: write fails when `handoff.json` omits the summary object.

3. `runtime_follow_on_write_rejects_publication_ready_with_blockers`
   - File: `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
   - Assert: contradictory readiness fields fail validation.

4. `runtime_follow_on_write_rejects_tier_mismatch`
   - File: `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
   - Assert: `achieved_tier != requested_tier` fails for this increment.

5. `runtime_follow_on_write_rejects_unaccounted_rich_surface`
   - File: `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
   - Assert: requested rich surface must be in `landed_surfaces` or `deferred_surfaces`.

6. `render_run_summary_renders_validated_semantic_fields`
   - File: either a new focused unit test alongside `render.rs` or a command test if that is simpler in this repo
   - Assert: markdown output is deterministic from the typed summary.

### Failure modes registry

| Codepath | Realistic failure | Test cover required | Error handling required | User experience if missed |
| --- | --- | --- | --- | --- |
| `validate_handoff` | Codex writes legacy handoff without new summary fields | yes | yes, hard fail | silent semantic downgrade in review |
| publication readiness logic | handoff says ready while blockers still exist | yes | yes, hard fail | next lane starts from a false green state |
| rich surface accounting | requested feature-rich surface disappears from output summary | yes | yes, hard fail | reviewers think omission was intentional when it was not |
| render path | markdown omits deferred surfaces even though JSON captured them | yes | yes, deterministic render | human reviewer misses the real scope cut |
| minimal-tier summary | justification exists in input but is not carried into output summary | yes | yes, hard fail | exception ships with no rationale |

Critical gap rule: any path that loses semantic context without failing validation is a critical gap. This plan removes those.

## Performance Review

This is a tiny performance surface, but there are still two boring rules worth locking in:

1. Parse `handoff.json` once.
   Recommendation: validate and render from the same typed object.
   Why: no duplicate parse/transform branches.

2. Keep semantic checks linear.
   Recommendation: validate surfaces and templates with set membership, not extra filesystem scans.
   Why: the runtime lane already does enough IO. The summary layer should be cheap.

No caching, concurrency, or new background work is justified here.

## Workstreams

### Workstream 1 - Schema and validation

- extend runtime-follow-on typed models
- extend `validate_handoff`
- extend `validate_write_mode`
- add publication readiness rules

### Workstream 2 - Rendering and prompt contract

- update prompt template
- update markdown renderer
- keep run status aligned

### Workstream 3 - Test harness and docs

- extend fake Codex fixture
- add entrypoint regressions
- update operator guide

## Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Define implementation summary schema | `crates/xtask/src/runtime_follow_on*` | — |
| Enforce richer handoff validation | `crates/xtask/src/runtime_follow_on*` | Define implementation summary schema |
| Render richer run summary | `crates/xtask/src/runtime_follow_on/render.rs` | Define implementation summary schema |
| Extend fake Codex fixture and entrypoint tests | `crates/xtask/tests/**` | Enforce richer handoff validation |
| Update operator guide | `docs/` | Define implementation summary schema |

## Parallel lanes

- Lane A: schema definition -> validator changes -> renderer changes
- Lane B: operator-guide updates after schema freeze
- Lane C: fake Codex fixture -> runtime-follow-on entrypoint tests, after validator expectations are stable

Execution order:

1. Launch Lane A first. It owns the contract.
2. Once the JSON shape is stable, Lane B and Lane C can run in parallel.
3. Merge B + C, then run final test and doc verification.

Conflict flags:

- Lane A and Lane C both depend on `runtime_follow_on` field names. Do not run them in parallel before the schema is frozen.

## NOT in scope

- Auto-infer achieved tier from code shape.
  Rationale: too clever for this seam, weak verification value for the effort.
- Teach publication refresh to consume more than `handoff.json`.
  Rationale: one handoff artifact is enough for this increment.
- Add support for `achieved_tier` downgrade reporting.
  Rationale: keep v1 semantics simple with equality to requested tier.
- Create or update publication-owned manifest files.
  Rationale: still the next lane.

## What already exists

- `runtime-follow-on` already freezes the input packet and owns `codex exec`.
- The validator already proves boundary ownership, generated wrapper-coverage discipline, and non-zero runtime writes.
- `run-summary.md` and `handoff.json` already exist, they are just too lean.
- `requested_tier`, minimal-justification input, and allowed rich surfaces are already present in the runtime input contract.
- The test harness already simulates success, invalid handoff, no-op runtime writes, and boundary violations.

This means the repo is not missing plumbing. It is missing summary semantics.

## Implementation Steps

1. Add typed summary structs to `models.rs`.
2. Extend dry-run artifact generation so `handoff.json` defaults include the richer shape.
3. Extend the prompt template so Codex must populate the richer handoff fields.
4. Extend `validate_handoff` with semantic checks for implementation summary and publication readiness.
5. Extend `render_run_summary` to present validated semantic fields.
6. Update the fake Codex success and failure scenarios to emit the richer handoff.
7. Add regression tests for the new success and failure modes.
8. Update the operator guide examples and required artifact descriptions.

## Completion Summary

- Step 0: Scope Challenge — scope accepted, narrowed to review-contract widening only
- Architecture Review: 4 concrete recommendations
- Code Quality Review: 4 concrete recommendations
- Test Review: diagram produced, 9 gaps identified
- Performance Review: 2 concrete recommendations
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 new items needed
- Failure modes: 0 unresolved critical gaps if the required tests land
- Outside voice: skipped, not needed for this backend-only contract-tightening pass
- Parallelization: 3 workstreams, 2 follow after schema freeze
- Lake Score: 6/6 recommendations chose the complete option

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 0 | — | — |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | — | — |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 1 | issues_open | focus narrowed to reviewer-summary contract widening; test and schema gaps enumerated in this plan |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | skipped | backend-only scope, existing design artifact remains sufficient |

**VERDICT:** ENG REVIEW COMPLETE FOR THIS PLANNING PASS. Implement from this plan. No new design doc required.
