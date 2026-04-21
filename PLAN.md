<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/feat-cli-agent-onboarding-factory-autoplan-restore-20260421-105543.md -->
# CLI Agent Onboarding Factory - PLAN

Source:
- `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-main-design-20260420-151505.md`
- `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`
- `docs/project_management/next/gemini-cli-onboarding/HANDOFF.md`

Status: M1 and M2 landed on `feat/cli-agent-onboarding-factory`; M3 ready for implementation planning
Last updated (UTC): 2026-04-21

## Purpose
M3 turns the post-M2 selection gap from folklore into governed repo truth.

M2 proved the control-plane mutation slice is real:
- `xtask onboard-agent --write` exists
- control-plane writes are transactional and replay-safe
- one real proving run landed for `gemini_cli`
- the proving run closed with committed packet artifacts and `make preflight`

That changed the next bottleneck.

The problem is no longer "can the repo write the control-plane packet?" The problem is that the repo can now point to three different truths without saying how they relate:
- a comparison packet that recommended `OpenCode`
- a proving run that landed `gemini_cli`
- a top-level plan that still talks like M2 is pending

If M3 only formalizes recommendation tooling, it will automate the least trustworthy part of the funnel. The next milestone has to formalize the selection-to-proof chain:
- what was compared
- what was recommended
- what was approved
- why approval diverged when it did
- what the proving run taught us

## Landed Baseline
These are already in the branch and are no longer plan items:

- `crates/xtask/data/agent_registry.toml` seeds `codex`, `claude_code`, `opencode`, and `gemini_cli`.
- `crates/xtask/src/onboard_agent.rs` implements `--dry-run` and `--write`.
- `crates/xtask/src/onboard_agent/mutation.rs` enforces path jailing, staged writes, and rollback.
- `crates/xtask/src/onboard_agent/validation.rs` enforces registry/package/filesystem fail-closed behavior for raw descriptor input.
- `docs/project_management/next/gemini-cli-onboarding/**` is the first closed proving-run packet.
- `docs/project_management/next/gemini-cli-onboarding/governance/proving-run-metrics.json` records M2 closeout metrics, but not enough provenance for M3.
- `crates/xtask/tests/onboard_agent_entrypoint.rs` and `crates/xtask/tests/onboard_agent_closeout_preview.rs` prove M2 packet generation and closeout rendering.

M3 must build on this exact repo state. It is not a partial M2 cleanup.

## Premise Challenge
| Premise | Verdict | Why |
|---|---|---|
| The next bottleneck is recommendation generation itself. | Reject | The sharper gap is decision provenance between comparison, approval, and proving run. |
| Recommendation and approval can be treated as the same artifact. | Reject | `OpenCode` was recommended, `gemini_cli` was landed. The repo needs room for explicit override truth. |
| Approval state belongs in `agent_registry.toml` or `cli_manifests/**`. | Reject | Registry metadata and manifest evidence are downstream truths. Approval is governance input, not runtime or publication truth. |
| `onboard-agent` should stay long-flag driven for real operator use. | Reject | Re-entering the full descriptor by hand after approval keeps the most error-prone seam manual. |
| M3 should add `recommend-agent` before it fixes approval and closeout governance. | Reject | That would formalize ceremony before the repo can explain why recommendation and reality diverged. |
| M3 should formalize a selection-to-proof chain, with explicit approval and closeout artifacts. | Accept | This is the smallest complete slice that makes agent five boring instead of archaeological. |
| The selection rubric needs an explicit mode for why we are choosing an agent now. | Accept | The repo mixed "factory validation" and "frontier expansion" logic in one packet and then shipped a different agent. |
| M3 should record whether `gemini_cli` superseded `OpenCode` or was only a proving-run vehicle. | Accept | The answer is now explicit: `gemini_cli` was the fourth-agent proving-run vehicle, while `OpenCode` remains the next recommendation lineage. |

## Scope Lock
- Rebaseline the top-level plan around post-M2 repo truth.
- Formalize three artifacts in the selection-to-proof chain:
  - candidate comparison packet
  - approval artifact
  - proving-run closeout artifact
- Keep approval and closeout artifacts under `docs/project_management/next/<prefix>/governance/`.
- Make `xtask onboard-agent` consume an approval artifact as the canonical operator path.
- Keep raw long-flag input only as a fixture/backfill path, not the preferred human workflow.
- Add one explicit closeout validation path so packet state is not inferred from metrics-file presence alone.
- Backfill one authoritative decision record explaining that `gemini_cli` was the fourth-agent proving-run vehicle while `OpenCode` remains the next recommendation lineage.
- Keep recommendation authoring human-in-the-loop in M3. The repo does not need automated candidate research to solve the current trust gap.

## Success Criteria
M3 is complete only when all of these are true:

- `PLAN.md` is post-M2 accurate and no longer frames M2 as pending.
- A committed approval artifact exists and records:
  - `comparison_ref`
  - `selection_mode`
  - `recommended_agent_id`
  - `approved_agent_id`
  - `override_reason` when those differ
  - the exact onboarding descriptor consumed by the factory
- `cargo run -p xtask -- onboard-agent --approval <path> --dry-run` exists.
- `cargo run -p xtask -- onboard-agent --approval <path> --write` exists.
- Artifact mode is exclusive with semantic descriptor flags. No dual authority.
- Generated packet outputs stamp the approval artifact path plus an immutable approval hash or id.
- Proving-run closeout can no longer be inferred from metrics-file presence alone.
- A validated closeout artifact exists and records:
  - approval source
  - exactly one of `duration_seconds` or `duration_missing_reason`
  - either explicit residual friction items or `explicit_none_reason`
  - proving-run state
- The repo records one authoritative answer to this question:
  - `gemini_cli` was the fourth-agent proving-run vehicle
  - `OpenCode` remains the next recommendation lineage and was not replaced by Gemini
- The historical `gemini_cli` proving run is backfilled or superseded so the closed packet has trustworthy provenance.
- A full M3 test plan exists at `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-feat-cli-agent-onboarding-factory-test-plan-20260421-105654.md`.

## What Already Exists
M3 must reuse these surfaces instead of inventing a second factory:

- Comparison inputs:
  - `docs/project_management/next/_templates/cli-agent-onboarding-packet-template.md`
  - `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`
- Control-plane entrypoints:
  - `crates/xtask/src/main.rs`
  - `crates/xtask/src/onboard_agent.rs`
  - `crates/xtask/src/onboard_agent/preview.rs`
  - `crates/xtask/src/onboard_agent/preview/render.rs`
  - `crates/xtask/src/onboard_agent/validation.rs`
- Truth-owning downstream artifacts:
  - `crates/xtask/data/agent_registry.toml`
  - `cli_manifests/<agent>/**`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- Historical proving-run evidence:
  - `docs/project_management/next/gemini-cli-onboarding/**`
- Existing test posture:
  - `crates/xtask/tests/onboard_agent_entrypoint.rs`
  - `crates/xtask/tests/onboard_agent_closeout_preview.rs`
  - `crates/xtask/tests/agent_registry.rs`
  - `crates/xtask/tests/c8_spec_capability_matrix_*.rs`

## Not In Scope
- Auto-generating the 3-candidate comparison from live web research.
- Reworking support-matrix or capability-matrix semantics.
- Moving approval truth into the registry or manifest roots.
- Generating wrapper crate or backend source files.
- Runtime-lane compression for wrapper/backend implementation.
- Update mode for already-onboarded agents.
- Universal agent lifecycle orchestration across recommendation, approval, onboarding, and maintenance in one monolithic command.

## Dream State Delta
```text
CURRENT STATE
comparison packet
    |
    +--> recommends OpenCode
    |
    +--> proving run lands Gemini CLI
    |
    +--> closeout says "no residual friction recorded"
    |
    +--> next operator has to infer what actually happened

M3
comparison packet
    |
    +--> approval artifact
    |       - recommended agent
    |       - approved agent
    |       - override reason when needed
    |       - exact onboarding descriptor
    |
    +--> onboard-agent --approval
    |
    +--> runtime lane
    |
    +--> validated closeout artifact
            - approval source
            - duration or explicit missing reason
            - residual friction or explicit none reason

12-MONTH IDEAL
comparison mode chosen explicitly
    |
    +--> boring approval artifact
    +--> boring onboard-agent invocation
    +--> boring closeout and feedback loop
    +--> next recommendation learns from the last proving run
```

## Implementation Alternatives
### Approach A: Recommendation Tooling First
Summary: build `recommend-agent` or a packet generator now, then retrofit approval and closeout provenance later.

Effort: M  
Risk: High

Pros:
- visible automation win
- reduces packet-authoring repetition

Cons:
- formalizes the least trustworthy stage first
- does not explain `OpenCode` versus `gemini_cli`
- creates recommendation theater if humans still override off-record

### Approach B: Selection-To-Proof Governance First
Summary: keep comparison authoring HITL, add immutable approval and closeout artifacts, make `onboard-agent` consume approved input, then revisit recommendation tooling with actual governance truth in place.

Effort: M  
Risk: Medium

Pros:
- fixes the observed trust gap directly
- removes descriptor re-entry from the operator path
- preserves the registry/manifest/runtime ownership boundary M2 established
- creates the evidence chain future recommendation automation can safely target

Cons:
- does not automate candidate research yet
- requires one historical backfill to explain the Gemini proving run without treating it as an OpenCode replacement
- adds lifecycle validation that current packet rendering does not yet enforce

### Approach C: Full Lifecycle CLI Suite Now
Summary: add `recommend-agent`, `approve-agent`, `onboard-agent`, and `close-proving-run` in one milestone.

Effort: XL  
Risk: High

Pros:
- cleanest long-term story
- one end-to-end command family

Cons:
- turns a bounded governance fix into a platform rewrite
- expands blast radius before the repo agrees on the post-M2 truth
- risks rebuilding the same ambiguity with more code

**Recommendation:** Choose Approach B. It is the smallest complete milestone that turns the current repo narrative into something trustworthy.

## Mode Selection
Auto-decided mode: `SELECTIVE EXPANSION`.

Reasoning:
- the repo already landed the M2 mutation slice and one real proving run
- the missing gap is governance and provenance, not a bigger platform rewrite
- adding approval and closeout artifacts is a complete lake
- full lifecycle recommendation automation is still ocean-boiling before the trust chain is fixed

Accepted expansion:
- add one explicit selection mode field to approval artifacts:
  - `factory_validation`
  - `frontier_expansion`

Deferred expansions:
- `recommend-agent` automation
- runtime-lane compression
- multi-agent update/drift maintenance flows

## M3 Plan Of Record
### Goal
Turn "we compared one thing, approved another, and closed a proving run without saying why" into a deterministic, auditable selection-to-proof chain.

### Milestone Outcome
At the end of M3:

- maintainers author or commit one comparison packet for a candidate set
- maintainers freeze one approval artifact that contains the exact onboarding descriptor and the approval rationale
- `xtask onboard-agent` consumes that approved input directly
- proving-run closeout cannot claim success without approval linkage, timing truth, and residual-friction truth
- the historical `OpenCode` recommendation and `gemini_cli` fourth-agent proving run are recorded once, explicitly, in repo truth

### Governance Chain
```text
comparison packet (informative)
        |
        v
approval artifact (machine-readable, immutable)
        |
        v
onboard-agent --approval
        |
        v
runtime-owned implementation lane
        |
        v
proving-run closeout artifact (machine-readable, validated)
        |
        v
closed packet + feedback into next selection
```

### Artifact Contract
#### 1. Comparison packet
Format: Markdown  
Owner: maintainer workflow  
Role: explain candidate set, rubric, recommendation, and evidence

Notes:
- remains informative, not normative
- may recommend one agent without locking approval to that winner

#### 2. Approval artifact
Path: `docs/project_management/next/<prefix>/governance/approved-agent.toml`  
Format: TOML  
Owner: governance input

Required fields:
- `artifact_version`
- `comparison_ref`
- `selection_mode`
- `recommended_agent_id`
- `approved_agent_id`
- `approval_commit`
- `approval_recorded_at`
- `override_reason` when `approved_agent_id != recommended_agent_id`
- embedded onboarding descriptor fields currently passed as long flags to `onboard-agent`

Rules:
- immutable once used for a write or proving run
- repo-relative path, jailed, and namespace-validated
- must not live in `agent_registry.toml` or `cli_manifests/**`

#### 3. Proving-run closeout artifact
Path: `docs/project_management/next/<prefix>/governance/proving-run-closeout.json`  
Format: JSON  
Owner: governance closeout

Required fields:
- `state = "closed"`
- `approval_ref`
- `approval_sha256` or equivalent immutable approval id
- `approval_source`
- `manual_control_plane_edits`
- `partial_write_incidents`
- `ambiguous_ownership_incidents`
- exactly one of:
  - `duration_seconds`
  - `duration_missing_reason`
- exactly one of:
  - `residual_friction`
  - `explicit_none_reason`
- `preflight_passed`
- `recorded_at`
- `commit`

Rules:
- metrics-file presence alone must not close a packet
- closeout must fail validation if approval linkage, timing truth, or residual-friction truth is missing

### Command Contract
M3 keeps the existing raw descriptor path for fixtures and backfills, but the canonical operator path becomes approval-driven:

```bash
cargo run -p xtask -- onboard-agent --approval docs/project_management/next/<prefix>/governance/approved-agent.toml --dry-run
cargo run -p xtask -- onboard-agent --approval docs/project_management/next/<prefix>/governance/approved-agent.toml --write
cargo run -p xtask -- close-proving-run --approval docs/project_management/next/<prefix>/governance/approved-agent.toml --closeout docs/project_management/next/<prefix>/governance/proving-run-closeout.json
```

Rules:
- `--approval` is mutually exclusive with semantic descriptor flags.
- Artifact mode must print the resolved approval artifact id/hash in stdout.
- Generated packet artifacts must stamp the approval reference and approval hash.
- `close-proving-run` validates the closeout artifact and refreshes packet closeout rendering.

### Architecture
```text
comparison.md
    |
    +--> approved-agent.toml
            |
            +--> onboard-agent --approval
                    |
                    +--> registry append / replay check
                    +--> docs packet materialization
                    +--> manifest-root skeleton materialization
                    +--> Cargo.toml workspace insertion
                    +--> docs/crates-io-release.md generated block refresh
                    |
                    v
              runtime-owned lane
                    |
                    v
           proving-run-closeout.json
                    |
                    +--> close-proving-run validation
                    +--> closed packet rendering
                    +--> feedback into next selection cycle
```

## Workstreams
### W1. Post-M2 Rebaseline
Goal: make the repo say what is actually true before adding new lifecycle logic.

Deliverables:
- rewrite `PLAN.md` around landed M2 plus M3 scope
- record explicitly that `gemini_cli` was the fourth-agent proving-run vehicle and did not replace `OpenCode`
- define the M3 governance state machine and artifact inventory

Exit criteria:
- no top-level planning doc still frames M2 as pending
- historical recommendation versus proving-run divergence has one explicit owner truth

### W2. Approval Artifact + `onboard-agent --approval`
Goal: remove descriptor re-entry from the operator path without blurring truth boundaries.

Deliverables:
- `approved-agent.toml` schema and validation
- `--approval` input mode for `onboard-agent`
- hash/id stamping in generated packet outputs
- namespace and provenance validation for approval artifacts

Primary touchpoints:
- `crates/xtask/src/main.rs`
- `crates/xtask/src/onboard_agent.rs`
- `crates/xtask/src/onboard_agent/validation.rs`

Exit criteria:
- artifact mode and raw semantic flags cannot be mixed
- approval artifacts are repo-jail-safe, immutable, and auditable
- dry-run and write parity still holds under approval mode

### W3. Closeout Contract + Packet State Hardening
Goal: stop inferring packet closure from loose metrics blobs.

Deliverables:
- validated closeout schema
- `close-proving-run` entrypoint
- rendering changes that require approval linkage plus complete closeout truth
- explicit execution-mode versus closeout-mode packet transitions

Primary touchpoints:
- `crates/xtask/src/main.rs`
- `crates/xtask/src/onboard_agent/preview.rs`
- `crates/xtask/src/onboard_agent/preview/render.rs`

Exit criteria:
- partial or stale closeout artifacts cannot render a closed packet
- packet closeout requires approval linkage, duration truth, and residual-friction truth

### W4. Historical Reconciliation
Goal: clean up the first real mismatch before the repo repeats it.

Deliverables:
- backfilled approval artifact for the `gemini_cli` proving run
- explicit recorded relationship to `cli-agent-onboarding-third-agent-packet.md`
- migrated or superseding closeout artifact with M3-required fields

Exit criteria:
- the historical `OpenCode` recommendation and `gemini_cli` fourth-agent proving run are legible in committed repo truth
- future maintainers do not have to infer why Gemini landed as the fourth agent without replacing OpenCode as the next recommended lineage

## Execution Sequence
### Phase 1. Rebaseline + Governance Schema
Outputs:
- post-M2 `PLAN.md`
- artifact definitions
- historical-decision resolution

Exit gate:
- no unresolved ambiguity about what M3 is trying to solve

### Phase 2. Approval-Driven Onboarding
Outputs:
- `--approval` mode
- approval artifact validation
- approval hash/id propagation through packet outputs

Exit gate:
- approval-driven dry-run and write produce the same mutations as the raw descriptor path

### Phase 3. Closeout Validation
Outputs:
- `close-proving-run`
- validated closeout schema
- packet state-machine hardening

Exit gate:
- metrics-file presence alone can no longer close a packet

### Phase 4. Historical Backfill
Outputs:
- reconciled Gemini approval chain
- migrated closeout truth
- updated operator docs or handoff references

Exit gate:
- the repo can explain the first real proving run without conversation archaeology

## Error & Rescue Registry
| Method / Codepath | What can go wrong | Exception / failure class | Rescued? | Rescue action | User sees |
|---|---|---|---|---|---|
| approval artifact parse | malformed TOML or missing required fields | validation error | yes | reject before planning writes | exit `2` |
| approval artifact path | artifact path escapes repo or points outside governance surfaces | ownership violation | yes | reject before artifact load | exit `2` |
| approval override validation | `recommended_agent_id != approved_agent_id` without `override_reason` | validation error | yes | reject before dry-run/write | exit `2` |
| approval replay check | approval descriptor differs from already-onboarded identical agent state | conflict | yes | reject with approval id/hash context | exit `2` |
| packet rendering | closeout artifact exists but approval linkage or state is incomplete | state transition error | yes | keep packet in execution mode | explicit validation failure |
| closeout validation | duration and residual-friction truth missing | validation error | yes | reject closeout until explicit truth exists | exit `2` |
| historical backfill | Gemini approval chain cannot be explained from available evidence | needs-context | no | record explicit open decision for maintainer resolution | blocked docs update |

## Test Diagram
```text
NEW GOVERNANCE FLOWS
====================
[+] comparison packet -> approved-agent.toml
    |
    ├── [GAP -> validation] approved differs from recommended without override reason fails
    ├── [GAP -> validation] selection_mode must be explicit
    └── [GAP -> validation] approval artifact path must stay inside repo/governance roots

[+] approved-agent.toml -> onboard-agent --approval
    |
    ├── [GAP -> integration] approval mode matches raw long-flag mutation plan exactly
    ├── [GAP -> integration] semantic flags are rejected when approval mode is used
    ├── [GAP -> integration] approval hash/id appears in stdout and generated packet files
    └── [GAP -> regression] replay-safe identical rerun still succeeds as a no-op

[+] runtime lane -> proving-run-closeout.json -> close-proving-run
    |
    ├── [GAP -> validation] closeout without approval linkage fails
    ├── [GAP -> validation] closeout without duration truth fails
    ├── [GAP -> validation] closeout without residual-friction truth fails
    ├── [GAP -> integration] stale metrics file alone does not close packet
    └── [GAP -> integration] packet stays in execution mode until validated closeout exists

HISTORICAL RECONCILIATION
=========================
[+] OpenCode recommendation -> Gemini proving run
    |
    ├── [GAP -> docs/validation] one authoritative override record exists
    └── [GAP -> regression] future maintainers can trace comparison -> approval -> closeout without conversation history
```

## Required Test Surfaces
- extend `crates/xtask/tests/onboard_agent_entrypoint.rs` for:
  - `--approval` happy path
  - `--approval` parity with the raw long-flag path
  - rejection when semantic flags are mixed with `--approval`
  - rejection when `approved_agent_id != recommended_agent_id` and `override_reason` is missing
  - approval artifact path provenance rejection
- extend `crates/xtask/tests/onboard_agent_closeout_preview.rs` for:
  - closeout rejected without approval linkage
  - closeout rejected without duration truth
  - closeout rejected without residual-friction truth
  - execution-mode packet preserved when only a loose metrics blob exists
- add lifecycle tests for create-only packet semantics versus immutable approval/closeout artifacts
- add a historical backfill validation test for the Gemini approval chain

Test plan artifact:
- `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-feat-cli-agent-onboarding-factory-test-plan-20260421-105654.md`

## Commands
- `cargo run -p xtask -- onboard-agent --approval docs/project_management/next/<prefix>/governance/approved-agent.toml --dry-run`
- `cargo run -p xtask -- onboard-agent --approval docs/project_management/next/<prefix>/governance/approved-agent.toml --write`
- `cargo run -p xtask -- close-proving-run --approval docs/project_management/next/<prefix>/governance/approved-agent.toml --closeout docs/project_management/next/<prefix>/governance/proving-run-closeout.json`
- `cargo test -p xtask`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix`
- `make preflight`

## Failure Modes Registry
| Codepath | Failure mode | Rescued? | Test? | User sees | Logged? |
|---|---|---|---|---|---|
| approval identity | packet claims an approved agent without immutable approval linkage | must be | add validation + integration tests | explicit validation failure | yes |
| dual authority | semantic flags drift from approval artifact input | must be | add entrypoint regression | explicit usage error | yes |
| governance drift | closeout artifact references the wrong approval or stale approval contents | must be | add hash-link validation test | closeout rejected | yes |
| closure theater | loose metrics file closes packet without timing or residual-friction truth | must be | add closeout regression | packet remains execution-mode | yes |
| historical ambiguity | recommendation and proving run diverge without override truth | must be | add backfill validation | blocked reconciliation | yes |

Critical gap rule:
- If any failure mode above lacks both validation and tests, M3 is not ready.

## Security Review
- Approval artifact input is a new trust boundary. It must be path-jailed, schema-validated, and provenance-validated before the repo trusts any descriptor field.
- Approval artifacts must stay out of `agent_registry.toml` and `cli_manifests/**` so governance truth does not masquerade as runtime or publication truth.
- Artifact mode must be exclusive with semantic descriptor flags. Mixed-authority invocation is a correctness bug.
- Closeout artifacts must not be treated as proof of success unless linked to a specific approval artifact and validated as complete.

## Performance Review
- Approval ingestion should parse once and feed the existing mutation-plan path. No duplicate rendering path.
- Approval hash stamping should be computed once per invocation, not per generated file.
- Closeout validation should refresh packet rendering deterministically without re-scanning unrelated repo surfaces.

## What The Implementer Needs To Know
### Hour 1
- M3 is a post-M2 governance milestone, not another mutation milestone.
- The key repo lie to remove is the disconnected `OpenCode` recommendation versus `gemini_cli` proving run.

### Hour 2-3
- Approval artifacts are governance input, not downstream truth.
- `onboard-agent --approval` must reuse the exact same mutation-plan core as raw descriptor input.

### Hour 4-5
- Packet closure cannot be inferred from metrics-file existence anymore.
- Historical backfill is part of the milestone, not optional cleanup.

### Hour 6+
- Keep recommendation authoring HITL for now.
- Do not widen scope into automated candidate research or runtime-lane compression.

## Parallelization Strategy
| Lane | Scope | Modules touched | Start gate | Must hand off |
|---|---|---|---|---|
| A. governance artifacts | approval schema, closeout schema, top-level rebaseline | `PLAN.md`, governance docs, `main.rs` | — | stable artifact names and invariants |
| B. approval-driven onboarding | `--approval` parsing, validation, hash propagation | `onboard_agent.rs`, `validation.rs`, tests | A | canonical approval ingestion contract |
| C. closeout hardening | `close-proving-run`, preview/render validation, closeout tests | `preview.rs`, `preview/render.rs`, tests | A | validated packet state transitions |
| D. historical reconciliation | Gemini backfill artifacts and docs updates | `docs/project_management/next/gemini-cli-onboarding/**` | A, B, C | authoritative OpenCode/Gemini relationship |

Recommended execution:
1. Land Lane A first.
2. Run Lanes B and C in parallel once artifact names and invariants are stable.
3. Land Lane D only after B and C define the final artifact contract.

## Deferred To TODOS.md
- Add `recommend-agent` automation or a deterministic packet generator after two governance-backed comparison cycles prove repetition is real.
- Compress the runtime-owned wrapper/backend lane after the governance chain exposes which runtime steps still dominate lead time.
- Add multi-agent drift/update maintenance flows after the repo finishes one post-M3 onboarding cycle with explicit approval and closeout linkage.

## CEO Dual Voices
### CODEX SAYS (CEO — strategy challenge)
- `PLAN.md` is stale and still optimizes a solved M2 problem.
- decision governance is broken because `OpenCode` was recommended and `gemini_cli` was landed
- the shortlist rubric optimized for architectural novelty instead of the real selection logic
- M2 closeout learned too little because timing and friction truth were not captured

### CLAUDE SUBAGENT (CEO — strategic independence)
- M3 should formalize the selection-to-proof contract, not recommendation theater
- approval, override, proving-run outcome, and feedback need one chain
- the repo needs explicit selection mode because "factory validation" and "frontier expansion" are different decisions

### CEO DUAL VOICES — CONSENSUS TABLE
| Dimension | Claude | Codex | Consensus |
|---|---|---|---|
| Premises valid? | concern | concern | CONFIRMED |
| Right problem to solve? | selection-to-proof | post-M2 governance | CONFIRMED |
| Scope calibration correct? | current note is too small | current note is stale | CONFIRMED |
| Alternatives sufficiently explored? | no | no | CONFIRMED |
| Competitive/market risks covered? | weakly | weakly | CONFIRMED |
| 6-month trajectory sound? | not under current framing | not under current framing | CONFIRMED |

## Design Review
Skipped, no UI scope.

## Eng Dual Voices
### CODEX SAYS (eng — architecture challenge)
- the current lifecycle has no trustworthy identity boundary
- create-only packet files cannot represent later lifecycle transitions safely
- closeout validation must be separate from onboarding mutation
- artifact mode beside semantic flags creates dual authority

### CLAUDE SUBAGENT (eng — independent review)
- packet closure currently depends on metrics-file presence, which is too weak
- approval artifacts belong under governance, not registry or manifests
- descriptor-file input adds a new trust boundary that current validators do not cover
- immutable approval records are required for replay-safe lifecycle history

### ENG DUAL VOICES — CONSENSUS TABLE
| Dimension | Claude | Codex | Consensus |
|---|---|---|---|
| Architecture sound? | concern | concern | CONFIRMED |
| Test coverage sufficient? | concern | concern | CONFIRMED |
| Performance risks addressed? | mild concern | mild concern | CONFIRMED |
| Security threats covered? | concern | concern | CONFIRMED |
| Error paths handled? | concern | concern | CONFIRMED |
| Deployment risk manageable? | concern | concern | CONFIRMED |

## Completion Summary
- Step 0: rebaselined the document from "M2 pending" to "M2 landed, M3 needed"
- CEO review: current "recommendation formalization" note rejected as too small and pointed at the wrong seam
- CEO voices: both outside voices converged on selection-to-proof governance as the real problem
- Design review: skipped, no UI scope
- Eng review: pinned approval artifacts, closeout validation, dual-authority rejection, and historical reconciliation as the M3 core
- Architecture: selection-to-proof chain defined with separate governance and downstream truth boundaries
- Test review: new lifecycle test diagram produced and artifact test plan path pinned
- Not in scope: written
- What already exists: written
- Failure modes: written with critical gap rule
- Parallelization: 4 lanes, with governance artifacts first and historical backfill last

## Cross-Phase Themes
- The repo narrative drifted behind shipped reality. M3 starts by fixing the story the repo tells about itself.
- Recommendation, approval, and proving run must not be collapsed into one truth source.
- Governance artifacts belong under `docs/project_management/next/<prefix>/governance/`, while registry and manifest artifacts remain downstream truth.
- The first real proving run is valuable only if the repo records what it learned.

## Decision Audit Trail
| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
|---|---|---|---|---|---|---|
| 1 | CEO | Rebaseline `PLAN.md` around landed M2 | mechanical | explicit over clever | The repo already shipped the proving run, so planning against a pending-M2 fiction is wrong | keep stale M2 framing |
| 2 | CEO | Reframe M3 around selection-to-proof governance | mechanical | choose completeness | Recommendation alone does not explain the repo's actual decision chain, and the Gemini/OpenCode relationship is now explicitly recorded | pure recommendation formalization |
| 3 | CEO | Keep recommendation authoring HITL in M3 | mechanical | pragmatic | Governance truth is the missing slice, not candidate automation | build `recommend-agent` first |
| 4 | CEO | Add explicit `selection_mode` to approval artifacts | taste | explicit over clever | The repo mixed factory validation and frontier expansion in one packet | keep one ambiguous rubric |
| 5 | Eng | Place approval artifacts under `docs/project_management/next/<prefix>/governance/` | mechanical | DRY | Registry and manifest artifacts already have separate owners | store approval in registry or manifests |
| 6 | Eng | Make `onboard-agent --approval` the canonical operator path | taste | explicit over clever | Re-entering the descriptor by hand is the wrong seam to preserve | long-flag operator workflow |
| 7 | Eng | Make artifact mode exclusive with semantic flags | mechanical | explicit over clever | Mixed authority would let operators mutate a different agent than the approved one | optional flag overrides in approval mode |
| 8 | Eng | Add validated closeout artifacts and a separate `close-proving-run` path | mechanical | choose completeness | Packet closure must become a validated lifecycle transition, not a loose metrics side effect | metrics-file presence closes packet |
| 9 | Eng | Require explicit override truth when approved agent differs from recommended agent | mechanical | explicit over clever | The `OpenCode` versus `gemini_cli` split is the observed repo bug | silent approval divergence |
| 10 | Eng | Backfill the Gemini proving run under the M3 artifact model | mechanical | boil lakes | Leaving the first mismatch unresolved would teach the repo the wrong lesson | defer historical reconciliation |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 1 | issues_open via `/autoplan` | current M3 note is too small; repo needs selection-to-proof governance and a post-M2 rebaseline |
| Codex Review | `codex exec` | Independent 2nd opinion | 2 | issues_open via `/autoplan` | stale top-level framing, broken decision governance, weak postmortem capture, and lifecycle-splitting requirements |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 1 | issues_open via `/autoplan` | approval artifact location, immutable identity, closeout validation, dual-authority rejection, and lifecycle tests pinned |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | skipped | no UI scope |

**CODEX:** Both Codex passes converged on the same correction. Stop planning M3 like recommendation theater and make the repo record what it actually decided and why.
**CROSS-MODEL:** Claude subagents and Codex agreed on the main direction. M3 must formalize approval provenance, closeout truth, and the recorded relationship where `gemini_cli` served as the fourth-agent proving-run vehicle without replacing `OpenCode` as the next recommendation lineage.
**UNRESOLVED:** 0
**VERDICT:** CEO + ENG CLEARED — M3 is concrete enough to implement.
