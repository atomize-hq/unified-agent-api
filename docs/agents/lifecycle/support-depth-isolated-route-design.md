# Isolated public maintenance routes: binding design

Status: IR-P1, IR-P2 and IR-P3 approved by the maintainer on 2026-10-25.
Implementation is incremental; approval does not provision a host or enable production.
Source baseline: staging `930e85317f07f0c5161f9ffd7dbf8023ba15c143` (PR #297).
This is step 2 of the approved [isolated proof plan](support-depth-isolated-proof-plan.md).
The [depth contract](../../specs/support-depth-contract.md), [registry contract](../../specs/agent-registry-contract.md#support-depth)
and [evidence contract](../../specs/support-depth-evidence-v1.md) remain normative.
This approval enables no route by itself, changes no registry/host setting and creates no background job.

## Outcome and scope

Run the existing public maintenance commands against one independently authorized disposable
repository, with real acquisition and request-bound P/O, authenticated per-obligation evidence,
manual closeout, separate promotion and final protected integration. Keep one Codex0.156.1
`run`/CORE headless promise, `agent_api.run`, BASE+OUT3+LC2+M and no new debt or blanket NA.
Synthetic adapter subprocesses retain their explicit fixture provenance; this is not qualification
of live upstream model output. Other agents, modes, targets and onboarding get preservation/refusal
coverage. The existing Annex A remains the sole route inventory.

PR #297 landed the read-only bridge, not that public chain. Its two fixed fixtures retain their
existing behavior. Public production loading still refuses depth and `production_results` still
returns empty verdicts. Ordinary non-depth commands keep their current behavior.

## Approved decisions

| Id | Recommendation | Reason / boundary |
| --- | --- | --- |
| IR-P1 — isolated authority | Bind exactly one disposable repository by fixed GitHub repository id, owner id, expected full name and protected staging, with a separately approved implementation baseline. Add one private isolated context alongside existing production behavior. Resolve it from authenticated host observations, never record/receipt/remote spelling/environment claims | Real public entrypoints need an isolated authority boundary. Names alone, a worktree or an arbitrary `--allow-depth` switch do not supply it. Exact repository provisioning and credential references are authorized separately before host writes; no production path is enabled |
| IR-P2 — target meaning | Enroll the packet on the actual upstream/promotion tuple `x86_64-unknown-linux-musl`; keep `x86_64-unknown-linux-gnu` as the Rust wrapper build/execution host in evidence | Codex RULES and the pinned0.156.1 artifact use musl; the public resolver requires a real selected shard. The existing CORE fixture's GNU enrollment cannot be copied into a real packet. This clarifies the approved single-Linux scope; it does not relabel a snapshot, claim GNU upstream acquisition or grant cross-target receipt reuse. The new packet gets fresh P/O, trust binding, mappings and adoption |
| IR-P3 — isolated packet producer terminality | For this new isolated-packet trust profile only, admit authenticated evidence after the existing `rust-tests` producer job is terminal and its collection/upload/binding steps and immutable artifacts are authenticated, even while the enclosing workflow runs | Repairs IR-ADV-01 (original P2): same-run IN1 cannot wait for completion of the workflow containing its own checks. Keep every applicable attempt and the full pre-MERGE workflow gate. Existing channel/CORE fixture profiles retain whole-workflow completion. This profile-specific policy is approved; reviewed contract/profile implementation is required before use. Existing profiles are unchanged |

The following are existing requirements, not requests for new exemptions: maintainer classification
and explicit P re-freeze, exact generation stand-down on the integration parent, separate closeout
and promotion, complete attempt discovery, independent mapping adoption, owning admission at apply,
all-root IN1, native MERGE queue, and separate later production authorization.

## Source facts that constrain implementation

| Present fact | Required work before a positive chain |
| --- | --- |
| `agent_registry.rs::parse` refuses declarations; preparation additionally refuses retained records | A scoped loader admitted only by the independently resolved isolated context. Leave ordinary parser and unsupported writer refusals intact |
| `support_depth.rs::resolve` refuses predicates, and requires selected targets in RULES or retained authority | Implement finite predicate/alias intake for the exact `codex exec --json` selector from pinned acquisition. Do not remove `--json` from P to avoid validation or broaden accepted selectors |
| `support_depth/acquisition.rs::derive` conservatively marks every selected surface `classification_required` | Establish reviewed effects classification under the existing P operation entries for the complete observed/claimed population. Unknown units remain unresolved. Exact assignments are a maintainer policy artifact, not an executor decision; inspect them before approving the packet P. Do not clear the list, shrink O, auto-inherit from names or add a second classification inventory |
| `manual.rs::admitted` refuses nonempty `acceptance_work`, as no semantic corroborator is installed | For this first positive use a real, audited no-new-work revalidation generation with no carried due work. Run actual audit/intake and prove that result. Include a nonempty-work rejection fork; never erase work to make the run pass. A release requiring semantic uplift needs a separate corroborator before it can be accepted |
| The bridge resolves its original Event/P/O from a fixture; its O has no classified surfaces and only three acquired references | Add a packet-bound collector/adapter path using actual request, registry, RULES, schema, union/shards, wrapper coverage and reports. A matching operation name does not make the fixture's P/O current |
| Production manual/promotion hosts and IN1 call production evidence; proposal scratch trees do not establish host identity | Thread one privately constructed isolated context through real public/lower paths and their scratch views; re-establish it before delivery and final integration |

If complete classification cannot be expressed/established by the current P representation,
stop that dependent increment and bring the exact representation change for review. This design
does not approve unnamed classifications, a new template or a schema change in advance. Every
observed/claimed unit still counts, including unsupported and passthrough units. If classification
requires more than CORE, report the actual policy choice and evidence work; do not reduce obligations.

IR-D1 (original P3) is an early implementation feasibility checkpoint: the standing Codex0.156.1
request contains 56 required uplift rows, and `support_depth/freeze.rs` carries same-version work
across Events. A fresh audit or Event alone cannot prove empty `acceptance_work`. Before dependent
positive-chain work, establish legitimate approved-baseline/ordinary revalidation history and audit
the actual carried work. If it remains nonempty, stop that dependent work and bring the exact
semantic corroborator needed for review. Do not erase/relabel required work, assume an empty list
or treat a baseline reset, private fixture or historical closure alone as public depth-positive proof.

The binding-foundation read-only probe on baseline930e8531 found the current ordinary audit
`satisfied`, its closeout bound to the exact old request, and an ordinary revalidation plan with
zero required uplifts and no depth binding. No plan was applied. This makes an ordinary bootstrap
plausible; protected delivery, ordinary closure and the later actual depth P/O still need proof.
Direct depth enrollment against the old request continues to carry its 56 rows.

## One private context, reused owning checks

Use a small private `IsolatedMaintenanceContext` (illustrative name) in xtask. It is not a new
public API, serializable token, generic producer registry or caller-supplied success callback.
Its resolver recognizes only the one compiled approved isolated profile. Until the repository
identity and approved source are installed, it cannot resolve positively.

The context binds:

- authenticated repository/owner identity, expected name, `refs/heads/staging` and its observed tip;
- independently approved validator/collector/workflow/input closure and original installation baseline;
- the actual workspace and Git ownership plus candidate/base identity, or the explicitly derived
  invocation-local proposal view of that workspace;
- the selected agent/version/upstream tuple, headless selector and capability, with isolated origin;
- the actual generation's request Event and frozen P/O when they exist. P-only preparation is valid
  only as reporting; it supplies no evidence, readiness or acceptance.

Registry remains the only declaration/path-enablement inventory. A declaration is necessary but
cannot manufacture isolated host identity; a context is necessary but cannot create an enrollment.
Reject any extra declared or retained depth selection outside the approved isolated scope, rather
than falling back to ordinary handling. Removed declarations and secondary roots retain all-root
mediation. Keep the compiled profile's allowed scope fixed; it is not another mutable policy list.

Resolve authority before the supported route uses its scoped loader. Pass it privately into
preparation, publication and the existing manual/promotion hosts. Preserve the current independent
P/O/evidence/debt predicates. A proposal inherits the context only from its observed caller; a
scratch tree with copied files cannot resolve itself as an approved repository. Recheck source,
host identities/tip and all local expected-before inputs at apply. A moving tip/input refuses or
requires a new plan; it is never silently resealed or retried against a different base.

At IN1, resolve the context afresh from the independently authenticated repository and exact
queue candidate/base, before requested-root narrowing. A local context or earlier check grants
no final permission. Reuse the current all-root comparison, generation, qualification and owning
closeout/promotion checks; retain App15368 status bindings and the terminal gate. Reuse the existing
job graph; this design mandates no new CI job, required check, queue substitute, persistent lease,
approval ledger or transaction engine. The ordering work below remains to be reviewed and proved.

The trusted validator/controller comes from independently approved protected source, not an
executor-modified candidate. Bootstrap the isolated protected baseline before the first depth
candidate. Source approval is checked before candidate code/archive decoding is trusted; a candidate
cannot endorse its own validator. Changes to the approved producer/workflow/requirement closure
need independent protected-source approval and affected proof again. Host reads must be available;
a missing credential, ambiguous/changed repository, incomplete readback or stale source cannot grant
isolated authority. A Git remote can locate objects but is never that independent observation.

## Real packet evidence and reporting

Reuse CORE case/build collection and the existing typed receipt, payloads, evaluator and opaque
full-record seal. Keep existing channel/CORE diagnostic fixtures unchanged. The isolated packet
path has a distinct trust binding, actual packet paths and isolated repository/PR association;
it cannot consume the canonical fixture's Event or simply copy its receipt/success labels.

Before collection, independently derive and validate the real generation from coherent stage-0
request/record/acquisition inputs. Derive the same P/O at currentness, closeout and promotion,
including retained-closeout handling after working request replacement. Bind complete semantic
inputs and modes/inventories. The approved profile fixes permissible scope, not the actual O hash.
A changed generation/P/O requires new or explicitly admissible reuse evidence; changed O clears E.
No cross-version or cross-target reuse is added.

Keep exhaustive host run/attempt discovery, immutable artifacts/log bindings, actual workflow/build
identity, dependency/environment currentness and per-obligation case subsets. Every applicable
attempt counts even if failed, unfinished or missing evidence. One exact authenticated maintainer
adoption may cover the twelve independently reviewed mappings; candidate source or generic code
review does not supply adequacy. New target/source/assertion/P/O/trust differences must be evaluated
under existing mapping-reuse rules; old fixture adoption is not automatically transferred.

Under approved IR-P3 for the new packet profile, workflow `in_progress` alone does not make an otherwise complete producer
unverified. Independently authenticate API run/attempt/head, approved workflow and checkout source,
exact terminal producer job/steps, timestamped binding logs and immutable artifact identity/digest.
Require a complete stable applicable run/attempt/job/artifact population; recheck identities and membership
without demanding that unrelated workflow progress stop. Missing, queued, running or aborted
producers remain unverified; missing/unavailable logs, incomplete artifacts or failed collection/
upload/binding never verify. Terminal failed-case witnesses and contradictions remain in evaluation,
including faithful witnesses from a job that failed elsewhere. Do not exclude the current run,
filter to the latest green attempt or invent an adoption fallback. All twelve mappings still need
independent adoption. Before MERGE, the whole workflow must finish through its existing 17-job
result policy (including candidate-justified skips), direct required checks and terminal gate.

Implementation must trace every evidence invocation, including public checks, support publication,
`make preflight`/support-matrix and all-root IN1. Schedule actual consumers after the terminal
producer and bound artifacts; the producer must neither invoke nor depend on its own packet
admission. Review minimal explicit `needs` edges in the existing workflow, preserving failure
observation rather than silently skipping consumers after a failed producer. A job condition alone
does not create conditional `needs`. Review actual scope and latency: serializing consumers behind
`rust-tests` can delay ordinary CI too. Do not claim no latency impact or add a new framework to
hide the dependency; prove the resulting invocation graph has no admission cycle.

Reporting may publish actual opaque isolated verdicts only inside the authenticated isolated
context. Canonical production consumers keep production-origin requirements and their current
unverified fallback. Qualified output is still an acceptance effect: reporting cannot invent
qualification before an owning closeout/promotion entry and current final admission. Preserve
ordinary historical rows and unverified publication; omit no unknown/failed obligations.

The full-record seal includes entries. After owning admission appends an entry, obtain a fresh
host-evaluated opaque set for that exact record state for rendering/recheck; never copy/reseal an
old verdict set or trust E labels. Admission itself still independently checks the entry's authority.

## Public routes and delivery

This table is the wiring checklist for Annex A, not a second route inventory.

| Existing route | Connection and retained boundary |
| --- | --- |
| MA1 preparation, MA2 acquisition/refreeze | Scoped public registry load, real freeze and same request bindings through builder **and** apply. Stage actual acquired bytes before O; return the existing exact delivery path set and support projections. P-only stays not ready. Preserve token/destination/stand-down protections |
| Support/runtime publication and audit | Consume scoped opaque results through existing renderers/owning guards; audit actual classification, work and debt. Preserve aggregate/secondary-root checks and runtime/capability semantics |
| MA4 executor | Use the real guarded entrypoint with captured Event/P/O/context and independent candidate. Only permitted reporting writes reach caller; finalize on every exit/spawn/gate/cancel failure. Executor cannot mutate request, marker, entries, acceptance or debt inventory |
| Both MA5 commands and exported lower closeout writers | Same private isolated host at planner/apply/owning proposal boundary. Require current bindings, all-due evidence, no unresolved classification/work/debt and exact-generation marker already on integration parent. Exercise both public commands, not just private `Host` fixtures |
| MA6 promotion | Real public plan/apply with isolated host and exact validation tip. Preserve retained closeout proof after working-file replacement, passed target attribution, own promotion entry, coherent pointer/publication/metadata and same-proposal marker retirement |
| IN1 integration and post-push observation | Resolve isolated context and re-run owning admission for exact candidate/base over every retained root. Native queue supplies the conditional ref update. Retain checked candidate/tree/parents and post-push readback |
| Unsupported MA3, neutral/onboarding/lower writers | Their existing refusals remain. Invoke valid public/lower paths with declaration and retained-record-only controls; ordinary counterparts continue to work |

Transport must derive the exact approved isolated destination from the same context, independently
resolve its host identity immediately before every mutation, and use existing expected-before/path
checks. Neither changed origin URL nor forwarded context fields authorize a push. A rejected push
is not rebased/retried. No delivery credential reaches the executor.

Run the executor in an ephemeral runner/container with only isolated files/Git metadata and no
production writes, host home, sibling checkout or production credential mounts. A local worktree,
validated path or context alone is not an OS sandbox. The trusted controller may need separately
scoped read-only access to canonical source provenance and write access only to the exact isolated
repository; use existing credential references, never new credentials in records/logs/chat.
Before host writes obtain exact repository/provisioning and least-privilege credential-reference
authorization, read back isolation/rules and compare production identities before/after the proof.

## Small implementation sequence

1. **Binding plumbing, fail closed.** Private fixed isolated-context resolver, approved-source and
   repository observations, scoped loader and plan/scratch/apply lifecycle. Read-only probes and
   negative/public ordinary controls; no positive depth writer enabled yet.
2. **Packet intake readiness.** The proposed [intake design and exact classifications](support-depth-packet-intake-design.md) brings PI-P1–PI-P3 for separate maintainer decisions. Exact predicate/alias and selected-unit classification handling,
   real target/acquisition and P/O currentness. First present exact effect classifications and any
   necessary P representation changes for maintainer review; implement only approved policy.
   Resolve the IR-D1 feasibility checkpoint early; prove unresolved/classification/work refusals
   and truthful P-only behavior.
3. **Packet-bound authenticated producer.** Reuse CORE collection/evaluation for the real frozen
   generation and isolated host scope. Prove all twelve obligations with positive/negative host
   fixtures, currentness, IR-P3 terminality/discovery and opaque entry-state behavior. Live adoption
   is separate host work; IR-P3 implementation remains a prerequisite for same-run positive integration.
4. **Connect public routes and IN1.** Thread the admitted context and hosts through the supported
   builders/apply/lower paths and final validator. Reporting may land earlier where its exact
   effects are guarded; acceptance remains refused until every owning/final predicate is wired.
   Prove real entrypoints, producer/consumer ordering and rollback/conflict preservation before
   claiming this increment complete.

These are causal source increments, not new governance gates. Split a diff only when its actual
size/authority boundary needs it. Every source PR uses the existing required gates and both
GPT-6.1 Sol High review lanes at standard/default speed. No merge or host activation follows from
this design document. After reviewed wiring, execute the approved connected chain/rejection matrix;
private fixtures or shell freeze stubs do not complete it. Production enablement remains later work.

## Verification and completion

For this documentation slice: check local links/source references, `git diff --check` and `make
hygiene`; both independent design lanes must finish with material findings resolved. No Rust or
workflow behavior changes, so a full repeated runtime gate suite is not a design acceptance claim.

For each implementation: targeted behavioral tests, `make preflight`, default-feature workspace
tests, backend type-leak guard, and current/tip manifest plus acquisition-plan checks for Codex,
Claude Code and OpenCode. Add meaningful production-entry tests under existing xtask integration
suites, collector/host tests under existing depth modules and workflow tests only where transport
changes. Keep Rustfmt, no unsafe, existing LOC limits and no new runtime dependencies.

The minimum positive is real MA1→MA2→MA4→audit→each MA5 alternative→separate MA6→IN1 under the
same selected profile, with independently adequate evidence. Reuse the proof plan's negative
matrix. Add focused controls for wrong repository id/owner/name, copied canonical record/receipt,
source/baseline/tip drift, forged/deserialized/scratch context, extra retained scopes, predicate
ambiguity, target relabeling, classification additions and nonempty carried work. Capture exact
before/after state on failed delivery and preserve externally changed state.

For IR-ADV-01, prove same-run MA5/MA6→IN1 can verify after the current producer is terminal while
its workflow still runs, then pass the unchanged queue gate. Controls must keep unfinished/missing/
aborted producers, unavailable logs, partial artifacts, failed cases, contradictory attempts and
missing adoption unverified/refused as appropriate. A failed producer with a faithful witness must
retain its counterexample. Test identity/population drift and producer self-admission; preserve
existing channel/CORE whole-workflow refusals and exact candidate/base conditional queue admission.
A positive evidence result never overrides a failed required check or terminal workflow gate.

Completion of this design means a reviewed binding/route plan and explicit decisions, not that
any currently refusing public path has become usable. All positive implementation and host proof
claims remain to be established on their actual source revisions.
