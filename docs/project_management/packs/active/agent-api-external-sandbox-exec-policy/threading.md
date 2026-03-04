# Threading — External sandbox execution policy (dangerous)

This section makes coupling explicit: contracts/interfaces, dependency edges, and sequencing.

## Contract registry

- **ES-C01 — External sandbox execution policy extension key**
  - **Type**: config (core extension key)
  - **Definition**: `agent_api.exec.external_sandbox.v1` is a boolean extension key, validated
    before spawn, that requests a backend to relax internal guardrails because the host provides
    isolation externally.
  - **Owner seam**: SEAM-1
  - **Consumers**: SEAM-2/3/4/5

- **ES-C02 — Non-interactive invariant (external sandbox mode)**
  - **Type**: policy
  - **Definition**: When `agent_api.exec.external_sandbox.v1 == true`, the backend MUST remain
    non-interactive. If `agent_api.exec.non_interactive == false` is explicitly requested (and both
    keys are supported), the backend MUST fail before spawn with `AgentWrapperError::InvalidRequest`
    (contradictory intent).
  - **Owner seam**: SEAM-1
  - **Consumers**: SEAM-3/4/5

- **ES-C03 — Safe default advertising**
  - **Type**: permission
  - **Definition**: Built-in backends SHOULD NOT advertise `agent_api.exec.external_sandbox.v1` by
    default; externally sandboxed hosts opt-in explicitly via backend configuration.
  - **Owner seam**: SEAM-2
  - **Consumers**: SEAM-3/4/5

- **ES-C04 — Codex mapping contract**
  - **Type**: config
  - **Definition**: When enabled + requested, Codex backend maps the key to
    `codex --dangerously-bypass-approvals-and-sandbox ...` (or equivalent builder override).
  - **Owner seam**: SEAM-3

- **ES-C05 — Claude mapping contract**
  - **Type**: config
  - **Definition**: When enabled + requested, Claude Code backend maps the key to
    `claude --print --dangerously-skip-permissions ...` and applies any additional required opt-in
    flag deterministically (pre-spawn) based on CLI capability.
  - **Owner seam**: SEAM-4

## Dependency graph (text)

- `SEAM-1 blocks SEAM-2` because: backend enablement needs the final key semantics and contradiction rules.
- `SEAM-2 blocks SEAM-3` because: Codex mapping must only be reachable behind explicit opt-in (no default advertising).
- `SEAM-2 blocks SEAM-4` because: Claude mapping must only be reachable behind explicit opt-in (no default advertising).
- `SEAM-3 blocks SEAM-5` because: tests must pin the final Codex mapping behavior.
- `SEAM-4 blocks SEAM-5` because: tests must pin the final Claude mapping behavior.

## Critical path

`SEAM-1 (contract)` → `SEAM-2 (enablement)` → `SEAM-3/SEAM-4 (backend mapping)` → `SEAM-5 (tests)`

## Parallelization notes / conflict-safe workstreams

- **WS-SPEC**: SEAM-1 docs-only edits under `docs/specs/universal-agent-api/`.
- **WS-ENABLEMENT**: SEAM-2 backend config + capability advertising (Codex + Claude Code backends).
- **WS-CODEX**: SEAM-3 Codex mapping + validations.
- **WS-CLAUDE**: SEAM-4 Claude mapping + validations.
- **WS-TESTS**: SEAM-5 tests (can start with harness-level ordering tests once SEAM-1 is stable).
- **WS-INT (Integration)**: run `make preflight` and validate matrix/spec conformance after merge.

## Open questions / spikes (to de-risk early)

1) **Claude CLI allow-flag detection**: confirm the exact versions/conditions under which
   `--allow-dangerously-skip-permissions` is required, and choose a deterministic pre-spawn
   detection strategy (e.g., parse `claude --help` once + cache).
2) **Interaction with other exec-policy keys**: decide whether
   `agent_api.exec.external_sandbox.v1 == true`:
   - overrides other exec-policy keys (Codex sandbox/approval keys), or
   - fails closed when combined (preferred if ambiguity/footguns exist).
