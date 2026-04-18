---
pack_id: opencode-implementation
pack_version: v1
pack_status: extracted
source_ref: docs/project_management/next/opencode-cli-onboarding/next-steps-handoff.md
execution_horizon:
  active_seam: SEAM-2
  next_seam: SEAM-3
---

# Scope Brief - OpenCode implementation

- **Goal**: create the single code-facing execution pack that turns the closed OpenCode
  onboarding/contracts work into implementation planning for `cli_manifests/opencode/`,
  `crates/opencode/`, and the `crates/agent_api` OpenCode backend.
- **Why now**: the repo has already selected OpenCode, locked the runtime/evidence contracts, and
  closed the onboarding pack; what is still missing is the implementation pack that makes the next
  code work deterministic.
- **Primary user(s) + JTBD**: repo maintainers need a crate-first execution plan they can follow to
  land OpenCode without reopening candidate selection, inventing a new bridge artifact, or
  accidentally conflating backend support with UAA promotion.
- **In-scope**:
  - one OpenCode implementation pack with multiple internal seams and workstreams
  - explicit ownership for manifest root, wrapper crate, `agent_api` backend, and bounded
    publication follow-through
  - reuse of `THR-04` and the closed onboarding closeouts as the authoritative inbound handoff
  - explicit verification surfaces, commands, and acceptance gates
  - explicit stale and reopen triggers carried forward from the closed onboarding pack
- **Out-of-scope**:
  - reopening OpenCode candidate selection
  - generic lifecycle, template, or future-agent process codification
  - reusable handoff manifest or bridge sidecar work
  - a generic scaffolder for future agents
  - active UAA promotion work unless the published stale triggers fire
- **Success criteria**:
  - the pack names one active seam and one next seam with a bounded future follow-through seam only
  - `SEAM-1` is sufficient to plan `crates/opencode/` and `cli_manifests/opencode/` concretely
  - `SEAM-2` is positioned to implement the OpenCode backend in `crates/agent_api` without
    redefining wrapper semantics
  - deterministic validation is the default proof path across wrapper, manifest, backend, and
    publication surfaces
  - the four support layers remain separate and UAA promotion stays out of scope by default
- **Constraints**:
  - this is one pack, not a family of sibling packs
  - no new bridge ledger, sidecar manifest, or lifecycle document
  - canonical OpenCode contract refs remain the existing `docs/specs/**` documents
  - exactly one `active` seam and one `next` seam are used for this extraction
  - live provider-backed smoke is basis-lock evidence only, not routine completion criteria
- **External systems / dependencies**:
  - OpenCode CLI (`opencode run --format json`)
  - provider-backed auth and model routing captured in `docs/specs/opencode-onboarding-evidence-contract.md`
  - Rust workspace plumbing (`Cargo.toml`, `crates/xtask`, `crates/agent_api`)
  - support publication outputs under `cli_manifests/support_matrix/current.json` and
    `docs/specs/unified-agent-api/support-matrix.md`
- **Known unknowns / risks**:
  - the real OpenCode JSON event stream may pressure wrapper-owned event normalization more than
    the planning smoke evidence suggested
  - the current support/publication tooling hard-codes only Codex and Claude roots/backends, so
    OpenCode publication follow-through is real bounded work
  - deterministic evidence layout for the first OpenCode manifest root may expose gaps that Codex
    or Claude already solved differently
  - backend capability advertisement may surface backend-only behavior that must remain visible
    without being misread as UAA support
- **Assumptions**:
  - the closed onboarding closeouts are the current authoritative basis until one of their stale
    triggers fires
  - the new wrapper crate will follow the existing naming pattern (`unified-agent-api-opencode`,
    library name `opencode`) unless seam-local review finds blocking repo evidence to the contrary
  - root-specific `xtask` additions are allowed when they are required to land OpenCode, but this
    pack does not authorize generic future-agent scaffolding work
  - support publication follow-through may update hard-coded root/backend registries, but it must
    not turn into UAA promotion planning
