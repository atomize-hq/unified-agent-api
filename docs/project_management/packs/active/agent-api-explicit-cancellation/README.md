# Execution Pack — Explicit cancellation (`agent_api`)

Source ADR: `docs/adr/0014-agent-api-explicit-cancellation.md`

This pack defines the concrete contracts, seams, and tests required to add an explicit cancellation
API to `agent_api` runs without undermining the existing drain-on-drop safety posture.

