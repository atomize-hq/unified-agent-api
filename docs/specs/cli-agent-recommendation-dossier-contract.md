# CLI Agent Recommendation Dossier Contract

Status: Draft  
Date (UTC): 2026-04-28  
Canonical location: `docs/specs/cli-agent-recommendation-dossier-contract.md`

Normative language: this contract uses RFC 2119 requirement keywords (`MUST`, `MUST NOT`, `SHOULD`).

## Purpose

This contract freezes the pre-create recommendation lane for selecting the next CLI agent.

It defines:

- the research-first workflow boundary
- the exact research scratch inputs
- the exact post-research runner inputs and outputs
- the dossier schema
- the frozen `research-metadata.json` envelope
- the packet constraints
- the promote-time Model B rules

If skill text, operator procedure, or planning prose diverge from this document, this contract wins.

## Normative References

- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/templates/agent-selection/cli-agent-selection-packet-template.md`

## Workflow Boundary

The recommendation lane is a frozen two-stage workflow:

1. A skill-led research phase writes reviewed research artifacts.
2. A deterministic runner phase consumes only those reviewed artifacts.
3. A promote phase commits review artifacts, updates the canonical packet, and renders the final approval artifact.

The runner is post-research only:

- it MUST NOT fetch open-ended web, docs, package-registry, or GitHub evidence
- it MUST NOT mutate the research artifacts
- it MUST NOT replace the reviewed seed snapshot

## Runner CLI Contract

`generate` is frozen to:

```sh
python3 scripts/recommend_next_agent.py generate \
  --seed-file docs/agents/selection/candidate-seed.toml \
  --research-dir docs/agents/.uaa-temp/recommend-next-agent/research/<run_id> \
  --run-id <run_id> \
  --scratch-root docs/agents/.uaa-temp/recommend-next-agent/runs
```

`promote` keeps the existing shape, but MUST promote from the frozen snapshot already copied into the run and MUST NOT reread the live seed file as reviewed input.

The repo-local scratch root for this lane is:

`docs/agents/.uaa-temp/recommend-next-agent/`

`.staging/` directories remain reserved for internal promote-time staging and MUST NOT be used as operator scratch space.

## Timestamp Format

All persisted timestamps in this milestone MUST use UTC RFC3339 / ISO-8601 with trailing `Z`.

This applies to:

- `generated_at`
- `approval_recorded_at`
- dossier / evidence `captured_at`

## Research Directory Contract

The research directory is:

`docs/agents/.uaa-temp/recommend-next-agent/research/<run_id>/`

It MUST contain:

- `seed.snapshot.toml`
- `research-summary.md`
- `research-metadata.json`
- `dossiers/<agent_id>.json`

Optional research artifacts MAY include:

- `evidence-cache/`
- `screenshots/`
- `notes/`

The dossier set is exact:

- if `seed.snapshot.toml` contains `N` seeded candidates, `dossiers/` MUST contain exactly `N` dossier files
- every seeded candidate MUST have exactly one dossier
- missing any seeded candidate dossier is a contract failure

## `research-metadata.json` Envelope

`research-metadata.json` MUST be valid JSON and MUST contain exactly these top-level fields:

- `run_id`: string
- `evidence_collection_time_seconds`: integer
- `fetched_source_count`: integer

No additional top-level fields are part of the v1 contract.

`fetched_source_count` is defined as the total unique fetched remote sources used by the research phase across all seeded candidates.

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

### Claims

`claims` MUST contain exactly these keys:

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

### Evidence

Each evidence object MUST contain:

- `evidence_id`
- `kind`, one of `official_doc`, `github`, `package_registry`, `ancillary`, `probe_output`
- optional `url`
- `title`
- `captured_at`
- `sha256`
- `excerpt`

### Probe Requests

`probe_requests` MUST remain an array in the dossier schema and MAY be empty.

Each probe request MUST contain:

- `probe_kind`, one of `help`, `version`
- `binary`
- `required_for_gate`, boolean

The milestone MUST NOT add a single-required-probe rule, an exactly-one-probe rule, or any other minimum-cardinality requirement for `probe_requests`.

## Evidence Budgets

Per candidate dossier limits are frozen to:

- max `12` evidence refs
- max `4` `official_doc` refs
- max `2` `package_registry` refs
- max `3` `github` refs
- max `3` `ancillary` refs
- max `3` blocked steps
- max `1200` chars per freeform note field

The committed review artifacts MUST NOT persist full remote page bodies.

## Probe Policy

The runner-owned probe policy is frozen to:

- allowed probe kinds: `help`, `version`
- allowed binary regex: `^[A-Za-z0-9._-]+$`
- max probes: `2` per candidate
- timeout: `5` seconds per probe
- max captured stdout+stderr: `32768` bytes per probe
- inherited environment only: `PATH`, `HOME`, `TMPDIR`

Disallowed probe forms include:

- shell strings
- `/` in the binary name
- env expansion
- redirection
- pipes
- authenticated commands
- network-dependent commands

The contract does not require any candidate to carry a required probe. `required_for_gate` remains per-entry metadata only.

If a probe violates allowlist, times out, exceeds the byte cap, or exits non-zero, the runner MUST record it in `candidate-validation-results/<agent_id>.json`.

The runner MUST escalate that probe failure to `candidate_error` only when the probe was `required_for_gate`. Otherwise, the runner MUST continue on dossier evidence alone.

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

## Run Directory Contract

The run directory is:

`docs/agents/.uaa-temp/recommend-next-agent/runs/<run_id>/`

The successful run artifact set MUST include:

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

Run-local cardinality is exact:

- `candidate-dossiers/<agent_id>.json` MUST exist for every seeded candidate
- `candidate-validation-results/<agent_id>.json` MUST exist for every seeded candidate

## `run-status.json`

`run-status.json` MUST contain at least:

- `run_id`
- `status`, one of `success`, `success_with_candidate_errors`, `insufficient_eligible_candidates`, `run_fatal`
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

`errors` MUST be an array of objects with:

- `scope`, one of `run`, `candidate`
- `agent_id`, nullable
- `code`
- `message`

### Metrics

`run-status.json.metrics` MUST contain all required metric keys in scratch output.

Approval-dependent metrics are:

- `maintainer_time_to_decision_seconds`
- `shortlist_override`
- `predicted_blocker_count`
- `later_discovered_blocker_count`

Those approval-dependent metrics MUST be `null` in scratch output. Promote MAY replace only those `null` values with finalized values.

Research / run-derived metrics MUST already be concrete in scratch output:

- `rejected_before_scoring_count`
- `evidence_collection_time_seconds`
- `fetched_source_count`

## `candidate-validation-results/<agent_id>.json`

Each validation result MUST contain at least:

- `agent_id`
- `status`, one of `eligible`, `candidate_rejected`, `candidate_error`
- `schema_valid`
- `hard_gate_results`
- `probe_results`
- `rejection_reasons`
- `error_reasons`
- `evidence_ids_used`
- `notes`

`hard_gate_results` MUST be keyed by:

- `non_interactive_execution`
- `offline_strategy`
- `observable_cli_surface`
- `redaction_fit`
- `crate_first_fit`
- `reproducibility`

Each hard-gate result MUST contain:

- `status`, one of `pass`, `fail`, `blocked`, `unknown`
- `rule_id`
- `rejection_reason`
- `evidence_ids`
- `notes`

Each probe result entry MUST contain:

- `probe_kind`
- `binary`
- `required_for_gate`
- `status`, one of `passed`, `failed`, `skipped`
- `exit_code`, nullable
- `timed_out`, boolean
- `captured_output_ref`, nullable
- `notes`

## Score Artifacts

`scorecard.json` top-level MUST include:

- `dimensions`
- `primary_dimensions`
- `secondary_dimensions`
- `shortlist_order`
- `recommended_agent_id`
- `candidates`

Each `scorecard.json.candidates[agent_id]` entry MUST include:

- `scores`
- `primary_sum`
- `secondary_sum`
- `notes`

`candidate-pool.json` top-level MUST include:

- `run_id`
- `candidates`

Each `candidate-pool.json.candidates[]` entry MUST include:

- `agent_id`
- `status`
- `rejection_reasons`
- `error_reasons`
- `shortlisted`
- `recommended`

`eligible-candidates.json` top-level MUST include:

- `run_id`
- `eligible_candidates`

Each `eligible-candidates.json.eligible_candidates[]` entry MUST include:

- `agent_id`
- `scores`
- `primary_sum`
- `secondary_sum`

Deterministic ordering is frozen to:

- `candidate-pool.json.candidates` follows candidate order from `seed.snapshot.toml`
- `eligible-candidates.json.eligible_candidates` follows the frozen shortlist sort order across all eligible candidates
- `scorecard.json.shortlist_order` remains the canonical exactly-3 shortlist order

## `sources.lock.json`

`sources.lock.json` MUST:

- cover every seeded candidate, not just shortlisted candidates
- be derived from dossier evidence objects plus runner probe outputs
- not require live network fetching by `generate`
- contain only bounded provenance metadata, not full page bodies

Top-level fields MUST include:

- `run_id`
- `generated_at`
- `candidates`

`candidates` is an ordered array in seed order.

Each candidate entry MUST contain:

- `agent_id`
- `evidence_refs`
- `probe_output_refs`

`evidence_refs` MUST preserve dossier evidence order. `probe_output_refs` MUST preserve probe execution order.

## Status and Failure Semantics

Candidate statuses are:

- `eligible`
- `candidate_rejected`
- `candidate_error`

Run statuses are:

- `success`
- `success_with_candidate_errors`
- `insufficient_eligible_candidates`
- `run_fatal`

`candidate_rejected` means the dossier was structurally valid enough to evaluate but failed a hard eligibility gate.

`candidate_error` means the runner could not safely evaluate the candidate because of malformed dossier input, schema validation failure, or a probe failure that blocked gate satisfaction.

Run-level fatal cases include:

- missing `--research-dir` after parsing
- missing `seed.snapshot.toml`
- unreadable or invalid `seed.snapshot.toml`
- missing dossier for any seeded candidate

`generate` exits `0` only for:

- `success`
- `success_with_candidate_errors` when at least 3 candidates remain eligible

It exits non-zero for:

- `insufficient_eligible_candidates`
- `run_fatal`

## Scoring Source Rules

The milestone preserves the existing public scorecard dimensions, the `0-3` bucket scale, the primary/secondary split, and the shortlist tie-break order.

What changes is the evidence source:

- no score dimension may rely on keyword-hit heuristics alone when a typed dossier claim exists
- `Architecture fit for this repo` and `Reproducibility & access friction` MUST read dossier claims first
- score notes and packet notes MUST cite dossier `evidence_id` values and/or probe result ids

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
- section 7 MUST contain these exact subsection labels, matching PLAN.md verbatim in wording and capitalization:
  - `Manifest root expectations`
  - `Wrapper crate expectations`
  - `agent_api` backend expectations
  - `UAA promotion expectations`
  - `Support/publication expectations`
  - `Likely seam risks`
- section 8 MUST contain these exact subsection labels, matching PLAN.md verbatim in wording and capitalization:
  - `Manifest-root artifacts`
  - `Wrapper-crate artifacts`
  - `agent_api` artifacts
  - `UAA promotion-gate artifacts`
  - `Docs/spec artifacts`
  - `Evidence/fixture artifacts`
- section 9 MUST contain these exact subsection labels, matching PLAN.md verbatim in wording and capitalization:
  - `Required workstreams`
  - `Required deliverables`
  - `Blocking risks`
  - `Acceptance gates`
- the appendix MUST include:
  - loser rationale for the other two shortlisted candidates
  - strategic contenders if any
  - dated evidence provenance

## No-Drift Template Rule

If `docs/templates/agent-selection/cli-agent-selection-packet-template.md` is updated, it MUST preserve:

- the packet title block shape
- section numbering
- section headings
- section order
- all `Provenance:` lines
- the fixed 3-candidate table shape
- the existing packet heading names without renaming

No template expansion may change section order or table shape.

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

## Committed Review Directory

The committed review directory is:

`docs/agents/selection/runs/<run_id>/`

It MUST include:

- `candidate-dossiers/**` for every seeded candidate
- `candidate-validation-results/**` for every seeded candidate
- all other run artifacts copied from the scratch run

## Metric Definitions

Required metric keys and derivations are:

- `maintainer_time_to_decision_seconds`
  - `approval_recorded_at - generated_at` in whole seconds
- `shortlist_override`
  - `true` when `approved_agent_id != recommended_agent_id`, else `false`
- `predicted_blocker_count`
  - count of blocked claim entries for the approved candidate plus count of `blocked_steps`
- `later_discovered_blocker_count`
  - integer count from later onboarding evidence or `null` if that evidence does not exist yet
- `rejected_before_scoring_count`
  - count of seeded candidates whose final status is `candidate_rejected` or `candidate_error`
- `evidence_collection_time_seconds`
  - copied from `research-metadata.json`
- `fetched_source_count`
  - copied from `research-metadata.json`

Later onboarding evidence is optional at proving-run time. If it does not exist yet, `later_discovered_blocker_count` MUST be `null` and MUST NOT block closeout.
