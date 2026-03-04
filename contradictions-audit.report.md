# Contradictions Audit Report

## Meta
- Generated at: 2026-03-04T14:51:54Z
- Files audited: 46
- Scan used: yes (`contradictions-audit.scan.json`)

## Summary
- Total issues: 2
- By severity: blocker=0, critical=0, major=1, minor=1
- High-confidence contradictions: 2

## Issue index
| ID | Severity | Confidence | Type | Subject | Files |
|---|---|---|---|---|---|
| CX-0001 | major | high | strength_mismatch | `agent_api.exec.external_sandbox.v1` contradiction rule with `agent_api.exec.non_interactive` | docs/project_management/packs/active/agent-api-external-sandbox-exec-policy/seam-1-external-sandbox-extension-key.md; docs/specs/universal-agent-api/extensions-spec.md |
| CX-0002 | minor | high | strength_mismatch | Default advertising posture for `agent_api.exec.external_sandbox.v1` (built-in backends) | docs/project_management/packs/active/agent-api-external-sandbox-exec-policy/threading.md; docs/specs/universal-agent-api/extensions-spec.md |

## Issues

### CX-0001 — Contradiction handling MUST vs SHOULD
- Severity: major
- Confidence: high
- Type: strength_mismatch
- Subject: `agent_api.exec.external_sandbox.v1` contradiction rule with `agent_api.exec.non_interactive`
- Scope: environment=all, version=v1, feature_flag=none, timeline=current
- Statement A: `docs/project_management/packs/active/agent-api-external-sandbox-exec-policy/seam-1-external-sandbox-extension-key.md:33-36` — “... the backend SHOULD fail closed with `AgentWrapperError::InvalidRequest`.”
- Statement B: `docs/specs/universal-agent-api/extensions-spec.md:125-127` — “... the backend MUST fail before spawn with `AgentWrapperError::InvalidRequest` (contradictory intent).”
- Why this conflicts: The planning pack describes the contradiction handling as a `SHOULD`, while the canonical extensions registry makes it a `MUST` pre-spawn failure. This normative-strength mismatch can mislead implementers/tests into treating the contradiction rule as discretionary, violating the approved contract.
- What must be true:
  - Whether the `external_sandbox=true` + `non_interactive=false` combination is universally invalid (hard error) for all backends that support both keys, or merely discouraged.
  - If it is universally invalid, all planning artifacts and tests should consistently state `MUST fail pre-spawn` (not `SHOULD`).
- Suggested evidence order:
  - codebase
  - tests
  - runtime-config
  - git-history
  - other-docs
  - external
  - decision

### CX-0002 — Default advertising MUST NOT vs SHOULD NOT
- Severity: minor
- Confidence: high
- Type: strength_mismatch
- Subject: Default advertising posture for `agent_api.exec.external_sandbox.v1` (built-in backends)
- Scope: environment=all, version=v1, feature_flag=none, timeline=current
- Statement A: `docs/project_management/packs/active/agent-api-external-sandbox-exec-policy/threading.md:25-26` — “Built-in backends MUST NOT advertise `agent_api.exec.external_sandbox.v1` by default...”
- Statement B: `docs/specs/universal-agent-api/extensions-spec.md:134-135` — “Built-in backends SHOULD NOT advertise this capability by default...”
- Why this conflicts: The contract-level spec describes default advertising as `SHOULD NOT`, but the planning pack's contract registry states `MUST NOT`. Even though “MUST NOT” satisfies “SHOULD NOT” in practice, the mismatch in normative strength is contradictions-class drift that obscures whether default advertising is discouraged or strictly forbidden for built-in backends.
- What must be true:
  - Whether any built-in backend is allowed (under any default configuration) to advertise `agent_api.exec.external_sandbox.v1` without explicit host enablement.
  - If the universal contract remains `SHOULD NOT`, planning docs should avoid strengthening it to `MUST NOT` unless a scoped rationale is stated.
- Suggested evidence order:
  - codebase
  - tests
  - runtime-config
  - git-history
  - other-docs
  - external
  - decision
