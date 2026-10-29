# Native execution fixture readiness

Baseline: staging `9966b2f23f93f66e09b6ad0738ea3ff38cc018b6` (PR #303).
This increment extends the existing example and collector. It changes no public SDK policy,
registry assignment, packet writer, evidence admission, profile, qualification or enrollment.
Live calls remain prohibited. There is no live switch, credential input, service or background job.

For optional installed-version diagnostics, use the separate
[portable native result command](support-depth-native-results.md). It leaves authentication with
the caller and exports the existing case-result format. It does not change this pinned offline
fixture, reinterpret its negative observations or create qualified maintenance evidence.

## Fixed fixture

The collector still uses a clean committed source tree, the pinned musl Codex 0.156.1 binary,
reviewed protocol/config acquisition, empty-root bubblewrap namespaces, cleared environment
and private home/project. No credentials or external network are available. Run the existing
`observe_codex_adapter_children.py` command from the [producer document](support-depth-native-producer-context.md).
It captures the 15 existing boundary cases plus one session chain and one cancellation case
for each of unattended, interactive and external variants. The extra cases are distinct from
BASE/template verification; the 53 approved template tuples and 207 proposed edges are unchanged.

Each chain uses production adapters in this fixed order:

1. Initial creates a native seed in the fresh private home.
2. Resume-last must return that seed's identity.
3. Resume-id explicitly uses the seed and must preserve it.
4. Fork-last follows the resumed seed and must return a new identity.
5. Fork-id explicitly uses the original seed and must return another new identity.

Only successful SDK completion, consistent native session-handle facets, a canonical native
session ID and the exact requested final reply `ok.` can admit a seed or continuation.
The missing-session placeholder used by the original boundary cases is never a seed.
Failed or missing seeds stop the chain; subsequent steps are recorded as unrun, without a
picker, imported session, synthetic rollout, fabricated positive, fallback or retry-to-pass.
Unit-test identities test state admission only; they establish no native execution.

Each executed step records full before/after private manifests and their SHA-256s, the selected
source session, returned session, SDK outcome and state refusal. Manifests include file digests,
modes, directory types and symlink targets without following symlinks; trace output is separate.
Before the next native invocation, the previous after-state must exactly match the current
before-state. A drifted state, config or foreign loader/credential input refuses before spawn.
The byte-exact config postcondition permits only unchanged config or the already approved
`[projects."/pilot/work"]` trusted-project addition. Native bundled `.system` assets remain
inventoried; no host/user skills, hooks, plugins, project instructions or credentials are admitted.
Artifact hashing is bounded to 4096 entries and 64 MiB per manifest.

Boundary/cancellation requests retain a three-second deadline. Session requests have a fixed
30-second deadline, with a 175-second outer limit for the five-step chain. These are fixture
constants, not public API limits or a claim that the live model will finish in that time.
Cancellation uses the existing public control handle after a valid session-handle event,
then drains events and awaits completion. If no such event arrives, the request is not claimed
as exercised cancellation; its actual timeout/error remains visible. There is no automatic
approval handler or substituted approval policy.

## Process and trace closure

PR #303 used strace `-qq`, which suppresses normal exit records. The offline comparison retained
all probe outputs: `-qq` produced no terminal markers and many empty thread files; `-q` retained
one terminal record for every traced PID. The collector now uses `-q -ttt`, preserving exits
and launch timestamps. This addresses observability, not a demonstrated production SDK leak.

Every traced PID must have exactly one final exit/signal record. Missing/ambiguous exits add
`incomplete_process_exit`; unknown/detached records still add `incomplete_trace`. Neither receives
complete child-boundary credit. Spawned-PID coverage and strict cwd/ownership parsing remain.
Session child order requires complete, unique timestamps and exactly the executed stage count;
every child's executable, environment, cwd, safety flags and selector remain checked independently.
Raw traces and bounded driver diagnostics are retained before parsing, including on outer timeout,
when private raw retention is requested. No negative sample is replaced by a later passing sample.

These postconditions observe private execution. They do not establish a general pre-spawn SDK
security boundary, evidence authentication, independent adequacy or successful model execution.
All `context_complete`/`execution_verified` flags remain false and packet binding remains null.

## Recorded offline result

The [refreshed audit](../../../cli_manifests/codex/reports/0.156.1/pilot-adapter-children.json)
records source `69fae8a658e012d66027592cc1b8c2b8e7331356` and all 21 cases. All 992 traced
PIDs have final exit/signal records. Sixteen cases have complete child-boundary observations;
five retain unknown syscall records and `incomplete_trace`: resume-last interactive, resume-id
external, fork-last unattended and fork-id unattended/external. The earlier comparison and
pre-repair capture remain private review records; no sample is retried into passing credit.

All three session-chain attempts timed out at their initial seed and record the other four
steps as unrun. No seed or successful continuation/fork is claimed. All three cancellation
cases observed a native session ID, requested cancellation through the public handle and
received the cancelled completion. These are observations of this private offline fixture,
not per-tuple adequacy or qualification. Eleven exact approved private-project config additions
were inventoried; every other config mutation remains refused.

## Remaining runtime facts

Provider `openai`, model `gpt-5`, the full controlled config and credential absence are bound input
intent, not a claim about every resolved endpoint or effective turn default. Native helper attempts
and bundled startup mutations remain inventoried. The six approved native enabled feature values
are preserved; `tool_registry` and other unexposed defaults, loader/provider effects and successful
native command/approval behavior remain unresolved. No unknown is cleared by a plausible config,
a successful process exit, a session ID or a content digest.

## Proposed later live-run scope — not authorized or enabled

Reuse this fixed fixture after remaining context facts and the live boundary have been reviewed:

| Item | Proposed scope / prerequisite |
| --- | --- |
| Native target | Only pinned 0.156.1 musl Linux, current reviewed source/config and private roots |
| Provider/model | Keep `openai` / `gpt-5`; establish exact effective endpoint/auth intent and current model availability before approval |
| Credential | A separately approved reference supplying only `CODEX_API_KEY`; never host auth/session state or secret bytes in chat/evidence |
| Requests | At most 15 session-chain adapter requests and three cancellation requests; zero harness retries. Native provider retries are unresolved, so this is not a bound of 18 billable API calls |
| Time/output | Current per-request and outer deadlines; exact short `ok.` reply, bounded diagnostics and artifact inventory |
| Budget | Recommend a total ceiling of US$5, subject to approval and a reviewed enforceable provider/token limit. No cost estimate or hard spend guarantee is established by this fixture |
| Tools/interactions | Only separately reviewed private scenario effects; no external tools, automatic approvals or skill execution inherited from startup inventory |
| Stop | Foreign/drifted context, unavailable effective defaults, failed seed, changed session/state, incomplete trace, unbounded retry/cost or unexpected effects stop qualification; retain raw outcomes |

No live request may run until its credential/model/endpoint, budget and effect scope are explicitly
authorized and enforceable. This increment supplies no network-enabled launcher or credential
plumbing. Positive native execution, full allowed/denied coverage, per-tuple packet evidence,
independent adequacy and authenticated adoption remain due before reviewed rows/profile can enable
the isolated public maintenance-chain proof. The [depth contract](../../specs/support-depth-contract.md)
and [evidence contract](../../specs/support-depth-evidence-v1.md) retain authority.
