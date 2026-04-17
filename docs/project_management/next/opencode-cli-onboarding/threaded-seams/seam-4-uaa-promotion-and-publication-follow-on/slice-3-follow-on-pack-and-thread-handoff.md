---
slice_id: S3
seam_id: SEAM-4
slice_kind: documentation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - capability-matrix or universal extension-registry rule changes
    - backend mapping changes that materially alter what OpenCode exposes
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-04
contracts_produced:
  - C-07
contracts_consumed: []
open_remediations: []
---
### S3 - follow-on-pack-and-thread-handoff

- **User/system value**: make the promotion seam exit with one explicit handoff for pack closeout
  and any future canonical-spec or capability-matrix work.
- **Scope (in/out)**:
  - In: follow-on pack requirement language, thread handoff framing, closeout-ready publication
    notes
  - Out: executing the follow-on pack itself or editing canonical specs in this seam
- **Acceptance criteria**:
  - the seam can publish an explicit follow-on-pack answer
  - `THR-04` handoff is concrete enough for pack closeout and future work
  - no future work is implied silently
- **Dependencies**:
  - `docs/project_management/next/opencode-cli-onboarding/seam-4-uaa-promotion-and-publication-follow-on.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- **Verification**:
  - confirm the handoff language tells downstream work whether follow-on execution is required
  - confirm no canonical spec or matrix edits are hidden inside this seam
- **Rollout/safety**:
  - keep future work explicit and bounded
  - avoid turning pack closeout into a cleanup bucket for promotion ambiguity
- **Review surface refs**:
  - `review.md#planned-seam-exit-gate-focus`

#### S3.T1 - Lock the follow-on answer and THR-04 handoff

- **Outcome**: the seam names one explicit follow-on-pack answer and one downstream-ready thread
  handoff for pack closeout and later work.
- **Inputs/outputs**:
  - Inputs: promotion recommendation posture, future-work triggers
  - Outputs: closeout-ready handoff framing for `THR-04`
- **Thread/contract refs**: `THR-04`, `C-07`
- **Implementation notes**:
  - keep the follow-on answer explicit even when it is “no follow-on required”
  - make future canonical spec or matrix work a separate execution decision, not an implied task
- **Acceptance criteria**:
  - downstream closeout can record one explicit handoff answer
  - future work can tell whether it needs a new execution pack
- **Test notes**:
  - compare the handoff against the promotion recommendation and follow-on triggers
- **Risk/rollback notes**:
  - if the handoff stays vague, future reviewers will reopen this seam just to infer next steps

Checklist:
- Implement: define the follow-on-pack answer and `THR-04` handoff
- Test: compare the handoff against the promotion recommendation
- Validate: confirm pack closeout can consume one explicit downstream answer
