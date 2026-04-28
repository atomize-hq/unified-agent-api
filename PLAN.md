<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/codex-recommend-next-agent-autoplan-restore-20260428-073823.md -->
# PLAN — LLM-Guided Research Orchestration For The Next CLI Agent

Status: ready for implementation  
Date: 2026-04-28  
Branch: `codex/recommend-next-agent`  
Repo: `atomize-hq/unified-agent-api`

## Source Inputs

- Prior approved design artifact:
  - `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-design-20260427-151419.md`
- Validation artifact for the shipped lane:
  - `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommendation-lane-validation-20260428-071743.md`
- Eng-review test artifact for the shipped lane:
  - `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-eng-review-test-plan-20260427-153026.md`
- Live repo surfaces:
  - `.codex/skills/recommend-next-agent/SKILL.md`
  - `scripts/recommend_next_agent.py`
  - `docs/agents/selection/candidate-seed.toml`
  - `docs/agents/selection/cli-agent-selection-packet.md`
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
- Normative contracts:
  - `docs/specs/cli-agent-onboarding-charter.md`
  - `docs/templates/agent-selection/cli-agent-selection-packet-template.md`
  - `crates/xtask/src/approval_artifact.rs`
  - new doc to add in this milestone: `docs/specs/cli-agent-recommendation-dossier-contract.md`

## Outcome

Build the next milestone of the recommendation lane so the repo-local skill becomes the real research workflow and the Python runner becomes the deterministic normalization, validation, render, and promotion engine.

The intended maintainer experience is:

1. Provide seed hints and shortlist intent.
2. Invoke the repo-local skill.
3. The LLM agent performs bounded candidate research using web search, official docs, package registries, GitHub metadata, and repo-fit reasoning.
4. The skill writes structured research proof for each candidate.
5. The deterministic runner optionally executes only allowlisted non-mutating local probes, validates the proof fields, rejects incomplete candidates before scoring, renders the canonical packet, drafts the approval artifact, and promotes the approved run.
6. The existing `cargo run -p xtask -- onboard-agent --approval ...` lane begins unchanged.

This plan does not add runtime wrapper/backend implementation for a chosen new agent. It makes the recommendation lane trustworthy enough that the eventual winner is backed by actual research rather than keyword heuristics.

## Challenge Resolution

The outside strategy review challenged whether a lighter finalist viability sprint should come first.

User decision on 2026-04-28:

- stay on the requested path
- implement the LLM-guided research milestone now
- keep the lighter finalist sprint as a documented alternative, not the active plan

## Premises

1. The missing product is not a better heuristic scorer. It is an LLM-guided research workflow that captures approval-grade proof before the deterministic engine runs.
2. The skill should own exploration and qualitative synthesis. The script should own contract enforcement, stable serialization, and repo mutation.
3. Human approve-or-override remains the product boundary. The system should make that decision obvious, not automatic.
4. The recommendation lane should stay pre-create and non-mutating until explicit promotion.
5. Full candidate CLI execution is not required for this milestone. Safe, allowlisted, non-mutating local probes are enough when available; provider-backed integration belongs to the later onboarding proving run.

## Scope Lock

### In scope

- Rewrite `.codex/skills/recommend-next-agent/SKILL.md` into a true research workflow for AI agents
- Add one normative dossier contract at `docs/specs/cli-agent-recommendation-dossier-contract.md`
- Add a structured research-proof contract that the skill must populate before the runner scores candidates
- Narrow `scripts/recommend_next_agent.py` to deterministic validation, scoring, rendering, and promotion over structured research inputs
- Upgrade the eligibility gate so candidates fail before scoring when proof is missing for non-interactive execution, fixture/offline strategy, redaction fit, crate-first fit, or reproducibility
- Upgrade the canonical packet renderer so the packet itself becomes the maintainer decision document
- Preserve the current `generate` / `promote` split and the existing `approved-agent.toml` handoff
- Freeze seed/default snapshots into each run so promote cannot drift from what was reviewed
- Add durable run-status artifacts for fatal and partial failures
- Add source budgets, size caps, and caching rules for research evidence
- Add tests that prove the richer packet shape and the proof-backed gate
- Add one proving run using the new research-assisted workflow

### Out of scope

- Full live provider-backed candidate execution during recommendation
- `xtask recommend-agent`
- New control-plane crates
- Dynamic candidate-count configuration
- Wrapper or `agent_api` implementation for the eventual winning agent
- Post-onboarding maintenance or upgrade-lane redesign

## What Already Exists

- The shipped skill exists, but it is thin orchestration around shell commands.
- The shipped runner already owns fetch, normalization, scoring, packet rendering, approval draft rendering, dry-run validation, and promote-time swap safety.
- The repo already has a canonical packet path and a packet template with the exact section contract.
- The repo already has a strict approval artifact loader and a stable create-mode handoff through `onboard-agent`.
- The current proving run already showed the real gap: the recommendation lane can promote artifacts mechanically, but it still makes judgments from heuristic signals instead of explicit research proof.

## NOT In Scope

- Solving the eventual winner's wrapper architecture
- Solving the eventual winner's backend mapping
- Solving candidate ranking for arbitrary external repos
- Solving upgrade recommendations for already-onboarded agents
- Solving live auth-gated evaluation before the onboarding proving run

## Existing Code Leverage Map

| Sub-problem | Existing surface to reuse | Why it stays |
| --- | --- | --- |
| Approval contract | `crates/xtask/src/approval_artifact.rs` | Already owns path, schema, and dry-run validation truth |
| Post-approval create lane | `crates/xtask/src/onboard_agent.rs` and `docs/cli-agent-onboarding-factory-operator-guide.md` | Already landed, do not reopen |
| Canonical packet shape | `docs/templates/agent-selection/cli-agent-selection-packet-template.md` | Already encodes the maintainer-facing packet contract |
| Seed pool and descriptor defaults | `docs/agents/selection/candidate-seed.toml` | Already the right place for repo-owned defaults and shortlist hints |
| Promote-time safety | `scripts/recommend_next_agent.py` current staging + rollback flow | Already good, preserve it |
| Review evidence roots | `docs/agents/selection/runs/<run_id>/` and `~/.gstack/projects/.../recommend-next-agent-runs/<run_id>/` | Already the right split between scratch and committed evidence |

## Dream State

```text
CURRENT
  Thin skill -> heuristic runner -> canonical packet/approval draft
  Good mechanics, weak research truth

THIS PLAN
  LLM research skill -> structured proof -> deterministic engine
  Stronger eligibility gate, stronger packet, same create-lane handoff

12-MONTH IDEAL
  Recommendation lane + proving-run telemetry + upgrade intelligence
  One shared evidence model for recommend, onboard, and maintain
```

## Alternatives

### Approach A: Harden Script-Only Heuristics

Summary: Keep the current skill thin and just add more scoring knobs and more fetch sources to the runner.

Effort: S  
Risk: High

Pros:
- smallest code diff
- easiest to ship fast

Cons:
- still makes judgment from proxies instead of proof
- keeps the skill from doing the actual work the user intended
- does not solve the maintainer-trust problem

### Approach B: Skill-Led Research + Deterministic Engine

Summary: Make the skill perform real candidate research and write structured proof. Keep the runner deterministic and boring.

Effort: M  
Risk: Medium

Pros:
- matches the original product intent
- preserves replayability and approval-artifact rigor
- makes the packet better without forcing the runner to become an LLM host

Cons:
- adds one more contract between skill output and runner input
- requires clearer prompt discipline and test fixtures

### Approach C: Full Candidate Execution Harness In The Recommendation Lane

Summary: Install and probe each candidate in isolated local environments before any recommendation is allowed.

Effort: L  
Risk: High

Pros:
- strongest empirical signal
- could catch docs-vs-reality gaps earlier

Cons:
- drags auth, platform, install, and sandbox complexity into the recommendation lane
- turns a pre-create research step into a mini proving run
- wrong milestone ordering

### Approach D: Operator-Led Finalist Sprint

Summary: Keep the current lane mostly manual for another cycle, use lightweight doc triage, then run a tiny executable viability spike on the top 1-2 finalists before automating more.

Effort: S  
Risk: Medium

Pros:
- fastest way to learn whether research is actually the bottleneck
- uses implementation evidence instead of recommendation prose

Cons:
- does not satisfy the user's explicit goal for a stronger skill-led workflow
- leaves the current thin skill in place longer
- still requires human archaeology every cycle

## Recommendation

Choose Approach B.

That is the active plan because it matches the user's intended product direction and fixes the thin-skill gap without dragging live provider execution into the wrong phase.

Both outside strategy voices challenged this choice and argued for a lighter executable viability sprint before more research infrastructure. That challenge remains documented below as rejected-for-now context, not as an open blocker.

## Architecture

```text
maintainer seed file + optional shortlist hints
                    |
                    v
   .codex/skills/recommend-next-agent/SKILL.md
                    |
                    +-------------------------------+
                    |                               |
                    v                               v
        web/doc/package/GitHub research        structured probe requests
                    |                               |
                    +---------------+---------------+
                                    |
                                    v
                 structured candidate research proof
                    (one dossier per candidate)
                                    |
                                    v
                   scripts/recommend_next_agent.py
                                    |
         +-------------+-------------+-------------+-----------+
         |             |                           |           |
         v             v                           v           v
 schema validation  allowlisted probes      deterministic scoring  packet + approval render
         |             |                           |           |
         +-------------+-------------+-------------+-----------+
                                    |
                                    v
                     explicit promote and dry-run validation
                                    |
                                    v
     docs/agents/selection/cli-agent-selection-packet.md
     docs/agents/selection/runs/<run_id>/**
     docs/agents/lifecycle/<pack>/governance/approved-agent.toml
```

## Execution Contract Freeze

The milestone is implementable only if the execution contract is frozen up front.

### Skill and runner interface

The skill must become a two-phase workflow:

1. Research phase
2. Deterministic runner phase

The skill owns research and dossier authoring. The runner owns validation, optional probes, scoring, packet rendering, approval drafting, and promotion.

The runner CLI contract for this milestone is:

```sh
python3 scripts/recommend_next_agent.py generate \
  --seed-file docs/agents/selection/candidate-seed.toml \
  --research-dir ~/.gstack/projects/<repo-slug>/recommend-next-agent-research/<run_id> \
  --run-id <run_id> \
  --scratch-root ~/.gstack/projects/<repo-slug>/recommend-next-agent-runs

python3 scripts/recommend_next_agent.py promote \
  --run-dir ~/.gstack/projects/<repo-slug>/recommend-next-agent-runs/<run_id> \
  --repo-run-root docs/agents/selection/runs \
  --approved-agent-id <agent_id> \
  --onboarding-pack-prefix <kebab-case-pack-prefix> \
  [--override-reason "<required when approved agent differs from recommended>"]
```

`generate` must not perform open-ended candidate research. It may only:

- load the frozen seed snapshot from the research directory
- validate dossiers
- execute allowlisted probes
- compute scores over validated dossiers
- write deterministic outputs into the run directory

### Research directory layout

The skill must create exactly one research directory per run at:

`~/.gstack/projects/<repo-slug>/recommend-next-agent-research/<run_id>/`

Required contents:

- `seed.snapshot.toml`
- `research-summary.md`
- `dossiers/<agent_id>.json`

Optional contents:

- `evidence-cache/`
- `screenshots/`
- `notes/`

`seed.snapshot.toml` is the full reviewed snapshot of `docs/agents/selection/candidate-seed.toml` used for the run. The runner must validate that the live seed file still exists, but it must score and promote only from `seed.snapshot.toml`.

### Scratch run directory layout

The runner must write exactly these top-level outputs under:

`~/.gstack/projects/<repo-slug>/recommend-next-agent-runs/<run_id>/`

- `run-status.json`
- `seed.snapshot.toml`
- `candidate-pool.json`
- `eligible-candidates.json`
- `scorecard.json`
- `sources.lock.json`
- `comparison.generated.md`
- `approval-draft.generated.toml`
- `run-summary.md`
- `candidate-dossiers/<agent_id>.json`
- `candidate-validation-results/<agent_id>.json`

The committed review directory under `docs/agents/selection/runs/<run_id>/` must contain a byte-copy of every run artifact except `run-status.json`, which may differ only in top-level `mode` and final file paths if needed for promotion bookkeeping.

### Status enums

The runner must use these candidate statuses:

- `eligible`
- `candidate_rejected`
- `candidate_error`

The runner must use these run statuses:

- `success`
- `success_with_candidate_errors`
- `insufficient_eligible_candidates`
- `run_fatal`

`candidate_rejected` means the dossier was valid enough to evaluate but failed a hard gate.

`candidate_error` means the runner could not safely evaluate the dossier because of malformed input, fetch failure on a runner-owned step, or probe failure that prevents evaluation.

`generate` exits `0` only for:

- `success`
- `success_with_candidate_errors`, but only when at least 3 candidates remain eligible

`generate` exits non-zero for:

- `insufficient_eligible_candidates`
- `run_fatal`

### Selection invariant

The packet still compares exactly 3 candidates.

Selection rules are frozen:

- candidates already onboarded in `crates/xtask/data/agent_registry.toml` are always `candidate_rejected`
- only `eligible` candidates may be scored
- exactly 3 scored candidates enter the comparison table
- if fewer than 3 `eligible` candidates remain, `generate` must fail closed after writing `run-status.json`, `candidate-pool.json`, `sources.lock.json`, and every available validation result

### Scoring contract freeze

This milestone does not redesign the public scorecard shape.

Keep these existing dimensions, score range, primary/secondary split, and shortlist ordering contract from `scripts/recommend_next_agent.py`:

- `Adoption & community pull`
- `CLI product maturity & release activity`
- `Installability & docs quality`
- `Reproducibility & access friction`
- `Architecture fit for this repo`
- `Capability expansion / future leverage`

Keep the existing 0-3 bucket scale.

Keep the existing shortlist tie-break order.

What changes in this milestone is not the public scorecard shape. What changes is the evidence source:

- no dimension may be scored from keyword hits alone when a typed dossier claim exists
- repo-fit and reproducibility dimensions must read dossier claims first
- the packet notes must cite dossier evidence or probe output ids, not just synthesized prose

### Probe contract freeze

V1 probe support is intentionally narrow.

Allowed probe kinds:

- `help`
- `version`

Allowed binary token pattern:

- `^[A-Za-z0-9._-]+$`

Disallowed:

- shell strings
- paths containing `/`
- environment-variable expansion
- redirection
- pipes
- candidate-authenticated commands
- network-dependent commands

Runner-owned probe execution limits:

- timeout: 5 seconds per probe
- max probes: 2 per candidate
- max captured stdout+stderr: 32768 bytes per probe
- inherited environment: `PATH`, `HOME`, `TMPDIR` only

If a probe exceeds limits or violates policy, record it in `candidate-validation-results/<agent_id>.json` and treat it as `candidate_error` only when the dossier required that probe to satisfy a hard gate. Otherwise, keep the candidate on dossier evidence alone.

### Evidence budget freeze

The skill may research broadly, but the dossier contract must stay small enough to review and test.

Per candidate dossier limits:

- max 12 evidence refs
- max 4 official-doc refs
- max 2 package-registry refs
- max 3 GitHub refs
- max 3 ancillary refs
- max 3 blocked steps
- max 1200 characters per freeform note field

The runner stores evidence metadata, hashes, and bounded excerpts. It does not persist full remote page bodies into committed review artifacts.

## Proposed Contract Changes

### Skill contract

The skill must stop being a shell wrapper.

It should explicitly instruct the AI agent to:

- read the seed file and shortlist intent
- research each candidate from official docs, GitHub, package registries, and repo-fit constraints
- capture proof for:
  - deterministic non-interactive execution surface
  - offline parser / fixture / fake-binary strategy
  - redaction and raw-output risk
  - crate-first onboarding fit
  - reproducibility caveats and blocked steps
- emit structured probe requests, not executable shell strings
- write structured research dossiers before calling the runner
- stop and fail closed when proof is insufficient

### Runner contract

The runner should no longer infer charter fit from loose keyword counts alone.

The runner should:

- load one versioned dossier schema owned by `docs/specs/cli-agent-recommendation-dossier-contract.md`
- accept structured research dossier inputs from the skill phase
- freeze the reviewed seed/default snapshot into the run directory and promote only from that frozen snapshot
- execute only allowlisted non-mutating probe argv recipes that are runner-owned, never source-derived shell text
- validate required proof fields
- reject candidates with named reasons before scoring
- score only over candidates with complete proof
- render the canonical packet and approval draft from proof-backed data
- write durable `run-status.json` plus per-candidate validation results on every failure path
- preserve current staging, rollback, byte-identity, and dry-run validation guarantees

### Research dossier contract

Each dossier must be one JSON object with this top-level shape:

- `schema_version`
- `agent_id`
- `display_name`
- `generated_at`
- `seed_snapshot_sha256`
- `official_links`
- `install_channels`
- `auth_prerequisites`
- `claims`
- `probe_requests`
- `blocked_steps`
- `normalized_caveats`
- `evidence`

`claims` must contain exactly these keys:

- `non_interactive_execution`
- `offline_strategy`
- `observable_cli_surface`
- `redaction_fit`
- `crate_first_fit`
- `reproducibility`
- `future_leverage`

Each claim object must contain:

- `state`, one of `verified`, `blocked`, `inferred`, `unknown`
- `summary`
- `evidence_ids`
- `blocked_by`, optional
- `notes`, optional

Each evidence object must contain:

- `evidence_id`
- `kind`, one of `official_doc`, `github`, `package_registry`, `ancillary`, `probe_output`
- `url`, optional for `probe_output`
- `title`
- `captured_at`
- `sha256`
- `excerpt`

Each probe request must contain:

- `probe_kind`, one of `help`, `version`
- `binary`
- `required_for_gate`, boolean

No dossier may contain raw shell commands, inline HTML bodies, or unbounded notes.

## Eligibility Gate

Every candidate must pass these externally observable checks before it can enter the 3-row comparison:

1. There is explicit proof of a plausible deterministic non-interactive execution surface.
2. There is explicit proof of a credible offline parser, fixture, or fake-binary strategy.
3. There is externally grounded evidence or a runner probe result for the observable parts of the candidate's execution surface.
4. The repo-specific claims about redaction fit and crate-first onboarding fit are marked as `verified`, `inferred`, `blocked`, or `unknown`, never implied by prose alone.
5. The research evidence is reproducible enough that another maintainer can repeat the reasoning later.

Candidates that fail any hard check remain in `candidate-pool.json` with named rejection reasons and do not appear in `eligible-candidates.json`, the scorecard shortlist, or the final packet.

Candidates that are strategically important but only partially proven may still appear in the packet appendix as `strategic contenders` with explicit implementation risks, but they are not eligible for automated recommendation.

## Packet Contract Freeze

The canonical packet must keep the existing section numbering and exactly-3 comparison table shape from the current renderer and the template.

This milestone adds exact requirements, not a new structure:

- section 1 still identifies the shortlist and recommendation
- section 4 still contains exactly 3 candidate rows
- section 5 must end with one explicit maintainer decision block:
  - `Approve recommended agent`
  - `Override to shortlisted alternative`
  - `Stop and expand research`
- section 6 must split:
  - reproducible now
  - blocked until later
- the appendix must include:
  - loser rationale for the two non-winning shortlisted candidates
  - strategic contenders that failed hard gating, if any
  - dated evidence provenance for all shortlisted candidates

The packet is the maintainer decision surface. A separate narrative memo is not required for approval.

## Workstreams

### Workstream 1: Replace Thin Skill With Research Workflow

Deliverables:
- rewritten `.codex/skills/recommend-next-agent/SKILL.md`
- explicit research steps, stop conditions, and dossier output contract
- clear distinction between research phase and runner phase

### Workstream 2: Add Structured Research Inputs

Deliverables:
- dossier schema documented in one normative spec plus the skill and runner
- scratch-root artifact layout for AI-produced research files
- stable serialization for proof-backed candidate dossiers
- frozen seed/default snapshot copied into the run directory

### Workstream 3: Narrow And Harden The Runner

Deliverables:
- runner-owned probe allowlist and capture policy
- validation of required proof fields before scoring
- named rejection reasons for missing proof
- deterministic scoring over proof-backed candidates only
- durable `run-status.json` and per-candidate validation results
- preserve current promote-time safety behavior

### Workstream 4: Upgrade The Maintainer Packet

Deliverables:
- richer section 5 recommendation rationale
- richer section 6 evaluation recipe
- explicit approve / override / stop decision block
- loser rationale for the non-winning shortlisted candidates

### Workstream 5: Test And Prove It

Deliverables:
- Python tests for dossier validation and fail-closed eligibility behavior
- Python tests for seed/default snapshot freeze between `generate` and `promote`
- Python tests for malformed, oversized, invalid-utf, and invalid-json dossier inputs
- Python tests for runner-owned probe timeout, non-zero exit, and capture redaction
- transaction tests for partial promote failure and rollback guarantees
- golden packet tests for the richer packet shape
- Rust validation coverage remains green for generated approval artifacts
- one proving run using the real research-assisted skill workflow

## Required Artifacts

- updated skill file
- proof-backed scratch dossiers
- updated scorecard and candidate-pool outputs
- richer canonical packet
- approval draft and final approval artifact
- one committed promoted run directory
- one test-plan artifact for this milestone

## Blocking Risks

| Risk | Why it matters | Mitigation |
| --- | --- | --- |
| The skill still behaves like a command wrapper | The product intent is still missed | Make dossier creation mandatory before runner invocation |
| Research proof fields become too loose | The gate regresses to prose theater | Keep runner-side required fields strict and fail closed |
| Local probe steps become a security hole | Research can execute source-derived commands or leak env-sensitive output | Move probes under runner control, enforce argv allowlist, cap bytes, redact tokens and paths |
| Packet grows without becoming clearer | Maintainer still cannot decide quickly | Add explicit decision block and loser rationale |
| Script and skill duplicate logic | Drift and contradictory behavior | Skill owns research, runner owns validation/render/promotion only |
| Generate/promote drift | Reviewed inputs differ from promoted inputs | Freeze seed/default snapshot into the run directory and promote from that snapshot only |
| Evidence blow-up or rate limits | Slow or flaky research runs | Add per-run budgets, caching, retries, and summary-first committed artifacts |

## Error & Rescue Registry

| Failure | User-visible impact | Rescue |
| --- | --- | --- |
| Skill cannot gather enough public proof | Candidate set looks under-specified | Fail closed, keep candidate in rejection log, expand seed set or gather more evidence |
| A candidate needs auth for all meaningful probes | Recommendation may overstate confidence | Record blocked steps explicitly and downgrade reproducibility confidence |
| Runner receives malformed dossier | Packet cannot be trusted | Validation error before scoring or promotion |
| Packet promotion fails after research succeeds | Maintainer loses time and trust | Preserve current staging + rollback flow |
| Live seed file changes between generate and promote | Final approval artifact no longer matches the reviewed run | Promote only from the frozen run snapshot and fail on drift |
| One candidate errors while others validate | Entire run becomes opaque | Write `run-status.json`, per-candidate validation results, and distinguish `candidate_rejected`, `candidate_error`, and `run_fatal` |

## Failure Modes Registry

| Area | Critical failure mode | Guard |
| --- | --- | --- |
| Skill research | LLM invents proof without source backing | Every proof field must carry source refs or explicit local-probe output |
| Eligibility gate | Incomplete candidates still get scored | Runner validates required proof fields and fails closed |
| Packet quality | Packet lists scores but not decisions | Required explicit decision block and loser rationale |
| Scope creep | Recommendation lane tries to become proving run | Runner-owned safe local probes only, no provider-backed execution |
| Determinism | Same dossier yields different packet shape | Stable serialization and golden tests |
| Contract drift | Prompt text and runner expectations diverge | One normative dossier contract doc plus versioned schema |
| Security | Research can execute injected commands or capture sensitive output | Explicit probe allowlist, byte caps, redaction, domain allowlist |

## Acceptance Gates

1. The skill file clearly instructs an AI agent to perform research before invoking the runner.
2. The runner requires structured proof fields before a candidate can be scored.
3. A candidate missing offline-parser / fixture proof is rejected before scoring.
4. The canonical packet includes winner rationale, loser rationale, reproducible-now vs blocked-later steps, and an explicit maintainer decision block.
5. `cargo run -p xtask -- onboard-agent --approval ... --dry-run` still validates the promoted approval artifact.
6. One proving run demonstrates the research-assisted flow end to end.
7. A normative dossier contract exists at `docs/specs/cli-agent-recommendation-dossier-contract.md` and is referenced by the skill, runner, and packet expectations.
8. `generate` freezes the reviewed seed/default snapshot into the run directory and `promote` consumes that frozen snapshot instead of re-reading live defaults.
9. Runner probe execution is bounded to runner-owned allowlisted argv recipes, never source-derived shell text.
10. One proving run records outcome metrics so the repo can compare this milestone against the lighter finalist-sprint alternative.

## Success Metrics

The milestone is only worth keeping if it improves decision quality or maintainer speed, not just packet sophistication.

Record these metrics in the first proving run:

- maintainer time-to-decision from skill invocation to approve-or-override
- shortlist override rate
- count of blockers predicted in the packet versus blockers discovered later in onboarding
- count of candidates rejected before scoring due to missing proof
- evidence collection time and total fetched-source count

Success looks like:

- the packet reduces manual archaeology enough that the maintainer can decide from the packet and appendix
- at least one pre-score rejection catches a candidate that the heuristic lane would have let through
- the proving run leaves a durable evidence chain another maintainer can replay without live memory

## Test Diagram

| New codepath / behavior | Coverage type | Required proof |
| --- | --- | --- |
| Skill research writes dossier inputs | manual proving run + fixture-backed golden example | skill-generated dossier files exist and are consumable |
| Runner validates required proof fields | Python unit tests | missing proof fails before scoring |
| Candidate rejection reasons survive to outputs | Python unit tests | `candidate-pool.json` contains named rejection reasons |
| Richer packet rendering | golden tests | required sections and decision block present |
| Promote-time approval validation | existing Rust + dry-run path | promoted approval artifact still passes real loader |
| Generate/promote snapshot freeze | Python unit tests | promoted outputs match the frozen seed/default snapshot, not later live edits |
| Probe security boundary | Python unit tests | only allowlisted argv recipes execute, captured output is capped and redacted |
| Partial run and promotion failure | Python + transaction tests | per-candidate failures survive to status artifacts and rollback remains intact |

## Initial Test Plan Artifact

`~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-test-plan-20260428-074000.md`

## Cross-Phase Themes

- Trust beats cleverness. The repo already has the clever part. The missing product value is evidence quality and maintainer confidence.
- The next milestone should tighten the human decision surface, not invent a second recommender.
- Both outside strategy voices challenged whether this should be the next milestone at all. That is a real product challenge, not a formatting nit.

## Historical Challenge

The outside strategy review did not agree that more recommendation infrastructure was obviously the right next move.

That challenge was:

- maybe the real bottleneck is not recommendation quality, it is late discovery during actual onboarding
- maybe the right next step is a lighter operator-led finalist sprint plus a tiny executable viability spike on the top 1-2 candidates
- maybe a heavy research-proof system overfits to current repo assumptions and still misses the real integration pain

Resolution:

- documented
- considered
- rejected for this milestone by user decision on 2026-04-28

## CEO Dual Voices

CEO DUAL VOICES — CONSENSUS TABLE:

| Topic | Codex CEO voice | Claude CEO voice | Consensus |
| --- | --- | --- | --- |
| Is the currently requested milestone directionally coherent? | Yes, but it may optimize the paperwork around selection instead of the true bottleneck | Yes, but it may be solving the wrong problem first | Partial |
| Is the current thin skill the real gap? | Yes, the skill does not do real research today | Yes, that gap is real | Yes |
| Should research proof alone be trusted? | No, proof may still fail to predict integration reality | No, LLM proof can still mislead | Yes |
| Is there a better lighter alternative? | Yes, shortlist plus tiny viability spike deserves real consideration | Yes, manual-first finalist sprint is the strongest omitted option | Yes |
| Should the repo keep human approval as the boundary? | Yes | Yes | Yes |
| Should this user challenge be auto-decided away? | No, it is a product choice for the user | No, preserve it for user decision | Yes |

CEO completion summary:

- The requested milestone is coherent, but it is not uncontested.
- The strongest challenge is not "the plan is bad," it is "the repo may be automating the wrong part first."
- The user-directed default remains Approach B, with the finalist-sprint alternative preserved explicitly.

## Design Review

Skipped, no UI scope.

Reason:

- this milestone changes a repo-local skill, a deterministic runner, and packet artifacts
- it does not introduce a new product surface that needs interaction or visual design review

## Eng Dual Voices

ENG DUAL VOICES — CONSENSUS TABLE:

| Topic | Codex eng voice | Claude eng voice | Consensus |
| --- | --- | --- | --- |
| Should dossier inputs be typed and versioned? | Yes, prose-heavy dossiers will drift | Yes, the schema needs typed claim states and evidence refs | Yes |
| Is generate/promote drift acceptable? | No, promote must use a frozen reviewed snapshot | No, re-reading live defaults breaks review integrity | Yes |
| Can the skill emit shell commands for probes? | No, that is not a real security boundary | No, probes must be runner-owned and allowlisted | Yes |
| Are durable failure artifacts required? | Yes, partial failures need explicit status surfaces | Yes, distinguish `candidate_rejected`, `candidate_error`, and `run_fatal` | Yes |
| Does the test plan need more failure coverage? | Yes, add malformed dossiers, utf/json errors, 429/500s, rollback, redaction | Yes, current coverage is too thin | Yes |
| Is there a measurable success criterion today? | Not yet, add outcome metrics | Not yet, artifact completion alone is not success | Yes |

Eng completion summary:

- The architecture direction is sound only if the research contract becomes explicit and versioned.
- Probe execution must move fully under runner control.
- This milestone needs outcome metrics, not just greener tests and bigger packets.

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
|---|-------|----------|----------------|-----------|-----------|----------|
| 1 | CEO | Replace the current plan-of-record instead of appending to the landed milestone | Mechanical | P3 Pragmatic | The current `PLAN.md` describes already-landed work and is now misleading | Leaving the stale milestone as active |
| 2 | CEO | Recommend skill-led research plus deterministic engine | Mechanical | P1 Completeness | It matches the intended product and fixes the actual trust gap | More heuristic scoring, or full execution harness now |
| 3 | CEO | Keep human approve-or-override as the product boundary | Mechanical | P5 Explicit over clever | The repo already depends on maintainer judgment and approval artifacts | Auto-approving winners |
| 4 | CEO | Keep live provider-backed execution out of this milestone | Mechanical | P3 Pragmatic | That belongs to the later proving run, not pre-create recommendation | Turning recommendation into a proving run |
| 5 | Eng | Add a normative dossier contract doc | Mechanical | P5 Explicit over clever | The skill and runner need one versioned truth surface for research claims | Freehand prompt-only dossier expectations |
| 6 | Eng | Move local probes under runner control | Mechanical | P5 Explicit over clever | Source-derived shell text is not a defensible security boundary | Skill-emitted shell snippets |
| 7 | Eng | Freeze seed/default snapshots per run | Mechanical | P3 Pragmatic | Reviewed inputs and promoted outputs must stay identical | Re-reading live `candidate-seed.toml` during promote |
| 8 | Eng | Add durable run-status outputs and per-candidate results | Mechanical | P1 Completeness | Partial failure needs explicit artifacts or the run becomes opaque | Fatal-only exit behavior |
| 9 | Eng | Add outcome metrics to the proving run | Taste | P2 Boil the lake | The repo needs evidence that this milestone beats the lighter alternative | Declaring success from artifact completion alone |

## Completion Summary

STATUS: DONE_WITH_CONCERNS

What changed:

- `PLAN.md` now treats the LLM-guided research skill as the next milestone, not the already-landed deterministic lane
- `TODOS.md` now names that work as the active pending milestone and marks the deterministic recommendation engine as completed
- the milestone contract now includes a versioned dossier spec, frozen reviewed snapshots, runner-owned probes, durable failure artifacts, and measurable success metrics

Concerns:

- the plan intentionally adds one new runner input contract, `--research-dir`, to keep the skill phase and runner phase separated cleanly
- the proving run still has to demonstrate that this heavier recommendation lane beats the lighter finalist-sprint alternative on real maintainer time and decision quality
- implementation should not start changing score dimensions or packet section numbering beyond what this plan freezes

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/autoplan` | Scope and strategy challenge | 2 | completed | Both voices agreed the thin skill is real, but challenged whether recommendation infrastructure is the true bottleneck |
| Codex Review | `/autoplan` | Independent external challenge | 2 | completed | Outside Codex reviews pushed the lighter finalist-sprint alternative and explicit success metrics |
| Eng Review | `/autoplan` | Architecture and test hardening | 2 | completed | Both voices required typed dossier schema, frozen snapshots, runner-owned probes, durable failure artifacts, and broader negative-path tests |
| Design Review | `/autoplan` | UI/UX gaps | 0 | skipped | No UI scope in this milestone |

**VERDICT:** NEXT MILESTONE READY FOR IMPLEMENTATION. The requested LLM-guided research direction is fully planned and technically hardened. The lighter finalist-sprint alternative remains documented as a rejected-for-now option, and the proving run must measure whether that rejection was correct.
