# ORCH_PLAN - Enclose The Agent-Maintenance Execution Relay

## Summary

This orchestration plan owns the current milestone in
`/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api` on branch
`codex/recommend-next-agent`.

Authoritative milestone source: `PLAN.md`
Milestone: `Enclose The Agent-Maintenance Execution Relay`

This is a follow-on to already-landed shared watcher plus packet-first PR work. The job here is
to enclose the local maintainer execution seam with:

- structured `[execution_contract]` request truth
- one shared execution-packet renderer
- `execute-agent-maintenance --dry-run|--write`
- one prepared `run_id` baseline
- temp relay evidence under `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`
- bounded write enforcement plus diff validation
- workflow recovery hardening
- docs and playbook alignment
- final proving

This milestone does not redesign the watcher, does not widen to packet-only relay execution, does
not widen beyond local Codex execution, and does not automate closeout.

Completion definition:

- lanes A through G below are merged back onto `codex/recommend-next-agent`
- the serial spine from schema -> shared renderer is preserved before downstream lane launch
- `execute-agent-maintenance --dry-run` and `--write` behave exactly as specified in `PLAN.md`
- manual closeout remains outside relay write mode
- all commands in `PLAN.md` section `Commands That Must Pass Before Landing` pass on the parent branch

## Parent Critical Path

Frozen parent spine:

```text
P0 baseline capture + stale-scope rejection
-> P1 schema lane launch and merge (Lane A)
-> P2 shared renderer lane plus closeout compatibility launch (Lanes B, E)
-> P3 packet generation and relay launch after renderer merge (Lanes C, D)
-> P4 workflow recovery hardening after packet generation is merged (Lane F)
-> P5 docs and playbook closeout after relay + workflow semantics are stable (Lane G)
-> P6 parent-only proving and acceptance
```

Parent-only completion gate:

1. Lane A is merged before Lane B starts.
2. Lane A is merged before Lane E starts.
3. Lane B is merged before Lanes C and D start.
4. Lane C is merged before Lane F starts.
5. Lane G waits until C, D, and F are stable enough that docs will not churn.
6. Final proving runs only on the parent branch after all lane merges are complete.

## Hard Guards

- `PLAN.md` is the only milestone authority. The previous `ORCH_PLAN.md` is stale and is valid
  only as a rejection source for outdated goals.
- Shared watcher topology and packet-first PR flow are already landed. Do not reopen watcher
  architecture, queue math, registry enrollment, or worker migration as new milestone goals.
- No watcher redesign.
- No earlier registry/watcher revamp scope.
- No packet-only relay executor in milestone 1. Packet-only agents remain on the existing packet PR path.
- No executor widening beyond local Codex.
- No GitHub-hosted or cloud-hosted autonomous relay execution.
- No automatic closeout. `close-agent-maintenance` remains manual and outside relay write mode.
- No promotion-pointer or publication-surface writes inside relay write mode.
- Temp relay evidence is host-owned and stays under
  `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`.
- Relay write-boundary enforcement must reuse the repo's existing path-jail machinery. Do not
  invent a second boundary system.
- `HANDOFF.md` remains the human entrypoint, but machine truth comes from structured request truth
  and the frozen dry-run packet, never from parsing rendered markdown.
- Any proposal that reintroduces stale ORCH scope such as watcher replacement,
  worker-entrypoint migration, or goose follow-on work halts the lane immediately.

## Authority Model

Parent-only authority:

- owns `PLAN.md` interpretation for this milestone
- owns this orchestration document
- owns launch order, relaunch decisions, and dependency freezes
- owns merge decisions onto `codex/recommend-next-agent`
- owns integration conflict resolution
- owns final proving, acceptance, and landing recommendation
- owns orchestration state under the `.runs/...` root below

Worker authority:

- may edit only the files assigned to the worker lane
- may run only the lane-scoped tests and validation called out for that lane
- may not merge, rebase other lanes, or update orchestration state
- may not widen milestone scope or alter frozen boundaries
- must report exact commands run, exact files changed, blockers, and unresolved assumptions

Merge policy:

- all worker branches fork from `codex/recommend-next-agent`
- workers should keep output to one reviewable commit when practical
- parent integrates worker output onto `codex/recommend-next-agent`
- preferred integration is `git cherry-pick -x <worker-commit>`; if drift makes that unsafe,
  parent manually reapplies the worker diff on the parent branch
- workers do not merge each other
- if a dependency freeze changes after a lane launches, parent stops the affected lane and relaunches it from the new freeze SHA

Concurrency cap:

- maximum concurrent workers: `3`
- recommended live cap for this milestone: `3`
- safe overlap is:
  - Lane A alone
  - Lanes B and E together
  - Lanes C, D, and E together after Lane B merges, if E is still open
  - Lane F alone or alongside only parent integration work
  - Lane G alone
- parent does not exceed the cap even if more lanes are technically unblocked; preserving the schema -> renderer -> packet/relay spine is more important than maximizing parallelism

## Orchestration State

Parent-owned orchestration root:

```text
/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.runs/enclose-agent-maintenance-execution-relay
```

Concrete state files:

- `baseline.json`
- `freeze.json`
- `lane-status.json`
- `worker-launches.json`
- `merge-log.md`
- `session-log.md`
- `acceptance.md`
- `final-proving.md`

Per-lane state records:

- `lane-a-schema.json`
- `lane-b-renderer.json`
- `lane-c-prepare.json`
- `lane-d-execute.json`
- `lane-e-closeout.json`
- `lane-f-workflows.json`
- `lane-g-docs.json`

Required contents:

- `baseline.json`
  - parent branch
  - parent HEAD SHA
  - dirty-worktree summary
  - `PLAN.md` hash
  - stale-ORCH rejection notes
- `freeze.json`
  - locked milestone boundaries
  - dependency order
  - lane ownership table
  - exact final proving commands
  - relaunch triggers
- `lane-status.json`
  - each lane status: `pending|running|blocked|merged|relaunch-required`
  - launch SHA
  - dependency SHA
  - blocking issue, if any
- `worker-launches.json`
  - lane id
  - branch
  - worktree path
  - launch timestamp
  - worker handoff packet version
- `merge-log.md`
  - merge order
  - conflicts encountered
  - post-merge smoke results
- `session-log.md`
  - parent decisions
  - halt events
  - relaunch reasons
- `acceptance.md`
  - milestone acceptance checklist mapped to `PLAN.md`
- `final-proving.md`
  - exact command results
  - final manual inspections
  - residual risk notes

Workers never write any file under the orchestration root.

## Branch And Worktree Layout

Repository root:

```text
/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api
```

Parent branch:

```text
codex/recommend-next-agent
```

Worker worktree root:

```text
/Users/spensermcconnell/__Active_Code/atomize-hq/wt
```

Frozen lane branches and worktrees:

| Lane | Purpose | Branch | Worktree |
| --- | --- | --- | --- |
| A | execution-contract schema | `codex/recommend-next-agent-relay-a-schema` | `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-relay-a-schema` |
| B | shared execution-packet renderer | `codex/recommend-next-agent-relay-b-renderer` | `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-relay-b-renderer` |
| C | packet generation | `codex/recommend-next-agent-relay-c-prepare` | `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-relay-c-prepare` |
| D | relay command | `codex/recommend-next-agent-relay-d-execute` | `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-relay-d-execute` |
| E | closeout compatibility | `codex/recommend-next-agent-relay-e-closeout` | `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-relay-e-closeout` |
| F | workflow recovery hardening | `codex/recommend-next-agent-relay-f-workflows` | `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-relay-f-workflows` |
| G | docs and playbooks | `codex/recommend-next-agent-relay-g-docs` | `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-relay-g-docs` |

Recommended creation pattern:

```sh
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-relay-a-schema -b codex/recommend-next-agent-relay-a-schema codex/recommend-next-agent
```

Repeat with the lane-specific path and branch for each worker.

## Ownership Split

Parent-only surfaces:

- `PLAN.md`
- `ORCH_PLAN.md`
- `.runs/enclose-agent-maintenance-execution-relay/**`
- final parent integration on `codex/recommend-next-agent`

Lane A: Execution-contract schema

- Owns:
  - `crates/xtask/src/agent_maintenance/request.rs`
  - `crates/xtask/tests/agent_maintenance_refresh.rs`
- Goal:
  - add `[execution_contract]` parsing and validation
  - preserve compatibility for manual requests without `[execution_contract]`
- Forbidden:
  - `docs.rs`
  - `prepare.rs`
  - `execute.rs`
  - workflows
  - docs and playbooks

Lane B: Shared execution-packet renderer

- Owns:
  - `crates/xtask/src/agent_maintenance/docs.rs`
  - renderer-specific assertions in `crates/xtask/tests/agent_maintenance_prepare.rs`
- Goal:
  - one shared renderer for `HANDOFF.md`, `governance/pr-summary.md`, and frozen relay prompt artifacts
  - prompt digest and linkage fail closed
- Forbidden:
  - `request.rs`
  - `prepare.rs`
  - `execute.rs`
  - workflows

Lane C: Packet generation

- Owns:
  - `crates/xtask/src/agent_maintenance/prepare.rs`
  - packet-generation assertions in `crates/xtask/tests/agent_maintenance_prepare.rs`
- Goal:
  - `prepare-agent-maintenance --write` becomes the sole writer of request truth plus packet docs
  - automated requests emit deterministic `execution_contract` and recovery data
- Forbidden:
  - `request.rs`
  - `docs.rs`
  - `execute.rs`
  - workflows

Lane D: Relay command

- Owns:
  - `crates/xtask/src/agent_maintenance/execute.rs`
  - `crates/xtask/src/agent_maintenance/mod.rs`
  - `crates/xtask/src/main.rs`
  - `crates/xtask/tests/agent_maintenance_execute.rs`
  - `crates/xtask/tests/support/agent_maintenance_harness.rs`
- Goal:
  - add `execute-agent-maintenance --dry-run|--write`
  - persist prepared run artifacts
  - enforce path jail plus diff validation
  - keep closeout manual
- Forbidden:
  - workflows
  - playbooks
  - closeout ownership outside compatibility hooks already merged from E

Lane E: Closeout compatibility

- Owns:
  - `crates/xtask/src/agent_maintenance/closeout/**`
  - `crates/xtask/tests/agent_maintenance_closeout.rs`
- Goal:
  - preserve manual closeout semantics while remaining compatible with new request metadata
- Forbidden:
  - relay command
  - workflows
  - docs/playbooks

Lane F: Workflow recovery hardening

- Owns:
  - `.github/workflows/agent-maintenance-release-watch.yml`
  - `.github/workflows/agent-maintenance-open-pr.yml`
  - `.github/workflows/codex-cli-update-snapshot.yml`
  - `.github/workflows/claude-code-update-snapshot.yml`
  - `crates/xtask/tests/c4_spec_ci_wiring.rs`
  - `crates/xtask/tests/agent_maintenance_watch.rs` if workflow-facing assertions need updates
- Goal:
  - preserve existing topology
  - ensure packet generation precedes PR creation
  - ensure `governance/pr-summary.md` remains the PR body source
  - make PR-creation recovery explicit
  - preserve one stale agent/version -> one branch/PR concurrency
- Forbidden:
  - watcher redesign
  - registry enrollment changes
  - relay execution logic

Lane G: Docs and playbooks

- Owns:
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
  - `cli_manifests/codex/OPS_PLAYBOOK.md`
  - `cli_manifests/claude_code/OPS_PLAYBOOK.md`
- Goal:
  - document the relay dry-run/write flow
  - freeze packet-only agents as deferred
  - make the manual-closeout boundary explicit
- Forbidden:
  - code changes
  - workflow edits

## Workstream Plan

### Parent-only serialized phases

P0. Baseline and freeze

- capture current parent SHA and dirty-state summary
- hash `PLAN.md`
- record stale assumptions rejected from the old `ORCH_PLAN.md`
- freeze the lane ownership table and proving commands

P1. Schema lane

- launch Lane A alone
- worker runs before handoff:
  - `cargo test -p xtask --test agent_maintenance_refresh`
- merge only after request parsing coverage is green
- record the schema freeze SHA in orchestration state

P2. Renderer lane

- launch Lanes B and E from the schema freeze SHA
- Lane B worker runs before handoff:
  - `cargo test -p xtask --test agent_maintenance_prepare`
- Lane E worker runs before handoff:
  - `cargo test -p xtask --test agent_maintenance_closeout`
- merge only after renderer exactness and digest checks are green
- record the renderer freeze SHA
- merge E whenever its compatibility tests are green; it does not wait on the renderer

P3. Parallel core execution lanes

- launch Lanes C and D from the renderer freeze SHA
- Lane C worker runs before handoff:
  - `cargo test -p xtask --test agent_maintenance_prepare`
- Lane D worker runs before handoff:
  - `cargo test -p xtask --test agent_maintenance_execute`
- merge C only after packet-generation behavior is stable
- merge D only after relay dry-run/write behavior is stable

P4. Workflow lane

- launch Lane F only after Lane C is merged
- worker runs before handoff:
  - `cargo test -p xtask --test agent_maintenance_watch`
  - `cargo test -p xtask --test c4_spec_ci_wiring`
- merge only after workflow contract tests confirm packet-first PR behavior and recovery semantics

P5. Docs lane

- launch Lane G only after D and F are stable enough that docs will not immediately drift
- worker runs before handoff:
  - no additional lane-only command is required beyond keeping docs aligned to the merged command surface from Lanes C, D, and F
- merge only after docs reflect the final command surface and boundaries

P6. Parent integration and final proving

- run the full required command set on the parent branch
- manually verify milestone boundary rules that are not fully encoded by tests
- record acceptance and residual risk

### Parallel worker phases

Phase 1:

- Lane A only

Phase 2:

- Lanes B and E may overlap only if E starts from the Lane A merge SHA
- B remains the serial spine and must merge before C or D launch

Phase 3:

- Lanes C and D can run in parallel from the Lane B merge SHA
- Lane E may still be running in parallel if it launched from the Lane A merge SHA
- C and D must treat the renderer contract from B as frozen

Phase 4:

- Lane F runs after C merges
- Lane G runs last after D and F stabilize

## Stop And Halt Conditions

- Halt any lane that proposes watcher redesign, worker-topology redesign, or registry-enrollment changes.
- Halt any lane that widens executor scope beyond local Codex.
- Halt any lane that tries to pull packet-only agents into relay write mode.
- Halt any lane that moves closeout into relay write mode.
- Halt any lane that writes outside its owned file set.
- Halt C, D, F, and G if Lane A or B contract changes after those lanes launch; relaunch them from the new freeze SHA.
- Halt F if packet generation truth is still moving.
- Halt G if relay or workflow semantics are not yet stable.
- Halt final landing if any required proving command fails.
- Halt final landing if dry-run or write mode writes outside the declared envelope or if the run artifact root is not the temp path required by `PLAN.md`.

## Parent-Only Proving And Integration Gates

Parent-only merge checklist for each lane:

1. worker diff touches only owned files
2. lane-scoped tests passed in the worker report
3. no milestone-boundary violation is visible in the diff
4. parent smoke-check after merge still passes for the merged surface

Parent-only final proving commands, copied exactly from `PLAN.md`:

```sh
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_refresh
cargo test -p xtask --test agent_maintenance_execute
cargo test -p xtask --test agent_maintenance_closeout
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test c4_spec_ci_wiring
make preflight
```

Parent merge and prove flow:

1. Merge Lane A, then run `cargo test -p xtask --test agent_maintenance_refresh`.
2. Launch or continue Lanes B and E from the Lane A merge SHA.
3. Merge Lane B, then run `cargo test -p xtask --test agent_maintenance_prepare`.
4. Merge Lane E whenever ready, then run `cargo test -p xtask --test agent_maintenance_closeout`.
5. Merge Lane C, then run `cargo test -p xtask --test agent_maintenance_prepare`.
6. Merge Lane D, then run `cargo test -p xtask --test agent_maintenance_execute`.
7. Merge Lane F, then run:
   `cargo test -p xtask --test agent_maintenance_watch`
   `cargo test -p xtask --test c4_spec_ci_wiring`
8. Merge Lane G after C, D, and F are stable.
9. Run the full final proving command set on `codex/recommend-next-agent`.
10. Accept the milestone only if the final command set passes and the relay boundary checks below still hold.

Parent-only acceptance checks after commands pass:

- automated upstream-release requests require `[execution_contract]`
- `HANDOFF.md`, `governance/pr-summary.md`, and frozen prompt artifacts come from one shared renderer contract
- `execute-agent-maintenance --dry-run` persists only temp evidence under
  `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`
- `execute-agent-maintenance --write` requires a prepared `run_id`
- relay write mode reuses the frozen dry-run baseline instead of reconstructing state from markdown
- diff validation and path jail enforce `execution_contract.writable_surfaces`
- write mode stops before closeout and prints the manual closeout step
- workflow recovery path is explicit when PR creation fails after packet generation
- docs and playbooks tell the same local relay story as the shipped command surface

## Context-Control Rules

- Parent context stays anchored to:
  - the milestone sections of `PLAN.md`
  - this orchestration file
  - the active lane ownership table
  - merge status and proving results
- Workers receive only:
  - milestone summary
  - locked decisions relevant to the lane
  - exact owned files
  - exact dependency SHA
  - required tests
  - explicit forbidden surfaces
- Do not hand workers the stale `ORCH_PLAN.md` content except as a warning about rejected goals.
- After a lane merges, parent drops that worker's detailed context and keeps only the merged outcome and any remaining risks.
- If a lane is relaunched, parent issues a new freeze SHA and treats prior worker context as stale.
- Parent never lets downstream lanes infer machine truth from rendered markdown. Frozen input contract plus renderer output remain the only execution basis.

## Tests And Acceptance By Lane

Lane A acceptance:

- `crates/xtask/tests/agent_maintenance_refresh.rs` covers valid and invalid `[execution_contract]`
- manual requests remain loadable without `[execution_contract]`

Lane B acceptance:

- shared renderer drives `HANDOFF.md`, `governance/pr-summary.md`, and frozen prompt output
- prompt digest and maintenance-root linkage fail closed

Lane C acceptance:

- `prepare-agent-maintenance --write` emits structured execution contract for automated requests
- writable surfaces, green gates, and recovery data are deterministic

Lane D acceptance:

- dry-run emits one `run_id` and one frozen packet under the temp run root
- write mode requires `--run-id`
- relay reuses the prepared baseline
- path-jail plus diff validation reject out-of-bounds writes
- no-op write and prompt mismatch fail closed
- closeout is not run automatically

Lane E acceptance:

- closeout remains explicit and manual
- new request metadata does not break closeout compatibility

Lane F acceptance:

- packet generation happens before PR creation
- `governance/pr-summary.md` remains the PR body source
- one stale agent/version still maps to one branch/PR
- recovery guidance is explicit when PR opening fails

Lane G acceptance:

- operator guide and playbooks match the final relay flow
- packet-only agents are still explicitly deferred
- manual closeout boundary is visible in maintainer-facing docs

Milestone acceptance:

- all lane acceptances above are satisfied
- all final proving commands pass on `codex/recommend-next-agent`
- no stale watcher-revamp work leaked back into scope

## Assumptions

- worker worktrees will be created under `/Users/spensermcconnell/__Active_Code/atomize-hq/wt`
- each lane can be reviewed and merged independently once its declared dependencies are merged
- the repo keeps the current test file layout from `PLAN.md`; if a lane must introduce a new focused test file to avoid unsafe overlap, parent records that deviation in `freeze.json` before launch
- no hidden milestone requirement exists beyond `PLAN.md` for human approvals or extra landing gates
