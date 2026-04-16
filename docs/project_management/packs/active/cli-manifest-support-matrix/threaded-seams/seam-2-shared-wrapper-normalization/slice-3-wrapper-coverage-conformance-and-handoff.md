---
slice_id: S3
seam_id: SEAM-2
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - shared module behavior diverges between current agents
    - handoff evidence omits the future-agent-shaped boundary
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
contracts_produced: []
contracts_consumed:
  - C-02
  - C-03
open_remediations: []
---
### S3 - Wrapper-coverage conformance and handoff

- **User/system value**: the seam exits with proof that the shared module preserves current behavior and with a crisp handoff for publication and validation consumers.
- **Scope (in/out)**:
  - In: targeted conformance coverage for the shared seam plus the downstream handoff notes needed for closeout.
  - Out: synthetic third-agent fixture expansion, publication rendering, and contradiction-policy enforcement.
- **Acceptance criteria**:
  - targeted tests or verification prove the shared seam preserves current Codex and Claude behavior.
  - the closeout evidence names the shared module, adapter surfaces, and verification commands explicitly.
  - downstream seams can tell which changes to shared normalization or root-intake shape would force revalidation.
- **Dependencies**:
  - landed outputs from `S1` and `S2`
  - existing xtask wrapper-coverage tests
- **Verification**:
  - run or update targeted wrapper-coverage tests
  - map every closeout evidence item to a file path or command output
  - confirm future-agent-shaped neutrality is preserved as a handoff rule, even though full synthetic-fixture coverage belongs to `SEAM-5`
- **Rollout/safety**:
  - conformance and handoff only
  - no publication or validator behavior hidden here
- **Review surface refs**:
  - `review.md#planned-seam-exit-gate-focus`
  - `../../threading.md`

#### S3.T1 - Prove behavior-preserving extraction

- **Outcome**: the shared seam lands with targeted evidence that current Codex and Claude behavior still matches expectations.
- **Inputs/outputs**:
  - Inputs: shared module, thin adapters, existing xtask wrapper-coverage tests
  - Outputs: conformance evidence ready for seam closeout
- **Thread/contract refs**: `THR-02`, `C-02`
- **Implementation notes**: keep verification narrow and seam-owned; do not pull in future publication or synthetic-fixture suites.
- **Acceptance criteria**: a reviewer can point to concrete test or command evidence showing no regression in current wrapper-coverage behavior.
- **Test notes**: run the targeted wrapper-coverage tests and capture the specific commands for closeout.
- **Risk/rollback notes**: weak verification would leave `SEAM-3` guessing whether the shared core is actually safe to consume.

#### S3.T2 - Record downstream stale triggers and handoff rules

- **Outcome**: later seams know exactly which shared-seam changes would require revalidation.
- **Inputs/outputs**:
  - Inputs: threading dependencies, shared module boundary, targeted verification results
  - Outputs: handoff-ready stale-trigger notes for seam closeout
- **Thread/contract refs**: `THR-02`, `C-02`, `C-03`
- **Implementation notes**: tie each stale trigger to a concrete shared module, adapter surface, or root-intake shape; do not use vague prose.
- **Acceptance criteria**: downstream seams can consume the handoff without re-reading pack extraction docs.
- **Test notes**: compare the final stale-trigger list against `SEAM-3` through `SEAM-5` dependency descriptions.
- **Risk/rollback notes**: missing handoff rules would make downstream promotion unsafe even if the extraction lands cleanly.

Checklist:
- Implement: capture the shared-seam conformance and downstream handoff evidence
- Test: run targeted wrapper-coverage verification
- Validate: confirm downstream stale triggers stay concrete and seam-owned
