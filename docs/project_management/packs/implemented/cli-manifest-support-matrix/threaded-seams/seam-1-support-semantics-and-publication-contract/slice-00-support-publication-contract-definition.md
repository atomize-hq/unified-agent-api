---
slice_id: S00
seam_id: SEAM-1
slice_kind: contract_definition
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - support layer vocabulary changes
    - canonical publication location changes
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
contracts_produced:
  - C-01
contracts_consumed: []
open_remediations: []
---
### S00 - Support publication contract definition

- **User/system value**: downstream seams get one execution-grade definition of published support truth before they freeze shared normalization, renderer, and validator behavior.
- **Scope (in/out)**:
  - In: define the four support layers, publication targets, authority rules, and verification checklist in the canonical support-matrix spec surfaces.
  - Out: generator implementation, row derivation, fixture suites, and repo-gate enforcement.
- **Acceptance criteria**:
  - `docs/specs/unified-agent-api/support-matrix.md` defines manifest support, backend support, UAA unified support, and passthrough visibility without conflating them with workflow status.
  - the contract names `cli_manifests/support_matrix/current.json` and `docs/specs/unified-agent-api/support-matrix.md` as the phase-1 publication surfaces.
  - the contract states that target-scoped rows are primary and per-version summaries are projections.
  - the contract leaves capability-matrix behavior separate from support-matrix publication.
- **Dependencies**:
  - current manifest README, validator, runbook, and RULES terminology
  - `docs/specs/unified-agent-api/README.md`
  - `crates/xtask/src/main.rs`
- **Verification**:
  - compare the final support-matrix spec against the current manifest docs and confirm every support-layer term resolves to one meaning
  - confirm the spec gives downstream seams enough detail to implement without reopening authority or output-path decisions
  - confirm the spec describes `validated` and `supported` as workflow/policy states distinct from published support rows
- **Rollout/safety**:
  - docs/spec-only boundary; no runtime `agent_api` change
  - capability-matrix workflow remains intact
- **Review surface refs**:
  - `review.md#likely-mismatch-hotspots`
  - `../../seam-1-support-semantics-and-publication-contract.md`

For a `slice_kind: contract_definition` slice that produces an owned contract:

- make the contract rules concrete enough that the producer seam can later satisfy `gates.pre_exec.contract`
- include a narrow verification plan with test locations, edge cases, and pass/fail conditions
- do not require the final accepted contract artifact to exist before the producer seam can become `exec-ready`

#### S00.T1 - Define support-layer semantics and publication authority

- **Outcome**: one canonical support-matrix spec defines the four support layers, target-first row primacy, and authority boundaries against workflow metadata.
- **Inputs/outputs**:
  - Inputs: `docs/specs/unified-agent-api/README.md`, manifest docs under `cli_manifests/**`
  - Outputs: `docs/specs/unified-agent-api/support-matrix.md`, updated UAA spec index link
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**: make the spec explicit about what is published support truth versus what remains pointer or workflow metadata.
- **Acceptance criteria**: the spec answers where support truth lives, what each layer means, and why `versions/<version>.json.status` is not published support truth.
- **Test notes**: review every term that uses `validated`, `supported`, or `current` across the touch surface and reconcile it to the new spec.
- **Risk/rollback notes**: if the vocabulary still admits multiple readings, downstream seams will freeze inconsistent behavior.

#### S00.T2 - Lock the publication targets and verification checklist

- **Outcome**: the owned contract names the JSON and Markdown targets and records how reviewers confirm drift has not re-entered the docs.
- **Inputs/outputs**:
  - Inputs: scope brief publication targets, manifest pointer semantics
  - Outputs: publication-target section and verification checklist in the canonical spec
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**: keep the checklist focused on authority, naming, and output-path invariants; do not turn it into generator implementation design.
- **Acceptance criteria**: the checklist tells an implementer exactly which surfaces must align before downstream seams can proceed.
- **Test notes**: validate that each listed touch surface is in scope for either S1, S2, or S3.
- **Risk/rollback notes**: ambiguous output targets will force later seams to reopen the contract and block promotion.

Checklist:
- Implement: define the canonical support-matrix contract and verification notes
- Test: cross-check terminology and publication targets against the current repo surfaces
- Validate: confirm `C-01` is concrete enough for downstream implementation planning
