---
seam_id: SEAM-3
seam_slug: agent-api-backend-mapping
type: integration
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-2-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
  stale_triggers:
    - changes to the wrapper-owned event/completion contract
    - capability or extension registry changes under `docs/specs/unified-agent-api/**`
    - new evidence that backend payload bounding or redaction must differ from current assumptions
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

# SEAM-3 - `agent_api` backend mapping

- **Goal / value**: turn the wrapper-owned OpenCode contract into a bounded `agent_api` backend
  surface without leaking backend specifics into the universal API.
- **Scope**
  - In:
    - define run-request mapping from the universal facade into the OpenCode wrapper
    - define event-bucket mapping into the universal envelope
    - define completion, redaction, and bounded-payload obligations
    - define capability advertisement and backend-specific extension-key posture for `opencode`
    - define fixture-backed validation requirements that do not require a live provider by default
  - Out:
    - changing wrapper-owned semantics from `SEAM-2`
    - promoting backend-specific behavior into universal `agent_api.*` capabilities
    - editing canonical universal specs in this extraction pass
- **Primary interfaces**
  - Inputs:
    - `THR-01`
    - `THR-02`
    - `C-01`
    - `C-02`
    - `docs/specs/opencode-wrapper-run-contract.md`
    - `docs/specs/opencode-onboarding-evidence-contract.md`
    - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
    - `docs/specs/unified-agent-api/extensions-spec.md`
  - Outputs:
    - backend-owned mapping and capability contract
    - backend-specific extension ownership guidance
    - `THR-03` handoff for promotion review
- **Key invariants / rules**:
  - the universal envelope stays small, stable, bounded, and redacted
  - completion finality remains mandatory
  - backend-specific behavior stays under `backend.opencode.*` until promotion is explicitly
    justified
  - wrapper contract changes must flow backward to `SEAM-2`, not be silently patched here
- **Dependencies**
  - Direct blockers:
    - `SEAM-2`
  - Transitive blockers:
    - `SEAM-1`
  - Direct consumers:
    - `SEAM-4`
  - Derived consumers:
    - future capability-matrix and backend regression work
- **Touch surface**:
  - `crates/agent_api/**`
  - future `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
  - `docs/project_management/next/opencode-cli-onboarding/`
- **Verification**:
  - if this seam **consumes** upstream contracts, verification may depend on accepted upstream
    evidence from `SEAM-1` and `SEAM-2`
  - seam-local review should prove the backend can advertise only what it really supports and fail
    closed on unsupported extension keys before spawn
  - verification should also prove redaction, bounded payloads, and completion-finality obligations
    remain explicit and testable
- **Canonical contract refs**:
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
- **Risks / unknowns**:
  - Risk: OpenCode event mapping may tempt the backend seam to smuggle raw or unstable payloads into
    the universal envelope.
  - De-risk plan: keep best-effort parity bounded by the charter's stable event buckets and data
    limits.
  - Risk: backend-specific extension behavior could drift into unofficial universal semantics.
  - De-risk plan: keep ownership and advertisement rules explicit and defer promotion to `SEAM-4`.
- **Rollout / safety**:
  - fixture-first validation remains the default gate
  - provider-backed smoke may supplement verification but must not become the only proof path
  - backend support must remain distinct from UAA promotion
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `active` because `SEAM-2` has now
    published a closeout-backed wrapper/manifest handoff and this seam owns the next executable
    backend work.
  - Which threads matter most: `THR-02`, `THR-03`
  - What the first seam-local review should focus on: whether the wrapper handoff is concrete
    enough, whether capability/extension advertisement is explicit, and whether test obligations
    cover redaction and completion-finality risk
  - Boundary slice intent: reserve `S00` because seam-local planning needs a contract-definition
    slice for backend-owned mapping and extension semantics before mapping slices begin
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-05`, `C-06`
  - Threads likely to advance: `THR-03`
  - Review-surface areas likely to shift after landing: the high-level workflow and repo touch
    surfaces
  - Downstream seams most likely to require revalidation: `SEAM-4`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in
    pre-exec verification for the producing seam.
