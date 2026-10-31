# Native runtime context and case-coverage closure

Status: NC-P1 and NC-P2 approved by the maintainer on 2026-10-29. The
[first context implementation](support-depth-native-context-guards.md) implements guards and finite
observations; remaining context facts and case/mapping closure still need evidence.
Baseline: staging `d43131b6ce1481c371e0848ac277b1d62b7a3d56` (PR #307).
The [native evidence contract](../../specs/support-depth-native-evidence-v2.md),
[depth contract](../../specs/support-depth-contract.md) and approved
[native evidence design](support-depth-native-evidence-design.md) govern. No live call, host write,
classification installation, producer activation or background job is part of this design task.

## Outcome and actual gaps

Make one packet-bound Codex attempt usable by maintenance without changing ordinary SDK behavior,
copying credentials into fixtures or rebuilding the maintenance pipeline. Keep the existing producer,
registration/completion records, evaluator and record-sealed consumers. The already completed local
smoke need not be repeated merely to establish that UAA can run Codex.

The scoped pilot remains Codex `0.156.1`, acquired `x86_64-unknown-linux-musl`, with a separately
identified GNU wrapper build. It covers initial, resume-last/id and fork-last/id, including their
unattended, interactive and external-sandbox branches. The [current classification inventory](support-depth-current-pilot-classifications.md)
contains 94 acquired identities / 207 operation-surface edges (71 initial, 94 resume, 42 fork).
These are proposed whole-regime assignments, not installed policy. The approved template union is
53 operation/obligation/target tuples: 17 initial, 18 resume, 18 fork. Neither count is a test count.

PR #307 supplies packet binding and retained finite outcomes, but `LIVE_PROFILE` is still absent.
Its twelve cases are six per unattended/interactive variant: initial, resume-last, resume-id,
fork-last, fork-id and cancellation of a separate **initial** request. They assert the exact reply,
session identity/order and owned cancellation. They reject tool items; no interaction handler is
installed. They do not establish approval decisions, mutations, no-match, external sandbox,
effective configuration, every operation's cleanup or all-due adequacy. The observed context still
says `effective_native_configuration = unresolved`, `complete = false`.

## Policy decisions

| Id | Recommendation | Concrete consequence / alternative |
| --- | --- | --- |
| NC-P1 — eligible caller-native context | Approve one fixed, producer-only context definition using the caller's existing selected native home and authentication resolver. Validate actual nonsecret context against reviewed typed expectations; refuse unknown or foreign effects. Replace the earlier private-home/all-off fixture premise for this native profile | A normal login can be used without copying it or rewriting the caller's config. Authentication alone does not approve arbitrary host config, plugins, skills or managed policy. A mismatching environment gets a finite refusal, not automatic sanitization. Alternative: retain the private fixture regime and arrange separate native login there; this adds setup and does not prove the caller-native context |
| NC-P2 — unexposed deterministic defaults | Permit individually named default derivations from the exact pinned upstream source, conditional on measured input/layer/override precedence and native corroboration of the claimed effect. Keep these distinct from direct runtime observations | Allows a finite context to close even when a diagnostic API omits an internal default. A schema omission, source hash or maintainer statement alone cannot establish the effective value. Alternative: require a native diagnostic for each such value; keep the affected evidence unverified until upstream exposes it or a separately reviewed diagnostic exists |

Both recommendations are approved. This authorizes the observation methods and implementation; it does
not approve arbitrary runtime values, an auth backend, any classification row, a substantive mapping
or a concrete completion/adoption statement. The exact finite values and row assignments must be
reviewed with implementation before profile installation. No exception to unknown-context refusal,
all-due evidence, independent adequacy or authenticated adoption is proposed.

## Finite context definition

Use a strict typed projection in the existing context payload, with source-owned expected values.
The context is per actual route/case; intent, observed facts, named derivations and outcomes remain
distinct. A single overall `complete` value is true only if every required predicate and every
applicable case context is established. It is never set from a successful reply.

| Required fact | Minimal producer check and retained fact |
| --- | --- |
| Runtime and build | Exact acquired binary/version/musl target, actual selected executable and transitive runtime closure; separately measured GNU collector/build graph/toolchain/features. Bind each real harness executable by its own digest |
| Native environment | Validate the complete effective child environment by finite admitted channels and known nonsecret values/absence. Bind parent/overlay precedence and native resolver dependencies. Do not use `env_clear` or silently repoint home to force a match. Unrecognized override, loader, proxy or service channels refuse. Variables independently reviewed as irrelevant need no machine-specific value allowlist |
| Home, cwd and roots | Preserve native auth/home resolution. Validate canonical selected home, disposable project, output/temp and allowed roots before assigning role labels. Reject aliases, escapes and drift; distinct actual roots cannot collapse into one role. Keep caller-specific paths out of the stable expected-facts digest |
| Config and requirements | Bounded in-memory inspection of effective config, origins/layers, requirements and relevant input correspondence. Retain only approved enums, booleans, bounded source-owned values and validated roles. Inspect actual system/user/project/session and applicable packaged/managed layers; inaccessible or unknown enterprise/cloud policy remains unresolved |
| Effective turn context | Model/provider/endpoint and service-tier intent, approval/reviewer/sandbox, roots, feature/tool surface and inherited session facts at the actual production route. Config intent or a separate config-only launch does not prove a turn's effective state. A native refusal remains its actual outcome |
| Loaders and native state | Inventory only approved nonsecret loader/config/assets and scenario-owned project/session effects. Deny unreviewed host/user instructions, hooks, skills, plugins, MCP/services or transitive launchers. Pinned bundled startup assets may be present under the previous approval, but their loading/execution needs its own effect proof |
| Authentication | Native resolver stays the owner. Record only approved source/store class and presence/intent facts, without account identity, secret value or secret digest. File/keyring/auto/ephemeral support is not inferred from preserving home. An unresolved resolver dependency or auth-dependent policy/endpoint remains unresolved |
| Changes and termination | Compare relevant nonsecret inputs before/after, retain approved native mutations and partial effects, and observe direct child/owned descendant cleanup. Auth refresh is native-owned: never inventory/hash the auth store or treat its bytes changing as ordinary context drift |

The old private context's six enabled features and exact `/pilot/work` trust stanza remain scoped
observations/approvals; do not transfer them to arbitrary caller paths or assume the authenticated
context has the same 149 effective values. The source-owned schema defines which facts are required;
actual approved values belong to the finite profile, not a caller-selected trust configuration.

Local non-auth eligibility checks may happen before registration. All native probes, including
config/features/auth-status probes, remain **after** independently confirmed start registration.
Discovery of a runtime unknown, foreign loader or context change then retains a registered refused
or incomplete attempt. Recheck at each relevant route/turn boundary and after execution; do not
hide a partial attempt by doing the native work before recording it.

Observe the actual production child and inherited session context. Managed initial/resume already
read config/requirements for roots; fork does not use those reads, and exec retains user config.
A separate direct app-server observation cannot supply missing facts for those paths. Add only a
source-owned internal diagnostic projection at the owning boundary if needed; no public observer,
config passthrough, native-binary shim or change to ordinary callers' validation/dispatch is needed.
If the pinned route cannot establish a required fact, keep that route unverified and report the
specific missing observation rather than installing a broad positive profile.

Never trace live `execve` environments or protocol frames as done in the earlier credential-free
offline audit. Never retain raw config/layer/origin responses, arbitrary native strings/errors,
full environment arrays, whole-home/session inventories or their hashes. Auth files/keyring entries
are unopened and unhashed by the collector. Perform finite projection before any serialization,
including journals and failure paths. Use synthetic sentinels to verify those boundaries.

### Named source-derived default example

The acquired protocol report pins upstream commit
`b412ff32c417f855c2b2d1581b77058eed87c84b`. Its config schema represents `features.tool_registry`
as an object; omission from a boolean listing does not mean false. At that exact commit,
[`core/src/config/mod.rs`](https://github.com/openai/codex/blob/b412ff32c417f855c2b2d1581b77058eed87c84b/codex-rs/core/src/config/mod.rs#L3480-L3493)
resolves its two optional booleans with default fallbacks. This is a source fact, not an observation
of the caller's runtime. Under NC-P2, a named derivation must bind acquired source bytes, binary
correspondence, actual effective input absence/precedence and a native case exercising the claimed
tool behavior. Any overriding layer, conditional branch or unresolved correspondence prevents that
derivation. Apply the same standard to each additional unexposed relevant default; no catch-all
“native defaults accepted” flag or list inferred solely from JSON schema is allowed.

### Existing fixed-digest integration

PR #308 replaced `native/bundle.rs`'s whole observed-context digest comparison with typed expectations.
Actual role paths and per-case observations cannot be preapproved as one machine-specific blob.
Separate stable reviewed expectations from variable retained observations; keep both content-bound
in the existing plan/bundle. The private admission path must parse the strict finite projection,
validate every expected value and path-role binding, and recompute completeness before using it.
A matching intent digest, normalized path or producer-supplied `complete:true` is insufficient.
Reject unknown fields, omitted required facts and mismatched source/route/context revisions. Keep
receipt revision 2 and the existing index/reference bounds; no generic context registry is needed.

## Coverage closure using existing cases

Use the existing `AdequacyMapping`/`AssertionBinding` representation. Each mapping names the exact
assertion, actual harness executable, production inputs, operation/selector/regime and substantive
counterfactual. The table below is an implementation inventory, not approved mappings or evidence.
Reuse and extend assertions rather than manufacture one test per template or acquired spelling.

| Family | Existing substantive source to reuse | Required remaining assertion / execution |
| --- | --- | --- |
| C1 — requests and denied intent | `depth_ci_core.rs`, `c1_codex_exec_policy.rs`, `support_depth_codex_pilot.rs`, `codex_native_bridge.rs` | Through the real production validators, cover five selectors × unattended/interactive/external branch decisions; accepted/default/invalid policy and sandbox values, granular fields, model/roots/overlay precedence, conflicting selectors and unsupported fork model/add-dirs. Add the producer-only forbidden-intent/no-spawn guard and exact 94-identity/207-edge correspondence, including claim-only `--full-auto`. A diagnostic route decoder alone is not production rejection evidence |
| C2 — output and diagnostics | `depth_ci_core.rs`, `c1_codex_stream_exec_adapter`, `jsonl_compat.rs`, `native_interactions.rs` | Retain known/unknown/malformed/oversized/sentinel event/frame cases, bounded capture, finality and output ownership, with actual selected parser/build provenance. Inject malformed protocol in hermetic fixtures; no need to provoke arbitrary malformed live model output |
| C3 — managed interactions | `codex_native_bridge.rs` plus `invalidation.rs`/`queued_reply.rs`, `codex_pinned_fork.rs` | Reuse actual handler correlation, all supported request/reply shapes, invalidation, stale grants, concurrent connections and disconnect tests. Add native approved bounded command/file allow/deny behavior and exact effects. Preserve native unsupported-policy outcomes; do not substitute policies or auto-approve |
| C4 — lifetime and partial effects | `codex_fork_setup_lifecycle.rs`, `session_fork_v1_codex/{cancellation,timeouts}.rs`, `c3_explicit_cancellation.rs`, producer interruption tests | Reuse explicit readiness/cancel synchronization, drain/drop/deadline/EOF and kill/reap fixtures. Add native resume/fork cancellation, per-operation timeout/drop/owned exit checks, plus before/after project/state/output effects on success, denial and partial failure. Initial smoke cancellation cannot stand in for every operation |
| C5 — native operation/regime | Existing twelve `codex_native_results` cases | Keep initial/resume/fork last/id identity assertions. Add external branch evidence in an explicitly isolated execution environment, native policy acceptance/rejection, selected roots/model/provider effects, native no-match last/id controls in disposable projects, and the C3/C4 native effects. Bind setup seeds as dependencies; no silent synthetic session or seed fallback |
| C6 — context and exposure | Existing offline config/adapter-child observations and sentinel tests | Implement the finite context checks above for authenticated production routes; prove foreign loader/override refusal, opaque credential handling, source/default correspondence and retained incomplete attempts. Offline `strace` data supplies diagnostic history only, never live credential-bearing proof |

C1-C4 hermetic tests establish the wrapper/adapter assertions they actually exercise. They are not
native turns, real approval execution or musl process/ABI/filesystem evidence. Label synthetic CLI
and GNU wrapper execution honestly; native musl residuals remain due under the exact approved target
pair. No cross-target reuse of process termination, ABI, path/permission or platform effects is
introduced. Existing Actions receipts keep their own origin/profile; they do not become native
receipts merely because their files were copied into a local attempt.

For the native attempt, extend the existing closed build/finite collector with the required fixed
native and hermetic case groups, binding each actual executable independently. Reuse existing test
helpers/assertions where practical; do not turn Cargo's overall exit status into invented individual
case outcomes. Keep locally executed hermetic outcomes labeled as such under NE-P1's local execution
trust. An existing CI run is reuse only if the current evaluator independently admits its exact
scope/provenance; never splice a favorable CI case into native execution. The shared fixed `native/cases.rs` population now contains 28 native continuity/cancellation/no-match
cases; any further population/harness expansion must update pre-registration equality before
registration.
There is no generic arbitrary-test command or second evidence evaluator.

Mappings may factor independent dimensions only where substantive independent review establishes
that factorization. Cover every behavior-changing joint branch, all accepted policy/default/granular
semantics, both last/id selectors and their denial/error outcomes. Unsupported native values get
version-bound truthful rejection evidence. A policy enum's successful serialization or a small
representative case cannot silently stand for another value. Do not freeze a misleading live-call
count: setup/session dependencies, native retries and required behavioral regimes determine it.

### Complete due-tuple crosscheck

Every populated cell below needs its own mapping and current evaluated result. I/R/F mean the
three operations, with the fixed musl obligation target throughout. Cells select required case
families, not automatic verification. Resume/fork cells include both last/id variants; all cells
include their applicable safety/transport regimes. Independent review must establish the actual
assertion set, not merely that each family name occurs in a receipt.

| Obligation | Initial | Resume | Fork |
| --- | --- | --- | --- |
| R1 | C1+C5 | C1+C5 | C1+C5 |
| V1 | C1+C5 | C1+C5 | C1+C5 |
| G1 | C1+C3+C5 | C1+C3+C5 | C1+C3+C5 |
| E1 | C1+C2+C3+C4+C5 | C1+C2+C3+C4+C5 | C1+C2+C3+C4+C5 |
| A1 | C5+C6 | C5+C6 | C5+C6 |
| X1 | C1+C2+C6 | C1+C2+C6 | C1+C2+C6 |
| X2 | C1+C4+C5+C6 | C1+C4+C5+C6 | C1+C4+C5+C6 |
| X3 | C2+C3+C6 | C2+C3+C6 | C2+C3+C6 |
| X4 | C1+C3+C5 | C1+C3+C5 | C1+C3+C5 |
| OUT1 | C2+C4 | C2+C4 | C2+C4 |
| OUT3 | C2+C3+C5 | C2+C3+C5 | C2+C3+C5 |
| OUT4 | C3+C4 | C3+C4 | C3+C4 |
| LC1 | C4+C5 | C4+C5 | C4+C5 |
| LC2 | C2+C4+C5 | C2+C4+C5 | C2+C4+C5 |
| LC4 | C3+C4+C5 | C3+C4+C5 | C3+C4+C5 |
| FX1 | C3+C4+C5+C6 | C3+C4+C5+C6 | C3+C4+C5+C6 |
| M | C1+C2+C3+C4+C5 | C1+C2+C3+C4+C5 | C1+C2+C3+C4+C5 |
| S1 | — | C1+C3+C5 | C1+C3+C5 |

All 207 classification proposals still need individual whole-regime disposition against measured
context and allowed/denied behavior. Reuse the current exact table and registry/P as the sole policy
inventory; do not add an executable side classification list. Batch the reviewed rows in one
maintainer decision if they are all resolved, but do not request blanket conditional approval of
unknown rows. Changed context/implementation changes the basis; old private-fixture/30-row approval
cannot clear the new multi-route population. No debt, NA, exclusion or reduced promise is proposed.

## Small implementation sequence and completion test

1. **Context implementation.** After NC-P1/P2 disposition, implement strict producer-only eligibility,
   secret-safe actual-route observations and admission predicates. Bind named acquired default facts
   where needed. Keep the profile absent. Tests: config/loader/env/role drift, unknown cloud/input,
   synthetic secrets in success/error/partial journals, forged completeness and unchanged ordinary
   SDK behavior. Keep real native/model calls out of required CI.
2. **Case and mapping closure.** Extend only the fixed collector case groups above, reusing substantive
   tests. Retain each outcome before the next case; interruption leaves remaining cases unrun.
   Supply the exact finite assertion/counterfactual mappings and 207 row dispositions for independent
   review and maintainer approval/adoption. Source-bound facts can be reviewed without credentials;
   actual native proof still needs a registered authorized packet attempt. If a diff is too large,
   split at its concrete context/case boundary, not into a new framework.
3. **Owning-path connection and isolated proof.** Connect admitted results to existing record-sealed
   consumers and finite public selection; install only explicitly approved immutable host/profile
   data. Run the [isolated maintenance chain](support-depth-isolated-proof-plan.md) after its exact
   repository/host/execution approvals. Preserve separate closeout/promotion and merge-queue checks.
   Production-origin evidence and per-agent activation follow that proof separately.

These are implementation boundaries, not extra approval ceremonies: reuse NE-P1/P2's whole-attempt
registration/completion and existing independent mapping review/adoption. No per-case human signoff,
new account/standing reviewer role, required CI job, scheduler or service is added. Native execution
and completion/adoption posting retain their existing concrete authorizations.

Design validation: source-reference/link checks, exact 53-tuple / 207-edge correspondence and both
GPT-6.1 Sol High review lanes at standard/default speed. Each implementation PR uses the existing
required gates (`make preflight`, default-feature workspace tests, backend type-leak guard and the
three agents' manifest/tip/acquisition gates) and both review lanes. Closure is achieved only when
all required facts/rows/mappings are resolved and current all-due results are independently admitted;
a reviewed plan alone does not make `complete:false` true or the full pipeline proven.

## Context/case implementation progress after PR #308

Actual owning-child thread replies and turn acceptance now have separate finite, content-bound
partial observations. Bounded disposable-project metadata sampling detects scenario changes while
withholding file contents and auth/native-home state. These do not clear any of the six complete
context facts. The native population is 28 fixed cases, adding resume/fork last/id cancellation and
fresh-project no-match controls to both existing regimes. All original baselines remain bound during
those controls. Cancellation means owned request cancellation; accepted-turn timeout/drop/exit and
bounded command/file allow/deny effects remain separate assertions.

C1's real backend policy matrix now covers both last/id selectors, with no-spawn invalid-intent
assertions across all three request regimes. The existing acquisition comparison explicitly checks
94 identities, 207 edges and 53 tuples, and retains claim-only `--full-auto` as unresolved. These
changes supply concrete assertions for future mappings, not approved classifications or installed
adequacy. Hermetic and native executable identities must still be bound independently before a
mixed population can register. Keep the existing native receipts and private profile absent until
all required observation methods, values, mappings and registered native corroboration are complete.

## Layer-input progress

The managed initial/resume route now retains a finite native layer stack and origin-class set from
its own `config/read` (see the [guard runbook](support-depth-native-context-guards.md)). Admission
refuses a present config without an eligible stack. This is the managed route's measured pre-turn
input. `layer_inputs` stays unresolved for every route, for three reasons:
- unattended initial/resume exec and fork do not read config, so they have no measured stack;
- turn overrides are applied after the read;
- no reviewed source binding or native corroboration names the fact yet.
