---
slice_id: S3
seam_id: SEAM-5
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - handoff evidence omits the future-agent fixture surface
    - downstream guidance stops naming the owned fixture/golden contract boundary
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-05
contracts_produced: []
contracts_consumed:
  - C-06
open_remediations: []
---
### S3 - Neutral handoff and thread surface

- **User/system value**: downstream fixture work gets a crisp neutral-hand-off boundary instead of an ad hoc regression bundle, with every proof point tied to a real repo surface.
- **Scope (in/out)**:
  - In: record the handoff evidence, downstream stale-trigger posture, and future-agent neutrality notes needed for seam exit.
  - Out: fixture-matrix expansion and golden-render behavior.
- **Acceptance criteria**:
  - the seam-exit evidence can name the future-agent fixture surface without extra interpretation.
  - downstream seams can tell which fixture or golden changes force revalidation.
  - the neutral fixture contract stays explicit enough for future onboarding work.
  - the handoff explicitly maps the neutral fixture proof, publication outputs, and downstream revalidation triggers to concrete paths and commands.
- **Verification**:
  - map each handoff item to a real file path or command output
  - confirm the final neutral fixture surface is tied to the shared model
  - confirm downstream stale triggers stay seam-owned and machine-readable
  - keep the evidence boundary documentary-only; do not add tests or regenerate outputs in this slice

Checklist:
- Implement: capture the neutral handoff and thread surface evidence
- Test: prove the handoff points at the same shared model used by the regression suites
- Validate: confirm `THR-05` can close out without reintroducing agent-specific branching

#### Evidence map

| Handoff item | Real repo path or command | Why it matters |
| --- | --- | --- |
| Neutral future-agent proof | `crates/xtask/tests/support_matrix_derivation.rs` and `cargo test -p xtask --test support_matrix_derivation -- --nocapture` | This is the landed synthetic future-agent-shaped coverage that proves the shared core stays shape-driven. |
| Publication surface for JSON | `cli_manifests/support_matrix/current.json` and `cargo run -p xtask -- support-matrix --check` | This is the committed JSON projection that must stay derived from the same row model as the tests. |
| Publication surface for Markdown | `docs/specs/unified-agent-api/support-matrix.md` and `cargo run -p xtask -- support-matrix --check` | This is the normative Markdown projection that must stay in lockstep with the JSON artifact. |
| Thread boundary | `docs/project_management/packs/active/cli-manifest-support-matrix/threading.md` (`C-07`, `THR-05`) | This is the contract/thread boundary S3 is documenting, not changing. |
| Downstream revalidation triggers | `threading.md`, `seam.md`, and the stale-triggers in this file | These tell future seams when fixture/golden drift requires revalidation without reopening S99. |

#### Downstream revalidation triggers

- Future-agent fixture coverage stops being shape-driven or starts branching on agent names.
- The synthetic future-agent root no longer exercises the same shared derivation path as Codex and Claude fixtures.
- `cli_manifests/support_matrix/current.json` and `docs/specs/unified-agent-api/support-matrix.md` diverge from the same derived row model.
- Row ordering, evidence-note wording, or projection rules change and require regression refresh in S1 or S2.
- The thread boundary stops naming `C-07` and `THR-05` concretely in downstream handoff evidence.
