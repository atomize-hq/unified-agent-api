---
seam_id: SEAM-1
seam_slug: wrapper-crate-and-manifest-foundation
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-1-wrapper-crate-and-manifest-foundation.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../../opencode-cli-onboarding/governance/seam-1-closeout.md
    - ../../../opencode-cli-onboarding/governance/seam-2-closeout.md
    - ../../../opencode-cli-onboarding/governance/seam-4-closeout.md
  required_threads:
    - THR-04
    - THR-05
  stale_triggers:
    - OpenCode CLI event-shape drift on the canonical `run --format json` surface
    - accepted v1 controls drift off the canonical `run --format json` surface
    - provider-auth or model-routing posture changes invalidate the evidence prerequisite record
    - manifest inventory, pointer rules, or deterministic replay posture drift from the closed onboarding basis
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
# SEAM-1 - Wrapper crate and manifest foundation

## Seam Brief (Restated)

- **Goal / value**: land the first OpenCode implementation foundation across `crates/opencode/`
  and `cli_manifests/opencode/` so downstream work consumes one wrapper-owned and manifest-owned
  truth instead of inferring behavior from the closed onboarding pack.
- **Type**: capability
- **Scope**
  - In:
    - create the initial `crates/opencode/` wrapper crate and workspace wiring needed to host it
    - create the initial `cli_manifests/opencode/` root with committed inventory, pointers,
      schemas, rules, reports, and current snapshot posture
    - keep parsing, event typing, completion handoff, offline parser, and redaction inside the
      wrapper seam
    - define deterministic fake-binary, fixture, transcript, and offline-parser proof paths for
      the wrapper and manifest root
    - add only the OpenCode-root-specific `xtask` or validation plumbing required to land this
      root
  - Out:
    - implementing the OpenCode backend under `crates/agent_api/`
    - generic scaffolding for future agents
    - widening scope to `serve`, `acp`, `run --attach`, or direct interactive TUI behavior
- **Touch surface**:
  - `Cargo.toml`
  - `crates/opencode/**`
  - `cli_manifests/opencode/**`
  - `crates/xtask/**`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
- **Verification**:
  - `review.md` is the authoritative pre-exec review artifact and now shows that helper-surface
    creep, manifest-root ambiguity, and deterministic-evidence drift are all closed for planning.
  - owned wrapper and manifest decisions are concrete enough in seam-local planning that execution
    can proceed without waiting on the seam's own post-exec publication artifacts.
  - targeted wrapper tests, fake-binary or transcript fixtures, and root validation remain the
    default proof path; live provider-backed smoke stays basis-lock evidence only.
- **Canonical contract refs**:
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
  - `docs/specs/unified-agent-api/support-matrix.md`
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed:
    - `../../../opencode-cli-onboarding/governance/seam-1-closeout.md`
    - `../../../opencode-cli-onboarding/governance/seam-2-closeout.md`
    - `../../../opencode-cli-onboarding/governance/seam-4-closeout.md`
  - Required threads: `THR-04`, `THR-05`
  - Stale triggers:
    - canonical OpenCode event-shape drift on `run --format json`
    - accepted control drift off the canonical run surface
    - provider-auth or model-routing posture changes invalidate the prerequisite record
    - manifest inventory or deterministic replay posture drifts from the closed onboarding basis
- **Threading constraints**
  - Upstream blockers: none; `THR-04` is now revalidated against the landed onboarding closeouts
    and published no-new-promotion recommendation.
  - Downstream blocked seams: `SEAM-2`, `SEAM-3`
  - Contracts produced: `C-01`, `C-02`
  - Contracts consumed: `C-07` via the upstream onboarding handoff
  - Canonical contract refs:
    - `docs/specs/opencode-wrapper-run-contract.md`
    - `docs/specs/opencode-onboarding-evidence-contract.md`
    - `docs/specs/opencode-cli-manifest-contract.md`
    - `docs/specs/unified-agent-api/support-matrix.md`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99`
- **Why this seam needs an explicit exit gate**: downstream backend and publication work need one
  closeout-backed record that the wrapper crate, manifest root, deterministic evidence posture, and
  `THR-05` handoff all landed without widening helper-surface scope or introducing generic
  scaffolding.
- **Expected contracts to publish**: `C-01`, `C-02`
- **Expected threads to publish / advance**: `THR-05`
- **Likely downstream stale triggers**:
  - wrapper event, completion, or redaction ownership drift
  - manifest-root inventory or pointer-rule drift
  - deterministic replay, fake-binary, or transcript posture drifts from landed evidence
- **Expected closeout evidence**:
  - landed `crates/opencode/**` wrapper artifacts and targeted proof coverage
  - landed `cli_manifests/opencode/**` root inventory and root-validator evidence
  - explicit `THR-05` publication for `SEAM-2` and `SEAM-3`

## Slice index

- `S00` -> `slice-00-workspace-and-manifest-contract-baselines.md`
- `S1` -> `slice-1-wrapper-crate-runtime-and-fixture-foundation.md`
- `S2` -> `slice-2-manifest-root-artifacts-and-validator-scope.md`
- `S3` -> `slice-3-deterministic-evidence-and-downstream-handoff.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
