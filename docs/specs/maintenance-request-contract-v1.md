# Maintenance Request Contract v1

Status: Draft  
Scope: automated upstream-release maintenance requests under `docs/agents/lifecycle/*-maintenance/governance/maintenance-request.toml`

## Normative language

This document uses RFC 2119 requirement keywords (`MUST`, `MUST NOT`, `SHOULD`).

## Purpose

Define the canonical packet contract for automated upstream-release maintenance and the ownership
boundary between:

- registry truth in `crates/xtask/data/agent_registry.toml`
- packet materialization in `prepare-agent-maintenance`
- local relay execution in `execute-agent-maintenance`
- closeout recording in `close-agent-maintenance`
- transport workflows under `.github/workflows/`

This contract exists so newly enrolled agents can share one maintenance packet shape even when
their upstream acquisition pipelines differ.

## Current scope

This document covers only automated upstream-release maintenance requests:

- `artifact_version = "2"`
- `trigger_kind = "upstream_release_detected"`
- a required `[detected_release]` table
- a required `[execution_contract]` table for relay execution
- either enrolled `dispatch_kind`, `workflow_dispatch` or `packet_pr`, when packet generation
  produces the shared prepared-run shape above

This document does not redefine:

- maintainer-authored legacy maintenance requests for non-release-watch flows
- workflow-specific binary acquisition steps
- packet-only maintainer handoff flows that omit the relay `[execution_contract]`, regardless of
  how the PR was opened
- onboarding or publication contracts outside the maintenance packet

## Canonical ownership split

The system MUST continue to separate responsibilities this way:

- `crates/xtask/data/agent_registry.toml` owns agent facts.
- `prepare-agent-maintenance` owns packet generation from those facts plus live release inputs.
- `maintenance-request.toml` owns the frozen relay contract for one prepared run.
- `execute-agent-maintenance` owns local validation, write-envelope enforcement, and gate
  execution.
- `close-agent-maintenance` owns explicit post-write closeout only.
- workflow YAML owns transport only: acquire upstream artifacts, invoke `prepare-agent-maintenance`,
  and open or refresh the PR. Workflow YAML MUST NOT become a second source of maintenance policy.

## Universal packet fields

Every automated upstream-release request MUST keep one shared top-level envelope shape.

| Field | Rule |
| --- | --- |
| `artifact_version` | MUST be `"2"` for this contract. |
| `agent_id` | MUST name one enrolled agent registry entry. |
| `trigger_kind` | MUST be `"upstream_release_detected"` for this contract. |
| `basis_ref` | MUST be a repo-relative baseline pointer owned by the agent manifest root. |
| `opened_from` | MUST be a repo-relative reference to the workflow or source that opened the packet. |
| `requested_control_plane_actions` | MUST remain a control-plane action list, not a runtime implementation plan. |
| `request_recorded_at` | MUST be an RFC 3339 UTC timestamp. |
| `request_commit` | MUST be the repo commit used when the packet was generated. |
| `[runtime_followup_required]` | MUST remain present, even when `required = false`. |

For release-watch packets in this milestone:

- `requested_control_plane_actions` MUST remain `["packet_doc_refresh"]`
- the packet MUST describe implementation and relay work through `[execution_contract]`, not by
  expanding `requested_control_plane_actions` into a second command queue
- `HANDOFF.md` under the maintenance root MUST remain the canonical contributor execution
  contract, while `governance/pr-summary.md` MUST remain derivative from the same packet context

## Universal detected-release fields

Automated upstream-release requests MUST carry one shared `[detected_release]` shape.

| Field | Rule |
| --- | --- |
| `detected_by` | MUST identify the repo-relative watcher surface that detected the release. |
| `current_validated` | MUST record the currently validated upstream version before this run. |
| `target_version` | MUST record the candidate version this packet targets. |
| `latest_stable` | MUST record the freshest stable upstream version observed by the watcher. |
| `version_policy` | MUST record the policy used to choose `target_version`. |
| `source_kind` | MUST describe the upstream discovery mechanism. |
| `source_ref` | MUST contain the normalized source identity for the chosen `source_kind`. |
| `dispatch_kind` | MUST match the registry-owned release-watch dispatch contract. |
| `dispatch_workflow` | MUST be materialized in the packet for both dispatch kinds. `workflow_dispatch` uses the registry-owned worker workflow filename; `packet_pr` uses the shared workflow `agent-maintenance-open-pr.yml`. |
| `branch_name` | MUST be the PR branch reserved for this maintenance run. |

The detected-release table is universal in structure. Worker-specific transport differences MUST be
expressed through values, not through a second per-agent schema.
`dispatch_kind` selects PR-opening transport only; it MUST NOT imply a narrower packet schema or a
missing relay execution contract.

## Universal execution-contract shape

Automated upstream-release requests that are intended for relay execution MUST carry one shared
`[execution_contract]` shape.

| Field | Rule |
| --- | --- |
| `executor` | MUST identify the local relay contract, not the wrapper crate being maintained. Steady-state packets MUST use `execute-agent-maintenance`. |
| `prompt_template_path` | MUST be a repo-relative prompt template path. |
| `prompt_sha256` | MUST match the rendered prompt template digest for `target_version`. |
| `pr_summary_path` | MUST be the repo-relative PR summary artifact for this maintenance root. |
| `closeout_path` | MUST be the repo-relative closeout artifact for this maintenance root. |
| `requires_manual_closeout` | MUST remain `true` for relay-executed upstream-release requests. |
| `writable_surfaces` | MUST enumerate the complete allowed write envelope for relay execution. |
| `read_only_inputs` | MUST enumerate the frozen read set the relay can rely on. |
| `ordered_commands` | MUST enumerate the command sequence expected during implementation. |
| `green_gates` | MUST enumerate the required gates that must pass before closeout. |
| `[execution_contract.recovery]` | MUST remain present and self-sufficient for packet regeneration and PR recovery. |

The relay MUST validate packet contents against this contract and MUST NOT derive a second hidden
write envelope or gate set from `agent_id`.

Recovery notes rendered into the packet SHOULD describe execution-host repair in terms of the
local execution host, not the maintained agent being updated.

## Registry-derived fields

The following request fields are machine-derived from registry truth or from deterministic paths
built from registry truth. Callers MUST NOT maintain parallel copies of these facts elsewhere.

| Packet field or surface | Registry source | Rule |
| --- | --- | --- |
| `agent_id` | `[[agents]].agent_id` | MUST match one registry entry exactly. |
| `basis_ref` | `[[agents]].manifest_root` | MUST derive from the agent manifest root, typically `latest_validated.txt`. |
| `detected_release.version_policy` | `maintenance.release_watch.version_policy` | MUST match the enrolled release-watch policy. |
| `detected_release.source_kind` | `maintenance.release_watch.upstream.source_kind` | MUST match registry truth. |
| `detected_release.source_ref` | `maintenance.release_watch.upstream.*` | MUST normalize the chosen upstream source into one comparable value. |
| `detected_release.dispatch_kind` | `maintenance.release_watch.dispatch_kind` | MUST match registry truth. |
| `detected_release.dispatch_workflow` | `maintenance.release_watch.dispatch_workflow` plus shared packet resolver | MUST match registry truth when dispatch uses `workflow_dispatch`, and MUST resolve to `agent-maintenance-open-pr.yml` when dispatch uses `packet_pr`. |
| `execution_contract.prompt_template_path` | `[[agents]].manifest_root` plus shared packet conventions | MUST derive from the maintenance packet root as the packet-owned prompt template path. |
| `execution_contract.read_only_inputs` | `[[agents]].manifest_root` plus `opened_from` | MUST include the packet-owned playbook, workflow plan, prompt template, and opening workflow path under the maintenance packet root. |
| `execution_contract.writable_surfaces` | `[[agents]].crate_path`, `[[agents]].manifest_root`, publication flags | MUST be derived from registry-owned write surfaces plus shared maintenance policy. |
| `execution_contract.green_gates` | publication flags and shared policy | MUST be generated from shared rules, not handwritten per workflow. |

If future agents require additional derived fields, the registry schema MUST own the source facts
first. The request packet MAY project them, but it MUST NOT invent a second control-plane store.

## Agent override hooks

The request contract is shared, but some values are intentionally agent-specific. These are the
allowed override hooks for v1.

| Hook | Why it may differ by agent |
| --- | --- |
| `basis_ref` path | Different agents have different manifest roots and validated-version pointers. |
| `detected_release.source_ref` | Upstreams differ, for example GitHub releases versus GCS object listings. |
| `detected_release.dispatch_workflow` | Different enrolled agents may still use different worker transport files for `workflow_dispatch`, while `packet_pr` materializes the shared `agent-maintenance-open-pr.yml` workflow. |
| `execution_contract.prompt_template_path` | Each agent may keep its own packet-owned prompt template under its maintenance root. |
| `execution_contract.writable_surfaces` | Wrapper crate paths, manifest artifacts, and approved spec writes differ by agent. |
| `execution_contract.read_only_inputs` | Agent-specific playbooks and workflow plans differ by maintenance root. |
| `execution_contract.ordered_commands` | Acquisition consequences and validation commands may differ by agent, but the shape remains shared. |
| `execution_contract.green_gates` | Publication and validation obligations may differ only where registry-owned flags justify the difference. |
| `execution_contract.recovery.notes` | Recovery guidance may mention agent-specific binary or auth repair steps. |

These hooks MUST stay narrow. Agent-specific values MUST NOT justify agent-specific packet schemas.

## Transport boundary

Worker workflows MAY differ in how they acquire and refresh upstream artifacts before packet
generation.

Worker workflows MUST NOT:

- hard-code a second execution-contract schema in YAML
- redefine writable surfaces in YAML
- redefine green gates in YAML
- encode prompt semantics inline instead of using the packet-owned template and summary artifacts

The worker's job is to produce artifacts, call `prepare-agent-maintenance`, and open the PR. That
is it.

## Relay boundary

`execute-agent-maintenance` MUST treat the prepared request packet as the authority for:

- `target_version`
- `branch_name`
- `prompt_sha256`
- `writable_surfaces`
- `read_only_inputs`
- `ordered_commands`
- `green_gates`
- `closeout_path`
- recovery guidance

The relay MUST reject packets whose prepared-run metadata does not match the live request packet.
The relay MUST stop before closeout. `close-agent-maintenance` remains the only closeout writer.

## Transitional compatibility

The current live implementation still contains milestone-1 behavior that is narrower than this
target contract in some places.

During the transition to full v1:

- historical packets and compatibility fixtures MAY still carry `execution_contract.executor = "codex"`
- validators MAY continue to accept that legacy executor value on the read path temporarily
- newly generated automated packets MUST use `execution_contract.executor = "execute-agent-maintenance"`
- new contract work MUST treat agent-specific executor naming as a compatibility artifact, not as
  the desired steady-state schema

The steady-state v1 contract is one shared relay identity with agent-specific values projected
through the narrow override hooks above.

## Acceptance criteria for v1 adoption

This contract is considered adopted when all of the following are true:

- an automated packet prepared for `workflow_dispatch` and an automated packet prepared for
  `packet_pr` share the same envelope, detected-release, and execution-contract schema
- `execute-agent-maintenance` can validate and execute either packet without an agent-specific
  executor special case
- registry truth remains the only enrollment and dispatch source of truth
- worker workflows remain transport-only surfaces
- prompt templates and PR summaries describe relay-owned work instead of acting like hidden
  workflow-specific contracts
