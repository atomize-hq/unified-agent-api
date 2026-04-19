---
slice_id: S1
seam_id: SEAM-4
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers: []
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-05
contracts_produced:
  - C-07
contracts_consumed:
  - C-02
  - C-09
open_remediations: []
candidate_subslices: []
---
### S1 - Claude model handoff and argv mapping

- **User/system value**: makes model selection work for the Claude Code print/session flows via the existing request/argv path, with deterministic ordering and without any new raw parsing.
- **Scope (in/out)**:
  - In:
    - consume typed `Option<String>` from SEAM-2 (`C-09`)
    - thread it into Claude Code request/build mapping
    - prove exactly one `--model <trimmed-id>` emission and correct ordering
    - explicitly exclude `--fallback-model` from this universal key
  - Out:
    - capability advertising / matrix publication (SEAM-2)
- **Acceptance criteria**:
  - `Some(id)` emits exactly one `--model <id>` pair
  - `None` emits no `--model`
  - ordering follows `docs/specs/claude-code-session-mapping-contract.md`
  - no raw parse sites exist outside SEAM-2's helper
- **Dependencies**: `THR-02` (typed helper), `C-09`, `C-02`
- **Verification**: targeted argv tests for print/session ordering + fallback exclusion

#### S1.T1 - Plumb typed model selection into Claude request/argv calls

- **Outcome**: Claude request/build path consumes `Option<String>` and emits `--model <trimmed-id>` only when `Some`.
- **Thread/contract refs**: `THR-05`, `C-09`, `C-07`
- **Acceptance criteria**: mapping code never inspects raw `request.extensions`.

