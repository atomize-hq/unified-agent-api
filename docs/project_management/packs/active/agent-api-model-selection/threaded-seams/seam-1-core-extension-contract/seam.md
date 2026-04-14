---
seam_id: SEAM-1
seam_slug: core-extension-contract
status: closed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-1-core-extension-contract.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - any canonical spec or registry delta for agent_api.config.model.v1 semantics
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S3
  status: passed
open_remediations: []
---
# SEAM-1 - Core extension key contract

## Seam Brief (Restated)

- **Goal / value**: keep `agent_api.config.model.v1` pinned to one verified canonical contract so downstream seams can implement safely against a single source of truth.
- **Type**: integration
- **Scope**
  - In:
    - verify that canonical specs + ADR + pack restatements agree on v1 semantics (C-01..C-04)
    - update canonical specs first if drift is found, then sync ADR + pack in the same change
    - publish a downstream-citable verification record with a commit/PR reference (not a local HEAD note)
  - Out:
    - backend advertising, normalization implementation, or argv wiring
- **Touch surface**:
  - `docs/specs/unified-agent-api/extensions-spec.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/contract.md`
  - `docs/specs/unified-agent-api/run-protocol-spec.md`
  - `docs/adr/0020-unified-agent-api-model-selection.md`
  - `docs/project_management/packs/active/agent-api-model-selection/{README.md,scope_brief.md,threading.md,seam-1-core-extension-contract.md}`
- **Verification**:
  - For a seam that produces owned contracts, this seam's pre-exec readiness is about making the contract text concrete and synchronized across canonical sources.
  - Publication or acceptance of the owned contract artifact is recorded as evidence in the verification record and later closeout, not treated as an external prerequisite.
- **Basis posture**:
  - Currentness: current
  - Upstream closeouts assumed: none
  - Required threads: `THR-01`
  - Stale triggers: canonical spec/reg entry deltas after the verification record is published
- **Threading constraints**
  - Upstream blockers: none
  - Downstream blocked seams: `SEAM-2`, `SEAM-3`, `SEAM-4`, `SEAM-5`
  - Contracts produced: `C-01`, `C-02`, `C-03`, `C-04`
  - Contracts consumed: none (canonical docs are evidence inputs)

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: downstream seams must be able to cite a single published verification record before implementing or advertising the capability.
- **Expected contracts to publish**: `C-01`, `C-02`, `C-03`, `C-04`
- **Expected threads to publish / advance**: `THR-01`
- **Likely downstream stale triggers**:
  - canonical spec text changes without re-running the verification pass
  - pack/ADR restatement drift after canonical changes
- **Expected closeout evidence**:
  - a verification record entry that cites a commit/PR reference
  - links to any canonical doc edits (if drift was found)
- **Contracts produced (owned)**:
  - `MS-C01`: unified extension-key definition for `agent_api.config.model.v1`; authoritative text lives in `docs/specs/unified-agent-api/extensions-spec.md` with registry anchoring in `docs/specs/unified-agent-api/capabilities-schema-spec.md`; S1 verifies and, if needed, reconciles the canonical wording.
  - `MS-C02`: absence semantics for the key; authoritative text lives in `docs/specs/unified-agent-api/extensions-spec.md`; S1 verifies that absence still preserves backend defaults everywhere the pack and ADR restate it.
  - `MS-C03`: pre-spawn validation schema and pinned `InvalidRequest` message; authoritative text lives in `docs/specs/unified-agent-api/extensions-spec.md` and inherited error taxonomy references in `docs/specs/unified-agent-api/contract.md`; S1 verifies the exact validation posture and S2 republishes it in synced planning docs.
  - `MS-C04`: backend-owned runtime rejection posture and terminal error-event rule; authoritative text lives across `docs/specs/unified-agent-api/extensions-spec.md`, `docs/specs/unified-agent-api/contract.md`, and `docs/specs/unified-agent-api/run-protocol-spec.md`; S1 verifies the cross-doc alignment and S2 records the gate that downstream seams depend on.
- **Contracts consumed**:
  - None from other seams. SEAM-1 is the producer seam for the contract set in this pack and uses canonical universal specs plus ADR/pack text only as evidence to verify or restate its own owned contracts.
- **Dependency edges honored**:
  - `SEAM-1 blocks SEAM-2`: S1 must finish with either reconciled canonical docs or a recorded pass before SEAM-2 can claim advertising/normalization work is unblocked.
  - `SEAM-1 blocks SEAM-3`: S2 publishes the synchronization reference that Codex mapping work must cite before merging.
  - `SEAM-1 blocks SEAM-4`: S2 publishes the synchronization reference that Claude mapping work must cite before merging.
  - `SEAM-1 blocks SEAM-5`: the tests seam may draft work earlier, but only the S2-published verification gate satisfies the blocker for implementation-adjacent assertions.
- **Parallelization notes**:
  - What can proceed now: S1.T1 can begin immediately because it has no upstream seam blockers; draft note-taking for S2 can happen in parallel as long as it does not claim the gate is satisfied.
  - What must wait: any final ADR/pack sync text, verification-record publication, or downstream seam unblock claims wait on S1 proving `pass: no unresolved canonical-doc delta`.

## Slice index

- `S1` -> `slice-1-canonical-drift-verification.md`
- `S2` -> `slice-2-adr-pack-sync-and-gate-publication.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
