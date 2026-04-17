---
seam_id: SEAM-2
seam_slug: wrapper-crate-and-manifest-foundation
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-2-wrapper-crate-and-manifest-foundation.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - any change to the canonical v1 run surface or deferred-surface policy
    - manifest-root artifact inventory changes in existing repo patterns
    - new evidence that fake-binary, fixture, or offline-parser posture must change
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S99
  status: pending
open_remediations: []
---
# SEAM-2 - Wrapper crate and manifest foundation

## Seam Brief (Restated)

- **Goal / value**: define the bounded implementation surface for `crates/opencode/` and
  `cli_manifests/opencode/` so backend work later consumes one wrapper-owned truth.
- **Type**: capability
- **Scope**
  - In:
    - define the OpenCode wrapper spawn, streaming, completion, parsing, and redaction boundaries
    - define offline parser, fixture, fake-binary, and maintainer-smoke posture for wrapper work
    - define the artifact inventory, pointer/update rules, and validation expectations for
      `cli_manifests/opencode/`
    - publish one downstream-ready handoff for `THR-02`
  - Out:
    - implementing the backend adapter under `crates/agent_api/`
    - promoting backend-specific behavior into universal `agent_api.*` capabilities
    - widening helper-surface scope beyond the `SEAM-1` contract
- **Touch surface**:
  - `crates/opencode/**`
  - `cli_manifests/opencode/**`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
  - `docs/project_management/next/opencode-cli-onboarding/`
- **Verification**:
  - `review.md` is the authoritative pre-exec review artifact and must show that wrapper-boundary
    creep, manifest-inventory drift, and fixture ambiguity are all closed.
  - owned wrapper and manifest contract decisions must be concrete enough in seam-local planning
    that implementation can proceed without reopening `SEAM-1`.
  - accepted or published owned-contract artifacts remain post-exec evidence recorded in seam exit
    and closeout, not a prerequisite for `exec-ready`.
- **Canonical contract refs**:
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed: `governance/seam-1-closeout.md`
  - Required threads: `THR-01`, `THR-02`
  - Stale triggers:
    - canonical run-surface or deferred-helper policy drift
    - manifest artifact inventory or pointer rule drift
    - fixture/fake-binary posture changes that alter reproducible validation
- **Threading constraints**
  - Upstream blockers: none; `THR-01` is now revalidated against the landed `SEAM-1` closeout.
  - Downstream blocked seams: `SEAM-3`, `SEAM-4`
  - Contracts produced: `C-03`, `C-04`
  - Contracts consumed: `C-01`, `C-02`
  - Canonical contract refs:
    - `docs/specs/opencode-wrapper-run-contract.md`
    - `docs/specs/opencode-onboarding-evidence-contract.md`
    - `docs/specs/opencode-cli-manifest-contract.md`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99`
- **Why this seam needs an explicit exit gate**: downstream backend and promotion seams need a
  closeout-backed record that wrapper-owned runtime detail, manifest inventory rules, and THR-02
  landed without silently widening helper-surface scope.
- **Expected contracts to publish**: `C-03`, `C-04`
- **Expected threads to publish / advance**: `THR-02`
- **Likely downstream stale triggers**:
  - wrapper event/completion boundary drift
  - manifest inventory or validation-rule drift
  - fixture/fake-binary posture drift that changes backend assumptions
- **Expected closeout evidence**:
  - landed wrapper-owned runtime/contract updates
  - landed manifest contract and inventory evidence
  - explicit publication of `THR-02` for downstream seams

## Slice index

- `S00` -> `slice-00-wrapper-and-manifest-contract-baselines.md`
- `S1` -> `slice-1-wrapper-runtime-contract-shape.md`
- `S2` -> `slice-2-manifest-inventory-and-evidence-layout.md`
- `S3` -> `slice-3-backend-handoff-and-fixture-boundary.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
