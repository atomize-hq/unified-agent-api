### S3b — SA-C05 tests: resume env overrides + termination

- **User/system value**: SA-C05 behavior is pinned by fast, deterministic `crates/codex` tests so downstream `agent_api` resume semantics can rely on env-override and termination invariants.
- **Scope (in/out)**:
  - In:
    - Add regression tests for streaming resume control:
      - env overrides apply without mutating parent env,
      - `termination.request_termination()` closes the resume event stream without polling completion.
  - Out:
    - `agent_api` Codex backend mapping + translation (see `S3c`/`S3d`).
    - `agent_api` fake-binary integration tests + capability advertisement (see `S3e`).
- **Acceptance criteria**:
  - Tests use existing fake Codex harness (`write_fake_codex`) and are deterministic.
  - Tests fail loudly on env merge or termination drift for the SA-C05 resume entrypoint.
- **Dependencies**:
  - `S3a` (SA-C05 entrypoint exists).
  - Normative: `docs/specs/codex-streaming-exec-contract.md` (termination semantics).
  - Normative: `docs/specs/universal-agent-api/contract.md` (env merge precedence).
- **Verification**:
  - `cargo test -p codex`
- **Rollout/safety**:
  - Tests only; no behavioral changes outside the Codex crate.

#### S3.T2 — Codex crate tests: env overrides + termination for streaming resume control

- **Outcome**: Regression tests that pin SA-C05 behavior in `crates/codex`.
- **Inputs/outputs**:
  - Inputs: fake Codex scripts (existing `write_fake_codex` harness).
  - Outputs:
    - A test proving env overrides apply to streaming resume without mutating parent env.
    - A test proving `termination.request_termination()` closes the resume event stream without polling completion.
  - Files:
    - `crates/codex/src/tests/**` (new tests near existing `stream_resume_timeout.rs` and `stream_exec_termination.rs`)
- **Verification**:
  - `cargo test -p codex`

Checklist:
- Implement:
  - Add a resume-env-overrides test mirroring `stream_exec_env_overrides.rs`.
  - Add a resume-termination test mirroring `stream_exec_termination.rs`.
- Test:
  - `cargo test -p codex`

