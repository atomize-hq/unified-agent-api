---
seam_id: SEAM-2
seam_slug: agent-api-opencode-backend
status: closed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-2-agent-api-opencode-backend.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../../opencode-cli-onboarding/governance/seam-3-closeout.md
    - ../../../opencode-cli-onboarding/governance/seam-4-closeout.md
  required_threads:
    - THR-04
    - THR-05
    - THR-06
  stale_triggers:
    - wrapper-owned event or completion semantics drift after `SEAM-1`
    - capability advertisement or extension-registry rules under `docs/specs/unified-agent-api/**` drift
    - redaction, bounded-payload, or validation posture drift from the backend contract
    - new multi-backend evidence changes the no-promotion recommendation carried by `THR-04`
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
# SEAM-2 - agent-api-opencode-backend

## Seam Brief (Restated)

- **Goal / value**: implement the OpenCode backend inside `crates/agent_api/` so the universal
  facade can consume the landed wrapper and manifest truth without leaking raw backend detail or
  collapsing backend support into UAA support.
- **Type**: integration
- **Scope**
  - In:
    - add OpenCode backend wiring in `crates/agent_api/**`
    - map request inputs, typed wrapper events, and completion state into the universal backend
      envelope
    - advertise only deterministic capabilities the backend can honor
    - keep unsupported extension keys fail closed before spawn
    - preserve redaction, bounded payloads, passthrough visibility, and DR-0012 completion gating
    - publish one downstream-ready handoff for `THR-06`
  - Out:
    - changing wrapper-owned transport or parser semantics
    - promoting backend-specific behavior into UAA support claims
    - support-matrix or capability-matrix publication work beyond what `SEAM-3` owns
- **Touch surface**:
  - `crates/agent_api/**`
  - `crates/agent_api/Cargo.toml`
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/unified-agent-api/run-protocol-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
- **Verification**:
  - `review.md` is the authoritative pre-exec artifact and must show that request mapping,
    capability posture, extension handling, and validation/redaction boundaries are concrete
  - consumed wrapper and manifest contracts must remain grounded in the landed `SEAM-1` closeout
    and the revalidated `THR-05` handoff
  - backend planning stays fixture-first and does not depend on live provider-backed smoke by
    default
- **Canonical contract refs**:
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
  - `docs/specs/unified-agent-api/run-protocol-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
    - `../../../opencode-cli-onboarding/governance/seam-3-closeout.md`
    - `../../../opencode-cli-onboarding/governance/seam-4-closeout.md`
  - Required threads: `THR-04`, `THR-05`, `THR-06`
  - Stale triggers:
    - wrapper-owned event or completion semantics drift after `SEAM-1`
    - capability advertisement or extension-registry rules drift
    - redaction, bounded-payload, or validation posture drift from the backend contract
    - new multi-backend evidence changes the no-promotion recommendation carried by `THR-04`
- **Threading constraints**
  - Upstream blockers: none; `THR-04` and `THR-05` are now current inputs for this seam.
  - Downstream blocked seams: `SEAM-3`
  - Contracts produced: `C-03`
  - Contracts consumed: `C-01`, `C-02`, `C-07`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99`
- **Why this seam needs an explicit exit gate**: downstream publication work needs a closeout-backed
  record that the backend mapping, capability posture, and `THR-06` handoff landed without
  silently widening wrapper or support claims.
- **Expected contracts to publish**: `C-03`
- **Expected threads to publish / advance**: `THR-06`
- **Likely downstream stale triggers**:
  - wrapper-owned runtime semantics drift
  - backend capability advertisement or extension ownership drift
  - validation, redaction, or bounded-payload posture drift
- **Expected closeout evidence**:
  - landed backend implementation and registration surfaces under `crates/agent_api/**`
  - explicit validation and redaction evidence
  - explicit publication of `THR-06` for `SEAM-3`

## Slice index

- `S00` -> `slice-00-backend-contract-and-registration-baselines.md`
- `S1` -> `slice-1-request-event-and-completion-mapping.md`
- `S2` -> `slice-2-capability-advertisement-and-extension-ownership.md`
- `S3` -> `slice-3-validation-and-redaction-boundary.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
