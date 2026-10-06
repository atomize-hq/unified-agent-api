# Agent Registry Contract

Status: Normative  
Scope: schema and ownership rules for `crates/xtask/data/agent_registry.toml`

## Normative language

This document uses RFC 2119 requirement keywords (`MUST`, `MUST NOT`, `SHOULD`).

## Purpose

Define the committed control-plane truth used by `xtask` for onboarded agents, including the
maintenance governance metadata consumed by `check-agent-drift` and the transport facts used to
derive one shared automated maintenance packet contract.

## Registry ownership

`crates/xtask/data/agent_registry.toml` is the canonical committed source of truth for:

- onboarded agent identity and repo paths
- capability declaration shape owned by the control plane
- publication flags and release track enrollment
- onboarding packet ownership
- maintenance governance checks for already-onboarded agents
- maintenance release-watch enrollment and upstream-watch metadata
- support-depth path enablement and depth enrollment declarations

Generated docs and maintenance packets MAY reference this registry, but they MUST NOT redefine its
schema.

## Entry shape

Each `[[agents]]` entry MUST continue to declare the existing identity, capability, publication,
release, and scaffold fields enforced by `xtask`.

Downstream `xtask` commands that materialize wrapper-crate files MUST use the entry's `crate_path`
directly. They MUST NOT derive a second crate-location contract from `agent_id`.

When wrapper scaffolding derives the crate-local Rust `[lib].name`, it MUST use the final
`crate_path` path component as the source basename, normalize `-` to `_`, and then require the
normalized result to match ASCII `[A-Za-z0-9_]+`. Hyphenated crate directories are therefore
valid location contracts, but basenames containing other punctuation, whitespace, or non-ASCII
characters are invalid for scaffoldable registry entries.

If capability publication is enabled for an agent, the registry publication block is also the
canonical source of the target-scoped publication contract. In particular:

- `publication.capability_matrix_target` MAY be omitted when the agent does not require a
  target-scoped capability publication declaration.
- `publication.capability_matrix_target` MUST be present when
  `publication.capability_matrix_enabled = true` and publication truth depends on a specific
  declared target.
- when present, `publication.capability_matrix_target` MUST equal one entry from
  `canonical_targets`
- target ordering in `canonical_targets` MUST NOT be treated as an implicit publication-selection
  contract

Registry-controlled publication truth for capability advertising MUST be derived from the shared
projection contract reused by publication generation and maintenance drift/closeout checks; callers
MUST NOT restate config-gated capability semantics independently.

If maintenance governance auditing is configured for an agent, it MUST live under:

```toml
[agents.maintenance]
[[agents.maintenance.governance_checks]]
```

If maintenance release-watch enrollment is configured for an agent, it MUST live under:

```toml
[agents.maintenance.release_watch]
[agents.maintenance.release_watch.upstream]
```

Absence of `maintenance.release_watch` is the only “not enrolled” state. Callers MUST NOT create a
second enrollment inventory outside the registry or represent unenrolled agents with
`enabled = false` placeholders.

Support depth, when it is configured for an agent, lives under:

```toml
[agents.support_depth]
[[agents.support_depth.enrollments]]
```

[Support depth](#support-depth) defines that table and states when it takes effect. In this
document “enrolled” without a qualifier keeps its existing meanings, and depth enrollment is
always written with the qualifier.

Approval artifacts and create-lane closeout consumers MUST preserve exactly two maintenance
approval modes:

- `release_watch_enrolled`
- `explicitly_deferred`

These modes do not create a second enrollment contract:

- `release_watch_enrolled` requires committed registry `maintenance.release_watch` truth for the
  same agent
- `explicitly_deferred` forbids committed registry `maintenance.release_watch` truth for the same
  agent
- callers MUST NOT introduce a third approval-maintenance mode or alternate release-watch
  enrollment storage outside the registry

Depth enrollment adds no approval-maintenance mode; see [Support depth](#support-depth).

## Maintenance release watch

`maintenance.release_watch` declares the machine-owned watch metadata for upstream release
detection. The schema is:

```toml
[agents.maintenance.release_watch]
enabled = true
version_policy = "latest_stable_minus_one" # or "upstream_stable_pointer"
dispatch_kind = "workflow_dispatch" # or "packet_pr"
dispatch_workflow = "example.yml"    # required only for workflow_dispatch

[agents.maintenance.release_watch.upstream]
source_kind = "github_releases"      # or "gcs_object_listing" or "npm_dist_tag"
```

Required top-level fields:

- `enabled`: boolean. When the block is present, it MUST be `true`.
- `version_policy`: one of `latest_stable_minus_one` or `upstream_stable_pointer`
- `dispatch_kind`: one of `workflow_dispatch` or `packet_pr`

Version-policy rules:

- `latest_stable_minus_one`: target the next-to-latest stable upstream release after the upstream
  history is sorted and deduplicated.
- `upstream_stable_pointer`: target the single upstream-resolved stable-channel version directly;
  no minus-one offset is applied.

Dispatch rules:

- The only live scheduled release-detection entrypoint is `.github/workflows/agent-maintenance-release-watch.yml`. Per-agent watcher workflows MUST NOT be treated as active release-watch entrypoints.
- `dispatch_kind = "packet_pr"` is the steady-state enrolled transport for automated maintenance.
- `dispatch_workflow` MUST be present only when `dispatch_kind = "workflow_dispatch"`.
- `dispatch_workflow` MUST be omitted when `dispatch_kind = "packet_pr"`.
- `dispatch_workflow`, when present, MUST be a non-empty workflow filename.
- The registry omission for `packet_pr` is intentional. Packet generation MUST materialize
  `detected_release.dispatch_workflow = "agent-maintenance-open-pr.yml"` in the request packet so
  the frozen packet still carries one fully resolved dispatch contract.
- `workflow_dispatch` is compatibility-only for historical or manual replay lanes. It MUST NOT
  become a second policy store for support-audit rules, writable-surface narrowing, or gate
  semantics.

Upstream rules:

- `source_kind = "github_releases"` requires:
  - `owner`
  - `repo`
  - `tag_prefix`
- `source_kind = "gcs_object_listing"` requires:
  - `bucket`
  - `prefix`
  - `version_marker`
- `source_kind = "npm_dist_tag"` requires:
  - `package`
  - `dist_tag`
- Source-specific fields from the non-selected source kind MUST NOT be present.

Current committed registry truth enables release-watch metadata for `codex`, `claude_code`, and
`opencode`. Enrolled automated maintenance is expected to converge on `dispatch_kind = "packet_pr"`
for each of those agents. Any temporary or historical `workflow_dispatch` entry exists only as a
compatibility state in the committed registry content, not as a permanent schema-level expectation
for future agents.

## Multi-target parity acquisition

Multi-target parity acquisition — downloading an upstream release for every expected target,
snapshotting each on a matching-OS runner, and merging a complete union — is enabled for an agent
when **both** of the following hold:

1. the agent carries `maintenance.release_watch` in this registry, and
2. the agent's `cli_manifests/<agent>/RULES.json` carries an `acquisition` block.

The registry MUST NOT gain a separate acquisition-enablement field. Such a field would be a second
enrollment inventory, which the “Absence of `maintenance.release_watch` is the only ‘not enrolled’
state” rule above already forbids, and it could disagree with the manifest it claims to describe.

The gate is enforced in one place — `xtask manifest-acquisition-plan` — which fails closed with a
distinct error for each reason (`UnknownAgent`, `NotEnrolled`, `NoAcquisitionBlock`).
`.github/workflows/agent-maintenance-open-pr.yml` treats a clean exit from that command as the
gate, so committed truth is the only input. An agent that satisfies neither condition keeps the
docs-only maintenance path with no behavior change.

Watch and acquire are independent. `maintenance.release_watch.upstream.source_kind` governs
release *detection*; `acquisition.source_kind` in the manifest governs where *binaries* come from.
An agent may legitimately watch one source and acquire from another.

## Support depth

The [support-depth contract](support-depth-contract.md) owns the rules for support depth: what a
depth enrollment selects, how one resolves, what the remainder is, what a lifecycle path is and
what enabling one requires. This section states only what the registry owns for them: where a
depth enrollment is declared, where the enablement of a lifecycle path is recorded, what absence
means and who may change either. Where it applies one of those rules it does not redefine it, and
a term it does not define has the meaning the support-depth contract gives it.

While the support-depth contract is a Draft this section binds nothing.

`support_depth` records, for one agent, the lifecycle paths the maintainer has enabled and the
depth enrollments the maintainer has approved. The schema is:

```toml
[agents.support_depth]
enabled_paths = ["maintenance"]

[[agents.support_depth.enrollments]]
lifecycle_path = "maintenance"
version = "1.2.3"
targets = ["linux-x64"]
```

The example shows the shape of the table. Under rule 5 no entry carries it until a later revision
of this contract defines the rest of a declaration.

Two terms are used below. A lifecycle path is **enabled for an agent** when the agent's
`enabled_paths` lists it. The key records the maintainer's authorization, which is item 5 of the
support-depth contract's path enablement. The other items of path enablement are not registry
facts. A **depth enrollment declaration** (below, a declaration) is one entry of `enrollments`.

| Field | Rule |
| --- | --- |
| `enabled_paths` | The lifecycle paths enabled for this agent: `maintenance`, `onboarding` or both, with this agent as the bounded production scope that the support-depth contract's path enablement asks for. It MUST be omitted when no path is enabled, and it MUST NOT repeat a path. |
| `enrollments` | One declaration for each depth enrollment the maintainer has approved for this agent. It MUST be omitted when there is none. |
| `lifecycle_path` | MUST be `maintenance` or `onboarding`. |
| `version` | MUST be one exact semantic version: the string that `detected_release.target_version` and the manifest root's `versions/<version>.json` use for that version. A range, a wildcard, a prefix such as `v` and a moving name such as `latest` are invalid. |
| `targets` | MUST list at least one target, without repeats. Each names a target in `union.expected_targets` of the manifest root's `RULES.json`, or a target that the version's depth record, as the integration branch holds it, already covers. Rule 12 covers any other target. |

The table MUST be omitted when it would hold neither key.

Rules:

1. The registry entry is the only place a depth enrollment is declared, and the only place the
   enablement of a lifecycle path is recorded. A workflow input, a command-line argument, a
   request field and a file under the manifest root MUST NOT declare either. None of them stands
   in for a missing declaration, except the depth record as rule 2 states. They carry what is
   derived from this table, as a maintenance request's own `[support_depth]` table does.
2. Once a generation of a version is committed, the support-depth contract's depth enrollment rule
   7 keeps everything that generation's selection covers depth-enrolled, whatever this table
   declares afterwards. That rule also says when a generation is committed. The depth record states
   what was resolved from this table. It is not a second place to declare a depth enrollment, and
   each later generation still resolves its policy from this table.
3. An entry with no declaration for a version declares no depth enrollment of that version. That
   absence is how the registry states the support-depth contract's remainder. That contract says
   what the remainder is and how it may be represented, and rule 2 covers a version that has a
   committed generation. The registry MUST NOT carry a disabled, empty or placeholder entry for a
   version or an operation that is not depth-enrolled.
4. A declaration states the whole positive selection the support-depth contract requires. Beyond
   the fields above, that is the operations and their promises, their modes and required values
   and, for each capability the declaration claims, the complete set of operations through which
   the agent's adapter honors it. The other policy entries the maintainer approves for the
   agent's depth scope tuples, such as classification entries, mode exclusions and overrides, are
   stated in the same `support_depth` table.
5. This revision does not define the fields that carry what rule 4 lists. They are an executable
   schema revision of this contract, which the support-depth contract requires to be adopted
   before a path is enabled. Until that revision is adopted an entry MUST NOT carry
   `support_depth`.
6. A declaration selects one exact upstream version of one agent on one lifecycle path. An entry
   MUST NOT carry two declarations with the same `version`, whether they name the same lifecycle
   path or different ones.
7. Depth enrollment is independent of release-watch enrollment. It adds no way to enroll an agent
   for release watch and creates no second release-watch inventory. A declaration on the
   maintenance path in an entry without `maintenance.release_watch` opens no generation, because
   the maintenance request contract opens generations for release-watch agents only.
8. The support-depth contract's depth enrollment rule 3 says when a declaration may be added or
   moved to another lifecycle path: only in a commit that leaves that path in `enabled_paths`. It
   also says when restoring a declaration is not an addition. Removing a path from `enabled_paths`
   withdraws the maintainer's authorization and removes no declaration. A declaration whose
   lifecycle path `enabled_paths` does not list, however that came about, is still a declaration
   and its version is depth-enrolled. The support-depth contract's path enablement says what its
   tuples may receive.
9. `support_depth` is maintainer-owned. A path is enabled, and a declaration is added, changed or
   removed, only by a maintainer's change to the registry. No command or workflow, and no
   executor run, writes this table. `onboard-agent` MUST NOT write it into the entry it appends
   for a new agent.
10. An approval artifact that approves depth enrollment for a new agent is not a second depth
    enrollment inventory. It requires the committed table in the same agent's entry, as
    `release_watch_enrolled` requires committed `maintenance.release_watch` truth. What an
    approval states for this table, and how it binds the committed table, is for the onboarding
    charter to define.
11. A change to this table changes no binding that a generation has frozen. The support-depth
    contract's Bindings rules say when a generation freezes P, and its depth admission predicate
    requires a frozen P to be current. Once a generation of a version is committed, removing the
    version's declaration, or changing it so that it covers or claims less than that generation's
    selection does, un-enrolls nothing. The support-depth contract's Annex B states what a
    selection covers and claims, and how a later selection is compared with it. Such a change
    leaves what the selection covers or claims without the policy it needs. Where the same tree
    holds a depth record of the version that covers or claims more than the declaration does, rule
    12 treats that as missing policy. A declaration changed to cover more takes effect for the
    generation that next freezes P.
12. The registry alone decides the field rules above, apart from whether a target is one that
    `union.expected_targets` lists or that the version's depth record, as the integration branch
    holds it, covers, and it alone decides rules 5 and 6. A registry that breaks one of these is
    invalid, as it is for any other schema rule of this contract. Rules 1, 3, 8 and 9 bind who may
    write the table and what a change to it may do, and breaking one makes no registry invalid.
    Anything else that keeps a version's policy from being resolved is missing policy for that
    agent and version, which the support-depth contract treats as an error: a target that is
    neither in `union.expected_targets` nor covered by the version's depth record as the
    integration branch holds it, missing operations, or a depth record in the same tree that covers
    or claims more than the version's declaration does, a removed declaration included. Rule 2
    holds in each of these cases. A reader MUST NOT treat an invalid registry or missing policy as
    "not depth-enrolled".

The support-depth contract's minimum machinery rule requires the failure a new field prevents to be
named. Without `enrollments`, nothing the maintainer owns would state which version, operations and
targets were approved, and depth enrollment would rest on a caller-supplied argument or on what a
run asserts, neither of which the support-depth contract accepts as authority. Without
`enabled_paths`, the only way to withdraw the authorization for a path would be to delete the
declarations made on it. A version that has no committed generation would then read as never
selected, so withdrawing the authorization would send its work back to the rules that predate the
support-depth contract, which that contract's path enablement forbids. Neither failure can occur
before the table is in use. They are what the two keys prevent from then on.

### Present behavior

None of this section is implemented. Today:

- The registry loader rejects unknown fields at the top level, in an agent entry and under
  `maintenance`. A registry that carried `support_depth` would fail to load, and the onboarding,
  maintenance and publication commands all load the registry.
- No entry carries the table, and no command reads a depth enrollment from anywhere.
- `onboard-agent --write` is the only command that writes the registry. It appends the new
  agent's entry, from an approval artifact or from command-line flags that describe the agent,
  and it renders no `maintenance` table into that entry. A maintainer adds release-watch
  enrollment by hand, and `close-proving-run` compares it with the approval.
- `.github/CODEOWNERS` assigns the registry to the maintainer. As read on 2026-10-05, the
  integration branch has no branch protection, so nothing enforces that ownership there.
- The relay's `writable_surfaces` do not include the registry.
- `manifest-version-metadata` and `manifest-retain` take a manifest root and do not read the
  registry. `manifest-validate` loads it only when it runs inside a workspace, to find the
  manifest roots whose support publication it checks, so it too fails on a registry it cannot
  load.

## Maintenance governance checks

Each `governance_checks` entry MUST declare:

- `path`: repo-relative file path to the historical governance surface
- `required`: boolean
- `comparison_kind`: one of:
  - `approved_agent_descriptor`
  - `markdown_capability_claim`
  - `markdown_support_claim`

Additional rules:

- `path` MUST be normalized and repo-relative.
- `path` values MUST be unique within one agent entry.
- `manual_reopen` MUST NOT be modeled in registry metadata; it remains a maintainer-authored
  maintenance-request trigger only.

### `approved_agent_descriptor`

Use this comparison kind only for approval artifacts under:

`docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml`

Rules:

- `path` MUST match the agent entry’s `scaffold.onboarding_pack_prefix`
- no markdown parser config may be present
- approval and governance comparison MAY omit `capability_matrix_target`; when absent, governance
  comparison MUST treat the field as not asserted rather than as a mismatch
- onboarding approval-mode MUST remain backward-compatible with legacy single-target descriptors,
  while newly generated descriptors MUST include `capability_matrix_target` whenever the registry
  contract requires it

### `markdown_capability_claim`

Use this comparison kind for historical Markdown surfaces that declare capability ids.

Required parser config:

- `start_marker`
- `end_marker`
- `extraction_mode = "inline_code_ids"`

The marked block MUST be the sole machine-audited capability claim for that check.

### `markdown_support_claim`

Use this comparison kind for historical Markdown surfaces that declare support posture.

Required parser config:

- `start_marker`
- `end_marker`
- `extraction_mode = "support_state_lines"`

The marked block MUST contain only structured `key = value` lines. M4 recognizes these keys:

- `backend_support`
- `uaa_support`

## Drift comparison boundary

Maintenance governance checks MUST compare historical governance claims against modeled current
truth, not against generated publication files as their primary source.

Specifically:

- `approved_agent_descriptor` compares against current registry-controlled truth
- `markdown_capability_claim` compares against current capability truth
- `markdown_support_claim` compares against current derived support rows

Prior maintenance closeouts MUST NOT permanently suppress a governance surface. If the same
historical surface drifts again later, `check-agent-drift` MUST report it again.
