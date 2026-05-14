# PLAN - Converge CLI Agent Maintenance Onto One Shared Packet-PR Support-Uplift Lane

Status: ready for implementation  
Date: 2026-05-13  
Working branch: `staging`  
Plan revision baseline: `b5ba0d73`  
Primary design inputs:
- `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-design-20260513-112453.md`
- `docs/cli-agent-maintenance-steady-state-plan.md`

Ground-truth checkpoint:
- `docs/cli-agent-onboarding-factory-workflow-atlas.md` already exists on this branch
- `opencode` already proved the shared `packet_pr` opening path
- `codex` and `claude_code` still carry transitional `workflow_dispatch` maintenance transport

Supersedes:
- the prior repo-root `PLAN.md` for the `opencode` packet proof milestone

## Executive Summary

The docs now describe one maintenance factory.

That claim is only half true in code.

The shared watcher exists. The shared packet opener exists. The shared request contract exists. The
shared relay exists. The atlas exists. But the success semantics are still too timid, and the live
transport story is still split:

- `opencode` uses the intended `packet_pr` lane
- `codex` and `claude_code` still depend on worker-specific `workflow_dispatch` maintenance flows
- packet docs and prompts still describe maintenance like artifact refresh plus incidental wrapper
  edits instead of an explicit non-TUI support audit and bounded support uplift lane

This plan lands the actual steady state the docs are now pointing at:

1. one shared release watcher
2. one shared `packet_pr` PR-opening transport for enrolled automated maintenance
3. one shared prepared maintenance packet contract
4. one shared bounded relay executor
5. one explicit support-surface audit for non-TUI CLI coverage
6. one explicit manual closeout step

## Objective

Make the repository able to truthfully claim:

> enrolled automated CLI maintenance runs use one shared `packet_pr` transport and one shared
> relay contract, and those runs can detect and land missing non-TUI support surface as part of
> bounded maintenance success rather than treating runtime support as optional fallout or leaving
> deliberate non-TUI gaps in place forever.

## Success Criteria

1. `crates/xtask/data/agent_registry.toml` records `dispatch_kind = "packet_pr"` for every
   enrolled automated release-watch agent.
2. `.github/workflows/agent-maintenance-release-watch.yml` fans out only to the shared
   `agent-maintenance-open-pr.yml` PR opener for enrolled automated maintenance lanes.
3. `.github/workflows/codex-cli-update-snapshot.yml` and
   `.github/workflows/claude-code-update-snapshot.yml` are retired as steady-state maintenance
   transports.
4. `docs/specs/maintenance-request-contract-v1.md`,
   `docs/specs/agent-registry-contract.md`, and
   `docs/specs/cli-agent-onboarding-charter.md` tell one consistent story about transport,
   packet ownership, relay ownership, and manual closeout.
5. `prepare-agent-maintenance` writes automated packets whose docs and prompt explicitly require a
   non-TUI support-surface audit across commands, subcommands, flags, global flags, and
   positional args.
6. `execute-agent-maintenance` validates and enforces the shared relay contract without relying on
   agent-specific executor semantics or worker-YAML-only policy.
7. Packet-owned docs stop describing success as "refresh artifacts and maybe touch code." They
   describe bounded support uplift when missing non-TUI surface is discovered.
8. The automated packet contract explicitly records the discovered upstream surface delta, the
   current wrapper/backend/manifest support delta, and the exact required non-TUI uplifts needed
   for the run to count as successful.
9. Existing deliberate non-TUI unsupported posture is burned down over time rather than carried as
   a permanent note. This includes the current `opencode` v1 restriction posture and the current
   "intentionally unsupported surface outside unified support" notes for other CLI agents.
10. A newly onboarded CLI agent is allowed to start at partial non-TUI support, but enrolled
    maintenance must ratchet that support upward over time whenever new upstream surface appears or
    previously known gaps become implementable within the bounded write envelope.
11. Support publication surfaces stop treating deliberate non-TUI omission as normal steady state.
    Any remaining unsupported non-TUI surface must be either:
    - TUI-only and explicitly excluded, or
    - blocked by a concrete upstream/platform limitation recorded in packet-owned truth.
12. Every automated maintenance packet distinguishes between:
    - `required_uplifts_this_run` for newly discovered or eligible preexisting gaps that must land
    - `deferred_preexisting_gaps` for older gaps that are explicitly out of scope for this run
13. A run cannot claim success while leaving unresolved:
    - newly discovered non-TUI surface, or
    - any preexisting gap that the packet marked as in-scope for this run
14. Shared regression coverage proves packet generation, relay validation, watcher dispatch, and
   transport convergence for `codex`, `claude_code`, and `opencode`.
15. At least one previously worker-backed agent, `codex`, completes a post-migration proving run on
   the shared lane or a full repo-owned dry-run plus write-mode simulation that exercises the same
   packet and gate contract end to end.
16. `docs/cli-agent-onboarding-factory-operator-guide.md` and
    `docs/cli-agent-onboarding-factory-workflow-atlas.md` remain explanatory surfaces, not shadow
    normative stores.

## Locked Decisions

1. The workflow atlas is already landed groundwork, not the milestone itself.
2. `refresh-agent` remains the manual drift and publication-refresh seam. It is not the automated
   upstream-release relay executor.
3. `execute-agent-maintenance` remains the shared local relay surface.
4. `close-agent-maintenance` remains explicit and manual.
5. Automated maintenance stays bounded to non-TUI support surface plus matching manifest,
   wrapper, backend, and packet-owned doc updates.
6. TUI parity is out of scope for this milestone.
7. Workflow YAML owns transport only. It must not become a second policy store for support audit,
   writable surfaces, or green gates.
8. The first required post-migration proving example is `codex`. `claude_code` must reach full
   transport and contract parity in code and tests in the same milestone, but it does not need to
   be the first live proving lane.
9. Deliberately unsupported non-TUI surface is not an acceptable steady-state product posture.
   It may exist temporarily for a newly onboarded agent, but maintenance must treat it as backlog
   to burn down, not as a permanent caveat.
10. `opencode` does not get a special carveout. Its current v1 restrictions must be removed on the
    same policy basis as every other CLI agent.

## Step 0 Scope Challenge

### Premise Challenge

| Premise | Assessment | Decision |
| --- | --- | --- |
| The missing milestone is still "write the workflow atlas." | Rejected. The atlas already exists and is useful, but it exposes the implementation gap instead of closing it. | Treat the atlas as shipped groundwork and focus this plan on contract and transport convergence. |
| Worker-specific maintenance workflows are a permanent architecture feature. | Rejected. The shared watcher plus `opencode` proof show they are transition scaffolding, not the destination. | Converge enrolled automated maintenance onto shared `packet_pr`. |
| `requested_control_plane_actions = ["packet_doc_refresh"]` is enough to describe success. | Rejected. That explains packet refresh, not support uplift. | Keep the control-plane list narrow, but make support audit and uplift explicit in the relay contract and packet docs. |
| Runtime support uplift can stay implicit inside prompts. | Rejected. Hidden policy is exactly how this repo drifts. | Move the support-audit requirement into the normative contract, packet rendering, and relay validation. |
| Existing "deliberately unsupported" non-TUI posture can remain as a permanent published caveat. | Rejected. That turns maintenance into a reporter of gaps instead of a closer of gaps. | Treat non-TUI unsupported posture as temporary debt that maintenance must ratchet downward. |
| Newly onboarded agents must start fully supported before they can be enrolled. | Rejected. That is too rigid and slows onboarding for the wrong reason. | Allow an initial lower support floor, but require enrolled maintenance to raise support over time. |
| We need another workflow family to replace the old worker flows. | Rejected. The repo already has the shared watcher and shared PR opener. | Reuse the existing watcher and `agent-maintenance-open-pr.yml`. |
| This milestone should include broad TUI parity. | Rejected. That is a second project and will blow up the write envelope. | Limit uplift to non-TUI commands, subcommands, flags, globals, and positional args. |
| One real `codex` proof with shared tests is too small to validate the architecture. | Rejected. The architecture is shared; one migrated live example plus strong shared tests is enough for this lake. | Require one migrated proof run, `codex`, and full shared regression coverage. |

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| Shared release detection | `.github/workflows/agent-maintenance-release-watch.yml`, `crates/xtask/src/agent_maintenance/watch.rs` | Reuse. Keep one watcher. |
| Shared PR opening | `.github/workflows/agent-maintenance-open-pr.yml` | Reuse. Promote to default enrolled transport. |
| Shared packet generation | `crates/xtask/src/agent_maintenance/prepare.rs` | Reuse and widen. Do not create a second packet generator. |
| Shared relay execution | `crates/xtask/src/agent_maintenance/execute/**` | Reuse and harden. Keep bounded writes and explicit gates. |
| Shared contract-policy derivation | `crates/xtask/src/agent_maintenance/contract_policy.rs` | Reuse. This is where transport and packet defaults should stay centralized. |
| Packet-owned docs renderer | `crates/xtask/src/agent_maintenance/docs.rs` | Reuse and rewrite wording so success semantics are truthful. |
| Visual system map | `docs/cli-agent-onboarding-factory-workflow-atlas.md` | Reuse. Keep it aligned with the normative contracts. |
| Existing packet-pr proof | `docs/agents/lifecycle/opencode-maintenance/governance/proof/**` | Reuse as the proof that the lane shape works. Do not repeat the `opencode` milestone. |
| Registry-driven dispatch metadata | `crates/xtask/data/agent_registry.toml`, `crates/xtask/src/agent_registry/release_watch.rs` | Reuse. Registry stays the only enrollment truth. |
| Worker-specific transport leftovers | `.github/workflows/codex-cli-update-snapshot.yml`, `.github/workflows/claude-code-update-snapshot.yml` | Retire as steady-state transport after shared-lane parity is green. |

### TODOS Cross-Reference

`TODOS.md` already carries:

- `Decide Whether Maintenance Transport Topology Still Needs Convergence After Worker/Runbook Cleanup`

This plan consumes that TODO directly. Landing this plan should retire or rewrite that item, not
leave it hanging as a duplicate reminder.

The other current TODOs are unrelated and do not block this milestone.

### NOT In Scope

1. Adding TUI parity through automated maintenance.
2. Inventing a second packet schema for migrated agents.
3. Replacing the shared watcher.
4. Automating `close-agent-maintenance`.
5. Merging manual drift maintenance and automated upstream-release maintenance into one command.
6. Reworking onboarding or publication lifecycle stages outside the maintenance blast radius.
7. Proving a live post-migration run for every enrolled agent in the same PR.
8. Forcing brand-new onboarded agents to launch at full parity on day one.

### Minimum Complete Change

The minimum complete implementation is:

1. freeze one truthful steady-state contract across registry spec, maintenance spec, and charter
2. rewrite packet-owned docs and prompt semantics so automated maintenance explicitly audits
   non-TUI support surface
3. teach the relay and validation layers to enforce the same support-aware contract
4. migrate `codex` and `claude_code` registry truth and watcher fanout to shared `packet_pr`
5. retire worker-specific steady-state maintenance transport
6. prove the migrated lane through shared regression coverage and one post-migration `codex` proof

Anything smaller still leaves the docs telling a story the code does not actually implement.

### Complexity, Search, Completeness, And Distribution Checks

**Complexity smell**

This plan will touch more than eight files and more than two modules.

That is acceptable because the blast radius is already tightly clustered:

- `docs/specs/**`
- `.github/workflows/**`
- `crates/xtask/src/agent_maintenance/**`
- `crates/xtask/src/agent_registry/**`
- `crates/xtask/tests/**`
- packet-owned lifecycle docs under `docs/agents/lifecycle/*-maintenance/**`

The right scope reduction is not "do less." It is "do not spill outside the maintenance factory."

**Search check**

- **[Layer 1]** Reuse the shared watcher and shared PR opener. They already exist.
- **[Layer 1]** Reuse the registry as the only release-watch enrollment store.
- **[Layer 1]** Reuse the shared relay executor and packet renderer.
- **[Layer 1]** Reuse packet-owned `HANDOFF.md`, `CI_WORKFLOWS_PLAN.md`, and `OPS_PLAYBOOK.md`
  surfaces instead of inventing another operator note layer.
- **[EUREKA]** The real bug is not transport fanout anymore. It is policy split. YAML, packet
  docs, prompt wording, and relay semantics still disagree about what "maintenance success" means.
  Fix that split and the architecture gets boring again.

**Completeness rule**

Do the whole lake:

- contract alignment
- packet wording alignment
- relay enforcement alignment
- registry and watcher convergence
- worker transport retirement
- shared regression coverage
- one migrated proof run

Do not stop at "the docs look clearer."

**Distribution check**

There is no new binary or package artifact here.

The shipped outputs are:

- one truthful steady-state maintenance contract
- one shared packet-pr transport story for enrolled agents
- one migrated proving example
- one docs surface that no longer lies

## Architecture Contract

### Current-State Versus Target-State Boundary

```text
CURRENT
=======
shared watcher
  -> packet_pr for opencode
  -> workflow_dispatch workers for codex and claude_code
  -> packet docs say packet refresh first
  -> relay exists, but success semantics are underspecified

TARGET
======
shared watcher
  -> shared packet_pr opener for every enrolled automated lane
  -> shared prepared packet
  -> shared relay
  -> explicit non-TUI support audit
  -> bounded support uplift
  -> ratchet down previously known unsupported non-TUI surface
  -> explicit manual closeout
```

### Steady-State Data Flow

```text
agent_registry.toml
        |
        v
agent-maintenance-release-watch.yml
        |
        v
stale_agents[] queue
        |
        v
agent-maintenance-open-pr.yml
        |
        v
prepare-agent-maintenance
        |
        v
maintenance-request.toml
HANDOFF.md
execute-agent-maintenance-prompt.md
pr-summary.md
        |
        v
execute-agent-maintenance --dry-run
        |
        v
support-surface audit
  - commands
  - subcommands
  - flags
  - global flags
  - positional args
  - exclude TUI-only surface
  - include already-known non-TUI unsupported surface
        |
        v
execute-agent-maintenance --write
        |
        v
bounded wrapper/backend/manifest/doc updates
        |
        v
green gates
        |
        v
close-agent-maintenance
```

### Ownership Split

| Layer | Owns | Must not own |
| --- | --- | --- |
| Registry | enrollment truth, upstream metadata, dispatch mode | packet policy, writable surfaces, gate lists |
| Shared watcher | stale detection and queue generation | agent-specific maintenance semantics |
| Shared PR opener | packet generation and PR creation | artifact-refresh policy, support-audit policy, runtime gate policy |
| Prepared packet | frozen request truth, writable surfaces, prompt digest, gates, recovery | hidden agent-specific behavior outside the request |
| Relay | validation, bounded writes, support-aware execution, green gates | transport policy, closeout automation |
| Manual closeout | explicit maintainer settlement | write-mode execution |

### Support Maturation Contract

Automated maintenance needs one explicit ratchet rule:

- newly onboarded agents may start with partial non-TUI support
- once an agent is enrolled in automated release-watch maintenance, non-TUI support should trend
  upward, not sideways
- already-known deliberate unsupported non-TUI gaps are backlog to close when bounded uplift is
  feasible
- newly discovered upstream non-TUI surface must be compared against current wrapper coverage,
  backend support, and published support posture during packet preparation
- TUI-only surface stays excluded by contract and does not count against this ratchet

Eligibility rule for preexisting gaps:

- a preexisting non-TUI gap is in-scope for the current maintenance run only when at least one of
  these is true:
  - the target upstream release introduced or changed adjacent surface in the same command family
  - the gap can be closed entirely inside the current packet `writable_surfaces`
  - the gap requires no new infra, no new transport, and no new cross-cutting abstraction
- otherwise, the gap must not be silently pulled into the run
- instead, it must be recorded as explicit deferred work with a concrete blocker or a separately
  planned seam

Steady-state rule:

```text
new upstream non-TUI surface discovered
        OR
known non-TUI unsupported surface still present
        |
        v
prepare packet records exact delta
        |
        v
relay either:
  - lands bounded uplift, or
  - fails/defer-with-rationale when blocked by a concrete upstream/platform limit
```

### Required Packet Audit Shape

The refined steady-state packet needs an explicit machine-readable audit block. The exact TOML
syntax can still be chosen during implementation, but the packet must carry these concepts:

| Field | Purpose |
| --- | --- |
| `support_surface_audit.required` | turns the support audit into a first-class gate, not prose |
| `support_surface_audit.surface_kinds[]` | `commands`, `subcommands`, `flags`, `global_flags`, `positional_args` |
| `support_surface_audit.excluded_surface_kinds[]` | explicit exclusions, initially TUI-only only |
| `support_surface_audit.discovered_upstream_surface[]` | newly observed non-TUI surface from the target version |
| `support_surface_audit.preexisting_unsupported_surface[]` | already-known non-TUI gaps that remain open on the current validated baseline |
| `support_surface_audit.eligible_preexisting_surface[]` | subset of preexisting gaps that are justified and bounded for this run |
| `support_surface_audit.missing_wrapper_support[]` | surface present upstream but absent from wrapper support |
| `support_surface_audit.missing_backend_support[]` | surface present in wrapper/manifests but absent from backend/UAA support |
| `support_surface_audit.required_uplifts_this_run[]` | exact bounded changes required for this run to count as success |
| `support_surface_audit.deferred_preexisting_gaps[]` | older gaps explicitly left out of this run, each with a concrete reason |
| `support_surface_audit.allowed_deferrals[]` | only concrete upstream/platform blockers, never vague “deliberately unsupported” posture |
| `support_surface_audit.defer_reason` | machine-readable reason per deferred gap or uplift blocker |
| `support_surface_audit.publication_impacts[]` | support-matrix or capability-publication rows/notes that must change when uplift lands |

If this block is absent, the packet is not a valid steady-state automated maintenance packet.

Allowed blocker taxonomy:

- `upstream_not_machine_exposed`
- `platform_evidence_missing`
- `requires_new_infra`
- `requires_new_architectural_seam`
- `outside_current_writable_surfaces`

Not valid as blocker reasons:

- `deliberately_unsupported`
- `too_much_work_right_now`
- `not_part_of_v1`

### Module Seams

| Module | Responsibility in this milestone |
| --- | --- |
| `crates/xtask/src/agent_maintenance/contract_policy.rs` | derive shared transport, prompt, recovery, and support-audit defaults |
| `crates/xtask/src/agent_maintenance/prepare.rs` | emit packet truth that matches the steady-state contract |
| `crates/xtask/src/agent_maintenance/docs.rs` | render packet docs and prompt with truthful support-uplift semantics |
| `crates/xtask/src/agent_maintenance/execute/**` | enforce packet continuity, support-aware execution, and bounded writes |
| `crates/xtask/src/agent_registry/release_watch.rs` | treat `packet_pr` as normal enrolled truth, not a special case |
| `.github/workflows/*.yml` | transport only, no shadow policy |
| `docs/specs/**` | normative truth only, no procedural drift |

## Code Quality Contract

1. No second policy store in workflow YAML. If a rule matters, it lives in `docs/specs/**` and in
   shared `xtask` code.
2. No agent-specific executor naming in new code. New automated packets use
   `execution_contract.executor = "execute-agent-maintenance"`.
3. No duplicate support-audit logic scattered across packet rendering, relay validation, and tests.
   Derive it once, thread it everywhere.
4. No hidden "best effort" success path. If support uplift is expected and not performed, the lane
   must fail or explicitly defer with packet-owned truth.
5. No new workflow family. Reuse the shared opener.
6. Update nearby ASCII diagrams and generated packet docs when the contract changes. Stale diagrams
   are bugs.
7. No published “intentionally unsupported” note for non-TUI surface unless it is backed by a
   concrete blocker recorded in the packet or support publication truth.

## Failure Modes Registry

| Codepath | Real failure | Test coverage required | Planned handling | Maintainer signal | Critical gap |
| --- | --- | --- | --- | --- | --- |
| Watcher dispatch resolution | watcher still emits worker workflows for migrated agents | `agent_maintenance_watch.rs`, `agent_registry.rs` | fail tests, block merge | explicit test failure | Yes |
| Packet generation | request still encodes artifact-refresh-only success wording | `agent_maintenance_prepare.rs`, doc-render tests | rewrite packet and prompt rendering | explicit diff and renderer test failure | Yes |
| Relay validation | write mode succeeds without an explicit support audit | `agent_maintenance_execute.rs` | add invariant, reject packet or run | explicit CLI validation failure | Yes |
| Support publication truth | support matrix still carries deliberate non-TUI unsupported notes after uplift should have landed | publication checks and packet proof review | rewrite publication surfaces and fail if stale | explicit support-matrix mismatch | Yes |
| Worker retirement | stale workflow still acts like source of truth | workflow wiring tests, spec CI wiring | delete or demote workflow, update references | explicit workflow or docs mismatch | No |
| Support uplift boundary | relay widens into TUI or unrelated runtime work | relay tests plus proof run diff review | bound writes to non-TUI surfaces only | explicit diff escape or validation failure | Yes |
| Support ratchet failure | newly discovered or preexisting non-TUI gaps remain open with no blocker and the run still claims success | execute tests, publication tests, proof review | fail closed until uplift lands or blocker is recorded | explicit packet audit failure | Yes |
| Backlog bucket creep | packet drags unrelated historic gaps into the run without eligibility proof | prepare/execute tests, packet proof review | require packet-scoped eligibility or explicit deferral | explicit packet audit failure | Yes |
| Docs drift | operator guide, atlas, spec, and packet docs disagree again | doc snapshot tests plus review diff | update all truth surfaces in same PR | review-time mismatch | No |
| Proving run | migrated `codex` lane cannot open or validate from shared packet_pr flow | live proof or full end-to-end simulation | fix within blast radius, rerun proof | explicit run failure | Yes |

## Test Review

### Test Framework And Commands

This is a Rust workspace.

Primary verification surfaces:

- `cargo test -p xtask`
- targeted suites under `crates/xtask/tests/agent_maintenance_*`
- `cargo test -p xtask agent_registry -- --nocapture`
- `cargo test -p xtask c4_spec_ci_wiring -- --nocapture`
- final `make preflight`

### Code Path Coverage

```text
CODE PATH COVERAGE
===========================
[+] watcher + registry dispatch
    │
    ├── [★★★ TESTED] workflow_dispatch entries stay valid while transitional
    ├── [GAP]         migrated codex resolves to packet_pr only
    ├── [GAP]         migrated claude_code resolves to packet_pr only
    └── [★★★ TESTED] opencode packet_pr dispatch stays valid

[+] prepare-agent-maintenance
    │
    ├── [★★★ TESTED] shared automated packet envelope renders
    ├── [★★★ TESTED] packet_pr dispatch_workflow materializes shared opener
    ├── [GAP]         support-audit language is explicit in request-owned docs
    ├── [GAP]         packet records discovered surface, eligible preexisting surface, deferred preexisting surface, and required uplifts for this run
    └── [GAP]         recreate/recovery guidance stops pointing at refresh-agent as the main story

[+] packet-owned docs
    │
    ├── [★★★ TESTED] HANDOFF/pr-summary/prompt render from request
    ├── [GAP]         prompt requires non-TUI support audit before success
    ├── [GAP]         HANDOFF describes support uplift as bounded relay work
    ├── [GAP]         CI_WORKFLOWS_PLAN/OPS_PLAYBOOK match the same contract
    └── [GAP]         opencode/other agent docs stop normalizing “deliberately unsupported” non-TUI posture

[+] execute-agent-maintenance
    │
    ├── [★★★ TESTED] packet continuity checks
    ├── [★★★ TESTED] writable-surface jail
    ├── [★★★ TESTED] green-gate execution
    ├── [GAP]         packet without support-audit truth is rejected or fails closed
    ├── [GAP]         packet with unresolved non-TUI uplift and no blocker cannot claim success
    ├── [GAP]         packet cannot silently absorb unrelated historic gaps with no eligibility proof
    └── [GAP]         migrated codex packet succeeds through shared relay path

[+] support publication truth
    │
    ├── [GAP]         support matrix stops carrying stale “intentionally unsupported” notes for non-TUI surface
    ├── [GAP]         opencode v1 restriction posture is removed from steady-state support publication
    └── [GAP]         newly onboarded agents can start partial, but later maintenance ratchets support upward

[+] workflow transport
    │
    ├── [★★★ TESTED] shared packet_pr opener exists
    ├── [GAP]         release watch fans out only to shared opener for enrolled agents
    └── [GAP] [→E2E] migrated codex PR-open path from watcher queue to packet render

─────────────────────────────────
COVERAGE: core relay and packet plumbing exist, steady-state semantics still incomplete
GAPS: 13 concrete contract, publication, and migration gaps
REGRESSION RULE: any migrated-lane failure becomes a required regression test before merge
─────────────────────────────────
```

### Test Requirements

1. Expand `crates/xtask/tests/agent_maintenance_watch.rs` and `agent_registry.rs` to assert that
   migrated `codex` and `claude_code` now resolve to `packet_pr` and the shared opener.
2. Expand `crates/xtask/tests/agent_maintenance_prepare.rs` and renderer tests to assert that the
   packet-owned prompt, `HANDOFF.md`, and `pr-summary.md` explicitly require support audit and
   bounded support uplift.
3. Expand `crates/xtask/tests/agent_maintenance_execute.rs` to fail closed when the support-audit
   contract is missing or inconsistent.
4. Expand support publication checks so stale “deliberately unsupported” non-TUI notes fail once
   uplift should have landed.
5. Add explicit regression coverage for the ratchet rule:
   a packet with preexisting non-TUI gaps and no blocker cannot succeed unchanged.
6. Add explicit regression coverage for the eligibility rule:
   a packet cannot pull unrelated preexisting gaps into `required_uplifts_this_run[]` without
   adjacent surface change, bounded writable surfaces, or a no-new-seam proof.
7. Add at least one focused test covering `opencode` restriction removal in support publication
   truth so it does not regress back to a carved-out posture.
8. Add explicit regression coverage for blocker taxonomy:
   invalid reasons like `deliberately_unsupported` or `not_part_of_v1` fail packet validation.
9. Keep `opencode` packet-pr coverage green while migrating the other agents. Do not regress the
   already-proved lane.
10. Prove one migrated `codex` path end to end, either as a real maintenance run or a full
   repo-owned simulation that uses the actual shared packet-pr transport and relay contract.
11. Finish with `make preflight`.

## Performance Review

This milestone is policy-heavy, not throughput-heavy, but there are still a few real performance
and operability constraints:

1. Do not duplicate artifact acquisition or packet rendering work across multiple workflows when
   one shared opener and one shared relay path can do it once.
2. Keep watcher fanout data-driven from registry truth so adding future agents does not turn into
   O(number of workflows) maintenance edits.
3. Keep relay validation deterministic and cheap. Digest checks, surface derivation, and support
   comparison must stay local and bounded.
4. Prefer targeted xtask tests while iterating, then one final `make preflight`. The full suite is
   expensive enough already.

## Implementation Plan

### Phase 1: Freeze The Normative Steady-State Contract

**Goal**

Make the specs and charter say exactly the same thing about automated maintenance.

**Primary files**

- `docs/specs/maintenance-request-contract-v1.md`
- `docs/specs/agent-registry-contract.md`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/specs/unified-agent-api/support-matrix.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/cli-agent-onboarding-factory-workflow-atlas.md`

**Actions**

1. Freeze one definition of automated maintenance success:
   support audit first, bounded non-TUI uplift second, green gates third, manual closeout last.
2. Make the registry spec explicit that enrolled `packet_pr` dispatch is the steady state and
   `workflow_dispatch` for `codex` and `claude_code` is transitional, not schema doctrine.
3. Keep `requested_control_plane_actions = ["packet_doc_refresh"]` narrow, but define the support
   uplift obligation in the execution contract and supporting prose.
4. Define the ratchet rule in normative prose:
   newly onboarded agents may begin partial, but enrolled automated maintenance must raise non-TUI
   support over time and must not preserve deliberate unsupported posture as steady state.
5. Align support publication rules so only TUI exclusions and concrete blockers can justify
   remaining non-TUI unsupported surface.
6. Define the eligibility rule for preexisting gaps:
   only packet-bounded, justified older gaps become `required_uplifts_this_run`; everything else
   becomes explicit deferred work.
7. Align the operator guide and atlas so they both point back to the same normative contract and
   no longer imply artifact-refresh-only semantics.

**Outputs**

- one normative story
- one operator story
- one atlas story

**Exit criteria**

- a reader can move from spec to guide to atlas without seeing contradictory success semantics

### Phase 2: Rewrite Packet Generation And Packet-Owned Docs Around Support Audit

**Goal**

Make generated maintenance packets describe the real job.

**Primary files**

- `crates/xtask/src/agent_maintenance/contract_policy.rs`
- `crates/xtask/src/agent_maintenance/prepare.rs`
- `crates/xtask/src/agent_maintenance/docs.rs`
- `crates/xtask/src/agent_maintenance/request.rs`
- `crates/xtask/src/agent_maintenance/request/automation.rs`
- `cli_manifests/support_matrix/current.json`
- `docs/agents/lifecycle/*-maintenance/CI_WORKFLOWS_PLAN.md`
- `docs/agents/lifecycle/*-maintenance/OPS_PLAYBOOK.md`

**Actions**

1. Centralize the support-audit contract in shared derivation code, not per-agent text.
2. Rewrite prompt rendering so support audit is explicit:
   compare upstream surface against wrapper coverage, backend support, and manifest truth.
3. Rewrite `HANDOFF.md` and `pr-summary.md` so the relay job is described as bounded support-aware
   maintenance, not packet refresh plus incidental edits.
4. Make the packet carry explicit deltas:
   discovered upstream surface, eligible preexisting surface, deferred preexisting surface,
   missing wrapper/backend support, required uplifts for this run, and allowed blocker-based
   deferrals.
5. Update packet-owned playbooks so `codex`, `claude_code`, and `opencode` all read like the same
   factory with narrow per-agent value differences.

**Outputs**

- regenerated maintenance packet docs with truthful wording
- support-aware prompt template semantics

**Exit criteria**

- packet docs alone are sufficient to explain the real maintenance job without reading workflow YAML

### Phase 3: Enforce Support-Aware Relay Semantics

**Goal**

Make `execute-agent-maintenance` fail closed when the support-aware contract is missing or broken.

**Primary files**

- `crates/xtask/src/agent_maintenance/execute.rs`
- `crates/xtask/src/agent_maintenance/execute/runtime.rs`
- `crates/xtask/src/agent_maintenance/execute/validate.rs`
- `crates/xtask/src/agent_maintenance/execute/workflow.rs`
- `crates/xtask/src/agent_maintenance/refresh.rs`

**Actions**

1. Add explicit validation that the prepared request and rendered docs carry the support-audit
   contract expected by the steady-state packet.
2. Ensure relay recovery guidance points back to shared packet regeneration and shared relay use,
   not back to the old worker-centered worldview.
3. Keep the write envelope tight:
   wrapper crate, backend module, manifest root, packet-owned docs, no TUI spillover.
4. Fail closed when a run leaves preexisting or newly discovered non-TUI gaps unresolved without a
   concrete blocker recorded in packet truth.
5. Fail closed when a packet marks preexisting gaps as in-scope without satisfying the eligibility
   rule for this run.
6. Keep closeout manual and untouched by write mode.

**Outputs**

- support-aware relay invariants
- shared failure-closed behavior

**Exit criteria**

- a packet that describes artifact refresh only cannot pass as a valid steady-state automated run

### Phase 4: Converge `codex` And `claude_code` Transport Onto Shared `packet_pr`

**Goal**

Make the live enrolled transport story match the docs.

**Primary files**

- `crates/xtask/data/agent_registry.toml`
- `crates/xtask/src/agent_registry/release_watch.rs`
- `crates/xtask/src/agent_maintenance/watch.rs`
- `.github/workflows/agent-maintenance-release-watch.yml`
- `.github/workflows/agent-maintenance-open-pr.yml`
- `.github/workflows/codex-cli-update-snapshot.yml`
- `.github/workflows/claude-code-update-snapshot.yml`

**Actions**

1. Migrate `codex` and `claude_code` registry truth from `workflow_dispatch` to `packet_pr`.
2. Make the shared watcher emit the shared opener for all enrolled agents.
3. Remove or clearly retire worker-specific workflow dispatch as steady-state transport.
4. Keep any agent-specific acquisition detail outside the policy layer. If the opener needs more
   inputs, derive them from registry truth and the packet, not from hidden YAML policy.

**Outputs**

- one shared enrolled transport story
- no worker-specific steady-state transport dependency

**Exit criteria**

- the registry, watcher, and workflows all agree that enrolled automated maintenance opens via
  `agent-maintenance-open-pr.yml`

### Phase 5: Refresh Historical Maintenance Packet Surfaces

**Goal**

Bring generated and committed lifecycle maintenance docs back into alignment after the contract
shift.

**Primary files**

- `docs/agents/lifecycle/codex-maintenance/**`
- `docs/agents/lifecycle/opencode-maintenance/**`
- `docs/agents/lifecycle/<agent_id>-maintenance/**` for any migrated packet surfaces that still
  need to be materialized or refreshed
- `docs/specs/unified-agent-api/support-matrix.md`
- `cli_manifests/support_matrix/current.json`

**Actions**

1. Regenerate packet-owned docs for migrated agents so they describe the same relay job.
2. Keep `opencode` truthful to the already-proved `packet_pr` lane while removing its v1
   deliberate unsupported posture from the steady-state story.
3. Update support publication surfaces so they stop normalizing deliberate non-TUI unsupported
   posture for any enrolled CLI agent.
4. Update packet-owned playbooks and workflow plans that still talk like worker transport is the
   normal path.

**Outputs**

- committed lifecycle maintenance docs that match the migrated contract

**Exit criteria**

- there is no packet-owned maintenance doc left that treats worker transport as the steady state

### Phase 6: Land Regression Coverage And One Migrated Proof

**Goal**

Prove that the new story works for a formerly worker-backed agent.

**Primary files**

- `crates/xtask/tests/agent_maintenance_prepare.rs`
- `crates/xtask/tests/agent_maintenance_execute.rs`
- `crates/xtask/tests/agent_maintenance_watch.rs`
- `crates/xtask/tests/agent_registry.rs`
- `crates/xtask/tests/c4_spec_ci_wiring.rs`
- migrated `codex` maintenance packet/proof surfaces as needed

**Actions**

1. Add regression coverage for transport convergence, support-aware packet rendering, and
   support-aware relay validation.
2. Run targeted xtask suites until green.
3. Prepare and validate one migrated `codex` maintenance lane through the shared `packet_pr`
   contract.
4. If the proof exposes a missing invariant, add the test first, then rerun the proof.
5. Finish with `make preflight`.

**Outputs**

- shared regression protection
- one migrated proof example

**Exit criteria**

- the repo can point to one previously worker-backed agent and show that the shared lane is real

## Worktree Parallelization Strategy

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| 1. Normative contract freeze | `docs/specs/`, `docs/` | — |
| 2. Packet rendering rewrite | `crates/xtask/src/agent_maintenance/`, `docs/agents/lifecycle/*-maintenance/` | 1 |
| 3. Relay enforcement | `crates/xtask/src/agent_maintenance/execute/`, `crates/xtask/src/agent_maintenance/` | 1 |
| 4. Transport convergence | `crates/xtask/data/`, `crates/xtask/src/agent_registry/`, `.github/workflows/` | 1 |
| 5. Regression coverage | `crates/xtask/tests/` | 2, 3, 4 |
| 6. Migrated proof and doc refresh | `docs/agents/lifecycle/*-maintenance/`, temp run outputs, proof surfaces | 2, 3, 4, 5 |

### Parallel Lanes

Lane A: Step 1 → Step 2  
Sequential, shared contract and packet-doc surfaces.

Lane B: Step 1 → Step 3  
Sequential after the contract freeze, but independent from packet-doc wording changes once the
contract is fixed.

Lane C: Step 1 → Step 4  
Sequential after the contract freeze, independent from most relay code changes.

Lane D: Step 5 → Step 6  
Sequential, shared test and proof surfaces.

### Execution Order

1. Launch Step 1 first. It sets the contract truth everyone else depends on.
2. After Step 1 lands or is at least text-stable, launch Lanes A, B, and C in parallel worktrees.
3. Merge A, B, and C.
4. Run Step 5 in a clean follow-up worktree once the shared code paths settle.
5. Run Step 6 last so the migrated proof reflects the final merged contract, renderer, relay, and
   transport state.

### Conflict Flags

1. Lanes A and B both touch `crates/xtask/src/agent_maintenance/contract_policy.rs` indirectly if
   the support-audit shape changes mid-implementation. Freeze the shared struct and field names
   before parallelizing.
2. Lanes A and C can both touch maintenance prose under `docs/` if workflow wording is edited
   twice. Keep Step 1 authoritative and avoid restating policy in Step 4.
3. Lanes B and C can both affect watcher or packet expectations used by shared tests. Do not start
   Step 5 until both are merged.

## Completion Summary

- Step 0: Scope Challenge, scope accepted as the minimum complete steady-state rewrite
- Architecture Review: one shared watcher, one shared packet_pr opener, one shared relay, explicit
  support audit, support ratchet, explicit manual closeout
- Code Quality Review: no shadow policy in YAML, no agent-specific packet schema, no hidden
  success semantics
- Test Review: coverage diagram produced, 13 concrete gaps identified
- Performance Review: no throughput blocker, but watcher and relay must stay data-driven and cheap
- NOT in scope: written
- What already exists: written
- TODOS cross-reference: one existing topology TODO is consumed by this plan
- Failure modes: 6 critical gaps flagged
- Parallelization: 4 lanes, 3 parallel after contract freeze, 1 final sequential proof lane
- Lake Score: choose the complete transport-and-contract convergence, not another docs-only pass

## Definition Of Done

This plan is done only when all of the following are true:

1. The specs, charter, operator guide, atlas, and packet-owned docs all describe the same steady
   state.
2. `codex` and `claude_code` no longer rely on worker-specific steady-state maintenance transport.
3. Shared packet rendering explicitly requires support audit, required uplift deltas, and
   blocker-only deferrals.
4. Shared relay validation enforces the same contract.
5. `opencode` and the other enrolled CLI agents no longer publish deliberate non-TUI unsupported
   posture as normal steady state.
6. Shared regression coverage protects the migrated lane and the support ratchet rule.
7. One migrated `codex` proof demonstrates that the post-`opencode` steady-state story is real.
