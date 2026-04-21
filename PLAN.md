<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/feat-cli-agent-onboarding-factory-autoplan-restore-20260421-105543.md -->
# CLI Agent Onboarding Factory - PLAN

Source:
- `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-main-design-20260420-151505.md`
- `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`
- `docs/project_management/next/gemini-cli-onboarding/HANDOFF.md`

Status: M1 and M2 landed on `feat/cli-agent-onboarding-factory`; M3 is the next implementation milestone
Last updated (UTC): 2026-04-21

## Purpose
M3 turns the repo's post-M2 selection gap into explicit, auditable repo truth.

M2 already proved the control-plane mutation slice:
- `xtask onboard-agent --write` exists
- control-plane writes are transactional and replay-safe
- one real proving run landed for `gemini_cli`
- the proving run closed with committed packet artifacts and `make preflight`

That changed the bottleneck. The repo can now point to three conflicting truths without saying how they relate:
- the comparison packet recommended `OpenCode`
- the first closed proving run landed `gemini_cli`
- the top-level plan historically spoke as if M2 were still pending

M3 fixes that seam. It formalizes the full selection-to-proof chain:
- what was compared
- what was recommended
- what was approved
- why approval diverged when it did
- what the proving run taught us

## Problem Statement
If M3 only adds recommendation tooling, it automates the least trustworthy part of the flow.

The real bug is not "we cannot pick another agent." The real bug is that the repo cannot currently explain the relationship between recommendation, approval, onboarding input, and proving-run closeout without conversation archaeology. The next milestone must make that chain boring and deterministic.

## Landed Baseline
These are already true in this branch and are not M3 work:

- `crates/xtask/data/agent_registry.toml` seeds `codex`, `claude_code`, `opencode`, and `gemini_cli`
- `crates/xtask/src/onboard_agent.rs` implements `--dry-run` and `--write`
- `crates/xtask/src/onboard_agent/mutation.rs` enforces path jailing, staged writes, and rollback
- `crates/xtask/src/onboard_agent/validation.rs` enforces registry/package/filesystem fail-closed behavior for raw descriptor input
- `docs/project_management/next/gemini-cli-onboarding/**` is the first closed proving-run packet
- `docs/project_management/next/gemini-cli-onboarding/governance/proving-run-metrics.json` records M2 closeout metrics, but not enough provenance for M3
- `crates/xtask/tests/onboard_agent_entrypoint.rs` and `crates/xtask/tests/onboard_agent_closeout_preview.rs` already prove M2 packet generation and closeout rendering

M3 builds on this exact repo state. It is not partial M2 cleanup.

## Scope Lock
In scope:
- rebaseline `PLAN.md` around post-M2 repo truth
- formalize three artifacts in the selection-to-proof chain:
  - comparison packet
  - approval artifact
  - proving-run closeout artifact
- keep approval and closeout artifacts under `docs/project_management/next/<prefix>/governance/`
- make `xtask onboard-agent` consume an approval artifact as the canonical operator path
- keep raw long-flag descriptor input only as a fixture/backfill path
- add one explicit closeout validation path so packet state is not inferred from metrics-file presence alone
- backfill one authoritative decision record explaining that `gemini_cli` was the fourth-agent proving-run vehicle while `OpenCode` remains the next recommendation lineage

## Not In Scope
- automated candidate research or `recommend-agent` in M3
- changing support-matrix or capability-matrix semantics
- moving approval truth into `agent_registry.toml` or `cli_manifests/**`
- generating wrapper crate or backend source files
- runtime-lane compression for wrapper/backend implementation
- update mode for already-onboarded agents
- universal lifecycle orchestration across recommendation, approval, onboarding, and maintenance in one command family

## Success Criteria
M3 is complete only when all of these are true:

- `PLAN.md` is post-M2 accurate and no longer frames M2 as pending
- a committed approval artifact exists and records:
  - `comparison_ref`
  - `selection_mode`
  - `recommended_agent_id`
  - `approved_agent_id`
  - `override_reason` when those differ
  - the exact onboarding descriptor consumed by the factory
- `cargo run -p xtask -- onboard-agent --approval <path> --dry-run` exists
- `cargo run -p xtask -- onboard-agent --approval <path> --write` exists
- artifact mode is exclusive with semantic descriptor flags
- generated packet outputs stamp the approval artifact path plus an immutable approval hash or id
- proving-run closeout can no longer be inferred from metrics-file presence alone
- a validated closeout artifact exists and records:
  - approval source
  - exactly one of `duration_seconds` or `duration_missing_reason`
  - either explicit residual friction items or `explicit_none_reason`
  - proving-run state
- the repo records one authoritative answer to this question:
  - `gemini_cli` was the fourth-agent proving-run vehicle
  - `OpenCode` remains the next recommendation lineage and was not replaced by Gemini
- the historical `gemini_cli` proving run is backfilled or superseded so the closed packet has trustworthy provenance
- the M3 test plan remains actionable and current at `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-feat-cli-agent-onboarding-factory-test-plan-20260421-105654.md`

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

## Chosen Approach
M3 is a governance-first milestone.

Keep recommendation authoring human-in-the-loop. Formalize approval and closeout truth first, then revisit recommendation automation after the repo has real feedback to automate against. That is the smallest complete slice that makes agent five boring instead of archaeological.

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

## Artifact Contract
### 1. Comparison packet
Format: Markdown
Owner: maintainer workflow
Role: explain candidate set, rubric, recommendation, and evidence

Rules:
- informative, not normative
- may recommend one agent without locking approval to that winner

### 2. Approval artifact
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
- `selection_mode` is explicit, with:
  - `factory_validation`
  - `frontier_expansion`

### 3. Proving-run closeout artifact
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

## Command Contract
M3 keeps the existing raw descriptor path for fixtures and backfills, but the canonical operator path becomes approval-driven:

```bash
cargo run -p xtask -- onboard-agent --approval docs/project_management/next/<prefix>/governance/approved-agent.toml --dry-run
cargo run -p xtask -- onboard-agent --approval docs/project_management/next/<prefix>/governance/approved-agent.toml --write
cargo run -p xtask -- close-proving-run --approval docs/project_management/next/<prefix>/governance/approved-agent.toml --closeout docs/project_management/next/<prefix>/governance/proving-run-closeout.json
```

Rules:
- `--approval` is mutually exclusive with semantic descriptor flags
- artifact mode prints the resolved approval artifact id or hash in stdout
- generated packet artifacts stamp the approval reference and approval hash
- `close-proving-run` validates the closeout artifact and refreshes packet closeout rendering

## Architecture
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
### W1. Rebaseline + Contract Definition
Goal: make the repo say what is actually true before adding new lifecycle logic.

Deliverables:
- rewrite `PLAN.md` around landed M2 plus M3 scope
- define the approval and closeout artifact contracts
- record explicitly that `gemini_cli` was the fourth-agent proving-run vehicle and did not replace `OpenCode`

Primary modules:
- `PLAN.md`
- `docs/project_management/next/**`
- `crates/xtask/src/main.rs`

Exit criteria:
- no top-level planning doc still frames M2 as pending
- historical recommendation versus proving-run divergence has one explicit owner truth
- artifact names, locations, and invariants are stable enough for implementation

### W2. Approval-Driven Onboarding
Goal: remove descriptor re-entry from the operator path without blurring truth boundaries.

Deliverables:
- `approved-agent.toml` schema and validation
- `--approval` input mode for `onboard-agent`
- approval hash/id stamping in generated packet outputs
- namespace and provenance validation for approval artifacts

Primary modules:
- `crates/xtask/src/main.rs`
- `crates/xtask/src/onboard_agent.rs`
- `crates/xtask/src/onboard_agent/validation.rs`
- `crates/xtask/tests/onboard_agent_entrypoint.rs`

Depends on:
- W1 artifact contract and path invariants

Exit criteria:
- artifact mode and raw semantic flags cannot be mixed
- approval artifacts are repo-jail-safe, immutable, and auditable
- dry-run and write parity still holds under approval mode

### W3. Closeout Validation + Packet State Hardening
Goal: stop inferring packet closure from loose metrics blobs.

Deliverables:
- validated closeout schema
- `close-proving-run` entrypoint
- rendering changes that require approval linkage plus complete closeout truth
- explicit execution-mode versus closeout-mode packet transitions

Primary modules:
- `crates/xtask/src/main.rs`
- `crates/xtask/src/onboard_agent/preview.rs`
- `crates/xtask/src/onboard_agent/preview/render.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview.rs`

Depends on:
- W1 artifact contract and path invariants

Exit criteria:
- partial or stale closeout artifacts cannot render a closed packet
- packet closeout requires approval linkage, duration truth, and residual-friction truth

### W4. Historical Reconciliation
Goal: clean up the first real mismatch before the repo repeats it.

Deliverables:
- backfilled approval artifact for the `gemini_cli` proving run
- explicit recorded relationship to `cli-agent-onboarding-third-agent-packet.md`
- migrated or superseding closeout artifact with M3-required fields

Primary modules:
- `docs/project_management/next/gemini-cli-onboarding/**`

Depends on:
- W1 contract decisions
- W2 approval artifact contract in final form
- W3 closeout artifact contract in final form

Exit criteria:
- the historical `OpenCode` recommendation and `gemini_cli` fourth-agent proving run are legible in committed repo truth
- future maintainers do not have to infer why Gemini landed as the fourth agent without replacing OpenCode as the next recommended lineage

## Implementation Sequence
### Phase 1. Rebaseline + Schema Lock
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
| Method / Codepath | What can go wrong | Failure class | Rescued? | Rescue action | User sees |
|---|---|---|---|---|---|
| approval artifact parse | malformed TOML or missing required fields | validation error | yes | reject before planning writes | exit `2` |
| approval artifact path | artifact path escapes repo or points outside governance surfaces | ownership violation | yes | reject before artifact load | exit `2` |
| approval override validation | `recommended_agent_id != approved_agent_id` without `override_reason` | validation error | yes | reject before dry-run/write | exit `2` |
| approval replay check | approval descriptor differs from already-onboarded identical agent state | conflict | yes | reject with approval id/hash context | exit `2` |
| packet rendering | closeout artifact exists but approval linkage or state is incomplete | state transition error | yes | keep packet in execution mode | explicit validation failure |
| closeout validation | duration and residual-friction truth missing | validation error | yes | reject closeout until explicit truth exists | exit `2` |
| historical backfill | Gemini approval chain cannot be explained from available evidence | needs-context | no | record explicit maintainer decision or block backfill | blocked docs update |

## Test Strategy
### Test Diagram
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

### Required Test Surfaces
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

### Verification Commands
- `cargo run -p xtask -- onboard-agent --approval docs/project_management/next/<prefix>/governance/approved-agent.toml --dry-run`
- `cargo run -p xtask -- onboard-agent --approval docs/project_management/next/<prefix>/governance/approved-agent.toml --write`
- `cargo run -p xtask -- close-proving-run --approval docs/project_management/next/<prefix>/governance/approved-agent.toml --closeout docs/project_management/next/<prefix>/governance/proving-run-closeout.json`
- `cargo test -p xtask`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix`
- `make preflight`

### Test Plan Artifact
- `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-feat-cli-agent-onboarding-factory-test-plan-20260421-105654.md`

## Failure Modes Registry
| Codepath | Failure mode | Test required? | Error handling required? | User sees | Logged? |
|---|---|---|---|---|---|
| approval identity | packet claims an approved agent without immutable approval linkage | yes | yes | explicit validation failure | yes |
| dual authority | semantic flags drift from approval artifact input | yes | yes | explicit usage error | yes |
| governance drift | closeout artifact references the wrong approval or stale approval contents | yes | yes | closeout rejected | yes |
| closure theater | loose metrics file closes packet without timing or residual-friction truth | yes | yes | packet remains execution-mode | yes |
| historical ambiguity | recommendation and proving run diverge without override truth | yes | yes | blocked reconciliation | yes |

Critical gap rule:
- if any failure mode above lacks both validation and tests, M3 is not ready

## Security Review
- approval artifact input is a new trust boundary and must be path-jailed, schema-validated, and provenance-validated before the repo trusts any descriptor field
- approval artifacts must stay out of `agent_registry.toml` and `cli_manifests/**` so governance truth does not masquerade as runtime or publication truth
- artifact mode must be exclusive with semantic descriptor flags, because mixed-authority invocation is a correctness bug
- closeout artifacts must not be treated as proof of success unless linked to a specific approval artifact and validated as complete

## Performance Review
- approval ingestion should parse once and feed the existing mutation-plan path, not create a second rendering path
- approval hash stamping should be computed once per invocation, not per generated file
- closeout validation should refresh packet rendering deterministically without re-scanning unrelated repo surfaces

## Worktree Parallelization Strategy
### Dependency Table
| Step | Modules touched | Depends on |
|---|---|---|
| W1. Rebaseline + contract definition | `PLAN.md`, `docs/project_management/next/**`, `crates/xtask/src/` | — |
| W2. Approval-driven onboarding | `crates/xtask/src/onboard_agent/**`, `crates/xtask/src/main.rs`, `crates/xtask/tests/**` | W1 |
| W3. Closeout validation + packet state hardening | `crates/xtask/src/onboard_agent/preview/**`, `crates/xtask/src/main.rs`, `crates/xtask/tests/**` | W1 |
| W4. Historical reconciliation | `docs/project_management/next/gemini-cli-onboarding/**` | W1, W2, W3 |

### Parallel Lanes
Lane A: W1
This is sequential. It locks the artifact names, invariants, and operator contract that the rest of the milestone depends on.

Lane B: W2
Approval parsing, validation, and `onboard-agent --approval`. Runs after Lane A.

Lane C: W3
Closeout schema, `close-proving-run`, and packet rendering hardening. Runs after Lane A.

Lane D: W4
Historical Gemini reconciliation. Runs after Lanes B and C finalize the artifact contract.

### Execution Order
1. Land Lane A first.
2. Launch Lanes B and C in parallel worktrees once artifact names and invariants are stable.
3. Merge B and C.
4. Launch Lane D after the approval and closeout contracts are both final.

### Conflict Flags
- Lanes B and C both touch `crates/xtask/src/main.rs`. Expect a small command-surface merge.
- Lanes B and C both touch `crates/xtask/tests/**`. Split test ownership up front to avoid fixture collisions.
- Lane D should not start early. If it backfills against a moving approval or closeout schema, it will encode the wrong historical truth.

## Deferred To TODOS.md
- add `recommend-agent` automation or a deterministic packet generator after two governance-backed comparison cycles prove repetition is real
- compress the runtime-owned wrapper/backend lane after the governance chain exposes which runtime steps still dominate lead time
- add multi-agent drift/update maintenance flows after the repo finishes one post-M3 onboarding cycle with explicit approval and closeout linkage

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
