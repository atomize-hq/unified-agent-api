---
seam_id: SEAM-3
seam_slug: agent-api-backend-mapping
status: closed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-3-agent-api-backend-mapping.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-2-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
  stale_triggers:
    - wrapper-owned event or completion semantics drift
    - manifest inventory or pointer-rule drift that changes backend assumptions
    - capability or extension registry changes under `docs/specs/unified-agent-api/**`
    - new evidence that bounded payload, redaction, or fixture posture must differ
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
# SEAM-3 - agent-api-backend-mapping

## Seam Brief (Restated)

- **Goal / value**: turn the published OpenCode wrapper and manifest handoff into a bounded
  `agent_api` backend surface without leaking backend specifics into the universal API.
- **Type**: integration
- **Scope**
  - In:
    - define request, event, and completion mapping from the wrapper contract into the universal
      backend envelope
    - define bounded payload, redaction, and fail-closed behavior for OpenCode backend responses
    - define capability advertisement and backend-specific extension ownership for `backend.opencode.*`
    - define fixture-first validation expectations that do not require a live provider by default
    - publish one downstream-ready handoff for `THR-03`
  - Out:
    - changing wrapper-owned semantics from `SEAM-2`
    - promoting backend-specific behavior into universal `agent_api.*` capabilities
    - editing universal canonical specs beyond the bounded backend-owned surfaces already named here
- **Touch surface**:
  - `crates/agent_api/**`
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
  - `docs/project_management/next/opencode-cli-onboarding/`
- **Verification**:
  - `review.md` is the authoritative pre-exec review artifact and must show that backend payload
    leakage, over-advertised capabilities, and validation ambiguity are all closed.
  - owned backend contract and extension decisions must be concrete enough in seam-local planning
    that implementation can proceed without reopening `SEAM-2`.
  - consumed wrapper and manifest contracts must remain current inputs grounded in the landed
    `SEAM-2` closeout and the published canonical docs under `docs/specs/**`.
- **Canonical contract refs**:
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed: `governance/seam-2-closeout.md`
  - Required threads: `THR-01`, `THR-02`, `THR-03`
  - Stale triggers:
    - wrapper-owned event or completion semantics drift
    - manifest inventory or pointer-rule drift that changes backend assumptions
    - capability or extension registry changes under `docs/specs/unified-agent-api/**`
    - new evidence that bounded payload, redaction, or fixture posture must differ
- **Threading constraints**
  - Upstream blockers: none; `THR-01` and `THR-02` are now revalidated against the landed
    `SEAM-1` and `SEAM-2` closeouts.
  - Downstream blocked seams: `SEAM-4`
  - Contracts produced: `C-05`, `C-06`
  - Contracts consumed: `C-01`, `C-02`, `C-03`, `C-04`
  - Canonical contract refs:
    - `docs/specs/opencode-agent-api-backend-contract.md`
    - `docs/specs/opencode-wrapper-run-contract.md`
    - `docs/specs/opencode-cli-manifest-contract.md`
    - `docs/specs/opencode-onboarding-evidence-contract.md`
    - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
    - `docs/specs/unified-agent-api/extensions-spec.md`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99`
- **Why this seam needs an explicit exit gate**: downstream promotion review needs a closeout-backed
  record that the backend mapping, capability advertisement posture, and `THR-03` handoff landed
  without silently widening universal support claims.
- **Expected contracts to publish**: `C-05`, `C-06`
- **Expected threads to publish / advance**: `THR-03`
- **Likely downstream stale triggers**:
  - wrapper contract drift that changes backend mapping inputs
  - capability advertisement or extension ownership drift
  - validation or redaction posture drift that changes promotion assumptions
- **Expected closeout evidence**:
  - landed backend-owned mapping and extension contract updates
  - landed validation posture and redaction boundary evidence
  - explicit publication of `THR-03` for downstream promotion review

## Slice index

- `S00` -> `slice-00-backend-contract-and-extension-baselines.md`
- `S1` -> `slice-1-request-event-and-completion-mapping.md`
- `S2` -> `slice-2-capability-advertisement-and-extension-ownership.md`
- `S3` -> `slice-3-validation-and-redaction-boundary.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-3-closeout.md`
