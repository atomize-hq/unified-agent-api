# PLAN - Enclose Create-Mode Closeout Without Ad Hoc Authoring

Status: planned
Date: 2026-05-03
Branch: `codex/recommend-next-agent`
Base branch: `main`
Repo: `atomize-hq/unified-agent-api`
Work item: `Enclose Create-Mode Closeout Without Ad Hoc Authoring`
Plan commit baseline: `f6023e1`

Separate design doc: not required. This is a backend and operator-workflow slice inside the
existing create-mode lifecycle. `PLAN.md` is the canonical design and execution record.

## Objective

Add one repo-owned closeout preparation seam so maintainers stop hand-authoring the full
`proving-run-closeout.json` shape after publication is green.

After this plan lands:

1. `refresh-publication --write` no longer hands the maintainer a blank JSON problem
2. `prepare-proving-run-closeout --approval <path> --check|--write` becomes the only way to
   materialize the create-mode closeout draft
3. `proving-run-closeout.json` can exist safely in a draft form without pretending the proving
   run is already closed
4. `close-proving-run` stops trusting human-authored machine fields and finalizes them from
   committed lifecycle and publication truth
5. the create-mode path becomes:

```text
runtime_follow_on
  -> prepare-publication
  -> refresh-publication
  -> prepare-proving-run-closeout
  -> close-proving-run
```

The non-negotiable outcome is simple:

```text
green publication
  -> repo-owned closeout draft
  -> bounded human inputs
  -> closed_baseline
```

No freehand artifact shape authoring in the middle.

## Source Inputs

- Backlog source:
  - `TODOS.md`
- Normative contracts:
  - `docs/specs/cli-agent-onboarding-charter.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/support-matrix.md`
- Procedure source:
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
- Implementation surfaces:
  - `crates/xtask/src/main.rs`
  - `crates/xtask/src/lib.rs`
  - `crates/xtask/src/agent_lifecycle.rs`
  - `crates/xtask/src/proving_run_closeout.rs`
  - `crates/xtask/src/publication_refresh.rs`
  - `crates/xtask/src/close_proving_run.rs`
  - `crates/xtask/src/historical_lifecycle_backfill.rs`
  - `crates/xtask/src/onboard_agent/preview/render.rs`
  - `crates/xtask/src/agent_maintenance/closeout/render.rs`
- Current tests and fixtures:
  - `crates/xtask/tests/refresh_publication_entrypoint.rs`
  - `crates/xtask/tests/onboard_agent_closeout_preview/preview_states.rs`
  - `crates/xtask/tests/onboard_agent_closeout_preview/closeout_schema_validation.rs`
  - `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_paths.rs`
  - `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`
  - `docs/agents/lifecycle/*/governance/proving-run-closeout.json`

## Verified Current State

These facts were re-verified on the current branch before this rewrite.

1. `refresh-publication --write` now commits `lifecycle_stage = published` and records
   `publication_packet_path` plus `publication_packet_sha256` in
   `lifecycle-state.json`.
2. `refresh-publication --write` still points the next command directly at
   `close-proving-run --approval <path> --closeout <path>`.
3. `close-proving-run` validates an already-authored `proving-run-closeout.json`, but it
   does not prepare or serialize that artifact itself.
4. `proving_run_closeout::RawProvingRunCloseout` currently accepts only `state = "closed"`.
   There is no safe draft state.
5. `onboard-agent` preview treats any valid closeout JSON as authoritative closed-packet
   truth and renders the packet as `closed_proving_run`.
6. `historical_lifecycle_backfill::ensure_closeout(...)` already proves the repo can
   synthesize a valid create-mode closeout JSON from machine-known truth when a file is
   missing.
7. Maintenance closeout already has a dedicated serializer and writer path under
   `crates/xtask/src/agent_maintenance/closeout/`. Create-mode closeout does not.
8. The current create-mode closeout examples mostly repeat machine-owned facts:
   approval path, approval sha, source label, preflight status, timestamp, commit.
9. The fields that still genuinely need human judgment are narrow:
   manual edit count, partial-write count, ambiguous-ownership count, duration, and
   residual friction.
10. The operator guide still instructs maintainers to author the entire
    `proving-run-closeout.json` by hand after publication is green.
11. Historical backfill uses `approval_source = "historical-lifecycle-backfill"`, while live
    closeout tests use `approval_source = "governance-review"`. That distinction exists in
    practice but is not centralized.
12. The repo already has the honest post-publication lifecycle state. This slice should build
    on `published`, not reopen stage semantics.

## Problem Statement

The lifecycle machine is enclosed through publication, but the last operator handoff is still
manual artifact shape assembly.

Current shape:

```text
refresh-publication --write
  -> writes published lifecycle truth
  -> points next command at close-proving-run

operator
  -> hand-builds proving-run-closeout.json

close-proving-run --write
  -> validates authored JSON
  -> advances to closed_baseline
```

That leaves five problems:

1. machine-known closeout facts are still recopied by humans
2. there is no repo-owned draft step between `published` and `closed_baseline`
3. the current schema cannot represent "prepared but not closed"
4. any naive auto-writer would accidentally create a file that previews as fully closed
5. machine-owned closeout serialization logic is split across historical backfill, tests, and
   operator instructions

Target shape:

```text
refresh-publication --write
  -> lifecycle_stage = published
  -> expected_next_command = prepare-proving-run-closeout --approval <path> --write

prepare-proving-run-closeout --write
  -> validates published continuity
  -> writes proving-run-closeout.json with state = prepared
  -> fills machine-owned fields
  -> seeds bounded placeholders for remaining human inputs
  -> refreshes packet docs to "closeout prepared"
  -> expected_next_command = close-proving-run --approval <path> --closeout <path>

human
  -> edits only bounded human-owned fields

close-proving-run --write
  -> revalidates publication continuity
  -> rejects unresolved placeholders
  -> rewrites machine-owned fields from current truth
  -> flips state = closed
  -> writes closed packet docs
  -> lifecycle_stage = closed_baseline
```

That is the boring lane we want.

## Step 0 Scope Challenge

### Premise Check

The repo does not need:

- a new create-mode lifecycle stage
- a second closeout artifact type
- a generic workflow engine

The repo does need:

- one explicit preparation command
- one safe draft state inside the canonical artifact
- one shared source of truth for create-mode closeout serialization

The whole game is making draft closeout possible without falsely claiming the run is already
closed.

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| deterministic closeout path | `agent_lifecycle::proving_run_closeout_path(...)` and `onboard_agent::preview::closeout_relative_path(...)` | Reuse one canonical path rule. Do not add a second draft path. |
| published continuity truth | `publication_refresh.rs` + `lifecycle-state.json` packet continuity fields | Reuse directly as the machine input for draft generation. |
| closeout validation | `proving_run_closeout.rs` | Extend it to understand `prepared` and `closed`. Do not fork a second validator. |
| closeout draft precedent | `historical_lifecycle_backfill::ensure_closeout(...)` | Extract and generalize. This is already the seed of the right behavior. |
| JSON serializer pattern | `agent_maintenance/closeout/render.rs::serialize_closeout_json(...)` | Mirror the pattern for proving-run closeout. Keep serializers explicit. |
| closed-packet rendering | `onboard_agent/preview/render.rs` | Extend the packet phase model. Do not create a parallel packet renderer. |
| final closeout command | `close_proving_run.rs` | Keep it as the only lifecycle-closing writer. Teach it to finalize prepared drafts. |

### Alternatives Considered

| Option | Summary | Pros | Cons | Decision |
| --- | --- | --- | --- | --- |
| A | Add `prepare-proving-run-closeout`, keep one canonical JSON path, add `prepared` state | Explicit ownership, safe draft semantics, clean operator next step, no second artifact type | Touches schema, preview docs, and command wiring | **Chosen.** |
| B | Add `--prepare` mode to `close-proving-run` | Fewer CLI entrypoints | Blurs prepare vs close ownership, worse operator story, harder lifecycle next-command wording | Rejected. |
| C | Auto-write a schema-valid `closed` skeleton and ask humans to edit it | Smallest code diff | Preview/docs would lie that the proving run is closed | Rejected. |
| D | Introduce `proving-run-closeout.draft.json` | Keeps final schema untouched | New artifact type, duplicate paths, worse ergonomics, more drift surface | Rejected. |

### Minimum Complete Change Set

Anything smaller than this is a shortcut that keeps the lane half-manual:

1. add `prepare-proving-run-closeout --approval <path> --check|--write`
2. extend create-mode closeout schema to represent both `prepared` and `closed`
3. centralize create-mode closeout build and serialize logic in `proving_run_closeout.rs`
4. change `refresh-publication --write` to point at the preparation command, not directly at
   `close-proving-run`
5. update `onboard-agent` preview and generated packet docs to distinguish `execution`,
   `closeout_prepared`, and `closed_proving_run`
6. update `close-proving-run` to finalize machine-owned fields and reject unresolved draft
   placeholders
7. move historical backfill onto the same shared create-mode closeout builder
8. add direct regression coverage for the full `published -> prepared -> closed_baseline` path
9. update operator and charter docs so there is one documented post-publication lane

### Complexity Check

This will touch more than 8 files. That is not overbuilt. It is the minimum complete version.

The seam spans:

- xtask CLI dispatch
- lifecycle next-command semantics
- create-mode closeout schema
- packet preview rendering
- historical backfill reuse
- final closeout validation
- operator docs and regression tests

Complexity controls:

- no new lifecycle stage
- no second closeout artifact type
- no maintenance-mode behavior change
- no new repo-wide publication contract
- no generic JSON templating framework

### Search / Build Decision

This is mostly a reuse correction with one first-principles architectural call.

- **[Layer 1]** Reuse `published` lifecycle continuity as the machine truth source.
- **[Layer 1]** Reuse the closeout path derivation that already exists in lifecycle helpers.
- **[Layer 1]** Reuse the historical backfill materialization precedent.
- **[Layer 1]** Reuse the maintenance closeout serializer pattern.
- **[Layer 3]** A safe draft must not look like a finished closeout. That is why the
  canonical artifact needs a `prepared` state instead of a fake `closed` skeleton.

No web search is needed here. The repo already contains the authoritative constraints.

### TODOS Cross-Reference

This plan closes:

- `Enclose Create-Mode Closeout Without Ad Hoc Authoring`

This plan depends on already-landed work:

- `Enclose The Publication Lane End To End`
- `Make The Published State Honest In The Lifecycle Model`

This plan does not implement:

- `Land The LLM-Guided Research Layer For The Recommendation Lane`
- `Decide Whether Capability Matrix Markdown Stays Canonical After M5`

### Completeness Check

The shortcut version would be "just generate a JSON skeleton." Not good enough.

The complete version is:

- machine-owned fields written by the repo
- draft semantics that do not falsely imply closure
- finalization that rewrites machine-owned truth
- preview/docs/operator flow that match the new contract
- regression coverage for the whole seam

That is still a boilable lake.

### Distribution Check

No new externally published artifact type is introduced. This is an internal xtask and
governance-flow change only.

## Locked Decisions

1. The new command is `prepare-proving-run-closeout --approval <path> --check|--write`.
2. `prepare-proving-run-closeout` derives the closeout path from the approval artifact. It does
   not take a separate `--closeout` argument.
3. `proving-run-closeout.json` remains the canonical artifact path. There is no
   `.draft.json`, `.template.json`, or sidecar prompt file.
4. Create-mode closeout `state` becomes a real enum with exactly two accepted values:
   `prepared` and `closed`.
5. `refresh-publication --write` sets
   `expected_next_command = prepare-proving-run-closeout --approval <path> --write`.
6. `prepare-proving-run-closeout --write` keeps `lifecycle_stage = published`. It updates only
   provenance and next-command fields, not the stage itself.
7. `prepare-proving-run-closeout --write` sets
   `expected_next_command = close-proving-run --approval <path> --closeout <path>`.
8. Machine-owned fields for live create-mode closeout are:
   `state`, `approval_ref`, `approval_sha256`, `approval_source`, `preflight_passed`,
   `recorded_at`, and `commit`.
9. Human-owned fields are:
   `manual_control_plane_edits`, `partial_write_incidents`,
   `ambiguous_ownership_incidents`, exactly one of
   `duration_seconds` or `duration_missing_reason`, and exactly one of
   `residual_friction` or `explicit_none_reason`.
10. Live create-mode prepared and closed artifacts use
    `approval_source = "governance-review"`. Historical backfill keeps its explicit historical
    source label.
11. Draft placeholders are allowed only in the prepared state. `close-proving-run` must reject
    unresolved placeholder text or unresolved human input branches.
12. `close-proving-run` rewrites machine-owned fields from current repo truth before persisting
    the final closed artifact.
13. `onboard-agent` preview must never render a prepared draft as a closed packet.
14. This milestone does not change maintenance closeout behavior or schema.

## Architecture Review

### Canonical State Machine

```text
approved
  -> enrolled
  -> runtime_integrated
  -> publication_ready
  -> published
       owner: refresh-publication --write
       next:  prepare-proving-run-closeout --approval <path> --write
  -> published
       owner: prepare-proving-run-closeout --write
       next:  close-proving-run --approval <path> --closeout <path>
  -> closed_baseline
       owner: close-proving-run --write
```

No new stage. One better handoff.

### Draft And Final Artifact Semantics

Prepared draft:

```json
{
  "state": "prepared",
  "approval_ref": "docs/agents/lifecycle/<prefix>/governance/approved-agent.toml",
  "approval_sha256": "<approval sha256>",
  "approval_source": "governance-review",
  "manual_control_plane_edits": 0,
  "partial_write_incidents": 0,
  "ambiguous_ownership_incidents": 0,
  "duration_missing_reason": "TODO: replace with duration_seconds or explain why exact duration is unavailable.",
  "explicit_none_reason": "TODO: replace with residual_friction items or explain why no residual friction remained.",
  "preflight_passed": true,
  "recorded_at": "<draft timestamp>",
  "commit": "<draft commit>"
}
```

Final closed artifact:

- same path
- same machine-owned continuity fields, refreshed from current truth
- `state = "closed"`
- placeholder strings forbidden
- closed packet docs become authoritative

### Field Ownership Matrix

| Field | Owner | Prepare-time source | Close-time source |
| --- | --- | --- | --- |
| `state` | machine | hard-coded `prepared` | hard-coded `closed` |
| `approval_ref` | machine | approval artifact path | approval artifact path |
| `approval_sha256` | machine | approval artifact sha256 | approval artifact sha256 |
| `approval_source` | machine | command identity | command identity |
| `manual_control_plane_edits` | human | seeded to `0` | validated from edited draft |
| `partial_write_incidents` | human | seeded to `0` | validated from edited draft |
| `ambiguous_ownership_incidents` | human | seeded to `0` | validated from edited draft |
| `duration_*` branch | human | seeded placeholder reason | validated from edited draft |
| `residual_friction` / `explicit_none_reason` | human | seeded placeholder explicit-none reason | validated from edited draft |
| `preflight_passed` | machine | derived from published lifecycle truth | derived from current published lifecycle truth |
| `recorded_at` | machine | draft generation timestamp | close command timestamp |
| `commit` | machine | current HEAD at draft time | current HEAD at close time |

### Command Ownership

```text
refresh-publication --write
  writes:
    lifecycle_stage = published
    publication continuity fields
    expected_next_command = prepare-proving-run-closeout --approval <path> --write
  does not write:
    proving-run-closeout.json

prepare-proving-run-closeout --write
  writes:
    proving-run-closeout.json with state = prepared
    packet docs in "closeout prepared" mode
    lifecycle provenance fields
    expected_next_command = close-proving-run --approval <path> --closeout <path>
  does not write:
    lifecycle_stage = closed_baseline

close-proving-run --write
  writes:
    proving-run-closeout.json with state = closed
    packet docs in closed mode
    lifecycle_stage = closed_baseline
    closeout_baseline_path
```

### Module / Responsibility Map

| Module | Responsibility in this slice |
| --- | --- |
| `crates/xtask/src/main.rs` | register the new command |
| `crates/xtask/src/lib.rs` | export the new module |
| `crates/xtask/src/agent_lifecycle.rs` | add helper for the prepare-closeout next command |
| `crates/xtask/src/proving_run_closeout.rs` | shared create-mode closeout state enum, builder, serializer, placeholder detection, validation helpers |
| `crates/xtask/src/prepare_proving_run_closeout.rs` | new check/write entrypoint for draft materialization |
| `crates/xtask/src/publication_refresh.rs` | point published state at the new prepare command |
| `crates/xtask/src/close_proving_run.rs` | finalize prepared drafts, rewrite machine-owned fields, preserve existing closeout gate |
| `crates/xtask/src/historical_lifecycle_backfill.rs` | switch inline JSON creation to shared create-mode builder |
| `crates/xtask/src/onboard_agent/preview/render.rs` | add `closeout_prepared` packet phase and copy |
| `docs/cli-agent-onboarding-factory-operator-guide.md` | document the new post-publication handoff |
| `docs/specs/cli-agent-onboarding-charter.md` | pin the new canonical next step |

### Data Flow

```text
published lifecycle-state.json
  + approval artifact
  + publication-ready.json continuity
  -> prepare_proving_run_closeout::build_draft(...)
      -> proving-run-closeout.json (prepared)
      -> packet docs (closeout prepared)
      -> lifecycle provenance update

prepared proving-run-closeout.json
  + human-owned metrics/final notes
  + current published continuity
  -> close_proving_run::finalize(...)
      -> proving-run-closeout.json (closed)
      -> packet docs (closed)
      -> lifecycle_state.json (closed_baseline)
```

### Transaction Ordering

Prepare write must be transactional across the draft artifact and the published state's next-step
metadata.

Required prepare ordering:

1. validate `published` lifecycle continuity and approval continuity
2. derive the deterministic closeout path
3. build the prepared draft from machine-owned truth
4. plan closeout JSON, packet-doc, and lifecycle-provenance mutations
5. apply the mutations as one bounded workspace change

Required close ordering:

1. validate `published` lifecycle continuity and publication continuity
2. load the prepared draft
3. reject unresolved placeholders or invalid human-owned branches
4. rebuild machine-owned fields from current truth
5. write the final closeout JSON
6. refresh closed packet docs
7. advance lifecycle state to `closed_baseline`

If any step before lifecycle advancement fails, the repo must not claim `closed_baseline`.

### Failure Modes Registry

| Codepath | Realistic failure | Test coverage requirement | Handling | User-visible outcome |
| --- | --- | --- | --- | --- |
| refresh -> prepare handoff | published state still points directly at `close-proving-run` | direct CLI regression | fail test, fix next-command wiring | clear xtask regression before ship |
| prepare draft | command runs before `published` exists | entrypoint test | hard reject with explicit stage error | clear error, no writes |
| prepared draft preview | draft renders as closed packet | preview-state test | new `closeout_prepared` phase | maintainers see the right next step |
| close finalization | maintainer leaves placeholder text in duration or friction fields | close-proving-run write test | hard reject with actionable message | no baseline advancement |
| machine continuity drift | human edits `approval_sha256` or approval path | validation test | hard reject and preserve files | explicit continuity error |
| historical backfill | shared serializer changes historical output shape unexpectedly | backfill regression test | reuse shared builder with historical source label | historical repair stays truthful |

No critical gaps remain if the plan above lands as written.

## Code Quality Review

This slice wants explicit code, not clever abstractions.

Implementation rules:

1. Extend `ProvingRunCloseout` instead of inventing a second parallel create-mode closeout model.
2. Put placeholder constants and state parsing in `proving_run_closeout.rs`, not scattered through
   entrypoints and tests.
3. Keep `prepare_proving_run_closeout.rs` thin. It should orchestrate validation and writes, not
   redefine the schema.
4. Keep `close_proving_run.rs` the only writer of `closed_baseline`.
5. Reuse the maintenance closeout serializer shape as a pattern, but do not try to genericize
   maintenance and create-mode closeout into one abstraction. Different domains, different truth.
6. Reuse existing path helpers. No stringly-typed duplicate path builders.
7. Migrate the historical backfill inline JSON builder to the shared serializer so the repo has
   exactly one create-mode closeout shape.

ASCII diagram maintenance:

- `PLAN.md` carries the canonical cross-file flow for this slice.
- If nearby comments in `publication_refresh.rs`, `close_proving_run.rs`, or
  `onboard_agent/preview/render.rs` describe the post-publication flow, update them in the same
  commit so they do not lie.

## Test Review

This slice is not done until the repo proves the full `published -> prepared -> closed_baseline`
seam directly.

### Code Path Coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/xtask/src/publication_refresh.rs
    │
    └── run_in_workspace(--write)
        ├── [★★ TESTED] commits lifecycle_stage = published
        ├── [GAP]      points next command at prepare-proving-run-closeout
        └── [GAP]      no longer points directly at close-proving-run

[+] crates/xtask/src/prepare_proving_run_closeout.rs
    │
    ├── run_in_workspace(--check)
    │   ├── [GAP] validates published continuity
    │   └── [GAP] validates existing draft freshness
    │
    └── run_in_workspace(--write)
        ├── [GAP] derives canonical closeout path from approval
        ├── [GAP] writes state = prepared
        ├── [GAP] seeds machine-owned fields
        ├── [GAP] seeds bounded human placeholders
        ├── [GAP] refreshes packet docs into prepared mode
        └── [GAP] updates expected_next_command to close-proving-run

[+] crates/xtask/src/proving_run_closeout.rs
    │
    ├── parse prepared draft
    ├── parse closed artifact
    ├── serialize prepared draft
    ├── serialize closed artifact
    └── [GAP] detect unresolved placeholder truth at close time

[+] crates/xtask/src/close_proving_run.rs
    │
    ├── [★★ TESTED] validates published continuity
    ├── [GAP] accepts prepared drafts as the normal input artifact
    ├── [GAP] rewrites machine-owned fields before final write
    ├── [GAP] rejects unresolved placeholders
    └── [★★ TESTED] advances lifecycle_state to closed_baseline

[+] crates/xtask/src/historical_lifecycle_backfill.rs
    │
    └── [GAP] emits closed artifacts through the shared serializer
```

### Operator Flow Coverage

```text
USER / OPERATOR FLOW COVERAGE
=============================
[+] Post-publication handoff
    │
    ├── [GAP] refresh-publication prints / records prepare-proving-run-closeout as next
    ├── [GAP] prepare-proving-run-closeout writes a prepared draft
    ├── [GAP] onboard-agent preview shows "closeout prepared", not "closed"
    └── [GAP] close-proving-run succeeds once only human-owned fields are resolved

[+] Draft safety
    │
    ├── [GAP] prepared draft does not mark packet closed
    ├── [GAP] unresolved TODO placeholders block close
    └── [GAP] manual edits to machine-owned continuity fields block close

[+] Historical compatibility
    │
    ├── [★★ TESTED] existing historical closed JSON still parses
    └── [GAP] historical backfill still emits valid closed JSON through shared code

─────────────────────────────────
COVERAGE TARGET: all changed paths
  New entrypoint suites: 1
  Existing suite updates: 5
  Critical workflow regressions: 9
QUALITY TARGET: no draft/final branch without a direct test
─────────────────────────────────
```

### Test Requirements To Add Or Update

| File | Required assertions |
| --- | --- |
| `crates/xtask/tests/refresh_publication_entrypoint.rs` | assert `refresh-publication --write` now records `prepare-proving-run-closeout --approval <path> --write` as the next command |
| `crates/xtask/tests/prepare_proving_run_closeout_entrypoint.rs` | new suite for check/write behavior on published fixtures |
| `crates/xtask/tests/prepare_proving_run_closeout_entrypoint.rs` | assert write mode derives the closeout path, writes `state = "prepared"`, and updates lifecycle next-command provenance |
| `crates/xtask/tests/prepare_proving_run_closeout_entrypoint.rs` | assert prepare rejects non-`published` lifecycle state |
| `crates/xtask/tests/onboard_agent_closeout_preview/preview_states.rs` | assert prepared draft renders packet state `closeout_prepared`, not `closed_proving_run` |
| `crates/xtask/tests/onboard_agent_closeout_preview/closeout_schema_validation.rs` | cover prepared-state parse/serialize rules and placeholder rejection rules |
| `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs` | assert close-proving-run finalizes a prepared draft, flips state to `closed`, and rewrites machine-owned fields |
| `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_paths.rs` | assert prepare infers the closeout path while close still consumes the explicit canonical path |
| `crates/xtask/tests/historical_lifecycle_backfill.rs` or equivalent | assert shared serializer preserves historical closeout validity |

### Required Verification Commands

Run at minimum:

```sh
cargo test -p xtask --test refresh_publication_entrypoint
cargo test -p xtask --test prepare_proving_run_closeout_entrypoint
cargo test -p xtask --test onboard_agent_closeout_preview
cargo test -p xtask --test historical_lifecycle_backfill_entrypoint
make test
```

Then the normal repo gate:

```sh
make preflight
```

## Performance Review

This is a control-plane feature. Runtime performance is not the story. Still, there are a few
ways to make it sloppy.

1. `prepare-proving-run-closeout` must not rerun the full publication gate. It should validate
   current published continuity, not redo `make preflight`.
2. Path, hash, and lifecycle reads should happen once per invocation and then flow through a
   small context struct, same style as `publication_refresh.rs`.
3. The new command should write only the draft JSON, the prepared packet docs, and the lifecycle
   provenance update. No repo-wide regeneration.
4. Historical backfill should keep its current bounded behavior. No extra git probes on the live
   prepare/close path.

The performance smell to avoid is using a lightweight draft step as a second publication gate.
That would make the operator lane slower without adding truth.

## Implementation Plan

### Phase 1. Shared closeout domain

Files:

- `crates/xtask/src/proving_run_closeout.rs`
- `crates/xtask/src/agent_lifecycle.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/main.rs`

Work:

1. Add a create-mode closeout state enum with `Prepared` and `Closed`.
2. Add shared builder helpers for:
   - prepared draft construction from approval + published lifecycle truth
   - closed artifact construction from current truth + validated human inputs
3. Add one explicit serializer for create-mode closeout JSON.
4. Add placeholder detection for the prepared-state human-owned fields.
5. Add lifecycle helper for the prepare-closeout next command.

Acceptance:

- the shared module can build both prepared and closed forms without inline `json!` in callers
- CLI wiring compiles with the new command registered

### Phase 2. Published to prepared handoff

Files:

- `crates/xtask/src/publication_refresh.rs`
- `crates/xtask/src/prepare_proving_run_closeout.rs`

Work:

1. Add the new entrypoint with `--check|--write`.
2. Validate that `published` lifecycle continuity exists before writing anything.
3. Derive the closeout path from approval truth.
4. Write the prepared draft and update lifecycle provenance plus next-command fields.
5. Update `refresh-publication --write` to point at the new prepare step.

Acceptance:

- published lifecycle records the prepare-closeout next step
- prepare-closeout write produces a deterministic draft without manual path input

### Phase 3. Packet preview and docs

Files:

- `crates/xtask/src/onboard_agent/preview/render.rs`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/specs/cli-agent-onboarding-charter.md`

Work:

1. Add a `closeout_prepared` packet phase.
2. Render prepared docs with explicit "fill the human-owned fields, then run close-proving-run"
   language.
3. Update operator and charter docs to reflect the new post-publication step.

Acceptance:

- a prepared draft never renders as a closed proving run
- the operator guide tells one exact story after publication

### Phase 4. Final closeout and historical reuse

Files:

- `crates/xtask/src/close_proving_run.rs`
- `crates/xtask/src/historical_lifecycle_backfill.rs`

Work:

1. Make `close-proving-run` load prepared drafts as the normal create-mode input.
2. Reject unresolved placeholder values or unresolved human-owned branches.
3. Rebuild machine-owned fields from current truth before the final write.
4. Persist the final `closed` artifact and then advance lifecycle state to
   `closed_baseline`.
5. Replace historical backfill inline closeout JSON generation with the shared builder and
   serializer.

Acceptance:

- close-proving-run finalizes prepared drafts
- historical backfill still emits truthful closed artifacts

### Phase 5. Tests and fixtures

Files:

- `crates/xtask/tests/refresh_publication_entrypoint.rs`
- `crates/xtask/tests/prepare_proving_run_closeout_entrypoint.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/**`
- any historical backfill test surface that covers emitted closeouts

Work:

1. Add the new entrypoint suite.
2. Update preview-state tests for prepared vs closed semantics.
3. Add close rejection tests for unresolved placeholders and continuity drift.
4. Prove the full `published -> prepared -> closed_baseline` path.

Acceptance:

- all changed branches have direct regression coverage
- `make preflight` is green

## NOT In Scope

These ideas were considered and explicitly deferred.

- Auto-capturing duration from lifecycle timestamps alone.
  Reason: duration is still partly a human truth question for proving-run interpretation.
- Re-running `make preflight` inside `prepare-proving-run-closeout`.
  Reason: published truth already proves the gate; duplicating it here would slow the lane without
  adding authority.
- Extending the same prep workflow to `maintenance-closeout.json`.
  Reason: maintenance already has its own request/closeout contract and is not the blocker here.
- Replacing `proving-run-closeout.json` with TOML or Markdown.
  Reason: pure format churn, zero lifecycle gain.
- Batch closeout preparation for multiple onboarding packs.
  Reason: unnecessary complexity for the current single-pack operator flow.

## TODOS.md Updates

No new TODOs are required to land this slice.

If a follow-on is ever needed, it should be a separate milestone:

- capture duration automatically only if real proving runs show that humans never provide better
  signal than machine timestamps

That is not part of this plan.

## Worktree Parallelization Strategy

This plan has one real parallel window, but not a huge one. The closeout contract has to lock
first.

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Shared closeout domain and CLI contract | `crates/xtask/src`, `crates/xtask/src/proving_run_closeout.rs`, `crates/xtask/src/agent_lifecycle.rs` | — |
| Prepare command and published-hand-off wiring | `crates/xtask/src`, `crates/xtask/src/publication_refresh.rs` | shared closeout domain and CLI contract |
| Packet preview and operator docs | `crates/xtask/src/onboard_agent/preview`, `docs/` | shared closeout domain and command names locked |
| Final closeout + historical backfill reuse | `crates/xtask/src`, `crates/xtask/src/close_proving_run.rs`, `crates/xtask/src/historical_lifecycle_backfill.rs` | shared closeout domain |
| Regression tests | `crates/xtask/tests/**` | prepare command, preview/docs, and final closeout behavior locked |

### Parallel Lanes

Lane A: shared closeout domain and CLI contract -> final closeout and historical reuse
Sequential, shared create-mode closeout core.

Lane B: packet preview and operator docs
Can run after Lane A freezes the `prepared` vs `closed` contract and command names.

Lane C: prepare command and publication hand-off wiring
Can run after Lane A freezes the shared builder and lifecycle helper names.

Lane D: regression tests
Runs after Lanes A, B, and C settle.

### Execution Order

1. Launch Lane A first. This locks the schema, builder, serializer, and command names.
2. Once Lane A is stable, launch Lane B and Lane C in parallel worktrees.
3. Merge B and C.
4. Run Lane D last because the tests depend on the final preview and command semantics.

### Conflict Flags

- Lane A and Lane C both touch `crates/xtask/src`. Keep their boundary explicit:
  Lane A owns shared closeout types and lifecycle helpers, Lane C owns the new entrypoint plus
  refresh wiring.
- Lane B and Lane D both depend on preview copy under
  `crates/xtask/src/onboard_agent/preview/`. Do not start D until B lands.

## Completion Summary

- Step 0: Scope Challenge, scope accepted with one new command and no new artifact type
- Architecture Review: 4 architectural issues resolved by locked decisions
- Code Quality Review: 5 rules locked to keep the diff explicit and small
- Test Review: coverage diagrams produced, 9 direct regression cases required
- Performance Review: 1 substantive guardrail, do not turn prepare-closeout into a second publication gate
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 new items proposed
- Failure modes: 0 critical gaps if implemented as planned
- Outside voice: not run for this rewrite
- Parallelization: 4 lanes total, 2 can overlap after the core contract lands
- Lake Score: 6/6 recommendations chose the complete option over the shortcut
