# PLAN - Unified Agent Lifecycle Support Maturity Model

Status: implementation-ready
Date: 2026-05-01
Branch: `codex/recommend-next-agent`
Base branch: `main`
Repo: `atomize-hq/unified-agent-api`
Work item: `Unified agent lifecycle support maturity model`

## Objective

Make agent support maturity explicit, committed, and machine-checked from approval through maintenance.

This repo already has the pieces: approval artifacts, control-plane onboarding, wrapper scaffolding, bounded runtime implementation, publication generators, create-mode closeout, and maintenance drift checks. What it still lacks is one canonical lifecycle record that every stage can read, validate, and advance. This plan adds that record, adds the missing `prepare-publication` seam, and turns the current create and maintenance lanes into one deterministic lifecycle without inventing a new orchestration system.

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
  - `docs/specs/agent-registry-contract.md`
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
  - `crates/xtask/src/agent_maintenance/closeout.rs`
- Existing tests and fixtures:
  - `crates/xtask/tests/onboard_agent_entrypoint/**`
  - `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
  - `crates/xtask/tests/onboard_agent_closeout_preview/**`
  - `crates/xtask/tests/agent_maintenance_closeout.rs`
  - `crates/xtask/tests/agent_maintenance_drift.rs`
  - `crates/xtask/tests/fixtures/fake_codex.sh`
- Queue context:
  - `TODOS.md`

## Outcome

After this plan lands:

1. Every onboarded agent has one committed lifecycle record at `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/lifecycle-state.json`.
2. `onboard-agent` seeds lifecycle state, `runtime-follow-on` advances it to `runtime_integrated`, `prepare-publication` advances it to `publication_ready`, and `close-proving-run` seals the create-mode baseline.
3. The runtime lane still owns runtime code only, but it now emits enough committed truth for publication and maintenance to proceed without archaeology.
4. The publication seam becomes explicit: `prepare-publication` validates readiness and writes the only committed publication handoff packet.
5. Maintenance drift checks compare published truth against the committed lifecycle baseline instead of inferring state from scattered artifacts.

## Problem Statement

The repo has multiple truthful subsystems and no single truthful lifecycle.

Approval artifacts know what an agent claims to be. The registry knows where it lives and whether it is enrolled for publication. `onboard-agent` knows how to register it. `runtime-follow-on` knows how to bound runtime work. `support-matrix` and `capability-matrix` know how to publish derived surfaces. `close-proving-run` and maintenance commands know how to record or compare evidence.

What is missing is the state machine that answers these questions in one place:

- what support level the agent has actually reached
- what evidence is still missing
- which command is legal to run next
- whether publication claims are legitimate yet
- what future maintenance must compare against

Without that lifecycle record, the repo can truthfully say an agent is enrolled while still not having publication-backed or first-class support semantics. That is the gap this plan closes.

## Scope

### In scope

- Add a canonical lifecycle record under `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/lifecycle-state.json`.
- Add one shared `xtask` lifecycle module for schema, loading, validation, and path helpers.
- Backfill lifecycle records for every agent in `crates/xtask/data/agent_registry.toml`.
- Teach `onboard-agent` to seed lifecycle state.
- Teach `runtime-follow-on` to read lifecycle state, carry approval capability and publication truth forward, and update lifecycle state on success or failure.
- Add `xtask prepare-publication` as the explicit `runtime_integrated -> publication_ready` seam.
- Teach `close-proving-run` and maintenance drift logic to consume lifecycle state.
- Update the operator guide and charter to reflect the new lifecycle contract.
- Add regression tests for every new transition and every new rejection path.

### Out of scope

- A new crate or service for lifecycle orchestration.
- Folding runtime implementation, publication refresh, and closeout into one giant command.
- Rewriting `support-matrix` or `capability-matrix` derivation from scratch.
- Dynamic backend discovery for capability publication.
- Automatic `first_class` promotion for future agents.
- Reworking every maintenance command into a new abstraction family.
- Expanding CI posture for every enrolled agent beyond lifecycle correctness checks in this milestone.

## Step 0 Scope Challenge

### What already exists

| Sub-problem | Existing surface to reuse | Reuse decision |
| --- | --- | --- |
| Approval truth | `crates/xtask/src/approval_artifact.rs` | Reuse directly. Approval remains frozen upstream truth. |
| Registry truth | `crates/xtask/src/agent_registry.rs` and `crates/xtask/data/agent_registry.toml` | Reuse directly. Do not duplicate repo location, manifest root, targets, or publication booleans. |
| Control-plane enrollment | `crates/xtask/src/onboard_agent.rs` | Reuse and extend. Seed lifecycle state here. |
| Wrapper shell generation | `crates/xtask/src/wrapper_scaffold.rs` | Reuse unchanged. It still owns shell scaffolding only. |
| Runtime lane | `crates/xtask/src/runtime_follow_on.rs` plus `models.rs` and `render.rs` | Reuse and widen. Keep runtime write boundaries where they are now. |
| Publication truth derivation | `crates/xtask/src/support_matrix.rs`, `crates/xtask/src/capability_matrix.rs` | Reuse. Add lifecycle continuity checks around them, not a rewrite. |
| Create-mode closeout | `crates/xtask/src/close_proving_run.rs` | Reuse and extend so closeout records lifecycle baseline. |
| Maintenance drift | `crates/xtask/src/agent_maintenance/drift/publication.rs` and `closeout.rs` | Reuse and extend so drift compares against lifecycle baseline. |
| Operator procedure | `docs/cli-agent-onboarding-factory-operator-guide.md` | Reuse and update in place. |
| Existing backlog intent | `TODOS.md` | Reuse. This plan is the implementation contract behind the existing runtime and publication follow-ons. |

### Minimum complete change set

The smallest complete version of this work is:

1. add one shared lifecycle schema module in `xtask`
2. commit one lifecycle-state file per onboarded agent
3. seed lifecycle state in `onboard-agent`
4. update `runtime-follow-on` to read and advance lifecycle state
5. add one new `prepare-publication` command and one committed `publication-ready.json` packet
6. update `close-proving-run` and maintenance drift checks to consume lifecycle state
7. add tests and doc updates for every transition

Anything smaller leaves the repo with split truth again.

### Complexity check

This plan touches more than 8 files and more than 2 modules. That would normally be a smell. Here it is justified because the lifecycle truth already spans five shipped owners:

- approval
- onboarding
- runtime
- publication and closeout
- maintenance

The scope reduction decision is:

- keep one new shared module: `crates/xtask/src/agent_lifecycle.rs`
- keep one new command: `prepare-publication`
- keep one new committed packet: `publication-ready.json`
- keep lifecycle state embedded in one file, not a new family of per-stage governance docs
- keep runtime write boundaries unchanged
- keep publication generators as derived-output writers, not lifecycle authorities

### Search/build check

The repo already contains the right primitives. Reuse them.

- Use the existing `xtask` subcommand pattern in `crates/xtask/src/main.rs`.
- Use the existing `serde` JSON model pattern from `runtime_follow_on/models.rs`.
- Use existing path validation and repo-root resolution patterns from `approval_artifact.rs`, `onboard_agent.rs`, and `close_proving_run.rs`.
- Use existing publication consistency checks in `support_matrix.rs` and drift inspection logic in `agent_maintenance/drift/publication.rs`.
- Keep the current explicit backend capability inventory in `capability_matrix.rs`; add lifecycle-aware validation around it instead of spending an innovation token on dynamic backend loading.

Layer judgment:

- **[Layer 1]** Reuse existing `xtask` CLI wiring, `serde` JSON models, path normalization, and publication checks.
- **[Layer 1]** Reuse `agent_registry.toml` as the source of backfill targets and publication enrollment truth.
- **[Layer 3]** The repo needs one explicit lifecycle record. Another richer scratch handoff file would encode the wrong model.

### TODOS cross-reference

This plan does not need a new backlog theme.

It turns the current diagnosis into an implementation path behind these existing TODOs:

- `Enclose The Runtime Follow-On In A Codex Exec Runner`
- `Enclose The Publication Refresh Follow-On After The Runtime Runner`

No new TODO entry is required in this milestone. The plan itself is the missing contract.

### Completeness decision

The shortcut version would be:

- add more prose to `HANDOFF.md`
- widen `handoff.json`
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
       creates lifecycle-state.json
       writes stage=enrolled tier=bootstrap
  -> scaffold-wrapper-crate --write
       writes crate shell only
  -> runtime-follow-on --dry-run
       freezes packet from approval + registry + lifecycle state
  -> runtime-follow-on --write
       writes runtime-owned code/evidence
       updates lifecycle-state.json to stage=runtime_integrated tier=baseline_runtime
  -> prepare-publication --check/--write
       validates runtime continuity
       writes publication-ready.json
       updates lifecycle-state.json to stage=publication_ready
  -> support-matrix --check
  -> capability-matrix --check
  -> capability-matrix-audit
  -> make preflight
       refreshes and validates published surfaces
  -> close-proving-run --write
       validates publication surfaces against publication-ready.json
       writes create-mode baseline
       updates lifecycle-state.json to stage=closed_baseline tier=publication_backed
  -> check-agent-drift / refresh-agent / close-agent-maintenance
       compare future repo truth against lifecycle baseline
```

### Authority and precedence

| Domain | Authoritative source | Writers | Consumers |
| --- | --- | --- | --- |
| Approval truth | `approved-agent.toml` | recommendation and promotion flow | onboard-agent, runtime-follow-on, prepare-publication |
| Registry truth | `crates/xtask/data/agent_registry.toml` | onboard-agent and registry maintenance | runtime-follow-on, publication generators, maintenance |
| Lifecycle progression | `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/lifecycle-state.json` | onboard-agent, runtime-follow-on, prepare-publication, close-proving-run, maintenance closeout | all lifecycle commands |
| Runtime implementation summary | `lifecycle-state.json` `implementation_summary` | runtime-follow-on | prepare-publication, close-proving-run, maintenance |
| Publication handoff | `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/publication-ready.json` | prepare-publication | publication checks, close-proving-run |
| Published support and capability surfaces | support-matrix and capability-matrix outputs | publication commands | close-proving-run, maintenance drift |
| Maintenance drift baseline | `lifecycle-state.json` plus closeout artifacts | close-proving-run and maintenance closeout | drift checks |

Precedence rules:

1. `approved-agent.toml` owns approval declaration truth. Lifecycle state may reference it, never override it.
2. `agent_registry.toml` owns repo placement, manifest root, target list, and publication enrollment booleans. Lifecycle state may validate continuity, never shadow those fields.
3. `lifecycle-state.json` owns lifecycle stage, support tier, side-state, evidence satisfaction, and next-command truth.
4. `publication-ready.json` is the only committed publication handoff artifact. Scratch `handoff.json` stays run evidence only.
5. Published support and capability surfaces remain derived outputs, not lifecycle authority.

### Important terminology decision

There are two different "tier" concepts in the repo today. They must stay separate.

1. `runtime profile`
   - current `runtime-follow-on` vocabulary
   - values: `minimal`, `default`, `feature_rich`
   - meaning: how much runtime implementation was requested and landed

2. `support tier`
   - new lifecycle overlay
   - values: `bootstrap`, `baseline_runtime`, `publication_backed`, `first_class`
   - meaning: how trustworthy and complete the repo's end-to-end support posture is

Do not overload one for the other. `runtime-follow-on` summary fields use runtime profile terms. `lifecycle-state.json` top-level fields use support tier terms.

### Lifecycle state contract

File:

- `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/lifecycle-state.json`

Serialization rules:

- JSON only
- `schema_version = "1"`
- string enums only
- `snake_case` enum values only
- command names stored as literal command strings
- repo-relative paths only, never absolute paths

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

Allowed `landed_surfaces` and `deferred_surfaces.surface` values:

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

### Stable evidence vocabulary

`required_evidence` and `satisfied_evidence` use a fixed core vocabulary in v1:

- `registry_entry`
- `docs_pack`
- `manifest_root_skeleton`
- `runtime_write_complete`
- `implementation_summary_present`
- `publication_packet_written`
- `support_matrix_check_green`
- `capability_matrix_check_green`
- `capability_matrix_audit_green`
- `preflight_green`
- `proving_run_closeout_written`
- `maintenance_closeout_written`

Rules:

1. `onboard-agent --write` must seed the first three evidence ids.
2. `runtime-follow-on --write` must satisfy `runtime_write_complete` and `implementation_summary_present`.
3. `prepare-publication --write` must satisfy `publication_packet_written`.
4. `close-proving-run --write` must only succeed after the four publication checks are satisfied.
5. `close-agent-maintenance` must satisfy `maintenance_closeout_written` when clearing `drifted`.

This keeps lifecycle evidence machine-checkable without creating another artifact family.

### Resting versus transitional stages

The current plan had one ambiguity: it defined `published` as a lifecycle stage, then had `close-proving-run` jump straight to `closed_baseline`.

Resolve that ambiguity as follows:

- Persisted resting stages in v1 are `enrolled`, `runtime_integrated`, `publication_ready`, and `closed_baseline`.
- `approved` remains an allowed enum value for schema compatibility and dry-run reasoning, but no v1 command writes a committed lifecycle file in `approved`.
- `published` remains an allowed enum value for schema compatibility and future split publication/closeout flows, but no v1 command writes it as a resting state.
- `close-proving-run --write` may accept input lifecycle state `publication_ready` or a legacy/manual `published`, but on success it writes `closed_baseline` directly in one atomic lifecycle update.

That keeps the schema forward-compatible while removing the current implementation ambiguity.

### Legal stage and support-tier combinations

| Stage | Allowed support tier(s) | Persisted in v1 |
| --- | --- | --- |
| `approved` | `bootstrap` | no |
| `enrolled` | `bootstrap` | yes |
| `runtime_integrated` | `bootstrap`, `baseline_runtime` | yes |
| `publication_ready` | `baseline_runtime` | yes |
| `published` | `publication_backed`, `first_class` | no |
| `closed_baseline` | `publication_backed`, `first_class` | yes |

Illegal combinations that must fail validation:

- `approved` with anything above `bootstrap`
- `publication_ready` with anything other than `baseline_runtime`
- `closed_baseline` with `bootstrap`
- `first_class` before publication truth exists

### Backfill targets and exact lifecycle paths

Backfill targets are derived from `crates/xtask/data/agent_registry.toml`, not hand-authored guesses.

| Agent | `onboarding_pack_prefix` | Lifecycle state path | Initial state |
| --- | --- | --- | --- |
| `codex` | `codex-cli-onboarding` | `docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json` | `closed_baseline`, `first_class` |
| `claude_code` | `claude-code-cli-onboarding` | `docs/agents/lifecycle/claude-code-cli-onboarding/governance/lifecycle-state.json` | `closed_baseline`, `first_class` |
| `opencode` | `opencode-cli-onboarding` | `docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json` | `closed_baseline`, `publication_backed` |
| `gemini_cli` | `gemini-cli-onboarding` | `docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json` | `closed_baseline`, `publication_backed` |
| `aider` | `aider-onboarding` | `docs/agents/lifecycle/aider-onboarding/governance/lifecycle-state.json` | `runtime_integrated`, `baseline_runtime` |

Notes:

1. `opencode-maintenance` stays a maintenance pack. It is not the create-mode lifecycle state location.
2. If implementation finds committed evidence that contradicts the initial table, the code trusts committed evidence and the plan table is updated in the same PR.
3. Backfill creates only the governance JSON required by this plan. It does not fabricate missing historical onboarding prose.

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

This packet is the only committed publication handoff. `handoff.json` remains runtime run evidence only.

### Command responsibility matrix

| Command | Allowed input state | Output state | Responsibilities |
| --- | --- | --- | --- |
| `onboard-agent --write` | approval artifact only | `enrolled`, `bootstrap` | create lifecycle state, seed initial evidence, point to scaffold step |
| `scaffold-wrapper-crate --write` | any enrolled agent | none | no lifecycle write |
| `runtime-follow-on --dry-run` | `enrolled` | none | refuse to prepare without lifecycle state, carry approval and publication truth into packet and prompt |
| `runtime-follow-on --write` | `enrolled` | `runtime_integrated`, usually `baseline_runtime` | validate bounded writes, persist implementation summary, record runtime evidence, record retryable or blocking failures |
| `prepare-publication --check` | `runtime_integrated` or `publication_ready` | none | re-validate continuity and publication readiness without writing |
| `prepare-publication --write` | `runtime_integrated` | `publication_ready` | validate approval continuity, runtime evidence completeness, command requirements, write `publication-ready.json` |
| `support-matrix --check` | `publication_ready` | none | derived-output validation only |
| `capability-matrix --check` | `publication_ready` | none | derived-output validation only |
| `capability-matrix-audit` | `publication_ready` | none | publication audit only |
| `make preflight` | `publication_ready` | none | final green gate only |
| `close-proving-run --write` | `publication_ready` or legacy/manual `published` | `closed_baseline`, default `publication_backed` | validate green publication state, record baseline continuity, write final closeout |
| `check-agent-drift` | `closed_baseline` | none | compare current published truth against lifecycle baseline, read-only |
| `close-agent-maintenance` | `closed_baseline` with `drifted` or fresh maintenance request | `closed_baseline` | clear `drifted`, update evidence, keep approval truth frozen |

Command rejection rules:

- `onboard-agent` rejects divergent existing lifecycle files and approval or registry mismatches.
- `runtime-follow-on` rejects missing lifecycle state, wrong stage, or write sets outside runtime-owned paths.
- `prepare-publication` rejects missing runtime evidence, missing implementation summary, capability inventory mismatches, or approval SHA/path drift.
- `close-proving-run` rejects stale support or capability outputs, missing publication packet continuity, or unresolved blockers.

## Implementation Plan

### Milestone 1 - Shared lifecycle schema and backfill

Goal: land the shared contract first, without changing runtime or publication behavior yet.

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
   - lifecycle state structs
   - publication packet structs
   - enum validation
   - stage and support-tier compatibility validation
   - repo-relative path helpers
   - load and write helpers
   - transition helpers so commands describe transitions instead of hand-rolling JSON mutations
2. Backfill lifecycle state for the five registry agents using the exact path table above.
3. Encode the `published` rule exactly once in the shared lifecycle module so every command shares the same behavior.
4. Update the charter and operator guide so lifecycle state is described as authoritative create-mode and maintenance truth.

#### Acceptance

- `cargo test -p xtask --test agent_lifecycle_state`
- `cargo test -p xtask --test agent_registry`
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
   - preview the exact lifecycle-state file contents
   - show `current_owner_command = "onboard-agent --write"`
   - show `expected_next_command = "scaffold-wrapper-crate --agent <agent_id> --write"`
2. `onboard-agent --write`
   - create `lifecycle-state.json`
   - set:
     - `lifecycle_stage = enrolled`
     - `support_tier = bootstrap`
     - `required_evidence = ["registry_entry", "docs_pack", "manifest_root_skeleton"]`
     - `satisfied_evidence = ["registry_entry", "docs_pack", "manifest_root_skeleton"]`
3. `runtime-follow-on --dry-run`
   - require `lifecycle_stage = enrolled`
   - load approval artifact, registry entry, and lifecycle state together
   - extend `InputContract` with approval capability and publication truth:
     - `canonical_targets`
     - `always_on_capabilities`
     - `target_gated_capabilities`
     - `config_gated_capabilities`
     - `backend_extensions`
     - `support_matrix_enabled`
     - `capability_matrix_enabled`
     - `capability_matrix_target`
   - render those fields into `codex-prompt.md`
4. `runtime-follow-on --write`
   - on success:
    - update lifecycle state to `runtime_integrated`
    - set support tier to `baseline_runtime`
     - write `implementation_summary`
     - satisfy `runtime_write_complete` and `implementation_summary_present`
     - set `expected_next_command = "prepare-publication --approval <path> --write"`
   - on validation failure:
     - append `failed_retryable` or `blocked` to `side_states`
     - append exact blocker text
     - keep lifecycle stage unchanged if the write did not complete
   - never write `publication-ready.json`
5. Keep scratch artifacts (`handoff.json`, `run-status.json`, `run-summary.md`) as evidence only.

#### Acceptance

- `cargo test -p xtask --test onboard_agent_entrypoint`
- `cargo test -p xtask --test runtime_follow_on_entrypoint`
- targeted lifecycle creation and update coverage in dry-run and write-mode fixtures

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
cargo run -p xtask -- prepare-publication --approval docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml --write
cargo run -p xtask -- prepare-publication --approval docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml --check
```

#### Work

1. Add `PreparePublication` subcommand wiring in `main.rs`.
2. `prepare-publication --write` must:
   - load approval artifact
   - load lifecycle state
   - require `lifecycle_stage = runtime_integrated`
   - validate approval SHA and path continuity
   - validate runtime evidence paths exist
   - validate `implementation_summary` is explicit and non-empty
   - validate required publication commands are exactly:
     - `cargo run -p xtask -- support-matrix --check`
     - `cargo run -p xtask -- capability-matrix --check`
     - `cargo run -p xtask -- capability-matrix-audit`
     - `make preflight`
   - write `publication-ready.json`
   - update lifecycle state to:
     - `lifecycle_stage = publication_ready`
     - `support_tier = baseline_runtime`
     - `expected_next_command = "support-matrix --check && capability-matrix --check && capability-matrix-audit && make preflight && close-proving-run --write"`
     - satisfy `publication_packet_written`
3. `prepare-publication --check` must re-validate the packet against current lifecycle, approval, and runtime truth without rewriting.
4. Add a capability inventory continuity check:
   - if an agent has `capability_matrix_enabled = true` but `capability_matrix.rs` cannot construct runtime capabilities for it, fail `prepare-publication` with an explicit error
   - do not refactor backend loading dynamically in this milestone
5. Do not let `prepare-publication` write any support-matrix or capability-matrix outputs.

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
   - require `lifecycle_stage = publication_ready` or legacy/manual `published`
   - validate the published support and capability surfaces are green for the agent named by the lifecycle state
   - validate `publication-ready.json` continuity against lifecycle and approval state
   - update lifecycle state to:
     - `lifecycle_stage = closed_baseline`
     - `support_tier = publication_backed` for new create-lane agents
     - `publication_packet_path = ...`
     - `publication_packet_sha256 = ...`
     - `closeout_baseline_path = ...`
     - satisfy:
       - `support_matrix_check_green`
       - `capability_matrix_check_green`
       - `capability_matrix_audit_green`
       - `preflight_green`
       - `proving_run_closeout_written`
     - clear `blocked`, `failed_retryable`, and `drifted`
     - preserve `deprecated` if already present
2. Keep `first_class` auto-promotion out of scope.
   - legacy backfills for `codex` and `claude_code` may remain `first_class`
   - new agents default to `publication_backed`
3. `check-agent-drift`
   - stay read-only
   - compare published support and capability outputs against lifecycle baseline
   - report `closed_baseline + drifted` semantics without mutating files
4. `close-agent-maintenance`
   - clear `drifted`
   - update evidence fields
   - record maintenance closeout continuity without rewriting approval truth

#### Acceptance

- `cargo test -p xtask --test onboard_agent_closeout_preview`
- `cargo test -p xtask --test agent_maintenance_closeout`
- `cargo test -p xtask --test agent_maintenance_drift`
- `make test`

## Code Quality Rules

1. Keep all lifecycle schema code in `crates/xtask/src/agent_lifecycle.rs`. Do not duplicate stage or support-tier enums across commands.
2. Keep JSON enums explicit and string-backed. No numeric discriminants.
3. Keep lifecycle write logic behind one helper API. Commands describe transitions; they do not hand-roll JSON mutations.
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
    ├── [NEW TEST] persisted-vs-transitional stage rules
    ├── [NEW TEST] invalid side-state strings
    ├── [NEW TEST] lifecycle-state path validation
    └── [NEW TEST] publication-ready packet validation

[+] onboard_agent::run
    ├── [UPDATE TEST] dry-run previews lifecycle-state.json
    ├── [UPDATE TEST] write seeds enrolled/bootstrap state
    ├── [UPDATE TEST] duplicate or divergent lifecycle-state seed rejected
    └── [UPDATE TEST] exact evidence ids are populated

[+] runtime_follow_on::{build_context, validate_write_mode, persist_dry_run_artifacts}
    ├── [UPDATE TEST] input contract carries approval capability/publication truth
    ├── [UPDATE TEST] prompt renders that truth
    ├── [UPDATE TEST] dry-run requires enrolled lifecycle state
    ├── [UPDATE TEST] success advances lifecycle to runtime_integrated
    ├── [UPDATE TEST] failure writes failed_retryable or blocked without claiming success
    ├── [UPDATE TEST] implementation_summary is required
    └── [UPDATE TEST] publication-ready packet is not written here

[+] prepare_publication::run
    ├── [NEW TEST] write requires runtime_integrated state
    ├── [NEW TEST] write rejects approval SHA/path drift
    ├── [NEW TEST] write rejects missing runtime evidence
    ├── [NEW TEST] write rejects capability inventory mismatch
    ├── [NEW TEST] write emits publication-ready.json
    └── [NEW TEST] check mode detects stale or inconsistent packet

[+] close_proving_run::run
    ├── [UPDATE TEST] requires publication_ready or legacy published state
    ├── [UPDATE TEST] rejects stale published surfaces
    ├── [UPDATE TEST] writes closed_baseline and baseline path
    ├── [UPDATE TEST] satisfies green publication evidence ids
    └── [UPDATE TEST] does not auto-promote to first_class

[+] agent_maintenance drift + closeout
    ├── [UPDATE TEST] drift reads lifecycle baseline
    ├── [UPDATE TEST] drift reports lifecycle continuity mismatch
    ├── [UPDATE TEST] maintenance closeout clears drifted
    └── [UPDATE TEST] maintenance closeout does not rewrite approval truth
```

### User flow coverage plan

```text
USER FLOW COVERAGE
==================
[+] New agent create lane
    ├── approval -> onboard-agent -> scaffold-wrapper-crate -> runtime-follow-on
    ├── runtime-follow-on -> prepare-publication -> publication checks
    └── publication checks -> close-proving-run -> closed_baseline

[+] Runtime retry flow
    ├── runtime-follow-on write fails validation
    ├── lifecycle marks failed_retryable or blocked
    └── second write succeeds and clears retryable state

[+] Publication blocked flow
    ├── runtime lane succeeds
    ├── prepare-publication rejects missing evidence or inventory mismatch
    └── operator gets exact blocker and no publication-ready packet

[+] Maintenance drift flow
    ├── closed baseline exists
    ├── published support or capability surface drifts
    └── drift command reports lifecycle mismatch without mutating files
```

### Regression rules

These are mandatory regression tests:

1. `runtime-follow-on` must not regress its write-boundary protections while adding lifecycle writes.
2. `support-matrix` and `capability-matrix` must continue generating their current outputs for existing agents unless this plan explicitly changes those outputs.
3. `close-proving-run` must keep its current path-validation behavior while adding lifecycle updates.
4. `opencode-maintenance` must remain a maintenance pack; the new create-mode lifecycle state for `opencode` must live under `opencode-cli-onboarding`.

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
2. Do not rescan all lifecycle packs from `runtime-follow-on` or `prepare-publication`.
3. Keep publication validation agent-scoped where possible. If a global generator must run, reuse existing derivation instead of adding a second pass.
4. Do not hash large trees repeatedly inside one command. Hash the specific lifecycle and packet files being validated.
5. Keep backfill manual and committed. Do not add a repo-wide migration runner that walks everything on every invocation.

Potential slow paths to watch:

- repeated full support and capability derivations during `prepare-publication`
- repeated `fs::canonicalize` and JSON parsing in hot loops
- double-reading the same lifecycle file from command code and validator helpers

## Failure Modes Registry

| Codepath | Realistic failure | Test required | Error handling required | User-visible result |
| --- | --- | --- | --- | --- |
| `onboard-agent` lifecycle seed | lifecycle path already exists with divergent approval or registry truth | yes | reject write with exact file path and mismatch field | clear validation error |
| `runtime-follow-on` lifecycle read | runtime packet prepared against missing or non-enrolled state | yes | reject dry-run and write | clear error, no scratch ambiguity |
| `runtime-follow-on` lifecycle update | runtime writes succeed but lifecycle write fails | yes | fail command and do not claim success | explicit failure, no silent partial close |
| `prepare-publication` capability continuity | agent is publication-enabled but capability inventory cannot construct runtime capabilities | yes | reject packet creation | explicit blocker naming `capability_matrix.rs` gap |
| `prepare-publication` missing evidence | runtime stage is set but required runtime evidence or summary is absent | yes | reject packet creation | explicit blocker list |
| `close-proving-run` stale publication surfaces | packet exists but published surfaces are stale or contradictory | yes | reject closeout | agent cannot reach closed baseline falsely |
| maintenance drift | published surface changes but lifecycle baseline is ignored | yes | report drift mismatch | actionable drift finding |

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
| M1 schema core | `crates/xtask/src/`, `crates/xtask/tests/` | — |
| M1 lifecycle backfill | `docs/agents/lifecycle/` | M1 schema core |
| M1 docs sync | `docs/specs/`, `docs/cli-agent-onboarding-factory-operator-guide.md` | M1 schema core |
| M2 onboard-agent integration | `crates/xtask/src/onboard_agent*`, `crates/xtask/tests/onboard_agent_entrypoint/**` | M1 schema core |
| M2 runtime-follow-on integration | `crates/xtask/src/runtime_follow_on*`, `crates/xtask/templates/`, `crates/xtask/tests/runtime_follow_on_entrypoint.rs` | M1 schema core |
| M3 prepare-publication seam | `crates/xtask/src/main.rs`, `crates/xtask/src/prepare_publication.rs`, `crates/xtask/src/{support_matrix,capability_matrix}.rs`, `crates/xtask/tests/` | M2 onboard + runtime merged |
| M4 closeout + maintenance continuity | `crates/xtask/src/close_proving_run.rs`, `crates/xtask/src/agent_maintenance/**`, `crates/xtask/tests/` | M3 |

### Parallel lanes

- Lane A: M1 schema core -> M2 onboard-agent integration
  - sequential, shared lifecycle and onboarding ownership
- Lane B: M1 schema core -> M2 runtime-follow-on integration
  - parallel with Lane A after schema freeze, separate primary command surface
- Lane C: M1 schema core -> lifecycle backfill + charter/operator-guide sync
  - parallel with Lanes A and B after schema freeze, docs and governance only
- Lane D: M3 -> M4
  - sequential, shared `main.rs`, publication seam, closeout, and maintenance ownership

### Execution order

1. Land M1 schema core first. Nothing else starts until the lifecycle schema, path rules, and stage semantics are frozen.
2. After M1 freezes, launch in parallel:
   - Lane A for `onboard-agent`
   - Lane B for `runtime-follow-on`
   - Lane C for backfill JSONs and doc sync
3. Merge A + B + C, then run full create-lane fixture coverage.
4. Launch Lane D for `prepare-publication`, then continue Lane D for closeout and maintenance continuity.

### Conflict flags

- `crates/xtask/src/main.rs` is a conflict magnet. Only one lane should own it at a time.
- `crates/xtask/src/agent_lifecycle.rs` must freeze before parallel lanes branch.
- `crates/xtask/src/runtime_follow_on.rs`, `models.rs`, and `render.rs` stay in one lane.
- `docs/cli-agent-onboarding-factory-operator-guide.md` needs one docs owner during schema freeze.
- `crates/xtask/tests/runtime_follow_on_entrypoint.rs` and `onboard_agent_entrypoint/**` can evolve in parallel, but integration assertions that span both commands belong in the merge lane after A + B converge.

Practical answer:

- parallelize onboarding, runtime integration, and doc/backfill work after the shared schema is frozen
- keep publication, closeout, and maintenance lifecycle logic sequential

## Completion Summary

- Step 0: Scope Challenge — accepted with one shared module, one new command, and no new crate
- Architecture Review: lifecycle truth unified without inventing a workflow engine
- Code Quality Review: duplicate state vocabularies explicitly forbidden
- Test Review: coverage diagram produced, all transitions mapped to tests
- Performance Review: repo-tooling slow paths identified and bounded
- NOT in scope: written
- What already exists: written
- Failure modes: critical gaps identified and blocked from shipping
- Parallelization: 4 lanes total, 3 parallel after schema freeze, 1 final sequential lane
- Lake Score: 8/8 major recommendations chose the complete option over the shortcut

## Implementation Order

Execute the work in this order:

1. M1 schema core
2. M1 backfill and doc sync
3. M2 onboard-agent integration
4. M2 runtime-follow-on integration
5. Merge and re-run full create-lane tests
6. M3 prepare-publication seam
7. M4 closeout and maintenance continuity

Do not start with publication generators. The lifecycle contract has to exist first or the rest of the work turns into another thin handoff file.
