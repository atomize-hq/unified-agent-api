# PLAN - Packet-First Contract With Narrow C-Tail

Status: proposed  
Date: 2026-05-10  
Branch: `staging`  
Base branch: `main`  
Repo: `atomize-hq/unified-agent-api`  
Work item: `Make the maintenance request packet plus relay contract the single source of truth for live maintenance enrollment`  
Plan commit baseline: `492356c`  
Design input: `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-design-20260510-101355.md`  
Supersedes: the current repo-root `PLAN.md`, which is about the older Codex stale-maintenance proof milestone and is no longer the right plan of record for `staging`.

## Objective

Land the approved `packet-first-contract-with-c-tail` milestone.

The repo already has a real maintenance factory:

- shared watcher: `.github/workflows/agent-maintenance-release-watch.yml`
- live workers: `.github/workflows/codex-cli-update-snapshot.yml`, `.github/workflows/claude-code-update-snapshot.yml`
- packet builder: `crates/xtask/src/agent_maintenance/prepare.rs`
- packet validator: `crates/xtask/src/agent_maintenance/request/automation.rs`
- local relay: `crates/xtask/src/agent_maintenance/execute.rs`

The problem is not missing machinery. The problem is split truth. The contract doc says the packet and relay should be shared, but the code and generated artifacts still leak milestone-1 assumptions like `execution_contract.executor = "codex"` and duplicated command/gate derivation.

This milestone fixes that seam without trying to unify all workers or rewrite every maintainer doc.

## Success Criteria

1. `prepare-agent-maintenance` emits Codex and Claude Code automated packets with one shared top-level envelope, one shared `[detected_release]` field set, and one shared `[execution_contract]` field set.
2. Newly generated packets use one shared relay executor identity, not an agent-specific wrapper identity.
3. `execute-agent-maintenance` validates the shared executor identity without requiring target-agent-specific branching.
4. Packet-owned fields that are currently hardcoded are derived from registry truth or one shared maintenance-policy module:
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
5. Normative docs under `docs/specs/**` match the live packet behavior. No spec/code contradiction remains around executor identity or packet transport metadata.
6. Generated maintainer surfaces stay in lockstep:
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
8. The follow-up milestone is recorded explicitly as `worker/runbook convergence`, not implied by vague TODO prose.

## Step 0 Scope Challenge

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| Release-watch enrollment truth | [`crates/xtask/data/agent_registry.toml`](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/data/agent_registry.toml) | Reuse. Registry remains the only enrollment source. |
| Shared stale-agent queue | [`crates/xtask/src/agent_maintenance/watch.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/agent_maintenance/watch.rs) | Reuse. No new watcher architecture. |
| Packet parsing + validation | [`crates/xtask/src/agent_maintenance/request.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/agent_maintenance/request.rs), [`request/automation.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/agent_maintenance/request/automation.rs) | Reuse and tighten. Remove milestone-1 assumptions here. |
| Packet generation | [`crates/xtask/src/agent_maintenance/prepare.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/agent_maintenance/prepare.rs) | Reuse and refactor. This is the main code seam. |
| Generated packet docs | [`crates/xtask/src/agent_maintenance/docs.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/agent_maintenance/docs.rs) | Reuse and deduplicate against packet policy code. |
| Relay execution boundary | [`crates/xtask/src/agent_maintenance/execute.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/agent_maintenance/execute.rs) | Reuse. Keep dry-run/write/manual-closeout boundary intact. |
| Live watcher and worker transport | [`.github/workflows/agent-maintenance-release-watch.yml`](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.github/workflows/agent-maintenance-release-watch.yml), [`.github/workflows/codex-cli-update-snapshot.yml`](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.github/workflows/codex-cli-update-snapshot.yml), [`.github/workflows/claude-code-update-snapshot.yml`](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.github/workflows/claude-code-update-snapshot.yml), [`.github/workflows/agent-maintenance-open-pr.yml`](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.github/workflows/agent-maintenance-open-pr.yml) | Reuse as transport only. No worker convergence in this milestone. |
| Normative packet contract | [`docs/specs/maintenance-request-contract-v1.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/maintenance-request-contract-v1.md) | Reuse as canonical contract, but tighten it to the actual implementation target. |
| Registry schema contract | [`docs/specs/agent-registry-contract.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/agent-registry-contract.md) | Reuse with narrow clarifications only. |

### Minimum Complete Change

The minimum complete milestone is:

1. define one shared packet/relay policy source in `xtask`
2. make packet generation and validation consume that source
3. make generated maintainer docs consume that same source
4. update the normative contract docs to match the chosen steady-state packet shape
5. add the regression coverage proving Codex and Claude Code now share the same contract shape

Anything smaller leaves hidden truth behind.

### Complexity Check

This will touch more than 8 files. That is acceptable here because the current problem is cross-surface contract drift. Pretending this can be fixed in two files is how the repo got split truth in the first place.

The important constraint is different:

- no new infrastructure
- at most one new `xtask` helper module for shared packet policy
- no new workflow family
- no new registry-owned freeform command arrays

That keeps the change engineered enough, not ornamental.

### Search / Build Decision

- **[Layer 1]** Reuse the existing watcher, worker, packet, and relay surfaces. No new maintenance control plane.
- **[Layer 1]** Keep workflow YAML transport-only. Do not move gates or write-envelope truth into workflow inputs.
- **[Layer 1]** Keep registry truth in `agent_registry.toml`. Do not create a second maintenance-contract store.
- **[Layer 3]** Treat `execution_contract.executor` as the relay identity, not the maintained agent identity. The executor field is naming *who executes the packet contract*, not *which agent is being updated*.

### Distribution Check

No new user-distributed artifact is introduced.

The deliverables are internal control-plane truth surfaces:

- normative docs under `docs/specs/**`
- `xtask` packet generation and validation code
- generated maintainer packet docs under `docs/agents/lifecycle/*-maintenance/**`

## Locked Decisions

These are the decisions that remove ambiguity from the design doc.

1. `execution_contract.executor` will be the shared relay identifier `execute-agent-maintenance`.
2. `prepare-agent-maintenance` will emit only `execute-agent-maintenance` for new packets.
3. Request validation will accept legacy `executor = "codex"` only as a backward-compatibility alias for already-committed packets and fixtures. Refreshing or regenerating a packet must normalize it to `execute-agent-maintenance`.
4. `detected_release.dispatch_workflow` will be present in the request packet for both dispatch kinds:
   - `workflow_dispatch` uses the registry-owned worker workflow filename
   - `packet_pr` uses the shared derived value `agent-maintenance-open-pr.yml`
5. The registry schema stays strict:
   - registry continues to omit `dispatch_workflow` for `packet_pr`
   - packet generation resolves the final workflow path
6. `ordered_commands` and `green_gates` remain shared-policy-generated in Rust. They will not move into freeform registry string arrays in this milestone.
7. `version_policy` in the packet will be read from registry truth, not hardcoded in `prepare.rs`.
8. Narrow C-tail means only docs that currently lie about the live topology or contract truth get edited. No broad runbook rewrite.
9. The explicit follow-up milestone after this lands is `worker/runbook convergence`.

## Architecture

### Current Drift

```text
agent_registry.toml
  -> watcher queue
  -> prepare.rs
       -> hardcodes version_policy
       -> hardcodes executor = "codex"
       -> derives gates/write set in one place
  -> request/automation.rs
       -> enforces milestone-1 executor = "codex"
  -> docs.rs
       -> derives similar gates/commands again
  -> generated HANDOFF.md / pr-summary.md
  -> execute-agent-maintenance

Result:
  one intended contract
  but multiple partially independent derivations
```

### Target Shape

```text
agent_registry.toml
  -> shared contract-policy module
       -> resolved detected_release values
       -> shared executor id
       -> writable surfaces
       -> read-only inputs
       -> ordered_commands
       -> green_gates
       -> recovery metadata
  -> prepare.rs
  -> request/automation.rs
  -> docs.rs
  -> closeout/render.rs (read path only if needed)
  -> generated request + HANDOFF + pr-summary
  -> execute-agent-maintenance

Result:
  one source of truth
  many projections
```

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
- `crates/xtask/src/agent_maintenance/request/automation.rs`
- new shared helper module under `crates/xtask/src/agent_maintenance/` for packet policy

Exact changes:

1. Add one shared helper module for:
   - shared executor id
   - resolved dispatch workflow
   - prompt template path
   - read-only inputs
   - writable surfaces
   - ordered commands
   - green gates
   - recovery notes
2. Move duplicated command/gate and write-surface derivation out of `prepare.rs` and `docs.rs`.
3. Keep policy generation deterministic and pure. No network calls, no workspace mutation inside the helper.

Acceptance:

1. `prepare.rs` and `docs.rs` no longer each own independent copies of green-gate policy.
2. One constant defines the steady-state executor identity.
3. One helper resolves `packet_pr` to `agent-maintenance-open-pr.yml`.

### Phase 2. Converge Packet Builder And Validator

Purpose: make the packet that gets written match the contract that gets enforced.

Primary modules:

- `crates/xtask/src/agent_maintenance/prepare.rs`
- `crates/xtask/src/agent_maintenance/request.rs`
- `crates/xtask/src/agent_maintenance/request/automation.rs`
- `crates/xtask/src/agent_maintenance/execute/*`

Exact changes:

1. In `prepare.rs`, derive `detected_release.version_policy` from `entry.maintenance.release_watch.version_policy`, not the literal `"latest_stable_minus_one"`.
2. Emit `execution_contract.executor = "execute-agent-maintenance"`.
3. Validate the shared executor identity in `request/automation.rs`.
4. Keep read compatibility for legacy `executor = "codex"` packets so already-committed packet fixtures and closeout paths still load.
5. Materialize `dispatch_workflow` consistently in packet generation and validation for both `workflow_dispatch` and `packet_pr`.
6. Ensure `execute-agent-maintenance` continues to treat the request packet as the authority for writable surfaces, prompt digest, and gates. No new hidden relay defaults.

Acceptance:

1. A Codex automated packet and a Claude Code automated packet parse under the same validator rules.
2. Historical packet fixtures with `executor = "codex"` still load where backward compatibility is required.
3. Newly generated packets always normalize to the shared executor.

### Phase 3. Refresh Generated Docs And Patch Narrow Truth Surfaces

Purpose: stop maintainer-facing docs from contradicting live code.

Primary modules and files:

- `crates/xtask/src/agent_maintenance/docs.rs`
- `docs/specs/maintenance-request-contract-v1.md`
- `docs/specs/agent-registry-contract.md`
- `cli_manifests/codex/OPS_PLAYBOOK.md`
- `cli_manifests/claude_code/OPS_PLAYBOOK.md`
- generated maintenance surfaces under `docs/agents/lifecycle/codex-maintenance/**`
- generated maintenance surfaces under `docs/agents/lifecycle/claude_code-maintenance/**` if committed outputs exist

Exact changes:

1. Update the normative packet contract doc to reflect the locked decisions above.
2. Clarify in the registry contract that registry omits `dispatch_workflow` for `packet_pr`, while packet generation resolves the final workflow path.
3. Refresh generated packet docs so `HANDOFF.md` and `pr-summary.md` display the shared executor identity and the resolved workflow truth.
4. Patch maintainer playbooks only where they currently imply:
   - agent-specific executor identity
   - workflow-owned gate truth
   - stale packet field semantics

Acceptance:

1. No maintainer doc claims `execution_contract.executor` names the maintained agent.
2. No doc claims workflow YAML is the owner of gate or write-envelope policy.
3. Generated maintenance docs remain renderer-owned. No manual edits to generated packet docs.

### Phase 4. Add Regression Coverage And Verification

Purpose: prove the new contract shape and prevent relapse.

Primary test files:

- `crates/xtask/tests/agent_maintenance_prepare.rs`
- `crates/xtask/tests/agent_maintenance_execute.rs`
- `crates/xtask/tests/agent_maintenance_closeout/request_and_schema.rs`
- `crates/xtask/tests/agent_maintenance_watch.rs`
- `crates/xtask/tests/c4_spec_ci_wiring.rs`
- harness files under `crates/xtask/tests/support/agent_maintenance_*`

Exact changes:

1. Add a Claude Code packet-generation regression test, not just Codex.
2. Add a validator compatibility test covering:
   - shared executor accepted
   - legacy `codex` executor alias accepted where intended
   - mismatched executor rejected
3. Add a packet-pr contract test proving the packet carries `dispatch_workflow = "agent-maintenance-open-pr.yml"`.
4. Update handoff/pr-summary lockstep tests to assert shared executor identity and shared policy outputs.
5. Preserve the existing fail-closed coverage:
   - prompt digest mismatch
   - out-of-bounds writes
   - noop runtime execution
   - manual closeout remains manual

Acceptance:

1. The request contract can no longer drift without a failing test.
2. Cross-agent packet parity is explicitly tested.
3. Legacy compatibility is deliberate, not accidental.

## Test Review

### Code Path Coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/xtask/src/agent_maintenance/prepare.rs
    ├── build_prepare_plan()
    │   ├── [ADD] Codex workflow_dispatch packet emits shared executor
    │   ├── [ADD] Claude workflow_dispatch packet emits same schema
    │   ├── [ADD] packet_pr packet emits resolved open-pr workflow
    │   └── [ADD] version_policy comes from registry, not a literal
    │
    └── build_execution_contract()
        ├── [ADD] shared executor constant
        ├── [ADD] shared gate derivation helper
        └── [ADD] shared writable/read-only derivation helper

[+] crates/xtask/src/agent_maintenance/request/automation.rs
    ├── validate_detected_release()
    │   ├── [ADD] workflow_dispatch packet accepted
    │   └── [ADD] packet_pr packet accepted with resolved generic workflow
    │
    └── validate_execution_contract()
        ├── [ADD] shared executor accepted
        ├── [ADD] legacy "codex" alias accepted where compatibility is required
        ├── [ADD] mismatched executor rejected
        └── [EXISTING] prompt digest mismatch rejected

[+] crates/xtask/src/agent_maintenance/docs.rs
    └── build_packet_docs_from_envelope()
        ├── [ADD] handoff renders shared executor
        ├── [ADD] pr-summary stays lockstep with same contract helper
        └── [ADD] packet_pr/workflow_dispatch truth matches request packet

[+] crates/xtask/src/agent_maintenance/execute/*
    └── dry-run/write context loading
        ├── [ADD] shared-executor packet accepted
        ├── [ADD] legacy packet still readable
        └── [EXISTING] prompt drift and write-boundary checks remain fail-closed
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

[+] packet_pr future-agent transport
    ├── [EXISTING] watcher and prepare packet_pr path coverage
    └── [ADD] request-schema coverage for resolved generic workflow

[+] Maintainer relay
    ├── [EXISTING] dry-run creates frozen packet
    ├── [EXISTING] write mode enforces writable envelope
    ├── [EXISTING] prompt drift fails closed
    └── [ADD] shared executor packet loads the same relay path
```

### Required Test Commands

Run at minimum:

```bash
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_execute
cargo test -p xtask --test agent_maintenance_closeout
cargo test -p xtask --test c4_spec_ci_wiring
cargo test -p xtask --test c0_spec_validate
make fmt-check
make clippy
make check
make test
```

## Failure Modes Registry

| Failure mode | Surface | User impact | Detection | Handling |
| --- | --- | --- | --- | --- |
| Shared contract doc says one thing, packet builder emits another | `docs/specs/**` vs `prepare.rs` | next maintainer inherits a hidden contract | spec + packet regression tests | single shared policy module and doc refresh |
| Claude packet generation still leaks Codex-specific executor or gate assumptions | `prepare.rs`, `docs.rs` | next live agent enrollment is fake, not real | new Claude packet tests | normalize shared executor and shared policy derivation |
| Legacy committed packets stop loading after validator hardening | `request/automation.rs`, closeout/read paths | historical closeout or replay breaks | compatibility tests | accept `codex` as read-only compatibility alias |
| `packet_pr` transport metadata stays contradictory | packet contract vs registry contract vs request packet | maintainers cannot trust recovery and transport instructions | request schema tests + spec updates | packet always carries resolved workflow path |
| Handoff and packet diverge again because command/gate derivation is duplicated | `docs.rs` vs `prepare.rs` | maintainer sees one gate list, relay enforces another | lockstep rendering tests | one helper reused by both surfaces |
| Dry-run packet becomes stale after prompt/template change | `execute/*` | write mode could apply unreviewed work | existing prompt-sha fail-closed test | preserve current dry-run/write validation boundary |

### Critical Gaps This Plan Must Close

1. There is no explicit cross-agent generated-packet parity test today.
2. The normative contract and the live packet behavior disagree on executor semantics.
3. The packet builder and the packet docs still duplicate policy derivation.

## Performance Review

No meaningful runtime performance work is required here. This milestone is about truth, not throughput.

Performance constraints:

1. Keep policy derivation pure and in-process. No network or shell calls in validation or doc rendering.
2. Do not widen relay write-envelope scans beyond the existing packet-owned surfaces.
3. Do not add repeated registry reloads in hot paths if the caller already has the entry loaded.

The only performance regression worth worrying about would be accidental extra filesystem churn from regenerating more packet docs than necessary. Avoid that by keeping the renderer ownership unchanged.

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
| 1. Lock contract decisions | `docs/specs/`, `PLAN.md` | — |
| 2. Extract shared packet policy | `crates/xtask/src/agent_maintenance/` | 1 |
| 3. Converge request builder + validator | `crates/xtask/src/agent_maintenance/`, `crates/xtask/src/agent_maintenance/request/` | 2 |
| 4. Refresh generated packet docs + narrow playbook patches | `crates/xtask/src/agent_maintenance/`, `docs/agents/lifecycle/`, `cli_manifests/*/OPS_PLAYBOOK.md`, `docs/specs/` | 2 |
| 5. Add regression tests and harness updates | `crates/xtask/tests/`, `crates/xtask/tests/support/` | 3 |
| 6. Final verification run | repo-wide validation commands | 4, 5 |

### Parallel Lanes

Lane A: Step 2 -> Step 3  
Sequential. Shared `crates/xtask/src/agent_maintenance/` ownership. This is the critical path.

Lane B: Step 4  
Can start after Step 2 stabilizes the constant names and helper signatures. Mostly docs and generated-surface convergence.

Lane C: Step 5  
Can start after Step 3 lands the final packet shape. Test ownership only.

### Execution Order

1. Launch Lane A first.
2. Once the shared helper and constant names are stable, launch Lane B and Lane C in parallel worktrees.
3. Merge Lane B and Lane C back into the main branch.
4. Run Step 6 once both are in.

### Conflict Flags

1. Lane A and Lane B both touch `crates/xtask/src/agent_maintenance/`. Do not start Lane B before the shared helper API is settled.
2. Lane C will likely touch assertions that depend on the final executor string and dispatch-workflow rules. Keep those constants fixed before parallelizing tests.

## Follow-Up Milestone

After this plan lands, the next planning session is:

`worker/runbook convergence`

That follow-up owns:

1. reducing worker-specific YAML differences
2. converging Codex and Claude worker flow shape around the now-trusted packet contract
3. simplifying the maintainer story further once the contract is actually true
4. preparing the next live agent enrollment seam without bespoke worker contracts

It does **not** belong inside this milestone.

## Exit Criteria

This plan is done when:

1. the shared executor identity is live in newly generated packets
2. Codex and Claude generated packets prove the same schema
3. the contract docs no longer contradict the implementation
4. the generated maintainer docs are derived from the same shared policy code
5. the regression suite prevents the repo from sliding back into split truth
