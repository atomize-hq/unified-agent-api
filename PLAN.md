# PLAN - First Honest Maintenance Proof Run

Status: ready for implementation  
Date: 2026-05-10  
Branch: `staging`  
Base branch: `main`  
Repo: `atomize-hq/unified-agent-api`  
Work item: `Prove the current maintenance topology end to end before any transport-topology redesign`  
Plan commit baseline: `ee8249a`  
Design input: `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-design-20260510-200245.md`  
Supersedes: the prior repo-root `PLAN.md` for `worker/runbook convergence on the shared packet contract`

## Executive Summary

The repo just finished the two milestones that made maintenance truthful on paper:

1. packet-first contract convergence
2. worker/runbook convergence on top of that shared packet contract

The next honest step is not another cleanup pass and not a universal workflow cutover. The next
step is to prove that the current topology works end to end for a real maintained agent without
tribal knowledge.

This plan makes that proof run explicit. It locks the first proving target to `codex`, requires
the run to start at the shared release-watch entrypoint, requires the downstream worker, packet,
local relay, recovery path, and manual closeout to be exercised in order, and records a written
decision at the end:

- keep the current split worker topology for now, with only localized fixes, or
- open a separate transport-topology convergence milestone next

That is the whole game. Prove first. Redesign second, only if the proof earns it.

## Objective

Run one decision-bearing maintenance proof pass for `codex` that starts at the shared watcher,
flows through the real downstream worker and generated packet, exercises the local relay and
manual closeout path, and exits with a written topology decision based on evidence instead of
architecture taste.

## Success Criteria

1. The proof begins with a manual trigger of `.github/workflows/agent-maintenance-release-watch.yml`,
   not a direct trigger of `codex-cli-update-snapshot.yml`.
2. The shared watcher emits or proves the expected stale-agent queue entry for `codex`, including
   the canonical branch name `automation/codex-maintenance-<target_version>`.
3. The downstream `codex-cli-update-snapshot.yml` worker runs from shared watcher dispatch and
   produces the expected maintenance packet surfaces under `docs/agents/lifecycle/codex-maintenance/**`.
4. The maintainer can follow the generated packet and run:
   - `cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --dry-run`
   - `cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --write --run-id <prepared_run_id>`
   without undocumented operator steps.
5. If PR creation fails after packet generation, the documented `refresh-agent --request ... --write`
   recovery path is sufficient to reopen the PR from the generated summary without inventing a new
   manual lane.
6. The run closes through `maintenance-closeout.json` plus `close-agent-maintenance`, and the
   closeout truth is explicit about resolved findings, deferred findings, and whether repo
   preflight passed.
7. The milestone exits with one written decision, backed by proof evidence:
   - keep the split `codex` / `claude_code` worker topology for now, or
   - open a separate transport-topology convergence follow-up next

## Step 0 Scope Challenge

### Premise Challenge

| Premise | Assessment | Decision |
| --- | --- | --- |
| The current maintenance topology is now truthful enough to be tested directly. | Accepted. The watcher, packet contract, worker/runbook language, relay contract, and closeout surfaces already align far better than they did before. | Prove this topology before redesigning it. |
| The cleanest first proving target is `codex`. | Accepted. `codex` is enrolled in maintenance, has a committed generated maintenance root, uses GitHub releases, and keeps the first proof focused. | Lock `codex` as the first proof target. |
| A proof run that starts at the downstream worker is good enough. | Rejected. That skips the exact shared watcher seam the last milestone was trying to make truthful. | Start at `agent-maintenance-release-watch.yml`. |
| The next milestone should also decide or implement a universal worker cutover. | Rejected. That mixes validation with redesign and makes failures ambiguous. | Defer transport redesign unless the proof run earns it. |
| Create-mode lifecycle proof for `goose` is the right next proving step here. | Rejected for this milestone. `goose` remains a separate create-lane proving target in `TODOS.md`; this milestone is about maintenance topology truth for already enrolled agents. | Keep `goose` out of scope here. |
| A proof run without an explicit exit rule is enough. | Rejected. That creates a ceremonial run with no decision value. | Use a predeclared topology-decision rule. |

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| Shared release detection and dispatch | `.github/workflows/agent-maintenance-release-watch.yml`, `crates/xtask/src/agent_maintenance/watch.rs`, `crates/xtask/data/agent_registry.toml` | Reuse exactly. The proof starts here. |
| `codex` maintenance worker | `.github/workflows/codex-cli-update-snapshot.yml` | Reuse exactly. Do not fork a proof-only worker. |
| Packet generation | `prepare-agent-maintenance`, `docs/agents/lifecycle/codex-maintenance/**`, `docs/specs/maintenance-request-contract-v1.md` | Reuse exactly. Generated packet surfaces are part of the proof evidence. |
| Local relay execution | `execute-agent-maintenance`, `cli_manifests/codex/OPS_PLAYBOOK.md`, `docs/cli-agent-onboarding-factory-operator-guide.md` | Reuse exactly. The proof must exercise this lane, not describe it abstractly. |
| Maintenance closeout | `close-agent-maintenance`, `docs/agents/lifecycle/*-maintenance/governance/maintenance-closeout.json` | Reuse exactly. No proof-specific closeout format. |
| Recovery path after PR-open failure | generated `HANDOFF.md`, generated `pr-summary.md`, `refresh-agent --request ... --write` | Reuse exactly. This path must be proven or truthfully flagged. |
| Deferred topology redesign question | `TODOS.md` | Reuse. Record the final decision there instead of inventing a new planning surface. |

### Existing Code Leverage Map

| Need | Best leverage point |
| --- | --- |
| Confirm enrolled agent truth | `crates/xtask/data/agent_registry.toml` |
| Prove watcher queue contents locally before spending Actions time | `cargo run -p xtask -- maintenance-watch --check` and `--emit-json` |
| Prove shared dispatch path | `.github/workflows/agent-maintenance-release-watch.yml` |
| Prove worker-specific acquisition + packet prep | `.github/workflows/codex-cli-update-snapshot.yml` |
| Prove generated packet truth | `docs/agents/lifecycle/codex-maintenance/{HANDOFF.md,governance/maintenance-request.toml,governance/pr-summary.md}` |
| Prove relay trust step and write envelope | `execute-agent-maintenance` |
| Prove manual closeout | `maintenance-closeout.json` + `close-agent-maintenance` |
| Record future topology decision | `TODOS.md` |

### Dream State

```text
CURRENT
  the maintenance topology is finally believable on paper
  but it has not yet been proven from shared watcher through manual closeout

THIS PLAN
  shared watcher -> codex worker -> generated packet -> local relay -> manual closeout
  all exercised end to end on one real maintenance lane
  every failure classified as host-env, local bug, doc drift, target-specific issue, or topology issue
  topology follow-up decision written from evidence

12-MONTH IDEAL
  any enrolled agent can enter maintenance from one shared watch contract,
  land through a boring downstream worker path,
  and tell maintainers exactly what to do next without archaeology
```

### Proof Scorecard

The run is decision-bearing only if it captures these six checkpoints:

| Checkpoint | Required evidence | Fails if |
| --- | --- | --- |
| Release-watch truth | local `maintenance-watch` queue + live shared watcher run | shared queue and live dispatch disagree or cannot name `codex` truthfully |
| Dispatch truth | downstream workflow run, branch name, and input payload | wrong worker, wrong branch, wrong versions, or undocumented manual dispatch correction |
| Packet truth | generated `maintenance-request.toml`, `HANDOFF.md`, and `pr-summary.md` | packet surfaces disagree with each other or with the run inputs |
| Relay truth | `execute-agent-maintenance --dry-run` and `--write` | maintainer needs undocumented steps or the relay violates the declared write/gate contract |
| Recovery truth | replay via `refresh-agent --request ... --write` if PR creation fails | recovery requires an unplanned lane or manual artifact surgery |
| Closeout truth | `maintenance-closeout.json` and `close-agent-maintenance` | the run cannot be closed honestly with resolved/deferred findings and preflight status |

### Implementation Alternatives

| Approach | Effort | Risk | Pros | Cons | Decision |
| --- | --- | --- | --- | --- | --- |
| Pure proof run, no explicit exit rule | M | Medium | Fastest to start | Ends in vibes instead of a decision | Reject |
| Proof-first with explicit topology exit criteria | M | Low | Keeps the milestone evidence-driven and decision-bearing | Slightly more upfront planning | Recommend |
| Universal workflow cutover first | L | High | Might reduce duplicate YAML | Redesign and validation become one ambiguous pile | Defer |

### Mode Selection

`PROOF_FIRST_WITH_LOCALIZED_REPAIR`

Meaning:

1. Prove the current topology as it exists today.
2. Fix only localized bugs, doc drift, or target-specific issues discovered during the proof.
3. Do not widen into a universal worker redesign inside this milestone.
4. Exit with a written keep-vs-cutover decision.

### Temporal Interrogation

**Hour 1**

Can a maintainer prove locally that `codex` is the stale queued agent and name the exact shared
watcher entrypoint, downstream worker, and branch family without reading code?

**Hour 6**

Can a maintainer trigger the shared watcher, follow the generated packet, and complete relay
dry-run plus write mode without undocumented steps?

**Day 1**

If the proof finds a failure, can the repo classify it cleanly as:

- host environment issue
- localized maintenance bug
- doc drift
- target-specific integration issue
- topology issue

without arguing from aesthetics?

**Week 1**

Did this milestone either:

1. prove the current split worker topology is fine for now, or
2. earn a focused transport-topology follow-up with concrete evidence?

If not, the proof was too fuzzy.

### Minimum Complete Change

The minimum complete milestone is:

1. freeze the proving target, scorecard, and topology exit rule
2. locally preflight the shared watcher queue before spending live Actions time
3. manually trigger the shared watcher and capture the real downstream worker evidence
4. consume the generated packet and exercise relay dry-run plus write mode
5. prove or truthfully fail the documented recovery path when PR opening fails
6. record maintenance closeout truth and a written topology decision
7. lock any discovered localized bugs with regression tests before declaring the proof done

Anything smaller is a demo, not a proof.

### Complexity Check

This plan should not introduce new infrastructure.

Smells that would mean we are overbuilding:

- a new proof-only xtask command
- a new workflow family just for proving
- a new packet schema or second closeout format
- a second proving target in the same milestone
- a universal worker redesign starting before the first proof finishes

The intended implementation is mostly existing commands, existing packet surfaces, and only the
smallest possible code fixes if the proof finds a real gap.

### Search / Build Decision

- **[Layer 1]** Reuse the existing shared watcher, downstream worker, packet generator, relay, and
  closeout commands.
- **[Layer 1]** Reuse the existing maintenance packet surfaces and closeout JSON instead of
  inventing a proof-run artifact family.
- **[Layer 1]** Reuse `TODOS.md` for the topology follow-up decision.
- **[Layer 3]** Treat the topology-decision rule as a repo-specific first-principles safeguard:
  prove the current system before changing it.

### Distribution Check

No new external artifact is introduced.

This is an internal factory milestone. Its outputs are:

- shared watcher and downstream worker run evidence
- generated `codex` maintenance packet surfaces
- temp relay proof evidence under `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`
- `docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json`
- a written keep-vs-cutover decision in `TODOS.md`

## Locked Decisions

1. The first proof target is `codex`, not `claude_code` and not `goose`.
2. The proof must start at `.github/workflows/agent-maintenance-release-watch.yml`.
3. Directly triggering `codex-cli-update-snapshot.yml` is not a valid opening move for this
   milestone.
4. This is a maintenance-mode proof, not a create-mode lifecycle proof.
5. The milestone reuses the current packet contract, relay contract, and closeout format. No new
   proof-only schema is allowed.
6. Local host issues such as missing Codex auth or a broken local binary are recorded as proof
   evidence, but they count as topology failures only if the documented host-repair path itself is
   unclear or broken.
7. Localized bugs found during the proof are fixed in this milestone only if they stay within the
   current topology and the blast radius is still boilable.
8. A universal worker cutover becomes eligible only if the proof hits the predeclared topology
   trigger rule below.
9. `claude_code` is a secondary control target only if the `codex` proof reveals ambiguity that
   looks target-specific rather than topology-wide.

## Architecture

### Current Topology

```text
agent_registry.toml
  -> release-watch truth
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
local preflight
  cargo run -p xtask -- maintenance-watch --check
  cargo run -p xtask -- maintenance-watch --emit-json _ci_tmp/maintenance-watch.json
        |
        v
shared watcher manual trigger
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
maintainer proof
  review HANDOFF.md / request / pr-summary
  execute-agent-maintenance --dry-run
  execute-agent-maintenance --write --run-id <prepared_run_id>
        |
        v
optional localized repair loop
  fix local bug/doc drift
  rerun only the impacted proof segment
        |
        v
manual closeout + decision
  maintenance-closeout.json
  close-agent-maintenance
  write keep-vs-cutover outcome to TODOS.md
```

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

This milestone is only complete if these artifacts exist and are cited in the closeout decision:

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
| Final topology decision | dated note in `TODOS.md` |

### Ownership Map

| Surface | Owner | Consumers | Rule |
| --- | --- | --- | --- |
| stale-agent truth | `agent_registry.toml` + `maintenance-watch` | shared watcher, downstream worker, maintainer | one source of truth |
| transport entrypoint | `agent-maintenance-release-watch.yml` | GitHub Actions, maintainers | start here, not downstream |
| downstream acquisition | `codex-cli-update-snapshot.yml` | automation lane | transport only, no second policy store |
| packet truth | `prepare-agent-maintenance` output | maintainers, relay, PR body | packet remains canonical |
| local relay truth | `execute-agent-maintenance` | maintainers | trust packet-owned write envelope and gates |
| closeout truth | `maintenance-closeout.json` + `close-agent-maintenance` | maintainers, future reviews | one explicit maintenance closeout path |
| redesign follow-up | `TODOS.md` | future planning | no silent topology drift |

## Implementation Plan

### Phase 1. Freeze The Proof Contract

Purpose: remove ambiguity before any run starts.

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
2. Freeze the topology-decision rule in this plan before any proof evidence is collected.
3. Record the exact proof evidence checklist so later closeout is not hand-wavy.
4. Do not create any proof-only workflow, proof-only command, or proof-only artifact type.

Proof:

1. The proving target, entrypoint, evidence packet, and exit rule are written before the live run.
2. Everyone can answer "what counts as topology failure?" from the plan alone.

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

2. Confirm the queue either contains `codex` as stale or truthfully explains why no stale agent is
   currently available for proof.
3. Record:
   - `agent_id`
   - `current_validated`
   - `latest_stable`
   - `target_version`
   - `dispatch_workflow`
   - `branch_name`
4. If the queue does not produce a codex proof target, stop and fix that localized issue before
   touching live workflows. Do not work around it by manually inventing worker inputs.

Proof:

1. Local queue truth matches registry expectations.
2. The proof can name the exact branch and downstream worker before GitHub dispatch.

### Phase 3. Trigger The Shared Watcher And Capture Dispatch Evidence

Purpose: prove the real entrypoint and real downstream fanout.

Primary surfaces:

- `.github/workflows/agent-maintenance-release-watch.yml`
- `.github/workflows/codex-cli-update-snapshot.yml`
- GitHub Actions run metadata

Exact tasks:

1. Manually trigger the shared watcher from `staging`:

   ```bash
   gh workflow run agent-maintenance-release-watch.yml --ref staging
   ```

   GitHub UI trigger is acceptable if CLI auth is unavailable.

2. Capture the shared watcher run URL and downstream `codex` worker run URL.
3. Verify the downstream worker inputs match the locally emitted queue:
   - same `agent_id`
   - same `current_version`
   - same `latest_stable`
   - same `target_version`
   - same `dispatch_kind`
   - same `branch_name`
4. Verify the downstream worker either:
   - opens the expected PR branch successfully, or
   - fails only at PR creation, with the documented replay path printed cleanly
5. Do not bypass this by manually running `prepare-agent-maintenance` first. The worker run must
   produce the packet as part of the proof.

Proof:

1. The shared watcher is a real working control-plane entrypoint, not just a diagram.
2. Shared queue truth survives the handoff to the worker workflow without mutation.

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
2. Run relay dry-run:

   ```bash
   cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --dry-run
   ```

3. Record the prepared `run_id` and the temp evidence directory under
   `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`.
4. Run relay write mode with the same `run_id`:

   ```bash
   cargo run -p xtask -- execute-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --write --run-id <prepared_run_id>
   ```

5. Classify any failure immediately:
   - `host_env`: local Codex binary/auth/setup problem, packet and docs are clear
   - `local_bug`: code or contract bug inside the existing topology
   - `doc_drift`: instructions are wrong or incomplete
   - `target_specific`: `codex`-specific issue that does not imply topology redesign
   - `topology_issue`: the split watch -> worker -> relay model itself is the problem
6. If the failure is `host_env`, repair only through the documented host-repair path and rerun
   the same proof segment.
7. If the failure is `local_bug`, `doc_drift`, or `target_specific`, fix it in this milestone if
   the fix is localized and the blast radius remains boilable.
8. If the failure is `topology_issue`, stop widening scope. Record it and move to the decision
   phase. Do not start a universal workflow cutover here.

Proof:

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

1. If PR creation failed during the worker phase, use the documented replay path exactly:

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

Proof:

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
3. Write the final topology decision to `TODOS.md` using this rule:

   **Keep current split topology for now** if all proof failures were `host_env`, `local_bug`,
   `doc_drift`, or `target_specific`, and all required fixes stayed localized.

   **Open transport-topology convergence next** only if one or more of these are true:

   - the shared watcher and downstream worker were individually correct, but the split between
     worker and local relay still forced repeated undocumented handoffs
   - the same failure pattern would hit both `codex` and `claude_code` because it lives in the
     shared watch -> packet -> relay architecture, not in target-specific acquisition
   - the smallest honest fix would require a new workflow family, a new schema, or a cross-module
     redesign larger than a boilable localized repair
4. If the decision is "open topology convergence next", record the trigger evidence explicitly.
   No vague "felt awkward" rationale.

Proof:

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
3. The key architecture discipline is separating these failure classes:
   - worker transport failure
   - packet generation failure
   - local host failure
   - relay contract failure
   - topology failure
4. The only legitimate reason to spend an innovation token on a universal worker cutover is if
   the proof shows the current boundary split itself is the bug.

## Code Quality Review

The code-quality target is "one state transition, one owner, one repair path."

1. Do not add a proof-only command when the existing commands already represent the state machine
   correctly.
2. Do not hand-edit generated packet docs to rescue the proof. If the packet is wrong, fix the
   generator or the shared source of truth.
3. Do not bury the topology decision in freeform prose. The closeout plus `TODOS.md` should make
   the outcome obvious to the next maintainer.
4. Keep local repairs explicit. A 20-line fix in the right existing module beats a new abstraction
   every time.
5. If the proof exposes a bug in packet wording or ownership, update the matching regression test
   in the same change. Stale truth gaps love to come back.

## Test Review

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

### Maintainer Flow Coverage

```text
USER FLOW COVERAGE
===========================
[+] Maintainer asks "is codex actually stale?"
    ├── [REQUIRED] local maintenance-watch queue
    └── [FAIL]     shared queue cannot identify codex without code-reading

[+] Maintainer asks "did the shared watcher really dispatch the right lane?"
    ├── [REQUIRED] shared watcher run URL
    ├── [REQUIRED] downstream worker run URL
    └── [FAIL]     maintainer must trigger codex worker manually

[+] Maintainer asks "what do I do next?"
    ├── [REQUIRED] generated HANDOFF.md / maintenance-request.toml / pr-summary.md
    └── [FAIL]     packet surfaces disagree or omit a step

[+] Maintainer asks "can I trust local execution?"
    ├── [REQUIRED] execute-agent-maintenance --dry-run
    ├── [REQUIRED] execute-agent-maintenance --write
    └── [FAIL]     undocumented setup or write-envelope surprises

[+] Maintainer asks "how do I recover if PR open failed?"
    ├── [REQUIRED] refresh-agent --request ... --write replay path
    └── [FAIL]     packet regeneration or PR reopen requires improvisation

[+] Maintainer asks "is this a local bug or a topology problem?"
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

## Performance Review

This is not a throughput project. The performance goal is decision latency.

1. The maintainer should get to "local bug vs topology issue" quickly, without spelunking.
2. Avoid any fix that adds extra round-trips, extra workflow hops, or duplicate packet materialize
   steps just to make the proof look cleaner.
3. If a localized bug fix requires touching more than one shared control-plane module and one
   worker-specific module, re-check whether that is actually a topology signal.
4. The fastest system is the one that fails clearly. Silent ambiguity is the slow path.

## DX Review

### Developer Journey Map

| Stage | What the maintainer tries to do | Current friction | Target after this milestone |
| --- | --- | --- | --- |
| 1. Find the real entrypoint | Decide which workflow to trigger first | Easy to bypass the shared watcher by habit | Shared watcher is mandatory and proven |
| 2. Trust the stale queue | Confirm `codex` is really the target | Today it is truthful in code, but not yet proven live | Local queue plus live dispatch agree |
| 3. Trust the downstream worker | Confirm the correct worker ran | Worker truth exists, but only live evidence proves it | Shared watcher -> codex worker becomes boring |
| 4. Find the packet | Know which generated file is canonical | `HANDOFF.md` is canonical, but the lane has not been fully exercised | Packet truth is proven, not inferred |
| 5. Run locally | Know what `execute-agent-maintenance` validates | Still unproven as a maintainer lane | Dry-run and write mode are exercised |
| 6. Recover from PR-open failure | Regenerate and reopen cleanly | Recovery exists on paper | Recovery is proven or truthfully flagged |
| 7. Close the run | Record resolved vs deferred findings | Manual closeout is documented, not yet proven here | Maintenance closeout becomes part of the proof |
| 8. Decide what to do next | Know whether to redesign topology | Too easy to argue from taste | Decision rule turns proof into action |
| 9. Hand off to the next maintainer | Leave a crisp answer | Today the answer is still theoretical | The repo contains evidence and a written decision |

### Developer Empathy Narrative

I got a maintenance alert. I need to know whether the system works, not whether the docs sound
confident.

If I have to guess which workflow to start with, guess whether the packet is trustworthy, guess
whether the local relay is optional, and then guess whether the failure I hit is just my laptop or
the repo's architecture, the maintenance system is still too magical in the bad way.

This milestone fixes that by forcing the whole path to earn trust. No heroics. No architecture
theater. Just one honest run.

### DX Scorecard

| Dimension | Current | Target | Notes |
| --- | --- | --- | --- |
| Entry-point clarity | 7/10 | 10/10 | Shared watcher is documented; this milestone proves it is the only honest start. |
| Queue trust | 7/10 | 9/10 | Local queue exists; live proof still missing. |
| Packet trust | 8/10 | 10/10 | Packet contract is strong; now it needs a real maintainer pass. |
| Relay ergonomics | 7/10 | 9/10 | Commands exist, but the maintainer path still needs proof. |
| Recovery clarity | 7/10 | 9/10 | Recovery wording is present; proof must show it is enough. |
| Failure classification | 4/10 | 9/10 | Today too much still becomes "maybe topology?" |
| Closeout honesty | 7/10 | 9/10 | Closeout path exists; it needs a real maintenance pass. |
| Decision-bearing follow-up | 3/10 | 10/10 | This is the main gap. We need a written keep-vs-cutover outcome. |

### TTHW Assessment

Current time-to-trustworthy-maintenance-verdict: about 30-45 minutes, mostly because the repo can
describe the lane but has not yet forced a maintainer through the full sequence.

Target time-to-trustworthy-maintenance-verdict after this milestone: under 15 minutes to say
"this is a local bug" or "this topology needs a follow-up."

### DX Implementation Checklist

- Make the shared watcher the explicit first step of the proof.
- Capture both local queue evidence and live dispatch evidence.
- Prove the packet surfaces in the order maintainers actually read them.
- Prove `execute-agent-maintenance --dry-run` before write mode.
- Prove the recovery path if PR opening fails.
- Record closeout truth using the existing maintenance closeout format.
- Exit with a written topology decision in `TODOS.md`.

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
| 1. Freeze proof rubric + local queue preflight | `PLAN.md`, `TODOS.md`, `crates/xtask/data/`, `crates/xtask/src/agent_maintenance/watch.rs` | — |
| 2. Shared watcher + downstream worker proof | workflows, GitHub Actions runs, `docs/agents/lifecycle/codex-maintenance/**` | 1 |
| 3. Local relay + closeout proof | `docs/agents/lifecycle/codex-maintenance/**`, `crates/xtask/src/agent_maintenance/execute*`, `crates/xtask/src/agent_maintenance/closeout*` | 2 |
| 4A. Shared control-plane fixes (conditional) | `crates/xtask/src/agent_maintenance/**`, workflows, specs | 3 |
| 4B. `codex`-specific fixes (conditional) | `crates/codex/**`, `cli_manifests/codex/**` | 3 |
| 4C. Docs/tests lock-in (conditional) | `docs/**`, `crates/xtask/tests/**` | 4A and/or 4B if behavior changed |
| 5. Final validation + written topology decision | repo-wide validation, `TODOS.md`, closeout surfaces | 4A, 4B, 4C |

### Parallel Lanes

Lane A: Steps 1 -> 2 -> 3  
This is the critical path. The proof itself is sequential.

Lane B: Step 4A  
Shared control-plane repair lane. Launch only if the proof finds a shared watcher / packet /
relay bug.

Lane C: Step 4B  
`codex`-specific repair lane. Launch only if the proof finds a target-specific worker or wrapper
bug that does not overlap shared control-plane modules.

Lane D: Step 4C -> 5  
Docs/tests lock-in lane. Start after the behavior-changing lanes settle, then run final validation
and record the topology decision.

### Execution Order

1. Launch Lane A alone.
2. If Lane A proves the current topology cleanly, skip B/C and move straight to Lane D step 5.
3. If Lane A finds disjoint localized bugs:
   - launch B and C in parallel worktrees
   - merge both
   - run D
4. If Lane A finds a true topology issue, do not launch B/C as a disguised redesign. Move straight
   to the written follow-up decision.

### Conflict Flags

1. `docs/agents/lifecycle/codex-maintenance/**` is shared by the worker proof, the relay proof,
   and closeout. Do not parallel-edit that root casually.
2. `crates/xtask/tests/**` usually locks strings or packet behavior from shared control-plane
   code. Let behavior settle before updating test expectations.
3. If a shared control-plane fix changes packet wording, docs/tests lock-in must happen after that
   wording is final.

## Cross-Phase Themes

**Theme: prove before redesign.**

The repo has already spent the hard work making the current topology truthful. The responsible next
step is to force that truth through a real maintenance run.

**Theme: classify failures, do not romanticize them.**

The milestone only works if failures come back as one of five crisp classes:
host-env, local bug, doc drift, target-specific issue, or topology issue.

**Theme: reuse the real state machine.**

If the proof requires a proof-only helper, the state machine is either wrong or the proof is
cheating.

## Follow-Up Question

After this milestone, the next explicit question is:

`transport-topology convergence review`

Open it only if the proof shows that the split between shared watcher, downstream worker, and
local relay is itself the source of repeated undocumented handoffs or requires a redesign larger
than a boilable localized repair.

If the proof does not cross that bar, keep the split topology, fix the local bugs, and move on.

## Completion Summary

- Step 0: Scope Challenge, accepted with codex locked as the first proving target
- Architecture Review: no new infra recommended, prove the existing state machine instead
- Code Quality Review: one owner per state transition, no proof-only abstractions
- Test Review: 6 critical proof checkpoints identified and mapped to commands/tests
- DX Review: current trust path strong on paper but still unproven end to end
- Failure modes: 8 concrete proof risks identified, 6 silent if missed
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
| 1 | CEO | Lock `codex` as the first proving target. | Mechanical | P3 pragmatic | It is the cleanest enrolled maintenance target with committed packet surfaces. | Starting with `goose` or a two-target proof |
| 2 | CEO | Start at the shared watcher, not the downstream worker. | Mechanical | P1 completeness | The shared watcher is the seam we actually need to prove. | Direct worker trigger as the opening move |
| 3 | CEO | Add an explicit topology-decision rule before the live run. | Mechanical | P5 explicit | Without it, the proof produces feelings instead of a decision. | Post-hoc interpretation |
| 4 | Eng | Reuse the existing watch -> worker -> packet -> relay -> closeout state machine. | Mechanical | P4 DRY | The repo already has the right boundary objects. | A proof-only command or workflow |
| 5 | Eng | Treat local host failures as evidence, but not topology failures by default. | Mechanical | P3 pragmatic | Bad local auth should not force a transport redesign. | Collapsing all failures into topology pain |
| 6 | Eng | Allow localized repairs inside the milestone only if the fix stays boilable. | Taste | P2 boil lakes | Small real bugs should be fixed while the context is hot. | Leaving known local bugs unfixed or starting a redesign |
| 7 | DX | Use the existing maintenance closeout JSON as the final evidence sink. | Mechanical | P5 explicit | One closeout format is better than a second proof artifact family. | A separate proof report schema |
| 8 | DX | Record the final keep-vs-cutover decision in `TODOS.md`. | Mechanical | P5 explicit | Future planning already lives there; the decision must be easy to find. | Hiding the outcome in conversational history |
| 9 | Eng | Use `claude_code` only as a secondary control target if `codex` reveals target-specific ambiguity. | Taste | P3 pragmatic | Two proving targets up front would dilute the milestone. | Mandatory two-target proof |
| 10 | CEO | Open transport-topology convergence only when the proof demonstrates a shared architectural failure. | User Challenge | P3 pragmatic | Duplicate YAML alone is not enough reason to redesign. | Cutover-by-aesthetic-discomfort |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 0 | — | — |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | — | — |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 0 | — | — |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | — | — |

**VERDICT:** NO REVIEWS YET — run `/autoplan` or the individual review skills on this new plan if you want the full review pipeline recorded.
