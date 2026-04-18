---
slice_id: S00
seam_id: SEAM-1
slice_kind: contract_definition
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - accepted controls or event-shape drift off the canonical `run --format json` surface
    - manifest-root inventory or validator rules drift from current repo patterns
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-04
  - THR-05
contracts_produced:
  - C-01
  - C-02
contracts_consumed:
  - C-07
open_remediations: []
---
### S00 - workspace-and-manifest-contract-baselines

- **User/system value**: give implementation one concrete repo-local baseline for the OpenCode
  crate, manifest root, validator boundary, and deterministic evidence ownership so code work does
  not spread through the workspace by implication.
- **Scope (in/out)**:
  - In: wrapper crate naming and workspace placement, manifest-root artifact inventory, root
    validator expectations, deterministic evidence ownership, and scope guardrails against generic
    future-agent scaffolding
  - Out: backend mapping under `crates/agent_api/`, support publication follow-through, and any
    helper-surface expansion
- **Acceptance criteria**:
  - the seam-local plan names the intended crate and workspace baseline clearly enough that
    implementation does not invent a second ownership boundary
  - the seam-local plan makes `cli_manifests/opencode/` inventory and validator obligations
    concrete enough to implement mechanically
  - the plan keeps root-specific `xtask` work bounded to OpenCode instead of turning into generic
    scaffolding
- **Dependencies**:
  - `../../../opencode-cli-onboarding/governance/seam-1-closeout.md`
  - `../../../opencode-cli-onboarding/governance/seam-2-closeout.md`
  - `../../../opencode-cli-onboarding/governance/seam-4-closeout.md`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
  - current repo patterns under `crates/codex/`, `crates/claude_code/`, and `cli_manifests/**`
- **Verification**:
  - compare the workspace and manifest decisions against current wrapper and manifest-root patterns
  - confirm the seam-local plan preserves the separation between manifest support, backend support,
    UAA support, and passthrough visibility
  - confirm downstream seams can cite the normative spec files and the landed `THR-05` handoff
    instead of this planning prose
- **Rollout/safety**:
  - fail closed on helper-surface expansion
  - fail closed on generic future-agent abstractions
  - preserve deterministic evidence as the default proof path
- **Review surface refs**:
  - `review.md#r1---foundation-handoff-flow`
  - `review.md#r2---repo-implementation-boundary`

#### S00.T1 - Lock the wrapper crate and workspace baseline

- **Outcome**: the seam-local plan names the intended crate path, package naming posture, workspace
  wiring, and wrapper-owned boundary that implementation must honor.
- **Inputs/outputs**:
  - Inputs: onboarding closeouts, `docs/specs/opencode-wrapper-run-contract.md`, current wrapper
    crate patterns
  - Outputs: concrete execution baseline for `Cargo.toml` and `crates/opencode/**`
- **Thread/contract refs**: `THR-04`, `THR-05`, `C-01`
- **Implementation notes**:
  - follow the existing naming pattern (`unified-agent-api-opencode`, library name `opencode`)
    unless blocking repo evidence appears during execution
  - keep event typing, completion handoff, parser ownership, and redaction in the wrapper seam
- **Acceptance criteria**:
  - implementation can add `crates/opencode/` without backend-owned ambiguity
  - helper surfaces remain explicitly deferred
- **Test notes**:
  - compare the baseline against current `crates/codex/` and `crates/claude_code/` conventions
- **Risk/rollback notes**:
  - if crate ownership or naming stays ambiguous, downstream seams will infer the wrong boundary

#### S00.T2 - Lock the manifest root and validator baseline

- **Outcome**: the seam-local plan names the required `cli_manifests/opencode/` inventory and the
  OpenCode-specific root-validator behavior needed to land that root safely.
- **Inputs/outputs**:
  - Inputs: `docs/specs/opencode-cli-manifest-contract.md`, current manifest roots, current
    `crates/xtask` validator posture
  - Outputs: concrete execution baseline for `cli_manifests/opencode/**` and bounded root-specific
    validator work
- **Thread/contract refs**: `THR-05`, `C-02`
- **Implementation notes**:
  - keep root validation OpenCode-specific and deterministic
  - keep manifest evidence distinct from backend or unified support claims
- **Acceptance criteria**:
  - implementation knows which committed artifacts are required to make the root valid
  - validator work remains bounded to OpenCode rather than future-agent template work
- **Test notes**:
  - compare the baseline against current `cli_manifests/codex/**` and
    `cli_manifests/claude_code/**` inventories and validator expectations
- **Risk/rollback notes**:
  - if the root inventory stays implicit, support publication and backend planning will consume
    inconsistent evidence

Checklist:
- Implement: lock the workspace, wrapper, manifest-root, and validator baselines for OpenCode
- Test: compare the plan against current repo wrapper and manifest-root norms
- Validate: confirm `C-01` and `C-02` are concrete enough for implementation to start
