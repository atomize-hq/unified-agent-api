# Threading — Universal Agent API session semantics (ADR-0015 + ADR-0017)

This section makes coupling explicit: contracts/interfaces, dependency edges, critical paths, and conflict-safe workstreams.

## Contract registry

- **Contract ID**: `SA-C01 typed id accessor helpers`
  - **Type**: API (library helpers)
  - **Owner seam**: SEAM-1
  - **Consumers (seams)**: SEAM-2, `crates/wrapper_events` (and any future session-id consumers)
  - **Definition**:
    - `codex::ThreadEvent::thread_id() -> Option<&str>`
    - `claude_code::ClaudeStreamJsonEvent::session_id() -> Option<&str>`
  - **Versioning/compat**: additive; keep return types as `Option<&str>` so absence remains valid.

- **Contract ID**: `SA-C02 session handle facet (handle.v1)`
  - **Type**: schema/event
  - **Owner seam**: SEAM-2
  - **Consumers (seams)**: SEAM-3 (resume-by-id UX), external orchestrators
  - **Definition**: When a backend advertises `agent_api.session.handle.v1`, it emits:
    - exactly one early `Status` event whose `data` is the handle facet, and
    - `completion.data` containing the handle facet when a completion is produced and the id is known,
    per `docs/specs/universal-agent-api/event-envelope-schema-spec.md`.
  - **Additional pinned rules** (restated for pack/test determinism):
    - `session.id` MUST be non-empty after trimming (whitespace-only ids are treated as “not known” and MUST NOT be emitted).
    - Oversize ids (`len(session.id) > 1024` bytes) MUST be omitted (MUST NOT truncate).
  - **Versioning/compat**: stable `schema` string; facet-level `session.id` is opaque and backend-defined.

- **Contract ID**: `SA-C03 resume extension key (resume.v1)`
  - **Type**: config/schema (core extension key)
  - **Owner seam**: SEAM-3
  - **Consumers (seams)**: external orchestrators
  - **Definition**: `agent_api.session.resume.v1` object with selector `"last"` or `"id"` (closed schema), validated pre-spawn and mapped to backend resume surfaces per `docs/specs/universal-agent-api/extensions-spec.md`.
  - **Versioning/compat**: closed `.v1` schema; new semantics require a new versioned key.

- **Contract ID**: `SA-C04 fork extension key (fork.v1)`
  - **Type**: config/schema (core extension key)
  - **Owner seam**: SEAM-4
  - **Consumers (seams)**: external orchestrators
  - **Definition**: `agent_api.session.fork.v1` object with selector `"last"` or `"id"` (closed schema), validated pre-spawn and mapped to backend fork surfaces per `docs/specs/universal-agent-api/extensions-spec.md`.
  - **Versioning/compat**: closed `.v1` schema; new semantics require a new versioned key.

- **Contract ID**: `SA-C05 codex streaming resume (control + env overrides)`
  - **Type**: API (wrapper/library surface)
  - **Owner seam**: SEAM-3
  - **Consumers (seams)**: SEAM-3 (Codex `agent_api` backend mapping)
  - **Definition**: `agent_api` MUST use a pinned, control-capable Codex wrapper entrypoint for
    `codex exec resume` that preserves the invariants needed by the Universal Agent API:
    - API shape (pinned):
      - `codex::CodexClient::stream_resume_with_env_overrides_control(request: codex::ResumeRequest, env_overrides: &BTreeMap<String, String>) -> Result<codex::ExecStreamControl, codex::ExecStreamError>`
      - `ExecStreamControl.termination` MUST always be present for this entrypoint.
    - Spawn + prompt plumbing (pinned for `agent_api.session.resume.v1`):
      - argv MUST be a streaming resume invocation of the form:
        - selector `"last"` → `codex exec --json resume --last -`
        - selector `"id"` → `codex exec --json resume <ID> -`
      - Wrapper streaming/default flags (e.g., `--skip-git-repo-check`, `--color <MODE>`, `--output-last-message <PATH>`) MUST be present for streaming resume; tests MUST NOT treat the forms above as the complete argv (see `docs/specs/codex-wrapper-coverage-scenarios-v1.md`, Scenarios 2–3).
      - stdin MUST receive the follow-up prompt (newline-terminated) and then be closed.
    - Env overrides (pinned):
      - `AgentWrapperRunRequest.env` MUST be applied as per-run env overrides on top of backend config env (request keys win; owned by `docs/specs/universal-agent-api/contract.md`).
    - Termination + timeout semantics (pinned):
      - MUST satisfy `docs/specs/codex-streaming-exec-contract.md` and the universal cancellation semantics in `docs/specs/universal-agent-api/run-protocol-spec.md`.
  - **Versioning/compat**: internal; keep behavior parity with exec where possible.

- **Contract ID**: `SA-C06 codex app-server fork RPC surface`
  - **Type**: API (JSON-RPC method contract + notifications)
  - **Owner seam**: SEAM-4
  - **Consumers (seams)**: SEAM-4 (Codex fork mapping in `agent_api`)
  - **Definition**: A headless fork flow implemented via `codex app-server` stdio JSON-RPC, with
    the concrete wire contract defined in `docs/specs/codex-app-server-jsonrpc-contract.md`.
    - Required methods (pinned):
      - `thread/list` (selector `"last"` only; filtered by effective working directory) using the pinned paging + deterministic selection algorithm.
      - `thread/fork` (fork source thread → new thread id).
      - `turn/start` (follow-up prompt on the forked thread) using the pinned prompt → `input[]` mapping.
    - Notifications (pinned minimum):
      - MUST be mapped into `AgentWrapperEvent` kinds/fields per the app-server contract while preserving Universal Agent API safety/bounds posture (no raw backend payload embedding in `data`).
    - Cancellation (pinned):
      - MUST use `$/cancelRequest` for in-flight `turn/start` requests and enforce universal cancellation precedence (`run-protocol-spec.md`).
  - **Versioning/compat**: pinned to the app-server wire schema emitted by the Codex CLI under test; this repo’s tests treat the contract doc as authoritative.

## Dependency graph (text)

- `SEAM-1 blocks SEAM-2` because: handle facet emission should source ids via typed accessors to avoid duplicated match logic in multiple crates.
- `SEAM-2 + SEAM-3 jointly unblock “resume-by-id UX”` because: callers need both (a) a stable id discovery surface (handle facet) and (b) a way to resume by id (selector `"id"`).
- `SEAM-4 (Codex fork) is blocked by SA-C06` because: a headless fork requires a pinned app-server RPC surface (`thread/fork` plus any required discovery) before `agent_api` can integrate it safely.

## Critical path

- Session handle facet (both backends): `SEAM-1 (accessors)` → `SEAM-2 (handle emission)`
- Resume-by-id UX end-to-end: `max(SEAM-3 (resume by id), SEAM-1 → SEAM-2 (id discovery))`
- Fork:
  - Claude can ship early once SEAM-4 Claude mapping + tests land.
  - Codex fork is gated by the app-server contract-definition work inside SEAM-4.

## Parallelization notes / conflict-safe workstreams

Because SEAM-2/3/4 all touch `crates/agent_api/src/backends/{codex,claude_code}.rs`, the safest parallelization is by **backend + crate** rather than by seam alone.

- **WS-A (Wrapper accessors + wrapper_events adoption)**: SEAM-1; touch surface:
  - `crates/codex/src/events.rs`
  - `crates/claude_code/src/stream_json.rs`
  - `crates/wrapper_events/src/codex_adapter.rs`
  - `crates/wrapper_events/src/claude_code_adapter.rs`
- **WS-B (Claude session semantics)**: Claude portions of SEAM-2/3/4; touch surface:
  - `crates/agent_api/src/backends/claude_code.rs`
  - `crates/agent_api/tests/**`
- **WS-C (Codex resume + handle)**: Codex portions of SEAM-2/3 plus SA-C05; touch surface:
  - `crates/agent_api/src/backends/codex.rs`
  - `crates/agent_api/src/backends/codex/mapping.rs`
  - `crates/codex/src/exec.rs`
  - `crates/codex/src/exec/streaming.rs`
  - `crates/agent_api/tests/**`
- **WS-D (Codex fork via app-server)**: Codex portion of SEAM-4 plus SA-C06; touch surface:
  - `crates/codex/src/mcp/protocol.rs`
  - `crates/codex/src/mcp/client.rs`
  - `crates/codex/src/mcp/tests_core/**`
  - `crates/agent_api/src/backends/codex.rs` (minimal wiring preferred; isolate logic in a new module if possible)
  - `crates/agent_api/tests/**`
- **WS-INT (Integration)**: lands WS-A, then merges WS-B/WS-C, then WS-D; runs the full suite and verifies behavior matches the canonical specs. After advertising new session capability ids, regenerate and commit the capability matrix (`cargo run -p xtask -- capability-matrix` → `docs/specs/universal-agent-api/capability-matrix.md`).
