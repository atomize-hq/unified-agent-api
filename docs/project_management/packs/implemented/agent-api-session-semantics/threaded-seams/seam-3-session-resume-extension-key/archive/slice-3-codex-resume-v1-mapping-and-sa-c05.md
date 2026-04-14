### S3 — Codex `agent_api.session.resume.v1` mapping + SA-C05 (control + env overrides)

- **User/system value**: Codex resume-by-last / resume-by-id is implemented with the Unified Agent API’s pinned invariants: closed-schema validation, deterministic CLI mapping, per-run env overrides, and a best-effort termination handle required for `run_control` cancellation semantics.
- **Scope (in/out)**:
  - In:
    - Implement SA-C05 in `crates/codex`:
      - `CodexClient::stream_resume_with_env_overrides_control(request: ResumeRequest, env_overrides: &BTreeMap<String, String>) -> ExecStreamControl`
      - `ExecStreamControl.termination` is always present for this entrypoint.
      - Apply per-run env overrides per universal contract (request keys win over backend defaults).
      - Pinned argv + stdin prompt plumbing for `codex exec --json resume` (Scenario 3).
    - Update `agent_api` Codex backend to support `agent_api.session.resume.v1`:
      - allowlist the extension key via `supported_extension_keys()`,
      - validate the closed-schema selector object (shared `S1` helper),
      - map `"last"`/`"id"` to the Codex resume selector and ensure prompt is stdin-based (`-`),
      - when `agent_api.exec.non_interactive == true`, set approval policy to “never” to avoid interactive prompts.
    - Implement selection failure translation per `extensions-spec.md` pinned messages and terminal `Error` event rule.
    - Add tests in both crates (`codex` + `agent_api`) pinning SA-C05 and SA-C03 Codex conformance.
  - Out:
    - Fork semantics (`agent_api.session.fork.v1`) (SEAM-4).
    - Handle facet emission (`agent_api.session.handle.v1`) (SEAM-2).
- **Acceptance criteria**:
  - SA-C05 entrypoint exists, is used by `agent_api`, and:
    - applies env overrides without mutating parent process env,
    - provides a termination handle whose request closes events without requiring polling completion,
    - preserves the pinned resume argv subsequences and stdin prompt plumbing.
  - When Codex advertises `agent_api.session.resume.v1`:
    - invalid schemas fail pre-spawn with `InvalidRequest`,
    - selection failures surface as `Backend("no session found")` / `Backend("session not found")` and emit exactly one terminal `Error` event when a stream exists.
- **Dependencies**:
  - `S1` shared resume selector parser.
  - Normative: `docs/specs/unified-agent-api/extensions-spec.md` (schema + selection failure + contradiction rules).
  - Normative: `docs/specs/codex-wrapper-coverage-scenarios-v1.md` (Scenario 3: argv + stdin prompt plumbing).
  - Normative: `docs/specs/codex-streaming-exec-contract.md` (termination + timeout semantics).
  - Normative: `docs/specs/unified-agent-api/contract.md` (env merge precedence + effective working dir).
- **Verification**:
  - `cargo test -p codex`
  - `cargo test -p agent_api --features codex`
- **Rollout/safety**:
  - Capability-gated: do not advertise `agent_api.session.resume.v1` for Codex until SA-C05 + mapping + tests are merged.

#### S3.T1 — Implement SA-C05: `CodexClient::stream_resume_with_env_overrides_control(...)`

- **Outcome**: A control-capable, env-override-capable streaming resume entrypoint that `agent_api` can use to preserve cancellation + env invariants.
- **Inputs/outputs**:
  - Inputs:
    - `ResumeRequest` (selector + prompt plumbing),
    - env overrides map (`BTreeMap<String, String>`).
  - Outputs:
    - New `CodexClient` method returning `ExecStreamControl`,
    - Underlying streaming wiring for `codex exec --json resume ...` that:
      - applies env overrides, and
      - wires `ExecTerminationHandle` like `stream_exec_with_env_overrides_control`.
  - Files:
    - `crates/codex/src/exec.rs`
    - `crates/codex/src/exec/streaming.rs`
- **Implementation notes**:
  - Prefer factoring by extracting a shared helper with the existing `stream_resume` to avoid drift (but keep the public API pinned by `threading.md`).
  - Ensure prompt plumbing for `agent_api` use:
    - always append `-` and write the prompt to stdin (newline-terminated) then close stdin.
  - Ensure `termination` is always present for this entrypoint (even if `ResumeRequest.prompt` is absent in other call sites).
- **Acceptance criteria**:
  - Signature matches `threading.md` exactly.
  - Termination semantics match `docs/specs/codex-streaming-exec-contract.md`.

Checklist:
- Implement:
  - Add `stream_resume_with_env_overrides_control` on `CodexClient`.
  - Add internal `stream_resume_with_env_overrides_control(...)` wiring in `exec/streaming.rs`.
- Test:
  - `cargo test -p codex`
- Validate:
  - `ExecStreamControl.termination` is returned and functional for resume.

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
  - Add `EXT_SESSION_RESUME_V1` to `supported_extension_keys()` only when fully implemented and tested.
  - Use the shared parser from `S1` (no duplicated JSON parsing).
  - Use SA-C05 entrypoint rather than existing `stream_resume` so env overrides + termination are available.
  - Preserve non-interactive policy behavior: approval policy is “never” when `agent_api.exec.non_interactive == true`.
- **Acceptance criteria**:
  - Resume mapping uses the pinned argv subsequence from `docs/specs/codex-wrapper-coverage-scenarios-v1.md` Scenario 3.
  - Validation errors are `InvalidRequest` and occur pre-spawn.

Checklist:
- Implement:
  - Parse resume selector object into a typed resume policy.
  - In spawn, call `stream_resume_with_env_overrides_control` and wire termination handle into `TerminationState`.
- Test:
  - `cargo test -p agent_api --features codex`

#### S3.T4 — Codex selection failure translation (pinned messages + terminal `Error` event rule)

- **Outcome**: Selection failures for resume surface as pinned safe `Backend` errors and emit exactly one terminal `Error` event when a stream exists.
- **Inputs/outputs**:
  - Inputs: resume selector (`last` vs `id`) and the Codex crate’s typed outcomes.
  - Outputs:
    - Completion resolves to `Err(AgentWrapperError::Backend { message: <pinned> })` for selection failures.
    - Event stream emits exactly one terminal `Error` event with `message == <pinned>` before closing.
  - Files:
    - `crates/agent_api/src/backends/codex.rs`
- **Implementation notes**:
  - Do not embed raw Codex stderr/stdout/JSONL lines in the surfaced messages.
  - Prefer classifying “not found” via a stable typed signal (e.g., a known error event type or wrapper error kind), falling back to a safe generic backend error message for other failures.
- **Acceptance criteria**:
  - Messages are exactly:
    - `"no session found"` for selector `"last"`,
    - `"session not found"` for selector `"id"`.

Checklist:
- Implement:
  - Add a safe “not found” classification path and map it to pinned messages.
  - Ensure the terminal error event is emitted exactly once in the stream.
- Test:
  - `cargo test -p agent_api --features codex`

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
  - Add the capability id after `S3.T1`–`S3.T5` land.
- Test:
  - `cargo test -p agent_api --features codex`

