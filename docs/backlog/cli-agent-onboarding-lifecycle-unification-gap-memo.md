# CLI Agent Onboarding Lifecycle Unification Gap Memo

Date: 2026-04-30
Branch: `codex/recommend-next-agent`
Status: pickup memo

## Purpose

This memo captures the lifecycle gap uncovered while validating the shipped `runtime-follow-on`
lane against the repo's existing deterministic capability publication and maintenance machinery.

The short version:

- `xtask runtime-follow-on --write` is already landed as a legitimate bounded Codex execution runner.
- It is good enough to implement runtime-owned code and evidence.
- The remaining lifecycle hole is publication/maintenance wiring after that runtime-owned evidence exists.

That missing publication/maintenance wiring is now the highest-leverage onboarding gap.

## Bottom line

The repo currently has four strong subsystems:

1. pre-create recommendation and approval
2. control-plane onboarding and wrapper-shell scaffolding
3. bounded runtime implementation via `runtime-follow-on`
4. deterministic publication, capability projection, and maintenance drift detection

What is missing is the lifecycle contract that turns those into one coherent system.

Today the repo closes the runtime seam, then relies on later lanes to line capability publication,
support publication, validation, and maintenance back up by convention.

That is why a newly onboarded agent can be "implemented" but still not land cleanly in:

- `docs/specs/unified-agent-api/capability-matrix.md`
- `docs/specs/unified-agent-api/support-matrix.md`
- maintenance drift and future upgrade workflows

## Why this surfaced now

This is a natural breakpoint, not a random miss.

The newer onboarding work correctly optimized for the narrow seam first:

- freeze a runtime packet
- let the repo own `codex exec`
- reject out-of-bounds writes
- reject publication-owned manifest edits
- require runtime-owned evidence

That was the right local move.

The next missing move is global: wire the runtime lane into the deterministic publication and
maintenance levers that already existed before the newer onboarding work started.

## Current system shape

```text
recommend-next-agent
  -> approved-agent.toml
  -> onboard-agent --write
  -> scaffold-wrapper-crate --write
  -> runtime-follow-on --dry-run / --write
  -> manual / loosely-coupled publication refresh
  -> support-matrix / capability-matrix / capability-matrix-audit / preflight
  -> proving-run closeout
  -> later maintenance / version drift handling
```

The system works mechanically, but the transitions after `runtime-follow-on` are still too loose.

## Exact missing contracts

### 1. Approval artifact to runtime packet contract is missing capability/publication context

The approval descriptor already carries the data needed to reason about capability publication:

- `always_on_capabilities`
- `target_gated_capabilities`
- `config_gated_capabilities`
- `backend_extensions`
- `canonical_targets`
- `capability_matrix_target`
- `support_matrix_enabled`
- `capability_matrix_enabled`

Source:
- `crates/xtask/src/approval_artifact.rs`

But `runtime-follow-on` only passes a narrow runtime contract into its packet:

- crate path
- backend module
- manifest root
- wrapper coverage source path
- requested tier
- allowed rich surfaces
- required test
- allowed write paths

Sources:
- `crates/xtask/src/runtime_follow_on.rs`
- `crates/xtask/src/runtime_follow_on/models.rs`
- `crates/xtask/templates/runtime_follow_on_codex_prompt.md`

That means Codex is asked to implement runtime code without being told the exact capability
declaration and publication contract that later generators will enforce.

### 2. Runtime output contract is missing the publication-ready capability handoff

The current `handoff.json`, `run-status.json`, and `run-summary.md` are still minimal.

They do not tell the next lane:

- which capability declaration was supposed to land
- which backend capability surfaces were intentionally implemented
- which target-scoped capability projections should become visible in `current.json`
- whether the output is actually ready for support/capability publication
- which publication-owned surfaces still need deterministic refresh

Sources:
- `crates/xtask/src/runtime_follow_on/models.rs`
- `crates/xtask/src/runtime_follow_on/render.rs`
- `crates/xtask/src/runtime_follow_on.rs`

### 3. New-agent capability promotion is not fully deterministic today

The capability matrix generator still owns a hardcoded runtime backend inventory:

- `runtime_backend_capabilities()` has explicit match arms for `aider`, `codex`,
  `claude_code`, `gemini_cli`, and `opencode`
- unknown enrolled backends fail with
  `capability-matrix registry enrolled unsupported backend`

Source:
- `crates/xtask/src/capability_matrix.rs`

That means a newly onboarded agent does **not** become eligible for deterministic capability
publication just by existing in the registry plus runtime lane outputs. Something still has to
promote the new backend into the capability generator's runtime inventory.

This is the largest hidden gap found in this review.

### 4. Publication refresh is deterministic, but it is not yet a first-class create-lane consumer

The repo already has deterministic publication surfaces and checks:

- `cli_manifests/<agent>/current.json`
- `docs/specs/unified-agent-api/support-matrix.md`
- `cli_manifests/support_matrix/current.json`
- `docs/specs/unified-agent-api/capability-matrix.md`
- `cargo run -p xtask -- capability-matrix-audit`
- `make preflight`

Sources:
- `crates/xtask/src/support_matrix.rs`
- `crates/xtask/src/capability_matrix.rs`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

But the runtime lane does not yet emit a machine-readable handoff rich enough for a publication
lane to consume without archaeology.

### 5. Onboarding and maintenance still meet too late

The maintenance lane already knows how to compare:

- registry truth
- `current.json`
- published support surfaces
- published capability surfaces

Sources:
- `crates/xtask/src/agent_maintenance/drift/shared.rs`
- `crates/xtask/src/agent_maintenance/drift/publication.rs`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

That is valuable. But it currently acts like a downstream detector instead of the second half of
the same lifecycle. The repo should treat onboarding closeout as the first committed baseline for
future maintenance and upgrade drift checks.

### 6. Version and promotion surfaces are not yet described as part of one unified agent lifecycle

Manifest roots already carry long-lived upgrade/promotion surfaces such as:

- `current.json`
- `latest_validated.txt`
- `min_supported.txt`
- `pointers/latest_supported/**`
- `pointers/latest_validated/**`
- `versions/**`
- `reports/**`

The create lane acknowledges them. The maintenance lane inspects some of them. But the newer
onboarding flow still treats them as follow-on artifacts instead of one continuous lifecycle from
approval to maintained agent.

## Existing machinery to reuse

| Surface | Role in the unified lifecycle |
| --- | --- |
| `docs/cli-agent-onboarding-factory-operator-guide.md` | Canonical create-mode and maintenance-mode procedure |
| `docs/specs/cli-agent-onboarding-charter.md` | Normative onboarding requirements |
| `crates/xtask/src/approval_artifact.rs` | Control-plane descriptor and capability/publication declaration truth |
| `crates/xtask/src/onboard_agent.rs` | Control-plane enrollment and initial generated packet/docs surfaces |
| `crates/xtask/src/onboard_agent/preview.rs` | Seeds `current.json` and operator handoff surfaces |
| `crates/xtask/src/wrapper_scaffold.rs` | Runtime-owned wrapper shell creation |
| `crates/xtask/src/runtime_follow_on.rs` | Repo-owned bounded Codex execution seam |
| `crates/xtask/src/capability_projection.rs` | Deterministic capability publication rules |
| `crates/xtask/src/capability_matrix.rs` | Capability publication generator and audit input |
| `crates/xtask/src/support_matrix.rs` | Support publication generator |
| `crates/xtask/src/agent_maintenance/drift/shared.rs` | Capability/support truth extraction used by maintenance |
| `crates/xtask/src/agent_maintenance/drift/publication.rs` | Drift detection against published capability/support surfaces |
| `TODOS.md` | Existing backlog hooks for runtime and publication follow-ons |

## Proposed end-to-end lifecycle

```text
recommendation + approval
  -> approved-agent.toml
  -> onboard-agent --write
       writes control-plane enrollment, docs pack, manifest-root skeleton
  -> scaffold-wrapper-crate --write
       writes minimal wrapper shell
  -> runtime-follow-on --dry-run
       freezes runtime packet with capability/publication contract included
  -> runtime-follow-on --write
       implements runtime code + runtime evidence + structured runtime handoff
  -> publication follow-on
       refreshes publication-owned manifest surfaces and generated docs
       validates capability projection and support publication
  -> green gate
       support-matrix --check
       capability-matrix --check
       capability-matrix-audit
       make preflight
  -> proving-run closeout
       records create-mode success baseline
  -> maintenance / upgrade lane
       detects drift, version movement, publication divergence, capability divergence
```

## Minimal change plan

This is the smallest complete plan that unifies the lifecycle without reopening the whole factory.

### Phase 1. Widen `runtime-follow-on` to carry capability/publication truth

Goal: keep runtime ownership where it is, but make the packet and handoff speak the same language
as publication and maintenance.

Required changes:

1. Extend `InputContract` with:
   - `canonical_targets`
   - `always_on_capabilities`
   - `target_gated_capabilities`
   - `config_gated_capabilities`
   - `backend_extensions`
   - `support_matrix_enabled`
   - `capability_matrix_enabled`
   - `capability_matrix_target`
2. Render those into `codex-prompt.md`.
3. Extend `handoff.json` with:
   - structured implementation summary
   - capability declaration summary
   - publication refresh readiness
   - explicit required publication outputs
4. Extend `validate_handoff` so runtime success means:
   - declared implementation tier is explicit
   - runtime capability intent is explicit
   - the publication lane can tell whether it has enough material to proceed

Primary files:
- `crates/xtask/src/runtime_follow_on.rs`
- `crates/xtask/src/runtime_follow_on/models.rs`
- `crates/xtask/src/runtime_follow_on/render.rs`
- `crates/xtask/templates/runtime_follow_on_codex_prompt.md`
- `crates/xtask/tests/runtime_follow_on_entrypoint.rs`

### Phase 2. Add a deterministic publication follow-on lane

Goal: make publication refresh a first-class consumer of the runtime handoff instead of a loose
operator checklist.

Recommended shape:

- add a new `xtask` lane dedicated to publication refresh after runtime completion
- read:
  - approval artifact
  - runtime `handoff.json`
  - runtime evidence under `cli_manifests/<agent>/snapshots/**` and `supplement/**`
- own writes to:
  - `cli_manifests/<agent>/current.json`
  - pointers / versions / reports where applicable
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `cli_manifests/support_matrix/current.json`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- run:
  - `support-matrix`
  - `capability-matrix`
  - `capability-matrix-audit`
  - freshness / consistency checks

Primary files:
- `crates/xtask/src/main.rs`
- `crates/xtask/src/support_matrix.rs`
- `crates/xtask/src/capability_matrix.rs`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `TODOS.md`

### Phase 3. Close the capability promotion gap for newly onboarded backends

Goal: ensure a new backend can be published by deterministic generators without hidden manual code
promotion steps.

Two viable options:

1. **Minimal patch path**
   - explicitly let the onboarding/runtime lifecycle promote the new backend into
     `capability_matrix.rs` runtime inventory
   - codify that this file is part of onboarding-owned promotion, not ad hoc repo surgery
2. **Better long-term path**
   - refactor capability-matrix backend inventory away from hardcoded match arms and toward a
     registry-driven or generated registration surface

Recommendation: start with option 1 unless the refactor is obviously small. The hardcoded match is
an ugly seam, but the repo needs a complete lifecycle first.

Primary files:
- `crates/xtask/src/capability_matrix.rs`
- `crates/agent_api/src/backends/mod.rs`
- `crates/agent_api/src/lib.rs`
- any new backend crate and backend module added by onboarding

### Phase 4. Tie create-mode closeout to maintenance / upgrade truth

Goal: once proving-run closeout lands, the repo should already know what future drift will be
measured against.

Required changes:

1. ensure create-mode closeout records the publication/capability surfaces that were considered
   green
2. ensure maintenance drift checks reuse the same runtime handoff vocabulary
3. document version / pointer / report surfaces as part of the ongoing agent lifecycle, not as
   isolated manifest trivia

Primary files:
- `crates/xtask/src/close_proving_run.rs`
- `crates/xtask/src/agent_maintenance/drift/shared.rs`
- `crates/xtask/src/agent_maintenance/drift/publication.rs`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

## Recommended first implementation slice

If the next agent needs one bounded starting point, do this first:

1. widen `runtime-follow-on` input + handoff contracts to include capability/publication truth
2. update the runtime prompt and tests to require that truth
3. write the follow-on plan for the publication lane immediately after that

Why this first:

- it preserves the already-correct runtime boundary
- it creates the machine-readable contract the next lane needs
- it avoids jumping straight into a larger publication refactor without a clean handoff language

## Non-goals

These should stay out of the first follow-up:

- rewriting the whole onboarding factory
- folding runtime implementation and publication refresh into one giant command
- redesigning support/capability generators from scratch
- changing the normative onboarding charter unless the lifecycle contract truly requires it

## Open questions the next agent should resolve early

1. Should the publication follow-on be a new command or an extension of existing `support-matrix`
   / `capability-matrix` orchestration?
2. Is the hardcoded backend inventory in `capability_matrix.rs` acceptable as an explicit
   onboarding promotion surface, or should it be generalized now?
3. Which manifest-root surfaces are truly generic across newer agents versus legacy
   `codex` / `claude_code`-specific?
4. Does create-mode closeout need new schema fields to record the green publication baseline for
   future maintenance?

## Critical file pointers for pickup

Start here:

- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/backlog/uaa-0022-runtime-follow-on-codex-runner.md`
- `docs/backlog/uaa-0022-runtime-follow-on-narrowness-report.md`
- `TODOS.md`

Runtime lane:

- `crates/xtask/src/runtime_follow_on.rs`
- `crates/xtask/src/runtime_follow_on/models.rs`
- `crates/xtask/src/runtime_follow_on/render.rs`
- `crates/xtask/templates/runtime_follow_on_codex_prompt.md`
- `crates/xtask/tests/runtime_follow_on_entrypoint.rs`

Control-plane and packet truth:

- `crates/xtask/src/approval_artifact.rs`
- `crates/xtask/src/onboard_agent.rs`
- `crates/xtask/src/onboard_agent/preview.rs`
- `crates/xtask/src/onboard_agent/preview/render.rs`
- `crates/xtask/src/wrapper_scaffold.rs`

Capability and support publication:

- `crates/xtask/src/capability_projection.rs`
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/support_matrix.rs`
- `docs/specs/unified-agent-api/capability-matrix.md`
- `docs/specs/unified-agent-api/support-matrix.md`

Maintenance / upgrade continuity:

- `crates/xtask/src/agent_maintenance/drift/shared.rs`
- `crates/xtask/src/agent_maintenance/drift/publication.rs`
- `crates/xtask/src/agent_maintenance/refresh.rs`
- `crates/xtask/src/agent_maintenance/closeout.rs`

Representative runtime and manifest references:

- `crates/agent_api/src/backends/opencode/**`
- `crates/opencode/**`
- `crates/agent_api/src/backends/gemini_cli/**`
- `crates/gemini_cli/**`
- `crates/agent_api/src/backends/aider/**`
- `crates/aider/**`
- `cli_manifests/opencode/**`
- `cli_manifests/gemini_cli/**`
- `cli_manifests/aider/**`

## Relationship to existing TODOs

This memo does not create a brand-new backlog theme. It refines and joins two existing TODOs:

- `Enclose The Runtime Follow-On In A Codex Exec Runner`
- `Enclose The Publication Refresh Follow-On After The Runtime Runner`

What changed is the diagnosis:

- the runtime runner is landed and legitimate
- the publication follow-on is now the main missing seam
- capability promotion and maintenance continuity must be treated as part of that same lifecycle

That is the next system.
