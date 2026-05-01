# Unified Agent Lifecycle Support Maturity Model Orchestration Plan

## Summary

- Execute on the live implementation branch `codex/recommend-next-agent`. Treat `main` only as the review base branch, never as the worker fork point.
- Plan authority is `PLAN.md`. If `PLAN.md`, current code, backlog notes, and older orchestration notes disagree, `PLAN.md` wins unless it conflicts with `docs/specs/**`; a spec conflict is a stop-and-escalate condition.
- Parent agent is the only integrator, merger, rebasing authority, and final verifier.
- Worker model for delegated lanes:
  - model: GPT-5.4
  - reasoning: high
- Keep the critical path local to the parent for:
  - kickoff and baseline capture
  - M1 lifecycle schema freeze
  - lifecycle-state backfills
  - charter and operator-guide authority updates
  - M3 `prepare-publication`
  - M4 closeout and maintenance continuity
  - final integration and final verification
- Launch parallel workers only after the lifecycle schema freeze commit exists.
- Use only 2 concurrent workers.
  - Worker 1: M2 `onboard-agent` integration
  - Worker 2: M2 `runtime-follow-on` integration
- Do not launch a docs/backfill worker.
  - `docs/cli-agent-onboarding-factory-operator-guide.md` is touched in M1, M2, M3, and M4.
  - The five `lifecycle-state.json` backfills encode the frozen schema directly and should stay parent-owned.
- Integration surface is explicit and intentionally local:
  - the parent integrates directly onto `codex/recommend-next-agent` in the primary repo worktree at `REPO_ROOT`
  - there is no separate integration branch for this run
  - this is an intentional tradeoff to keep schema freeze, publication seam, closeout seam, and final verification on one live execution surface
- No human approval gates are planned for this run.
- Canonical orchestration state owned only by the parent:
  - `REPO_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api`
  - `WORKTREE_ROOT=/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/unified-agent-api-lifecycle-maturity`
  - `ORCH_RUN_ROOT=$REPO_ROOT/.runs/unified-agent-lifecycle-maturity`
  - `RUNTIME_EVIDENCE_ROOT=$REPO_ROOT/docs/agents/.uaa-temp`
- Authored governance outputs in scope:
  - `docs/agents/lifecycle/*/governance/lifecycle-state.json`
  - `docs/agents/lifecycle/*/governance/publication-ready.json`
- Generated evidence in scope but never hand-authored:
  - `docs/agents/.uaa-temp/**`

## Approval Gates

- This plan has zero human approval gates.
- The only intentional pauses are:
  - hard stop conditions
  - worker bounce-backs
  - schema-freeze reopen handling
  - final verification failures that require narrowing back to the touched scope

## Worker Model

- Parent agent owns all integration, orchestration state, freeze decisions, merge sequencing, and acceptance decisions.
- Workers may edit only their assigned files and must return:
  - changed files
  - commands run
  - exit codes
  - blockers or assumptions
- Workers do not merge branches.
- Workers do not edit `$ORCH_RUN_ROOT/**`.
- Workers do not author or hand-edit `.uaa-temp` evidence.
- Any worker that needs a parent-owned file must stop and bounce the request back.

## Hard Guards

- Scope is locked to the current `PLAN.md` milestone.
- Do not add a new crate.
- Do not add a new service.
- Do not add a second committed runtime summary artifact.
- Do not add any artifact family beyond:
  - `lifecycle-state.json`
  - `publication-ready.json`
- `lifecycle-state.json` and `publication-ready.json` are authored governance outputs.
- `.uaa-temp` runtime artifacts are generated evidence only.
- Keep all lifecycle schema code in `crates/xtask/src/agent_lifecycle.rs`.
- Do not duplicate lifecycle enums or transition rules across commands.
- Persisted resting stages in v1 are exactly:
  - `enrolled`
  - `runtime_integrated`
  - `publication_ready`
  - `closed_baseline`
- `published` remains schema-valid but is not a resting stage in v1.
- `close-proving-run --write` may accept `publication_ready` or legacy/manual `published`, but on success it writes `closed_baseline`.
- `opencode-maintenance` remains maintenance-only and is not the create-mode lifecycle-state location.
- The legal create-mode lifecycle-state backfill paths are exactly:
  - `docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json`
  - `docs/agents/lifecycle/claude-code-cli-onboarding/governance/lifecycle-state.json`
  - `docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json`
  - `docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json`
  - `docs/agents/lifecycle/aider-onboarding/governance/lifecycle-state.json`
- The required publication commands are exactly:
  - `cargo run -p xtask -- support-matrix --check`
  - `cargo run -p xtask -- capability-matrix --check`
  - `cargo run -p xtask -- capability-matrix-audit`
  - `make preflight`
- Critical gaps that must not ship:
  - lifecycle update failure with command success
  - publication-ready packet emitted without runtime evidence
  - closeout succeeding without green published surfaces
  - maintenance drift check ignoring lifecycle baseline
- Stop conditions for the full run:
  - `PLAN.md` conflicts with `docs/specs/**`
  - M1 backfill requires inventing ungrounded historical state
  - M3 requires dynamic backend loading or publication-surface rewrites beyond continuity checks
  - M4 requires weakening the green-publication gate to pass tests
  - any lane needs a new artifact family or new service to complete

## Orchestration State

Canonical parent-owned files under `$ORCH_RUN_ROOT`:

- `baseline.json`
- `tasks.json`
- `session-log.md`
- `schema-freeze.json`
- `merge-log.md`
- `acceptance.md`

Per-task sentinels under `$REPO_ROOT/.runs/<task-id>/`:

- `started.json`
- `status.json`
- `done.json` or `blocked.json`

`baseline.json` must record:

- current branch
- current sha
- review base branch
- current dirty-file summary
- existing create-mode governance inputs that actually exist today
- registry-derived expected create-mode lifecycle pack targets, even if those directories or files do not exist yet
- files expected to be created later in the run

`schema-freeze.json` must record:

- `freeze_commit`
- `lifecycle_stage_values`
- `support_tier_values`
- `side_state_values`
- `required_evidence_values`
- `implementation_summary_surface_values`
- `resting_stage_values`
- `published_resting_rule`
- `required_publication_commands`
- `backfill_paths`
- `opencode_create_mode_path`

Workers consume `schema-freeze.json` and do not redefine any of its vocabulary locally.

## Worktree Plan

Integration surface:

- branch: `codex/recommend-next-agent`
- worktree: `REPO_ROOT`
- owner: parent only

Worker worktrees created only after schema freeze:

- onboarding worker:
  - branch: `codex/recommend-next-agent-onboard`
  - worktree: `$WORKTREE_ROOT/onboard`
- runtime worker:
  - branch: `codex/recommend-next-agent-runtime`
  - worktree: `$WORKTREE_ROOT/runtime`

Creation commands from `REPO_ROOT` after `schema-freeze.json` exists:

```bash
mkdir -p "$WORKTREE_ROOT" "$ORCH_RUN_ROOT"
FREEZE_SHA=$(jq -r '.freeze_commit' "$ORCH_RUN_ROOT/schema-freeze.json")

git worktree add -b codex/recommend-next-agent-onboard "$WORKTREE_ROOT/onboard" "$FREEZE_SHA"
git worktree add -b codex/recommend-next-agent-runtime "$WORKTREE_ROOT/runtime" "$FREEZE_SHA"
```

Worktree rules:

- Never fork workers from `main`.
- Never fork workers from anything other than `freeze_commit`.
- Never reuse a dirty worker worktree.
- Parent integrates only in `REPO_ROOT` on `codex/recommend-next-agent`.
- There is no dedicated integration branch for this run.

## Restart And Reopen Rule

- If the parent changes any schema-freeze semantic surface after either worker launches, both workers are stale.
- Schema-freeze semantic surfaces are:
  - `crates/xtask/src/agent_lifecycle.rs`
  - `crates/xtask/src/lib.rs`
  - any frozen enum vocabulary
  - any frozen evidence vocabulary
  - any frozen command-string contract
  - any frozen backfill path or initial-state rule
- If the parent changes only onboarding-owned files after the onboarding worker launches, the onboarding worker is stale.
- If the parent changes only runtime-owned files after the runtime worker launches, the runtime worker is stale.
- Stale worker handling is mandatory:
  1. Mark the worker sentinel as stale or blocked.
  2. Discard the worker branch and worktree.
  3. Regenerate `schema-freeze.json` if the freeze changed.
  4. Recreate the worker worktree from the new `freeze_commit`.
  5. Relaunch the worker with the new freeze context.
- Never merge stale worker output.
- Never cherry-pick stale worker commits into `codex/recommend-next-agent`.

## Merge Policy

- Parent merges worker branches only into the explicit integration surface:
  - `REPO_ROOT` on branch `codex/recommend-next-agent`
- Parent merges from branch heads only.
- Parent does not ask workers to rebase themselves.
- Parent merges the onboarding worker first.
- Parent reruns the onboarding gate after that merge.
- Parent merges the runtime worker second.
- Parent reruns the combined M2 gate after both merges.
- If a worker changed a forbidden file, reject the lane and relaunch or finish it locally.
- Mechanical conflict resolution is allowed locally.
- Semantic conflict resolution that changes frozen lifecycle meaning is a stop-and-reopen event.

## Task Graph

Critical path:

1. `task/lifecycle-00-baseline`
2. `task/lifecycle-01-schema-freeze`
3. launch in parallel:
   - `task/lifecycle-02a-onboard`
   - `task/lifecycle-02b-runtime`
4. `task/lifecycle-02c-m2-converge`
5. `task/lifecycle-03-prepare-publication`
6. `task/lifecycle-04-closeout-maintenance`
7. `task/lifecycle-05-final-verify`

Parallel-safe tasks:

- `task/lifecycle-02a-onboard`
- `task/lifecycle-02b-runtime`

Deliberately serialized tasks:

- `task/lifecycle-00-baseline`
- `task/lifecycle-01-schema-freeze`
- `task/lifecycle-02c-m2-converge`
- `task/lifecycle-03-prepare-publication`
- `task/lifecycle-04-closeout-maintenance`
- `task/lifecycle-05-final-verify`

## Workstream Plan

### WS-BASELINE

#### `task/lifecycle-00-baseline` — parent only

Existing required files that must already exist:

- `PLAN.md`
- `crates/xtask/data/agent_registry.toml`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/main.rs`
- `crates/xtask/src/onboard_agent.rs`
- `crates/xtask/src/runtime_follow_on.rs`
- `crates/xtask/src/runtime_follow_on/models.rs`
- `crates/xtask/src/runtime_follow_on/render.rs`
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/support_matrix.rs`
- `crates/xtask/src/close_proving_run.rs`
- `crates/xtask/src/agent_maintenance/closeout.rs`
- `crates/xtask/src/agent_maintenance/drift/publication.rs`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml`
- `docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml`
- `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`

Expected files that do not need to exist yet:

- `crates/xtask/src/agent_lifecycle.rs`
- `crates/xtask/tests/agent_lifecycle_state.rs`
- `docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json`
- `docs/agents/lifecycle/claude-code-cli-onboarding/governance/lifecycle-state.json`
- `docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json`
- `docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json`
- `docs/agents/lifecycle/aider-onboarding/governance/lifecycle-state.json`
- `crates/xtask/src/prepare_publication.rs`
- `crates/xtask/tests/prepare_publication_entrypoint.rs`
- any committed `publication-ready.json` outputs

Owned files:

- `$ORCH_RUN_ROOT/baseline.json`
- `$ORCH_RUN_ROOT/tasks.json`
- `$ORCH_RUN_ROOT/session-log.md`
- `$REPO_ROOT/.runs/task-lifecycle-00-baseline/*`

Forbidden files:

- all product source files

Required commands:

```bash
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git status --short
test -f PLAN.md
test -f crates/xtask/data/agent_registry.toml
test -f crates/xtask/src/lib.rs
test -f crates/xtask/src/main.rs
test -f crates/xtask/src/onboard_agent.rs
test -f crates/xtask/src/runtime_follow_on.rs
test -f crates/xtask/src/runtime_follow_on/models.rs
test -f crates/xtask/src/runtime_follow_on/render.rs
test -f crates/xtask/src/capability_matrix.rs
test -f crates/xtask/src/support_matrix.rs
test -f crates/xtask/src/close_proving_run.rs
test -f crates/xtask/src/agent_maintenance/closeout.rs
test -f crates/xtask/src/agent_maintenance/drift/publication.rs
test -f docs/specs/cli-agent-onboarding-charter.md
test -f docs/cli-agent-onboarding-factory-operator-guide.md
test -f docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml
test -f docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml
test -f docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml
rg -n 'codex|claude_code|opencode|gemini_cli|aider|onboarding_pack_prefix|support_matrix_enabled|capability_matrix_enabled' crates/xtask/data/agent_registry.toml
```

Acceptance:

- current branch is `codex/recommend-next-agent`
- `PLAN.md` is present and is the run authority
- the baseline validates only real current surfaces
- nonexistent create-mode `approved-agent.toml` paths are not treated as baseline failures
- `baseline.json` records registry-derived expected create-mode lifecycle pack targets for later backfill
- any unrelated worktree dirt is recorded, not reverted

### WS-FREEZE

#### `task/lifecycle-01-schema-freeze` — parent only

Existing required files that must already exist before this task starts:

- `crates/xtask/data/agent_registry.toml`
- `crates/xtask/src/lib.rs`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

Files expected to be created during this task:

- `crates/xtask/src/agent_lifecycle.rs`
- `crates/xtask/tests/agent_lifecycle_state.rs`
- `docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json`
- `docs/agents/lifecycle/claude-code-cli-onboarding/governance/lifecycle-state.json`
- `docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json`
- `docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json`
- `docs/agents/lifecycle/aider-onboarding/governance/lifecycle-state.json`

Owned files:

- `crates/xtask/src/agent_lifecycle.rs`
- `crates/xtask/tests/agent_lifecycle_state.rs`
- `crates/xtask/src/lib.rs`
- `docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json`
- `docs/agents/lifecycle/claude-code-cli-onboarding/governance/lifecycle-state.json`
- `docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json`
- `docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json`
- `docs/agents/lifecycle/aider-onboarding/governance/lifecycle-state.json`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `$ORCH_RUN_ROOT/schema-freeze.json`
- `$ORCH_RUN_ROOT/session-log.md`
- `$REPO_ROOT/.runs/task-lifecycle-01-schema-freeze/*`

Forbidden files:

- `crates/xtask/src/onboard_agent.rs`
- `crates/xtask/src/onboard_agent/**`
- `crates/xtask/src/runtime_follow_on.rs`
- `crates/xtask/src/runtime_follow_on/models.rs`
- `crates/xtask/src/runtime_follow_on/render.rs`
- `crates/xtask/templates/runtime_follow_on_codex_prompt.md`
- `crates/xtask/src/main.rs`
- `crates/xtask/src/prepare_publication.rs`
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/support_matrix.rs`
- `crates/xtask/src/close_proving_run.rs`
- `crates/xtask/src/agent_maintenance/**`
- `docs/agents/lifecycle/opencode-maintenance/**`

Required work:

1. Add the shared lifecycle module in `crates/xtask/src/agent_lifecycle.rs`.
2. Freeze lifecycle-state schema, publication-ready packet schema, validation helpers, and transition helpers.
3. Encode the legal stage and support-tier matrix exactly once.
4. Encode the v1 resting-stage rule exactly once, including `published` compatibility without treating it as a resting stage.
5. Backfill the five legal lifecycle-state files using the exact PLAN table.
6. Keep `opencode-maintenance` untouched as maintenance-only.
7. Update `crates/xtask/src/lib.rs` to expose the shared lifecycle module.
8. Update the charter and operator guide so lifecycle-state becomes the authoritative create-mode and maintenance baseline.
9. Write `schema-freeze.json` from the exact head commit that contains the finished M1 tree.

Required commands:

```bash
cargo test -p xtask --test agent_lifecycle_state
cargo test -p xtask --test agent_registry
make check
```

Acceptance:

- `agent_lifecycle.rs` is the only lifecycle schema owner
- the five lifecycle-state backfills exist at the exact legal paths
- initial backfill states match `PLAN.md` unless committed evidence proves otherwise and the same PR updates the plan basis
- `published` is schema-valid but not persisted as a v1 resting stage
- `schema-freeze.json` exists and captures the frozen vocabulary and command set
- worker lanes do not launch until this task is complete on `codex/recommend-next-agent`

### WS-M2-ONBOARD

#### `task/lifecycle-02a-onboard` — worker 1

Owned files:

- `crates/xtask/src/onboard_agent.rs`
- `crates/xtask/src/onboard_agent/**` when mechanically required by the existing module split
- `crates/xtask/tests/onboard_agent_entrypoint/**`

Forbidden files:

- `crates/xtask/src/agent_lifecycle.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/main.rs`
- `crates/xtask/src/runtime_follow_on.rs`
- `crates/xtask/src/runtime_follow_on/**`
- `crates/xtask/templates/runtime_follow_on_codex_prompt.md`
- `crates/xtask/src/prepare_publication.rs`
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/support_matrix.rs`
- `crates/xtask/src/close_proving_run.rs`
- `crates/xtask/src/agent_maintenance/**`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `$ORCH_RUN_ROOT/**`

Required work:

1. Make `onboard-agent --dry-run` preview the exact lifecycle-state seed.
2. Ensure the preview includes:
   - `current_owner_command = "onboard-agent --write"`
   - `expected_next_command = "scaffold-wrapper-crate --agent <agent_id> --write"`
3. Make `onboard-agent --write` create `lifecycle-state.json`.
4. Seed:
   - `lifecycle_stage = enrolled`
   - `support_tier = bootstrap`
   - `required_evidence = ["registry_entry", "docs_pack", "manifest_root_skeleton"]`
   - `satisfied_evidence = ["registry_entry", "docs_pack", "manifest_root_skeleton"]`
5. Reject divergent pre-existing lifecycle-state files instead of silently rewriting them.
6. Use `agent_lifecycle` helpers for lifecycle writes instead of hand-rolled JSON mutations.
7. Do not implement runtime or publication transitions here.

Required commands:

```bash
cargo test -p xtask --test onboard_agent_entrypoint
```

Acceptance:

- dry-run previews the lifecycle-state file contents
- write mode seeds enrolled/bootstrap exactly
- exact evidence ids are populated
- duplicate or divergent lifecycle seeds are rejected
- the worker may update any `crates/xtask/tests/onboard_agent_entrypoint/**` surfaces mechanically required by the preview and write-mode changes
- no runtime-follow-on, prepare-publication, closeout, maintenance, or docs files are changed

Bounce-back rules:

- if the worker needs a schema change, stop and return the required `agent_lifecycle.rs` change
- if the worker needs operator-guide wording changes, stop and return the exact procedure delta

### WS-M2-RUNTIME

#### `task/lifecycle-02b-runtime` — worker 2

Owned files:

- `crates/xtask/src/runtime_follow_on.rs`
- `crates/xtask/src/runtime_follow_on/models.rs`
- `crates/xtask/src/runtime_follow_on/render.rs`
- `crates/xtask/templates/runtime_follow_on_codex_prompt.md`
- `crates/xtask/tests/runtime_follow_on_entrypoint.rs`

Forbidden files:

- `crates/xtask/src/agent_lifecycle.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/main.rs`
- `crates/xtask/src/onboard_agent.rs`
- `crates/xtask/src/onboard_agent/**`
- `crates/xtask/src/prepare_publication.rs`
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/support_matrix.rs`
- `crates/xtask/src/close_proving_run.rs`
- `crates/xtask/src/agent_maintenance/**`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `$ORCH_RUN_ROOT/**`

Required work:

1. Require an enrolled lifecycle-state for `runtime-follow-on --dry-run` and `--write`.
2. Extend `InputContract` with approval capability and publication truth:
   - `canonical_targets`
   - `always_on_capabilities`
   - `target_gated_capabilities`
   - `config_gated_capabilities`
   - `backend_extensions`
   - `support_matrix_enabled`
   - `capability_matrix_enabled`
   - `capability_matrix_target`
3. Render those fields into `crates/xtask/templates/runtime_follow_on_codex_prompt.md`.
4. On successful write:
   - advance lifecycle state to `runtime_integrated`
   - set support tier to `baseline_runtime`
   - persist `implementation_summary`
   - satisfy `runtime_write_complete`
   - satisfy `implementation_summary_present`
   - set `expected_next_command = "prepare-publication --approval <path> --write"`
5. On validation failure:
   - add `failed_retryable` or `blocked`
   - append exact blocker text
   - keep lifecycle stage unchanged if runtime ownership did not complete
6. Never write `publication-ready.json`.
7. Treat `handoff.json`, `run-status.json`, and `run-summary.md` as evidence only.

Required commands:

```bash
cargo test -p xtask --test runtime_follow_on_entrypoint
```

Acceptance:

- dry-run refuses missing or non-enrolled lifecycle state
- prompt and input contract carry the publication/capability truth from the registry and approval artifact
- success advances lifecycle to `runtime_integrated`
- failure never reports command success if the lifecycle update failed
- runtime lane does not emit `publication-ready.json`
- the worker may update runtime-follow-on entrypoint assertions only in `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
- no onboard-agent, prepare-publication, closeout, maintenance, or docs files are changed

Bounce-back rules:

- if the worker needs a new field, enum, or evidence id, stop and return the requested freeze change
- if the worker needs to touch `main.rs`, `capability_matrix.rs`, or `support_matrix.rs`, stop and bounce back

### WS-M2-CONVERGE

#### `task/lifecycle-02c-m2-converge` — parent only

Owned files:

- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `$ORCH_RUN_ROOT/merge-log.md`
- `$ORCH_RUN_ROOT/session-log.md`
- `$REPO_ROOT/.runs/task-lifecycle-02c-m2-converge/*`

Required integration sequence:

1. Merge `codex/recommend-next-agent-onboard` into the explicit integration surface:
   - `REPO_ROOT` on `codex/recommend-next-agent`
2. Run the onboarding gate.
3. Merge `codex/recommend-next-agent-runtime` into the same integration surface.
4. Run the combined M2 gate.
5. Update the operator guide locally for the M2 create-lane flow once both code paths are real.
6. Do not reopen lifecycle schema semantics here.

Required commands:

```bash
cargo test -p xtask --test onboard_agent_entrypoint
cargo test -p xtask --test runtime_follow_on_entrypoint
```

Acceptance:

- M2 code is merged on the live branch integration surface
- operator guide reflects seeded lifecycle-state and runtime evidence semantics
- no second handoff artifact is introduced
- runtime evidence remains `.uaa-temp` only

### WS-M3

#### `task/lifecycle-03-prepare-publication` — parent only

Owned files:

- `crates/xtask/src/prepare_publication.rs`
- `crates/xtask/tests/prepare_publication_entrypoint.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/main.rs`
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/support_matrix.rs`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `$ORCH_RUN_ROOT/session-log.md`
- `$REPO_ROOT/.runs/task-lifecycle-03-prepare-publication/*`

Forbidden files:

- any new crate path
- any new service path
- any new artifact-family path

Required work:

1. Add `prepare-publication` command wiring in `crates/xtask/src/main.rs`.
2. Export the command in `crates/xtask/src/lib.rs` as needed.
3. Implement `crates/xtask/src/prepare_publication.rs`.
4. `--write` must:
   - require `runtime_integrated`
   - validate approval path and SHA continuity
   - validate runtime evidence exists
   - validate `implementation_summary` is explicit and non-empty
   - validate the exact four required publication commands
   - write `publication-ready.json`
   - advance lifecycle state to `publication_ready`
   - satisfy `publication_packet_written`
   - set `expected_next_command = "support-matrix --check && capability-matrix --check && capability-matrix-audit && make preflight && close-proving-run --write"`
5. `--check` must revalidate without rewriting.
6. Add capability inventory continuity checks through `capability_matrix.rs`.
7. Do not implement dynamic backend loading.
8. Do not let `prepare-publication` write support-matrix or capability-matrix outputs.
9. Update charter and operator guide to make publication-ready the only committed publication handoff.

Required commands:

```bash
cargo test -p xtask --test prepare_publication_entrypoint
cargo test -p xtask --test runtime_follow_on_entrypoint
make check
```

Acceptance:

- `prepare-publication` is the only runtime-to-publication seam
- `publication-ready.json` is the only committed publication handoff packet
- required publication commands match the frozen exact strings
- capability continuity failures block packet creation explicitly
- no publication outputs are generated by this command

### WS-M4

#### `task/lifecycle-04-closeout-maintenance` — parent only

Owned files:

- `crates/xtask/src/close_proving_run.rs`
- `crates/xtask/src/agent_maintenance/closeout.rs`
- `crates/xtask/src/agent_maintenance/closeout/**` when mechanically required
- `crates/xtask/src/agent_maintenance/drift/publication.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_paths.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`
- `crates/xtask/tests/agent_maintenance_closeout.rs`
- `crates/xtask/tests/agent_maintenance_drift.rs`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `$ORCH_RUN_ROOT/session-log.md`
- `$REPO_ROOT/.runs/task-lifecycle-04-closeout-maintenance/*`

Required work:

1. Require `publication_ready` or legacy/manual `published` for `close-proving-run --write`.
2. Validate green publication surfaces before closeout succeeds.
3. Validate `publication-ready.json` continuity against lifecycle and approval state.
4. On success, write:
   - `lifecycle_stage = closed_baseline`
   - `support_tier = publication_backed` for new create-lane agents
   - `publication_packet_path`
   - `publication_packet_sha256`
   - `closeout_baseline_path`
5. Satisfy:
   - `support_matrix_check_green`
   - `capability_matrix_check_green`
   - `capability_matrix_audit_green`
   - `preflight_green`
   - `proving_run_closeout_written`
6. Clear `blocked`, `failed_retryable`, and `drifted`.
7. Preserve `deprecated` if present.
8. Keep `first_class` auto-promotion out of scope.
9. Make `check-agent-drift` compare published truth against the lifecycle baseline.
10. Make `close-agent-maintenance` clear `drifted`, update evidence, and avoid rewriting approval truth.

Required commands:

```bash
cargo test -p xtask --test onboard_agent_closeout_preview
cargo test -p xtask --test agent_maintenance_closeout
cargo test -p xtask --test agent_maintenance_drift
make test
```

Acceptance:

- closeout cannot succeed without green published surfaces
- `closed_baseline` becomes the maintenance baseline
- drift checks read lifecycle baseline instead of inferring from scattered artifacts
- maintenance closeout clears drift without mutating approval truth
- `opencode-maintenance` remains maintenance-only

### WS-VERIFY

#### `task/lifecycle-05-final-verify` — parent only

Owned files:

- the full milestone touch set
- `$ORCH_RUN_ROOT/acceptance.md`
- `$ORCH_RUN_ROOT/merge-log.md`
- `$ORCH_RUN_ROOT/session-log.md`
- `$REPO_ROOT/.runs/task-lifecycle-05-final-verify/*`

Final operator verification sequence:

1. Replay the narrow M1 schema gate:
   ```bash
   cargo test -p xtask --test agent_lifecycle_state
   ```
2. Replay the narrow M2 onboarding gate:
   ```bash
   cargo test -p xtask --test onboard_agent_entrypoint
   ```
3. Replay the narrow M2 runtime gate:
   ```bash
   cargo test -p xtask --test runtime_follow_on_entrypoint
   ```
4. Replay the narrow M3 publication seam gate:
   ```bash
   cargo test -p xtask --test prepare_publication_entrypoint
   ```
5. Replay the narrow M4 closeout and maintenance gates:
   ```bash
   cargo test -p xtask --test onboard_agent_closeout_preview
   cargo test -p xtask --test agent_maintenance_closeout
   cargo test -p xtask --test agent_maintenance_drift
   ```
6. Run the repo-format and lint gates after the narrower xtask tests are green:
   ```bash
   make fmt-check
   make clippy
   ```
7. Run the final whole-repo gate last:
   ```bash
   make preflight
   ```

Required artifact inspection:

```bash
test -f docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json
test -f docs/agents/lifecycle/claude-code-cli-onboarding/governance/lifecycle-state.json
test -f docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json
test -f docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json
test -f docs/agents/lifecycle/aider-onboarding/governance/lifecycle-state.json
rg -n "publication-ready.json|lifecycle-state.json|closed_baseline|runtime_integrated|publication_ready" docs/specs/cli-agent-onboarding-charter.md docs/cli-agent-onboarding-factory-operator-guide.md
```

Verification rules:

- Run the verification sequence in the listed order.
- Treat `make preflight` as the final whole-repo closeout gate only after all narrower xtask tests, `make fmt-check`, and `make clippy` have passed.
- If a failure is caused by this milestone’s files, patch inside the milestone touch set and rerun from the first failed step.
- If a failure is caused by unrelated pre-existing repo state outside the touch set, record it in `acceptance.md` and stop instead of widening scope.
- Do not author `.uaa-temp` evidence during final verification except through tests or existing command behavior.
- Do not mutate the five committed backfill lifecycle files for ad hoc manual proof runs.

Acceptance:

- the live branch `codex/recommend-next-agent` contains the full milestone
- the explicit integration surface remained `REPO_ROOT` on `codex/recommend-next-agent`
- the lifecycle schema is centralized in `crates/xtask/src/agent_lifecycle.rs`
- onboarding seeds lifecycle state
- runtime-follow-on advances lifecycle and records evidence without claiming success on lifecycle-write failure
- prepare-publication writes the only committed publication handoff
- close-proving-run seals `closed_baseline` only after green published surfaces
- maintenance drift reads lifecycle baseline
- no unauthorized artifact family exists
- `$ORCH_RUN_ROOT` contains:
  - `baseline.json`
  - `tasks.json`
  - `session-log.md`
  - `schema-freeze.json`
  - `merge-log.md`
  - `acceptance.md`

## Context-Control Rules

- Parent keeps only these materials active in working context:
  - `PLAN.md`
  - this `ORCH_PLAN.md`
  - `$ORCH_RUN_ROOT/tasks.json`
  - `$ORCH_RUN_ROOT/schema-freeze.json` after M1
  - the latest live-branch diff summary
- Worker prompts contain only:
  - owned files
  - forbidden files
  - the exact relevant `PLAN.md` excerpt
  - the frozen vocabulary from `schema-freeze.json`
  - required commands
  - bounce-back rules
- Workers return narrow summaries only.
- Parent records decisions in `$ORCH_RUN_ROOT/session-log.md`.
- Close worker lanes immediately after merge or rejection.
- Do not ingest full worker transcripts back into parent context unless a specific failure excerpt is required.

## Tests And Acceptance

Parent gates by phase:

- M1:
  - `cargo test -p xtask --test agent_lifecycle_state`
  - `cargo test -p xtask --test agent_registry`
  - `make check`
- M2 worker A:
  - `cargo test -p xtask --test onboard_agent_entrypoint`
- M2 worker B:
  - `cargo test -p xtask --test runtime_follow_on_entrypoint`
- M2 converge:
  - `cargo test -p xtask --test onboard_agent_entrypoint`
  - `cargo test -p xtask --test runtime_follow_on_entrypoint`
- M3:
  - `cargo test -p xtask --test prepare_publication_entrypoint`
  - `cargo test -p xtask --test runtime_follow_on_entrypoint`
  - `make check`
- M4:
  - `cargo test -p xtask --test onboard_agent_closeout_preview`
  - `cargo test -p xtask --test agent_maintenance_closeout`
  - `cargo test -p xtask --test agent_maintenance_drift`
  - `make test`
- Final:
  - `make fmt-check`
  - `make clippy`
  - `make preflight`

Final acceptance checklist:

- M1 shipped the shared lifecycle schema, tests, and the five legal backfills
- M2 shipped seeded onboarding lifecycle state and runtime lifecycle advancement
- M3 shipped `prepare-publication` with exact required publication commands
- M4 shipped closeout and maintenance continuity from the lifecycle baseline
- `published` is supported for compatibility but is not a v1 resting stage
- `opencode-maintenance` was not used as the create-mode lifecycle-state location
- the four critical gaps are explicitly blocked by code and tests
- `make preflight` ran last as the final whole-repo gate after the narrower phase gates

## Assumptions

- `codex/recommend-next-agent` is the correct live implementation branch for this run.
- `main` remains only the review base branch.
- The current repo structure is stable enough that M1 can freeze the lifecycle contract before any worker launches.
- The baseline can rely on existing real surfaces plus registry-derived discovery for create-mode pack targets that do not yet exist today.
- The five lifecycle backfills can be grounded from committed repo truth without inventing new artifact families.
- Existing test harnesses are sufficient; this milestone does not require a new harness family.
- The operator guide can remain parent-owned for the full run without slowing the critical path more than workerizing it would save.
- Keeping M3 and M4 local is faster and safer than splitting `main.rs`, publication continuity, closeout, and maintenance semantics across additional workers.
