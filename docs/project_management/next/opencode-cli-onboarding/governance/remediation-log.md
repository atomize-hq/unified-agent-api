# Remediation Log - OpenCode CLI onboarding

## Open remediations

```yaml
remediation_id: REM-001
origin_phase: pre_exec
source_gate: revalidation
related_seam: SEAM-1
related_slice: null
related_thread: THR-01
related_contract: C-02
related_artifact: docs/project_management/next/cli-agent-onboarding-third-agent-packet.md
severity: blocking
status: open
owner_seam: SEAM-1
blocked_targets:
  - seam: SEAM-1
    field: status
    value: exec-ready
summary: provider-backed smoke evidence is maintainer-specific and does not yet define a reusable fixture-versus-smoke reproducibility envelope
required_fix: turn the packet observations into explicit fixture, maintainer-smoke, prerequisite, and reopen rules before SEAM-1 is promoted beyond proposed
resolution_evidence: []
```

```yaml
remediation_id: REM-002
origin_phase: pre_exec
source_gate: review
related_seam: SEAM-1
related_slice: null
related_thread: THR-01
related_contract: C-01
related_artifact: docs/project_management/next/opencode-cli-onboarding/plan.md
severity: material
status: open
owner_seam: SEAM-1
blocked_targets: []
summary: helper surfaces such as serve acp attach and interactive TUI remain attractive scope-expansion paths and could blur the v1 wrapper boundary
required_fix: keep the deferred-surface list and reopen criteria explicit in SEAM-1 and carry that boundary forward into SEAM-2 and SEAM-3 review artifacts
resolution_evidence: []
```

Rules:

- Use canonical YAML blocks for remediation entries.
- Use seam ownership only. Do not emit `WS-*` owners.
- For `severity: blocking`, `blocked_targets` must not be empty.
- For `severity: material` or `follow_up`, use `blocked_targets: []` unless a concrete blocked
  transition also applies.

## Resolved remediations

- None yet.
