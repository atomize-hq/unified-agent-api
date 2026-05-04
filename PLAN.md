# PLAN - Enclose The Recommendation Research Host Surface

Status: planned
Date: 2026-05-04
Branch: `codex/recommend-next-agent`
Base branch: `main`
Repo: `atomize-hq/unified-agent-api`
Work item: `Land The LLM-Guided Research Layer For The Recommendation Lane`
Plan commit baseline: `98b66c1`
Design input: `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-design-20260504-110422.md`

This file is the implementation plan of record for the next P1 after create-mode closeout
landed at branch head. The design doc establishes priority and direction. This plan locks the
engineering shape, file ownership, tests, failure handling, and rollout order.

## Objective

Replace the ambient `recommend-next-agent` research workflow with one repo-owned, bounded,
replayable research runner that gathers discovery and dossier evidence before the existing
deterministic `generate` and `promote` steps run.

After this lands:

1. maintainers stop relying on freehand skill execution as the real research host surface
2. one repo-owned `xtask` command owns discovery prompt rendering, Codex invocation, bounded
   writes, freeze handoff, validation, and execution evidence
3. `scripts/recommend_next_agent.py generate` stays deterministic and post-research only
4. `promote` and the final `approved-agent.toml` handoff into `xtask onboard-agent` stay
   unchanged
5. every promoted recommendation run has a durable packet proving what Codex was asked to do,
   what it wrote, and whether the repo accepted the result

The non-negotiable outcome is:

```text
repo-owned bounded AI research
  -> frozen reviewed seed
  -> structured dossiers
  -> deterministic generate/promote
  -> maintainer approve-or-override
  -> existing create lane
```

## Why This, Why Now

The design doc closed the prior milestone honestly: branch head now satisfies the create-mode
closeout plan at `98b66c1`. That doc also named the next bottleneck correctly. The remaining gap
is no longer publication or closeout control-plane mechanics. It is recommendation quality and
trust.

Today the repo owns the deterministic packet after research, but it does not own the AI research
host surface that produced the input. That is the gap this plan closes.

## Source Inputs

- Priority source:
  - `TODOS.md`
  - `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-design-20260504-110422.md`
- Normative contracts:
  - `docs/specs/cli-agent-recommendation-dossier-contract.md`
  - `docs/specs/cli-agent-onboarding-charter.md`
  - `docs/templates/agent-selection/cli-agent-selection-packet-template.md`
- Procedure and skill surfaces:
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
  - `.codex/skills/recommend-next-agent/SKILL.md`
- Existing deterministic runner:
  - `scripts/recommend_next_agent.py`
  - `scripts/test_recommend_next_agent.py`
- Existing candidate inputs:
  - `docs/agents/selection/candidate-seed.toml`
  - `docs/agents/selection/discovery-hints.json`
  - `docs/agents/selection/cli-agent-selection-packet.md`
- Reusable bounded Codex execution pattern:
  - `crates/xtask/src/runtime_follow_on.rs`
  - `crates/xtask/src/runtime_follow_on/codex_exec.rs`
  - `crates/xtask/src/runtime_follow_on/models.rs`
  - `crates/xtask/src/runtime_follow_on/render.rs`
  - `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
- Existing create-lane handoff truth:
  - `crates/xtask/src/approval_artifact.rs`
  - `crates/xtask/src/onboard_agent/**`

## Problem Statement

Current shape:

```text
maintainer
  -> follows skill text
  -> manually performs discovery / freeze / dossier authoring

python runner
  -> validates whatever research tree exists
  -> scores, renders, promotes
```

That leaves five concrete problems:

1. the repo does not own how Codex is invoked for recommendation research
2. discovery and dossier authoring are not replayable from one bounded command
3. there is no machine-generated packet proving prompt, stdout, stderr, and bounded-write results
4. pass-2 widening is still procedure text instead of repo-owned behavior
5. maintainers can trust deterministic post-research outputs, but not the host surface that
   created the research inputs

Target shape:

```text
xtask recommend-next-agent-research --dry-run
  -> validates inputs
  -> freezes the execution contract
  -> writes prompts, allowed roots, and expected artifacts

xtask recommend-next-agent-research --write
  -> runs bounded Codex discovery
  -> validates discovery outputs
  -> runs repo-owned freeze-discovery
  -> runs bounded Codex dossier authoring
  -> validates research outputs
  -> records execution evidence and run status

python3 scripts/recommend_next_agent.py generate
  -> reads only frozen research artifacts
  -> scores, renders, and reports insufficiency deterministically

python3 scripts/recommend_next_agent.py promote
  -> promotes reviewed output unchanged
  -> renders final approved-agent handoff
```

## Step 0 Scope Challenge

### Premise Check

The repo does not need:

- a new scoring model
- a new approval artifact type
- live candidate execution during research hosting
- a second deterministic runner
- packet-template redesign

The repo does need:

- one repo-owned bounded research runner in `xtask`
- one execution-packet root for research-host evidence
- one explicit split where Codex writes discovery/dossier artifacts and the repo owns every
  validation boundary around them
- direct support for pass-2 widening as code, not operator choreography

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| discovery freeze | `scripts/recommend_next_agent.py freeze-discovery` | Reuse directly. Codex must not write `seed.snapshot.toml`. |
| deterministic post-research validation | `scripts/recommend_next_agent.py generate` | Reuse directly. Do not duplicate scoring. |
| safe promotion and create-lane handoff | `scripts/recommend_next_agent.py promote` + `crates/xtask/src/approval_artifact.rs` | Reuse directly. |
| recommendation contract | `docs/specs/cli-agent-recommendation-dossier-contract.md` | Reuse as normative truth, then extend to describe repo-owned host execution. |
| bounded Codex host pattern | `crates/xtask/src/runtime_follow_on.rs` and submodules | Mirror the shape. Do not invent a second style. |
| runtime-follow-on test strategy | `crates/xtask/tests/runtime_follow_on_entrypoint.rs` + support harnesses | Reuse the harness pattern for the new xtask entrypoint. |
| canonical packet and final approval artifact | `docs/agents/selection/cli-agent-selection-packet.md` and `approved-agent.toml` | Keep unchanged. |

### Minimum Complete Change

Anything smaller leaves the trust gap open. The minimum complete change set is:

1. add `cargo run -p xtask -- recommend-next-agent-research --dry-run|--write`
2. add a research execution-packet root under `docs/agents/.uaa-temp/recommend-next-agent/research-runs/<run_id>/`
3. render discovery and research prompts from repo code, not skill prose
4. run `freeze-discovery` between discovery and dossier authoring inside the repo-owned command
5. validate that Codex writes only the allowed discovery and research roots
6. record prompt, stdout, stderr, written-path evidence, validation report, and run status
7. rewrite the skill and operator guide into thin wrappers over the repo-owned flow
8. prove the lane end to end with one committed recommendation run generated through the new host
   surface

### Complexity Check

This work will touch more than 8 files. That is acceptable and expected because the seam spans:

- xtask CLI dispatch
- repo-owned prompt rendering
- Codex execution and bounded-write validation
- recommendation-specific packet models and renderers
- docs/spec/operator/skill alignment
- xtask entrypoint tests
- possibly narrow Python test adjustments if contract wording or error handling changes

Complexity controls:

- no new scoring dimensions
- no new approval artifact type
- no general-purpose Codex orchestration framework beyond what this runner and runtime-follow-on
  genuinely share
- no public CLI shape change for `generate` or `promote`

### Search / Build Decision

- **[Layer 1]** Reuse `freeze-discovery`, `generate`, `promote`, the dossier contract, and the
  runtime-follow-on execution pattern.
- **[Layer 1]** Reuse existing `xtask` entrypoint and test-harness conventions.
- **[Layer 3]** The product gap is not more Python heuristics. The missing product is repo-owned
  execution of the AI research step.

No web research is required. The repo already contains the authoritative constraints.

### Distribution Check

No new external artifact type is introduced. This is an internal `xtask` + docs + skill workflow
change only. No build or publish pipeline work is required.

## Locked Decisions

1. The host surface is a repo-owned `xtask` command plus a thin skill wrapper, not a skill-only
   procedure.
2. The Python runner remains post-research only. It does not become the Codex host.
3. The repo, not Codex, owns `freeze-discovery`.
4. The execution packet root is `docs/agents/.uaa-temp/recommend-next-agent/research-runs/<run_id>/`.
5. Codex may write only under:
   - `docs/agents/.uaa-temp/recommend-next-agent/discovery/<run_id>/`
   - `docs/agents/.uaa-temp/recommend-next-agent/research/<run_id>/`
6. `generate` and `promote` keep their current public CLI shape.
7. Pass 1 and pass 2 query families stay frozen to the contract and are rendered from repo code.
8. Pass 2 widening is supported directly by the new runner and requires prior insufficiency input.
9. Safe local `help` and `version` probes remain owned by the Python runner and dossier contract,
   not by the new xtask host.
10. The canonical packet path and `approved-agent.toml` handoff path remain unchanged.

## Architecture

### New Command Contract

Add a new xtask subcommand:

```text
cargo run -p xtask -- recommend-next-agent-research --dry-run --pass pass1 [--run-id <id>]
cargo run -p xtask -- recommend-next-agent-research --write   --pass pass1 --run-id <id>

cargo run -p xtask -- recommend-next-agent-research --dry-run --pass pass2 --prior-run-dir <run_dir> [--run-id <id>]
cargo run -p xtask -- recommend-next-agent-research --write   --pass pass2 --prior-run-dir <run_dir> --run-id <id>
```

Argument rules:

- `--run-id` is optional for `--dry-run`, required for `--write`
- `--pass` is required and limited to `pass1|pass2`
- `--prior-run-dir` is required for `pass2`, forbidden for `pass1`
- `--codex-binary <path>` is optional and mirrors `runtime-follow-on`
- `--write` requires a preexisting dry-run packet for the same `run_id`

### Artifact Ownership

| Root | Owner | Purpose |
| --- | --- | --- |
| `docs/agents/.uaa-temp/recommend-next-agent/research-runs/<run_id>/` | xtask | execution packet, prompts, stdout/stderr, written-path evidence, validation report, run status |
| `docs/agents/.uaa-temp/recommend-next-agent/discovery/<run_id>/` | Codex, bounded by xtask | discovery artifacts only |
| `docs/agents/.uaa-temp/recommend-next-agent/research/<run_id>/` | Codex plus repo-owned `freeze-discovery` | frozen seed plus dossiers and research metadata |
| `docs/agents/.uaa-temp/recommend-next-agent/runs/<run_id>/` | existing Python runner | deterministic evaluation outputs |
| `docs/agents/selection/runs/<run_id>/` | existing `promote` | committed review evidence |

### End-To-End Flow

```text
xtask recommend-next-agent-research --dry-run
  -> load discovery hints + registry + contract inputs
  -> render input-contract.json
  -> render discovery-prompt.md and research-prompt.md
  -> write run-status.json = dry_run_prepared

xtask recommend-next-agent-research --write
  -> re-load dry-run contract
  -> execute Codex discovery with bounded write roots
  -> validate discovery artifact set
  -> run python3 scripts/recommend_next_agent.py freeze-discovery
  -> execute Codex research with bounded write roots
  -> validate research artifact set and seed identity
  -> write validation-report.json, codex-execution*.json, written-paths*.json, run-summary.md

python3 scripts/recommend_next_agent.py generate
  -> deterministic evaluation only

python3 scripts/recommend_next_agent.py promote
  -> canonical packet + final approval artifact
```

### Dependency Graph

```text
xtask main/lib
  -> recommend_next_agent_research entrypoint
      -> recommendation-specific models/render/validation helpers
      -> shared runtime-follow-on-style codex exec pattern
      -> scripts/recommend_next_agent.py freeze-discovery
      -> docs/specs/cli-agent-recommendation-dossier-contract.md
      -> discovery hints + agent registry

generate/promote remain downstream consumers only
```

### Packet Files

The execution packet root MUST contain:

- `input-contract.json`
- `discovery-prompt.md`
- `research-prompt.md`
- `codex-execution.discovery.json`
- `codex-execution.research.json`
- `codex-stdout.discovery.log`
- `codex-stderr.discovery.log`
- `codex-stdout.research.log`
- `codex-stderr.research.log`
- `written-paths.discovery.json`
- `written-paths.research.json`
- `validation-report.json`
- `run-status.json`
- `run-summary.md`

### Prompt Ownership

The prompt body is repo-owned and versioned in `xtask`, not in the skill.

The discovery prompt MUST include:

- run id
- pass number
- fixed query family
- discovery hints path or `none`
- currently onboarded agent ids
- exact allowed output root
- exact required discovery artifact set
- explicit prohibition on writing outside the discovery root

The research prompt MUST include:

- frozen `seed.snapshot.toml` path
- dossier contract path
- exact allowed output root
- exact required research artifact set
- explicit note that `probe_requests` are structured metadata, not shell instructions

### Validation Model

The repo validates four boundaries:

1. dry-run contract completeness before any Codex invocation
2. discovery output completeness and bounded writes after discovery execution
3. freeze handoff success and reviewed-seed identity before research execution
4. research output completeness and dossier/seed identity before downstream `generate`

The command fails closed when:

- Codex writes outside the allowed roots
- required discovery or research files are missing
- discovery or research JSON/TOML/Markdown validation fails
- `freeze-discovery` fails
- dossier filenames, `agent_id`, or `seed_snapshot_sha256` do not match the frozen seed

### Pass-2 Widening Rules

Pass 2 is part of the product, not a follow-up.

Rules:

- pass 1 uses the existing fixed query family
- pass 2 requires a prior insufficiency run directory
- pass 2 excludes every pass-1 candidate already seen, accepted, or rejected
- if pass 1 had zero survivors after hard rejection, pass 2 omits the candidate-relative query
- pass 2 emits at most 3 new candidates
- pass 2 always uses a fresh `run_id`
- pass 2 never mutates pass-1 discovery, research, or evaluation artifacts

## File-Level Change Plan

### Required new or updated Rust surfaces

- `crates/xtask/src/main.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/recommend_next_agent_research.rs`
- optional helper submodule directory only if the entrypoint becomes unwieldy:
  - `crates/xtask/src/recommend_next_agent_research/codex_exec.rs`
  - `crates/xtask/src/recommend_next_agent_research/models.rs`
  - `crates/xtask/src/recommend_next_agent_research/render.rs`
  - `crates/xtask/src/recommend_next_agent_research/validation.rs`

Implementation rule: start with a single entrypoint file. Extract helpers only when the same code
is used more than once or the file becomes structurally unclear. Explicit over clever.

### Required docs and operator surfaces

- `docs/specs/cli-agent-recommendation-dossier-contract.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `.codex/skills/recommend-next-agent/SKILL.md`

### Expected test surfaces

- `crates/xtask/tests/recommend_next_agent_research_entrypoint.rs`
- `crates/xtask/tests/support/recommend_next_agent_research_harness.rs`
- `scripts/test_recommend_next_agent.py` only if the frozen contract needs Python-side adjustment

## Implementation Workstreams

### Workstream 1: Contract And CLI Surface

Deliverables:

- add the new xtask subcommand to `main.rs` and `lib.rs`
- define `Args`, `Pass`, run status, and validation report models
- extend the normative spec to describe:
  - the repo-owned host command
  - the execution packet root
  - dry-run and write semantics
  - pass-2 runner inputs

Exit condition:

- the command parses, dry-run can render a packet, and the docs/spec already describe the same
  surface the code is about to implement

### Workstream 2: Dry-Run Packet Rendering

Deliverables:

- input-contract renderer
- discovery prompt renderer
- research prompt renderer
- run-summary and run-status renderers for dry-run mode

Exit condition:

- `--dry-run` writes a complete packet without invoking Codex and without mutating discovery or
  research roots

### Workstream 3: Write-Mode Discovery And Freeze Handoff

Deliverables:

- bounded discovery Codex execution
- discovery artifact validation
- repo-owned `freeze-discovery` subprocess handoff
- discovery execution evidence capture

Exit condition:

- `--write` fails immediately if discovery writes outside the allowed root or if freeze cannot
  produce a valid `seed.snapshot.toml`

### Workstream 4: Write-Mode Research Validation

Deliverables:

- bounded dossier-authoring Codex execution
- research artifact validation
- seed/dossier identity checks
- final packet status and validation summary

Exit condition:

- the run packet proves exactly what Codex wrote and the repo accepts only a contract-valid
  research tree

### Workstream 5: Skill And Operator Rewrite

Deliverables:

- skill rewritten as a thin wrapper over:
  1. `xtask recommend-next-agent-research --dry-run`
  2. `xtask recommend-next-agent-research --write`
  3. `python3 scripts/recommend_next_agent.py generate`
  4. optional repo-owned pass-2 rerun
  5. `python3 scripts/recommend_next_agent.py promote`
- operator guide updated to the same exact procedure

Exit condition:

- there is no remaining documented path that tells maintainers to run discovery or dossier
  authoring freehand outside the repo-owned host command

### Workstream 6: Verification And Proving Run

Deliverables:

- xtask entrypoint tests
- any required narrow Python regression tests
- one real recommendation proving run using the new host surface end to end

Exit condition:

- one promoted run exists whose discovery and research inputs were created through the new runner,
  not by ad hoc skill execution

## Architecture Review

### System Design

This is a well-bounded control-plane addition. The new runner sits strictly upstream of
`generate` and `promote`. That is the correct boundary. It avoids reopening the deterministic
post-research engine and keeps the new logic focused on orchestration, validation, and evidence.

### Coupling And Reuse

The new xtask entrypoint should mirror `runtime_follow_on`, not fork it conceptually. Reuse the
same style of:

- explicit `--dry-run` then `--write`
- pre-rendered prompt packet
- Codex execution evidence capture
- bounded write validation
- machine-readable run status

Do not extract a global "generic AI runner" in this milestone. That spends an innovation token on
abstraction instead of the product gap. If both runners share a small, identical utility, extract
that utility only.

### Data Flow Diagram

```text
contract inputs
  -> xtask dry-run packet
  -> Codex discovery write
  -> repo validation
  -> freeze-discovery
  -> Codex dossier write
  -> repo validation
  -> deterministic generate
  -> promote
  -> approved-agent.toml
```

### Security And Blast Radius

- Codex write scope is limited to scratch roots. That keeps blast radius out of repo-tracked
  canonical surfaces.
- `generate` and `promote` remain the only writers of repo-tracked review and approval artifacts.
- A bad research run should fail closed in scratch space, not leak into packet promotion.

### Production Failure Scenario

Realistic failure: Codex writes a seemingly valid dossier tree for the wrong seed after pass-2
widening. If the repo does not validate `seed_snapshot_sha256` and candidate identity, the user
gets a plausible but invalid recommendation packet. This plan prevents that by making identity
checks mandatory before `generate` can run.

## Code Quality Review

### DRY And Module Boundaries

- Keep recommendation-specific policy in the recommendation runner, not in a runtime-follow-on
  shared module.
- Share only generic mechanics: process execution, path validation, JSON file helpers, and prompt
  packet writing if the interfaces truly line up.
- Do not duplicate contract constants across Rust, skill text, and docs. Rust owns runtime
  behavior. The spec owns normative wording. The skill and operator guide point at both.

### Explicit Over Clever

- Use one explicit `Pass` enum and branch behavior directly on it.
- Persist both discovery and research execution evidence separately. Do not compress them into one
  ambiguous "codex run" blob.
- Write explicit validators for required artifact sets instead of relying on missing-file side
  effects later in `generate`.

### Under-Engineering To Avoid

- skill-only orchestration
- silent fallback to freehand pass-2 widening
- write-mode that can run without a dry-run baseline
- acceptance based on "looks good" files without identity checks

### Over-Engineering To Avoid

- a cross-repo generic AI workflow framework
- speculative live-probe orchestration in xtask
- new packet formats for downstream review or approval

## Test Review

### Planned Coverage Diagram

```text
XTASK ENTRYPOINT COVERAGE
=========================
[+] recommend-next-agent-research --dry-run
    ├── [NEW TEST] rejects pass2 without --prior-run-dir
    ├── [NEW TEST] rejects --write without --run-id
    ├── [NEW TEST] writes complete packet for pass1
    └── [NEW TEST] writes complete packet for pass2 with prior insufficiency input

[+] recommend-next-agent-research --write
    ├── discovery execution
    │   ├── [NEW TEST] accepts exact required discovery artifact set
    │   ├── [NEW TEST] rejects missing discovery artifact
    │   └── [NEW TEST] rejects writes outside discovery root
    ├── freeze handoff
    │   ├── [NEW TEST] invokes freeze-discovery with expected args
    │   └── [NEW TEST] fails closed on freeze-discovery error
    ├── research execution
    │   ├── [NEW TEST] accepts exact required research artifact set
    │   ├── [NEW TEST] rejects missing dossier
    │   ├── [NEW TEST] rejects dossier filename / agent_id mismatch
    │   └── [NEW TEST] rejects seed_snapshot_sha256 mismatch
    └── run evidence
        ├── [NEW TEST] writes stdout/stderr/evidence/status/summary
        └── [NEW TEST] records discovery and research written paths separately

PYTHON RUNNER REGRESSION COVERAGE
=================================
[+] freeze-discovery / generate / promote
    ├── [EXISTING TEST] frozen seed, insufficiency, promote invariants
    ├── [EXISTING TEST] pass1 -> expand_discovery and pass2 -> stop semantics
    ├── [EXISTING TEST] discovery provenance copied into run and review artifacts
    └── [NEW TEST ONLY IF NEEDED] any contract wording or CLI handoff delta introduced by xtask

USER FLOW COVERAGE
==================
[+] Pass 1 success
    ├── [NEW XTASK TEST] dry-run + write create valid research root
    └── [EXISTING PYTHON TEST] generate/promote succeed with valid frozen input

[+] Pass 1 insufficiency -> pass 2 retry
    ├── [NEW XTASK TEST] pass2 requires prior insufficiency run
    ├── [NEW XTASK TEST] pass2 excludes previously seen candidates in rendered prompt
    └── [EXISTING PYTHON TEST] insufficiency next_action semantics stay correct

[+] Failure handling
    ├── [NEW XTASK TEST] Codex missing or non-zero exit fails closed
    ├── [NEW XTASK TEST] malformed discovery JSON/TOML fails closed
    ├── [NEW XTASK TEST] malformed dossier tree fails closed
    └── [MANUAL PROVING RUN] one real run validates end-to-end operator ergonomics
```

### Test Files To Add

1. `crates/xtask/tests/recommend_next_agent_research_entrypoint.rs`
   - happy-path dry-run
   - happy-path write
   - invalid args
   - outside-root writes
   - freeze failure
   - seed/dossier identity failures
2. `crates/xtask/tests/support/recommend_next_agent_research_harness.rs`
   - fixture setup
   - fake Codex outputs
   - fake `freeze-discovery` subprocess support
3. `scripts/test_recommend_next_agent.py`
   - update only if a real Python contract gap appears during implementation

### LLM Prompt Change Rule

This milestone changes prompt templates, but there is no repo-native probabilistic eval harness
for recommendation research quality today. The correct test strategy is:

- deterministic xtask contract tests for prompt packet contents and write validation
- existing Python contract tests for downstream runner behavior
- one real proving run to validate operator reality

Do not pretend a fake "LLM eval suite" exists. Test the contract and run one honest proving flow.

## Performance Review

1. The runner is process- and filesystem-bound, not CPU-bound. The performance risk is accidental
   repeated hashing, repeated directory walks, or loading large logs fully into memory. Stream logs
   to files and keep JSON summaries small.
2. Validation should scale linearly with artifact count. The contract artifact sets are tiny. Keep
   validators explicit and O(n).
3. Pass 2 must reuse prior insufficiency outputs as inputs, not regenerate or re-parse unnecessary
   downstream artifacts.
4. The command must fail quickly on invalid dry-run baseline or invalid write roots. Do not spend
   Codex time before basic repo validation passes.

## Failure Modes

| Codepath | Real failure | Test? | Error handling? | User impact if missed |
| --- | --- | --- | --- | --- |
| dry-run packet render | prompt or contract omits a required root | yes | fail before write mode | Codex writes to the wrong place |
| discovery execution | Codex writes outside discovery root | yes | fail closed with written-path report | scratch pollution and unsafe trust boundary |
| freeze handoff | discovery artifacts look present but freeze rejects them | yes | stop before research | invalid reviewed seed enters research |
| research execution | dossier file set is incomplete | yes | fail closed before `generate` | misleading partial recommendation evidence |
| dossier identity | `agent_id` or snapshot hash mismatches frozen seed | yes | fail closed | wrong candidate scored against wrong evidence |
| pass-2 widening | prior insufficiency context is ignored | yes | reject pass2 invocation | repeated candidate pool and fake widening |
| downstream generate | xtask silently mutates Python runner inputs | covered by existing Python tests plus proving run | keep runner CLI unchanged | deterministic boundary drifts |
| docs/skill drift | operator guide still documents ambient flow | review + doc update | single canonical procedure | maintainers keep using the old path |

Critical gap rule: any path that could silently produce a recommendation packet from invalid seed
identity is a release blocker.

## Worktree Parallelization Strategy

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| 1. Lock command + contract shape | `crates/xtask/src/`, `docs/specs/`, `docs/cli-agent-onboarding-factory-operator-guide.md`, `.codex/skills/` | — |
| 2. Implement dry-run and write entrypoint core | `crates/xtask/src/`, `crates/xtask/tests/support/` | 1 |
| 3. Add xtask entrypoint tests | `crates/xtask/tests/`, `crates/xtask/tests/support/` | 2 |
| 4. Adjust Python runner tests only if contract gaps appear | `scripts/`, `scripts/test_recommend_next_agent.py` | 1 |
| 5. Rewrite operator docs and skill to thin-wrapper flow | `docs/`, `.codex/skills/` | 1 |
| 6. Run proving flow and promote one reviewed run | scratch roots under `docs/agents/.uaa-temp/`, committed `docs/agents/selection/`, lifecycle governance path | 2, 3, 4, 5 |

### Parallel Lanes

- Lane A: `Step 2 -> Step 3`
  - sequential, shared `crates/xtask/`
- Lane B: `Step 5`
  - independent once Step 1 is locked
- Lane C: `Step 4`
  - independent only if a genuine Python contract delta is discovered
- Lane D: `Step 6`
  - final sequential lane after A, B, and C finish

### Execution Order

Launch Step 2 and Step 5 in parallel after Step 1 is locked.

If Python changes are required, launch Step 4 in parallel with late Step 2 or early Step 3.

After A, B, and optional C finish, run Step 6 alone. The proving flow depends on the actual
command, tests, and docs all being aligned.

### Conflict Flags

- Steps 2 and 3 both touch `crates/xtask/`. Keep them in one lane.
- Step 1 and Step 5 both touch docs surfaces. Step 5 must treat Step 1 as the source of truth.
- Step 6 touches repo-tracked recommendation outputs. Do not run it in parallel with doc or runner
  changes.

## NOT In Scope

- changing the deterministic scoring rubric or shortlist dimensions
- redesigning the canonical selection packet template
- changing `generate` or `promote` public CLI shape
- adding live candidate execution or sandboxed probes to xtask research hosting
- adding more than one widening retry
- building a generic cross-repo AI orchestration framework
- deciding whether capability-matrix markdown remains canonical after M5

## Acceptance Gates

1. `cargo run -p xtask -- recommend-next-agent-research --dry-run --pass pass1`
   writes a complete execution packet with no Codex invocation.
2. `cargo run -p xtask -- recommend-next-agent-research --write --pass pass1 --run-id <id>`
   executes bounded discovery and research, then fails closed on any contract violation.
3. The command, not the skill, runs `freeze-discovery`.
4. Pass 2 widening is supported directly in the new runner and requires prior insufficiency input.
5. `python3 scripts/recommend_next_agent.py generate --research-dir ... --run-id ... --scratch-root ...`
   works unchanged against the resulting research tree.
6. `python3 scripts/recommend_next_agent.py promote ...` works unchanged and still renders the
   canonical packet and final approval artifact.
7. The skill and operator guide both describe the same repo-owned procedure.
8. One proving run created through the new host surface produces:
   - a promoted review run
   - the canonical packet
   - `docs/agents/lifecycle/<pack>/governance/approved-agent.toml`

## Verification Plan

### Automated

- `cargo test -p xtask --test recommend_next_agent_research_entrypoint`
- `python3 -m unittest scripts/test_recommend_next_agent.py`
- `cargo test -p xtask --test recommend_next_agent_approval_artifact`
- `make check`
- `make test`
- `make preflight`

### Manual proving flow

1. `cargo run -p xtask -- recommend-next-agent-research --dry-run --pass pass1`
2. `cargo run -p xtask -- recommend-next-agent-research --write --pass pass1 --run-id <id>`
3. `python3 scripts/recommend_next_agent.py generate --research-dir ... --run-id <id> --scratch-root docs/agents/.uaa-temp/recommend-next-agent/runs`
4. if insufficient, rerun pass 2 through the same xtask surface with a fresh `run_id`
5. `python3 scripts/recommend_next_agent.py promote --run-dir ... --repo-run-root docs/agents/selection/runs --approved-agent-id <agent_id> --onboarding-pack-prefix <prefix>`
6. `cargo run -p xtask -- onboard-agent --approval docs/agents/lifecycle/<prefix>/governance/approved-agent.toml --dry-run`

## Success Metric

The recommendation lane is improved only if the repo owns the AI research step, not just the
packet after it.

Success looks like:

- maintainers can rerun recommendation research from one explicit repo command
- every promoted recommendation run has a durable execution packet proving prompt, writes, and
  validation outcome
- the deterministic runner stays boring
- later onboarding surprises are about candidate quality, not about how recommendation evidence
  was gathered
