# CLI Agent Maintenance Steady-State Plan

Status: Draft  
Scope: converging enrolled agent maintenance onto shared `packet_pr` transport plus explicit
non-TUI support uplift

This document is a planning surface, not a normative contract.

Use it to drive the next maintenance rewrite.
Do not treat it as the source of truth over `docs/specs/**`.

## Why this plan exists

The repo has already proved the important transport thesis:

- one shared watcher can detect stale enrolled agents
- one shared packet PR opener can open a maintenance lane
- one shared automated maintenance request schema can describe the lane
- one shared relay can execute bounded writes against that prepared packet

`opencode` proved that path successfully with `dispatch_kind = "packet_pr"`.

That means the remaining `codex` and `claude_code` worker-specific snapshot workflows are not the
destination. They are migration leftovers.

At the same time, the maintenance packet contract is still too conservative. It optimizes for:

- packet refresh
- version-scoped manifest refresh
- wrapper/backend edits only when artifact deltas force them

That is mechanically safe, but it is not the intended product.

The intended product is:

- shared release detection
- shared packet preparation
- shared packet PR opening
- shared bounded relay
- maintenance runs that can add newly available non-TUI support surface over time

This plan defines how to get there.

## Target steady state

The desired steady-state model is:

1. One shared scheduled release watcher:
   `.github/workflows/agent-maintenance-release-watch.yml`
2. One shared PR-opening transport:
   `.github/workflows/agent-maintenance-open-pr.yml`
3. One shared automated maintenance packet shape:
   `artifact_version = "2"` plus `[detected_release]` plus `[execution_contract]`
4. One shared local relay:
   `cargo run -p xtask -- execute-agent-maintenance ...`
5. One explicit manual closeout step:
   `cargo run -p xtask -- close-agent-maintenance ...`
6. One explicit support-uplift decision inside automated maintenance:
   detect missing non-TUI surface and land bounded wrapper/backend support when needed

In that steady state:

- `dispatch_kind = "packet_pr"` becomes the normal enrolled transport
- worker-specific `workflow_dispatch` maintenance transport is retired
- `refresh-agent` remains the manual drift and publication-refresh seam, not the automated
  upstream-release executor

## Non-goals

This rewrite does not aim to:

- add broad TUI parity through maintenance
- turn maintenance into an unbounded implementation lane
- merge manual drift maintenance and automated upstream-release maintenance into one command
- remove explicit maintainer closeout

The rewrite is intentionally narrower:

- retire per-agent worker transports
- keep the shared packet and relay
- widen automated maintenance so it can add missing non-TUI support surface

## Current state versus target state

| Topic | Current state | Target state |
| --- | --- | --- |
| Watcher | shared | shared |
| PR opener | shared for `packet_pr`, per-agent workers still active for `workflow_dispatch` | shared `agent-maintenance-open-pr.yml` for all enrolled automated release-watch maintenance |
| Packet schema | mostly shared | shared |
| Relay executor | shared `execute-agent-maintenance` | shared `execute-agent-maintenance` |
| Closeout | explicit manual | explicit manual |
| Automated maintenance job | packet + manifest refresh first | support uplift for missing non-TUI surface plus matching manifest/publication refresh |
| `codex` / `claude_code` workflows | transitional active transports | retired |

## Required contract changes

### 1. Rewrite maintenance success semantics

Current problem:

- `requested_control_plane_actions = ["packet_doc_refresh"]` is still the defining story
- the packet PR prompt still frames wrapper/backend edits as optional fallout from artifact deltas

Required change:

- preserve the control-plane action list as control-plane only
- add explicit relay-owned support-uplift semantics to the automated maintenance contract
- encode that automated upstream-release maintenance must audit new non-TUI surface before it can
  claim success

Primary files:

- `docs/specs/maintenance-request-contract-v1.md`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/agents/lifecycle/<agent_id>-maintenance/governance/execute-agent-maintenance-prompt.md`
- `crates/xtask/src/agent_maintenance/contract_policy.rs`

### 2. Converge transport on `packet_pr`

Current problem:

- registry truth still allows enrolled `workflow_dispatch` worker workflows for `codex` and
  `claude_code`
- the shared watcher still fans out to those workflows as live transports

Required change:

- migrate enrolled automated release-watch maintenance to `dispatch_kind = "packet_pr"`
- make `agent-maintenance-open-pr.yml` the normal opening workflow for all enrolled agents
- retire `codex-cli-update-snapshot.yml` and `claude-code-update-snapshot.yml` as steady-state
  maintenance transports

Primary files:

- `crates/xtask/data/agent_registry.toml`
- `.github/workflows/agent-maintenance-release-watch.yml`
- `.github/workflows/agent-maintenance-open-pr.yml`
- `.github/workflows/codex-cli-update-snapshot.yml`
- `.github/workflows/claude-code-update-snapshot.yml`
- `docs/specs/agent-registry-contract.md`

### 3. Add explicit non-TUI support-surface audit

Current problem:

- snapshot delta exists
- wrapper coverage exists
- backend support exists
- but the automated maintenance contract does not explicitly compare them as part of lane success

Required change:

- define the support surfaces maintenance is responsible for:
  commands, subcommands, flags, global flags, positional args
- explicitly exclude TUI-only surface
- require packet preparation or relay validation to surface whether support uplift is needed
- require the prompt and run summary to call out newly available missing support, not just artifact
  churn

Primary files:

- `docs/specs/maintenance-request-contract-v1.md`
- `docs/agents/lifecycle/*-maintenance/CI_WORKFLOWS_PLAN.md`
- `docs/agents/lifecycle/*-maintenance/OPS_PLAYBOOK.md`
- `crates/xtask/src/agent_maintenance/docs.rs`
- `crates/xtask/src/agent_maintenance/execute/*`
- wrapper coverage and manifest support derivation codepaths

### 4. Preserve boundedness

Current problem:

- widening maintenance scope can accidentally turn it into an unbounded codegen lane

Required change:

- keep `writable_surfaces` explicit and packet-owned
- keep closeout manual
- keep green gates explicit
- define support uplift as bounded to non-TUI surfaces discoverable from committed snapshots and
  wrapper/backend truth

Primary files:

- `crates/xtask/src/agent_maintenance/prepare.rs`
- `crates/xtask/src/agent_maintenance/execute/workflow.rs`
- `crates/xtask/src/agent_maintenance/execute/validate.rs`
- `docs/specs/maintenance-request-contract-v1.md`

## Proposed workstreams

## Workstream 1: Transport convergence

Goal:
- make `packet_pr` the default and retire worker-specific steady-state transport

Deliverables:
- registry enrollment updated for `codex` and `claude_code`
- watcher fanout still works with only shared packet PR dispatch
- worker-specific maintenance workflows retired or clearly marked historical

Acceptance:
- all enrolled automated release-watch agents use `dispatch_kind = "packet_pr"`
- watcher output resolves `dispatch_workflow = "agent-maintenance-open-pr.yml"` for all enrolled
  agents

## Workstream 2: Packet contract rewrite

Goal:
- update the maintenance request contract so automated maintenance is explicitly support-aware

Deliverables:
- contract language distinguishes packet refresh from maintenance success
- relay contract documents support-surface audit and non-TUI uplift expectations
- prompt template stops treating wrapper/backend changes as incidental

Acceptance:
- a prepared automated maintenance packet says what support uplift, if any, is expected
- the packet can still be validated and executed generically

## Workstream 3: Relay and packet rendering updates

Goal:
- make the generated packet docs and prompt describe the real intended job

Deliverables:
- updated `HANDOFF.md`
- updated `pr-summary.md`
- updated prompt template
- updated run summaries and validation reports, if needed

Acceptance:
- packet-owned docs no longer read like "refresh artifacts and maybe touch code"
- packet-owned docs read like "audit and land missing non-TUI support when present"

## Workstream 4: Regression coverage

Goal:
- prove the new steady state without reopening hidden contracts

Deliverables:
- registry tests for `packet_pr` convergence
- watcher tests for fully shared transport
- prepare/execute tests for support-uplift packet semantics
- at least one real proving example after migration

Acceptance:
- no enrolled automated release-watch agent depends on worker-specific transport
- fixtures and proof examples reflect the same steady-state story

## Migration order

1. Freeze the desired contract in docs first.
2. Update packet rendering and relay wording second.
3. Migrate registry dispatch truth for `codex` and `claude_code` to `packet_pr`.
4. Retire worker-specific maintenance transports.
5. Prove one post-migration real automated run on a previously worker-backed agent.

That order matters.

If transport is retired before the shared packet and relay semantics are rewritten, the repo will
standardize the old conservative maintenance behavior instead of the intended one.

## Risks

### Risk 1: Support uplift becomes too fuzzy

Mitigation:
- define the maintained support surface narrowly
- exclude TUI
- require packet-owned writable surfaces and green gates

### Risk 2: Worker retirement loses useful upstream acquisition logic

Mitigation:
- move any still-needed acquisition logic into shared packet preparation or shared transport before
  retiring worker-specific flows

### Risk 3: Contract docs say more than the relay can verify

Mitigation:
- land contract and relay changes together
- add tests at `prepare`, `watch`, and `execute` seams

## Success criteria

This rewrite is done when all of the following are true:

- enrolled automated upstream-release maintenance defaults to `packet_pr`
- the shared watcher plus shared packet PR opener is the normal transport for all enrolled agents
- `codex` and `claude_code` worker-specific maintenance transports are retired
- automated maintenance packets explicitly distinguish:
  - no support uplift needed
  - support uplift required for missing non-TUI surface
- relay-owned packet docs and prompt instruct support uplift when needed
- at least one real post-migration maintenance run proves the new steady state on an agent that
  previously used worker-specific transport

## Immediate next steps

1. Update the maintenance request contract wording in
   `docs/specs/maintenance-request-contract-v1.md`.
2. Update the atlas and operator guide only where they still imply `workflow_dispatch` is a
   co-equal long-term transport.
3. Draft the concrete registry and workflow migration diff for `codex` and `claude_code`.
4. Rewrite the generated maintenance prompt contract to make non-TUI support uplift explicit.
5. Only then migrate the worker-backed agents onto `packet_pr`.
