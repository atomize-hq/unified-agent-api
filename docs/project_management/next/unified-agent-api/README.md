# Unified Agent API (Planning Pack)

Source ADR:
- `docs/adr/0009-unified-agent-api.md`

Canonical specs (source of truth):
- `docs/specs/unified-agent-api/`

Key planning artifacts:
- `docs/project_management/next/unified-agent-api/plan.md`
- `docs/project_management/next/unified-agent-api/tasks.json`
- `docs/project_management/next/unified-agent-api/spec_manifest.md`
- `docs/project_management/next/unified-agent-api/impact_map.md`
- `docs/project_management/next/unified-agent-api/ci_checkpoint_plan.md`
- `docs/project_management/next/unified-agent-api/decision_register.md`

Triads:
- `C0`: core crate surface (`crates/agent_api`, no backends)
- `C1`: Codex backend (feature-gated)
- `C2`: Claude Code backend (feature-gated)

Execution gate:
- Do not start execution triads until `quality_gate_report.md` is reviewed and `RECOMMENDATION: ACCEPT`.
