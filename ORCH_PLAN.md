# ORCH_PLAN - Worker/Runbook Convergence On Shared Packet Contract

## Summary

Current branch: `staging`  
Authoritative milestone source: repo-root `PLAN.md`  
Milestone target: `Worker/Runbook Convergence On Shared Packet Contract`  
Plan baseline SHA: `12e373a`  
Parent role: sole integrator, sole owner of checkpointing, regeneration, and landing  
Worker model: `GPT-5.4` with `reasoning_effort=high`  
Worker concurrency cap: `2` lanes plus the parent

Why the cap is `2`:

- Phase 1 wording and contract decisions are serialized.
- The current workspace is already dirty in core seam files, generated packet docs, specs, and some tests.
- More fanout would create overlap on exactly the files this milestone must stabilize first.

Current dirty-tree constraint to honor before any implementation:

- Do not assume a clean checkout.
- Review and snapshot the existing modified files first.
- Parent must adopt, defer, or explicitly exclude each dirty seam file before worker launch.
- No worker may touch a file that is already dirty in the user checkout unless the parent has first absorbed that file into the frozen checkpoint.

Worktree root:

```text
/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-worker-runbook-convergence
```

Run-state root:

```text
/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.runs/worker-runbook-convergence-shared-packet-contract
```

This plan replaces the stale repo-root `ORCH_PLAN.md`, which still targets the prior `packet-first-contract-with-c-tail` session.

## Hard Guards

- `PLAN.md` is authoritative over this file on any conflict.
- Parent is the only integrator.
- No transport-topology redesign.
- No new workflow family.
- No closeout semantic change.
- No second policy store.
- No registry schema expansion for freeform worker behavior.
- No worker acquisition unification attempt.
- No hand-maintained generated packet docs.
- `HANDOFF.md` remains canonical and `governance/pr-summary.md` remains derivative.
- `.github/workflows/agent-maintenance-release-watch.yml` remains the only live scheduled release-watch entry point.
- The canonical maintenance branch family stays `automation/<agent_id>-maintenance-<target_version>`.
- Generated packet docs must be fixed through renderer and regeneration paths, not by ad hoc manual edits.
- `PLAN.md` and `TODOS.md` stay parent-owned because they are already dirty and define milestone boundary.

## Authority Model

Parent-only authority:

- interpret milestone scope from `PLAN.md`
- inventory and snapshot the dirty tree
- decide file ownership and lane boundaries
- settle the Phase 1 wording ledger
- freeze checkpoint `C1`
- integrate all worker output
- own all regeneration and final verification
- record the explicit follow-up boundary in `PLAN.md` and `TODOS.md`
- decide whether any discovered surface is active, derived, or out of scope
- land the reviewed stack back onto `staging-live` or `staging` only after all gates pass

Worker authority:

- work only on assigned files from the frozen `C1` base
- use only the canonical wording and branch/watcher decisions published by the parent
- return `ready-for-parent`, `blocked`, or `no-op`
- never merge, never rebase another lane, never widen scope
- never touch parent-owned dirty files
- never hand-edit generated packet roots

## Worktree And Branch Strategy

Use the current `staging` checkout for inspection only. Do not use it as the implementation surface because it already contains user changes.

Initial worktrees:

- `staging-live`
  - path: `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-worker-runbook-convergence/staging-live`
  - branch: `staging`
  - purpose: final landing and smoke diff only
- `parent-core`
  - path: `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-worker-runbook-convergence/parent-core`
  - branch: `codex/worker-runbook-convergence-core`
  - purpose: parent critical path, integration, regen, final validation

Worker worktrees, created only after `C1` freeze:

- `lane-b-runbooks`
  - path: `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-worker-runbook-convergence/lane-b-runbooks`
  - branch: `codex/worker-runbook-convergence-runbooks`
- `lane-c-regressions`
  - path: `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-worker-runbook-convergence/lane-c-regressions`
  - branch: `codex/worker-runbook-convergence-regressions`

Exact creation commands:

```bash
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-worker-runbook-convergence/staging-live staging
git worktree add -b codex/worker-runbook-convergence-core /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-worker-runbook-convergence/parent-core origin/staging
```

After `C1_SHA` is recorded:

```bash
git worktree add -b codex/worker-runbook-convergence-runbooks /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-worker-runbook-convergence/lane-b-runbooks "$C1_SHA"
git worktree add -b codex/worker-runbook-convergence-regressions /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-worker-runbook-convergence/lane-c-regressions "$C1_SHA"
```

All worker branches must start from the exact frozen `C1_SHA`, not from the live dirty checkout.

## Run-State Source Of Truth

Parent-owned run-state files under:

```text
.runs/worker-runbook-convergence-shared-packet-contract/
```

Required records:

- `baseline.json`
- `dirty-tree.md`
- `dirty-tree.diffstat.txt`
- `ownership-map.md`
- `string-ledger.md`
- `freeze.json`
- `lane-status.json`
- `merge-log.md`
- `regen.md`
- `final-gates.md`
- `acceptance.md`

Minimum parent snapshot content:

- `git status --short --branch`
- `git diff --stat`
- file-by-file classification: `adopt-now`, `defer`, `out-of-scope`
- explicit list of files blocked from worker ownership because they are already dirty
- baseline reference to `PLAN.md` at commit `12e373a`

Workers do not write under `.runs/**`.

## Launch Order

### P0 - Parent Baseline And Dirty-Tree Intake

Parent-only.

Required review before any new work:

- `PLAN.md`
- `TODOS.md`
- `cli_manifests/claude_code/OPS_PLAYBOOK.md`
- `crates/xtask/src/agent_maintenance/contract_policy.rs`
- `crates/xtask/src/agent_maintenance/docs.rs`
- `crates/xtask/src/agent_maintenance/execute.rs`
- `crates/xtask/src/agent_maintenance/execute/runtime.rs`
- `crates/xtask/src/agent_maintenance/mod.rs`
- `crates/xtask/src/agent_maintenance/prepare.rs`
- `crates/xtask/src/agent_maintenance/request.rs`
- `crates/xtask/src/agent_maintenance/request/automation.rs`
- `crates/xtask/src/agent_maintenance/watch.rs`
- `crates/xtask/tests/agent_maintenance_closeout.rs`
- `crates/xtask/tests/agent_maintenance_prepare.rs`
- `crates/xtask/tests/agent_maintenance_refresh.rs`
- `crates/xtask/tests/agent_maintenance_refresh/automated_requests.rs`
- `crates/xtask/tests/agent_maintenance_watch.rs`
- `crates/xtask/tests/support/agent_maintenance_refresh_harness.rs`
- `docs/agents/lifecycle/codex-maintenance/HANDOFF.md`
- `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
- `docs/agents/lifecycle/codex-maintenance/governance/remediation-log.md`
- `docs/specs/agent-registry-contract.md`
- `docs/specs/maintenance-request-contract-v1.md`

Required outcome of P0:

- snapshot current modifications
- decide which dirty files are already part of this milestone and must be preserved
- identify which dirty files remain parent-owned through the entire run
- publish the no-overlap ownership map before worker launch

Stop immediately if:

- the dirty seam contains user changes whose intent cannot be classified safely
- a worker lane would need to start on a file that is already dirty and unresolved
- the milestone boundary in `PLAN.md` changes during intake

### P1 - Parent Critical Path And Wording Freeze

Parent-only.

Goal: settle the shared operational story before any parallelism.

Parent-owned critical path files:

- `crates/xtask/src/agent_maintenance/contract_policy.rs`
- `crates/xtask/src/agent_maintenance/docs.rs`
- `crates/xtask/src/agent_maintenance/execute.rs`
- `crates/xtask/src/agent_maintenance/execute/runtime.rs`
- `crates/xtask/src/agent_maintenance/mod.rs`
- `crates/xtask/src/agent_maintenance/prepare.rs`
- `crates/xtask/src/agent_maintenance/request.rs`
- `crates/xtask/src/agent_maintenance/request/automation.rs`
- `crates/xtask/src/agent_maintenance/watch.rs`
- dirty seam tests tied directly to those paths:
  - `crates/xtask/tests/agent_maintenance_prepare.rs`
  - `crates/xtask/tests/agent_maintenance_refresh.rs`
  - `crates/xtask/tests/agent_maintenance_refresh/automated_requests.rs`
  - `crates/xtask/tests/agent_maintenance_watch.rs`
  - `crates/xtask/tests/support/agent_maintenance_refresh_harness.rs`

Mandatory decisions to freeze in `string-ledger.md`:

- canonical live watcher reference
- canonical branch family
- canonical recovery and replay wording
- canonical generated ownership marker language
- canonical execution-agent wording
- canonical packet ownership story
- explicit wording for write-envelope honesty
- explicit confirmation that workflow YAML stays transport-only

`C1` freeze gate:

- the shared watcher, branch, replay, recovery, and execution-agent wording are stable enough that workers can update downstream surfaces without inventing strings
- all currently dirty core seam files needed for those decisions have been absorbed into `parent-core`
- the parent has published a final ownership map showing which dirty files remain parent-only

Required `C1` validation:

```bash
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_refresh
```

Record `C1_SHA` in `freeze.json`. No worker starts before that SHA exists.

## Post-Checkpoint Parallel Worker Lanes

### Lane B - Runbooks, Workflows, And Active Human Surfaces

Model: `GPT-5.4` with `reasoning_effort=high`  
Launch base: exact `C1_SHA`

Owned files:

- `.github/workflows/agent-maintenance-release-watch.yml`
- `.github/workflows/agent-maintenance-open-pr.yml`
- `.github/workflows/codex-cli-update-snapshot.yml`
- `.github/workflows/claude-code-update-snapshot.yml`
- `cli_manifests/codex/OPS_PLAYBOOK.md`
- `cli_manifests/codex/CI_WORKFLOWS_PLAN.md`
- `cli_manifests/codex/PR_BODY_TEMPLATE.md`
- `cli_manifests/claude_code/CI_WORKFLOWS_PLAN.md`
- `cli_manifests/claude_code/PR_BODY_TEMPLATE.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `crates/xtask/tests/c4_spec_ci_wiring.rs`

Explicitly forbidden:

- `cli_manifests/claude_code/OPS_PLAYBOOK.md` because it is already dirty and stays parent-owned
- any file under `crates/xtask/src/agent_maintenance/**`
- any generated maintenance root under `docs/agents/lifecycle/*-maintenance/**`
- `docs/specs/**`
- `PLAN.md`
- `TODOS.md`

Mission:

- align workflow fallback text, replay wording, and runbook wording to the frozen string ledger
- remove live references to deleted per-agent watcher workflows
- align branch examples and PR-template wording to `automation/<agent_id>-maintenance-<target_version>`
- keep workflow YAML transport-only
- make the operator guide honest about the live write envelope
- update `c4_spec_ci_wiring.rs` to lock the new watcher and branch truth

Required lane validation:

```bash
cargo test -p xtask --test c4_spec_ci_wiring
```

Lane B stop conditions:

- any required change lands in a parent-owned dirty file
- any required wording contradicts the frozen string ledger
- the lane would require workflow-topology redesign instead of wording convergence

### Lane C - Clean Regression Net

Model: `GPT-5.4` with `reasoning_effort=high`  
Launch base: exact `C1_SHA`

Owned files:

- `crates/xtask/tests/agent_maintenance_execute.rs`
- `crates/xtask/tests/support/agent_maintenance_harness.rs`
- `crates/xtask/tests/support/agent_maintenance_closeout_harness.rs`

Explicitly forbidden:

- dirty test files already owned by the parent
- any source file under `crates/xtask/src/**`
- any workflow YAML
- any spec doc
- any generated maintenance root
- `PLAN.md`
- `TODOS.md`

Mission:

- lock the execution-agent wording and maintained-agent/executor distinction
- add or strengthen clean Claude Code parity coverage where this can be done without touching dirty prepare/refresh tests
- close any clean regression gaps around active worker/runbook convergence that do not require parent-owned fixtures

Required lane validation:

```bash
cargo test -p xtask --test agent_maintenance_execute
```

Lane C stop conditions:

- parity coverage requires changes in dirty parent-owned prepare or refresh tests
- the lane needs source-code changes instead of clean regression assertions
- the lane would force closeout semantic changes

### Worker Handoff Contract

Every worker must return:

- lane id
- status: `ready-for-parent`, `blocked`, or `no-op`
- `C1_SHA`
- changed files
- commands run
- validation results
- commit SHA
- unresolved assumptions

## P2 - Parent Integration, Dirty-Surface Completion, And Deterministic Regen

Parent-only.

Merge order:

1. integrate Lane B into `parent-core`
2. rebase Lane C onto the new parent tip only if needed
3. integrate Lane C
4. complete all parent-owned dirty surfaces using the now-final downstream wording
5. run deterministic regeneration for in-scope maintenance roots
6. update `PLAN.md` and `TODOS.md` with the explicit follow-up boundary last

Parent-owned completion set after worker merge:

- `cli_manifests/claude_code/OPS_PLAYBOOK.md`
- `docs/specs/agent-registry-contract.md`
- `docs/specs/maintenance-request-contract-v1.md`
- `docs/agents/lifecycle/codex-maintenance/HANDOFF.md`
- `docs/agents/lifecycle/codex-maintenance/README.md`
- `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
- `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md`
- `docs/agents/lifecycle/codex-maintenance/governance/remediation-log.md`
- `docs/agents/lifecycle/codex-maintenance/review_surfaces.md`
- `docs/agents/lifecycle/codex-maintenance/scope_brief.md`
- `docs/agents/lifecycle/codex-maintenance/seam_map.md`
- `docs/agents/lifecycle/codex-maintenance/threading.md`
- remaining dirty tests:
  - `crates/xtask/tests/agent_maintenance_closeout.rs`
  - `crates/xtask/tests/agent_maintenance_prepare.rs`
  - `crates/xtask/tests/agent_maintenance_refresh.rs`
  - `crates/xtask/tests/agent_maintenance_refresh/automated_requests.rs`
  - `crates/xtask/tests/agent_maintenance_watch.rs`
  - `crates/xtask/tests/support/agent_maintenance_refresh_harness.rs`
- `PLAN.md`
- `TODOS.md`

### Deterministic Validation And Regen Flow

Use one deterministic decision tree for generated maintenance roots.

In-scope committed maintenance roots:

- `docs/agents/lifecycle/codex-maintenance/**`

Explicitly not created in this milestone:

- `docs/agents/lifecycle/claude_code-maintenance/**`

Default out of scope unless parent proves active dependency:

- `docs/agents/lifecycle/opencode-maintenance/**`

Regen decision tree:

1. Inspect the committed request packet and determine whether request-owned fields must change.
2. If request-owned fields change, the parent must use the packet-preparation path first.
3. If request-owned fields do not change and only derived packet docs need rerendering, the parent may use the refresh path directly.
4. After any packet-preparation write, the parent must run `refresh-agent --dry-run` as a parity check.
5. If prepare and refresh disagree, stop and fix the source-of-truth split before landing.
6. Never hand-edit generated maintenance docs to force agreement.

Request-owned fields that force packet preparation first include, at minimum:

- branch family materialized in the request
- watcher or dispatch fields materialized in the request
- recovery or ownership fields emitted into the request contract
- any other top-level or section-owned request data, not just renderer text

Direct-refresh-only path is allowed only when:

- the request packet is already current
- the changes are limited to renderer-owned derived docs
- `refresh-agent --dry-run` from the current request is expected to converge without request mutation

Example parity check after a packet-preparation write:

```bash
cargo run -p xtask -- refresh-agent \
  --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml \
  --dry-run
```

Required parent regen evidence in `regen.md`:

- whether the parent used packet preparation first or direct refresh
- exact command path used
- whether request fields changed or only derived docs changed
- before/after file list for the codex maintenance root
- final dry-run result showing no remaining drift

## P3 - Parent Final Validation, Landing, And Acceptance Recording

Parent-only.

This phase begins only after:

- all worker output is integrated into `parent-core`
- all parent-owned dirty docs/specs/generated surfaces are complete
- deterministic regen has converged
- no unresolved merge hotspot remains

Required final validation from `parent-core`:

```bash
cargo test -p xtask --test c0_spec_validate
cargo test -p xtask --test c4_spec_ci_wiring
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_execute
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test agent_maintenance_refresh
cargo test -p xtask --test agent_maintenance_closeout
make fmt-check
make clippy
make check
make test
```

Landing rules:

- `staging-live` stays untouched until all final gates pass on `parent-core`.
- Parent lands back onto `staging-live` or `staging` only after the full validation set is green.
- Preferred flow: fast-forward or cherry-pick the reviewed `parent-core` commit stack onto `staging-live`, inspect the smoke diff there, then update `staging`.
- If the landing path cannot remain clean and reviewable, stop and resolve before touching `staging`.

Required landing procedure:

1. verify `parent-core` is green
2. update `merge-log.md` and `final-gates.md`
3. move `staging-live` to the reviewed parent result
4. run a final smoke diff on `staging-live`
5. record landed commit SHA(s) and gate results in `acceptance.md`
6. only then land or fast-forward `staging`

## Merge Hotspots

Parent must inspect these manually before landing:

- `crates/xtask/src/agent_maintenance/contract_policy.rs`
- `crates/xtask/src/agent_maintenance/docs.rs`
- `crates/xtask/src/agent_maintenance/execute/runtime.rs`
- `crates/xtask/src/agent_maintenance/prepare.rs`
- `crates/xtask/src/agent_maintenance/request/automation.rs`
- `crates/xtask/src/agent_maintenance/watch.rs`
- `cli_manifests/claude_code/OPS_PLAYBOOK.md`
- `docs/specs/maintenance-request-contract-v1.md`
- `docs/specs/agent-registry-contract.md`
- `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
- `docs/agents/lifecycle/codex-maintenance/HANDOFF.md`
- `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md`
- `crates/xtask/tests/agent_maintenance_prepare.rs`
- `crates/xtask/tests/agent_maintenance_refresh/automated_requests.rs`
- `crates/xtask/tests/c4_spec_ci_wiring.rs`
- `crates/xtask/tests/agent_maintenance_execute.rs`

## Context-Control Rules

- Read only the active milestone files and their directly coupled tests.
- Do not widen into unrelated crates or unrelated `xtask` domains.
- Do not bulk-load `docs/`; open only named maintenance, operator, or spec surfaces.
- Treat generated packet roots as outputs, not freeform design documents.
- Treat current dirty files as reserved until the parent explicitly releases them.
- If a worker discovers a needed edit outside its ownership map, it must stop and return `blocked`.
- Do not let workers invent alternate branch families, watcher names, or replay phrasing.

## Tests And Acceptance

### Operational Story

Done means:

- one live watcher is named everywhere active maintenance is described
- one maintenance branch family is named everywhere active maintenance is described
- workflow fallbacks, playbooks, operator guide, packet docs, and tests tell the same replay and recovery story
- shared relay wording clearly distinguishes maintained agent from execution agent

### Generated Packet Surfaces

Done means:

- generated automated packet docs no longer carry stale ownership markers
- generated automated packet docs no longer describe the lane with stale worker-era framing
- `HANDOFF.md` remains canonical
- `governance/pr-summary.md` remains derivative
- no generated maintenance root is hand-maintained to force agreement

### Regression Net

Done means:

- stale watcher names fail tests
- stale branch-family examples fail tests
- stale packet ownership wording fails tests
- execution-agent wording is covered
- Codex and Claude Code convergence is proven either in parent-owned prepare/refresh tests or in clean execute/harness tests, without inventing a new committed Claude maintenance root

### Workspace Boundary

Done means:

- no transport redesign was introduced
- no new workflow family was introduced
- no second policy store was introduced
- no closeout semantic change was introduced
- worker lanes never touched parent-owned dirty files

### Required Final Commands

```bash
cargo test -p xtask --test c0_spec_validate
cargo test -p xtask --test c4_spec_ci_wiring
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_execute
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test agent_maintenance_refresh
cargo test -p xtask --test agent_maintenance_closeout
make fmt-check
make clippy
make check
make test
```

## Stop Conditions

Stop and re-plan immediately if any of the following occur:

- the dirty-tree review cannot safely distinguish user work from lane work
- Phase 1 cannot settle without redesigning transport topology
- any worker needs a parent-owned dirty file
- generated packet docs cannot be regenerated deterministically from the chosen command path
- `refresh-agent` and packet preparation disagree after regen
- a second policy store appears necessary
- a new workflow family appears necessary
- closeout behavior would need to change
- a clean Claude proof cannot be achieved without creating a new committed `claude_code-maintenance/**` root
- `PLAN.md` changes milestone boundaries after `C1`

## Follow-Up Boundary

This milestone ends at worker/runbook convergence on the current shared packet contract.

Explicitly deferred:

- transport-topology convergence review
- worker acquisition unification
- new replay command families
- new workflow families
- closeout automation
- any second store of packet policy truth

The only acceptable follow-up note in `PLAN.md` and `TODOS.md` is that broader transport-topology convergence remains a separate later decision.

## Assumptions

- `docs/agents/lifecycle/codex-maintenance/**` is the only committed automated maintenance root that must be regenerated for this milestone.
- `claude_code` parity is proven through code and tests, not by creating a new committed maintenance root.
- The currently dirty seam files are intended inputs to preserve and reconcile, not noise to discard.
- Any additional active watcher or branch-family references discovered during P0 in the named maintenance surfaces are part of this same convergence pass, not a separate milestone.

## Exit Criteria

This plan is complete when:

1. the parent has preserved and integrated the intended dirty seam without overwriting user work
2. the shared watcher is the only live watcher named across active surfaces
3. one maintenance branch family is used everywhere active maintenance is described
4. generated packet docs, source playbooks, workflows, operator docs, and tests tell one story
5. deterministic regeneration is proven for the codex maintenance root
6. the final gate suite passes from `parent-core` before anything lands back on `staging`
