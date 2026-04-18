---
slice_id: S1
seam_id: SEAM-1
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - wrapper event, completion, parser, or redaction ownership drifts into downstream seams
    - deterministic fixture or fake-binary posture no longer matches the canonical run contract
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
  - THR-05
contracts_produced:
  - C-01
contracts_consumed:
  - C-07
open_remediations: []
---
### S1 - wrapper-crate-runtime-and-fixture-foundation

- **User/system value**: establish the concrete `crates/opencode/` wrapper foundation so backend
  work later consumes a real wrapper-owned runtime boundary instead of planning-only intent.
- **Scope (in/out)**:
  - In: new crate skeleton, workspace wiring, wrapper spawn surface, typed-event and completion
    handoff entry points, parser and redaction boundaries, and deterministic fixture or fake-binary
    proof surfaces
  - Out: `crates/agent_api/` backend mapping, support publication, and helper-surface recovery
- **Acceptance criteria**:
  - `crates/opencode/` exists in the workspace with a bounded wrapper-owned API surface
  - implementation preserves the canonical `run --format json` boundary and accepted controls only
  - deterministic transcript, fake-binary, or offline-parser proof exists for wrapper behavior
- **Dependencies**:
  - `Cargo.toml`
  - `docs/specs/opencode-wrapper-run-contract.md`
  - `docs/specs/opencode-onboarding-evidence-contract.md`
  - current wrapper patterns under `crates/codex/` and `crates/claude_code/`
- **Verification**:
  - targeted wrapper tests prove `--model`, `--session` or `--continue`, `--fork`, and `--dir`
    handling without relying on live provider-backed smoke
  - fixture or fake-binary coverage proves parser, event typing, completion handoff, and redaction
  - wrapper APIs do not surface helper-only behavior or raw provider diagnostics as canonical truth
- **Rollout/safety**:
  - keep the wrapper automation-safe and headless by default
  - fail closed when runtime evidence contradicts the canonical contract
  - keep live provider-backed smoke out of default completion criteria
- **Review surface refs**:
  - `review.md#r2---repo-implementation-boundary`

#### S1.T1 - Materialize the wrapper crate and workspace wiring

- **Outcome**: the repo has a concrete `crates/opencode/` home and the workspace can build or test
  it without backend or publication work leaking into the same change set.
- **Inputs/outputs**:
  - Inputs: `Cargo.toml`, current wrapper crate patterns, S00 baseline
  - Outputs: workspace membership and initial crate structure under `crates/opencode/**`
- **Thread/contract refs**: `THR-05`, `C-01`
- **Implementation notes**:
  - keep the public surface aligned to the canonical run contract
  - avoid generic future-agent abstractions
- **Acceptance criteria**:
  - the crate is buildable and ready for bounded wrapper implementation
  - naming and path conventions match current repo norms
- **Test notes**:
  - run targeted crate builds or tests once the crate exists
- **Risk/rollback notes**:
  - if workspace wiring becomes generic scaffolding, trim it back before landing

#### S1.T2 - Land deterministic wrapper proof surfaces

- **Outcome**: wrapper implementation has deterministic transcript, fake-binary, or offline-parser
  coverage proving the canonical run behavior without live-account dependence.
- **Inputs/outputs**:
  - Inputs: canonical run contract, evidence contract, current fixture norms
  - Outputs: deterministic wrapper proof paths under `crates/opencode/**` and related fixture
    locations
- **Thread/contract refs**: `THR-05`, `C-01`
- **Implementation notes**:
  - keep parser ownership and completion handoff in the wrapper seam
  - use live smoke only as stale-trigger or basis-lock evidence
- **Acceptance criteria**:
  - deterministic proof covers parser, event typing, completion handoff, and redaction
  - later seams can consume wrapper outputs without re-deriving runtime semantics
- **Test notes**:
  - prefer targeted crate tests and replay-oriented coverage over provider-backed runs
- **Risk/rollback notes**:
  - if deterministic proof is missing, backend work will reopen this seam with avoidable ambiguity

Checklist:
- Implement: land the crate skeleton, runtime boundary, and deterministic wrapper proof surfaces
- Test: prove wrapper behavior with targeted tests and deterministic evidence
- Validate: confirm downstream seams can consume wrapper-owned behavior without inferring it
