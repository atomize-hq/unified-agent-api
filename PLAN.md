# PLAN — Recommendation Lane V2 Discovery And Evaluation Split

Status: ready for implementation  
Date: 2026-04-28  
Branch: `codex/recommend-next-agent`  
Base branch: `main`  
Repo: `atomize-hq/unified-agent-api`

## Source Inputs

- Initial design artifact:
  - external gstack project design artifact `spensermcconnell-codex-recommend-next-agent-v2-design-20260428-201353.md`
- Live repo workflow surfaces:
  - `.codex/skills/recommend-next-agent/SKILL.md`
  - `scripts/recommend_next_agent.py`
  - `scripts/test_recommend_next_agent.py`
  - `docs/specs/cli-agent-recommendation-dossier-contract.md`
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
  - `docs/templates/agent-selection/cli-agent-selection-packet-template.md`
  - `docs/agents/selection/candidate-seed.toml`
  - `docs/agents/selection/cli-agent-selection-packet.md`
  - `docs/agents/selection/runs/20260428T185841Z-cli-recommendation/**`
  - `crates/xtask/data/agent_registry.toml`
  - `crates/xtask/src/approval_artifact.rs`

## Outcome

Land the missing discovery lane in front of the already-shipped seeded evaluation lane.

This plan does not replace the deterministic runner. It does not reopen `xtask onboard-agent`. It does not merge discovery and evaluation into one opaque AI session.

It does four things, end to end:

1. Add a bounded discovery stage that can find plausible candidate CLIs without hand-authoring the initial seed.
2. Freeze discovery output into a reviewed seed snapshot before research and scoring begin.
3. Keep `scripts/recommend_next_agent.py generate` deterministic and post-research only.
4. Add the structured insufficiency loop so “fewer than 3 eligible candidates” returns to discovery once instead of forcing undocumented seed surgery.

## Problem Statement

The current recommendation lane is real, but it starts too late.

Today the repo can:

- read a committed seed file
- freeze research artifacts for that exact pool
- run deterministic eligibility and shortlist scoring
- render a canonical packet
- render `approved-agent.toml`
- hand off cleanly into `xtask onboard-agent`

What it cannot do yet:

- discover candidates from current public market signals
- build a candidate pool from repo-fit policy plus live public surfaces
- preserve discovery provenance as a reviewable artifact
- widen the pool once when evaluation collapses below 3 eligible candidates
- give maintainers a one-command “find the next plausible agents” workflow

That is the missing product. Not “better scoring.” Discovery before scoring.

## Step 0 Scope Challenge

### What existing code already solves the sub-problems

- `scripts/recommend_next_agent.py` already owns frozen-input validation, eligibility, shortlist selection, packet render, promote, and approval-artifact drafting.
- `scripts/test_recommend_next_agent.py` already owns fixture-backed recommendation-lane contract tests.
- `docs/specs/cli-agent-recommendation-dossier-contract.md` already owns the post-research recommendation contract.
- `docs/cli-agent-onboarding-factory-operator-guide.md` already owns the operator procedure for research, generate, promote, and onboarding handoff.
- `crates/xtask/data/agent_registry.toml` already owns the onboarded-agent truth that discovery and evaluation must both respect.
- `crates/xtask/src/approval_artifact.rs` already owns normative approval-artifact validation and must remain the create-lane truth.

### Minimum change set

Keep the existing evaluation and promotion architecture. Add the smallest boring discovery layer that makes the flow complete.

The minimum authored touch set is:

- `.codex/skills/recommend-next-agent/SKILL.md`
- `docs/specs/cli-agent-recommendation-dossier-contract.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `scripts/recommend_next_agent.py`
- `scripts/test_recommend_next_agent.py`
- one fresh promoted recommendation run under `docs/agents/selection/runs/<fresh-run-id>/`
- refreshed canonical packet at `docs/agents/selection/cli-agent-selection-packet.md`
- resulting `docs/agents/lifecycle/<pack>/governance/approved-agent.toml`

Optional but likely touched:

- `docs/templates/agent-selection/cli-agent-selection-packet-template.md`

### Complexity check

This slice adds one new workflow stage, but it should still stay inside one script and one skill.

No new Rust command.  
No new crate.  
No new service.  
No new committed artifact root outside `docs/agents/selection/runs/**`.

The real implementation remains concentrated in:

- one repo-local skill
- one Python control-plane script
- one Python test file
- one contract doc
- one operator guide

That is “engineered enough.” Anything larger is spending an innovation token for no user benefit.

### Search check

Discovery is the one part of this lane that legitimately needs live search.

Use search only for frontier generation and candidate nomination. Do not let search become score truth. The boring validation layers remain:

- official docs
- GitHub repos / releases
- package registries
- frozen research dossiers

That is the right [Layer 1] / [Layer 2] split:

- [Layer 2] web search for “what should we even look at?”
- [Layer 1] deterministic validated inputs for “is this candidate actually viable?”

### TODOS cross-reference

`TODOS.md` already names the active follow-on:

`Land The LLM-Guided Research Layer For The Recommendation Lane`

This plan is that TODO, but reframed to make the missing seam concrete:

- discovery first
- research second
- evaluation third
- approval handoff unchanged

No new TODO should be added before this lands. This is the current bottleneck.

### Completeness check

The shortcut version would be:

- ask the model to suggest names ad hoc
- paste those names into `candidate-seed.toml`
- pretend that counts as discovery

That saves almost no time and preserves the core defect: no durable boundary between “the model searched” and “the runner evaluated.”

The complete version is still a boilable lake:

- bounded discovery artifacts
- reviewed seed freeze
- deterministic evaluation on frozen inputs
- one structured retry loop
- explicit review evidence from discovery through approval-ready output

### Distribution check

No new runtime artifact is introduced.

This plan publishes process truth, not a new binary:

- scratch discovery and research artifacts under `docs/agents/.uaa-temp/**`
- committed review evidence under `docs/agents/selection/runs/**`
- canonical packet at `docs/agents/selection/cli-agent-selection-packet.md`
- approval artifact at `docs/agents/lifecycle/<pack>/governance/approved-agent.toml`

No CI publish workflow is needed beyond the existing docs / control-plane gates.

## Premise Challenge

### Premises kept

1. Discovery and evaluation should be separate lanes with different trust models.
2. The deterministic runner should stay post-research only.
3. `approved-agent.toml` remains the normative approval artifact.
4. Already-onboarded agents remain hard-ineligible everywhere.
5. Discovery should produce a reviewed seed artifact, not shortlist or approve directly.
6. The skill should own the full orchestrated experience from discovery through approval-ready output.

### Premises rejected

1. The committed `docs/agents/selection/candidate-seed.toml` should not remain the primary runtime input for v2.
   It stays as a fallback example and baseline curated pool, not the reviewed seed for the main flow.
2. The evaluation runner should not fetch open-ended evidence.
   That would collapse the trust boundary and make replay/audit weaker.
3. “Auto-expand when under 3 eligible” should not mean unbounded retries.
   One bounded widening pass is enough for MVP.

## Scope Lock

### In scope

- add scratch discovery artifacts under `docs/agents/.uaa-temp/recommend-next-agent/discovery/<run_id>/`
- add optional discovery hints input at `docs/agents/selection/discovery-hints.json`
- define discovery artifact schemas and validation rules
- add a freeze step that copies reviewed discovery output into the research root
- change `generate` so it treats `research/<run_id>/seed.snapshot.toml` as the seed authority
- preserve deterministic research, evaluation, shortlist, packet, and promote behavior after freeze
- add one structured insufficiency retry path back to discovery
- preserve promote-time validation and `xtask onboard-agent --approval ...` unchanged
- commit discovery provenance into the promoted review run

### Out of scope

- replacing the deterministic evaluation runner with an AI-only workflow
- redesigning shortlist scoring dimensions
- widening the packet into a second approval artifact
- adding screenshots, browser automation, or cached HTML bodies to discovery
- adding more than one widening retry in a single invocation
- introducing Reddit, Discord, or private sources into discovery
- migrating recommendation logic into `xtask`
- onboarding the selected winner itself

## NOT in scope

- New Rust control-plane commands.
  Rationale: the Python runner already owns this workflow and the approval boundary is already in Rust.
- Discovery-owned scoring or ranking.
  Rationale: discovery nominates candidates; evaluation scores shortlisted candidates.
- Autonomous multi-pass frontier search.
  Rationale: one widening retry is enough for MVP and keeps operator cost bounded.
- Committed reviewed-seed canon under `docs/agents/selection/`.
  Rationale: keep committed canon limited to promoted review evidence and the packet, not another seed truth surface.
- Replacing the current packet shape.
  Rationale: the maintainer decision surface is already established and downstream artifacts rely on it.

## What already exists

- `docs/agents/selection/candidate-seed.toml` already expresses the current curated pool and descriptor defaults.
- `scripts/recommend_next_agent.py` already validates dossiers, rejects already-onboarded agents, scores candidates, renders the packet, promotes review artifacts, and drafts `approved-agent.toml`.
- `scripts/test_recommend_next_agent.py` already proves the core evaluation and promote invariants.
- `docs/specs/cli-agent-recommendation-dossier-contract.md` already freezes the research and evaluation artifact model.
- `docs/agents/selection/runs/20260428T185841Z-cli-recommendation/**` already proves the post-research lane works mechanically.
- `crates/xtask/data/agent_registry.toml` already defines the hard ineligibility boundary for onboarded agents such as `opencode` and `gemini_cli`.

The plan reuses all of that. Discovery is the missing front half, not a replacement system.

## Existing Code Leverage Map

| Sub-problem | Existing surface to reuse | Planned change |
| --- | --- | --- |
| Candidate eligibility truth | `crates/xtask/data/agent_registry.toml` | reuse as discovery and evaluation hard reject list |
| Frozen research/run lifecycle | `scripts/recommend_next_agent.py` | keep intact, but feed it a reviewed frozen seed instead of the live committed seed |
| Recommendation contract | `docs/specs/cli-agent-recommendation-dossier-contract.md` | extend it upward to include discovery and freeze rules |
| Operator flow | `docs/cli-agent-onboarding-factory-operator-guide.md` | make discovery the front door and document the retry loop |
| Canonical packet + approval artifact | promote path + `approval_artifact.rs` | keep unchanged except for richer provenance and reviewed-input source |
| Regression safety | `scripts/test_recommend_next_agent.py` | add discovery, freeze, retry-loop, and reviewed-seed tests |

## Dream State

```text
CURRENT
  committed candidate-seed.toml
    -> frozen research
    -> deterministic evaluation
    -> packet + approval artifact
  good back half, missing front half

THIS PLAN
  discovery
    -> reviewed generated seed
    -> frozen research
    -> deterministic evaluation
    -> bounded insufficiency retry once
    -> packet + approval artifact
  complete recommendation lane

12-MONTH IDEAL
  discovery
    -> research
    -> deterministic recommendation
    -> finalist proving
    -> onboarding
    -> maintenance
  one continuous evidence chain across the full lifecycle
```

## Implementation Alternatives

### Approach A: Keep hand-authored `candidate-seed.toml`, only tighten the skill prose

Summary: leave the runtime lane unchanged and tell maintainers to use the skill more carefully when building the seed.

Effort: S  
Risk: High

Pros:

- smallest code diff
- no artifact-model changes

Cons:

- does not produce a durable discovery boundary
- does not create a reviewable discovery provenance surface
- keeps “find candidates” as manual hidden work

### Approach B: Add discovery artifacts plus a freeze step, keep evaluation deterministic

Summary: let the skill own public-surface discovery, write bounded scratch artifacts, freeze a reviewed seed into the research root, then run the existing deterministic lane.

Effort: M  
Risk: Medium

Pros:

- fixes the real missing product without reopening the deterministic back half
- preserves replay and audit where they already matter
- gives maintainers a one-command path with explicit review seams
- keeps the blast radius mostly in one script and two docs

Cons:

- adds one more scratch artifact root
- requires contract changes in both docs and tests

### Approach C: Make `generate` discover candidates itself

Summary: teach the runner to search the web, form the pool, research, score, and promote in one pass.

Effort: L  
Risk: High

Pros:

- one CLI command on paper
- fewer explicit stages

Cons:

- destroys the clean post-research deterministic boundary
- makes audit and replay weaker
- mixes probabilistic search with contract-heavy evaluation

## Recommendation

Choose Approach B.

That is the smallest plan that makes the lane complete without turning the deterministic runner into a magical blob.

Discovery should be open-ended. Evaluation should stay boring. That is the whole game.

## Architecture Review

### Architecture decision

Keep the lane split into four explicit stages:

`discovery -> freeze -> research -> deterministic evaluation/promote`

The new rule is simple:

- discovery may search
- freeze validates and snapshots
- research writes dossiers against the frozen snapshot
- evaluation consumes only the frozen snapshot and dossiers

### State machine

```text
discovery_pending
  |
  v
discovery_complete
  |
  v
seed_reviewed
  |
  v
research_frozen
  |
  v
evaluation_complete ----------------------+
  |                                       |
  | >= 3 eligible                         | < 3 eligible
  v                                       |
packet_promoted                           |
  |                                       |
  v                                       |
approval_ready                            |
                                          |
evaluation_insufficient_candidates -------+
  |
  | if pass_count == 1
  v
discovery_pending (widened pass 2)

If pass_count == 2 and still < 3 eligible:
  stop with explicit insufficiency outcome
```

### Dependency graph

```text
docs/agents/selection/discovery-hints.json (optional)
                    |
                    v
  .codex/skills/recommend-next-agent/SKILL.md
                    |
                    v
docs/agents/.uaa-temp/recommend-next-agent/discovery/<run_id>/
  ├── candidate-seed.generated.toml
  ├── discovery-summary.md
  └── sources.lock.json
                    |
                    v
scripts/recommend_next_agent.py freeze-discovery
  ├── validate_discovery_seed(...)
  ├── validate_discovery_summary(...)
  ├── validate_discovery_sources_lock(...)
  ├── reject_registry_duplicates(...)
  └── copy reviewed artifacts into research root
                    |
                    v
docs/agents/.uaa-temp/recommend-next-agent/research/<run_id>/
  ├── seed.snapshot.toml
  ├── discovery-input/
  │   ├── candidate-seed.generated.toml
  │   ├── discovery-summary.md
  │   └── sources.lock.json
  ├── research-summary.md
  ├── research-metadata.json
  └── dossiers/<agent_id>.json
                    |
                    v
scripts/recommend_next_agent.py generate
  ├── read only research/<run_id>/seed.snapshot.toml
  ├── validate dossiers against frozen snapshot
  ├── hard-reject onboarded agents again
  ├── score eligible candidates
  ├── emit insufficiency summary if < 3 eligible
  └── render packet + approval draft
                    |
                    v
scripts/recommend_next_agent.py promote
  ├── copy committed review artifacts
  ├── copy discovery provenance into committed run
  ├── update canonical packet
  ├── render final approved-agent.toml
  └── validate with xtask dry-run
```

### Discovery artifact model

#### Scratch discovery root

New scratch root:

`docs/agents/.uaa-temp/recommend-next-agent/discovery/<run_id>/`

Required files:

- `candidate-seed.generated.toml`
- `discovery-summary.md`
- `sources.lock.json`

No screenshots.  
No HTML cache.  
No extra artifact types in MVP.

#### Discovery hints

Optional control-plane input:

`docs/agents/selection/discovery-hints.json`

Exact MVP shape:

```json
{
  "preferred_licenses": ["oss", "commercial_ok"],
  "avoid_account_gated": false,
  "prefer_observable_cli": true,
  "include_candidates": ["aider"],
  "exclude_candidates": ["opencode"],
  "notes": "short optional maintainer guidance"
}
```

Rules:

- every field is optional
- `preferred_licenses` may contain only `oss` and `commercial_ok`
- `include_candidates` and `exclude_candidates` must be unique
- if the same candidate appears in both `include_candidates` and `exclude_candidates`, `exclude_candidates` wins
- `include_candidates` may bypass soft discovery preferences, but must not bypass hard discovery rejections
- `exclude_candidates` is absolute for the current invocation, even if the candidate would otherwise be nominated by search
- `notes` is advisory only and must never be parsed into scoring, ranking, or hidden inclusion rules
- hints influence discovery inclusion, not evaluation scoring

Strict schema rules:

- unknown top-level keys are a validation error
- non-array values for `preferred_licenses`, `include_candidates`, or `exclude_candidates` are a validation error
- non-boolean values for `avoid_account_gated` or `prefer_observable_cli` are a validation error
- non-string entries inside any array field are a validation error
- empty strings are invalid in `include_candidates`, `exclude_candidates`, and `notes`

#### `candidate-seed.generated.toml`

Keep the current seed shape:

- `[defaults.descriptor]`
- `[candidate.<agent_id>]`

Required per-candidate fields:

- `display_name`
- `research_urls`
- `install_channels`
- `auth_notes`

Discovery-specific reasoning does not go in the generated seed. It goes in `discovery-summary.md`.

#### `discovery-summary.md`

Required top section:

- discovery run id
- discovery pass number
- exact query strings used
- source classes consulted
- hints file used or `none`
- candidate ids nominated by web-search frontier signals
- candidate ids nominated by direct official-source discovery

Required per-candidate section:

- candidate id and display name
- why it entered the pool
- which source first introduced it
- which hint or default policy affected inclusion
- one obvious caveat known before research freeze

#### Discovery `sources.lock.json`

Purpose: provenance lock, not internet replay engine.

Required per-entry fields:

- `candidate_id`
- `source_kind`
- `url`
- `title`
- `captured_at`
- `sha256`
- `role`, one of `frontier_signal`, `discovery_seed`, `install_surface`, `docs_surface`

For `web_search_result` entries only, also require:

- `query`
- `rank`

Hashing rule:

- `sha256` must be the SHA-256 of a canonical UTF-8 serialization of exactly this per-entry object, with keys sorted and no extra whitespace:
  - `candidate_id`
  - `source_kind`
  - `url`
  - `title`
  - `captured_at`
  - `role`
  - plus `query` and `rank` when `source_kind = web_search_result`
- the hash must not be computed from live page bodies, screenshots, or fetched HTML
- two identical logical source entries must therefore produce the same `sha256` across reruns

### Discovery nomination rules

Pass 1 exact query set:

- `best AI coding CLI`
- `AI agent CLI tools`
- `developer agent command line`

Pass 1 nomination algorithm:

1. Collect candidates from the first-page results for the three fixed queries above.
2. Normalize candidate ids to the upstream project / CLI identity used in the repo.
3. Deduplicate by normalized candidate id.
4. Drop candidates rejected by any hard discovery rejection rule.
5. Apply `exclude_candidates`.
6. Force-add valid `include_candidates` that are not already present.
7. Sort remaining candidates by:
   - highest count of distinct source entries
   - then presence of both docs and install surfaces
   - then alphabetical `candidate_id`
8. Emit exactly 5 candidates unless fewer than 5 survive hard rejection. This is a hard cap, not guidance.

Pass 2 widening query set:

- `alternatives to <top surviving candidate>`
- `top coding agent CLI open source`
- `CLI coding assistant blog`

Pass 2 nomination algorithm:

1. Start from the pass 1 rejection summary.
2. Search only the fixed widening query family above.
3. Exclude every candidate already seen in pass 1, whether accepted or rejected.
4. Apply the same hard discovery rejection rules.
5. Allow soft-preference relaxation only as already defined in this plan.
6. Emit at most 3 new candidates. This is a hard cap.

### Architecture findings resolved in this plan

1. The reviewed seed authority must move from the live committed seed file to the frozen research snapshot.
   That removes the current ambiguity about which seed actually powered the run.
2. Discovery provenance must survive promotion.
   If the committed run only preserves evaluation evidence, maintainers still cannot audit why a candidate entered the pool.
3. Already-onboarded rejection must happen twice.
   Discovery should filter onboarded agents early, and `generate` should still reject them again as defense in depth.
4. The insufficiency loop must be structured, not conversational.
   The skill needs a machine-readable outcome telling it whether to widen or stop.

## Code Quality Review

### Code-organization decisions

- Keep all control-plane logic inside `scripts/recommend_next_agent.py`.
  Do not split discovery validation into a second script.
- Add one new subcommand:
  - `freeze-discovery`
- Keep `generate` and `promote` as the only other public subcommands.
- Add typed helpers for discovery artifacts near the existing seed/dossier helpers.
- Extend existing promote logic instead of adding a second promote path for discovery-enabled runs.

### DRY rules for this slice

- One normative recommendation-lane contract file:
  - `docs/specs/cli-agent-recommendation-dossier-contract.md`
- One operator workflow doc:
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
- One discovery-validation implementation:
  - `scripts/recommend_next_agent.py freeze-discovery`
- One deterministic evaluation implementation:
  - `scripts/recommend_next_agent.py generate`

### Minimal-diff guardrails

- Do not add a second packet format.
- Do not add a second committed seed canon.
- Do not change `approved-agent.toml` schema.
- Do not add new Rust validation unless dry-run proves a real approval-boundary defect.
- Do not make discovery a general-purpose crawler. Stay within the bounded source classes.

## Performance Review

This slice adds search and artifact validation, so the performance risks are mostly about not letting discovery sprawl.

- Cap discovery to 2 passes per invocation.
- Pass 1 must emit at most 5 unique candidates.
- Pass 2 must add at most 3 new unique candidates.
- Hard cap the reviewed seed at 8 candidates before research freeze.
- `freeze-discovery` validation must stay linear in candidate count and source-lock entries.
- `generate` must continue to operate in `O(candidates + dossiers + evidence)` time with no network calls.

Performance is not the hard problem here. Boundedness is.

## Contract Changes

### 1. Extend the recommendation contract upward to discovery

Keep `docs/specs/cli-agent-recommendation-dossier-contract.md` as the single normative recommendation-lane contract for MVP, even though the filename predates discovery.

Add new top-level sections for:

- discovery input contract
- allowed discovery sources and roles
- discovery artifact schemas
- freeze step semantics
- bounded insufficiency retry rules

This avoids creating two normative docs for one workflow.

### 2. Change the reviewed seed authority

Update the contract so:

- `research/<run_id>/seed.snapshot.toml` is the only reviewed seed used by `generate`
- `generate` must not reread `docs/agents/selection/candidate-seed.toml`
- `candidate-seed.toml` remains a fallback curated pool and example, not the reviewed runtime input for v2

### 3. Define discovery-source classes and reject unsupported ones

Allowed source classes in MVP:

- `web_search_result`
- `official_doc`
- `github`
- `package_registry`

Explicitly disallowed in MVP:

- Reddit
- Discord
- private/internal sources

### 4. Define obvious discovery-stage rejections

Discovery must reject before research when any of these are true:

- candidate already exists in `agent_registry.toml`
- no public CLI install path can be found
- no public CLI docs or CLI evidence surface exists
- candidate is primarily an SDK/library with no standalone CLI workflow
- candidate requires closed/private distribution with no public install or docs evidence

`generate` must still enforce onboarded-agent rejection again after freeze.

### 5. Define the bounded widening loop

Exact MVP rule:

- maximum 2 discovery passes per skill invocation
- pass 1 uses default discovery
- pass 2 widens using adjacent candidates and relaxed soft preferences only
- if fewer than 3 eligible candidates survive after pass 2, stop with `evaluation_insufficient_candidates`
- if pass 1 yields fewer than 3 discovered candidates after hard rejection, pass 2 is still allowed once
- no pass may mutate or delete prior-pass discovery artifacts; each pass must write its own `<run_id>`-scoped discovery directory

### 6. Extend the promoted review artifact set

Promoted run must now also include discovery provenance under:

`docs/agents/selection/runs/<run_id>/discovery/`

Required committed files:

- `discovery/candidate-seed.generated.toml`
- `discovery/discovery-summary.md`
- `discovery/sources.lock.json`

That keeps the why-this-pool evidence next to the why-this-shortlist evidence.

## Skill Changes

Update `.codex/skills/recommend-next-agent/SKILL.md` so the default workflow becomes:

1. read optional `docs/agents/selection/discovery-hints.json`
2. run discovery pass 1 against the fixed MVP query set and allowed public sources
3. write scratch discovery artifacts
4. stop for maintainer review or light edit of `candidate-seed.generated.toml`
5. run `freeze-discovery`
6. complete research dossiers against the frozen seed snapshot
7. run `generate`
8. if `generate` returns `insufficient_eligible_candidates`, run one widened discovery pass using the fixed pass 2 query family and repeat steps 3-7 once
9. if still insufficient, stop and report structured failure
10. otherwise review, promote, and hand off to approval

The skill must stop short of implementation. Recommendation only.

## Runner Changes

### 1. Add `freeze-discovery`

New public command:

```sh
python3 scripts/recommend_next_agent.py freeze-discovery \
  --discovery-dir docs/agents/.uaa-temp/recommend-next-agent/discovery/<run_id> \
  --research-dir docs/agents/.uaa-temp/recommend-next-agent/research/<run_id>
```

Responsibilities:

- validate `candidate-seed.generated.toml`
- validate `discovery-summary.md`
- validate discovery `sources.lock.json`
- reject duplicate candidate ids
- reject candidates already onboarded in the registry
- copy the reviewed generated seed to `research/<run_id>/seed.snapshot.toml`
- copy all three discovery artifacts to `research/<run_id>/discovery-input/`

### 2. Make `generate` read only the frozen research seed

Change the generate contract to:

```sh
python3 scripts/recommend_next_agent.py generate \
  --research-dir docs/agents/.uaa-temp/recommend-next-agent/research/<run_id> \
  --run-id <run_id> \
  --scratch-root docs/agents/.uaa-temp/recommend-next-agent/runs
```

Behavior change:

- remove dependency on `--seed-file`
- read `research/<run_id>/seed.snapshot.toml` as the seed source of truth
- continue validating that dossiers match that exact frozen seed snapshot

### 3. Preserve and surface insufficiency reasons

When fewer than 3 eligible candidates survive, `generate` must write:

- `run-status.json.status = "insufficient_eligible_candidates"`
- `run-status.json.next_action = "expand_discovery"`
- `run-summary.md` rejection summary grouped by reason

Required artifact behavior in the insufficiency case:

- must write:
  - `run-status.json`
  - `seed.snapshot.toml`
  - `candidate-pool.json`
  - `eligible-candidates.json`
  - `candidate-validation-results/<agent_id>.json` for every seeded candidate
  - `candidate-dossiers/<agent_id>.json` for every seeded candidate
  - `run-summary.md`
  - `discovery/` copied from `research/discovery-input/`
- must not write:
  - `scorecard.json`
  - `sources.lock.json` at the evaluation-run root
  - `comparison.generated.md`
  - `approval-draft.generated.toml`
- the absence of those four success-only artifacts is part of the contract, not an incidental implementation detail

Required grouped reasons:

- `already_onboarded`
- `missing_public_install_surface`
- `missing_public_cli_surface`
- `sdk_not_cli_product`
- `insufficient_dossier_proof`
- `other_candidate_error`

### 4. Copy discovery provenance into the scratch run

When `generate` runs, regardless of success or insufficiency, copy:

- `research/discovery-input/candidate-seed.generated.toml`
- `research/discovery-input/discovery-summary.md`
- `research/discovery-input/sources.lock.json`

into:

`runs/<run_id>/discovery/`

This makes promotion a straight byte-copy for discovery provenance just like the rest of the review artifacts.

### 5. Extend `promote`

`promote` must now copy `runs/<run_id>/discovery/**` into the committed review run at the same relative path.

Backward-compatibility rule:

- discovery provenance is required for all v2 runs created from this plan onward
- legacy committed runs created before this plan remain valid without `discovery/`
- `promote` must fail if asked to promote a v2 scratch run without `runs/<run_id>/discovery/**`
- tests must distinguish legacy-fixture runs from v2 runs explicitly rather than relying on file absence by accident

Promotion invariants remain:

- canonical packet is byte-identical to scratch `comparison.generated.md`
- approval artifact is rendered at promote time from approved inputs
- staged dry-run validation happens before live swap

## Test Review

### Coverage diagram

```text
CODE PATH COVERAGE
===========================
[+] Discovery artifact validation
    │
    ├── [GAP] duplicate candidate ids fail freeze
    ├── [GAP] onboarded agent in generated seed fails freeze
    ├── [GAP] discovery sources.lock missing required fields fails freeze
    ├── [GAP] unsupported source_kind fails freeze
    └── [GAP] discovery summary missing per-candidate rationale fails freeze

[+] Reviewed seed freeze
    │
    ├── [GAP] reviewed candidate-seed.generated.toml copies to research/seed.snapshot.toml
    ├── [GAP] discovery artifacts copy to research/discovery-input/**
    └── [GAP] freeze never mutates the live committed candidate-seed.toml

[+] Deterministic evaluation
    │
    ├── [GAP] generate reads research seed snapshot, not docs/agents/selection/candidate-seed.toml
    ├── [EXISTING] dossiers must match frozen snapshot sha
    ├── [EXISTING] onboarded agents are rejected before scoring
    ├── [GAP] insufficiency outcome emits next_action=expand_discovery
    └── [EXISTING] shortlist/promotion mechanics stay deterministic

[+] Promote path
    │
    ├── [GAP] discovery provenance copies into committed review run
    ├── [EXISTING] canonical packet stays byte-identical to scratch packet
    ├── [EXISTING] approval artifact dry-run validation still passes
    └── [EXISTING] rollback semantics remain intact

USER FLOW COVERAGE
===========================
[+] Default happy path
    │
    ├── discovery pass 1
    ├── maintainer reviews generated seed
    ├── freeze
    ├── research
    ├── generate >= 3 eligible
    └── promote + approval-ready output

[+] Insufficient-candidate recovery
    │
    ├── generate returns insufficient_eligible_candidates
    ├── skill widens discovery once
    ├── freeze/research/generate rerun
    └── either succeed or stop with explicit insufficiency

[+] Boundary failures
    │
    ├── discovery suggests already-onboarded agent
    ├── maintainer edits generated seed into invalid state
    ├── research dossiers do not match reviewed seed snapshot
    └── promote would otherwise lose discovery provenance

─────────────────────────────────
CURRENT COVERAGE
  Strong: deterministic generate/promote and approval-artifact boundaries
  Missing: discovery artifacts, reviewed-seed freeze, insufficiency loop
  Critical gaps: 3

CRITICAL GAPS
  1. no durable discovery provenance
  2. generate still implicitly anchors to the live committed seed
  3. insufficient-candidate recovery is manual and undocumented
─────────────────────────────────
```

### Required tests in `scripts/test_recommend_next_agent.py`

1. `test_freeze_discovery_rejects_duplicate_candidate_ids`
2. `test_freeze_discovery_rejects_onboarded_agent_ids_from_registry`
3. `test_freeze_discovery_rejects_sources_lock_with_missing_required_fields`
4. `test_freeze_discovery_rejects_unsupported_source_kind`
5. `test_freeze_discovery_copies_reviewed_seed_and_provenance_into_research_root`
6. `test_generate_reads_research_seed_snapshot_instead_of_live_candidate_seed`
7. `test_generate_emits_next_action_expand_discovery_when_fewer_than_three_eligible_survive`
8. `test_generate_copies_discovery_provenance_into_scratch_run`
9. `test_promote_copies_discovery_provenance_into_committed_review_run`
10. `test_promote_still_keeps_canonical_packet_byte_identical_to_scratch_packet`
11. `cargo test -p xtask --test recommend_next_agent_approval_artifact -- --nocapture`
12. `cargo run -p xtask -- onboard-agent --approval docs/agents/lifecycle/<pack>/governance/approved-agent.toml --dry-run`

### Regression rule

The current live-seed coupling is a behavior defect now that discovery becomes the primary entry path.

That makes test 6 mandatory. No deferral.

## Failure Modes Registry

| Codepath | Realistic production failure | Test required | Error handling | User impact if unhandled |
| --- | --- | --- | --- | --- |
| discovery artifact write | model nominates already-onboarded `opencode` again | yes | freeze rejects with explicit reason | wasted research time and a misleading pool |
| discovery provenance lock | source entry omits query or rank for web-search nomination | yes | freeze rejects malformed lock | maintainers cannot audit why candidate entered the pool |
| reviewed seed freeze | skill reviews one file but generate uses another | yes | generate reads only `research/seed.snapshot.toml` | shortlist cannot be reproduced from reviewed inputs |
| insufficiency loop | generate fails below 3 eligible and skill silently stops | yes | `next_action=expand_discovery` plus grouped rejection summary | maintainers have no idea whether to widen or fix dossiers |
| promote path | canonical run loses discovery provenance even though evaluation succeeded | yes | promote must copy `discovery/**` byte-for-byte | future reviewers can see shortlist truth but not pool-formation truth |
| approval handoff | new discovery stage accidentally changes approval artifact semantics | existing + keep | Rust dry-run validation | wrong create-lane mutation against invalid governance input |

Critical gaps are any path where there is no test, no explicit error, and silent loss of provenance. This plan closes those.

## Implementation Plan

### Workstream 1 — Normative contract and operator flow

Files:

- `docs/specs/cli-agent-recommendation-dossier-contract.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/templates/agent-selection/cli-agent-selection-packet-template.md`

Tasks:

1. Extend the contract to define discovery, freeze, and insufficiency-retry semantics.
2. Rewrite the operator guide so discovery is the primary front door.
3. Update the generate CLI contract to remove `--seed-file`.
4. Document the new committed review artifact `discovery/` subtree.

Acceptance gate:

- docs all agree on artifact roots, reviewed-seed authority, and retry behavior

### Workstream 2 — Skill orchestration

Files:

- `.codex/skills/recommend-next-agent/SKILL.md`

Tasks:

1. Make discovery pass 1 the default starting point.
2. Add optional `discovery-hints.json` intake.
3. Add explicit review/freeze step before dossier authoring.
4. Add the single widening retry rule.
5. Keep the lane ending at maintainer approve-or-override.

Acceptance gate:

- one skill invocation can drive discovery through approval-ready output without hidden manual seed surgery

### Workstream 3 — Discovery freeze and reviewed-seed authority

Files:

- `scripts/recommend_next_agent.py`

Owned functions:

- new `freeze-discovery` subcommand
- discovery artifact validators
- seed-loading path inside `generate`

Tasks:

1. Add discovery constants and artifact validators.
2. Add `freeze-discovery`.
3. Change `generate` to load only `research/seed.snapshot.toml`.
4. Keep registry-based ineligibility checks in `generate` as defense in depth.

Acceptance gate:

- the reviewed research snapshot is the only seed input used by evaluation

### Workstream 4 — Insufficiency signaling and promote provenance

Files:

- `scripts/recommend_next_agent.py`

Owned functions:

- run-status/run-summary emission
- scratch run artifact creation
- `promote`

Tasks:

1. Emit grouped insufficiency reasons and `next_action=expand_discovery`.
2. Copy discovery provenance into the scratch run.
3. Copy discovery provenance into the committed review run.
4. Preserve all existing promote-time validation and rollback behavior.

Acceptance gate:

- every promoted run explains both why the pool existed and why the shortlist won

### Workstream 5 — Tests and proving run

Files:

- `scripts/test_recommend_next_agent.py`
- `docs/agents/selection/cli-agent-selection-packet.md`
- `docs/agents/selection/runs/<fresh-run-id>/**`
- `docs/agents/lifecycle/<pack>/governance/approved-agent.toml`

Tasks:

1. Add discovery and freeze tests.
2. Add reviewed-seed-authority regression tests.
3. Add insufficiency-signaling tests.
4. Generate one fresh promoted run using the v2 flow.
5. Re-run the Rust approval test and `xtask` dry-run.

Acceptance gate:

- green tests plus one committed run proving the discovery-enabled lane end to end

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| 1. Contract + operator doc update | `docs/specs/`, `docs/`, `docs/templates/` | — |
| 2. Skill orchestration update | `.codex/skills/` | 1 |
| 3. Discovery freeze + reviewed-seed authority | `scripts/` | 1 |
| 4. Insufficiency signaling + promote provenance | `scripts/` | 3 |
| 5. Tests | `scripts/` | 3, 4 |
| 6. Proving run + canonical packet refresh | `docs/agents/selection/`, `docs/agents/lifecycle/`, `scripts/`, `crates/xtask/` | 2, 5 |

### Parallel lanes

- Lane A: step 1 -> step 2
  Docs and skill lane. Mostly prose and workflow truth.
- Lane B: step 3 -> step 4 -> step 5
  Script lane. Keep all `scripts/` work in one lane because the same module owns freeze, generate, promote, and tests.
- Lane C: step 6
  Final proving lane. Wait for A and B to merge.

### Execution order

1. Launch lane A and lane B in parallel worktrees.
2. Merge lane A first if command syntax or artifact roots changed.
3. Merge lane B after Python tests are green.
4. Run lane C last to generate the fresh promoted run and approval dry-run proof.

### Conflict flags

- Steps 3, 4, and 5 all touch `scripts/recommend_next_agent.py` and `scripts/test_recommend_next_agent.py`. Do not split them further.
- Step 6 rewrites canonical review artifacts. Do not start it before the script lane is final.

If only one engineer is available, execute the same lane order sequentially.

## Verification Commands

Run in this order:

```sh
python3 -m unittest scripts/test_recommend_next_agent.py
cargo test -p xtask --test recommend_next_agent_approval_artifact -- --nocapture
python3 scripts/recommend_next_agent.py freeze-discovery \
  --discovery-dir docs/agents/.uaa-temp/recommend-next-agent/discovery/<fresh-run-id> \
  --research-dir docs/agents/.uaa-temp/recommend-next-agent/research/<fresh-run-id>
python3 scripts/recommend_next_agent.py generate \
  --research-dir docs/agents/.uaa-temp/recommend-next-agent/research/<fresh-run-id> \
  --run-id <fresh-run-id> \
  --scratch-root docs/agents/.uaa-temp/recommend-next-agent/runs
python3 scripts/recommend_next_agent.py promote \
  --run-dir docs/agents/.uaa-temp/recommend-next-agent/runs/<fresh-run-id> \
  --repo-run-root docs/agents/selection/runs \
  --approved-agent-id <agent_id> \
  --onboarding-pack-prefix <pack-prefix> \
  [--override-reason "..."]
cargo run -p xtask -- onboard-agent --approval docs/agents/lifecycle/<pack-prefix>/governance/approved-agent.toml --dry-run
```

## Acceptance Criteria

This plan is done only when all of the following are true:

1. The default skill path starts with discovery, not hand-authored seed editing.
2. Discovery produces exactly three bounded scratch artifacts.
3. `freeze-discovery` validates and snapshots the reviewed generated seed into the research root.
4. `generate` reads only `research/<run_id>/seed.snapshot.toml` and never rereads the live committed seed.
5. Already-onboarded agents are rejected in discovery and still rejected again in evaluation.
6. Fewer-than-3-eligible outcomes emit structured insufficiency output and trigger at most one widening retry.
7. Discovery provenance survives into the committed review run.
8. The canonical packet and final approval artifact still satisfy existing promote-time and Rust dry-run invariants.
9. One fresh promoted run proves the v2 lane end to end.

## Distribution Check

No new published runtime artifact is introduced.

This slice ships workflow truth:

- scratch discovery inputs
- frozen research inputs
- committed review provenance
- canonical packet
- approval artifact

That means the only required “distribution” work is keeping the docs, script, tests, and promoted review evidence in sync.

## Completion Summary

- Step 0: Scope Challenge — accepted as-is; discovery is the missing front half and the deterministic runner remains intact
- Architecture Review: 4 architecture decisions locked, including reviewed-seed authority and discovery provenance promotion
- Code Quality Review: 5 structure constraints locked around one script, one skill, and one contract owner
- Test Review: diagram produced, 9 concrete gaps identified, 1 reviewed-seed regression test mandated
- Performance Review: bounded and acceptable, with explicit pass and candidate caps
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 proposed, because this plan is the active TODO
- Failure modes: 3 critical provenance/boundary gaps addressed by scope
- Parallelization: 3 lanes total, 2 launchable in parallel, 1 final proving lane
- Lake Score: 9/9 recommendations chose the complete option over the shortcut

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | CEO | Make discovery the default front door | mechanical | P1 completeness | the shipped lane starts too late for the intended product | keeping hand-authored seeds as the main path |
| 2 | CEO | Keep evaluation deterministic and post-research only | mechanical | P5 explicit over clever | preserves the strongest existing trust boundary | teaching generate to search the web |
| 3 | CEO | Keep `approved-agent.toml` normative | mechanical | P5 explicit over clever | prevents packet/governance role confusion | packet-native approval |
| 4 | Eng | Add `freeze-discovery` instead of a second script or a second crate | mechanical | P3 pragmatic | smallest code move that hardens the reviewed-seed seam | new Rust command or helper script sprawl |
| 5 | Eng | Make `research/seed.snapshot.toml` the only seed authority | mechanical | P1 completeness | removes ambiguity about what inputs powered the run | continuing to reread the live committed seed |
| 6 | Eng | Preserve onboarded-agent rejection in both discovery and evaluation | mechanical | P1 completeness | defense in depth on the most obvious bad candidate class | trusting discovery alone |
| 7 | Eng | Bound retry to one widening pass | taste | P3 pragmatic | fixes the common failure path without making search open-ended | autonomous multi-pass search loops |
| 8 | Eng | Promote discovery provenance into committed runs | mechanical | P1 completeness | shortlist truth without pool-formation truth is still incomplete | scratch-only discovery evidence |
| 9 | Eng | Keep all `scripts/` work in one implementation lane | mechanical | P3 pragmatic | same module owns freeze, generate, promote, and tests | splitting script work across multiple worktrees |
