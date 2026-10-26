# Codex Streaming Exec Contract (v1)

Status: **Normative**  
Scope: live streaming via `codex::CodexClient::stream_exec*` / `stream_resume`

## Normative language

This document uses RFC 2119-style requirement keywords (`MUST`, `MUST NOT`, `SHOULD`).

## Purpose

Define a **zero-ambiguity** contract for the Codex crate’s *live streaming* APIs,
specifically around:

- when runtime driver tasks begin executing
- how `timeout` is interpreted for streaming handles
- how termination is requested and observed

This contract is intentionally scoped to the `codex` crate’s streaming runtime behavior. It does
not define cross-backend universal semantics; see `docs/specs/unified-agent-api/run-protocol-spec.md`
for the `agent_api` run lifecycle rules.

## `agent_api.config.model.v1` mapping (pinned)

The model-selection extension key is owned by:

- `docs/specs/unified-agent-api/extensions-spec.md`

When `extensions["agent_api.config.model.v1"]` is accepted for a Codex exec/resume run, the Codex
streaming wrapper MUST:

- emit exactly one `--model <trimmed-id>` pair,
- use the effective trimmed model id from the universal extension contract, and
- omit `--model` entirely when the key is absent.

Placement rules (pinned):

- The pair MUST appear after any wrapper-owned CLI overrides have been applied.
- The pair MUST appear before capability-guarded `--add-dir` emission.
- Resume runs MUST preserve the same effective model id that a fresh exec run would emit; the
  wrapper MUST NOT silently drop an accepted model id on `stream_resume`.

Verification linkage (pinned):

- The SEAM-3 Codex mapping verification plan MUST include an argv-layout assertion proving these
  placement rules end to end:
  wrapper-owned CLI overrides first, then exactly one `--model <trimmed-id>` pair, then any accepted
  capability-guarded `--add-dir` emission.
- That verification MUST cover both fresh exec and resume flows so a resume-specific reordering or
  omission cannot drift from the normative contract.

Exclusion rules (pinned):

- This key MUST NOT, by itself, authorize any additional Codex CLI override beyond `--model`.
- Runtime rejection of the accepted model id remains backend-owned and MUST follow the safe
  `AgentWrapperError::Backend` translation requirements from
  `docs/specs/unified-agent-api/extensions-spec.md`.

Runtime rejection parity (pinned):

- If an accepted `agent_api.config.model.v1` value is rejected after the Codex streaming wrapper
  has already returned a run handle and the consumer-visible events stream is still open, the
  driver MUST emit exactly one terminal `AgentWrapperEventKind::Error` event carrying the same
  safe/redacted `message` that later appears in
  `AgentWrapperError::Backend { message }`, then close the stream.
- The completion error payload and the terminal error event MUST therefore carry byte-for-byte
  identical safe/redacted text for the rejection so downstream orchestrators can compare them
  deterministically.
- This event/completion parity requirement is owned by
  `docs/specs/unified-agent-api/extensions-spec.md`
  (`agent_api.config.model.v1`, "Runtime rejection behavior (v1, normative)").

## `agent_api.exec.add_dirs.v1` mapping (pinned)

The add-dir extension key is owned by:

- `docs/specs/unified-agent-api/extensions-spec.md`

When `extensions["agent_api.exec.add_dirs.v1"]` is accepted for a Codex exec/resume run, the Codex
streaming wrapper MUST:

- emit one repeated `--add-dir <dir>` pair per normalized unique directory,
- preserve the normalized unique directory order from the universal extension contract, and
- omit `--add-dir` entirely when the key is absent.

Placement rules (pinned):

- For both `stream_exec*` and `stream_resume`, the effective argv layout for accepted model/add-dir
  inputs MUST remain: wrapper-owned CLI overrides first, then exactly one `--model <trimmed-id>`
  pair when present, then repeated `--add-dir <dir>` pairs when present.
- Any accepted `--model <trimmed-id>` pair MUST appear before the first emitted `--add-dir`.
- The wrapper MUST NOT reorder the normalized directory list while emitting the repeated pairs.
- Fresh exec and resume runs MUST emit the same effective repeated `--add-dir <dir>` sequence for
  the same normalized directory list; the wrapper MUST NOT silently drop accepted directories on
  `stream_resume`.

Exclusion rules (pinned):

- This key MUST NOT, by itself, authorize any additional Codex CLI override beyond the repeated
  `--add-dir` pairs.
- Runtime rejection of the accepted directory list remains backend-owned and MUST follow the safe
  `AgentWrapperError::Backend` translation requirements from
  `docs/specs/unified-agent-api/extensions-spec.md`.

## Runtime semantics (v1, pinned)

### Spawn + driver start (pinned)

When a `codex::CodexClient` streaming method (e.g. `stream_exec`, `stream_resume`,
`stream_exec_with_env_overrides`) returns `Ok(...)`, the wrapper MUST:

- have already spawned the underlying `codex exec --json ...` process (or equivalent), and
- have started the internal driver tasks responsible for:
  - reading stdout (JSONL) and producing the typed event stream, and
  - waiting for process exit and producing the completion outcome.

Critically: starting and driving those tasks MUST NOT depend on the consumer polling/awaiting the
returned `completion` future.

Rationale: downstream orchestrators (e.g. Substrate) commonly drain `events` first and only await
`completion` later. Streaming timeouts and explicit termination must still take effect in that
pattern.

### Internal handoff backpressure (pinned)

If the streaming wrapper inserts an internal task or channel between the Codex event source and the
returned `events` stream, that handoff MUST preserve downstream backpressure.

Concretely:

- the wrapper MUST NOT eagerly drain Codex events into an unbounded in-memory queue ahead of
  consumer polling, and
- slow or absent polling of the returned `events` stream MUST eventually stall upstream event
  forwarding once the bounded handoff is full.

Rationale: live-driver startup is required, but it must remain compatible with the repo’s
bounded-memory posture and the pull-sensitive semantics expected by downstream consumers.

### Timeout semantics (pinned)

`CodexClientBuilder::timeout(...)` MUST be interpreted as a wall-clock bound on the streaming run
starting at handle creation time.

Concretely:

- The timeout countdown MUST start no later than the moment the streaming method returns `Ok(...)`.
- The timeout MUST NOT be delayed until the consumer first polls/awaits `completion`.

If the timeout triggers, the wrapper MUST request best-effort termination of the underlying child
process (e.g., via kill-on-drop and/or an explicit kill request).

### Explicit termination semantics (pinned)

For streaming entrypoints that expose a termination handle (e.g., `ExecStreamControl`):

- `ExecTerminationHandle::request_termination()` MUST be idempotent and best-effort.
- A termination request MUST be observed by the streaming driver regardless of whether the consumer
  is polling/awaiting `completion`.
- Once termination is requested, the wrapper MUST request best-effort termination of the underlying
  child process.
- If the consumer continues polling `events`, the `events` stream SHOULD reach finality (`None`)
  once the underlying process has been terminated and stdout has been fully observed/closed.

## Notes / non-goals

- This contract does not require that `completion` implies the `events` stream has been fully
  drained by the consumer.
- This contract does not supersede the universal `agent_api` completion gating rules (DR-0012);
  `agent_api` is responsible for enforcing those semantics at the universal boundary.


## Unified adapter approval intent on pinned Codex 0.156.1

The unified adapter's initial and resume paths use exec when noninteractive is absent or true,
and retain the existing external-sandbox bypass route. Ordinary exec emits root
`--ask-for-approval never` and explicit `--config 'approval_policy="never"'`. The latter preserves
Never even when native AutoReview configuration removes exec's headless default. The configured
reviewer is unchanged. External bypass retains its existing validation, opt-in and warning and
MUST NOT receive conflicting approval overrides.

With `agent_api.exec.non_interactive=false`, initial/resume use one owned native app-server process:
`thread/start` or `thread/resume`, followed by `turn/start`. Fork uses `thread/fork` followed by
`turn/start`. No failed native operation falls back to exec, creates a new session on resume failure,
or substitutes another approval policy. Handler registration alone does not change routing.

Explicit `never`, `on-request`, `untrusted` and the pinned granular object are forwarded unchanged
at thread and turn start. Absent policy remains absent, preserving native defaults and reviewer
selection through the existing home/configuration environment. Granular policy requires boolean
`sandbox_approval`, `rules`, and `mcp_elicitations`; optional boolean `request_permissions` and
`skill_approval` preserve native false defaults, including explicit false. Unknown/malformed fields
are rejected. Noninteractive true still rejects any explicit policy other than Never. Historical
`on-failure` remains accepted by validation but returns a truthful redacted native unsupported-policy
error on this pinned runtime. It is never aliased to another policy.

The native route preserves request-over-backend model precedence, wrapper GPT-5 reasoning defaults,
working directory and environment/home precedence, deterministic last/id selection, session handles,
actual completed assistant final text, event bounds and completion after cleanup. Explicit zero
timeout disables the deadline. The one run deadline includes initialization, config reads, selection,
thread/turn acceptance, callbacks and execution. Cancellation and process cleanup do not depend on
consumer event polling. Fork's model/add-dir refusals remain unchanged.

For initial/resume, the native process receives the selected `sandbox_mode` as a startup config
override. `config/read` for the effective absolute cwd and `configRequirements/read` supply the
native current configuration. The adapter supplies the complete `runtimeWorkspaceRoots` at thread
start/resume, including when no additional directories were requested. Its ordered, first-occurrence
deduplicated vector is `[cwd] + normalized requested add_dirs + legacy ambient writable_roots`,
with the last term included only when both native `default_permissions` and requirements
`allowedPermissionProfiles` are absent or null. These predicates mirror the pinned runtime's
explicit legacy sandbox override and profile selection; native Codex still enforces permissions.
Malformed configuration fails safely. Roots are not filesystem-canonicalized, dropped, or confused
with an incremental `runtimeWorkspaceRoots` alias. Supplying the current complete vector also
prevents resume from restoring an obsolete persisted root vector.

See the [native interaction contract](codex-interaction-contract.md) for callback ownership,
file-change context, structured responses, limits and refusal. Fork's wrapper transport is defined
by the [app-server contract](codex-app-server-jsonrpc-contract.md).

Migration: callers setting noninteractive false now receive app-server defaults instead of exec's
implicit Never. Set an explicit supported native policy when that intent matters, and register an
application handler if client interaction is expected. Existing default/unattended and external
bypass callers retain their routes. No approval UI or policy evaluator is supplied by UAA.

These adapter translation requirements do not qualify support depth. Source-bound fixtures and
isolated offline configuration observations do not prove successful model execution, real native
approval execution or independent adequacy.
