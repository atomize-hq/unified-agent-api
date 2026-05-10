<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/staging-autoplan-restore-20260510-170703.md -->
# PLAN - Worker/Runbook Convergence On Shared Packet Contract

Status: ready for implementation  
Date: 2026-05-10  
Branch: `staging`  
Base branch: `main`  
Repo: `atomize-hq/unified-agent-api`  
Work item: `Converge worker-facing and maintainer-facing maintenance surfaces on the landed shared packet contract`  
Plan commit baseline: `12e373a`  
Design input: user milestone brief in this thread on 2026-05-10; no design doc file was present on disk  
Supersedes: the prior repo-root `PLAN.md` for `packet-first-contract-with-c-tail`, which has now landed

## Executive Summary

The contract core is done. Packet generation, request validation, relay identity, and generated
packet docs now point at one shared maintenance contract.

What still drifts is the operational story around that contract. The two worker workflows, the
source playbooks that feed the packet, the broader operator guide, the generated packet docs, the
PR templates, and the locking tests still describe a partly older topology. Some surfaces still
name deleted per-worker watchers. Some still use stale branch families. Some still describe the
automated lane as packet-only even though the generated write envelope says otherwise. Some still
say `generated-by: xtask refresh-agent` or talk about "local Codex preflight" without explaining
the shared relay role cleanly.

This milestone fixes that operational drift without reopening the contract milestone. It converges
the worker/runbook surface on top of the shared packet contract. It does not change closeout
semantics, add new workflow families, or create a second policy store. It also does not attempt a
full transport rewrite. If transport topology still deserves its own convergence pass after this
cleanup, that becomes an explicit follow-up decision, not hidden scope.

## Objective

Land the approved `worker/runbook convergence` milestone so the live worker workflows, generated
packet docs, maintainer playbooks, operator guide, source templates, and locking tests all tell
the same maintenance story the shared packet contract already enforces.

## Success Criteria

1. All active maintenance docs and tests treat
   `.github/workflows/agent-maintenance-release-watch.yml` as the only live release-watch entry
   point. No active maintenance surface references deleted per-agent watcher workflows as live.
2. All active maintenance surfaces use one canonical maintenance branch family:
   `automation/<agent_id>-maintenance-<target_version>`.
3. Source playbooks and templates that feed automated packets align with the generated packet
   contract:
   - `HANDOFF.md` stays canonical
   - `governance/pr-summary.md` stays derivative
   - workflow YAML stays transport-only
   - packet docs and runbooks stop mixing older worker-era language into the relay story
4. Generated packet docs stop advertising stale ownership and stale lane wording:
   - no automated packet doc keeps `generated-by: xtask refresh-agent`
   - automated packet docs do not call the lane "worker-owned parity surfaces" when the packet
     write envelope includes broader declared surfaces
5. Replay and recovery instructions converge on one honest story:
   - active docs no longer mix stale branch names and stale watcher names
   - recovery text no longer implies an older per-worker topology
   - if `refresh-agent --request ... --write` remains the regeneration path, it is framed honestly
     as current packet regeneration, not as a second hidden maintenance lane
6. Shared runtime wording distinguishes the maintained agent from the execution agent cleanly.
   If Codex remains the local execution binary, the shared relay surfaces say so explicitly instead
   of sounding lane-specific by accident.
7. The locking tests cover both Codex and Claude Code worker/runbook convergence:
   - active watcher references
   - branch family
   - generated packet wording
   - recovery text
   - source template alignment
8. Any broader transport-topology convergence question is recorded explicitly as a follow-up
   decision and is not silently implemented inside this milestone.

## Step 0 Scope Challenge

### Premise Challenge

| Premise | Assessment | Decision |
| --- | --- | --- |
| The contract core is stable enough to clean up operational surfaces on top of it. | Accepted. The packet contract, relay identity, and generated packet docs already landed. | Keep as the foundation. |
| The remaining problem is just prose duplication. | Rejected. The generator, runtime wording, workflows, templates, and tests still encode stale operational truth. | Expand scope to code + docs + tests, not docs only. |
| No new infra is required. | Accepted. Existing worker workflows, packet renderer, and tests are enough. | Keep the current control plane. |
| This milestone should also decide or change the transport model. | Challenged. That is a larger question than worker/runbook convergence and touches the live dispatch topology. | Record as a follow-up decision, not implementation scope here. |
| Closeout semantics should stay unchanged. | Accepted. The user explicitly excluded that work and the relay contract already treats closeout as manual. | Keep out of scope. |

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| Shared release-watch topology | `.github/workflows/agent-maintenance-release-watch.yml`, `crates/xtask/src/agent_maintenance/watch.rs`, `crates/xtask/data/agent_registry.toml` | Reuse. Shared watcher stays canonical. |
| Shared packet contract truth | `docs/specs/maintenance-request-contract-v1.md`, `crates/xtask/src/agent_maintenance/contract_policy.rs`, `prepare.rs`, `request/automation.rs` | Reuse. Do not reopen contract semantics unless drift forces a wording clarification. |
| Generated automated packet docs | `crates/xtask/src/agent_maintenance/docs.rs`, `docs/agents/lifecycle/*-maintenance/**` | Reuse and retune wording/ownership through the renderer only. |
| Relay runtime wording | `crates/xtask/src/agent_maintenance/execute/runtime.rs` | Reuse and clarify. Do not change execution-agent behavior. |
| Live worker transport | `.github/workflows/codex-cli-update-snapshot.yml`, `.github/workflows/claude-code-update-snapshot.yml`, `.github/workflows/agent-maintenance-open-pr.yml` | Reuse. Converge names and recovery language, not acquisition internals. |
| Source playbooks and templates | `cli_manifests/*/OPS_PLAYBOOK.md`, `cli_manifests/*/CI_WORKFLOWS_PLAN.md`, `cli_manifests/*/PR_BODY_TEMPLATE.md` | Reuse and update. These are packet inputs and are in scope. |
| Broader operator procedure | `docs/cli-agent-onboarding-factory-operator-guide.md` | Reuse and correct. It must stop contradicting live packet truth. |
| Locking tests and harnesses | `crates/xtask/tests/c4_spec_ci_wiring.rs`, `agent_maintenance_prepare.rs`, support harnesses | Reuse and update. They currently lock stale wording and names. |

### Existing Code Leverage Map

| Need | Best leverage point |
| --- | --- |
| Shared branch naming and watcher truth | `watch.rs` and `agent_registry.toml` |
| Shared replay/recovery wording | `contract_policy.rs` |
| Generated packet wording | `docs.rs` |
| Shared preflight wording | `execute/runtime.rs` |
| Worker transport references | `codex-cli-update-snapshot.yml`, `claude-code-update-snapshot.yml`, `agent-maintenance-open-pr.yml` |
| Maintainer-facing source docs | `OPS_PLAYBOOK.md`, `CI_WORKFLOWS_PLAN.md`, `PR_BODY_TEMPLATE.md`, operator guide |
| Drift prevention | `c4_spec_ci_wiring.rs`, `agent_maintenance_prepare.rs`, support harnesses |

### Dream State

```text
CURRENT
  shared packet contract is true
  but workers, packet docs, playbooks, templates, operator guide, and tests still disagree

THIS PLAN
  all active maintenance surfaces tell one operational story
  shared watcher is canonical
  branch/replay/recovery wording matches the generated packet contract
  stale worker-era references are removed from source docs and tests

12-MONTH IDEAL
  a new enrolled agent joins maintenance through one watcher contract,
  one worker archetype, one maintainer story, and one locked regression surface,
  with no bespoke topology explanations left in human docs
```

### Implementation Alternatives

| Approach | Effort | Risk | Pros | Cons | Decision |
| --- | --- | --- | --- | --- | --- |
| Doc-only cleanup | S | High | Fastest superficial pass | Generator and tests would reintroduce stale wording; does not fix source-of-truth inputs | Reject |
| Generator + docs + tests convergence on current transport model | M | Medium | Closes the real drift seam without reopening the control plane | Touches many files across code/docs/tests | Recommend |
| Full worker YAML convergence plus transport topology redesign now | L | High | Could simplify longer-term topology | Breaks the milestone boundary and mixes operational cleanup with architecture change | Defer |

### Mode Selection

`SELECTIVE EXPANSION`

Hold the contract milestone boundary. Expand only where the packet contract cannot stay true unless
code, docs, workflows, and tests are updated together.

### Temporal Interrogation

**Hour 1**

The maintainer should be able to answer: which watcher is live, which worker replay to use, which
branch family is expected, which packet is canonical, and what command regenerates packet truth if
PR creation fails.

**Hour 6**

After reading the operator guide, an agent playbook, and the generated `HANDOFF.md`, the maintainer
should not discover that those three documents disagree about the lane.

**Week 6**

When the next agent enrolls in maintenance, the repo should not need another prose cleanup because
source playbooks, generated packet docs, and tests already share the same worker/runbook story.

### Minimum Complete Change

The minimum complete milestone is:

1. converge shared generator and runtime wording that still exports stale worker-era language
2. converge active worker workflows and packet-fed source playbooks/templates on the same watcher,
   branch, replay, and packet-ownership story
3. correct the broader operator guide where it still contradicts the generated packet envelope
4. update locking tests and harnesses so stale watcher names, branch families, and recovery strings
   cannot drift back in
5. record the transport-topology question explicitly as a later follow-up instead of leaving it as
   vague prose inside this milestone

Anything smaller preserves a split story.

### Complexity Check

This milestone will touch multiple code, workflow, doc, and test files. That is justified because
the operational truth is currently split across exactly those surfaces.

The real guardrails are:

- no new workflow family
- no closeout semantic change
- no second policy store
- no registry schema expansion for freeform worker behavior
- no worker acquisition unification attempt
- no renaming of underlying host commands unless it can be done without new infrastructure

### Search / Build Decision

- **[Layer 1]** Reuse the landed shared packet contract and shared watcher.
- **[Layer 1]** Reuse generated packet docs. Fix the renderer, not the rendered output by hand.
- **[Layer 1]** Reuse current worker workflows. Align the operational story first.
- **[Layer 3]** Treat stale wording in source playbooks/templates/tests as contract drift because
  those surfaces feed or constrain the generated packet story.

### Distribution Check

No new user-facing distribution artifact is introduced.

The deliverables are internal factory truth surfaces:

- shared generator/runtime wording
- worker workflows
- packet-fed playbooks and templates
- broader operator docs
- regression tests and harnesses

## Locked Decisions

1. This milestone sits strictly after contract convergence. It does not redesign the packet schema.
2. The shared watcher remains the only live scheduled release-watch entry point.
3. The canonical maintenance branch family is `automation/<agent_id>-maintenance-<target_version>`.
4. The canonical packet root family remains `docs/agents/lifecycle/<agent_id>-maintenance/**`.
   That means `claude_code-maintenance` keeps the underscore because `agent_id` is the live key.
5. `HANDOFF.md` remains the canonical contributor execution contract. `governance/pr-summary.md`
   remains derivative.
6. Source playbooks and templates are in scope because they are read-only inputs for the packet
   contract and because maintainers still use them directly.
7. Generated packet docs remain renderer-owned. No hand-edited fixes in generated maintenance roots.
8. Replay and recovery instructions may be rewritten for honesty and consistency, but this
   milestone does not invent a new command family or a second policy store.
9. Shared relay wording must distinguish the maintained agent from the execution agent. If the
   local executor is still Codex, say that explicitly without confusing it for maintained-agent
   identity.
10. This milestone does not collapse Codex and Claude acquisition logic into one worker workflow.
11. Whether the repo should take a broader transport-topology convergence step after this lands is
    a separate follow-up question, not implementation scope here.

## Architecture

### Current Drift

```text
agent_registry.toml + watch.rs
  -> shared watcher queue and canonical branch family

contract_policy.rs + docs.rs + execute/runtime.rs
  -> shared packet contract
  -> but still export stale ownership/recovery/preflight wording

worker workflows
  -> mostly use shared maintenance inputs
  -> but recovery text and source plan references still carry older branch/topology language

OPS_PLAYBOOK.md / CI_WORKFLOWS_PLAN.md / PR_BODY_TEMPLATE.md / operator guide
  -> partly reflect the shared watcher + packet contract
  -> partly reference deleted watchers, stale branch families, or old packet-only framing

tests and harnesses
  -> still lock some stale watcher names, branch prefixes, and recovery strings

Result:
  the contract is true
  the operational story is not
```

### Target Shape

```text
agent_registry.toml + watch.rs
  -> shared watcher truth

contract_policy.rs
  -> shared branch/recovery/read-input/write-envelope phrasing

docs.rs
  -> generated packet docs with correct ownership markers and wording

execute/runtime.rs
  -> shared relay wording that explains the execution agent cleanly

worker workflows
  -> transport-only acquisition + packet preparation + PR open/repair

source playbooks/templates/operator guide
  -> same branch family
  -> same watcher topology
  -> same packet ownership story

tests
  -> prevent stale watcher names, stale branch families, and stale recovery language
```

### Blast Radius

Primary blast radius:

- shared maintenance code:
  - `crates/xtask/src/agent_maintenance/contract_policy.rs`
  - `crates/xtask/src/agent_maintenance/docs.rs`
  - `crates/xtask/src/agent_maintenance/execute/runtime.rs`
  - `crates/xtask/src/agent_maintenance/watch.rs`
- live workflows:
  - `.github/workflows/agent-maintenance-release-watch.yml`
  - `.github/workflows/agent-maintenance-open-pr.yml`
  - `.github/workflows/codex-cli-update-snapshot.yml`
  - `.github/workflows/claude-code-update-snapshot.yml`
- source playbooks and templates:
  - `cli_manifests/codex/{OPS_PLAYBOOK.md,CI_WORKFLOWS_PLAN.md,PR_BODY_TEMPLATE.md}`
  - `cli_manifests/claude_code/{OPS_PLAYBOOK.md,CI_WORKFLOWS_PLAN.md,PR_BODY_TEMPLATE.md}`
- operator docs:
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
  - `docs/specs/maintenance-request-contract-v1.md`
- generated packet surfaces:
  - `docs/agents/lifecycle/*-maintenance/**`
- tests:
  - `crates/xtask/tests/c4_spec_ci_wiring.rs`
  - `crates/xtask/tests/agent_maintenance_prepare.rs`
  - `crates/xtask/tests/support/agent_maintenance_*`

### Ownership Map

| Surface | Owner | Consumers | Rule |
| --- | --- | --- | --- |
| Shared watcher topology | `agent_registry.toml` + `watch.rs` | workflows, operator docs, tests | one live watcher story |
| Branch family | watcher output + packet contract | workflows, playbooks, templates, packet docs, tests | one branch family everywhere |
| Recovery phrasing | `contract_policy.rs` | packet docs, workflow fallback text, tests | one packet-regeneration story |
| Generated packet wording | `docs.rs` | `HANDOFF.md`, `README.md`, `scope_brief.md`, `threading.md`, `pr-summary.md` | fix renderer only |
| Execution-agent wording | `execute/runtime.rs` + recovery notes | relay evidence, playbooks, tests | explicit shared-relay wording |
| Source playbooks/templates | committed docs under `cli_manifests/*/` | maintainers, packet read-only inputs | must align with the generated packet contract |
| Broader operator procedure | `docs/cli-agent-onboarding-factory-operator-guide.md` | maintainers | must not contradict the packet write envelope |
| Drift prevention | `c4_spec_ci_wiring.rs`, prepare tests, harnesses | CI | lock the new story |

## Implementation Plan

### Phase 1. Converge Shared Generator And Runtime Wording

Purpose: fix the shared code paths that still export stale worker-era language into packet docs,
recovery notes, and relay evidence.

Primary modules:

- `crates/xtask/src/agent_maintenance/contract_policy.rs`
- `crates/xtask/src/agent_maintenance/docs.rs`
- `crates/xtask/src/agent_maintenance/execute/runtime.rs`

Exact changes:

1. Replace stale automated packet ownership markers so generated automated docs no longer claim
   `generated-by: xtask refresh-agent`.
2. Rewrite automated packet prose in `docs.rs` where it still says:
   - "worker-owned parity surfaces"
   - stale packet-only framing
   - stale threading/recovery wording that no longer matches the shared watcher + relay contract
3. Converge recovery notes in `contract_policy.rs` so packet docs, workflow fallback text, and
   tests describe one current packet-regeneration path.
4. Clarify relay preflight wording in `execute/runtime.rs` so the shared lane explains the local
   execution agent cleanly instead of sounding like maintained-agent-specific language by accident.
5. Keep behavior unchanged. This phase is wording and ownership convergence, not execution-model
   change.

Proof:

1. Generated automated packet docs advertise current ownership.
2. Shared recovery notes no longer read like a leftover manual maintenance lane.
3. Relay preflight evidence is explicit about what is being validated and why.

### Phase 2. Converge Worker Workflows, Source Playbooks, And Templates

Purpose: make the human-facing worker and runbook surfaces match the landed packet contract.

Primary files:

- `.github/workflows/codex-cli-update-snapshot.yml`
- `.github/workflows/claude-code-update-snapshot.yml`
- `.github/workflows/agent-maintenance-open-pr.yml`
- `cli_manifests/codex/{OPS_PLAYBOOK.md,CI_WORKFLOWS_PLAN.md,PR_BODY_TEMPLATE.md}`
- `cli_manifests/claude_code/{OPS_PLAYBOOK.md,CI_WORKFLOWS_PLAN.md,PR_BODY_TEMPLATE.md}`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/specs/maintenance-request-contract-v1.md`

Exact changes:

1. Remove live references to deleted per-agent watcher workflows from active playbooks, CI plans,
   and tests.
2. Align branch family references across workflows, playbooks, templates, generated packet docs,
   and tests to `automation/<agent_id>-maintenance-<target_version>`.
3. Align PR template headers and instructions with the live maintenance branch family instead of
   older `automation/codex-cli-<version>` / `automation/claude-code-<version>` wording.
4. Align workflow fallback text and replay instructions with the shared packet contract and current
   recovery path.
5. Correct the operator guide where it still claims the automated upstream-release lane does not
   rewrite surfaces that the current generated write envelope actually includes.
6. Keep workflow YAML transport-only. Do not extract a new shared worker or introduce a new
   dispatch mode.

Proof:

1. Maintainers reading the playbook, operator guide, PR template, and generated `HANDOFF.md` learn
   the same lane.
2. No active maintenance surface names a deleted watcher as live.
3. No active maintenance surface uses a stale maintenance branch family.

### Phase 3. Lock The Story In Tests

Purpose: prevent worker/runbook drift from reappearing through stale strings or stale source docs.

Primary test files:

- `crates/xtask/tests/c4_spec_ci_wiring.rs`
- `crates/xtask/tests/agent_maintenance_prepare.rs`
- `crates/xtask/tests/agent_maintenance_execute.rs`
- `crates/xtask/tests/agent_maintenance_watch.rs`
- `crates/xtask/tests/support/agent_maintenance_harness.rs`
- `crates/xtask/tests/support/agent_maintenance_refresh_harness.rs`
- `crates/xtask/tests/support/agent_maintenance_closeout_harness.rs`

Exact changes:

1. Update stale watcher expectations in `c4_spec_ci_wiring.rs`.
2. Update stale branch-family assertions in `c4_spec_ci_wiring.rs`, templates, and generated
   packet surfaces.
3. Update recovery-string expectations where they still lock stale wording from
   `refresh-agent --request` or older per-worker topology assumptions.
4. Update automated packet doc expectations for the new ownership marker and wording.
5. Add or strengthen Claude Code packet-generation assertions so worker/runbook convergence is
   proven for both enrolled automated agents even when only one committed maintenance root is
   currently present in the repo.

Proof:

1. A stale watcher name or stale maintenance branch family fails CI.
2. Generated packet wording and recovery notes are regression-tested.
3. Claude Code worker/runbook convergence is tested, not assumed.

### Phase 4. Record The Explicit Follow-Up Boundary

Purpose: keep this milestone honest and leave the next transport question explicit instead of
hidden.

Primary files:

- `PLAN.md`
- `TODOS.md`
- any touched runbooks/specs that need a narrow follow-up note

Exact changes:

1. Record that this milestone converges the worker/runbook surface on the current transport model.
2. Record a separate follow-up question for any broader transport-topology convergence decision.
3. Do not start implementing transport-model changes inside this milestone.

Proof:

1. The repo has one clear next question instead of vague "maybe more convergence later" prose.
2. This milestone still lands as one boilable unit.

## Architecture Review

This plan is intentionally operational, not infrastructural.

1. The current repo already has the right core seam: shared watcher, shared packet contract, local
   relay, worker transport. The problem is that active docs and tests still disagree about how that
   seam works.
2. The best leverage is the shared generator/runtime layer plus the packet-fed source docs, not a
   YAML rewrite.
3. The only architecture risk is accidental scope creep into transport redesign. The plan blocks
   that by making the transport decision a separate follow-up.
4. The biggest user-facing failure today is trust erosion. A maintainer sees one branch family in
   the packet, another in the template, and deleted watchers in the CI plan. That turns a "boring"
   lane into detective work.

## Code Quality Review

The code-quality target is one operational story per fact.

1. `contract_policy.rs` should be the source of shared recovery phrasing and packet-fed read/write
   framing. Duplicating that story in workflow-specific prose is how drift returns.
2. `docs.rs` must stop stamping automated packet docs with old ownership and old lane wording.
3. `execute/runtime.rs` must stop sounding like the maintained agent and the execution agent are
   the same thing when they are not.
4. Source playbooks and templates are effectively part of the contract input surface because they
   are named in `read_only_inputs` and because maintainers read them directly. Treat them like code.
5. The tests already know too much about stale strings. Update them with intent, not just find and
   replace.

## Test Review

### Code Path Coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/xtask/src/agent_maintenance/contract_policy.rs
    ├── [CHANGE] shared recovery notes
    ├── [CHANGE] branch/replay wording
    └── [KEEP] no behavioral expansion of the contract

[+] crates/xtask/src/agent_maintenance/docs.rs
    ├── [CHANGE] automated ownership marker
    ├── [CHANGE] README / scope_brief / threading / review_surfaces wording
    ├── [CHANGE] HANDOFF / pr-summary framing
    └── [KEEP] HANDOFF canonical, pr-summary derivative

[+] crates/xtask/src/agent_maintenance/execute/runtime.rs
    ├── [CHANGE] execution-agent preflight wording
    └── [KEEP] preflight behavior and write gating

[+] worker workflows
    ├── [CHANGE] fallback and replay guidance
    ├── [KEEP] shared maintenance inputs
    └── [KEEP] acquisition internals stay worker-specific

[+] source playbooks/templates/operator guide
    ├── [CHANGE] live watcher references
    ├── [CHANGE] branch family
    ├── [CHANGE] packet ownership story
    └── [CHANGE] write-envelope honesty

[+] crates/xtask/tests/*
    ├── [CHANGE] watcher reference assertions
    ├── [CHANGE] branch-family assertions
    ├── [CHANGE] recovery-string assertions
    └── [ADD] Claude packet/runbook parity coverage where missing
```

### Maintainer Flow Coverage

```text
USER FLOW COVERAGE
===========================
[+] Shared watcher -> Codex worker -> prepare packet -> maintainer relay
    ├── [KEEP] worker transport
    └── [CHANGE] branch/replay/runbook wording convergence

[+] Shared watcher -> Claude worker -> prepare packet -> maintainer relay
    ├── [KEEP] worker transport
    └── [CHANGE] stale watcher/branch/playbook/template wording

[+] Manual worker replay
    ├── [CHANGE] replay input descriptions
    └── [KEEP] workflow_dispatch surface

[+] PR creation failure recovery
    ├── [CHANGE] honest packet-regeneration instructions
    └── [KEEP] current recovery command family unless a later milestone replaces it

[+] Maintainer relay
    ├── [CHANGE] execution-agent preflight explanation
    └── [KEEP] dry-run/write/closeout boundaries
```

### Required Test Commands

Run at minimum:

```bash
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

## Performance Review

There is no throughput goal here. The important performance property is cognitive performance:
how long it takes a maintainer to trust the lane.

Guardrails:

1. Do not add new runtime subprocesses or network calls.
2. Do not widen the relay write envelope just to make docs simpler.
3. Do not add new workflow fanout or retry loops.
4. Keep the change mostly textual and test-assertion level outside the shared generator/runtime
   modules.

## DX Review

### Developer Journey Map

| Stage | What the maintainer tries to do | Current friction | Target after this milestone |
| --- | --- | --- | --- |
| 1. Notice stale release | Understand which watcher is live | CI plans still mention deleted watchers | One shared watcher named everywhere |
| 2. Pick the right worker replay | Determine which workflow to rerun | Docs mix live worker and stale branch language | One replay story per agent, same input shape |
| 3. Locate the packet | Find the canonical request and handoff | Generated docs and source docs use mixed wording | Packet ownership made explicit |
| 4. Trust the branch name | Confirm which PR branch should exist | Templates still use older branch families | One maintenance branch family everywhere |
| 5. Understand the relay | Know what `execute-agent-maintenance` is validating | "local Codex preflight" reads like lane confusion | Explicit execution-agent explanation |
| 6. Recover from PR failure | Regenerate packet truth safely | Recovery language still smells like old control-plane flow | Honest, repeatable packet-regeneration instructions |
| 7. Compare Codex vs Claude | Decide whether both lanes behave the same way | Claude docs still carry more stale topology language | Same high-level lane story, agent-specific acquisition only |
| 8. Close the run | Know what stays manual | Closeout itself is fine | Preserve current closeout clarity |
| 9. Hand off the next step | Know whether transport convergence is still pending | Today it is vague | Explicit follow-up question recorded |

### Developer Empathy Narrative

I got a maintenance alert. I opened the worker playbook to replay it, then I opened the generated
packet because that is supposed to be canonical. If those two surfaces disagree about the branch
name, the watcher, or the recovery command, I stop trusting the lane. I start re-reading workflow
YAML instead of just doing the work.

This milestone matters because it removes that trust tax. The maintainer should spend time on the
actual wrapper update or doc change, not on figuring out which doc is stale.

### DX Scorecard

| Dimension | Current | Target | Notes |
| --- | --- | --- | --- |
| Getting started clarity | 5/10 | 9/10 | Shared watcher truth is live in code, but not in all docs. |
| Replay ergonomics | 5/10 | 9/10 | Inputs are shared; wording still drifts. |
| Naming consistency | 4/10 | 9/10 | Branch families still conflict across templates and tests. |
| Error/recovery clarity | 5/10 | 8/10 | Recovery exists, but it still sounds like older control-plane flow. |
| Packet ownership clarity | 6/10 | 9/10 | `HANDOFF.md` is canonical, but other docs still blur that fact. |
| Worker parity story | 5/10 | 8/10 | Same input contract, different acquisition internals, mixed docs. |
| Operator guide honesty | 4/10 | 9/10 | Current write-envelope statement contradicts live packet truth. |
| Drift resistance | 6/10 | 9/10 | Tests exist, but some lock stale strings today. |

### TTHW Assessment

Current time-to-honest-worker-replay: about 15-20 minutes for a maintainer who cross-checks docs
instead of trusting the first surface they open.

Target time-to-honest-worker-replay after this milestone: under 7 minutes.

### DX Implementation Checklist

- Remove deleted watcher references from active maintenance docs and tests.
- Align branch-family wording across templates, playbooks, generated docs, and tests.
- Make the operator guide match the actual packet write envelope.
- Make recovery guidance honest and consistent across packet docs, workflow fallback text, and
  source playbooks.
- Clarify shared relay execution-agent wording.
- Add or strengthen Claude parity tests for generated worker/runbook surfaces.

## Failure Modes Registry

| Failure mode | Surface | Test coverage required | Handling required | Silent if missed? | Status |
| --- | --- | --- | --- | --- | --- |
| Active docs still name deleted per-agent watchers | CI plans, playbooks, tests | `c4_spec_ci_wiring` + doc assertions | remove stale references | yes | must close |
| Branch family stays split across packet docs and templates | templates, playbooks, packet docs, tests | prepare tests + CI wiring tests | one canonical branch family | yes | must close |
| Operator guide keeps lying about the write envelope | operator guide vs generated packet docs | doc assertions + review | align guide to live packet truth | yes | must close |
| Automated packet docs keep stale ownership markers | `docs.rs` + generated roots | prepare tests | change renderer-owned marker | yes | must close |
| Recovery guidance still reads like the older manual lane | `contract_policy.rs`, workflow fallbacks, tests | prepare tests + CI wiring tests | converge recovery text | yes | must close |
| Shared relay wording confuses maintained agent and executor | `execute/runtime.rs`, playbooks, generated notes | execute tests | clarify wording without changing behavior | no | should close |
| Claude lane remains under-tested for runbook convergence | prepare tests/harnesses | new or stronger Claude assertions | add parity coverage | yes | must close |

## NOT In Scope

1. Changing closeout semantics or making closeout automatic.
2. Creating a new workflow family or a new dispatch kind.
3. Introducing a second policy store outside the registry plus shared packet policy code.
4. Collapsing Codex and Claude acquisition logic into one worker workflow.
5. Renaming the underlying host command family as infrastructure work.
6. Reopening the maintenance packet contract milestone unless a wording clarification is strictly
   required for honesty.
7. New-agent enrollment work for `opencode`, `goose`, or any other candidate.

## Worktree Parallelization Strategy

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| 1. Converge generator/runtime wording | `crates/xtask/src/agent_maintenance/` | — |
| 2. Converge worker workflows + source docs/templates | workflows, `cli_manifests/*/`, operator docs | 1 |
| 3. Update tests and harnesses | `crates/xtask/tests/**` | 1, 2 |
| 4. Final verification | repo-wide validation commands | 2, 3 |

### Parallel Lanes

Lane A: Step 1  
Settle the shared wording and recovery story first. This is the critical path.

Lane B: Step 2  
Can begin once Phase 1 settles the exact wording and branch/recovery decisions.

Lane C: Step 3  
Can begin once Phase 2 settles the new strings and expected packet wording.

### Conflict Flags

1. `contract_policy.rs`, `docs.rs`, and the workflow fallback text must agree. Do not update test
   expectations before those strings are stable.
2. PR template branch headers and CI wiring tests both lock branch-family wording. Change them in
   the same pass.
3. If the operator guide write-envelope statement changes, re-check generated packet docs and
   shared writable-surface derivation to avoid introducing a new contradiction.

## Cross-Phase Themes

**Theme: the contract is right, but the inputs and explanations around it are stale.**

This appeared in strategy review, engineering review, and DX review independently. The problem is
not lack of machinery. The problem is that packet-fed source docs, generated packet docs, worker
fallback text, and locking tests still describe different versions of the lane.

**Theme: naming drift is doing real damage.**

Deleted watcher files, stale branch families, and mixed underscore/dash branch examples all force
the maintainer to verify the lane manually instead of trusting the packet contract.

**Theme: replay and recovery are the highest-trust failure point.**

If packet regeneration and PR recovery read like leftovers from the old transport era, the lane
feels unsafe even when the actual contract is fine.

## Follow-Up Question

After this milestone lands, the next explicit question is:

`transport-topology convergence review`

That follow-up would decide, not assume:

1. whether the current worker transport split is still the right long-term shape
2. whether replay/regeneration should keep the current command family or earn a more direct packet
   preparation surface
3. whether packet-pr fallback and worker dispatch should converge further now that the contract is
   stable

That is not implementation scope in this milestone.

## Completion Summary

- Step 0: Scope Challenge, accepted with one major boundary correction
- Architecture Review: 1 strategic scope warning, keep transport-topology decisions out
- Code Quality Review: 5 mandatory wording/input/test convergence targets identified
- Test Review: diagrams produced, 6 mandatory assertion families identified
- DX Review: current operational DX scored 4-6/10 depending on surface; target 8-9/10 after
  convergence
- Failure modes: 6 silent drift risks identified and required to close
- NOT in scope: written
- Cross-phase themes: written
- Follow-up transport question: made explicit

## Exit Criteria

This plan is done when:

1. the shared watcher is the only live watcher named across active maintenance surfaces
2. one maintenance branch family is used everywhere active maintenance is described
3. generated automated packet docs no longer carry stale ownership or stale worker-era framing
4. packet-fed source playbooks/templates and the broader operator guide match the generated packet
   story
5. recovery and replay instructions tell one honest story
6. the regression suite prevents stale watcher names, stale branch families, and stale packet
   wording from returning

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | CEO | Keep this milestone on top of the landed packet contract instead of reopening contract design. | Mechanical | P3 pragmatic | The contract core already landed; the remaining drift is operational. | Reopening packet semantics now |
| 2 | CEO | Reject doc-only cleanup and include generator/runtime/tests in scope. | Mechanical | P1 completeness | Source docs and tests currently regenerate or lock the stale story. | Prose-only cleanup |
| 3 | CEO | Keep shared watcher as the only live release-watch entry point. | Mechanical | P4 DRY | The repo already centralized watcher truth in code and workflow. | Reintroducing per-agent watcher explanations |
| 4 | CEO | Treat broader transport-topology convergence as a separate follow-up question. | User Challenge | P3 pragmatic | The current milestone is boilable; transport redesign is a larger architectural step. | Deciding or changing the transport model inside this milestone |
| 5 | Eng | Keep `automation/<agent_id>-maintenance-<target_version>` as the canonical branch family. | Mechanical | P5 explicit | That is the live branch contract emitted by the shared watcher and packets today. | Preserving stale `automation/codex-cli-*` or `automation/claude-code-*` examples |
| 6 | Eng | Fix generated packet docs through `docs.rs`, not by hand. | Mechanical | P4 DRY | Generated docs must stay renderer-owned or drift will return immediately. | Hand-editing maintenance roots |
| 7 | DX | Align the operator guide to the actual packet write envelope instead of the current packet-only claim. | Mechanical | P1 completeness | The guide must not contradict the live packet truth. | Leaving the contradiction in place |
| 8 | DX | Clarify shared relay preflight wording instead of pretending the executor is agent-agnostic or lane-specific. | Taste | P5 explicit | The relay still uses Codex locally; the wording should explain that cleanly. | Leaving ambiguous "local Codex preflight" phrasing everywhere |
| 9 | Eng | Prove Claude worker/runbook convergence in tests rather than requiring a committed second maintenance root as the milestone vehicle. | Taste | P3 pragmatic | Tests are cheaper and less brittle than committing synthetic packet roots for every lane. | Requiring committed generated packet docs as the only proof |
| 10 | DX | Keep the current command family and fix replay/recovery wording around it in this milestone. | Taste | P3 pragmatic | Renaming commands would become infra work; the immediate problem is stale instructions. | New command family or command-surface rewrite now |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
| --- | --- | --- | --- | --- | --- |
| CEO Review | `/autoplan` | Scope & strategy | 1 | issues_open | 1 user challenge, 1 major scope correction, 2 cross-surface blind spots |
| Codex Review | `codex exec` | Independent 2nd opinion | 1 | issues_open | confirmed split between operational convergence and transport-topology decision |
| Eng Review | `/autoplan` | Architecture & tests | 1 | issues_open | 6 assertion/update families required across code, workflows, and tests |
| Design Review | skipped | No UI scope detected | 0 | skipped | no UI surfaces in scope |
| DX Review | `/autoplan` fallback rubric | Developer workflow clarity | 1 | issues_open | current operational DX 4-6/10, target 8-9/10 after convergence |

**VERDICT:** READY WITH ONE USER CHALLENGE.

The implementation milestone is coherent if it is kept to worker/runbook convergence on the
current transport model. The only flagged challenge is whether you still want the broader
"decide whether transport convergence is needed" question inside this milestone, because both the
primary review and the independent Codex pass recommend making that a separate follow-up decision.
