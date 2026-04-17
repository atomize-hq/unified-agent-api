---
slice_id: S00
seam_id: SEAM-3
slice_kind: contract_definition
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - support row fields change
    - JSON and Markdown projection ownership shifts
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-02
  - THR-03
contracts_produced:
  - C-04
  - C-05
contracts_consumed:
  - C-01
  - C-02
  - C-03
open_remediations: []
---
### S00 - Row-model and projection contract definition

- **User/system value**: downstream seams get one execution-grade definition of the support row model and Markdown projection boundary before `xtask support-matrix` starts writing artifacts.
- **Scope (in/out)**:
  - In: define row fields, ordering, evidence-note rules, and the one-model/two-renderers boundary for `C-04` and `C-05`.
  - Out: implementation of derivation, artifact writing, contradiction checks, or fixture/golden coverage.
- **Acceptance criteria**:
  - the contract names target rows as the primitive publication unit.
  - the contract lists the row fields that `SEAM-4` and `SEAM-5` will consume.
  - the contract states that Markdown is a projection of the derived JSON model rather than a second truth source.
  - the contract leaves contradiction policy and fixture expansion to later seams.
- **Dependencies**:
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
  - `docs/specs/unified-agent-api/support-matrix.md`
- **Verification**:
  - compare the row-model baseline against the canonical support semantics and the landed root-intake handoff
  - confirm the projection boundary is explicit enough for `SEAM-4` and `SEAM-5` to consume without reopening ownership decisions
- **Rollout/safety**:
  - planning and contract-definition only
  - no support publication output yet

#### S00.T1 - Freeze the support row model

- **Outcome**: one contract states exactly which row fields and evidence notes the derivation seam owns.
- **Acceptance criteria**: a reviewer can name the primitive row fields and explain how target-scoped truth is preserved.

#### S00.T2 - Lock the Markdown projection boundary

- **Outcome**: the seam records exactly what the Markdown projection may do and what must remain owned by the derived row model.
- **Acceptance criteria**: a reviewer can tell which behavior belongs to row derivation and which belongs to Markdown rendering without re-reading pack-level prose.

Checklist:
- Implement: define the row-model and projection contract
- Test: map each claimed field or boundary to a real consumer need
- Validate: confirm `C-04` and `C-05` are concrete enough for downstream planning
