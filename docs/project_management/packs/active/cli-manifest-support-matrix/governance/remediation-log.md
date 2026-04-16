# Remediation Log - CLI manifest support matrix

## Open remediations

None.

## Canonical remediation schema

```yaml
remediation_id: REM-001
origin_phase: pre_exec
source_gate: review
related_seam: SEAM-1
related_slice: null
related_thread: THR-01
related_contract: C-01
related_artifact: docs/project_management/packs/active/cli-manifest-support-matrix/seam-1-support-semantics-and-publication-contract.md
severity: blocking
status: open
owner_seam: SEAM-1
blocked_targets:
  - seam: SEAM-1
    field: status
    value: exec-ready
summary: example placeholder only; replace with a real machine-readable finding when a remediation is opened
required_fix: replace this example with the concrete corrective action
resolution_evidence: []
```

Rules:

- use seam ownership only
- blocking remediations must name concrete `blocked_targets`
- material and follow-up remediations may leave `blocked_targets: []` when no specific transition is blocked
- move resolved items to the resolved section with `status: resolved` and non-empty `resolution_evidence`

## Resolved remediations

None.
