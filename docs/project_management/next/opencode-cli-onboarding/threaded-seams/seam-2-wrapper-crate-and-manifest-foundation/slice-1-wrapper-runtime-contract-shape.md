---
slice_id: S1
seam_id: SEAM-2
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - helper surfaces become necessary to obtain wrapper-owned semantics
    - event/completion ownership drifts out of the wrapper seam
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
contracts_produced:
  - C-03
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
---
### S1 - wrapper-runtime-contract-shape

- **User/system value**: keep wrapper implementation bounded by one explicit spawn/stream/parser/
  completion contract so backend work later stays consumer-shaped.
- **Scope (in/out)**:
  - In: wrapper spawn surface, typed event ownership, completion-finality handoff, offline parser,
    redaction boundary
  - Out: backend mapping, capability promotion, and support publication
- **Acceptance criteria**:
  - the wrapper seam owns typed event and completion semantics for the canonical run path
  - offline parser and deterministic replay expectations stay explicit
  - helper surfaces remain deferred and backend-specific
- **Dependencies**:
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `governance/seam-1-closeout.md`
  - existing wrapper patterns under `crates/codex/` and `crates/claude_code/`
- **Verification**:
  - confirm the wrapper contract now answers what `crates/opencode/` must implement without asking
    `SEAM-3` to guess
  - confirm raw-line leakage and provider-specific diagnostics remain out of the wrapper boundary
- **Rollout/safety**:
  - keep the wrapper seam headless and automation-safe by default
  - preserve fixture-first validation and deterministic replay posture
- **Review surface refs**:
  - `review.md#r2---wrapper-owned-boundary`

#### S1.T1 - Lock wrapper-owned event and completion semantics

- **Outcome**: the contract explicitly names the wrapper-owned typed event, completion-finality,
  parser, and redaction obligations for the canonical run surface.
- **Inputs/outputs**:
  - Inputs: `SEAM-1` closeout, existing wrapper-crate patterns
  - Outputs: wrapper-owned runtime sections in `docs/specs/opencode-wrapper-run-contract.md`
- **Thread/contract refs**: `THR-02`, `C-03`
- **Implementation notes**:
  - keep the wrapper handoff consumer-shaped for `SEAM-3`
  - avoid backend-specific payload decisions here
- **Acceptance criteria**:
  - `SEAM-3` can cite the wrapper contract alone for event/completion ownership
  - completion finality remains explicit and testable
- **Test notes**:
  - compare the resulting contract sections against current wrapper test and redaction norms
- **Risk/rollback notes**:
  - if wrapper-owned semantics stay ambiguous, backend code will hard-code the wrong boundary

Checklist:
- Implement: tighten the wrapper-owned runtime contract for spawn, typed events, completion, parser,
  and redaction
- Test: compare the contract against landed `SEAM-1` inputs and existing wrapper-crate norms
- Validate: confirm backend mapping work can consume this boundary without reopening `SEAM-1`
