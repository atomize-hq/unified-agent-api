# PLAN - UAA-0022 Runtime Follow-On Review Contract Widening

Status: ready for implementation
Date: 2026-04-30
Branch: `codex/recommend-next-agent`
Base branch: `main`
Repo: `atomize-hq/unified-agent-api`
Work item: `uaa-0022`

## Objective

Keep the shipped `xtask runtime-follow-on` lane exactly where it is on execution ownership and write-boundary control, then widen the thin part that remains: the maintainer-facing review contract.

The deliverable is not a new runner. The deliverable is a stronger machine-readable summary that tells reviewers and the next lane what actually landed, what tier it landed at, what richer surfaces were deferred, and whether publication refresh is truly ready.

## Source Inputs

- Gap report: `docs/backlog/uaa-0022-runtime-follow-on-narrowness-report.md`
- Original milestone note: `docs/backlog/uaa-0022-runtime-follow-on-codex-runner.md`
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
- Queue context: `TODOS.md`

## Outcome

After this plan lands:

1. `handoff.json` carries a typed implementation summary, not just completion booleans.
2. `run-summary.md` is rendered from validated structured data, not from thin prose.
3. `run-status.json` exposes the small set of extra fields needed for dashboards and replay.
4. `validate_handoff` enforces semantic rules for tier reporting, template lineage, deferred surfaces, and publication readiness.
5. Publication refresh remains a separate lane. This plan only makes that handoff trustworthy.

## Problem Statement

The current runtime-follow-on lane proves the control seam:

- it owns the Codex execution step
- it freezes the dry-run packet
- it rejects out-of-bounds writes
- it rejects publication-owned manifest edits
- it rejects no-op write runs
- it requires a minimally valid `handoff.json`

What it does not prove yet is the reviewer-facing meaning of a successful run. Today the repo can tell that Codex stayed in bounds and changed runtime-owned files. It still cannot tell, in a machine-checked way:

- what tier actually landed
- which baseline template was used
- which richer surfaces were intentionally deferred
- why a minimal result is acceptable when `minimal` was requested
- whether the next publication lane is merely required or truly ready

That is the gap this plan closes.

## Scope

### In scope

- Add a typed implementation summary to the runtime-follow-on model layer.
- Extend `handoff.json` with semantic fields that publication refresh can trust.
- Render `run-summary.md` from validated structured fields.
- Mirror only the minimal extra status fields into `run-status.json`.
- Extend `validate_handoff` and `validate_write_mode` with semantic checks.
- Extend the fake Codex fixture and entrypoint tests to lock the new contract.
- Update the operator guide to match the strengthened runtime handoff.

### Out of scope

- Replacing `xtask runtime-follow-on` with a new command or host surface.
- Reopening the runtime/publication ownership boundary.
- Adding a new crate, binary, service, or scratch artifact family.
- Auto-inferring achieved tier by reading Rust semantics or AST shape.
- Merging publication refresh or proving-run closeout into runtime-follow-on.
- Rewriting the existing design doc. The current inputs are sufficient.

## Step 0 Scope Challenge

### What already exists

| Sub-problem | Existing surface to reuse | Reuse decision |
| --- | --- | --- |
| Runner entrypoint and packet lifecycle | `crates/xtask/src/runtime_follow_on.rs` | Reuse directly. Keep one runtime lane. |
| Typed JSON models | `crates/xtask/src/runtime_follow_on/models.rs` | Extend in place. Do not create a second schema surface. |
| Human-readable summary rendering | `crates/xtask/src/runtime_follow_on/render.rs` | Reuse, but drive it from validated fields. |
| Prompt ownership | `crates/xtask/templates/runtime_follow_on_codex_prompt.md` | Reuse. Expand required handoff instructions there. |
| Validation harness | `validate_write_mode`, `validate_handoff` in `crates/xtask/src/runtime_follow_on.rs` | Reuse. Add semantic checks instead of parallel validators. |
| Regression harness | `crates/xtask/tests/runtime_follow_on_entrypoint.rs`, `crates/xtask/tests/fixtures/fake_codex.sh` | Reuse and extend. No new harness layer. |
| Live operator procedure | `docs/cli-agent-onboarding-factory-operator-guide.md` | Reuse and update. The guide must match the strengthened contract. |
| Downstream follow-on | `TODOS.md` entry "Enclose The Publication Refresh Follow-On After The Runtime Runner" | Reuse. This plan feeds that TODO instead of replacing it. |

### Minimum complete change set

The smallest complete version of this work is:

1. add one canonical `ImplementationSummary` object to the runtime-follow-on model layer
2. embed that object inside `handoff.json`
3. add `publication_refresh_ready` semantics to the handoff
4. render `run-summary.md` from the validated summary object plus the validation report
5. mirror a small subset of those fields into `run-status.json`
6. add regression tests for success and failure semantics
7. update the operator guide

No new subcommand. No second handoff format. No standalone `implementation-summary.json`.

### Complexity check

This work touches one Rust module cluster, one prompt template, one test harness, and one operator doc set. That is comfortably inside "engineered enough" territory.

Expected touched files:

- `PLAN.md`
- `crates/xtask/src/runtime_follow_on.rs`
- `crates/xtask/src/runtime_follow_on/models.rs`
- `crates/xtask/src/runtime_follow_on/render.rs`
- `crates/xtask/templates/runtime_follow_on_codex_prompt.md`
- `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
- `crates/xtask/tests/fixtures/fake_codex.sh`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

### TODOS cross-reference

`TODOS.md` already contains the correct downstream item for publication refresh. No new TODO is required. This plan should make that existing follow-on easier by handing it a richer and stricter `handoff.json`.

### Completeness decision

The shortcut version would be "add more prose to `run-summary.md`."

That is not acceptable. It would improve readability but not enforceability. The complete version is still cheap here because the runner already has a typed model layer and a test harness. This is a boilable lake:

- typed fields
- typed validation
- generated markdown from the same source
- regression tests that lock behavior

## Architecture

### Current to target flow

```text
approved-agent.toml + agent_registry.toml + dry-run packet
                │
                ▼
      runtime_follow_on::build_context
                │
                ├──────────────▶ input-contract.json
                ├──────────────▶ codex-prompt.md
                └──────────────▶ requested tier / allowed rich surfaces / required test
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
   boundary + manifest      summary semantics      test + publication readiness
           │                       │                        │
           └───────────────────────┴──────────────┬─────────┘
                                                  ▼
                                 render_run_summary() from validated data
                                                  │
                                                  ▼
                    run-status.json + run-summary.md + validation-report.json
                                                  │
                                                  ▼
                          publication refresh consumes the same handoff.json
```

### Canonical model changes

All new semantics live in `crates/xtask/src/runtime_follow_on/models.rs`.

Add these Rust types:

- `AchievedTier` enum with exactly: `default`, `minimal`, `feature-rich`
- `TemplateId` enum with exactly: `opencode`, `gemini_cli`, `codex`, `claude_code`
- `SurfaceId` enum with exactly:
  - `wrapper_runtime`
  - `backend_harness`
  - `agent_api_onboarding_test`
  - `wrapper_coverage_source`
  - `runtime_manifest_evidence`
  - `add_dirs`
  - `external_sandbox_policy`
  - `mcp_management`
  - `richer_session_runtime`
- `DeferredSurface { surface: SurfaceId, reason: String }`
- `ImplementationSummary`

`ImplementationSummary` shape:

```json
{
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
  "deferred_surfaces": [
    {
      "surface": "mcp_management",
      "reason": "not approved for this runtime lane"
    }
  ],
  "minimal_tier_justification": null
}
```

Serialization rules for every new enum in this section:

- Use `serde` string enums with `kebab-case` values.
- JSON must never contain numeric discriminants.
- Validation failures must name the offending field.

### Exact artifact contract changes

#### `InputContract`

Add these fields:

- `expected_default_surfaces: Vec<SurfaceId>`
- `known_template_ids: Vec<TemplateId>`
- `known_rich_surfaces: Vec<SurfaceId>`

Populate them in `build_context`. Do not derive them from prompt text at validation time.

`expected_default_surfaces` must contain exactly:

- `wrapper_runtime`
- `backend_harness`
- `agent_api_onboarding_test`
- `wrapper_coverage_source`
- `runtime_manifest_evidence`

`known_template_ids` must contain exactly:

- `opencode`
- `gemini_cli`
- `codex`
- `claude_code`

`known_rich_surfaces` must contain exactly:

- `add_dirs`
- `external_sandbox_policy`
- `mcp_management`
- `richer_session_runtime`

#### `HandoffContract`

Keep existing fields and add exactly:

- `publication_refresh_ready: bool`
- `implementation_summary: ImplementationSummary`

Dry-run `handoff.json` must still be emitted, but it must now contain:

- `runtime_lane_complete = false`
- `publication_refresh_required = true`
- `publication_refresh_ready = false`
- `blockers = ["Pending runtime follow-on implementation."]`
- a placeholder `implementation_summary` with this exact shape:

```json
{
  "achieved_tier": "<requested_tier>",
  "primary_template": "opencode",
  "template_lineage": [],
  "landed_surfaces": [],
  "deferred_surfaces": [],
  "minimal_tier_justification": null
}
```

The dry-run placeholder is intentionally parseable but not semantically sufficient for write-mode success.

Successful write-mode `handoff.json` must contain all legacy fields plus the new fields in the same object. No second machine-readable summary file is allowed.

#### `RunStatus`

Add only these mirrored fields:

- `tier_achieved: Option<AchievedTier>`
- `primary_template: Option<TemplateId>`
- `publication_refresh_ready: bool`
- `deferred_surface_count: Option<usize>`

`RunStatus` remains operator-focused. It must not duplicate the full summary object.

Field population rules:

- dry-run:
  - `tier_achieved = null`
  - `primary_template = null`
  - `publication_refresh_ready = false`
  - `deferred_surface_count = null`
- write success:
  - `tier_achieved = implementation_summary.achieved_tier`
  - `primary_template = implementation_summary.primary_template`
  - `publication_refresh_ready = handoff.publication_refresh_ready`
  - `deferred_surface_count = implementation_summary.deferred_surfaces.len()`
- write failure:
  - `tier_achieved = null`
  - `primary_template = null`
  - `publication_refresh_ready = false`
  - `deferred_surface_count = null`

#### `ValidationReport`

Keep the current shape. Add new `ValidationCheck` entries; do not add a new report format.

Required new check names:

- `implementation_summary_present`
- `implementation_summary_semantics`
- `publication_refresh_readiness`

`ValidationReport.status` remains:

- `prepared` for dry-run
- `pass` for write success
- `fail` for write failure

### Validation rules

`validate_handoff` becomes the semantic gate. It must enforce all of the following:

1. `implementation_summary` exists and parses into the typed model.
2. `achieved_tier` is a known enum value.
3. `primary_template` is a known enum value.
4. `template_lineage` is non-empty.
5. `template_lineage` contains `primary_template`.
6. `achieved_tier == requested_tier` for this milestone.
7. `achieved_tier = minimal` requires non-empty `minimal_tier_justification`.
8. `achieved_tier != minimal` requires `minimal_tier_justification = null`.
9. Every surface in `allow_rich_surface` is accounted for in either `landed_surfaces` or `deferred_surfaces`.
10. Every `deferred_surfaces[].reason` is non-empty after trim.
11. `publication_refresh_ready = true` is allowed only when:
    - `runtime_lane_complete = true`
    - `blockers` is empty
    - the canonical publication command set is present
12. `required_commands` must match the canonical set after normalization:
    - `support-matrix --check`
    - `capability-matrix --check`
    - `capability-matrix-audit`
    - `make preflight`
13. `landed_surfaces` must be unique.
14. `deferred_surfaces[].surface` must be unique.
15. A surface cannot appear in both `landed_surfaces` and `deferred_surfaces`.
16. Every surface in `landed_surfaces` must be either:
    - one of `expected_default_surfaces`, or
    - one of `known_rich_surfaces`
17. Every surface in `deferred_surfaces` must be one of `known_rich_surfaces`.
18. A rich surface may appear in `landed_surfaces` or `deferred_surfaces` only if it was explicitly requested through `allow_rich_surface`.
19. Tier-to-surface rules are fixed:
    - `default` requires all `expected_default_surfaces` in `landed_surfaces`
    - `minimal` requires at least:
      - `wrapper_runtime`
      - `wrapper_coverage_source`
      - `runtime_manifest_evidence`
    - `feature-rich` requires all `expected_default_surfaces` plus every surface from `allow_rich_surface` in `landed_surfaces`
20. For `minimal`, any omitted default surface must appear in `deferred_surfaces` with a non-empty reason.

Normalization rules for `required_commands`:

- Trim leading and trailing ASCII whitespace on each command.
- Drop empty strings after trim.
- Compare as a deduplicated `BTreeSet<String>`.
- Keep command strings case-sensitive.
- The normalized set must equal the canonical set exactly. Superset and subset both fail.

Surface classification rules:

- Default surfaces:
  - `wrapper_runtime`
  - `backend_harness`
  - `agent_api_onboarding_test`
  - `wrapper_coverage_source`
  - `runtime_manifest_evidence`
- Rich surfaces:
  - `add_dirs`
  - `external_sandbox_policy`
  - `mcp_management`
  - `richer_session_runtime`

Template-lineage rules:

- `primary_template` must appear exactly once in `template_lineage`.
- `template_lineage` order is meaningful: nearest baseline first, additional references after it.
- Empty `template_lineage` is allowed only in the dry-run placeholder handoff. It is forbidden in write-mode validation.

What the validator must not do:

- inspect Rust code to "prove" the landed tier
- infer template lineage from file diffs
- decide whether the implementation quality is good enough beyond the declared summary contract

This is explicit validation, not clever inference.

### Rendering rules

`render_run_summary` must render from validated structured data plus the existing validation report. It must not invent semantics and it must not parse `handoff.json` a second time.

Required sections in `run-summary.md`:

1. Outcome
2. Implementation summary
3. Validation checks
4. Deferred surfaces
5. Publication refresh handoff
6. Written paths
7. Errors, when present

Required content in "Implementation summary":

- requested tier
- achieved tier
- primary template
- template lineage
- landed surfaces
- minimal-tier justification when applicable

Required content in "Publication refresh handoff":

- `publication_refresh_required`
- `publication_refresh_ready`
- blockers
- required commands

Ordering rules for `run-summary.md`:

- `landed_surfaces` render in enum declaration order.
- `deferred_surfaces` render in enum declaration order by `surface`.
- `required_commands` render in canonical command order.
- validation checks render in the order they were appended in `validate_write_mode`.

### Prompt contract

Update `crates/xtask/templates/runtime_follow_on_codex_prompt.md` so Codex is told to write the richer `handoff.json` contract directly.

The prompt must state:

- `handoff.json` is the only machine-readable handoff artifact
- `implementation_summary` is required
- all summary values must use the repo-owned enums from the packet
- every requested rich surface must be either landed or explicitly deferred
- `publication_refresh_ready` must remain `false` when blockers exist
- `template_lineage` must be an ordered array of repo-owned template ids
- rich surfaces not approved through `allow_rich_surface` are forbidden

The prompt packet must expose the exact enum vocabularies and tier-to-surface rules above so Codex is not guessing strings.

## Implementation Plan

### Phase 1 - Schema

Files:

- `crates/xtask/src/runtime_follow_on/models.rs`
- `crates/xtask/src/runtime_follow_on.rs`

Work:

1. Add the new enums and structs.
2. Extend `InputContract`, `HandoffContract`, and `RunStatus`.
3. Seed `expected_default_surfaces`, `known_template_ids`, and `known_rich_surfaces` in `build_context`.
4. Extend dry-run `handoff.json` defaults so the placeholder shape already matches the new contract.
5. Extend prompt rendering so the enum vocabularies and tier-to-surface rules are visible in the packet.

Exit condition:

- dry-run artifacts serialize successfully with the new schema
- no write-mode behavior changes yet beyond compile-safe model updates

### Phase 2 - Semantic validation

Files:

- `crates/xtask/src/runtime_follow_on.rs`

Work:

1. Extend `validate_handoff` with the 20 rules above.
2. Record distinct `ValidationCheck` entries for summary presence, summary semantics, and publication readiness.
3. Keep all existing boundary, manifest-split, and non-zero-write checks unchanged.
4. Populate the new `RunStatus` fields only on write success.

Exit condition:

- malformed or contradictory summary semantics fail before a run can be treated as successful

### Phase 3 - Rendering and prompt alignment

Files:

- `crates/xtask/src/runtime_follow_on/render.rs`
- `crates/xtask/templates/runtime_follow_on_codex_prompt.md`

Work:

1. Make `render_run_summary` output the new maintainer-facing sections from validated data.
2. Mirror the minimal extra fields into `RunStatus`.
3. Update the prompt so fake Codex and real Codex are both aiming at the same handoff contract.
4. Keep render ordering deterministic using the ordering rules above.

Exit condition:

- summary markdown and status JSON are both deterministic projections of validated data

### Phase 4 - Regression harness

Files:

- `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
- `crates/xtask/tests/fixtures/fake_codex.sh`

Work:

1. Extend the fixture script to emit the richer success path.
2. Add explicit failure scenarios for missing summary, contradictory readiness, tier mismatch, required-command mismatch, duplicate surfaces, unauthorized rich surfaces, and unaccounted rich surfaces.
3. Add or extend tests to lock the rendered markdown output.

Exit condition:

- success path proves the richer summary contract
- every new semantic failure mode has a regression test

### Phase 5 - Docs

Files:

- `docs/cli-agent-onboarding-factory-operator-guide.md`

Work:

1. Update the required runtime scratch artifacts description.
2. Document the richer `handoff.json` contract and `publication_refresh_ready`.
3. State that publication refresh still consumes the same handoff artifact and remains the next lane.

Exit condition:

- the live operator guide matches the shipped schema and validation behavior

## Test Plan

Project test framework: Rust workspace tests under `cargo test`.

Target verification commands:

```sh
cargo test -p xtask --test runtime_follow_on_entrypoint
cargo test -p xtask runtime_follow_on
make fmt-check
make clippy
```

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/xtask/src/runtime_follow_on/models.rs
    │
    ├── ImplementationSummary serde contract
    │   ├── [GAP] valid default summary round-trips
    │   ├── [GAP] minimal summary requires justification
    │   └── [GAP] invalid enum values are rejected
    │
[+] crates/xtask/src/runtime_follow_on.rs
    │
    ├── build_context()
    │   ├── [GAP] expected_default_surfaces are populated
    │   └── [GAP] known template and rich-surface ids are populated
    │
    ├── validate_handoff()
    │   ├── [★★ TESTED] missing required legacy field fails
    │   ├── [GAP] missing implementation_summary fails
    │   ├── [GAP] publication_refresh_ready=true with blockers fails
    │   ├── [GAP] achieved_tier != requested_tier fails
    │   ├── [GAP] minimal without summary justification fails
    │   ├── [GAP] requested rich surface omitted from landed/deferred fails
    │   ├── [GAP] unapproved rich surface in landed_surfaces fails
    │   ├── [GAP] duplicate landed/deferred surfaces fail
    │   └── [GAP] required_commands set mismatch fails
    │
    ├── validate_write_mode()
    │   ├── [★★★ TESTED] out-of-bounds path rejection
    │   ├── [★★★ TESTED] generated wrapper_coverage.json rejection
    │   ├── [★★ TESTED] no-op runtime run rejection
    │   └── [GAP] successful semantic summary feeds status + markdown
    │
[+] crates/xtask/src/runtime_follow_on/render.rs
    │
    ├── render_run_summary()
    │   ├── [GAP] success summary shows achieved tier, template lineage, deferred surfaces
    │   └── [GAP] failure summary still shows partial semantic context
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] Dry-run packet preparation
    │
    ├── [★★ TESTED] bounded scratch packet is written
    └── [GAP] placeholder implementation_summary is emitted in dry-run handoff

[+] Default-tier write success
    │
    ├── [★★ TESTED] write path requires real runtime-owned changes
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
    ├── [★★ TESTED] required command presence is enforced today
    └── [GAP] [→INTEG] publication_refresh_ready semantics are enforced
```

### Required tests to add

Add these tests in `crates/xtask/tests/runtime_follow_on_entrypoint.rs` unless a narrower unit test is clearly simpler:

1. `runtime_follow_on_dry_run_emits_placeholder_implementation_summary`
2. `runtime_follow_on_write_accepts_rich_implementation_summary`
3. `runtime_follow_on_write_rejects_missing_implementation_summary`
4. `runtime_follow_on_write_rejects_publication_ready_with_blockers`
5. `runtime_follow_on_write_rejects_tier_mismatch`
6. `runtime_follow_on_write_rejects_minimal_summary_without_justification`
7. `runtime_follow_on_write_rejects_unaccounted_rich_surface`
8. `runtime_follow_on_write_rejects_unapproved_rich_surface`
9. `runtime_follow_on_write_rejects_duplicate_surface_entries`
10. `runtime_follow_on_write_rejects_required_command_set_mismatch`
11. `runtime_follow_on_write_records_status_fields_from_validated_summary`
12. `render_run_summary_renders_validated_semantic_fields`

Each test must assert exact operator-visible behavior, not just non-throw behavior.

## Failure Modes Registry

| Codepath | Realistic failure | Test required | Error handling required | Operator-visible impact if missed |
| --- | --- | --- | --- | --- |
| `validate_handoff` | Codex writes the legacy handoff shape with no `implementation_summary` | yes | hard fail | semantic downgrade looks like a valid success |
| publication readiness logic | handoff says ready while blockers still exist | yes | hard fail | publication lane starts from a false green state |
| rich surface accounting | approved rich surface disappears from the output summary | yes | hard fail | reviewers think omission was intentional |
| minimal-tier exception flow | justification exists in input but is absent from the handoff summary | yes | hard fail | exception-tier ship has no rationale |
| renderer | markdown omits deferred surfaces while JSON includes them | yes | deterministic render fix | reviewer misses the real scope cut |
| status projection | `run-status.json` claims readiness without matching handoff semantics | yes | hard fail or projection fix | dashboards show a false-ready run |

Critical gap rule: any path that loses semantic context without failing validation is a release-blocking defect for this milestone.

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Define summary enums and structs | `crates/xtask/src/runtime_follow_on/` | — |
| Populate packet defaults and known value sets | `crates/xtask/src/runtime_follow_on/` | Define summary enums and structs |
| Enforce richer handoff validation | `crates/xtask/src/runtime_follow_on/` | Populate packet defaults and known value sets |
| Render richer summary and status projection | `crates/xtask/src/runtime_follow_on/` | Define summary enums and structs |
| Extend fake Codex scenarios and entrypoint tests | `crates/xtask/tests/` | Enforce richer handoff validation |
| Update operator guide | `docs/` | Define summary enums and structs |

### Parallel lanes

- Lane A: define summary enums and structs -> populate packet defaults and known value sets -> enforce richer handoff validation -> render richer summary and status projection
- Lane B: update operator guide after Lane A freezes the schema vocabulary
- Lane C: extend fake Codex scenarios -> add entrypoint regressions after Lane A freezes validator expectations

### Execution order

1. Launch Lane A first. It owns the contract and the validator.
2. Once Lane A has frozen field names and validation rules, launch Lane B and Lane C in parallel worktrees.
3. Merge B and C.
4. Run the targeted xtask tests, then the formatting and clippy gates.

### Conflict flags

- Lane A and Lane C both depend on exact field names in `handoff.json`. Do not run them in parallel before the schema is frozen.
- Lane A and Lane B both depend on the final terminology for `publication_refresh_ready` and surface ids. Freeze names before updating docs.

## NOT in scope

- Auto-grading whether backend code is "truly default-tier" by inspecting implementation shape.
  Rationale: too clever, low signal, and not needed to close the review-contract seam.
- Adding a second machine-readable artifact beside `handoff.json`.
  Rationale: one handoff file is enough for this increment.
- Allowing `achieved_tier` to differ from `requested_tier`.
  Rationale: keep v1 semantics strict and reviewable.
- Updating publication-owned manifest outputs.
  Rationale: still the next lane.

## Definition of Done

This plan is complete only when all of the following are true:

1. `handoff.json` includes `publication_refresh_ready` and a typed `implementation_summary`.
2. `validate_handoff` rejects missing, contradictory, incomplete, duplicated, or unauthorized summary semantics.
3. `run-summary.md` renders achieved tier, template lineage, landed surfaces, deferred surfaces, blockers, and required commands from validated data.
4. `run-status.json` exposes the agreed mirrored fields and nothing more.
5. The fake Codex fixture can emit both valid and invalid richer handoff scenarios.
6. The entrypoint regression suite covers every new semantic failure mode listed above.
7. `docs/cli-agent-onboarding-factory-operator-guide.md` documents the strengthened runtime handoff accurately.
8. `cargo test -p xtask --test runtime_follow_on_entrypoint`, `cargo test -p xtask runtime_follow_on`, `make fmt-check`, and `make clippy` all pass.

## Completion Summary

- Scope: accepted as-is, narrowed to review-contract widening only
- Architecture: one runtime lane, one handoff artifact, one typed summary model
- Code quality: no new abstraction layer, no duplicate schema writer, no second handoff file
- Tests: success and failure semantics are fully specified
- Failure modes: all silent semantic-downgrade paths are explicitly blocked
- Parallelization: 3 lanes, with docs and tests parallelized after schema freeze
- Follow-on: publication refresh remains a separate milestone fed by the richer handoff
