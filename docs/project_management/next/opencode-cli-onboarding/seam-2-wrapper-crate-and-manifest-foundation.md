---
seam_id: SEAM-2
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
    - governance/seam-1-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - any change to the canonical v1 run surface or deferred-surface policy
    - manifest-root artifact inventory changes in existing repo patterns
    - new evidence that fake-binary, fixture, or offline-parser posture must change
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

# SEAM-2 - Wrapper crate and manifest foundation

- **Goal / value**: define the bounded implementation planning surface for `crates/opencode/` and
  `cli_manifests/opencode/` so backend work later consumes one wrapper-owned truth.
- **Scope**
  - In:
    - define the intended OpenCode wrapper spawn, streaming, completion, parsing, and redaction
      boundaries
    - define offline parser, fixture, fake-binary, and maintainer-smoke posture for wrapper work
    - define the artifact inventory, pointer/update rules, and validation expectations for
      `cli_manifests/opencode/`
    - define the handoff inputs that the backend seam may consume without reopening SEAM-1
  - Out:
    - implementing the backend adapter under `crates/agent_api/`
    - promoting capabilities into the universal facade
    - expanding helper-surface scope beyond the SEAM-1 contract
- **Primary interfaces**
  - Inputs:
    - `THR-01`
    - `C-01`
    - `C-02`
    - `docs/specs/opencode-wrapper-run-contract.md`
    - `docs/specs/opencode-onboarding-evidence-contract.md`
    - existing repo patterns under `crates/codex/`, `crates/claude_code/`, and `cli_manifests/**`
  - Outputs:
    - wrapper-owned event/completion/redaction contract
    - manifest-root artifact inventory and update rules
    - `THR-02` handoff for backend mapping
- **Key invariants / rules**:
  - wrapper-owned semantics stop at the wrapper boundary; backend behavior must consume them rather
    than redefining them
  - fixture-first validation remains the default, with maintainer smoke called out separately when a
    real provider is required
  - manifest evidence must fit the repo's existing truth-store model rather than inventing a second
    mutable ledger
  - the wrapper seam may not claim support for helper surfaces still deferred by `SEAM-1`
- **Dependencies**
  - Direct blockers:
    - `SEAM-1`
  - Transitive blockers:
    - none beyond `SEAM-1`
  - Direct consumers:
    - `SEAM-3`
  - Derived consumers:
    - future wrapper parity and support publication work
- **Touch surface**:
  - `crates/opencode/**`
  - `cli_manifests/opencode/**`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
  - `docs/project_management/next/opencode-cli-onboarding/`
- **Verification**:
  - seam-local review should prove the wrapper-owned boundaries are concrete enough to implement
    without the backend seam inventing new semantics
  - verification should separate automated fixtures, offline parsing, fake-binary possibilities, and
    maintainer-smoke obligations
  - because this seam **produces** owned contracts, verification should focus on those contracts
    becoming concrete enough for planning and implementation rather than requiring final accepted
    publication artifacts to exist already
- **Canonical contract refs**:
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- **Risks / unknowns**:
  - Risk: OpenCode's actual event surface may not line up cleanly with the wrapper shape implied by
    packet-era smoke.
  - De-risk plan: keep typed-event ownership in the wrapper seam and make parser/fixture posture
    explicit before backend planning starts.
  - Risk: manifest-root expectations may drift from existing repo evidence patterns.
  - De-risk plan: model the inventory directly on current `cli_manifests/**` norms and make
    ownership/update rules concrete.
- **Rollout / safety**:
  - this seam landed as a planning/docs contract for later implementation work
  - downstream implementation must preserve redaction and completion-finality expectations
  - helper-surface expansion is a blocker, not an opportunistic stretch goal
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `future` because it has landed and now
    serves as closeout-backed upstream evidence for the backend and promotion seams.
  - Which threads matter most: `THR-01`, `THR-02`
  - What the first seam-local review should focus on: whether the wrapper-owned event/completion
    boundary is explicit, whether manifest inventory/update rules are concrete, and whether the
    fixture/fake-binary posture is realistic
  - Boundary slice intent: reserve `S00` if seam-local planning needs a dedicated contract-definition
    slice for the wrapper or manifest schema before implementation slices begin
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-03`, `C-04`
  - Threads likely to advance: `THR-02`
  - Review-surface areas likely to shift after landing: the repo touch-surface map and the
    contract/dependency flow
  - Downstream seams most likely to require revalidation: `SEAM-3`, `SEAM-4`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in
    pre-exec verification for the producing seam.
