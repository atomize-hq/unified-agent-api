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
- Fresh eng-review test artifact for this follow-on:
  - `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-eng-review-test-plan-20260428-134309.md`
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

This is the smallest plan that actually fixes the current defect.

Do **not** widen this into a finalist truth sprint yet.  
Do **not** fake it with more packet strings and `assertIn(...)`.

## Architecture

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
              +--> validate dossier schema
              |
              +--> evaluate gate sufficiency
              |      - HARD_GATE_RULES
              |      - evidence kind checks
              |      - required probe checks
              |      - named rejection reasons
              |
              +--> score eligible candidates
              |
              +--> build decision surface
              |      - recommendation rationale
              |      - loser rationale
              |      - reproducible-now / blocked-later
              |      - repo-fit expectations
              |      - artifact expectations
              |      - workstreams / risks / gates
              |
              +--> validate decision surface
              |
              +--> render comparison.generated.md
              +--> render approval-draft.generated.toml
              |
              v
      promote reviewed run
              |
              +--> canonical packet byte-copy
              +--> final approved-agent.toml render
              +--> xtask dry-run validation
```

## Contract Changes

### 1. Add hard-gate sufficiency rules to the dossier contract

Extend `docs/specs/cli-agent-recommendation-dossier-contract.md` with one new normative subsection:

`## Hard Gate Sufficiency Rules`

Add one table with the exact rules below:

| Claim key | Allowed pass states | Required evidence kinds | Required probe rule | Reject when |
| --- | --- | --- | --- | --- |
| `non_interactive_execution` | `verified` only | at least one `official_doc` and one of `package_registry` or `probe_output` | if a `required_for_gate` probe exists for this claim, it must pass | state is `inferred`, `unknown`, or `blocked`; required evidence kinds missing |
| `observable_cli_surface` | `verified` only | at least one of `official_doc`, `github`, or `probe_output` | if a `required_for_gate` probe exists for this claim, it must pass | state is `inferred`, `unknown`, or `blocked`; no qualifying evidence |
| `offline_strategy` | `verified` or `inferred` | at least one of `official_doc` or `github` | none | state is `unknown` or `blocked`; `blocked_by` present on a passing claim |
| `redaction_fit` | `verified` or `inferred` | at least one of `github` or `probe_output` | none | state is `unknown` or `blocked`; `blocked_by` present on a passing claim |
| `crate_first_fit` | `verified` or `inferred` | at least one of `official_doc`, `github`, or `package_registry` | none | state is `unknown` or `blocked`; `blocked_by` present on a passing claim |
| `reproducibility` | `verified` or `inferred` | at least one `official_doc` and one `package_registry` | none | state is `unknown` or `blocked`; required evidence kinds missing; `blocked_by` present on a passing claim |

Add one explicit rule below the table:

- generic prose in `summary` or `notes` is never sufficient on its own; hard-gate pass/fail is driven by state, evidence kind coverage, and required probe results

### 2. Clarify packet decision-surface requirements

Promote the current “recommended subsections” in sections 7-9 of the packet template to required subsection labels.

The template and contract must require these exact subsection labels:

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

## Skill Changes

Update `.codex/skills/recommend-next-agent/SKILL.md` so the research phase instructs the operator / model to:

- prefer `verified` for `non_interactive_execution` and `observable_cli_surface`
- request `help` / `version` probes when public install evidence is not enough
- use `inferred` only for repo-fit claims that are explicitly allowed by the contract
- record blocked provider-backed proof under `blocked_steps` instead of smuggling it into a passable hard gate

No new workflow stages. Just better proof discipline.

## Runner Changes

### 1. Replace permissive hard-gate logic

In `scripts/recommend_next_agent.py`:

- remove the current `claim_state_allows_gate_pass(...)` behavior
- add a module-level `HARD_GATE_RULES` constant keyed by claim name
- add one helper: `evaluate_gate_sufficiency(...)`

`evaluate_gate_sufficiency(...)` must evaluate, in order:

1. state allowed by rule
2. required evidence kinds present
3. required probe kinds passed when applicable
4. `blocked_by` absent for inferred-allowed passes
5. return a typed result:
   - `status`
   - `evidence_ids`
   - `notes`
   - `rule_id`
   - `rejection_reason`

`candidate-validation-results/<agent>.json` must then include named rejection reasons such as:

- `non_interactive_execution requires verified state`
- `reproducibility missing package_registry evidence`
- `observable_cli_surface required probe failed`

### 2. Add a structured decision-surface model

Inside `scripts/recommend_next_agent.py`, add one typed intermediate model for packet sections 5-9.

Use `TypedDict` definitions in the same file. No new module.

Minimum shapes:

- `DecisionSurface`
- `RecommendationSection`
- `EvaluationRecipeSection`
- `RepoFitSection`
- `RequiredArtifactsSection`
- `WorkstreamsSection`

The builder function:

`build_decision_surface(seed, dossiers, scores, candidate_results, shortlist_ids, recommended_agent_id) -> DecisionSurface`

It must derive:

- winner rationale tied to exact evidence ids / probe refs
- loser rationale for the other two shortlisted candidates
- reproducible-now steps
- blocked-until-later steps
- repo-fit expectations derived from descriptor fields and current repo architecture
- required artifacts grouped by onboarding stage
- workstreams / deliverables / blocking risks / acceptance gates

### 3. Validate the decision surface before rendering markdown

Add:

`validate_decision_surface(surface: DecisionSurface) -> None`

It must fail when any required subsection is missing or empty.

Examples of invalid surface state:

- no loser rationale for a shortlisted loser
- no runnable commands in section 6
- section 7 missing wrapper-crate expectations
- section 8 not grouping artifacts by onboarding stage
- section 9 missing acceptance gates

### 4. Render markdown from the structured model

Update `render_comparison_packet(...)` so sections 5-9 are rendered from `DecisionSurface`, not hand-assembled boilerplate.

Keep these invariants unchanged:

- section order
- provenance lines
- exactly-3 table shape
- final section-5 decision block
- section-6 split into `reproducible now` and `blocked until later`

### 5. Keep approval governance unchanged

`render_approval_toml(...)` stays the same except for consuming the same shortlisted / approved candidate choices already produced by the hardened lane.

No Rust schema change.  
No `comparison_ref` change.  
No change to override semantics.

## Test Diagram & Verification Matrix

```text
CODE PATH COVERAGE
===========================
[+] Gate sufficiency evaluation
    │
    ├── [GAP] strict verified-only claims reject `inferred`
    ├── [GAP] required evidence kinds missing -> named rejection
    ├── [GAP] required probe failure -> candidate_error or reject
    └── [GAP] fewer than 3 eligible -> insufficient_eligible_candidates

[+] Decision surface builder
    │
    ├── [GAP] winner rationale cites exact evidence ids
    ├── [GAP] each shortlisted loser gets explicit rationale
    ├── [GAP] section 6 split is populated with commands/artifacts/blockers
    ├── [GAP] section 7 contains all repo-fit categories
    ├── [GAP] section 8 contains staged artifact categories
    └── [GAP] section 9 contains workstreams, risks, and gates

[+] Packet / validator alignment
    │
    ├── [GAP] semantically empty sections 5-9 are rejected
    ├── [GAP] template subsection labels are enforced
    └── [EXISTING] section order / provenance lines are enforced

[+] Promote / approval path
    │
    ├── [EXISTING] scratch packet remains immutable after promote
    ├── [EXISTING] canonical packet is byte-identical to committed run copy
    ├── [EXISTING] override artifact requires override_reason
    └── [EXISTING] xtask onboard-agent dry-run validates final TOML

─────────────────────────────────
CURRENT COVERAGE
  Strong: 4 paths
  Shallow: 4 paths
  Gaps: 9 paths

TARGET COVERAGE AFTER THIS PLAN
  Strong: all critical paths above
  Shallow: 0 for sections 5-9
  Critical gaps: 0
─────────────────────────────────
```

## Test Plan

### New / expanded tests in `scripts/test_recommend_next_agent.py`

1. `test_generate_rejects_inferred_non_interactive_execution_even_with_evidence_ids`
   - regression test
   - proves the current bad behavior is gone

2. `test_generate_rejects_reproducibility_without_both_doc_and_package_evidence`
   - hard-gate sufficiency test

3. `test_generate_requires_required_probe_for_observable_cli_surface_when_requested`
   - probe rule test

4. `test_generate_allows_inferred_redaction_fit_only_with_allowed_evidence_kind_and_no_blocked_by`
   - allowed-inference rule test

5. `test_decision_surface_validation_rejects_missing_repo_fit_categories`
   - semantic validation test

6. `test_decision_surface_validation_rejects_missing_artifact_or_gate_sections`
   - semantic validation test

7. `test_generated_packet_sections_5_through_9_match_expected_golden_output`
   - golden test
   - compare a deterministic fixture packet body or section slice, not just marker presence

8. `test_packet_contract_rejects_semantically_empty_sections_5_through_9`
   - negative packet-contract test

9. `test_promote_preserves_scratch_outputs_and_only_changes_allowed_review_fields`
   - keep existing regression coverage

10. `cargo test -p xtask --test recommend_next_agent_approval_artifact -- --nocapture`
    - unchanged Rust approval boundary proof

11. `cargo test -p xtask --test onboard_agent_entrypoint approval_mode -- --nocapture`
    - unchanged create-lane handoff proof

12. `cargo run -p xtask -- onboard-agent --approval docs/agents/lifecycle/<pack>/governance/approved-agent.toml --dry-run`
    - proving-run acceptance gate

## Failure Modes Registry

| Codepath | Realistic production failure | Test required | Error handling | User impact if unhandled |
| --- | --- | --- | --- | --- |
| gate sufficiency evaluator | candidate with docs-only `inferred` claims reaches shortlist | yes | named rejection reason in candidate validation result | silent false confidence in shortlist |
| required probe handling | probe times out and lane still claims observable CLI proof | yes | `candidate_error` or rejection with explicit reason | maintainer approves a non-runnable candidate |
| decision surface builder | packet recommends a winner but hides provider-backed blocker | yes | blocked step must be rendered in section 6 and section 9 risks | human sees an “approve” packet that should have been “stop” |
| repo-fit render | section 7 omits wrapper/backend seam expectations | yes | `validate_decision_surface(...)` must fail | runtime onboarding starts with missing seam map |
| artifact/workstream render | sections 8-9 are syntactically present but operationally empty | yes | packet validation must fail before promote | maintainer gets no usable handoff |
| promote / approval path | approval TOML drifts from packet decision outcome | existing + keep | Rust dry-run validation | control-plane mutation against wrong packet |

Current critical gaps before implementation:

1. inferred hard-gate claims can pass silently  
2. semantically empty decision sections can still promote

This plan must close both.

## Implementation Plan

### Workstream 1 — Contract and template truth

Files:

- `docs/specs/cli-agent-recommendation-dossier-contract.md`
- `docs/templates/agent-selection/cli-agent-selection-packet-template.md`
- `.codex/skills/recommend-next-agent/SKILL.md`

Tasks:

1. Add `Hard Gate Sufficiency Rules` to the dossier contract.
2. Promote section 7-9 subsection labels from “recommended” to required.
3. State explicitly that the packet is the maintainer decision surface while `approved-agent.toml` remains the normative approval artifact.
4. Update the skill so research authors know which claims may still use `inferred`.

Acceptance gate:

- contract, template, and skill say the same thing about critical claim proof and packet section content

### Workstream 2 — Runner semantics

Files:

- `scripts/recommend_next_agent.py`

Tasks:

1. Add `HARD_GATE_RULES`.
2. Replace the current `claim_state_allows_gate_pass(...)` behavior.
3. Add `evaluate_gate_sufficiency(...)`.
4. Add named rejection reasons and rule ids into candidate validation output.
5. Preserve the current shortlist order, score buckets, and promote flow.

Acceptance gate:

- a dossier with insufficient critical proof never reaches the eligible shortlist

### Workstream 3 — Structured decision surface

Files:

- `scripts/recommend_next_agent.py`

Tasks:

1. Add `TypedDict` models for sections 5-9.
2. Build the decision surface from shortlist data plus the recommended dossier.
3. Add `validate_decision_surface(...)`.
4. Render sections 5-9 from the validated model.

Acceptance gate:

- sections 5-9 are non-empty because the model is non-empty, not because strings happened to be present

### Workstream 4 — Tests and proving run

Files:

- `scripts/test_recommend_next_agent.py`
- `docs/agents/selection/cli-agent-selection-packet.md`
- `docs/agents/selection/runs/<fresh-run-id>/**`
- `docs/agents/lifecycle/<pack>/governance/approved-agent.toml`

Tasks:

1. Add the new gate sufficiency tests.
2. Add the decision-surface validation tests.
3. Replace marker tests with a real section-5-through-9 golden test.
4. Generate one fresh promoted run with the hardened lane.
5. Re-run the Rust approval tests and `xtask` dry-run.

Acceptance gate:

- green Python tests
- green Rust approval tests
- green `xtask onboard-agent --approval ... --dry-run`
- canonical packet byte-identical to the committed run copy

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
3. allowed inferred claims have explicit evidence-kind rules.
4. packet sections 5-9 are rendered from a structured model and validated before render.
5. the packet template and contract require the exact subsection labels used by the renderer.
6. semantically empty sections 5-9 fail tests.
7. one fresh promoted run exists and its canonical packet matches the committed run copy byte-for-byte.
8. the final approval artifact still passes the real Rust loader and `xtask` dry-run path.

## Distribution Check

No new artifact type is introduced here.

The lane still publishes:

- committed review evidence under `docs/agents/selection/runs/`
- one canonical comparison packet
- one approval artifact consumed by `xtask`

No new CI/CD channel is required for this slice.

## Sequential Implementation, No Parallelization Opportunity

The semantic contract, runner logic, packet renderer, validator behavior, and tests all converge on the same primary module and the same exact section requirements.

Parallel worktrees would create merge churn with almost no throughput gain. Implement this sequentially:

1. contract/template/skill truth
2. runner semantics
3. decision-surface render + validation
4. tests
5. fresh proving run and dry-run validation

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
| 8 | Eng | Replace marker tests with golden + negative semantic tests | mechanical | P1 completeness | presence tests are too weak for a governance surface | more `assertIn(...)` |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 1 | CLEAR | narrowed the milestone to trust-surface hardening; kept approval artifact normative; deferred finalist truth sprint |
| Codex Review | `codex exec` | Independent 2nd opinion | 2 | CLEAR | both outside reads agreed the right slice is semantics, not formatting; both warned against hardening prose without hardening truth |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 1 | CLEAR | 2 critical gaps identified and closed in plan scope: permissive hard-gate semantics and semantically thin sections 5-9 |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | — | skipped, no real UI scope in this follow-on |

**CODEX:** Two outside passes both converged on the same warning: do not ship a string-level patch. Fix gate semantics and decision-surface structure together.

**CROSS-MODEL:** Primary review and Codex agreed on the same high-confidence theme, the lane’s defect is semantic trust, not missing mechanics.

**UNRESOLVED:** 0

**VERDICT:** CEO + ENG CLEARED — ready to implement. This plan is intentionally narrow and fully executable without further scope decisions.
