---
slice_id: S3
seam_id: SEAM-1
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - touched docs diverge from the canonical support-matrix contract
    - downstream seams rely on plan prose instead of repo-owned contract surfaces
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
contracts_produced: []
contracts_consumed:
  - C-01
open_remediations: []
---
### S3 - Support publication touch-surface conformance

- **User/system value**: the seam exits with a crisp downstream handoff, so later seams can trust the repo-owned contract surfaces without reopening planning questions.
- **Scope (in/out)**:
  - In: define and verify the touch-surface checklist that must stay aligned when this seam lands.
  - Out: downstream normalization, publication rendering, and validator behavior beyond the conformance notes needed for handoff.
- **Acceptance criteria**:
  - the seam records exactly which repo surfaces must stay aligned for `THR-01` to publish.
  - the closeout plan tells downstream seams what evidence proves the contract actually landed, using explicit file paths or command outputs.
  - the touch-surface checklist is narrow enough to run during seam execution and closeout.
- **Dependencies**:
  - `S00` contract-definition output
  - `S1` and `S2` planned touch surfaces
- **Verification**:
  - ensure every touched path appears in either the seam brief, `review.md`, or this slice with a clear reason and a linked evidence item
  - confirm downstream seams can derive their stale triggers from the recorded touch surfaces and the seam-exit evidence list
  - confirm no extra user-facing contract surface is introduced outside `docs/specs/**` and the planned JSON artifact
- **Rollout/safety**:
  - planning and conformance only
  - no new outward-facing contract surfaces
- **Review surface refs**:
  - `review.md#planned-seam-exit-gate-focus`
  - `../../threading.md`

#### S3.T1 - Record the execution and closeout evidence checklist

- **Outcome**: the seam has a tight list of evidence required for landing, closeout, and downstream revalidation.
- **Inputs/outputs**:
  - Inputs: seam brief, `review.md`, `threading.md`
  - Outputs: execution checklist and seam-exit evidence plan embodied in this slice and `S99`
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**: keep the list bounded to contract surfaces, command naming, and terminology alignment evidence. The checklist should explicitly name these evidence items:
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/README.md`
  - `cli_manifests/codex/README.md`
  - `cli_manifests/claude_code/README.md`
  - `cli_manifests/codex/VALIDATOR_SPEC.md`
  - `cli_manifests/claude_code/VALIDATOR_SPEC.md`
  - `cli_manifests/codex/CI_AGENT_RUNBOOK.md`
  - `cli_manifests/claude_code/CI_AGENT_RUNBOOK.md`
  - `cli_manifests/codex/RULES.json`
  - `cli_manifests/claude_code/RULES.json`
  - `cargo run -p xtask -- support-matrix --help`
  - `cargo run -p xtask -- --help`
- **Acceptance criteria**: a reviewer can tell what must land before `THR-01` becomes `published`, and each item resolves to one concrete file path or command output.
- **Test notes**: verify each evidence item maps to a touched file or command output, and that each one is a documented touch surface rather than plan prose.
- **Risk/rollback notes**: vague closeout evidence will leave downstream promotion ambiguous even if the seam lands.

#### S3.T2 - Confirm downstream stale triggers and handoff rules

- **Outcome**: later seams know exactly which contract changes require revalidation once `SEAM-1` lands.
- **Inputs/outputs**:
  - Inputs: `threading.md`, seam brief stale triggers, touch-surface inventory
  - Outputs: downstream stale-trigger notes and handoff rules ready for closeout
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**: tie each stale trigger to a concrete contract surface or command name, not general prose. Record the handoff rules as a direct mapping:
  - support-layer vocabulary changes -> `docs/specs/unified-agent-api/support-matrix.md` and the linked manifest docs
  - canonical publication location changes -> `docs/specs/unified-agent-api/README.md` and `docs/specs/unified-agent-api/support-matrix.md`
  - neutral `xtask support-matrix` naming changes -> `cargo run -p xtask -- support-matrix --help`
  - manifest prose that reintroduces `validated` as published support truth -> `cli_manifests/codex/README.md`, `cli_manifests/claude_code/README.md`, and the validator/runbook surfaces
- **Acceptance criteria**: downstream seams can consume the handoff without rereading pack extraction docs, because each stale trigger names the surface that would force revalidation.
- **Test notes**: compare the final stale-trigger list against `SEAM-2` through `SEAM-5` dependency descriptions and confirm every entry points back to a concrete repo artifact or help command.
- **Risk/rollback notes**: missing stale-trigger rules will cause unsafe downstream promotion after contract changes.

Checklist:
- Implement: record the bounded touch-surface and closeout checklist with explicit file/output references
- Test: map each checklist item to a real repo surface or command output
- Validate: confirm downstream handoff rules are concrete enough for `THR-01` publication and stay inside seam-local conformance / closeout-prep surfaces
