---
slice_id: S1
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers: []
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
contracts_produced:
  - C-01
  - C-02
  - C-03
  - C-04
contracts_consumed: []
open_remediations: []
candidate_subslices: []
---
### S1 - Canonical drift verification (C-01..C-04)

- **User/system value**: ensures downstream seams implement against one canonical truth and do not inherit ambiguous or contradictory doc guidance.
- **Scope (in/out)**:
  - In:
    - compare canonical owner spec + registry entry + inherited run lifecycle/error baselines
    - detect any mismatch against ADR/pack restatements
    - if mismatch exists, update canonical specs first, then sync ADR + pack
  - Out:
    - backend code changes
- **Acceptance criteria**:
  - mismatch-free alignment across the compared sources for v1 semantics (trim/bounds/absence, invalid template, runtime rejection posture)
  - any drift is resolved with canonical-first edits
- **Dependencies**: none
- **Verification**:
  - record a repeatable comparison scope (files + headings)
  - append a pass/fail entry under `../../seam-1-core-extension-contract.md`
- **Rollout/safety**: treat unresolved drift as blocking for SEAM-2 and beyond.

#### S1.T1 - Run and record the drift comparison

- **Outcome**: a clear pass/fail result plus the exact compared sources list.
- **Thread/contract refs**: `THR-01`, `C-01..C-04`
- **Acceptance criteria**: record includes date, verifier, compared sources, and pass/fail.

Checklist:
- Implement: enumerate the comparison points for MS-C01 through MS-C04 and record the alignment result for each source.
- Test: re-read every cited section to confirm the checklist did not miss capability bucket placement, byte bounds, or runtime rejection wording.
- Validate: confirm the checklist produces either "no unresolved canonical-doc delta" or a concrete patch list limited to canonical docs.
- Validate: compared sources list matches the seam brief.
- Validate: result is recorded under the pack seam brief (`../../seam-1-core-extension-contract.md`).
- Cleanup: remove any provisional notes that duplicate authoritative text once the final verdict is captured.

#### S1.T2 — Reconcile canonical universal docs if drift is found

- **Outcome**: Canonical universal spec files reflect one consistent truth for model-selection semantics before any non-normative sync work proceeds.
- **Inputs/outputs**:
  - Inputs: mismatch list from S1.T1
  - Outputs: patches to the affected files in `docs/specs/unified-agent-api/` and, only if required by the mismatch, the inherited baseline docs that define the error/run-lifecycle behavior
- **Implementation notes**:
  - patch the owner spec first, then the inherited baseline doc only when the mismatch actually lives there
  - do not edit ADR-0020 or pack files in this task; those belong to S2 after canonical truth is settled
- **Acceptance criteria**:
  - every mismatch from S1.T1 is either resolved in canonical docs or explicitly marked as a blocker with rationale
  - no patch introduces backend-specific mapping commitments outside the existing seam scope
- **Test notes**:
  - rerun the same drift checklist after patching and confirm the mismatched items now read as aligned
- **Risk/rollback notes**:
  - if a proposed patch would expand scope beyond MS-C01 through MS-C04, defer it instead of bundling unrelated policy changes

Checklist:
- Implement: update only the canonical files needed to eliminate the concrete mismatch list from S1.T1.
- Test: rerun the full comparison against the patched canonical text and verify the prior deltas are closed.
- Validate: confirm the resulting wording still matches SEAM-1 scope and does not invent wrapper-owned model behavior.
- Cleanup: trim provisional commentary from the patch once the canonical wording is self-sufficient.

#### S1.T3 — Produce the final verification verdict for handoff to S2

- **Outcome**: SEAM-1 has a final pass/fail result with enough evidence for S2 to publish or to stop the seam as blocked.
- **Inputs/outputs**:
  - Inputs: final comparison result from S1.T1 and any canonical patches from S1.T2
  - Outputs: a concise verification verdict, the compared-source list, and the synchronization reference S2 will record
- **Implementation notes**:
  - when no patch was needed, still produce an explicit final verdict anchored to the exact compared sources
  - when the work is still uncommitted, capture the current `git HEAD` plus working-tree delta note; replace it later once a commit or PR exists
- **Acceptance criteria**:
  - the verdict is specific enough that S2 can append it without reinterpretation
  - the verdict clearly states whether downstream seams remain blocked or may cite the pass
- **Test notes**:
  - confirm the compared-source list matches the seam brief verification requirements exactly
- **Risk/rollback notes**:
  - do not let a stale provisional synchronization reference survive once a commit or PR reference exists

Checklist:
- Implement: prepare the final compared-source list, verdict string, and synchronization reference for publication.
- Test: cross-check the verdict inputs against the seam brief to ensure no required source or rule was omitted.
- Validate: confirm the handoff package is sufficient for S2 to update the verification record without re-running discovery.
- Cleanup: discard superseded provisional verdicts so downstream seams see one current gate state.
