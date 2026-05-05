# PLAN - Enclose The Agent-Maintenance Execution Relay

Status: proposed
Date: 2026-05-05
Branch: `codex/recommend-next-agent`
Base branch: `main`
Repo: `atomize-hq/unified-agent-api`
Work item: `Land The Agent-Maintenance Execution Relay Follow-On`
Plan commit baseline: `ad6749a`
Design input: `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-design-20260505-103414.md`
Review addendum: `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-eng-review-test-plan-20260505-172540.md`
Supersedes: the earlier maintenance-CI registry/watcher `PLAN.md`, whose core watcher, packet, and worker-migration slices are already landed on this branch

## Objective

Keep the shared maintenance watcher and packet-first PR flow exactly as landed, then close the
next honest seam:

1. make the maintenance execution contract repo-owned instead of living only inside rendered prose
2. add one local maintainer/contributor relay command,
   `cargo run -p xtask -- execute-agent-maintenance --request ... --dry-run|--write`
3. guarantee that `HANDOFF.md` and `governance/pr-summary.md` are faithful projections of
   structured request truth
4. guarantee that relay write mode cannot escape the declared write envelope
5. stop before closeout and promotion so human review remains explicit

The non-negotiable outcome is:

```text
maintenance-watch queue
  -> worker or packet-only PR opens from generated packet docs
  -> maintainer reads one canonical HANDOFF.md
  -> execute-agent-maintenance --dry-run proves trust boundary
  -> execute-agent-maintenance --write runs the bounded Codex relay + green gates
  -> maintainer reviews diff
  -> explicit close-agent-maintenance remains manual
```

This is the missing product layer. The current packet-first PRs are useful, but they still ask the
maintainer to trust human-readable instructions more than repo-owned execution truth.

This document is the Milestone 2 follow-on from the approved design input: execution automation
stays local, bounded, reproducible, and explicitly stops before closeout.

## Why This, Why Now

This branch already landed the maintenance intake revamp:

- registry-owned `maintenance.release_watch` enrollment
- `maintenance-watch` queue emission
- `prepare-agent-maintenance --write`
- packet-first PR summaries
- shared watcher workflow
- migrated Codex and Claude worker entrypoints

That part is real. The remaining gap is narrower and sharper:

- the exact execution contract is still rendered markdown, not structured request truth
- there is no repo-owned dry-run proving what can be written, what commands will run, and what
  recovery path exists
- there is no repo-owned write path that runs the bounded coding-agent relay and the required
  gates before handing control back to the maintainer

If the repo skips this seam, every future maintenance PR still depends on tribal knowledge:
"read the packet carefully and do the right thing." That is not boring enough.

## Source Inputs

- Priority inputs:
  - `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-design-20260505-103414.md`
  - `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-eng-review-test-plan-20260505-172540.md`
  - `TODOS.md`
- Existing landed maintenance surfaces:
  - `crates/xtask/src/agent_maintenance/request.rs`
  - `crates/xtask/src/agent_maintenance/docs.rs`
  - `crates/xtask/src/agent_maintenance/prepare.rs`
  - `crates/xtask/src/agent_maintenance/watch.rs`
  - `crates/xtask/src/agent_maintenance/closeout/**`
  - `crates/xtask/src/main.rs`
- Existing live workflows:
  - `.github/workflows/agent-maintenance-release-watch.yml`
  - `.github/workflows/agent-maintenance-open-pr.yml`
  - `.github/workflows/codex-cli-update-snapshot.yml`
  - `.github/workflows/claude-code-update-snapshot.yml`
- Existing operator and packet surfaces:
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
  - `cli_manifests/codex/OPS_PLAYBOOK.md`
  - `cli_manifests/codex/CI_WORKFLOWS_PLAN.md`
  - `cli_manifests/codex/PR_BODY_TEMPLATE.md`
  - `cli_manifests/claude_code/OPS_PLAYBOOK.md`
  - `cli_manifests/claude_code/CI_WORKFLOWS_PLAN.md`
  - `cli_manifests/claude_code/PR_BODY_TEMPLATE.md`
- Existing verification surfaces:
  - `crates/xtask/tests/agent_maintenance_prepare.rs`
  - `crates/xtask/tests/agent_maintenance_refresh.rs`
  - `crates/xtask/tests/agent_maintenance_watch.rs`
  - `crates/xtask/tests/agent_maintenance_closeout.rs`
  - `crates/xtask/tests/c4_spec_ci_wiring.rs`
  - `crates/xtask/tests/support/agent_maintenance_harness.rs`

## Problem Statement

Current landed shape:

```text
shared watcher
  -> stale-agent queue
  -> worker or packet-only PR
  -> generated maintenance-request.toml + HANDOFF.md + pr-summary.md
  -> maintainer manually interprets HANDOFF.md
  -> maintainer manually drives Codex, gates, and recovery
  -> manual closeout
```

That leaves seven concrete problems:

1. the execution contract is rendered text, not structured request truth
2. there is no repo-owned dry-run that proves the exact write envelope before any mutation
3. there is no repo-owned preflight for local Codex availability and auth
4. worker PR creation recovery is implied, not explicit and machine-derived
5. `HANDOFF.md` and `pr-summary.md` are generated from shared context, but there is no explicit
   validation surface for prompt-digest and contract consistency
6. packet-only agents are deferred in principle, but the relay path is not frozen clearly enough
   to prevent accidental widening
7. closeout remains manual, which is correct, but the current packet does not make the
   stop-before-closeout boundary operationally obvious enough

Target shape:

```text
shared watcher
  -> stale-agent queue
  -> worker or packet-only PR
  -> structured maintenance request with execution_contract
  -> generated HANDOFF.md + derivative pr-summary.md
  -> execute-agent-maintenance --dry-run
       prints exact writable surfaces
       prints exact ordered commands
       prints exact green gates
       prints exact recovery path
       verifies local Codex preflight
  -> execute-agent-maintenance --write
       runs bounded Codex relay
       enforces path jail
       runs green gates
       stops before closeout
  -> maintainer reviews diff and runs explicit closeout
```

## Step 0 Scope Challenge

### Premise Check

The repo does not need:

- a new watcher architecture
- GitHub-hosted autonomous Codex execution
- automatic closeout
- promotion pointer writes inside the upstream-release relay
- relay support for every packet-only agent in milestone 1
- a general multi-provider execution framework

The repo does need:

- one structured execution contract in maintenance request truth
- one local dry-run/write relay surface
- one explicit recovery contract for PR-creation and preflight failure cases
- one strict path jail for relay write mode
- one clean rule that packet-only agents stay deferred

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| stale-agent detection | `crates/xtask/src/agent_maintenance/watch.rs` | Reuse unchanged as the only detector. This milestone does not revisit queue math. |
| automated request creation | `crates/xtask/src/agent_maintenance/prepare.rs` | Reuse and extend. It should become the sole writer of the structured execution contract. |
| packet rendering | `crates/xtask/src/agent_maintenance/docs.rs` | Reuse and extend. `HANDOFF.md` stays canonical and `pr-summary.md` stays derivative. |
| request parsing + validation | `crates/xtask/src/agent_maintenance/request.rs` | Reuse and extend. New execution-contract fields belong here. |
| closeout semantics | `crates/xtask/src/agent_maintenance/closeout/**` | Reuse. Preserve manual closeout and new request metadata without widening scope. |
| workflow topology | `.github/workflows/agent-maintenance-release-watch.yml`, `.github/workflows/agent-maintenance-open-pr.yml`, worker workflows | Reuse. Only harden packet timing, recovery, and concurrency semantics. |
| path-jail mutation primitives | `workspace_mutation::*` used by `prepare-agent-maintenance` | Reuse directly for relay write enforcement. Do not invent a second write-boundary mechanism. |
| repo-owned dry-run/write pattern | `runtime-follow-on --dry-run|--write`, `recommend-next-agent-research --dry-run|--write` | Reuse the same host-owned contract shape. Dry-run proves the packet. Write executes the bounded seam. |
| green gates | `codex-validate`, `support-matrix --check`, `capability-matrix --check`, `capability-matrix-audit`, `make preflight` | Reuse exactly. Relay adds orchestration, not new validation philosophy. |

### Minimum Complete Change

The minimum complete change set is:

1. extend automated maintenance requests with a structured `[execution_contract]` block
2. render `HANDOFF.md` and `governance/pr-summary.md` from that exact contract
3. add `execute-agent-maintenance --dry-run|--write`
4. enforce local Codex binary/auth preflight before write mode
5. enforce the declared write envelope during write mode
6. encode one explicit manual recovery path for PR creation or post-packet failure
7. update workflows, playbooks, and operator docs so the relay contract is the only story
8. add coverage for request parsing, rendering consistency, relay preview, path jail, and
   workflow recovery

Anything smaller still leaves the critical execution seam half-owned.

### Complexity Check

This work touches more than 8 files. That is acceptable because the seam is still one coherent
control-plane slice:

- maintenance request truth
- packet rendering
- local relay execution
- worker/open-pr recovery behavior
- operator packet documentation
- workflow contract tests

Complexity controls:

- initial relay executor is Codex only
- local relay only, no cloud runner
- keep manual closeout outside the relay
- keep packet-only agents on the existing packet-only path
- keep support/capability/release-doc publication surfaces out of scope

### Search / Build Decision

- **[Layer 1]** Reuse `workspace_mutation` path-jail enforcement instead of inventing a new file-boundary checker.
- **[Layer 1]** Reuse the existing `prepare-agent-maintenance` packet writer and make it the sole source of execution-contract truth.
- **[Layer 1]** Reuse the repo's existing dry-run/write command pattern from `runtime-follow-on` and `recommend-next-agent-research`.
- **[Layer 1]** Reuse worker PR-body generation from `governance/pr-summary.md`; do not bring back inline YAML PR bodies.
- **[Layer 3]** The missing product is not "better markdown." It is a machine-readable execution contract that markdown can project from.

### Distribution Check

This is still a repo-internal feature delivered through `xtask`, generated packet docs, and
GitHub PRs.

The real distribution surfaces are:

- maintainers running `execute-agent-maintenance --dry-run|--write`
- contributors consuming `HANDOFF.md` and `pr-summary.md`
- future repos inheriting the same bounded execution relay pattern

## Locked Decisions

1. The shared watcher and packet-first PR topology stay in place. This plan is a follow-on, not a redesign.
2. `HANDOFF.md` remains the canonical human entrypoint, but its contents are derived from structured request truth.
3. `governance/pr-summary.md` remains derivative. It must never become an independent source of truth.
4. `execute-agent-maintenance --dry-run` is the required trust step before `--write`.
5. `execute-agent-maintenance --write` may write only paths enumerated in the execution contract.
6. Relay write mode stops before `close-agent-maintenance`. Closeout remains explicit and manual.
7. Automated upstream-release relay stays packet-only with respect to promotion pointers and publication surfaces.
8. Packet-only agents remain deferred. They keep the open-PR path and must not accidentally inherit the relay.
9. PR-creation failure recovery is one explicit machine-derived path, not tribal knowledge.
10. The worker workflows remain responsible for generating artifacts and packet docs before PR creation. The local relay owns only the maintainer/contributor execution seam after the PR exists.
11. `execute-agent-maintenance --dry-run` may persist evidence only under a temp run root such as `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`; it must not mutate wrapper, manifest, or maintenance packet truth.
12. `execute-agent-maintenance --write` validates against one prepared dry-run baseline for the same `run_id`, so boundary enforcement is deterministic instead of reconstructing state from live markdown.

## Target Architecture

### Architecture Overview

```text
maintenance-watch
  -> stale_agents[]
  -> worker workflow or packet-only opener
  -> prepare-agent-maintenance --write
       writes maintenance-request.toml
       writes execution_contract
       writes HANDOFF.md
       writes governance/pr-summary.md
  -> PR opens from pr-summary.md

local maintainer/contributor
  -> execute-agent-maintenance --dry-run
       validates request + execution_contract
       validates local Codex preflight
       writes frozen run packet + run_id
       prints write envelope / gates / recovery
  -> execute-agent-maintenance --write
       reuses prepared run_id baseline
       runs bounded Codex relay
       enforces path jail + diff validation
       runs green gates
       prints explicit closeout command
```

### Branch And State Flow

```text
nightly watcher on staging
  -> one queue item per stale agent/version
  -> branch automation/<agent_id>-maintenance-<target_version>
  -> packet docs generated before PR creation
  -> PR body sourced from governance/pr-summary.md
  -> maintainer runs dry-run locally to prepare one frozen relay packet
  -> maintainer runs write mode against that prepared run_id
  -> relay stops before closeout
  -> maintainer reviews and closes lane explicitly
```

### Request Contract Additions

Extend `crates/xtask/src/agent_maintenance/request.rs` so automated upstream-release requests
carry structured relay truth.

The request shape becomes:

```toml
artifact_version = "2"
agent_id = "codex"
trigger_kind = "upstream_release_detected"
basis_ref = "cli_manifests/codex/latest_validated.txt"
opened_from = ".github/workflows/codex-cli-update-snapshot.yml"
requested_control_plane_actions = ["packet_doc_refresh"]
request_recorded_at = "2026-05-05T18:00:00Z"
request_commit = "ad6749a"

[detected_release]
detected_by = ".github/workflows/agent-maintenance-release-watch.yml"
current_validated = "0.97.0"
target_version = "0.98.0"
latest_stable = "0.99.0"
version_policy = "latest_stable_minus_one"
source_kind = "github_releases"
source_ref = "openai/codex"
dispatch_kind = "workflow_dispatch"
dispatch_workflow = "codex-cli-update-snapshot.yml"
branch_name = "automation/codex-maintenance-0.98.0"

[execution_contract]
executor = "codex"
prompt_template_path = "cli_manifests/codex/PR_BODY_TEMPLATE.md"
prompt_sha256 = "<sha256 of rendered coding-agent prompt>"
pr_summary_path = "docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md"
closeout_path = "docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json"
requires_manual_closeout = true
writable_surfaces = [
  "docs/agents/lifecycle/codex-maintenance/**",
  "crates/codex/**",
  "crates/agent_api/**",
  "cli_manifests/codex/artifacts.lock.json",
  "cli_manifests/codex/snapshots/0.98.0/**",
  "cli_manifests/codex/reports/0.98.0/**",
  "cli_manifests/codex/versions/0.98.0.json",
  "cli_manifests/codex/wrapper_coverage.json",
]
read_only_inputs = [
  "cli_manifests/codex/OPS_PLAYBOOK.md",
  "cli_manifests/codex/CI_WORKFLOWS_PLAN.md",
  "cli_manifests/codex/PR_BODY_TEMPLATE.md",
  ".github/workflows/codex-cli-update-snapshot.yml",
]
ordered_commands = [
  "cargo run -p xtask -- codex-validate --root cli_manifests/codex",
  "cargo run -p xtask -- support-matrix --check",
  "cargo run -p xtask -- capability-matrix --check",
  "cargo run -p xtask -- capability-matrix-audit",
  "make preflight",
]
green_gates = [
  "cargo run -p xtask -- codex-validate --root cli_manifests/codex",
  "cargo run -p xtask -- support-matrix --check",
  "cargo run -p xtask -- capability-matrix --check",
  "cargo run -p xtask -- capability-matrix-audit",
  "make preflight",
]

[execution_contract.recovery]
recreate_packet_command = "cargo run -p xtask -- prepare-agent-maintenance ... --write"
reopen_pr_body_path = "docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md"
reopen_pr_branch = "automation/codex-maintenance-0.98.0"
notes = [
  "If PR creation fails after packet generation, rerun packet creation and reopen the PR from the generated pr-summary path.",
  "If local Codex preflight fails, fix binary/auth and rerun execute-agent-maintenance --dry-run before write mode.",
]
```

Validation rules:

- `[execution_contract]` is required for `trigger_kind = "upstream_release_detected"`
- `executor` must be `codex` in milestone 1
- `prompt_sha256` must match the rendered prompt block used in `HANDOFF.md`
- `pr_summary_path` must live under the same maintenance root as the request
- `requires_manual_closeout` must be `true` in milestone 1
- `writable_surfaces` must be non-empty and repo-relative
- recovery branch/path values must match `[detected_release].branch_name` and the generated
  `pr-summary.md`

Backward-compatibility rule:

- manual maintenance requests remain valid without `[execution_contract]`
- historical automated requests may still load for read-only closeout/inspection
- new automated writes must emit the full relay contract

### Packet Rendering Rules

`prepare-agent-maintenance --write` becomes the sole writer of:

- `governance/maintenance-request.toml`
- `HANDOFF.md`
- `governance/pr-summary.md`

Rendering rules:

1. `HANDOFF.md`, `governance/pr-summary.md`, and the relay's frozen prompt artifact must all come from one shared execution-packet renderer over `prompt_template_path`; the relay must never scrape markdown to recover execution facts.
2. `HANDOFF.md` must render `writable_surfaces`, `read_only_inputs`, `ordered_commands`,
   `green_gates`, and `recovery` from the structured execution contract, not recomputed prose.
3. `governance/pr-summary.md` must point back to `HANDOFF.md` and must not contain any execution
   facts missing from the request truth.
4. The renderer must fail closed if prompt digest, maintenance root, or branch linkage is
   inconsistent.

### New xtask Surface: Execution Relay

Add:

```sh
cargo run -p xtask -- execute-agent-maintenance \
  --request docs/agents/lifecycle/<agent_id>-maintenance/governance/maintenance-request.toml \
  --dry-run

cargo run -p xtask -- execute-agent-maintenance \
  --request docs/agents/lifecycle/<agent_id>-maintenance/governance/maintenance-request.toml \
  --write \
  --run-id <prepared_run_id>
```

Responsibilities:

1. load and validate the maintenance request
2. require `[execution_contract]` for automated upstream-release requests
3. build one frozen execution packet from structured request truth plus the shared prompt renderer
4. preflight the local Codex binary and auth during dry-run and again before write mode
5. persist the frozen prompt, input contract, and repo baseline under a temp run root
6. preview the exact write envelope, ordered commands, green gates, and recovery path in dry-run
7. require `--run-id` for write mode so the command validates against one prepared baseline
8. run the frozen Codex prompt in write mode with a bounded write root
9. reject any write outside `execution_contract.writable_surfaces`
10. run the exact green gates from the execution contract after Codex returns and after boundary validation passes
11. stop before `close-agent-maintenance`
12. print the exact closeout command as the next manual step

Dry-run output contract:

- generated `run_id`
- temp run root
- request path
- agent id
- target version
- branch name
- executor
- prompt digest
- exact writable surfaces
- exact read-only inputs
- exact ordered commands
- exact green gates
- exact recovery path
- exact closeout command
- explicit statement that closeout is not run automatically

Write-mode contract:

- require an existing dry-run packet for the same `run_id`
- reload the frozen input contract and frozen prompt instead of regenerating them
- rerun the same request + preflight validation as dry-run
- invoke Codex using the exact frozen prompt
- diff the workspace against the prepared baseline while ignoring the temp run root
- fail if Codex writes outside the allowed surfaces
- fail if no runtime-owned write occurred
- run the declared green gates in order only after boundary validation succeeds
- write execution evidence, validation report, written-path list, and run summary for operator recovery
- stop and print recovery guidance on any failure
- never run `close-agent-maintenance`

### Relay Run Artifact Contract

Mirror the existing repo-owned dry-run/write host-surface pattern used by
`runtime-follow-on` and `recommend-next-agent-research`.

`execute-agent-maintenance --dry-run` writes only temp evidence under:

```text
docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/
  input-contract.json
  codex-prompt.md
  run-status.json
  run-summary.md
  validation-report.json
  written-paths.json
```

`execute-agent-maintenance --write` adds:

```text
  codex-execution.json
  codex-stdout.log
  codex-stderr.log
```

Rules:

1. The temp run root is excluded from repo diff validation.
2. The temp run root is not part of `execution_contract.writable_surfaces`; it is host-owned evidence, not maintainer-owned output.
3. The frozen prompt artifact must hash to `execution_contract.prompt_sha256`.
4. The relay prints `run_id` on dry-run and requires the same `run_id` on write.
5. `HANDOFF.md` remains the canonical human contract, but the relay consumes the frozen input contract and frozen prompt, never the rendered markdown.

### Workflow Hardening

Shared watcher workflow:

- keep the current queue-emission topology
- retain one queue item per stale agent/version
- add or preserve concurrency so the same agent/version cannot race the same branch concurrently

Generic packet-only PR opener:

- keep `prepare-agent-maintenance --write` before PR creation
- keep `governance/pr-summary.md` as `body-path`
- print one explicit manual recovery path if PR creation fails after packet generation

Codex worker workflow:

- keep artifact acquisition and snapshot generation behavior
- ensure packet generation happens after artifact outputs are ready and before PR creation
- ensure `governance/pr-summary.md` stays the PR body source
- emit the same explicit recovery path on PR-creation failure

Claude worker workflow:

- same hardening as Codex
- same packet-first PR-body contract
- same single recovery path

### File-Level Implementation Map

| Surface | Planned change |
| --- | --- |
| `crates/xtask/src/agent_maintenance/request.rs` | add `execution_contract` + recovery parsing and validation |
| `crates/xtask/src/agent_maintenance/docs.rs` | factor a shared execution-packet renderer so `HANDOFF.md`, `pr-summary.md`, and relay prompt artifacts stay byte-identical to the same request truth |
| `crates/xtask/src/agent_maintenance/prepare.rs` | emit the structured execution contract during automated packet creation |
| `crates/xtask/src/agent_maintenance/execute.rs` | new relay command, dry-run/write, prepared-run artifacts, preflight, prompt invocation, diff-based boundary validation, gate execution, and recovery evidence |
| `crates/xtask/src/agent_maintenance/mod.rs` | export `execute` module |
| `crates/xtask/src/main.rs` | expose `execute-agent-maintenance` |
| `crates/xtask/src/agent_maintenance/closeout/**` | preserve compatibility with new request metadata and manual-closeout boundary |
| `.github/workflows/agent-maintenance-release-watch.yml` | keep topology, harden concurrency/queue invariants if needed |
| `.github/workflows/agent-maintenance-open-pr.yml` | preserve packet-first PR creation, add explicit recovery semantics |
| `.github/workflows/codex-cli-update-snapshot.yml` | preserve worker behavior, add packet timing + recovery semantics |
| `.github/workflows/claude-code-update-snapshot.yml` | preserve worker behavior, add packet timing + recovery semantics |
| `docs/cli-agent-onboarding-factory-operator-guide.md` | document relay dry-run/write flow and manual-closeout boundary |
| `cli_manifests/codex/OPS_PLAYBOOK.md` | align local maintainer execution to the relay contract |
| `cli_manifests/claude_code/OPS_PLAYBOOK.md` | align local maintainer execution to the relay contract |

## Code Quality Rules For This Slice

1. Structured request truth owns execution facts. Rendered markdown does not.
2. Relay write-boundary enforcement must reuse the repo's existing path-jail machinery.
3. Workflow YAML may route work, but it may not duplicate execution-contract truth.
4. Packet-only and relay-enabled paths stay explicit. No hidden widening by convention.
5. Recovery behavior must be one explicit path per failure class, not scattered log text.
6. Manual closeout remains explicit. Do not smuggle closeout into relay write mode.
7. The relay may consume shared renderer outputs, but it may not parse `HANDOFF.md` or `pr-summary.md` to reconstruct machine truth.

## Implementation Slices

### Slice 1. Execution-Contract Schema

Ship the request and renderer schema first.

Done means:

- automated requests carry `[execution_contract]`
- `HANDOFF.md` and `pr-summary.md` project from that exact truth
- prompt digest and path linkage validate cleanly

### Slice 2. Local Relay Command

Add `execute-agent-maintenance --dry-run|--write`.

Done means:

- dry-run emits one frozen execution packet and one `run_id` without mutating repo-owned maintenance or wrapper surfaces
- write mode requires that prepared `run_id` and reuses the frozen prompt/input contract instead of regenerating them
- write mode invokes Codex with bounded writes
- boundary validation combines the declared write envelope with an actual repo diff against the prepared baseline
- path jail blocks any write outside `writable_surfaces`
- run evidence is persisted for recovery and operator audit
- manual closeout remains outside the command

Implementation sequence:

1. **2A. Prepare relay packet**
   - load `maintenance-request.toml`
   - validate `[execution_contract]`, branch linkage, prompt digest, and maintenance root linkage
   - render the exact prompt from the shared execution-packet renderer
   - snapshot the repo baseline, then persist `input-contract.json`, `codex-prompt.md`, and dry-run summaries under `docs/agents/.uaa-temp/agent-maintenance/runs/<run_id>/`
2. **2B. Execute bounded write**
   - require `--run-id`
   - rerun Codex preflight
   - execute the frozen prompt
   - diff the repo against the prepared baseline while excluding the temp run root
   - fail closed on any out-of-bounds write, missing required write, or prompt-digest mismatch
3. **2C. Validate and hand off**
   - run the exact green gates from the execution contract in order
   - persist execution evidence and validation summaries
   - print the exact closeout command plus the machine-derived recovery path
   - stop before closeout

### Slice 3. Workflow Recovery Hardening

Harden the shared opener and specialized workers.

Done means:

- packet generation always precedes PR creation
- PR body always comes from generated `pr-summary.md`
- recovery path is explicit when PR creation fails
- shared queue/branch concurrency remains one stale agent/version -> one branch/PR

### Slice 4. Operator Docs And Playbooks

Update all maintainer-facing docs.

Done means:

- operator guide tells the same story as the code
- playbooks point to relay dry-run/write, not manual translation
- packet-only agents remain explicitly deferred

### Slice 5. Verification Closeout

Prove schema, relay, workflows, and docs together.

Done means:

- request parsing and renderer consistency are covered
- relay preview and write-boundary enforcement are covered
- workflow contract tests match the live topology
- docs and playbooks match the final command surface

## Test Review

### Code Path Coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/xtask/src/agent_maintenance/request.rs
    ├── parse valid execution_contract block
    ├── reject missing executor for automated requests
    ├── reject missing prompt_sha256
    ├── reject writable_surfaces outside repo-relative paths
    ├── reject recovery branch/path mismatch
    └── preserve compatibility for manual requests without execution_contract

[+] crates/xtask/src/agent_maintenance/docs.rs
    ├── HANDOFF renders execution_contract exactly
    ├── pr-summary remains derivative of the same truth
    ├── prompt digest matches rendered prompt block
    └── malformed execution_contract fails closed

[+] crates/xtask/src/agent_maintenance/prepare.rs
    ├── emits execution_contract for automated requests
    ├── populates writable_surfaces deterministically
    ├── populates recovery contract deterministically
    └── keeps packet-only exclusions explicit

[+] crates/xtask/src/agent_maintenance/execute.rs
    ├── dry-run writes frozen input-contract + prompt artifacts and prints exact write envelope and gates
    ├── preflight fails when codex binary is missing
    ├── preflight fails when auth is missing
    ├── write mode requires a prepared run_id baseline
    ├── write mode refuses writes outside writable_surfaces
    ├── write mode fails if frozen prompt digest and request truth diverge
    ├── write mode runs green gates in order
    └── write mode stops before closeout

[+] workflow migrations
    ├── packet-only opener uses generated pr-summary.md
    ├── codex worker prepares packet before PR creation
    ├── claude worker prepares packet before PR creation
    └── watcher/workers preserve one stale agent/version -> one branch/PR
```

### User Flow Coverage

```text
USER FLOW COVERAGE
===========================
[+] Shared watcher -> PR path
    ├── stale codex version -> codex worker -> PR opens with generated pr-summary
    ├── stale claude version -> claude worker -> PR opens with generated pr-summary
    ├── packet_pr agent -> generic opener -> packet-only PR path
    └── repeat run for same agent/version -> same branch/PR, not duplicate spray

[+] Maintainer relay trust path
    ├── maintainer reads HANDOFF.md only
    ├── dry-run prints exact writable surfaces
    ├── dry-run prints exact ordered commands
    ├── dry-run prints exact green gates
    ├── dry-run prints exact recovery path
    ├── dry-run emits a run_id and frozen prompt packet
    └── dry-run states that closeout remains manual

[+] Relay execution path
    ├── write mode reuses the prepared run_id baseline
    ├── write mode passes Codex preflight
    ├── write mode applies bounded changes only
    ├── write mode runs green gates
    ├── write mode fails closed on out-of-bounds writes
    └── maintainer runs explicit closeout afterward

[+] Recovery path
    ├── PR creation fails after packet generation
    ├── maintainer reruns the generated recovery path
    └── packet docs remain the same truth during recovery
```

### Test Files To Add Or Update

| Surface | Test location | Required assertions |
| --- | --- | --- |
| execution-contract parsing | `crates/xtask/tests/agent_maintenance_refresh.rs` or a request-focused sibling | valid/invalid `[execution_contract]` parsing, compatibility rules, recovery linkage validation |
| packet generation | `crates/xtask/tests/agent_maintenance_prepare.rs` | generated request includes execution contract, prompt digest, writable surfaces, green gates, and recovery path |
| relay command | `crates/xtask/tests/agent_maintenance_execute.rs` | dry-run packet artifacts, missing codex/auth preflight, run-id requirement, path-jail rejection, no-op write rejection, closeout omission, gate ordering |
| renderer consistency | `crates/xtask/tests/agent_maintenance_prepare.rs` | `HANDOFF.md` and `pr-summary.md` remain byte-consistent with request truth |
| closeout compatibility | `crates/xtask/tests/agent_maintenance_closeout.rs` | closeout preserves new request metadata and still requires explicit human step |
| workflow contract | `crates/xtask/tests/c4_spec_ci_wiring.rs` | packet-first PR bodies, recovery semantics, shared concurrency/branch invariants, no stale watcher references |
| harness support | `crates/xtask/tests/support/agent_maintenance_harness.rs` | fixture helpers for request v2 execution-contract generation, prepared run roots, and relay-path testing |

### Commands That Must Pass Before Landing

```sh
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_refresh
cargo test -p xtask --test agent_maintenance_execute
cargo test -p xtask --test agent_maintenance_closeout
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test c4_spec_ci_wiring
make preflight
```

## Failure Modes Registry

| Codepath | Realistic failure | Test coverage required | Error handling required | User-visible outcome |
| --- | --- | --- | --- | --- |
| request parsing | malformed or partial `[execution_contract]` block | yes | fail closed with request path + missing field | relay refuses to start |
| packet rendering | prompt digest no longer matches rendered prompt block | yes | fail closed before packet write | no misleading `HANDOFF.md` lands |
| relay preflight | local `codex` binary missing or auth missing | yes | dry-run/write stops with exact fix guidance | maintainer sees one explicit remediation path |
| relay baseline | maintainer runs `--write` without a prepared `run_id` or with stale baseline state | yes | fail closed and point to the exact dry-run to rerun | maintainer does not guess which prompt/baseline is in force |
| relay path jail | Codex writes outside `writable_surfaces` | yes | fail closed and report offending path | no out-of-bounds mutation survives |
| worker PR creation | artifacts + packet docs generated but PR open fails | yes | print one explicit recovery path using generated `pr-summary.md` | maintainer can reopen without guessing |
| watcher concurrency | same stale agent/version dispatched twice | yes | deterministic branch concurrency | same branch/PR reused, not duplicated |
| closeout boundary | maintainer assumes relay already finalized closeout | yes | dry-run/write explicitly states closeout is manual | no false-positive closed lane |

Any path that mutates outside the declared write envelope or silently finalizes closeout is a
critical gap and blocks landing.

## Performance Review

This slice is operationally light in dry-run mode and intentionally heavier in write mode.

Dry-run must stay cheap:

- parse request
- validate digest/linkage
- preflight local Codex availability
- write the frozen temp run packet
- print the contract

Write mode is allowed to be heavier because it is the bounded execution seam:

- one local Codex run
- one repo diff against the prepared baseline
- one ordered gate sequence
- zero network fanout beyond what the existing workflow/product already requires

The real performance risks are:

- repeated local retries because preflight guidance is unclear
- unnecessary re-runs because recovery is ambiguous
- gate cost hiding inside repeated failed write attempts

Controls:

- fail fast on missing binary/auth in dry-run
- fail fast on request/render mismatch before Codex runs
- fail fast on out-of-bounds writes
- keep closeout outside the command so reruns stay explicit

## NOT in scope

- GitHub-hosted or cloud-hosted autonomous maintenance execution
- automatic closeout
- promotion pointer updates such as `latest_validated.txt` or `min_supported.txt`
- support/capability/release-doc publication refresh in the upstream-release relay
- widening the relay path to packet-only agents that do not yet have a complete execution basis
- a generic multi-executor framework beyond Codex
- redesigning the shared watcher or stale-agent queue contract

## Worktree Parallelization Strategy

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Execution-contract schema | `crates/xtask/src/agent_maintenance/request.rs`, `crates/xtask/tests/` | — |
| Shared execution-packet renderer | `crates/xtask/src/agent_maintenance/docs.rs`, `crates/xtask/tests/` | Execution-contract schema |
| Packet generation | `crates/xtask/src/agent_maintenance/prepare.rs`, `crates/xtask/tests/` | Shared execution-packet renderer |
| Relay command | `crates/xtask/src/agent_maintenance/execute.rs`, `crates/xtask/src/main.rs`, `crates/xtask/tests/` | Shared execution-packet renderer |
| Workflow recovery hardening | `.github/workflows/`, `crates/xtask/tests/c4_spec_ci_wiring.rs` | Packet generation |
| Closeout compatibility | `crates/xtask/src/agent_maintenance/closeout/`, `crates/xtask/tests/` | Execution-contract schema |
| Docs + playbook closeout | `docs/`, `cli_manifests/*/OPS_PLAYBOOK.md`, `PLAN.md` | Packet generation + relay + workflow semantics stable |

### Parallel Lanes

Lane A: Execution-contract schema

Lane B: Shared execution-packet renderer
Sequential inside the lane because it owns the shared renderer contract that both packet docs and the relay consume.

Lane C: Packet generation
Sequential inside the lane because it owns `prepare.rs` and prepare-focused tests.

Lane D: Relay command
Sequential inside the lane because it owns `execute.rs`, `main.rs`, and relay-specific tests.

Lane E: Closeout compatibility
Independent after schema freeze, because it focuses on `closeout/**` and request compatibility.

Lane F: Workflow recovery hardening
Waits for packet-generation truth, then owns workflow YAML and `c4_spec_ci_wiring.rs`.

Lane G: Docs + playbook closeout
Waits until the relay contract and workflow semantics are stable.

### Execution Order

1. Launch Lane A first.
2. After A stabilizes, launch B + E in parallel worktrees.
3. Merge B before launching C + D, because both packet generation and relay execution depend on the shared renderer contract.
4. After B lands, launch C + D in parallel worktrees.
5. Merge E whenever its compatibility tests are green because it only depends on the schema.
6. Merge C before F, because workflows depend on final packet-generation truth.
7. Merge D once relay tests are green.
8. Launch and merge F.
9. Run G last, then full verification.

### Conflict Flags

- Lanes B, C, D, and E all touch `crates/xtask/tests/`. Keep test-file ownership explicit.
- Lanes C and D both depend on the shared renderer contract from `docs.rs`; do not let either fork its own prompt-building helper.
- Lanes F and G both touch maintainer-facing workflow semantics. Do docs last.

## Completion Summary

- Step 0: Scope Challenge, scope reduced to the execution relay follow-on instead of reopening watcher architecture
- Architecture Review: locked on structured execution truth, local Codex relay, bounded writes, explicit recovery, and manual closeout
- Code Quality Review: request truth owns execution semantics, markdown stays projection-only
- Test Review: request parsing, renderer consistency, relay dry-run/write, workflow recovery, and closeout boundary all have explicit homes
- Performance Review: dry-run stays cheap, write mode is bounded and explicit
- NOT in scope: written
- What already exists: written
- Failure modes: critical gaps identified and blocked
- Parallelization: 7 steps, with schema -> shared renderer as the serial spine, then packet generation + relay + closeout compatibility split into safe worktree lanes

That is the whole job. Do not redesign the watcher. Do not widen to cloud execution. Make the
existing maintenance PR packet honest enough that a maintainer can trust one dry-run, one write
run, and one explicit closeout step.
