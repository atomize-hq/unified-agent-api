# TODOS

## Pending

### Make Goose The Explicit P1 End-To-End Lifecycle Validation

**What:** Take `docs/agents/lifecycle/goose/governance/approved-agent.toml` through the full repo-owned create lane: `scaffold-wrapper-crate`, `runtime-follow-on`, `prepare-publication`, `refresh-publication`, `prepare-proving-run-closeout`, and `close-proving-run`.

**Why:** The remaining open design gap is agent maintenance CI enrollment and release-watch orchestration for onboarded CLI agents. Once that lands, `goose` should be the first explicit end-to-end proving target so the repo validates the whole lifecycle on a real approved agent instead of treating the approval artifact as the finish line.

**Context:** Recommendation research already promoted `goose` and recorded the final approval handoff at `docs/agents/lifecycle/goose/governance/approved-agent.toml`. The publication and closeout machinery is now landed. The follow-up after landing agent-maintenance CI work is to run one honest full-lifecycle proving pass for `goose`.

**Effort:** M
**Priority:** P1
**Depends on:** Deciding and landing the agent-maintenance CI architecture, either per-agent maintenance automation enrollment or one shared watcher/orchestrator driven from registry truth

### Decide Whether Capability Matrix Markdown Stays Canonical After M5

**What:** After M5 lands, decide whether `docs/specs/unified-agent-api/capability-matrix.md` remains the canonical published truth surface or becomes a rendered view over a more structured canonical artifact.

**Why:** The 2026-04-23 `/autoplan` for M5 found that local and CI verification currently reason about capability publication differently. M5 should unify the gate first. The next question is whether markdown itself should stay the canonical control-plane truth or only the human-readable publication surface.

**Context:** `make preflight` currently omits capability publication freshness while CI runs `cargo run -p xtask -- capability-matrix`, a `git diff`, and `cargo run -p xtask -- capability-matrix-audit`. That is good enough to fix in M5, but the longer-term truth-surface decision should be made explicitly before more agents or more publication consumers pile on.

**Effort:** S
**Priority:** P2
**Depends on:** M5 landing with one canonical capability projection contract and one shared local/CI check-only gate

## Retired / Superseded

### Enclose The Publication Refresh Follow-On After The Runtime Runner

**What:** Add the next bounded automation seam after `uaa-0022`: deterministic refresh of runtime-derived manifest evidence into publication-owned support and capability surfaces, validation gates, and proving-run-closeout readiness.

**Why:** The runtime runner milestone deliberately stops before publication refresh so the runtime seam stays boilable. But the real create-lane done-state still requires support/capability refresh, green validation, and closeout. If this follow-on is not captured explicitly, the runtime runner risks becoming a well-documented island instead of a step toward a full green lane.

**Context:** This item is no longer a live pending milestone. Its scope was absorbed by the landed 2026-05-02 through 2026-05-04 sequence: `refresh-publication --write` now owns publication refresh, `published` is the honest committed post-refresh lifecycle state, and `prepare-proving-run-closeout` plus `close-proving-run` now finish the green publication to `closed_baseline` handoff without ad hoc artifact authoring.

**Effort:** M
**Priority:** P2
**Depends on:** `uaa-0022` landing with a structured runtime summary and explicit handoff into publication refresh
**Retired:** superseded by landed publication + closeout work on 2026-05-04

## Completed

### Land The LLM-Guided Research Layer For The Recommendation Lane

**What:** Replace the thin repo-local `recommend-next-agent` skill with a real AI research workflow that gathers explicit proof for candidate charter fit, then feed that structured research into the existing deterministic runner for validation, rendering, promotion, and approval-artifact drafting.

**Why:** The shipped recommendation lane now works mechanically, but it still ranks candidates from heuristic signals in `scripts/recommend_next_agent.py` instead of using the skill as the actual research layer. That misses the intended product. Maintainers need a recommendation packet they can trust because an AI agent did the research and the runner enforced the contract.

**Context:** This landed on 2026-05-04. `cargo run -p xtask -- recommend-next-agent-research --dry-run|--write` now owns bounded discovery prompt rendering, Codex execution, `freeze-discovery`, research validation, and execution evidence; the skill and operator guide are thin wrappers over that repo-owned flow; proving completed successfully for `20260504T212721Z-cli-recommendation-v3-pass1`; promotion produced `docs/agents/selection/runs/20260504T212721Z-cli-recommendation-v3-pass1`; and the final approval handoff was recorded at `docs/agents/lifecycle/goose/governance/approved-agent.toml`.

**Effort:** M
**Priority:** P1
**Depends on:** The current deterministic runner, packet template, approval-artifact contract, and operator guide remaining the control-plane truth
**Completed:** landed on 2026-05-04

### Enclose Create-Mode Closeout Without Ad Hoc Authoring

**What:** Add a repo-owned closeout preparation flow that materializes or scaffolds `proving-run-closeout.json` from machine-known lifecycle facts plus the minimal remaining human inputs, so the create lane can advance from green publication to `closed_baseline` without freehand artifact authoring.

**Why:** `close-proving-run` is already a strong validator and packet refresher, but it still depended on a separately authored closeout JSON artifact. That meant the lifecycle machine was still not fully enclosed even after publication was green.

**Context:** This landed on 2026-05-04. `refresh-publication --write` now hands off to `prepare-proving-run-closeout --approval <path> --write`, the canonical `proving-run-closeout.json` supports truthful `prepared` and `closed` states, prepared packet docs render as `closeout_prepared`, `close-proving-run` rewrites machine-owned fields from current lifecycle/publication truth and rejects unresolved placeholders, and historical backfill now uses the shared create-mode closeout builder/serializer instead of bespoke inline JSON.

**Effort:** S
**Priority:** P1
**Depends on:** A truthful enclosed publication lane and an explicit decision on the canonical post-publication lifecycle state
**Completed:** landed on 2026-05-04

### Make The Published State Honest In The Lifecycle Model

**What:** Resolve the mismatch between the lifecycle schema and the live lane by either making `published` a real committed transition or removing/replacing it so the state machine matches the actual path from runtime integration to closeout.

**Why:** The repo currently talks about green published state, but the live create lane does not appear to write `LifecycleStage::Published`. It transitions from `runtime_integrated` to `publication_ready` and then to `closed_baseline`, which means “published” is more of a validated condition than a committed lifecycle state.

**Context:** This landed on 2026-05-03 with `refresh-publication --write` as the sole committed writer of `LifecycleStage::Published`, the canonical create-mode path corrected to `publication_ready -> published -> closed_baseline`, maintenance and closeout semantics aligned to treat `published` as the normal post-refresh state, and docs plus regression coverage updated to reflect one honest lifecycle story.

**Effort:** S
**Priority:** P1
**Depends on:** The publication lane being enclosed enough to define exactly when publication is complete and what lifecycle evidence that completion owns
**Completed:** landed on 2026-05-03

### Enclose The Publication Lane End To End

**What:** Add one repo-owned publication command that consumes `publication-ready.json`, writes the required publication-owned support/capability outputs, runs the green publication checks, and fails transactionally if any required surface cannot be made green.

**Why:** `prepare-publication` currently records the handoff into `publication_ready`, but it does not actually write the published support/capability surfaces. That leaves the operator packet with a check-only next step while the real write commands still live outside the committed handoff contract.

**Context:** This landed on 2026-05-02 with `refresh-publication --approval <path> --check|--write` as the sole publication consumer, shared publication planning between create-mode and maintenance-mode, transactional publication-owned output writes with rollback on gate failure, updated lifecycle/operator-guide next-command semantics, and regression coverage for stale detection, rollback, surface selection, and idempotent rerun.

**Effort:** M
**Priority:** P1
**Depends on:** The capability publication foundation above so publication refresh can reason about any enrolled agent without hidden backend-specific code edits
**Completed:** landed on 2026-05-02

### Land The Generic Capability Publication Foundation

**What:** Replace the hardcoded built-in backend inventory in capability publication with one registry- and lifecycle-driven model that all publication consumers share: `capability-matrix`, `capability-matrix-audit`, `check-agent-drift`, and `close-proving-run`.

**Why:** A newly enrolled agent still cannot flow generically through capability publication today. The current generator knows only the backends compiled into `crates/xtask/src/capability_matrix.rs`, so adding an agent to `agent_registry.toml` is not enough. Hidden Rust edits are still required before publication can become truthful.

**Context:** This landed on 2026-05-02 with a shared `crates/xtask/src/capability_publication.rs` source, generator/audit/closeout/drift/prepare-publication convergence, lifecycle-backed publication wording in the capability matrix spec, and a replayed final verification chain that uses a stage-appropriate `prepare-publication --check` target.

**Effort:** M
**Priority:** P1
**Depends on:** The current lifecycle record, approval artifact capability declarations, and manifest-root capability projection contract staying authoritative inputs during the transition
**Completed:** landed on 2026-05-02

### Enclose The Runtime Follow-On In A Codex Exec Runner

**What:** Add a bounded runtime-lane execution path for create-mode onboarding that starts from the control-plane packet and scaffolding output, then drives the operator-guide "Finish the runtime follow-on" work through Codex with explicit file targets, baseline expectations, and required evidence outputs.

**Why:** The recommendation lane now closes the pre-create gap, and the control-plane commands already own enrollment, scaffolding, packet closeout, and now the runtime execution seam itself. The next remaining create-lane gap is not runtime implementation anymore; it is deterministic publication refresh and maintenance wiring after runtime-owned evidence exists.

**Context:** The runtime runner landed on 2026-04-29 and now owns `runtime-follow-on --approval <path> --dry-run/--write`, bounded runtime evidence, and handoff validation ahead of publication refresh. `opencode` remains the default baseline template. The deferred publication/closeout follow-on later landed across the 2026-05-02 through 2026-05-04 publication, lifecycle, and closeout milestones above.

**Effort:** M
**Priority:** P1
**Depends on:** The shipped M6 control-plane boundary, the backend-harness path in `agent_api`, and a backlog-only milestone spec that pinned the runtime-lane tiering and runner contract
**Completed:** landed on 2026-04-29

### Land The Deterministic Recommendation Engine v1

**What:** Add the repo-local `recommend-next-agent` skill, the deterministic `scripts/recommend_next_agent.py` runner, the candidate seed file, the canonical packet promotion flow, and the approval-artifact draft handoff into `xtask onboard-agent`.

**Why:** This closed the mechanical pre-create gap. Maintainers can now produce a promoted run, a canonical selection packet, and a valid `approved-agent.toml` handoff instead of authoring those artifacts by hand.

**Context:** The 2026-04-28 validation on `codex/recommend-next-agent` found that this milestone landed correctly for promotion mechanics, byte-identity guarantees, and approval-artifact validation, but it also showed the next gap: the skill is still too thin and the runner still relies on heuristic proof. That follow-on is now the active milestone above.

**Effort:** M
**Priority:** P1
**Depends on:** Current M3 governance surfaces staying the approval truth, and one committed implementation plan for the repo-local skill plus `scripts/` runner
**Completed:** v0.4.0 (2026-04-28)

### Implement The M4 Post-Onboarding Maintenance Lane

**What:** Add a separate maintenance lifecycle for already-onboarded agents: agent-scoped drift detection, a dedicated maintenance packet/request, bounded control-plane refresh ergonomics, and explicit maintenance closeout.

**Why:** `onboard-agent` is the create-mode bridge for new agents. Once an agent is already in the repo, maintainers still need a boring way to detect and repair drift across registry truth, publication outputs, release docs, and closed packet/governance docs without reopening new-agent onboarding.

**Context:** The 2026-04-21 M4 `/autoplan` for the CLI agent onboarding factory reframed this work as a separate post-onboarding lifecycle milestone. The plan anchors the proving run on OpenCode because a prior external test outcome artifact from 2026-04-20 already documented a real stale closeout claim in `.archived/project_management/next/opencode-implementation/governance/seam-2-closeout.md`.

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
