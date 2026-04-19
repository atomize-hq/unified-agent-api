---
pack_id: opencode-cli-onboarding
pack_version: v1
pack_status: extracted
source_ref: docs/project_management/next/cli-agent-onboarding-third-agent-packet.md
execution_horizon:
  active_seam: null
  next_seam: null
---

# Scope Brief - OpenCode CLI onboarding

- **Goal**: preserve the closed OpenCode onboarding pack as one governance-ready planning surface
  that downstream work can trust without reopening already-supported crate-first sequencing.
- **Why now**: the repo already selected `OpenCode` as the first real third CLI agent and gathered
  maintainer smoke evidence for the likely v1 run surface, but the current planning artifacts are
  still organized as phase tasks rather than stable seams, threads, and closeout boundaries.
- **Primary user(s) + JTBD**:
  - wrappers maintainers: "freeze the OpenCode v1 wrapper seam and wrapper-owned artifact boundary
    before implementation planning spreads across crates"
  - `agent_api` maintainers: "consume a concrete wrapper handoff so backend mapping stays
    mechanical, bounded, and safe-by-default"
  - reviewers and release operators: "keep backend support, backend-specific extension coverage,
    and UAA promotion decisions separated and auditable"
- **In-scope**:
  - freeze the canonical v1 OpenCode runtime surface and the evidence needed to trust it
  - define the wrapper crate and `cli_manifests/opencode/` planning boundary
  - define the bounded `agent_api` backend mapping, capability, and extension planning boundary
  - define the UAA promotion-review seam and the conditions for any follow-on canonical spec or
    capability-matrix work
  - scaffold threading, review surfaces, and governance closeout for those seams
- **Out-of-scope**:
  - implementing `crates/opencode/`, `cli_manifests/opencode/`, or `crates/agent_api/`
  - bundling `opencode serve`, `opencode acp`, `opencode run --attach`, or direct interactive TUI
    operation into the v1 wrapper seam
  - claiming UAA-promoted support before backend scope is concrete and the multi-backend promotion
    rule is satisfied
- **Success criteria**:
  - this directory remains a v2.5 seam pack with no active, next, or future seam in the forward
    window
  - `SEAM-1` through `SEAM-4` remain closed with closeout-backed handoffs instead of packet-era
    candidate language
  - the four landed normative OpenCode specs remain the canonical downstream contract surface:
    `opencode-wrapper-run-contract.md`, `opencode-onboarding-evidence-contract.md`,
    `opencode-cli-manifest-contract.md`, and `opencode-agent-api-backend-contract.md`
  - backend support, backend-specific extension coverage, and UAA promotion decisions remain
    separated and auditable in the closed pack record
- **Constraints**:
  - this closeout-consistency cleanup is docs-only and stays inside
    `docs/project_management/next/opencode-cli-onboarding/` plus the canonical evidence contract in
    `docs/specs/opencode-onboarding-evidence-contract.md`
  - the onboarding charter and `docs/specs/**` remain authoritative when planning prose drifts
  - exactly one `active` seam is used at a time, and `next` is used when future work remains in
    the forward queue
  - lifecycle state and basis freshness must remain separate
  - OpenCode backend-specific capability or extension behavior must remain backend-specific until
    promotion is justified by the canonical universal rules
- **External systems / dependencies**:
  - `OpenCode` CLI install paths, auth flows, and provider-backed model execution
  - existing wrapper crates under `crates/codex/` and `crates/claude_code/`
  - future manifest evidence root `cli_manifests/opencode/`
  - universal API specs under `docs/specs/unified-agent-api/**`
  - repo validation surfaces such as `cargo run -p xtask -- capability-matrix` and future
    OpenCode-specific validation commands
- **Known unknowns / risks**:
  - provider-auth friction may make reproducible wrapper evidence harder than packet-era smoke
    suggests
  - OpenCode event shape or helper-surface ergonomics may tempt downstream work to widen the v1
    wrapper contract too early
  - capability and extension publication could drift if the backend seam invents semantics not
    already grounded in the wrapper or universal specs
  - the UAA promotion seam could accidentally become a cleanup bucket unless backend-specific
    fallback behavior stays explicit
- **Assumptions**:
  - execution horizon is inferred from the critical path: contract/evidence hardening first, then
    wrapper + manifest planning, then backend mapping, then promotion review
  - the landed `SEAM-4` closeout now resolves the promotion-review seam with no queued follow-on
    seam required under the current evidence basis
  - the landed OpenCode-specific durable contracts live under `docs/specs/**`, and any later
    additive contract surfaces should continue to follow that canonical posture
