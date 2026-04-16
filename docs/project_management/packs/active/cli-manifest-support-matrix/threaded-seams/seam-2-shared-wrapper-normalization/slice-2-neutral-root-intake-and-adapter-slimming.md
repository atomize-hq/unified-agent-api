---
slice_id: S2
seam_id: SEAM-2
slice_kind: adoption
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - versions, pointer, current, or report paths change
    - adapters keep root-intake semantics hidden outside the shared seam
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
contracts_produced: []
contracts_consumed:
  - C-02
  - C-03
open_remediations: []
---
### S2 - Neutral root intake and adapter slimming

- **User/system value**: downstream publication work can consume one root-shaped intake contract instead of re-discovering how Codex and Claude roots expose versions, pointers, current metadata, and coverage reports.
- **Scope (in/out)**:
  - In: define and adopt the neutral root-intake interface, and shrink adapter ownership down to root-specific defaults and loading glue.
  - Out: row derivation, publication rendering, contradiction checks, or future-agent fixture expansion.
- **Acceptance criteria**:
  - the shared seam exposes one neutral intake shape for versions, pointers, current metadata, and coverage reports.
  - Codex and Claude adapters no longer own duplicated intake semantics beyond root-specific defaults and wiring.
  - the future `support-matrix` implementation has a clear consumer boundary without reopening `SEAM-1`.
- **Dependencies**:
  - `S00` contract-definition output
  - `S1` shared normalization extraction
  - current manifest roots under `cli_manifests/codex/**` and `cli_manifests/claude_code/**`
- **Verification**:
  - inspect the shared intake surface against the actual root layouts
  - confirm the adapters remain thin and descriptive
  - confirm no publication or validator work was pulled into this seam
- **Rollout/safety**:
  - additive seam-boundary adoption only
  - no support publication output yet
- **Review surface refs**:
  - `review.md#likely-mismatch-hotspots`
  - `../../threading.md`

#### S2.T1 - Define the neutral root-intake shape

- **Outcome**: downstream consumers can request versions, pointers, current metadata, and coverage reports from one neutral shared interface.
- **Inputs/outputs**:
  - Inputs: current manifest-root layout and shared contract decisions
  - Outputs: neutral intake types or helpers inside the shared seam
- **Thread/contract refs**: `THR-01`, `THR-02`, `C-03`
- **Implementation notes**: keep the intake contract root-shaped and future-agent-ready; do not invent a generic framework or renderer API.
- **Acceptance criteria**: a reviewer can map each intake field back to an existing root surface for both current agents.
- **Test notes**: inspect or unit-test the intake helpers against both current root layouts.
- **Risk/rollback notes**: an intake shape that depends on current agent names will block future-agent onboarding.

#### S2.T2 - Reduce adapters to root-specific loading glue

- **Outcome**: Codex and Claude modules keep only the minimum root-specific behavior after shared intake adoption.
- **Inputs/outputs**:
  - Inputs: current adapters plus the new shared intake boundary
  - Outputs: thinner adapter modules with explicit ownership
- **Thread/contract refs**: `THR-02`, `C-02`, `C-03`
- **Implementation notes**: preserve current root-specific defaults and path selection while moving reusable logic into the shared seam.
- **Acceptance criteria**: adapter modules no longer duplicate intake semantics and remain easy to review for root-specific behavior only.
- **Test notes**: compare adapter responsibilities before and after the change.
- **Risk/rollback notes**: leaving too much logic in the adapters weakens the whole seam and keeps downstream code coupled to current agents.

Checklist:
- Implement: adopt the neutral root-intake interface and slim the adapters
- Test: verify the shared intake shape still matches both current roots
- Validate: confirm the future `support-matrix` seam has a concrete consumer boundary
