### S3c — `agent_api` Codex backend: resume.v1 validation + mapping via SA-C05

- **User/system value**: The `agent_api` Codex backend can interpret `extensions["agent_api.session.resume.v1"]` (closed schema), map selectors deterministically to Codex resume spawn semantics, and wire a termination handle for cancellation.
- **Scope (in/out)**:
  - In:
    - Parse/validate `extensions["agent_api.session.resume.v1"]` via the shared helper from `S1` (closed schema).
    - Map `"last"` / `"id"` to the Codex resume selector and ensure prompt is stdin-based (`-`) per Scenario 3.
    - When `agent_api.exec.non_interactive == true` (default), set approval policy to “never” to avoid interactive prompts.
    - Use SA-C05 (`stream_resume_with_env_overrides_control`) so per-run env overrides + termination are available.
  - Out:
    - Selection failure translation + terminal `Error` event rule (see `S3d`).
    - Fake-binary integration tests + capability advertisement (see `S3e`).
- **Acceptance criteria**:
  - Implementation uses the shared resume selector parser (no duplicated JSON parsing).
  - When the `resume.v1` key is enabled in `supported_extension_keys()` (done in `S3e` alongside tests), invalid schemas fail pre-spawn with `AgentWrapperError::InvalidRequest`.
  - Spawn wiring uses the pinned resume argv/stdin semantics from `docs/specs/codex-wrapper-coverage-scenarios-v1.md` (Scenario 3).
  - A termination handle is installed so `run_control` cancellation can request termination.
- **Dependencies**:
  - `S1` shared resume selector parser.
  - `S3a` SA-C05 entrypoint in `crates/codex`.
  - Normative: `docs/specs/unified-agent-api/extensions-spec.md` (schema + staged-rollout precedence).
  - Normative: `docs/specs/codex-wrapper-coverage-scenarios-v1.md` (Scenario 3: argv + stdin prompt plumbing).
- **Verification**:
  - `cargo test -p agent_api --features codex`
- **Rollout/safety**:
  - Do not advertise `"agent_api.session.resume.v1"` in capabilities until `S3e` integration tests pass.

#### S3.T3 — `agent_api` Codex backend: validate + map `resume.v1` and use SA-C05 entrypoint

- **Outcome**: `agent_api` Codex backend supports `agent_api.session.resume.v1` and maps it through the SA-C05 wrapper entrypoint while preserving universal invariants.
- **Inputs/outputs**:
  - Inputs: `AgentWrapperRunRequest.extensions["agent_api.session.resume.v1"]`, prompt, env, non-interactive policy.
  - Outputs:
    - Resume selector mapped to a Codex resume request (`--last` or `ID`) with prompt on stdin via `-`.
    - Per-run env overrides applied via SA-C05.
    - Termination handle installed so `run_control` cancellation can request termination.
  - Files:
    - `crates/agent_api/src/backends/codex.rs`
- **Implementation notes**:
  - Use the shared parser from `S1` (no duplicated JSON parsing).
  - Use SA-C05 entrypoint rather than existing `stream_resume` so env overrides + termination are available.
  - Preserve non-interactive policy behavior: approval policy is “never” when `agent_api.exec.non_interactive == true`.
- **Acceptance criteria**:
  - Resume mapping uses the pinned argv subsequence from `docs/specs/codex-wrapper-coverage-scenarios-v1.md` Scenario 3.
  - Validation errors are `InvalidRequest` and occur pre-spawn (once the key is enabled; see `S3e`).

Checklist:
- Implement:
  - Parse resume selector object into a typed resume policy.
  - In spawn, call `stream_resume_with_env_overrides_control` and wire termination handle into `TerminationState`.
- Test:
  - `cargo test -p agent_api --features codex`

