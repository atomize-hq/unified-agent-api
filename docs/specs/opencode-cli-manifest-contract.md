# OpenCode CLI Manifest Contract (v1)

Status: Normative
Scope: canonical manifest-root contract for `cli_manifests/opencode/`

## Normative language

This document uses RFC 2119 requirement keywords (`MUST`, `MUST NOT`, `SHOULD`).

## Purpose

Define the manifest-root contract for OpenCode support evidence so downstream wrapper, backend,
and promotion planning can rely on one auditable root instead of packet-era prose or ad hoc
repository conventions.

This contract is about manifest-root evidence only. It does not define wrapper runtime semantics or
backend mapping behavior.

## Normative references

- `docs/specs/opencode-wrapper-run-contract.md`
- `docs/specs/opencode-onboarding-evidence-contract.md`
- `docs/specs/unified-agent-api/support-matrix.md`
- `docs/specs/unified-agent-api/capability-matrix.md`

If there is a conflict between planning prose and this document, this contract wins for the
OpenCode manifest-root evidence surface.

## Canonical manifest root

The canonical OpenCode manifest root MUST be `cli_manifests/opencode/`.

The root MUST use the same evidence model already established by the repo’s other CLI manifest
roots. The required artifact inventory for the root is:

- committed pointer files for promotion state
- committed version metadata for validated versions
- committed snapshot artifacts for observed upstream surfaces
- committed coverage or report artifacts for wrapper-derived review
- a committed root snapshot file representing the current validated state
- a wrapper coverage declaration or equivalent committed coverage manifest
- `README.md` with human-readable root conventions and generation guidance
- `SCHEMA.json` defining committed artifact shape rules
- `RULES.json` defining deterministic merge and comparison rules
- `VALIDATOR_SPEC.md` defining how the rules are enforced
- `VERSION_METADATA_SCHEMA.json` defining version metadata shape
- `artifacts.lock.json` capturing the committed artifact inventory
- `current.json` as the current root snapshot or union snapshot, as applicable
- `min_supported.txt` and `latest_validated.txt` as the authoritative promotion pointers
- `pointers/latest_supported/**` and `pointers/latest_validated/**` as per-target pointer files
- `versions/*.json` as per-version workflow metadata
- `reports/**` as committed coverage or validation reports
- `snapshots/**` as committed upstream snapshot artifacts
- `wrapper_coverage.json` as the committed wrapper coverage declaration or equivalent manifest
- `supplement/commands.json` when explicit help omissions must be recorded

This contract intentionally separates upstream manifest evidence from wrapper support claims and
from UAA unified-support claims.

Optional or debug-only raw help captures MAY exist as generated artifacts, but they MUST NOT be the
authoritative committed support signal.

## Pointer and update rules

- `min_supported.txt` and `latest_validated.txt` MUST remain the authoritative promotion pointers.
- Pointer files MUST contain a single line and MUST end with a newline.
- Pointer files SHOULD exist for every expected target triple, even when a value is not yet known.
- `none` is the required placeholder when a pointer target has no known supported or validated
  version.
- `current.json` MUST reflect the currently validated snapshot state for the root.
- Version metadata files under `versions/*.json` MUST track workflow status for committed versions
  and MUST be kept deterministic.
- Snapshot, report, and coverage artifacts MUST be updated mechanically from the validated evidence
  set; they MUST NOT be handwritten as free-form support claims.
- `artifacts.lock.json` MUST describe the committed inventory for the root and MUST stay aligned
  with the artifact classes listed above.

## Metadata posture

The OpenCode manifest root MUST preserve the repo's existing truth-store model:

- `versions/*.json` is workflow metadata, not published support truth.
- `current.json`, pointer files, snapshots, and reports are evidence artifacts, not wrapper or UAA
  support claims.
- `wrapper_coverage.json` is the committed wrapper coverage declaration for manifest-root review;
  it MUST remain distinct from backend reports and unified-support publication.
- `supplement/commands.json` MAY record explicit help omissions, but it MUST NOT fabricate help
  detail or act as a support ledger.

## Evidence expectations

The manifest root MUST preserve the repo’s separation between three evidence layers:

- upstream manifest support
- wrapper/backend-crate support
- UAA unified support

For OpenCode, the manifest root MAY prove upstream CLI inventory and wrapper coverage, but it MUST
NOT by itself claim backend support or unified support.

The committed evidence set SHOULD be sufficient for reviewers to answer:

- which OpenCode CLI commands and flags were observed
- which target triples were validated
- which versions are current, minimum supported, and latest validated
- which wrapper-covered commands are supported, intentionally unsupported, or missing

## Validation and retention

- Committed artifacts under the manifest root MUST validate against the root schema and rules.
- Retention SHOULD follow the repo’s sliding-window pattern for validated snapshots and reports,
  plus any versions referenced by the promotion pointers.
- Deterministic ordering rules SHOULD be used for command inventories, flags, and report material
  so repeated runs do not churn committed artifacts unnecessarily.

## Support publication boundary

- Adding or updating `cli_manifests/opencode/` evidence MUST NOT, by itself, create backend support
  or UAA support.
- Backend support begins only when the wrapper crate produces committed wrapper-derived evidence
  and reports that satisfy the repo’s support-publication policy.
- UAA promotion remains governed by the universal support and promotion contracts, not by this
  manifest root.

## Baseline verification checklist

Before this contract is treated as settled, the repo SHOULD confirm:

- the artifact inventory matches the established manifest-root pattern used by existing CLI roots
- the inventory is explicit enough that later seams can distinguish the root from wrapper-backed or
  UAA-backed support evidence
- promotion pointers are explicit and mechanically checkable
- committed evidence stays separate from raw help captures
- wrapper coverage remains distinct from backend or UAA support claims
- the root can be cited directly by later seams without referring back to planning prose
