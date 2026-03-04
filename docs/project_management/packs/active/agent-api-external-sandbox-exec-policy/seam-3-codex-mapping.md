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
  - Map `extensions["agent_api.exec.external_sandbox.v1"] == true` using exactly one **canonical**
    mechanism (pinned):
    - Codex exec/resume flows: call
      `codex::CodexClientBuilder::dangerously_bypass_approvals_and_sandbox(true)`.
    - Codex fork flow (app-server JSON-RPC): set `approval_policy="never"` and
      `sandbox="danger-full-access"` on all applicable RPC surfaces (no spawn+retry loop).
  - Ensure mapping applies consistently across every Codex run entrypoint:
    - exec (`spawn_exec_or_resume_flow` with `resume=None`)
    - resume (`spawn_exec_or_resume_flow` with `resume=Some(...)`)
    - fork (`spawn_fork_v1_flow`)
- Out:
  - Changes to Codex wrapper crate unless required (assumed already supported).

## Primary interfaces (contracts)

- **Input**: `extensions["agent_api.exec.external_sandbox.v1"] == true` (when capability is enabled)
- **Output**: Codex CLI invocation includes the dangerous bypass override and remains non-interactive.

## Key invariants / rules

- MUST NOT hang on prompts.
- MUST be validated before spawn.
- MUST fail before spawn with `AgentWrapperError::InvalidRequest` on explicit contradiction with
  `agent_api.exec.non_interactive == false`.
- MUST reject ambiguous exec-policy combinations:
  - when `agent_api.exec.external_sandbox.v1 == true`, the request MUST NOT include any
    `backend.*.exec.*` keys (including `backend.codex.exec.approval_policy` and
    `backend.codex.exec.sandbox_mode`) per `docs/specs/universal-agent-api/extensions-spec.md`.
- Equivalent mapping definition (pinned; used by tests):
  - Exec/resume: argv MUST contain exactly one `--dangerously-bypass-approvals-and-sandbox`, and
    MUST NOT contain any of: `--full-auto`, `--ask-for-approval`, `--sandbox`.
  - Fork (app-server): `approval_policy` MUST resolve to `"never"` and `sandbox` MUST resolve to
    `"danger-full-access"` for the fork + turn-start surfaces.
- Unavailable mapping primitive behavior (pinned):
  - The backend MUST NOT attempt a fallback mapping (no spawn then retry with different flags).
  - If the installed Codex binary rejects the pinned flag or the app-server rejects the pinned
    sandbox/approval values, the backend MUST fail the run as `AgentWrapperError::Backend { message }`
    with a safe/redacted `message`.

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
  - forbidden combinations fail pre-spawn as `InvalidRequest`:
    - `external_sandbox=true` + `backend.codex.exec.approval_policy=*`
    - `external_sandbox=true` + `backend.codex.exec.sandbox_mode=*`
  - exec/resume mapping:
    - argv includes `--dangerously-bypass-approvals-and-sandbox`
    - argv excludes `--full-auto`, `--ask-for-approval`, `--sandbox`
  - fork mapping:
    - app-server RPC uses `approval_policy="never"` + `sandbox="danger-full-access"`.

## Risks / unknowns

- Codex binary/app-server version mismatch: if an installed upstream version rejects any pinned
  mapping primitive (flag / sandbox / approval), the backend MUST fail closed per the pinned
  “unavailable mapping primitive” behavior above (no fallback mapping).

## Rollout / safety

- Only reachable behind explicit host opt-in (SEAM-2).
