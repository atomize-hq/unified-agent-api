---
seam_id: SEAM-4
seam_slug: uaa-promotion-and-publication-follow-on
status: closed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-4-uaa-promotion-and-publication-follow-on.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-3-closeout.md
  required_threads:
    - THR-03
    - THR-04
  stale_triggers:
    - any change in built-in backend support that affects promotion eligibility
    - capability-matrix or universal extension-registry rule changes
    - backend mapping changes that materially alter what OpenCode exposes
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
  planned_location: S99
  status: passed
open_remediations: []
---
# SEAM-4 - uaa-promotion-and-publication-follow-on

## Seam Brief (Restated)

- **Goal / value**: produce an explicit, reviewable answer to what OpenCode support remains
  backend-specific and what, if anything, is justified for UAA promotion once backend behavior is
  concrete.
- **Type**: conformance
- **Scope**
  - In:
    - review the backend seam's actual capability and extension surface
    - distinguish backend support, backend-specific extension coverage, and candidate
      `agent_api.*` promotion
    - identify any required follow-on pack for canonical spec or capability-matrix changes
    - preserve explicit non-promotion paths for backend-specific or unstable behavior
  - Out:
    - editing canonical specs or capability matrices in this seam
    - reopening wrapper or backend scope except through explicit blocker escalation
    - treating backend completion as automatic universal promotion
- **Touch surface**:
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
  - future `docs/project_management/**` follow-on packs
  - `docs/project_management/next/opencode-cli-onboarding/`
- **Verification**:
  - `review.md` is the authoritative pre-exec review artifact and must show that backend evidence,
    multi-backend promotion rules, and no-promotion outputs are all concrete enough to execute.
  - `review.md` must also show that the capability matrix is supporting evidence only, not runtime
    truth.
  - owned promotion-review outputs must be concrete in seam-local planning before execution; they
    do not require canonical spec or matrix edits to land in this seam.
  - accepted or published owned-contract artifacts remain post-exec evidence recorded in seam exit
    and closeout, not a prerequisite for `exec-ready`.
- **Canonical contract refs**:
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed: `governance/seam-3-closeout.md`
  - Required threads: `THR-03`, `THR-04`
  - Stale triggers:
    - any change in built-in backend support that affects promotion eligibility
    - capability-matrix or universal extension-registry rule changes
    - backend mapping changes that materially alter what OpenCode exposes
- **Threading constraints**
  - Upstream blockers: none; `THR-03` is now revalidated against the landed `SEAM-3` closeout.
  - Downstream blocked seams: pack closeout and future follow-on packs
  - Contracts produced: `C-07`
  - Contracts consumed: `C-05`, `C-06`
  - Canonical contract refs:
    - `docs/specs/opencode-agent-api-backend-contract.md`
    - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
    - `docs/specs/unified-agent-api/extensions-spec.md`
    - `docs/specs/unified-agent-api/capability-matrix.md`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99`
- **Why this seam needs an explicit exit gate**: pack closeout and future follow-on work need a
  closeout-backed record of what stayed backend-specific, what if anything is recommended for UAA
  promotion, whether a follow-on execution pack is required, and whether the answer is explicitly
  no-promotion when cross-backend evidence is missing.
- **Expected contracts to publish**: `C-07`
- **Expected threads to publish / advance**: `THR-04`
- **Likely downstream stale triggers**:
  - backend mapping or capability advertisement drift
  - capability-matrix or universal extension-registry rule changes
  - new multi-backend evidence that changes promotion eligibility
- **Expected closeout evidence**:
  - explicit promotion recommendation or explicit no-promotion outcome
  - explicit follow-on pack requirement or an explicit no-follow-on answer
  - explicit publication of `THR-04` for pack closeout and future work
  - explicit statement that the capability matrix was used only as supporting evidence, not as
    runtime truth

## Slice index

- `S1` -> `slice-1-backend-evidence-and-publication-boundary-review.md`
- `S2` -> `slice-2-promotion-recommendation-and-no-promotion-routing.md`
- `S3` -> `slice-3-follow-on-pack-and-thread-handoff.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-4-closeout.md`
