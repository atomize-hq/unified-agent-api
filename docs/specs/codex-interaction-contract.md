# Codex native interaction contract

Status: **Normative**. Scope: `agent_api::backends::codex`, pinned Codex `0.156.1`
(`b412ff32c417f855c2b2d1581b77058eed87c84b`).

`CodexBackend::with_interaction_handler` registers an asynchronous application callback without
changing native routing, reviewer, policy, sandbox or permissions. Existing backend configuration
and universal request/handle structs remain source compatible. Handler signatures contain no
wrapper backend types. Applications own decisions and presentation.

`CodexInteractionRequest` carries the original native `method` and complete `params`, plus separate
optional `item_context`. Responses are `CodexInteractionReply::Result(native_json_result)` or
`Unsupported`; handler errors are redacted `CodexInteractionError`. Native request IDs and connection
ownership remain private. Results are checked against the pinned native response schemas and
returned unchanged on the original request. Unsupported, invalid, failed or panicking callbacks
cannot grant access. Callbacks are owned futures within the run, including both their construction
and polling; no callback task is detached. Dropping the future instructs applications to abandon
associated UI work.

The supported methods are `item/commandExecution/requestApproval`,
`item/fileChange/requestApproval`, `item/permissions/requestApproval`, `item/tool/requestUserInput`
and `mcpServer/elicitation/request`. Other native requests fail explicitly. Ordinary MCP approval
messages are a separate protocol.

The selected thread comes only from successful thread RPC acknowledgement. Turn-scoped requests
wait for the authoritative `turn/start` acknowledgement; their claimed IDs cannot establish scope.
All four item methods require matching thread, turn and item identities. MCP requests require the
selected thread; absent or null `turnId` is preserved and dispatched once the thread is known,
including before turn acceptance. A non-null MCP turn requires the accepted turn. Pre-acceptance
messages are replayed in transport order when their scope becomes eligible.

For file-change approvals, context is the complete earlier active native `fileChange` item from
`item/started` for the selected thread/turn/item. It preserves array order, all JSON values, decoded
UTF-8 paths/diffs and change kinds. Original approval parameters are never augmented. Other methods
receive no item context. Missing, malformed, wrong-scope, completed or changed context fails safely
before a decision is used. Changed replacement items or `item/fileChange/patchUpdated` invalidate
the run's pending decisions; identical repeated context is harmless. Completing a prompted file
item invalidates its pending decision. Context is not reconstructed from files or resumed history.

Native resolution, cancellation, timeout, disconnect, selected-thread close, terminal completion
and run shutdown invalidate owned pending work and drop callback futures. The transport checks
single-use ownership again when writing a response. Reused numeric or string IDs on other runs or
connections cannot reuse an earlier decision. No timeout response is synthesized as consent.

Interaction frames and each serialized request-plus-context delivery are limited to 1 MiB.
Each run retains at most 16 native requests and 16 private item contexts. Transport queued JSON,
owned requests and adapter retained JSON share one 16 MiB retention budget; copied delivery JSON
is charged separately. Overflow fails explicitly without truncation or eviction. Ordinary event
payload bounds are independent. Private paths, diffs and native requests do not enter ordinary
public events or redacted error messages. The callback is a trusted application boundary and may
receive sensitive native data.

The native event handoff and its pending forwarding queue each use the existing backend event
channel capacity. Forwarding is polled alongside native reading and callbacks. Overflow fails
explicitly; it cannot grow an unbounded event queue or silently discard output. Process cleanup
and callback invalidation precede final event draining. Universal completion still obeys the
existing event-finality gate.

No handler is required when native Codex completes without a client request, including native
AutoReview selected through existing home/configuration. An actual request without a handler,
or an interaction during noninteractive mode, produces the safe `approval required` failure,
interrupts an accepted turn when applicable, and cleans up. Registration alone does not select
AutoReview or authorize an automatic decision.

Protocol fixtures establish adapter behavior and source correspondence. They do not establish
successful native model execution, real command/file approval execution, pilot qualification,
A1 provenance or adequacy.
