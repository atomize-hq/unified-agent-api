# PLAN - Unified Agent Lifecycle Support Maturity Model

Status: ready for implementation
Date: 2026-05-01
Branch: `codex/recommend-next-agent`
Base branch: `main`
Repo: `atomize-hq/unified-agent-api`
Work item: `Unified agent lifecycle support maturity model`

## Objective

Make agent support maturity explicit, committed, and machine-checked from approval through maintenance.

The repo already has the pieces: approval artifacts, control-plane onboarding, wrapper scaffolding, bounded runtime implementation, publication generators, closeout, and maintenance drift checks. What it does not have is one canonical lifecycle record that all of those stages can read, advance, and validate. This plan adds that record, adds the missing `prepare-publication` seam, and wires the current commands into one deterministic lifecycle without rewriting the factory.

## Source Inputs

- Design doc:
  - `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-design-20260430-214712.md`
- Gap memo:
  - `docs/backlog/cli-agent-onboarding-lifecycle-unification-gap-memo.md`
- Normative and operator docs:
  - `docs/specs/cli-agent-onboarding-charter.md`
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- Current lifecycle owners:
  - `crates/xtask/src/approval_artifact.rs`
  - `crates/xtask/src/agent_registry.rs`
  - `crates/xtask/src/onboard_agent.rs`
  - `crates/xtask/src/wrapper_scaffold.rs`
  - `crates/xtask/src/runtime_follow_on.rs`
  - `crates/xtask/src/runtime_follow_on/models.rs`
  - `crates/xtask/src/runtime_follow_on/render.rs`
  - `crates/xtask/src/capability_matrix.rs`
  - `crates/xtask/src/support_matrix.rs`
  - `crates/xtask/src/close_proving_run.rs`
  - `crates/xtask/src/agent_maintenance/drift/publication.rs`
- Existing tests and fixtures:
  - `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
  - `crates/xtask/tests/onboard_agent_entrypoint/**`
  - `crates/xtask/tests/close_proving_run_paths.rs`
  - `crates/xtask/tests/close_proving_run_write.rs`
  - `crates/xtask/tests/agent_maintenance_drift.rs`
  - `crates/xtask/tests/fixtures/fake_codex.sh`
- Queue context:
  - `TODOS.md`

## Outcome

After this plan lands:

1. Every onboarded agent has one committed lifecycle record at `docs/agents/lifecycle/<pack>/governance/lifecycle-state.json`.
2. `onboard-agent` seeds lifecycle state, `runtime-follow-on` advances it to runtime-integrated, `prepare-publication` advances it to publication-ready, and `close-proving-run` closes the baseline.
3. The runtime lane still owns runtime code only, but it now emits enough committed truth for publication and maintenance to proceed without archaeology.
4. The publication seam is explicit: `prepare-publication` validates readiness and writes the only committed publication handoff packet.
5. Maintenance drift checks compare published truth against the committed lifecycle baseline instead of inferring state from scattered artifacts.

## Problem Statement

The repo currently has multiple truthful subsystems and no single truthful lifecycle.

Approval artifacts know what an agent claims to be. The registry knows where it lives. `onboard-agent` knows how to enroll it. `runtime-follow-on` knows how to bound runtime work. `support-matrix` and `capability-matrix` know how to publish derived surfaces. `close-proving-run` and maintenance commands know how to close and compare evidence.

What is missing is the state machine that says what support level an agent has actually reached, what evidence is still missing, which command is allowed to run next, and whether publication or maintenance claims are legitimate yet.

That is why the repo can truthfully say an agent is enrolled while still not having first-class or even publication-backed support semantics. The lifecycle model needs to become an explicit contract, not a mental model spread across commands and docs.

## Scope

### In scope

- Add a canonical lifecycle record under `docs/agents/lifecycle/<pack>/governance/lifecycle-state.json`.
- Add one shared `xtask` lifecycle module for schema, loading, validation, and path helpers.
- Backfill lifecycle records for existing onboarded agents.
- Teach `onboard-agent` to seed lifecycle state.
- Teach `runtime-follow-on` to read lifecycle state, carry approval capability/publication truth forward, and update lifecycle state on success.
- Add `xtask prepare-publication` as the explicit `runtime_integrated -> publication_ready` seam.
- Teach `close-proving-run` and maintenance drift logic to consume lifecycle state.
- Update the operator guide and charter to reflect the new lifecycle contract.
- Add regression tests for every new transition and every new rejection path.

### Out of scope

- A new crate or service for lifecycle orchestration.
- Folding runtime implementation, publication refresh, and closeout into one giant command.
- Rewriting support-matrix or capability-matrix derivation from scratch.
- Dynamic backend plugin discovery for capability publication.
- Automatic first-class promotion for future agents.
- Reworking every existing maintenance command into a new abstraction family.
- Expanding CI posture for every enrolled agent beyond lifecycle correctness checks.

## Step 0 Scope Challenge

### What already exists

| Sub-problem | Existing surface to reuse | Reuse decision |
| --- | --- | --- |
| Approval truth | `crates/xtask/src/approval_artifact.rs` | Reuse directly. Approval remains frozen upstream truth. |
| Registry truth | `crates/xtask/src/agent_registry.rs` | Reuse directly. Do not duplicate agent location or publication flags. |
| Control-plane enrollment | `crates/xtask/src/onboard_agent.rs` | Reuse and extend. Seed lifecycle state here. |
| Wrapper shell generation | `crates/xtask/src/wrapper_scaffold.rs` | Reuse unchanged except docs/help text references if needed. |
| Runtime lane | `crates/xtask/src/runtime_follow_on.rs` + `models.rs` + `render.rs` | Reuse and widen. Keep runtime write boundary exactly where it is. |
| Publication truth derivation | `crates/xtask/src/support_matrix.rs`, `crates/xtask/src/capability_matrix.rs` | Reuse. Add lifecycle continuity checks around them, not a rewrite. |
| Proving-run closeout | `crates/xtask/src/close_proving_run.rs` | Reuse and extend so closeout records lifecycle baseline. |
| Maintenance drift | `crates/xtask/src/agent_maintenance/drift/publication.rs` | Reuse and extend so drift compares against lifecycle baseline. |
| Operator procedure | `docs/cli-agent-onboarding-factory-operator-guide.md` | Reuse and update in place. |
| Existing backlog intent | `TODOS.md` | Reuse. This plan closes the lifecycle truth gap behind the existing runtime/publication follow-ons. |

### Minimum complete change set

The smallest complete version of this work is:

1. add one shared lifecycle schema module in `xtask`
2. commit one lifecycle-state file per onboarded agent
3. seed lifecycle state in `onboard-agent`
4. update `runtime-follow-on` to read and advance lifecycle state
5. add one new `prepare-publication` command and one committed `publication-ready.json` packet
6. update `close-proving-run` and maintenance drift checks to consume lifecycle state
7. add tests and doc updates for every transition

Anything smaller leaves the repo with partial truth again.

### Complexity check

This plan touches more than 8 files and more than 2 modules. That is a smell by default. Here it is justified because the change crosses five already-shipped lifecycle owners:

- approval
- onboarding
- runtime
- publication/closeout
- maintenance

The scope reduction decision is:

- keep one new shared module: `crates/xtask/src/agent_lifecycle.rs`
- keep one new command: `xtask prepare-publication`
- do not add a second handoff artifact family beyond `lifecycle-state.json` and `publication-ready.json`
- do not create a new crate
- do not change runtime write boundaries

### Search/build check

Repo search already shows the right primitives. Reuse them.

- Use the existing `xtask` subcommand pattern in `crates/xtask/src/main.rs`.
- Use the existing `serde` JSON model pattern from `runtime_follow_on/models.rs`.
- Use existing path validation and repo-root resolution patterns from `approval_artifact.rs`, `onboard_agent.rs`, and `close_proving_run.rs`.
- Use existing publication consistency checks in `support_matrix.rs` and drift inspection logic in `agent_maintenance/drift/publication.rs`.
- Keep the current explicit backend capability inventory in `capability_matrix.rs`; add lifecycle-aware validation around it instead of spending an innovation token on dynamic backend loading.

### TODOS cross-reference

This plan does not require a new backlog theme.

It turns the current diagnosis into an implementation path behind these existing TODOs:

- `Enclose The Runtime Follow-On In A Codex Exec Runner`
- `Enclose The Publication Refresh Follow-On After The Runtime Runner`

No new TODO entry is required in this milestone. The plan itself is the missing contract.

### Completeness decision

The shortcut version would be:

- add more prose to `HANDOFF.md`
- maybe widen `handoff.json`
- keep lifecycle truth split across commands

That is not acceptable. The repo already has deterministic machinery. With AI-assisted coding, the extra cost of a real lifecycle record, real transition validation, and real regression tests is low. This is a boilable lake.

### Distribution check

No new distributed artifact type is introduced. This is an internal `xtask` and committed-governance change.

- no new binary distribution pipeline
- no new package manager surface
- no container or service rollout

The only new machine-readable artifacts are committed repo governance files:

- `lifecycle-state.json`
- `publication-ready.json`

## Architecture

### Current to target flow

```text
recommend-next-agent
  -> approved-agent.toml
  -> onboard-agent --write
       seeds lifecycle-state.json as enrolled/bootstrap
  -> scaffold-wrapper-crate --write
       writes crate shell only
  -> runtime-follow-on --dry-run
       freezes packet from approval + registry + lifecycle state
  -> runtime-follow-on --write
       writes runtime-owned code/evidence
       updates lifecycle-state.json to runtime_integrated
  -> prepare-publication --write
       validates publication continuity
       writes publication-ready.json
       updates lifecycle-state.json to publication_ready
  -> support-matrix / capability-matrix / capability-matrix-audit / preflight
       refresh and verify published surfaces
  -> close-proving-run --write
       validates green publication state
       advances lifecycle state to closed_baseline
  -> check-agent-drift / refresh-agent / close-agent-maintenance
       compare future repo truth against lifecycle baseline
```

### Authority and precedence

| Domain | Authoritative source | Writers | Consumers |
| --- | --- | --- | --- |
| Approval truth | `approved-agent.toml` | recommendation/promotion flow | onboard-agent, runtime-follow-on, prepare-publication |
| Registry truth | `crates/xtask/data/agent_registry.toml` | onboard-agent | runtime-follow-on, publication generators, maintenance |
| Lifecycle progression | `docs/agents/lifecycle/<pack>/governance/lifecycle-state.json` | onboard-agent, runtime-follow-on, prepare-publication, close-proving-run, maintenance closeout | all lifecycle commands |
| Runtime implementation summary | `lifecycle-state.json` `implementation_summary` | runtime-follow-on | prepare-publication, close-proving-run, maintenance |
| Publication handoff | `docs/agents/lifecycle/<pack>/governance/publication-ready.json` | prepare-publication | operator workflow, close-proving-run |
| Published support/capability surfaces | `support-matrix`, `capability-matrix`, `capability-matrix-audit` outputs | publication commands | close-proving-run, maintenance drift |
| Maintenance drift baseline | `lifecycle-state.json` + create/maintenance closeout artifacts | close-proving-run, maintenance closeout | drift checks |

Precedence rules:

1. `approved-agent.toml` owns approval declaration truth. Lifecycle state may reference it, never override it.
2. `agent_registry.toml` owns repo placement and publication enrollment flags. Lifecycle state may validate continuity, never shadow those fields.
3. `lifecycle-state.json` owns stage, tier, side-state, evidence satisfaction, and next-command truth.
4. `publication-ready.json` is the only committed publication handoff artifact. Scratch `handoff.json` stays run evidence only.
5. Published support/capability surfaces remain derived outputs, not lifecycle authority.

### Important terminology decision

There are two different "tier" concepts in this repo today. They must stay separate.

1. `runtime profile`
   - current `runtime-follow-on` vocabulary
   - values: `minimal`, `default`, `feature_rich`
   - meaning: how much runtime implementation was requested/landed

2. `support tier`
   - new lifecycle overlay
   - values: `bootstrap`, `baseline_runtime`, `publication_backed`, `first_class`
   - meaning: how trustworthy and complete the repo's end-to-end support posture is

Do not overload one for the other. `runtime-follow-on` summary fields use runtime profile terms. `lifecycle-state.json` top-level uses support tier terms.

### Lifecycle state contract

File:

- `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/lifecycle-state.json`

Serialization rules:

- JSON only
- `schema_version = "1"`
- string enums only, no numeric discriminants
- lifecycle and side-state enums use `snake_case`
- command names are stored as literal command strings

Top-level fields:

- `schema_version: "1"`
- `agent_id: string`
- `onboarding_pack_prefix: string`
- `approval_artifact_path: string`
- `approval_artifact_sha256: string`
- `lifecycle_stage: approved | enrolled | runtime_integrated | publication_ready | published | closed_baseline`
- `support_tier: bootstrap | baseline_runtime | publication_backed | first_class`
- `side_states: string[]`
  - allowed: `blocked`, `failed_retryable`, `drifted`, `deprecated`
- `current_owner_command: string`
- `expected_next_command: string`
- `last_transition_at: RFC3339 string`
- `last_transition_by: string`
- `required_evidence: string[]`
- `satisfied_evidence: string[]`
- `blocking_issues: string[]`
- `retryable_failures: string[]`
- `implementation_summary: object | null`
- `publication_packet_path: string | null`
- `publication_packet_sha256: string | null`
- `closeout_baseline_path: string | null`

`implementation_summary` fields:

- `requested_runtime_profile: minimal | default | feature_rich`
- `achieved_runtime_profile: minimal | default | feature_rich`
- `primary_template: opencode | gemini_cli | codex | claude_code | aider`
- `template_lineage: string[]`
- `landed_surfaces: string[]`
- `deferred_surfaces: { surface: string, reason: string }[]`
- `minimal_profile_justification: string | null`

Allowed `landed_surfaces` / `deferred_surfaces.surface` values:

- `wrapper_runtime`
- `backend_harness`
- `agent_api_onboarding_test`
- `wrapper_coverage_source`
- `runtime_manifest_evidence`
- `add_dirs`
- `external_sandbox_policy`
- `mcp_management`
- `session_resume`
- `session_fork`
- `structured_tools`

Writer rules:

- `onboard-agent`: may create the file and set `approved -> enrolled`
- `scaffold-wrapper-crate`: must not mutate lifecycle state
- `runtime-follow-on`: may advance `enrolled -> runtime_integrated`
- `prepare-publication`: may advance `runtime_integrated -> publication_ready`
- `close-proving-run`: may atomically validate `published` state and write final `closed_baseline`
- maintenance closeout: may clear `drifted`, update satisfied evidence, and keep stage at `closed_baseline`

### Legal stage and support-tier combinations

| Stage | Allowed support tier(s) |
| --- | --- |
| `approved` | `bootstrap` |
| `enrolled` | `bootstrap` |
| `runtime_integrated` | `bootstrap`, `baseline_runtime` |
| `publication_ready` | `baseline_runtime` |
| `published` | `publication_backed`, `first_class` |
| `closed_baseline` | `baseline_runtime`, `publication_backed`, `first_class` |

Illegal combinations that must fail validation:

- `approved` with anything above `bootstrap`
- `published` with `bootstrap`
- `first_class` before `published`
- `closed_baseline` with `bootstrap`

### Publication-ready packet contract

File:

- `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/publication-ready.json`

Producer:

- `cargo run -p xtask -- prepare-publication --approval <path> --write`

Validator:

- `cargo run -p xtask -- prepare-publication --approval <path> --check`

Fields:

- `schema_version: "1"`
- `agent_id: string`
- `approval_artifact_path: string`
- `approval_artifact_sha256: string`
- `lifecycle_state_path: string`
- `lifecycle_state_sha256: string`
- `lifecycle_stage: "publication_ready"`
- `support_tier_at_emit: "baseline_runtime"`
- `manifest_root: string`
- `expected_targets: string[]`
- `capability_publication_enabled: bool`
- `support_publication_enabled: bool`
- `capability_matrix_target: string | null`
- `required_commands: string[]`
- `required_publication_outputs: string[]`
- `runtime_evidence_paths: string[]`
- `publication_owned_paths: string[]`
- `blocking_issues: string[]`
- `implementation_summary: object`

This packet is the only committed publication handoff. Keep `handoff.json` as run evidence, not as long-lived lifecycle truth.

### Command responsibility matrix

| Command | Transition | New responsibilities |
| --- | --- | --- |
| `onboard-agent` | `approved -> enrolled` | Seed lifecycle state, set bootstrap tier, define required evidence, define next command |
| `scaffold-wrapper-crate` | none | No lifecycle write. Still shell only. |
| `runtime-follow-on --dry-run` | none | Refuse to prepare without enrolled lifecycle state; carry approval + publication truth into frozen packet |
| `runtime-follow-on --write` | `enrolled -> runtime_integrated` | Validate runtime-owned writes, write `implementation_summary`, mark satisfied runtime evidence |
| `prepare-publication --write` | `runtime_integrated -> publication_ready` | Validate approval continuity, runtime evidence completeness, publication command requirements, write packet |
| `support-matrix` / `capability-matrix` | none | Keep deriving global published surfaces; no lifecycle writes |
| `close-proving-run` | `publication_ready -> published -> closed_baseline` | Validate green publication state, record baseline path, finalize support tier |
| `check-agent-drift` | none | Compare published surfaces against lifecycle baseline; read-only |
| `close-agent-maintenance` | `closed_baseline + drifted -> closed_baseline` | Clear drift and update evidence after maintenance closeout |

## Implementation Plan

### Milestone 1 - Shared lifecycle schema and backfill

Goal: land the shared contract first, without changing the runtime/publication boundary yet.

#### Files

- New:
  - `crates/xtask/src/agent_lifecycle.rs`
  - `crates/xtask/tests/agent_lifecycle_state.rs`
  - `docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json`
  - `docs/agents/lifecycle/claude-code-cli-onboarding/governance/lifecycle-state.json`
  - `docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json`
  - `docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json`
  - `docs/agents/lifecycle/aider-onboarding/governance/lifecycle-state.json`
- Updated:
  - `crates/xtask/src/lib.rs`
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
  - `docs/specs/cli-agent-onboarding-charter.md`

#### Work

1. Add `agent_lifecycle.rs` with:
   - state and packet structs
   - enum validation
   - path helpers
   - load/write helpers
   - stage/tier compatibility validation
2. Backfill lifecycle state for existing agents.
3. Do not fabricate historical onboarding packet docs for legacy agents. Backfill only the governance JSON where the pack is missing today.
4. Add tests for:
   - legal/illegal stage+tier combinations
   - invalid side-state strings
   - path validation
   - sha/path round-trip behavior

#### Initial backfill targets

Use these exact initial states unless the implementing diff finds committed evidence that contradicts them:

| Agent | Lifecycle stage | Support tier | Reason |
| --- | --- | --- | --- |
| `codex` | `closed_baseline` | `first_class` | strongest current wrapper, capability, and support posture |
| `claude_code` | `closed_baseline` | `first_class` | same category as codex |
| `opencode` | `closed_baseline` | `publication_backed` | published support/capability evidence exists, but not first-class |
| `gemini_cli` | `closed_baseline` | `publication_backed` | proving-run closeout exists and publication surfaces are enrolled |
| `aider` | `runtime_integrated` | `baseline_runtime` | wrapper/backend landed, but no closed create-mode baseline yet |

If an audit during implementation contradicts this table, the code should trust committed evidence and update the table in the same PR.

#### Acceptance

- `cargo test -p xtask --test agent_lifecycle_state`
- `make check`

### Milestone 2 - Onboarding and runtime integration

Goal: make lifecycle state part of the create lane before publication.

#### Files

- Updated:
  - `crates/xtask/src/onboard_agent.rs`
  - `crates/xtask/src/runtime_follow_on.rs`
  - `crates/xtask/src/runtime_follow_on/models.rs`
  - `crates/xtask/src/runtime_follow_on/render.rs`
  - `crates/xtask/templates/runtime_follow_on_codex_prompt.md`
  - `crates/xtask/tests/onboard_agent_entrypoint/write_mode.rs`
  - `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
  - `docs/cli-agent-onboarding-factory-operator-guide.md`

#### Work

1. `onboard-agent --dry-run`
   - preview the new `lifecycle-state.json`
   - show `current_owner_command = "onboard-agent"`
   - show `expected_next_command = "scaffold-wrapper-crate --agent <agent_id> --write"`
2. `onboard-agent --write`
   - create `lifecycle-state.json`
   - set:
     - `lifecycle_stage = enrolled`
     - `support_tier = bootstrap`
     - `required_evidence = [registry_entry, docs_pack, manifest_root_skeleton]`
3. `runtime-follow-on --dry-run`
   - require enrolled lifecycle state
   - extend `InputContract` with approval capability/publication fields:
     - `canonical_targets`
     - `always_on_capabilities`
     - `target_gated_capabilities`
     - `config_gated_capabilities`
     - `backend_extensions`
     - `support_matrix_enabled`
     - `capability_matrix_enabled`
     - `capability_matrix_target`
   - render those into `codex-prompt.md`
4. `runtime-follow-on --write`
   - on success, update `lifecycle-state.json`
   - set:
     - `lifecycle_stage = runtime_integrated`
     - `support_tier = baseline_runtime`
     - `implementation_summary = ...`
     - `expected_next_command = "prepare-publication --approval <path> --write"`
   - on validation failure:
     - add `failed_retryable` or `blocked`
     - append exact blocker text
   - do not write `publication-ready.json`
5. Keep scratch artifacts (`handoff.json`, `run-status.json`, `run-summary.md`) as evidence only.

#### Acceptance

- `cargo test -p xtask --test onboard_agent_entrypoint`
- `cargo test -p xtask --test runtime_follow_on_entrypoint`
- targeted dry-run/write fixture coverage for lifecycle state creation and update

### Milestone 3 - Prepare-publication seam

Goal: add the missing deterministic bridge between runtime truth and publication refresh.

#### Files

- New:
  - `crates/xtask/src/prepare_publication.rs`
  - `crates/xtask/tests/prepare_publication_entrypoint.rs`
- Updated:
  - `crates/xtask/src/lib.rs`
  - `crates/xtask/src/main.rs`
  - `crates/xtask/src/capability_matrix.rs`
  - `crates/xtask/src/support_matrix.rs`
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
  - `docs/specs/cli-agent-onboarding-charter.md`

#### Command contract

```sh
cargo run -p xtask -- prepare-publication --approval docs/agents/lifecycle/<pack>/governance/approved-agent.toml --write
cargo run -p xtask -- prepare-publication --approval docs/agents/lifecycle/<pack>/governance/approved-agent.toml --check
```

#### Work

1. Add `PreparePublication` subcommand wiring in `main.rs`.
2. `prepare-publication --write` must:
   - load approval artifact
   - load lifecycle state
   - require `lifecycle_stage = runtime_integrated`
   - validate approval sha/path continuity
   - validate runtime evidence paths exist
   - validate runtime summary is explicit
   - validate required publication commands are exactly:
     - `support-matrix --check`
     - `capability-matrix --check`
     - `capability-matrix-audit`
     - `make preflight`
   - write `publication-ready.json`
   - update lifecycle state to:
     - `lifecycle_stage = publication_ready`
     - `expected_next_command = "support-matrix && capability-matrix && capability-matrix-audit && make preflight"`
3. `prepare-publication --check` must re-validate the packet against current lifecycle/approval/runtime truth without rewriting.
4. Add a capability inventory continuity check:
   - if an agent has `capability_matrix_enabled = true` but `capability_matrix.rs` cannot construct runtime capabilities for it, fail `prepare-publication` with an explicit error
   - do not refactor backend loading dynamically in this milestone
5. Do not let `prepare-publication` write any published support/capability outputs.

#### Acceptance

- `cargo test -p xtask --test prepare_publication_entrypoint`
- `cargo test -p xtask --test runtime_follow_on_entrypoint`
- `make check`

### Milestone 4 - Closeout and maintenance continuity

Goal: make create-mode closeout the baseline that future maintenance compares against.

#### Files

- Updated:
  - `crates/xtask/src/close_proving_run.rs`
  - `crates/xtask/src/agent_maintenance/closeout.rs`
  - `crates/xtask/src/agent_maintenance/drift/publication.rs`
  - `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_paths.rs`
  - `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`
  - `crates/xtask/tests/agent_maintenance_closeout.rs`
  - `crates/xtask/tests/agent_maintenance_drift.rs`
  - `docs/cli-agent-onboarding-factory-operator-guide.md`

#### Work

1. `close-proving-run`
   - require `lifecycle_stage = publication_ready`
   - validate the published support/capability surfaces are green for the agent
   - record `publication-ready.json` continuity
   - update lifecycle state to:
     - `lifecycle_stage = closed_baseline`
     - `publication_packet_path = ...`
     - `closeout_baseline_path = ...`
     - `side_states` cleared except `deprecated`
   - default future-agent tier outcome to `publication_backed`
2. Keep `first_class` auto-promotion out of scope.
   - Existing legacy backfills may stay `first_class`
   - Future automatic first-class elevation is deferred
3. `check-agent-drift`
   - stay read-only
   - compare published support/capability outputs against lifecycle baseline
   - when drift is found, report that the agent should be treated as `closed_baseline + drifted`
4. `close-agent-maintenance`
   - clear `drifted`
   - update evidence fields in lifecycle state

#### Acceptance

- `cargo test -p xtask --test onboard_agent_closeout_preview`
- `cargo test -p xtask --test agent_maintenance_closeout`
- `cargo test -p xtask --test agent_maintenance_drift`
- `make test`

## Code Quality Rules

1. Keep all lifecycle schema code in `crates/xtask/src/agent_lifecycle.rs`. Do not duplicate stage or tier enums in multiple commands.
2. Keep JSON enums explicit and string-backed. No numeric discriminants.
3. Keep lifecycle write logic behind one helper API. Commands should describe transitions, not hand-roll file mutations.
4. Keep scratch runtime artifacts and committed lifecycle artifacts separate.
5. Do not add a second committed runtime summary file. `implementation_summary` belongs inside `lifecycle-state.json`.
6. Do not build a generic workflow engine. One shared module plus explicit command logic is enough.
7. Update nearby ASCII diagrams if the flow changes during implementation. Stale diagrams are a bug.

## Test Review

### Test framework

This repo is a Rust workspace.

- primary test runner: `cargo test`
- repo gate: `make test`
- targeted crate gate: `cargo test -p xtask`

### Code path coverage plan

```text
CODE PATH COVERAGE PLAN
=======================
[+] agent_lifecycle.rs
    ├── [NEW TEST] legal stage+tier matrix
    ├── [NEW TEST] illegal stage+tier combinations
    ├── [NEW TEST] lifecycle-state path validation
    └── [NEW TEST] publication-ready packet validation

[+] onboard_agent::run
    ├── [UPDATE TEST] dry-run previews lifecycle-state.json
    ├── [UPDATE TEST] write seeds enrolled/bootstrap state
    └── [GAP TODAY -> PLAN] duplicate or conflicting lifecycle-state seed rejected

[+] runtime_follow_on::build_context / persist_dry_run_artifacts
    ├── [UPDATE TEST] input-contract carries approval capability/publication truth
    ├── [UPDATE TEST] prompt renders that truth
    ├── [UPDATE TEST] dry-run requires enrolled lifecycle state
    └── [UPDATE TEST] scratch handoff remains non-authoritative

[+] runtime_follow_on::validate_write_mode
    ├── [UPDATE TEST] success advances lifecycle to runtime_integrated
    ├── [UPDATE TEST] failure writes failed_retryable or blocked state
    ├── [UPDATE TEST] implementation_summary required
    └── [UPDATE TEST] publication-ready packet is not written here

[+] prepare_publication::run
    ├── [NEW TEST] write requires runtime_integrated state
    ├── [NEW TEST] write rejects missing runtime evidence
    ├── [NEW TEST] write rejects capability inventory mismatch
    ├── [NEW TEST] write emits publication-ready.json
    └── [NEW TEST] check mode detects stale or inconsistent packet

[+] close_proving_run::run
    ├── [UPDATE TEST] requires publication_ready state
    ├── [UPDATE TEST] rejects stale published surfaces
    ├── [UPDATE TEST] writes closed_baseline and baseline path
    └── [UPDATE TEST] does not auto-promote to first_class

[+] agent_maintenance drift + closeout
    ├── [UPDATE TEST] drift reads lifecycle baseline
    ├── [UPDATE TEST] drift reports lifecycle continuity mismatch
    └── [UPDATE TEST] maintenance closeout clears drifted
```

### User flow coverage plan

```text
USER FLOW COVERAGE
==================
[+] New agent create lane
    ├── approval -> onboard-agent -> scaffold-wrapper-crate -> runtime-follow-on -> prepare-publication -> publication -> close-proving-run
    └── [→E2E-ish fixture flow] cover with xtask fixture integration across commands

[+] Runtime retry flow
    ├── runtime-follow-on write fails validation
    ├── lifecycle marks failed_retryable
    └── second write succeeds and clears retryable failure

[+] Publication blocked flow
    ├── runtime lane succeeds
    ├── prepare-publication rejects missing evidence or inventory mismatch
    └── operator gets exact blocker and no publication-ready packet

[+] Maintenance drift flow
    ├── closed baseline exists
    ├── published support/capability surface drifts
    └── drift command reports lifecycle mismatch without mutating files
```

### Regression rules

These are mandatory regression tests:

1. `runtime-follow-on` must not regress its write boundary protections while adding lifecycle writes.
2. `support-matrix` and `capability-matrix` must continue generating their current outputs for existing agents unless the lifecycle plan explicitly changes those outputs.
3. `close-proving-run` must keep its current path validation behavior while adding lifecycle updates.

### Required test files

- New:
  - `crates/xtask/tests/agent_lifecycle_state.rs`
  - `crates/xtask/tests/prepare_publication_entrypoint.rs`
- Updated:
  - `crates/xtask/tests/onboard_agent_entrypoint/write_mode.rs`
  - `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
  - `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_paths.rs`
  - `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`
  - `crates/xtask/tests/agent_maintenance_closeout.rs`
  - `crates/xtask/tests/agent_maintenance_drift.rs`

## Performance Review

This is not a request-path performance project. The risks are repo-tooling latency and repeated file parsing.

Rules:

1. Parse approval artifact and lifecycle state once per command invocation.
2. Do not rescan all lifecycle packs from runtime-follow-on or prepare-publication.
3. Keep publication validation agent-scoped where possible. If a global generator must run, reuse existing derivation instead of adding a second pass.
4. Do not hash large trees repeatedly inside one command. Hash the specific lifecycle/packet files being validated.
5. Keep backfill manual and committed. Do not add a repo-wide migration runner that walks everything on every invocation.

Potential slow paths to watch:

- repeated `support_matrix` and `capability_matrix` full derivations during `prepare-publication`
- repeated `fs::canonicalize` and JSON parsing in hot loops
- double-reading the same lifecycle file from command and validator helper

## Failure Modes Registry

| Codepath | Realistic failure | Test required | Error handling required | User-visible result |
| --- | --- | --- | --- | --- |
| `onboard-agent` lifecycle seed | pack path exists but lifecycle file is malformed or divergent | yes | reject write with exact path and field | clear validation error |
| `runtime-follow-on` lifecycle read | runtime packet prepared against missing or non-enrolled state | yes | reject dry-run/write | clear error, no scratch ambiguity |
| `runtime-follow-on` lifecycle update | runtime writes succeed but lifecycle write fails | yes | fail command, leave scratch evidence, do not claim success | explicit failure, no silent partial close |
| `prepare-publication` capability continuity | agent enrolled for capability publication but inventory cannot construct runtime backend capabilities | yes | reject packet creation | explicit blocker naming `capability_matrix.rs` gap |
| `prepare-publication` missing evidence | runtime stage set but wrapper coverage/report evidence absent | yes | reject packet creation | explicit blocker list |
| `close-proving-run` stale publication surfaces | packet exists but published surfaces are stale or contradictory | yes | reject closeout | agent cannot reach closed baseline falsely |
| maintenance drift | published surface changes but lifecycle baseline not consulted | yes | report drift mismatch | actionable drift finding |

Critical gaps that must not ship:

- lifecycle update failure with command success
- publication-ready packet emitted without runtime evidence
- closeout succeeding without green published surfaces
- maintenance drift check ignoring lifecycle baseline

## NOT in scope

- Dynamic backend loading for `capability_matrix.rs`
- Automatic first-class promotion rules for future agents
- A one-command create lane from approval through closeout
- Moving maintenance artifacts out of `docs/agents/lifecycle/<agent>-maintenance/**`
- Replacing `runtime-follow-on` scratch artifacts with committed governance artifacts
- Deleting `support_matrix_enabled` or `capability_matrix_enabled` from the registry in this milestone

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| M1 shared lifecycle schema + backfill | `crates/xtask/src/`, `crates/xtask/tests/`, `docs/agents/lifecycle/` | — |
| M2 onboard + runtime integration | `crates/xtask/src/`, `crates/xtask/tests/`, `crates/xtask/templates/` | M1 |
| M3 prepare-publication seam | `crates/xtask/src/`, `crates/xtask/tests/`, `docs/` | M2 |
| M4 closeout + maintenance continuity | `crates/xtask/src/`, `crates/xtask/tests/`, `docs/` | M3 |
| Docs polish and contract sync | `docs/specs/`, `docs/cli-agent-onboarding-factory-operator-guide.md` | M1 schema freeze, then parallel with M2/M3 code |

### Parallel lanes

- Lane A: M1 shared lifecycle schema -> M2 onboard/runtime integration -> M3 prepare-publication -> M4 closeout/maintenance
  - sequential, shared `crates/xtask/src/`
- Lane B: docs and lifecycle backfill content after M1 schema freeze
  - can run in parallel with M2 and M3 if it avoids parent-owned command logic
- Lane C: test-file updates after each milestone freeze
  - can run in parallel with docs lane, but not with active edits to the same command module

### Execution order

1. Land M1 schema and backfill first.
2. Once the lifecycle schema is frozen, launch:
   - Lane A code work for M2
   - Lane B docs/backfill polish
3. After M2 code freeze, run Lane C test updates in parallel with remaining docs work.
4. Repeat the same pattern for M3 and M4.

### Conflict flags

- `crates/xtask/src/main.rs` is a conflict magnet. Only one lane should own it at a time.
- `crates/xtask/src/runtime_follow_on.rs`, `models.rs`, and `render.rs` should stay in one lane.
- `docs/cli-agent-onboarding-factory-operator-guide.md` will conflict with any lane rewriting the workflow steps. Keep one docs owner.

Practical answer:

- parallelize docs and tests after each command-contract freeze
- keep lifecycle command logic sequential

## Completion Summary

- Step 0: Scope Challenge — accepted with one shared module and one new command
- Architecture Review: lifecycle truth unified without a new crate
- Code Quality Review: duplicate state vocabularies explicitly forbidden
- Test Review: coverage diagram produced, all new transitions mapped to tests
- Performance Review: repo-tooling slow paths identified and bounded
- NOT in scope: written
- What already exists: written
- Failure modes: critical gaps identified and blocked from shipping
- Parallelization: 3 lanes, 1 primary sequential lane plus 2 safe side lanes
- Lake Score: 8/8 major recommendations chose the complete option over the shortcut

## Implementation Order

Do the work in this order:

1. M1 schema + backfill
2. M2 onboard/runtime integration
3. M3 prepare-publication
4. M4 closeout/maintenance continuity

Do not start with publication generators. The lifecycle contract has to exist first or the rest of the work turns into another thin handoff file.
