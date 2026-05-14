# PLAN - Converge CLI Agent Maintenance Onto One Shared Packet-PR Support-Uplift Lane

Status: ready for implementation  
Date: 2026-05-13  
Working branch: `staging`  
Plan revision baseline: `b5ba0d73`  
Primary design inputs:
- `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-design-20260513-112453.md`
- `docs/cli-agent-maintenance-steady-state-plan.md`

Disposition of draft inputs:
- `docs/cli-agent-maintenance-steady-state-plan.md` is an input to this rewrite only. Phase 1 must
  retire it as a live planning surface by moving any still-needed normative content into
  `docs/specs/**` and downgrading the draft doc to an archived pointer.

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
   the shared lane through the real shared watcher/opener/relay path and produces a real PR-owned
   maintenance artifact trail.
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
11. Packet-local write envelopes do not own policy. The eligible `writable_surfaces` for automated
    maintenance must be derived from registry/spec truth and shared maintenance policy, not narrowed
    ad hoc to justify a deferral.
12. A deferred support gap is valid only when it points to a tracked follow-on seam with an owner
    and milestone or to a concrete upstream/platform blocker that is external to this repo.

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
| Registry | enrollment truth, upstream metadata, dispatch mode, and write-envelope inputs | packet policy, ad hoc write-envelope narrowing, gate lists |
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
- a successful enrolled maintenance run must not increase the committed non-TUI support debt count
  for that agent unless the packet records a concrete upstream/platform blocker

Machine-checkable ratchet requirement:

- Phase 1 must create one committed baseline inventory at
  `docs/specs/unified-agent-api/non-tui-support-debt.md`
- each row in that inventory must identify: `agent_id`, surface family, exact unsupported surface,
  current reason, blocker class if any, owner, and target milestone or follow-on seam
- packet preparation must derive the current agent debt set from the same evidence model used by
  support publication, not from prompt prose
- relay validation and/or publication checks must compare pre-run and post-run debt counts for the
  target agent
- a run may only finish green when:
  - newly discovered non-TUI surface is handled or concretely blocked
  - all `required_uplifts_this_run[]` are handled
  - post-run debt count is less than or equal to pre-run debt count
- a newly onboarded agent may start partial, but its first maintenance run with either discovered
  surface or eligible preexisting gaps must produce at least one required uplift or one concrete
  tracked blocker

Eligibility rule for preexisting gaps:

- a preexisting non-TUI gap is in-scope for the current maintenance run only when at least one of
  these is true:
  - the target upstream release introduced or changed adjacent surface in the same command family
  - the gap can be closed entirely inside the registry/spec-derived maintenance write envelope for
    this agent
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

The packet shape is no longer TBD. Phase 1 freezes the exact support-audit schema below and
Phases 2 through 6 implement it without renaming fields.

```toml
[support_surface_audit]
required = true
surface_kinds = ["commands", "subcommands", "flags", "global_flags", "positional_args"]
excluded_surface_kinds = ["tui_only"]
allowed_deferrals = [
  "upstream_not_machine_exposed",
  "platform_evidence_missing",
  "requires_new_infra",
  "requires_new_architectural_seam",
  "outside_registry_maintenance_write_envelope",
]
pre_run_debt_count = 0
expected_post_run_debt_count = 0

[[support_surface_audit.discovered_upstream_surface]]
surface_kind = "flags"
command_path = "codex exec"
surface_id = "--json"
evidence_ref = "cli_manifests/codex/raw_help/..."

[[support_surface_audit.removed_upstream_surface]]
surface_kind = "flags"
command_path = "codex exec"
surface_id = "--legacy"
evidence_ref = "cli_manifests/codex/raw_help/..."

[[support_surface_audit.preexisting_unsupported_surface]]
surface_kind = "global_flags"
command_path = "claude"
surface_id = "--output-format"
debt_ref = "docs/specs/unified-agent-api/non-tui-support-debt.md#claude-code-output-format"

[[support_surface_audit.eligible_preexisting_surface]]
surface_kind = "global_flags"
command_path = "claude"
surface_id = "--output-format"
eligibility_reason = "adjacent_surface_changed"

[[support_surface_audit.missing_wrapper_support]]
surface_kind = "flags"
command_path = "codex exec"
surface_id = "--json"

[[support_surface_audit.missing_backend_support]]
surface_kind = "flags"
command_path = "codex exec"
surface_id = "--json"

[[support_surface_audit.required_uplifts_this_run]]
surface_kind = "flags"
command_path = "codex exec"
surface_id = "--json"
reason = "new_upstream_surface"
required_writes = ["wrapper", "backend", "manifest", "publication"]

[[support_surface_audit.deferred_preexisting_gaps]]
surface_kind = "global_flags"
command_path = "claude"
surface_id = "--output-format"
defer_reason = "requires_new_architectural_seam"
blocking_follow_on = "TODOS.md#close-claude-code-global-flag-gap"

[[support_surface_audit.publication_impacts]]
surface_kind = "flags"
command_path = "codex exec"
surface_id = "--json"
surface_doc = "docs/specs/unified-agent-api/support-matrix.md"
```

Required record shape rules:

| Record | Required keys | Notes |
| --- | --- | --- |
| surface row | `surface_kind`, `command_path`, `surface_id` | shared identity for every audit list |
| evidence-backed row | surface row + `evidence_ref` | used for discovered or removed upstream surface |
| debt-backed row | surface row + `debt_ref` | used for preexisting inventory rows |
| eligible row | surface row + `eligibility_reason` | only `adjacent_surface_changed`, `bounded_write_envelope`, or `no_new_seam_required` |
| uplift row | surface row + `reason`, `required_writes` | `required_writes` values limited to `wrapper`, `backend`, `manifest`, `publication`, `packet_docs` |
| deferred row | surface row + `defer_reason`, `blocking_follow_on` when repo-owned | `blocking_follow_on` omitted only for concrete external blockers |
| publication impact row | surface row + `surface_doc` | ties uplift to published truth |

Field invariants:

1. `required = true` for every enrolled automated maintenance packet.
2. `required_uplifts_this_run[]` equals:
   - all newly discovered non-TUI gaps with no allowed blocker, plus
   - all eligible preexisting gaps with no allowed blocker.
3. `deferred_preexisting_gaps[]` may contain only preexisting gaps, never newly discovered surface.
4. Every deferred row must use one `allowed_deferrals[]` value.
5. `expected_post_run_debt_count` must equal:
   `pre_run_debt_count - closed_gap_count + newly_blocked_external_gap_count`.
   It must never exceed `pre_run_debt_count`.
6. If `removed_upstream_surface[]` is non-empty, publication truth must contract in the same run or
   the packet is invalid.
7. If this block is absent, malformed, or derived partly from prompt prose instead of shared code,
   the packet is invalid.

Allowed blocker taxonomy:

- `upstream_not_machine_exposed`
- `platform_evidence_missing`
- `requires_new_infra`
- `requires_new_architectural_seam`
- `outside_registry_maintenance_write_envelope`

Not valid as blocker reasons:

- `deliberately_unsupported`
- `too_much_work_right_now`
- `not_part_of_v1`

Additional blocker rules:

- `requires_new_infra`, `requires_new_architectural_seam`, and
  `outside_registry_maintenance_write_envelope` are valid only when the packet points to a tracked
  follow-on seam or TODO with an owner and milestone.
- deleting or rewording a support-publication caveat does not satisfy the ratchet. The underlying
  gap must either be closed or carried as a concrete blocked inventory row.

### Spec Ownership Map

Phase 1 must leave one explicit ownership map so the three specs stop drifting:

| Concept | Owning truth |
| --- | --- |
| packet shape and execution contract | `docs/specs/maintenance-request-contract-v1.md` |
| enrolled transport, agent facts, and derived write envelope inputs | `docs/specs/agent-registry-contract.md` |
| onboarding expectations, maintenance maturity, and support posture policy | `docs/specs/cli-agent-onboarding-charter.md` |
| published support posture and debt visibility | `docs/specs/unified-agent-api/support-matrix.md` plus `docs/specs/unified-agent-api/non-tui-support-debt.md` |
| operator flow narrative | `docs/cli-agent-onboarding-factory-operator-guide.md` as explanatory only |
| workflow atlas narrative | `docs/cli-agent-onboarding-factory-workflow-atlas.md` as explanatory only |

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
| Proving run | migrated `codex` lane cannot open or validate from shared packet_pr flow | real shared watcher/opener/relay proof run | fix within blast radius, rerun proof | explicit run failure | Yes |
| Debt baseline drift | support debt inventory and support publication disagree about open non-TUI gaps | publication checks, packet derivation tests, proof review | fail until inventory and publication truth reconcile | explicit debt mismatch | Yes |
| Deferral laundering | packet defers repo-owned support work without a tracked seam, owner, and milestone | execute validation tests, packet proof review | reject packet until follow-on reference is present | explicit packet validation failure | Yes |

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
9. Add derivation tests for the committed support-debt inventory:
   inventory rows, support publication, and packet audit counts must agree for the target agent.
10. Add validation tests that repo-owned deferrals fail unless they include a tracked follow-on seam
    or TODO reference with owner and milestone.
11. Add regression coverage for upstream surface removal or rename so publication truth can shrink,
    not only grow.
12. Prove one migrated `codex` path end to end as a real maintenance run through the shared watcher,
    shared opener, and shared relay, producing the same PR-owned artifact trail maintainers will use
    in steady state.
13. Finish with `make preflight`.
14. Keep `opencode` packet-pr coverage green while migrating the other agents. Do not regress the
    already-proved lane.

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

## Implementation Sequencing Contract

This plan is only safe if sequencing is explicit.

1. Phase 1 is serial. It must end with text-stable spec field names, blocker enums, and debt-row
   shape before any parallel code lane starts.
2. Phases 2, 3, and 4 may run in parallel only after Phase 1 freezes:
   - the support-audit field names
   - the debt inventory row shape
   - the allowed blocker taxonomy
   - the meaning of `required_uplifts_this_run[]`
3. Phase 5 is an integration pass, not a fourth design lane. It starts only after Phases 2, 3, and
   4 merge.
4. Phase 6 is last. Real proof and final publication refresh happen only after code, docs, and
   tests are already green in-repo.
5. Any proof failure that exposes a missing invariant immediately creates a regression test before
   the proof is rerun.
6. No phase may invent new policy in workflow YAML, packet prose, or ad hoc test fixtures after
   Phase 1. Policy changes go back through the spec-owning surfaces first.

## Implementation Plan

### Phase 1: Freeze The Normative Steady-State Contract

**Goal**

Freeze one contract so later phases are implementation, not negotiation.

**Primary files**

- `docs/specs/maintenance-request-contract-v1.md`
- `docs/specs/agent-registry-contract.md`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/specs/unified-agent-api/support-matrix.md`
- `docs/specs/unified-agent-api/non-tui-support-debt.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/cli-agent-onboarding-factory-workflow-atlas.md`
- `docs/cli-agent-maintenance-steady-state-plan.md`

**Write scope**

- docs/spec truth
- explanatory docs
- new debt inventory
- no Rust or workflow changes yet

**Actions**

1. Freeze one definition of automated maintenance success:
   support audit first, bounded non-TUI uplift second, green gates third, manual closeout last.
2. Amend the maintenance request contract so the exact `support_surface_audit` schema in this plan
   becomes normative, including field names, enum values, and invariants.
3. Amend the registry contract so `packet_pr` is the steady-state enrolled transport and
   `workflow_dispatch` for `codex` and `claude_code` is explicitly transitional content, not a
   schema-level expectation.
4. Amend the charter so newly onboarded agents may start partial, but enrolled maintenance must
   ratchet non-TUI support upward and may not normalize deliberate unsupported posture.
5. Create `docs/specs/unified-agent-api/non-tui-support-debt.md` with one row per known enrolled
   non-TUI gap, including current `opencode` v1 restrictions and any support-matrix caveats now
   published for `codex` or `claude_code`.
6. Amend `docs/specs/unified-agent-api/support-matrix.md` so it points to the debt inventory as the
   only valid place for temporary non-TUI blockers.
7. Retire `docs/cli-agent-maintenance-steady-state-plan.md` as a live design surface. Default path:
   reduce it to a short archived pointer to the normative specs. Do not keep duplicated rules there.
8. Align the operator guide and workflow atlas so they explicitly point back to the same normative
   fields and no longer imply artifact-refresh-only success.
9. Freeze the ownership map:
   registry facts in the registry spec, packet shape in the maintenance-request spec, support debt
   truth in the support-matrix plus debt inventory pair, workflow docs as explanatory only.

**Verification**

- manual diff review across the three specs for field-name identity
- `rg -n 'deliberately unsupported|packet_doc_refresh|workflow_dispatch|packet_pr' docs/specs docs/cli-agent-*`
- confirm the debt inventory covers every non-TUI caveat currently published in support surfaces

**Outputs**

- one normative story
- one operator story
- one atlas story
- one committed debt baseline
- one frozen support-audit field set

**Exit criteria**

- a reader can move from spec to guide to atlas without seeing contradictory success semantics
- the draft steady-state plan no longer acts as a competing live source of truth
- Phase 2 through 4 can branch without arguing about names or blockers

### Phase 2: Rewrite Packet Generation And Packet-Owned Docs Around Support Audit

**Goal**

Make prepared packets and packet-owned docs explain the real maintenance job with no human
interpretation step.

**Primary files**

- `crates/xtask/src/agent_maintenance/contract_policy.rs`
- `crates/xtask/src/agent_maintenance/prepare.rs`
- `crates/xtask/src/agent_maintenance/docs.rs`
- `crates/xtask/src/agent_maintenance/request.rs`
- `crates/xtask/src/agent_maintenance/request/automation.rs`
- `docs/agents/lifecycle/codex-maintenance/**`
- `docs/agents/lifecycle/opencode-maintenance/**`

**Write scope**

- shared packet derivation and rendering code
- existing maintenance packet roots for `codex` and `opencode`
- no relay semantics or watcher transport changes yet

**Actions**

1. Add one shared typed representation for the support-audit block in the request/prepare layer.
   This type is the only source for field names, allowed enum values, and packet serialization.
2. Make packet preparation derive the audit block from:
   upstream help evidence, wrapper coverage truth, backend support truth, support publication truth,
   and the committed debt inventory.
3. Rewrite prompt rendering so the operator job is explicit:
   compare upstream surface against wrapper coverage and backend support, then either land bounded
   uplift or fail with a concrete blocker.
4. Rewrite `HANDOFF.md` and `governance/pr-summary.md` so success is described as bounded
   support-aware maintenance, not packet refresh plus incidental edits.
5. Rewrite `CI_WORKFLOWS_PLAN.md` and `OPS_PLAYBOOK.md` for `codex` and `opencode` so they both
   describe the same factory. If `claude_code` does not yet have a maintenance packet root, Phase 5
   will materialize it after transport convergence.
6. Ensure recovery guidance always points to regenerate packet, rerun relay, and preserve bounded
   writes. It must not point operators back to worker-specific worldview or manual drift paths as
   the main happy path.

**Verification**

- targeted packet-generation tests in `crates/xtask/tests/agent_maintenance_prepare.rs`
- snapshot diff review of rendered `maintenance-request.toml`, `HANDOFF.md`, prompt, and
  `pr-summary.md`
- confirm no per-agent renderer code invents extra support-audit fields

**Outputs**

- regenerated maintenance packet docs with truthful wording
- support-aware prompt template semantics
- frozen support-audit field names in shared code

**Exit criteria**

- packet docs alone are sufficient to explain the real maintenance job without reading workflow YAML
- `codex` and `opencode` packet roots serialize the same support-audit contract

### Phase 3: Enforce Support-Aware Relay Semantics

**Goal**

Make `execute-agent-maintenance` reject packets that do not satisfy the support-aware contract.

**Primary files**

- `crates/xtask/src/agent_maintenance/execute.rs`
- `crates/xtask/src/agent_maintenance/execute/runtime.rs`
- `crates/xtask/src/agent_maintenance/execute/validate.rs`
- `crates/xtask/src/agent_maintenance/execute/workflow.rs`
- `crates/xtask/src/agent_maintenance/execute/packet.rs`
- `crates/xtask/src/agent_maintenance/execute/types.rs`

**Write scope**

- relay validation and write-mode checks
- no watcher transport wiring
- no fresh spec invention

**Actions**

1. Validate the full support-audit block before write mode starts:
   required presence, enum validity, row shape, count invariants, and deterministic continuity with
   prepared packet artifacts.
2. Fail closed when a packet omits newly discovered non-TUI surface from
   `required_uplifts_this_run[]` without an allowed blocker.
3. Fail closed when a packet marks preexisting gaps as in-scope without one allowed
   `eligibility_reason`.
4. Fail closed when repo-owned deferrals do not point to a tracked follow-on seam or TODO with
   owner and milestone.
5. Fail closed when `expected_post_run_debt_count > pre_run_debt_count`.
6. Enforce the bounded write envelope:
   wrapper crate, backend module, manifest root, packet-owned docs, support publication surfaces,
   and nothing TUI-related.
7. Keep closeout manual and untouched by write mode.

**Verification**

- targeted validation tests in `crates/xtask/tests/agent_maintenance_execute.rs`
- negative fixtures for each invalid blocker and count mismatch
- dry-run plus write-mode validation parity checks

**Outputs**

- support-aware relay invariants
- shared failure-closed behavior

**Exit criteria**

- a packet that describes artifact refresh only cannot pass as a valid steady-state automated run
- the relay never needs agent-specific hidden policy to decide success

### Phase 4: Converge `codex` And `claude_code` Transport Onto Shared `packet_pr`

**Goal**

Make the live release-watch topology match the contract.

**Primary files**

- `crates/xtask/data/agent_registry.toml`
- `crates/xtask/src/agent_registry.rs`
- `crates/xtask/src/agent_registry/release_watch.rs`
- `crates/xtask/src/agent_maintenance/watch.rs`
- `.github/workflows/agent-maintenance-release-watch.yml`
- `.github/workflows/agent-maintenance-open-pr.yml`
- `.github/workflows/codex-cli-update-snapshot.yml`
- `.github/workflows/claude-code-update-snapshot.yml`

**Write scope**

- registry enrollment truth
- watcher/open-pr workflow wiring
- retirement of worker-specific steady-state transport

**Actions**

1. Migrate `codex` and `claude_code` registry truth from `workflow_dispatch` to `packet_pr`.
2. Make the shared watcher materialize `agent-maintenance-open-pr.yml` for every enrolled agent.
3. Remove worker-specific snapshot workflows from the steady-state path.
   Default action: delete both workflow files if nothing else depends on them.
   Fallback only if deletion is blocked: keep them as unscheduled, non-registry-referenced,
   clearly historical/manual-only surfaces with header comments stating they are not release-watch
   transport.
4. Keep any agent-specific acquisition detail outside the policy layer. If the shared opener needs
   more data, derive it from registry truth and the prepared packet, not from inline YAML policy.

**Verification**

- targeted watch/registry tests in `crates/xtask/tests/agent_maintenance_watch.rs` and
  `crates/xtask/tests/agent_registry.rs`
- workflow diff review proving no scheduled or registry-driven path still points at worker flows
- `cargo test -p xtask c4_spec_ci_wiring -- --nocapture`

**Outputs**

- one shared enrolled transport story
- no worker-specific steady-state transport dependency

**Exit criteria**

- the registry, watcher, and workflows all agree that enrolled automated maintenance opens via
  `agent-maintenance-open-pr.yml`
- no live scheduled path dispatches `codex-cli-update-snapshot.yml` or
  `claude-code-update-snapshot.yml`

### Phase 5: Refresh Historical Maintenance Packet Surfaces

**Goal**

Bring committed maintenance packet surfaces and support publication back into alignment after code
convergence.

**Primary files**

- `docs/agents/lifecycle/codex-maintenance/**`
- `docs/agents/lifecycle/opencode-maintenance/**`
- `docs/agents/lifecycle/claude-code-cli-onboarding/**`
- `docs/specs/unified-agent-api/support-matrix.md`
- `cli_manifests/support_matrix/current.json`

**Write scope**

- committed lifecycle maintenance docs
- support publication truth
- no new transport policy

**Actions**

1. Regenerate packet-owned docs for `codex` and `opencode` from the new shared renderer.
2. Materialize the committed `claude_code` maintenance packet root if transport convergence now
   makes it a steady-state enrolled maintenance lane and those docs do not already exist.
3. Update support publication surfaces so deliberate non-TUI unsupported posture is never presented
   as normal steady state. Every remaining caveat must point to a debt row or a concrete blocker.
4. Remove `opencode`'s v1 carveout from the steady-state story while preserving already-landed
   proof artifacts as historical evidence.
5. Update any packet-owned playbooks or workflow plans that still talk like worker transport is the
   normal path.

**Verification**

- regenerated doc diff review
- support-matrix diff review against debt inventory rows
- confirm old `opencode` proof artifacts remain intact and only surrounding explanatory surfaces
  change

**Outputs**

- committed lifecycle maintenance docs that match the migrated contract
- support publication truth that matches the debt baseline

**Exit criteria**

- there is no packet-owned maintenance doc left that treats worker transport as the steady state
- every published non-TUI caveat has either been removed or tied to the debt inventory

### Phase 6: Land Regression Coverage And One Migrated Proof

**Goal**

Prove the final architecture with tests first and one real migrated `codex` run second.

**Primary files**

- `crates/xtask/tests/agent_maintenance_prepare.rs`
- `crates/xtask/tests/agent_maintenance_execute.rs`
- `crates/xtask/tests/agent_maintenance_watch.rs`
- `crates/xtask/tests/agent_registry.rs`
- `crates/xtask/tests/c4_spec_ci_wiring.rs`
- migrated `codex` maintenance packet and proof surfaces

**Write scope**

- regression tests
- proof artifacts
- no new product-policy invention

**Actions**

1. Add regression coverage for:
   transport convergence, support-aware packet rendering, support-aware relay validation, blocker
   taxonomy, debt-count ratchet, and support-publication contraction on upstream removal.
2. Run targeted xtask suites until green before attempting the real proof.
3. Prepare and validate one migrated `codex` maintenance lane through the shared `packet_pr`
   contract.
4. Run one real proving path via the shared watcher, shared opener, and shared relay stack.
5. Preserve the resulting packet-owned evidence, including the real PR-owned artifact trail.
6. If the proof exposes a missing invariant, add the test first, then rerun the proof.
7. Finish with `make preflight`.

**Verification**

- green targeted xtask suites
- green `make preflight`
- proof artifact review for packet, validation report, written-paths report, and PR-owned docs

**Outputs**

- shared regression protection
- one migrated proof example

**Exit criteria**

- the repo can point to one previously worker-backed agent and show that the shared lane is real
- the proof uses the same watcher/opener/relay path future maintenance runs will use

## Worktree Parallelization Strategy

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| 0. Contract freeze | `docs/specs/`, `docs/cli-agent-*`, `docs/cli-agent-maintenance-steady-state-plan.md` | — |
| A. Packet derivation + packet docs | `crates/xtask/src/agent_maintenance/`, `docs/agents/lifecycle/codex-maintenance/`, `docs/agents/lifecycle/opencode-maintenance/` | 0 |
| B. Relay validation | `crates/xtask/src/agent_maintenance/execute/`, `crates/xtask/src/agent_maintenance/execute.rs` | 0 |
| C. Transport convergence | `crates/xtask/data/`, `crates/xtask/src/agent_registry/`, `crates/xtask/src/agent_maintenance/watch.rs`, `.github/workflows/` | 0 |
| D. Integration doc refresh | `docs/agents/lifecycle/`, `docs/specs/unified-agent-api/support-matrix.md`, `cli_manifests/support_matrix/` | A, B, C |
| E. Regression coverage + proof | `crates/xtask/tests/`, proof surfaces under `docs/agents/lifecycle/*-maintenance/governance/proof/` | A, B, C, D |

### Parallel Lanes

Lane 0: Contract freeze  
Serial. Nobody branches until the spec fields, debt-row shape, and blocker enums are fixed.

Lane A: Packet derivation + packet docs  
Touches `crates/xtask/src/agent_maintenance/{contract_policy.rs,prepare.rs,docs.rs,request*.rs}`
and committed `codex`/`opencode` maintenance packet roots.

Lane B: Relay validation  
Touches `crates/xtask/src/agent_maintenance/execute/**` only. It consumes the frozen packet shape
but does not rewrite packet prose.

Lane C: Transport convergence  
Touches registry, watcher, and workflow modules only. It does not rewrite relay validation or
packet-doc semantics.

Lane D: Integration doc refresh  
Starts after A, B, and C merge. It regenerates lifecycle maintenance docs, aligns support
publication, and materializes any missing steady-state packet roots such as `claude_code` if
needed.

Lane E: Regression coverage + proof  
Starts last. Tests and real proof run only after the merged steady state exists.

### Execution Order

1. Land Lane 0 first and tag that commit as the parallelization baseline.
2. Branch Lanes A, B, and C from that exact baseline and work them in parallel worktrees.
3. Merge A, B, and C back into one integration branch.
4. Run Lane D on the merged integration branch, not in parallel.
5. Run Lane E last so the proof reflects the final merged contract, renderer, relay, transport, and
   publication truth.

### Conflict Flags

1. Lane A and Lane B both depend on the exact support-audit schema. If that schema changes after
   Lane 0, parallelization stops and both lanes must be rebased from a new contract-freeze commit.
2. Lane A and Lane C both affect what downstream tests will expect. Do not start Lane E until both
   are merged and the shared packet shape plus transport shape are stable.
3. Lane D is intentionally serial because it touches lifecycle docs and support publication that
   depend on all earlier code lanes.
4. Only one person or one integrating agent should own merges from A, B, and C back to the shared
   integration branch. This is not a democracy problem. It is a merge-conflict avoidance problem.

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
- Failure modes: critical proof, debt, and deferral gaps explicitly flagged
- Parallelization: 6 execution steps, 3 parallel code lanes after one serial contract freeze
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
   posture as normal steady state, and the committed debt inventory shows a net reduction or a
   concrete external blocker for every remaining row touched by this milestone.
6. Shared regression coverage protects the migrated lane and the support ratchet rule.
7. One migrated `codex` proof demonstrates that the post-`opencode` steady-state story is real by
   succeeding through the real shared watcher/opener/relay path and producing a real PR-owned
   artifact trail.
8. The worker-specific snapshot workflows are either deleted or explicitly demoted to manual-only
   historical utilities with no scheduled or registry-driven role.
9. `docs/cli-agent-maintenance-steady-state-plan.md` no longer carries live normative content.
