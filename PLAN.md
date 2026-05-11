# PLAN - Maintenance Settlement Must Gate `closed_baseline`

Status: ready for implementation  
Date: 2026-05-11  
Working branch: `staging`  
Plan revision baseline: `35cf547`  
Design input: `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-design-20260511-000001.md`  
Supersedes: the prior repo-root `PLAN.md` for the maintenance proof-run milestone

## Executive Summary

The shared watcher is not the bug.

The create lane is.

Today an agent can finish publication honestly and still reach `closed_baseline` without ever
settling whether ongoing maintenance is enrolled or explicitly deferred. That makes
`closed_baseline` over-claim. This milestone fixes the seam where that lie enters the system:
approval artifact -> `onboard-agent` -> `close-proving-run` -> lifecycle evidence.

The implementation stays boring on purpose:

- keep `artifact_version = "1"`
- keep `crates/xtask/data/agent_registry.toml` as the only release-watch enrollment store
- add one bounded approval-side maintenance decision: `release_watch_enrolled` or `explicitly_deferred`
- make `close-proving-run` fail closed unless approval truth and registry truth match one legal branch
- snapshot the settled maintenance decision into machine-owned closeout evidence before
  `closed_baseline` is legal

## Objective

Make maintenance settlement part of done for create-mode onboarding so an agent cannot reach
`closed_baseline` until the repo has one explicit, validated answer to exactly one question:

1. is this agent enrolled in release-watch maintenance, or
2. is this agent explicitly deferred from that lane for now?

## Success Criteria

1. The approval artifact contract supports an explicit maintenance decision under
   `[descriptor.maintenance]` without bumping `artifact_version`.
2. `onboard-agent --write` materializes `maintenance.release_watch` into
   `crates/xtask/data/agent_registry.toml` only when approval mode is
   `release_watch_enrolled`.
3. `onboard-agent --write` leaves `maintenance.release_watch` absent when approval mode is
   `explicitly_deferred`.
4. `close-proving-run` fails closed unless approval truth and registry truth match one of the two
   allowed branches.
5. `docs/agents/lifecycle/*/governance/proving-run-closeout.json` captures immutable
   machine-owned maintenance settlement evidence for newly closed runs.
6. `LifecycleStage::ClosedBaseline` requires a new evidence bit for settled maintenance truth.
7. `opencode` and `gemini_cli` are backfilled to truthful committed state without inventing a
   second enrollment inventory.
8. `aider` proves the tightened deferred branch from its current `publication_ready` state and
   cannot close until maintenance settlement is explicit.
9. `make preflight` passes.

## Locked Decisions

1. Keep `artifact_version = "1"`.
2. The only allowed approval-side maintenance modes in this milestone are:
   - `release_watch_enrolled`
   - `explicitly_deferred`
3. `crates/xtask/data/agent_registry.toml` remains the only release-watch enrollment store.
4. Approval-side deferral is evidence, not enrollment.
5. `opencode`, `gemini_cli`, and `aider` all use `explicitly_deferred` in this milestone.
6. `codex` and `claude_code` registry release-watch blocks stay as they are.
7. No watcher-topology, packet-topology, or workflow-topology redesign is in scope here.
8. No new `manual_only` or broader maintenance taxonomy is introduced.
9. `close-proving-run` is the only command that may write final maintenance settlement evidence
   into the closed closeout artifact.

## Step 0 Scope Contract

### Premise Challenge

| Premise | Assessment | Decision |
| --- | --- | --- |
| The watcher side is wrong. | Rejected. `maintenance-watch` and `prepare-agent-maintenance` already consume registry truth consistently. | Do not change watcher topology in this milestone. |
| The missing truth belongs in the create lane. | Accepted. Approval, onboarding, lifecycle, and closeout are the broken seam. | Fix the approval-to-closeout contract. |
| This needs a new enrollment inventory outside the registry. | Rejected. The registry contract explicitly forbids that. | Registry remains the only enrollment store. |
| This needs a full artifact-version bump. | Rejected. The change is additive and bounded. | Keep `artifact_version = "1"`. |
| This needs a broader maintenance taxonomy now. | Rejected. That is bigger than the bug. | Only add `release_watch_enrolled` and `explicitly_deferred`. |
| Already-closed agents can stay ambiguous. | Rejected. They are the current lie in committed history. | Backfill `opencode` and `gemini_cli`. |
| The in-flight proof target should also widen release-watch rollout. | Rejected. That mixes lifecycle settlement with rollout expansion. | Keep `aider` deferred in this milestone and prove the deferred branch. |

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| Registry release-watch schema and validation | `crates/xtask/src/agent_registry/release_watch.rs`, `docs/specs/agent-registry-contract.md` | Reuse. Do not fork a second validator. |
| Approval artifact loading and validation | `crates/xtask/src/approval_artifact.rs` | Extend additively. |
| Approval-to-registry materialization | `crates/xtask/src/onboard_agent.rs`, `crates/xtask/src/onboard_agent/descriptor.rs`, `crates/xtask/src/onboard_agent/approval.rs`, `crates/xtask/src/onboard_agent/preview.rs` | Reuse. Add maintenance decision threading here. |
| Closeout gate | `crates/xtask/src/close_proving_run.rs` | Tighten here. This is the final truth gate. |
| Lifecycle evidence model | `crates/xtask/src/agent_lifecycle.rs` | Reuse. Add one evidence id, not a new stage. |
| Closeout artifact schema | `crates/xtask/src/proving_run_closeout.rs` | Extend with machine-owned settlement snapshot. |
| Historical closed-baseline repair | `crates/xtask/src/historical_lifecycle_backfill.rs` | Reuse for closed-agent backfill instead of manual artifact surgery. |
| Current affected governance artifacts | `docs/agents/lifecycle/opencode-cli-onboarding/**`, `docs/agents/lifecycle/gemini-cli-onboarding/**`, `docs/agents/lifecycle/aider-onboarding/**` | Update directly as part of this PR. |

### Minimum Complete Change

The minimum complete implementation is:

1. add approval-side maintenance truth
2. thread it into `onboard-agent`
3. enforce it in `close-proving-run`
4. persist immutable maintenance settlement in closeout evidence
5. add the new lifecycle evidence requirement
6. backfill `opencode` and `gemini_cli`
7. prove `aider` on the deferred branch
8. update normative docs and tests

Anything smaller leaves `closed_baseline` dishonest.

### Complexity, Completeness, And Distribution Checks

**Complexity smell**

This change necessarily touches more than eight files. That is acceptable because the contract
already spans approval parsing, onboarding materialization, lifecycle evidence, closeout,
historical backfill, committed governance artifacts, and docs. The guardrail is not "touch fewer
files." The guardrail is "do not add a new artifact version, a new lifecycle stage, a second
enrollment store, or a generic policy engine to paper over one in-flight agent."

**Completeness rule**

Do the full thing:

- approval truth
- registry materialization
- final closeout predicate
- immutable evidence
- historical backfill
- live proof
- docs
- tests

There is no honest half-version of this milestone.

**Distribution check**

No external artifact is introduced. Outputs remain repo-native:

- updated approval artifacts
- updated registry truth
- updated lifecycle and closeout governance artifacts
- updated normative docs
- updated tests

### Blocking Preconditions

1. Release-watch validation remains single-sourced. Approval-side validation must reuse the same
   rules as the registry block.
2. `artifact_version` stays `1`.
3. `closed_baseline` stays the final create-mode stage. No new lifecycle stage is introduced.
4. `prepare-agent-maintenance` behavior stays unchanged. Deferred agents remain ineligible.
5. The milestone treats `opencode`, `gemini_cli`, and `aider` as `explicitly_deferred`.
6. `aider` is already at `publication_ready`, so its approval continuity files must be updated
   atomically in this PR when the approval artifact SHA changes. Do not invent a new generic
   continuity-refresh command just for this.

## Final Contract To Land

### Approval Artifact Contract

Add one bounded maintenance table under `descriptor`:

```toml
[descriptor.maintenance]
mode = "release_watch_enrolled" # or "explicitly_deferred"

[descriptor.maintenance.release_watch]
enabled = true
version_policy = "latest_stable_minus_one"
dispatch_kind = "workflow_dispatch" # or "packet_pr"
dispatch_workflow = "example.yml"   # required only for workflow_dispatch

[descriptor.maintenance.release_watch.upstream]
source_kind = "github_releases"     # or "gcs_object_listing"
owner = "..."
repo = "..."
tag_prefix = "..."

[descriptor.maintenance.deferral]
reason = "..."
follow_up = "..."
approved_scope = "create_lane_closeout"
```

Rules:

- `mode = "release_watch_enrolled"` requires `release_watch` and forbids `deferral`
- `mode = "explicitly_deferred"` requires `deferral` and forbids `release_watch`
- `descriptor.maintenance.release_watch` must satisfy the exact same field rules as registry
  `maintenance.release_watch`
- `deferral.reason` and `deferral.follow_up` must be non-empty
- `approved_scope` is fixed to `create_lane_closeout`
- parser compatibility remains additive, but every approval artifact created or edited by this
  milestone must carry `descriptor.maintenance`

### Registry Materialization Invariant

- if approval mode is `release_watch_enrolled`, `onboard-agent --write` must materialize
  `[agents.maintenance.release_watch]` and that block must validate under the existing registry
  contract
- if approval mode is `explicitly_deferred`, `onboard-agent --write` must not write
  `maintenance.release_watch` for that agent
- deferral never creates a second inventory. It is closeout-policy evidence only

### Closeout Predicate

`close-proving-run` must enforce exactly one of these branches before `closed_baseline` is legal:

```text
PASS branch A: release_watch_enrolled
  approval.mode = release_watch_enrolled
  AND registry.maintenance.release_watch exists
  AND normalized(approval.release_watch) == normalized(registry.release_watch)

PASS branch B: explicitly_deferred
  approval.mode = explicitly_deferred
  AND registry.maintenance.release_watch is absent
  AND approval.deferral.reason is non-empty
  AND approval.deferral.follow_up is non-empty
  AND approval.deferral.approved_scope = create_lane_closeout

FAIL everything else
```

That includes:

- missing approval maintenance table
- enrolled approval with missing registry block
- deferred approval with any registry release-watch block
- normalized field mismatch in the enrolled branch
- malformed deferral payload in the deferred branch

### Machine-Owned Closeout Snapshot

The final closed `proving-run-closeout.json` must gain a machine-owned
`maintenance_settlement` object with this exact semantic payload:

```json
{
  "maintenance_settlement": {
    "mode": "release_watch_enrolled | explicitly_deferred",
    "approval_section_sha256": "<sha256 of descriptor.maintenance canonical form>",
    "normalized_release_watch": {
      "...": "present only for release_watch_enrolled"
    },
    "deferral": {
      "reason": "...",
      "follow_up": "...",
      "approved_scope": "create_lane_closeout"
    }
  }
}
```

Rules:

- `mode` is always present
- `approval_section_sha256` is always present
- `normalized_release_watch` is present only for `release_watch_enrolled`
- `deferral` is present only for `explicitly_deferred`
- `prepare-proving-run-closeout` may omit this field or leave it null in the prepared artifact
- `close-proving-run` owns the final value and overwrites any stale or hand-authored contents

### Lifecycle Rule

`crates/xtask/src/agent_lifecycle.rs` must add one new evidence id,
`maintenance_readiness_settled`, and require it for `ClosedBaseline`.

This milestone does not make `published` imply maintenance truth. The hard gate is final closeout.

### Deferred Steady-State Rule

Deferred agents may still reach `closed_baseline`, but they remain manual-maintenance agents until
a later enrollment change lands.

Explicitly:

- shared watcher fan-out is not expected
- `prepare-agent-maintenance` remains invalid
- `check-agent-drift` remains valid
- no maintenance request lane is supported while deferred
- if drift is detected while deferred, the legal next step is to update the approval artifact to
  `release_watch_enrolled`, materialize the matching registry block, and only then use the normal
  maintenance commands

### Historical Migration Policy

- `opencode` -> `explicitly_deferred`
- `gemini_cli` -> `explicitly_deferred`
- `aider` -> `explicitly_deferred` for this proof-run milestone

No registry enrollment is added for any of those three agents in this PR.

## Architecture Review

### Boundary Decision

The correct boundary is:

```text
approval artifact decides intent
        |
        v
onboard-agent materializes registry truth
        |
        v
close-proving-run proves approval truth == registry truth
        |
        v
closed_baseline records immutable settlement evidence
```

Do not move the decision into the watcher. Do not move enrollment out of the registry.

### Dependency Graph

```text
approval_artifact.rs
  ├── parses descriptor.maintenance
  └── reuses release_watch validator
          |
          v
onboard_agent/*
  ├── preview renders approved maintenance mode
  ├── draft carries maintenance decision
  └── write path materializes registry branch
          |
          v
agent_registry.toml
          |
          v
close_proving_run.rs
  ├── loads approval artifact
  ├── loads registry entry
  ├── evaluates exact two-branch predicate
  └── writes maintenance_settlement snapshot
          |
          v
proving_run_closeout.rs + agent_lifecycle.rs
  └── closed_baseline requires maintenance_readiness_settled
          |
          v
historical_lifecycle_backfill.rs
  └── replays the same truth onto already-closed agents
```

### File-Level Ownership

| Area | Files | Purpose |
| --- | --- | --- |
| Shared maintenance schema | `crates/xtask/src/agent_registry/release_watch.rs`, `crates/xtask/src/approval_artifact.rs` | Reuse one validator and parse approval-side maintenance truth. |
| Approval-mode onboarding | `crates/xtask/src/onboard_agent.rs`, `crates/xtask/src/onboard_agent/descriptor.rs`, `crates/xtask/src/onboard_agent/approval.rs`, `crates/xtask/src/onboard_agent/preview.rs` | Thread maintenance decision into draft state, preview output, registry text, and generated docs. |
| Lifecycle evidence | `crates/xtask/src/agent_lifecycle.rs` | Add `maintenance_readiness_settled` evidence and require it for `ClosedBaseline`. |
| Final closeout | `crates/xtask/src/close_proving_run.rs`, `crates/xtask/src/proving_run_closeout.rs` | Enforce predicate and snapshot immutable settlement evidence. |
| Historical repair | `crates/xtask/src/historical_lifecycle_backfill.rs` | Regenerate closed-baseline artifacts with settlement evidence for legacy agents. |
| Affected governance artifacts | `docs/agents/lifecycle/opencode-cli-onboarding/governance/approved-agent.toml`, `docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml`, `docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml` | Commit truthful maintenance decisions. |
| Normative docs | `docs/specs/agent-registry-contract.md`, `docs/specs/cli-agent-onboarding-charter.md`, `docs/cli-agent-onboarding-factory-operator-guide.md` | Make the contract and operator story truthful. |

### Production Failure Scenario Per New Path

| Codepath | Real failure | Planned handling |
| --- | --- | --- |
| Approval parsing | Approval artifact mixes `release_watch` and `deferral` | Validation error at load time. |
| Onboarding write | Deferred approval accidentally writes registry release-watch block | Integration test and write-path assertion. |
| Closeout compare | Approval says enrolled but registry block differs in one field | Closeout hard-fails with mismatch detail. |
| Closeout evidence | Prepared artifact carries stale hand-written settlement payload | `close-proving-run` overwrites machine-owned settlement fields. |
| Historical backfill | Legacy closed agent gets new evidence id but stale closeout JSON | Backfill rewrites closeout artifact and lifecycle evidence together. |
| `aider` proof | Approval SHA changes but lifecycle/publication continuity stays stale | Explicit continuity update in the same PR before proof commands run. |

## Implementation Plan

### Phase 1: Shared Contract Primitives

**Goal:** model the maintenance decision once and validate it once.

**Files**

- `crates/xtask/src/agent_registry/release_watch.rs`
- `crates/xtask/src/approval_artifact.rs`

**Concrete changes**

1. Promote the release-watch validation entrypoint so approval parsing can reuse it instead of
   copying field rules.
2. Extend `ApprovalDescriptor` with a maintenance decision model: mode enum, enrolled payload, and
   deferral payload.
3. Add one canonical normalization and hashing path for the approval-side maintenance section.
4. Keep parser compatibility for legacy artifacts, but make all forward-moving flows touched in
   this milestone reject missing maintenance truth where required.

**Verification**

- approval artifacts with either maintenance mode parse cleanly
- malformed mixed-mode artifacts fail with precise errors
- the release-watch field rules live in one place

**Exit criteria**

- no duplicated release-watch validation logic exists
- normalization and settlement hashing are defined once

### Phase 2: Approval-To-Registry Materialization

**Goal:** make `onboard-agent --write` carry maintenance truth into committed registry state.

**Files**

- `crates/xtask/src/onboard_agent.rs`
- `crates/xtask/src/onboard_agent/descriptor.rs`
- `crates/xtask/src/onboard_agent/approval.rs`
- `crates/xtask/src/onboard_agent/preview.rs`
- `crates/xtask/data/agent_registry.toml`

**Concrete changes**

1. Thread maintenance truth through `DraftDescriptorInput` and `DraftEntry`.
2. Render the registry preview and write path so `release_watch_enrolled` writes
   `[agents.maintenance.release_watch]`, while `explicitly_deferred` writes no registry
   release-watch block.
3. Surface the approved maintenance decision in dry-run output and generated packet previews so the
   operator story is explicit before runtime work begins.
4. Keep approval-mode `onboard-agent --write` idempotent and byte-stable on replay.

**Verification**

- approval-mode preview shows the maintenance decision
- approval-mode write produces the correct registry state for each branch
- replaying the same approval artifact is byte-stable

**Exit criteria**

- deferred approval cannot silently enroll
- enrolled approval cannot silently omit the registry block

### Phase 3: Closeout Predicate And Immutable Evidence

**Goal:** make `closed_baseline` impossible until maintenance settlement is proven.

**Files**

- `crates/xtask/src/close_proving_run.rs`
- `crates/xtask/src/proving_run_closeout.rs`
- `crates/xtask/src/agent_lifecycle.rs`
- `crates/xtask/src/prepare_proving_run_closeout.rs`

**Concrete changes**

1. Add `EvidenceId::MaintenanceReadinessSettled`.
2. Require it in `CLOSED_BASELINE_MINIMUM_EVIDENCE`.
3. Extend the closeout artifact with the machine-owned `maintenance_settlement` snapshot defined
   above.
4. Teach `close-proving-run` to:
   - load approval maintenance truth
   - load the registry entry
   - evaluate the exact two-branch predicate
   - fail closed on mismatch or absence
   - write the final immutable settlement snapshot into the closed closeout artifact
5. Keep `prepare-proving-run-closeout` boring. It prepares the draft; it does not decide
   maintenance truth.
6. Handle the in-flight `aider` continuity update explicitly in-repo:
   update the approval artifact, `lifecycle-state.json`, and `publication-ready.json` together in
   the same PR.

**Verification**

- `close-proving-run` rejects ambiguous or mismatched maintenance state
- `ClosedBaseline` requires the new evidence bit
- the closed closeout artifact carries immutable maintenance settlement proof

**Exit criteria**

- there is no legal `closed_baseline` path without `maintenance_readiness_settled`
- editing the approval artifact later cannot change what a previously closed run recorded

### Phase 4: Historical Backfill For Closed Agents

**Goal:** make already-closed agents truthful without widening rollout.

**Files**

- `crates/xtask/src/historical_lifecycle_backfill.rs`
- `docs/agents/lifecycle/opencode-cli-onboarding/governance/approved-agent.toml`
- `docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml`
- their matching committed closeout and lifecycle artifacts

**Concrete changes**

1. Extend `historical-lifecycle-backfill` so it can regenerate closed-baseline evidence with the
   new maintenance settlement snapshot.
2. Backfill `opencode` and `gemini_cli` approval artifacts with explicit deferral metadata.
3. Regenerate their closed-baseline closeout and lifecycle continuity so the new evidence bit is
   satisfied truthfully.

**Verification**

- both agents remain `closed_baseline`
- both approval artifacts state `explicitly_deferred`
- both closed-baseline artifacts carry immutable maintenance settlement evidence

**Exit criteria**

- historical truth matches current contract without adding registry enrollment

### Phase 5: Live Proof On `aider`

**Goal:** prove the tightened deferred branch on a real in-flight agent.

**Files**

- `docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml`
- `docs/agents/lifecycle/aider-onboarding/governance/lifecycle-state.json`
- `docs/agents/lifecycle/aider-onboarding/governance/publication-ready.json`
- the resulting `proving-run-closeout.json`

**Concrete changes**

1. Add `descriptor.maintenance` deferral metadata to `aider`’s approval artifact.
2. Update committed approval continuity files for the new approval SHA.
3. Run:
   - `cargo run -p xtask -- refresh-publication --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --write`
   - `cargo run -p xtask -- prepare-proving-run-closeout --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --write`
4. Complete bounded human fields in `proving-run-closeout.json`.
5. Run:
   - `cargo run -p xtask -- close-proving-run --approval docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml --closeout docs/agents/lifecycle/aider-onboarding/governance/proving-run-closeout.json`

**Verification**

- `aider` cannot close without explicit maintenance settlement
- `aider` can close once the deferred branch is satisfied
- `aider` remains unenrolled in the registry in this milestone

**Exit criteria**

- the real in-flight path proves the same rules the backfill path claims

### Phase 6: Docs And Final Verification

**Goal:** make the spec, runbook, and validation suite match the landed code.

**Files**

- `docs/specs/agent-registry-contract.md`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

**Concrete changes**

1. Document the new approval-side maintenance table and its two allowed modes.
2. State explicitly that registry omission remains the only unenrolled state.
3. Document the closeout predicate and the fact that deferred agents stay ineligible for
   `prepare-agent-maintenance`.
4. Document the one-time `aider` continuity migration as repo work, not an operator command.

**Final verification**

```sh
cargo test -p xtask recommend_next_agent_approval_artifact -- --nocapture
cargo test -p xtask onboard_agent_entrypoint -- approval_mode
cargo test -p xtask onboard_agent_closeout_preview -- close_proving_run_write
cargo test -p xtask historical_lifecycle_backfill_entrypoint -- --nocapture
cargo test -p xtask prepare_proving_run_closeout_entrypoint -- --nocapture
make preflight
```

**Exit criteria**

- normative docs and code tell the same story
- proof path and backfill path both pass

## Code Quality Review

### DRY Rules

1. Do not duplicate release-watch validation logic in `approval_artifact.rs`.
2. Do not add a second normalization or hashing path for maintenance settlement.
3. Do not add a generic new command for one in-flight `aider` continuity update.

### Explicit Over Clever

Use concrete structs and enums.

Do not hide the two-branch predicate behind a generic policy engine. Two branches. Named fields.
Readable in 30 seconds.

### Minimal-Diff Rules

Prefer:

- one approval-side maintenance struct family
- one new lifecycle evidence id
- one closeout snapshot object

Avoid:

- artifact version bump
- lifecycle stage expansion
- watcher refactor
- generic policy engine
- broader maintenance profile taxonomy

### Naming And Ownership Rules

- "enrollment" means registry `maintenance.release_watch` only
- "deferral" means approval-side closeout policy evidence only
- "settlement" means the final proved relationship between approval truth and registry truth
- machine-owned settlement fields live in closeout/lifecycle artifacts, not in human-edited packet prose

## Test Review

### Test Framework

Rust integration tests under `crates/xtask/tests/` remain the primary harness.

### Code Path Coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/xtask/src/approval_artifact.rs
    ├── [ADD] release_watch_enrolled parses and validates
    ├── [ADD] explicitly_deferred parses and validates
    ├── [ADD] mixed-mode artifact fails
    └── [ADD] missing maintenance table is rejected by forward-moving flows

[+] crates/xtask/src/onboard_agent/*
    ├── [ADD] approval-mode dry-run shows maintenance decision
    ├── [ADD] enrolled mode writes registry release_watch block
    ├── [ADD] deferred mode omits registry release_watch block
    └── [ADD] replay stays byte-identical

[+] crates/xtask/src/close_proving_run.rs
    ├── [ADD] enrolled branch succeeds on normalized match
    ├── [ADD] enrolled branch fails on normalized mismatch
    ├── [ADD] deferred branch succeeds on absent registry block
    ├── [ADD] deferred branch fails when registry block exists
    └── [ADD] closeout writes maintenance_settlement snapshot

[+] crates/xtask/src/historical_lifecycle_backfill.rs
    ├── [ADD] closed legacy agent receives maintenance settlement snapshot
    └── [ADD] lifecycle evidence includes maintenance_readiness_settled

[+] aider continuity migration
    ├── [ADD] lifecycle-state.json approval SHA continuity updates
    ├── [ADD] publication-ready.json approval SHA continuity updates
    └── [ADD] live proof path can continue to closeout without manual JSON surgery
```

### User / Operator Flow Coverage

```text
USER FLOW COVERAGE
===========================
[+] New approved agent with maintenance enrollment
    ├── [ADD] onboard-agent preview explains enrollment
    ├── [ADD] onboard-agent write materializes registry block
    └── [ADD] close-proving-run accepts enrolled match

[+] New approved agent with explicit deferral
    ├── [ADD] onboard-agent preview explains deferral
    ├── [ADD] registry remains unenrolled
    └── [ADD] close-proving-run accepts deferred branch only

[+] Already-closed agent backfill
    ├── [ADD] approval artifact updated with deferral metadata
    └── [ADD] backfill rewrites lifecycle + closeout evidence consistently

[+] In-flight agent at publication_ready
    ├── [ADD] approval continuity update stays coherent
    └── [ADD] closeout cannot finish until maintenance settlement is explicit
```

### Exact Test Files To Extend

- `crates/xtask/tests/recommend_next_agent_approval_artifact.rs`
- `crates/xtask/tests/onboard_agent_entrypoint/approval_mode.rs`
- `crates/xtask/tests/onboard_agent_closeout_preview/close_proving_run_write.rs`
- `crates/xtask/tests/historical_lifecycle_backfill_entrypoint.rs`
- `crates/xtask/tests/prepare_proving_run_closeout_entrypoint.rs`
- `crates/xtask/tests/agent_lifecycle_state.rs`

### Regression Rule

This is a regression if any path can still do one of these after the change:

1. reach `closed_baseline` with no maintenance decision
2. reach `closed_baseline` with approval truth that disagrees with registry truth
3. rewrite the meaning of a closed run by editing the approval artifact later

Every one of those cases gets a regression test. No exceptions.

## Performance Review

This is not a runtime-hot-path milestone. The performance risks are boring but real:

1. Do not repeatedly reparse or rehash the same approval and registry data inside one closeout run.
2. Keep normalization/comparison in memory. One registry load, one approval load, one normalized
   compare.
3. Do not add shell-outs or git invocations to the steady-state validation path. Historical
   backfill can afford them. Closeout should not.

## Failure Modes Registry

| Failure mode | Test required | Error handling | User-visible outcome |
| --- | --- | --- | --- |
| Approval has no maintenance table | Yes | validation error | explicit stop before write or closeout |
| Deferred approval writes registry release-watch anyway | Yes | write-path invariant + regression test | explicit failure, no silent enrollment |
| Enrolled approval and registry differ after normalization | Yes | closeout hard-fail | exact mismatch before `closed_baseline` |
| Closed historical agent lacks settlement evidence after backfill | Yes | backfill rewrites lifecycle + closeout together | repo does not ship half-migrated history |
| Prepared closeout carries stale machine-owned settlement fields | Yes | `close-proving-run` overwrites final snapshot | no silent reuse of stale truth |
| `aider` approval SHA changes but continuity files stay stale | Yes | proof blocked until continuity updated | explicit stop, not silent drift |

**Critical gap definition**

Any path that has:

- no maintenance settlement test
- no closeout guard
- and can still end in `closed_baseline`

is a critical gap. The milestone is not done until that count is zero.

## NOT In Scope

1. Enrolling `opencode`, `gemini_cli`, or `aider` into shared release-watch maintenance.
2. Changing `.github/workflows/agent-maintenance-release-watch.yml` or downstream maintenance
   worker topology.
3. Introducing a broader maintenance profile taxonomy beyond the two bounded modes above.
4. Bumping approval artifact version to `2`.
5. Adding a new lifecycle stage between `published` and `closed_baseline`.
6. Building a generic "approval continuity refresh" command. `aider` gets a one-time in-repo
   continuity migration instead.

## TODOS.md Impact

No new repo-wide TODO is required for this milestone. The follow-on work is already captured by the
existing `TODOS.md` item:

- `Make Goose The Explicit P1 End-To-End Lifecycle Validation`

This milestone is one of its blocking prerequisites because Goose cannot be the honest end-to-end
proof target until maintenance settlement semantics are truthful.

If implementation uncovers desire for wider release-watch rollout, that becomes a separate future
milestone, not an expansion of this PR.

## Worktree Parallelization Strategy

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| 1. Shared maintenance contract primitives | `crates/xtask/src/agent_registry/`, `crates/xtask/src/approval_artifact.rs` | — |
| 2. Approval/onboarding materialization | `crates/xtask/src/onboard_agent/`, `crates/xtask/data/` | Step 1 |
| 3. Closeout + lifecycle enforcement | `crates/xtask/src/close_proving_run.rs`, `crates/xtask/src/proving_run_closeout.rs`, `crates/xtask/src/agent_lifecycle.rs` | Step 1 |
| 4. Historical backfill | `crates/xtask/src/historical_lifecycle_backfill.rs`, `docs/agents/lifecycle/opencode-cli-onboarding/`, `docs/agents/lifecycle/gemini-cli-onboarding/` | Steps 1 and 3 |
| 5. Docs/spec truth update | `docs/specs/`, `docs/cli-agent-onboarding-factory-operator-guide.md` | Contract names frozen from Steps 1, 2, and 3 |
| 6. Aider live proof + continuity update | `docs/agents/lifecycle/aider-onboarding/` | Steps 2 and 3 |
| 7. Final verification | workspace-wide tests and `make preflight` | Steps 2, 3, 4, 5, and 6 |

### Parallel Lanes

- Lane A: Step 1 -> Step 2  
  Sequential because Step 2 depends on the shared contract model from Step 1.
- Lane B: Step 1 -> Step 3 -> Step 4  
  Sequential because backfill depends on the final closeout and lifecycle evidence model.
- Lane C: Step 5  
  Starts after exact contract names and closeout snapshot fields are frozen.
- Lane D: Step 6  
  Starts after Steps 2 and 3 merge because `aider` proof depends on both.

### Execution Order

1. Land Step 1 first. No parallelism before that.
2. Launch Lane A and Lane B in parallel worktrees after Step 1 merges cleanly.
3. Start Lane C once the contract field names are stable from Lanes A and B.
4. Start Lane D once Lanes A and B are merged, because `aider` proof needs the real code.
5. Merge all lanes.
6. Run the verification suite and the `aider` proof last.

### Conflict Flags

- Lanes A and B both depend on Step 1 and may share touch points in `approval_artifact.rs` if
  Step 1 is not isolated cleanly first. Do not start them before Step 1 merges.
- Lanes B and D both touch `docs/agents/lifecycle/aider-onboarding/` if backfill logic is made
  too generic. Keep historical backfill scoped to `opencode` and `gemini_cli`.
- Lane C should not start early if exact field names for `maintenance_settlement` are still moving.

## Completion Summary

- Step 0: scope accepted as-is, with rollout expansion explicitly deferred
- Architecture review: one intentional boundary, approval -> registry -> closeout -> evidence
- Code quality review: DRY enforced around one validator and one normalization path
- Test review: full new coverage required across approval, onboarding, closeout, backfill, and
  `aider` continuity
- Performance review: low-risk, but avoid repeated parse/hash work and new shell-outs
- NOT in scope: written
- What already exists: written
- TODOS.md impact: no new TODO, existing Goose follow-on remains the right next milestone
- Failure modes: all critical gaps must be driven to zero
- Parallelization: 4 lanes after one shared-contract foundation step
- Lake Score: choose the complete option on approval truth, closeout truth, historical truth, and
  live proof

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 0 | — | — |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | — | — |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 0 | — | plan hardened into an implementation contract, but no persisted review run yet |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | — | skipped, no UI scope in this milestone |

**VERDICT:** Cohesive implementation plan is ready, but no review pipeline metadata has been logged yet. Run `/autoplan` or the individual review skills if you want persisted review records.
