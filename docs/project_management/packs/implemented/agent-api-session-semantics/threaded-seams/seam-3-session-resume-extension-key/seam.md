# SEAM-3 — Session resume extension key (`agent_api.session.resume.v1`) (uaa-0004 + uaa-0005)

## Seam Brief (Restated)

- **Seam ID**: SEAM-3
- **Name**: Universal resume semantics via `AgentWrapperRunRequest.extensions["agent_api.session.resume.v1"]`
- **Goal / value**: Let orchestrators resume the most recent session (“last”) or a specific session/thread (“id”) across built-in backends with deterministic, fail-closed validation and capability-gated rollout.
- **Type**: capability
- **Scope**
  - In:
    - Implement `agent_api.session.resume.v1` (SA-C03) end-to-end for built-in backends that can support it:
      - closed-schema JSON validation and mutual exclusivity with `agent_api.session.fork.v1` (per `docs/specs/universal-agent-api/extensions-spec.md`),
      - deterministic CLI spawn mapping per backend-owned mapping contracts,
      - selection-failure translation to pinned safe messages (`"no session found"` / `"session not found"`),
      - capability advertisement only after behavior + tests land.
    - Implement SA-C05 (Codex wrapper): a control-capable, env-override-capable streaming resume entrypoint used by `agent_api`:
      - `codex::CodexClient::stream_resume_with_env_overrides_control(...) -> ExecStreamControl`
      - `ExecStreamControl.termination` is always present for this entrypoint.
    - Pin regression tests for:
      - schema/type errors and closed-schema enforcement,
      - staged-rollout precedence (R0): unsupported key → `UnsupportedCapability` wins over contradiction rules,
      - CLI argv + prompt plumbing (fake-binary where appropriate),
      - selection-failure behavior and the terminal `Error` event rule.
  - Out:
    - Fork semantics (`agent_api.session.fork.v1`) (SEAM-4).
    - Session handle facet emission (`agent_api.session.handle.v1`) (SEAM-2).
- **Touch surface**:
  - `crates/agent_api/src/backends/claude_code.rs`
  - `crates/agent_api/src/backends/codex.rs`
  - `crates/agent_api/tests/**`
  - `crates/agent_api/src/bin/fake_claude_stream_json_agent_api.rs` (mapping/selection-failure harness)
  - `crates/agent_api/src/bin/fake_codex_stream_exec_scenarios_agent_api.rs` (mapping/selection-failure harness)
  - `crates/codex/src/exec.rs` (SA-C05 API surface)
  - `crates/codex/src/exec/streaming.rs` (SA-C05 spawn wiring for `codex exec --json resume`)
  - `crates/codex/src/tests/**` (SA-C05 tests)
- **Verification**:
  - For each backend that advertises `agent_api.session.resume.v1`:
    - selector `"last"` maps to the pinned CLI form and produces a live event stream + completion on success,
    - selector `"id"` maps to the pinned CLI form and produces a live event stream + completion on success,
    - invalid schemas fail pre-spawn with `AgentWrapperError::InvalidRequest`,
    - selection failures follow `extensions-spec.md` pinned messages and event emission rule.
- **Threading constraints**
  - Upstream blockers: none (but Codex support is internally gated by SA-C05 landing first).
  - Downstream blocked seams: none (but SEAM-2 + SEAM-3 jointly unblock the full “resume-by-id UX”).
  - Contracts produced (owned):
    - `SA-C03 resume extension key (resume.v1)`
    - `SA-C05 codex streaming resume (control + env overrides)`
  - Contracts consumed:
    - Normative: `docs/specs/universal-agent-api/extensions-spec.md` (schema + precedence + failure semantics)
    - Normative: `docs/specs/claude-code-session-mapping-contract.md` (Claude argv mapping + safe error translation)
    - Normative: `docs/specs/codex-wrapper-coverage-scenarios-v1.md` (Scenario 3: argv + stdin plumbing)
    - Normative: `docs/specs/codex-streaming-exec-contract.md` (termination + timeout semantics)
    - Normative: `docs/specs/universal-agent-api/contract.md` (env merge precedence + effective working dir)
    - Normative: `docs/specs/universal-agent-api/run-protocol-spec.md` (validation timing and fail-closed rules)

## Slice index

- `S1` → `slice-1-resume-v1-validation-and-precedence.md`: Shared resume selector parser + closed-schema validation + precedence pinning tests.
- `S2` → `slice-2-claude-resume-v1-mapping.md`: Claude Code backend mapping + selection-failure translation + tests + capability advertisement.
- `S3` → `slice-3-codex-resume-v1-mapping-and-sa-c05.md`: Codex wrapper SA-C05 + Codex backend mapping + selection-failure translation + tests + capability advertisement.

## Threading Alignment (mandatory)

- **Contracts produced (owned)**:
  - `SA-C03 resume extension key (resume.v1)`:
    - Definition (per `threading.md` + `extensions-spec.md`): closed-schema object `{ selector: "last" | "id", id?: string }` validated pre-spawn; maps to backend resume surfaces; selection failures map to pinned safe backend messages and obey the terminal `Error` event rule when a stream exists.
    - Where it lives (implementation):
      - Shared validation helper(s): implemented in `agent_api` (see `S1`).
      - Claude mapping: `crates/agent_api/src/backends/claude_code.rs` (`S2`).
      - Codex mapping: `crates/agent_api/src/backends/codex.rs` (`S3`).
    - Produced by:
      - `S1` publishes shared parsing/validation + precedence tests.
      - `S2` completes Claude backend conformance for SA-C03 and advertises the capability id.
      - `S3` completes Codex backend conformance for SA-C03 and advertises the capability id (after SA-C05 exists).
  - `SA-C05 codex streaming resume (control + env overrides)`:
    - Definition (per `threading.md`): `codex::CodexClient::stream_resume_with_env_overrides_control(request: codex::ResumeRequest, env_overrides: &BTreeMap<String, String>) -> Result<codex::ExecStreamControl, codex::ExecStreamError>` with `termination` always present, plus pinned argv/stdin/env rules.
    - Where it lives:
      - `crates/codex/src/exec.rs`
      - `crates/codex/src/exec/streaming.rs`
    - Produced by:
      - `S3` (lands before advertising Codex support for `agent_api.session.resume.v1` in `agent_api`).
- **Contracts consumed**:
  - `docs/specs/universal-agent-api/extensions-spec.md`:
    - Resume schema + closed-schema rule.
    - R0 precedence: unsupported keys fail with `UnsupportedCapability` before value validation or mutual exclusivity.
    - Pinned selection-failure messages and terminal `Error` event rule when a stream exists.
  - `docs/specs/claude-code-session-mapping-contract.md`:
    - Pinned Claude argv subsequences for selector `"last"` / `"id"`.
    - Safe error translation requirements.
  - `docs/specs/codex-wrapper-coverage-scenarios-v1.md` (Scenario 3) + `docs/specs/codex-streaming-exec-contract.md`:
    - Pinned Codex argv subsequence and stdin prompt plumbing.
    - Termination/timeout semantics for streaming control entrypoints.
  - `docs/specs/universal-agent-api/contract.md`:
    - Env override merge rule (request keys win).
    - Effective working directory scoping for selector `"last"`.
- **Dependency edges honored**:
  - `SEAM-2 + SEAM-3 jointly unblock “resume-by-id UX”`: this seam ships resume-by-id behavior; SEAM-2 ships id discovery. This plan does not embed SEAM-2 logic, but keeps semantics compatible for orchestrators.
- **Parallelization notes**:
  - What can proceed now:
    - `S1` immediately (shared helper + tests; low conflict surface).
    - `S2` in WS-B (Claude backend + fake Claude harness + tests).
    - `S3` in WS-C (Codex wrapper SA-C05 + Codex backend + fake Codex harness + tests).
  - What must wait:
    - Advertising Codex `agent_api.session.resume.v1` in `agent_api` must wait until SA-C05 is merged and tested (pinned by `threading.md`).
    - Full end-to-end “discover id then resume-by-id” UX tests can land later in WS-INT once SEAM-2 is merged (out-of-scope for this seam’s task list).

