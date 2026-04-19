---
slice_id: S00
seam_id: SEAM-2
slice_kind: contract_definition
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - wrapper-owned event or completion semantics drift
    - manifest-root inventory or pointer rules drift
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
contracts_produced:
  - C-03
  - C-04
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
---
### S00 - wrapper-and-manifest-contract-baselines

- **User/system value**: give downstream seams repo-owned wrapper and manifest baselines so
  implementation does not depend on packet-era prose or implicit repo conventions.
- **Scope (in/out)**:
  - In: define the wrapper-owned runtime boundary additions and the manifest-root contract baseline
    under `docs/specs/**`
  - Out: backend mapping implementation, capability promotion, and support publication
- **Acceptance criteria**:
  - `docs/specs/opencode-wrapper-run-contract.md` is concrete enough about wrapper-owned event,
    completion, parsing, and redaction ownership that `SEAM-3` cannot invent new wrapper semantics
  - `docs/specs/opencode-cli-manifest-contract.md` defines root artifacts, pointer/update rules,
    and evidence expectations for `cli_manifests/opencode/`
  - neither contract silently widens helper-surface scope beyond the landed `SEAM-1` boundary
- **Dependencies**:
  - `governance/seam-1-closeout.md`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - current repo patterns under `crates/codex/`, `crates/claude_code/`, and `cli_manifests/**`
- **Verification**:
  - compare each contract decision against the landed `SEAM-1` closeout and current repo patterns
  - confirm the baselines live under `docs/specs/**`, which is this repo's normative contract
    surface
  - confirm later seams can cite the contract files directly without packet prose
- **Rollout/safety**:
  - fail closed on helper-surface expansion
  - keep backend mapping and support publication downstream of these baselines
- **Review surface refs**:
  - `review.md#r1---wrapper-and-manifest-handoff`
  - `review.md#r2---wrapper-owned-boundary`

#### S00.T1 - Define the wrapper-owned runtime detail baseline

- **Outcome**: the wrapper contract explicitly names wrapper-owned spawn, typed-stream,
  completion-finality, parser, and redaction boundaries for the canonical run surface.
- **Inputs/outputs**:
  - Inputs: landed `SEAM-1` runtime/evidence contracts, current wrapper-crate patterns
  - Outputs: updated `docs/specs/opencode-wrapper-run-contract.md`
- **Thread/contract refs**: `THR-01`, `THR-02`, `C-03`
- **Implementation notes**:
  - keep helper surfaces deferred
  - keep raw backend lines and provider-specific diagnostics out of the wrapper API by default
- **Acceptance criteria**:
  - later seams can cite one wrapper-owned contract for runtime detail
  - completion and parser ownership remain at the wrapper seam, not the backend seam
- **Test notes**:
  - compare the contract against the landed `SEAM-1` boundary and current wrapper-crate norms
- **Risk/rollback notes**:
  - if wrapper ownership remains ambiguous, `SEAM-3` will invent semantics that belong upstream

#### S00.T2 - Define the manifest-root contract baseline

- **Outcome**: one manifest contract names the artifact inventory, pointer/update rules, and
  validation/evidence expectations for `cli_manifests/opencode/`.
- **Inputs/outputs**:
  - Inputs: current `cli_manifests/**` patterns, `SEAM-1` evidence contract
  - Outputs: `docs/specs/opencode-cli-manifest-contract.md`
- **Thread/contract refs**: `THR-02`, `C-04`
- **Implementation notes**:
  - keep the manifest contract aligned with existing root-local evidence patterns
  - separate runtime/protocol evidence from generic help-surface coverage
- **Acceptance criteria**:
  - later seams can tell which artifacts are required for wrapper support publication
  - pointer and version metadata rules are explicit enough to validate mechanically
- **Test notes**:
  - compare the inventory against current `cli_manifests/codex/**` and
    `cli_manifests/claude_code/**` roots
- **Risk/rollback notes**:
  - implicit manifest rules will force downstream seams to reopen the contract boundary

Checklist:
- Implement: define the wrapper-owned and manifest-root contract baselines under `docs/specs/**`
- Test: cross-check contract decisions against the landed `SEAM-1` closeout and current repo norms
- Validate: confirm `C-03` and `C-04` are concrete enough for downstream implementation planning
