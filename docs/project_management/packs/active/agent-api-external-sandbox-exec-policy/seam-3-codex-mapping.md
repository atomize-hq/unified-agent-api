# SEAM-3 — Codex backend mapping

- **Name**: Codex mapping for `agent_api.exec.external_sandbox.v1`
- **Type**: capability (backend mapping)
- **Goal / user value**: when enabled + requested, run Codex in a mode compatible with external
  sandboxing by relaxing internal approvals/sandbox guardrails without prompting.

## Scope

- In:
  - Validate the new key (boolean) before spawn.
  - Enforce the non-interactive invariant and contradiction rule with
    `agent_api.exec.non_interactive`.
  - Map `agent_api.exec.external_sandbox.v1 == true` to:
    - `codex --dangerously-bypass-approvals-and-sandbox exec ...`, or
    - `CodexClientBuilder::dangerously_bypass_approvals_and_sandbox(true)` (preferred, if available).
  - Ensure mapping applies consistently across flows that spawn Codex (exec, resume, fork flow).
- Out:
  - Changes to Codex wrapper crate unless required (assumed already supported).

## Primary interfaces (contracts)

- **Input**: `extensions["agent_api.exec.external_sandbox.v1"] == true` (when capability is enabled)
- **Output**: Codex CLI invocation includes the dangerous bypass override and remains non-interactive.

## Key invariants / rules

- MUST NOT hang on prompts.
- MUST be validated before spawn.
- SHOULD fail closed on explicit contradiction with `agent_api.exec.non_interactive == false`.

## Dependencies

- Blocks: SEAM-5 (tests).
- Blocked by: SEAM-1 (semantics) + SEAM-2 (enablement).

## Touch surface

- `crates/agent_api/src/backends/codex.rs`
- `crates/agent_api/src/backends/codex/exec.rs`
- `crates/agent_api/src/backends/codex/fork.rs`
- `crates/agent_api/src/backends/codex/tests.rs`
- (likely no change) `crates/codex/src/builder/mod.rs` already exposes `dangerously_bypass_approvals_and_sandbox(...)`.

## Verification

- Unit tests that pin:
  - default capabilities do not advertise the key,
  - contradiction behavior (`external_sandbox=true` + `non_interactive=false`) fails pre-spawn, and
  - the generated argv/builder config includes the dangerous bypass override when requested.

## Risks / unknowns

- Interaction with existing Codex exec-policy keys (`backend.codex.exec.approval_policy`,
  `backend.codex.exec.sandbox_mode`) when external sandbox mode is requested.

## Rollout / safety

- Only reachable behind explicit host opt-in (SEAM-2).

