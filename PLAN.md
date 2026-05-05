# PLAN - Make Agent-Maintenance CI Registry-Driven And Contributor-Ready

Status: proposed
Date: 2026-05-05
Branch: `codex/recommend-next-agent`
Base branch: `main`
Repo: `atomize-hq/unified-agent-api`
Work item: `Land The Agent-Maintenance CI Architecture Revamp`
Plan commit baseline: `65ed435`
Design input: `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-design-20260505-103414.md`
Supersedes: the completed recommendation-research `PLAN.md` that no longer reflects the active milestone

## Objective

Replace the repo's per-agent release-watch sprawl with one registry-driven maintenance intake
surface that:

1. reads watch-enabled agents from repo truth
2. computes the target upgrade version from repo-owned upstream metadata, starting with
   `latest stable - 1`
3. opens exactly one PR per stale agent
4. materializes a contributor-ready maintenance packet with the exact prompt, commands, touched
   surfaces, and green gates needed to complete the upgrade work
5. reuses the existing maintenance lane and existing specialized update workflows where they
   already exist
6. defers autonomous "run the AI agent and push the branch for me" execution to the next
   milestone

The non-negotiable outcome is:

```text
agent_registry maintenance.release_watch metadata
  -> maintenance-watch xtask command
  -> stale-agent queue JSON
  -> one stale agent fanout at a time
  -> prepare-agent-maintenance packet creation
  -> worker PR or packet-only PR
  -> maintainer or contributor follows the exact packet
  -> existing validation and maintenance closeout surfaces
```

Initial milestone boundary:

- `codex` and `claude_code` are the first watch-enabled agents
- generic `packet_pr` support lands in the same milestone
- no existing packet-only agent is enrolled until its packet basis docs are complete

That is the smallest honest landing. The architecture becomes dynamic now. Broader agent rollout
comes after the surface is real.

## Why This, Why Now

The repo already owns lifecycle enrollment, runtime follow-on, publication refresh, proving-run
closeout, maintenance drift detection, maintenance refresh, and maintenance closeout.

What it still does not own cleanly is the top of the maintenance funnel. Codex and Claude Code
each have bespoke release-watch logic and bespoke automation topology. New agents would either add
more permanent YAML or wait indefinitely for maintenance automation.

That is the last boring but critical control-plane gap before `goose` can become the first honest
end-to-end lifecycle proving target in follow-on work already captured in `TODOS.md`.

## Source Inputs

- Priority source:
  - `TODOS.md`
  - `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-design-20260505-103414.md`
- Normative contracts:
  - `docs/specs/agent-registry-contract.md`
  - `docs/specs/cli-agent-onboarding-charter.md`
  - `docs/specs/unified-agent-api/support-matrix.md`
- Live operator procedure:
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
- Existing maintenance surfaces:
  - `crates/xtask/src/agent_maintenance/request.rs`
  - `crates/xtask/src/agent_maintenance/refresh.rs`
  - `crates/xtask/src/agent_maintenance/docs.rs`
  - `crates/xtask/src/agent_maintenance/closeout/**`
  - `crates/xtask/src/agent_maintenance/drift/**`
- Existing seed automations:
  - `.github/workflows/codex-cli-release-watch.yml`
  - `.github/workflows/codex-cli-update-snapshot.yml`
  - `.github/workflows/claude-code-release-watch.yml`
  - `.github/workflows/claude-code-update-snapshot.yml`
- Existing agent-local operator guidance:
  - `cli_manifests/codex/OPS_PLAYBOOK.md`
  - `cli_manifests/codex/CI_WORKFLOWS_PLAN.md`
  - `cli_manifests/codex/PR_BODY_TEMPLATE.md`
  - `cli_manifests/claude_code/OPS_PLAYBOOK.md`
  - `cli_manifests/claude_code/CI_WORKFLOWS_PLAN.md`
  - `cli_manifests/claude_code/PR_BODY_TEMPLATE.md`
- Existing verification surfaces:
  - `crates/xtask/tests/agent_registry.rs`
  - `crates/xtask/tests/agent_maintenance_drift.rs`
  - `crates/xtask/tests/agent_maintenance_refresh.rs`
  - `crates/xtask/tests/agent_maintenance_closeout.rs`
  - `crates/xtask/tests/c4_spec_ci_wiring.rs`

## Problem Statement

Current shape:

```text
codex release watch workflow
  -> hardcoded GitHub release query
  -> dispatch codex update workflow
  -> open codex-only automation PR

claude release watch workflow
  -> hardcoded GCS stable pointer query
  -> dispatch claude update workflow
  -> open claude-only automation PR

other onboarded agents
  -> no shared release-watch enrollment
  -> only enter maintenance after humans notice drift
```

That leaves six concrete problems:

1. release-watch enrollment is not driven by `agent_registry.toml`
2. upstream version-query policy is not canonical repo truth
3. the repo has no shared stale-agent queue artifact
4. maintenance packets are not opened automatically from upstream-version drift
5. contributor instructions are inconsistent across agent update PRs
6. every new agent risks adding more permanent YAML instead of inheriting the shared topology

Target shape:

```text
shared maintenance watch workflow on staging
  -> cargo run -p xtask -- maintenance-watch --emit-json <queue>
  -> reads watch-enabled agents from registry
  -> computes stale agents from repo-owned version policy
  -> fans out one agent at a time

per-agent fanout
  -> dispatch specialized worker workflow
     OR run generic packet-only PR workflow

packet creation
  -> cargo run -p xtask -- prepare-agent-maintenance --write ...
  -> writes maintenance-request.toml
  -> writes generated maintenance packet docs

worker or contributor
  -> follows exact prompt + commands from the packet
  -> runs existing green gates
  -> closes maintenance with existing xtask surfaces
```

## Step 0 Scope Challenge

### Premise Check

The repo does not need:

- a multi-agent bundle PR model
- a cloud execution runner in this milestone
- a generic replacement for every specialized update workflow before the watcher lands
- a lifecycle model rewrite
- a plugin-style upstream source framework
- promotion-policy changes beyond codifying the target version rule

The repo does need:

- one registry-owned maintenance enrollment and upstream query contract
- one shared stale-agent detector and queue emitter
- one repo-owned way to create a maintenance packet from version drift
- one shared watcher/orchestrator workflow
- migration of Codex and Claude Code off bespoke release-watch scheduling onto the shared watcher
- one honest rule for which agents are watch-enabled now vs merely onboarded

### What Already Exists

| Sub-problem | Existing surface | Reuse decision |
| --- | --- | --- |
| current maintenance truth | `crates/xtask/data/agent_registry.toml` + `docs/specs/agent-registry-contract.md` | Reuse and extend. Do not invent a second metadata file. |
| maintenance drift detection | `cargo run -p xtask -- check-agent-drift --agent <id>` + `crates/xtask/src/agent_maintenance/drift/**` | Reuse directly as the maintenance baseline checker. |
| maintenance request validation | `crates/xtask/src/agent_maintenance/request.rs` | Reuse and extend. New automated requests must still flow through the same validator. |
| maintenance packet rendering | `crates/xtask/src/agent_maintenance/docs.rs` + `refresh.rs` | Reuse the renderer. Do not fork a second packet-doc system for release watch. |
| maintenance closeout | `crates/xtask/src/agent_maintenance/closeout/**` | Reuse and extend so new trigger kinds stay truthful through closeout. |
| per-agent update execution seeds | `.github/workflows/codex-cli-update-snapshot.yml`, `.github/workflows/claude-code-update-snapshot.yml` | Reuse in milestone 1. Remove watch responsibility, keep agent-specific execution where it already works. |
| agent-local work-queue guidance | `cli_manifests/*/OPS_PLAYBOOK.md`, `CI_WORKFLOWS_PLAN.md`, `PR_BODY_TEMPLATE.md` | Reuse when present. These remain packet inputs, not new truth surfaces. |
| version truth inside repo | `cli_manifests/<agent>/latest_validated.txt`, `artifacts.lock.json`, snapshots, reports, versions | Reuse directly. This remains the baseline comparator. |
| green gates | `support-matrix --check`, `capability-matrix --check`, `capability-matrix-audit`, `make preflight` | Reuse unchanged. |
| CI workflow contract tests | `crates/xtask/tests/c4_spec_ci_wiring.rs` | Extend instead of adding ad hoc shell validation. |

### Minimum Complete Change

The minimum complete change set is:

1. extend `agent_registry.toml` with watch-enrollment and upstream-source metadata
2. add a repo-owned `xtask` detector that emits a stale-agent queue JSON artifact
3. add a repo-owned `xtask` packet creator that writes `maintenance-request.toml` and the
   generated maintenance packet docs for version-drift work
4. add one shared scheduled GitHub Actions watcher/orchestrator
5. add one generic packet-only PR workflow for future agents without a specialized worker
6. migrate Codex and Claude Code so the shared watcher becomes their only release-watch entrypoint
7. update the contracts and operator guide so the packet format, branch rules, and workflow
   ownership are explicit
8. add fixture-backed tests for registry parsing, queue generation, packet creation, closeout
   compatibility, and CI wiring

Anything smaller leaves the automation surface half-owned.

### Complexity Check

This work touches more than 8 files. That is acceptable because the seam crosses one coherent
control-plane slice:

- registry truth
- xtask command surface
- maintenance request contract
- maintenance packet rendering
- GitHub Actions watcher topology
- worker input normalization
- workflow contract tests

Complexity controls:

- support exactly two upstream source kinds in v1: `github_releases` and `gcs_object_listing`
- do not replace the existing Codex and Claude Code update workers in this milestone
- do not widen the maintenance closeout model beyond the new trigger data it must preserve
- do not add autonomous execution
- do not create a second packet root outside `docs/agents/lifecycle/<agent_id>-maintenance/`
- do not enroll existing packet-only agents until their packet-basis docs are explicit

### Search / Build Decision

- **[Layer 1]** Reuse the existing maintenance lane, request validator, packet renderer, and
  closeout flow.
- **[Layer 1]** Reuse the existing Codex and Claude Code update workers as the initial execution
  targets.
- **[Layer 1]** Reuse `c4_spec_ci_wiring.rs` as the CI workflow contract guardrail.
- **[Layer 1]** Reuse the Google Cloud Storage JSON object-listing API for Claude Code version
  history instead of pretending the `stable` pointer alone can support `latest_stable_minus_one`.
- **[Layer 3]** The missing product is not more workflow YAML. It is repo-owned maintenance
  detection and packet creation driven from one registry contract.

### Distribution Check

This is a repo-internal GitHub Actions plus `xtask` surface. There is no new external package to
publish, but the PR packet itself is a real distribution surface for maintainers and fork
contributors and must be treated as part of the product.

## Locked Decisions

1. Watch enrollment is opt-in per agent through a new `[agents.maintenance.release_watch]` block
   in `crates/xtask/data/agent_registry.toml`. Being onboarded is not the same as being
   watch-enabled.
2. The only target version policy shipped in this milestone is the named enum
   `latest_stable_minus_one`.
3. Pointer-only upstream sources are not supported in this milestone. `latest_stable_minus_one`
   requires upstream history. The initial supported source kinds are:
   - `github_releases`
   - `gcs_object_listing`
4. The shared watcher and the worker dispatches operate against `staging`, because the existing
   snapshot workers already check out `staging` and open PRs back to `staging`.
5. One stale agent equals one branch and one PR. Bundled multi-agent maintenance PRs are
   forbidden.
6. Queue emission is the only place stale detection happens. Workflow YAML and `github-script`
   must not duplicate version-comparison logic.
7. The maintenance packet root stays
   `docs/agents/lifecycle/<agent_id>-maintenance/`, preserving the exact `agent_id` spelling.
   Example: `claude_code` maps to `docs/agents/lifecycle/claude_code-maintenance/`.
8. `prepare-agent-maintenance --write` owns first creation of a maintenance packet root when it
   does not exist. It writes the request and initial packet docs directly. It does not shell out to
   `refresh-agent`.
9. `maintenance-request.toml` is bumped to `artifact_version = "2"` for automated release-watch
   requests. Parser, refresh, and closeout paths are updated in the same change.
10. Automated release-watch requests use a new `trigger_kind = "upstream_release_detected"` and a
    new `[detected_release]` block. This is not folded into `drift_detected`.
11. `opened_from` records the workflow that actually created the request or PR. The watcher origin
    is stored inside `[detected_release]` as `detected_by`.
12. Existing `codex-cli-release-watch.yml` and `claude-code-release-watch.yml` are deleted in this
    milestone. No compatibility shims. The shared watcher is the only release-watch entrypoint.
13. Codex and Claude Code keep specialized execution workers in milestone 1, but those workers
    stop owning schedule and upstream querying.
14. Generic `packet_pr` support lands now but is not enabled for current non-worker agents until
    their packet basis docs are complete enough to produce an exact contributor packet.
15. Automated release-watch requests set `requested_control_plane_actions = ["packet_doc_refresh"]`
    in milestone 1. Publication refresh and runtime-owned work stay in the worker or human lane.

## Target Architecture

### Architecture Overview

```text
crates/xtask/data/agent_registry.toml
  └── agents[*].maintenance.release_watch
        ├── enabled
        ├── version_policy = latest_stable_minus_one
        ├── dispatch_kind = workflow_dispatch | packet_pr
        └── upstream source metadata

maintenance-watch
  ├── read registry
  ├── query upstream release history
  ├── read cli_manifests/<agent>/latest_validated.txt
  ├── compute target_version
  ├── drop clean agents
  └── emit stale-agent queue JSON

agent-maintenance-release-watch.yml
  ├── checkout staging
  ├── run maintenance-watch --emit-json
  └── fan out one queue item at a time
       ├── workflow_dispatch -> existing specialized worker
       └── packet_pr        -> generic packet-only PR opener

prepare-agent-maintenance --write
  ├── write docs/agents/lifecycle/<agent_id>-maintenance/governance/maintenance-request.toml
  ├── write README.md / scope_brief.md / seam_map.md / threading.md / review_surfaces.md / HANDOFF.md
  └── write governance/remediation-log.md

worker PR or packet-only PR
  ├── include the generated maintenance packet
  ├── include exact next commands
  ├── include green gates
  └── target base branch staging
```

### Branch And State Flow

```text
nightly schedule on staging
  -> shared watcher runs on staging
  -> queue entry for stale agent
  -> worker/open-pr workflow dispatched with ref=staging
  -> workflow checks out staging
  -> branch automation/<agent_id>-maintenance-<target_version>
  -> PR base staging
  -> rerun for same agent/version updates same branch + same PR
```

### Registry Contract Additions

Extend `crates/xtask/data/agent_registry.toml` and `docs/specs/agent-registry-contract.md` with
one new nested block under `[agents.maintenance]`:

```toml
[agents.maintenance.release_watch]
enabled = true
version_policy = "latest_stable_minus_one"
dispatch_kind = "workflow_dispatch" # or "packet_pr"
dispatch_workflow = "codex-cli-update-snapshot.yml" # required when dispatch_kind = "workflow_dispatch"

[agents.maintenance.release_watch.upstream]
source_kind = "github_releases" # or "gcs_object_listing"

# github_releases fields
owner = "openai"
repo = "codex"
tag_prefix = "rust-v"

# gcs_object_listing fields
bucket = "claude-code-dist-86c565f3-f756-42ad-8dfa-d59b1c096819"
prefix = "claude-code-releases/"
version_marker = "manifest.json"
```

Validation rules:

- `enabled = true` requires a valid `upstream` block
- `dispatch_workflow` is required for `workflow_dispatch`, forbidden for `packet_pr`
- `version_policy` must be a known enum, not free text
- `source_kind = github_releases` requires `owner`, `repo`, and `tag_prefix`
- `source_kind = gcs_object_listing` requires `bucket`, `prefix`, and `version_marker`
- pointer-only sources are rejected for `latest_stable_minus_one`
- release-watch-enabled agents must still have a valid `manifest_root`
- release-watch-enabled agents in milestone 1 must also have `latest_validated.txt`
- initial rollout sets `enabled = true` for `codex` and `claude_code` only

Concrete upstream behavior:

- `github_releases`: fetch stable releases, filter tags by `tag_prefix`, parse strict semver,
  sort descending, choose latest stable and stable-minus-one
- `gcs_object_listing`: query the GCS JSON API for object names under `prefix`, extract the first
  path component whose object set contains `version_marker`, parse strict semver, deduplicate,
  sort descending, choose latest stable and stable-minus-one

This resolves the main design gap in the draft: Claude Code cannot compute stable-minus-one from
the `stable` pointer alone. It needs release history, and the bucket listing already provides it.

### Maintenance Request Contract Additions

Extend `crates/xtask/src/agent_maintenance/request.rs`,
`docs/specs/cli-agent-onboarding-charter.md`, and the maintenance packet docs so automated
version-drift requests are first-class:

```toml
artifact_version = "2"
agent_id = "codex"
trigger_kind = "upstream_release_detected"
basis_ref = "cli_manifests/codex/latest_validated.txt"
opened_from = ".github/workflows/codex-cli-update-snapshot.yml"
requested_control_plane_actions = ["packet_doc_refresh"]
request_recorded_at = "2026-05-05T15:00:00Z"
request_commit = "abcdef1"

[runtime_followup_required]
required = true
items = [
  "Refresh codex manifest artifacts for the target version",
  "Run the codex validation gates",
  "Review and merge the generated maintenance packet",
]

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
```

Contract rules:

- `artifact_version = "2"` is required for automated release-watch requests
- `trigger_kind = "upstream_release_detected"` is valid only when `[detected_release]` is present
- `basis_ref` remains a repo-relative path
- `opened_from` is the workflow that wrote the request
- `detected_release.detected_by` is the watcher workflow that found the stale version
- `branch_name` is authoritative for PR deduping
- `requested_control_plane_actions` is `["packet_doc_refresh"]` in milestone 1

Closeout implication:

- `close-agent-maintenance` and its types/renderers must preserve the new trigger kind and
  detected-release metadata so the final `HANDOFF.md` remains truthful

### New xtask Surface 1: Stale-Agent Detector

Add:

```text
cargo run -p xtask -- maintenance-watch --check
cargo run -p xtask -- maintenance-watch --emit-json _ci_tmp/maintenance-watch.json
```

Behavior:

- load `agent_registry.toml`
- select `agents[*].maintenance.release_watch.enabled = true`
- read `cli_manifests/<agent>/latest_validated.txt`
- query upstream release history per registry metadata
- compute `latest_stable` and `target_version`
- compare `target_version` against `latest_validated.txt`
- emit a human summary and, when requested, a JSON queue artifact

Queue artifact shape:

```json
{
  "schema_version": 1,
  "generated_at": "2026-05-05T15:00:00Z",
  "stale_agents": [
    {
      "agent_id": "codex",
      "manifest_root": "cli_manifests/codex",
      "current_validated": "0.97.0",
      "latest_stable": "0.99.0",
      "target_version": "0.98.0",
      "version_policy": "latest_stable_minus_one",
      "dispatch_kind": "workflow_dispatch",
      "dispatch_workflow": "codex-cli-update-snapshot.yml",
      "maintenance_root": "docs/agents/lifecycle/codex-maintenance",
      "request_path": "docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml",
      "opened_from": ".github/workflows/codex-cli-update-snapshot.yml",
      "detected_by": ".github/workflows/agent-maintenance-release-watch.yml",
      "branch_name": "automation/codex-maintenance-0.98.0"
    }
  ]
}
```

Hard-fail conditions:

- malformed registry metadata
- malformed `latest_validated.txt`
- malformed upstream response
- unsupported `version_policy`
- unsupported `source_kind`
- `dispatch_kind = workflow_dispatch` with missing workflow id

No-op conditions:

- no enrolled agents
- fewer than two stable versions for a `latest_stable_minus_one` source
- computed candidate equals current validated
- computed candidate is older than current validated

Implementation rule:

- keep this explicit and small, one enum plus two source-specific helpers
- do not build a generic plugin registry for upstream sources in this milestone

### New xtask Surface 2: Maintenance Packet Creator

Add:

```text
cargo run -p xtask -- prepare-agent-maintenance \
  --agent <agent_id> \
  --current-version <version> \
  --latest-stable <version> \
  --target-version <version> \
  --opened-from <workflow-path> \
  --detected-by <workflow-path> \
  --dispatch-kind <workflow_dispatch|packet_pr> \
  [--dispatch-workflow <workflow-id>] \
  --branch-name <branch> \
  --dry-run

cargo run -p xtask -- prepare-agent-maintenance ... --write
```

Behavior:

- validate the agent exists in `agent_registry.toml`
- validate the provided release data against the registry's `release_watch` metadata
- create the maintenance root if it does not already exist
- synthesize `maintenance-request.toml` with `artifact_version = "2"` and
  `trigger_kind = "upstream_release_detected"`
- render `README.md`, `scope_brief.md`, `seam_map.md`, `threading.md`, `review_surfaces.md`,
  `HANDOFF.md`, and `governance/remediation-log.md`
- use the same doc renderer contract as `refresh-agent`, but do the initial write directly

`HANDOFF.md` must include:

- current validated version
- latest stable version
- target version
- why the agent is stale
- exact repo commands to run next
- exact validation gates
- touched surface inventory
- packet basis paths
- closeout command
- explicit "do not edit by hand" notes for generated surfaces

This is the most important product surface in the milestone. The PR packet is the feature.

### Shared Workflow Topology

Add one new scheduled workflow:

`/.github/workflows/agent-maintenance-release-watch.yml`

Responsibilities:

1. check out `staging`
2. run `cargo run -p xtask -- maintenance-watch --emit-json _ci_tmp/maintenance-watch.json`
3. stop early if `stale_agents` is empty
4. fan out over `stale_agents`
5. for `dispatch_kind = workflow_dispatch`, dispatch the named worker workflow with:
   - `agent_id`
   - `current_version`
   - `latest_stable`
   - `target_version`
   - `branch_name`
   - `detected_by`
6. for `dispatch_kind = packet_pr`, invoke one new generic workflow that opens a packet-only PR
7. dispatch all downstream workflows with `ref: staging`

Add one new generic workflow:

`/.github/workflows/agent-maintenance-open-pr.yml`

Responsibilities:

1. check out `staging`
2. run `prepare-agent-maintenance --write`
3. create or update branch `automation/<agent_id>-maintenance-<target_version>`
4. open one PR whose body begins with the generated packet summary
5. never attempt artifact acquisition, snapshot generation, or publication refresh

### Migration of Existing Worker Workflows

Codex and Claude Code keep their specialized execution workflows in milestone 1, but both become
worker-only surfaces:

- `.github/workflows/codex-cli-update-snapshot.yml`
- `.github/workflows/claude-code-update-snapshot.yml`

The old scheduled watcher files are deleted:

- `.github/workflows/codex-cli-release-watch.yml`
- `.github/workflows/claude-code-release-watch.yml`

Required worker changes:

1. accept `agent_id`, `current_version`, `latest_stable`, `target_version`, `branch_name`, and
   `detected_by`
2. normalize all internal use of "version" to `target_version`
3. run `prepare-agent-maintenance --write` before PR creation so the PR always contains the
   maintenance packet
4. prepend or append the generated packet summary to the PR body
5. preserve the existing artifact acquisition, snapshot, union, report, version-metadata,
   validation, and PR-creation behavior that already works
6. target branch `automation/<agent_id>-maintenance-<target_version>` and base `staging`

Agent-specific notes:

- Codex keeps its existing work-queue summary generation and `PR_BODY_TEMPLATE.md`, but the packet
  summary becomes the first part of the PR body
- Claude Code upgrades from a thin handwritten PR body to the same packet-first model

### Maintenance Packet Content Rules

The packet opened by automation is deterministic and minimal:

- machine contract:
  - `docs/agents/lifecycle/<agent_id>-maintenance/governance/maintenance-request.toml`
- generated human entrypoints:
  - `docs/agents/lifecycle/<agent_id>-maintenance/HANDOFF.md`
  - `docs/agents/lifecycle/<agent_id>-maintenance/threading.md`
  - `docs/agents/lifecycle/<agent_id>-maintenance/review_surfaces.md`

The packet must call out:

- exact prompt for the coding agent
- exact repo commands to run
- exact files and directories expected to change
- exact green gates
- exact follow-up closeout command

### File-Level Implementation Map

| Surface | Planned change |
| --- | --- |
| `crates/xtask/data/agent_registry.toml` | add `maintenance.release_watch` metadata; enable `codex` and `claude_code` in milestone 1 |
| `crates/xtask/src/agent_registry.rs` | parse and validate `release_watch` metadata |
| `crates/xtask/src/agent_maintenance/mod.rs` | export new `watch` and `prepare` modules |
| `crates/xtask/src/agent_maintenance/request.rs` | add `artifact_version = "2"`, `upstream_release_detected`, and `detected_release` fields |
| `crates/xtask/src/agent_maintenance/docs.rs` | render packet-first contributor guidance with release metadata |
| `crates/xtask/src/agent_maintenance/watch.rs` | new detector and queue emitter |
| `crates/xtask/src/agent_maintenance/prepare.rs` | new packet creator for release-watch requests |
| `crates/xtask/src/agent_maintenance/closeout/types.rs` | preserve new trigger kind and detected-release metadata through closeout |
| `crates/xtask/src/agent_maintenance/closeout/write.rs` | keep final handoff truthful for automated release-watch runs |
| `crates/xtask/src/main.rs` | expose `maintenance-watch` and `prepare-agent-maintenance` |
| `.github/workflows/agent-maintenance-release-watch.yml` | new shared watcher/orchestrator |
| `.github/workflows/agent-maintenance-open-pr.yml` | new generic packet-only PR opener |
| `.github/workflows/codex-cli-update-snapshot.yml` | worker-only migration plus packet generation |
| `.github/workflows/claude-code-update-snapshot.yml` | worker-only migration plus packet generation |
| `.github/workflows/codex-cli-release-watch.yml` | delete |
| `.github/workflows/claude-code-release-watch.yml` | delete |
| `docs/specs/agent-registry-contract.md` | document `release_watch` enrollment and source metadata |
| `docs/specs/cli-agent-onboarding-charter.md` | document the new request version and maintenance watch ownership |
| `docs/cli-agent-onboarding-factory-operator-guide.md` | document the live shared watcher, packet creator, and branch rules |

## Code Quality Rules For This Slice

1. Registry parsing and version-policy logic live in Rust, not copied into workflow JavaScript.
2. Workflow YAML may dispatch work, but it may not reimplement stale detection.
3. `maintenance-request.toml` remains the only machine-readable packet contract.
4. Existing maintenance packet roots stay authoritative. No `docs/agents/.uaa-temp/...` fork for
   this slice.
5. Specialized worker workflows are allowed only for execution differences. Detection and packet
   creation are shared.
6. Use one enum plus two source-specific parsing helpers. Do not spend an innovation token on a
   provider plugin framework.
7. Keep packet-root naming consistent with exact `agent_id` spelling, even when it means
   underscores in maintenance root names.

## Implementation Slices

### Slice 1. Contract Extensions

Ship the metadata and request-schema extensions first.

Done means:

- `agent_registry.toml` can express watch enrollment and upstream source truth
- `maintenance-request.toml` can represent upstream version drift explicitly
- `artifact_version = "2"` is validated
- contract docs are updated before workflow edits

### Slice 2. Shared Detector And Queue Emitter

Add `maintenance-watch` with fixture-backed tests.

Done means:

- local dry-run summary exists
- JSON queue artifact exists
- GitHub releases and GCS listings both support `latest_stable_minus_one`
- no stale agent opens work when candidate is not strictly newer
- stale-agent detection is no longer hardcoded in workflow JavaScript

### Slice 3. Packet Creator And Packet Rendering

Add `prepare-agent-maintenance --dry-run|--write`.

Done means:

- automation can create a maintenance packet without hand-authoring the request
- missing maintenance roots are created on first write
- `HANDOFF.md` becomes the canonical contributor packet entrypoint
- exact prompt, commands, touched surfaces, and gates are rendered from repo truth
- closeout remains truthful for the new automated trigger

### Slice 4. Shared Workflow Topology

Add the shared watcher and the generic packet-only PR flow.

Done means:

- nightly schedule exists in one workflow
- fanout reads the queue artifact
- packet-only agents have a real PR path even without a specialized worker
- no legacy scheduled watcher workflows remain

### Slice 5. Codex And Claude Code Migration

Migrate the current live automations onto the shared entrypoint.

Done means:

- Codex and Claude Code no longer own their own release-watch schedule
- both workers accept shared payloads
- both workers write the maintenance packet into their PRs
- both workers keep their existing artifact-generation behavior
- both workers use the shared branch naming scheme

### Slice 6. Proof And Documentation Closeout

Prove the architecture with the migrated workers and refreshed docs.

Done means:

- operator guide matches the live workflows
- contracts are truthful
- test suite covers registry, detector, packet creation, closeout compatibility, and CI topology
- milestone 1 rollout is explicitly codex + claude_code only

## Test Review

### Code Path Coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/xtask/src/agent_registry.rs
    ├── parse valid release_watch metadata
    ├── reject missing dispatch_workflow for workflow_dispatch
    ├── reject pointer-only sources for latest_stable_minus_one
    ├── reject malformed github_releases metadata
    ├── reject malformed gcs_object_listing metadata
    └── reject unknown version_policy

[+] crates/xtask/src/agent_maintenance/watch.rs
    ├── no enrolled agents -> empty queue
    ├── github_releases source -> latest_stable and stable_minus_one computed
    ├── gcs_object_listing source -> latest_stable and stable_minus_one computed
    ├── candidate == latest_validated -> no-op
    ├── candidate < latest_validated -> no-op
    ├── stale agent -> queue entry emitted
    └── malformed upstream response -> hard fail

[+] crates/xtask/src/agent_maintenance/prepare.rs / request.rs
    ├── trigger_kind = upstream_release_detected parses
    ├── detected_release block validates
    ├── dry-run previews exact file set
    ├── write creates missing maintenance root
    └── write renders request + packet docs inside docs/agents/lifecycle/<agent_id>-maintenance/

[+] crates/xtask/src/agent_maintenance/closeout/**
    ├── closeout accepts artifact_version = 2 request linkage
    ├── closeout preserves upstream_release_detected in final handoff
    └── final HANDOFF.md stays truthful for automated release-watch runs

[+] workflow migrations
    ├── shared watcher dispatches specialized workers correctly
    ├── generic packet-only workflow opens one PR for one agent
    ├── codex worker accepts shared payload and retains work-queue summary
    └── claude worker accepts shared payload and writes packet-first PR body
```

### User Flow Coverage

```text
USER FLOW COVERAGE
===========================
[+] Shared watcher nightly run
    ├── no stale agents -> exits clean, opens nothing
    ├── one stale codex release -> dispatches codex worker
    ├── one stale claude release -> dispatches claude worker
    ├── packet_pr queue entry -> opens packet-only PR
    └── multiple stale agents -> one branch/PR path each, never bundled

[+] Contributor-ready PR packet
    ├── packet explains why agent is stale
    ├── packet includes exact prompt
    ├── packet includes exact repo commands
    ├── packet lists touched surfaces
    ├── packet includes closeout command + green gates
    └── rerun for same target updates same branch instead of spraying duplicates

[+] Closeout truth
    ├── request version 2 stays readable after work completes
    ├── final HANDOFF.md still names the automated trigger
    └── maintainer can trace the request back to watcher + worker workflow origins
```

### Test Files To Add Or Update

| Surface | Test location | Required assertions |
| --- | --- | --- |
| registry metadata validation | `crates/xtask/tests/agent_registry.rs` | new `release_watch` fields parse; invalid source/dispatch combos fail |
| stale-agent detector | `crates/xtask/tests/agent_maintenance_watch.rs` | queue generation, no-op paths, malformed upstream handling, GitHub and GCS stable-minus-one logic |
| packet creator | `crates/xtask/tests/agent_maintenance_prepare.rs` | dry-run preview, initial root creation, request + packet file set, branch/request path correctness |
| maintenance request schema | `crates/xtask/tests/agent_maintenance_refresh.rs` | `artifact_version = "2"`, `upstream_release_detected`, `detected_release` parsing |
| closeout compatibility | `crates/xtask/tests/agent_maintenance_closeout.rs` | final handoff preserves automated trigger and request linkage |
| workflow contract | `crates/xtask/tests/c4_spec_ci_wiring.rs` | shared watcher exists, legacy watchers deleted, workers accept shared inputs, shared branch/base rules hold |
| end-to-end detector fixtures | `crates/xtask/tests/support/agent_maintenance_harness.rs` or sibling harness | fixture registry + manifest roots + upstream JSON produce the expected queue |

### Commands That Must Pass Before Landing

```sh
cargo test -p xtask --test agent_registry
cargo test -p xtask --test agent_maintenance_drift
cargo test -p xtask --test agent_maintenance_refresh
cargo test -p xtask --test agent_maintenance_prepare
cargo test -p xtask --test agent_maintenance_watch
cargo test -p xtask --test agent_maintenance_closeout
cargo test -p xtask --test c4_spec_ci_wiring
make preflight
```

## Failure Modes Registry

| Codepath | Realistic failure | Test coverage required | Error handling required | User-visible outcome |
| --- | --- | --- | --- | --- |
| `maintenance-watch` GitHub release query | upstream tag format changes | yes | hard fail with agent id + repo + tag prefix | workflow fails closed before dispatch |
| `maintenance-watch` GCS object listing | bucket listing shape changes or version extraction breaks | yes | hard fail with bucket + prefix | workflow fails closed before dispatch |
| queue fanout | stale agent computed twice across reruns | yes | deterministic `branch_name` per agent/version | same PR is updated, not duplicated |
| packet creator | request written outside maintenance root | yes | path-jail validation | creator fails closed |
| request schema evolution | closeout no longer understands request version 2 | yes | versioned request parsing in closeout | maintenance lane blocks before false closeout |
| specialized worker dispatch | workflow id missing or mistyped | yes | registry validation + dispatch step failure | no silent drop of stale agent |
| contributor packet | PR opens without exact commands/prompt | yes | content assertions in packet-renderer tests | packet is blocked as invalid in tests |

Any path that opens a PR without exact commands and green gates is a critical gap and must block
landing.

## Performance Review

This slice is operationally light if the shared watcher stays narrow:

- the watcher does O(enrolled agents) registry reads plus one upstream-history query per agent
- it must not build snapshots or wrapper crates itself
- heavy work remains isolated inside per-agent worker flows
- queue emission is tiny JSON, not a committed artifact
- matrix fanout should start conservative, `max-parallel: 2` is enough

The real performance risk is not CPU. It is PR storm behavior and upstream rate limits.

Controls:

- no-op when the target version is not strictly newer
- deterministic branch names so reruns update the same PR
- shared watcher performs detection only, not artifact work
- initial rollout limits watch-enabled agents to two known workers

## NOT in scope

- autonomous execution that runs Codex or another agent and pushes branch updates automatically
- a generic artifact-acquisition engine for every agent
- enrolling current non-worker agents in release watch before their packet basis docs are complete
- multi-agent upgrade PRs
- release promotion workflow changes
- `goose` lifecycle proving work after maintenance CI lands
- maintenance closeout model changes beyond preserving the new automated-trigger data
- a pointer-only upstream source contract

## Worktree Parallelization Strategy

### Dependency Table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Contract extensions | `crates/xtask/data/`, `crates/xtask/src/agent_registry.rs`, `docs/specs/`, `docs/cli-agent-onboarding-factory-operator-guide.md` | — |
| Detector implementation | `crates/xtask/src/agent_maintenance/`, `crates/xtask/src/main.rs`, `crates/xtask/tests/` | Contract extensions |
| Packet creator implementation | `crates/xtask/src/agent_maintenance/`, `docs/agents/lifecycle/`, `crates/xtask/tests/` | Contract extensions |
| Closeout compatibility updates | `crates/xtask/src/agent_maintenance/closeout/`, `crates/xtask/tests/` | Contract extensions |
| Shared watcher workflow | `.github/workflows/`, `crates/xtask/tests/c4_spec_ci_wiring.rs` | Detector implementation |
| Generic packet-only PR workflow | `.github/workflows/`, packet creator, CI wiring tests | Packet creator implementation |
| Codex worker migration | `.github/workflows/`, `cli_manifests/codex/**`, CI wiring tests | Detector + packet creator |
| Claude worker migration | `.github/workflows/`, `cli_manifests/claude_code/**`, CI wiring tests | Detector + packet creator |
| Docs closeout | `docs/specs/`, `docs/cli-agent-onboarding-factory-operator-guide.md`, `PLAN.md` | Detector + packet creator + workflow topology |

### Parallel Lanes

Lane A: Contract extensions

Lane B: Detector implementation and tests
Sequential inside the lane because it all touches `crates/xtask/src/agent_maintenance/` and the
same queue contract.

Lane C: Packet creator implementation and tests
Sequential inside the lane because it all touches `crates/xtask/src/agent_maintenance/` and
generated packet docs.

Lane D: Closeout compatibility
Sequential inside the lane because it touches the same maintenance request truth and closeout
surfaces.

Lane E: Shared watcher workflow + generic packet-only PR workflow
Sequential, shared `.github/workflows/` and `c4_spec_ci_wiring.rs`.

Lane F: Codex worker migration
Independent from Claude after the packet creator contract is stable.

Lane G: Claude worker migration
Independent from Codex after the packet creator contract is stable.

Lane H: Docs closeout
Waits until contract, detector, packet creator, and workflow topology are stable.

### Execution Order

1. Launch Lane A first.
2. After A lands locally, launch B + C + D in parallel worktrees.
3. After B, C, and D converge, launch E + F + G in parallel worktrees.
4. Merge E first because it owns the shared workflow topology and CI wiring baseline.
5. Merge F and G after rebasing onto E if needed.
6. Run H last, then full verification.

### Conflict Flags

- Lanes B, C, and D all touch `crates/xtask/src/agent_maintenance/`. Split ownership by module:
  - B owns `watch.rs`
  - C owns `prepare.rs` and packet-rendering changes
  - D owns `closeout/**`
- Lanes E, F, and G all touch `.github/workflows/` and `crates/xtask/tests/c4_spec_ci_wiring.rs`.
  Keep one owner for the CI wiring test file or merge sequentially at the end.
- Docs closeout must happen last or it will drift immediately.

## Completion Summary

- Step 0: Scope Challenge, scope accepted as shared watcher + packet creator + worker migration
- Architecture Review: locked on registry-owned watch metadata, history-capable upstream sources,
  request version 2, shared detector, shared packet creator, and one PR per stale agent
- Code Quality Review: DRY boundary is `xtask`, not workflow JavaScript
- Test Review: detector, request contract, packet creation, closeout truth, and CI topology all
  have explicit test homes
- Performance Review: watcher stays light, worker flows stay isolated
- NOT in scope: written
- What already exists: written
- Failure modes: critical gaps identified and blocked
- Parallelization: 9 steps, with B/C/D then E/F/G as the main parallel lanes

That is the whole job. Land the shared watcher and packet creator, migrate Codex and Claude Code
onto it, keep packet-only support ready but disabled for incomplete agents, and leave autonomous
execution for the next milestone.
