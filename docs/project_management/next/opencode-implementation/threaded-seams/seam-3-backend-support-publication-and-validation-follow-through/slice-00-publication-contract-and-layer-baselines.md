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
    - support-matrix semantics, capability-inventory semantics, or committed root-set assumptions drift
    - publication evidence starts implying UAA promotion or collapses support layers together
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
  - THR-06
  - THR-07
contracts_produced:
  - C-04
contracts_consumed:
  - C-01
  - C-02
  - C-03
  - C-07
open_remediations: []
---
### S00 - publication-contract-and-layer-baselines

- **User/system value**: give `SEAM-3` one concrete publication baseline so OpenCode support can
  be surfaced mechanically without blurring backend support into UAA support.
- **Scope (in/out)**:
  - In: support-layer semantics, OpenCode root/backend enrollment boundary, capability-inventory
    separation, passthrough-visibility wording, and deterministic publication verification posture
  - Out: new backend runtime behavior, wrapper contract changes, or generic future-agent
    publication scaffolding
- **Acceptance criteria**:
  - the seam-local plan names exactly which publication surfaces OpenCode must join and what those
    surfaces are allowed to mean
  - the plan keeps support-matrix publication and capability inventory separate and reviewable
  - the plan makes explicit that backend support and passthrough visibility do not imply UAA
    promotion
- **Dependencies**:
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
  - `../../../opencode-cli-onboarding/governance/seam-4-closeout.md`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
  - `docs/specs/opencode-cli-manifest-contract.md`
  - `docs/specs/opencode-agent-api-backend-contract.md`
- **Verification**:
  - compare the publication baseline against current `support-matrix` and `capability-matrix`
    semantics plus the landed OpenCode manifest/backend evidence
  - confirm the seam-local plan preserves the separation between manifest support, backend support,
    UAA unified support, and passthrough visibility
  - confirm downstream readers can cite committed outputs and `THR-07` instead of inferring intent
    from planning prose
- **Rollout/safety**:
  - fail closed on UAA-promotion implications
  - fail closed on support-layer collapse
  - preserve committed-evidence and drift-check commands as the default proof path
- **Review surface refs**:
  - `review.md#r1---publication-handoff-flow`
  - `review.md#r2---support-layer-boundary`

#### S00.T1 - Lock the support-layer publication boundary

- **Outcome**: the seam-local plan names the exact support-layer meaning OpenCode publication is
  allowed to carry.
- **Inputs/outputs**:
  - Inputs: `THR-04`, `THR-05`, `THR-06`, `docs/specs/unified-agent-api/support-matrix.md`,
    landed manifest/backend evidence
  - Outputs: concrete execution baseline for support rows, notes, and non-promotion wording
- **Thread/contract refs**: `THR-04`, `THR-06`, `THR-07`, `C-04`
- **Implementation notes**:
  - keep manifest support, backend support, UAA unified support, and passthrough visibility as
    distinct user-facing meanings
  - make the no-promotion posture explicit in committed publication surfaces rather than leaving it
    implicit in review prose
  - keep this slice doc-only: S00 defines the boundary, while S1-S3 own the code, output, and
    validation changes
- **Acceptance criteria**:
  - implementation has one explicit publication meaning baseline to follow
  - implementation knows backend support and capability inventory must not imply universal support
  - implementation knows passthrough visibility stays a separate explanation surface
- **Test notes**:
  - compare the baseline against current support-matrix semantics and OpenCode stale-trigger rules
- **Risk/rollback notes**:
  - if support-layer meaning stays implicit, OpenCode publication will overclaim support

#### S00.T2 - Lock the OpenCode publication touch surface and proof path

- **Outcome**: the seam-local plan names the exact files and deterministic commands this seam must
  update and prove.
- **Inputs/outputs**:
  - Inputs: current `crates/xtask` publication code, current support outputs, landed OpenCode root
    and backend evidence
  - Outputs: concrete execution baseline for `crates/xtask/**`,
    `cli_manifests/support_matrix/current.json`, and the normative docs under
    `docs/specs/unified-agent-api/**`
- **Thread/contract refs**: `THR-05`, `THR-06`, `THR-07`, `C-04`
- **Implementation notes**:
  - treat `cargo run -p xtask -- support-matrix --check`,
    `cargo run -p xtask -- capability-matrix`, and
    `cargo run -p xtask -- codex-validate --root cli_manifests/opencode` as the default proof set
  - keep generator and committed-output changes bounded to OpenCode publication follow-through
  - do not turn OpenCode enrollment into generic future-agent framework work
- **Acceptance criteria**:
  - implementation knows the exact publication touch surface it owns
  - implementation knows which deterministic commands must pass before closeout
  - implementation knows this seam owns projection of landed truth, not creation of new runtime
    truth
- **Test notes**:
  - compare the planned touch surface against current support publication and capability inventory
    code paths
- **Risk/rollback notes**:
  - if the proof path stays vague, later validation will drift back to ad hoc interpretation

Checklist:
- Implement: lock the publication semantics, OpenCode enrollment boundary, and deterministic proof
  path
- Test: compare the plan against current support publication and capability inventory semantics
- Validate: confirm `C-04` is concrete enough for conformance work to start
- Guardrail: do not treat S00 as authority to change backend runtime behavior, wrapper contracts,
  or UAA promotion scope
