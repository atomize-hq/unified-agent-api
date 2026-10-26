# Complete Codex pilot policy proposal

Status: implementation direction approved, 2026-10-26. The maintainer approved CP-1,
CP-2 and CP-3's resolved-row direction; CP-3 does **not** approve any U-* row or missing evidence.
This approval authorizes the bounded fork/approval compatibility repairs and tests; it does not
install the proposed profile, enroll operations or establish qualification.

Original design dated 2026-10-25. Baseline staging
`1f38133ffc68f4c7a254f05a6ce63b1543f619d3` (PR #299). This document completes the
PI-P3 design direction with exact proposed flows, dispositions and implementation prerequisites.
PI-P1's30 initial assignments remain approved; PI-P2 is implemented; PI-P3 did **not** approve
these new declarations, a value boundary, protocol acquisition, evidence or host activation.
The [exact appendix](support-depth-complete-pilot-classifications.md) is the review inventory;
the registry/P remains the sole executable policy inventory. The
[depth](../../specs/support-depth-contract.md), [registry](../../specs/agent-registry-contract.md)
and [evidence](../../specs/support-depth-evidence-v1.md) contracts control. Changes described
below are proposed amendments and implementation work, never an interpretation that enables them today.

## Recommendation and completion claim

Claim only `agent_api.run`, mapped to **all three** actual adapter operations: initial exec,
exec resume and managed app-server fork. Preserve every exposed last/id variant and every supported
policy combination below. Other advertised capability ids remain unchanged and not automatically
depth-qualified; their existing semantics still apply wherever a run reaches them. Do not call any
of these operations wrapper-only. Generic wrapper entrypoints remain subject to their own promises;
qualifying a typed adapter boundary would not qualify arbitrary wrapper config or passthrough.

Recommend interpreting the five literal selectors inside the existing fixed isolated profile, with
a source-owned controlled execution context. This adds no public registry field or general CLI
grammar. It requires a reviewed whole-unit/context clarification and private intake extension,
and distinguishes the real flows without hiding acquired units.
Until that extension, complete classification and adequate proof exist, **retain refusal** of new
mapped intake and positive freeze. A complete design can identify a required repair or unknown;
it cannot make that operation ready by assigning a plausible template. No row in this proposal is
execution evidence, an installed classification, an NA/debt/exclusion or evidence adoption.

The approved implementation direction covers these three policy decisions: CP-1 selects this complete mapped scope and
templates; CP-2 permits the finite boundary/ancestry/protocol representation below; CP-3 accepts
only individually resolved appendix rows with their whole bound regimes. Recommend all three as
an implementation direction, with CP-3 expressly withholding the U-* rows until their prerequisites
are met. The alternative is reviewing every possible generic config/profile/feature/hook/provider
regime before enrollment. That avoids a boundary extension but is substantially broader; arbitrary
strings cannot be qualified by adding more templates.

## Actual production flows

These are repository source facts at the baseline, not observations of the musl binary. Follow
[harness](../../../crates/agent_api/src/backends/codex/harness.rs),
[policy](../../../crates/agent_api/src/backends/codex/policy.rs),
[exec](../../../crates/agent_api/src/backends/codex/exec.rs),
[fork](../../../crates/agent_api/src/backends/codex/fork.rs) and
[selectors](../../../crates/agent_api/src/backends/session_selectors.rs).
Unknown extension keys and malformed requests are rejected by the production harness before spawn.
Both session extensions together reject; last requires exactly `{"selector":"last"}`; id requires
`{"selector":"id","id":<nonblank string>}`. Unknown object keys reject. Id bytes, including
surrounding whitespace, are preserved after the nonblank check. A false or malformed selector is
not absence, and selection failure must not fall back to initial.

| Proposed operation | Actual dispatch and invocation | Selection and result boundary |
| --- | --- | --- |
| `run-initial` | Neither session extension; `spawn_exec_or_resume_flow` calls `stream_exec_with_env_overrides_control`. Root overrides precede `exec --color never --skip-git-repo-check --json`; model/add-dir are appended at exec scope, then `--output-last-message <temporary path>`; prompt goes through piped stdin | `ephemeral=false`, `ignore_rules=false`, `ignore_user_config=false`; no requested schema/log/output path. Temporary output is still a real mutation. JSON events and last-message completion traverse the adapter |
| `run-resume` | Resume extension, fork absent; same root/exec options and temporary output; then `resume --last -` or `resume <id> -`, prompt plus newline on stdin | Adapter exposes last/id, not wrapper `All`; no picker or silent new session. Parent exec options are parsed before resume, so exact child-path acquisition alone misses part of the actual invocation |
| `run-fork` | Fork extension, resume absent; `spawn_fork_v1_flow` launches bare `codex app-server` through `CodexAppServer`; analytics-default flag is false/absent; stdio is piped, cwd/env/home applied to child | Initialize with client name `agent_api`, package version and `experimentalApi=true`; last performs paged `thread/list`; id uses its bytes directly; then `thread/fork`, followed by `turn/start` on the returned new id. This is **not** `codex exec fork` |

Working directory precedence is request, backend default, then captured run-start cwd; unresolved
cwd fails. Initial/resume model precedence is request model override, backend model, then upstream
configuration. Model-specific wrapper reasoning defaults can emit root `--config` values; these
are not caller-free arbitrary config approval. Nonempty add-dir requests are normalized relative
to effective cwd, capability-probed with the effective environment, then emitted on initial/resume;
runtime refusal is an error, not a successful run without requested roots. Fork rejects nonempty
add-dirs and accepted request model overrides before spawn. Backend `config.model` is not forwarded
by the fork path; no claim that it overrides a forked model is proposed. Environment overlays and
`CODEX_HOME` affect configuration, authentication, provider and session selection in every flow;
they are effects inputs even without corresponding argv tokens.

| Policy variant, in every flow unless stated | Accepted behavior / rejection |
| --- | --- |
| Noninteractive absent or true | Defaults true; approval must be absent or `never`; adapter selects never. Sandbox defaults `workspace-write`; explicit modes are `read-only`, `workspace-write`, `danger-full-access` |
| Noninteractive false | Optional approval is exactly `untrusted`, `on-failure`, `on-request`, `never`; absent means upstream effective policy. No generic interactive picker or approval-response interface is promised; approval-required/error terminal handling needs proof |
| External sandbox absent or false | Normal path; extension is admitted only when backend opt-in advertises support |
| External sandbox true | Requires backend opt-in, rejects explicitly noninteractive=false and every `backend.codex.exec.*` key; initial/resume emit bypass and omit conflicting safety argv, fork sends never/danger-full-access. Adapter emits the existing warning; this is not host sandbox authorization |
| Model/add-dirs/env/timeout/cancel | Typed validation and supported/rejected combinations remain controlling; source-bound tests must cover them. Zero effective timeout means no timeout; cancellation and timeout remain distinct outcomes |

Pinned upstream [shared options](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/utils/cli/src/shared_options.rs)
inherit root model/provider/profile/cwd only when local values are absent; local safety selection
wins, root worktree/bypass-hook flags combine, root images/add-dirs prepend. Root config overrides
prepend before local config and feature conversions. `strict-config` combines by OR. Root remote
intent is rejected for exec. Root approval/search/no-alt-screen/no-daemon are outside that exec
inheritance call. In particular the wrapper emits root `--ask-for-approval`, but pinned
[exec source](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/exec/src/lib.rs)
sets headless approval to never and has an auto-review/config exception. The default never trace
therefore does not demonstrate a failure; nor does accepting the root flag prove an explicit
noninteractive=false approval request is honored. Require an effective-policy test for each accepted
approval value and auto-review interaction; repair the translation if that test demonstrates loss.
Do not mark an intentional requested approval policy as a harmless ignored flag to close the row.
Root search/no-daemon/UI controls can receive explicit no-headless-effect/denial dispositions only
within a reviewed whole invocation boundary. App-server has different root handling; never inherit
exec conclusions by spelling.

The bounded compatibility follow-up now preserves normal initial/resume Never intent with an
explicit approval-policy config override, including the upstream AutoReview exception, while leaving
reviewer choice intact. An isolated offline pinned-binary observation reproduced the original loss
and the effective corrected policy; production adapter fixtures cover initial, resume-last and
resume-id with default/explicit Never, absent policy and external bypass. The disposition of explicit
non-Never exec/resume requests remains pending a separate maintainer decision; no silent policy
substitution, automatic-review enablement or protocol rerouting is authorized by this repair.
The [streaming contract](../../specs/codex-streaming-exec-contract.md) records the adapter translation.

## Exact proposed operation and mode declarations

One enrollment: agent `codex`, version `0.156.1`, target **only**
`x86_64-unknown-linux-musl`, lifecycle maintenance. GNU is the wrapper build/execution host and
must be bound separately in evidence. There are three operations and five selectors:

| selector | command_path | mode | Exact discriminator | operation / templates |
| --- | --- | --- | --- | --- |
| `initial-json` | `codex exec` | `adapter-initial-json` | Neither session extension; acquired zero-arity `--json` presence | `run-initial`: CORE + ADMIN |
| `resume-last-json` | `codex exec resume` | `adapter-resume-last-json` | Resume last, fork absent; zero-arity `--json`, `--last`; no explicit session id | `run-resume`: CORE + SESSION + ADMIN |
| `resume-id-json` | `codex exec resume` | `adapter-resume-id-json` | Resume id, fork absent; zero-arity `--json`; id nonblank; `--last` absent | `run-resume`: CORE + SESSION + ADMIN |
| `fork-last-stdio` | `codex app-server` | `adapter-fork-last-stdio` | Fork last, resume absent; owned stdio and last-list selection | `run-fork`: CORE + SESSION + MANAGED + ADMIN |
| `fork-id-stdio` | `codex app-server` | `adapter-fork-id-stdio` | Fork id, resume absent; owned stdio and explicit source id | `run-fork`: CORE + SESSION + MANAGED + ADMIN |

These names and discriminators are a proposed exact fixed-profile contract, **not supported by
the current resolver**. Use existing selector/command_path/mode fields; do not add adapter_flow or
invocation_boundary registry fields. Each operation serves exactly its table rows, has `shared_mapping=mapped`,
`forwarding=false`, no NA permissions or overrides, and a complete BASE promise. The sole claimed
capability mapping is `agent_api.run -> [run-initial, run-resume, run-fork]`. Denied combinations
are tested within these boundaries; no extra diagnostic/excluded modes quietly relax run promises.
Normal/noninteractive/external variants and finite safety choices are obligatory subcases of each
selector, not hidden extra operations. Discriminators are disjoint on the validated typed request;
malformed, unsupported or overlapping selectors refuse. No command-prefix match decides a mode.

ADMIN covers the initial/resume output-file effect and potential selected worktree effect. For fork
it covers the explicit creation/persistence of a new thread, including partial creation if turn start
fails. SESSION covers the chosen/continued/new identity. MANAGED covers protocol ownership; CORE
retains the streaming run promise. Internal `thread/list` does not promise a public QUERY records
interface, and stdio does not add REMOTE. If a reviewed configuration actually starts listeners or
other persistent services, it is outside the proposed boundary and unresolved, not implicitly covered
by this table.

BASE is exactly `R1,V1,G1,E1,A1,X1,X2,X3,X4`. Add unique template obligations and per-operation `M`:

| Operation | Obligations beyond BASE, including mapping | Planned due tuples per target |
| --- | --- | --- |
| `run-initial` | OUT3, LC2, OUT1, LC1, FX1, M | 15 |
| `run-resume` | OUT3, LC2, S1, OUT1, LC1, FX1, M | 16 |
| `run-fork` | OUT3, LC2, S1, OUT4, LC4, OUT1, LC1, FX1, M | 18 |

Thus49 proposed operation/obligation/target tuples, **not49 test cases** and not an implemented
verdict profile. Both selector variants must be covered for each shared resume/fork obligation;
identical obligation ids across operations do not merge. Additional effects established during
review increase this population. Templates do not waive evidence for the inherited BASE promises.
`M` is the [run protocol](../../specs/unified-agent-api/run-protocol-spec.md)'s minimum semantics
through the production adapter: prompt/capability/extension validation before spawn, event/finality,
process exit, cancellation and exactly-once completion. Existing [extension semantics](../../specs/unified-agent-api/extensions-spec.md)
include selector mutual exclusion, last/id behavior, no-match errors, model/add-dirs conflicts,
noninteractive and external-sandbox requirements. Do not manufacture one verified M from a wrapper
fixture while leaving another adapter flow unproved.

## Representation, acquisition and guarded freeze

Reuse existing `GitInputs -> Source -> Proposal`, per-operation rows in P, semantic classification
basis and O raw file references. There is no second policy file, evaluator network lookup, new
service, classifier DSL, CI job or required check. The appendix is documentation, not runtime input.

1. **Fixed profile interpretation.** The private profile recognizes exactly the five literal
   selector/path/mode triples above and the complete three-operation mapping. Its source-owned
   decoder checks the exact typed request conditions in the flow table and actual serializer
   shape. It does not infer effects from arbitrary mode strings or first-match predicates.
   `--json` is interpreted only at its exact acquired path and zero arity; last/id discrimination
   comes from the typed adapter request. Generic resolution keeps its existing refusals. Bind the
   profile implementation, decoder, serializer/parser, request contracts and context constants by
   content in the existing per-operation basis. Registry/P is still the sole inventory of approved
   operation/row/template assignments: the profile defines the finite admitted execution shape,
   not a second classification list. This is enough mechanical meaning without two new fields.
2. **Controlled context, unchanged public SDK.** CP-2 proposes qualification specifically for the
   following pilot-owned context, invoked through unchanged production adapters. Calls outside
   it retain their ordinary SDK behavior but receive no qualification claim from this pilot.
   The normative whole-acquired-value/default rule currently does not express this restriction:
   approve that explicit scope clarification before implementing it. If the maintainer requires
   unrestricted public run/config behavior to be qualified instead, reject CP-2 and retain unknowns;
   no narrower fixture can establish that broader claim.

   | Input | Exact proposed admitted regime; refusal outside it |
   | --- | --- |
   | Producer environment | Start with a cleared environment, then admit only absolute pinned PATH, HOME, CODEX_HOME, TMPDIR, XDG_CONFIG_HOME, XDG_DATA_HOME, XDG_CACHE_HOME, LANG=`C.UTF-8`, TZ=`UTC` and injected CODEX_API_KEY; all home/XDG/temp paths are private pilot roots, CODEX_HOME=H. CODEX_BINARY and RUST_LOG are absent here. Backend binary is explicitly the absolute pinned executable B and backend codex_home is H in every flow. No proxy, provider/base-url, cloud, remote, feature or config environment overrides. Bind credential name/presence/intent, never secret bytes; missing credentials refuse live positive proof, with explicit absence for negative local fixtures |
   | Effective child environment | Initial/resume and their executed capability probes inherit that producer environment, then the wrapper generates CODEX_BINARY=B, CODEX_HOME=H and RUST_LOG=`error`, then applies the normalized per-call overlay. The direct managed-fork child inherits the producer environment, then applies backend code_home and its normalized overlay; it generates neither CODEX_BINARY nor RUST_LOG, so both remain absent in this fork context. Validate these separate final environments, not only the producer allowlist; overlay precedence and admissible values are defined below |
   | Home and project | New private home H; project W with only scenario-owned Git metadata, source fixture files and session/output artifacts. No ancestor/project/user AGENTS, rules, `.codex`, profiles, hook/skill/plugin/MCP files or symlinks; no preexisting home files except the exact config below. No `/etc/codex` policy directory or unbound managed policy/cloud response. Pin filesystem/host source correspondence; if effective-loader inputs cannot be inventoried or isolated, refuse rather than assuming absence |
   | Config | Only H/config.toml, generated from source-owned constants: model_provider=`openai`, model=`gpt-5`, approval_policy=`never`, sandbox_mode=`workspace-write`, web_search=`disabled`, analytics.enabled=false. No model_providers/base-url overrides, profiles, hooks, notify, apps, plugins, MCP servers or external instruction files. Every boolean key in the pinned config schema's finite features.properties set is explicitly false (including aliases); unknown/additional feature keys refuse. Bind the full generated file bytes and effective feature/default projection. If this disables a feature actually required for the promised flow, stop and bring the exact needed change for review; do not auto-enable defaults |
   | Model and safety variants | Existing typed model string forwarding and error semantics remain; generated reasoning config is exactly model_reasoning_effort=`medium`, model_reasoning_summary=`auto`, model_verbosity=`low` for the existing wrapper model-family rules, no other caller config. Test request-over-backend precedence and open-value runtime rejection. Three existing sandbox values and the accepted noninteractive/approval/external combinations above are all due; they override the controlled default through the actual adapter, not through a test replacement |
   | Cwd, add-dirs, sessions, artifacts | Absolute private W is the default; request/backend/captured-cwd variants use declared private fixture roots. Add-dirs must name declared fixture roots; outside paths are rejected by the qualification context before invocation. Seed sessions are created by the pinned binary in H/W and recorded in before/after manifests; last/id never consult another home. Initial/resume temporary last-message output and fork session persistence are allowed effects. No caller profile/features/hook bypass/output-schema/worktree/terminal/listener/remote argv is admitted |
   | Context currentness | Compare approved context bytes, declared roots, absence constraints and complete effective source/default facts, including generated child environment and overlays, before each run and again before evidence packaging. Record mutations under declared roots, credential-safe diagnostics and exact binary/source identity. Extra input, unavailable effective config, drift or unexpected service/listener refuses qualification; it does not cause the SDK to reject ordinary callers or broaden P |

   Effective environment precedence is source-defined. The [request normalizer](../../../crates/agent_api/src/backend_harness/normalize.rs)
   first merges backend defaults then request entries, so request wins on duplicate keys.
   For initial/resume, [CommandEnvironment](../../../crates/codex/src/home.rs) injects CODEX_BINARY
   from the configured executable and CODEX_HOME from the configured home;
   [defaults](../../../crates/codex/src/defaults.rs) inject RUST_LOG=`error` only when RUST_LOG is
   absent from the **producer process** environment. [Streaming](../../../crates/codex/src/exec/streaming.rs)
   then applies the normalized per-call overlay last. [Executed capability probes](../../../crates/codex/src/client_core.rs)
   use the same injection/overlay order; bind their resolved executable and effective cwd too.
   The profile admits overlay entries only when they preserve the exact final context values:
   CODEX_BINARY=B and RUST_LOG=`error` on exec/probes, CODEX_HOME=H on all flows, and the admitted
   producer values for other keys. Fork overlays must omit CODEX_BINARY and RUST_LOG. A foreign
   binary/home, changed or empty diagnostic value, new key or unbound credential intent refuses
   qualification even when the SDK would accept it. Overlay CODEX_BINARY does not select the
   already configured command executable; both the actual executable and environment intent must
   independently remain bound to B.

   Fork's [adapter](../../../crates/agent_api/src/backends/codex/fork.rs) constructs StdioServerConfig
   directly and calls CodexAppServer::with_capabilities; [JSON-RPC launch](../../../crates/codex/src/mcp/jsonrpc.rs)
   applies code_home first and config.env afterward, without CommandEnvironment injection.
   It does not traverse [AppRuntime::into_launcher](../../../crates/codex/src/mcp/app.rs), whose
   [merge_stdio_env](../../../crates/codex/src/mcp/runtime.rs) instead merges code_home, base env,
   then runtime env before the same transport launch. That neighboring app path also generates
   neither CODEX_BINARY nor RUST_LOG; its precedence must not be silently substituted for the
   direct fork path. Include these normalizer/injection/default/probe/launcher sources in the
   existing content-bound source closure. BASE evidence must check the final effective environment
   and executable at actual initial/resume, probe and fork spawns, with adverse overlay/home/log
   drift controls and credential-safe capture. A producer-only allowlist or source inspection alone
   does not prove that execution boundary. This requirement changes no public SDK behavior.

   These are concrete proposed policy constants, not claims that this schema/configuration has
   executed successfully. The newly pinned config schema corroborates names/types; it does not
   attest effective defaults, binary behavior or model availability. No live model request is
   authorized here. Generic wrapper arbitrary-value claims remain accounted and unqualified.
   The profile validates context and allowed/denied invocation families mechanically; rationale
   strings do not approve subsets. Changing this boundary requires maintainer P/basis review.
3. **Ancestry and basis.** For resume, explicitly add exact `codex exec` ancestor surfaces to the
   exact `codex exec resume` + effective-global population. Preserve original identities and source
   provenance, including parent positional/command grammar with a denied competing-initial intent.
   This conservative closure includes more than emitted tokens; it avoids disappearing effectful
   parent inputs. Validate acquired aliases/arity on each selected path, never all descendant paths.
   Extend classification ownership accordingly; current rows cannot own ancestor units. Derive one
   basis per operation from all served modes on the one target, the union of owned identities,
   original + normalized rows/claims/features/rules/schema and bound profile/context/protocol facts.
   Exclude registry bytes, P/O/records/adoption/basis literals from the implementation-source digest
   to avoid self-hash cycles; protected registry authority remains separately bound by the existing
   context. A content digest establishes currentness, never grants authority. Identical
   surfaces in two modes remain one operation unit with both mode applicability facts; across
   operations they remain separate obligations. Preserve all14 current input references in O.
4. **Protocol facts.** CLI acquisition yields no protocol-method SurfaceIdentity. Add one bounded,
   versioned protocol facts report under `reports/0.156.1/`, acquired with existing raw references
   and source provenance, containing the generated upstream schema and exact selected protocol
   subset correspondence. Bind its complete raw digest in O and semantic/content digest in the
   fork basis (do not force schema numeric payloads through unsupported B.7 JSON projection).
   Use a fixed selected subset in source, not another maintainer policy inventory. The report must
   establish handshake, fork/list/turn/cancel/shutdown parameters, responses and notifications as
   actually used; missing or incompatible schema refuses. Adding this input is a proposed
   acquisition/format amendment; today the14 inputs contain no such report. Do not put it among
   snapshot target files or invent RPC names as CLI flags. The selected CLI surface count does not
   certify protocol completeness.
5. **Fail-closed readiness.** Reject unrecognized fixed triples/context, arbitrary predicates, inconsistent raw
   data, stale basis, unknown effective defaults, missing protocol facts or U-* row prerequisites.
   Row templates cover the whole mechanically bound unit/default regime. No rationale condition
   can approve a subset. With today's unchanged whole-unit format the U-config/feature/profile/
   hooks/provider rows stay classification_required. Only after the reviewed boundary extension
   changes and enforces the selected regime may a fresh explicit P row cover the resulting full
   allowed/denied regime. Generic wrapper arbitrary-value claims remain unqualified. A present
   malformed or partial acquisition is an error, never a P-only absence.
6. **Authority.** Resolve the installed fixed isolated profile before writers use its scoped registry.
   No profile exists now. Widen fixed scope only after exact declarations and proof are approved;
   rows or caller input cannot widen it. Preparation, refreeze, closeout and retained promotion use
   the same resolution/derive path. Complete mapping plus structural acceptance is necessary but
   cannot mechanically prove all production control paths or independent adequacy.

Current intake supports only one exact mode/target for classification-bearing policy and acquired
`--json` presence, refuses new mapped selections including P-only, and has no source-owned typed
flow/context/config/protocol interpretation. Keep these refusals until the above prerequisites are implemented;
accepting five arbitrary mode strings would not implement this design. Exact acquired aliases only;
source-only `--experimental-json`, `--not-so-yolo`, `--yolo`, hidden remote-control/daemon switches
remain acquisition gaps, never silently normalized. Unreviewed added facts refuse, even if an old
row happened to use the same flag spelling.

## Evidence and bounded implementation order

First establish pinned-runtime compatibility through the production adapter: exact fork message
shapes/terminality/handshake/cancel and effective approval policy, using the source discrepancies
in the appendix. Repair demonstrated failures with production-path regression evidence. Then
implement/source-review CP-1/CP-2's fixed selection/context, ancestry, per-operation basis and honest
protocol acquisition. Prove five disjoint mode populations, changed/added/normalized-out facts,
malformed versus truly absent acquisition, optional-field byte compatibility, and initial-only
mapping refusal. No installed profile or positive verdict is enabled at this stage.

Resolve appendix prerequisites against the exact controlled context and defaults. Demonstrate denied
terminal/action/remote/provider/config intents without spawn or unintended effects, and repair any
lost approval/model/directory/session semantics shown by production-path tests. Needed compatibility
or unknown effect decisions return to the maintainer; never resolve them by hiding a row. This is
also where source acquisition for hidden aliases, ThreadSource values and protocol schema belongs.

Then build the packet-specific producer using existing evidence machinery. For **every** due tuple,
name exact case ids/assertions, production functions/parsers/adapter paths, adverse/counterfactual
controls, target/binary/config/source identity and adequacy gaps. A1 needs musl binary provenance,
not merely the pinned tag or a GNU wrapper build. Source facts, producer success, authenticated
provenance and adequate independent evidence are different claims. The existing CORE producer is a
fixed synthetic twelve-group fixture (eleven obligation ids plus M), built with all features;
it is not these49 tuples and its adoption cannot be broadened by renaming it. Preserve its whole
workflow requirement. A future packet producer uses the already designed terminal-job provenance
path and exact authenticated maintainer digest endorsement after independent adequacy; do not add
a standing reviewer role or new CI job.

| Proof family | Required positive and adverse coverage |
| --- | --- |
| BASE, every operation | Typed validation and no-spawn errors; exact grammar/alias/precedence/default/config/env closure; provenance; cwd/home/auth isolation; error redaction and bounded output; supported/rejected policy variants |
| CORE | Actual production JSON event parser for exec/resume and selected notification mapper for fork; known/unknown/malformed messages, order/identity, EOF versus success, terminal error, streaming before completion, buffering/drain/drop/reaping |
| ADMIN | Temporary and explicit output paths where promised; before/after scope, denied writes, partial output/worktree/thread creation on failure, cleanup, retries only as promised; no claim of transactional fork rollback |
| SESSION | Resume last/id and fork last/id separately; no-match/not-found/conflict/invalid id, returned identity versus continued identity; deterministic fork paging/ties/repeated cursor; no picker or fallback |
| MANAGED | Real client/process handshake/readiness; request correlation, selected notifications, cancellation, approval-required terminal path, disconnect, shutdown/reaping, startup/turn timeout and drop ownership |
| M | Production adapter run request, capability/extension admission, prompt delivery, normalized events, cancellation, exactly-once completion and process exit across all five selectors |

Finally enable the reviewed fixed scope and attempt the real prepare/refreeze/evidence/closeout/
retained-promotion chain, with appropriate stand-down committed to staging before actual packet
closeout. All56 carried rows and nonshrinking retained O work survive; old ordinary bootstrap/
closeout is not depth proof. Legitimate P-only state remains possible only with genuine absence
and preserved carried work; partial data cannot masquerade as absence. No enrollment, profile,
writer permission, evidence adoption or production origin is installed by this docs task.

Eventual code changes use the existing12 local gate groups and both GPT-6.1 Sol High review lanes
at standard/default speed. This docs-only design is checked through source/population/hash/link
reconciliation, hygiene and diff checks, then both owner-arranged frozen-source reviews. No broad
Rust run or successful packet execution is claimed.
