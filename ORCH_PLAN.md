# ORCH_PLAN - Packet-First Maintenance Contract Execution

## Summary

Current branch: `staging`  
Authoritative milestone source: repo-root `PLAN.md`  
Branch strategy: parent integrates on top of `origin/staging` and lands back to `staging`; worker
lanes branch only from the parent-owned frozen core checkpoint, never directly from `origin/staging`
after launch.

Parent critical path location:

```text
crates/xtask/src/agent_maintenance/**
```

Worker concurrency cap: `2` worker lanes plus the parent.  
Reason: the only safe parallelization after the core checkpoint is doc truth and regression
coverage. The contract core is serialized.

Worktree root:

```text
/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-packet-first-contract
```

Worktree and branch layout:

- `staging-live`
  - path: `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-packet-first-contract/staging-live`
  - branch: `staging`
- `parent-core`
  - path: `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-packet-first-contract/parent-core`
  - branch: `codex/packet-first-contract-core`
- `lane-b-docs`
  - path: `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-packet-first-contract/lane-b-docs`
  - branch: `codex/packet-first-contract-doc-truth`
- `lane-c-tests`
  - path: `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-packet-first-contract/lane-c-tests`
  - branch: `codex/packet-first-contract-regressions`

Run-state root:

```text
/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.runs/packet-first-contract-with-c-tail
```

Derived-artifact policy:

- `docs/agents/lifecycle/*-maintenance/**` are derived packet surfaces, not independent policy
  documents.
- `.runs/**` are derived orchestration state, never assumed tracked deliverables.
- Generated packet docs are only treated as tracked live surfaces when they already exist as
  committed maintenance roots on `staging`. This milestone does not invent new tracked maintenance
  roots unless `PLAN.md` explicitly requires it.

This plan replaces the stale repo-root `ORCH_PLAN.md`, which targeted the older real-proof session
and is not authoritative for this milestone.

## Session Target

Land the `packet-first-contract-with-c-tail` milestone from `PLAN.md` by converging packet
generation, packet validation, and maintainer packet docs around one shared maintenance-contract
policy source inside `xtask`, while preserving:

- transport-only workflows
- one new helper module maximum
- no new infra
- no registry freeform command arrays
- manual closeout
- legacy packet read compatibility
- explicit follow-up milestone `worker/runbook convergence`

The critical path is contract truth inside `xtask`, not workflow redesign.

## Completion Definition

This session is complete only when all of the following are true:

1. `prepare-agent-maintenance` emits automated Codex and Claude Code packets with one shared
   top-level envelope, one shared `[detected_release]` schema, and one shared
   `[execution_contract]` schema.
2. Newly generated automated packets emit `execution_contract.executor = "execute-agent-maintenance"`.
3. `request/automation.rs` validates the shared executor as steady-state truth and still accepts
   legacy `executor = "codex"` only as backward-compatible read input for already-committed
   artifacts and fixtures.
4. Packet-owned derived fields are emitted from one shared Rust policy source rather than split
   across `prepare.rs` and `docs.rs`.
5. `docs/specs/maintenance-request-contract-v1.md` and
   `docs/specs/agent-registry-contract.md` match live packet behavior exactly, including relay
   identity and `packet_pr` workflow materialization.
6. Live committed maintenance packet surfaces that already exist on `staging` are regenerated into
   lockstep with packet truth.
7. Regression coverage proves Codex generation, Claude Code generation, `workflow_dispatch`,
   `packet_pr`, legacy compatibility, prompt-digest fail-closed behavior, and write-envelope
   fail-closed behavior.
8. `close-agent-maintenance` remains manual and unchanged.
9. No workflow YAML becomes a second contract-policy owner.
10. The next milestone is recorded explicitly as `worker/runbook convergence`.

## Hard Guards

- `PLAN.md` wins over this file on any conflict.
- The parent agent is the only integrator.
- The parent-owned critical path stays inside `crates/xtask/src/agent_maintenance/**`.
- Only one new helper module may be introduced under `crates/xtask/src/agent_maintenance/`.
- No `.github/workflows/*.yml` edits are allowed in this milestone.
- No new infra, no new workflow family, no second contract store, no registry-owned freeform
  command arrays.
- `crates/xtask/data/agent_registry.toml` remains the only enrollment and release-watch source of
  truth.
- `execution_contract.executor` names the relay surface, not the maintained wrapper crate.
- `dispatch_workflow` stays materialized in the request packet for both `workflow_dispatch` and
  `packet_pr`.
- Registry `packet_pr` entries still omit `dispatch_workflow`; packet generation resolves it to
  `agent-maintenance-open-pr.yml`.
- Generated packet docs must be regenerated, never hand-maintained as parallel policy.
- Manual closeout stays manual.
- Legacy compatibility is read-path compatibility only. No newly generated packet may continue to
  emit `executor = "codex"`.

## Authority Model

Parent-only authority:

- interpret `PLAN.md`
- own `.runs/packet-first-contract-with-c-tail/**`
- own the serialized contract-core lane
- freeze checkpoints and lane launch SHAs
- integrate worker output
- run regeneration, final verification, and landing onto `staging`
- decide whether a discovered doc or packet surface is in scope for this milestone

Worker authority:

- operate only inside the assigned branch and worktree
- touch only the files owned by that lane
- run only the lane-scoped validation commands
- return `ready-for-parent`, `blocked`, or `no-op`
- never merge, never rebase other lanes, never write orchestration state, never widen scope

## Run-State Source Of Truth

Parent-owned run-state root:

```text
/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.runs/packet-first-contract-with-c-tail
```

Required parent-owned records:

- `baseline.json`
- `freeze.json`
- `lane-status.json`
- `artifacts/core-validation.md`
- `artifacts/doc-truth.md`
- `artifacts/regression-net.md`
- `artifacts/regen.md`
- `artifacts/final-gates.md`
- `acceptance.md`

Workers never write under `.runs/packet-first-contract-with-c-tail/**`.

## Worktree Strategy

Initial parent setup:

```bash
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-packet-first-contract/staging-live staging
git worktree add -b codex/packet-first-contract-core /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-packet-first-contract/parent-core origin/staging
```

`staging-live` remains clean until the parent is ready to land reviewed commits from `parent-core`.

Worker lanes are created only after the parent freezes checkpoint `C1` inside `parent-core`.

## Launch Order

### P0 - Parent Baseline And Freeze

Parent-only.

Actions:

- record current `origin/staging` SHA, local dirty state summary, and `PLAN.md` hash
- inspect the live seam:
  - `crates/xtask/src/agent_maintenance/{prepare.rs,request/automation.rs,docs.rs,watch.rs,execute.rs,mod.rs}`
  - `docs/specs/{maintenance-request-contract-v1.md,agent-registry-contract.md}`
  - `crates/xtask/tests/{agent_maintenance_prepare.rs,agent_maintenance_watch.rs,agent_maintenance_execute.rs,agent_maintenance_refresh.rs,agent_maintenance_closeout.rs,c4_spec_ci_wiring.rs,c0_spec_validate.rs}`
- record the already-confirmed contract drift:
  - `prepare.rs` hardcodes `executor = "codex"`
  - `prepare.rs` hardcodes `version_policy = "latest_stable_minus_one"`
  - `request/automation.rs` enforces milestone-1 executor semantics
  - `docs.rs` still carries a parallel automated rendering path
- initialize `freeze.json` with lane ownership, launch order, and stop conditions

Stop if:

- there are conflicting uncommitted changes inside the exact seam files
- more than one new helper module appears necessary
- `PLAN.md` changes after baseline capture and before coding starts

### P1 - Parent Critical Path: Contract Core

Parent-only. This is the milestone spine.

Owned surfaces:

- `crates/xtask/src/agent_maintenance/mod.rs`
- one new helper module, expected at `crates/xtask/src/agent_maintenance/contract_policy.rs`
- `crates/xtask/src/agent_maintenance/prepare.rs`
- `crates/xtask/src/agent_maintenance/request/automation.rs`
- `crates/xtask/src/agent_maintenance/docs.rs`
- `crates/xtask/src/agent_maintenance/watch.rs` only if the shared resolver is reused there
- `crates/xtask/src/agent_maintenance/execute.rs` only if constant ownership must move cleanly
- directly coupled seam tests needed to keep the checkpoint green:
  - `crates/xtask/tests/agent_maintenance_prepare.rs`
  - `crates/xtask/tests/agent_maintenance_watch.rs`
  - `crates/xtask/tests/agent_maintenance_refresh/automated_requests.rs`

Explicitly forbidden in P1:

- normative spec docs
- playbooks
- any workflow YAML
- broad regression-net work outside the directly coupled seam
- hand edits under `docs/agents/lifecycle/*-maintenance/**`

Required outcomes:

- one shared policy owner for:
  - canonical executor identity
  - resolved dispatch workflow
  - prompt template path
  - writable surfaces
  - read-only inputs
  - ordered commands
  - green gates
  - recovery metadata
- `prepare.rs` becomes a thin projection over that policy
- `request/automation.rs` validates shared steady-state truth plus legacy read alias
- `docs.rs` consumes the same shared truth for automated packet rendering
- `watch.rs` continues to materialize `agent-maintenance-open-pr.yml` for `packet_pr`
- no workflow edits leak in

Required checkpoint validation at `C1`:

```bash
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_refresh
```

`C1` freeze procedure in `parent-core`:

```bash
git rev-parse HEAD
```

Record that SHA as `C1_SHA` in `freeze.json`.

`C1` inspection pause:

- confirm only one new helper module was added
- confirm newly generated packets normalize to `execute-agent-maintenance`
- confirm no workflow changes leaked in
- confirm automated packet rendering is now sourced from shared contract truth

### Lane B - Normative Truth Surfaces

Launch only after `C1` freeze.

Create the worktree from the exact frozen parent SHA:

```bash
git worktree add -b codex/packet-first-contract-doc-truth /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-packet-first-contract/lane-b-docs "$C1_SHA"
```

Owned surfaces:

- `docs/specs/maintenance-request-contract-v1.md`
- `docs/specs/agent-registry-contract.md`
- narrow maintainer playbooks only if they still lie about executor or workflow-policy ownership:
  - `cli_manifests/codex/OPS_PLAYBOOK.md`
  - `cli_manifests/claude_code/OPS_PLAYBOOK.md`

Explicitly forbidden:

- any Rust source file
- any test file
- any workflow YAML
- any hand edits to generated packet docs

Mission:

- converge the normative docs to the frozen `C1` steady-state contract
- remove contradictions around executor identity and `packet_pr` materialized workflow truth
- patch only narrow live-topology or packet-truth lies in playbooks

Lane B required validation:

```bash
cargo test -p xtask --test c0_spec_validate
```

Lane B stop conditions:

- a required doc change depends on changing Rust after `C1`
- the lane would require broad runbook rewrite
- the lane would need to hand-edit generated packet docs

### Lane C - Regression Net

Launch only after `C1` freeze.

Create the worktree from the exact frozen parent SHA:

```bash
git worktree add -b codex/packet-first-contract-regressions /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-packet-first-contract/lane-c-tests "$C1_SHA"
```

Owned surfaces:

- `crates/xtask/tests/agent_maintenance_execute.rs`
- `crates/xtask/tests/agent_maintenance_closeout.rs`
- `crates/xtask/tests/agent_maintenance_closeout/request_and_schema.rs`
- `crates/xtask/tests/c4_spec_ci_wiring.rs`
- `crates/xtask/tests/c0_spec_validate.rs`
- additional `crates/xtask/tests/support/agent_maintenance_*` harness files only when required

Explicitly forbidden:

- any `crates/xtask/src/**` file
- any spec doc
- any workflow YAML
- any generated packet doc

Mission:

- add parity and compatibility coverage beyond the `C1` seam tests
- prove Codex and Claude packet parity explicitly
- prove shared-executor acceptance, legacy alias acceptance where intended, and wrong-executor rejection
- prove `packet_pr` carries `dispatch_workflow = "agent-maintenance-open-pr.yml"`
- preserve existing fail-closed runtime tests

Lane C required validation:

```bash
cargo test -p xtask --test agent_maintenance_execute
cargo test -p xtask --test agent_maintenance_closeout
cargo test -p xtask --test c4_spec_ci_wiring
cargo test -p xtask --test c0_spec_validate
```

Lane C stop conditions:

- the lane needs source-code changes to make the tests coherent
- fixture updates imply a different steady-state contract than `C1`
- closeout semantics would have to change

### Worker Handoff Contract

Before the parent considers merge, each worker must return exactly:

- lane id
- status: `ready-for-parent`, `blocked`, or `no-op`
- launch base SHA, which must equal `C1_SHA`
- changed files
- commands run
- validation results
- commit SHA
- unresolved risks or assumptions

The parent remains the only integrator. Workers do not merge, do not rebase other lanes, and do
not touch `staging-live`.

### P2 - Parent Integration And Regeneration

Parent-only.

Merge order:

1. integrate Lane B into `parent-core`
2. rebase Lane C onto the new `parent-core` tip only if necessary
3. integrate Lane C into `parent-core`

Why this order:

- Lane B locks the final normative wording and any narrow playbook truth
- Lane C should validate the final contract language and final field names

#### Live Maintenance Surface Decision Table

Parent determines what packet surfaces are live on this branch before regeneration.

Current branch facts already observed:

- committed automated live root exists for `codex`:
  - `docs/agents/lifecycle/codex-maintenance/**`
- no committed automated live root exists for `claude_code`
- `docs/agents/lifecycle/opencode-maintenance/**` exists, but it is a historical/manual/test
  surface and is not part of the current packet-first maintenance-contract milestone

Parent regeneration scope rule:

- regenerate committed automated maintenance roots that already exist on `staging` and are part of
  the current milestone
- do not create a new `docs/agents/lifecycle/claude_code-maintenance/**` root in this milestone
  just to mirror Codex
- do not regenerate `opencode-maintenance/**` as a live surface; only update tests or fixtures if
  a regression lane explicitly requires it

That means this milestone must regenerate `codex-maintenance/**` and must prove `claude_code`
parity through code and tests, not by inventing a new committed packet root.

#### Deterministic Regeneration Decision Tree

This repo does not currently expose a single replay command that round-trips an existing committed
automated request back through `prepare-agent-maintenance`. The parent therefore uses the committed
request packet itself as the replay source of truth.

For each in-scope committed automated maintenance root:

1. Load the committed request packet path:
   - example live path now:
     - `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
2. Read the exact committed request fields to reconstruct `prepare-agent-maintenance` arguments:
   - `agent_id`
   - `request_recorded_at`
   - `request_commit`
   - `opened_from`
   - `detected_release.current_validated`
   - `detected_release.latest_stable`
   - `detected_release.target_version`
   - `detected_release.detected_by`
   - `detected_release.dispatch_kind`
   - `detected_release.dispatch_workflow`
   - `detected_release.branch_name`
3. Decide the writer:
   - use `prepare-agent-maintenance` when the request packet itself must change
   - use `refresh-agent --request ... --write` only when the request packet is already current and
     only derived packet docs need re-rendering
4. After any `prepare-agent-maintenance --write`, run `refresh-agent --request ... --dry-run` as
   a parity check
   - if `refresh-agent --dry-run` still plans changes, stop: prepare and refresh are not
     converged yet
   - do not silently let both commands write different truth

For this milestone, the parent should assume `prepare-agent-maintenance` is required for
`codex-maintenance`, because the committed request currently contains at least one stale top-level
contract field: `execution_contract.executor = "codex"`.

#### Parent Regeneration Procedure

Example reconstruction workflow for `codex-maintenance`:

1. Inspect the committed request packet:

```bash
sed -n '1,220p' docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml
```

2. Reconstruct prepare arguments from the committed request packet fields and run dry-run first:

```bash
cargo run -p xtask -- prepare-agent-maintenance \
  --agent "codex" \
  --current-version "0.97.0" \
  --latest-stable "0.128.0" \
  --target-version "0.125.0" \
  --opened-from ".github/workflows/codex-cli-update-snapshot.yml" \
  --detected-by ".github/workflows/agent-maintenance-release-watch.yml" \
  --dispatch-kind "workflow_dispatch" \
  --dispatch-workflow "codex-cli-update-snapshot.yml" \
  --branch-name "automation/codex-maintenance-0.125.0" \
  --request-recorded-at "2026-05-07T06:24:24Z" \
  --request-commit "1e44a63ca3d2b0de4686725ca7a79793b90f8b57" \
  --dry-run
```

3. If the dry-run preview is sane, rerun with `--write`:

```bash
cargo run -p xtask -- prepare-agent-maintenance \
  --agent "codex" \
  --current-version "0.97.0" \
  --latest-stable "0.128.0" \
  --target-version "0.125.0" \
  --opened-from ".github/workflows/codex-cli-update-snapshot.yml" \
  --detected-by ".github/workflows/agent-maintenance-release-watch.yml" \
  --dispatch-kind "workflow_dispatch" \
  --dispatch-workflow "codex-cli-update-snapshot.yml" \
  --branch-name "automation/codex-maintenance-0.125.0" \
  --request-recorded-at "2026-05-07T06:24:24Z" \
  --request-commit "1e44a63ca3d2b0de4686725ca7a79793b90f8b57" \
  --write
```

4. Immediately run the packet-doc recovery path in dry-run mode only:

```bash
cargo run -p xtask -- refresh-agent \
  --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml \
  --dry-run
```

Expected result:

- either zero additional churn beyond the newly prepared request and packet docs
- or no planned changes at all

If `refresh-agent --dry-run` still plans packet-doc mutations after `prepare-agent-maintenance --write`,
stop and fix the code path divergence before landing.

#### Unsafe Reconstruction Stop Conditions

Stop regeneration immediately if any of the following occur:

- the committed request cannot be loaded as an automated request with `[detected_release]`
- any required replay field is missing, malformed, or no longer round-trips cleanly to a valid
  `prepare-agent-maintenance` invocation
- the reconstructed `--opened-from` or `--dispatch-workflow` no longer matches the registry or
  live workflow contract after `C1`
- `prepare-agent-maintenance --dry-run` plans to write outside the expected maintenance root and
  request-owned files
- `refresh-agent --dry-run` disagrees with the just-written prepare output
- regenerating `claude_code-maintenance/**` would require creating a new committed maintenance root
  not already present on `staging`

### P3 - Parent Final Validation, Landing, And Acceptance

Parent-only.

Final validation gate:

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

Landing procedure:

- keep `staging-live` clean until all gates pass on `parent-core`
- once green, fast-forward or cherry-pick the reviewed stack from `parent-core` onto `staging-live`
- run a final smoke diff on `staging-live`
- record landed commit SHAs and gate outputs in `acceptance.md`

## Conflict Flags And Merge Hotspots

Real hotspots the parent must inspect manually:

- `crates/xtask/src/agent_maintenance/docs.rs`
  - P1 changes renderer logic
  - any later doc wording or regeneration mismatch will show up here first
- request fixtures and schema assertions
  - `crates/xtask/tests/agent_maintenance_prepare.rs`
  - `crates/xtask/tests/agent_maintenance_refresh/automated_requests.rs`
  - `crates/xtask/tests/support/agent_maintenance_*`
  - executor string and recovery-command expectations are likely to churn together
- committed packet surfaces under `docs/agents/lifecycle/codex-maintenance/**`
  - request packet, `HANDOFF.md`, and `pr-summary.md` must stay lockstep
  - if only one of these changes, regeneration is incomplete
- `docs/specs/maintenance-request-contract-v1.md` vs code assertions
  - wording drift here can invalidate `c0_spec_validate` or future human interpretation even if
    code is green
- `request/automation.rs` vs execute/read-path tests
  - relaxing to accept legacy `codex` while normalizing new packets to `execute-agent-maintenance`
    is a genuine compatibility seam

## Context-Control Rules

- Read only `PLAN.md`, the normative spec files, the active-lane source files, and the active-lane
  tests.
- Do not expand into unrelated crates or unrelated `xtask` domains.
- Do not bulk-load `docs/`; open only the specific maintenance or spec surfaces required by the
  lane.
- Do not let worker lanes read or edit each other’s owned files after launch.
- Treat `.runs/packet-first-contract-with-c-tail/**` as parent-only.
- Treat generated packet docs as outputs, not as design documents.
- If a lane needs a file outside its ownership map, it must hand back `blocked`.

## Tests And Acceptance

### Core Contract

Done means all of the following are true:

- `prepare-agent-maintenance` generates `execution_contract.executor = "execute-agent-maintenance"`
  for new automated packets
- `request/automation.rs` accepts the shared executor and accepts legacy `codex` only where
  compatibility is intended
- `dispatch_workflow` remains materialized for both `workflow_dispatch` and `packet_pr`
- shared policy derivation owns executor, workflow resolution, read/write surfaces, ordered
  commands, green gates, and recovery guidance

Required evidence:

- `cargo test -p xtask --test agent_maintenance_prepare`
- `cargo test -p xtask --test agent_maintenance_watch`
- `cargo test -p xtask --test agent_maintenance_refresh`

### Docs

Done means all of the following are true:

- `docs/specs/maintenance-request-contract-v1.md` describes the same steady-state contract the code emits
- `docs/specs/agent-registry-contract.md` describes registry omission of `dispatch_workflow` for
  `packet_pr` while packet generation materializes the generic workflow
- no normative doc claims the maintained wrapper crate is the executor

Required evidence:

- manual diff inspection against `C1`
- `cargo test -p xtask --test c0_spec_validate`

### Generated Packet Surfaces

Done means all of the following are true:

- the committed `codex-maintenance` request packet is regenerated through `prepare-agent-maintenance`
- `HANDOFF.md` and `governance/pr-summary.md` are regenerated from the same packet truth
- `refresh-agent --request ... --dry-run` shows no additional doc drift after the prepare rewrite
- no new `claude_code-maintenance/**` committed root is invented in this milestone
- no live `opencode-maintenance/**` regeneration is performed

Required evidence:

- regenerated `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
- regenerated `docs/agents/lifecycle/codex-maintenance/HANDOFF.md`
- regenerated `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md`
- parent-recorded regen log in `artifacts/regen.md`

### Regression Net

Done means all of the following are true:

- Codex and Claude Code packet parity is tested explicitly
- shared executor acceptance is tested
- legacy executor alias acceptance is tested where intended
- wrong-executor rejection is tested
- `packet_pr` generic workflow materialization is tested
- prompt-digest and write-envelope fail-closed behavior remain green

Required evidence:

- `cargo test -p xtask --test agent_maintenance_execute`
- `cargo test -p xtask --test agent_maintenance_closeout`
- `cargo test -p xtask --test c4_spec_ci_wiring`
- `cargo test -p xtask --test c0_spec_validate`

### Workspace Boundary

Done means all of the following are true:

- only one new helper module was added
- no workflow YAML changed
- no second contract store was introduced
- closeout semantics remain manual
- final workspace gates pass from `parent-core` before landing to `staging`

Required evidence:

- `make fmt-check`
- `make clippy`
- `make check`
- `make test`

## Stop Conditions

Stop and re-plan immediately if any of the following occur:

- more than one new helper module is needed
- a workflow YAML edit seems required
- manual closeout semantics would have to change
- legacy committed packets cannot be kept readable with the planned compatibility alias
- the registry would need new freeform command arrays or a second policy store
- generated packet docs cannot be refreshed deterministically from request or prepare entrypoints
- `prepare-agent-maintenance` and `refresh-agent` cannot be brought to parity for automated packet docs
- regenerating `claude_code-maintenance/**` would require creating a new committed maintenance root
  not already present on `staging`
- a worker lane needs to touch parent-owned core files after `C1` freeze

## Follow-Up Milestone

This milestone ends with contract convergence only.

The explicit next milestone remains:

`worker/runbook convergence`

That follow-up owns worker-shape simplification, broader runbook cleanup, and any future transport
convergence. It does not belong in this session.
