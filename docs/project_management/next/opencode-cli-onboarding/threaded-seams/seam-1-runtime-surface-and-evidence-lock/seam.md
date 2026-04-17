---
seam_id: SEAM-1
seam_slug: runtime-surface-and-evidence-lock
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-1-runtime-surface-and-evidence-lock.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - OpenCode CLI event-shape drift on the canonical run surface
    - new evidence that `serve`, `acp`, `run --attach`, or interactive TUI flow must be in v1
    - provider-auth posture changes that invalidate the current maintainer smoke assumptions
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
# SEAM-1 - Runtime surface and evidence lock

## Seam Brief (Restated)

- **Goal / value**: freeze the exact OpenCode v1 runtime surface and evidence posture so every
  downstream seam consumes concrete repo-owned contract text instead of packet prose.
- **Type**: integration
- **Scope**
  - In:
    - lock `opencode run --format json` as the only canonical v1 wrapper surface
    - define the prerequisite, smoke, replay, and reopen rules that make that choice actionable
    - keep `serve`, `acp`, `run --attach`, and direct interactive TUI flows explicitly deferred
    - publish one downstream-ready handoff for `THR-01`
  - Out:
    - implementing `crates/opencode/`
    - defining wrapper event taxonomy or manifest inventory rules
    - mapping OpenCode output into `agent_api`
    - making UAA promotion decisions
- **Touch surface**:
  - `../../seam-1-runtime-surface-and-evidence-lock.md`
  - `../../review_surfaces.md`
  - `../../../cli-agent-onboarding-charter.md`
  - `../../../cli-agent-onboarding-third-agent-packet.md`
  - `../../../../specs/opencode-wrapper-run-contract.md`
  - `../../../../specs/opencode-onboarding-evidence-contract.md`
- **Verification**:
  - `review.md` is the authoritative pre-exec review artifact and must show that helper-surface
    creep, evidence ambiguity, and canonical-doc drift are all closed.
  - `docs/specs/opencode-wrapper-run-contract.md` must make the runtime boundary concrete enough
    that `SEAM-2` can implement without reopening v1 scope.
  - `docs/specs/opencode-onboarding-evidence-contract.md` must make prerequisites, replay versus
    live-smoke posture, and reopen triggers concrete enough that downstream seams can treat the
    basis as current.
- **Canonical contract refs**:
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed: none
  - Required threads: `THR-01`
  - Stale triggers:
    - OpenCode CLI event-shape drift on `run --format json`
    - any need to pull `serve`, `acp`, `run --attach`, or interactive TUI behavior into v1
    - provider/auth changes that break the current maintainer-smoke assumptions or deterministic
      replay plan
- **Threading constraints**
  - Upstream blockers: none
  - Downstream blocked seams: `SEAM-2`, `SEAM-3`, `SEAM-4`
  - Contracts produced: `C-01`, `C-02`
  - Contracts consumed: none
  - Canonical contract refs:
    - `docs/specs/opencode-wrapper-run-contract.md`
    - `docs/specs/opencode-onboarding-evidence-contract.md`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99`
- **Why this seam needs an explicit exit gate**: downstream seams cannot legally activate against
  packet prose; they need a closeout-backed record that the runtime and evidence contracts landed
  unchanged and that no helper surface was silently promoted.
- **Expected contracts to publish**: `C-01`, `C-02`
- **Expected threads to publish / advance**: `THR-01`
- **Likely downstream stale triggers**:
  - canonical run-surface semantics drift
  - evidence posture drift that changes the fixture-versus-live-smoke boundary
  - helper-surface promotion pressure
- **Expected closeout evidence**:
  - landed copies or diffs for the two OpenCode spec baselines
  - recorded evidence that the deferred-surface policy still holds
  - explicit publication of `THR-01` for downstream seams

## Slice index

- `S00` -> `slice-00-contract-baselines.md`
- `S1` -> `slice-1-runtime-surface-lock.md`
- `S2` -> `slice-2-evidence-envelope.md`
- `S3` -> `slice-3-downstream-handoff-check.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
