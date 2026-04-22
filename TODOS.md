# TODOS

## Pending

### Formalize Recommendation Automation After Two Governance-Backed Selection Cycles

**What:** Revisit `recommend-agent` automation or a deterministic packet generator only after the repo completes two onboarding selections with explicit comparison, approval, and closeout linkage.

**Why:** M3 identified that the next missing truth is governance and provenance, not candidate automation. The repo should automate recommendation only after it has real approval-versus-recommendation feedback to target.

**Context:** The 2026-04-21 `/autoplan` rebaseline for `PLAN.md` reframed M3 around selection-to-proof governance because `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md` recommended `OpenCode` while the first closed M2 proving run landed `gemini_cli`. That mismatch needs explicit comparison, approval, override, and closeout truth before the repo turns recommendation into tooling.

**Effort:** M
**Priority:** P2
**Depends on:** M3 landing with governance-backed approval artifacts and validated proving-run closeout artifacts

### Compress The Runtime-Owned Onboarding Lane After Governance Truth Lands

**What:** Identify and reduce the dominant wrapper/backend/manual evidence steps that still control lead time after M3 makes selection and approval provenance explicit.

**Why:** M2 solved control-plane mutation. M3 solves governance truth. The next likely bottleneck is the runtime-owned lane, but the repo should name that only after the governance chain is trustworthy enough to measure it cleanly.

**Context:** The 2026-04-21 `/autoplan` for the CLI agent onboarding factory explicitly deferred runtime-lane compression because the current repo still cannot say why `gemini_cli` was approved after `OpenCode` was recommended, or what residual friction the first proving run actually exposed. Once M3 closes that gap, the runtime lane becomes the right next target.

**Effort:** M
**Priority:** P2
**Depends on:** M3 governance artifacts plus one post-M3 onboarding cycle with recorded duration and residual friction truth

## Completed

### Implement The M4 Post-Onboarding Maintenance Lane

**What:** Add a separate maintenance lifecycle for already-onboarded agents: agent-scoped drift detection, a dedicated maintenance packet/request, bounded control-plane refresh ergonomics, and explicit maintenance closeout.

**Why:** `onboard-agent` is the create-mode bridge for new agents. Once an agent is already in the repo, maintainers still need a boring way to detect and repair drift across registry truth, publication outputs, release docs, and closed packet/governance docs without reopening new-agent onboarding.

**Context:** The 2026-04-21 M4 `/autoplan` for the CLI agent onboarding factory reframed this work as a separate post-onboarding lifecycle milestone. The plan anchors the proving run on OpenCode because `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-main-test-outcome-20260420-091704.md` already documents a real stale closeout claim in `docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md`.

**Effort:** M
**Priority:** P2
**Depends on:** M3 landing cleanly with approval-driven onboarding and validated proving-run closeout
**Completed:** v0.3.0 (2026-04-22)

### Create The OpenCode Execution Pack

**What:** Create a single `docs/project_management/next/opencode-implementation/` execution pack that turns the closed OpenCode onboarding contracts into implementation-ready seams for `cli_manifests/opencode/`, `crates/opencode/`, and the OpenCode `crates/agent_api` backend.

**Why:** The repo already finished candidate selection and contract locking for OpenCode, but implementation still lacks one code-facing plan-of-record. This execution pack is the missing bridge between the closed onboarding pack and actual landing work.

**Context:** The 2026-04-18 `/plan-eng-review` for `docs/project_management/next/opencode-cli-onboarding/next-steps-handoff.md` reduced scope to one execution pack, reused the existing `THR-04` and seam closeouts as the bridge, kept UAA promotion out of scope unless stale triggers fire, and required an explicit verification matrix. The pack should consume the published OpenCode contracts and closeouts directly, stay crate-first, and make deterministic replay/fake-binary/fixture validation the default proof path instead of live provider smoke.

**Effort:** M
**Priority:** P2
**Depends on:** Closed OpenCode onboarding pack and normative contracts already landed under `docs/project_management/next/opencode-cli-onboarding/` and `docs/specs/opencode-*.md`
**Completed:** v0.2.3 (2026-04-18)

### Select The First Real Third CLI Agent And Prepare Its Onboarding Packet

**What:** Choose the first real third CLI agent target after phase 1 lands and create a bounded onboarding packet for adding it to the manifest, backend-crate, and UAA promotion pipeline.

**Why:** Phase 1 intentionally proves future-agent readiness with synthetic fixtures only; this follow-on task turns that architectural readiness into an actual new agent integration when the repo is ready for product expansion.

**Context:** The 2026-04-15 `/plan-eng-review` for CLI manifest support-matrix automation explicitly deferred real third-agent onboarding to keep phase 1 focused on semantics cleanup, neutral parity/support tooling, generated support-matrix publication, and validator hardening. The next step after phase 1 is to pick one concrete CLI agent, document why it is the right target, define its manifest/root conventions, identify any upstream-specific seams that the new neutral modules still do not cover, and produce the implementation packet for snapshot, union, wrapper coverage, validation, backend-crate support, and UAA promotion decisions.

**Effort:** M
**Priority:** P2
**Depends on:** Phase 1 support-matrix and neutral parity tooling landing cleanly
**Completed:** planning docs landed (2026-04-18)
