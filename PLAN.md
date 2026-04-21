<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/feat-cli-agent-onboarding-factory-autoplan-restore-20260421-105543.md -->
# CLI Agent Onboarding Factory - PLAN

Source:
- `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-main-design-20260420-151505.md`
- `docs/project_management/next/opencode-cli-onboarding/next-steps-handoff.md`
- `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-main-test-outcome-20260420-091704.md`
- `docs/project_management/next/gemini-cli-onboarding/HANDOFF.md`

Status: M1, M2, and M3 landed on `feat/cli-agent-onboarding-factory`; M4 is the next implementation milestone
Last updated (UTC): 2026-04-21

## Purpose
M4 turns post-onboarding maintenance from repo archaeology into a separate, repeatable lifecycle.

M1 created the reproducible onboarding bridge.
M2 added write-mode and proved the bridge on one real agent.
M3 formalized comparison -> approval -> proving-run closeout governance.

That leaves the next bottleneck. Once an agent is already in the repo, maintainers still have to discover drift manually across:
- `crates/xtask/data/agent_registry.toml`
- `cli_manifests/<agent>/**`
- `docs/specs/unified-agent-api/support-matrix.md`
- `docs/specs/unified-agent-api/capability-matrix.md`
- `docs/crates-io-release.md`
- closed onboarding and implementation packet docs

`onboard-agent` is not the answer to that problem. It is the create-mode bridge for new agents. M4 needs a separate maintenance lane for already-onboarded agents.

## Problem Statement
If an onboarded agent changes upstream or repo truth drifts, the current repo has no single maintenance entrypoint.

OpenCode already showed the failure shape. The landing itself succeeded, but `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-main-test-outcome-20260420-091704.md` records that `docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md` understates the landed OpenCode capability set versus:
- `crates/agent_api/src/backends/opencode/backend.rs`
- `docs/specs/opencode-agent-api-backend-contract.md`
- `docs/specs/unified-agent-api/capability-matrix.md`

That is exactly the class of bug M4 should eliminate:
- landed runtime/backend truth says one thing
- generated publication says one thing
- closed packet/governance docs say something else
- the operator has to manually rediscover the right repair path

The repo can now onboard a new agent with governed create-mode. It still cannot repair an existing agent boringly once drift appears. M4 must fix that.

## Landed Baseline
These are already true in this branch and are not M4 work:

- `crates/xtask/data/agent_registry.toml` seeds `codex`, `claude_code`, `opencode`, and `gemini_cli`.
- `crates/xtask/src/onboard_agent.rs` implements `--dry-run`, `--write`, and `--approval` for new-agent control-plane mutation.
- `crates/xtask/src/approval_artifact.rs` and `crates/xtask/src/proving_run_closeout.rs` validate approval and closeout truth.
- `crates/xtask/src/close_proving_run.rs` refreshes onboarding packet docs from a validated proving-run closeout artifact.
- `crates/xtask/src/support_matrix.rs`, `crates/xtask/src/support_matrix/derive.rs`, and `crates/xtask/src/support_matrix/consistency.rs` already derive and fail closed on support-publication drift.
- `crates/xtask/src/capability_matrix.rs` already derives capability publication from registry enrollment plus runtime/backend truth.
- `docs/project_management/next/gemini-cli-onboarding/**` is the first closed factory-backed proving-run packet.
- OpenCode is already landed, and `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-main-test-outcome-20260420-091704.md` documents a real post-onboarding drift issue to use as M4 input.

M4 builds on this exact repo state. It is not M2 or M3 cleanup, and it is not permission to widen `onboard-agent` into an everything command.

## Scope Lock
In scope:
- define a separate post-onboarding maintenance lifecycle for already-onboarded agents
- add agent-scoped drift detection across registry truth, manifest evidence, publication outputs, release docs, and packet/governance docs
- add a separate maintenance packet root under `docs/project_management/next/<agent>-maintenance/`
- add separate maintenance request and maintenance closeout artifacts
- add separate refresh ergonomics for control-plane-owned maintenance work
- keep maintenance writes bounded to control-plane-owned and generated surfaces
- use OpenCode as the first maintenance proving run because it has a real documented post-onboarding drift issue
- make reopen and closeout rules explicit so closed onboarding packets stay immutable

## Not In Scope
- adding update mode to `xtask onboard-agent`
- changing the recommendation, approval, or new-agent onboarding flow from M3
- generating or mutating runtime-owned wrapper/backend code under `crates/<agent>/` or `crates/agent_api/src/backends/<agent>/`
- mutating raw manifest evidence under `cli_manifests/<agent>/current.json`, `versions/`, `pointers/`, or `reports/` from the control plane
- collapsing recommendation, onboarding, proving-run closeout, and maintenance into one universal lifecycle command family
- changing support-matrix or capability-matrix semantics
- automating candidate research or building `recommend-agent`
- building a framework-scale runtime abstraction because one agent drifted

## Success Criteria
M4 is complete only when all of these are true:

- `xtask onboard-agent` remains create-only for new agents. Already-onboarded maintenance does not flow through it.
- `cargo run -p xtask -- check-agent-drift --agent <agent_id>` exists and:
  - exits `0` when the agent is clean
  - exits `2` when drift or validation problems are found
  - emits explicit drift categories instead of a generic failure blob
- `cargo run -p xtask -- refresh-agent --request <path> --dry-run` exists for already-onboarded agents.
- `cargo run -p xtask -- refresh-agent --request <path> --write` exists and shares the exact same render plan as `--dry-run`.
- `cargo run -p xtask -- close-agent-maintenance --request <path> --closeout <path>` exists and validates maintenance closure truth.
- Maintenance write mode mutates only:
  - `docs/project_management/next/<agent>-maintenance/**`
  - generated publication outputs from existing generators
  - the generated block inside `docs/crates-io-release.md` when it drifted
- Maintenance write mode never mutates:
  - `crates/<agent>/**`
  - `crates/agent_api/src/backends/<agent>/**`
  - raw manifest evidence files under `cli_manifests/<agent>/**`
  - historical onboarding packet roots such as `docs/project_management/next/<agent>-cli-onboarding/**`
- Closed onboarding packets remain immutable. Maintenance history is recorded in the separate maintenance pack.
- OpenCode is used as the proving run and the known stale capability claim in `docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md` becomes legible, repairable, and closeable through the M4 flow.
- The M4 test plan exists and remains current at `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-feat-cli-agent-onboarding-factory-test-plan-20260421-233454.md`.

## What Already Exists
M4 must reuse these surfaces instead of inventing a second factory:

- Registry and path truth:
  - `crates/xtask/data/agent_registry.toml`
  - `crates/xtask/src/agent_registry.rs`
- Existing control-plane mutation primitives:
  - `crates/xtask/src/onboard_agent.rs`
  - `crates/xtask/src/onboard_agent/preview.rs`
  - `crates/xtask/src/onboard_agent/preview/render.rs`
  - `crates/xtask/src/onboard_agent/validation.rs`
  - `crates/xtask/src/workspace_mutation.rs`
- Existing proving-run governance primitives:
  - `crates/xtask/src/approval_artifact.rs`
  - `crates/xtask/src/proving_run_closeout.rs`
  - `crates/xtask/src/close_proving_run.rs`
- Existing drift-sensitive publication surfaces:
  - `crates/xtask/src/support_matrix.rs`
  - `crates/xtask/src/support_matrix/derive.rs`
  - `crates/xtask/src/support_matrix/consistency.rs`
  - `crates/xtask/src/capability_matrix.rs`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- Existing release/doc generation surface:
  - `docs/crates-io-release.md`
- Historical maintenance input:
  - `docs/project_management/next/opencode-cli-onboarding/next-steps-handoff.md`
  - `docs/project_management/next/opencode-implementation/**`
  - `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-main-test-outcome-20260420-091704.md`

## Chosen Approach
M4 is a separate maintenance lane, not onboarding scope creep.

The repo already has a governed create flow:
- comparison
- approval
- create-mode onboarding
- proving-run closeout

The missing lifecycle is what happens after that when an onboarded agent drifts. The smallest complete M4 is:
- one drift detector
- one maintenance request artifact
- one bounded control-plane refresh command
- one maintenance closeout artifact
- one real proving run on OpenCode

Anything bigger is ocean-boiling. Anything smaller leaves the repo in the same archaeological post-onboarding posture that OpenCode exposed.

## Dream State Delta
```text
CURRENT STATE
already-onboarded agent
    |
    +--> drift appears in docs, publication, or governance truth
    +--> maintainer manually compares registry, manifests, runtime code, and packet docs
    +--> repair path is rediscovered from repo history

M4
already-onboarded agent
    |
    +--> check-agent-drift --agent <id>
    +--> maintenance-request.toml
    +--> refresh-agent --dry-run / --write
    +--> runtime/evidence follow-up when required
    +--> close-agent-maintenance

12-MONTH IDEAL
already-onboarded agent
    |
    +--> boring per-agent drift checks
    +--> boring maintenance packets
    +--> boring refresh/closeout loop
    +--> no reopen requires conversation archaeology
```

## M4 Plan Of Record
### Goal
Make already-onboarded agents repairable without reopening new-agent onboarding.

### Milestone Outcome
At the end of M4:

- maintainers can detect drift for one onboarded agent in one command
- maintainers can open one bounded maintenance packet for that agent
- control-plane-owned repair steps are previewable and replay-safe
- runtime/evidence follow-up stays explicit and separate
- maintenance closure records exactly what was resolved, what was deferred, and whether `make preflight` passed
- OpenCode proves the workflow on a real post-onboarding drift case

### Maintenance Chain
```text
drift trigger or stale proof
        |
        v
check-agent-drift --agent <agent_id>
        |
        v
maintenance-request.toml
        |
        v
refresh-agent --dry-run / --write
        |
        +--> control-plane-owned refreshes
        +--> explicit runtime/evidence follow-up list
        |
        v
close-agent-maintenance
        |
        v
closed maintenance pack + reopen trigger record
```

## Artifact Contract
### 1. Maintenance request artifact
Path: `docs/project_management/next/<agent>-maintenance/governance/maintenance-request.toml`
Format: TOML
Owner: maintainer workflow

Required fields:
- `artifact_version`
- `agent_id`
- `trigger_kind`
- `basis_ref`
- `opened_from`
- `requested_control_plane_actions`
- `runtime_followup_required`
- `request_recorded_at`
- `request_commit`

Rules:
- must reference an already-onboarded agent in `agent_registry.toml`
- must not be used to create a new agent
- must live under the maintenance pack root, not the onboarding pack root
- `requested_control_plane_actions` may include only bounded maintenance actions:
  - `packet_doc_refresh`
  - `support_matrix_refresh`
  - `capability_matrix_refresh`
  - `release_doc_refresh`

### 2. Maintenance closeout artifact
Path: `docs/project_management/next/<agent>-maintenance/governance/maintenance-closeout.json`
Format: JSON
Owner: maintenance closeout

Required fields:
- `request_ref`
- `request_sha256`
- `resolved_findings`
- exactly one of:
  - `deferred_findings`
  - `explicit_none_reason`
- `preflight_passed`
- `recorded_at`
- `commit`

Rules:
- closeout must fail validation if it cannot link back to the request artifact
- closeout must state what was resolved and whether anything remains deferred
- maintenance closure must not mutate historical onboarding packet docs directly

## Command Contract
M4 adds a separate maintenance command set.

### Drift detection
```bash
cargo run -p xtask -- check-agent-drift --agent <agent_id>
```

Rules:
- read-only
- agent must already exist in `crates/xtask/data/agent_registry.toml`
- exit `0` means no drift
- exit `2` means drift or maintenance preconditions failed
- output categories must include, when present:
  - registry versus manifest evidence drift
  - runtime/backend versus capability publication drift
  - support publication drift
  - release/doc generated-block drift
  - closed packet/governance doc drift

### Maintenance refresh
```bash
cargo run -p xtask -- refresh-agent --request docs/project_management/next/<agent>-maintenance/governance/maintenance-request.toml --dry-run
cargo run -p xtask -- refresh-agent --request docs/project_management/next/<agent>-maintenance/governance/maintenance-request.toml --write
```

Rules:
- `--dry-run` and `--write` are mutually exclusive
- dry-run and write must share one in-memory render plan
- request artifact path is jailed and maintenance-root validated
- unknown agent ids fail closed
- request actions that imply runtime-owned mutations fail closed
- exact-byte replay after an identical write is a success no-op

### Maintenance closeout
```bash
cargo run -p xtask -- close-agent-maintenance --request docs/project_management/next/<agent>-maintenance/governance/maintenance-request.toml --closeout docs/project_management/next/<agent>-maintenance/governance/maintenance-closeout.json
```

Rules:
- validates request linkage and request hash
- requires explicit resolved findings plus either deferred findings or `explicit_none_reason`
- refreshes maintenance packet docs only
- does not reopen or rewrite the historical onboarding packet root

## Controlled Write Set
The maintenance lane is intentionally narrow.

| Surface | Owner | M4 write mode |
|---|---|---|
| `docs/project_management/next/<agent>-maintenance/**` | maintenance control plane | write |
| `docs/specs/unified-agent-api/support-matrix.md` and `cli_manifests/support_matrix/current.json` via existing generator | generated publication | write |
| `docs/specs/unified-agent-api/capability-matrix.md` via existing generator | generated publication | write |
| generated block in `docs/crates-io-release.md` | generated publication | write when drifted |
| `crates/xtask/data/agent_registry.toml` | registry truth | read-only in M4 unless explicit follow-on reopening is approved |
| `cli_manifests/<agent>/current.json`, `versions/`, `pointers/`, `reports/` | manifest evidence | never |
| `crates/<agent>/**` | runtime owner | never |
| `crates/agent_api/src/backends/<agent>/**` | runtime owner | never |
| `docs/project_management/next/<agent>-cli-onboarding/**` | historical onboarding packet | never |

## Workstreams
### W1. Agent-Scoped Drift Detection
Goal: stop making operators manually discover which truth surfaces disagree.

Deliverables:
- `check-agent-drift` entrypoint
- one explicit drift category taxonomy
- agent-scoped output that aggregates existing validators instead of duplicating them

Primary modules:
- `crates/xtask/src/support_matrix.rs`
- `crates/xtask/src/support_matrix/derive.rs`
- `crates/xtask/src/support_matrix/consistency.rs`
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/agent_registry.rs`

Exit criteria:
- one command can tell maintainers whether an onboarded agent is clean or which surfaces drifted

### W2. Maintenance Request + Refresh
Goal: add a separate operator path for already-onboarded agents.

Deliverables:
- `maintenance-request.toml` schema
- `refresh-agent --dry-run`
- `refresh-agent --write`
- maintenance pack scaffold under `docs/project_management/next/<agent>-maintenance/`

Primary modules:
- `crates/xtask/src/main.rs`
- new maintenance command module(s) under `crates/xtask/src/`
- `crates/xtask/src/workspace_mutation.rs`

Exit criteria:
- already-onboarded maintenance no longer requires `onboard-agent`
- refresh writes stay bounded to maintenance and generated publication surfaces

### W3. Maintenance Closeout + Reopen Rules
Goal: close repairs deterministically without mutating historical onboarding truth.

Deliverables:
- `maintenance-closeout.json` schema
- `close-agent-maintenance` entrypoint
- explicit reopen rules for recurring drift

Primary modules:
- `crates/xtask/src/main.rs`
- new maintenance closeout module(s) under `crates/xtask/src/`
- maintenance packet docs under `docs/project_management/next/<agent>-maintenance/**`

Exit criteria:
- closed maintenance runs are explicit and replay-safe
- reopening uses the maintenance lane, not edits to closed onboarding packets

### W4. OpenCode Maintenance Proving Run
Goal: prove the maintenance lane on a real post-onboarding drift issue.

Deliverables:
- `docs/project_management/next/opencode-maintenance/**`
- a maintenance request that cites the stale capability claim
- refreshed maintenance packet truth
- validated maintenance closeout

Primary modules:
- `docs/project_management/next/opencode-implementation/**`
- `docs/specs/opencode-agent-api-backend-contract.md`
- `docs/specs/unified-agent-api/capability-matrix.md`

Exit criteria:
- the repo can repair the OpenCode stale closeout claim through the new M4 flow without conversation archaeology

## Implementation Sequence
### Phase 1. Drift Contract Lock
Outputs:
- drift taxonomy
- `check-agent-drift`
- OpenCode proving-run target confirmed

Exit gate:
- one command can expose the OpenCode stale capability claim as maintenance drift

### Phase 2. Maintenance Request + Refresh
Outputs:
- maintenance request schema
- maintenance pack scaffold
- `refresh-agent --dry-run/--write`

Exit gate:
- maintenance writes are bounded and replay-safe

### Phase 3. Maintenance Closeout
Outputs:
- closeout schema
- `close-agent-maintenance`
- reopen rules

Exit gate:
- maintenance history closes without mutating closed onboarding packets

### Phase 4. OpenCode Proving Run
Outputs:
- OpenCode maintenance pack
- repaired capability-claim truth
- validated closeout

Exit gate:
- the repo can repair one already-onboarded agent boringly

## Error & Rescue Registry
| Method / Codepath | What can go wrong | Failure class | Rescued? | Rescue action | User sees |
|---|---|---|---|---|---|
| `check-agent-drift --agent` | unknown or non-onboarded agent id | validation error | yes | reject before comparison work | exit `2` |
| drift aggregation | one source loads, another source fails | partial truth | yes | fail closed with category-specific error | explicit drift/load failure |
| maintenance request parse | malformed TOML or invalid `requested_control_plane_actions` | validation error | yes | reject before refresh plan build | exit `2` |
| maintenance request path | request artifact escapes maintenance root | ownership violation | yes | reject before artifact load | exit `2` |
| refresh write plan | request implies runtime-owned mutation | scope violation | yes | reject before any writes | exit `2` |
| refresh apply | one generated surface diverges mid-transaction | mutation error | yes | rollback staged writes | repo unchanged |
| maintenance closeout | request linkage missing or hashes do not match | validation error | yes | reject closeout | exit `2` |
| OpenCode proving run | stale claim cannot be reconciled to runtime/spec truth | needs-context | no | block closeout until maintainer decides source of truth | blocked docs update |

## Test Strategy
### Test Diagram
```text
POST-ONBOARDING MAINTENANCE
===========================
[+] already-onboarded agent -> check-agent-drift
    |
    ├── [GAP -> validation] unknown agent fails closed
    ├── [GAP -> aggregation] support publication drift is surfaced per agent
    ├── [GAP -> aggregation] capability/runtime drift is surfaced per agent
    └── [GAP -> aggregation] governance packet drift is surfaced per agent

[+] maintenance-request.toml -> refresh-agent --dry-run / --write
    |
    ├── [GAP -> validation] request outside maintenance root fails
    ├── [GAP -> validation] runtime-owned actions are rejected
    ├── [GAP -> integration] dry-run and write share the same plan
    ├── [GAP -> integration] historical onboarding packet remains untouched
    └── [GAP -> regression] identical replay is a no-op

[+] maintenance-closeout.json -> close-agent-maintenance
    |
    ├── [GAP -> validation] request hash/linkage is required
    ├── [GAP -> validation] resolved plus deferred/explicit-none truth is required
    └── [GAP -> integration] maintenance packet docs refresh without touching onboarding packet docs

OPENCODE PROVING RUN
====================
[+] stale `SEAM-2` capability claim -> maintenance request -> refresh -> closeout
    |
    ├── [GAP -> docs/validation] stale capability claim becomes explicit maintenance drift
    └── [GAP -> regression] repair path is reproducible without conversation history
```

### Required Test Surfaces
- Add `crates/xtask/tests/agent_maintenance_drift.rs`
  - `check_agent_drift_reports_clean_agent`
  - `check_agent_drift_rejects_unknown_agent`
  - `check_agent_drift_reports_support_publication_mismatch`
  - `check_agent_drift_reports_capability_truth_mismatch`
  - `check_agent_drift_reports_governance_doc_mismatch`
- Add `crates/xtask/tests/agent_maintenance_refresh.rs`
  - `refresh_agent_dry_run_matches_write_plan`
  - `refresh_agent_rejects_request_outside_maintenance_root`
  - `refresh_agent_rejects_runtime_owned_actions`
  - `refresh_agent_does_not_touch_onboarding_packet_root`
  - `refresh_agent_replay_is_noop`
- Add `crates/xtask/tests/agent_maintenance_closeout.rs`
  - `close_agent_maintenance_requires_request_linkage`
  - `close_agent_maintenance_requires_resolved_and_deferred_truth`
  - `close_agent_maintenance_rejects_symlinked_output`
  - `opencode_maintenance_proving_run_fixes_stale_capability_claim`

### Verification Commands
- `cargo run -p xtask -- check-agent-drift --agent opencode`
- `cargo run -p xtask -- refresh-agent --request docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml --dry-run`
- `cargo run -p xtask -- refresh-agent --request docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml --write`
- `cargo run -p xtask -- close-agent-maintenance --request docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml --closeout docs/project_management/next/opencode-maintenance/governance/maintenance-closeout.json`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix`
- `cargo test -p xtask`
- `make preflight`

### Test Plan Artifact
- `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-feat-cli-agent-onboarding-factory-test-plan-20260421-233454.md`

## Failure Modes Registry
| Codepath | Failure mode | Test required? | Error handling required? | User sees | Logged? |
|---|---|---|---|---|---|
| drift detection | known drift exists but stays hidden in repo-wide generators only | yes | yes | false clean state | yes |
| maintenance scope | maintenance path widens into new-agent onboarding or runtime mutation | yes | yes | unsafe write rejection | yes |
| packet immutability | refresh mutates historical onboarding packets | yes | yes | validation failure | yes |
| publication repair | agent-scoped repair misses global generated outputs | yes | yes | stale support/capability docs remain | yes |
| governance truth | maintenance closeout claims clean state while deferrals still exist | yes | yes | closeout rejected | yes |
| OpenCode proving run | stale capability claim remains unrepairable without archaeology | yes | yes | blocked proving run | yes |

Critical gap rule:
- if maintenance can mutate runtime-owned code or historical onboarding packet roots, M4 is not ready
- if OpenCode cannot prove the repair lane on a real drift case, M4 is not ready

## Security Review
- maintenance request and closeout artifacts are new trust boundaries and must be path-jailed
- `refresh-agent` must never infer permission to mutate runtime-owned code from a maintenance request
- agent-scoped drift checks must not trust packet docs over runtime/spec truth
- global generated outputs must refresh deterministically or fail closed
- the maintenance lane should reuse the same symlink and rollback protections that M2/M3 added for onboarding

## Performance Review
- `check-agent-drift` should aggregate existing support/capability validators instead of re-implementing them
- `refresh-agent` should batch planned writes into one transaction instead of re-running file updates per surface
- maintenance should stay agent-scoped at the operator layer even when publication outputs are global files

## Worktree Parallelization Strategy
### Dependency Table
| Step | Modules touched | Depends on |
|---|---|---|
| W1. drift detection | `crates/xtask/src/support_matrix/**`, `crates/xtask/src/capability_matrix.rs`, new maintenance drift module(s), tests | — |
| W2. request + refresh | `crates/xtask/src/main.rs`, new maintenance refresh module(s), `crates/xtask/src/workspace_mutation.rs`, tests | W1 |
| W3. closeout + reopen rules | `crates/xtask/src/main.rs`, new maintenance closeout module(s), maintenance docs templates, tests | W2 |
| W4. OpenCode proving run | `docs/project_management/next/opencode-maintenance/**`, related governance docs | W1, W2, W3 |

### Parallel Lanes
Lane A: W1
This lands first. The drift taxonomy and detection output define what the rest of M4 is repairing.

Lane B: W2
Refresh ergonomics. Runs after W1.

Lane C: W3
Closeout and reopen rules. Runs after W2 stabilizes the request schema.

Lane D: W4
OpenCode proving run. Runs last, because it should consume the final maintenance contract rather than encode a moving target.

### Conflict Flags
- W2 and W3 both touch `crates/xtask/src/main.rs` and the new maintenance module namespace.
- W1 and W2 both touch `crates/xtask/tests/**`. Split test ownership early.
- W4 should not start before request and closeout schemas stabilize or it will encode the wrong maintenance truth.

## Deferred To TODOS.md
- automate maintenance-request generation from upstream release scans only after two successful maintenance cycles prove the shape
- add manifest-evidence refresh helpers only after the repo proves it can keep manifest evidence ownership separate from control-plane refresh
- consider batched multi-agent maintenance scheduling only after per-agent maintenance is boring

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 1 | issues_open via `/autoplan` | M4 must be a separate post-onboarding lifecycle, not `onboard-agent` update mode or lifecycle-command sprawl |
| Codex Review | `codex exec` | Independent 2nd opinion | 1 | partial / timed out | outside-voice attempt timed out after repo sweep; usable signal still matched the local read that OpenCode is the right proving run because it has a real post-onboarding drift issue |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 1 | issues_open via `/autoplan` | drift aggregation, separate maintenance request/closeout artifacts, bounded refresh writes, and historical packet immutability must be pinned |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | skipped | no UI scope |

**CEO:** The strategic trap is obvious. If M4 widens `onboard-agent` or invents a universal lifecycle umbrella, the repo will spend a milestone rebuilding abstractions instead of fixing the first real maintenance seam.
**ENG:** The engineering seam is also clear. The repo already has most of the primitives. M4 should compose them into agent-scoped drift detection, separate maintenance packets, and bounded refresh writes instead of duplicating generator logic.
**CROSS-PHASE THEME:** OpenCode is the high-confidence proving run because it already produced a concrete post-onboarding drift issue in committed artifacts.
**UNRESOLVED:** 0
**VERDICT:** CEO + ENG CLEARED — M4 is concrete enough to implement.
