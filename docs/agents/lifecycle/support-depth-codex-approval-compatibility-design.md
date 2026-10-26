# Native Codex approvals through UAA

Status: implementation approved, 2026-10-26. The maintainer selected native compatibility:
UAA transports supported Codex configuration and interactions; consuming applications choose
policy, decisions and presentation. This supersedes the proposed blanket refusal of explicit
non-Never initial/resume policies. The maintainer subsequently approved implementing this design
in PR #300. Implementation and configuration checks do not qualify a pilot.

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

Before this correction, initial/resume used `codex exec`; fork used `codex app-server`. The initial/resume root
approval flag is not a reliable effective-policy control. Ordinary headless exec selects Never;
AutoReview configuration is a distinct exception. PR #300 explicitly preserves Never intent and
left other policy requests unchanged. The bridge corrects the route for those requests.

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
`CodexBackendConfig` and universal run request/handle structs source-compatible. Integration shape:

```rust
// Public types are specified in docs/specs/codex-interaction-contract.md.
CodexBackend::new(config).with_interaction_handler(handler)
// handler(request { method, params, item_context }) -> Future<Result<Reply, InteractionError>>
// Reply = Result(native_json_result) | Unsupported
```

The handler receives the native method, complete original JSON parameters and separate optional
`item_context` JSON. Context never replaces, augments or rewrites native `params`. The library
privately owns the originating run/connection and numeric-or-string RPC ID. Replies cannot select
another ID or connection. Native discriminated response bodies are preserved; a single approve/reject
boolean would lose session grants, cancellation, amendments and permission-subset responses. Validate
known request/response shapes against the pinned contract, including allowed decision variants;
do not reimplement Codex's policy evaluator. The explicit callback is a trusted application
boundary: requests may contain commands, paths or MCP data and are not copied into ordinary
public events, logs or redacted diagnostics.

Keep the handler types local to `agent_api::backends::codex` and privately adapt to the low-level
wrapper's corresponding app-server request/reply API. No `codex` dependency types appear in UAA
public signatures/reexports; the existing backend-type-leak guard remains controlling. Preserve
old wrapper construction APIs. Add `ThreadStartParamsV2`/`thread_start_v2`,
`ThreadResumeParamsV2`/`thread_resume_v2`, `ThreadForkParamsV2`/`thread_fork_v2` and
`TurnStartParamsV3`/`turn_start_v3`, each carrying the native string-or-granular policy shape.
Existing `ThreadForkParams.approval_policy` and `TurnStartParamsV2.approval_policy` are
`Option<String>`; changing them would break callers. The additive turn suffix versions the Rust
construction API, not Codex's v2 wire protocol. Preserve existing structs/methods and their behavior;
all four new methods target the existing native RPC names. Ordinary MCP send_approval is a
different protocol and is not reused for app-server responses.

First supported interactions are the three native v2 approval requests (command execution, file
change, additional permissions), tool user-input and MCP elicitation. These are requests requiring
a response, not notification events. The same thin request/reply mechanism carries their native
payloads. Dynamic tools, account token refresh, attestation and unrelated server methods are not
claimed supported by this approval increment; return the existing explicit unsupported failure.
The handler can bridge to an application's UI, service or own policy code without UAA providing it.

## Private file-change context

For `item/fileChange/requestApproval`, `item_context` is required: the complete native
`fileChange` item from its associated `item/started`, including `changes[].{path,kind,diff}`.
Other supported methods receive `None` in this increment. The retained pinned schemas
`FileChangeRequestApprovalParams.json` and `v2/ItemStartedNotification.json` establish the split:
approval parameters contain identities and optional reason/grant root, not the changes.
The current UAA mapper in `crates/agent_api/src/backends/codex/fork/notifications.rs` emits item
metadata only. Capture context before that mapping; preserve the ordinary event contract.

Keep a bounded private association keyed by owned run/connection, selected thread, accepted turn
and native item ID. Only an active `fileChange` item from that exact scope is eligible; never use
another item's context, a prior turn's item, historical resume data or filesystem reconstruction.
Process incoming notifications/requests in transport order, including replay after thread/turn
acceptance. Retain eligible `item/started` context until item completion or run invalidation.
At approval dispatch the earlier matching context must exist; missing, malformed, completed or
wrong-scope context fails explicitly through the safe interaction failure and cleanup path before
calling the handler. A request preceding its context is a missing-context failure; do not wait
indefinitely or show a context-free prompt. Prove the pinned native ordering before enabling this
route, alongside the production-path cases below.

Give each prompt an immutable snapshot of that native item. Preserve array order, all JSON values
and exact UTF-8 bytes of decoded path/diff strings and kind fields; JSON lexical whitespace/escape
spelling is not an API promise. No diff normalization, truncation or synthesized changes. Keep the
snapshot linked privately to the originating request's single-use reply ownership. A later change
to the retained `changes` (including native `item/fileChange/patchUpdated`), replacement of that
item identity or item completion invalidates pending callbacks and the old context before a reply
can be written. Identical repeated context is harmless; changed context must fail safely and clean
up rather than silently update a prompt. A retry requires a fresh native request with eligible
context. Check invalidation and response ownership together when sending the reply.

Apply the 1 MiB interaction bound to incoming context frames and to each combined serialized
request-plus-context delivery. Retain at most 16 item contexts and at most 16 MiB of interaction
request/context JSON per run, including pre-acceptance queues and pending snapshots; shared
immutable context need only be counted once. Overflow fails explicitly and cleans up; do not
evict required context silently. Release context at item completion and run close. Context shares
the callback's trusted boundary: no paths, diffs or native payload copies in ordinary events,
logs or diagnostics. This is transient correlation within the existing run, not a public event
expansion or persistent approval store.

## Ownership, refusal and completion

- Install the request receiver before initialization or any thread/turn RPC; keep reading output
  while an asynchronous handler waits. Do not hold the writer/pending-RPC lock across user code.
- Scope every request to the owned run/connection and selected thread. Before the thread RPC
  establishes the authoritative selected thread (the new thread for fork), queue early requests
  within the same bounds; reject mismatches when replaying, or invalidate on thread RPC failure.
  Preserve server-originated IDs independently from client request IDs; equal numbers are not
  collisions. The private identity includes run/connection and the server's typed RPC ID.
- Command/file/additional-permission approvals and tool user-input require matching native thread,
  turn and item identities. Until `turn/start` establishes the authoritative turn, queue within
  the same bounds, then replay in transport order and reject wrong-turn requests. Never infer turn
  ownership from a request's own claimed ID.
- MCP elicitation is method-specific: the pinned `McpServerElicitationRequestParams` schema makes
  `turnId` optional/nullable because elicitation is a standalone request. With absent or null
  `turnId`, use the selected thread plus owned run/connection/server-request identity; dispatch as
  soon as thread ownership is established, including before turn acceptance. Do not invent a turn
  ID, wait for one or attach the current turn. Preserve absent versus null in original `params`.
  If `turnId` is present and non-null, require the same authoritative turn match as above, including
  bounded queuing before acceptance and rejection of wrong/stale turns. A different thread or
  connection never becomes eligible by omitting `turnId`.
- No handler, handler Unsupported/error/panic, invalid reply or unknown method never grants access.
  Return a native RPC error and the pinned safe approval-required/client-interaction-required run
  failure, then interrupt/clean up. Do not wait indefinitely for an absent handler. Configured
  native AutoReview can complete without a client request and does not require a fake handler.
- Noninteractive=true never invokes an approval handler. An unexpected interaction fails promptly
  with the existing safe refusal. External-bypass validation/warnings are unchanged.
- `serverRequest/resolved` invalidates the matching owned connection/request identity, including
  an absent/null-turn MCP elicitation. Cancel/timeout, disconnect, selected-thread close, run close
  and terminal run completion invalidate all pending callbacks, early queues and item context for
  that run, including standalone MCP requests. A matching turn's terminal completion also invalidates
  its turn-scoped requests/context; standalone requests still end when the run terminates. Drop the
  corresponding handler futures; a host must abandon associated pending UI work when its future
  is cancelled. Late/duplicate replies cannot authorize another request or be sent after
  invalidation, even if another connection reuses the same RPC ID. No automatic timeout grant or
  deny is synthesized as user consent.
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
2. **UAA adapter and routing, together.** Add the backend-local handler, private item correlation
   and all four additive thread-start/resume/fork and turn-start methods/types named above;
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

Required implementation cases for the accepted discovery causes (original labels/priorities kept):

- **AD-D01 P2 / NAD-ADV-01 P2:** drive the production adapter with pinned-schema file-change
  notifications and approval requests. Assert complete original `params` equality, separate complete
  `item_context` equality and decoded path/diff UTF-8 byte equality, with multiple changes, distinct
  kinds, Unicode and multiline diffs. Send two interleaved items and overlapping numeric/string RPC
  IDs; each callback must see only its own run/connection/thread/turn/item snapshot and replies must
  return only on that request's connection/ID. Assert ordinary events/logs/diagnostics omit context.
  Exercise item-before-request and queued pre-acceptance replay; request-before-item, missing,
  malformed, wrong-item/thread/turn/connection and prior-turn/completed context must fail without a
  callback or approval. Change context through `item/fileChange/patchUpdated` or a replacement
  `item/started` while the handler waits, and complete the item before reply; the old future/reply
  must be invalidated, while an identical context repeat preserves the snapshot.
  Test exactly-at/above frame, combined-delivery, retained-byte and context-count bounds without
  truncation or silent eviction; every failure must retain cleanup and safe redacted errors.
- **NAD-ADV-02 P2 / AD-D02 P3:** deliver standalone MCP requests with separately absent and null
  `turnId` after thread selection but before turn acceptance; both must reach the handler promptly,
  retaining original params, and reply on their own connection/ID without synthesizing a turn.
  Also cover delivery before thread acceptance, thread-RPC failure, matching non-null turn queued
  before turn acceptance, wrong/stale non-null turn and wrong thread/connection with null turn.
  Only established matching scope may invoke the handler. Resolve, cancel, expire, terminate,
  disconnect and close while standalone handlers wait; drop futures, reject late/duplicate replies
  and prevent reused IDs on another run/connection from authorizing anything.
- **AD-D03 P3:** prove existing public request construction still compiles unchanged. Serialize each
  new thread-start/resume/fork and turn-start shape with the same granular object, including explicit
  false controls; assert exact native RPC method/JSON correspondence and no policy substitution.

These are implementation acceptance obligations. The canonical
[interaction contract](../../specs/codex-interaction-contract.md) defines the implemented handler,
routing and native configuration mapping. Offline native configuration observations and adapter
fixtures cover settings and protocol behavior; successful native approval execution and pilot
qualification still require their own evidence.

The root mapping reads native `config/read` (without layers) and `configRequirements/read` before
thread creation/resume. App-server startup receives the selected `sandbox_mode` config override,
matching exec's explicit sandbox selection. With that selection the pinned native implementation
uses legacy syntax and discards a persisted selected profile. Supply an explicit first-occurrence
union of effective cwd and normalized requested add-dirs; append native-resolved legacy writable
roots only when both `default_permissions` and `allowedPermissionProfiles` are absent/null.
Supply this complete vector before start/resume, including when add-dirs is empty, so resume cannot
restore stale roots. Native Codex still owns profile compilation, requirements and enforcement.
These read-only configuration RPCs are additional pilot effects to classify before enrollment;
they do not expand the existing approved registry inventory.

Implementation uses Rust2021, existing Tokio/serde machinery, safe owned-process patterns and the
existing 700 Rust-code-LOC limit, with no new dependency or generic harness rewrite. Run targeted
wrapper/UAA tests during each change; both requested review lanes and all12 required local gate
groups apply before each implementation PR is ready. Existing gate commands are `make preflight`,
`cargo test --workspace --all-targets`, `cargo run -p xtask -- agent-api-backend-type-leak-guard`,
and each agent's manifest validation, `--tip origin/staging` validation and acquisition-plan check.
No new required checks, automation, registry fields or production settings are introduced here.

The two recommendations are the semantic routing above and this backend-local native handler.
The maintainer approved both recommendations and implementation. Required implementation review
and gates apply to the complete source candidate; this document alone is not runtime proof.
