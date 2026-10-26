# Native Codex approvals through UAA

Status: proposed technical design, 2026-10-26. The maintainer selected native compatibility:
UAA transports supported Codex configuration and interactions; consuming applications choose
policy, decisions and presentation. This supersedes the proposed blanket refusal of explicit
non-Never initial/resume policies. It does not approve runtime implementation or qualify a pilot.
PR #300's existing compatibility repairs remain separate and its approval bridge is not implemented.

## Outcome and boundary

A caller can start, resume or fork a Codex run with the native approval settings supported by the
selected runtime. If Codex needs a client decision, UAA delivers the complete request to an
application-supplied asynchronous handler and returns that application's response to Codex.
UAA supplies no approval UI, automatic approver, permission database or cross-backend policy engine.
Existing unattended defaults and the explicit external-sandbox contract remain intact.

The pinned reference is Codex `0.156.1`, upstream commit
`b412ff32c417f855c2b2d1581b77058eed87c84b`, Linux musl binary SHA-256
`0b2e9301d6100dddda3b9d5c80ebaeaa3a2f1962388f2f36f6b96a9f08b1f33f`.
Supporting Codex as supplied does not mean silently upgrading that qualification target or
claiming every upstream feature. New protocol features remain explicit additions.

## Facts driving the change

Initial/resume currently use `codex exec`; fork uses `codex app-server`. The initial/resume root
approval flag is not a reliable effective-policy control. Ordinary headless exec selects Never;
AutoReview configuration is a distinct exception. PR #300 explicitly preserves Never intent and
leaves other policy requests unchanged. This proposal fixes the route for those other requests.

| Requested policy | Pinned CLI path | Pinned app-server path |
| --- | --- | --- |
| `never` | Supported; explicit config preserves intent | Start/resume/fork accept it |
| `on-request` | Root flag can be accepted while ordinary exec uses Never | Fresh start/resume/fork return `on-request` |
| `untrusted` | Root flag rejected; a public config file selecting it is also retired | Explicit start/resume/fork RPC overrides still accept and return `untrusted` |
| `on-failure` | Rejected | Not a member of the pinned RPC policy enum; rejected |
| `granular` object | Not supported by the current UAA typed option | Pinned experimental RPC enum; fresh start/resume return its five controls |

Sources: pinned [exec implementation](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/exec/src/lib.rs),
[RPC policy enum](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/app-server-protocol/src/protocol/v2/shared.rs),
[config validation](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/core/src/config/mod.rs), and
[thread/turn request types](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/app-server-protocol/src/protocol/v2/thread.rs).
The RPC schema and CLI schema differ; neither one establishes the other's supported values.

Isolated actual-binary observations used an empty credential environment, private homes and a
network namespace with no external access. They establish effective response settings and native
rejections, not successful command execution, human approval processing, model runs or adequacy.
A first sequential resume probe reused one already-loaded thread; later responses retained that
thread's earlier settings. Fresh-process probes established each resume override independently.
The implementation must use one owned process per run, as the existing fork lifecycle does.

## Recommended routing

Choose the route from existing request semantics; callers do not need a transport selector.

| Request | Initial/resume | Fork |
| --- | --- | --- |
| Noninteractive absent or true | Keep current exec path and explicit Never preservation | Keep app-server with explicit Never |
| Noninteractive false | Use app-server `thread/start` or `thread/resume`, then `turn/start` | Use existing `thread/fork`, then `turn/start` |
| External sandbox true | Keep existing bypass path, opt-in and contradictions | Keep existing Never/danger-full-access mapping |

Changing noninteractive=false initial/resume to app-server is an observable correction: native
app-server defaults replace headless exec's implicit Never. Document this in the implementation's
migration note and test it. Do not silently fall back to exec, create a new session on resume
failure, enable AutoReview or retry under another policy. Handler registration alone does not
change routing, noninteractive mode, sandbox, reviewer or permissions.

Forward explicit policy unchanged at thread creation/resume/fork and turn start. With no explicit
policy under noninteractive=false, omit the override and retain native effective configuration.
Preserve reviewer selection from the application's existing `CODEX_HOME`/configuration mechanism;
no new reviewer extension is needed for this slice. User review and AutoReview remain separate
native choices. A handler is not evidence that the caller selected a particular reviewer.

Extend `backend.codex.exec.approval_policy` additively to admit the pinned native granular object:
`{"granular":{"sandbox_approval":true,"rules":true,"mcp_elicitations":true,
"request_permissions":false,"skill_approval":false}}`. Preserve the exact three required fields
and native optional defaults; reject malformed/unknown fields rather than inventing meaning.
Noninteractive=true still rejects explicit policies other than Never. Keep the legacy string
`on-failure` accepted by the old type/validation surface, with a truthful, redacted runtime
unsupported-policy error where this runtime rejects it. Do not alias it to on-request or remove a
public enum variant as part of this bridge. There is no claim that obsolete native behavior works.

Initial/resume must retain cwd/env/home precedence, request-over-backend model selection and
reasoning configuration, supported sandbox/add-dir semantics, deterministic last/id selection,
session handles, final text, event bounds and completion-after-cleanup behavior. Resume is resume,
not fork. Fork's existing model/add-dir refusals are unchanged. Before enabling the new route,
prove these mappings against the pinned source/schema and production-path tests. In particular,
additional writable roots need an exact native sandbox/config mapping: do not substitute
`runtimeWorkspaceRoots` merely because the names resemble add-dirs, erase ambient permissions,
or drop requested directories. A route is not ready while that required correspondence is missing.

## Small application integration surface

Add a Codex-backend-local asynchronous handler through a builder on `CodexBackend`, leaving
`CodexBackendConfig` and universal run request/handle structs source-compatible. Proposed shape:

```rust
// Sketch: names and concrete error types belong to the implementation contract.
CodexBackend::new(config).with_interaction_handler(handler)
// handler(request { method, params }) -> Future<Result<Reply, InteractionError>>
// Reply = Result(native_json_result) | Unsupported
```

The handler receives native method and complete JSON parameters; the library privately owns the
originating run/connection and numeric-or-string RPC ID. Replies cannot select another ID or
connection. Native discriminated response bodies are preserved; a single approve/reject boolean
would lose session grants, cancellation, amendments and permission-subset responses. Validate
known request/response shapes against the pinned contract, including allowed decision variants;
do not reimplement Codex's policy evaluator. The explicit callback is a trusted application
boundary: requests may contain commands, paths or MCP data and are not copied into ordinary
public events, logs or redacted diagnostics.

Keep the handler types local to `agent_api::backends::codex` and privately adapt to the low-level
wrapper's corresponding app-server request/reply API. No `codex` dependency types appear in UAA
public signatures/reexports; the existing backend-type-leak guard remains controlling. Preserve
old wrapper construction APIs; add explicit v2 thread-start/resume request methods/types instead
of adding required fields to existing public structs. Ordinary MCP send_approval is a different
protocol and is not reused for app-server responses.

First supported interactions are the three native v2 approval requests (command execution, file
change, additional permissions), tool user-input and MCP elicitation. These are requests requiring
a response, not notification events. The same thin request/reply mechanism carries their native
payloads. Dynamic tools, account token refresh, attestation and unrelated server methods are not
claimed supported by this approval increment; return the existing explicit unsupported failure.
The handler can bridge to an application's UI, service or own policy code without UAA providing it.

## Ownership, refusal and completion

- Install the request receiver before initialization or any thread/turn RPC; keep reading output
  while an asynchronous handler waits. Do not hold the writer/pending-RPC lock across user code.
- Scope requests to the owned run and selected thread/turn. Queue early turn requests only until
  the authoritative turn/start identity is known; bound that queue and reject mismatches. Preserve
  server-originated IDs independently from client request IDs; equal numbers are not collisions.
- No handler, handler Unsupported/error/panic, invalid reply or unknown method never grants access.
  Return a native RPC error and the pinned safe approval-required/client-interaction-required run
  failure, then interrupt/clean up. Do not wait indefinitely for an absent handler. Configured
  native AutoReview can complete without a client request and does not require a fake handler.
- Noninteractive=true never invokes an approval handler. An unexpected interaction fails promptly
  with the existing safe refusal. External-bypass validation/warnings are unchanged.
- Cancel/timeout, serverRequest/resolved, disconnect and terminal completion invalidate pending
  callbacks. Drop the corresponding handler futures; a host must abandon associated pending UI
  work when its future is cancelled. Late/duplicate replies cannot authorize another request or
  be sent after invalidation. No automatic timeout grant or deny is synthesized as user consent.
- Keep callback work owned by the run; no detached executor, persistent prompt service or jobs.
  Use one run deadline through setup, waiting for host decisions and execution; cleanup retains
  its existing bounded shutdown grace after expiry. Explicit zero timeout remains no timeout;
  cancellation stays effective without event polling.
- Bound interaction frames to 1 MiB and pending requests to 16 per run, with no truncation of a
  request shown for a decision. Exceeding either limit fails explicitly and cleans up; ordinary
  event envelope limits remain independent. A native interaction cannot bypass bounded transport.
- A replied approval, thread creation or accepted turn is not run success. Await matching actual
  terminal completion, drain output and reap the owned child, retaining PR #300's lifecycle repairs.

## Small implementation sequence and acceptance

1. **Low-level app-server request/reply support.** Add opt-in request delivery with private,
   single-use response ownership; preserve current refusal when no consumer is registered.
   Cover string/numeric overlapping IDs, complete native decision bodies, unknown/malformed
   requests/replies, pending limits, resolved requests, callback cancellation and process shutdown.
2. **UAA adapter and routing, together.** Add the backend-local handler and v2 start/resume types,
   route noninteractive=false initial/resume through the owned app-server lifecycle, extend granular
   policy validation and use the handler for fork. Cover every previously accepted model/add-dir/
   sandbox/session combination and native/default/reviewer policy matrix before exposing this path.
   Update canonical contracts and migration notes with the actual new behavior in this increment.
3. **End-to-end verification and pilot accounting.** Prove production adapter start/resume/fork
   against source-bound fixtures plus isolated native-binary configuration checks. Native requests
   must reach the host and exact replies must resume or decline correctly; test wrong-run, resolved,
   cancel/timeout and malformed decisions. Successful model/command approval evidence needs separate
   explicit runtime proof; synthetic protocol success is not pilot qualification or A1 provenance.
   Update affected pilot flow/effect classifications before any enrollment or qualification.

Implementation uses Rust2021, existing Tokio/serde machinery, safe owned-process patterns and the
existing 700 Rust-code-LOC limit, with no new dependency or generic harness rewrite. Run targeted
wrapper/UAA tests during each change; both requested review lanes and all12 required local gate
groups apply before each implementation PR is ready. Existing gate commands are `make preflight`,
`cargo test --workspace --all-targets`, `cargo run -p xtask -- agent-api-backend-type-leak-guard`,
and each agent's manifest validation, `--tip origin/staging` validation and acquisition-plan check.
No new required checks, automation, registry fields or production settings are introduced here.

The two recommendations are the semantic routing above and this backend-local native handler.
They implement the maintainer's selected direction. Review this concrete plan before changing the
public/runtime behavior; approval of the direction is not a claim that the bridge is implemented.
