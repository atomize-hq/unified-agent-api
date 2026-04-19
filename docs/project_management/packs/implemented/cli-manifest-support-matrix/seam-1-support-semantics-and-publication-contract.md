---
seam_id: SEAM-1
seam_slug: support-semantics-and-publication-contract
type: integration
status: closed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - support layer vocabulary changes
    - canonical publication location changes
    - neutral xtask entrypoint naming changes
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

# SEAM-1 - Support semantics and publication contract

- **Goal / value**: pin one deterministic meaning for published support truth before the repo starts generating new artifacts from current manifest evidence.
- **Current planning posture**: closed. The canonical support-matrix spec, manifest terminology alignment, and neutral `xtask support-matrix` entrypoint all landed and are recorded in `governance/seam-1-closeout.md`.
- **Scope**
  - In:
    - define the separation between manifest support, backend support, UAA unified support, and passthrough visibility
    - establish `cli_manifests/support_matrix/current.json` and `docs/specs/unified-agent-api/support-matrix.md` as the phase-1 publication surfaces
    - remove stale "planned" or incorrect support-language from manifest docs and validator docs
    - wire the neutral `xtask support-matrix` command entrypoint in `crates/xtask/src/main.rs`
  - Out:
    - extracting shared normalization logic
    - implementing row derivation or renderers
    - validator contradiction checks and fixture suites
- **Primary interfaces**
  - Inputs:
    - `docs/project_management/next/cli-manifest-support-matrix/plan.md`
    - current manifest docs under `cli_manifests/codex/**` and `cli_manifests/claude_code/**`
    - canonical UAA docs under `docs/specs/unified-agent-api/**`
  - Outputs:
    - locked support semantics in `docs/specs/unified-agent-api/support-matrix.md`
    - updated UAA spec linkage in `docs/specs/unified-agent-api/README.md`
    - renamed/neutral command contract in `crates/xtask/src/main.rs`
- **Key invariants / rules**:
  - target-scoped rows are the primitive; per-version summaries are projections
  - `versions/<version>.json.status` is workflow metadata, not published support truth
  - capability matrix stays separate from support matrix
  - backend-specific passthrough remains visible but must not count as UAA unified support
- **Dependencies**
  - Direct blockers:
    - none
  - Transitive blockers:
    - none
  - Direct consumers:
    - `SEAM-2`, `SEAM-3`, `SEAM-4`, `SEAM-5`
  - Derived consumers:
    - future agent-manifest onboarding work
- **Touch surface**:
  - `docs/specs/unified-agent-api/README.md`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `cli_manifests/codex/README.md`
  - `cli_manifests/claude_code/README.md`
  - `cli_manifests/codex/VALIDATOR_SPEC.md`
  - `cli_manifests/claude_code/VALIDATOR_SPEC.md`
  - `cli_manifests/codex/CI_AGENT_RUNBOOK.md`
  - `cli_manifests/claude_code/CI_AGENT_RUNBOOK.md`
  - `cli_manifests/codex/RULES.json`
  - `cli_manifests/claude_code/RULES.json`
  - `crates/xtask/src/main.rs`
- **Verification**:
  - spec and manifest docs agree on support-layer vocabulary and authority
  - `xtask` exposes `support-matrix` without altering existing capability-matrix behavior
  - downstream seams can cite one canonical semantics source rather than plan prose
  - if this seam **produces** an owned contract, verification is the contract becoming concrete enough for seam-local planning and implementation; it does not require final generated artifacts to exist yet
- **Canonical contract refs**:
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/README.md`
- **Risks / unknowns**:
  - Risk: freezing today’s contradictory terms would turn a docs problem into a generator contract problem.
  - De-risk plan: land the semantics doc and naming cleanup before any shared row-model implementation starts.
- **Rollout / safety**:
  - additive at phase 1
  - no runtime `agent_api` behavior change
  - existing capability-matrix workflows remain intact
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `future` because it has left the forward planning window after publishing `THR-01` and closing its seam-exit record.
  - Which threads matter most: `THR-01`
  - What the first seam-local review focused on: whether the semantics doc fully disambiguates validated vs supported vs unified support, whether any manifest docs still contradict that pinned meaning, and whether `xtask` command naming still leaks capability-matrix assumptions into the support workflow
  - Boundary slice intent: reserve `S00` for contract-definition cleanup if seam-local planning finds unresolved semantics drift between the UAA support doc, manifest docs, and xtask command naming
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-01`
  - Threads likely to advance: `THR-01`
  - Review-surface areas likely to shift after landing: support publication workflow and touch-surface map
  - Downstream seams most likely to require revalidation: `SEAM-2`, `SEAM-3`, `SEAM-4`, `SEAM-5`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
