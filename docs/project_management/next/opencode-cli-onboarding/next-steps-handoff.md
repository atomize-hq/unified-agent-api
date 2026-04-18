# OpenCode Onboarding - Next Steps Handoff

Status: Draft handoff, updated with `/plan-eng-review` decisions through 2026-04-18  
Date (UTC): 2026-04-18  
Branch when written: `feat/opencode-cli-onboarding`  
Commit when written: `2637101`

## Purpose

This file is the restart point for a fresh `/plan-eng-review` or `/office-hours` session.

It captures:
- what is already true in the repo
- what the OpenCode planning work actually accomplished
- why the next step still feels ambiguous
- the recommended operating-model change
- the concrete deliverables to create next

Use this file instead of re-reading the full packet history unless a session needs seam-local detail.

## Current State Summary

The repo has already completed the first real third-agent selection and contract-locking pass for `OpenCode`.

That work is real and landed:
- the candidate-selection packet exists in `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`
- the extracted OpenCode seam pack exists in `docs/project_management/next/opencode-cli-onboarding/`
- the OpenCode normative contracts exist under `docs/specs/`
- the pack is formally closed with no active or next seam

The important truth is this:

```text
third-agent selection and contract hardening = done
actual implementation-pack promotion = not yet defined
```

So the repo has a good answer to "which agent and what contract shape?" but not yet a canonical answer to "what exact artifact gets created next so implementation can begin mechanically?"

## What Landed

### Canonical selection result

The repo selected `OpenCode` as the first real third CLI agent after comparing it with `Gemini CLI` and `aider`.

Why `OpenCode` won:
- strongest combination of current pull and terminal-native agent workflow
- useful extra runtime surface for stress-testing the repo's neutral seams
- enough novelty to validate the architecture without turning the effort into a science project

Primary packet:
- `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`

### Canonical OpenCode v1 runtime decision

The repo already locked the v1 wrapper surface:
- canonical runtime seam: `opencode run --format json`
- deferred surfaces: `serve`, `acp`, `run --attach`, interactive TUI-first flows

Normative sources:
- `docs/specs/opencode-wrapper-run-contract.md`
- `docs/specs/opencode-onboarding-evidence-contract.md`

### Canonical crate-first sequencing

The intended execution order is already consistent across the docs:

```text
runtime/evidence lock
  -> wrapper crate + manifest foundation
  -> agent_api backend mapping
  -> UAA promotion review
```

That sequencing is right. The missing piece is not the order. The missing piece is the execution
pack that consumes the already-published closeout/thread bridge and turns the closed contract pack
into active execution work.

### Canonical support/publication posture

The repo already has the right future-agent-shaped support model:
- manifest support
- backend support
- UAA unified support
- passthrough visibility

Normative source:
- `docs/specs/unified-agent-api/support-matrix.md`

This matters because the next process step must preserve this separation instead of collapsing everything into "OpenCode support."

## Review Decisions Locked So Far

These decisions were accepted during `/plan-eng-review` and should be treated as the current plan
of record unless later evidence reopens them.

1. Scope is reduced to one OpenCode execution pack.
   - Do not standardize the whole future-agent factory first.
   - Do not make process codification a prerequisite for landing OpenCode.
2. The bridge reuses the closed-pack artifacts that already exist.
   - `THR-04`, `threading.md`, and the seam closeouts are the authoritative handoff.
   - Do not add a new reusable handoff manifest or parallel bridge ledger for this pass.
3. UAA promotion is not an active execution lane in the OpenCode implementation pack.
   - The closed pack already records that no additional UAA promotion follow-on is required under
     the current evidence basis.
   - Reopen only if the published stale triggers fire.
4. The execution-pack README should be the plan-of-record for OpenCode landing.
   - Do not create a separate generic lifecycle doc yet.
   - Revisit generic lifecycle/codification docs only after OpenCode lands and repetition is real.
5. The execution pack must carry an explicit verification matrix.
   - No vague "test plan later" placeholder.
   - The pack must name exact validator commands, wrapper/fixture test surfaces, and `agent_api`
     regression coverage expectations.
6. Deterministic replay, fake-binary, and fixture validation are the default proof path.
   - Live provider-backed smoke is for initial basis lock or stale-trigger revalidation only.
   - Do not make routine slice completion depend on authenticated OpenCode runs.
7. The OpenCode landing work should live in one execution pack with multiple seams/workstreams.
   - Use one `opencode-implementation/` plan-of-record.
   - Split work inside that pack by manifest root, wrapper crate, and `agent_api` backend seams.

## What The Review Concluded

The CEO review conclusion was:

```text
Do not reopen OpenCode selection.
Do not build a generator yet.
Do formalize the bridge from "closed onboarding contract pack"
to "active implementation pack".
```

The current system is good at:
- choosing a candidate
- proving repo fit
- freezing the runtime boundary
- writing canonical specs
- closing the planning pack cleanly

The current system is weak at:
- telling the next operator which execution artifact to create after pack closeout
- making that handoff obvious from the existing closeout and thread records
- defining the exact verification burden for the first code-facing OpenCode work
- separating "land OpenCode once" from "codify the whole process for agent four"

## Root Cause Of The Ambiguity

The ambiguity is structural, not motivational.

The template intentionally stops at a "shape-agnostic implementation handoff." That was a reasonable first move because the team had not used the template once yet.

Now that the first real use is complete, that deliberate vagueness has become the new bottleneck.

The current end state looks like this:

```text
candidate packet
  -> closed contract pack
  -> ????
  -> code implementation
```

That `????` is the actual missing artifact.

Because it does not exist yet:
- the OpenCode pack reads as complete
- the repo still lacks a deterministic entrypoint for implementation planning
- future agent 4 onboarding would hit the same ambiguity again

## Landscape Synthesis

### Layer 1 - Tried and true

The stable pattern in this repo is:
- docs first
- narrow contracts first
- neutral shared semantics first
- implementation only after contract surfaces are locked

That pattern has worked well in the support-matrix and model-selection work. The answer is not to abandon it.

### Layer 2 - External market reality

The current CLI-agent market is converging on terminal-native, scriptable, model-backed agents with multiple helper surfaces.

External sources reviewed during the CEO review:
- OpenCode docs: <https://opencode.ai/en/docs>
- Google Gemini CLI announcement: <https://blog.google/technology/developers/introducing-gemini-cli-open-source-ai-agent>
- aider docs: <https://aider.chat/docs/>

Important market observation:
- the winning products are broadening surface area fast
- this repo should not try to support every surface in v1
- the repo's leverage comes from choosing one canonical surface, publishing explicit deferred surfaces, and making the rest mechanical later

### Layer 3 - First-principles conclusion

The missing value is not "better candidate research."

The missing value is "make OpenCode landing boring after the first proof."

That means:
- one explicit execution pack
- one explicit rule for which existing closeout artifacts it consumes
- one explicit verification matrix
- only after the second successful run, consider codifying or scaffolding the process further

## Recommended Operating Model Change

For the current step, keep the operating-model change minimal:
- preserve the existing candidate/contract pack and its closeout records
- create one OpenCode execution pack that consumes those records directly
- defer generic lifecycle/codification work until OpenCode proves the second-use pattern

The execution handoff should look like this:

```text
closed contract pack
  -> existing closeout + thread handoff
  -> OpenCode execution pack
```

The execution pack should consume these already-published facts instead of restating them in a new
bridge artifact:
- canonical runtime surface and deferred surfaces
- manifest-root contract and artifact inventory expectations
- backend mapping boundary and capability/extension ownership
- support-layer separation and the current "no new UAA follow-on" posture
- stale/reopen triggers from the seam closeouts

## What Already Exists

Existing repo/doc surfaces the execution pack should reuse directly:
- `docs/project_management/next/opencode-cli-onboarding/threading.md`
- `docs/project_management/next/opencode-cli-onboarding/governance/seam-1-closeout.md`
- `docs/project_management/next/opencode-cli-onboarding/governance/seam-2-closeout.md`
- `docs/project_management/next/opencode-cli-onboarding/governance/seam-3-closeout.md`
- `docs/project_management/next/opencode-cli-onboarding/governance/seam-4-closeout.md`
- `docs/specs/opencode-wrapper-run-contract.md`
- `docs/specs/opencode-onboarding-evidence-contract.md`
- `docs/specs/opencode-cli-manifest-contract.md`
- `docs/specs/opencode-agent-api-backend-contract.md`
- `docs/specs/unified-agent-api/support-matrix.md`
- existing implementation and validation patterns in `crates/codex/`, `crates/claude_code/`,
  `crates/agent_api/`, and `crates/xtask/`

These are already enough to define:
- the wrapper boundary
- the manifest-root contract
- the backend mapping boundary
- the current no-follow-on UAA promotion posture
- the stale triggers that would justify reopening promotion/spec work

## NOT in scope

Explicitly deferred for this next step:
- generic CLI-agent lifecycle documentation
  - Rationale: one more abstraction layer before a second proof run is more likely to drift than help.
- reusable handoff manifest templates or YAML sidecars
  - Rationale: the closed pack already published the needed bridge via `THR-04` and seam closeouts.
- active UAA promotion/publication work for OpenCode
  - Rationale: `SEAM-4` already recorded that no additional UAA promotion follow-on is required under the current evidence basis.
- scaffolding, generators, or `xtask` automation for future agent onboarding
  - Rationale: tooling should come after repetition proves where the actual friction is.
- reopening candidate selection
  - Rationale: the repo already selected OpenCode and nothing in the current evidence makes that basis stale.

## Deliverables To Create Next

These are the deliverables a fresh planning session should flesh out and approve.

### D1. OpenCode execution pack

Suggested path:
- `docs/project_management/next/opencode-implementation/`

Purpose:
- convert the already-closed OpenCode contract decisions into executable implementation planning
- serve as the plan-of-record for landing OpenCode

This is not another candidate packet.
This is the code-facing pack.

Suggested seam/workstream shape:
- manifest root for `cli_manifests/opencode/`
- wrapper crate `crates/opencode/`
- `crates/agent_api` OpenCode backend

Recommended pack shape:
- one execution pack
- multiple seams/workstreams inside the pack, sequenced crate-first
- no sibling pack family unless later scope growth proves the split is necessary

What it should explicitly consume:
- `docs/project_management/next/opencode-cli-onboarding/threading.md`
- `docs/project_management/next/opencode-cli-onboarding/governance/seam-1-closeout.md`
- `docs/project_management/next/opencode-cli-onboarding/governance/seam-2-closeout.md`
- `docs/project_management/next/opencode-cli-onboarding/governance/seam-3-closeout.md`
- `docs/project_management/next/opencode-cli-onboarding/governance/seam-4-closeout.md`
- `docs/specs/opencode-wrapper-run-contract.md`
- `docs/specs/opencode-onboarding-evidence-contract.md`
- `docs/specs/opencode-cli-manifest-contract.md`
- `docs/specs/opencode-agent-api-backend-contract.md`

Expected outputs:
- implementation-ready seams or task-pack equivalents
- explicit acceptance gates for each workstream
- fixture and replay strategy
- a verification matrix with named test files and commands
- an explicit note that UAA promotion is out of scope unless stale triggers fire

Minimum required verification scope:
- wrapper/fixture coverage for the canonical `opencode run --format json` path
- accepted-control coverage for `--model`, `--session` / `--continue`, `--fork`, and `--dir`
- fail-closed coverage for deferred/helper surfaces
- `agent_api` mapping/regression coverage for redaction, bounded payloads, unsupported extensions,
  capability advertisement, and DR-0012 completion gating
- manifest-root validator coverage, including the eventual `codex-validate --root cli_manifests/opencode`
  gate and any required support-matrix publication checks
- replay/fake-binary/fixture validation as the default done-ness gate, with live provider smoke
  reserved for basis revalidation only

### D2. Post-proof codification checkpoint

Purpose:
- decide what, if anything, actually repeated during the OpenCode landing and is worth promoting
  into a reusable lifecycle or template artifact later

This should stay out of the critical path for landing OpenCode itself.

Candidates for later codification:
- execution-pack template shape, if repetition proves it
- repo checklist for future agents
- eventual scaffolder or `xtask` support, only if repetition proves it is real leverage

## Recommended Session Sequencing

### Option A - Engineering-first

Use `/plan-eng-review` on this file first if the immediate goal is to lock execution mechanics.

Why:
- the main gap is architectural and process-boundary clarity
- the next work is about lifecycle, artifacts, ownership, and execution shape

Suggested objective for that session:
- lock D1
- decide whether `opencode-implementation/` should be one pack or a small pack family
- define the verification matrix and workstream ownership

### Option B - Strategy-first

Use `/office-hours` on this file first if the immediate goal is to rethink the ideal long-term factory before locking execution shape.

Why:
- `/office-hours` is better if the team wants to ask "what is the boring, scalable, inevitable version of this process?"
- it is the right place to challenge whether the two-pack system is enough or whether there should really be a three-stage model

Suggested objective for that session:
- design the ideal long-term operating model
- decide what must exist now versus what should wait until OpenCode proves the process

### Practical recommendation

Run `/plan-eng-review` first.

Reason:
- the short-term blocker is execution ambiguity, not vision ambiguity
- the repo already has enough product direction to proceed
- once the bridge and implementation-pack shape are locked, `/office-hours` can still be used later for the bigger "codify the whole machine" question

## Open Questions The Next Session Should Resolve

1. Which seam/workstream owns `cli_manifests/opencode/` versus `crates/opencode/` versus `crates/agent_api/` planning inside the single execution pack?
2. What exact evidence class should count as backend-support publication for OpenCode beyond `wrapper_coverage.json`?
3. Which exact validator and test commands are required before the execution pack can call a slice done?
4. When OpenCode lands, what specific proof threshold justifies codifying the process for agent four?
5. What should remain docs-first forever, and what would actually benefit from helper tooling later?

## Failure Modes To Avoid

### Failure mode 1: Reopening selection

Do not spend the next session re-arguing whether `OpenCode` was the right winner unless new external evidence materially changes the basis.

That decision is already good enough and already locked.

### Failure mode 2: Skipping the bridge artifact

Do not jump straight from the closed pack into ad hoc implementation planning without consuming the
existing bridge.

That bridge is already published in `THR-04`, `threading.md`, and the seam closeouts. Ignoring it
would solve OpenCode locally while preserving the process hole.

### Failure mode 3: Premature scaffolding

Do not build a generator, automation command, or large framework before the implementation-pack shape is proven by one full OpenCode landing.

That would be another 200-line config file to print hello world.

### Failure mode 4: Collapsing support layers

Do not let OpenCode implementation planning blur:
- manifest support
- backend support
- UAA unified support
- passthrough visibility

The support-matrix work already paid for that clarity. Keep it.

### Failure mode 5: Treating test coverage as a later implementation detail

Do not let the execution pack say only "test and validation plan" without naming the actual
commands, regression surfaces, and validator gates.

That is how the happy path gets implemented first and the fail-closed edges get rediscovered in
review.

### Failure mode 6: Letting live provider smoke become the default developer loop

Do not let routine verification depend on authenticated OpenCode runs when replay, fake-binary, or
fixture evidence can prove the same contract mechanically.

That would make every implementation slice slower, flakier, and more expensive to verify than the
repo's contracts require.

## Source Files Worth Reading First

If a fresh session needs supporting detail, read these in order:

1. `docs/project_management/next/opencode-cli-onboarding/next-steps-handoff.md`
2. `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`
3. `docs/project_management/next/opencode-cli-onboarding/README.md`
4. `docs/specs/opencode-wrapper-run-contract.md`
5. `docs/specs/opencode-onboarding-evidence-contract.md`
6. `docs/specs/opencode-cli-manifest-contract.md`
7. `docs/specs/opencode-agent-api-backend-contract.md`
8. `docs/specs/unified-agent-api/support-matrix.md`

## Suggested Kickoff Prompt For `/plan-eng-review`

Use this file as the primary brief.

Suggested prompt:

> We already completed the first real third-agent contract pack for OpenCode. Read `docs/project_management/next/opencode-cli-onboarding/next-steps-handoff.md` first. The job is to create the OpenCode execution pack that consumes the existing closeout/thread handoff, plans `cli_manifests/opencode/`, `crates/opencode/`, and `crates/agent_api` work, and names the exact verification matrix required to land it. Do not reopen candidate selection, do not add a new bridge artifact, and do not reopen UAA promotion unless the stale triggers in the closed pack fire.

## Suggested Kickoff Prompt For `/office-hours`

Use this file as the primary brief.

Suggested prompt:

> Read `docs/project_management/next/opencode-cli-onboarding/next-steps-handoff.md` first. We already proved the first agent-selection and contract-locking cycle with OpenCode. The question now is: what is the best long-term operating model that makes future CLI-agent onboarding boring, deterministic, and low-ceremony without prematurely building tooling? Help design the ideal process, then identify the smallest set of deliverables we should create now to land OpenCode and prove the model once.

## Bottom Line

The next move is not "more OpenCode research."

The next move is:
- create the OpenCode execution pack
- make it consume the existing closeout/thread handoff
- give it an explicit verification matrix
- land OpenCode
- only then decide what process deserves codification for future agents

That is the whole game.

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 1 | ISSUES_OPEN | 3 proposals, 0 accepted, 0 deferred |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | — | — |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 1 | CLEAN | 23 issues, 0 critical gaps |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | — | — |

- **UNRESOLVED:** 0
- **VERDICT:** ENG CLEARED — ready to create the OpenCode execution pack. CEO review remains informational and does not block implementation.
