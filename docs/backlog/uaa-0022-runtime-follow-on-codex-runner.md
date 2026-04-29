<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/codex-recommend-next-agent-autoplan-restore-20260429-132103.md -->
# uaa-0022 — Runtime follow-on milestone: enclose the implementation lane in Codex

Status: Approved for implementation planning (backlog-only; not part of canonical onboarding procedure yet)

This note is the milestone definition for the next onboarding automation seam.

The recommendation lane is now good enough to get to approval. The control-plane commands are now
good enough to enroll a new agent, scaffold the wrapper shell, refresh publication surfaces, and
close the proving run. The remaining bottleneck is the part in the middle where someone still has
to translate repo patterns into real wrapper and backend code.

That is the seam.

Why backlog-only:
- The shipped procedure in `docs/cli-agent-onboarding-factory-operator-guide.md` remains the live
  source of truth until this milestone is actually implemented.
- This note is a decision record plus milestone scope, not a claim that the operator flow already
  changed.

## Why this milestone matters

Right now, create-mode onboarding still depends on a maintainer being able to do all of this from
judgment:
- implement wrapper/runtime code in `crates/<agent_id>`
- implement the harnessed `agent_api` backend in `crates/agent_api/src/backends/<agent_id>`
- author the wrapper coverage source-of-truth at the registry-owned path
- populate committed runtime evidence under `cli_manifests/<agent_id>/`

That is exactly where lead time and inconsistency now live.

This is the whole game for the next milestone: make the runtime follow-on boring, bounded, and
reviewable.

## Decision record

### 1. The next milestone is narrower than "fully automate onboarding"

This milestone does **not** mean "all remaining create-mode work becomes one command."

It means:
- the operator-guide runtime follow-on gets enclosed in a bounded Codex execution path
- the repo owns the execution recipe
- the output is reviewable against a known baseline

It does **not** mean:
- publication refresh after runtime evidence is folded into the same milestone
- closeout governance becomes fully automatic
- the create-mode operator guide gets rewritten before the runner exists

### 2. Default support tier is `opencode`-level

The expected baseline for new agent support is now:
- `default` = `opencode`-level support

Why:
- it exercises the backend harness path
- it includes session semantics
- it is closer to the universal minimum this repo actually wants from a serious onboarded backend

### 3. `minimal` is allowed only as an exception

- `minimal` = `gemini_cli`-level support
- `minimal` is **not** the target baseline
- `minimal` is allowed only when the implementation packet records an explicit justification for
  why the agent is shipping below the default tier

If Codex lands a minimal implementation, the repo should force the answer to two questions:
- why is `default` not the right bar for this agent right now?
- what exact follow-up would be required to reach `default`?

### 4. `codex` and `claude_code` are feature-rich references, not templates

They remain valuable examples for optional richer surfaces such as:
- add-dirs
- external sandbox policy
- MCP management
- richer session/runtime handling

But the runner should not start there by default. That path is how a bounded implementation lane
turns back into a vibe-based scavenger hunt.

## Milestone scope

This milestone is only about enclosing the runtime follow-on.

In scope:
- a repo-owned Codex execution recipe for the runtime lane
- a pinned file/intake contract for what Codex must read before writing code
- a pinned output contract for what Codex must produce
- a review rule for exception-tier (`minimal`) outcomes
- a baseline-template rule that defaults to `opencode`

Out of scope:
- publication refresh automation after runtime evidence exists
- redefining the control-plane boundary from M6
- expanding the baseline tiers beyond the current `default` / `minimal` / `feature-rich` framing
- introducing new universal minimums beyond what this milestone explicitly records

## Runner contract

### Required inputs

The runner should assume the create-mode packet and scaffold already exist, then execute the
runtime follow-on from those concrete inputs.

Required reads:
- `docs/cli-agent-onboarding-factory-operator-guide.md` (runtime follow-on section)
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/adr/0013-agent-api-backend-harness.md`
- the target agent approval artifact
- the generated onboarding handoff packet
- `crates/agent_api/src/backends/opencode/**`
- `crates/opencode/**`

Conditional reads:
- `crates/agent_api/src/backends/gemini_cli/**` only when considering a `minimal` exception
- `crates/agent_api/src/backends/codex/**` for optional richer patterns
- `crates/agent_api/src/backends/claude_code/**` for optional richer patterns
- wrapper coverage source files in existing wrapper crates when the target agent needs richer
  coverage declarations

### Required outputs

The runner should produce:
- code changes under `crates/<agent_id>`
- code changes under `crates/agent_api/src/backends/<agent_id>`
- wrapper coverage source-of-truth updates at the registry-owned path
- committed runtime evidence under `cli_manifests/<agent_id>/`
- an implementation summary that states:
  - achieved tier: `default`, `minimal`, or `feature-rich`
  - primary template used
  - if `minimal`, the justification for shipping below `default`
  - if not `feature-rich`, what richer surfaces were intentionally deferred

## Review bar

The runner is only useful if the output is easy to review.

That means the repo should be able to answer:
- Did Codex target the right baseline?
- If it shipped below baseline, did it explain why?
- Did it follow the backend-harness pattern instead of cloning old glue?
- Did it leave a crisp follow-up when richer behavior was intentionally deferred?

If the answer to any of those is "not really," the milestone did not land.

## Success criteria

This milestone is defined well enough to implement when the repo has all of the following written
down:
- the next automation seam is explicitly the runtime follow-on
- `opencode` is the default baseline template
- `gemini_cli` is exception-only and requires justification
- `codex` / `claude_code` are richer references, not the default starting point
- publication refresh automation is explicitly deferred to the next milestone
- the runner has a clear input contract and output contract

## Follow-on milestone

After this lands, the next sensible seam is narrower:
- deterministic refresh of manifest evidence and publication surfaces from committed runtime outputs

That is a real follow-on. Not this one.

## /autoplan Phase 1 - CEO Review

### Step 0A - Premise Challenge

#### Premise 1: The right seam is the runtime follow-on after control-plane enrollment
Assessment: Mostly valid.

`onboard-agent` and `scaffold-wrapper-crate` already draw a clean boundary in `docs/cli-agent-onboarding-factory-operator-guide.md` and `docs/specs/cli-agent-onboarding-charter.md`. The repo does have a real "middle seam" where a maintainer still translates examples into runtime code. That part is worth enclosing.

The gap in the current premise is that the note treats "Codex writes wrapper and backend code" as if that is the end of the create lane. It is not. The charter's real definition of deterministic onboarding also includes capability declaration, manifest evidence, publication refresh, and green validation. The premise should therefore be: automate the runtime lane, but make its output feed the real done-state mechanically.

#### Premise 2: `opencode` should be the default baseline
Assessment: Valid as an implementation baseline, overstated as strategy.

As a repo pattern, `opencode` is the right default reference because it exercises the harness and session-aware backend path without dragging in the much larger `codex` and `claude_code` surface areas. As a strategic policy, the current note is too rigid. The repo's own charter says semantic parity across agents is a non-goal, so the plan should frame `opencode` as the default implementation baseline, not as proof that every worthwhile agent must ship at that exact richness level.

#### Premise 3: Publication refresh and closeout should stay out of this milestone
Assessment: Valid with one missing condition.

Keeping publication refresh automation out of the same milestone preserves a bounded seam. That is good. What is missing is an explicit handoff contract that tells reviewers when the runtime lane is done enough to safely start publication refresh and closeout. Without that handoff, the milestone becomes an island.

#### Premise 4: Reviewability is the key success criterion
Assessment: Incomplete.

Reviewability matters, but by itself it is a craftsmanship metric. The plan also needs success criteria tied to actual throughput: shorter onboarding lead time, lower review time, and fewer runtime-lane regressions escaping into later publication work.

### Step 0B - Existing Code Leverage

| Sub-problem | Existing code / flow | Reuse decision |
|---|---|---|
| Control-plane to runtime boundary | `docs/cli-agent-onboarding-factory-operator-guide.md`, `docs/specs/cli-agent-onboarding-charter.md` | Reuse directly. Do not redraw the boundary. |
| Runtime checklist and path ownership | `crates/xtask/src/onboard_agent/preview/render.rs` | Reuse. The new lane should consume the same handoff truth, not restate it. |
| Approval artifact and registry-owned source paths | `crates/xtask/src/approval_artifact.rs`, `crates/xtask/src/agent_registry.rs`, `crates/xtask/data/agent_registry.toml` | Reuse. The runner should read these, not derive parallel config. |
| Default backend implementation pattern | `crates/agent_api/src/backends/opencode/**`, `crates/opencode/**` | Reuse as the default baseline template. |
| Minimal exception pattern | `crates/agent_api/src/backends/gemini_cli/**`, `crates/gemini_cli/**` | Reuse only for exception-tier outcomes. |
| Rich optional surfaces | `crates/agent_api/src/backends/codex/**`, `crates/codex/**`, `crates/agent_api/src/backends/claude_code/**`, `crates/claude_code/**` | Reuse as opt-in references, not the default template. |
| Wrapper coverage source-of-truth pattern | `crates/codex/src/wrapper_coverage_manifest.rs`, `crates/claude_code/src/wrapper_coverage_manifest.rs` | Reuse. Keep coverage truth in wrapper-crate source. |
| Manifest evidence shape | `cli_manifests/opencode/**`, `cli_manifests/gemini_cli/**` | Reuse as concrete output examples. |

Prior learning applied: `wrapper-scaffold-hardcodes-agentid-crate-path` (confidence 9, from 2026-04-23). The runtime lane should trust registry-owned crate and source paths instead of deriving new path contracts from `agent_id`.

### Step 0C - Dream State Mapping

```text
CURRENT STATE                  THIS PLAN                          12-MONTH IDEAL
approval + control-plane       one bounded runtime packet         approved agent to green,
enrollment are deterministic   that reads registry/approval       published, validated, and
but runtime implementation     truth, writes only runtime-owned   reviewable with one
still depends on maintainer    outputs, and emits a structured    deterministic lane and
judgment and example hunting   handoff into publication/closeout  measured success criteria
```

### Step 0C-bis - Implementation Alternatives

APPROACH A: Skill-Only Runtime Orchestrator
  Summary: Put the entire runtime lane into a repo-local skill that tells Codex what to read and write, with no repo command underneath.
  Effort:  S
  Risk:    Med
  Pros:
  - fastest to prototype
  - low Rust implementation overhead
  - easy to iterate on prompts
  Cons:
  - weak testability
  - execution contract lives in prompt text instead of repo-owned code
  - easier to drift from approval and registry truth
  Reuses: operator guide, approval artifact, existing backend examples

APPROACH B: Repo Command Plus Thin Skill Wrapper
  Summary: Add a repo-owned runtime packet/runner command that materializes the exact read/write contract, then keep the skill as the orchestration layer that invokes it.
  Effort:  M
  Risk:    Low
  Pros:
  - explicit, testable contract
  - easy to review and replay
  - keeps orchestration separate from source-of-truth generation
  Cons:
  - more up-front structure
  - requires command surface design
  Reuses: `xtask` patterns, onboarding preview renderers, approval artifact parsing, registry truth

APPROACH C: End-to-End Green-Lane Automation
  Summary: Extend the milestone to cover runtime packet execution, publication refresh, validation, and closeout in one end-to-end path.
  Effort:  L
  Risk:    Med
  Pros:
  - most complete user outcome
  - strongest throughput win if it lands cleanly
  - aligns directly with the create lane's real done-state
  Cons:
  - larger blast radius
  - harder to verify incrementally
  - risks collapsing too many seams at once
  Reuses: everything above plus support/capability publication and proving-run closeout flows

RECOMMENDATION: Choose APPROACH B because it preserves the bounded runtime seam while making the contract explicit, testable, and reusable.

### Step 0D - Mode-Specific Analysis

Mode: SELECTIVE EXPANSION

Complexity check:
- The eventual implementation will almost certainly touch more than 8 files and more than 2 modules. That is a smell only if the milestone tries to automate the full create lane.
- The current bounded seam is acceptable, but only if the plan stays disciplined about runtime-owned outputs and avoids pulling publication refresh into scope.

Minimum set of changes that achieves the goal:
- define the runtime packet host surface
- pin required reads
- pin allowed write targets
- pin achieved-tier summary schema
- define the reviewer checklist and success metrics

Expansion scan:
- 10x version: approved artifact to green, published backend
- delight opportunities:
  - machine-readable runtime summary
  - explicit write-boundary allowlist
  - reviewer-ready diff summary grouped by output type
  - preflight handoff checklist
  - one replayable packet for failed runtime attempts
- platform potential: the runtime packet can become the reusable middle seam for every future agent

Cherry-pick decisions:
- ACCEPT: add a green-lane handoff contract to publication refresh and closeout
- ACCEPT: add explicit success metrics
- DEFER: automate publication refresh inside the same runner
- DEFER: broader policy for which agent classes deserve onboarding

### Step 0D-POST - Persist CEO Plan

CEO plan written to:
- `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/ceo-plans/2026-04-29-runtime-follow-on-codex-lane.md`

Spec review unavailable - presenting unreviewed doc.

### Step 0E - Temporal Interrogation

- HOUR 1 (human) / first 5 minutes (CC+gstack):
  The implementer needs one packet source, one allowed write boundary, and one baseline template choice. Without that, the lane immediately degrades into reference hunting.
- HOUR 2-3 (human) / minutes 5-15 (CC+gstack):
  The ambiguity will be whether the lane is allowed to touch control-plane truth, generated publication outputs, or only runtime-owned surfaces. That must be resolved now.
- HOUR 4-5 (human) / minutes 15-25 (CC+gstack):
  The surprise will be wrapper coverage ownership. If the packet does not name the registry-owned source path explicitly, contributors will try to edit generated `wrapper_coverage.json` or infer the wrong source file.
- HOUR 6+ (human) / minutes 25-45 (CC+gstack):
  The team will wish the plan had specified a structured achieved-tier summary, failure taxonomy, and handoff checklist for publication refresh and proving-run closeout.

### Step 0F - Mode Selection Confirmation

Selected mode: SELECTIVE EXPANSION

Why:
- the user already bounded the seam correctly
- the plan still benefits from a few adjacent additions that are in blast radius and cheap
- the full end-to-end green lane is better as a follow-on milestone than a silent scope creep inside this one

Chosen implementation approach under this mode:
- APPROACH B: Repo command plus thin skill wrapper

### Step 0.5 - Dual Voices

#### CLAUDE SUBAGENT (CEO - strategic independence)
Unavailable in this session. Session policy does not permit delegating to a subagent without an explicit user request for delegation.

#### CODEX SAYS (CEO - strategy challenge)
- The current note optimizes a code-writing subroutine, not the full trust boundary of the create lane.
- `opencode` as a hard strategic baseline risks overfitting to internal convenience instead of actual agent value.
- Deferring publication refresh and closeout is fine only if the runtime lane emits a crisp handoff into the real done-state.
- Reviewability alone is not a sufficient milestone metric; the plan needs throughput and defect metrics.
- The plan lacks a portfolio thesis for when onboarding should be refused or deprioritized.

#### CEO DUAL VOICES - CONSENSUS TABLE

```text
═══════════════════════════════════════════════════════════════
  Dimension                           Claude  Codex  Consensus
  ──────────────────────────────────── ─────── ─────── ─────────
  1. Premises valid?                  N/A     Mixed   N/A
  2. Right problem to solve?          N/A     Yes*    N/A
  3. Scope calibration correct?       N/A     Mixed   N/A
  4. Alternatives sufficiently explored? N/A  No     N/A
  5. Competitive/market risks covered?N/A     No     N/A
  6. 6-month trajectory sound?        N/A     Mixed   N/A
═══════════════════════════════════════════════════════════════
```

`Yes*` means Codex agreed the runtime seam is real, but argued the plan must explicitly connect runtime output to the full create-lane done-state.

### Section 1 - Architecture Review

What I examined:
- the milestone note
- the operator-guide runtime boundary
- the onboarding charter
- `crates/xtask/src/onboard_agent/preview/render.rs`
- existing wrapper/backend and wrapper-coverage patterns

Findings:
- The current plan has the right top-level boundary but no pinned host surface. Without deciding where the runtime packet actually lives, the milestone is still prose, not architecture.
- The cleanest architecture is a repo-owned command plus a thin skill wrapper. That keeps prompt orchestration out of the contract surface and allows direct tests against the runner.
- The plan must define a hard write allowlist: `crates/<agent_id>/**`, `crates/agent_api/src/backends/<agent_id>/**`, registry-owned wrapper coverage source path, and `cli_manifests/<agent_id>/**`. Everything else is read-only unless a future milestone says otherwise.

Required ASCII diagram:

```text
approved-agent.toml
        │
        ▼
  onboard-agent --write
        │
        ▼
  scaffold-wrapper-crate --write
        │
        ▼
  runtime packet / runner
    │      │        │        │
    │      │        │        └──▶ cli_manifests/<agent_id>/**
    │      │        └────────────▶ wrapper coverage source path
    │      └─────────────────────▶ crates/agent_api/src/backends/<agent_id>/**
    └────────────────────────────▶ crates/<agent_id>/**
        │
        ▼
 achieved-tier summary + green-lane handoff
        │
        ▼
 publication refresh / validation / closeout (follow-on lane)
```

Auto-decision:
- Accept the runtime seam, but expand the plan to require a host surface decision and a write-boundary allowlist.

### Section 2 - Error & Rescue Map

The current note does not define runtime-lane failures at all. That is the biggest gap before implementation.

#### Error & Rescue Registry

| Method / Codepath | What can go wrong | Exception / Failure class | Rescued? | Rescue action | User sees |
|---|---|---|---|---|---|
| runtime packet bootstrap | missing approval artifact or packet path | `MissingInput` | Y | fail fast with exact missing path | clear error with required input list |
| runtime packet bootstrap | registry path disagrees with approval artifact | `PathContractMismatch` | Y | stop before writes, print expected vs actual | clear contract mismatch |
| baseline selection | requested `minimal` without justification | `TierPolicyViolation` | Y | reject output and require explicit rationale field | explicit policy failure |
| Codex execution | writes outside allowed runtime targets | `WriteBoundaryViolation` | Y | reject run, list offending paths | run rejected before review |
| backend implementation | compile or typecheck failure | `BuildFailure` | Y | keep summary, mark lane incomplete, preserve logs | explicit build failure |
| wrapper coverage update | generated JSON edited instead of source path | `CoverageSourceViolation` | Y | reject and point to source path | explicit coverage-source failure |
| manifest evidence generation | missing `current.json`, pointers, or reports | `ManifestEvidenceIncomplete` | Y | fail checklist with missing outputs | explicit missing evidence |
| summary emission | no achieved-tier summary or missing deferred-surface list | `SummaryContractViolation` | Y | reject output as unreviewable | explicit summary contract failure |

The plan should rescue all of these with fail-fast, reviewer-visible errors. Silent partial output is unacceptable.

### Section 3 - Security & Threat Model

The new attack surface is not external user auth. It is automation trust.

Findings:
- Without a write-boundary allowlist, the runtime lane can mutate control-plane truth or generated publication files and make review provenance muddy.
- Without explicit "no live secrets by default" guidance, Codex may attempt real CLI calls instead of staying on fixture/fake-binary or bounded evidence paths.
- The runner should reject unknown extension or richer-surface requests unless the packet or baseline explicitly allows them. This mirrors the backend harness fail-closed philosophy.

Threat summary:

| Threat | Likelihood | Impact | Mitigation status |
|---|---|---|---|
| write outside runtime-owned surfaces | Med | High | GAP - add allowlist |
| accidental live credential use during runtime generation | Med | High | GAP - require default offline or fixture-first posture |
| richer-surface drift sneaking into default baseline | High | Med | GAP - require explicit achieved-tier summary and deferred-surface list |

### Section 4 - Data Flow & Interaction Edge Cases

This is a docs-and-runner lane, not a UI feature, but it still has edge cases that matter.

```text
INPUT ──▶ VALIDATION ──▶ TEMPLATE SELECTION ──▶ CODE WRITES ──▶ EVIDENCE WRITES ──▶ SUMMARY
  │            │                │                    │                 │                │
  ▼            ▼                ▼                    ▼                 ▼                ▼
missing?   wrong pack?     invalid tier?       wrong paths?      partial set?     empty summary?
stale?     stale scaffold? richer leak?        compile fail?     stale pointers?  missing defer list?
```

Unhandled edge cases in the current note:
- existing agent already scaffolded but runtime lane re-run
- hyphenated or nontrivial `crate_path` and source-path handling
- partial success where code changes exist but manifest evidence does not
- rerun after one failed attempt without a replayable packet or preserved summary

Auto-decision:
- Add explicit rerun semantics and partial-output handling to the plan. Preserve failure summaries instead of asking operators to reconstruct what happened.

### Section 5 - Code Quality Review

The current plan is directionally good, but it risks duplicating code-owned truth in prose.

Findings:
- The plan should not restate crate paths, manifest roots, or wrapper-coverage source paths by hand when those already exist in approval and registry truth.
- A skill-only solution would hide the contract in prompt text and weaken tests. That is clever in the wrong way. The explicit path is a repo command plus thin orchestration wrapper.
- The achieved-tier summary needs a pinned shape. Freeform prose will drift and make review slower over time.

Auto-decision:
- Reuse existing onboarding and registry truth surfaces wherever possible.
- Add a machine-readable summary shape alongside the human-readable summary.

### Section 6 - Test Review

The milestone note currently names review goals, but it does not yet define the test surface needed to trust the runtime lane.

Coverage diagram for the future implementation:

```text
CODE PATH COVERAGE
===========================
[+] Runtime packet assembly
    ├── [GAP] approval + handoff inputs resolved from repo truth
    ├── [GAP] registry/approval mismatch rejected
    └── [GAP] write-boundary allowlist enforced

[+] Baseline selection
    ├── [GAP] default tier uses opencode reference set
    ├── [GAP] minimal tier requires explicit justification
    └── [GAP] feature-rich surfaces require explicit opt-in

[+] Runtime output validation
    ├── [GAP] wrapper coverage source updated at source path, not generated JSON
    ├── [GAP] manifest evidence set complete enough for later refresh
    └── [GAP] achieved-tier summary contains all required fields

USER FLOW COVERAGE
===========================
[+] Maintainer runs runtime lane after scaffold
    ├── [GAP] successful default-tier path
    ├── [GAP] partial failure with preserved summary and no silent success
    └── [GAP] rerun after failed attempt

CRITICAL GAPS
=============
- no contract test for allowed write targets
- no contract test for minimal-exception rejection
- no test for coverage-source vs generated-json ownership
```

Test plan requirements that must be added to implementation:
- unit tests for packet assembly and path validation
- unit tests for tier policy enforcement
- integration tests for write-boundary rejection
- integration tests for achieved-tier summary schema
- fixture-based tests proving the runner can succeed on a default-tier reference path without a real CLI

### Section 7 - Performance Review

No significant runtime performance concern is visible in the plan itself. This is primarily a deterministic orchestration and validation lane.

What I examined:
- whether the plan proposed live discovery, repeated heavy scans, or large generated artifacts in the hot path
- whether it bundled publication refresh or large matrix generation into the same step

Why nothing was flagged:
- the bounded runtime lane is not latency-sensitive in the same way as a user-facing request path
- the main risk is wasted maintainer time from failed or ambiguous runs, not CPU or memory saturation

### Section 8 - Observability & Debuggability Review

This section matters a lot because failed runtime attempts will be expensive to reconstruct if the runner emits only prose.

Findings:
- The plan needs a structured run-status artifact or equivalent summary with: input refs, template chosen, files written, validation checks run, failures, and deferred richer surfaces.
- The runner should preserve failure summaries on unsuccessful runs. Operators need evidence, not "try again."
- Reviewers need a diff summary grouped by output class: wrapper code, backend code, wrapper coverage source, manifest evidence.

Auto-decision:
- Add structured run output and preserved failure logs to the plan.

### Section 9 - Deployment & Rollout Review

This milestone is not a production feature rollout, but it does have rollout risk inside the operator workflow.

Findings:
- The operator guide should remain unchanged until the runtime lane is proven on at least one real onboarding target. The backlog-only status is correct today.
- The runner should be introduced as experimental or branch-local first. The rollback posture is simple: stop using the runner and continue with the current manual runtime follow-on.

Deployment sequence:

```text
plan + runner contract land
        │
        ▼
prove on one real onboarding target
        │
        ▼
validate runtime summary + evidence quality
        │
        ▼
decide whether to promote into shipped operator workflow
```

### Section 10 - Long-Term Trajectory Review

Current reversibility: 4/5. The milestone is reversible as long as the operator guide remains the canonical procedure and the runner is treated as an additive execution aid.

Debt risk:
- if the host surface is skill-only, prompt drift becomes technical debt fast
- if success metrics are absent, the repo may celebrate reviewability while missing throughput gains
- if the plan never defines when to refuse low-value agents, the factory may optimize the wrong portfolio

Six-month trajectory:
- good if the lane becomes packetized, measured, and reusable
- bad if it becomes a one-off code-writing macro with weak output contracts

### Section 11 - Design & UX Review

Skipped. No UI scope detected in the plan text, and the milestone is not about rendered product surfaces.

## NOT in scope

- Automating publication refresh inside the same milestone
  Rationale: keep the runtime seam bounded; handle the green-lane closure next.
- Automating proving-run closeout inside the same milestone
  Rationale: closeout depends on runtime evidence and publication validation being trustworthy first.
- Rewriting `onboard-agent` to generate runtime code directly
  Rationale: violates the control-plane/runtime ownership split.
- Broad portfolio policy for which agents deserve onboarding
  Rationale: strategically important, but larger than this seam.

## What already exists

- `onboard-agent` already owns registry/docs/manifest/release enrollment.
- `scaffold-wrapper-crate` already owns the minimal wrapper shell.
- `onboard_agent` preview renderers already encode the remaining runtime checklist.
- `approval_artifact.rs` and `agent_registry.rs` already pin wrapper coverage source paths and onboarding pack ownership.
- `opencode`, `gemini_cli`, `codex`, and `claude_code` already provide concrete reference implementations across the default, minimal, and feature-rich tiers.
- Existing wrapper coverage manifest sources already show the correct pattern for registry-owned coverage truth staying in wrapper-crate code.

## Dream state delta

If the current note lands unchanged, the repo gets a better-written runtime recipe.

If the CEO-reviewed version lands, the repo gets:
- a bounded runtime seam
- a real host surface decision
- a write-boundary contract
- a machine-readable achieved-tier summary
- a handoff contract into the actual green lane
- measurable success criteria

That still is not the 12-month ideal, but it moves toward it cleanly instead of creating a docs-only cul-de-sac.

## Failure Modes Registry

| Codepath | Failure mode | Rescued? | Test? | User sees? | Logged? |
|---|---|---:|---:|---|---:|
| packet bootstrap | missing approval artifact | Y | N | clear input error | N |
| packet bootstrap | registry path mismatch | Y | N | clear contract error | N |
| baseline selection | minimal tier without justification | Y | N | explicit policy error | N |
| runtime execution | writes outside allowed paths | Y | N | run rejected | N |
| runtime execution | compile/typecheck failure | Y | N | explicit build failure | N |
| coverage update | generated JSON edited instead of source path | Y | N | explicit source-path failure | N |
| evidence write | partial manifest evidence only | Y | N | incomplete-run failure | N |
| summary emission | summary missing required fields | Y | N | unreviewable-run failure | N |

Any row with `Test? = N` and `Logged? = N` is a review gap to fix during implementation planning.

## Completion Summary

```text
+====================================================================+
|            MEGA PLAN REVIEW - COMPLETION SUMMARY                   |
+====================================================================+
| Mode selected        | SELECTIVE EXPANSION                         |
| Step 0               | runtime seam accepted, host surface needed  |
| Section 1  (Arch)    | 3 issues found                              |
| Section 2  (Errors)  | 8 error paths mapped, 0 silent rescues ok   |
| Section 3  (Security)| 3 issues found, 2 high impact               |
| Section 4  (Data/UX) | 4 edge cases mapped, 4 unhandled            |
| Section 5  (Quality) | 3 issues found                              |
| Section 6  (Tests)   | Diagram produced, 7 major gaps              |
| Section 7  (Perf)    | 0 issues found                              |
| Section 8  (Observ)  | 3 gaps found                                |
| Section 9  (Deploy)  | 2 risks flagged                             |
| Section 10 (Future)  | Reversibility: 4/5, debt items: 3           |
| Section 11 (Design)  | SKIPPED (no UI scope)                       |
+--------------------------------------------------------------------+
| NOT in scope         | written (4 items)                           |
| What already exists  | written                                     |
| Dream state delta    | written                                     |
| Error/rescue registry| 8 methods, 0 silent-failure accepts         |
| Failure modes        | 8 total, 0 accepted critical silent gaps    |
| TODOS.md updates     | 0 proposed in Phase 1                       |
| Scope proposals      | 4 proposed, 2 accepted                      |
| CEO plan             | written                                     |
| Outside voice        | ran (codex-only)                            |
| Diagrams produced    | 4 (system, data flow, deployment, dream)    |
| Unresolved decisions | 3 (premises gate below)                     |
+====================================================================+
```

<!-- AUTONOMOUS DECISION LOG -->
## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
|---|---|---|---|---|---|---|
| 1 | CEO | Use SELECTIVE EXPANSION mode | Mechanical | P2 + P3 | The seam is already bounded, but a few cheap adjacent fixes improve completeness | HOLD SCOPE, EXPANSION |
| 2 | CEO | Prefer repo command plus thin skill wrapper | Taste | P5 | Contract surfaces belong in repo-owned code, not only in prompt text | Skill-only runner |
| 3 | CEO | Keep runtime seam separate from `onboard-agent` | Mechanical | P4 + P5 | Preserves the established control-plane/runtime boundary | Folding runtime generation into control-plane command |
| 4 | CEO | Accept green-lane handoff contract into scope | Mechanical | P1 + P2 | It is in blast radius and prevents a local optimization trap | Leaving runtime output disconnected from done-state |
| 5 | CEO | Accept success metrics into scope | Mechanical | P1 | Reviewability alone is insufficient as a milestone outcome | Doc-only acceptance criteria |
| 6 | CEO | Defer publication refresh automation | Mechanical | P3 | Valuable, but a follow-on milestone once runtime outputs are deterministic | Bundling refresh into this milestone |
| 7 | CEO | Treat `opencode` as default implementation baseline, not universal product bar | Taste | P3 + P5 | Keeps the implementation default while avoiding overclaiming parity policy | Hard strategic baseline for every agent |
| 8 | CEO | Require write-boundary allowlist and offline-first posture | Mechanical | P1 + P5 | Automation trust is the primary security boundary here | Implicit or reviewer-only enforcement |
| 9 | CEO | Require structured achieved-tier summary | Mechanical | P5 | Review must be explicit and machine-checkable | Freeform prose summary only |
| 10 | CEO | Preserve failed-run summaries and rerun semantics | Mechanical | P1 | Failures will be common during onboarding and must be debuggable | Silent partial output or "try again" workflow |

## /autoplan Phase 2 - Design Review

Skipped. No UI scope detected in the plan text, and no rendered product surface is being introduced by this milestone.

## /autoplan Phase 3 - Eng Review

### Step 0 - Scope Challenge

#### What existing code already partially or fully solves each sub-problem?

| Sub-problem | Existing code / flow | Reuse decision |
|---|---|---|
| `codex exec`-driven orchestration with pinned output files | `.codex/skills/recommend-next-agent/SKILL.md`, `scripts/recommend_next_agent.py`, `scripts/spawn_worker.py` | Reuse the pattern. The runtime runner should look like a deterministic Codex lane with pinned artifacts, not an ad hoc prompt. |
| Approval and path truth | `crates/xtask/src/approval_artifact.rs`, `crates/xtask/src/agent_registry.rs` | Reuse directly as machine-owned truth. Do not promote generated packet markdown into executable authority. |
| Runtime checklist wording | `crates/xtask/src/onboard_agent/preview/render.rs` | Reuse as evidence and handoff language, not as a second authority surface. |
| Safe repo-relative write enforcement | `crates/xtask/src/workspace_mutation.rs` | Reuse the same jail semantics for any runner-managed file writes or validations. |
| Default backend wiring pattern | `crates/agent_api/src/backends/opencode/**`, `crates/opencode/**` | Reuse as the default implementation baseline. |
| Minimal exception wiring pattern | `crates/agent_api/src/backends/gemini_cli/**`, `crates/gemini_cli/**` | Reuse as the exception-tier reference only. |
| Wrapper coverage source-of-truth pattern | `crates/codex/src/wrapper_coverage_manifest.rs`, `crates/claude_code/src/wrapper_coverage_manifest.rs` | Reuse. Generated JSON remains derived output. |
| Required `agent_api` backend test posture | `crates/agent_api/tests/**` and existing C1/C2-style tests | Reuse. Default-tier runtime onboarding must include `agent_api` tests. |

Prior learning applied: `approval-artifact-pack-prefix-mismatch` (confidence 9, from 2026-04-21). The runner should trust the validated approval artifact parser, not restate onboarding pack path logic itself.

Prior learning applied: `wrapper-scaffold-hardcodes-agentid-crate-path` (confidence 9, from 2026-04-23). The runner should always source crate and coverage paths from registry and approval truth, not derive them from `agent_id`.

#### Minimum set of changes that achieves the goal

- Define the normative host surface as a `codex exec` runner whose runtime skill is baked in or explicitly loaded by the runner.
- Split machine-owned inputs from reviewer-evidence docs.
- Expand the write boundary to include required `agent_api` test files for default-tier onboarding.
- Split runtime-owned manifest evidence from publication-owned manifest state.
- Define a machine-readable summary and handoff artifact.

#### Complexity check

This future implementation will touch more than 8 files and more than 2 modules. That is acceptable only because the lane spans wrapper crate code, backend adapter code, `agent_api` tests, wrapper coverage source, manifest evidence, and runner artifacts. Trying to hide that complexity behind a smaller write boundary would create a fake-complete plan.

#### Search check

Search unavailable for external best-practice validation inside this review pass, proceeding with in-distribution repository knowledge only.

#### Completeness check

The current plan was still a shortcut. It bounded the runtime lane, but it under-described the required test surface and over-trusted generated packet prose. The complete version keeps the same seam but adds the missing contract details instead of pretending the shorter note is good enough.

#### Distribution check

No new end-user artifact type is being introduced. The runner is an internal repo workflow surface, not a published binary or package.

### Step 0.5 - Dual Voices

#### CLAUDE SUBAGENT (eng - independent review)
Unavailable in this session. Session policy does not permit delegating to a subagent without an explicit user request for delegation.

#### CODEX SAYS (eng - architecture challenge)
- Generated onboarding packet docs must remain reviewer evidence, not executable truth.
- The write boundary is too narrow if default-tier onboarding is supposed to satisfy the charter's required `agent_api` tests.
- `cli_manifests/<agent_id>/` needs an internal split between runtime-owned evidence and publication-owned promotion state.
- Tier labels need to map to concrete capability and audit rules, not only prose.

#### ENG DUAL VOICES - CONSENSUS TABLE

```text
═══════════════════════════════════════════════════════════════
  Dimension                           Claude  Codex  Consensus
  ──────────────────────────────────── ─────── ─────── ─────────
  1. Architecture sound?              N/A     Mixed   N/A
  2. Test coverage sufficient?        N/A     No      N/A
  3. Performance risks addressed?     N/A     Mostly  N/A
  4. Security threats covered?        N/A     Mixed   N/A
  5. Error paths handled?             N/A     No      N/A
  6. Deployment risk manageable?      N/A     Yes*    N/A
═══════════════════════════════════════════════════════════════
```

`Yes*` means deployment risk is manageable only if the runner remains backlog-only until proven on a real onboarding target.

### Section 1 - Architecture Review

The architecture is now good enough only if the host surface is made explicit.

The reviewed plan should define the normative host surface as:
- a repo-owned `codex exec` runner
- with the runtime skill baked into the runner payload or explicitly loaded by the runner
- producing deterministic scratch/review artifacts similar to the recommendation lane
- consuming approval and registry truth as machine inputs

The skill is part of the execution contract, not ambient local state.

#### Architecture ASCII Diagram

```text
approved-agent.toml + agent_registry.toml
                │
                ├──────────────▶ path / tier / capability truth
                │
                ▼
      runtime-runner input assembly
                │
                ├──────────────▶ operator-guide + charter + ADR-0013
                │
                ├──────────────▶ runtime skill payload
                │                 (baked in or explicitly loaded)
                ▼
       codex exec runtime runner
                │
     ┌──────────┼──────────┬──────────────┬────────────────────┐
     │          │          │              │                    │
     ▼          ▼          ▼              ▼                    ▼
crates/<id>  crates/agent_api/   crates/agent_api/   wrapper coverage   cli_manifests/<id>/
runtime      src/backends/<id>/  tests/**            source path        runtime-owned evidence only
code         backend code        required C1/C2
                                 onboarding tests
     │
     ▼
run-status.json + run-summary.md + green-lane handoff artifact
     │
     ▼
publication refresh / validation / closeout
(next milestone, separate owner seam)
```

#### Architecture findings

- The generated onboarding handoff packet must be downgraded from required executable input to reviewer evidence only. Approval artifact and registry truth are the machine-owned sources.
- The write boundary must include `crates/agent_api/tests/**` for default-tier onboarding. Otherwise the plan cannot meet the charter's required backend test posture.
- `cli_manifests/<agent_id>/` must be subdivided. Runtime lane writes only runtime-owned evidence, not publication-owned pointers, root snapshots, or matrix outputs.
- The runner should emit a machine-readable handoff artifact for the next lane instead of relying on prose summary alone.

### Section 2 - Code Quality Review

The plan is much stronger now, but three quality traps remain.

1. Dual authority trap:
   If packet markdown is an executable input, the plan creates two truth systems. That is unnecessary duplication.
2. Hidden ambient dependency trap:
   If the runtime skill is not baked in or explicitly loaded by the runner, the lane becomes environment-sensitive and non-replayable.
3. Freeform summary trap:
   The achieved-tier summary needs a pinned schema, not just narrative fields in markdown.

Recommended machine-readable summary shape:
- `run-status.json`
  - `run_id`
  - `workflow_version`
  - `approval_artifact_path`
  - `agent_id`
  - `host_surface`
  - `loaded_skill_ref`
  - `tier_requested`
  - `tier_achieved`
  - `primary_template`
  - `written_paths`
  - `validation_checks`
  - `handoff_ready`
  - `errors`
- `run-summary.md`
  - human-readable review summary only

### Section 3 - Test Review

#### Test diagram mapping codepaths to coverage

```text
NEW OPERATOR FLOWS
===========================
[+] Runtime runner after onboard + scaffold
    ├── [GAP] Happy path default-tier onboarding
    ├── [GAP] Minimal-tier request rejected without justification
    ├── [GAP] Rerun after partial failure preserves prior summary
    └── [GAP] Runtime handoff artifact marks publication lane readiness

NEW DATA FLOWS
===========================
[+] approval artifact + registry -> runner input assembly
    ├── [GAP] Path mismatch rejected
    ├── [GAP] Pack-prefix mismatch rejected
    └── [GAP] Capability/tier contract materialized into summary

[+] runner -> codex exec with loaded runtime skill
    ├── [GAP] Skill is explicitly loaded or baked in
    └── [GAP] Ambient local skill discovery not required

[+] runner -> runtime-owned writes
    ├── [GAP] wrapper crate writes allowed
    ├── [GAP] backend adapter writes allowed
    ├── [GAP] `agent_api` onboarding tests allowed
    ├── [GAP] wrapper coverage source path allowed
    └── [GAP] publication-owned files rejected

NEW CODEPATHS / BRANCHES
===========================
[+] Tier policy
    ├── [GAP] default -> opencode template
    ├── [GAP] minimal -> explicit exception
    └── [GAP] feature-rich -> opt-in references only

[+] Manifest evidence split
    ├── [GAP] runtime-owned evidence files accepted
    └── [GAP] publication-owned pointers/current snapshots rejected

NEW ERROR / RESCUE PATHS
===========================
[+] MissingInput
    └── [GAP] test exact missing-path error
[+] WriteBoundaryViolation
    └── [GAP] test write rejection with offending path list
[+] TierPolicyViolation
    └── [GAP] test rejection without justification
[+] CoverageSourceViolation
    └── [GAP] test generated JSON edit rejection
[+] SummaryContractViolation
    └── [GAP] test incomplete summary rejection
```

Coverage today in the plan: incomplete. The reviewed plan must require:
- unit tests for approval/registry input assembly and mismatch rejection
- integration tests for write-boundary allowlist enforcement
- integration tests for explicit skill-loading contract
- integration tests for runtime-owned vs publication-owned manifest file split
- integration tests for summary-schema validation
- fixture-based `agent_api` backend tests proving default-tier onboarding is charter-complete

#### Test plan artifact

Written to:
- `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-eng-review-test-plan-20260429-135452.md`

### Section 4 - Performance Review

This lane is not user-latency-sensitive, but it can still waste maintainer time badly if the contract is loose.

Performance findings:
- Avoid scanning whole manifest roots when only runtime-owned evidence files matter in this milestone.
- Keep runner artifacts bounded and deterministic, mirroring the recommendation lane pattern instead of inventing a larger scratch surface.
- Reuse existing path-jail and approval parsing logic instead of reparsing the repo state in multiple places.

No database or memory hotspot concerns were identified in the current plan text.

## Phase 3 - Required Outputs

### Additional "NOT in scope" clarifications

- Publication-owned pointer promotion under `cli_manifests/<agent_id>/`
  Rationale: belongs to the next lane after runtime evidence is complete.
- Matrix publication surfaces such as `cli_manifests/support_matrix/current.json` and `docs/specs/unified-agent-api/*.md`
  Rationale: publication truth remains outside this milestone.

### Updated Failure Modes Registry

| Codepath | Failure mode | Rescued? | Test? | User sees? | Logged? |
|---|---|---:|---:|---|---:|
| runner input assembly | generated packet markdown disagrees with approval/registry truth | Y | N | explicit machine-truth precedence error | N |
| runtime write set | attempt to write outside allowed runtime paths | Y | N | explicit offending-path rejection | N |
| default-tier lane | required `agent_api` tests omitted | Y | N | lane incomplete, not green-handoff-ready | N |
| manifest evidence write | publication-owned manifest state edited by runtime lane | Y | N | explicit seam-violation error | N |
| skill loading | runner depends on ambient skill presence | Y | N | explicit missing skill payload/load contract error | N |
| summary generation | missing handoff artifact or tier fields | Y | N | explicit unreviewable output error | N |

### Phase 3 Completion Summary

```text
+====================================================================+
|                  ENG REVIEW - COMPLETION SUMMARY                   |
+====================================================================+
| Step 0               | scope accepted, contract tightened          |
| Architecture Review  | 4 issues found                              |
| Code Quality Review  | 3 issues found                              |
| Test Review          | diagram produced, 12 major gaps            |
| Performance Review   | 2 boundedness issues found                 |
+--------------------------------------------------------------------+
| NOT in scope         | updated                                     |
| What already exists  | reused                                      |
| TODOS.md updates     | publication follow-on added                 |
| Test plan artifact   | written                                     |
| Outside voice        | ran (codex-only)                            |
+====================================================================+
```

## Cross-Phase Themes

**Theme: Machine truth over prose** - flagged in Phase 1 and Phase 3.
The plan must keep approval artifact and registry truth authoritative, with packet docs and summaries as evidence or projection only.

**Theme: Bounded seam, but complete handoff** - flagged in Phase 1 and Phase 3.
The runtime lane should stay separate from publication refresh, but its outputs must be machine-readable and sufficient for the next lane to start without archaeology.

**Theme: Default tier must be contract-complete** - flagged in Phase 1 and Phase 3.
Calling something `default` is not enough. The write boundary, required tests, and capability posture must make that tier real.

## /autoplan Pre-Gate Verification Notes

- CEO outputs: complete
- Design outputs: skipped correctly, no UI scope
- Eng outputs: complete after the additions above
- Audit trail: non-empty

### Phase 3 Decisions - Decision Audit Trail Additions

| 11 | Eng | Make the normative host surface a `codex exec` runner with baked-in or explicitly loaded runtime skill | Mechanical | P5 | Removes ambient-environment ambiguity and keeps the contract explicit | Generic repo command, ambient skill discovery |
| 12 | Eng | Use approval artifact + registry as machine truth; packet docs are reviewer evidence only | Mechanical | P4 + P5 | Avoids dual-authority drift between generated prose and executable truth | Generated packet markdown as required executable input |
| 13 | Eng | Expand the write boundary to include `crates/agent_api/tests/**` for default-tier onboarding | Mechanical | P1 | The charter requires backend onboarding tests; the smaller boundary was incomplete | Narrow runtime-only writes with silent test omission |
| 14 | Eng | Split `cli_manifests/<agent_id>/` into runtime-owned evidence vs publication-owned state | Mechanical | P1 + P5 | Keeps the publication seam deferred without making manifest writes ambiguous | Whole-root write permission |
| 15 | Eng | Require machine-readable `run-status.json` plus human `run-summary.md` | Mechanical | P5 | Review and replay must be deterministic | Freeform markdown summary only |
| 16 | Eng | Add publication-refresh follow-on to `TODOS.md` | Mechanical | P2 | Keeps the green-lane closure explicitly captured without expanding this milestone | Implicit follow-on only in prose |

## /autoplan Final Approval

Approval date: 2026-04-29

User approval outcome:
- Choice 1 approved: keep the CEO default-tier framing, with `opencode` as the default implementation baseline rather than a universal product bar.
- Choice 2 approved with clarification: "repo command plus thin skill wrapper" means a repo-owned `codex exec` runner whose runtime skill is baked in or explicitly loaded by the runner.
- Final gate outcome: `A` approved as-is after the clarification above was folded into the reviewed plan.

Approval effect:
- This backlog note is now the approved implementation-planning record for `uaa-0022`.
- Publication refresh remains explicitly deferred to the follow-on milestone captured in `TODOS.md`.
