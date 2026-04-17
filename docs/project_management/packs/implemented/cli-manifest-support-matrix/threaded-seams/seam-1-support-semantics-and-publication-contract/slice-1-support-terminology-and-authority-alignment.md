---
slice_id: S1
seam_id: SEAM-1
slice_kind: documentation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - support layer vocabulary changes
    - manifest docs retain pre-contract wording
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
### S1 - Support terminology and authority alignment

- **User/system value**: maintainers and downstream seams read one consistent story across manifest docs, validator specs, and runbooks instead of reverse-engineering support meaning from mixed prose.
- **Scope (in/out)**:
  - In: align `cli_manifests/codex/**`, `cli_manifests/claude_code/**`, and `docs/specs/unified-agent-api/README.md` to the owned support-publication contract.
  - Out: schema changes, generator implementation, or policy rules that belong to later seams.
- **Acceptance criteria**:
  - both manifest READMEs, validator specs, and CI runbooks refer back to the canonical support-matrix spec for support publication meaning.
  - any language that implies `validated` alone is published support truth is removed or clarified.
  - the support matrix remains distinct from the capability matrix across all touched docs.
- **Dependencies**:
  - `S00` contract-definition output
  - current terminology in manifest READMEs, validator specs, runbooks, and RULES
- **Verification**:
  - diff each touched prose surface against `docs/specs/unified-agent-api/support-matrix.md`
  - confirm every mention of support state aligns with the target-first contract
  - confirm downstream seams no longer need to cite pack planning docs for core semantics
- **Rollout/safety**:
  - docs-only change
  - no runtime or artifact-shape changes
- **Review surface refs**:
  - `review.md#likely-mismatch-hotspots`
  - `review.md#pre-exec-gate-disposition`

#### S1.T1 - Align UAA spec index and manifest READMEs

- **Outcome**: the canonical UAA spec index and both manifest READMEs point to the same support-matrix authority and use the same support vocabulary.
- **Inputs/outputs**:
  - Inputs: `docs/specs/unified-agent-api/README.md`, `cli_manifests/codex/README.md`, `cli_manifests/claude_code/README.md`
  - Outputs: aligned authority references and clarified terminology
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**: keep capability-matrix references intact; only separate them cleanly from support publication.
- **Acceptance criteria**: a reviewer can tell which surface is canonical and which surfaces are descriptive without reading plan docs.
- **Test notes**: search the touched files for `validated`, `supported`, and `capability matrix` and confirm each use still fits the contract.
- **Risk/rollback notes**: if README-level terminology drifts, downstream seams will consume conflicting semantics.

#### S1.T2 - Align validator and runbook prose with the contract

- **Outcome**: validator specs and CI runbooks describe workflow metadata and support guarantees in language that matches the owned contract.
- **Inputs/outputs**:
  - Inputs: `cli_manifests/*/VALIDATOR_SPEC.md`, `cli_manifests/*/CI_AGENT_RUNBOOK.md`, `cli_manifests/*/RULES.json`
  - Outputs: clarified references and explicit separation between workflow status and published support truth
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**: keep enforcement behavior untouched; only normalize language and references needed for downstream interpretation.
- **Acceptance criteria**: validator and runbook prose no longer imply they are the primary publication contract for support truth.
- **Test notes**: inspect representative `validated` and `supported` sections in both agent trees after edits.
- **Risk/rollback notes**: lingering contradictory prose will cause later validation logic to encode the wrong meaning.

Checklist:
- Implement: align documentation surfaces to the owned contract
- Test: re-scan touched files for conflicting support vocabulary
- Validate: confirm every touched doc identifies the canonical support-matrix contract correctly
