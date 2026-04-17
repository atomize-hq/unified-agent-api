# Seam Map - OpenCode CLI onboarding

Primary axis: **crate-first onboarding with risk-first contract hardening**. The pack freezes one
headless OpenCode v1 run surface first, then hands that contract to the wrapper/manifest seam,
then to the `agent_api` backend seam, and only then to the UAA promotion seam.

## Execution horizon (v2.5 policy)

- Active seam: `SEAM-4`
- Next seam: none
- Future seams: none

Note: `SEAM-2` and `SEAM-3` are now closed and serve as closeout-backed upstream evidence, and
`SEAM-4` is the active promotion-review seam for the pack.

## Seams

1. **SEAM-1 - Runtime surface and evidence lock**
   - Execution horizon: future
   - Type: integration
   - Owns: the canonical v1 OpenCode run surface, deferred-surface policy, install/auth/provider
     posture, maintainer smoke expectations, and the explicit handoff contract that downstream
     wrapper planning must consume.
   - Why it is first: the old pack already showed that every downstream decision depends on whether
     `opencode run --format json` really is the bounded wrapper seam and what evidence is required
     before anyone treats that as current input.
   - Expected outputs:
     - explicit runtime/evidence contract text grounded in the source packet and charter
     - a downstream-ready deferred-surface list for `serve`, `acp`, `run --attach`, and direct
       interactive TUI posture
     - a seam-exit handoff that lets the wrapper seam consume one locked input rather than packet
       prose

2. **SEAM-2 - Wrapper crate and manifest foundation**
   - Execution horizon: future
   - Type: capability
   - Owns: the implementation planning boundary for `crates/opencode/` and
     `cli_manifests/opencode/`, including spawn/stream/completion/parsing boundaries, fixture and
     fake-binary posture, and manifest-root artifact inventory/update rules.
   - Why it is now future: the seam has landed and now serves as the published upstream handoff
     that `SEAM-3` consumes.
   - Expected outputs:
     - a wrapper-owned event/completion/redaction contract for OpenCode
     - a manifest-root artifact contract for `cli_manifests/opencode/`
     - explicit downstream inputs for the backend seam without reopening the runtime lock

3. **SEAM-3 - `agent_api` backend mapping**
   - Execution horizon: future
   - Type: integration
   - Owns: mapping the wrapper contract into `AgentWrapperRunRequest`, `AgentWrapperEvent`, and
     `AgentWrapperCompletion`; capability advertisement; backend-specific extension ownership; and
     fixture-first validation requirements.
   - Why it is now future: the seam has landed and now serves as the closeout-backed upstream
     backend handoff that `SEAM-4` consumes.
   - Expected outputs:
     - a backend-owned mapping contract for `opencode`
     - explicit capability and extension boundaries that stay aligned with the universal specs
     - a seam-exit handoff that gives the promotion seam concrete backend behavior to review

4. **SEAM-4 - UAA promotion and publication follow-on**
   - Execution horizon: active
   - Type: conformance
   - Owns: the boundary between backend support and UAA-promoted support, including which behaviors
     remain backend-specific, which candidate `agent_api.*` promotions are justified, and whether a
     separate follow-on pack is required for canonical spec or capability-matrix updates.
   - Why it is now active: `SEAM-3` has published concrete backend behavior and extension ownership
     through closeout-backed `THR-03`, so promotion review is now the current execution target.
   - Expected outputs:
     - an explicit backend-support versus UAA-promotion recommendation
     - a bounded follow-on pack recommendation for any canonical spec or matrix changes
     - recorded non-promotion paths for backend-specific or unstable OpenCode behavior
