# ORCH_PLAN - Prove The Generic Maintenance Lane With `opencode`

## Summary

| Item | Value |
| --- | --- |
| Current branch context | `staging` |
| Authoritative plan source | local `PLAN.md` in this workspace |
| `PLAN.md` status | `ready for implementation` |
| `PLAN.md` date | `2026-05-11` |
| Plan title | `Prove The Generic Maintenance Lane With opencode` |
| Plan baseline SHA | `4944c0f` |
| Workspace members | `agent_api`, `codex`, `claude_code`, `opencode`, `gemini_cli`, `aider`, `wrapper_events`, `xtask` |
| Existing `ORCH_PLAN.md` status | stale; fully replaced by this document |
| Run slug | `opencode-generic-maintenance-proof` |
| Parent role | sole critical-path integrator, sole proof operator, sole final verifier, sole merge decision-maker |
| Worker model | `GPT-5.4` with `reasoning_effort=high` |
| Worker concurrency cap | `0` before `C1`; maximum `2` active worker lanes after `C1` |

This orchestration plan is execution-focused. It assumes the local `PLAN.md` is authoritative because it is user-modified in the working tree. The parent agent must preserve that local truth, replace no planning inputs, and run the milestone against the current `staging` workspace state rather than recreating a clean branch from elsewhere.

The dependency graph is fixed:

1. Parent-only Phase 1 freezes `opencode` upstream truth, pointer truth, and release-watch enrollment truth.
2. Exactly two disjoint worker lanes run in parallel from the same frozen base:
   - watcher/registry/CI truth
   - automated request + packet normalization
3. Parent merges both lanes and only then runs proof capture.
4. After proof capture, one bounded post-proof docs/support publication worker lane runs from the proof-stable parent base.
5. Parent merges that lane and then runs the final verification and final merge decision.

## Hard Guards

- `PLAN.md` wins over this file on any conflict.
- Treat the current local `staging` workspace as authoritative because `PLAN.md` is modified in the working tree.
- `opencode` is the only new enrollment target in this milestone.
- `dispatch_kind = "packet_pr"` is mandatory for `opencode`.
- No bespoke worker workflow is allowed.
- Upstream release detection for `opencode` is frozen to GitHub Releases from `anomalyco/opencode` with `tag_prefix = "v"`.
- `cli_manifests/opencode/latest_validated.txt` must be promoted from `none` to a real semver baseline before enrollment is considered valid.
- The baseline to promote is the already-committed validated root truth: `1.4.11`.
- The proof bar is mandatory:
  - one real shared watcher -> generic opener run
  - one local `execute-agent-maintenance --dry-run` from the generated request
- No new source kind is allowed.
- No watcher topology redesign is allowed.
- No packet schema redesign is allowed.
- No full write-mode `opencode` maintenance closeout is allowed in this milestone.
- No `latest_supported.txt` promotion is allowed in this milestone.
- Support-matrix publication follow-through must happen after `latest_validated` promotion so published repo truth stops advertising `opencode` pointer promotion as `none`.
- Parent-only ownership covers:
  - Phase 1 freeze
  - worker launch approval
  - all merge decisions
  - proof execution
  - final verification
  - final merge/landing decision
- Worker lanes must stop immediately if they need:
  - a third parallel lane
  - edits to `PLAN.md` or `ORCH_PLAN.md`
  - edits outside their frozen file map
  - changes to Phase 1 frozen contract surfaces
  - a new workflow family, source kind, or packet contract

## Worktree Strategy

### Roots

- Parent live workspace:
  `/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api`
- Worker worktree root:
  `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-opencode-generic-maintenance-proof`
- Run-state root:
  `/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.runs/opencode-generic-maintenance-proof`

### Required Execution Surfaces

| Surface | Branch | Base | Purpose |
| --- | --- | --- | --- |
| `parent-live` | local `staging` | current workspace state | authoritative execution surface because local `PLAN.md` is modified and authoritative |
| `ws-a-watcher-registry-ci-truth` | `codex/opencode-watcher-registry-ci-truth` | `C1_SHA` | worker lane A after Phase 1 freeze |
| `ws-b-request-packet-normalization` | `codex/opencode-request-packet-normalization` | `C1_SHA` | worker lane B after Phase 1 freeze |
| `ws-c-post-proof-docs-publication` | `codex/opencode-post-proof-docs-publication` | `C4_SHA` | worker lane C after proof capture freeze |

No additional worker lanes are allowed. Proof capture and final merge decisions stay on `parent-live`. Lane C is sequential after proof capture and may not overlap with proof execution.

### Worktree Creation Rules

- Do not move `parent-live` off `staging`.
- Do not discard or restage the local `PLAN.md` change.
- Do not create worker worktrees until `C1_SHA` is frozen and committed on `parent-live`.
- Create `ws-a` and `ws-b` from the exact same `C1_SHA`.
- Create `ws-c` only from the frozen proof-stable `C4_SHA`.

### Worktree Creation Commands

Run only after `C1_SHA` exists:

```bash
git worktree add -b codex/opencode-watcher-registry-ci-truth \
  /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-opencode-generic-maintenance-proof/ws-a-watcher-registry-ci-truth \
  "$C1_SHA"

git worktree add -b codex/opencode-request-packet-normalization \
  /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-opencode-generic-maintenance-proof/ws-b-request-packet-normalization \
  "$C1_SHA"
```

Run only after `C4_SHA` exists:

```bash
git worktree add -b codex/opencode-post-proof-docs-publication \
  /Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-opencode-generic-maintenance-proof/ws-c-post-proof-docs-publication \
  "$C4_SHA"
```

## Run-State And Freeze Artifacts

### Authoritative Run-State Files

- `.runs/opencode-generic-maintenance-proof/tasks.json`
- `.runs/opencode-generic-maintenance-proof/freeze.json`
- `.runs/opencode-generic-maintenance-proof/lane-status.json`
- `.runs/opencode-generic-maintenance-proof/session-log.md`

### Required Derived Artifacts

- `.runs/opencode-generic-maintenance-proof/baseline.json`
- `.runs/opencode-generic-maintenance-proof/phase1-freeze.md`
- `.runs/opencode-generic-maintenance-proof/integration-notes.md`
- `.runs/opencode-generic-maintenance-proof/worker-briefs/ws-a.md`
- `.runs/opencode-generic-maintenance-proof/worker-briefs/ws-b.md`
- `.runs/opencode-generic-maintenance-proof/worker-briefs/ws-c.md`
- `.runs/opencode-generic-maintenance-proof/proof-gate.md`
- `.runs/opencode-generic-maintenance-proof/proof-run.md`
- `.runs/opencode-generic-maintenance-proof/final-gates.md`
- `.runs/opencode-generic-maintenance-proof/decision.md`
- `.runs/opencode-generic-maintenance-proof/acceptance.md`

### Task Queue Convention

`tasks.json` is the only queue of record. Minimum entry shape:

```json
{
  "id": "task/p1.3",
  "workstream": "WS-P1",
  "title": "Freeze opencode pointer and release-watch truth",
  "status": "pending|in_progress|blocked|completed",
  "owner": "parent|ws-a|ws-b|ws-c",
  "depends_on": ["task/p1.2"],
  "checkpoint": "C0|C1|C2|C3|C4|C5|null",
  "worktree": "parent-live|ws-a-watcher-registry-ci-truth|ws-b-request-packet-normalization|ws-c-post-proof-docs-publication",
  "notes": "short current state"
}
```

### Freeze Points

| Checkpoint | Meaning | Required contents |
| --- | --- | --- |
| `C0` | baseline captured before implementation | current `staging` HEAD SHA, dirty-file snapshot including `PLAN.md`, plan baseline SHA `4944c0f`, workspace-member inventory, worktree root, run-state root |
| `C1` | Phase 1 freeze complete; only legal worker launch base | promoted `latest_validated = 1.4.11`, frozen upstream source contract, committed `packet_pr` registry truth, parent validation results, worker file maps, lane launch approval |
| `C2` | both worker lanes integrated on parent | merged lane SHAs, no unresolved file-map overlap, regenerated `opencode` maintenance root stable, proof launch approval |
| `C3` | proof archived and stable | queue JSON, workflow dispatch summary, request SHA, dry-run output, proof path recorded, note that docs/support publication may now begin |
| `C4` | post-proof docs lane launch base frozen | proof-stable parent SHA, allowed lane C file map, lane C brief path, impacted checks to rerun after merge |
| `C5` | final acceptance frozen | merged lane C SHA, support-matrix refresh results, docs parity notes, targeted test matrix, `maintenance-watch` proof rerun status, `make preflight` result, final decision/acceptance |

## Workstream Plan

### WS-K0 - Parent Bootstrap

Type: parent-only  
Launch gate: immediate

Mission:

- preserve the authoritative local `PLAN.md` state
- initialize run-state artifacts
- capture `C0`

Tasks:

- record current `staging` workspace status without cleaning or rewriting it
- initialize `.runs/opencode-generic-maintenance-proof/`
- seed `tasks.json`, `freeze.json`, `lane-status.json`, and `session-log.md`
- write `baseline.json`
- freeze `C0`

Acceptance:

- local `PLAN.md` is explicitly recorded as authoritative
- no code changes start before `C0`
- worker branches are not created yet

### WS-P1 - Phase 1 Freeze: Upstream, Pointer, And Enrollment Truth

Type: parent-only  
Launch gate: `C0`

Owned surfaces:

- `cli_manifests/opencode/latest_validated.txt`
- `cli_manifests/opencode/VALIDATOR_SPEC.md`
- `cli_manifests/opencode/RULES.json` only if needed for pointer-truth coherence
- `crates/xtask/data/agent_registry.toml`
- `docs/specs/agent-registry-contract.md`

Mission:

- promote `latest_validated.txt` from `none` to `1.4.11`
- freeze upstream release detection to GitHub Releases `anomalyco/opencode` with `tag_prefix = "v"`
- land `maintenance.release_watch` enrollment for `opencode`
- require `dispatch_kind = "packet_pr"` and omit any registry `dispatch_workflow`
- make pointer truth, validator truth, and registry truth coherent before any parallel work

Required validation:

```bash
cargo run -p xtask -- codex-validate --root cli_manifests/opencode
cargo test -p xtask --test agent_registry -- --nocapture
```

Stop conditions:

- any proposed fix leaves `latest_validated.txt` at `none`
- any proposed fix introduces a new upstream source kind
- any proposed fix adds a bespoke `opencode` workflow or registry `dispatch_workflow`
- any proposed fix depends on parallel work before Phase 1 is merged

Acceptance:

- `maintenance-watch` can legally parse `opencode` root pointer truth
- `opencode` release-watch enrollment is committed and contract-valid
- `C1_SHA` is frozen and becomes the only legal worker base

Parent action at exit:

- write `phase1-freeze.md`
- freeze `C1`
- issue worker briefs for `ws-a` and `ws-b`

### WS-A - Watcher / Registry / CI Truth Lane

Type: worker  
Launch gate: `C1_SHA`

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-opencode-generic-maintenance-proof/ws-a-watcher-registry-ci-truth`

Owned surfaces:

- `crates/xtask/src/agent_maintenance/watch.rs`
- `crates/xtask/tests/agent_registry.rs`
- `crates/xtask/tests/agent_maintenance_watch.rs`
- `crates/xtask/tests/c4_spec_ci_wiring.rs`
- fixtures under `crates/xtask/tests/fixtures/**` only if required by those tests

Mission:

- widen registry/watcher/CI truth so the repo no longer behaves like only `codex` and `claude_code` can be enrolled
- prove `opencode` stale-agent queue emission under the shared watcher
- prove `packet_pr` resolves to `agent-maintenance-open-pr.yml` through the shared path
- keep watcher topology unchanged

Required validation:

```bash
cargo test -p xtask --test agent_registry -- --nocapture
cargo test -p xtask --test agent_maintenance_watch -- --nocapture
cargo test -p xtask --test c4_spec_ci_wiring -- --nocapture
```

Stop conditions:

- needs edits to Phase 1 frozen files
- needs edits under `docs/agents/lifecycle/opencode-maintenance/**`
- needs a new watcher topology or workflow family
- requires more than lane A ownership to make tests pass

Acceptance:

- `opencode` is included in release-watch truth and queue emission truth
- `packet_pr` materializes `agent-maintenance-open-pr.yml` through the shared lane
- CI/spec tests no longer encode a two-agent assumption

### WS-B - Automated Request + Packet Normalization Lane

Type: worker  
Launch gate: `C1_SHA`

Worktree:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/wt/unified-agent-api-opencode-generic-maintenance-proof/ws-b-request-packet-normalization`

Owned surfaces:

- `crates/xtask/src/agent_maintenance/contract_policy.rs`
- `crates/xtask/src/agent_maintenance/prepare.rs`
- `crates/xtask/src/agent_maintenance/request/automation.rs`
- `crates/xtask/src/agent_maintenance/docs.rs`
- `crates/xtask/tests/agent_maintenance_prepare.rs`
- `crates/xtask/tests/agent_maintenance_refresh/automated_requests.rs`
- `crates/xtask/tests/agent_maintenance_execute.rs`
- `docs/agents/lifecycle/opencode-maintenance/**`

Mission:

- normalize the live `opencode` maintenance root onto the automated request contract
- generate request truth with:
  - `artifact_version = "2"`
  - `trigger_kind = "upstream_release_detected"`
  - `basis_ref = "cli_manifests/opencode/latest_validated.txt"`
  - `opened_from = ".github/workflows/agent-maintenance-open-pr.yml"`
  - `requested_control_plane_actions = ["packet_doc_refresh"]`
  - `[detected_release]`
  - `[execution_contract]`
  - `dispatch_kind = "packet_pr"`
  - `dispatch_workflow = "agent-maintenance-open-pr.yml"`
  - `executor = "execute-agent-maintenance"`
- regenerate machine-owned packet docs from request truth, not legacy `drift_detected` prose
- make the generated request dry-run executable locally

Required validation:

```bash
cargo test -p xtask --test agent_maintenance_prepare -- --nocapture
cargo test -p xtask --test agent_maintenance_refresh -- --nocapture
cargo test -p xtask --test agent_maintenance_execute -- --nocapture
```

Stop conditions:

- needs edits to Phase 1 frozen files
- needs watcher topology or source-kind changes
- needs proof-archive or support-matrix publication edits before lane merge
- preserves legacy `drift_detected` request semantics in the live `opencode` maintenance root

Acceptance:

- the live `opencode` maintenance root matches the shared automated contract
- packet docs are request-derived and machine-owned
- the frozen request is locally dry-run executable

### WS-P2 - Parent Integration Gate

Type: parent-only  
Launch gate: both worker lanes running from `C1_SHA`

Mission:

- integrate exactly two worker lanes onto `parent-live`
- preserve the frozen dependency graph
- refuse overlap creep

Execution order:

1. Review lane A and lane B independently against their frozen file maps.
2. Merge a ready lane only if it does not cross into the other lane’s owned surfaces.
3. Re-run the earliest impacted targeted tests after each merge.
4. Merge the second lane.
5. Freeze `C2` only after both lanes are merged and `opencode` maintenance root output is stable.

Stop conditions:

- a worker lane edits outside its file map
- a worker lane needs a Phase 1 contract rewrite
- a merge would force a third lane or parent-side speculative redesign

Acceptance:

- both lanes merge onto one parent branch head
- watcher truth and packet truth agree on the same `opencode` basis
- proof may begin

### WS-P3 - Parent Proof Capture

Type: parent-only  
Launch gate: `C2`

Owned surfaces:

- `docs/agents/lifecycle/opencode-maintenance/governance/proof/**`
- temporary captured outputs summarized into committed proof artifacts

Mission:

- run one real shared watcher -> generic opener proof from the merged parent branch
- prove the generated request is executable with local dry-run

Execution:

1. Create the bounded stale scenario from the frozen `latest_validated = 1.4.11` baseline.
2. Run the shared watcher manually on `staging`.
3. Capture emitted queue truth showing:
   - `agent_id = "opencode"`
   - `dispatch_kind = "packet_pr"`
   - `dispatch_workflow = "agent-maintenance-open-pr.yml"`
4. Capture generic opener evidence for the generated request and packet docs.
5. Run:
   ```bash
   cargo run -p xtask -- execute-agent-maintenance --dry-run --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml
   ```
6. Archive required proof artifacts:
   - `watch-queue.json`
   - `workflow-dispatch-summary.md`
   - `request-sha256.txt`
   - `execute-dry-run.txt`
   - `proof-notes.md`
7. Freeze `C3`.
8. Freeze `C4` from the proof-stable parent branch and launch the post-proof docs/support publication lane.

Stop conditions:

- proof requires changing watcher/packet contract code afterward
- proof does not go through the shared watcher and generic opener
- dry-run fails against the generated request

Acceptance:

- one real proof archive exists under `docs/agents/lifecycle/opencode-maintenance/governance/proof/`
- the archived proof matches the merged parent branch truth
- docs/support publication may now start

### WS-C - Post-Proof Docs / Support Publication Lane

Type: worker  
Launch gate: `C4_SHA`

Owned surfaces:

- `docs/specs/agent-registry-contract.md`
- `docs/specs/maintenance-request-contract-v1.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `cli_manifests/support_matrix/current.json`
- `docs/specs/unified-agent-api/support-matrix.md`
- `crates/xtask/src/support_matrix.rs` only if required to keep publication output truthful

Mission:

- update docs/specs/operator/support publication surfaces to match the proof-stable parent state
- publish support-matrix truth after `latest_validated` promotion and after proof exists

Required validation:

```bash
cargo run -p xtask -- support-matrix
cargo run -p xtask -- support-matrix --check
```

Stop conditions:

- needs edits under `docs/agents/lifecycle/opencode-maintenance/governance/proof/**`
- needs edits outside docs/publication ownership
- needs watcher/packet contract code changes
- needs `latest_supported` promotion or packet-schema changes

Acceptance:

- docs/specs/operator outputs describe the proof-backed state
- support-matrix outputs no longer advertise `opencode` pointer promotion as `none`
- lane C stays inside its bounded docs/publication ownership

### WS-P4 - Parent Final Integration

Type: parent-only  
Launch gate: lane C returned `ready-for-parent`

Mission:

- review and merge the post-proof docs/support publication lane
- rerun impacted checks on the merged parent branch

Execution:

1. Review lane C strictly against its frozen file map.
2. Merge lane C onto `parent-live`.
3. Re-run the earliest impacted docs/publication checks.
4. Record merge and rerun results in `integration-notes.md`.

Stop conditions:

- lane C touched proof artifacts
- lane C touched code outside docs/publication ownership
- rerun checks show post-proof publication drift

Acceptance:

- post-proof docs/publication changes are integrated on the parent branch
- parent remains sole integrator before final gates

### WS-P5 - Parent Final Verification And Decision

Type: parent-only  
Launch gate: `WS-P4` merged

Mission:

- run the full final gate matrix on the proof-carrying, lane-C-merged parent branch
- make the final verification and merge decision

Execution:

1. Re-run targeted tests and publication checks on the merged parent branch.
2. Run `maintenance-watch --emit-json` as a final sanity check on merged truth.
3. Run `make preflight`.
4. Freeze `C5`.
5. Update `final-gates.md`, `decision.md`, and `acceptance.md`.

Stop conditions:

- proof archive is still changing
- final verification surfaces contradictory post-proof publication state
- final verification requires reopening lane topology or ownership

Acceptance:

- code, packet docs, proof archive, support publication, and specs tell one story
- all final gates pass on the proof-carrying, lane-C-merged parent branch
- parent can make the final merge decision

### Launch Order

1. `WS-K0`
2. `WS-P1`
3. freeze `C1`
4. launch `WS-A` and `WS-B` in parallel
5. `WS-P2` parent integration
6. freeze `C2`
7. `WS-P3` proof capture
8. freeze `C3`
9. freeze `C4`
10. launch `WS-C`
11. `WS-P4` parent final integration
12. `WS-P5` parent final verification and decision
13. freeze `C5`

Parallelism rules:

- no parallel worker activity before `C1`
- maximum `2` active worker lanes total
- only `WS-A` and `WS-B` may overlap
- proof capture cannot overlap with worker lanes
- `WS-C` is sequential after proof capture and may not overlap with proof execution
- final verification cannot start before `WS-C` merges

## Context-Control Rules

- Parent writes every worker brief.
- Every worker brief must include only:
  - workstream id
  - `C1_SHA` or `C4_SHA` as applicable
  - mission
  - exact owned file map
  - required validations
  - stop conditions
  - return contract
- Allowed run-state inputs:
  - `WS-A`
  - `.runs/opencode-generic-maintenance-proof/phase1-freeze.md`
  - `.runs/opencode-generic-maintenance-proof/worker-briefs/ws-a.md`
  - `WS-B`
  - `.runs/opencode-generic-maintenance-proof/phase1-freeze.md`
  - `.runs/opencode-generic-maintenance-proof/worker-briefs/ws-b.md`
  - `WS-C`
  - `.runs/opencode-generic-maintenance-proof/proof-run.md`
  - `.runs/opencode-generic-maintenance-proof/worker-briefs/ws-c.md`
- Workers may not read or edit:
  - `PLAN.md`
  - `ORCH_PLAN.md`
  - full `.runs/**` outside their allowed inputs
  - the other lane’s worktree
  - proof artifacts
  - support-matrix publication artifacts unless they are lane C owned outputs
- `WS-C` may edit only:
  - `docs/specs/agent-registry-contract.md`
  - `docs/specs/maintenance-request-contract-v1.md`
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
  - `cli_manifests/support_matrix/current.json`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `crates/xtask/src/support_matrix.rs` only if required
- `WS-C` is explicitly forbidden from touching:
  - `docs/agents/lifecycle/opencode-maintenance/governance/proof/**`
  - `crates/xtask/src/agent_maintenance/**`
  - `crates/xtask/src/agent_registry/**`
  - any workflow YAML
- Workers must return:
  - `workstream id`
  - `status = ready-for-parent|blocked|no-op`
  - `base checkpoint SHA`
  - changed files
  - commands run
  - exit code for each command
- If a worker discovers overlap, it must stop as `blocked` rather than self-expanding scope.
- If Phase 1 contract truth moves after worker launch, parent must stop both lanes, revise on `parent-live`, freeze a new `C1`, and relaunch from the new base.

## Tests And Acceptance

### Phase Gates

| Gate | Required proof |
| --- | --- |
| Phase 1 gate | `latest_validated = 1.4.11`, GitHub Releases `anomalyco/opencode` + `v`, `packet_pr` enrollment committed, no bespoke workflow |
| Parallel merge gate | lane A and lane B both merged on parent, no file-map overlap, live `opencode` maintenance root stable |
| Proof gate | archived watcher queue, generic opener evidence, request SHA, dry-run success |
| Post-proof lane launch gate | `C4_SHA` frozen from the proof-stable parent branch with bounded lane C ownership |
| Publication merge gate | lane C merged on parent and impacted docs/publication checks rerun successfully |
| Final gate | targeted tests, `maintenance-watch --emit-json`, support-matrix checks, `make preflight`, `C5` frozen |

### Required Final Commands

```bash
cargo run -p xtask -- support-matrix
cargo run -p xtask -- support-matrix --check
cargo test -p xtask --test agent_registry -- --nocapture
cargo test -p xtask --test agent_maintenance_watch -- --nocapture
cargo test -p xtask --test c4_spec_ci_wiring -- --nocapture
cargo test -p xtask --test agent_maintenance_prepare -- --nocapture
cargo test -p xtask --test agent_maintenance_refresh -- --nocapture
cargo test -p xtask --test agent_maintenance_execute -- --nocapture
cargo run -p xtask -- maintenance-watch --emit-json _ci_tmp/maintenance-watch.json
make preflight
```

### Milestone Acceptance Criteria

- `opencode` is the only newly enrolled release-watch target.
- `dispatch_kind = "packet_pr"` is committed for `opencode` and no registry `dispatch_workflow` is added.
- `latest_validated.txt` is promoted from `none` to `1.4.11` before enrollment truth is considered complete.
- Watcher/registry/CI truth covers a real third enrolled agent.
- The live `opencode` maintenance root is normalized onto the automated request contract.
- One real shared watcher -> generic opener proof is archived.
- The generated request passes local `execute-agent-maintenance --dry-run`.
- Support publication follows the pointer promotion and no longer reports `opencode` pointer promotion as `none`.
- Parent executes all critical-path integration, proof, final verification, and final merge decisions.

## Assumptions

- `cli_manifests/opencode/versions/1.4.11.json` already exists and is valid enough to back the root-pointer promotion.
- The repo’s shared watcher and generic opener workflows already exist and are the only intended transport surfaces for this milestone.
- The parent can manually run the shared watcher proof on `staging` and capture the resulting evidence.
- Any discovery that `opencode` requires a different acquisition source or a new schema is a stop-and-replan event, not an in-flight expansion.
- The current local `PLAN.md` remains authoritative for the duration of this session unless the user changes it again.
