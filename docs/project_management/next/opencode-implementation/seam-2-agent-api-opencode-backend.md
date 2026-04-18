---
seam_id: SEAM-2
seam_slug: agent-api-opencode-backend
type: integration
status: proposed
execution_horizon: next
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - ../opencode-cli-onboarding/governance/seam-3-closeout.md
    - ../opencode-cli-onboarding/governance/seam-4-closeout.md
  required_threads:
    - THR-04
    - THR-05
    - THR-06
  stale_triggers:
    - wrapper-owned event or completion semantics drift after `SEAM-1` lands
    - capability advertisement or extension-registry rules under `docs/specs/unified-agent-api/**` drift
    - redaction, bounded-payload, or validation posture drift from the backend contract
    - new multi-backend evidence changes the no-promotion recommendation carried by `THR-04`
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S99
  status: pending
open_remediations: []
---

# SEAM-2 - `agent_api` OpenCode backend

- **Goal / value**: implement the OpenCode backend inside `crates/agent_api/` so the universal
  facade can run OpenCode through the landed wrapper without leaking raw backend semantics or
  confusing backend support with UAA support.
- **Scope**
  - In:
    - add OpenCode backend wiring in `crates/agent_api/`
    - map request inputs onto the landed wrapper surface
    - map wrapper-owned events and completion into the universal envelope
    - advertise only deterministic capabilities the backend can honor
    - keep unsupported backend-specific extension keys fail closed before spawn
    - preserve redaction, bounded payloads, passthrough visibility, and DR-0012 completion gating
  - Out:
    - changing wrapper-owned transport or parser semantics
    - promoting backend-specific behavior into universal support claims
    - broad publication work beyond what the backend needs to expose to `SEAM-3`
    - reopening the no-active-UAA-promotion recommendation without a stale trigger
- **Primary interfaces**
  - Inputs:
    - `THR-04`
    - `THR-05`
    - `docs/specs/opencode-agent-api-backend-contract.md`
    - `docs/specs/opencode-wrapper-run-contract.md`
    - `docs/specs/unified-agent-api/run-protocol-spec.md`
    - `docs/specs/unified-agent-api/extensions-spec.md`
    - existing backend harness patterns under `crates/agent_api/src/backends/**`
  - Outputs:
    - `crates/agent_api/**` OpenCode backend implementation
    - OpenCode feature or backend registration plumbing in `crates/agent_api/Cargo.toml` and
      related backend modules
    - deterministic backend regression tests
    - `THR-06`
- **Key invariants / rules**:
  - wrapper-owned semantics stay upstream of this seam
  - public payloads remain bounded, redacted, and free of raw protocol lines or provider-specific
    diagnostics
  - capability advertisement remains fail closed and no broader than the backend can honor
  - DR-0012 completion gating is preserved through the existing backend harness posture rather than
    ad hoc backend-local gating logic
  - backend-specific behavior remains backend-visible until a stale-trigger-driven promotion pass
    proves otherwise
- **Dependencies**
  - Direct blockers:
    - `SEAM-1`
  - Transitive blockers:
    - `THR-04`
  - Direct consumers:
    - `SEAM-3`
  - Derived consumers:
    - future capability inventory and support publication maintenance
- **Touch surface**:
  - `crates/agent_api/**`
  - `crates/agent_api/Cargo.toml`
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/unified-agent-api/run-protocol-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
- **Verification**:
  - targeted backend tests should run under `cargo test -p unified-agent-api --features opencode`
  - regression coverage must explicitly include redaction, bounded payloads, unsupported
    extensions, capability advertisement, and DR-0012 completion gating
  - if this seam **consumes** upstream contracts, verification depends on accepted `SEAM-1`
    evidence rather than re-deriving wrapper behavior locally
  - live provider-backed OpenCode runs are not the default done-ness gate for this seam
- **Canonical contract refs**:
  - `docs/specs/opencode-agent-api-backend-contract.md`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/unified-agent-api/run-protocol-spec.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
- **Risks / unknowns**:
  - Risk: event translation may tempt the backend to leak raw or unstable payloads into the
    universal envelope.
  - De-risk plan: keep bounded-payload and redaction coverage explicit at the backend-test layer.
  - Risk: extension handling or capability advertisement may over-claim support because OpenCode
    exposes additional helper surfaces outside v1 scope.
  - De-risk plan: keep unsupported behavior fail closed and pin capabilities to the backend
    contract.
  - Risk: backend tests may bypass the canonical harness-owned finality posture.
  - De-risk plan: use existing harness patterns and keep DR-0012 checks explicit.
- **Rollout / safety**:
  - keep the backend implementation crate-first and evidence-first
  - no UAA promotion work is implied by backend completion
  - treat upstream wrapper or registry drift as a stop-and-revalidate condition
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `next` because it depends on the landed
    wrapper and manifest foundation from `SEAM-1` and should not become the source of truth for
    wrapper semantics.
  - Which threads matter most: `THR-05`, `THR-06`
  - What the first seam-local review should focus on: request mapping, event/completion mapping,
    capability posture, extension ownership, and harness-owned finality behavior
  - Boundary slice intent: reserve `S00` because seam-local planning will likely need a dedicated
    backend-contract and registration boundary before implementation slices begin
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-03`
  - Threads likely to advance: `THR-06`
  - Review-surface areas likely to shift after landing: the end-to-end run workflow and the
    support-layer publication boundary
  - Downstream seams most likely to require revalidation: `SEAM-3`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in
    pre-exec verification for the producing seam.
