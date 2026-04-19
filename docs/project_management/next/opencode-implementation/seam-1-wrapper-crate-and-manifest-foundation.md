---
seam_id: SEAM-1
seam_slug: wrapper-crate-and-manifest-foundation
type: capability
status: closed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - ../opencode-cli-onboarding/governance/seam-1-closeout.md
    - ../opencode-cli-onboarding/governance/seam-2-closeout.md
    - ../opencode-cli-onboarding/governance/seam-4-closeout.md
  required_threads:
    - THR-04
    - THR-05
  stale_triggers:
    - OpenCode CLI event-shape drift on the canonical `run --format json` surface
    - any accepted v1 control (`--model`, `--session` / `--continue`, `--fork`, `--dir`) moves off the canonical surface
    - provider-auth or model-routing posture changes invalidate the baseline prerequisite record
    - manifest inventory, pointer rules, or deterministic replay posture drift from the closed onboarding basis
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

# SEAM-1 - Wrapper crate and manifest foundation

- **Goal / value**: land the concrete OpenCode implementation foundation so downstream work can
  consume one wrapper-owned and manifest-owned truth instead of inferring behavior from the closed
  onboarding pack.
- **Scope**
  - In:
    - create the initial `crates/opencode/` wrapper crate and workspace wiring needed to host it
    - create the initial `cli_manifests/opencode/` root with committed inventory, schema/rules,
      pointers, reports, version metadata, and current snapshot posture
    - keep parsing, event typing, completion handoff, offline parser, and redaction inside the
      wrapper seam
    - define deterministic fake-binary, fixture, transcript, and offline-parser proof paths for
      the wrapper and manifest root
    - add only the root-specific `xtask` or validation plumbing required to land OpenCode itself
  - Out:
    - implementing the OpenCode backend under `crates/agent_api/`
    - generic scaffolding for future agents
    - UAA promotion or universal capability expansion
    - widening scope to `serve`, `acp`, `run --attach`, or direct interactive TUI behavior
- **Primary interfaces**
  - Inputs:
    - `THR-04`
    - `docs/specs/opencode-wrapper-run-contract.md`
    - `docs/specs/opencode-onboarding-evidence-contract.md`
    - `docs/specs/opencode-cli-manifest-contract.md`
    - existing repo patterns under `crates/codex/`, `crates/claude_code/`, and `cli_manifests/**`
  - Outputs:
    - `crates/opencode/**`
    - `cli_manifests/opencode/**`
    - OpenCode-specific root validation flow under `crates/xtask/**` as needed for this root only
    - `THR-05`
- **Key invariants / rules**:
  - `opencode run --format json` remains the only canonical runtime transport
  - accepted controls remain limited to `--model`, `--session` / `--continue`, `--fork`, and
    `--dir`
  - helper surfaces remain deferred and fail closed in v1
  - manifest support, backend support, UAA unified support, and passthrough visibility remain
    separate publication layers
  - deterministic replay, fake-binary, fixture, and offline-parser evidence are the default proof
    path
- **Dependencies**
  - Direct blockers:
    - none beyond the current closed onboarding basis carried by `THR-04`
  - Transitive blockers:
    - none
  - Direct consumers:
    - `SEAM-2`, `SEAM-3`
  - Derived consumers:
    - future OpenCode release review and regression work
- **Touch surface**:
  - `Cargo.toml`
  - `crates/opencode/**`
  - `cli_manifests/opencode/**`
  - `crates/xtask/**`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
- **Verification**:
  - targeted wrapper tests must prove deterministic handling of `--model`, `--session` /
    `--continue`, `--fork`, and `--dir`
  - transcript fixtures or fake-binary inputs must prove parser, event typing, completion handoff,
    and redaction without requiring a live provider account by default
  - manifest-root verification must pass `cargo run -p xtask -- codex-validate --root cli_manifests/opencode`
  - if this seam **produces** owned contracts, verification is about making wrapper and manifest
    behavior concrete enough for implementation and publication follow-through rather than requiring
    live support promotion already to exist
- **Canonical contract refs**:
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
  - `docs/specs/unified-agent-api/support-matrix.md`
- **Risks / unknowns**:
  - Risk: the real JSON event surface may force wrapper-owned parsing decisions that are not fully
    implied by planning smoke.
  - De-risk plan: make transcript and fake-binary coverage first-class before backend planning.
  - Risk: the first OpenCode root may overfit Codex or Claude root layouts instead of reusing the
    shared truth-store model cleanly.
  - De-risk plan: keep the manifest contract authoritative and make root-validator expectations
    explicit from the start.
  - Risk: root-specific `xtask` work may drift into generic scaffolding.
  - De-risk plan: keep changes bounded to OpenCode root needs and avoid introducing future-agent
    template abstractions in this seam.
- **Rollout / safety**:
  - keep all helper surfaces explicitly deferred
  - treat live provider-backed smoke as stale-trigger evidence only
  - fail closed when runtime evidence contradicts the closed onboarding basis
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `active` because the repo currently has
    no OpenCode wrapper crate or manifest root, and every downstream seam depends on these landed
    artifacts existing first.
  - Which threads matter most: `THR-04`, `THR-05`
  - What the first seam-local review should focus on: wrapper ownership boundaries, manifest-root
    inventory and validator shape, and deterministic evidence posture
  - Boundary slice intent: reserve `S00` because seam-local planning is likely to need a dedicated
    contract-definition boundary before implementation slices begin
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-01`, `C-02`
  - Threads likely to advance: `THR-05`
  - Review-surface areas likely to shift after landing: the repo touch-surface map and the
    deterministic-versus-live evidence boundary
  - Downstream seams most likely to require revalidation: `SEAM-2`, `SEAM-3`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in
    pre-exec verification for the producing seam.
