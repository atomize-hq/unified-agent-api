# PLAN - Packet-First Contract With Narrow C-Tail

Status: ready for implementation  
Date: 2026-05-10  
Branch: `staging`  
Base branch: `main`  
Repo: `atomize-hq/unified-agent-api`  
Work item: `Make the maintenance request packet plus relay contract the single source of truth for live maintenance enrollment`  
Plan commit baseline: `492356c`  
Design input: `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-design-20260510-101355.md`  
Supersedes: the prior repo-root `PLAN.md` that still framed the older stale-maintenance proof milestone

## Executive Summary

This milestone fixes one specific seam: the repo already has a real maintenance factory, but the
truth is still split across packet generation, packet validation, and generated maintainer docs.
`prepare.rs` still hardcodes milestone-1 assumptions, `request/automation.rs` still enforces them,
and `docs.rs` still re-derives parts of the same contract a second time.

The implementation target is boring on purpose. One registry-backed packet shape. One relay
identity. One shared policy source for derived contract fields. Workers stay transport-only.
Maintainer docs become projections of packet truth instead of a parallel source of policy.

This is not worker convergence. This is contract convergence, plus the minimum doc pass required to
stop lying about the live topology. The explicit follow-up milestone remains
`worker/runbook convergence`.

## Objective

Land the approved `packet-first-contract-with-c-tail` milestone without introducing new
infrastructure or broadening the blast radius beyond the maintenance contract seam.

The repo already has the core factory pieces:

- shared watcher: `.github/workflows/agent-maintenance-release-watch.yml`
- live workers: `.github/workflows/codex-cli-update-snapshot.yml`,
  `.github/workflows/claude-code-update-snapshot.yml`
- packet builder: `crates/xtask/src/agent_maintenance/prepare.rs`
- packet validator: `crates/xtask/src/agent_maintenance/request/automation.rs`
- local relay: `crates/xtask/src/agent_maintenance/execute.rs`
- generated maintainer packet surfaces under `docs/agents/lifecycle/*-maintenance/**`

The problem is not missing machinery. The problem is split truth at the contract boundary where the
next live maintenance agent would have to join.

## Success Criteria

1. `prepare-agent-maintenance` emits Codex and Claude Code automated packets with one shared
   top-level envelope, one shared `[detected_release]` schema, and one shared
   `[execution_contract]` schema.
2. Newly generated packets use the shared relay executor identity
   `execute-agent-maintenance`, not an agent-specific wrapper identity.
3. `request/automation.rs` validates the steady-state relay identity, while preserving explicit
   read compatibility for already-committed legacy packets that still say `executor = "codex"`.
4. Packet-owned fields that are currently duplicated or hardcoded are derived from one shared
   `xtask` policy source:
   - `detected_release.version_policy`
   - `detected_release.source_kind`
   - `detected_release.source_ref`
   - `detected_release.dispatch_kind`
   - `detected_release.dispatch_workflow`
   - `execution_contract.writable_surfaces`
   - `execution_contract.read_only_inputs`
   - `execution_contract.ordered_commands`
   - `execution_contract.green_gates`
   - `execution_contract.recovery.*`
5. `docs/specs/maintenance-request-contract-v1.md` and
   `docs/specs/agent-registry-contract.md` match the live packet behavior exactly. No spec/code
   contradiction remains around executor identity or packet transport metadata.
6. Generated maintainer surfaces stay in lockstep with packet truth:
   - `docs/agents/lifecycle/*-maintenance/governance/maintenance-request.toml`
   - `docs/agents/lifecycle/*-maintenance/HANDOFF.md`
   - `docs/agents/lifecycle/*-maintenance/governance/pr-summary.md`
7. Regression coverage proves:
   - Codex packet generation
   - Claude Code packet generation
   - `workflow_dispatch` transport
   - `packet_pr` transport
   - legacy packet compatibility for already-committed artifacts
   - prompt-digest and write-envelope fail-closed behavior
8. The next milestone is recorded explicitly as `worker/runbook convergence`, not implied by vague
   TODO prose.

## Step 0 Scope Challenge

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| Release-watch enrollment truth | `crates/xtask/data/agent_registry.toml` | Reuse. Registry remains the only enrollment source. |
| Shared stale-agent queue | `crates/xtask/src/agent_maintenance/watch.rs` | Reuse. No new watcher architecture. |
| Packet parsing + validation | `crates/xtask/src/agent_maintenance/request.rs`, `crates/xtask/src/agent_maintenance/request/automation.rs` | Reuse and tighten. Remove milestone-1 assumptions here. |
| Packet generation | `crates/xtask/src/agent_maintenance/prepare.rs` | Reuse and refactor. This is the main code seam. |
| Relay identity and execution boundary | `crates/xtask/src/agent_maintenance/execute.rs` | Reuse. The relay already advertises `EXECUTE_HOST_SURFACE = "execute-agent-maintenance"`. The rest of the system needs to converge on that truth. |
| Generated packet docs | `crates/xtask/src/agent_maintenance/docs.rs` | Reuse and deduplicate. Stop re-deriving command and gate policy a second time. |
| Generated closeout/read surfaces | `crates/xtask/src/agent_maintenance/closeout/render.rs`, `closeout/types.rs` | Reuse. Keep read-path compatibility intact if packet field semantics tighten. |
| Live watcher and worker transport | `.github/workflows/agent-maintenance-release-watch.yml`, `.github/workflows/codex-cli-update-snapshot.yml`, `.github/workflows/claude-code-update-snapshot.yml`, `.github/workflows/agent-maintenance-open-pr.yml` | Reuse as transport only. No worker convergence in this milestone. |
| Normative packet contract | `docs/specs/maintenance-request-contract-v1.md` | Reuse as canonical contract, but tighten it to the actual implementation target. |
| Registry schema contract | `docs/specs/agent-registry-contract.md` | Reuse with narrow clarifications only. |

### Minimum Complete Change

The minimum complete milestone is:

1. define one shared maintenance-contract policy source inside `xtask`
2. make packet generation consume that source
3. make packet validation consume that source or validate against its exact outputs
4. make generated maintainer docs consume that same source instead of re-deriving policy
5. update the normative contract docs to the exact steady-state packet behavior
6. add regression coverage that proves Codex and Claude Code now share the same contract shape

Anything smaller leaves hidden truth behind.

### Complexity Check

This will touch more than 8 files. That is acceptable because the defect is cross-surface contract
drift. Pretending this is a two-file cleanup would just preserve the hidden contract.

The complexity guardrails are stricter than the file count:

- no new infrastructure
- exactly one new `xtask` helper module under `crates/xtask/src/agent_maintenance/`
- no new workflow family
- no new registry-owned freeform command arrays
- no new agent-specific packet schema

That keeps the change engineered enough, not ornamental.

### Search / Build Decision

- **[Layer 1]** Reuse the existing watcher, worker, packet, and relay surfaces. No new maintenance
  control plane.
- **[Layer 1]** Keep workflow YAML transport-only. Do not move gate or write-envelope truth into
  workflow inputs.
- **[Layer 1]** Keep registry truth in `agent_registry.toml`. Do not create a second
  maintenance-contract store.
- **[Layer 3]** Treat `execution_contract.executor` as the relay identity, not the maintained
  agent identity. The field names who executes the packet contract, not which wrapper crate is
  being updated.

### Distribution Check

No new user-distributed artifact is introduced.

The deliverables are internal control-plane truth surfaces:

- normative docs under `docs/specs/**`
- `xtask` packet generation and validation code
- generated maintainer packet docs under `docs/agents/lifecycle/*-maintenance/**`

## Locked Decisions

These decisions remove ambiguity from the design doc and from the older plan draft.

1. The steady-state executor value is `execute-agent-maintenance`.
2. The canonical source for that value must live in shared Rust code, not as repeated string
   literals. The implementation may either expose `execute::EXECUTE_HOST_SURFACE` cleanly or move
   the constant into the new shared contract-policy module, but there must be one canonical owner.
3. New packets always normalize to `execute-agent-maintenance`.
4. Request validation accepts legacy `executor = "codex"` only as a backward-compatibility alias
   for already-committed packets and fixtures. That alias is read-only compatibility, not live
   contract truth.
5. `detected_release.dispatch_workflow` stays materialized in the request packet for both dispatch
   kinds:
   - `workflow_dispatch` uses the registry-owned worker workflow filename
   - `packet_pr` uses the derived shared workflow `agent-maintenance-open-pr.yml`
6. The registry contract stays strict:
   - registry continues to omit `dispatch_workflow` for `packet_pr`
   - packet generation resolves the final workflow filename into the packet
7. `ordered_commands` and `green_gates` remain generated in Rust. They do not move into freeform
   registry string arrays in this milestone.
8. `ordered_commands` is still the maintainer-facing command list for the packet. This milestone
   centralizes its derivation. It does not invent a second work queue.
9. Generated maintainer docs are derivative from packet truth or the same shared policy source. No
   hand-maintained parallel contract survives this milestone.
10. Narrow C-tail means only docs that currently lie about the live topology or contract truth get
    edited. No broad runbook rewrite.
11. The explicit follow-up milestone after this lands is `worker/runbook convergence`.

## Architecture

### Current Drift

```text
agent_registry.toml
  -> watch.rs
  -> prepare.rs
       -> hardcodes version_policy
       -> hardcodes executor = "codex"
       -> resolves dispatch_workflow locally
       -> derives writable/gate policy locally
  -> request/automation.rs
       -> enforces milestone-1 executor = "codex"
       -> requires materialized detected_release fields
  -> docs.rs
       -> re-derives ordered_commands
       -> re-derives green_gates
  -> generated HANDOFF.md / pr-summary.md
  -> execute.rs
       -> already defines EXECUTE_HOST_SURFACE = "execute-agent-maintenance"

Result:
  one intended contract
  but multiple partially independent derivations
```

### Target Shape

```text
agent_registry.toml
  -> agent_maintenance/contract_policy.rs
       -> canonical executor identity
       -> resolved detected_release values
       -> writable surfaces
       -> read-only inputs
       -> ordered_commands
       -> green_gates
       -> recovery metadata
  -> prepare.rs
  -> request/automation.rs
  -> docs.rs
  -> closeout/render.rs (read-path only if needed)
  -> generated request + HANDOFF + pr-summary
  -> execute.rs

Result:
  one source of truth
  many projections
```

### Blast Radius

The primary blast radius is narrow but real:

- code: `crates/xtask/src/agent_maintenance/{prepare.rs,request.rs,request/automation.rs,docs.rs,execute.rs,mod.rs}`
- likely new helper: `crates/xtask/src/agent_maintenance/contract_policy.rs`
- specs: `docs/specs/maintenance-request-contract-v1.md`,
  `docs/specs/agent-registry-contract.md`
- generated maintenance surfaces: `docs/agents/lifecycle/*-maintenance/**`
- targeted playbooks if they repeat packet lies:
  `cli_manifests/codex/OPS_PLAYBOOK.md`, `cli_manifests/claude_code/OPS_PLAYBOOK.md`
- tests and harnesses under `crates/xtask/tests/**`

### Ownership Map

| Surface | Owner | Consumers | Rule |
| --- | --- | --- | --- |
| `agent_id`, `manifest_root`, release-watch enrollment | registry | watcher, prepare, docs, validation | registry is canonical |
| `detected_release.version_policy`, `source_kind`, `source_ref`, `dispatch_kind` | registry + shared resolver | prepare, validation, docs | packet is a resolved projection |
| `detected_release.dispatch_workflow` | shared resolver | prepare, validation, docs, workflow specs | always materialized in packet |
| `execution_contract.executor` | shared relay constant | prepare, validation, docs, execute | one value, not per-agent |
| `writable_surfaces`, `read_only_inputs`, `ordered_commands`, `green_gates` | shared policy module | prepare, docs, execute validation | no duplicated derivation |
| request packet TOML | prepare renderer | execute, refresh, docs | frozen per run |
| `HANDOFF.md` and `pr-summary.md` | shared packet doc renderer | maintainers, PR creation | derivative from packet truth, not hand-edited |

## Implementation Plan

### Phase 1. Extract Shared Contract Policy

Purpose: remove duplicate policy derivation and define the steady-state packet shape in code once.

Primary modules:

- `crates/xtask/src/agent_maintenance/prepare.rs`
- `crates/xtask/src/agent_maintenance/docs.rs`
- `crates/xtask/src/agent_maintenance/execute.rs`
- new helper module:
  `crates/xtask/src/agent_maintenance/contract_policy.rs`

Exact changes:

1. Add `contract_policy.rs` with one public surface for:
   - canonical executor id
   - resolved dispatch workflow
   - prompt template path
   - read-only inputs
   - writable surfaces
   - ordered commands
   - green gates
   - recovery notes
2. Route `prepare.rs` through the shared policy module instead of hardcoding:
   - `executor = "codex"`
   - literal version-policy assumptions
   - duplicate gate and write-envelope logic
3. Route `docs.rs` through the same policy source so `HANDOFF.md` and `pr-summary.md` stop
   re-deriving `ordered_commands` and `green_gates` independently.
4. Keep policy generation deterministic and pure. No network calls, no workspace mutation, no CLI
   subprocesses inside the helper.

Proof:

1. `prepare.rs` and `docs.rs` no longer each own independent copies of gate or write-surface
   policy.
2. One canonical owner defines the steady-state executor identity.
3. One helper resolves `packet_pr` to `agent-maintenance-open-pr.yml`.

### Phase 2. Converge Packet Builder And Validator

Purpose: make the packet that gets written match the contract that gets enforced.

Primary modules:

- `crates/xtask/src/agent_maintenance/prepare.rs`
- `crates/xtask/src/agent_maintenance/request.rs`
- `crates/xtask/src/agent_maintenance/request/automation.rs`
- `crates/xtask/src/agent_maintenance/execute/*`

Exact changes:

1. In `prepare.rs`, derive `detected_release.version_policy` from
   `entry.maintenance.release_watch.version_policy`, not the literal
   `"latest_stable_minus_one"`.
2. Emit `execution_contract.executor = "execute-agent-maintenance"`.
3. Validate the shared executor identity in `request/automation.rs`.
4. Preserve read compatibility for legacy `executor = "codex"` packets so already-committed packet
   fixtures, refresh flows, and closeout/read paths still load.
5. Keep `detected_release.dispatch_workflow` materialized in packet generation and validation for
   both `workflow_dispatch` and `packet_pr`.
6. Ensure `execute-agent-maintenance` continues to treat the request packet as the authority for
   writable surfaces, prompt digest, ordered commands, green gates, and recovery paths. No hidden
   relay defaults.

Proof:

1. A Codex automated packet and a Claude Code automated packet parse under the same validator
   rules.
2. Historical packet fixtures with `executor = "codex"` still load where backward compatibility is
   required.
3. Newly generated packets always normalize to the shared executor.

### Phase 3. Refresh Generated Docs And Patch Narrow Truth Surfaces

Purpose: stop maintainer-facing docs from contradicting live code.

Primary modules and files:

- `crates/xtask/src/agent_maintenance/docs.rs`
- `docs/specs/maintenance-request-contract-v1.md`
- `docs/specs/agent-registry-contract.md`
- `cli_manifests/codex/OPS_PLAYBOOK.md`
- `cli_manifests/claude_code/OPS_PLAYBOOK.md`
- generated maintenance surfaces under `docs/agents/lifecycle/*-maintenance/**`

Exact changes:

1. Update the normative packet contract doc to match the locked decisions above.
2. Clarify in the registry contract that registry omits `dispatch_workflow` for `packet_pr`, while
   packet generation resolves the final workflow path into the request packet.
3. Refresh generated packet docs so `HANDOFF.md` and `pr-summary.md` display the shared executor
   identity and the resolved workflow truth.
4. Patch maintainer playbooks only where they currently imply:
   - agent-specific executor identity
   - workflow-owned gate truth
   - stale packet field semantics
5. Keep generated maintenance docs renderer-owned. No hand edits to generated packet docs.

Proof:

1. No maintainer doc claims `execution_contract.executor` names the maintained agent.
2. No doc claims workflow YAML is the owner of gate or write-envelope policy.
3. The spec no longer says `dispatch_workflow` is omitted from `packet_pr` packets when the live
   implementation and tests require the resolved field to exist.

### Phase 4. Add Regression Coverage And Verification

Purpose: prove the new contract shape and prevent relapse.

Primary test files:

- `crates/xtask/tests/agent_maintenance_prepare.rs`
- `crates/xtask/tests/agent_maintenance_execute.rs`
- `crates/xtask/tests/agent_maintenance_refresh/automated_requests.rs`
- `crates/xtask/tests/agent_maintenance_closeout/request_and_schema.rs`
- `crates/xtask/tests/agent_maintenance_watch.rs`
- `crates/xtask/tests/c4_spec_ci_wiring.rs`
- `crates/xtask/tests/c0_spec_validate.rs`
- harness files under `crates/xtask/tests/support/agent_maintenance_*`

Exact changes:

1. Update existing prepare assertions that currently require `executor = "codex"`.
2. Add a Claude Code packet-generation regression test, not just Codex.
3. Add validator compatibility coverage for:
   - shared executor accepted
   - legacy `codex` executor alias accepted where intended
   - mismatched executor rejected
4. Add a `packet_pr` contract test proving the request packet carries
   `dispatch_workflow = "agent-maintenance-open-pr.yml"`.
5. Update handoff/pr-summary lockstep tests to assert shared executor identity and shared policy
   outputs.
6. Preserve the existing fail-closed coverage:
   - prompt digest mismatch
   - out-of-bounds writes
   - noop runtime execution
   - manual closeout remains manual

Proof:

1. The request contract can no longer drift without a failing test.
2. Cross-agent packet parity is explicitly tested.
3. Legacy compatibility is deliberate, not accidental.

## Architecture Review

This plan is intentionally boring by default.

1. No new control plane is introduced. Existing watcher, worker, packet, relay, and closeout
   surfaces remain in place.
2. The only new code surface is one shared helper module. That is the minimum structure needed to
   delete duplicate derivation without scattering constants further.
3. The relay boundary remains strict: packet truth stays frozen at prepare time, write mode still
   validates against the dry-run baseline, and closeout remains manual.
4. Security posture does not widen. The write envelope remains packet-owned and validated. The
   change reduces hidden policy, which makes accidental overreach less likely.
5. Distribution architecture does not change. This is an internal factory milestone, not a new
   user-facing artifact.

## Code Quality Review

The code-quality target is not “less code.” It is “one obvious owner per fact.”

1. `prepare.rs` must stop hardcoding contract fields that `execute.rs` or the registry already own.
2. `docs.rs` must stop re-deriving `ordered_commands` and `green_gates` separately from the packet
   builder. If `HANDOFF.md` and the request packet disagree, the maintainer loses.
3. The new shared helper must stay pure and explicit. No trait maze, no macro layer, no generalized
   policy engine.
4. The registry must remain structured. Do not replace typed release-watch fields with freeform
   string arrays just to avoid writing Rust.
5. If any touched files contain nearby ASCII diagrams or contract comments, update them in the same
   change. Stale diagrams are worse than no diagrams.

## Test Review

### Code Path Coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/xtask/src/agent_maintenance/contract_policy.rs
    ├── [ADD] canonical executor identity
    ├── [ADD] resolved dispatch_workflow for both dispatch kinds
    ├── [ADD] shared read/write surface derivation
    └── [ADD] shared ordered_commands / green_gates / recovery derivation

[+] crates/xtask/src/agent_maintenance/prepare.rs
    ├── build_prepare_plan()
    │   ├── [ADD] Codex workflow_dispatch packet emits shared executor
    │   ├── [ADD] Claude workflow_dispatch packet emits same schema
    │   ├── [ADD] packet_pr packet emits resolved open-pr workflow
    │   └── [ADD] version_policy comes from registry, not a literal
    │
    └── build_execution_contract()
        └── [CHANGE] becomes a thin projection over shared contract policy

[+] crates/xtask/src/agent_maintenance/request/automation.rs
    ├── validate_detected_release()
    │   ├── [KEEP] materialized dispatch_workflow required in the packet
    │   └── [ADD] packet_pr and workflow_dispatch both validate cleanly
    │
    └── validate_execution_contract()
        ├── [ADD] shared executor accepted
        ├── [ADD] legacy "codex" alias accepted where compatibility is required
        ├── [ADD] mismatched executor rejected
        └── [KEEP] prompt digest mismatch rejected

[+] crates/xtask/src/agent_maintenance/docs.rs
    └── build_packet_docs_from_envelope()
        ├── [ADD] handoff renders shared executor
        ├── [ADD] pr-summary stays lockstep with the same contract helper
        └── [REMOVE] duplicate gate/command derivation paths

[+] crates/xtask/src/agent_maintenance/execute/*
    └── dry-run/write context loading
        ├── [ADD] shared-executor packet accepted
        ├── [ADD] legacy packet still readable where intended
        └── [KEEP] prompt drift and write-boundary checks remain fail-closed
```

### Maintainer Flow Coverage

```text
USER FLOW COVERAGE
===========================
[+] Shared watcher -> Codex worker -> packet generation
    ├── [EXISTING] watcher queue frozen-field coverage
    └── [ADD] packet contract schema assertions

[+] Shared watcher -> Claude worker -> packet generation
    ├── [EXISTING] watcher queue frozen-field coverage
    └── [ADD] generated packet parity coverage

[+] packet_pr transport
    ├── [EXISTING] watcher and prepare packet_pr path coverage
    └── [ADD] request-schema coverage for resolved generic workflow

[+] Maintainer relay
    ├── [EXISTING] dry-run creates frozen packet
    ├── [EXISTING] write mode enforces writable envelope
    ├── [EXISTING] prompt drift fails closed
    └── [ADD] shared executor packet loads the same relay path

[+] Historical packet compatibility
    ├── [ADD] legacy executor alias accepted for committed artifacts
    └── [ADD] regenerated packet normalizes to shared executor
```

### Required Test Commands

Run at minimum:

```bash
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_execute
cargo test -p xtask --test agent_maintenance_refresh
cargo test -p xtask --test agent_maintenance_closeout
cargo test -p xtask --test c4_spec_ci_wiring
cargo test -p xtask --test c0_spec_validate
make fmt-check
make clippy
make check
make test
```

## Performance Review

No meaningful runtime throughput work is required here. This milestone is about truth, not speed.

Performance guardrails:

1. Keep policy derivation pure and in-process. No network or shell calls in validation or doc
   rendering.
2. Do not widen relay write-envelope scans beyond the existing packet-owned surfaces.
3. Do not add repeated registry reloads in hot paths if the caller already has the entry loaded.
4. Avoid unnecessary doc churn. Keep generated-surface ownership unchanged so refresh only touches
   the intended maintenance packet files.

The only performance regression worth worrying about is accidental filesystem churn from widening
the generated surface set. Do not do that.

## Failure Modes Registry

| Failure mode | Surface | Test coverage required | Handling required | Silent if missed? | Status |
| --- | --- | --- | --- | --- | --- |
| Shared contract doc says one thing, packet builder emits another | `docs/specs/**` vs `prepare.rs` | spec + packet regression tests | one shared policy module and doc refresh | yes | must close |
| Claude packet generation still leaks Codex-specific executor or gate assumptions | `prepare.rs`, `docs.rs` | explicit Claude parity test | normalize shared executor and shared policy derivation | yes | must close |
| Legacy committed packets stop loading after validator hardening | `request/automation.rs`, refresh, closeout/read paths | compatibility tests | accept `codex` as read-only compatibility alias | yes | must close |
| `packet_pr` transport metadata stays contradictory | packet contract vs registry contract vs request packet | request schema tests + spec updates | packet always carries resolved workflow path | yes | must close |
| Handoff and packet diverge again because command/gate derivation is duplicated | `docs.rs` vs `prepare.rs` | lockstep rendering tests | one helper reused by both surfaces | yes | must close |
| Dry-run packet becomes stale after prompt/template change | `execute/*` | existing prompt-sha fail-closed test | preserve current dry-run/write validation boundary | no, write mode already rejects | keep |

### Critical Gaps This Plan Must Close

1. There is no explicit cross-agent generated-packet parity test today.
2. The normative contract and the live packet behavior disagree on executor semantics.
3. The packet builder and the packet docs still duplicate policy derivation.
4. `agent_maintenance_refresh` still has codex-specific request-validation expectations that must
   be rewritten as shared-contract plus legacy-compatibility assertions.

## NOT In Scope

1. Full worker YAML unification across Codex and Claude Code.
2. Broad maintainer runbook rewrite.
3. New-agent enrollment for `opencode`, `goose`, or any other candidate.
4. Changing the manual closeout boundary.
5. Replacing `xtask codex-validate` with a newly named generic validator subcommand.
6. Reworking watcher scheduling, concurrency, or dispatch fanout architecture.
7. Expanding registry schema into freeform command-list storage.

## Worktree Parallelization Strategy

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| 1. Extract shared contract policy | `crates/xtask/src/agent_maintenance/` | — |
| 2. Converge packet builder + validator | `crates/xtask/src/agent_maintenance/`, `crates/xtask/src/agent_maintenance/request/` | 1 |
| 3. Refresh generated docs + patch narrow truth surfaces | `crates/xtask/src/agent_maintenance/`, `docs/specs/`, `docs/agents/lifecycle/`, `cli_manifests/*/` | 1 |
| 4. Add regression tests and harness updates | `crates/xtask/tests/`, `crates/xtask/tests/support/` | 2 |
| 5. Final verification run | repo-wide validation commands | 3, 4 |

### Parallel Lanes

Lane A: Step 1 -> Step 2  
Sequential. Shared `crates/xtask/src/agent_maintenance/` ownership. This is the critical path.

Lane B: Step 3  
Can start after Step 1 lands or is stable enough that the shared helper names and field semantics
are fixed. This lane is mostly doc convergence plus generated-surface refresh.

Lane C: Step 4  
Can start after Step 2 lands the final packet shape. Test ownership only.

### Execution Order

1. Launch Lane A first and settle the helper API plus steady-state field values.
2. Once Lane A has stabilized the contract surface, launch Lane B and Lane C in parallel
   worktrees.
3. Merge Lane B and Lane C back into the main branch.
4. Run Step 5 once both are in.

### Conflict Flags

1. Lane A and Lane B both touch `crates/xtask/src/agent_maintenance/`. Do not start Lane B before
   the shared helper API and executor semantics are settled.
2. Lane C will update assertions that depend on the final executor string and
   `dispatch_workflow` rules. Keep those constants fixed before parallelizing tests.
3. If Lane B needs renderer changes in `docs.rs`, it must either branch after Lane A lands or
   coordinate carefully. `docs.rs` is the only meaningful merge-conflict hotspot in this plan.

## Follow-Up Milestone

After this plan lands, the next planning session is:

`worker/runbook convergence`

That follow-up owns:

1. reducing worker-specific YAML differences
2. converging Codex and Claude worker flow shape around the now-trusted packet contract
3. simplifying the maintainer story further once the contract is actually true
4. preparing the next live agent enrollment seam without bespoke worker contracts

It does not belong inside this milestone.

## Completion Summary

- Step 0: Scope Challenge — scope accepted as-is; the seam is wide in file count but narrow in
  architecture
- Architecture Review: 0 open architecture questions; one shared helper module is the chosen seam
- Code Quality Review: 3 mandatory dedupe targets (`executor`, `dispatch_workflow`,
  command/gate policy)
- Test Review: diagrams produced, 4 mandatory regression additions identified
- Performance Review: 0 throughput concerns, 4 guardrails recorded
- NOT in scope: written
- What already exists: written
- Failure modes: 4 silent-contract gaps flagged and required to close
- Parallelization: 3 lanes, 2 can run in parallel after the core helper lands
- Lake Score: 5/5 recommendations chose the complete option over the shortcut

## Exit Criteria

This plan is done when:

1. the shared executor identity is live in newly generated packets
2. Codex and Claude generated packets prove the same schema
3. the contract docs no longer contradict the implementation
4. the generated maintainer docs are derived from the same shared policy code
5. the regression suite prevents the repo from sliding back into split truth
