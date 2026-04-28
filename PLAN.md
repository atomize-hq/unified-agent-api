<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/codex-recommend-next-agent-autoplan-restore-20260428-133720.md -->
# PLAN — Recommendation Lane Trust Surface Hardening

Status: ready for implementation  
Date: 2026-04-28  
Branch: `codex/recommend-next-agent`  
Base branch: `main`  
Repo: `atomize-hq/unified-agent-api`

## Source Inputs

- Prior approved design artifact:
  - `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-design-20260427-151419.md`
- Validation artifact for the landed lane:
  - `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommendation-lane-validation-20260428-071743.md`
- Prior eng-review test artifact:
  - `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-eng-review-test-plan-20260427-153026.md`
- Fresh eng-review test artifact for this consolidation pass:
  - `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-eng-review-test-plan-20260428-140435.md`
- Live repo surfaces:
  - `.codex/skills/recommend-next-agent/SKILL.md`
  - `scripts/recommend_next_agent.py`
  - `scripts/test_recommend_next_agent.py`
  - `docs/specs/cli-agent-recommendation-dossier-contract.md`
  - `docs/templates/agent-selection/cli-agent-selection-packet-template.md`
  - `docs/agents/selection/cli-agent-selection-packet.md`
  - `docs/agents/selection/runs/20260428T164358Z-cli-recommendation/**`
  - `crates/xtask/src/approval_artifact.rs`

## Outcome

Land the missing trust layer in the already-landed recommendation lane.

This slice does **not** rebuild discovery, does **not** reopen `xtask onboard-agent`, and does **not** add runtime onboarding for a new agent.

It does three things, end to end:

1. Tighten the hard-gate semantics so critical onboarding claims cannot pass on generic `inferred` posture alone.
2. Make the canonical packet a real maintainer decision surface for sections 5-9, while keeping `approved-agent.toml` as the normative approval artifact.
3. Replace shallow packet-presence tests with semantic tests that lock the gate rules, decision surface, and promote-time invariants.

## Problem Statement

The recommendation lane is now mechanically valid:

- it can consume frozen research artifacts
- it can generate a shortlist
- it can render a canonical packet
- it can render an approval draft
- it can promote a reviewed run
- it can hand off to `cargo run -p xtask -- onboard-agent --approval ...`

But it still has two trust gaps:

1. The hard gate accepts too much ambiguity.
   Today `evaluate_hard_gate(...)` treats `verified` and `inferred` as equally passable as long as some `evidence_ids` exist.
2. The packet is structurally correct but semantically thin.
   Sections 5-9 mostly satisfy headings and fixed markers, but they do not yet function as the maintainer’s real approve / override / stop surface.

That combination is dangerous. Formal governance shape plus soft epistemic standards. It looks trustworthy before it is trustworthy.

## Step 0 Scope Challenge

### What existing code already solves the sub-problems

- Frozen research/run lifecycle already exists in `scripts/recommend_next_agent.py`.
- Approval artifact validation already exists in `crates/xtask/src/approval_artifact.rs`.
- Canonical packet layout already exists in `docs/templates/agent-selection/cli-agent-selection-packet-template.md`.
- Promote-time byte-identity and dry-run validation already exist and are working.
- The current proving run already gives us real fixtures to harden against.

### Minimum change set

This plan stays inside the current lane and reuses the existing script-first architecture.

No new Rust command.  
No new control-plane crate.  
No new external service.  
No new artifact type.

The minimum repo touch set is:

- `.codex/skills/recommend-next-agent/SKILL.md`
- `docs/specs/cli-agent-recommendation-dossier-contract.md`
- `docs/templates/agent-selection/cli-agent-selection-packet-template.md`
- `scripts/recommend_next_agent.py`
- `scripts/test_recommend_next_agent.py`
- `docs/agents/selection/cli-agent-selection-packet.md`
- one fresh promoted run under `docs/agents/selection/runs/<fresh-run-id>/`
- the resulting `docs/agents/lifecycle/<pack>/governance/approved-agent.toml`

### Complexity check

This is a narrow slice with one primary logic module: `scripts/recommend_next_agent.py`.

The file count looks larger because the lane commits generated review artifacts. That is acceptable. The real authored logic change is still concentrated in:

- one Python runner
- one Python test file
- one contract doc
- one packet template
- one skill prompt

That is “engineered enough,” not a rewrite.

### Search check

No new framework, concurrency model, or infrastructure component is introduced here.

This is a semantics pass over an existing script-and-contract lane, not a platform choice. The right move is [Layer 1]: reuse the existing boring control-plane surfaces and tighten their rules.

### TODOS cross-reference

`TODOS.md` already contains the broader pending item “Land The LLM-Guided Research Layer For The Recommendation Lane.”

This plan is intentionally narrower than that TODO. It closes the validation-proven trust gap first so the repo stops certifying speculation as recommendation truth. The TODO can be rewritten after this lands; it is not a blocker for this slice.

### Completeness check

The shortcut version would be:

- ban a few phrases in packet prose
- add more `assertIn(...)` checks
- leave the gate semantics fuzzy

That would save almost no time and preserve the real defect. Not acceptable.

The complete version is still a boilable lake:

- define gate sufficiency rules explicitly
- introduce a structured decision-surface model inside the existing runner
- validate that model before rendering markdown
- lock it with semantic tests and one fresh proving run

## Premise Challenge

### Premise kept

1. The next step is a trust-surface hardening pass over the existing recommendation lane, not a new discovery subsystem.
2. The lane should remain script-first and deterministic after research artifacts are frozen.
3. The approval artifact stays normative. The packet becomes the human decision surface, not a second validator target.
4. Tests must validate semantics, not just section presence.

### Premise rejected

1. “Make the packet itself the decision document” must **not** mean “make the packet the normative source of truth.”
   That would conflict with `approved-agent.toml` and the Rust loader.
2. “Tighten the gate” must **not** mean “ban generic wording.”
   The current bug is not wording. It is that gate sufficiency ignores claim semantics entirely.

## Challenge Resolution

Outside review raised one valid challenge:

- the deeper product issue is still selection truth, not merely packet quality

That challenge is correct, but it does not change the next step.

We are **not** widening this plan into a finalist viability sprint or provider-backed proving lane. That is the next likely milestone after this one. This plan only closes the trust hole in the current governance surface.

Import the lesson, not the scope expansion:

- do not pretend packet hardening solves finalist truth end to end
- do make the lane honest about what is proven now versus blocked until later

## Scope Lock

### In scope

- define hard-gate sufficiency rules per critical claim
- update the dossier contract so “pass” has explicit evidence requirements
- update the repo-local skill so research authors know what proof the runner now expects
- add a structured decision-surface model for packet sections 5-9 inside `scripts/recommend_next_agent.py`
- render sections 5-9 from that model, not from ad hoc boilerplate strings
- validate decision-surface substance before markdown render and again via packet contract checks
- add negative and golden tests for gate sufficiency and packet semantics
- produce one fresh promoted run and approval artifact using the hardened lane
- preserve the current `generate` / `promote` split and existing `xtask` handoff unchanged

### Out of scope

- new candidate discovery logic
- provider-backed finalist execution during recommendation
- replacing the approval artifact with packet-native governance
- migrating the runner into `xtask`
- wrapper or `agent_api` implementation for the selected winner
- post-onboarding maintenance work
- ranking-model redesign beyond what is necessary to remove gate ambiguity

## NOT in scope

- Finalist proving sprint with live provider credentials.
  Rationale: that is a larger “selection truth” milestone and would reopen sequencing.
- Approval artifact schema changes in Rust.
  Rationale: the Rust contract is already correct and should stay the source of truth.
- New packet artifact types or JSON packet publication.
  Rationale: the current markdown + TOML split is sufficient once semantics are hardened.
- Candidate-count configurability.
  Rationale: the exactly-3 packet shape is already frozen and good enough.
- `xtask recommend-agent`.
  Rationale: wrong abstraction layer for this milestone.

## What already exists

- `scripts/recommend_next_agent.py` already owns frozen-input validation, shortlist generation, packet render, approval draft render, and promote-time invariants.
- `scripts/test_recommend_next_agent.py` already owns fixture-backed contract tests and promote-time invariants.
- `docs/specs/cli-agent-recommendation-dossier-contract.md` already freezes the dossier shape, research envelope, run artifacts, and packet constraints.
- `docs/templates/agent-selection/cli-agent-selection-packet-template.md` already states the required outputs for sections 5-9.
- `crates/xtask/src/approval_artifact.rs` already owns approval artifact truth, canonical `comparison_ref`, and override validation.

The plan reuses all of those. No parallel system.

## Existing Code Leverage Map

| Sub-problem | Existing surface to reuse | Planned change |
| --- | --- | --- |
| Critical claim gating | `evaluate_hard_gate(...)` in `scripts/recommend_next_agent.py` | replace permissive state check with explicit claim rules |
| Dossier truth shape | `docs/specs/cli-agent-recommendation-dossier-contract.md` | add gate sufficiency matrix and decision-surface rules |
| Human decision packet | packet template + current renderer | preserve section order, replace boilerplate content with structured decision data |
| Approval governance | `render_approval_toml(...)` + Rust loader | leave semantics unchanged, validate no drift |
| Promote invariants | existing promote path + tests | preserve byte identity for scratch packet, revalidate final approval artifact |
| Regression safety | `scripts/test_recommend_next_agent.py` | upgrade from marker tests to semantic tests |

## Dream State

```text
CURRENT
  frozen research -> permissive gate -> thin packet -> valid approval artifact
  good mechanics, soft trust

THIS PLAN
  frozen research -> explicit gate sufficiency rules -> structured decision surface
  -> substantive packet -> valid approval artifact
  good mechanics, honest trust

12-MONTH IDEAL
  recommendation lane -> finalist truth lane -> onboarding lane -> maintenance lane
  one shared evidence model across recommend, prove, onboard, maintain
```

## Implementation Alternatives

### Approach A: String-Level Patch

Summary: keep the current gating model, tweak packet prose, and replace a few shallow tests.

Effort: S  
Risk: High

Pros:
- smallest diff
- fastest patch

Cons:
- fixes presentation more than truth
- keeps semantics split across contract, renderer, validator, and tests
- does not eliminate the `inferred` hard-gate defect

### Approach B: Semantics Pass Inside The Existing Runner

Summary: keep the current lane architecture, but add explicit gate sufficiency rules and a structured decision-surface model inside the existing Python runner.

Effort: M  
Risk: Medium

Pros:
- fixes the real trust gap without reopening the lane architecture
- preserves the current skill -> runner -> promote -> approval flow
- gives tests something stable and meaningful to lock

Cons:
- still leaves the runner as a large Python file
- requires careful contract wording so docs and code do not drift

### Approach C: Finalist Truth Sprint

Summary: defer trust-surface hardening and instead require a mini proving run on 1-2 finalists before recommendation.

Effort: L  
Risk: High

Pros:
- attacks the deeper truth problem directly
- would produce stronger empirical signal

Cons:
- wrong ordering for the user’s requested next step
- pulls auth, install, and runtime friction into the recommendation slice
- would delay a fix for the already-proven governance trust defect

## Recommendation

Choose Approach B.

This remains the smallest plan that fixes the real defect without reopening lane architecture that already works.

Do **not** widen this into a finalist-truth sprint yet.  
Do **not** ship a string-only patch that leaves gate semantics fuzzy.

## Architecture Review

### Architecture decision

Keep the lane in its existing shape:

`research artifacts -> scripts/recommend_next_agent.py -> comparison.generated.md + approval-draft.generated.toml -> promote -> approved-agent.toml -> xtask dry-run`

The fix is inside the existing runner and contract surfaces. No new command surface. No Rust schema expansion. No second packet validator.

### Dependency graph

```text
research dir
  ├── seed.snapshot.toml
  ├── research-summary.md
  ├── research-metadata.json
  └── dossiers/<agent_id>.json
              |
              v
      scripts/recommend_next_agent.py
              |
              +--> validate_dossier(...)
              +--> validate_probe_requests(...)
              +--> execute_probe(...)
              +--> evaluate_gate_sufficiency(...)
              |      - rule lookup
              |      - allowed state check
              |      - evidence-kind sufficiency
              |      - required probe pass check
              |      - blocked_by rejection
              |
              +--> score_candidate(...)
              +--> build_decision_surface(...)
              +--> validate_decision_surface(...)
              +--> render_comparison_packet(...)
              +--> validate_packet_contract(...)
              +--> render_approval_toml(...)
              |
              v
      docs/agents/selection/runs/<run_id>/**
              |
              +--> comparison.generated.md
              +--> approval-draft.generated.toml
              +--> candidate-validation-results/*.json
              |
              v
      promote_recommendation(...)
              |
              +--> docs/agents/selection/cli-agent-selection-packet.md
              +--> docs/agents/lifecycle/<pack>/governance/approved-agent.toml
              +--> cargo run -p xtask -- onboard-agent --approval ... --dry-run
```

### Architecture findings resolved in this plan

1. Gate truth must be rule-driven, not state-driven.
   The current bug lives in `claim_state_allows_gate_pass(...)` plus `evaluate_hard_gate(...)`, where `inferred` plus any `evidence_ids` can pass. This plan replaces that with claim-specific sufficiency rules.

2. Packet semantics must stay on the Python side.
   The right place to enforce sections 5-9 substance is `validate_packet_contract(...)` in `scripts/recommend_next_agent.py`, not a new Rust validator. Rust remains the approval-artifact boundary only.

3. The packet is the human decision surface, not the approval source of truth.
   `crates/xtask/src/approval_artifact.rs` already hard-locks `comparison_ref` and override semantics. Keep that boundary intact.

4. The runner remains script-first.
   This slice does not justify spending an innovation token on `xtask recommend-agent`. The current Python runner already owns deterministic artifact generation and is the boring place to harden semantics.

### Distribution architecture

No new artifact type is introduced.

This slice still distributes only:

- committed review evidence under `docs/agents/selection/runs/`
- the canonical packet at `docs/agents/selection/cli-agent-selection-packet.md`
- the approval artifact at `docs/agents/lifecycle/<pack>/governance/approved-agent.toml`

## Code Quality Review

### Code-organization decisions

- Keep all new logic in `scripts/recommend_next_agent.py`.
  The file is already the deterministic control-plane implementation. Splitting one new helper and one intermediate model into another module would increase cognitive overhead without reducing risk.
- Add new `TypedDict` shapes beside the existing typed structures.
  Put the decision-surface types near the existing runner data types so a maintainer can read the whole recommendation pipeline in one file.
- Reuse existing packet helpers.
  Extend `packet_section_slice(...)`, `validate_packet_contract(...)`, and the current render path. Do not add a second markdown post-processor.
- Reuse existing candidate-validation artifacts.
  Strengthen `candidate-validation-results/<agent>.json` with structured rejection reasons and rule ids instead of inventing a second diagnostics file.

### DRY rules for this slice

- One normative gate table in `docs/specs/cli-agent-recommendation-dossier-contract.md`.
- One executable gate table in `HARD_GATE_RULES` in `scripts/recommend_next_agent.py`.
- One packet section-shape authority in `docs/templates/agent-selection/cli-agent-selection-packet-template.md`.
- One runtime packet enforcer in `validate_packet_contract(...)`.

The docs and runner will necessarily mirror the same concepts, but the plan must keep them line-for-line compatible so drift is test-detectable instead of review-detectable.

### Minimal-diff guardrails

- Remove `claim_state_allows_gate_pass(...)` entirely instead of keeping two gate paths alive.
- Preserve `render_approval_toml(...)`, `promote_recommendation(...)`, shortlist ordering, score dimensions, and run-artifact layout.
- Do not touch Rust unless the existing dry-run proves an actual approval-boundary defect. This plan assumes it will not.

## Performance Review

This slice is not performance-driven, but it still needs explicit guardrails so semantics hardening does not accidentally make the runner sloppy.

- Gate evaluation must stay bounded per candidate.
  The runner already limits probes with `MAX_PROBES_PER_CANDIDATE`. The new sufficiency logic must remain `O(claims + evidence + probes)` per dossier, not repeated rescans of the full evidence list for every rule.
- Probe execution behavior must not expand.
  No new probe kinds, no new binaries, no new concurrency model. The only change is how existing required probes affect pass/fail.
- Packet rendering must stay deterministic.
  The golden test should compare stable section slices for sections 5-9, not full-file timestamps, so reruns stay cheap and deterministic.

Performance risk is low. The real requirement is to avoid accidental complexity while hardening correctness.

## Contract Changes

### 1. Add hard-gate sufficiency rules to the dossier contract

Extend `docs/specs/cli-agent-recommendation-dossier-contract.md` with one new normative subsection:

`## Hard Gate Sufficiency Rules`

Add this exact rule table:

| Claim key | Allowed pass states | Required evidence kinds | Required probe rule | Reject when |
| --- | --- | --- | --- | --- |
| `non_interactive_execution` | `verified` only | at least one `official_doc` and one of `package_registry` or `probe_output` | if a `required_for_gate` probe exists for this claim, it must pass | state is `inferred`, `unknown`, or `blocked`; required evidence kinds missing |
| `observable_cli_surface` | `verified` only | at least one of `official_doc`, `github`, or `probe_output` | if a `required_for_gate` probe exists for this claim, it must pass | state is `inferred`, `unknown`, or `blocked`; no qualifying evidence |
| `offline_strategy` | `verified` or `inferred` | at least one of `official_doc` or `github` | none | state is `unknown` or `blocked`; `blocked_by` present on a passing claim |
| `redaction_fit` | `verified` or `inferred` | at least one of `github` or `probe_output` | none | state is `unknown` or `blocked`; `blocked_by` present on a passing claim |
| `crate_first_fit` | `verified` or `inferred` | at least one of `official_doc`, `github`, or `package_registry` | none | state is `unknown` or `blocked`; `blocked_by` present on a passing claim |
| `reproducibility` | `verified` or `inferred` | at least one `official_doc` and one `package_registry` | none | state is `unknown` or `blocked`; required evidence kinds missing; `blocked_by` present on a passing claim |

Add one explicit rule below the table:

- generic prose in `summary` or `notes` is never sufficient on its own; hard-gate pass/fail is driven by state, evidence-kind coverage, and required probe results

### 2. Clarify packet decision-surface requirements

Promote the current soft guidance in packet sections 7-9 into hard required subsection labels in both:

- `docs/specs/cli-agent-recommendation-dossier-contract.md`
- `docs/templates/agent-selection/cli-agent-selection-packet-template.md`

Required labels:

- Section 7:
  - `Manifest root expectations`
  - `Wrapper crate expectations`
  - ``agent_api backend expectations``
  - `UAA promotion expectations`
  - `Support/publication expectations`
  - `Likely seam risks`
- Section 8:
  - `Manifest-root artifacts`
  - `Wrapper-crate artifacts`
  - ``agent_api artifacts``
  - `UAA promotion-gate artifacts`
  - `Docs/spec artifacts`
  - `Evidence/fixture artifacts`
- Section 9:
  - `Required workstreams`
  - `Required deliverables`
  - `Blocking risks`
  - `Acceptance gates`

### 3. Tighten packet constraints without changing packet shape

Update the contract language so it is explicit that:

- section numbering and order stay frozen
- section 5 keeps the three decision lines exactly as-is
- section 6 keeps the `reproducible now` / `blocked until later` split exactly as-is
- sections 7-9 must now be semantically populated, not merely present

## Skill Changes

Update `.codex/skills/recommend-next-agent/SKILL.md` so the research phase instructs the operator or model to:

- prefer `verified` for `non_interactive_execution` and `observable_cli_surface`
- request `help` or `version` probes when public install evidence is not enough
- use `inferred` only for repo-fit claims explicitly allowed by the contract
- record provider-backed blockers in `blocked_steps`, not as passable hard-gate support

No new workflow stages. Just stricter proof discipline.

## Runner Changes

### 1. Replace permissive hard-gate logic

In `scripts/recommend_next_agent.py`:

- delete `claim_state_allows_gate_pass(...)`
- replace `evaluate_hard_gate(...)` with `evaluate_gate_sufficiency(...)`
- add a module-level `HARD_GATE_RULES` constant keyed by claim name

`evaluate_gate_sufficiency(...)` must evaluate, in order:

1. state allowed by rule
2. required evidence kinds present
3. required `required_for_gate` probes passed when applicable
4. `blocked_by` absent when the rule allows inferred claims
5. return a structured result with:
   - `status`
   - `evidence_ids`
   - `notes`
   - `rule_id`
   - `rejection_reason`

`candidate-validation-results/<agent>.json` must carry named reasons such as:

- `non_interactive_execution requires verified state`
- `reproducibility missing package_registry evidence`
- `observable_cli_surface required probe failed`

This is the core behavioral change. A dossier with vague evidence must stop at candidate validation, not make it into the shortlist.

### 2. Make evidence-kind checks explicit and cheap

When validating each dossier:

- build one `evidence_by_id` map
- build one `evidence_kind_set`
- fold probe output refs into the gate result only when the rule allows them

This avoids repeated scanning and makes rejection reasoning deterministic.

### 3. Add a structured decision-surface model

Inside `scripts/recommend_next_agent.py`, add `TypedDict` definitions for packet sections 5-9:

- `DecisionSurface`
- `RecommendationSection`
- `EvaluationRecipeSection`
- `RepoFitSection`
- `RequiredArtifactsSection`
- `WorkstreamsSection`

Add:

`build_decision_surface(seed, dossiers, scores, candidate_results, shortlist_ids, recommended_agent_id) -> DecisionSurface`

It must derive:

- winner rationale tied to exact evidence ids or probe refs
- loser rationale for the other two shortlisted candidates
- `reproducible now` content with install paths, prerequisites, commands, evidence, and saved artifacts
- `blocked until later` content sourced from `blocked_steps`
- repo-fit expectations derived from `recommended.derived_descriptor(seed.defaults)` plus current repo stages
- required artifacts grouped by onboarding stage
- required workstreams, deliverables, blocking risks, and acceptance gates

### 4. Validate the decision surface before rendering markdown

Add:

`validate_decision_surface(surface: DecisionSurface) -> None`

This validator must fail when any required subsection is missing or empty.

Invalid examples:

- no loser rationale for a shortlisted loser
- no runnable commands in section 6
- section 7 missing wrapper-crate expectations
- section 8 not grouped by onboarding stage
- section 9 missing acceptance gates

### 5. Render sections 5-9 from the structured model

Update `render_comparison_packet(...)` so sections 5-9 render from `DecisionSurface`, not hand-built boilerplate strings.

Keep these invariants unchanged:

- packet topmatter
- section order
- provenance lines
- exactly-3 comparison-table shape
- final section-5 decision block
- section-6 split into `reproducible now` and `blocked until later`

### 6. Extend `validate_packet_contract(...)`, do not bypass it

The existing packet validator already enforces:

- heading order
- provenance lines
- section-4 table shape
- section-5 citation presence
- decision block tail
- section-6 split
- appendix requirements

Extend that same function to also reject:

- missing section-7 required labels
- missing section-8 required labels
- missing section-9 required labels
- semantically empty section bodies under those labels

Do not add a second validator. One packet render path, one packet contract check.

### 7. Keep approval governance unchanged

`render_approval_toml(...)`, `promote_recommendation(...)`, and `crates/xtask/src/approval_artifact.rs` remain behaviorally unchanged except for consuming the hardened shortlist and packet outputs.

No Rust schema change.  
No `comparison_ref` change.  
No override semantics change.

## Test Review

### Coverage diagram

```text
CODE PATH COVERAGE
===========================
[+] Gate sufficiency evaluation
    │
    ├── [GAP] strict verified-only claims reject `inferred`
    ├── [GAP] required evidence kinds missing -> named rejection reason
    ├── [GAP] required probe failure -> candidate_error or reject
    ├── [GAP] inferred-allowed claim with `blocked_by` still rejects
    └── [EXISTING] fewer than 3 eligible -> `insufficient_eligible_candidates`

[+] Decision surface builder
    │
    ├── [GAP] winner rationale cites exact evidence ids
    ├── [GAP] each shortlisted loser gets explicit rationale
    ├── [GAP] section 6 has real commands, evidence, blockers, and saved artifacts
    ├── [GAP] section 7 emits all repo-fit categories
    ├── [GAP] section 8 emits staged artifact categories
    └── [GAP] section 9 emits workstreams, risks, and gates

[+] Packet / validator alignment
    │
    ├── [GAP] semantically empty sections 7-9 are rejected
    ├── [GAP] required subsection labels are enforced
    ├── [EXISTING] section order and provenance lines are enforced
    └── [EXISTING] section 5 decision tail is enforced

[+] Promote / approval path
    │
    ├── [EXISTING] scratch packet remains immutable after promote
    ├── [EXISTING] canonical packet is byte-identical to committed run copy
    ├── [EXISTING] override artifact requires override_reason
    └── [EXISTING] xtask onboard-agent dry-run validates final TOML

─────────────────────────────────
CURRENT COVERAGE
  Strong: approval-boundary and immutability checks
  Shallow: packet semantic content for sections 5-9
  Critical gaps: 2

CRITICAL GAPS
  1. inferred hard-gate claims can pass silently
  2. semantically empty decision sections can still promote
─────────────────────────────────
```

### Required tests in `scripts/test_recommend_next_agent.py`

1. `test_generate_rejects_inferred_non_interactive_execution_even_with_evidence_ids`
   Regression test. Proves the current bug is gone.

2. `test_generate_rejects_reproducibility_without_both_doc_and_package_evidence`
   Hard-gate sufficiency test for required evidence kinds.

3. `test_generate_requires_required_probe_for_observable_cli_surface_when_requested`
   Probe rule test for required gate probes.

4. `test_generate_allows_inferred_redaction_fit_only_with_allowed_evidence_kind_and_no_blocked_by`
   Allowed-inference rule test.

5. `test_decision_surface_validation_rejects_missing_repo_fit_categories`
   Semantic validation test for section 7.

6. `test_decision_surface_validation_rejects_missing_artifact_or_gate_sections`
   Semantic validation test for sections 8 and 9.

7. `test_generated_packet_sections_5_through_9_match_expected_golden_output`
   Golden test. Compare stable section slices, not full timestamped packet bytes.

8. `test_packet_contract_rejects_semantically_empty_sections_5_through_9`
   Negative packet-contract test.

9. `test_promote_preserves_scratch_outputs_and_only_changes_allowed_review_fields`
   Keep existing promote immutability coverage.

10. `cargo test -p xtask --test recommend_next_agent_approval_artifact -- --nocapture`
    Approval artifact boundary proof, unchanged.

11. `cargo test -p xtask --test onboard_agent_entrypoint approval_mode -- --nocapture`
    Existing create-lane handoff proof, unchanged.

12. `cargo run -p xtask -- onboard-agent --approval docs/agents/lifecycle/<pack>/governance/approved-agent.toml --dry-run`
    Final proving-run acceptance gate.

### Regression rule

The current permissive hard-gate behavior is a regression in trust semantics, not a new feature gap.

That means test 1 is mandatory. No deferral. No “follow-up hardening” excuse.

## Failure Modes Registry

| Codepath | Realistic production failure | Test required | Error handling | User impact if unhandled |
| --- | --- | --- | --- | --- |
| gate sufficiency evaluator | candidate with docs-only `inferred` claims reaches shortlist | yes | named rejection reason in candidate validation result | silent false confidence in shortlist |
| required probe handling | probe times out and lane still claims observable CLI proof | yes | `candidate_error` or rejection with explicit reason | maintainer approves a non-runnable candidate |
| decision surface builder | packet recommends a winner but hides provider-backed blocker | yes | blocked step must render in section 6 and section 9 risks | human sees “approve” when the right outcome is “stop” |
| repo-fit render | section 7 omits wrapper/backend seam expectations | yes | `validate_decision_surface(...)` and `validate_packet_contract(...)` fail | onboarding starts without a usable seam map |
| artifact/workstream render | sections 8-9 are syntactically present but operationally empty | yes | packet validation fails before promote | maintainer gets no usable handoff |
| promote / approval path | approval TOML drifts from packet decision outcome | existing + keep | Rust dry-run validation | control-plane mutation against the wrong packet |

## Implementation Plan

### Workstream 1 — Contract, template, and skill truth

Files:

- `docs/specs/cli-agent-recommendation-dossier-contract.md`
- `docs/templates/agent-selection/cli-agent-selection-packet-template.md`
- `.codex/skills/recommend-next-agent/SKILL.md`

Tasks:

1. Add `Hard Gate Sufficiency Rules` to the dossier contract.
2. Promote section 7-9 subsection labels from recommendation text to required outputs.
3. State explicitly that the packet is the maintainer decision surface while `approved-agent.toml` remains the normative approval artifact.
4. Update the skill so research authors know exactly which claims may still use `inferred`.

Acceptance gate:

- contract, template, and skill say the same thing about critical-claim proof and packet-section content

### Workstream 2 — Gate sufficiency engine

Files:

- `scripts/recommend_next_agent.py`

Owned functions:

- `evaluate_hard_gate(...)` -> replaced by `evaluate_gate_sufficiency(...)`
- generate-path gate loop around `for gate_key in HARD_GATE_KEYS`
- candidate validation payload construction

Tasks:

1. Add `HARD_GATE_RULES`.
2. Replace permissive state-only pass logic.
3. Emit rule ids and named rejection reasons in candidate validation output.
4. Preserve shortlist ordering, score buckets, and promote flow.

Acceptance gate:

- a dossier with insufficient critical proof never reaches the eligible shortlist

### Workstream 3 — Structured decision surface and packet validation

Files:

- `scripts/recommend_next_agent.py`

Owned functions:

- `render_comparison_packet(...)`
- `validate_packet_contract(...)`
- new `build_decision_surface(...)`
- new `validate_decision_surface(...)`

Tasks:

1. Add `TypedDict` models for sections 5-9.
2. Build the decision surface from shortlist data plus the recommended dossier.
3. Validate the decision surface before markdown render.
4. Render sections 5-9 from the validated model.
5. Extend packet contract validation to reject missing or empty required subsection labels.

Acceptance gate:

- sections 5-9 are substantive because the model is substantive, not because boilerplate strings happen to exist

### Workstream 4 — Tests and proving run

Files:

- `scripts/test_recommend_next_agent.py`
- `docs/agents/selection/cli-agent-selection-packet.md`
- `docs/agents/selection/runs/<fresh-run-id>/**`
- `docs/agents/lifecycle/<pack>/governance/approved-agent.toml`

Tasks:

1. Add the new gate sufficiency tests.
2. Add the decision-surface validation tests.
3. Replace marker assertions with a real section-5-through-9 golden test.
4. Generate one fresh promoted run with the hardened lane.
5. Re-run the Rust approval tests and `xtask` dry-run.

Acceptance gate:

- green Python tests
- green Rust approval tests
- green `xtask onboard-agent --approval ... --dry-run`
- canonical packet byte-identical to the committed run copy

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| 1. Contract/template/skill truth | `docs/specs/`, `docs/templates/`, `.codex/skills/` | — |
| 2. Gate sufficiency engine | `scripts/` | — |
| 3. Decision surface + packet validation | `scripts/` | 2 |
| 4. Semantic tests | `scripts/` | 2, 3 |
| 5. Proving run + approval dry-run | `docs/agents/selection/`, `docs/agents/lifecycle/`, `scripts/`, `crates/xtask/` | 1, 4 |

### Parallel lanes

- Lane A: step 1
  Docs-only lane. Safe to run independently once this plan is frozen.
- Lane B: step 2 -> step 3 -> step 4
  Sequential script lane. These steps all touch `scripts/recommend_next_agent.py` and `scripts/test_recommend_next_agent.py`, so keep them in one worktree.
- Lane C: step 5
  Validation and artifact lane. Wait until lanes A and B merge.

### Execution order

1. Launch lane A and lane B in parallel worktrees.
2. Merge lane A first if it changes required labels or rule wording.
3. Merge lane B after script and tests are green.
4. Run lane C last to generate the fresh promoted run and final dry-run proof.

### Conflict flags

- Steps 2, 3, and 4 all touch `scripts/`. Do not split them across separate worktrees.
- Step 5 rewrites canonical review artifacts. Do not start it before both earlier lanes have merged.

If only one engineer is available, execute the same lane order sequentially.

## Verification Commands

Run in this order:

```sh
python3 -m unittest scripts/test_recommend_next_agent.py
cargo test -p xtask --test recommend_next_agent_approval_artifact -- --nocapture
cargo test -p xtask --test onboard_agent_entrypoint approval_mode -- --nocapture
python3 scripts/recommend_next_agent.py generate \
  --seed-file docs/agents/selection/candidate-seed.toml \
  --research-dir ~/.gstack/projects/atomize-hq-unified-agent-api/recommend-next-agent-research/<fresh-run-id> \
  --run-id <fresh-run-id> \
  --scratch-root ~/.gstack/projects/atomize-hq-unified-agent-api/recommend-next-agent-runs
python3 scripts/recommend_next_agent.py promote \
  --run-dir ~/.gstack/projects/atomize-hq-unified-agent-api/recommend-next-agent-runs/<fresh-run-id> \
  --repo-run-root docs/agents/selection/runs \
  --approved-agent-id <agent_id> \
  --onboarding-pack-prefix <pack-prefix> \
  [--override-reason "..."]
cargo run -p xtask -- onboard-agent --approval docs/agents/lifecycle/<pack-prefix>/governance/approved-agent.toml --dry-run
```

## Acceptance Criteria

This plan is done only when all of the following are true:

1. `inferred` no longer passes `non_interactive_execution` or `observable_cli_surface`.
2. `reproducibility` cannot pass without both doc and package evidence.
3. inferred-allowed claims have explicit evidence-kind and `blocked_by` rules.
4. packet sections 5-9 are rendered from a structured model and validated before render.
5. the packet template and contract require the exact subsection labels used by the renderer and validator.
6. semantically empty sections 7-9 fail tests.
7. one fresh promoted run exists and its canonical packet matches the committed run copy byte-for-byte.
8. the final approval artifact still passes the Rust loader and `xtask` dry-run path.

## Distribution Check

No new artifact type is introduced here.

This slice still publishes only:

- committed review evidence under `docs/agents/selection/runs/`
- the canonical comparison packet at `docs/agents/selection/cli-agent-selection-packet.md`
- the approval artifact consumed by `xtask` at `docs/agents/lifecycle/<pack>/governance/approved-agent.toml`

Target-platform packaging, release publishing, and installer distribution are not part of this milestone because the output is governance truth, not a new runtime binary.

## Completion Summary

- Step 0: Scope Challenge — scope accepted as-is, with the deeper finalist-truth milestone explicitly deferred
- Architecture Review: 4 implementation-shaping issues resolved in-plan
- Code Quality Review: 4 structure constraints locked
- Test Review: diagram produced, 8 concrete gaps identified, 1 regression test mandated
- Performance Review: bounded and acceptable, no new concurrency or infra
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 proposed for this slice
- Failure modes: 2 critical gaps flagged and addressed by plan scope
- Outside voice: prior cleared outside-review conclusion preserved, no new scope change introduced in this consolidation pass
- Parallelization: 3 lanes total, 2 launchable in parallel, 1 final sequential validation lane
- Lake Score: 8/8 recommendations chose the complete option over the shortcut

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | CEO | Narrow the milestone to trust-surface hardening, not a new research subsystem | mechanical | P3 pragmatic | the mechanical lane already landed; the trust gap is the live defect | reopening discovery architecture |
| 2 | CEO | Keep `approved-agent.toml` normative and treat the packet as the human decision surface only | mechanical | P5 explicit over clever | avoids packet/governance role confusion | packet-native governance rewrite |
| 3 | CEO | Defer finalist proving sprint | taste | P3 pragmatic | valid future milestone, wrong sequencing for this slice | widening scope into live provider-backed evaluation |
| 4 | Eng | Replace permissive gate-state logic with explicit per-claim sufficiency rules | mechanical | P1 completeness | closes the actual hard-gate defect | phrase-level heuristics |
| 5 | Eng | Add a structured decision-surface model inside the existing Python runner | mechanical | P5 explicit over clever | meaningful semantics with minimal architectural churn | more regex/string validation only |
| 6 | Eng | Keep the implementation script-first, no `xtask recommend-agent` | mechanical | P4 DRY | current Rust surfaces already solve the approval boundary | second command surface |
| 7 | Eng | Promote section 7-9 subsection labels to required template outputs | mechanical | P1 completeness | gives tests and renderer stable anchors | leaving them as “recommended” prose |
| 8 | Eng | Replace marker tests with golden and negative semantic tests | mechanical | P1 completeness | presence tests are too weak for a governance surface | more `assertIn(...)` |
| 9 | Eng | Split execution into docs lane, script lane, and final proving lane | mechanical | P3 pragmatic | there is real parallel throughput if `scripts/` remains single-owner | false “no parallelization” sequentialism |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 1 | CLEAR | narrowed the milestone to trust-surface hardening; kept approval artifact normative; deferred finalist truth sprint |
| Codex Review | `codex exec` | Independent 2nd opinion | 2 | CLEAR | both outside reads agreed the right slice is semantics, not formatting; both warned against hardening prose without hardening truth |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 1 | CLEAR | 2 critical gaps identified and closed in plan scope: permissive hard-gate semantics and semantically thin sections 5-9 |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | — | skipped, no real UI scope in this follow-on |

**CODEX:** Two outside passes converged on the same warning: do not ship a string-level patch. Fix gate semantics and decision-surface structure together.

**CROSS-MODEL:** Primary review and Codex agreed on the same high-confidence theme, the lane’s defect is semantic trust, not missing mechanics.

**UNRESOLVED:** 0

**VERDICT:** CEO + ENG CLEARED — ready to implement. This consolidation pass tightened structure and removed execution ambiguity without changing milestone scope.
