---
slice_id: S2
seam_id: SEAM-2
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - manifest-root artifact inventory changes in existing repo patterns
    - protocol evidence no longer aligns with the canonical run surface
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-02
contracts_produced:
  - C-04
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
---
### S2 - manifest-inventory-and-evidence-layout

- **User/system value**: give wrapper and support publication work one explicit
  `cli_manifests/opencode/` inventory so evidence stays auditable and mechanical.
- **Scope (in/out)**:
  - In: root artifact inventory, pointer/update rules, current/version metadata posture, protocol
    evidence layout, validation expectations
  - Out: backend mapping, UAA promotion, and implementation code
- **Acceptance criteria**:
  - the manifest contract names the required root artifacts and their ownership
  - pointer/update rules and version metadata posture are explicit
  - runtime/protocol evidence stays distinct from generic help-surface coverage
- **Dependencies**:
  - current manifest patterns under `cli_manifests/**`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - `governance/seam-1-closeout.md`
- **Verification**:
  - compare the proposed inventory against current codex and claude manifest roots
  - confirm later seams can tell which artifacts are sufficient for backend support publication
- **Rollout/safety**:
  - preserve this repo's one-truth-store manifest model
  - keep provider-backed smoke separate from deterministic repo evidence
- **Review surface refs**:
  - `review.md#r1---wrapper-and-manifest-handoff`

#### S2.T1 - Lock the manifest-root artifact inventory

- **Outcome**: the manifest contract explicitly names the required files, pointer rules, and
  protocol-evidence layout for `cli_manifests/opencode/`.
- **Inputs/outputs**:
  - Inputs: existing manifest roots, landed `SEAM-1` evidence posture
  - Outputs: `docs/specs/opencode-cli-manifest-contract.md`
- **Thread/contract refs**: `THR-02`, `C-04`
- **Implementation notes**:
  - keep protocol evidence distinct from generic wrapper coverage
  - keep pointer and version metadata rules deterministic
- **Acceptance criteria**:
  - later support publication can cite one manifest contract file
  - protocol evidence, replay evidence, and wrapper coverage remain complementary rather than
    interchangeable
- **Test notes**:
  - compare the inventory against existing `current.json`, `versions/*.json`, `pointers/**`, and
    `reports/**` patterns in current manifest roots
- **Risk/rollback notes**:
  - if the inventory stays implicit, support publication will drift and backend work will consume
    inconsistent evidence

Checklist:
- Implement: define the `cli_manifests/opencode/` inventory, pointer/update rules, and evidence
  layout
- Test: compare the contract against current manifest-root patterns and landed `SEAM-1` evidence
- Validate: confirm wrapper support publication has one deterministic manifest contract to follow
