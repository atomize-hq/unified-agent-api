# Complete pilot: exact surfaces and proposed dispositions

Status: proposal accompanying the [complete policy](support-depth-complete-pilot-policy.md),
2026-10-25, baseline `1f38133ffc68f4c7a254f05a6ce63b1543f619d3`. PI-P1's30 approved
initial assignments are preserved separately in the table. Every other proposal requires
maintainer approval. No assignment here is adequate execution proof or an installed registry row.
The [previous54-row appendix](support-depth-packet-intake-classifications.md) remains the record
of that approval. This appendix accounts for new flow populations without redefining that approval.

## Population and table conventions

Current exact-path acquisition would own54 initial identities,48 resume identities and42 app-server
CLI identities:144 operation/surface edges,94 distinct CLI identities. It does **not** currently
accept a complete mapped declaration. Resume's48 are incomplete for the actual invocation:
the wrapper emits options before `resume` at parent `codex exec` scope. The policy therefore
proposes an explicit additional29-parent-identity closure, making resume77 and the three operations
173 edges. Parent command/PROMPT grammar is retained conservatively and competing initial intent
must be denied. This is a proposed ownership/basis extension, not a prefix-matching rule or a
claim that current intake already selects those parents. Last/id modes share the operation's
surface population but require separate selector and evidence subcases.

All four original target shards and the normalized union agree on applicability for every observed
selected identity: `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`,
`aarch64-apple-darwin`, `x86_64-pc-windows-msvc`. This pilot enrolls **only the first**; availability
on others is not enrollment or proof. Wrapper-only `--full-auto` has no observed target row;
its unbounded wrapper claim applies to the selected target too. Every coverage level, including
passthrough, is retained. No selected row has an unsupported literal claim, and unclaimed means
no literal coverage row at that exact identity. No name exclusion removes an identity.

`O+U` means observed in original shards and union; `O only` is an original local row normalized
out of the union because it matches a root flag; `claim only` has no observed row. A command row
is `commands` for one-token paths and `subcommands` for resume. `G` rows are effective globals,
`I` exact exec, `R` exact resume, `F` exact app-server. Initial=G+I; current resume=G+R;
proposed resume=G+I+R; fork CLI=G+F. Root PROMPT and nested exec fork/review/app-server tooling
commands are not members. They are not the adapter's fork route. No RPC method is invented as a
SurfaceIdentity. Protocol population is a separate required acquired-facts gap below.

Values are help placeholders, never a complete accepted enumeration or default. Short aliases
are only the acquired spellings. Source-only aliases still refuse. Old assignment is the exact
PI-P1 initial disposition, or `new`; it applies to initial only. The proposal column applies to
exec/resume G rows and path-local rows. The final column gives fork's separately assessed G row;
no blanket inherited CORE is used for app-server root flags. Codes are review dispositions, not a
new policy syntax or executable classifier. Every P-* row remains pending its stated boundary,
source and evidence prerequisites; U-* remains classification_required today.

| Id | surface_kind | command_path | surface_id | Acquired alias / arity | Provenance | Literal wrapper level | PI-P1 initial | Proposal | Fork global proposal |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| G01 | `global_flags` | `codex` | `--add-dir` | — / value DIR | O+U | explicit | CORE | P-core | U-root-fork |
| G02 | `global_flags` | `codex` | `--approve-for-me` | — / presence | O+U | passthrough | CORE | P-core | U-root-fork |
| G03 | `global_flags` | `codex` | `--ask-for-approval` | -a / value APPROVAL_POLICY | O+U | explicit | U-root | U-approval | U-root-fork |
| G04 | `global_flags` | `codex` | `--cd` | -C / value DIR | O+U | explicit | CORE | P-core | U-root-fork |
| G05 | `global_flags` | `codex` | `--config` | -c / value key=value | O+U | passthrough | U-config | U-config | U-config |
| G06 | `global_flags` | `codex` | `--dangerously-bypass-approvals-and-sandbox` | — / presence | O+U | explicit | CORE | P-core | U-root-fork |
| G07 | `global_flags` | `codex` | `--dangerously-bypass-hook-trust` | — / presence | O+U | passthrough | U-hooks | U-hooks | U-root-fork |
| G08 | `global_flags` | `codex` | `--disable` | — / value FEATURE | O+U | passthrough | U-feature | U-feature | U-feature |
| G09 | `global_flags` | `codex` | `--enable` | — / value FEATURE | O+U | passthrough | U-feature | U-feature | U-feature |
| G10 | `global_flags` | `codex` | `--full-auto` | — / unobserved | claim only | explicit | U-claim | U-claim | U-claim |
| G11 | `global_flags` | `codex` | `--help` | -h / presence | O+U | explicit | U-terminal | P-terminal | P-terminal |
| G12 | `global_flags` | `codex` | `--image` | -i / value FILE | O+U | explicit | CORE | P-core | U-root-fork |
| G13 | `global_flags` | `codex` | `--local-provider` | — / value OSS_PROVIDER | O+U | explicit | U-provider | U-provider | U-root-fork |
| G14 | `global_flags` | `codex` | `--model` | -m / value MODEL | O+U | explicit | CORE | P-core | U-root-fork |
| G15 | `global_flags` | `codex` | `--no-alt-screen` | — / presence | O+U | unclaimed | U-root | P-root | U-root-fork |
| G16 | `global_flags` | `codex` | `--no-daemon` | — / presence | O+U | passthrough | U-root | P-root | U-root-fork |
| G17 | `global_flags` | `codex` | `--oss` | — / presence | O+U | explicit | U-provider | U-provider | U-root-fork |
| G18 | `global_flags` | `codex` | `--profile` | -p / value CONFIG_PROFILE_V2 | O+U | explicit | U-profile | U-profile | U-profile |
| G19 | `global_flags` | `codex` | `--remote` | — / value ADDR | O+U | explicit | CORE-denial | P-denied-remote | U-transport |
| G20 | `global_flags` | `codex` | `--remote-auth-token-env` | — / value ENV_VAR | O+U | explicit | CORE-denial | P-denied-remote | U-transport |
| G21 | `global_flags` | `codex` | `--sandbox` | -s / value SANDBOX_MODE | O+U | explicit | CORE | P-core | U-root-fork |
| G22 | `global_flags` | `codex` | `--search` | — / presence | O+U | explicit | U-root | P-root | U-root-fork |
| G23 | `global_flags` | `codex` | `--strict-config` | — / presence | O+U | passthrough | CORE | P-core | U-root-fork |
| G24 | `global_flags` | `codex` | `--version` | -V / presence | O+U | explicit | U-terminal | P-terminal | P-terminal |
| G25 | `global_flags` | `codex` | `--worktree` | — / presence | O+U | passthrough | CORE+ADMIN | P-admin | U-root-fork |
| I01 | `commands` | `codex exec` | `exec` | — / command | O+U | explicit | CORE | P-core | — |
| I02 | `flags` | `codex exec` | `--add-dir` | — / value DIR | O only | unclaimed | CORE | P-core | — |
| I03 | `flags` | `codex exec` | `--approve-for-me` | — / presence | O only | unclaimed | CORE | P-core | — |
| I04 | `flags` | `codex exec` | `--cd` | -C / value DIR | O only | unclaimed | CORE | P-core | — |
| I05 | `flags` | `codex exec` | `--color` | — / value COLOR | O+U | explicit | CORE | P-core | — |
| I06 | `flags` | `codex exec` | `--config` | -c / value key=value | O only | unclaimed | U-config | U-config | — |
| I07 | `flags` | `codex exec` | `--dangerously-bypass-approvals-and-sandbox` | — / presence | O only | unclaimed | CORE | P-core | — |
| I08 | `flags` | `codex exec` | `--dangerously-bypass-hook-trust` | — / presence | O only | unclaimed | U-hooks | U-hooks | — |
| I09 | `flags` | `codex exec` | `--disable` | — / value FEATURE | O only | unclaimed | U-feature | U-feature | — |
| I10 | `flags` | `codex exec` | `--enable` | — / value FEATURE | O only | unclaimed | U-feature | U-feature | — |
| I11 | `flags` | `codex exec` | `--ephemeral` | — / presence | O+U | explicit | CORE | P-core | — |
| I12 | `flags` | `codex exec` | `--help` | -h / presence | O only | unclaimed | U-terminal | P-terminal | — |
| I13 | `flags` | `codex exec` | `--ignore-rules` | — / presence | O+U | explicit | CORE | P-core | — |
| I14 | `flags` | `codex exec` | `--ignore-user-config` | — / presence | O+U | explicit | CORE | P-core | — |
| I15 | `flags` | `codex exec` | `--image` | -i / value FILE | O only | unclaimed | CORE | P-core | — |
| I16 | `flags` | `codex exec` | `--json` | — / presence | O+U | explicit | CORE | P-core | — |
| I17 | `flags` | `codex exec` | `--local-provider` | — / value OSS_PROVIDER | O only | unclaimed | U-provider | U-provider | — |
| I18 | `flags` | `codex exec` | `--model` | -m / value MODEL | O only | unclaimed | CORE | P-core | — |
| I19 | `flags` | `codex exec` | `--oss` | — / presence | O only | unclaimed | U-provider | U-provider | — |
| I20 | `flags` | `codex exec` | `--output-last-message` | -o / value FILE | O+U | explicit | CORE+ADMIN | P-admin | — |
| I21 | `flags` | `codex exec` | `--output-schema` | — / value FILE | O+U | explicit | CORE | P-core | — |
| I22 | `flags` | `codex exec` | `--profile` | -p / value CONFIG_PROFILE_V2 | O only | unclaimed | U-profile | U-profile | — |
| I23 | `flags` | `codex exec` | `--sandbox` | -s / value SANDBOX_MODE | O only | unclaimed | CORE | P-core | — |
| I24 | `flags` | `codex exec` | `--skip-git-repo-check` | — / presence | O+U | explicit | CORE | P-core | — |
| I25 | `flags` | `codex exec` | `--strict-config` | — / presence | O only | unclaimed | CORE | P-core | — |
| I26 | `flags` | `codex exec` | `--thread-source` | — / value SOURCE | O+U | passthrough | U-source | U-source | — |
| I27 | `flags` | `codex exec` | `--version` | -V / presence | O only | unclaimed | U-terminal | P-terminal | — |
| I28 | `flags` | `codex exec` | `--worktree` | — / presence | O only | unclaimed | CORE+ADMIN | P-admin | — |
| I29 | `positional_args` | `codex exec` | `PROMPT` | — / positional | O+U | explicit | CORE | P-core | — |
| R01 | `flags` | `codex exec resume` | `--all` | — / presence | O+U | explicit | new | P-session-deny | — |
| R02 | `flags` | `codex exec resume` | `--config` | -c / value key=value | O only | unclaimed | new | U-config | — |
| R03 | `flags` | `codex exec resume` | `--dangerously-bypass-approvals-and-sandbox` | — / presence | O only | unclaimed | new | P-core | — |
| R04 | `flags` | `codex exec resume` | `--dangerously-bypass-hook-trust` | — / presence | O only | unclaimed | new | U-hooks | — |
| R05 | `flags` | `codex exec resume` | `--disable` | — / value FEATURE | O only | unclaimed | new | U-feature | — |
| R06 | `flags` | `codex exec resume` | `--enable` | — / value FEATURE | O only | unclaimed | new | U-feature | — |
| R07 | `flags` | `codex exec resume` | `--ephemeral` | — / presence | O+U | explicit | new | P-core | — |
| R08 | `flags` | `codex exec resume` | `--help` | -h / presence | O only | unclaimed | new | P-terminal | — |
| R09 | `flags` | `codex exec resume` | `--ignore-rules` | — / presence | O+U | explicit | new | P-core | — |
| R10 | `flags` | `codex exec resume` | `--ignore-user-config` | — / presence | O+U | explicit | new | P-core | — |
| R11 | `flags` | `codex exec resume` | `--image` | -i / value FILE | O only | unclaimed | new | P-core | — |
| R12 | `flags` | `codex exec resume` | `--json` | — / presence | O+U | explicit | new | P-core | — |
| R13 | `flags` | `codex exec resume` | `--last` | — / presence | O+U | explicit | new | P-session | — |
| R14 | `flags` | `codex exec resume` | `--model` | -m / value MODEL | O only | unclaimed | new | P-core | — |
| R15 | `flags` | `codex exec resume` | `--output-last-message` | -o / value FILE | O+U | explicit | new | P-admin | — |
| R16 | `flags` | `codex exec resume` | `--output-schema` | — / value FILE | O+U | passthrough | new | P-core | — |
| R17 | `flags` | `codex exec resume` | `--skip-git-repo-check` | — / presence | O+U | explicit | new | P-core | — |
| R18 | `flags` | `codex exec resume` | `--strict-config` | — / presence | O only | unclaimed | new | P-core | — |
| R19 | `flags` | `codex exec resume` | `--thread-source` | — / value SOURCE | O+U | passthrough | new | U-source | — |
| R20 | `flags` | `codex exec resume` | `--worktree` | — / presence | O only | unclaimed | new | P-admin | — |
| R21 | `positional_args` | `codex exec resume` | `PROMPT` | — / positional | O+U | explicit | new | P-core | — |
| R22 | `positional_args` | `codex exec resume` | `SESSION_ID` | — / positional | O+U | explicit | new | P-session | — |
| R23 | `subcommands` | `codex exec resume` | `resume` | — / command | O+U | explicit | new | P-session | — |
| F01 | `commands` | `codex app-server` | `app-server` | — / command | O+U | explicit | new | P-managed-admin | — |
| F02 | `flags` | `codex app-server` | `--analytics-default-enabled` | — / presence | O+U | explicit | new | U-analytics | — |
| F03 | `flags` | `codex app-server` | `--code-mode-host` | — / value URL | O+U | passthrough | new | U-transport | — |
| F04 | `flags` | `codex app-server` | `--config` | -c / value key=value | O only | unclaimed | new | U-config | — |
| F05 | `flags` | `codex app-server` | `--disable` | — / value FEATURE | O only | unclaimed | new | U-feature | — |
| F06 | `flags` | `codex app-server` | `--enable` | — / value FEATURE | O only | unclaimed | new | U-feature | — |
| F07 | `flags` | `codex app-server` | `--help` | -h / presence | O only | unclaimed | new | P-terminal | — |
| F08 | `flags` | `codex app-server` | `--listen` | — / value URL | O+U | explicit | new | U-transport | — |
| F09 | `flags` | `codex app-server` | `--stdio` | — / presence | O+U | passthrough | new | P-managed | — |
| F10 | `flags` | `codex app-server` | `--strict-config` | — / presence | O only | unclaimed | new | P-managed | — |
| F11 | `flags` | `codex app-server` | `--ws-audience` | — / value AUDIENCE | O+U | explicit | new | U-transport | — |
| F12 | `flags` | `codex app-server` | `--ws-auth` | — / value MODE | O+U | explicit | new | U-transport | — |
| F13 | `flags` | `codex app-server` | `--ws-issuer` | — / value ISSUER | O+U | explicit | new | U-transport | — |
| F14 | `flags` | `codex app-server` | `--ws-max-clock-skew-seconds` | — / value SECONDS | O+U | explicit | new | U-transport | — |
| F15 | `flags` | `codex app-server` | `--ws-shared-secret-file` | — / value PATH | O+U | explicit | new | U-transport | — |
| F16 | `flags` | `codex app-server` | `--ws-token-file` | — / value PATH | O+U | explicit | new | U-transport | — |
| F17 | `flags` | `codex app-server` | `--ws-token-sha256` | — / value HEX | O+U | explicit | new | U-transport | — |

## Exact remaining24 initial dispositions

The previous U labels remain stable; proposals below explain a finite resolution route, never a
partial value approval in rationale. P-* is a proposed template assignment only after CP-2's
mechanical boundary exists and the maintainer reviews the whole resulting allowed/denied regime.
U-* requires the named source/representation/repair work before any assignment. Defaults and
absent flags are part of every row. None is EXCLUDED, NA, debt or a waiver of M.

| Original label / exact table identities | Recommendation and templates after prerequisites | What must actually change or be proved |
| --- | --- | --- |
| U-config: G05, I06 | Retain U-config now. Recommend CORE for the finite adapter-generated reasoning/approval config and complete denial of caller arbitrary config under CP-2 | Source-bound guard must recognize exact generated keys/values and reject other CLI keys, env/home/project config or profile effects outside the bound context. Existing free-string forwarding is not approved. Review effective configuration, not just emitted argv |
| U-hooks: G07, I08 | Retain U-hooks; proposed boundary forbids hook-trust bypass and unreviewed hook execution entirely, permitting a later explicit CORE denied/no-effect row | Pin hooks-disabled effective configuration and hook sources/absence; guard it before spawn. The acquired all-features snapshot enables hooks, so feature acquisition does not establish the guarded runtime default |
| U-feature: G08,G09,I09,I10 | Retain U-feature; later explicit CORE denial for all caller feature toggles, with the bound effective feature regime reviewed | Bind the complete feature/default regime, not only flag names. Generated changes, user/project config and environment cannot bypass the guard; unknown/new features refuse. Do not survey or approve arbitrary enabled/disabled feature strings |
| U-terminal: G11,G24,I12,I27 | Recommend P-terminal: explicit CORE rejected-action/no-spawn row in the run boundary | Help/version cannot complete an event run. Guard every alias/action spelling using coherent acquisition and fixed typed boundary, keep them in O. Separate generic wrapper help/version remains diagnostic work, not qualified by this run |
| U-provider: G13,G17,I17,I19 | Retain U-provider; later explicit CORE denial of OSS/local-provider switches and bind one reviewed effective normal provider context | Arbitrary model ids still use the existing typed model promise; they do not approve provider changes. Exact endpoint/auth/config defaults and noninteractive behavior must be bound; provider interaction/picker or implicit service startup refuses |
| U-profile: G18,I22 | Retain U-profile; later explicit CORE denial of profile selection in the guarded interface | No unbound profile file is loaded. Effective home/project configuration must be frozen/validated; forbidding argv alone does not forbid implicit profile/config effects |
| U-source: I26 | Retain U-source; later explicit CORE denial of explicit thread-source override and source-bound default User | Help does not acquire ThreadSource enum/default semantics. Pin upstream protocol enum and serialization/source meaning, extend semantic facts, and verify default plus rejection. Do not invent accepted enum values |
| U-root: G03,G15,G16,G22 | G03 retains U-approval; G15/G16/G22 recommend P-root: explicit CORE denied-intent/no-headless-effect within the boundary | Root approval requires effective-policy proof for all admitted variants, repairing translation if needed. Root no-alt-screen/no-daemon/search must be rejected or proved inert without sacrificing the actual run promise. Root source omission alone is not a no-effect proof |
| U-claim: G10 | Retain U-claim; recommend reject obsolete full-auto intent at this pinned version and reconcile the explicit wrapper claim | It is absent from all original/union help and the pinned SharedCliOptions; wrapper emission still exists under its FullAuto safety option. Demonstrate version-bound rejection/no-effect and repair emitter/report truthfulness if needed. Do not remove the identity from historical accounting or infer hidden compatibility solely from missing help |

This lists2+2+4+4+4+2+1+4+1=24 exact initial unknowns. Source-informed proposals do not change
today's30-resolved/24-unresolved intake result. The recommended finite route is **deny unbounded
caller knobs and mechanically bind effective defaults**, then approve fresh whole-regime rows;
not leave those strings “conditionally approved.” If maintaining arbitrary ambient config for the
qualified interface is required, CP-2 cannot resolve these rows: broader effect review or a changed
completion claim needs an explicit maintainer decision.

## Additional flow dispositions and effect code definitions

| Code / applicable identities | Exact recommendation and evidence boundary |
| --- | --- |
| P-core | Explicit CORE proposal: each listed local run/prompt/JSON/image/model/directory/safety/config-loading unit stays within the reviewed run boundary. These are individual assignments, not name inheritance; preserve defaults, local/root precedence and denied unsupported variants. Prior30 approval is only as shown in its own column |
| P-admin | Explicit CORE+ADMIN: I20/R15 output-file mutation, I28/R20/G25 worktree mutation. The adapter's implicit temporary output makes ADMIN due even when no caller path is exposed. Worktree intent, if outside the adapter's boundary, is fully denied; wrapper worktree support and its mutation promise are not erased |
| P-denied-remote | Explicit CORE denied/no-effect for exec/resume G19/G20 (`--remote`, `--remote-auth-token-env`); the same app-server globals remain separately unresolved. No REMOTE operation is claimed |
| P-root | Explicit CORE denial/inert-intent only after exact guard/effective-policy proof; no new template and no claim that a TUI flag is excluded from headless accounting |
| P-terminal | Explicit CORE denied/no-spawn action behavior, not successful streaming. A blanket DIAG assignment to a mapped run would erase M and is not proposed |
| P-session | Explicit CORE+SESSION for R's command, last and SESSION_ID; last/id modes and no fallback/no picker must both be proved. PROMPT remains explicit CORE but the owning operation still carries SESSION |
| P-session-deny | R01 `--all`: explicit CORE+SESSION denied unsupported selector; never substitute wrapper's All selector for adapter last. Proof includes no spawn/selection fallback and fixed cwd-filter behavior |
| P-managed | Explicit CORE+MANAGED for F's stdio/strict-config controls under the bound startup context. MANAGED retains handshake, selected messages, cancellation/disconnect/shutdown and real ownership; it is not forwarding permission |
| P-managed-admin | F01 command: explicit CORE+SESSION+MANAGED+ADMIN for managed fork and new-thread persistence, including failure after creation. Both selectors remain due |
| U-approval | G03 root approval: effective-policy mapping remains unresolved; headless default never does not prove every explicit policy. Test/repair against production adapter + pinned runtime |
| U-root-fork | Every G row marked here needs separate app-server source/default review. Prefer whole denied unsupported intent under CP-2, then explicit CORE+MANAGED assignment; root auto-review can become config and is not assumed inert. Exec's root inheritance result does not apply to app-server |
| U-analytics | F02 analytics-default flag is absent/false in adapter config, while upstream user config can still opt in. Retain unresolved until the effective telemetry/config boundary is bound; then explicit CORE+MANAGED denial of caller override with reviewed default. No claim of network-free execution |
| U-transport | F's code-mode-host/listen/ws-* and fork G's remote/auth intent: retain unresolved. Recommend guard all alternate host/listener/WebSocket/auth modes and bind stdio default before explicit CORE+MANAGED denied-intent assignment. A raw string URL is not a REMOTE qualification; hidden/persisted remote-control startup must also be bound |
| U-config/U-feature/U-profile/U-hooks/U-provider/U-source/U-claim | Same finite prerequisites as the exact initial-disposition table, separately approved per owning operation; no transfer of initial approval to resume/fork |

CP-3's proposed final assignments under the approved CP-2 context are finite:

| Current prerequisite code | Exact proposed row templates after prerequisite; proof needed |
| --- | --- |
| U-config, U-feature, U-profile, U-hooks, U-provider, U-source | CORE on initial/resume; CORE+MANAGED on fork where present. Reject the whole caller-controlled knob; only the exact source-generated config and fixed context/defaults are admitted. Prove absence/loading/denial and the generated/default behavior. No arbitrary wrapper regime is qualified |
| U-claim | CORE denied/no-effect for each operation; reconcile emitter/claim against pinned runtime and keep historical identity. A demonstrated unsupported emission requires a wrapper compatibility repair, not deletion from O |
| U-approval | CORE for initial/resume, with exact accepted approval variants honored or existing pinned approval-required error behavior. Prove effective policy and repair demonstrated translation loss before assigning; default-never-only proof cannot resolve it |
| U-root-fork | CORE+MANAGED, full denial of unsupported root intent in this controlled interface, with explicit typed protocol safety/cwd behavior covered by parent obligations. Prove no startup/config side effect escapes the bound context, especially root auto-review conversion |
| U-analytics | CORE+MANAGED: effective analytics.enabled=false and default flag absent; reject override. Prove actual effective context and credential-safe diagnostics, without promising globally network-free behavior |
| U-transport | CORE+MANAGED: only owned stdio startup; every alternate host/listener/WebSocket/credential-intent value is denied by the profile context. Prove no listener/persisted remote-control startup; remote-mode support remains unqualified |

These are proposed whole-regime assignments, not current approvals. Each applies separately to
its operation/identity column, including explicit ancestor ownership for resume. After CP-2, no
arbitrary-value survey is left on the recommended route: unresolved work is the named guard,
source correspondence and behavior proof/repair. If the context is not approved or cannot be
mechanically isolated, all these U rows remain unresolved and positive freeze refuses.

Parent I command and PROMPT are retained in resume's conservative closure with explicit
CORE+SESSION competing-initial-intent rejection; other I proposals retain their stated effects,
with SESSION already due from the parent operation. R output schema is an input-file shaping
promise, not QUERY. R `--all` changes selection and is not ordinary CORE inheritance. Fork's F
transport controls would change process/protocol ownership if admitted, so they stay U-transport
until the guard proves their complete denial. Defaults that start implicit listeners/services are
not denied merely because the wrapper passes no explicit flag.

## Managed protocol prerequisite

The fourteen current acquisition files contain CLI names/arity/claims, **no complete acquired
app-server protocol schema or selected message contract**. Current repository source shows this
subset to reconcile with a pinned generated upstream schema:

- Initialization request/response, experimentalApi/client metadata,
  startup timeout/readiness and process lifecycle.
- For last only: `thread/list` with effective cwd, limit100, updated-at sorting and cursors; the
  wrapper selects greatest `(updated_at,created_at,id)` across all pages, rejecting repeated cursors.
  Id skips that list and preserves supplied bytes.
- `thread/fork`: source id, cwd, approvalPolicy, sandbox, omitted persistExtendedHistory; response
  supplies the new thread id. `turn/start` uses that id, text prompt, cwd and approvalPolicy.
- Selected turn/thread/item notifications and completion/error payloads actually decoded by
  `map_app_server_notification`; approval-required detection/cancel handling, request correlation,
  disconnect, timeout and `shutdown`. `cancel(request_id)` is the actual wrapper cancellation path;
  do not substitute the unused `turn/interrupt` API as evidence.

Exact current client/source correspondence is below. Field names shown are serialized names,
not Rust identifier aliases. The source comparison identifies finite compatibility proof/repair
questions; it is not an observed failure of the acquired binary.

| Stage | Current production client | Pinned upstream source correspondence and required proof |
| --- | --- | --- |
| Initialize | Request `initialize` with clientInfo={name,version}, protocolVersion=`2024-11-05`, capabilities.experimentalApi=true; await response within3s; no initialized notification sent on this path | common.rs declares initialize and client `initialized`. CP-C1: verify required handshake and accepted fields against pinned server/binary; enum existence alone does not establish mandatory sequencing. Repair a demonstrated mismatch |
| Last selection | `thread/list` with cwd, cursor=null first then page cursor, limit=100, sortKey=`updated_at`; archived/modelProviders/sourceKinds omitted. Read data[].id/createdAt/updatedAt/cwd and nextCursor; maximize tuple across all pages | v2/thread.rs supports those fields, but omitted sourceKinds defaults to interactive sources. CP-C2: prove last selector's promised source population, cwd filtering, ties, pagination and no-match; do not assume sessions created by exec are listed by an omitted filter |
| Fork | `thread/fork` with threadId,cwd,approvalPolicy when set,sandbox; persistExtendedHistory omitted. Read response.thread.id, ignore other fields | v2/thread.rs supports the emitted fields; its larger response includes model/provider/cwd/policy. No source or schema claim that omitted persistence means no mutation. Test nonexistent source id and partial creation/turn-start failure |
| Turn | `turn/start` with threadId,cwd,approvalPolicy when set,input=[{type:`text`,text,text_elements:[]}]; wait on RPC response; response success currently leads to shutdown and synthetic success status | v2/turn.rs returns `{turn:...}`; CP-C3: establish acceptance versus terminal completion, input field compatibility, real process exit/reaping and normalized completion. Waiting for request acceptance alone cannot establish terminality |
| Text/reasoning | Mapper recognizes `agentMessage/delta` with string params; `reasoning/text/delta` with content.text/text/string | common.rs declares `item/agentMessage/delta` and `item/reasoning/textDelta`. CP-C4: exact method/payload mismatch is a source-supported compatibility gap. Prove version-bound emissions/any aliases and repair parser/routing as demonstrated; no default alias compatibility is assumed |
| Status/items/errors | `turn/started`,`turn/completed` become status; `item/started`,`item/completed` read item_id,thread_id,turn_id,item_type; `error` reads error.message or message; unknown/malformed forms map to None | common.rs declares these methods, but a name match does not establish field compatibility. CP-C4 also requires pinned generated payload schema and actual event fixtures, including unknown/malformed handling. Do not declare all notifications acquired from this checklist |
| Approval | `codex/event`, msg or direct object, type=`approval_required` or `approval`, nonblank approval_id or id; sends cancel, stops forwarding and emits pinned approval-required terminal error | CP-C1/CP-C4: verify actual pinned server request/notification approval mechanism and no unhandled server request; no approval-response capability is invented |
| Cancel/timeout/shutdown | `$/cancelRequest` notification params={id:requestId}; timeout sends cancel+shutdown; shutdown request null waits at most5s then exit notification null; drop aborts tasks/kills child. RPC/channel errors map to backend errors; not-found fork error has pinned classification | CP-C1/CP-C3: prove actual server acceptance, cancellation target, disconnect and child reaping; no substitution of the unused turn/interrupt API. Synthetic success is not proof of upstream process exit |

Relevant production files are
[protocol](../../../crates/codex/src/mcp/protocol.rs),
[client](../../../crates/codex/src/mcp/client.rs),
[transport](../../../crates/codex/src/mcp/jsonrpc.rs) and
[fork mapper/flow](../../../crates/agent_api/src/backends/codex/fork.rs).
CP-C1 through CP-C4 are stable compatibility prerequisite labels, not reviewer findings or a runtime
failure claim. The smallest next implementation assignment should establish these exact paths with
the pinned binary and production adapter, repairing demonstrated mismatches before positive freeze
plumbing. Root approval translation is the separate U-approval effective-policy test.

This remains a source checklist, not protocol SurfaceIdentity rows or a complete upstream schema
claim. The proposed report must acquire the generated selected payload schema and bind it with
source-to-binary correspondence. Missing facts keep fork unready; shell `exec fork` evidence does
not satisfy this path.

## Bound source facts and raw inputs

The selected musl binary records SHA-256
`0b2e9301d6100dddda3b9d5c80ebaeaa3a2f1962388f2f36f6b96a9f08b1f33f`.
The official annotated tag `rust-v0.156.1`, tag object
`81e8e29b2956dfe9b092c63953a9ed282781e77c`, points to commit
`b412ff32c417f855c2b2d1581b77058eed87c84b`. Source correspondence corroborates design,
not an A1 build attestation or runtime proof. Cached official-source bytes were copied unchanged
from the prior design evidence; no moving latest was fetched.

| Pinned primary source | SHA-256 of reviewed source bytes |
| --- | --- |
| [exec CLI](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/exec/src/cli.rs) | `6abb86feb8fac2a192b34a617634527d0747b6d7fa09ee4827c0dc95abbe2f75` |
| [shared options](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/utils/cli/src/shared_options.rs) | `6eefb5fcf0545fca54ffa1f80a6d3630c8d9b61ec449466d5834ac9c489f76d8` |
| [root dispatcher](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/cli/src/main.rs) | `b4307eff0383a45263a9c328fcf6c13f73c4cd659368043ce71bdef1424f5532` |
| [exec implementation](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/exec/src/lib.rs) | `bbacb1efa0741f7e039e48742e1dc4db927c21cd353dc02c0b811293525fd1e1` |

Additional pinned primary sources were acquired solely for this design; they are **not** current
O inputs or an installed protocol report:

| Pinned primary source | SHA-256 |
| --- | --- |
| [config schema](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/core/config.schema.json) | `b373250e2565043b270d22df0e8cf32cacb36dd32fa2dc1e1e871115e73caab8` |
| [protocol common](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/app-server-protocol/src/protocol/common.rs) | `02cd72be646c4388e90f21ed651838aeab3d6273ade14715e38f2de6c095ba99` |
| [protocol thread](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/app-server-protocol/src/protocol/v2/thread.rs) | `761fb1a61a8107ab37dcfd7f997a4983bd58aab81cf84355ae30197ed809c8ac` |
| [protocol turn](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/app-server-protocol/src/protocol/v2/turn.rs) | `43fc4673df433892f0426b0fb7bac9cf3f429001fe1e05efcd6e4ae52e41d086` |
| [protocol notification](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/app-server-protocol/src/protocol/v2/notification.rs) | `c51b00e47732fe19e68110c42c4fe4212cc9bbbe20f252d6417e78a62b34afa7` |

The current fourteen exact raw inputs, independently rehashed for this proposal:

| Path | SHA-256 |
| --- | --- |
| `cli_manifests/codex/RULES.json` | `c12e256023cc452ae7aa758f67b48d9862e7becb6681876de6a8eddb985cb9d8` |
| `cli_manifests/codex/SCHEMA.json` | `96f1afe9e3388686c7c9519a9234e8d01411a2d72e7daa072ffef6deeccc74d5` |
| `cli_manifests/codex/wrapper_coverage.json` | `6cd7ef079f893eb4a3745bcd948fbae432fe025568f156bda267dcbdde881e70` |
| `cli_manifests/codex/snapshots/0.156.1/aarch64-apple-darwin.json` | `a43fff11fc62a2786e55fbd635625c95d8a4875240ad43edb50e9947eefda594` |
| `cli_manifests/codex/snapshots/0.156.1/aarch64-unknown-linux-musl.json` | `3b06cf8f5129a8cf3fbb44a17ad2c86560c99f1875abac52fd0654c0a52f88b6` |
| `cli_manifests/codex/snapshots/0.156.1/union.json` | `fb82e3d34f2f6bbe6886defaea1cb624c5b9c284255aa7bac552250e732d4006` |
| `cli_manifests/codex/snapshots/0.156.1/x86_64-pc-windows-msvc.json` | `708cb0657b6b01a9d28e2c34666656eb9faf7f52e56eecc29eb585ef423b8577` |
| `cli_manifests/codex/snapshots/0.156.1/x86_64-unknown-linux-musl.json` | `dfaac28c036883b523bf156e17e79bcdb76ca39259a06b4f0da35a2d063f894b` |
| `cli_manifests/codex/reports/0.156.1/coverage.aarch64-apple-darwin.json` | `7fb6a6ce0a27b9d58a4a01c1851dc6bdb00b92a487807946b7e04e74ef43cb6d` |
| `cli_manifests/codex/reports/0.156.1/coverage.aarch64-unknown-linux-musl.json` | `2d10c3de71991ebe7b244b63c732237b3dbe66026f5c7746bd911d0212e4c491` |
| `cli_manifests/codex/reports/0.156.1/coverage.all.json` | `1fca4c05dddafb15b349dbcbbea1cc87f0478c72be2a42ac84a12171c7c9760d` |
| `cli_manifests/codex/reports/0.156.1/coverage.any.json` | `6855143fb19ba2fac771248b02325ae12a03be7b6163f9bcb9bf879344466d15` |
| `cli_manifests/codex/reports/0.156.1/coverage.x86_64-pc-windows-msvc.json` | `5aa368afdc4cb2c2519c6366d1fc59635b8c35cdac68b0296cf35c407866e2f8` |
| `cli_manifests/codex/reports/0.156.1/coverage.x86_64-unknown-linux-musl.json` | `9bf777fe1bca1bd620663959c339754e4ba4473e73196d20dcd780dc7c67fd4b` |
