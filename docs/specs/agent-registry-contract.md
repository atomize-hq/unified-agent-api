# Agent Registry Contract

Status: Normative  
Scope: schema and ownership rules for `crates/xtask/data/agent_registry.toml`

## Normative language

This document uses RFC 2119 requirement keywords (`MUST`, `MUST NOT`, `SHOULD`).

## Purpose

Define the committed control-plane truth used by `xtask` for onboarded agents, including the
maintenance governance metadata consumed by `check-agent-drift`.

## Registry ownership

`crates/xtask/data/agent_registry.toml` is the canonical committed source of truth for:

- onboarded agent identity and repo paths
- capability declaration shape owned by the control plane
- publication flags and release track enrollment
- onboarding packet ownership
- maintenance governance checks for already-onboarded agents

Generated docs and maintenance packets MAY reference this registry, but they MUST NOT redefine its
schema.

## Entry shape

Each `[[agents]]` entry MUST continue to declare the existing identity, capability, publication,
release, and scaffold fields enforced by `xtask`.

Downstream `xtask` commands that materialize wrapper-crate files MUST use the entry's `crate_path`
directly. They MUST NOT derive a second crate-location contract from `agent_id`.

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
