# CLI Agent Recommendation Dossier Contract

Status: Draft  
Date (UTC): 2026-04-28  
Canonical location: `docs/specs/cli-agent-recommendation-dossier-contract.md`

Normative language: this contract uses RFC 2119 requirement keywords (`MUST`, `MUST NOT`, `SHOULD`).

## Purpose

This contract freezes the discovery-enabled v2 recommendation lane for selecting the next CLI agent.

It defines:

- the discovery, freeze, research, generate, and promote boundaries
- the fixed discovery query families and nomination rules
- the reviewed-seed authority
- the exact scratch and committed artifact roots
- the insufficiency and widening semantics
- the packet constraints and promote-time Model B rules

If skill text, operator procedure, or planning prose diverge from this document, this contract wins.

## Normative References

- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/templates/agent-selection/cli-agent-selection-packet-template.md`

## Workflow Boundary

The recommendation lane is a frozen five-stage workflow:

1. Discovery may search public sources and nominate candidates.
2. `freeze-discovery` validates discovery artifacts and freezes the reviewed seed.
3. Research writes dossiers only against the frozen reviewed seed.
4. `generate` consumes only frozen research artifacts and produces the evaluation run.
5. `promote` commits review evidence, updates the canonical packet, and renders the final approval artifact.

The deterministic boundary is unchanged:

- discovery MAY search
- freeze MUST validate and snapshot
- research MUST work from the frozen snapshot
- `generate` MUST be post-research only
- `promote` MUST stay approval-artifact preserving

The runner is post-research only:

- it MUST NOT fetch open-ended web, docs, package-registry, or GitHub evidence
- it MUST NOT mutate the research artifacts
- it MUST NOT reread `docs/agents/selection/candidate-seed.toml` as reviewed input

## Artifact Roots

The repo-local scratch root for this lane is:

`docs/agents/.uaa-temp/recommend-next-agent/`

Owned subroots are:

- discovery: `docs/agents/.uaa-temp/recommend-next-agent/discovery/<run_id>/`
- research: `docs/agents/.uaa-temp/recommend-next-agent/research/<run_id>/`
- evaluation runs: `docs/agents/.uaa-temp/recommend-next-agent/runs/<run_id>/`

Promoted review evidence lives under:

`docs/agents/selection/runs/<run_id>/`

The canonical comparison packet is:

`docs/agents/selection/cli-agent-selection-packet.md`

The create-lane approval artifact is:

`docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml`

`.staging/` directories remain reserved for internal promote-time staging and MUST NOT be used as operator scratch space.

## Discovery Inputs

### Optional Hints

Optional control-plane input:

`docs/agents/selection/discovery-hints.json`

Exact v2 shape:

```json
{
  "preferred_licenses": ["oss", "commercial_ok"],
  "avoid_account_gated": false,
  "prefer_observable_cli": true,
  "include_candidates": ["aider"],
  "exclude_candidates": ["opencode"],
  "notes": "short optional maintainer guidance"
}
```

Rules:

- every field is optional
- unknown top-level keys are a validation error
- `preferred_licenses` may contain only `oss` and `commercial_ok`
- `include_candidates` and `exclude_candidates` MUST contain unique non-empty strings
- `avoid_account_gated` and `prefer_observable_cli` MUST be booleans
- `notes` MUST be a non-empty string when present

Discovery-hint precedence is frozen to:

1. hard discovery rejections win over everything
2. `exclude_candidates` wins over `include_candidates`
3. valid `include_candidates` MAY bypass soft discovery preferences only
4. soft preferences from `preferred_licenses`, `avoid_account_gated`, and `prefer_observable_cli` influence nomination ordering and survivor choice only
5. `notes` is advisory only and MUST NOT be parsed into scoring, ranking, or hidden inclusion rules

Hints influence discovery inclusion only. They MUST NOT affect evaluation scoring.

### Allowed Source Classes

Allowed discovery source classes in v2 are:

- `web_search_result`
- `official_doc`
- `github`
- `package_registry`

Explicitly disallowed in v2:

- Reddit
- Discord
- private/internal sources

### Hard Discovery Rejections

Discovery MUST reject a candidate before research when any of these are true:

- the candidate already exists in `crates/xtask/data/agent_registry.toml`
- no public CLI install path can be found
- no public CLI docs or other public CLI evidence surface exists
- the candidate is primarily an SDK/library with no standalone CLI workflow
- the candidate requires closed/private distribution with no public install or docs evidence

`generate` MUST still enforce already-onboarded rejection again as defense in depth.

## Discovery Scratch Contract

The discovery directory is:

`docs/agents/.uaa-temp/recommend-next-agent/discovery/<run_id>/`

It MUST contain exactly:

- `candidate-seed.generated.toml`
- `discovery-summary.md`
- `sources.lock.json`

No screenshots, HTML caches, or additional artifact types are part of v2.

### `candidate-seed.generated.toml`

The generated seed MUST keep the existing seed shape:

- `[defaults.descriptor]`
- `[candidate.<agent_id>]`

Required per-candidate fields are unchanged:

- `display_name`
- `research_urls`
- `install_channels`
- `auth_notes`

Discovery rationale belongs in `discovery-summary.md`, not in the generated seed.

### `discovery-summary.md`

The summary MUST contain:

- discovery run id
- discovery pass number
- exact query strings used
- source classes consulted
- hints file used or `none`
- candidate ids nominated by web-search frontier signals
- candidate ids nominated by direct official-source discovery

Each candidate section MUST contain:

- candidate id and display name
- why it entered the pool
- which source first introduced it
- which hint or default policy affected inclusion
- one obvious caveat known before research freeze

### Discovery `sources.lock.json`

Purpose: provenance lock, not internet replay engine.

Each entry MUST contain:

- `candidate_id`
- `source_kind`
- `url`
- `title`
- `captured_at`
- `sha256`
- `role`, one of `frontier_signal`, `discovery_seed`, `install_surface`, `docs_surface`

Entries with `source_kind = web_search_result` MUST also contain:

- `query`
- `rank`

`sha256` is frozen to this rule:

- compute the hash from a canonical UTF-8 serialization of exactly the per-entry object
- sort keys lexicographically
- emit no extra whitespace
- include only:
  - `candidate_id`
  - `source_kind`
  - `url`
  - `title`
  - `captured_at`
  - `role`
  - `query` and `rank` when `source_kind = web_search_result`
- DO NOT hash live page bodies, screenshots, or fetched HTML

Two logically identical entries MUST therefore produce the same `sha256` across reruns.

## Discovery Query Families And Nomination Rules

### Pass 1 Fixed Query Family

Pass 1 MUST use exactly these queries:

- `best AI coding CLI`
- `AI agent CLI tools`
- `developer agent command line`

### Pass 1 Nomination Algorithm

Pass 1 nomination is frozen to:

1. collect candidates from first-page results for the three fixed pass 1 queries
2. normalize each candidate to the upstream project / CLI identity used in this repo
3. deduplicate by normalized `candidate_id`
4. drop candidates rejected by any hard discovery rejection rule
5. apply `exclude_candidates`
6. force-add valid `include_candidates` not already present
7. sort survivors by:
   - highest count of distinct source entries
   - then presence of both docs and install surfaces
   - then alphabetical `candidate_id`
8. emit exactly 5 candidates unless fewer than 5 survive hard rejection

The pass 1 emission cap of 5 is a hard cap.

### Pass 2 Fixed Widening Query Family

Pass 2 widening is frozen to this family:

- candidate-relative query: `alternatives to <top surviving candidate>`
- generic query: `top coding agent CLI open source`
- generic query: `CLI coding assistant blog`

Zero-survivor fallback is part of the contract:

- if pass 1 has zero surviving candidates after hard rejection, pass 2 MUST omit the candidate-relative query
- in that zero-survivor case, pass 2 MUST use only the two generic widening queries

### Pass 2 Widening Nomination Algorithm

Pass 2 nomination is frozen to:

1. start from the pass 1 rejection summary
2. run only the fixed pass 2 widening query family
3. exclude every candidate already seen in pass 1, whether accepted or rejected
4. apply the same hard discovery rejection rules
5. apply the same hint-precedence rules
6. allow soft-preference relaxation only for pass 2 survivor selection
7. emit at most 3 new candidates

The pass 2 add cap of 3 is a hard cap.

## Distinct Pass Ownership And Freeze Semantics

The widening loop is frozen to one retry:

- maximum 2 discovery passes per skill invocation
- pass 1 is the default entry point
- pass 2 is the only allowed widening pass
- if fewer than 3 eligible candidates survive after pass 2, stop with explicit insufficiency

Pass ownership rules are:

- each pass MUST write its own discovery directory under its own `run_id`
- a later pass MUST NOT mutate, delete, or overwrite an earlier pass directory
- the reviewed seed for a pass is authoritative only for the research and evaluation run derived from that same pass
- only the final promoted evaluation run becomes committed review evidence

`freeze-discovery` is the only operation that may create the reviewed seed authority.

## Runner CLI Contract

`freeze-discovery` is frozen to:

```sh
python3 scripts/recommend_next_agent.py freeze-discovery \
  --discovery-dir docs/agents/.uaa-temp/recommend-next-agent/discovery/<run_id> \
  --research-dir docs/agents/.uaa-temp/recommend-next-agent/research/<run_id>
```

Responsibilities:

- validate `candidate-seed.generated.toml`
- validate `discovery-summary.md`
- validate discovery `sources.lock.json`
- reject duplicate candidate ids
- reject candidates already onboarded in the registry
- copy the reviewed generated seed to `research/<run_id>/seed.snapshot.toml`
- copy all three discovery artifacts to `research/<run_id>/discovery-input/`

`generate` is frozen to:

```sh
python3 scripts/recommend_next_agent.py generate \
  --research-dir docs/agents/.uaa-temp/recommend-next-agent/research/<run_id> \
  --run-id <run_id> \
  --scratch-root docs/agents/.uaa-temp/recommend-next-agent/runs
```

`generate` no longer accepts `--seed-file`.

`promote` keeps the existing public shape:

```sh
python3 scripts/recommend_next_agent.py promote \
  --run-dir docs/agents/.uaa-temp/recommend-next-agent/runs/<run_id> \
  --repo-run-root docs/agents/selection/runs \
  --approved-agent-id <agent_id> \
  --onboarding-pack-prefix <kebab-case-pack-prefix> \
  [--override-reason "<required when approved agent differs from recommended>"]
```

## Research Directory Contract

The research directory is:

`docs/agents/.uaa-temp/recommend-next-agent/research/<run_id>/`

It MUST contain:

- `seed.snapshot.toml`
- `discovery-input/candidate-seed.generated.toml`
- `discovery-input/discovery-summary.md`
- `discovery-input/sources.lock.json`
- `research-summary.md`
- `research-metadata.json`
- `dossiers/<agent_id>.json`

`seed.snapshot.toml` is the only reviewed seed authority used by `generate`.

`docs/agents/selection/candidate-seed.toml` remains a fallback curated pool and example. It is not the v2 reviewed runtime input.

## `research-metadata.json` Envelope

`research-metadata.json` MUST be valid JSON and MUST contain exactly these top-level fields:

- `run_id`
- `evidence_collection_time_seconds`
- `fetched_source_count`

No additional top-level fields are part of the v2 contract.

## Research Input Identity Rules

These values MUST agree exactly:

- `research-metadata.json.run_id`
- CLI `--run-id`
- research directory basename
- run directory basename

Each dossier filename `dossiers/<agent_id>.json` MUST match the dossier’s top-level `agent_id`.

Each dossier `agent_id` MUST correspond to a candidate present in `seed.snapshot.toml`.

Each dossier `seed_snapshot_sha256` MUST equal the SHA-256 of the actual `seed.snapshot.toml` used for the run.

Any mismatch above is an input/schema failure, not a soft warning.

## Dossier Schema

Each dossier MUST be one JSON object with exactly these top-level fields:

- `schema_version`
- `agent_id`
- `display_name`
- `generated_at`
- `seed_snapshot_sha256`
- `official_links`
- `install_channels`
- `auth_prerequisites`
- `claims`
- `probe_requests`
- `blocked_steps`
- `normalized_caveats`
- `evidence`

`claims` MUST contain exactly:

- `non_interactive_execution`
- `offline_strategy`
- `observable_cli_surface`
- `redaction_fit`
- `crate_first_fit`
- `reproducibility`
- `future_leverage`

Each claim MUST contain:

- `state`, one of `verified`, `blocked`, `inferred`, `unknown`
- `summary`
- `evidence_ids`
- optional `blocked_by`
- optional `notes`

Each evidence object MUST contain:

- `evidence_id`
- `kind`, one of `official_doc`, `github`, `package_registry`, `ancillary`, `probe_output`
- optional `url`
- `title`
- `captured_at`
- `sha256`
- `excerpt`

`probe_requests` MUST remain an array in the dossier schema and MAY be empty.

Each probe request MUST contain:

- `probe_kind`, one of `help`, `version`
- `binary`
- `required_for_gate`, boolean

The contract MUST NOT add a single-required-probe rule, an exactly-one-probe rule, or any other minimum-cardinality requirement for `probe_requests`.

## Hard Gate Sufficiency Rules

Hard-gate pass/fail is driven by claim state, evidence-kind coverage, and any required probe results. Generic prose in `summary` or `notes` is never sufficient on its own.

| Claim key | Allowed pass states | Required evidence kinds | Required probe rule | Reject when |
| --- | --- | --- | --- | --- |
| `non_interactive_execution` | `verified` only | at least one `official_doc` and one of `package_registry` or `probe_output` | if any `required_for_gate` probe exists under the existing schema, it MUST pass | state is `inferred`, `unknown`, or `blocked`; required evidence kinds missing |
| `observable_cli_surface` | `verified` only | at least one of `official_doc`, `github`, or `probe_output` | if any `required_for_gate` probe exists under the existing schema, it MUST pass | state is `inferred`, `unknown`, or `blocked`; no qualifying evidence |
| `offline_strategy` | `verified` or `inferred` | at least one of `official_doc` or `github` | none | state is `unknown` or `blocked`; `blocked_by` present on a passing claim |
| `redaction_fit` | `verified` or `inferred` | at least one of `github` or `probe_output` | none | state is `unknown` or `blocked`; `blocked_by` present on a passing claim |
| `crate_first_fit` | `verified` or `inferred` | at least one of `official_doc`, `github`, or `package_registry` | none | state is `unknown` or `blocked`; `blocked_by` present on a passing claim |
| `reproducibility` | `verified` or `inferred` | at least one `official_doc` and one `package_registry` | none | state is `unknown` or `blocked`; required evidence kinds missing; `blocked_by` present on a passing claim |

## Evaluation Run Directory Contract

The run directory is:

`docs/agents/.uaa-temp/recommend-next-agent/runs/<run_id>/`

All v2 scratch runs MUST set `run-status.json.workflow_version = "discovery_enabled_v2"`.

`promote` MUST branch on `workflow_version`, not on incidental file absence.

For `workflow_version = "discovery_enabled_v2"`:

- `discovery/**` is required
- promote MUST fail if `discovery/**` is missing

The successful v2 run artifact set MUST include:

- `run-status.json`
- `seed.snapshot.toml`
- `candidate-pool.json`
- `eligible-candidates.json`
- `scorecard.json`
- `sources.lock.json`
- `comparison.generated.md`
- `approval-draft.generated.toml`
- `run-summary.md`
- `candidate-dossiers/<agent_id>.json`
- `candidate-validation-results/<agent_id>.json`
- `discovery/candidate-seed.generated.toml`
- `discovery/discovery-summary.md`
- `discovery/sources.lock.json`

`generate` MUST copy `research/discovery-input/**` into `runs/<run_id>/discovery/` on both success and insufficiency.

## `run-status.json`

`run-status.json` MUST contain at least:

- `workflow_version`
- `run_id`
- `status`, one of `success`, `success_with_candidate_errors`, `insufficient_eligible_candidates`, `run_fatal`
- `next_action`, one of `none`, `expand_discovery`, `stop`
- `generated_at`
- `research_dir`
- `run_dir`
- `eligible_candidate_ids`
- `shortlist_ids`
- `recommended_agent_id`
- `candidate_status_counts`
- `metrics`
- `errors`
- `approved_agent_id`
- `approval_recorded_at`
- `override_reason`
- `committed_review_dir`
- `committed_packet_path`
- `committed_approval_artifact_path`

Scratch `run-status.json` MUST already contain all promote-time bookkeeping fields with `null` values where not yet known.

`candidate_status_counts` MUST always include integer keys:

- `eligible`
- `candidate_rejected`
- `candidate_error`

## Insufficiency Semantics

When fewer than 3 eligible candidates survive:

- `run-status.json.status` MUST be `insufficient_eligible_candidates`
- `run-status.json.next_action` MUST be `expand_discovery` after pass 1
- `run-status.json.next_action` MUST be `stop` after pass 2
- `run-summary.md` MUST include grouped rejection reasons

Grouped insufficiency reasons are frozen to:

- `already_onboarded`
- `missing_public_install_surface`
- `missing_public_cli_surface`
- `sdk_not_cli_product`
- `insufficient_dossier_proof`
- `other_candidate_error`

In the insufficiency case, `generate` MUST write:

- `run-status.json`
- `seed.snapshot.toml`
- `candidate-pool.json`
- `eligible-candidates.json`
- `candidate-dossiers/<agent_id>.json` for every seeded candidate
- `candidate-validation-results/<agent_id>.json` for every seeded candidate
- `run-summary.md`
- `discovery/**`

In the insufficiency case, `generate` MUST NOT write:

- `scorecard.json`
- evaluation-run `sources.lock.json`
- `comparison.generated.md`
- `approval-draft.generated.toml`

The absence of those success-only artifacts is part of the contract.

## Evaluation `sources.lock.json`

The evaluation-run `sources.lock.json` MUST:

- cover every seeded candidate, not just shortlisted candidates
- be derived from dossier evidence objects plus runner probe outputs
- not require live network fetching by `generate`
- contain only bounded provenance metadata, not full page bodies

Top-level fields MUST include:

- `run_id`
- `generated_at`
- `candidates`

`candidates` is an ordered array in seed order.

## Packet Constraints

The canonical packet MUST preserve the existing section numbering and exactly-3 comparison-table shape from `docs/templates/agent-selection/cli-agent-selection-packet-template.md`.

Additional required packet rules:

- section 4 MUST contain exactly 3 candidate rows
- section 4 notes MUST reference dossier evidence ids and/or probe result ids
- section 5 rationale MUST reference dossier evidence ids and/or probe result ids
- freeform uncited rationale is insufficient
- the canonical packet is the maintainer decision surface for approve-or-override, but `approved-agent.toml` remains the normative approval artifact consumed by the create lane
- section 5 MUST end with exactly these three non-empty lines in this order:
  - `Approve recommended agent`
  - `Override to shortlisted alternative`
  - `Stop and expand research`
- section 6 MUST preserve exactly this split:
  - `reproducible now`
  - `blocked until later`
- sections 7-9 are semantically required implementation-handoff sections, not merely present headings
- section 7 MUST contain these exact subsection labels:
  - `Manifest root expectations`
  - `Wrapper crate expectations`
  - `agent_api` backend expectations
  - `UAA promotion expectations`
  - `Support/publication expectations`
  - `Likely seam risks`
- section 8 MUST contain these exact subsection labels:
  - `Manifest-root artifacts`
  - `Wrapper-crate artifacts`
  - `agent_api` artifacts
  - `UAA promotion-gate artifacts`
  - `Docs/spec artifacts`
  - `Evidence/fixture artifacts`
- section 9 MUST contain these exact subsection labels:
  - `Required workstreams`
  - `Required deliverables`
  - `Blocking risks`
  - `Acceptance gates`

## Template Audit Result

No packet-template change is required for v2.

Rationale:

- discovery changes pool formation and provenance, not the maintainer decision surface
- the stable packet heading set, section order, decision lines, section 6 split, and fixed 3-candidate table shape already cover the promoted output
- packet provenance remains anchored by the existing template plus the newly required committed `discovery/**` subtree

## No-Drift Template Rule

If `docs/templates/agent-selection/cli-agent-selection-packet-template.md` is updated later, it MUST preserve:

- the packet title block shape
- section numbering
- section headings
- section order
- all `Provenance:` lines
- the fixed 3-candidate table shape
- the existing packet heading names without renaming

## Promote Semantics (Model B)

Promotion uses Model B with these exact exceptions:

- only `run-status.json` and `run-summary.md` may differ at promote time
- all other run artifacts remain byte-copies

Allowed `run-status.json` deltas are limited to:

- finalized metrics
- approved / recommended decision bookkeeping
- final committed path references

Allowed `run-summary.md` deltas are limited to:

- finalized metrics summary
- approved / recommended decision summary

No other sections or evidence content may change at promote time.

`promote` MUST derive reviewed descriptor/default content from `seed.snapshot.toml` already copied into the run.

The live seed file MAY be checked for existence only and MUST NOT be used as the reviewed input source.

For v2 runs, `promote` MUST copy `runs/<run_id>/discovery/**` into the committed review run at the same relative path.

## Committed Review Directory

The committed review directory is:

`docs/agents/selection/runs/<run_id>/`

It MUST include:

- `candidate-dossiers/**` for every seeded candidate
- `candidate-validation-results/**` for every seeded candidate
- `discovery/candidate-seed.generated.toml`
- `discovery/discovery-summary.md`
- `discovery/sources.lock.json`
- all other run artifacts copied from the scratch run
