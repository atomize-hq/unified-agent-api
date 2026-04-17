---
seam_id: SEAM-1
seam_slug: runtime-surface-and-evidence-lock
type: integration
status: proposed
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - OpenCode CLI event-shape drift on the canonical run surface
    - new evidence that `serve`, `acp`, `run --attach`, or interactive TUI flow must be in v1
    - provider-auth posture changes that invalidate the current maintainer smoke assumptions
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
open_remediations:
  - REM-001
  - REM-002
---

# SEAM-1 - Runtime surface and evidence lock

- **Goal / value**: replace packet-era OpenCode runtime notes with one explicit v1 run-surface and
  evidence handoff that every downstream seam can trust.
- **Scope**
  - In:
    - freeze `opencode run --format json` as the canonical v1 wrapper seam unless contradictory
      evidence appears
    - capture install paths, auth/provider prerequisites, and maintainer smoke posture
    - define the deferred-surface policy for `serve`, `acp`, `run --attach`, and direct
      interactive TUI operation
    - define revalidation and reopen triggers so downstream seams know when to stop and escalate
  - Out:
    - implementing wrapper or backend code
    - defining final wrapper artifact inventory
    - mapping OpenCode output into `agent_api` events
    - making UAA promotion decisions
- **Primary interfaces**
  - Inputs:
    - `docs/project_management/next/cli-agent-onboarding-charter.md`
    - `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`
    - `docs/project_management/next/opencode-cli-onboarding/plan.md`
    - maintainer-backed smoke observations captured in the source packet
  - Outputs:
    - downstream-ready runtime/evidence contract text
    - explicit deferred-surface list and reopen criteria
    - `THR-01` handoff for the wrapper and backend seams
- **Key invariants / rules**:
  - the v1 wrapper seam stays headless and automation-safe by default
  - raw backend lines or provider secrets must not become wrapper-library API surface by default
  - downstream seams may not silently widen scope to helper surfaces that SEAM-1 keeps deferred
  - contradictory evidence reopens SEAM-1 rather than being normalized away in SEAM-2 or SEAM-3
- **Dependencies**
  - Direct blockers:
    - none
  - Transitive blockers:
    - none
  - Direct consumers:
    - `SEAM-2`, `SEAM-3`, `SEAM-4`
  - Derived consumers:
    - future OpenCode implementation and validation packs
- **Touch surface**:
  - `docs/project_management/next/opencode-cli-onboarding/`
  - `docs/project_management/next/cli-agent-onboarding-charter.md`
  - `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`
  - future `docs/specs/opencode-wrapper-run-contract.md`
  - future `docs/specs/opencode-onboarding-evidence-contract.md`
- **Verification**:
  - seam-local review should prove the runtime surface, deferred-surface policy, and evidence
    posture are concrete enough to drive wrapper planning without ambiguity
  - verification should confirm the contract names all known prerequisite classes: install, auth,
    provider selection, smoke evidence, and reopen triggers
  - because this seam **produces** an owned contract, verification is about making that contract
    concrete enough for downstream planning and implementation rather than requiring the final
    accepted artifact to exist already
- **Canonical contract refs**:
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - `docs/specs/unified-agent-api/extensions-spec.md`
- **Risks / unknowns**:
  - Risk: provider-backed smoke evidence may overfit one maintainer environment.
  - De-risk plan: separate fixture-backed expectations from maintainer-smoke expectations and make
    reopen triggers explicit.
  - Risk: helper surfaces may look attractive enough to pressure the v1 seam boundary.
  - De-risk plan: publish a hard deferred-surface list with explicit "reopen only if" criteria.
- **Rollout / safety**:
  - docs/planning-only in this extraction pass
  - fail closed on contradictory evidence
  - preserve crate-first sequencing so downstream seams consume, rather than reinvent, this contract
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `active` because every downstream planning
    seam depends on one explicit runtime/evidence contract instead of packet prose.
  - Which threads matter most: `THR-01`
  - What the first seam-local review should focus on: whether the current smoke evidence is enough
    to freeze the headless run surface, whether deferred helpers are stated without ambiguity, and
    whether provider/auth posture is concrete enough for downstream planning
  - Boundary slice intent: reserve `S00` in downstream seam-local planning if contract-definition
    cleanup is still needed between the packet evidence, the charter, and the future OpenCode run
    contract doc
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-01`, `C-02`
  - Threads likely to advance: `THR-01`
  - Review-surface areas likely to shift after landing: the high-level onboarding workflow and the
    contract/dependency flow
  - Downstream seams most likely to require revalidation: `SEAM-2`, `SEAM-3`, `SEAM-4`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in
    pre-exec verification for the producing seam.
