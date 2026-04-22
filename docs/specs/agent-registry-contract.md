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

Use this comparison kind only for historical approval artifacts under:

`docs/project_management/next/<onboarding_pack_prefix>/governance/approved-agent.toml`

Rules:

- `path` MUST match the agent entry’s `scaffold.onboarding_pack_prefix`
- no markdown parser config may be present

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
