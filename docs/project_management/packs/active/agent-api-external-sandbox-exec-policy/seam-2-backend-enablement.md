# SEAM-2 — Backend enablement + capability advertising

- **Name**: built-in backend opt-in for `agent_api.exec.external_sandbox.v1`
- **Type**: platform (host configuration) + risk (dangerous capability)
- **Goal / user value**: ensure externally sandboxed hosts can opt-in to this capability, while
  built-in backends remain safe-by-default and do not advertise it automatically.

## Scope

- In:
  - Add an explicit backend config toggle (default `false`) that controls:
    - whether `agent_api.exec.external_sandbox.v1` appears in `capabilities().ids`, and
    - whether the key is accepted by the harness allowlist (`supported_extension_keys()`).
  - Apply to both built-in backends:
    - Codex (`crates/agent_api/src/backends/codex.rs`)
    - Claude Code (`crates/agent_api/src/backends/claude_code.rs`)
- Out:
  - Implementing the actual CLI mapping (SEAM-3/4).

## Primary interfaces (contracts)

- **Backend config** (host-controlled, not a per-run extension key):
  - A new boolean toggle such as `allow_external_sandbox_exec` (exact name TBD) that defaults to
    `false`.
  - When `false`:
    - capability id `agent_api.exec.external_sandbox.v1` is absent from `capabilities()`, and
    - attempts to send the extension key fail closed as `UnsupportedCapability`.
  - When `true`:
    - the backend advertises the capability id, and
    - the key becomes eligible for validation/mapping (still fail-closed on invalid value).

## Key invariants / rules

- Capability advertising must remain **off by default**.
- The harness allowlist and the capability set must remain aligned (no “advertise but reject”, or
  “accept but don’t advertise”).

## Dependencies

- Blocks: SEAM-3/4/5 (ensures the mapping is reachable only behind opt-in).
- Blocked by: SEAM-1 (key semantics).

## Touch surface

- `crates/agent_api/src/backends/codex.rs`
- `crates/agent_api/src/backends/claude_code.rs`

## Verification

- Unit tests:
  - default backend instances do **not** contain `agent_api.exec.external_sandbox.v1` in
    `capabilities()`.
  - when the opt-in toggle is enabled, the capability appears and the key is accepted for further
    validation.

## Risks / unknowns

- Choosing the most ergonomic and least footgun-prone configuration surface for hosts (config field
  vs feature flag vs constructor variant).

## Rollout / safety

- This is a "double opt-in" design:
  1) host enables the backend capability explicitly, and
  2) host sets the per-run extension key explicitly.

