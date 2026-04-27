# TODOS

## Pending

### Decide Whether Capability Matrix Markdown Stays Canonical After M5

**What:** After M5 lands, decide whether `docs/specs/unified-agent-api/capability-matrix.md` remains the canonical published truth surface or becomes a rendered view over a more structured canonical artifact.

**Why:** The 2026-04-23 `/autoplan` for M5 found that local and CI verification currently reason about capability publication differently. M5 should unify the gate first. The next question is whether markdown itself should stay the canonical control-plane truth or only the human-readable publication surface.

**Context:** `make preflight` currently omits capability publication freshness while CI runs `cargo run -p xtask -- capability-matrix`, a `git diff`, and `cargo run -p xtask -- capability-matrix-audit`. That is good enough to fix in M5, but the longer-term truth-surface decision should be made explicitly before more agents or more publication consumers pile on.

**Effort:** S
**Priority:** P2
**Depends on:** M5 landing with one canonical capability projection contract and one shared local/CI check-only gate

### Land The Pre-Create Recommendation Lane

**What:** Add the repo-local skill and deterministic runner that research next-agent candidates, apply hard onboarding-fit gates, render the fixed 3-candidate comparison packet, and draft the maintainer-facing `approved-agent.toml` input for the existing `onboard-agent` lane.

**Why:** The repo now has the core post-approval factory. The missing seam is earlier: maintainers still have to do the candidate archaeology by hand before they can run `cargo run -p xtask -- onboard-agent --approval ...`. This work turns that research step into an approval-grade control-plane lane.

**Context:** The 2026-04-27 office-hours and `/plan-eng-review` reframe narrowed the problem from "create-mode gaps" to the pre-create recommendation lane. The repo already has `docs/agents/selection/cli-agent-selection-packet.md`, `docs/templates/agent-selection/cli-agent-selection-packet-template.md`, `crates/xtask/src/approval_artifact.rs`, and the shipped `onboard-agent` workflow. The lane should reuse those surfaces, keep the maintainer as approve-or-override HITL, pin v1 to exactly 3 candidates, and separate run-local evidence capture from explicit promotion to the canonical packet path.

**Effort:** M
**Priority:** P1
**Depends on:** Current M3 governance surfaces staying the approval truth, and one committed implementation plan for the repo-local skill plus `scripts/` runner

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

**Context:** The 2026-04-21 M4 `/autoplan` for the CLI agent onboarding factory reframed this work as a separate post-onboarding lifecycle milestone. The plan anchors the proving run on OpenCode because `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-main-test-outcome-20260420-091704.md` already documents a real stale closeout claim in `.archived/project_management/next/opencode-implementation/governance/seam-2-closeout.md`.

**Effort:** M
**Priority:** P2
**Depends on:** M3 landing cleanly with approval-driven onboarding and validated proving-run closeout
**Completed:** v0.3.0 (2026-04-22)

### Create The OpenCode Execution Pack

**What:** Create a single `.archived/project_management/next/opencode-implementation/` execution pack that turns the closed OpenCode onboarding contracts into implementation-ready seams for `cli_manifests/opencode/`, `crates/opencode/`, and the OpenCode `crates/agent_api` backend.

**Why:** The repo already finished candidate selection and contract locking for OpenCode, but implementation still lacks one code-facing plan-of-record. This execution pack is the missing bridge between the closed onboarding pack and actual landing work.

**Context:** The 2026-04-18 `/plan-eng-review` for `.archived/project_management/next/opencode-cli-onboarding/next-steps-handoff.md` reduced scope to one execution pack, reused the existing `THR-04` and seam closeouts as the bridge, kept UAA promotion out of scope unless stale triggers fire, and required an explicit verification matrix. The pack should consume the published OpenCode contracts and closeouts directly, stay crate-first, and make deterministic replay/fake-binary/fixture validation the default proof path instead of live provider smoke.

**Effort:** M
**Priority:** P2
**Depends on:** Closed OpenCode onboarding pack and normative contracts already landed under `.archived/project_management/next/opencode-cli-onboarding/` and `docs/specs/opencode-*.md`
**Completed:** v0.2.3 (2026-04-18)

### Select The First Real Third CLI Agent And Prepare Its Onboarding Packet

**What:** Choose the first real third CLI agent target after phase 1 lands and create a bounded onboarding packet for adding it to the manifest, backend-crate, and UAA promotion pipeline.

**Why:** Phase 1 intentionally proves future-agent readiness with synthetic fixtures only; this follow-on task turns that architectural readiness into an actual new agent integration when the repo is ready for product expansion.

**Context:** The 2026-04-15 `/plan-eng-review` for CLI manifest support-matrix automation explicitly deferred real third-agent onboarding to keep phase 1 focused on semantics cleanup, neutral parity/support tooling, generated support-matrix publication, and validator hardening. The next step after phase 1 is to pick one concrete CLI agent, document why it is the right target, define its manifest/root conventions, identify any upstream-specific seams that the new neutral modules still do not cover, and produce the implementation packet for snapshot, union, wrapper coverage, validation, backend-crate support, and UAA promotion decisions.

**Effort:** M
**Priority:** P2
**Depends on:** Phase 1 support-matrix and neutral parity tooling landing cleanly
**Completed:** planning docs landed (2026-04-18)
