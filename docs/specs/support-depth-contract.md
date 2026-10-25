# Support-Depth Contract

Status: Normative
Date (UTC): 2026-10-02
Scope: semantic support obligations, their evidence and bindings, depth enrollment, depth admission of writes to depth-gated outputs, path enablement and additive shared integration, for the maintenance and onboarding lifecycles

## Normative language

This document uses RFC 2119 requirement keywords (`MUST`, `MUST NOT`, `SHOULD`, `MAY`).

While its status is Draft this document binds nothing. Adoption is the maintainer changing the
status to `Normative`. Adoption alone enables no lifecycle path and makes no depth enrollment; see
[Path enablement](#path-enablement).

## Purpose

Name coverage records that a command, flag or argument appears in wrapper coverage. It does not
record whether the wrapper validates its inputs, parses its output, reports its errors or manages
its process to any stated standard. This contract adds that second statement, the support depth of
an operation, beside name coverage and without changing it.

It defines:

- what a wrapper commits to when it claims an operation, and the obligations that follow
- what evidence verifies an obligation, how evidence is bound to its inputs, and when it may be
  reused
- how a bounded scope is selected for these rules, and what stays outside it
- the depth admission rules every write to a depth-gated output must satisfy
- the prerequisites for enabling these rules on a lifecycle path
- how depth results are published, and the rules for later additive shared integration

The maintenance and onboarding paths MUST evaluate these rules through one shared implementation.
Neither path keeps its own copy.

The research and rationale behind these rules are in the
[proposal](../agents/lifecycle/support-depth-policy-proposal.md) and its
[research companion](../agents/lifecycle/support-depth-policy-research-decisions.md). Those
documents are not normative.

## What this contract does not change

| Existing rule | Owner | Relationship to this contract |
| --- | --- | --- |
| Wrapper coverage levels `explicit`, `passthrough`, `unsupported`, `intentionally_unsupported` and `unknown` | [Coverage generator contract](codex-wrapper-coverage-generator-contract.md) | Unchanged. A coverage level never states depth. |
| Name identity `(surface_kind, command_path, surface_id)`, the support-surface audit, required uplifts and allowed deferrals | [Maintenance request contract](maintenance-request-contract-v1.md) | Unchanged. Depth obligations are additional. |
| Debt rows and their target- and version-scoped authorization | [Debt inventory](unified-agent-api/non-tui-support-debt.md) | Unchanged. A debt row stays name-level; see [Debt operations](#debt-operations). |
| Release-watch enrollment | [Registry contract](agent-registry-contract.md) | Unchanged. Depth enrollment is a separate selection, declared in the same registry entry. |
| Capability ids and their minimum semantics | [Capabilities spec](unified-agent-api/capabilities-schema-spec.md) and each capability's owner document | Unchanged. |
| The capability promotion rule and its allowlist | [Onboarding charter](cli-agent-onboarding-charter.md) | Unchanged. Depth admission is added for the capabilities a depth enrollment claims. |
| Lifecycle stages and support tiers | The committed lifecycle record, as the onboarding charter designates | Unchanged. Neither is a depth result. |
| Runtime profiles `minimal`, `default` and `feature_rich` | The lifecycle implementation in `crates/xtask/src/agent_lifecycle.rs`; no normative owner | Unchanged. A runtime profile is not a depth result. |
| Surface exclusions in `parity_exclusions` | Each manifest root's `RULES.json` and validator spec, where the root has them | Unchanged. See classification rule 7 for mode exclusions. |
| The four support layers, and the row model and derivation of every support publication row without depth facts | [Support matrix spec](unified-agent-api/support-matrix.md) | Unchanged. That spec states what a depth-enrolled row adds and how its `uaa_support` is derived. |
| The capability matrix as a publication of advertising | [Capabilities spec](unified-agent-api/capabilities-schema-spec.md) | Unchanged. It publishes no depth fact. |
| The version-only runtime-support payload | [Runtime-support contract](unified-agent-api/runtime-support-contract.md) | Unchanged. Depth facts never enter it. |
| Manual maintenance closeout | Maintenance request contract | Kept. Depth admission is added to it. |
| Manual proving-run closeout and maintainer-gated promotion | Onboarding charter | Kept. Depth admission is added to them. |

Other contracts reference this one for depth rules and MUST NOT restate them.

## Terms

- **Operation.** One upstream CLI invocation mode together with the interface the wrapper promises
  for it. An operation is identified by its command path, its mode and the flags, values and
  defaults that select that mode. One operation MAY own several name identities, and one name
  identity MAY belong to several operations.
- **Interface promise.** What the wrapper, and the `agent_api` adapter where the operation is
  mapped, commit to for an operation across the dimensions in
  [Dimensions and results](#dimensions-and-results).
- **Obligation.** One requirement of an interface promise, carrying a stable id. It is distinct
  from the maintenance request contract's "remaining obligation", which is name-level work still
  owed for a gap identity.
- **Obligation template.** A reusable set of obligations selected by an operation's effects. A
  template is a requirement, never an achieved status. It is not a capability bucket, a lifecycle
  support tier, a runtime profile or a runtime family; `runtime_family` keeps the meaning the
  runtime-support contract gives it. In this contract "template" always means an obligation
  template.
- **Generation.** One packet generation for one agent and one exact upstream version, opened by
  one Event. A maintenance regeneration that records a new Event, the nightly re-dispatch
  included, opens a new generation; the `--from-request` re-freeze completes the same generation.
  One onboarding create-lane run under one approval artifact is one generation.
- **Depth enrollment.** A maintainer-approved positive selection of scope for this contract, for
  one agent, one lifecycle path and one exact upstream version. It covers every generation of that
  version. It is always written with the qualifier. Unqualified "enrolled" in other documents keeps
  its existing meanings: release-watch enrollment of a registry entry, enrollment for publication,
  and the `enrolled` lifecycle stage.
- **Depth scope tuple.** One `(agent, lifecycle path, exact upstream version, operation and
  promise, modes and required values, target)` inside a depth enrollment.
- **Event, P, O and E.** The four bindings defined in [Bindings](#bindings).
- **Depth record.** The durable record of one depth-enrolled version of one agent: one committed
  file in that version's `reports/<version>/` directory under the agent's manifest root. For the
  version's latest generation it states the Event, the resolved depth enrollment selection, the
  content identities of P and O, the identity of E with the P and O identities that E references,
  each obligation's result for each target and, for a `verified` result, the committed evidence
  it rests on, named by path and content identity, and the capability mappings those results
  serve. It also lists an acceptance entry for every closeout, publication or promotion that made
  an acceptance effect for the version. An entry names that closeout, publication or promotion
  and identifies everything the record stated when the entry was made, acceptance entries aside.
  The record is first written when a generation of the version first freezes P, and it is
  extended as O is frozen and as E changes. A later generation of the same version continues the
  same record: the record then states that generation's Event and P, shows O, E and results as
  not yet produced until that generation produces them, and keeps its acceptance entries. Annex B
  defines the record's file name, everything it states, the P, O and E identities and the form
  of an acceptance entry, so that validation can decide, from the record alone, whether each
  identity and each entry agrees with what the record states. A manifest root's validator spec
  adopts that annex.
- **Working files.** The per-agent maintenance packet files of the current generation under
  `docs/agents/lifecycle/`, such as `maintenance-request.toml` and `maintenance-closeout.json`. A
  later generation replaces them.
- **Remainder.** Everything outside every depth enrollment.
- **Depth admission.** The check defined in [Depth admission](#depth-admission). It is distinct
  from the stand-down admission gate of `close-agent-maintenance`. That gate asks whether a packet
  freeze exists, and this contract leaves it unchanged.
- **Depth-gated output** and **depth-gated effect.** An output listed in
  [Depth-gated effects](#depth-gated-effects), and a change to such an output that touches a depth
  scope tuple.
- **Integration branch.** The branch that holds committed publication truth, currently `staging`.
- **Packet freeze.** On the maintenance path, a stand-down marker that the integration branch holds
  for one agent and one exact upstream version. Whoever works that version's packet declares it, to
  stand automation down from the packet. A marker states no selection and commits none. The
  [maintenance request contract](maintenance-request-contract-v1.md#packet-freeze) states its form,
  who declares it, which generation of the version it names, and what it requires of a
  depth-enrolled generation's closeout. It is not a freeze of the request.
- **Committed generation.** A generation whose selection can no longer be given up.
  [Depth enrollment](#depth-enrollment) rule 7 says when a generation is committed and what follows
  from it.
- **Route.** A supported command, workflow, script or lower-level interface, or a merge into the
  integration branch, that can produce a depth-gated effect.

## Minimum machinery

Requirements in this contract MUST be met through existing lifecycle scope, approvals, packet
inputs, validation, tests, reports and closeout wherever those suffice.

A new field, artifact, registry, schema, approval step or persistent mechanism MAY be introduced
only when it prevents a concrete, reachable authority, scope, stale-input or verification failure
that existing machinery cannot prevent. The change that introduces it MUST name that failure and
MUST choose the smallest sufficient addition. This rule is applied inside ordinary design review;
it creates no justification document, checklist or approval gate.

Implementations MUST NOT add a legacy inventory, grandfathering rule, semantic carry-forward
ledger, migration engine, historical-format translator, second release-watch or depth enrollment
inventory, second exclusion list or mutable status ledger for this contract.

## Dimensions and results

An interface promise MUST state every dimension below.

| Id | Dimension | The promise states | What verifies it |
| --- | --- | --- | --- |
| `R1` | Request | Public owner, mode, dedicated controls, defaults, the accepted subset and any intentionally opaque arguments | The production entry receives the intended values. Method existence is not evidence. |
| `V1` | Validation | Which invalid, conflicting and unsupported inputs are rejected before spawn | Rejection tests showing that no child was spawned. Validity decided upstream is stated as such and its failure is tested. |
| `G1` | Values and grammar | Upstream-observed, wrapper-accepted and required values; order, arity, repetition and precedence | Pinned upstream evidence for upstream claims and wrapper tests for accepted behavior. Dynamic identifiers stay open values. |
| `E1` | Errors | The distinctions the interface promises among validation, spawn, exit, timeout, cancel and malformed output | Failure-path tests and redaction checks. No taxonomy beyond what upstream establishes. |
| `A1` | Applicability | Exact upstream version, targets, feature and configuration regime | Test environment and binary or fixture provenance for that version and target. |
| `X1` | Credentials | Credential sources and exposure channels | Synthetic sentinels. No credential in fixtures, logs, receipts or errors. |
| `X2` | Environment and filesystem | Inherited or cleared environment, effective home, config, cache, working directory, writes and network | Isolated sentinel effects and override precedence. A wrapper is not a sandbox. |
| `X3` | Diagnostics | Byte bounds, truncation, raw-output ownership and safe error projection | Malformed, oversized and sentinel-bearing output cases. |
| `X4` | Noninteractive effects | Stdin and TTY behavior, prompts, confirmations and side effects | No automatic answer outside a reviewed interaction. A timeout does not show that no mutation occurred. |
| none | Output | The promised form: bounded capture, records, events or an owned I/O handle | The output obligations its templates select. |
| none | Lifecycle | A finite run, a streaming run, a launcher or a managed protocol | The lifecycle obligations its templates select. |
| `M` | Shared mapping | Wrapper-only, mapped to named capability ids, or a separately proposed extension | For a mapped operation, the capability's minimum semantics exercised through the production adapter. |

`R1`, `V1`, `G1`, `E1`, `A1` and `X1` to `X4` are the BASE obligations and apply to every
executable operation. Output and lifecycle obligations come from the operation's templates.

Each obligation in an operation's set produces exactly one result: `verified`, `unverified`,
`failed` or `not_applicable`.

- `not_applicable` MUST carry a rationale and is valid only where the resolved policy permits it
  for the stated promise. An unavailable target or an unknown effect is not `not_applicable`.
- An unknown output contract is `unverified`. A known counterexample is `failed`.
- A result is `unverified` until it is verified again when the P or the O that its depth record's
  E references is not the one the record states, or when it is invalidated under
  [Reuse and invalidation](#reuse-and-invalidation).
- An obligation with no admissible evidence is `unverified`. While qualifying runs for an
  obligation contradict each other, its result is `unverified`.
- A constrained value subset, such as JSON-only output, is an applicable promise about that
  subset. It MUST NOT be reported as a promise about every upstream value.
- No score, percentage or highest-template label substitutes for per-obligation results.

## Obligation templates

Obligations that templates add to BASE:

| Id | Obligation |
| --- | --- |
| `OUT1` | Bounded capture: explicit byte bounds and truncation, a safe diagnostic projection and truthful status. Promised generic-JSON decoding and malformed-output handling are kept without a record schema. A receipt is parsed only where upstream defines one. |
| `OUT2` | Records: a verified record schema and selectors, malformed and unknown fields, and filtering, ordering and pagination only where upstream exposes them. |
| `OUT3` | Events: a version-bound production parser; known, unknown and malformed events; ordering, session attribution and a terminal result. |
| `OUT4` | Owned I/O: a dedicated handle with explicit ownership of the child's input and output. |
| `LC1` | Finite run: timeout, cancel and child reaping. |
| `LC2` | Streaming run: events may precede completion and EOF is not proof of success; cancel, timeout, drain and drop behavior, bounded buffering and child reaping match the promise. |
| `LC3` | Launcher: spawn failure, I/O draining and backpressure ownership, a stated readiness definition, child exit, stop and drop behavior, and reaping. |
| `LC4` | Managed protocol: initialization and readiness, request and response correlation, notification routing, cancellation, disconnect, terminality and shutdown, across the actual client and process ownership. |
| `FX1` | Mutation: explicit scope and destructive intent, denied-write no-effect, before and after state, retry and idempotency only as promised, and partial effects on failure. |
| `S1` | Session: explicit id, last-session and fork selectors, conflict rules, no-match behavior and returned or continued identity, with no interactive picker or silent new session where the promise forbids one. |
| `N1` | Remote: endpoint and credential intent, local versus remote path intent, and client disconnect and cancel semantics. Ownership of the remote server is excluded unless separately promised. |
| `B1` | Reviewed boundary: a named, constrained invocation path whose effects have been reviewed. |
| `MCP1` | The [MCP management spec](unified-agent-api/mcp-management-spec.md)'s rules for effective home, target and configuration checks, write opt-in, output and errors, its pinned caveats included. |
| `EX1` | Mode exclusion: classification evidence under `R1`, `G1` and `A1`, and rejection tests at the headless boundary. |

| Template | Obligations beyond BASE | Is forwarding to the upstream CLI sufficient? |
| --- | --- | --- |
| `CORE` | `OUT3`, `LC2` | Never as the whole promise. |
| `SESSION` | `S1` | Never as the whole promise. |
| `QUERY` | `OUT2`, `LC1` | Never for a parsed-record promise. |
| `ADMIN` | `OUT1`, `LC1`, `FX1` | Only for a typed intent whose scope is not carried in arbitrary strings. |
| `DIAG` | `OUT1`, `LC1`, `B1` | Yes, inside the reviewed boundary. |
| `OPAQUE` | `OUT1`, `LC1` | Yes. It is an explicitly approved promise, not a substitute for a missing schema. |
| `LAUNCH` | `OUT4`, `LC3` | Only with a dedicated handle and a tested division of protocol ownership. |
| `MANAGED` | `OUT4`, `OUT3` for the selected messages, `LC4` | Never. |
| `MCP` | `MCP1` | For the existing opaque output promise only. |
| `REMOTE` | `N1` | Never from accepting a URL alone. |
| `EXCLUDED` | None. An excluded mode has `EX1` instead of BASE. | Not applicable. No support claim is made. |

An executable operation's obligation set is BASE, `M` for a mapped operation, and the obligations
of every template its promise selects. An excluded mode's set is `EX1` alone.

Classification rules:

1. A **classification selector** MUST name an exact command path and mode, with explicit flag and
   value predicates where they change effects. Aliases are resolved before matching.
2. Classification selectors that overlap with different effects or requirements are an error.
   First match does not win.
3. A flag, value or changed default that alters an operation's effects MUST NOT inherit the parent
   command's templates. A refresh flag on a query, an output path on a diagnostic and a listener
   address on a launcher are examples.
4. A unit with an unknown effect, a new persistent mode or no resolvable template is
   `classification_required`. Acquisition MAY record it. Execution readiness for the affected
   operation MUST wait for classification.
5. A unit inside a governed operation inherits that operation's templates. It inherits a
   forwarding permission only when evidence shows it stays inside the approved effects boundary.
6. Resolving `classification_required`, deciding that a unit alters effects and assigning templates
   are policy decisions the maintainer makes as a change to P. Whether evidence shows that a unit
   stays inside an approved boundary is decided by the independent adequacy review. The executor
   decides neither.
7. `EXCLUDED` applies to one mode. It MUST NOT extend to a headless mode of the same command or to
   the command as a whole. A mode exclusion is a classification entry in P. It MUST NOT create a
   second exclusion list beside `parity_exclusions`, and it never removes a unit from name-level
   accounting.
8. An override names the exact scope, the replacement obligations, the rationale, the versions and
   targets, a maintainer approval reference and an expiry or review condition. It MUST NOT erase
   an existing promise, replace `M` obligations or evade the debt contract. When its condition is
   reached the override lapses and the replaced obligations apply again. An executor MUST NOT
   author an override, relax a requirement or broaden an exclusion to close a packet.
9. Evaluation covers every observed or claimed unit of a depth-enrolled operation, whatever its
   coverage level. A unit at `passthrough` or `unsupported` is evaluated, not skipped. The
   name-level audit lists of the maintenance request contract are unaffected.
10. A generic arbitrary-argument escape hatch never qualifies an operation or satisfies a template.

## Shared mapping and capabilities

Every interface promise MUST declare exactly one shared mapping:

- **wrapper-only**: reachable through the wrapper crate and not through `agent_api`
- **mapped**: backs one or more named capability ids
- **proposed extension**: a new shared capability under separate review

Rules:

1. A mapped operation's `M` obligations are the minimum semantics that the capability's owner
   document defines. This contract neither restates nor changes those semantics.
2. `M` MUST be verified through the production adapter path. Wrapper tests do not verify it.
3. A wrapper-only operation creates no adapter obligation and no advertising.
4. Qualifying a wrapper operation MUST NOT add a capability id to any backend's advertised set.
   Advertising stays governed by the capability documents and the charter's promotion rule.
5. For each capability a depth enrollment claims, the approved policy MUST name the complete set
   of operations through which that agent's adapter honors the capability, covering every flow
   the adapter exposes for it. That mapping is part of the depth record. The capability is
   **depth-qualified** for one `(agent, version, target)` only when every operation in that set is
   depth-enrolled on that target and qualified there, and at least one of that capability's `M`
   obligations, of an operation in the set, has the result `verified` on that target. An operation
   is depth-enrolled on a target when the record's selection holds a depth scope tuple for it on
   that target. The mapping of a claimed capability MUST name at least one operation, and policy
   resolution MUST refuse one that names none.
6. A capability that no depth enrollment claims is **not assessed**. It is neither qualified nor
   failed.
7. The charter's promotion allowlist is an exception to the two-backend threshold only. It is not
   qualification, lifecycle eligibility or permission to advertise.
8. That a capability is absent from the capability matrix because safe defaults leave it off
   establishes nothing about it. The absence is not evidence of qualification, it removes no
   obligation, and it does not keep a claimed capability from being depth-qualified on its own
   evidence.
9. Maintenance MUST preserve or repair mappings that already exist. Adding a mapping is not
   maintenance work and never becomes due through discovery.

The [support matrix spec](unified-agent-api/support-matrix.md#support-depth) owns how
capability-level results are published, and the
[capabilities spec](unified-agent-api/capabilities-schema-spec.md#support-depth) states how the
capability matrix relates to them.

## Evidence

### Evidence classes

Three classes exist, and none substitutes for another:

- **Discovery** binds help output, snapshots or source to an identity and a mode grammar.
- **Controlled-binary mechanics** exercises the production request, process, parser and adapter
  paths under deterministic scenarios.
- **Upstream provenance** binds real output and effects to an exact version, target and build, or
  to an explicit compatible source contract.

Help output does not verify parsing. An argv echo does not verify lifecycle. A fixture written to
match the parser tests mechanics and does not show that upstream emits it.

### Verification evidence

Evidence that verifies an obligation MUST establish, directly or by reference:

- the obligation ids, and the P and O it was produced under
- the exact target and upstream version
- upstream binary, source or fixture provenance
- content identities of the production code, tests and dependencies it covers
- toolchain, features and the relevant environment
- the tests requested and the tests actually executed, with per-case outcomes
- the producer and the run identity

Verification MUST be rejected when:

- a bound test is missing, filtered out, skipped, ignored or aborted, or the executed selection
  differs from the bound one
- the command succeeded and executed zero tests
- the agent, target or upstream version is wrong
- the binary, features, toolchain or build inputs changed
- an assertion, fixture or requirement is stale
- the evidence was produced under a different P or O and no reuse binding covers it
- the producer is not one the existing repository and CI trust rules recognize, or the evidence
  comes from an unrelated CI job
- the only support is an author-entered success value

Contradictory qualifying runs require investigation. A favorable run MUST NOT be selected
silently. A CI conclusion associated with a pull request head establishes that association. It
does not establish which tree was built.

The [revision-1 evidence receipt contract](support-depth-evidence-v1.md) specifies the typed
execution/adequacy receipt, strict workspace file binding and the approved initial CI producer
and authenticated maintainer-endorsement profiles. Its preparatory reader establishes form and
internal content identity only; it does not supply the verification or admission predicate here.

### Adequacy

An obligation is `verified` only when a reviewer independent of the executor has bound that
obligation to the assertions that exercise it, by content identity and not by test name, count or
coverage percentage. The review establishes that removing the production validation, parser or
cleanup under test would fail the assertion.

Adequacy review uses the existing independent review and closeout authority. It creates no
standing reviewer role. An unchanged, admissible mapping is not reviewed again.

### Reuse and invalidation

- Evidence applies to its exact upstream version and target. A maintainer-approved compatibility
  rule MAY extend it per obligation. The rule names the versions, the targets, the invariants it
  relies on, the residual tests it requires and what invalidates it.
- Process termination, ABI, path, permission and other platform-dependent proof MUST NOT be reused
  across targets under such a rule.
- A change to the implementation, tests, fixtures, requirements, upstream inputs or dependencies
  an obligation covers invalidates its result. The dependency set MUST include at least the
  affected crate's full resolved build graph and every explicitly referenced adapter, protocol,
  test and fixture input. Missing dependency provenance is `unverified`, not an empty set. An
  unrelated change invalidates nothing.
- Evidence produced under one P or O verifies an obligation under another only through a **reuse
  binding** recorded in E. For each reused obligation the binding establishes that the
  obligation's definition and the applicable trust and review rules are unchanged, that the
  version and target are identical or covered by an approved compatibility rule, and that every
  input in the obligation's dependency set is unchanged. The evidence keeps its original P and O,
  and newly due or changed cases run fresh.
- Evidence verifies an obligation of a production depth enrollment only if it was produced under
  the P and O of a production depth enrollment. Evidence produced under one version's P and O
  serves another version only through a reuse binding under an approved compatibility rule.
  Evidence produced outside every production depth enrollment, a successful closeout included, is
  not admissible for a production depth enrollment.
- Evidence is repository verification inside the existing repository and CI trust boundary. It is
  not remote attestation.

## Proof, depth enrollment, qualification and acceptance

These are four separate facts. One existing approval MAY establish more than one of them where its
recorded authority covers them. They do not require separate status fields, artifacts or approval
steps.

| Fact | Establishes | Does not establish |
| --- | --- | --- |
| Path machinery proof | A lifecycle path can enforce this contract across its whole acceptance chain, shown in isolation | Qualification of any real operation |
| Depth enrollment | These rules apply to one exact approved scope | That implementation or evidence work is finished |
| Qualification | Every obligation of one promise is `verified`, or `not_applicable` where permitted and justified, on exact inputs | Closeout, publication or promotion authority |
| Acceptance | The qualification and lifecycle requirements for one authorized effect are met at the time of that effect | Authority for another version, target, scope or release |

Rules:

1. Depth enrollment MAY begin with implementation and evidence work outstanding.
2. Authorized debt MAY disposition the name-level work it covers for acceptance under its
   controlling contract. It never yields `verified` and never makes an operation qualified.
3. An empty due list is not qualification. A qualified subset MUST be named by its exact promise
   and scope.
4. The remainder carries no depth claim. It MUST NOT be counted as `verified` or `not_applicable`.
5. Publishing a truthful assessment with `unverified` or `failed` results, or before results exist,
   MUST remain possible. It MUST NOT require qualification, and it MUST NOT produce an acceptance
   effect.

## Bindings

A depth-enrolled generation keeps four logical bindings. They are relationships, not four files,
stores or schemas, and existing packet, report and closeout fields carry them where sufficient.
Shared storage MUST NOT merge their authority or their invalidation.

| Binding | Content | Who may change it | A change invalidates |
| --- | --- | --- | --- |
| **Event** | Maintenance: `request_commit`, `request_recorded_at`, `trigger_kind` and `opened_from`. Onboarding: the approval artifact's `approval_commit` and `approval_recorded_at`. | Nobody inside a generation. A later generation has its own Event. | Nothing. It is attribution, not tested code. |
| **P**, policy | Resolved rules, templates and classifications, promises and subsets, overrides and exclusions, depth enrollment selectors, path enablement, the debt delegation and the initial authorization baseline, for the depth scope tuples it governs | The maintainer, through explicit re-freeze or supersession. On the maintenance path P changes only when a new generation opens, as the maintenance request contract's freeze rules state | O, E and every dependent closeout and publication result |
| **O**, obligations | The P reference, exact version and targets, acquired input identities, operation-to-surface edges and the concrete obligation set, independently required acceptance work included | Only the existing acquisition and preparation path, at its freeze and at any later re-freeze. A re-freeze MUST NOT change the agent, version, targets or depth enrollment. The executor never changes O. | E and every dependent closeout and publication result |
| **E**, execution | The P and O references, implementation and evidence identities, reuse bindings, materialized debt grants, validated transitions from P's baseline and derived results | Execution, evidence refresh and delegated debt transitions | Dependent closeout and publication results |

Rules:

1. Acquisition MUST NOT grant, widen or create policy authority. A unit discovered under an
   approved template changes O and not P.
2. E MUST NOT rewrite P or a frozen O. Reconciliation compares every changed grant against P's
   baseline and the allowed transitions. An unchanged policy identity is not sufficient.
3. A policy amendment is prospective. It MUST NOT relabel evidence produced under an earlier P as
   meeting the amended rules.
4. A debt transition is not a policy waiver. See [Debt operations](#debt-operations).
5. P is resolved for the depth scope tuples it governs. A change to another agent's or path's
   selection, or the enablement of another path or of the same path for another agent, does not
   change it. A change to a shared governing rule does.
6. A binding's identity MUST NOT depend on its own digest, on a future output or on the commit that
   will contain it.
7. The freeze points of the maintenance request contract are unchanged. P is frozen when the
   generation opens, at the request's first freeze. On the acquisition lane that opens without
   target reports, O is frozen at the existing second freeze and the opening placeholder stays
   non-executable. A lane whose reports exist at open keeps its existing treatment. A generation in
   which O is never frozen from target reports, on the docs-only lane or on an acquisition lane
   stood down before its second freeze, cannot reach acceptance for depth-enrolled scope. That
   scope is reported as insufficient depth and never as not depth-enrolled. The
   [maintenance request contract](maintenance-request-contract-v1.md#depth-enrolled-generations)
   states how a request carries P and O and what each freeze writes. The
   [onboarding charter](cli-agent-onboarding-charter.md#support-depth) states where P and O are
   frozen on the create lane; see [Path enablement](#path-enablement).
8. A consumer acting on depth-enrolled scope MUST require the current executable schema revision
   and every binding the depth record states. A binding that the record shows its generation has
   not yet produced is read as such, and its scope as insufficient depth. A missing field or an
   unsupported revision is an error. None of these is read as "not depth-enrolled".
9. Artifacts completed under an earlier contract keep their original meaning. They are not
   translated, backfilled or presented as depth evidence. An unfinished generation that an adopted
   revision makes incompatible MUST stop. Work continues only in a new generation prepared through
   existing preparation, with fresh bindings; on the onboarding path that means a new approval
   artifact.

### Debt operations

Debt rows are name-level. A row dispositions the name-level required work it covers. Until the debt
contract is amended to identify depth obligations explicitly, no row dispositions a depth
obligation, and a due depth obligation is dispositioned only by being satisfied. The **frozen
delegation** is the set of debt rows and renewal permissions in force when P was frozen, and the
**initial authorization baseline** is the content of those rows at that time.

| Operation | Who | Limits | Binding treatment |
| --- | --- | --- | --- |
| Renew an existing grant | Executor | Only a row inside the frozen delegation, and only while its `blocker_class` still holds. Only `scope_target_triples`, `authorized_at_version` and `authorization_evidence_ref` change. Targets stay inside the delegation and match current target reports. | P unchanged. E changes, and dependent closeout and publication results become stale. |
| Retire debt | Executor | Only under the retirement conditions of the maintenance request contract. Absence from help output is not proof of removal. | As above. |
| Refresh evidence | Executor | Inside the selected contract. | P and grants unchanged. |
| Add debt or widen authority | Maintainer | A new identity, `blocker_class` or `current_reason`, or a target outside the delegation. The executing packet MUST NOT defer a newly discovered non-TUI gap. | P re-freeze or supersession, with downstream invalidation. |

An unsupported or unproved transition MUST fail closed. Execution never closes or promotes its own
work.

### Read-only delegated-debt planning

The D3 planner reconciles the complete agent debt inventory with P's frozen baseline and E's
materialized grants/retirements, including rows outside the selected version or targets. It MUST
match row ids and full immutable row contents, reject duplicate ids and overlapping grants, and
permit renewal changes only to the three fields listed above. Undeclared additions, deletions or
changes MUST refuse; candidate inventory never redefines P. Other agents' rows confer no authority.

Renewal MUST stay within the frozen row's targets and current same-surface coverage observations.
Whole-row retirement MUST account for every frozen target using exact-target reports; a remaining
gap or missing/contradictory report refuses. A report observation of coverage, exclusion or absence
is a claim requiring corroboration, not proof of semantic coverage, valid exclusion or upstream
removal. Absence from help MUST NOT be treated as upstream removal.

A read-only plan MUST expose unresolved proof requirements: renewal owes current evidence that
the unchanged blocker holds, with its owner, milestone and follow-on; retirement owes live
coverage/exclusion proof or explicit upstream-removal proof for every target. Content hashes and
report observations alone MUST NOT discharge these requirements. D3 installs no new semantic
proof producer. Conditional planning success MUST NOT authorize writes, closeout, promotion,
publication upgrades or debt waivers. Owning routes MUST independently establish these facts
before admission; unsupported proof stays unresolved. No second inventory, evidence schema,
permission token or automatic maintainer endorsement is introduced.

The planner MUST preserve Event, P, O, acceptance entries and unrelated E results. Materialized
debt edits change E and the stated record digest; an earlier entry cannot accept the new state.
Strict audit reconciliation and audit-status MUST compare the same debt baseline projection,
including contributing row references, deferral reason/follow-on and counts. Projection equality
does not replace full inventory comparison. Surface counts remain aggregated; when grants
contribute, only the smallest contributing grant supplies rendered metadata.

## Depth enrollment

1. A depth enrollment is a positive selection that binds the agent, the lifecycle path, one exact
   upstream version, the operations and their promises, their modes and required values and
   explicit targets. It covers every generation of that version. This revision defines depth
   enrollment of one exact upstream version only. It defines no rule that carries a selection to a
   later version.
2. A later version inherits nothing from an earlier depth enrollment: no depth enrollment, no
   qualification and no publication authority. How a later version may move a depth-enrolled
   version's pointer or replace its working files is set out in [Later versions](#later-versions).
3. A production depth enrollment is one declared in the registry as the integration branch holds
   it, or one that rule 7 keeps. The [registry contract](agent-registry-contract.md#support-depth)
   states how one is declared. A declaration MUST NOT be added, or moved to another lifecycle path,
   unless the same commit leaves that lifecycle path enabled for its agent. Restoring the
   declaration of a version that has a committed generation, on the lifecycle path that
   generation's selection names, is not an addition. A selection made inside an isolated proof
   workspace is not a production depth enrollment and MUST NOT produce a depth-gated effect on
   production outputs.
4. The remainder is declared once, by rule. It MUST NOT be enumerated per historical operation,
   modeled as disabled placeholders or compared against an adoption baseline.
5. The remainder carries no depth claim. It is not an exclusion, a waiver or debt, and every
   existing maintenance, regression, onboarding, debt, acquisition and publication duty still
   applies to it.
6. Depth enrollment adds obligations. It MUST NOT remove or narrow required uplifts, target
   acquisition completeness, existing shared promises or the release-watch ratchet.
7. Resolution MUST be deterministic. Registry-owned authority and the frozen request or approval
   resolve a version's depth enrollment. Each generation freezes its P from registry-owned
   authority, and where the tree it freezes in already holds the version's depth record, that P
   MUST cover every invocation mode the record's selection covers, on each target the mode is
   covered on, and MUST claim every capability id the record claims. A generation is committed from
   the time the integration branch holds the depth record as the generation wrote or continued it.
   On the maintenance path the merge of its packet brings that about, and a packet freeze commits
   nothing. Everything a committed generation's selection covers
   stays depth-enrolled under a production depth enrollment, whatever the registry declares
   afterwards, and every later generation of the version MUST cover what that selection covers and
   claim what it claims. A generation that is not committed may be replaced or abandoned together
   with its record, and nothing of it carries over. Whether a generation is committed is
   established at the integration step, against the tip. Before that step a route reads the record
   its own tree holds as the one the integration branch holds. An executor's assertion, an
   optional field of a request or approval and a caller-supplied argument are not authority.
8. Missing policy for selected scope, an unsupported schema revision, unresolved or overlapping
   depth enrollment selectors, contradictory generation references and deleted bindings are errors.
   None of them resolves to "not depth-enrolled". A removed depth record is a deleted binding. A
   replaced working file is not, because the depth record identifies the bindings that file
   carried.
9. A flag, value or default discovered on a depth-enrolled operation stays attached to that
   operation. It MUST NOT fall into the remainder because a depth enrollment selector matched
   the earlier values.
10. Work that is due in a committed generation stays due until it is satisfied or validly
    dispositioned. Moving it to another packet, ending the selection, pausing, removing
    advertising, deleting a declaration, or replacing or abandoning the generation does not
    discharge it. The version's depth record carries that work, and a later generation of the
    version carries it in turn. A generation that was never committed leaves no due work when it is
    replaced or abandoned.
11. Depth enrollment MUST NOT create a second release-watch enrollment inventory.

## Depth admission

### Depth-gated effects

A depth-gated effect is a write, replacement, regeneration, removal or restoration of a
depth-gated output that changes a value belonging to at least one depth scope tuple.

| Depth-gated output | Values that belong to a depth scope tuple |
| --- | --- |
| Depth records | Everything the depth record of the tuple's version states for the tuple, and its acceptance entries |
| Support publication rows and their Markdown projection | The row for the tuple's agent, version and target |
| Capability publication | The entry for each capability a depth enrollment claims, for the tuple's agent |
| Version metadata | The tuple's version file: its status and its per-target outcomes |
| The manifest root's `current.json` | Whether it lists the tuple's target as expected |
| The `latest_validated` and `latest_supported` pointers, the root `latest_validated.txt` included | Each pointer for the tuple's target, and the root pointer, while it names the tuple's version, and any change that would make it name that version |
| The embedded runtime-support projection | The record for the tuple's runtime family and target while it names the tuple's version, and any change that would make it name that version |
| Stand-down markers | Removal of a marker for the tuple's version. Declaring or changing a marker is not a depth-gated effect. |
| Maintenance requests and maintenance and proving-run closeout records | The current file while it belongs to a generation of the tuple's version |
| The lifecycle record | Its `published` and `closed_baseline` stages, its `proving_run_closeout_written` and `maintenance_closeout_written` evidence ids, its drift side state, and the publication packet and proving-run closeout references it holds. Each belongs to the tuples of the generation whose packet or closeout it reports, as that packet or closeout stands in the same revision |

The lifecycle record names no version, so its values are attributed through the packet or closeout
they report and never through the route that writes them. A `published` stage reports the
publication packet the lifecycle record names. A `closed_baseline` stage and the
`proving_run_closeout_written` evidence id report the proving-run closeout it names. The
`maintenance_closeout_written` evidence id and the absence of a drift side state report the agent's
maintenance closeout. A reported packet or closeout belongs to the generation whose frozen request
or approval it names. Once a later generation has replaced that request or approval, it belongs to
the version whose depth record lists the acceptance entry that names it. No other lifecycle value
belongs to a tuple.

A depth record binds each `verified` result to committed evidence by path and content identity, and
that binding alone decides what the result's evidence is. Coverage reports and
`wrapper_coverage.json` are name-coverage artifacts and are not depth-gated outputs. A result that
depends on their content is invalidated by a change to that content under
[Reuse and invalidation](#reuse-and-invalidation), and this contract does not otherwise restrict
regenerating them. Changing or removing bound evidence is not prohibited either. The record is not
rewritten for it: the result has no current evidence and is published as `unverified` until it is
verified again.

The following **record invariants** hold for every depth-gated output:

1. A depth-enrolled version has a depth record from the time a generation first freezes P for it.
   A depth record MUST NOT be removed, whether by retention pruning or otherwise, and an
   acceptance entry MUST NOT be removed or altered. A record's selection MUST NOT shrink: every
   invocation mode it covers stays covered on each target it is covered on, whichever operations
   then serve it, a mode that P comes to exclude under classification rule 7 stays covered, and
   a capability its mappings claim stays claimed. Annex B states how a record's coverage is
   compared. A manifest root whose validator spec does not adopt Annex B MUST NOT hold a depth
   record.
2. What a record states changes only when a later generation of the same version continues the
   record, or through a change that [Bindings](#bindings) permits to the binding concerned.
   Annex B assigns everything a record states, its Event aside, to P, O or E: the selection and
   the capability mappings belong to P, the obligations to O, and the results, their bound
   evidence and the P and O identities that E references to E. Listing an acceptance entry is
   not a change to what a record states. The P, O and E identities a record states each cover
   what it states under that binding: nothing stated under a binding changes while that
   binding's identity stays the same.
3. A published result is the result the record states, or `unverified`. It MUST be `unverified`
   in three cases: the record does not yet state the result; the P or the O that its E references
   is not the one the record states; or the record states the result as `verified` and binds no
   evidence for it, or binds evidence that is absent from the revision or has another content
   identity. A change to a dependency outside the bound evidence is not within this invariant. A
   promise is published as qualified, or a capability as depth-qualified, only when every result
   the claim rests on is published as `verified`, or as `not_applicable` where P permits it. A
   capability is published as depth-qualified for a target only when its mapping meets
   [Shared mapping](#shared-mapping-and-capabilities) rule 5 on that target. A tuple the record
   selects MUST NOT be published as not depth-enrolled.
4. Every acceptance effect other than listing an acceptance entry MUST have an acceptance entry
   made for what the record stated when the effect was made. A promise is published as qualified,
   or a capability as depth-qualified, only while an acceptance entry exists that was made for
   what the record states now. In the lifecycle record of an agent that has a depth record, a
   `published` or `closed_baseline` stage and a closeout evidence id MUST report a packet or
   closeout that the same revision holds.

A depth-gated effect is an **acceptance effect** for a tuple when it does any of the following:

- sets or advances a pointer, or the runtime-support projection, to the tuple's version
- sets the tuple's version status to `validated` or `supported`, or records a passed per-target
  outcome
- retires a stand-down marker
- creates or semantically changes a maintenance closeout record, records a proving-run closeout
  as `closed`, or semantically changes it while it remains `closed`. Actual timestamp values count;
  serialization and set ordering alone do not
- advances the lifecycle record's stage to `published` or `closed_baseline`, adds one of its
  closeout evidence ids or clears a drift side state. A still-present accepting stage, evidence id
  or absence-of-drift assertion also counts when it reports a different actual packet or closeout
  present in the candidate, including replacement bytes at the same path. Reference identity uses
  the path and raw content digest. Disappearance alone supplies no acceptance destination and
  imposes no general maintenance-closeout-presence requirement
- lists an acceptance entry in a depth record, or publishes a promise as qualified or a capability
  as depth-qualified
- adds capability advertising

When `drifted` is removed, including by removal of the lifecycle record, and the agent has a depth
record in either revision, the candidate maintenance closeout MUST resolve its generation in that
same revision. A destination covered by a retained record requires its current entry. A positively
resolved ordinary destination is ordinary; missing, malformed, conflicting or unresolved attribution
MUST fail. An unchanged absence of drift is not a new clearance. A binding held only in the tip
cannot authorize candidate artifact bytes.

Every other depth-gated effect is a **reporting effect**.

### Depth admission predicate

For every depth scope tuple a depth-gated effect touches, the route MUST establish:

1. that depth enrollment resolves for the tuple under [Depth enrollment](#depth-enrollment);
2. that what it writes states the tuple's results as they are. A published result is written as
   `unverified` in each case record invariant 3 names, and when
   [Reuse and invalidation](#reuse-and-invalidation) has invalidated it. Nothing is written as
   better than the evidence the record binds for it supports; and
3. that it removes no depth record, removes or alters no acceptance entry, shrinks no record's
   selection, and changes what a depth record states only as record invariant 2 permits.

For an acceptance effect the route MUST also establish:

4. that it holds authority for the tuple under the lifecycle path that owns the tuple's depth
   enrollment;
5. that P, O and E are present, mutually consistent and current; and
6. that no unit of the tuple's operations is `classification_required`, and that every
   independently required work item and every due depth obligation authoritative for the tuple is
   satisfied or validly dispositioned under its controlling contract.

A reporting effect needs no lifecycle-path authority. If depth admission fails, every depth-gated
output MUST be left unchanged.

On the maintenance path, the authority that item 4 requires for the closeout of a depth-enrolled
generation includes a packet freeze that names the generation. The
[maintenance request contract](maintenance-request-contract-v1.md#packet-freeze) states that
requirement. The stand-down admission gate asks only whether a marker exists for the version, not
which generation it names.

### Complete mediation

Every route MUST, before a depth-gated effect, either establish depth admission for every tuple the
effect touches or refuse the effect.

1. The tuples an effect touches are determined by the values it would change, not by the agent,
   version or scope the request names. A regeneration requested for one agent that would change
   another agent's depth-enrolled row touches that row's tuple. A request for a version outside
   every depth enrollment that would move a depth-enrolled pointer touches that pointer's tuple;
   see [Later versions](#later-versions).
2. Not matching a depth enrollment is never permission to change a depth-enrolled tuple's depth
   record.
3. For an acceptance effect, authority is established per touched tuple under the lifecycle path
   that owns that tuple's depth enrollment. Reaching a shared writer from another path confers
   none. A gate that admits on the wrong path's authority does not satisfy this section.
4. A route MUST NOT produce an acceptance effect on a tuple whose depth enrollment belongs to
   another lifecycle path.
5. A lower-level writer is governed through every caller that can reach it, not only its usual
   one.
6. A route MUST NOT narrow the touched set because a caller omitted an optional scope or
   depth enrollment argument.
7. Repository validation is the backstop for this section. On the integration branch tip, and on
   every merge result proposed for the integration branch, it MUST fail when a record invariant
   in [Depth-gated effects](#depth-gated-effects) does not hold, whatever produced the change: a
   supported route, unsupported tooling or a hand edit. This contract does not enumerate the
   checks that enforce the invariants. A manifest root's validator spec states them, beside its
   adoption of Annex B, and they are part of the enforcement that
   [Path enablement](#path-enablement) item 4 requires.

### Later versions

An effect for a later version that moves a depth-enrolled tuple's pointer or runtime-support
record, changes its row because a pointer moved, or replaces the working files of the tuple's
generation, displaces that tuple.

1. For the displaced tuple the effect is a reporting effect. Its row continues to state its own
   results, and its depth record is not rewritten.
2. The later version's state is admitted under the rules that apply to it: its own depth
   enrollment where it has one, and otherwise the existing lifecycle rules, which this contract
   leaves unchanged.

### Final depth admission

The gated sequence is the part of a route that reads the current state of the outputs it will
change, establishes depth admission and writes them, together with any restoration on failure. A
route MUST:

1. hold serialization ownership of the gated sequence before its first write;
2. establish depth admission while holding that ownership, against the frozen bindings, current
   authority, the relevant inputs and the expected state of the outputs it will change; and
3. keep that depth admission valid through its last write and through any failure handling.

For a route whose writes reach the integration branch by merge, the writes on its branch prepare a
candidate. Its gated sequence is the integration step alone: reading the integration branch tip,
forming the merge result, establishing depth admission against that result and merging. Depth
admission established while the candidate was prepared was established earlier and MUST be
established again at the integration step. The integration step MUST change the integration branch
only through a ref update that is refused when the tip moved after depth admission was
established, such as a fast-forward-only push or a merge that re-checks the exact merge result.
That ref update is the integration step's serialization: with it, depth admission established
against the exact merge result satisfies the three requirements above for that step.

Serialization:

- Two routes share a serialization domain when one can change a depth-gated output or a depth
  admission input that the other writes or relies on. Routes that only read the same inputs do not
  share a domain for that reason. Ownership excludes every other member of the domain from those
  changes until the gated sequence ends. Serialization within one route's own lane does not
  satisfy this rule.
- An invalidating change is any change, by any writer, to the bindings, authority, relevant inputs
  or expected output state that depth admission relied on. A check made inside ownership, with
  every invalidating change excluded until the gated sequence ends, is the final check and need
  not be repeated. A result obtained earlier MUST be established again.
- This revision requires serialization of gated sequences other than the integration step, whose
  serialization is its tip-conditional ref update. Per-writer compare-and-refuse MAY replace the
  serialization of those other sequences later by amendment.
- No cross-process lock or atomic multi-file transaction is assumed to exist. A route demonstrates
  its serialization by an Annex A entry that names every writer in its domain, together with a
  conformance test. A route that cannot demonstrate it MUST refuse depth-gated effects.

Restoration and interruption:

- Restoring outputs from a snapshot is a depth-gated effect. A restoration that would overwrite
  state the route no longer owns MUST NOT run; the route reports the conflict and stops.
- An interrupted sequence MUST NOT leave in place any acceptance effect for which depth admission
  was not established.

### Route inventory

Annex A lists every depth-gated output with its routes. For each output it records the affected
scope, aggregate and indirect changes included; every reachable route, merges into the integration
branch included; the authority each route acts under; whether the route establishes depth
admission or refuses; the other writes and restorations in its serialization domain; and the source
references and conformance evidence. It states present behavior and required behavior separately.

- The inventory is built from the outputs backward to every writer and from the entrypoints
  forward to their effects, and the two views are reconciled.
- It is final for the source revision it names. It MUST be established again against the revision
  that carries enforcement before a path is enabled.
- A change that adds or alters a route MUST update the inventory and its conformance coverage
  before that route may perform a depth-gated effect.

## Path enablement

Enabling a path turns enforcement on for that path's scope. It asserts nothing about any
operation's qualification.

The maintenance path MAY be enabled when all of the following hold:

1. this contract and the executable schema revisions it requires are adopted, with explicit
   maintenance path and selection authority. Those revisions are the registry contract's
   `support_depth` table, the depth record of Annex B and the support matrix's next revision, and
   for the onboarding path the field of the approval artifact that states the version;
2. the complete maintenance chain of acquisition, frozen obligations, execution, audit, manual
   closeout and separate promotion is proven end to end in an isolated workspace that cannot
   change production pointers or closeouts;
3. every route in Annex A satisfies [Complete mediation](#complete-mediation) and
   [Final depth admission](#final-depth-admission), routes that belong to the onboarding path and
   merges into the integration branch included;
4. that enforcement is landed, with reviewed positive and negative gate evidence; and
5. the maintainer authorizes enabling the path for a bounded production scope.

Enabling the maintenance path does not require the onboarding path to be proven. A single route
that fails item 3 blocks it.

The maintainer's authorization under item 5 is recorded in the registry, agent by agent, as the
[registry contract](agent-registry-contract.md#support-depth) states. Each authorization enables
the path for one agent, and the conditions for enabling that path MUST hold when it is given.
Items 1 to 4 concern the path as a whole.

The onboarding path MAY be enabled when items 1, 3, 4 and 5 hold for it, the onboarding charter
defines where P and O are frozen on the create lane, at a point where the agent's manifest root,
its validator spec and its exact upstream version exist, the charter names the committed artifact
that states, beside the depth record, the P and O identities a create-lane freeze froze, with who
writes it and who may replace it, and an independent new-entry run in an
isolated workspace proves approval, the onboarding checklist's implementation and evidence steps,
runtime evidence selection, publication and proving-run closeout. Maintenance evidence does not
substitute. Enabling it MUST establish item 3 again for the maintenance path.

A synthetic agent in an isolated workspace MAY prove the onboarding machinery. It qualifies no
real agent. A real agent's promised scope is qualified by its own upstream evidence before any
acceptance effect.

A cross-agent workflow that adds shared mappings MUST NOT be enabled until the maintenance and
onboarding chains are both proven and landed.

When a prerequisite stops holding, new acceptance effects for depth-enrolled tuples on that path
MUST be refused. When the maintainer withdraws the authorization for an agent, new acceptance
effects for that agent's depth-enrolled tuples on the path MUST be refused. The path MUST NOT
fall back to rules that predate this contract. A production depth enrollment whose path is not
enabled for its agent, however that came about, stays a production depth enrollment: its tuples
stay depth-enrolled, reporting effects for them continue, and they receive no acceptance effect.

## Additive shared integration

Adding a shared mapping for an operation that already landed in a wrapper is separate work from
maintenance and from the onboarding checklist's integration steps. This contract fixes its rules
and defines no command for it.

1. Candidates are derived from landed wrapper operations, their approved promises and their
   existing evidence. Names and help output alone do not make a candidate. Derivation reuses the
   shared implementation that [Purpose](#purpose) requires and MUST NOT become a second coverage or
   qualification engine.
2. Candidates are grouped by proven request, effect, output, error and lifecycle semantics. Equal
   command names are not equivalence. An uncertain equivalence stays a visible question.
3. Each candidate receives one disposition from the maintainer or from an existing, explicitly
   bounded approval authority: map to an existing capability, propose an extension, keep
   wrapper-only, or defer with a rationale and an owner. A disposition is a routing decision, not
   an achieved result.
4. A pending or deferred candidate is neither verified shared support nor debt. It MUST NOT excuse
   required wrapper work, and it never blocks maintenance acceptance.
5. Selected work runs through the existing packet machinery with a frozen input identity and write
   envelope. Stale inputs for a participating agent or for the shared contract MUST be rejected at
   preparation, execution and acceptance. An edit to an agent that is not participating is not
   staleness.
6. A candidate keeps a stable identity. An unchanged rescan reuses the open packet. When a group
   splits or merges, every selected operation keeps exactly one active disposition and packet
   owner, and no existing mapping, debt identity or approved selection is orphaned or widened.
7. Publication requires approved advertising, adapter evidence and the charter's promotion rule. A
   single-agent selection does not bypass any of them. A backend-namespaced result MAY stay
   backend-specific and is then not a promoted universal capability.
8. Once a mapping is accepted, later maintenance MUST preserve it.

## Publication

- Depth facts MUST be published from depth records. A depth record is committed evidence under the
  agent's manifest root in `reports/**`, an evidence category the support matrix's neutral root
  intake already reads. A second evidence store MUST NOT be introduced. The
  [support matrix spec](unified-agent-api/support-matrix.md#support-depth) states what a
  depth-enrolled row publishes.
- Aggregates MUST keep per-target results. One target's evidence never qualifies another.
- Machine-readable and Markdown depth facts MUST agree.
- The runtime-support payload stays version-only. Keeping depth facts out of that payload does not
  exempt a depth-enrolled tuple's pointer change from depth admission.
- Workflow YAML transports inputs and invokes shared commands. It MUST NOT carry template,
  depth enrollment, exception or depth admission policy.

Reports MUST keep these outcomes distinguishable:

| Outcome | Meaning |
| --- | --- |
| Qualified forwarding | A bounded forwarding promise with every obligation `verified`. It MAY still carry the `passthrough` coverage level. |
| Insufficient depth | A depth-enrolled obligation is `failed` or `unverified`, or the version's obligations are not yet frozen. It is due. |
| Authorized debt | Unavailable behavior covered by a valid target- and version-scoped name-level grant. Never `verified`. |
| Mode exclusion | One TUI mode excluded with a rationale, without a second exclusion list. |
| Not depth-enrolled | The remainder. No depth claim. |
| Classification required | A unit of a depth-enrolled operation has no resolved template. It is recorded and published, and it blocks execution readiness and acceptance effects for that operation. |
| Unresolved | Depth enrollment could not be resolved. Validation reports it as an error, and depth-gated effects for the tuple are refused until it is resolved. It is never reported as not depth-enrolled. |

## Conformance

An implementation conforms when tests on the production paths demonstrate every requirement of
this contract, positive and negative cases included. The acceptance cases in section 7 of the
[proposal](../agents/lifecycle/support-depth-policy-proposal.md) are non-normative guidance for
those tests. The checks and attack scenarios in the
[validation test list](../agents/lifecycle/support-depth-validation-test-list.md) are non-normative
guidance for the tests of the record invariants.

D4's MA4 implementation executes in an independent candidate and compares host-held invocation
snapshots through the existing effect detector (`xtask/agent_maintenance/execute/guarded.rs`,
`execute/candidate.rs`, `xtask/depth_acceptance/relay.rs`). It refuses acceptance effects,
request/marker changes and unresolved debt authority before expected-before delivery. Rejected
candidate outputs are retained outside the caller workspace. This is bounded relay mediation;
closeout, promotion, remaining writers, final integration admission and the full maintenance proof
remain separate work, and production depth remains disabled.

## Annex A: route inventory

Source revision: `staging` at `dd10558157583f1969b4ac0a412107ef617648e1`
(PR #296, D7 plus the stream-parity synchronization repair). This inventory replaces the
pre-enforcement snapshot at `f61534be` and its bounded updates. Paths below are repository-relative;
`xtask/` abbreviates `crates/xtask/src/`, and `wf/` abbreviates `.github/workflows/`.
It inventories present source behavior, not completion of the isolated maintenance-chain proof.
The [phase E proof plan](../agents/lifecycle/support-depth-isolated-proof-plan.md) names the
remaining evidence and proposed scope; its policy recommendations are not authorizations.

Host configuration was read without mutation on 2026-10-24 America/Indianapolis
(`2026-10-25T03:04:34Z`): `scripts/check_staging_ruleset.py` reported `match`, ruleset
`25609013`, desired SHA-256 `7dc816f61e8cd940def0922db09021b6a7da54b79266dc1e2cac757690de9d3d`.
The active staging rules have no bypass actors or overlapping legacy protection, require a PR,
MERGE-only native queue, strict status checks bound to GitHub Actions app `15368`, the terminal
`Staging integration gate` and five direct checks. The terminal gate evaluates the existing
17 jobs; legitimate skips are justified from candidate inputs. This is a time-bounded
configuration observation, not a fresh credential-specific host-proof run. The versioned desired
[ruleset](../agents/lifecycle/staging-integration-ruleset.json) and
[runbook](../agents/lifecycle/staging-integration-enforcement.md) remain authoritative for comparison.
PR #296's queue CI [40155444440](https://github.com/atomize-hq/unified-agent-api/actions/runs/40155444440)
and post-merge depth validation
[40156225694](https://github.com/atomize-hq/unified-agent-api/actions/runs/40156225694) passed.

Three distinctions apply throughout this inventory:

- **Public refusal:** `AgentRegistry::parse` rejects every declared `support_depth` table.
  Additional retained-record guards prevent omission of the declaration from bypassing unsupported
  writers. A refused route has no positive production-depth claim.
- **Owning core:** preparation, relay, closeout and promotion have bounded private/admitted paths
  with independent checks. Their positive fixed-producer fixtures prove composition, not a public
  CLI path with authenticated adequate evidence.
- **Final integration:** D7 re-establishes admission against the exact candidate and parent before
  the native queue's conditional ref update. A local plan, caller name, matching digest or existing
  entry is not an integration receipt. `production_results` currently returns no verified results;
  the installed authenticated CI profile proves isolated V1 only. Thus no production promise or
  full acceptance-chain success follows from the installed guards.

### A.1 Routes

Existing route IDs are preserved; NE8 and LW1 make the generated advertising writer and lower
storage boundaries explicit. "Branch" means candidate preparation followed by IN1. A neutral
route has no lifecycle acceptance authority. Source references name the entry and owning boundary;
A.4 distinguishes existing tests from proof still required.

| Id | Entrypoint / callers | Outputs and scope | Present depth behavior / owning source | Integration |
| --- | --- | --- | --- | --- |
| MA1 | `prepare-agent-maintenance`; open-PR workflow, maintainer; nightly release-watch dispatch | Request/packet; private core freezes P, then O where acquisition exists, record and coherent aggregate publication | Public builder/apply refuse declarations and retained depth; private `prepare/{depth,guard}.rs` preserves selection/entries/due work, refuses acceptance, binds/rebuilds before apply. `prepare.rs`, `prepare/report.rs`; exact workflow delivery via `scripts/maintenance_transport.py` | Branch |
| MA2 | `parity-acquire.yml`, reusable or dispatch | Acquisition inputs, reported metadata, packet re-freeze, complete support JSON/Markdown/runtime projection; aggregate effects | Committing destination refuses `staging`/`main` before checkout; requires `AUTOMATION_TOKEN`; stages acquisition before re-freeze and exact delivery before audit; rejected push is not rebased/retried. Depth preparation and neutral writers still refuse public depth scope; shell transport is not depth authority | Preparation branch, PR then IN1 |
| MA3 | `refresh-agent --write` | Packet docs and requested support/runtime/capability projections, aggregate scope | `refresh.rs::apply_refresh_plan` and neutral batch boundary refuse depth workspace; no admitted positive depth refresh path | Branch |
| MA4 | `execute-agent-maintenance --write`; executor may invoke generators | Frozen write envelope: wrapper/adapter code, version reports/metadata, publication, request/marker directories and debt inventory | `execute/{guarded,candidate,delivery_index,tree}.rs` captures independent candidate/Git state, finalizes after all execution outcomes and delivers only checked reporting changes. `depth_acceptance/relay.rs` preserves Event/P/O/entries, refuses acceptance, request/marker edits, changed frozen inputs and uncorroborated debt edits. Candidate is not an OS sandbox | Branch |
| MA5 | `close-agent-maintenance`, `prepare-agent-closeout`, exported lower closeout planner/writer | Closeout and docs; first command may update lifecycle evidence/drift; owning admission adds exact entry | `closeout/write.rs::CloseoutPlan`, `prepare_closeout.rs::write_and_verify` use `closeout_proposal::Proposal` at apply. Shared `depth_acceptance/manual.rs` requires current request path/bytes, exact generation marker on integration parent, current P/O/E, evaluated evidence and debt. Public loading/evidence still prevent production depth success | Branch |
| MA6 | `manifest-promote`; `parity-promote.yml` | Root/per-target pointers, `current.json`, validated metadata, marker retirement, own entry and aggregate support/runtime/capability outputs | `manifest_promotion::{build_plan,PromotionPlan::apply}` checks clean exact validation revision and pinned integration tip, renders one proposal, re-admits at apply. `depth_acceptance/promotion.rs` requires maintenance authority, admitted retained closeout, current evidence/debt and its own target entry. Workflow no longer carries pointer/marker policy in shell | Promotion PR, IN1 |
| ON1 | `onboard-agent --write` | Manifest scaffold/current and lifecycle state | `onboard_agent.rs` refuses depth workspace before writes; LW1 also guards lower delivery. No depth enrollment/acceptance path | Branch |
| ON2 | `runtime-follow-on`, `repair-runtime-evidence` | Runtime implementation/evidence and pre-publication lifecycle; executor can affect indirect inputs | Both public write entries refuse depth workspace; lifecycle lower writer also refuses. No onboarding depth freeze/writer is installed | Branch |
| ON3 | `prepare-publication` | Publication packet, lifecycle `publication_ready` | Entry and lower packet/lifecycle writers refuse depth workspace; no acceptance authority for maintenance scope | Branch |
| ON4 | `refresh-publication` | Aggregate support/runtime/capability outputs; lifecycle `published` | `publication_refresh.rs` and neutral/lower writers refuse depth workspace before effects. No onboarding depth publication path | Branch |
| ON5 | `close-proving-run`, `prepare-proving-run-closeout` | Proving closeout; lifecycle `closed_baseline`, evidence lists/drift | Both entries refuse depth workspace; lower lifecycle/batch writers also refuse. D7 rejects unsupported onboarding acceptance effects, including incorrectly attributed drift clearance | Branch |
| NE1 | `support-matrix` write; maintenance/publication callers | Complete support bundle across agents | `support_matrix/publication.rs::write_publication_artifacts` refuses depth workspace. Pure generation/rendering is not delivery authority; owning proposals use renderers then admit aggregate effects | Caller / IN1 |
| NE2 | `capability-matrix` write | Aggregate capability publication | `capability_matrix.rs` refuses depth workspace; added advertising is also detected at final integration, regardless of how code was edited | Caller / IN1 |
| NE3 | `manifest-version-metadata`, historical alias `codex-version-metadata` | One version's status and target outcomes | `manifest_version_metadata.rs::run_with_default_root` refuses depth manifest; internal read-only planner is used by owning promotion. Caller `--status` is not authority | Caller / IN1 |
| NE4 | Manifest validator fix modes and aliases | Pointer formatting/missing `none` pointers; `current.json` | `manifest_validate/fix_mode.rs` refuses depth manifest before writes. Check mode is read-only; `--tip` additionally invokes repository-wide IN1 admission | Caller / IN1 |
| NE5 | Retention commands and aliases, write mode | Removes snapshot/report directories, potentially retained records | `manifest_retain.rs` refuses depth manifest before pruning. It has no authority to remove a depth record | Caller / IN1 |
| NE6 | Codex/Claude/OpenCode snapshot legacy `--out-dir` | `current.json` when directory is a manifest root | All three snapshot entries use `depth_routes::refuse_destination` before legacy output; arbitrary unsupported edits still meet IN1. Versioned snapshot output is an acquisition input, not a depth acceptance output | Caller / IN1 |
| NE7 | `historical-lifecycle-backfill` | Packet/proving-closeout/lifecycle for several agents | Entry refuses depth workspace; lower writers refuse as well. Historical status grants no authority over a retained depth generation | Caller / IN1 |
| NE8 | `runtime-advertising-model` write | `crates/agent_api/src/runtime_advertising/data.rs`, indirect capability advertising | `runtime_advertising/mod.rs::run` refuses depth workspace on write; check mode is non-mutating. D7 compares actual runtime advertising on any candidate | Caller / IN1 |
| LW1 | Public `workspace_mutation::apply_mutations`; lifecycle/packet storage helpers, every caller | Proposed outputs across all scopes, lifecycle and publication packets | `workspace_mutation/delivery.rs`, `agent_lifecycle/storage.rs` refuse declared/retained depth and proposed record/enrollment destinations. Crate-private `apply_admitted_mutations` is reachable from MA1/MA4/MA5/MA6 only after owning checks; it grants no authority by itself | Caller / IN1 |
| IN1 | Integration ref update, including all branch writers | Every depth-gated output and dependency; every registered retained root | Required checks and native queue validate exact merge result/base; `manifest_validate/depth/merge.rs` calls `depth_acceptance/integration.rs` before requested-root narrowing. Shared generation/closeout/promotion/current-qualification checks; native tip-conditional MERGE ref update | `staging` itself |
| HE1 | Hand edit, external tool, unsupported output-path use | Any output or invalidating dependency | No caller authority. Candidate is judged from its values by IN1; forbidden entries, lost records, stale qualification and wrong lifecycle effects refuse | IN1 |

The request nesting environment flag is an ergonomic refusal, not a security boundary.
MA4's host guard applies independently of what the executor invokes. Other branch-local tools
and restorations prepare candidates; none may present an earlier check as final integration
admission. Public depth refusal is intentionally broader than the minimal touched tuple.

### A.2 Outputs

The backward view below reconciles the forward routes. IN1 and HE1 reach every row.
An executor can attempt out-of-envelope writes too; MA4 observes/refuses those attempts rather
than gaining authority over additional outputs. LW1 is a
lower delivery boundary for proposed batches/lifecycle files, not an extra lifecycle owner.
Statements about owning positive behavior refer to the bounded cores, subject to A.1's refusal
and evidence limits; they do not assert public production enablement.

| Output | Writers / indirect routes | Present check and remaining requirement |
| --- | --- | --- |
| Depth records and entries | MA1 (P/O), MA2 via re-freeze, MA4 (reporting E), MA5/MA6 (own entry); LW1 and NE5 can otherwise target/remove them | Private freeze/relay/owning admission preserves prior records, entries, selection and due work; unsupported lower/pruning delivery refuses. IN1 checks new/changed generations, retained invariants and exact new entry authority for all roots. Public positive chain remains unproved |
| Support publication JSON and Markdown | MA1 private bundle; MA2/MA4/MA6 via owning proposal/renderers; MA3, ON4, NE1, LW1 | Aggregate current derivation and actual-effect detection; neutral/public depth paths refuse. Existing qualification is re-evaluated even without record changes. Dependency invalidation may downgrade honestly to `unverified` |
| Capability publication / advertising | MA3, ON4, NE2, NE8; MA4/MA6; any adapter/runtime code edit under HE1 | Neutral guards; real source/projection comparison and aggregate currentness at IN1. Added advertising needs its own admitted acceptance destination; generated freshness alone is insufficient |
| Version metadata | MA2/MA4/MA6, NE3, LW1 | Neutral write refuses depth; owning promotion checks exact validated commit/targets. IN1 detects status and passed-target acceptance values, including hand edits |
| `current.json` | MA6, NE4, NE6, ON1, LW1 | Guarded promotion or explicit unsupported refusal. Expected-target changes are checked independently of an unchanged pointer/projection |
| Root and per-target pointers | MA6, NE4, LW1 | One admitted promotion proposal; fix mode refuses depth. Destination tuple receives acceptance; displaced tuple remains reporting and keeps its record/work |
| Embedded runtime-support projection | MA1 private bundle; MA2/MA4/MA6; MA3, ON4, NE1, LW1 | Delivered with complete support bundle; aggregate derivation and IN1 check every touched tuple, not just the named agent |
| Stand-down marker removal | MA6, MA4 envelope, LW1 | Guarded promotion retires its marker coherently; relay forbids marker changes. Parent marker is required for new closeout authority. Marker addition/change is not itself depth-gated, but cannot self-authorize a closeout on the candidate |
| Maintenance request | MA1, MA2 re-freeze, MA4 envelope, LW1 | Shared Event/P/O binding and same-version/lifecycle freeze; private preparation preserves displaced records. Relay refuses request changes; IN1 validates reporting-only generation changes across roots |
| Maintenance closeout | MA5; MA4 envelope/LW1 are potential unsupported writers | Both manual writers re-admit the exact proposal and add its entry in the same checked batch; relay refuses acceptance. Existing output/absence survives refusal; current production evidence cannot authorize a positive result |
| Proving-run closeout | ON5, NE7, LW1 | Unsupported depth workspace refuses at entry/lower boundary; IN1 refuses unsupported onboarding acceptance. No maintenance authority is borrowed |
| Lifecycle acceptance values | ON1–ON5, NE7, MA5, LW1; MA4 envelope can reach maintenance files | Public unsupported guards and value-derived attribution; MA5's owning batch may report its closeout in private proofs. Drift clearance requires candidate generation attribution even when the lifecycle file disappears |

Input-only writers remain relevant without becoming new depth-gated outputs:
`manifest-union`, `manifest-report`, versioned snapshots, wrapper-coverage generators, code/evidence
edits and MA4 debt edits can invalidate O/E, due work or publication. Their normal output paths do
not grant acceptance. Frozen acquisition changes require explicit re-freeze; retained qualification
must be re-evaluated at IN1. Debt transition planning is read-only (`support_depth/debt.rs`);
MA4 currently refuses debt inventory changes for retained depth rather than granting a renewal
or retirement from executor assertions. Direct maintainer edits still meet IN1 and the current
closeout/debt predicates. Reports and coverage files do not become authoritative depth evidence
merely by being collected.

### A.3 Serialization domains and restoration

| Domain | Writers and restorations | Present ownership / limit |
| --- | --- | --- |
| Integration branch | IN1, through it all routes, aggregate regeneration and hand edits | Native queue validates exact candidate/base and conditionally updates the ref. Effective staging policy requires the queue and checks; D7 is the semantic backstop. Post-push `depth-tip.yml` detects the actual before/after transition, but cannot retroactively prevent a push |
| Packet candidate | MA1/MA2 automation, maintainer MA3/MA5, MA4 executor/delivery | Workflow groups and fresh stand-down checks reduce collisions; they are not a cross-route lock. MA1 rebuilds bound proposal; MA4 runs against an independent candidate with Git/index/output observations; MA5 re-admits observed inputs. Expected-before delivery/rollback protects observed edits, not an atomic filesystem transaction |
| Promotion candidate | MA6 and aggregate renderers | Exact HEAD/validation/integration revision, one proposal re-admitted at apply; source changes, marker/evidence/debt changes and moved integration tip refuse. Final authority still belongs to IN1 |
| Aggregate publication | MA1/MA2/MA3/MA4/MA6, ON4, NE1/NE2/NE8, code editors | Candidate may touch other agents; all-root admission and aggregate derivation run again on the merge result. Branch-local textual conflict resolution is not evidence of admission |

`workspace_mutation` classifies expected-before files and restores only its observed owned writes;
its rollback preserves intervening bytes/type/mode changes or reports a conflict. D4 rejects a
failed executor candidate instead of copying unauthorized product changes back into the caller;
diagnostics/recovery candidate remain attributable. D5 no longer writes a new closeout then
restores an old artifact after validation: both manual paths use a prospective checked batch.

Unsupported onboarding publication/lifecycle restoration paths remain in
`publication_refresh.rs`, `prepare_publication.rs` and `repair_runtime_evidence.rs`.
They are reached only after the depth workspace refusal in supported use; no depth-safe
serialization claim is made for those legacy restoration implementations. Removing their refusal
would require fresh ownership/restoration proof. There is no cross-process lock or crash-atomic
multi-file transaction. If a route performs a final gated sequence outside IN1, the existing
[Final depth admission](#final-depth-admission) serialization requirement still applies; E1 changes
none of those rules.

### A.4 Reconciliation and conformance status

The forward/backward scan included public dispatch in `xtask/main.rs`, write-side entrypoints,
all callers of crate-private admitted delivery, public lower storage, `scripts/`, workflow shell,
restorations and generated runtime advertising. NE8 and LW1 were previously implicit; promotion
shell writes, unsafe legacy retention and the unprotected-staging description are replaced above
by their current owning-command/refusal/host behavior.

Existing source and conformance anchors (test names are searchable in the named files):

| Routes / obligation | Existing evidence | What is still due |
| --- | --- | --- |
| MA1 P/O and transport, MA2 intake/delivery | `tests/agent_maintenance_prepare/depth_core.rs`: `private_preparation_plans_p_only_and_coherent_o_for_every_adopted_root`, stale-plan/refreeze/displacement cases; `tests/c4_spec_ci_wiring/{workflow_transport,acquisition_destination,acquisition_token,acquisition_refreeze}.rs`; `scripts/test_maintenance_transport.py` | Connected acquisition→freeze proof through actual supported isolated entrypoints, not shell stubs or a global parser bypass |
| MA4 reporting/failure/restoration | `tests/agent_maintenance_execute/relay.rs`; `tests/c0_spec_validate/depth_records/acceptance/{relay,relay_boundary}.rs`; `depth_acceptance/relay.rs` | Connect executor-start capture to real evidence, audit and both final acceptance steps in the same isolated chain |
| MA5 owning closeout | `depth_acceptance/manual/tests.rs` and `manual/tests/regressions.rs`: exact parent generation, missing due obligations, stale bindings, concurrent input/output and public lower-boundary refusals; ordinary `tests/agent_maintenance_closeout/` | Authenticated all-due positive closeout, both public manual entrypoints, preserving an older closeout on negative outcomes |
| MA6 / LW1 | `depth_acceptance/manual/tests/promotions.rs`; `tests/depth_promotion.rs`: coherent ordinary promotion, stale plan, nested anchors, proposed depth record and lower packet/batch refusal | Genuine evidence-backed positive promotion after closeout/replaced working files and native queue delivery |
| ON1–ON5, NE1–NE8, LW1 | Source guards in A.1; lower-route/nested/proposed-output cases in `tests/depth_promotion.rs`; ordinary entrypoint suites | A retained-depth refusal case at every actual unsupported public entrypoint/lower boundary, with no changed protected output; source inspection alone is not that execution proof |
| Debt | Conditional renewal/whole-row-retirement planners in `tests/support_depth_debt.rs`, `tests/support_depth/debt.rs`, retaining corroboration requirements; audit reconciliation consumers | Connected refusal of unsupported debt edits, including widened/new debt. The positive chain remains debt-free; a successful new debt writer requires separate design. Debt never waives depth |
| Evidence | `tests/depth_evidence*.rs`, `tests/depth_ci_archive.rs`; authenticated isolated V1 profile in `depth_evidence/verdict/github/` | The installed isolated profile is fixed to canonical repository, `codex/0.0.0`, Linux `controlled_channel` V1. It cannot be pointed at a new repository or production generation by editing receipt fields. `depth_evidence/results.rs::production_results` returns empty verdicts. Real profile integration/all-due coverage remains work |
| IN1 / HE1 | `depth_acceptance/manual/tests/integration.rs`: forged entries/request hash, other-root generation, unchanged qualification, moving parent, unsupported lifecycle and local/final agreement; `tests/c0_spec_validate/depth_records/merge*`; `tests/c4_spec_ci_wiring/{integration_gate,depth_merge}.rs`; `scripts/test_staging_{integration,ruleset}.py`; successful PR #296 queue/push checks | End-to-end depth-positive host run and forbidden-depth candidate refusal on an isolated protected integration branch. The ordinary production merge and earlier host refusal exercises do not prove that positive chain |
| Rollback | `workspace_mutation/tests.rs`: intervening edit/symlink/mode/removal; relay delivery conflict tests | Connected failure checks must preserve concurrent caller state and fail without a final acceptance effect |

Only `codex`, `claude_code` and `opencode` currently have Annex-B-adopting validator specs and
the fixed per-root CI loop. `aider` and `gemini_cli` do not. D7's all-root admission is not a
substitute for adding a root's canonical validator adoption and complete manifest validation
before that root may hold a depth record. The isolated refusal matrix must include this boundary.

No executable command for depth freeze/debt transition is implied by its pure planner API.
`depth-ci-collect` writes only a non-authoritative `target/depth-ci` witness;
`depth-ci-prove` is a read-only isolated preview. Diagnostic comparison/check modes do not grant
write authority. Maintenance audit-status/watch/drift and their summary projections, acquisition
plans, scaffold/research metadata and release docs are not depth-gated outputs in supported use.
Caller-controlled paths do not create an exception to HE1/IN1. The `main` release/publish boundary
remains outside this staging inventory and is not certified by this refresh.

This is a source inventory plus existing evidence mapping, not a path-enablement certificate.
The production refusal, empty production evidence bridge, missing connected positive/public-route
proof and unsupported-route negative coverage above must be resolved or retained as explicit
refusals before the relevant scope is enabled. No future profile may turn isolated `0.0.0`
receipts, a test producer, an executor statement or a skipped obligation into production adequacy.

### A.5 Promotion after the working files were replaced

A later version's generation can open between a version's closeout (MA5) and its promotion
(MA6). Opening it replaces the maintenance request, which leaves the earlier closeout bound to a
request that no longer exists, and the later version's own closeout then replaces that file.
Promotion of the earlier version has its depth record and nothing else from its working files.

The record stands in for the replaced files and for nothing else. P and O are present for
predicate item 5 when the record states their content identities. MA6 still establishes every
other part of the predicate against the exact merge result, as for any acceptance effect,
including:

- the record lists the closeout's acceptance entry and still states what that entry was made for;
- every result the promotion rests on is `not_applicable` where P permits it, or is `verified`
  with its bound evidence present in the merge result under the content identity the record
  names, and no result has been invalidated under
  [Reuse and invalidation](#reuse-and-invalidation), as predicate items 2 and 5 require;
- the debt grants E materializes still hold, and the path is still enabled for the tuple's
  agent; and
- predicate items 4 and 6 hold.

MA6 lists its own acceptance entry in the same change. When any of these fails, MA6 refuses and
the version is closed out again in a new generation. A later version's packet that changed
wrapper code, tests or fixtures an earlier version's results rest on is one ordinary case of
that refusal. A later packet that updated a debt row the earlier version's tuples depend on is
another.

Promoting an earlier version after a later one moves the pointers back. That is an acceptance
effect for the earlier version and a reporting effect for the later one, and MA6 as it stands
overwrites the pointers unconditionally. Whether a promotion may move a pointer to an earlier
version is decided by the maintenance lifecycle rules, which this contract leaves unchanged.

## Annex B: the depth record

This annex defines the depth record that [Terms](#terms) introduces: where it is kept, what it
states, the identities it carries and the form of an acceptance entry. The record invariants in
[Depth-gated effects](#depth-gated-effects) say what must hold of a record, and the checks that
enforce them are not part of this annex. This annex is the one definition for every manifest
root. A root's validator spec adopts it by a section that says so.

B.6 and B.7 are the first executable revision of this annex, revision 1: the key names and value
encodings of the record, the form in which an operation, its promise and an invocation mode are
stated, and the serialization the digests in B.3 and B.5 are taken over. It is among the
revisions that [Path enablement](#path-enablement) item 1 requires.

The validator specs of the `codex`, `claude_code` and `opencode` roots adopt this annex and state
the executable depth-record checks; `--tip` additionally establishes repository-wide admission. A manifest root MUST NOT hold a depth record until its validator spec
states the checks that [Complete mediation](#complete-mediation) rule 7 requires.

### B.1 File and revision

1. A version's depth record is the file `reports/<version>/depth-record.json` under the agent's
   manifest root. `<version>` is the exact upstream version, written as the root's
   `versions/<version>.json` writes it. A version has one record.
2. The record is a JSON document. Its `schema_version` is an integer that states the executable
   revision of this annex that the record follows, and the first executable revision is 1. A
   consumer MUST require the current executable revision and reject any other, as
   [Bindings](#bindings) rule 8 requires. It MUST NOT read a record of another revision, or a
   record it cannot parse, as "not depth-enrolled".
3. In this annex the content identity of a committed file is the SHA-256 digest of the file's
   bytes, written as 64 lowercase hexadecimal characters.

### B.2 What a record states

A record states an Event and three parts, and it lists acceptance entries. Each part states one
binding of the version's latest generation.

- **Event.** The fields that [Bindings](#bindings) lists for the lifecycle path that owns the
  depth enrollment.
- **Policy part**, for P:
  - the resolved selection: the agent, the lifecycle path, the exact upstream version, the
    operations with their promises, the modes and required values of each, and the targets;
  - for each promise, where it permits a `not_applicable` result;
  - what the selection covers, as B.4 defines it;
  - the capability mappings: for each capability the selection claims, the complete set of
    operations through which the agent's adapter honors it;
  - the classification entries, mode exclusions and overrides that apply to those operations;
    and
  - the frozen delegation and the initial authorization baseline that
    [Debt operations](#debt-operations) defines: every debt row in force for the agent when P
    was frozen, with the content the row had then.
- **Obligations part**, for O:
  - the policy identity that O was frozen under;
  - the targets that O is frozen for;
  - the acquired inputs that O was derived from;
  - for each operation, the name identities it owns and its concrete obligation set;
  - every unit of a selected operation that is `classification_required`; and
  - the independently required acceptance work.
- **Execution part**, for E:
  - the policy identity and the obligations identity that E references;
  - for each obligation and each target, its result, with the rationale of a `not_applicable`
    result and, for a `verified` result, the committed evidence it rests on;
  - every reuse binding; and
  - the debt grants that E materializes for the version, each as the granted row with its
    content.
- **Acceptance entries.** Listed, not stated: one entry for each closeout, publication or
  promotion that made an acceptance effect for the version, as B.5 defines it.

Rules:

1. The items listed for a part are stated in full. Whatever else [Bindings](#bindings) gives the
   binding as content, the part states as well, in full or by content identity as B.6 decides. A part therefore states its whole binding.
2. A committed file is stated by its path and its content identity. It is not copied into the
   record.
3. Everything a part states belongs to that part's binding for record invariant 2. The Event
   belongs to no part, and neither do the acceptance entries.
4. A part that the latest generation has not yet produced is stated as not yet produced. An
   absent part is an error and MUST NOT be read as not yet produced. The policy part is never
   stated as not yet produced, because the record is first written when P is frozen.
5. A record's selection MUST name the agent whose manifest root holds the record and the version
   whose directory holds it.

### B.3 Identities

1. A record states one identity for each part that has been produced: the policy identity in
   its policy part, the obligations identity in its obligations part and the execution identity
   in its execution part. A part stated as not yet produced has none. They are the content
   identities of P, O and E that this contract and the
   [maintenance request contract](maintenance-request-contract-v1.md#depth-enrolled-generations)
   refer to.
2. Each identity is the SHA-256 digest of its own part with the identity itself left out,
   written as 64 lowercase hexadecimal characters. The digest is taken over the serialization
   that B.7 defines, which is deterministic, so that one part has one digest whoever computes
   it.
3. No part states anything of a later binding: the policy part states nothing of O or E, and the
   obligations part states nothing of E. With rule 2 this keeps every identity independent of
   its own digest and of a later output, as [Bindings](#bindings) rule 6 requires.

### B.4 Covered modes

1. The policy part states what the selection covers as a list of covered invocation modes, each
   with the targets it is covered on. The capabilities the selection claims are the ones its
   capability mappings state, and they need no second list.
2. A covered mode is named apart from the operations that serve it. Its name is the depth
   enrollment selector under which the selection first covered it, in the form rule 6 gives, and the mode keeps that name from the time a generation that covers it is
   committed. Each operation states which covered modes it serves.
3. A P that renames, divides, merges or reclassifies operations changes which operations serve a
   covered mode. It neither renames nor drops the mode. A mode that P comes to exclude under
   classification rule 7 stays in the list and names the classification entry that excludes it.
   The registry contract requires a declaration to keep one operation that is not a mode
   exclusion, so P cannot come to exclude the mode that a selection's last such operation
   serves. That operation keeps its classification, and its results stay `unverified` until
   they are verified.
4. A listed mode counts as covered on a target only where an operation of the resolved selection
   serves it on that target, or where the mode names the classification entry that excludes it.
   A mode or a target listed with neither is lacking for rule 5.
5. Record invariant 1 is decided on this list and on the capability mappings. A selection has
   shrunk when the record that would replace the one the integration branch holds lacks a
   covered mode, a target that the mode is covered on, or the mapping of a capability that the
   record it replaces claims.

6. In revision 1 the list of covered modes is the `modes` list of the selection, and no second
   list exists. A mode's `selector` is the covered mode's name, the targets it is selected on are
   the targets it is covered on, an operation's `serves` states the covered modes it serves, and
   a mode that an `EXCLUDED` operation serves names that operation as the classification entry
   that excludes it. The registry contract already requires every mode of a declaration to be
   served, so every listed mode counts as covered under rule 4.
7. A record lacks a covered mode of the record it replaces unless it lists a mode with the same
   `selector`, `command_path`, `mode` and set of `predicates`. A kept name that selects anything else is
   another mode. A selection covers more by adding a selector, never by changing one.

### B.5 Acceptance entries

1. An acceptance entry states three things:
   - what it backs: a maintenance closeout, a proving-run closeout, a publication or a
     promotion;
   - what it names: for a closeout, the closeout record, and for a publication, the publication
     packet, each by path and content identity; for a promotion, the targets whose pointers the
     promotion set; and
   - what the record stated when the entry was made: one digest over the Event and the three
     parts as they then stood, their identities included, in the serialization that B.3 uses.
     The list of acceptance entries is left out of it.
2. An entry identifies what the record states now when that digest equals the digest of the
   record's present Event and parts. Record invariant 4 speaks of an acceptance entry made for
   what the record states in this sense.

[Minimum machinery](#minimum-machinery) requires the failure a new mechanism prevents to be
named. Two choices in this annex add structure:

- The list of covered modes exists because an operation can be renamed or divided. Without a
  name for a mode that outlives its operation, a P that drops a mode and a P that only renames
  the operation serving it change the record in the same way, and record invariant 1 could not
  be decided.
- The baseline is stated in the policy part because the debt rows it is taken from stay inside a
  run's `writable_surfaces`, and every re-freeze takes the request's audit rows from the tree
  again. Nothing else would state what a debt transition is compared with.

### B.6 Encoding

This section and B.7 are revision 1. Every object below takes exactly the members listed. A
consumer MUST reject a member that is not listed and a listed member that is missing.

1. One content has one written form:
   - A member marked optional is left out when it states nothing and is never `null`. Every
     other member is written, an empty list as `[]`. The only `null` values are the
     `obligations` and `execution` members of a part not yet produced. No object states a
     member twice.
   - `enrollment` is written as the declaration writes it: its lists in the declaration's
     order, and a mode's `targets` present only where the declaration states it. That order is
     part of what P states, so a declaration that is reordered is a changed P. Whether a
     record's `enrollment` agrees with the declaration is not decided by the record alone and
     belongs to depth admission.
   - `acceptance_entries` is in the order the entries were made.
   - Every other list is a set. It is written without repeats, in ascending byte order of its
     entries' B.7 serialization.
2. Apart from `schema_version` a record holds no number. Every digest is 64 lowercase
   hexadecimal characters.
3. A **file reference** is `{ "path", "sha256" }`: a repository-relative path with `/`
   separators, no backslash and no empty, `.` or `..` segment, and the content identity of the
   file.
4. A **surface** is a name identity as the
   [maintenance request contract](maintenance-request-contract-v1.md) defines it:
   `{ "surface_kind", "command_path", "surface_id" }`.
5. A **debt row** is one row of the debt inventory, with a member for the row id and one for
   each key of the row: `row_id`, `agent_id`, `surface_kind`, `command_path`, `surface_id`,
   `current_reason`, `blocker_class`, `owner`, `milestone`, `follow_on`, `evidence_ref`,
   `scope_target_triples`, `authorized_at_version` and `authorization_evidence_ref`. Each is a
   string, except `scope_target_triples`, which is the list of targets. A target holds no
   whitespace, `"` or `\`. `command_path` is rooted at the row's `agent_id`, as the maintenance
   request contract requires of a debt row.
6. An **obligation reference** is `{ "operation", "id", "target" }`, where `id` is an obligation
   id. For the `M` obligation of a capability it is `{ "operation", "capability", "target" }`
   and has no `id`. It states exactly one of the two.

The record:

| Member | Value |
| --- | --- |
| `schema_version` | `1`. |
| `event` | The Event. `lifecycle_path` is `"maintenance"` or `"onboarding"` and MUST be the selection's. With it, the maintenance Event states `request_commit`, `request_recorded_at`, `trigger_kind` and `opened_from`, and the onboarding Event `approval_commit` and `approval_recorded_at`, each a string that is not blank, written as its request or approval artifact writes it. |
| `policy` | The policy part. |
| `obligations` | The obligations part, or `null` while not yet produced. |
| `execution` | The execution part, or `null` while not yet produced. |
| `acceptance_entries` | The list of acceptance entries, in the order they were made. |

The policy part:

| Member | Value |
| --- | --- |
| `identity` | The policy identity. |
| `selection` | `{ "agent_id", "enrollment" }`. `enrollment` is the declaration in the form the [registry contract](agent-registry-contract.md#support-depth) gives one, with the same member names and the same rules, except where rule 1 of this section fixes the written form: `lifecycle_path`, `version`, `targets`, `modes`, `operations` and `capabilities`. Under that rule `predicates`, `not_applicable_permitted` and `capabilities` are always written, as `[]` where the declaration states none. It states the operations with their promises, the modes and required values of each, the targets, where a promise permits `not_applicable`, the capability mappings, the classification entries and the mode exclusions. Revision 1 has no override. |
| `rules_revision` | The revision of the governing rules when P was frozen, a string that is not blank. |
| `base` | The BASE obligation ids as P resolved them. |
| `templates` | One `{ "template", "obligations" }` for each obligation template an operation of the selection selects, stating the obligations the template added when P was frozen. |
| `path_enabled` | Whether the selection's lifecycle path was enabled for the agent when P was frozen. |
| `debt_baseline` | The frozen delegation and the initial authorization baseline: a debt row for every row in force for the agent when P was frozen. Each row's `agent_id` is the selection's, and row ids do not repeat. |

`base` and `templates` state which obligations the governing rules select. Without them a change
to an obligation template would leave a frozen P with the same identity and another meaning.

`rules_revision` stands for the governing rules that those two members do not state: what an
obligation id requires, the classification rules and a capability's minimum semantics. The
repository holds one current value, beside the template table that policy resolution reads, and
the first is `"1"`. A change to one of those rules MUST change that value in the same commit.
Policy resolution then gives another policy identity, a P frozen before the change is no longer
current, and evidence produced under it serves the new P only through a reuse binding. Without
this member such a change would leave every frozen P with the same identity, and
[Bindings](#bindings) rules 3 and 5 could not be decided from a record.

The obligations part:

| Member | Value |
| --- | --- |
| `identity` | The obligations identity. |
| `policy_identity` | The policy identity that O was frozen under. |
| `targets` | The targets that O is frozen for, without repeats. |
| `acquired_inputs` | A file reference for each acquired input that O was derived from. A path does not repeat. |
| `operations` | One `{ "id", "surfaces", "obligations" }` for each operation: the surfaces it owns and its obligation ids. An `M` obligation is not listed. An operation has one for each capability mapping that names it. Operation ids do not repeat. |
| `classification_required` | One `{ "operation", "surface" }` for each surface of an operation that carries a unit that is `classification_required`. |
| `acceptance_work` | The independently required acceptance work: one `{ "surface", "targets" }` for each name-level item the generation's request holds as required, with the targets it is required on. A surface has one entry, which states every target. On the onboarding path the approval artifact holds no such item and the list is `[]`. |

The execution part:

| Member | Value |
| --- | --- |
| `identity` | The execution identity. |
| `policy_identity`, `obligations_identity` | The P and O identities that E references. |
| `results` | One `{ "obligation", "result", "rationale", "evidence" }` for each result. `obligation` is an obligation reference and does not repeat. `result` is `"verified"`, `"unverified"`, `"failed"` or `"not_applicable"`. `rationale` is optional: a `not_applicable` result MUST state one that is not blank and no other result may. `evidence` is a list of file references in which a path does not repeat, and only a `verified` result may bind any. The implementation identities that [Bindings](#bindings) gives E are stated by that evidence. |
| `reuse_bindings` | One `{ "obligation", "policy_identity", "obligations_identity" }` for each reused obligation: the P and O that its evidence was produced under. `obligation` does not repeat. |
| `debt_grants` | A debt row for each grant that E materializes for the version. Each row's `agent_id` is the selection's, its `authorized_at_version` is the selection's version, and row ids do not repeat. |
| `debt_retired` | The row ids of the baseline rows that E retires. A transition from P's baseline is a grant or a retirement stated here, and no row is both. |

An acceptance entry states `backs`, `record_sha256` and one further member:

| `backs` | Further member |
| --- | --- |
| `"maintenance_closeout"`, `"proving_run_closeout"` | `closeout`, a file reference to the closeout record. |
| `"publication"` | `packet`, a file reference to the publication packet. |
| `"promotion"` | `targets`, the targets whose pointers the promotion set: at least one, without repeats. |

`record_sha256` is the digest that B.5 rule 1 defines.

What this section requires is what a record decides by itself, and a consumer MUST reject a
record that breaks it. Whether the parts agree with one another, with the registry and with a
request is [Depth admission](#depth-admission): a `policy_identity` that E references and the
record does not state, for example, is not a malformed record. Record invariant 3 says how its
results are published.

### B.7 Serialization and digests

1. A digest is the SHA-256 digest of a JSON value serialized as RFC 8785 requires: no
   insignificant whitespace, the members of an object sorted by name, and each string escaped as
   that RFC states. B.6 rule 2 leaves no number to serialize, and every member name is ASCII.
2. An identity is taken over its part's object with the `identity` member left out.
3. The digest of an acceptance entry is taken over the object
   `{ "event", "policy", "obligations", "execution" }` whose members are those of the record as
   they then stood, a part not yet produced as `null`.
4. Rule 1 of B.6 makes the serialization deterministic: a content has one written form, so one
   part has one digest. A consumer MUST reject a record that is not in that form.

Test vector for rule 2. The part `{"z":"a","a":["é\n",true,null],"identity":"left out"}` is
serialized as the 32 bytes of `{"a":["é\n",true,null],"z":"a"}`, in which `é` is the two bytes
`c3 a9` and `\n` is the two-character escape. Its identity is
`1019655b4e277dabfe0cb08487ef861e99531f65f058a980e76a1927afe4c441`.

### B.8 Present behavior

Source references are to `staging` at `f61534be` and use the abbreviations of Annex A. Today:

- `xtask/depth_record.rs` reads a record and decides what B.6 and B.7 require, the digest of
  B.5, the comparison of B.4, and whether a record states what a maintenance request froze. It is
  not at `f61534be`. `manifest-validate` and the support matrix generator call it.
- No manifest root holds a depth record, and no command writes one. Revision 1 gained
  the declaration's `forwarding` member before any record existed, so no record of an earlier
  form is held anywhere.
- The commands that parse a version's coverage reports open them by exact name or scan
  `reports/<version>/` for names that begin with `coverage.` and end with `.json`.
  `manifest-validate` and the support matrix do the first
  (`xtask/manifest_validate/versions.rs:279-316`, `xtask/support_matrix/derive.rs:575-600`). The
  support-surface audit and `maintenance-audit-status` do the second
  (`xtask/agent_maintenance/support_audit.rs:598-620`,
  `xtask/agent_maintenance/audit_status/evidence.rs:174-204`). None of them would take
  `depth-record.json` for a coverage report.
- `prepare-agent-closeout` lists every regular file directly under `reports/<version>/` as a
  version-scoped manifest artifact of the packet. It refuses when it finds no such artifact: no
  file there, none under `snapshots/<version>/` and no `versions/<version>.json`
  (`xtask/agent_maintenance/closeout/findings.rs:158-197,243-255`). A depth record written at the
  first freeze would satisfy that guard by itself.
- `manifest-retain` removes `reports/<version>/` for a version outside its keep set, and a depth
  record with it, as Annex A records.
- `manifest-validate` reads one manifest root (`xtask/manifest_validate.rs:140-149`). Inside a
  workspace it also checks the support-matrix artifact, and for that it loads the registry and
  the manifest roots the support matrix publishes (`xtask/manifest_validate.rs:576-690`,
  `xtask/support_matrix/consistency.rs:163-170`, `xtask/support_matrix/derive.rs:258-270`). Since
  `f61534be` it also reads the root's depth records, the agent's standing maintenance request and
  its lifecycle record, and decides checks 1, 3, 10 and 13 of the validation test list, which the
  three validator specs state. Through the support-matrix check it decides checks 5, 6, 7, 8 and
  12: the support matrix reads the depth records of every root it publishes, and each file a
  record binds for its content identity, and the check compares each published row's depth facts
  with the derived ones. With `--tip`, it also reads the integration-tip records and their registry
  from Git objects, and a changed onboarding record's versioned approval, to decide checks 2, 4
  and 11. PR validation verifies the test merge's parents against event base/head; merge-group
  validation compares the checked whole group with its supplied base. Light push-to-staging
  validation compares event SHA with event.before. With `--tip`, check 9 also compares every
  acceptance-effect category through `xtask/depth_acceptance/`, using each revision's own immutable
  inputs and the checked runtime advertising model. It requires a candidate record-wide current
  entry for every covered acceptance destination. The read-only comparison remains diagnostic;
  D7 below adds full owning admission, including entry listing, to the validation path.
  The remaining inventory, isolated chain proof and enablement work remain prerequisites (uaa-0073).
- In the three roots that have them, `RULES.json` names the record's path as
  `storage.committed.reports.depth_record`, which is how a root states that it adopts this annex.
  `SCHEMA.json` defines no shape for a depth record: the typed reader rejects any member B.6 does
  not define.
- The repository's existing content identities are SHA-256 digests of a file's bytes, such as
  `request_sha256`, of a rendered text, such as `prompt_sha256`, or of a typed value's
  serialization, such as a proving-run closeout's `release_watch_sha256`.

### D6 bounded promotion implementation

`xtask manifest-promote --agent <agent> --version <version> --validation-commit <sha>
--passed-target <target>...` plans a maintenance promotion without writing the caller workspace;
`--write` applies its opaque proposal. The owning command MUST bind validation to the same clean
HEAD and locally fetched `origin/staging` commit at planning and apply. Workflow validation jobs
and the promotion checkout MUST use the plan's pinned commit. A moved integration tip requires
new validation; acquired union membership alone MUST NOT supply passed targets.

The proposal includes version metadata, root/per-target pointers, `current.json`, support JSON,
Markdown and runtime projection, capability publication, any matching promotion entries and the
promoted version's marker retirement. It renders into a disposable observed-input view and runs
canonical manifest validation before caller delivery. Unvalidated targets' pointers are preserved;
if their existing pointers contradict the proposed metadata, the candidate MUST refuse. It keeps
the lifecycle's existing version-order policy, including proposing an earlier retained version.

For every actual depth acceptance destination, including another agent affected by aggregate
publication, the private owning admission core MUST require maintenance lifecycle authority, a
closeout entry for the current record statement, independently current P/O/E and bound evidence,
and current debt. It reuses manual closeout's due-obligation and unresolved-work refusal. It MUST
preserve historical entries and append the promotion's own current-digest entry in the same batch.
A replaced working request/closeout does not replace the durable entry's authority (Annex A.5).
A marker present on the integration tip MUST match that generation and be retired in the same
proposal. Timestamp-bound markers remain usable after working-file replacement; a hash-only marker
requires exact closeout bytes that establish its request digest, otherwise promotion refuses.

Unsupported onboarding, neutral generators, fix/retention/legacy/backfill routes and exported lower
writers conservatively refuse writes when a declaration or retained depth record is present in
scope, including proposed declarations/records and authority above a caller-supplied nested root.
Every scratch output MUST be jailed before any write, even in read-only planning. Their read-only
modes remain available. The public generic mutation writer supplies no
lifecycle authority; checked owning routes use a crate-private delivery core after their own
admission. This prevents a cached raw mutation list or a caller-supplied publication/refresh bundle
from bypassing an owning route. Existing expected-before rollback handles marker deletion alongside
writes, preserving an observed concurrent replacement and keeping recovery backups on conflict.
Observation and repair are best effort, not a filesystem lock or crash-atomic multi-file commit.

These are bounded command and workflow guards. Normal production registry loading still refuses
depth enrollment, and the production evidence boundary still cannot qualify a complete promise.
Private fixed-producer tests prove admission composition, not authenticated production adequacy.
D7's exact-result IN1 admission, route inventory re-establishment and the full isolated maintenance
chain remain required before separately authorized production enrollment. D6 does not install host
settings, merge its own PR or create a background automation.


### D7 final integration admission

With `--tip`, `manifest-validate` MUST re-establish depth admission through the shared owning
predicates for every registered agent, independently of the requested manifest root. It compares
the supplied immutable parent with the candidate, preserves retained entries and selections, and
checks current request correspondence and new/changed-statement freeze authority even for
reporting-only changes in another registered root. Historical unchanged statements need no
surviving working request or approval. It checks every acceptance destination and every newly
listed entry. A new entry MUST describe the
current record and match the independently admitted owning entry; copying an entry or selecting a
workflow/CLI name supplies no authority. Unsupported onboarding acceptance remains refused.

Maintenance closeout uses the same predicate as both local closeout writers: the actual candidate
closeout, exact current request path/content binding, current P/O/E and independently evaluated
obligations/debt, maintenance lifecycle authority and a generation-matching marker already present
on the supplied integration parent. A candidate-only marker cannot authorize its own closeout.
Promotion reuses the local promotion predicate and requires an admitted retained closeout from the
parent or a newly admitted candidate closeout, plus its own current target entry and coherent
marker retirement. Replaced working files do not erase already established retained authority.

An unchanged qualified promise or depth-qualified capability MUST also re-establish current
bindings and evidence. Dependency-only invalidation cannot leave an existing claim qualified merely
because the record and its entries did not change. Reporting invalidated results as `unverified`
remains permitted without a new acceptance entry, subject to the existing record/publication
invariants. Aggregate publication must agree with the current derived facts.

The existing CI adapter MUST refuse an integration checkout with uncommitted tracked changes or
untracked candidate files. PR event parents and the merge-group candidate/base remain bound to
exact commits; the existing terminal gate evaluates those same results, and the native merge queue
supplies the tip-conditional ref update. Resolved HEAD/parent changes during admission refuse. A
local read-only validation or cached verdict is not a reusable integration receipt. These changes
add no CI job, host setting, global lock or alternate integration framework.

Normal production registry loading continues to refuse depth enrollment. Private fixed-producer
positives establish composition and agreement of the local and integration predicates, not an
authenticated production promise or complete maintenance-chain conformance. Phase E must still
re-establish the route inventory and prove that chain in isolation before separately authorized
production enrollment. D7 does not activate production or merge its own implementation PR.
