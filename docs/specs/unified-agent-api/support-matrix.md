# Support Matrix Spec — Unified Agent API

Status: Approved
Approved (UTC): 2026-04-15
Canonical location: `docs/specs/unified-agent-api/support-matrix.md`

This document is the authoritative contract for support publication semantics in the Unified Agent API spec set.

Normative language: this spec uses RFC 2119 requirement keywords (`MUST`, `MUST NOT`, `SHOULD`).

## Purpose

This spec defines the meaning of published support truth for the Unified Agent API documentation layer.
It separates support publication from capability advertising so the two concerns do not drift together.

## Support layers

Support publication MUST distinguish the following four layers:

- `manifest support`: what the committed CLI manifest evidence says about a target or version.
- `backend support`: what a backend crate can safely support based on its implementation and manifest inputs.
- `UAA unified support`: what the Unified Agent API can claim as a deterministic cross-agent support statement.
- `passthrough visibility`: backend-specific surface area that remains visible but is not promoted into unified support.

These layers MUST NOT be conflated with workflow status fields, pointer files, or generated overview artifacts.

## Publication targets

The phase-1 publication targets are:

- `cli_manifests/support_matrix/current.json`
- `docs/specs/unified-agent-api/support-matrix.md`

The JSON artifact is the machine-readable publication surface.
This Markdown document is the normative human-readable projection.

Both publication targets MUST describe the same support model.
If they disagree, the repository is in an invalid publication state.

## Target-first primacy

Support truth MUST be target-scoped first.
Per-target rows are the primary publication unit.
Per-version summaries, if present elsewhere in the repository, MUST be treated as projections derived from those rows.

Support publication MUST preserve these distinctions:

- a target can be supported even when another target is not.
- a version summary MUST NOT collapse partial target support into a version-global claim.
- pointer state MAY inform publication, but pointer state alone is not support truth.

## Authority rules

Published support rows MUST be derived from committed repository evidence.

For this spec set:

- manifest evidence is authoritative for manifest support.
- backend implementation evidence is authoritative for backend support.
- Unified Agent API publication text is authoritative for unified support semantics.
- passthrough visibility MUST remain explicit when a backend exposes behavior that is not part of unified support.

The following MUST remain separate from published support truth:

- `validated` and `supported` status fields in version metadata
- generated capability inventory
- runtime backend capability checks

## Neutral root intake

The support-matrix pipeline MUST consume committed evidence from each agent root through one neutral root-intake contract.

For phase 1, that intake contract MUST be limited to these evidence categories under `cli_manifests/<agent>/`:

- `versions/*.json` version metadata
- `pointers/latest_supported/*.txt` and `pointers/latest_validated/*.txt`
- `current.json`
- `reports/**`

This intake contract MUST remain shape-driven rather than agent-name-driven:

- the pipeline MUST reason about root-local evidence categories and paths, not special-case Codex or Claude by name inside shared intake logic.
- the contract MAY preserve root-native target identifiers as loaded evidence; later derivation decides how publication rows compare or project them.
- the contract MUST NOT introduce a second support evidence store outside the committed manifest roots.

This intake contract governs evidence loading only. It MUST NOT change publication targets, support-layer meanings, or the distinction between `validated` and `supported`.

## Validated versus supported

`validated` and `supported` are distinct workflow states.

- `validated` means a version passed the validation matrix and is promotion-grade for the version pointer flow.
- `supported` means wrapper coverage satisfies the stronger support policy for the version and target surface.

The repository MUST NOT treat `validated` as equivalent to `supported`.
The repository MUST NOT treat workflow status as a published support row.
The repository MUST NOT use workflow status as a substitute for target-scoped support evidence.

## Separation from the capability matrix

The support matrix MUST remain separate from `docs/specs/unified-agent-api/capability-matrix.md`.

- The capability matrix documents backend capability advertising.
- The support matrix documents published support truth.
- The two artifacts MAY share source evidence, but they MUST NOT share meaning.
- A change to one artifact MUST NOT be assumed to update the other.

If a reader needs backend capability coverage, they SHOULD use the capability matrix.
If a reader needs published support truth, they MUST use the support matrix.

## Verification checklist

Before downstream work consumes this contract, reviewers MUST confirm:

- the canonical support publication targets are named exactly once and without ambiguity.
- the four support layers have distinct meanings and no overlap with workflow metadata.
- target-scoped rows are described as primary and per-version summaries as derived projections.
- the neutral root-intake contract is limited to committed root evidence and does not introduce agent-name-specific loading semantics.
- `validated` is not treated as equivalent to `supported`.
- the support matrix is explicitly separate from the capability matrix.
- the spec is sufficient for downstream implementation without reopening authority or output-path decisions.

## Change control

Any future update to support publication semantics MUST update this spec first.
If the publication model changes, the README index and any dependent publication docs MUST be updated in the same change.
