# Isolated Codex runtime context and current classification inventory

Baseline: staging `6eac22fd70077e20b031eea32593b478c9f79912` (PR #301).
This slice records a native, offline configuration observation and exact proposed
effect classifications. It changes no ordinary SDK behavior and installs no registry
assignments, isolated writer profile, evidence adoption or qualification.

## Reproduction and scope

```sh
python3 scripts/observe_codex_pilot_context.py \
  --binary /absolute/path/to/pinned/codex \
  --output /private/output/pilot-runtime-context.json
```

Linux and bubblewrap are required. Both the musl binary and the complete protocol/config
report must match their pinned SHA-256 values before any launch. The collector creates
an empty filesystem root, private pid/user/network namespaces and private home/work/temp/XDG
directories. The only mounted executable is the pinned binary at `/bin/codex`. It clears
the child environment, supplies the exact recorded nonsecret variables and sends no
credential, thread/turn or model request. No host `/etc/codex`, ancestor project, home,
skills, hooks or plugins can be read through this filesystem. The configuration is the
same expected config bound by the landed read-only readiness planner; a regression checks
the full bytes, binary digest and protocol digest against its candidate bases.

The [recorded native observation](../../../cli_manifests/codex/reports/0.156.1/pilot-runtime-context.json)
contains the bounded JSON-RPC exchange, full config/origin/layer responses, feature listing,
exact child environment, collector/helper/launcher digests and before/after file digests.
It is an audit snapshot: random installation ids, SQLite state, diagnostics and runtime
metadata may differ on repetition. Recompute current observations for packet evidence.
No runtime evaluator consumes this snapshot as E or clears readiness from its presence.

The collector launches Codex directly. Its explicit environment is **not an observation
of the production adapter's final child environment**, capability probes, overlays or
root translations. Its minimal namespace is also not a proven host for positive model,
sandbox or approval execution. Native diagnostics explicitly report that host bubblewrap
is absent from the child PATH and that Codex would use its bundled copy. No sandboxed
command is executed to validate that fallback here. These limits stay visible.

## Native findings

| Input/effect | Observed result | Limit / consequence |
| --- | --- | --- |
| Controlled config | Native config/read accepts model `gpt-5`, provider `openai`, approval `never`, workspace-write, disabled web search/analytics and157 explicit false feature entries | Configuration projection is not every resolved internal default or effective turn policy |
| Loader layers | User config and an empty system layer; no project layer; configured requirements are null | Observed only in this empty-root namespace; producer must bind the actual private Git project and loader context |
| Effective feature listing |149 canonical feature rows; six true despite explicit false config | Keep config and effective values distinct; aliases, removed flags and effective runtime values are not interchangeable |
| Object-only setting | tool_registry omitted from both feature listing and config projection | Default remains unobserved; do not silently interpret omission as false |
| Native startup mutation | Config-only app-server initialization installs bundled system skills, installation id and SQLite state under private CODEX_HOME | ADMIN/FX1 must cover initialization and partial startup failure, not only output/session writes; files are inventoried, skill execution is not proved |
| Source correspondence | Exact musl binary/protocol digests and collector/helper source hashes recorded | Not A1 build/host attestation, authenticated packet evidence, adequacy or an upstream behavior claim on another target/version |
| Session/model execution | None requested | Actual initial/resume/fork, native approvals, session persistence, cancel and cleanup obligations remain for the producer |

The six enabled feature names are `item_ids`, `resize_all_images`,
`terminal_resize_reflow`, `tool_search_always_defer_mcp_tools`, `tui_app_server`
and `unified_exec`. In particular, an emitted `unified_exec=false` is not evidence
that unified execution is disabled. The snapshot retains all149 values, not just those six.
The initial file inventory contains only config.toml; the separate after-config-read
inventory proves that startup itself creates bundled assets and state before features/list.

## Maintainer context decisions

These recommendations clarify the private pilot's reviewed context; they do not restrict
normal SDK callers or approve individual classification rows. Both were submitted to the
maintainer during this slice. Until explicitly approved, retain the affected context gaps.

1. Keep the explicit false configuration unchanged and bind the observed native effective
   feature values, including the six true values. Do not patch Codex or claim an all-off
   runtime. Review their actual effects in native packet evidence.
2. Permit and inventory the pinned binary's native startup artifacts under private
   CODEX_HOME, including bundled system skills and SQLite files. Continue to deny external
   user/host skills, hooks and plugins. Do not approve skill loading/execution from the
   existence of those files; investigate and prove that behavior separately.

tool_registry and other unexposed internal defaults remain unresolved. There is no
recommendation to waive an observation, invent a default, enable unrelated functionality,
install the proposed profile or substitute fixture behavior for native execution.

## Exact current effect proposals

The [current classification table](support-depth-current-pilot-classifications.md) enumerates
all207 operation/surface edges (71 initial,94 resume,42 fork), using the94 acquired CLI
identities. It includes exec ancestry, managed initial/resume app-server effects, original
shard rows normalized out of the union, and the claim-only obsolete full-auto unit. A
regression compares the table's exact memberships against actual candidate P/O. Proposed
assignments cover whole controlled allowed/denied regimes, including absence/defaults;
they are not executable classifications. The earlier30 approved initial assignments are
not transferred to the changed multi-route basis. The readiness API continues reporting
all uninstalled rows and incomplete runtime/producer/profile/chain requirements.

MANAGED on initial/resume is already approved, alongside fork. The existing template
baseline stays53 operation/obligation/target tuples:17 initial,18 resume,18 fork. All
proposed table templates are within that baseline; this is not53 test cases or a maximum
on effects. Individual row approval and adequate native evidence are still required.

The acquired protocol closures are separate facts, never fabricated CLI identities:

| Exact native method/message | Current route/effect and proposed coverage |
| --- | --- |
| initialize | All managed variants; MANAGED readiness/correlation; startup writes also ADMIN |
| config/read, configRequirements/read | Managed initial/resume before thread operations; configuration/requirements and effective roots; CORE+MANAGED; only these reads and initialize are observed here |
| thread/list | Managed resume-last and fork-last; SESSION selection/pagination/no-match, MANAGED correlation; no public QUERY promise |
| thread/start | Managed initial; CORE+ADMIN+MANAGED, including partial native state creation |
| thread/resume | Managed resume-id/last; CORE+SESSION+ADMIN+MANAGED |
| thread/fork | Fork-id/last; CORE+SESSION+ADMIN+MANAGED, new identity and partial persistence |
| turn/start, turn/interrupt | All managed variants; CORE+MANAGED+ADMIN, safety/model/prompt, completion/cancel and permitted mutations |
| item/commandExecution/requestApproval, item/fileChange/requestApproval, item/permissions/requestApproval | All applicable managed variants; CORE+MANAGED+ADMIN, immutable owned context and native decisions; real execution evidence remains due |
| item/tool/requestUserInput, mcpServer/elicitation/request | Applicable managed variants; CORE+MANAGED, native interaction and nullable-turn ownership; no additional public MCP management promise |
| thread/started, thread/closed, turn/started, turn/completed | CORE+MANAGED; SESSION attribution where selected; terminality and lifecycle |
| item/started, item/completed, item/fileChange/patchUpdated | CORE+MANAGED+ADMIN; immutable correlated item context and mutation handling |
| item/agentMessage/delta, item/reasoning/textDelta, serverRequest/resolved, error | CORE+MANAGED; bounded ordered delivery, request invalidation and truthful errors |

The pinned schema closure includes the complete response and transitive parameter types.
This table does not approve arbitrary RPC passthrough. Methods/messages not admitted by
the production adapter retain its existing refusal/unknown handling. Native shutdown is
stdin EOF plus owned cleanup, not an invented shutdown RPC. Exec variants retain the
production CLI JSON parser and their own runtime evidence requirements.

Next is the packet-specific native producer over production adapters: observe the actual
child boundary and native turns/interactions, obtain adequate evidence for every due
tuple, then prove the public isolated prepare/refreeze/evidence/closeout/promotion chain.
Live model calls need separate authorization. Production enrollment follows that isolated
proof and adoption. No background jobs or schedulers are introduced.
