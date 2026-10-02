# Support-Depth Contract

Status: Draft, awaiting maintainer approval
Date (UTC): 2026-10-02
Scope: semantic support obligations, evidence, depth enrollment and admission rules shared by the maintenance and onboarding lifecycles

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
- what evidence verifies an obligation, and when evidence may be reused
- how a bounded scope is selected for these rules, and what stays outside it
- the admission rules every write to an acceptance output must satisfy
- the prerequisites for enabling these rules on a lifecycle path

The research and rationale behind these rules are in the
[proposal](../agents/lifecycle/support-depth-policy-proposal.md) and its
[research companion](../agents/lifecycle/support-depth-policy-research-decisions.md). Those
documents are not normative.

## What this contract does not change

| Existing rule | Owner | Effect of this contract |
| --- | --- | --- |
| Wrapper coverage levels `explicit`, `passthrough`, `unsupported`, `intentionally_unsupported` and `unknown` | [Coverage generator contract](codex-wrapper-coverage-generator-contract.md) | None. A coverage level never states depth. |
| Name identity `(surface_kind, command_path, surface_id)`, the support-surface audit, required uplifts and allowed deferrals | [Maintenance request contract](maintenance-request-contract-v1.md) | None. Depth obligations are additional. |
| Debt rows and their target- and version-scoped authorization | [Debt inventory](unified-agent-api/non-tui-support-debt.md) | None. Debt never verifies an obligation. |
| Release-watch enrollment | [Registry contract](agent-registry-contract.md) | None. Depth enrollment is a separate selection. |
| Capability ids, their minimum semantics and the promotion rule | [Capabilities spec](unified-agent-api/capabilities-schema-spec.md), [onboarding charter](cli-agent-onboarding-charter.md) | None. See [Shared mapping and capabilities](#shared-mapping-and-capabilities). |
| Lifecycle stages, support tiers and runtime profiles | [Onboarding charter](cli-agent-onboarding-charter.md) | None. None of them is a depth result. |
| The version-only runtime-support payload | [Runtime-support contract](unified-agent-api/runtime-support-contract.md) | None. Depth facts never enter it. |
| Manual maintenance closeout, manual proving-run closeout and separate promotion | Maintenance request contract, onboarding charter | None. Depth admission is added to them. |

Each surface above keeps exactly one owner document. Other contracts reference this one for depth
rules and MUST NOT restate them.

## Terms

- **Operation.** One upstream CLI invocation mode together with the interface the wrapper promises
  for it. An operation is identified by its command path, its mode and the flags, values and
  defaults that select that mode. One operation MAY own several name identities, and one name
  identity MAY belong to several operations.
- **Interface promise.** What the wrapper, and the `agent_api` adapter where the operation is
  mapped, commit to for an operation across the dimensions in
  [Dimensions and results](#dimensions-and-results).
- **Obligation.** One requirement of an interface promise, carrying a stable id.
- **Obligation template.** A reusable set of obligations selected by an operation's effects. A
  template is a requirement, never an achieved status. It is not a capability bucket, a lifecycle
  support tier, a runtime profile or a runtime family; `runtime_family` keeps the meaning the
  runtime-support contract gives it.
- **Generation.** One maintenance run or one onboarding create-lane run for one exact upstream
  version.
- **Depth enrollment.** A maintainer-approved positive selection of scope for this contract. It is
  always written with the qualifier. Unqualified "enrolled" in other contracts keeps its existing
  meanings: release-watch enrollment in the registry, and the `enrolled` lifecycle stage.
- **Depth scope tuple.** One `(agent, lifecycle path, exact upstream version, operation and
  promise, modes and required values, target)` inside a depth enrollment.
- **Protected effect.** A change to an authoritative acceptance output that alters the result for
  at least one depth scope tuple. See [Protected effects](#protected-effects).
- **Route.** A supported command, workflow, script or lower-level interface that can produce a
  protected effect.

## Minimum machinery

Requirements in this contract MUST be met through existing lifecycle scope, approvals, packet
inputs, validation, tests, reports and closeout wherever those suffice.

A new field, artifact, registry, schema, approval step or persistent mechanism MAY be introduced
only when it prevents a concrete, reachable authority, scope, stale-input or verification failure
that existing machinery cannot prevent. The change that introduces it MUST name that failure and
MUST choose the smallest sufficient addition. This rule is applied inside ordinary design review;
it creates no justification document, checklist or approval gate.

Implementations MUST NOT add a legacy inventory, grandfathering rule, semantic carry-forward
ledger, migration engine, historical-format translator, second enrollment inventory or mutable
status ledger for this contract.

## Dimensions and results

An interface promise MUST state every dimension below. `R1` through `X4` are the BASE obligations
and apply to every executable operation.

| Id | Dimension | The promise states | What verifies it |
| --- | --- | --- | --- |
| `R1` | Request | Public owner, mode, dedicated controls, defaults, the accepted subset and any intentionally opaque arguments | The production entry receives the intended values. Method existence is not evidence. |
| `V1` | Validation | Which invalid, conflicting and unsupported inputs are rejected before spawn | Rejection tests showing that no child was spawned. Validity decided upstream is stated as such and its failure is tested. |
| `G1` | Values and grammar | Upstream-observed, wrapper-accepted and required values; order, arity, repetition and precedence | Pinned upstream evidence for upstream claims and wrapper tests for accepted behavior. Dynamic identifiers stay open values. |
| `O` | Output | Bounded opaque output, a parsed finite result or structured events; unknown and malformed handling | A version-bound output contract, provenance-bearing fixtures and parser tests that include negative cases. |
| `E1` | Errors | The distinctions the interface promises among validation, spawn, exit, timeout, cancel and malformed output | Failure-path tests and redaction checks. No taxonomy beyond what upstream establishes. |
| `L` | Lifecycle | Finite completion and reaping, or persistent ownership; cancel, timeout, readiness and shutdown where promised | Tests of the production ownership path. A launcher cannot verify protocol management. |
| `A1` | Applicability | Exact upstream version, targets, feature and configuration regime | Test environment and binary or fixture provenance for that version and target. |
| `X1` | Credentials | Credential sources and exposure channels | Synthetic sentinels. No credential in fixtures, logs, receipts or errors. |
| `X2` | Environment and filesystem | Inherited or cleared environment, effective home, config, cache, working directory, writes and network | Isolated sentinel effects and override precedence. A wrapper is not a sandbox. |
| `X3` | Diagnostics | Byte bounds, truncation, raw-output ownership and safe error projection | Malformed, oversized and sentinel-bearing output cases. |
| `X4` | Noninteractive effects | Stdin and TTY behavior, prompts, confirmations and side effects | No automatic answer outside a reviewed interaction. A timeout does not show that no mutation occurred. |
| `M` | Shared mapping | Wrapper-only, mapped to named capability ids, or a separately proposed extension | For a mapped operation, the capability's minimum semantics exercised through the production adapter. |

Each applicable obligation produces exactly one result: `verified`, `unverified`, `failed` or
`not_applicable`.

- `not_applicable` MUST carry a rationale and is valid only where the resolved policy permits it
  for the stated promise. An unavailable target or an unknown effect is not `not_applicable`.
- An unknown output contract is `unverified`. A known counterexample is `failed`.
- A constrained value subset, such as JSON-only output, is an applicable promise about that
  subset. It MUST NOT be reported as a promise about every upstream value.
- No score, percentage or highest-template label substitutes for per-obligation results.

## Obligation templates

An executable operation's obligation set is BASE plus every template its promise selects.

| Template | Adds to BASE | Is forwarding to the upstream CLI sufficient? |
| --- | --- | --- |
| `CORE` | A version-bound production parser; known, unknown and malformed events; ordering and a terminal result; events before completion, cancel and timeout, bounded buffering and child reaping | Never as the whole promise. |
| `SESSION` | Explicit id, last-session and fork selectors, conflict rules, no-match behavior and continued identity | Never as the whole promise. |
| `QUERY` | A verified record schema and selectors; filtering, ordering and pagination only where upstream exposes them; finite capture and cleanup | Never for a parsed-record promise. |
| `ADMIN` | Explicit scope and destructive intent, denied-write no-effect, before and after state and a bounded outcome; a parsed receipt only where upstream defines one | Only for a typed intent whose scope is not carried in arbitrary strings. |
| `DIAG` | A named constrained route, a reviewed effects boundary, bounded capture, truthful status, timeout, cancel and reaping | Yes, inside the reviewed boundary. |
| `OPAQUE` | An explicitly approved bounded-capture promise with truthful status | Yes. It is an approved promise, not a substitute for a missing schema. |
| `LAUNCH` | A dedicated handle; spawn failure, I/O ownership, a stated readiness definition, exit, stop and drop behavior and reaping | Only with a dedicated handle and a tested division of protocol ownership. |
| `MANAGED` | `LAUNCH` plus initialization, request and response correlation, notification routing, cancellation, disconnect and shutdown | Never. |
| `MCP` | Isolated effective home, target and configuration admission, write opt-in and the output and error rules of the [MCP management spec](unified-agent-api/mcp-management-spec.md) | For the existing opaque output promise only. |
| `REMOTE` | Endpoint and credential intent and client disconnect and cancel semantics. Ownership of the remote server is excluded unless separately promised. | Never from accepting a URL alone. |
| `EXCLUDED` | Classification evidence and rejection tests at the headless boundary. No executable promise. | Not applicable. No support claim is made. |

Classification rules:

1. A selector MUST name an exact command path and mode, with explicit flag and value predicates
   where they change effects. Aliases are resolved before matching.
2. Selectors that overlap with different effects or requirements are an error. First match does
   not win.
3. A flag, value or changed default that alters an operation's effects MUST NOT inherit the parent
   command's template. A refresh flag on a query, an output path on a diagnostic and a listener
   address on a launcher are examples.
4. A unit with an unknown effect, a new persistent mode or no resolvable template is
   `classification_required`. Acquisition MAY record it. Execution readiness for the affected
   operation MUST wait for classification.
5. A unit inside a governed operation inherits that operation's template. It inherits a forwarding
   permission only when evidence shows it stays inside the approved effects boundary.
6. `EXCLUDED` applies to one mode. It MUST NOT cover a command that also has a headless mode.
7. An override names the exact scope, the replacement obligations, the rationale, the versions and
   targets and a maintainer approval reference. An executor MUST NOT author an override, relax a
   requirement or broaden an exclusion to close a packet.
8. Evaluation covers every observed or claimed unit of a depth-enrolled operation, whatever its
   coverage level. A unit at `passthrough` or `unsupported` is evaluated, not skipped. The
   name-level audit lists of the maintenance request contract are unaffected.
9. A generic arbitrary-argument escape hatch never qualifies an operation or satisfies a template.

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
   of operations that back it for that agent. The capability is **depth-qualified** for one
   `(agent, version, target)` only when every operation in that set is depth-enrolled and
   qualified, `M` included.
6. A capability that no depth enrollment claims is **not assessed**. It is neither qualified nor
   failed.
7. The charter's promotion allowlist is an exception to the two-backend threshold only. It is not
   qualification, lifecycle eligibility or permission to advertise.
8. A capability absent from the capability matrix because safe defaults leave it off is neither
   qualified nor free of obligations.
9. Maintenance MUST preserve or repair mappings that already exist. Adding a mapping is not
   maintenance work and never becomes due through discovery.

The support matrix contract owns how capability-level results are published.

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

- the obligation ids and the policy and obligation bindings it was produced under
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
- the bindings differ and no validated reuse binding covers the difference
- the producer is unsupported, or the evidence comes from an unrelated CI job
- the only support is an author-entered success value

A CI conclusion associated with a pull request head establishes that association. It does not
establish which tree was built.

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
  affected crate, its transitive build dependencies and every explicitly referenced adapter,
  protocol, test and fixture input. Missing dependency provenance is `unverified`, not an empty
  set. An unrelated change invalidates nothing.
- Evidence produced before a generation's depth enrollment, a successful closeout included, is
  admissible for that generation only when it meets every rule in this section.
- Evidence is repository verification inside the existing repository and CI trust boundary. It is
  not remote attestation.

## Proof, enrollment, qualification and acceptance

These are four separate facts. One existing approval MAY establish more than one of them where its
recorded authority covers them. They do not require separate status fields, artifacts or approval
steps.

| Fact | Establishes | Does not establish |
| --- | --- | --- |
| Path machinery proof | A lifecycle path can enforce this contract across its whole acceptance chain, shown in isolation | Qualification of any real operation |
| Depth enrollment | These rules apply to one exact approved scope | That implementation or evidence work is finished |
| Qualification | Every applicable obligation of one promise is `verified`, and every `not_applicable` is permitted and justified, on exact inputs | Closeout, publication or promotion authority |
| Acceptance | The qualification and lifecycle requirements for one authorized effect are met at the time of that effect | Authority for another version, target, scope or release |

Rules:

1. Depth enrollment MAY begin with implementation and evidence work outstanding.
2. Authorized debt MAY disposition due work for acceptance under its controlling contract. It
   never yields `verified` and never makes an operation qualified.
3. An empty due list is not qualification. A qualified subset MUST be named by its exact promise
   and scope.
4. Scope outside depth enrollment carries no depth claim. It MUST NOT be counted as `verified` or
   `not_applicable`.
5. Publishing a truthful pending, partial or failed assessment MUST remain possible. It MUST NOT
   require qualification, and it MUST NOT advance a pointer, retire a marker or add advertising.

## Bindings

A depth-enrolled generation keeps four logical bindings. They are relationships, not four files,
stores or schemas, and existing packet, report and closeout fields carry them where sufficient.
Shared storage MUST NOT merge their authority or their invalidation.

| Binding | Content | May change by | A change invalidates |
| --- | --- | --- | --- |
| **Event** | `request_commit`, `request_recorded_at`, trigger and source | Never inside a generation. A new generation has its own Event and a predecessor reference. | Nothing. It is attribution, not tested code. |
| **P**, policy | Resolved rules and templates, promises and subsets, overrides and exclusions, path enablement, depth enrollment selectors, the debt delegation and the initial authorization baseline | Maintainer approval, through explicit re-freeze or supersession | O, E and every dependent closeout and publication result |
| **O**, obligations | The P reference, exact version and targets, acquired input identities, operation-to-surface edges and the concrete obligation set, independently required acceptance work included | Acquisition at its freeze; afterwards only explicit re-freeze | E and every dependent closeout and publication result |
| **E**, execution | The P and O references, implementation and evidence identities, materialized debt grants, validated transitions from P's baseline and derived results | Execution, evidence refresh and delegated debt transitions | Dependent closeout and publication results |

Rules:

1. Acquisition MUST NOT grant, widen or create policy authority. A unit discovered under an
   approved template changes O and not P.
2. E MUST NOT rewrite P or a frozen O. Reconciliation compares every changed grant against P's
   baseline and the allowed transitions. An unchanged policy identity is not sufficient.
3. A policy amendment is prospective. It MUST NOT relabel evidence produced under an earlier P as
   meeting the amended rules.
4. A debt transition is not a policy waiver. See [Debt operations](#debt-operations).
5. P is resolved for the affected acceptance scope. A change to an unrelated agent's or path's
   selection does not change it. A change to a shared governing rule does.
6. No binding's identity may depend on its own digest, on a future output or on the commit that
   will contain it.
7. The freeze points of the maintenance request contract are unchanged. On the acquisition lane
   that opens without target reports, O is frozen at the existing second freeze and the opening
   placeholder stays non-executable. A lane whose reports exist at open, the docs-only lane and a
   declared stand-down keep their existing treatment.
8. A consumer acting on depth-enrolled scope MUST require the current executable schema revision
   and all four bindings. A missing field or an unsupported revision is an error. It is never read
   as "not depth-enrolled".
9. Artifacts completed under an earlier contract keep their original meaning. They are not
   translated, backfilled or presented as depth evidence. An unfinished generation that an adopted
   revision makes incompatible MUST stop. Continuing it uses the existing preparation and
   supersession path with fresh bindings.

### Debt operations

| Operation | Who | Limits | Binding treatment |
| --- | --- | --- | --- |
| Renew an existing grant | Executor | Only a debt identity the frozen delegation covers, and only while the same blocker holds. Only `scope_target_triples`, `authorized_at_version` and `authorization_evidence_ref` change. Targets stay inside the delegation and match current target reports. | P unchanged. E changes, and dependent closeout and publication results become stale. |
| Retire satisfied debt | Executor | Only when live evidence covers every scoped target and obligation, or proves upstream removal under existing rules. Partial satisfaction keeps the row. Absence from help output is not proof. | As above. |
| Refresh evidence | Executor | Inside the selected contract. | P and grants unchanged. |
| Add debt or widen authority | Maintainer | A new identity, reason, obligation coverage or target outside the delegation. The executing packet MUST NOT defer a newly discovered non-TUI gap. | P re-freeze or supersession, with downstream invalidation. |

An unsupported or unproved transition MUST fail closed. Execution never closes or promotes its own
work.

## Depth enrollment

1. A depth enrollment is a positive selection that binds the agent, the lifecycle path, one exact
   upstream version, the operations and their promises, their modes and required values and
   explicit targets. This revision defines enrollment of one exact generation only. It defines no
   rule that carries a selection to a later version.
2. A later version inherits nothing from an earlier depth enrollment: no enrollment, no
   qualification and no publication authority.
3. A production depth enrollment MUST name an enabled lifecycle path. A selection made inside an
   isolated proof workspace is not a production depth enrollment and MUST NOT produce a protected
   effect on production outputs.
4. Everything outside the positive selection is the **unenrolled remainder**. The remainder is
   declared once, by rule. It MUST NOT be enumerated per historical operation, modeled as disabled
   placeholders or compared against an adoption baseline.
5. The unenrolled remainder carries no depth claim. It is not an exclusion, a waiver or debt, and
   every existing maintenance, regression, onboarding, debt, acquisition and publication duty
   still applies to it.
6. Depth enrollment adds obligations. It MUST NOT remove or narrow required uplifts, target
   acquisition completeness, existing shared promises or the release-watch ratchet.
7. Resolution MUST be deterministic and MUST come from registry-owned authority together with the
   approved request or approval for that generation. An executor's assertion, an optional field
   and a caller-supplied argument are not authority.
8. Missing policy for selected scope, an unsupported schema revision, unresolved or overlapping
   selectors, contradictory generation references and deleted bindings are errors. None of them
   resolves to "not depth-enrolled".
9. A flag, value or default discovered on a depth-enrolled operation stays attached to that
   operation. It MUST NOT fall into the remainder because a selector matched the earlier values.
10. Work that is due in a frozen generation stays due until it is satisfied or validly
    dispositioned. Moving it to another packet, ending the selection, pausing, removing
    advertising or deleting a declaration does not discharge it.
11. Depth enrollment MUST NOT create a second release-watch enrollment inventory.

## Admission

### Protected effects

A protected effect is a write, replacement, regeneration, removal or restoration of any output
below that alters the result for at least one depth scope tuple:

- support publication rows and their Markdown projection
- capability publication for an agent that has a depth-enrolled mapped operation
- version metadata
- the `latest_validated` and `latest_supported` pointers
- the embedded runtime-support projection
- an automation stand-down marker
- a maintenance closeout or proving-run closeout record
- the publication stages of the lifecycle record

Protected effects are of two kinds:

- A **reporting effect** publishes an assessment as it stands, pending, partial and failed results
  included.
- An **acceptance effect** advances a pointer, retires a stand-down marker, records a closeout as
  closed, publishes qualified support or adds capability advertising.

### Admission predicate

For every depth scope tuple a protected effect touches, the route MUST establish that:

1. depth enrollment resolves for the tuple under [Depth enrollment](#depth-enrollment);
2. the route holds authority for the tuple under the lifecycle path that owns its depth
   enrollment;
3. P, O and E are present, mutually consistent and current for the candidate being written;
4. the content to be published states the results as they are; and
5. for an acceptance effect, every independently required work item and every due depth
   obligation authoritative for the tuple is satisfied or validly dispositioned under its
   controlling contract.

A failed admission MUST leave every authoritative output unchanged.

### Complete mediation

Every route MUST, before a protected effect, either enforce the admission predicate for every
tuple the effect touches or refuse the effect.

1. The tuples an effect touches are determined by what the effect would change, not by the agent,
   version or scope the request names. A regeneration requested for one agent that would replace
   another agent's depth-enrolled row touches that row. A request for a version outside every
   depth enrollment that would replace a depth-enrolled pointer or row touches that tuple.
2. Not matching a depth enrollment is never permission to overwrite a depth-enrolled result.
3. Authority is established per touched tuple, under the lifecycle path that owns that tuple's
   depth enrollment. Reaching a shared writer from another path confers none. A gate that admits
   on the wrong path's authority does not satisfy this section.
4. A route MUST NOT produce an acceptance effect on a tuple whose depth enrollment belongs to
   another lifecycle path.
5. A lower-level writer is governed through every caller that can reach it, not only its usual
   one.
6. A caller MUST NOT narrow the touched set by omitting an optional scope or enrollment argument.
7. This section covers the commands, workflows, scripts and library interfaces the lifecycle
   supports for these outputs. Hand edits of generated outputs are outside it. Repository
   validation MUST reject depth-enrolled output that lacks its bindings or contradicts
   authoritative scope.

### Final admission

The protected operation is the whole sequence that reads current state, plans, writes and, on
failure, restores. A route MUST:

1. hold serialization ownership of the protected operation before its first write;
2. establish the admission predicate while holding that ownership, against the frozen bindings,
   current authority, the relevant inputs and the expected state of the outputs it will change;
   and
3. keep that authorization valid through its last write and through any failure handling.

Rules:

- An admission result obtained before ownership was held MUST be established again. A check made
  inside ownership, with every invalidating change excluded until the effect completes, is the
  final check and need not be repeated.
- Restoring outputs from a snapshot is a protected effect. A restoration MUST NOT overwrite state
  it no longer owns.
- Freshness means the frozen selection is still authorized under current authority. A route MUST
  NOT substitute a newer policy for the frozen P.
- This revision requires serialization of protected operations. Per-writer compare-and-refuse MAY
  replace it later by amendment.
- No cross-process lock or atomic multi-file transaction is assumed to exist. The serialization
  boundary MUST be demonstrated for each route. A route for which it cannot be demonstrated MUST
  refuse protected effects.
- An interrupted operation MUST NOT leave output that reads as a completed acceptance.

### Route inventory

Annex A lists every protected effect with its routes. For each effect it records the affected
scope, aggregate and indirect changes included; every reachable route; the authority each route
acts under; whether the route enforces admission or refuses; which other writes and restorations
can interfere; and the source references and conformance evidence. It states present behavior and
required behavior separately.

- The inventory is built from the outputs backward to every writer and from the entrypoints
  forward to their effects, and the two views are reconciled.
- It is final for the source revision it names. It MUST be established again against the revision
  that carries enforcement before a path is enabled.
- A change that adds or alters a route MUST update the inventory and its conformance coverage
  before that route may perform a protected effect.

## Path enablement

Enabling a path turns enforcement on. It asserts nothing about any operation's qualification.

The maintenance path MAY be enabled when all of the following hold:

1. this contract and the executable schema revisions it requires are adopted, with explicit
   maintenance path and selection authority;
2. the complete maintenance chain of acquisition, frozen obligations, execution, audit, manual
   closeout and separate promotion is proven end to end in an isolated workspace that cannot
   change production pointers or closeouts;
3. every route in Annex A satisfies [Complete mediation](#complete-mediation) and
   [Final admission](#final-admission), routes that belong to the onboarding path included;
4. that enforcement is landed, with reviewed positive and negative gate evidence; and
5. the maintainer authorizes enabling the path for a bounded production scope.

Enabling the maintenance path does not require the onboarding path to be proven. A single route
that fails item 3 blocks it.

The onboarding path MAY be enabled when items 1, 3, 4 and 5 hold for it and an independent
new-entry run proves approval, minimum integration and runtime profile, runtime evidence
selection, publication and proving-run closeout. Maintenance evidence does not substitute.
Enabling it MUST establish item 3 again for the maintenance path.

A synthetic agent in an isolated workspace MAY prove the onboarding machinery. It qualifies no
real agent. A real agent's promised scope is qualified by its own upstream evidence before any
acceptance effect.

A cross-agent workflow that adds shared mappings MUST NOT be enabled until both paths are enabled.

When a prerequisite stops holding, new acceptance effects on that path MUST be refused. The path
MUST NOT fall back to rules that predate this contract.

## Additive shared integration

Adding a shared mapping for an operation that already landed in a wrapper is separate work from
maintenance and from onboarding minimum integration. This contract fixes its rules and defines no
command for it.

1. Candidates are derived from landed wrapper operations, their approved promises and their
   existing evidence. Names and help output alone do not make a candidate. Derivation reuses the
   shared evaluator and MUST NOT become a second coverage or qualification engine.
2. Candidates are grouped by proven request, effect, output, error and lifecycle semantics. Equal
   command names are not equivalence. An uncertain equivalence stays a visible question.
3. Each candidate receives one disposition from the maintainer or an explicitly bounded approval
   authority: map to an existing capability, propose an extension, keep wrapper-only, or defer with
   a rationale and an owner. A disposition is a routing decision, not an achieved result.
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

- Depth results MUST be published as committed evidence under the agent's manifest root, in an
  evidence category the support matrix's neutral root intake already reads. A second evidence
  store MUST NOT be introduced.
- Aggregates MUST keep per-target results. One target's evidence never qualifies another.
- Machine-readable and Markdown depth facts MUST agree.
- The runtime-support payload stays version-only. Keeping depth facts out of that payload does not
  exempt a depth-enrolled pointer change from admission.
- Workflow YAML transports inputs and invokes shared commands. It MUST NOT carry template,
  enrollment, exception or admission policy.

Reports MUST keep these outcomes distinguishable:

| Outcome | Meaning |
| --- | --- |
| Qualified forwarding | A bounded forwarding promise with every obligation `verified`. It MAY still carry the `passthrough` coverage level. |
| Insufficient depth | A depth-enrolled obligation is `failed` or `unverified`. It is due. |
| Authorized debt | Unavailable behavior under a valid target- and version-scoped grant. Never `verified`. |
| Mode exclusion | One TUI mode excluded with a rationale and visible accounting. |
| Not depth-enrolled | No depth claim. |
| Unresolved | Enrollment, classification or evidence could not be resolved. This is an error and is never reported as not depth-enrolled. |

## Conformance

An implementation conforms when tests on the production paths demonstrate every row.

| Area | Required behavior |
| --- | --- |
| Classification | Approved diagnostic forwarding qualifies. An unknown effect blocks readiness. A mutating or persistent option does not inherit a diagnostic template. Overlapping selectors fail. |
| Accounting | Inside depth-enrolled scope, passthrough flags and arguments and commands at coverage level `unsupported` are evaluated and reported. A command with both TUI and headless modes keeps its headless obligations. |
| Values | The accepted subset produces the correct argv. Invalid values, arity and conflicts are rejected before spawn where promised. A constrained subset is never reported as every value. |
| Evidence | A missing test, zero executed tests, a wrong target or version, a stale input, an unsupported producer and an author-entered success value each fail to verify an obligation. |
| Runtime | Malformed output, nonzero exit, redaction, cancel, timeout, EOF and reaping are tested on the promised path. Launcher evidence does not verify a managed protocol. |
| Bindings | Discovery under unchanged rules changes O and not P. A post-freeze acquisition change invalidates dependents. Delegated renewal and fully evidenced retirement keep P. New rows, widened authority and unproved transitions are refused. |
| Depth enrollment | The remainder makes no claim and waives nothing. Missing or malformed enrollment is refused. Enrolling an existing operation requires its full proof. A later version inherits nothing. |
| Qualification | Authorized debt yields partial acceptance and never a qualified operation. Scope outside depth enrollment never counts as `verified`. |
| Mediation | With the maintenance path enabled, an onboarding route's acceptance effect on a maintenance tuple is refused. A regeneration for another agent or version that would change a depth-enrolled result is admitted against that result or refused. |
| Final admission | A change to authority, inputs or outputs after an earlier pass prevents the effect. A restoration does not overwrite newer state. An interrupted operation does not read as accepted. |
| Reporting | With evidence outstanding, a truthful pending or failed assessment publishes and every acceptance effect is refused. |
| Publication | Per-target machine-readable and Markdown facts agree. The runtime-support record stays version-only. Every promotion entrypoint refuses due gaps and leaves pointers and markers unchanged. |
| Shared mapping | A wrapper-only operation creates no adapter deficit and no advertising. A regression in an existing mapping blocks acceptance. Removing a declaration does not discharge due work. |
| Contract change | A wrong schema revision and a re-bound historical result are refused. Reprepare preserves lineage and converts nothing. |

## Annex A: route inventory

Not yet written. This contract MUST NOT be approved without it.
