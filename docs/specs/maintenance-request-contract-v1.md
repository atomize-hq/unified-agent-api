# Maintenance Request Contract v1

Status: Normative  
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
their upstream acquisition pipelines differ, while enforcing one support-aware maintenance success
definition:

1. support-surface audit first
2. bounded non-TUI support uplift second
3. green gates third
4. manual closeout last

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
- support-depth rules, which the [support-depth contract](support-depth-contract.md) owns.
  [Depth-enrolled generations](#depth-enrolled-generations) states what this contract adds for
  them

## Canonical ownership split

The system MUST continue to separate responsibilities this way:

- `crates/xtask/data/agent_registry.toml` owns agent facts.
- `prepare-agent-maintenance` owns packet generation from those facts plus live release inputs.
- `maintenance-request.toml` owns the frozen relay contract for one prepared run.
- `execute-agent-maintenance` owns local validation, write-envelope enforcement, and gate
  execution.
- `close-agent-maintenance` owns explicit post-write closeout only.
- workflow YAML owns transport only: acquire upstream artifacts, invoke `prepare-agent-maintenance`,
  and open or refresh the PR. On the acquisition lane, when no target report exists at open, one
  generation freezes the request twice: once at open, then inside `parity-acquire` with
  `prepare-agent-maintenance --from-request` after the union/report/validate step has produced the
  target artifacts and before the maintenance audit gate. The acquisition commit then carries the
  regenerated request, packet docs, and artifacts together. Only the second freeze is a relay
  contract; the first is a placeholder that no strict loader accepts. The docs-only lane
  (`acquire=false`) has no acquisition and no second freeze. If the target reports already exist on
  base at open, the first freeze is not a placeholder. A declared stand-down skips the second freeze
  (`uaa-0063`). Workflow YAML MUST NOT become a second source of maintenance policy.

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
| `request_recorded_at` | MUST be an RFC 3339 UTC timestamp. A post-acquisition `--from-request` re-freeze MUST preserve it because that re-freeze completes the same generation. |
| `request_commit` | MUST be the commit of the event that opened the generation: the opening workflow's `github.sha`. It is not guaranteed to equal the checked-out base. A post-acquisition `--from-request` re-freeze MUST preserve it, so it identifies neither the base nor the tree from which the completed audit was derived; the acquisition commit records that tree. |
| `[runtime_followup_required]` | MUST remain present, even when `required = false`. |

For release-watch packets in this milestone:

- `requested_control_plane_actions` MUST remain `["packet_doc_refresh"]`
- regeneration or validation of the library-only validated-runtime projection at `crates/agent_api/src/runtime_support_data.rs` MUST stay inside the existing publication and packet machinery; packets MUST NOT add a second control-plane action for runtime-support work
- the packet MUST describe implementation and relay work through `[execution_contract]`, not by
  expanding `requested_control_plane_actions` into a second command queue
- the packet MUST carry `[support_surface_audit]` and MUST use that block, not prompt prose, to
  describe non-TUI support uplift expectations for the run
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

## Universal support-surface-audit shape

Automated upstream-release requests MUST carry one shared `[support_surface_audit]` shape.

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

[[support_surface_audit.unbaselined_gap_surface]]
surface_kind = "flags"
command_path = "codex exec"
surface_id = "--json"
evidence_ref = "cli_manifests/codex/raw_help/..."

[[support_surface_audit.unmatched_debt_surface]]
surface_kind = "flags"
command_path = "codex exec"
surface_id = "--legacy"
debt_ref = "docs/specs/unified-agent-api/non-tui-support-debt.md#codex-exec-legacy-flag"
observation = "not_observed"

[[support_surface_audit.preexisting_unsupported_surface]]
surface_kind = "global_flags"
command_path = "claude_code"
surface_id = "--output-format"
debt_ref = "docs/specs/unified-agent-api/non-tui-support-debt.md#claude-code-output-format"

[[support_surface_audit.eligible_preexisting_surface]]
surface_kind = "global_flags"
command_path = "claude_code"
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
reason = "unbaselined_gap"
required_writes = ["wrapper", "backend", "manifest", "publication"]

[[support_surface_audit.deferred_preexisting_gaps]]
surface_kind = "global_flags"
command_path = "claude_code"
surface_id = "--output-format"
defer_reason = "requires_new_architectural_seam"
blocking_follow_on = "TODOS.md#close-claude-code-install-maintenance-gap"

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
| evidence-backed row | surface row + `evidence_ref` | used for unbaselined gap surface |
| debt-backed row | surface row + `debt_ref` | used for preexisting inventory rows |
| unmatched debt row | surface row + `debt_ref`, `observation` | `observation` only `covered_by_wrapper`, `excluded_by_rules`, or `not_observed` |
| eligible row | surface row + `eligibility_reason` | only `adjacent_surface_changed`, `bounded_write_envelope`, or `no_new_seam_required` |
| uplift row | surface row + `reason`, `required_writes` | shared code writes `reason = "unbaselined_gap"`: a gap surface no debt row covers, whether upstream added it in this release or it was never baselined; `required_writes` values limited to `wrapper`, `backend`, `manifest`, `publication`, `packet_docs` |
| deferred row | surface row + `defer_reason`, `blocking_follow_on` when repo-owned | `blocking_follow_on` omitted only for concrete external blockers |
| publication impact row | surface row + `surface_doc` | ties uplift to published truth |

Surface identity rules. When coverage reports exist for the target version, shared code derives
the gap surfaces and their target masks from the `coverage.<target>.json` exact-target reports,
not from the aggregate `coverage.any.json` row's upstream-availability mask. The aggregate report
remains the evidence reference and unmatched-debt classification source. Gap identities fill
`missing_wrapper_support`, `missing_backend_support`, and `publication_impacts`. A gap with an
inventory identity remains a preexisting and deferred row so debt-baseline continuity is explicit;
if one or more target obligations remain unauthorized, that same aggregate identity also becomes a
discovered row and a required uplift. A debt inventory row that equals no gap surface becomes an
`unmatched_debt_surface` row carrying the debt row's identity and `debt_ref`, with an `observation`
taken from live evidence: `excluded_by_rules` when the report lists the surface under
`deltas.excluded_commands`, `deltas.excluded_flags`, or `deltas.excluded_args`; otherwise
`covered_by_wrapper` when the target version's `snapshots/<version>/union.json` lists it; otherwise
`not_observed`. Classifying unmatched rows requires `snapshots/<version>/union.json` and an
any-target report (`platform_filter.mode = "any"`) whose `inputs.upstream.targets` equal the
union's input targets; otherwise the evidence is invalid. Shared code never derives a removal: help
output cannot prove that upstream removed a surface, because upstream can hide one (see
[Hidden upstream surfaces and wrapper-only rows](#hidden-upstream-surfaces-and-wrapper-only-rows)).
Without per-target reports, every surface row uses the identities written in the debt inventory
rows, no row is unmatched, and no authorization grant applies: the rows remain required work until
target-specific gap evidence exists. Shared code leaves `eligible_preexisting_surface` empty.
`path` is the report row's command path below the agent's own command.

| Report row | `surface_kind` | `command_path` | `surface_id` |
| --- | --- | --- | --- |
| command, empty `path` (the agent's root command) | `commands` | `<agent_id>` | `<agent_id>` |
| command, one path element | `commands` | `<agent_id> <path>` | last path element |
| command, two or more path elements | `subcommands` | `<agent_id> <path...>` | last path element |
| flag (`key`) | `global_flags` when `path` is empty, else `flags` | `<agent_id>` or `<agent_id> <path...>` | `key` |
| positional argument (`name`) | `positional_args` | `<agent_id>` or `<agent_id> <path...>` | `name` |

The three fields together are the name identity; a consumer MUST NOT match surfaces on
`surface_id` alone, because the root command and a command named like the agent share one.
`command_path` is rooted at the registry `agent_id` (for example `claude_code install`), never at
the upstream binary name, and debt inventory rows MUST use the same form so they match
report-derived surfaces. Identity is name-only: it does not compare accepted values, arity, or
output shape, so a flag the wrapper supports for only some values (opencode `run --format` accepts
only `json` through the wrapper) counts as covered (`uaa-0041`).

Authorization is narrower than identity. The normative debt inventory MUST carry the contract
marker `support-debt-authorization-contract-target-version-v1`; a reader MUST reject an inventory
without that exact marker and MUST reject unknown row bullet keys. Every debt row MUST provide a
non-empty comma-separated `scope_target_triples`, exactly one canonical semver in
`authorized_at_version`, and an `authorization_evidence_ref`. Omission, an empty or malformed
value, a target absent from that agent's `union.expected_targets`, or a scope that exceeds the
targets on which the referenced coverage report observed the same surface is invalid evidence.
`evidence_ref` remains historical provenance and MUST NOT be rewritten as authorization evidence.

For a gap identity `K` at version `V`, let `G` be the union of targets whose exact-target report
lists the gap and `A` be the union of scopes from valid rows with identity `K` and
`authorized_at_version = V`. The remaining obligation is `G - A`; the aggregate gap is deferred
in full only when that set is empty. Version matching is exact. Targets are explicit triples:
authorization does not expand platforms, wildcards, ranges, or omitted scope. Rows for one
identity MAY cover
disjoint targets at one version or the same targets at different versions. Two rows that cover the
same `(K, V, target)` are invalid, independent of row order. A valid row at another version is
inapplicable rather than invalid. Audit list rows and debt counts remain aggregated by the
three-field surface identity; when multiple grant rows contribute, the lexicographically smallest
row id supplies the rendered `debt_ref`, deferral reason, and follow-on.

A report's `deltas.missing_commands`, `deltas.missing_flags`, and `deltas.missing_args` MUST be
arrays. `deltas.intentionally_unsupported` MAY be absent, which is how the report writer records an
empty list; when present it MUST be an array. `deltas.excluded_commands`, `deltas.excluded_flags`,
and `deltas.excluded_args` follow the same rule, and shared code reads them only to classify
unmatched debt rows. `deltas.unsupported`, which lists commands whose wrapper coverage level is
`unsupported`, is not a gap list, so such a command never becomes an uplift or matches a debt row
(`uaa-0042`). A report row is invalid evidence when `path` is
missing or is not an array of strings, when `key` or `name` is present but not a string, when it
carries both `key` and `name`, or when its shape does not match the list it appears in (commands
carry neither field, flags carry `key`, arguments carry `name`).

Field invariants:

1. `required` MUST be `true` for every enrolled automated maintenance packet.
2. `required_uplifts_this_run[]` MUST equal:
   - all newly discovered non-TUI gaps with no allowed blocker, plus
   - all eligible preexisting gaps with no allowed blocker.
3. `deferred_preexisting_gaps[]` MAY contain only preexisting gaps, never newly discovered
   surface.
4. Every deferred row MUST use one `allowed_deferrals[]` value.
5. `expected_post_run_debt_count` MUST equal:
   `pre_run_debt_count - closed_gap_count + newly_blocked_external_gap_count`.
   It MUST never exceed `pre_run_debt_count`.
6. `unmatched_debt_surface[]` MUST be empty before a packet closes. Retire a `covered_by_wrapper`
   or `excluded_by_rules` row from the debt inventory, or correct the wrapper coverage or parity
   exclusion that disagrees with it. Retire a `not_observed` row only with evidence that upstream
   removed the surface; if upstream hides it instead, keep it observable with a supplement, which
   today can carry only commands (`uaa-0040`). `close-agent-maintenance` enforces this by deriving
   the audit live rather than reading the frozen block, because a request that records the same
   unmatched rows as the live audit reconciles `exact` and would otherwise close (`uaa-0039`).
7. If this block is absent, malformed, or derived partly from prompt prose instead of shared code,
   the packet is invalid.

Allowed deferral taxonomy:

- `upstream_not_machine_exposed`
- `platform_evidence_missing`
- `requires_new_infra`
- `requires_new_architectural_seam`
- `outside_registry_maintenance_write_envelope`

Invalid deferral reasons:

- `deliberately_unsupported`
- `too_much_work_right_now`
- `not_part_of_v1`

Additional blocker rules:

- `requires_new_infra`, `requires_new_architectural_seam`, and
  `outside_registry_maintenance_write_envelope` are valid only when the packet points to a tracked
  follow-on seam or TODO with an owner and milestone.
- deleting or rewording a support-publication caveat does not satisfy the ratchet. The underlying
  gap MUST either be closed or carried as a concrete blocked inventory row.

### Hidden upstream surfaces and wrapper-only rows

Acquisition discovers upstream surface from each binary's help output, plus any supplement and
feature probe the agent's snapshot command applies. A surface that upstream hides from help is
absent from the union, and that absence is not evidence that upstream removed it.

- A hidden upstream surface is not a support-surface obligation by itself. Shared code never
  derives a gap or an uplift for it.
- It stays an obligation when the wrapper claims it. The coverage report lists every wrapper claim
  that the union does not show under `deltas.wrapper_only_commands`, `deltas.wrapper_only_flags`, or
  `deltas.wrapper_only_args`.
- Before a packet closes, each wrapper-only row MUST be sorted into one category: hidden upstream
  but still supported, supported only on older upstream versions, obsolete, an unsubstantiated
  wrapper claim, or a discovery bug.
- A wrapper-only surface sorted obsolete MUST contract publication truth in the same run, or the
  packet is invalid.
- An unsubstantiated wrapper claim MUST also contract publication truth in the same run. This
  category records that the repository did not establish the claimed upstream surface; it makes no
  assertion that upstream ever supported or later removed it.
- The record is `wrapper_only_dispositions[]` in the closeout artifact, and
  `close-agent-maintenance` enforces it (`uaa-0039`). Each entry carries the surface identity
  (`surface_kind`, `command_path`, `surface_id`), a `category`, an `evidence_ref` that MUST resolve
  to a file in the repository, and a non-empty `note`. `older_upstream_only` MUST also carry
  `last_supported_version`; `discovery_bug` MUST also carry `follow_on`. Neither field is allowed on
  any other category, so a row cannot imply tracking its category never established.

| `category` | Means | Additional evidence |
| --- | --- | --- |
| `hidden_upstream_supported` | Upstream still ships it but hides it from help | — |
| `older_upstream_only` | Upstream shipped it in an earlier version | `last_supported_version` |
| `obsolete` | Upstream removed it and the wrapper claim is withdrawn | the contraction itself |
| `unsubstantiated_wrapper_claim` | The wrapper claim did not establish an upstream surface and is withdrawn | the contraction itself |
| `discovery_bug` | The union should have shown it; acquisition is at fault | `follow_on` |

- The closeout binds the report it adjudicated in `wrapper_only_baseline_ref`, which MUST be the
  report `select_report_path` chooses — `coverage.any.json` whenever it exists. The binding is
  required because a row correctly sorted obsolete or `unsubstantiated_wrapper_claim` is gone from
  the regenerated report by the time closeout runs: judged against the final set alone, its
  disposition would look extraneous. Both fields are omitted when the agent has no wrapper-only row
  and records no disposition, and MUST be omitted when the request declares no detected release,
  because nothing then binds a version.
- Enforcement re-derives from the repository rather than reading the artifact's claims. A
  disposition sorted `obsolete` or `unsubstantiated_wrapper_claim` whose surface is still in the
  live wrapper-only report is rejected: that is what "contract publication truth in the same run"
  means once the claim is withdrawn and the report regenerated. A live wrapper-only row with no
  disposition is rejected, as is a disposition for a surface that is neither live nor one of those
  two withdrawal categories.

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
The relay MUST also validate `[support_surface_audit]` continuity, row-shape validity, allowed
deferrals, and debt-count invariants before write mode.

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
- hard-code or reinterpret the support-surface-audit schema in YAML
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
- `support_surface_audit`

The relay MUST reject packets whose prepared-run metadata does not match the live request packet.
The relay MUST stop before closeout. `close-agent-maintenance` remains the only closeout writer.
`prepare-agent-closeout` generates the closeout artifact that `close-agent-maintenance` records;
see [Closeout](#closeout).

## Depth-enrolled generations

The [support-depth contract](support-depth-contract.md) owns the rules for support depth: depth
enrollment, depth scope tuples, the depth record, the bindings Event, P, O and E, depth admission,
and which changes are acceptance effects and which are reporting effects. This section states only
what this contract owns for them: how a request carries the bindings, what each freeze does, what
relay execution may change and what closeout establishes. Where it applies one of those rules it
does not redefine it, and a term it does not define has the meaning the support-depth contract
gives it.

A **depth-enrolled generation** is a generation whose `agent_id` and
`detected_release.target_version` a depth enrollment selects on the maintenance path. The rules
below are of two kinds:

- Rules about what a request carries, what a freeze writes, what a closeout lists and how a run
  treats its own request apply to depth-enrolled generations. Every other request stays as the
  sections above describe it.
- Rules about what a relay run or a closeout command may change apply to every run and command
  whose writes touch a depth scope tuple, whichever generation it belongs to. The support-depth
  contract determines the tuples a write touches from the values it changes, not from the
  request's agent or version. A later version's run reaches an earlier version's tuples, because
  its `writable_surfaces` cover the same maintenance root and the same aggregate publication.

While the support-depth contract is a Draft this section binds nothing.

Two terms are used below. A **validating loader** is any command that validates the request
against this contract when it loads it, whether or not it tolerates audit drift. The reader behind
`--from-request` is not one. A **re-freeze** is a freeze that records the Event the standing
request already states, whether `--from-request` supplies the recorded values or they are passed
explicitly. A freeze that is not a re-freeze opens a new generation and is that generation's
first freeze.

### Request fields

A depth-enrolled generation's request carries one table beyond the fields above:

```toml
[support_depth]
policy_identity = "<sha256 of P>"
obligations_identity = "<sha256 of O>"
```

| Field | Rule |
| --- | --- |
| `policy_identity` | MUST be the content identity of the P this generation froze at its first freeze. A re-freeze MUST preserve it because a re-freeze completes the same generation. |
| `obligations_identity` | MUST be the content identity of the O this generation last froze. It MUST be absent while no freeze of this generation has frozen O. |

Rules:

1. The table MUST be present in a depth-enrolled generation's request and absent from every other
   request. It is part of the shared shape: its presence follows depth enrollment and never
   `agent_id`. Whether a generation is depth-enrolled is resolved under the support-depth
   contract's depth enrollment rules, and a freeze MUST refuse when that resolution fails. A
   validating loader MUST reject a request that breaks this rule, and MUST NOT read a missing
   table as "not depth-enrolled".
2. Event needs no field of its own. It is the request's `request_commit` and
   `request_recorded_at`, with `trigger_kind` and `opened_from` as its trigger and source.
3. Each identity is a SHA-256 digest written as 64 lowercase hex characters, the form
   `prompt_sha256` uses. What each digest is taken over is defined with the depth record's schema.
   The table carries the two identities and nothing else, and `[support_surface_audit]` is
   unchanged.
4. A validating loader MUST reject the request unless the target version's depth record states
   the request's Event and the same two identities, with O shown as not yet produced while
   `obligations_identity` is absent.
5. `artifact_version` stays `"2"`. Validating loaders reject unknown keys today, so one that
   predates this section rejects the table, and rule 1 rejects a request that lacks it.

The two fields are the only addition to the request. The support-depth contract's minimum
machinery rule requires the failure they prevent to be named. The depth record is otherwise the
only place that states which P and O a generation froze. Execution writes to that record, and
`writable_surfaces` also cover the debt rows that P's baseline is taken from and the target
version's snapshots and reports that O is derived from, so neither identity can be recomputed
from the tree. A record whose P or O has been restated, by a hand edit or by a run that was
interrupted before the relay checked it, then reads as truthful in any single tree, and a single
tree is what repository validation reads. A second statement in the request, the packet artifact
that both the relay and the closeout already bind by content, lets one tree be checked: the
record must state what the request states. Two fields are needed because a generation can freeze
P and never freeze O.

### Freezes

The freeze points are unchanged. For a depth-enrolled generation:

1. The first freeze freezes P. `prepare-agent-maintenance` MUST write `policy_identity` and, in
   the same invocation, MUST write the target version's depth record, or continue it when an
   earlier generation of that version wrote it. This holds on every lane, the docs-only lane
   included.
2. Every freeze at which the target version's target reports exist freezes O, whether it is the
   first freeze or a re-freeze. Such a freeze MUST write `obligations_identity` and, in the same
   invocation, MUST extend the depth record with O.
3. A re-freeze MUST preserve `policy_identity` and MUST derive O under the frozen P. It MUST
   refuse when P has changed since the first freeze, and when it cannot establish that P is
   unchanged. A debt transition that the support-depth contract's debt operations assign to
   execution changes E and is not a change to P, so the comparison is with P's baseline as the
   first freeze froze it, not with the debt rows as the tree now holds them. Where the content of
   that baseline is stated is for the depth record's schema to define. The request's
   `[support_surface_audit]` rows cannot serve, because every re-freeze takes them again from the
   tree.
4. On this path P changes only when a new generation opens, and within a generation O is frozen
   again only by a re-freeze.

### Relay execution

For a depth-enrolled generation:

1. `[support_depth]` is part of the request's authority for the run. While the request carries no
   `obligations_identity` the generation has no frozen depth obligations for a run to work to,
   and the support-depth contract reports its depth-enrolled scope as insufficient depth.
2. A run MUST NOT change the request, whatever `writable_surfaces` lists. The relay MUST treat a
   change to the request as it treats a write to the closeout path.

For every relay run whose changes touch a depth scope tuple, whichever generation the run belongs
to:

3. Relay execution makes reporting effects only. The relay MUST treat the run as failed when its
   changes include an acceptance effect for a depth scope tuple, removal of a stand-down marker
   included, or a depth-gated effect for which the relay cannot establish depth admission.
4. When a run fails under rule 2 or 3, the relay MUST restore every change it attributes to the
   run, so that those paths are as they were when the executor started, before it reports the
   failure. Restoring the depth-gated outputs alone is not enough: the run may also have changed
   the evidence and dependencies those outputs rest on, and a restored result would then claim
   more than the tree supports. The support-depth contract's restoration rules govern the
   restore. The relay MAY keep a copy of the changes it reverts in the run directory.
5. Rules 2 and 3 are evaluated on every write-mode run in which the executor ran, whatever else
   failed.

Rules 2 to 5 are shared policy for every agent. `writable_surfaces` stays the declared list, and
these rules limit what a run may change inside it, as the exclusion of the closeout path already
does. They are not a second envelope derived from `agent_id`.

### Closeout

`prepare-agent-closeout` generates the closeout artifact that `close-agent-maintenance` records,
and both write the closeout path. The Relay boundary's statement that `close-agent-maintenance`
remains the only closeout writer is about the relay, which never writes the closeout. It does not
put `prepare-agent-closeout` outside the rules below.

For each of the two commands, when its writes touch a depth scope tuple, whichever generation it
closes:

1. The command MUST establish depth admission for every such tuple, as the support-depth contract
   requires of every route.
2. Unless depth admission is established and the artifact the command wrote validates, the
   command MUST leave every file as it was before the invocation. An artifact the command wrote
   and then rejected MUST NOT stay at the closeout path, whether or not a closeout existed
   before. The command MAY report it or keep it elsewhere.

For a depth-enrolled generation, writing the closeout artifact is an acceptance effect, and so are
the lifecycle-record changes `close-agent-maintenance` makes with it:

3. Each command MUST ensure, in the same invocation, that the target version's depth record lists
   the acceptance entry for the closeout it writes, as record invariant 4 of the support-depth
   contract requires. It lists the entry only once the artifact has validated, so that a refusal
   never has an entry to take back.

The closeout artifact gains no field. It already binds the request it closes by content
(`request_sha256`), and through the request the two identities. Closeout's other checks are
unchanged.

### Present behavior

None of this section is implemented, and it has no effect until a depth enrollment exists. Today:

- No request carries `[support_depth]`, and `--from-request` reads a fixed list of recorded
  fields and regenerates the rest.
- The workflows that open a packet commit the maintenance root only. A depth record written under
  the manifest root at the first freeze would not be in the opening commit. On the acquisition
  lane the acquisition commit carries `reports/<version>/`; on the docs-only lane nothing does.
- `writable_surfaces` cover the request and the stand-down markers through
  `{maintenance_root}/**`.
- The relay checks the declared list after the executor has written. It takes its baseline at
  the dry run, as digests without content, and it leaves a failed run's changes in place.
- `prepare-agent-closeout` writes the closeout artifact, then validates it through the
  validator's own input path, and leaves a rejected artifact in place when no closeout existed
  before.

The support-depth contract's path enablement requires this enforcement to land, with tests,
before the maintenance path is enabled.

## Transitional compatibility

The current live implementation still contains milestone-1 behavior that is narrower than this
target contract in some places.

During the transition to full v1:

- historical packets and compatibility fixtures MAY still carry `execution_contract.executor = "codex"`
- validators MAY continue to accept that legacy executor value on the read path temporarily
- newly generated automated packets MUST use `execution_contract.executor = "execute-agent-maintenance"`
- new contract work MUST treat agent-specific executor naming as a compatibility artifact, not as
  the desired steady-state schema
- newly generated automated packets for enrolled maintenance MUST use `dispatch_kind = "packet_pr"`
  unless a manual or historical replay lane explicitly proves that a compatibility transport is
  still required

The steady-state v1 contract is one shared relay identity with agent-specific values projected
through the narrow override hooks above.

## Acceptance criteria for v1 adoption

This contract is considered adopted when all of the following are true:

- an automated packet prepared for `workflow_dispatch` and an automated packet prepared for
  `packet_pr` share the same envelope, detected-release, and execution-contract schema
- automated packets share one exact support-surface-audit schema with no agent-local field names or
  hidden prompt-only policy
- `execute-agent-maintenance` can validate and execute either packet without an agent-specific
  executor special case
- registry truth remains the only enrollment and dispatch source of truth
- worker workflows remain transport-only surfaces
- prompt templates and PR summaries describe relay-owned support uplift instead of acting like
  hidden workflow-specific contracts
