### S3e — `agent_api` integration tests (fake Codex) + enable capability for `resume.v1`

- **User/system value**: Deterministic integration tests pin Codex resume argv/stdin plumbing and selection-failure semantics, then Codex can safely advertise `"agent_api.session.resume.v1"`.
- **Scope (in/out)**:
  - In:
    - Extend the fake Codex binary harness to validate `exec --json resume ...` argv and stdin prompt plumbing.
    - Add `agent_api` integration tests for selector `"last"` / `"id"` mapping and pinned not-found failures.
    - Enable the `resume.v1` extension key gate and advertise the capability id only after the above tests pass.
  - Out:
    - None (this is the rollout gate for Codex resume support in `agent_api`).
- **Acceptance criteria**:
  - Integration tests assert:
    - argv contains `exec --json resume --last -` (or `exec --json resume <ID> -`) as an ordered subsequence,
    - prompt is written to stdin (newline-terminated) and stdin is closed,
    - selection failure yields pinned backend messages and the terminal `Error` event rule.
  - After tests land:
    - Codex backend `supported_extension_keys()` includes `"agent_api.session.resume.v1"`,
    - Codex `capabilities().ids` includes `"agent_api.session.resume.v1"`.
- **Dependencies**:
  - `S3a`/`S3b` (SA-C05 exists and is tested in `crates/codex`).
  - `S3c`/`S3d` (mapping + failure translation are implemented in `agent_api`).
  - Normative: `docs/specs/universal-agent-api/extensions-spec.md` (pinned messages + terminal error event rule).
  - Normative: `docs/specs/codex-wrapper-coverage-scenarios-v1.md` (Scenario 3: argv + stdin prompt plumbing).
- **Verification**:
  - `cargo test -p agent_api --features codex`
- **Rollout/safety**:
  - Capability-gated: do not advertise `"agent_api.session.resume.v1"` until this sub-slice’s integration tests are passing.

#### S3.T5 — Pin integration tests with fake Codex binary (argv + stdin prompt + failure behavior)

- **Outcome**: Deterministic tests pinning the `agent_api` Codex backend’s resume mapping and failure semantics without using the real `codex` CLI.
- **Inputs/outputs**:
  - Inputs:
    - fake Codex binary scenarios (extend existing harness or add a dedicated resume harness),
    - `AgentWrapperRunRequest` with `resume.v1` selectors.
  - Outputs:
    - Tests that assert:
      - argv contains `exec --json resume --last -` (or `exec --json resume <ID> -`) as an ordered subsequence,
      - prompt is written to stdin (newline-terminated) and stdin is closed,
      - selection failure yields pinned backend messages and terminal `Error` event rule.
  - Files:
    - `crates/agent_api/src/bin/fake_codex_stream_exec_scenarios_agent_api.rs` (extend to validate `resume` argv + stdin where feasible) and/or
    - `crates/agent_api/src/bin/fake_codex_stream_json_agent_api.rs` (if a separate JSONL-focused harness is a better fit),
    - `crates/agent_api/tests/**` (new focused test file)
- **Verification**:
  - `cargo test -p agent_api --features codex`

Checklist:
- Implement:
  - Extend/add fake binary support for `exec resume` argv + stdin assertions.
  - Add tests for selector `"last"`/`"id"` mapping and not-found failures.
- Test:
  - `cargo test -p agent_api --features codex`

#### S3.T6 — Advertise `agent_api.session.resume.v1` capability id (Codex)

- **Outcome**: Codex backend capabilities include `"agent_api.session.resume.v1"` only after SA-C05 + mapping + tests land.
- **Inputs/outputs**:
  - Output: Codex `capabilities().ids` includes `"agent_api.session.resume.v1"`.
  - Files:
    - `crates/agent_api/src/backends/codex.rs`
- **Acceptance criteria**:
  - Capability advertisement matches behavior: no “advertise without SA-C05 + mapping + tests”.

Checklist:
- Implement:
  - Add `"agent_api.session.resume.v1"` to the Codex backend allowlist (`supported_extension_keys()`) and to `capabilities().ids` after `S3.T5` tests are in place and passing.
- Test:
  - `cargo test -p agent_api --features codex`

