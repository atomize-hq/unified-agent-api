# SEAM-5 — Tests

- **Name**: MCP management regression tests
- **Type**: integration (verification)
- **Goal / user value**: Prevent drift in the universal MCP management surface by verifying capability gating, request validation,
  output bounds, safe default advertising, and backend mappings.

## Scope

### In

- Unit tests for:
  - pin the requirements in the canonical MCP management spec:
    - `docs/specs/universal-agent-api/mcp-management-spec.md` → transport validation, context precedence, execution semantics,
      output truncation algorithm, built-in backend behavior, and verification policy.
- Integration tests:
  - run `list/get/add/remove` against an isolated home directory,
  - are deterministic and offline by default (see the canonical verification policy in the MCP management spec).

### Out

- End-to-end tests that require a real networked MCP server.
- Tests that assert a universal structured output schema (v1 returns bounded stdout/stderr).

## Primary interfaces (contracts)

- `AgentWrapperGateway::{mcp_list,mcp_get,mcp_add,mcp_remove}`
- `agent_api::mcp::{AgentWrapperMcp*Request, AgentWrapperMcpCommandOutput}`

## Key invariants / rules

- Tests must verify MCP management outputs are not emitted as run events.
- Tests must verify default advertising posture stays safe (write ops off unless enabled).

## Canonical sources (for tests)

- Spec + mapping truth: `docs/specs/universal-agent-api/mcp-management-spec.md`
- Capability gating + error taxonomy: `docs/specs/universal-agent-api/contract.md`

## Dependencies

- **Blocked by**:
  - SEAM-1 (contract + surface)
  - SEAM-2 (enablement + isolation)
  - SEAM-3/4 (mapping)

## Touch surface

- `crates/agent_api/src/**` tests (unit + integration)
- Potentially `crates/agent_api/src/backend_harness/**` (if reusing harness patterns for process isolation)

## Verification (definition of done)

- `make test` passes with new unit coverage.
- Integration tests follow the canonical verification policy in `docs/specs/universal-agent-api/mcp-management-spec.md`.
- `make preflight` passes once implementation lands.

## Risks / unknowns

- **Binary availability**: true end-to-end MCP tests may require installed `codex`/`claude` binaries; keep those tests opt-in or
  use hermetic fake binaries where possible.

## Rollout / safety

- Treat tests as the primary guardrail preventing accidental promotion of backend-specific behavior into universal v1.
