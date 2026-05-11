# PLAN - First Honest Maintenance Proof Run

Status: ready for implementation  
Date: 2026-05-10  
Working branch: `staging`  
Workflow checkout ref: `staging`  
Repo default branch: `main`  
Convergence baseline: `ee8249a`  
Plan revision baseline: `bbd15f6`  
Proof target: `codex`  
Design input: `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-design-20260510-200245.md`  
Supersedes: the prior repo-root `PLAN.md` for `worker/runbook convergence on the shared packet contract`

## Executive Summary

The repo just spent two milestones making the maintenance story truthful:

1. packet-first contract convergence
2. worker/runbook convergence on top of that shared packet contract

The next honest step is not more cleanup and not a universal workflow cutover. The next honest
step is to prove that the current maintenance topology actually works end to end for one real
maintained agent without tribal knowledge.

This plan locks the first proof target to `codex`, starts at the real shared watcher entrypoint,
forces the downstream worker, generated packet, local relay, recovery path, and manual closeout
to run in order, and ends with one explicit decision:

- keep the current split worker topology for now, with only localized fixes, or
- open a separate transport-topology convergence follow-up next

Prove first. Redesign second, only if the proof earns it.

## Objective

Run one decision-bearing maintenance proof pass for `codex` that starts at the shared watcher,
flows through the real downstream worker and generated packet, exercises the local relay and
manual closeout path, and exits with a written topology decision based on evidence instead of
architecture taste.

## Success Criteria

1. The proof begins with `.github/workflows/agent-maintenance-release-watch.yml`, not a direct
   trigger of `.github/workflows/codex-cli-update-snapshot.yml`.
2. The shared watcher emits or proves the expected stale-agent queue entry for `codex`, including
   the canonical branch shape `automation/codex-maintenance-<target_version>`.
3. The downstream `codex-cli-update-snapshot.yml` worker runs from shared watcher dispatch and
   produces the expected maintenance packet surfaces under
   `docs/agents/lifecycle/codex-maintenance/**`.
4. A maintainer can follow the generated packet and run:
   - `cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --dry-run`
   - `cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --write --run-id <prepared_run_id>`
   without undocumented operator steps.
5. If PR creation fails after packet generation, the documented
   `refresh-agent --request ... --write` recovery path is sufficient to reopen the PR from the
   frozen request and generated summary without artifact surgery.
6. The run closes through `maintenance-closeout.json` plus `close-agent-maintenance`, and the
   closeout truth is explicit about resolved findings, deferred findings, and whether repo
   preflight passed.
7. The milestone exits with one written decision backed by proof evidence:
   - keep the split `codex` / `claude_code` worker topology for now, or
   - open a separate transport-topology convergence follow-up next

## Step 0 Scope Challenge

### Premise Challenge

| Premise | Assessment | Decision |
| --- | --- | --- |
| The current maintenance topology is finally truthful enough to be tested directly. | Accepted. The watcher, packet contract, worker/runbook wording, relay contract, and closeout surfaces now line up well enough that a proof run measures reality, not stale docs. | Prove this topology before redesigning it. |
| `codex` is the cleanest first proving target. | Accepted. `codex` is release-watch enrolled, has a committed generated maintenance root, and already uses the shared packet/relay path. | Lock `codex` as the first proof target. |
| A proof run that starts at the downstream worker is good enough. | Rejected. That skips the exact shared watcher seam the last milestone was trying to make truthful. | Start at `agent-maintenance-release-watch.yml`. |
| The next milestone should also decide or implement a universal worker cutover. | Rejected. That mixes validation with redesign and makes every failure ambiguous. | Defer topology redesign unless the proof earns it. |
| `goose` should be part of this proof. | Rejected for this milestone. `goose` remains a separate create-lane proving target already captured in `TODOS.md`. | Keep `goose` out of scope here. |
| A proof run without an explicit exit rule is enough. | Rejected. That creates a ceremonial run with no decision value. | Use a predeclared topology decision rule. |

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| Shared release detection and dispatch | `.github/workflows/agent-maintenance-release-watch.yml`, `crates/xtask/src/agent_maintenance/watch.rs`, `crates/xtask/data/agent_registry.toml` | Reuse exactly. The proof starts here. |
| `codex` maintenance worker | `.github/workflows/codex-cli-update-snapshot.yml` | Reuse exactly. Do not fork a proof-only worker. |
| Packet generation | `prepare-agent-maintenance`, `docs/agents/lifecycle/codex-maintenance/**`, `docs/specs/maintenance-request-contract-v1.md` | Reuse exactly. Generated packet surfaces are part of the proof evidence. |
| Local relay execution | `execute-agent-maintenance`, `docs/cli-agent-onboarding-factory-operator-guide.md`, `cli_manifests/codex/OPS_PLAYBOOK.md` | Reuse exactly. The proof must exercise this lane, not describe it abstractly. |
| Maintenance closeout | `close-agent-maintenance`, `docs/agents/lifecycle/*-maintenance/governance/maintenance-closeout.json` | Reuse exactly. No proof-specific closeout format. |
| Recovery after PR-open failure | generated `HANDOFF.md`, generated `pr-summary.md`, `refresh-agent --request ... --write` | Reuse exactly. This path must be proven or truthfully flagged. |
| Future topology decision | `TODOS.md` | Reuse. Record the outcome there instead of inventing a second planning surface. |

### Existing Code Leverage Map

| Need | Best leverage point |
| --- | --- |
| Confirm enrolled agent truth | `crates/xtask/data/agent_registry.toml` |
| Prove watcher queue contents locally before spending Actions time | `cargo run -p xtask -- maintenance-watch --check` and `--emit-json` |
| Prove shared dispatch path | `.github/workflows/agent-maintenance-release-watch.yml` |
| Prove worker-specific acquisition + packet preparation | `.github/workflows/codex-cli-update-snapshot.yml` |
| Prove generated packet truth | `docs/agents/lifecycle/codex-maintenance/{HANDOFF.md,governance/maintenance-request.toml,governance/pr-summary.md}` |
| Prove relay trust step and write envelope | `execute-agent-maintenance` |
| Prove manual closeout | `maintenance-closeout.json` + `close-agent-maintenance` |
| Record future topology decision | `TODOS.md` |

### Minimum Complete Change

The minimum complete milestone is:

1. freeze the proving target, scorecard, and topology exit rule
2. locally preflight the shared watcher queue before spending live Actions time
3. manually trigger the shared watcher and capture the real downstream worker evidence
4. consume the generated packet and exercise relay dry-run plus write mode
5. prove or truthfully fail the documented recovery path if PR opening fails
6. record maintenance closeout truth and a written topology decision
7. lock any discovered localized bugs with regression tests before declaring the proof done

Anything smaller is a demo, not a proof.

### Complexity Check

This milestone should not introduce new infrastructure.

Smells that mean the plan is overbuilt:

- a new proof-only xtask command
- a new workflow family just for proving
- a new packet schema or second closeout format
- a second proving target in the same milestone
- a universal worker redesign starting before the first proof finishes

### Search / Build Decision

- **[Layer 1]** Reuse the existing shared watcher, downstream worker, packet generator, relay,
  and closeout commands.
- **[Layer 1]** Reuse the existing maintenance packet surfaces and closeout JSON instead of
  inventing a proof-run artifact family.
- **[Layer 1]** Reuse `TODOS.md` for the topology follow-up decision.
- **[Layer 3]** Treat the topology decision rule as a repo-specific first-principles safeguard:
  prove the current system before changing it.

### Distribution Check

No new external artifact is introduced.

This is an internal factory milestone. Its outputs are:

- shared watcher and downstream worker run evidence
- generated `codex` maintenance packet surfaces
- temp relay proof evidence under `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`
- `docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json`
- a written keep-vs-cutover decision in `TODOS.md`

### Blocking Preconditions

The proof MUST NOT start until all five conditions below are true.

| Precondition | Why it matters | Required action |
| --- | --- | --- |
| `origin/staging` contains the intended maintenance machinery | Both live workflows explicitly check out `staging`, not the local worktree. A local-only commit means the proof runs stale code. | Run `git fetch origin`, compare `git rev-parse staging` vs `git rev-parse origin/staging`, and push `staging` first if they differ. |
| `codex` is stale at execution time | This proof is about the real shared watcher queue, not synthetic worker inputs. | `maintenance-watch --emit-json` must surface a real `codex` queue item. If it does not, stop. Do not fabricate inputs. |
| GitHub dispatch permissions exist | The proof requires a real shared watcher run. | Use `gh workflow run ...` if authenticated, otherwise dispatch from the GitHub UI. |
| The local execution host is available or repairable through the documented path | `execute-agent-maintenance --dry-run` is a required trust step. | If host preflight fails, repair the local Codex CLI/auth state and rerun dry-run. Do not widen repo scope until that path is proven insufficient. |
| The queue-emitted version is treated as the only live target truth | The committed `docs/agents/lifecycle/codex-maintenance/**` surfaces currently show an older prepared lane shape. The live proof may target a newer version. | Trust the queue-generated `target_version` and branch name from the current run, not the pre-existing example packet. |

If `maintenance-watch` shows no stale `codex` candidate at execution time, this milestone is
`BLOCKED: no live stale codex target`. The correct move is to wait for the next eligible upstream
release or intentionally create a new maintenance situation through the normal maintenance lane.
Do not invent a proof-only stale target.

## Locked Decisions

1. The first proof target is `codex`, not `claude_code` and not `goose`.
2. The proof must start at `.github/workflows/agent-maintenance-release-watch.yml`.
3. Directly triggering `codex-cli-update-snapshot.yml` is not a valid opening move for this
   milestone.
4. This is a maintenance-mode proof, not a create-mode lifecycle proof.
5. The milestone reuses the current packet contract, relay contract, and closeout format. No new
   proof-only schema is allowed.
6. Local host issues such as missing Codex auth or a broken local binary are recorded as evidence,
   but they count as topology failures only if the documented host-repair path itself is unclear
   or broken.
7. Localized bugs found during the proof are fixed in this milestone only if they stay within the
   current topology and the blast radius is still boilable.
8. A universal worker cutover becomes eligible only if the proof hits the predeclared topology
   trigger rule in Phase 6.
9. `claude_code` is a secondary control target only if the `codex` proof reveals ambiguity that
   looks target-specific rather than topology-wide.

## Architecture

### Current Topology

```text
agent_registry.toml
  -> maintenance.release_watch truth
  -> current_validated pointer
  -> downstream dispatch workflow per enrolled agent

agent-maintenance-release-watch.yml
  -> checks out staging
  -> runs maintenance-watch
  -> fans out stale enrolled agents

codex-cli-update-snapshot.yml
  -> acquires upstream artifacts
  -> snapshots / unions / reports / validates
  -> prepare-agent-maintenance --write
  -> opens or refreshes maintenance PR

generated codex maintenance packet
  -> HANDOFF.md (canonical)
  -> maintenance-request.toml (frozen relay contract)
  -> pr-summary.md (derivative)

local maintainer relay
  -> execute-agent-maintenance --dry-run
  -> execute-agent-maintenance --write --run-id <prepared_run_id>

manual closeout
  -> maintenance-closeout.json
  -> close-agent-maintenance
```

### Target Proof Flow

```text
Phase 1: prerequisite lock
  push staging if needed
  confirm proof target + decision rule
        |
        v
Phase 2: local queue preflight
  cargo run -p xtask -- maintenance-watch --check
  cargo run -p xtask -- maintenance-watch --emit-json _ci_tmp/maintenance-watch.json
        |
        v
Phase 3: shared watcher
  gh workflow run agent-maintenance-release-watch.yml --ref staging
        |
        v
shared queue dispatch
  codex -> codex-cli-update-snapshot.yml
        |
        v
worker outputs
  artifacts + generated packet + PR open attempt
        |
        v
Phase 4: maintainer proof
  review HANDOFF.md / request / pr-summary
  execute-agent-maintenance --dry-run
  execute-agent-maintenance --write --run-id <prepared_run_id>
        |
        v
Phase 5: recovery + closeout
  optional refresh-agent replay if PR open failed
  maintenance-closeout.json
  close-agent-maintenance
        |
        v
Phase 6: localized fixes + decision
  regression tests
  rerun impacted gates
  write keep-vs-cutover verdict to TODOS.md
```

### Source-of-Truth Boundaries

| Surface | Owner | Consumers | Rule |
| --- | --- | --- | --- |
| stale-agent truth | `agent_registry.toml` + `maintenance-watch` | shared watcher, downstream worker, maintainer | one source of truth |
| transport entrypoint | `agent-maintenance-release-watch.yml` | GitHub Actions, maintainers | start here, not downstream |
| downstream acquisition | `codex-cli-update-snapshot.yml` | automation lane | transport only, no second policy store |
| packet truth | `prepare-agent-maintenance` output | maintainers, relay, PR body | `HANDOFF.md` remains canonical |
| local relay truth | `execute-agent-maintenance` | maintainers | trust packet-owned write envelope and gates |
| closeout truth | `maintenance-closeout.json` + `close-agent-maintenance` | maintainers, future reviews | one explicit maintenance closeout path |
| redesign follow-up | `TODOS.md` | future planning | no silent topology drift |

### Blast Radius

Primary blast radius:

- shared watch and registry truth:
  - `.github/workflows/agent-maintenance-release-watch.yml`
  - `crates/xtask/src/agent_maintenance/watch.rs`
  - `crates/xtask/data/agent_registry.toml`
- `codex` maintenance worker:
  - `.github/workflows/codex-cli-update-snapshot.yml`
  - `cli_manifests/codex/**`
- shared maintenance packet and relay:
  - `crates/xtask/src/agent_maintenance/{prepare.rs,execute.rs,execute/**,closeout.rs,closeout/**,docs.rs,contract_policy.rs}`
  - `docs/specs/maintenance-request-contract-v1.md`
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
- generated packet surfaces:
  - `docs/agents/lifecycle/codex-maintenance/**`
- regression coverage:
  - `crates/xtask/tests/{agent_maintenance_watch.rs,agent_maintenance_prepare.rs,agent_maintenance_execute.rs,agent_maintenance_closeout.rs,c4_spec_ci_wiring.rs}`
  - `crates/xtask/tests/support/agent_maintenance_*`
- written decision:
  - `TODOS.md`

### Evidence Packet

This milestone is complete only if these artifacts exist and are cited in the closeout decision.

| Evidence | Path or source |
| --- | --- |
| Local queue preflight | `_ci_tmp/maintenance-watch.json` from local `maintenance-watch --emit-json` |
| Shared watcher run | GitHub Actions run URL for `agent-maintenance-release-watch.yml` |
| Downstream worker run | GitHub Actions run URL for `codex-cli-update-snapshot.yml` |
| Frozen request | `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml` |
| Canonical execution contract | `docs/agents/lifecycle/codex-maintenance/HANDOFF.md` |
| Derivative PR body | `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md` |
| Relay dry-run/write evidence | `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/` |
| Branch / PR result | `automation/codex-maintenance-<target_version>` plus PR URL or replay evidence |
| Closeout truth | `docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json` |
| Final topology decision | dated note under the existing transport-topology TODO in `TODOS.md` |

## Implementation Plan

### Phase 1. Freeze The Proof Contract And Prerequisites

Purpose: remove ambiguity before any live run starts.

Primary surfaces:

- `PLAN.md`
- `TODOS.md`
- `crates/xtask/data/agent_registry.toml`
- `.github/workflows/agent-maintenance-release-watch.yml`
- `.github/workflows/codex-cli-update-snapshot.yml`
- `docs/specs/maintenance-request-contract-v1.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

Exact tasks:

1. Confirm `codex` is still the correct first proving target from registry truth and committed
   maintenance docs.
2. Freeze the topology decision rule in this plan before any proof evidence is collected.
3. Verify remote branch alignment:

   ```bash
   git fetch origin
   git rev-parse staging
   git rev-parse origin/staging
   ```

   If those SHAs differ, push `staging` before continuing. Do not run the live proof against a
   stale remote ref.

4. Treat the pre-existing `docs/agents/lifecycle/codex-maintenance/**` files as shape references
   only until the worker regenerates them during Phase 3.
5. Do not create any proof-only workflow, proof-only command, or proof-only artifact type.

Exit criteria:

1. Everyone can answer "what counts as topology failure?" from this plan alone.
2. The remote `staging` ref is confirmed to contain the intended maintenance machinery.

### Phase 2. Preflight The Shared Watcher Locally

Purpose: prove the expected queue before spending GitHub Actions time.

Primary surfaces:

- `crates/xtask/src/agent_maintenance/watch.rs`
- `crates/xtask/data/agent_registry.toml`
- local temp output `_ci_tmp/maintenance-watch.json`

Exact tasks:

1. Run:

   ```bash
   cargo run -p xtask -- maintenance-watch --check
   cargo run -p xtask -- maintenance-watch --emit-json _ci_tmp/maintenance-watch.json
   ```

2. Confirm the queue contains `codex` as a stale enrolled agent.
3. Record from the emitted queue item:
   - `agent_id`
   - `current_validated`
   - `latest_stable`
   - `target_version`
   - `dispatch_workflow`
   - `branch_name`
   - `detected_by`
4. If the queue does not produce a `codex` proof target, stop. This is not a call to handcraft
   worker inputs. It is either:
   - `blocked_no_stale_codex`, or
   - a localized watcher / registry truth bug that must be fixed before touching live workflows
5. Save the emitted JSON as proof input for Phase 3 comparison.

Exit criteria:

1. Local queue truth matches registry expectations.
2. The proof can name the exact branch and downstream worker before GitHub dispatch.

### Phase 3. Trigger The Shared Watcher And Capture Dispatch Evidence

Purpose: prove the real entrypoint and real downstream fanout.

Primary surfaces:

- `.github/workflows/agent-maintenance-release-watch.yml`
- `.github/workflows/codex-cli-update-snapshot.yml`
- GitHub Actions run metadata
- regenerated `docs/agents/lifecycle/codex-maintenance/**`

Exact tasks:

1. Manually trigger the shared watcher from `staging`:

   ```bash
   gh workflow run agent-maintenance-release-watch.yml --ref staging
   ```

   GitHub UI trigger is acceptable if CLI auth is unavailable.

2. Capture the shared watcher run URL and the downstream `codex` worker run URL. If using `gh`,
   the preferred evidence capture commands are:

   ```bash
   gh run list --workflow agent-maintenance-release-watch.yml --branch staging --limit 1
   gh run list --workflow codex-cli-update-snapshot.yml --branch staging --limit 1
   ```

3. Verify the downstream worker inputs match the locally emitted queue:
   - same `agent_id`
   - same `current_version` / `current_validated`
   - same `latest_stable`
   - same `target_version`
   - same `dispatch_kind`
   - same `dispatch_workflow`
   - same `branch_name`
4. If the live watcher sees a newer release than the local preflight and the queue item differs,
   discard the old local preflight evidence and restart from Phase 2. Do not mix queue evidence
   from two different target versions.
5. Verify the downstream worker either:
   - opens the expected PR branch successfully, or
   - fails only at PR creation, with the documented replay path printed cleanly
6. Do not bypass this phase by manually running `prepare-agent-maintenance` first. The worker run
   must produce the packet as part of the proof.

Exit criteria:

1. The shared watcher is proven to be a real working control-plane entrypoint.
2. Shared queue truth survives the handoff to the worker workflow without mutation.
3. The maintenance packet at `docs/agents/lifecycle/codex-maintenance/**` now belongs to the
   current live proof run, not just the previously committed example lane.

### Phase 4. Consume The Generated Packet And Prove The Local Relay

Purpose: prove the maintainer-facing execution lane, not just worker artifact generation.

Primary surfaces:

- `docs/agents/lifecycle/codex-maintenance/HANDOFF.md`
- `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
- `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md`
- `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`
- `execute-agent-maintenance`

Exact tasks:

1. Review the generated packet in this order:
   - `HANDOFF.md`
   - `maintenance-request.toml`
   - `pr-summary.md`
2. Confirm the packet agrees on:
   - `target_version`
   - `branch_name`
   - `detected_by`
   - `dispatch_workflow`
   - `executor = "execute-agent-maintenance"`
   - `closeout_path`
3. Run relay dry-run:

   ```bash
   cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --dry-run
   ```

4. Record the prepared `run_id` and the temp evidence directory under
   `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`.
5. Run relay write mode with the same `run_id`:

   ```bash
   cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --write --run-id <prepared_run_id>
   ```

6. Classify any failure immediately using the table below.

### Failure Classification Table

| Class | Meaning | Required response |
| --- | --- | --- |
| `host_env` | local Codex binary, auth, or execution-host setup problem while packet and docs remain clear | repair the local host using the documented path, then rerun the same proof segment |
| `local_bug` | code or contract bug inside the existing topology | fix locally if the blast radius stays boilable, then rerun the impacted segment |
| `doc_drift` | instructions are wrong or incomplete | fix the docs plus the matching regression lock, then rerun the impacted segment |
| `target_specific` | `codex`-specific issue that does not imply topology redesign | fix locally if boilable; do not escalate architecture yet |
| `topology_issue` | the split watch -> worker -> packet -> relay model itself is the problem | stop widening scope, record the trigger evidence, move to the decision phase |

Rules:

1. If the failure is `host_env`, repair only through the documented host-repair path and rerun
   the same proof segment.
2. If the failure is `local_bug`, `doc_drift`, or `target_specific`, fix it in this milestone if
   the fix is localized and the blast radius remains boilable.
3. If the failure is `topology_issue`, stop widening scope. Do not start a universal workflow
   cutover here.

Exit criteria:

1. A maintainer can get from generated packet to relay dry-run and write mode without hidden
   steps, or the exact gap is proven.
2. Failures are classified cleanly instead of becoming folklore.

### Phase 5. Prove Recovery And Manual Closeout

Purpose: prove that the lane closes honestly, including failure handling.

Primary surfaces:

- `refresh-agent`
- `docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json`
- `close-agent-maintenance`
- generated maintenance closeout packet surfaces

Exact tasks:

1. If PR creation failed during Phase 3, use the documented replay path exactly:

   ```bash
   cargo run -p xtask -- refresh-agent --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --write
   gh pr create --base staging --head "automation/codex-maintenance-<target_version>" --title "..." --body-file docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md
   ```

2. Confirm that replay does not require packet surgery, branch renaming, or ad hoc prose edits.
3. Populate `maintenance-closeout.json` truthfully:
   - `resolved_findings`
   - `deferred_findings` or `explicit_none_reason`
   - `preflight_passed`
   - `recorded_at`
   - `commit`
4. Close the run:

   ```bash
   cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json
   ```

5. Verify the generated post-closeout packet surfaces still tell the same story.

Exit criteria:

1. Recovery is real if needed.
2. Closeout is explicit, manual, and self-sufficient.

### Phase 6. Lock In Localized Fixes And Record The Topology Decision

Purpose: prevent the proof from becoming a one-off anecdote.

Primary surfaces:

- `crates/xtask/tests/{agent_maintenance_watch.rs,agent_maintenance_prepare.rs,agent_maintenance_execute.rs,agent_maintenance_closeout.rs,c4_spec_ci_wiring.rs}`
- `crates/xtask/tests/support/agent_maintenance_*`
- targeted runtime or docs files touched by localized fixes
- `TODOS.md`

Exact tasks:

1. For each localized bug or doc drift fixed during the proof, add or strengthen the narrowest
   regression test that would have caught it.
2. Re-run the impacted xtask test suites plus the exact packet-owned green gates.
3. Append a dated evidence note under the existing transport-topology TODO in `TODOS.md` with:
   - execution date
   - target version
   - watcher run URL
   - worker run URL
   - verdict
   - evidence summary
4. Write the final topology decision using this rule:

   **Keep current split topology for now** if all proof failures were `host_env`, `local_bug`,
   `doc_drift`, or `target_specific`, and all required fixes stayed localized.

   **Open transport-topology convergence next** only if one or more of these are true:

   - the shared watcher and downstream worker were individually correct, but the split between
     worker and local relay still forced repeated undocumented handoffs
   - the same failure pattern would hit both `codex` and `claude_code` because it lives in the
     shared watch -> packet -> relay architecture, not in target-specific acquisition
   - the smallest honest fix would require a new workflow family, a new schema, or a cross-module
     redesign larger than a boilable localized repair

5. If the decision is "open topology convergence next", record the trigger evidence explicitly.
   No vague "felt awkward" rationale.

Exit criteria:

1. The milestone exits with a falsifiable decision and regression coverage, not just notes.

## Architecture Review

This plan deliberately spends zero innovation tokens on new orchestration.

1. The repo already has the right boundary objects:
   - shared watcher
   - downstream worker
   - generated packet
   - local relay
   - manual closeout
2. The proof should validate those boundaries, not hide them behind a proof helper.
3. The key architectural discipline is separating these failure classes:
   - worker transport failure
   - packet generation failure
   - local host failure
   - relay contract failure
   - topology failure
4. The only legitimate reason to spend an innovation token on a universal worker cutover is if
   the proof shows the current boundary split itself is the bug.
5. Distribution architecture is unchanged. No new binary, package, or pipeline is introduced by
   this milestone.

## Code Quality Review

The code-quality target is "one state transition, one owner, one repair path."

1. Do not add a proof-only command when the existing commands already represent the state machine
   correctly.
2. Do not hand-edit generated packet docs to rescue the proof. If the packet is wrong, fix the
   generator or the shared source of truth.
3. Do not bury the topology decision in freeform prose. The closeout plus `TODOS.md` should make
   the outcome obvious to the next maintainer.
4. Keep local repairs explicit. A 20-line fix in the right existing module beats a new
   abstraction every time.
5. If the proof exposes a bug in packet wording or ownership, update the matching regression test
   in the same change. Stale truth gaps love to come back.
6. If Phase 4 shows the packet and worker disagree on the same field, treat that as a generator or
   ownership bug first, not as a docs cleanup task.

## Test Review

100 percent relevant path coverage is the goal for this milestone. The proof is only trustworthy
if every new or newly stressed codepath has a corresponding command, artifact, or regression lock.

### Code Path Coverage

```text
CODE PATH COVERAGE
===========================
[+] Shared queue truth
    ├── [REQUIRED] cargo run -p xtask -- maintenance-watch --check
    ├── [REQUIRED] cargo run -p xtask -- maintenance-watch --emit-json _ci_tmp/maintenance-watch.json
    └── [LOCK]     cargo test -p xtask --test agent_maintenance_watch

[+] Shared workflow dispatch
    ├── [REQUIRED] manual trigger of agent-maintenance-release-watch.yml
    ├── [REQUIRED] downstream dispatch to codex-cli-update-snapshot.yml
    └── [LOCK]     cargo test -p xtask --test c4_spec_ci_wiring

[+] Downstream codex worker
    ├── [REQUIRED] pinned artifact prep
    ├── [REQUIRED] snapshot -> union -> report -> validate
    ├── [REQUIRED] prepare-agent-maintenance --write
    ├── [REQUIRED] PR open or documented replay failure
    └── [LOCK]     targeted workflow + prepare tests if localized fixes land

[+] Generated maintenance packet
    ├── [REQUIRED] HANDOFF.md canonical truth
    ├── [REQUIRED] maintenance-request.toml frozen relay contract
    ├── [REQUIRED] pr-summary.md derivative truth
    └── [LOCK]     cargo test -p xtask --test agent_maintenance_prepare

[+] Local relay
    ├── [REQUIRED] execute-agent-maintenance --dry-run
    ├── [REQUIRED] execute-agent-maintenance --write --run-id <prepared_run_id>
    └── [LOCK]     cargo test -p xtask --test agent_maintenance_execute

[+] Manual closeout
    ├── [REQUIRED] maintenance-closeout.json truth
    ├── [REQUIRED] close-agent-maintenance
    └── [LOCK]     cargo test -p xtask --test agent_maintenance_closeout
```

### Operator Flow Coverage

```text
OPERATOR FLOW COVERAGE
===========================
[+] "Is codex actually stale?"
    ├── [REQUIRED] local maintenance-watch queue
    └── [FAIL]     shared queue cannot identify codex without code-reading

[+] "Did the shared watcher dispatch the right lane?"
    ├── [REQUIRED] shared watcher run URL
    ├── [REQUIRED] downstream worker run URL
    └── [FAIL]     maintainer must trigger codex worker manually

[+] "What do I do next?"
    ├── [REQUIRED] generated HANDOFF.md / maintenance-request.toml / pr-summary.md
    └── [FAIL]     packet surfaces disagree or omit a step

[+] "Can I trust local execution?"
    ├── [REQUIRED] execute-agent-maintenance --dry-run
    ├── [REQUIRED] execute-agent-maintenance --write
    └── [FAIL]     undocumented setup or write-envelope surprises

[+] "How do I recover if PR open failed?"
    ├── [REQUIRED] refresh-agent --request ... --write replay path
    └── [FAIL]     packet regeneration or PR reopen requires improvisation

[+] "Is this a local bug or a topology problem?"
    ├── [REQUIRED] explicit failure classification
    └── [FAIL]     ambiguity remains after the proof
```

### Required Test Commands

Run at minimum:

```bash
cargo run -p xtask -- maintenance-watch --check
cargo run -p xtask -- maintenance-watch --emit-json _ci_tmp/maintenance-watch.json
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test c4_spec_ci_wiring
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_execute
cargo test -p xtask --test agent_maintenance_closeout
```

If localized code fixes land during the proof, also run the impacted suite plus the packet-owned
green gates from the generated request:

```bash
cargo run -p xtask -- support-matrix --check
cargo run -p xtask -- capability-matrix --check
cargo run -p xtask -- capability-matrix-audit
make preflight
```

### Regression Rule For This Milestone

If the proof reveals a bug in a currently documented step that previously claimed to work, add a
regression test in the same milestone. No exceptions. A proof run that finds a hole but does not
lock it is just a better bug report.

## Failure Modes Registry

| Failure mode | Surface | Test coverage required | Handling required | Silent if missed? | Status |
| --- | --- | --- | --- | --- | --- |
| Local queue does not identify `codex` correctly | `maintenance-watch`, registry | `agent_maintenance_watch` | fix registry or queue logic before live run | yes | must close |
| Shared watcher dispatches wrong worker or wrong branch | shared watcher, registry, workflow inputs | `c4_spec_ci_wiring`, live run evidence | fix shared dispatch truth | yes | must close |
| Worker generates packet surfaces that disagree with each other | generated maintenance root | `agent_maintenance_prepare` | fix packet generation or source-of-truth inputs | yes | must close |
| Local relay requires undocumented setup beyond the written host-repair path | `execute-agent-maintenance`, playbooks, operator guide | `agent_maintenance_execute` + live proof | fix docs or relay validation wording/behavior | yes | must close |
| PR-open recovery requires artifact surgery beyond `refresh-agent --request ... --write` | worker failure path, packet recovery notes | workflow replay proof + targeted tests | fix recovery contract or workflow fallback | yes | must close |
| Closeout cannot truthfully express the result | `maintenance-closeout.json`, `close-agent-maintenance` | `agent_maintenance_closeout` | fix closeout schema/rendering or instructions | yes | must close |
| A bug looks like topology pain only because `codex` is special | `codex` worker or wrapper specifics | targeted `codex` tests + optional `claude_code` control check | classify as target-specific, not topology-wide | no | should close |
| The same undocumented handoff exists even when all local bugs are fixed | watch -> worker -> packet -> relay boundary | live proof + written decision rule | escalate to transport-topology follow-up | yes | topology trigger |

## Performance Review

This is not a throughput project. The performance goal is decision latency.

1. The maintainer should get to "local bug vs topology issue" quickly, without spelunking.
2. Avoid any fix that adds extra round-trips, extra workflow hops, or duplicate packet
   materialization steps just to make the proof look cleaner.
3. If a localized bug fix requires touching more than one shared control-plane module and one
   worker-specific module, re-check whether that is actually a topology signal.
4. The fastest system is the one that fails clearly. Silent ambiguity is the slow path.

## NOT In Scope

1. A universal worker cutover in this milestone.
2. A second proving target in the same milestone unless `codex` reveals a target-specific
   ambiguity that requires `claude_code` as a control check.
3. Create-mode lifecycle proof for `goose`.
4. New workflow families, new dispatch kinds, or a proof-only command.
5. Automatic maintenance closeout.
6. Rewriting the maintenance packet contract unless the proof finds a localized contract bug.
7. Any new publication or onboarding distribution surface.
8. General architecture cleanup that is not required to complete or classify the proof.

## Worktree Parallelization Strategy

Baseline: mostly sequential until the proof identifies disjoint fixes.

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| 1. Freeze proof rubric + prerequisite lock | `PLAN.md`, `TODOS.md`, `crates/xtask/data/`, workflows, specs, operator docs | — |
| 2. Local queue preflight | `crates/xtask/src/agent_maintenance/watch.rs`, `crates/xtask/data/` | 1 |
| 3. Shared watcher + downstream worker proof | workflows, GitHub Actions runs, `docs/agents/lifecycle/codex-maintenance/**` | 2 |
| 4. Local relay + closeout proof | `docs/agents/lifecycle/codex-maintenance/**`, `crates/xtask/src/agent_maintenance/execute*`, `crates/xtask/src/agent_maintenance/closeout*` | 3 |
| 5A. Shared control-plane fixes (conditional) | `crates/xtask/src/agent_maintenance/**`, workflows, specs | 4 |
| 5B. `codex`-specific fixes (conditional) | `crates/codex/**`, `cli_manifests/codex/**` | 4 |
| 5C. Docs/tests lock-in (conditional) | `docs/**`, `crates/xtask/tests/**` | 5A and/or 5B if behavior changed |
| 6. Final validation + written topology decision | repo-wide validation, `TODOS.md`, closeout surfaces | 5A, 5B, 5C |

### Parallel Lanes

Lane A: Steps 1 -> 2 -> 3 -> 4  
This is the critical path. The proof itself is sequential.

Lane B: Step 5A  
Shared control-plane repair lane. Launch only if the proof finds a shared watcher, packet, or
relay bug.

Lane C: Step 5B  
`codex`-specific repair lane. Launch only if the proof finds a target-specific worker or wrapper
bug that does not overlap shared control-plane modules.

Lane D: Step 5C -> 6  
Docs/tests lock-in lane. Start after the behavior-changing lanes settle, then run final
validation and record the topology decision.

### Execution Order

1. Launch Lane A alone.
2. If Lane A proves the current topology cleanly, skip B/C and move straight to Lane D step 6.
3. If Lane A finds disjoint localized bugs:
   - launch B and C in parallel worktrees
   - merge both
   - run D
4. If Lane A finds a true topology issue, do not launch B/C as a disguised redesign. Move
   straight to the written follow-up decision.

### Conflict Flags

1. `docs/agents/lifecycle/codex-maintenance/**` is shared by the worker proof, the relay proof,
   and closeout. Do not parallel-edit that root casually.
2. `crates/xtask/tests/**` usually locks strings or packet behavior from shared control-plane
   code. Let behavior settle before updating test expectations.
3. If a shared control-plane fix changes packet wording, docs/tests lock-in must happen after that
   wording is final.

## Completion Summary

- Step 0: Scope Challenge, accepted with `codex` locked as the first proving target
- Architecture Review: no new infra recommended, prove the existing state machine instead
- Code Quality Review: one owner per state transition, no proof-only abstractions
- Test Review: 6 critical proof checkpoints identified and mapped to commands/tests
- Failure modes: 8 concrete proof risks identified, 6 must close, 1 should close, and 1 is the topology trigger
- NOT in scope: written
- Parallelization: conditional, mostly sequential until proof findings split cleanly
- Topology decision rule: written and explicit

## Exit Criteria

This plan is done when:

1. `codex` is proven or truthfully blocked from the shared watcher entrypoint onward
2. the downstream worker and generated packet are exercised from real shared dispatch
3. the local relay dry-run and write path are exercised from the generated packet
4. recovery is either proven or truthfully flagged as insufficient
5. maintenance closeout is recorded through the existing closeout path
6. localized bugs found during the proof are covered by regression tests
7. `TODOS.md` contains a written keep-vs-cutover decision backed by concrete evidence

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Step 0 | Lock `codex` as the first proving target. | Mechanical | P3 pragmatic | It is the cleanest enrolled maintenance target with committed packet surfaces. | Starting with `goose` or a two-target proof |
| 2 | Step 0 | Start at the shared watcher, not the downstream worker. | Mechanical | P1 completeness | The shared watcher is the seam we actually need to prove. | Direct worker trigger as the opening move |
| 3 | Step 0 | Add an explicit topology decision rule before the live run. | Mechanical | P5 explicit | Without it, the proof produces feelings instead of a decision. | Post-hoc interpretation |
| 4 | Phase 1 | Treat remote `staging` alignment as a hard prerequisite. | Mechanical | P5 explicit | The workflows check out `origin/staging`, not the local worktree. | Running the proof on stale remote state |
| 5 | Phase 2 | Require a real stale `codex` queue item at execution time. | Mechanical | P1 completeness | A fabricated queue item would not prove the live watcher contract. | Synthetic worker inputs |
| 6 | Phase 4 | Reuse the existing watch -> worker -> packet -> relay -> closeout state machine. | Mechanical | P4 DRY | The repo already has the right boundary objects. | A proof-only command or workflow |
| 7 | Phase 4 | Treat local host failures as evidence, but not topology failures by default. | Mechanical | P3 pragmatic | Bad local auth should not force a transport redesign. | Collapsing all failures into topology pain |
| 8 | Phase 6 | Allow localized repairs inside the milestone only if the fix stays boilable. | Taste | P2 boil lakes | Small real bugs should be fixed while the context is hot. | Leaving known local bugs unfixed or starting a redesign |
| 9 | Phase 6 | Use the existing maintenance closeout JSON as the final evidence sink. | Mechanical | P5 explicit | One closeout format is better than a second proof artifact family. | A separate proof report schema |
| 10 | Phase 6 | Record the final keep-vs-cutover decision in `TODOS.md`. | Mechanical | P5 explicit | Future planning already lives there; the decision must be easy to find. | Hiding the outcome in conversational history |
| 11 | Phase 6 | Use `claude_code` only as a secondary control target if `codex` reveals target-specific ambiguity. | Taste | P3 pragmatic | Two proving targets up front would dilute the milestone. | Mandatory two-target proof |
| 12 | Phase 6 | Open transport-topology convergence only when the proof demonstrates a shared architectural failure. | User Challenge | P3 pragmatic | Duplicate YAML alone is not enough reason to redesign. | Cutover-by-aesthetic-discomfort |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 0 | — | — |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | — | — |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 0 | — | — |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | — | — |

**VERDICT:** NO REVIEWS YET — run `/autoplan` or the individual review skills on this plan if you want the review pipeline recorded after this cohesion pass.
