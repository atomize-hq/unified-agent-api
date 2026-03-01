### S4 — Codex `agent_api.session.fork.v1` integration (app-server flow + bounded events + cancellation)

- This slice was decomposed into sub-slices in this directory:
  - `slice-4-codex-fork-v1-agent-api-integration/`
- Archived original: `archive/slice-4-codex-fork-v1-agent-api-integration.md`

#### Sub-slices

- `subslice-1-fork-policy-extraction.md` (S4a) — Parse/validate `fork.v1` and plumb the typed selector into Codex backend policy (no behavior change yet).
- `subslice-2-core-app-server-fork-flow.md` (S4b) — Implement the core app-server fork flow (`initialize` → `thread/list?` → `thread/fork` → `turn/start`) and expose a run handle.
- `subslice-3-event-mapping-safety-and-selection-failures.md` (S4c) — Bounded notification→event mapping, non-interactive fail-fast, selection-failure translation, and `$ /cancelRequest` wiring.
- `subslice-4-agent-api-integration-tests-fake-app-server.md` (S4d) — Fake `codex app-server` JSON-RPC binary + `agent_api` integration tests for success/failure/safety/cancel paths.
- `subslice-5-capability-advertisement.md` (S4e) — Advertise `agent_api.session.fork.v1` for Codex after tests pass.
