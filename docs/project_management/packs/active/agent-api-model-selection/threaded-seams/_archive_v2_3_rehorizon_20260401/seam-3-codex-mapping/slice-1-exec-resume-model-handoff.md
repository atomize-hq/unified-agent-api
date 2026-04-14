---
slice_id: S1
seam_id: SEAM-3
slice_kind: delivery
execution_horizon: next
status: decomposed
plan_version: v1
basis:
  currentness: provisional
  basis_ref: seam.md#basis
  stale_triggers: []
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-04
contracts_produced:
  - C-06
contracts_consumed:
  - C-02
  - C-09
open_remediations: []
candidate_subslices: []
---
### S1 - Exec/resume model handoff and argv mapping

- **User/system value**: makes model selection actually work for the Codex exec/resume flows via the existing builder path, with deterministic ordering and without any new raw parsing.
- **Scope (in/out)**:
  - In:
    - consume typed `Option<String>` from SEAM-2
    - thread it into Codex policy/builder mapping
    - prove exactly one `--model <trimmed-id>` emission and correct ordering
  - Out:
    - fork rejection (S2)
    - runtime rejection translation (S3)
- **Acceptance criteria**:
  - `Some(id)` emits exactly one `--model <id>` pair for exec + resume
  - `None` emits no `--model`
  - ordering follows the Codex contract docs (wrapper overrides before `--model`; capability-guarded `--add-dir` after)
  - no raw parse sites exist outside SEAM-2's helper
- **Dependencies**: `THR-02` (typed helper), `C-09`, `C-02`
- **Verification**: targeted argv tests for exec/resume ordering
- **Rollout/safety**: keep mapping localized to Codex surfaces; do not introduce a second `--model` emission site.

#### S1.T1 - Plumb typed model selection into Codex policy/builder calls

- **Outcome**: Codex policy/build path consumes `Option<String>` and calls `.model(trimmed_id)` only when `Some`.
- **Thread/contract refs**: `THR-04`, `C-09`, `C-06`
- **Acceptance criteria**: mapping code never inspects raw `request.extensions`.

Checklist:
- Implement: typed plumbing + builder call site
- Test: exec/resume argv tests
- Validate: `rg` confirms no new parse sites
