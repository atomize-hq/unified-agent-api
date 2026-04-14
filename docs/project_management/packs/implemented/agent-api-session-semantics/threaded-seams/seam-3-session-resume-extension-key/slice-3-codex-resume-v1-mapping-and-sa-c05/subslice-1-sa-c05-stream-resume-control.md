### S3a — SA-C05: Codex streaming resume control (env overrides + termination)

- **User/system value**: `agent_api` can call a control-capable Codex resume stream that honors per-run env overrides and always provides a termination handle for `run_control` cancellation semantics.
- **Scope (in/out)**:
  - In:
    - Add `CodexClient::stream_resume_with_env_overrides_control(request: ResumeRequest, env_overrides: &BTreeMap<String, String>) -> ExecStreamControl`.
    - Wire streaming spawn for `codex exec --json resume ...` with pinned argv + stdin prompt plumbing (Scenario 3).
    - Ensure `ExecStreamControl.termination` is always present for this entrypoint.
    - Apply env overrides per `docs/specs/unified-agent-api/contract.md` (request keys win over backend defaults).
  - Out:
    - Codex crate regression tests (see `S3b`).
    - `agent_api` Codex backend wiring (see `S3c`/`S3d`).
    - `agent_api` integration tests + capability advertisement (see `S3e`).
- **Acceptance criteria**:
  - Signature matches `threading.md` exactly.
  - Resume spawn uses stdin prompt plumbing (`-` + newline-terminated prompt written to stdin, then stdin closed).
  - Env overrides are applied without mutating the parent process environment.
  - Returned `ExecStreamControl` always includes a usable `termination` handle.
- **Dependencies**:
  - Normative: `docs/specs/codex-wrapper-coverage-scenarios-v1.md` (Scenario 3: argv + stdin prompt plumbing).
  - Normative: `docs/specs/codex-streaming-exec-contract.md` (termination + timeout semantics).
  - Normative: `docs/specs/unified-agent-api/contract.md` (env merge precedence).
- **Verification**:
  - `cargo test -p codex`
- **Rollout/safety**:
  - Additive API surface only; no `agent_api` capability advertisement occurs in this sub-slice.

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

