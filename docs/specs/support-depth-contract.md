# Support-Depth Contract

Status: Draft, awaiting maintainer approval
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
| Release-watch enrollment | [Registry contract](agent-registry-contract.md) | Unchanged. Depth enrollment is a separate selection. |
| Capability ids and their minimum semantics | [Capabilities spec](unified-agent-api/capabilities-schema-spec.md) and each capability's owner document | Unchanged. |
| The capability promotion rule and its allowlist | [Onboarding charter](cli-agent-onboarding-charter.md) | Unchanged. Depth admission is added for the capabilities a depth enrollment claims. |
| Lifecycle stages and support tiers | The committed lifecycle record, as the onboarding charter designates | Unchanged. Neither is a depth result. |
| Runtime profiles `minimal`, `default` and `feature_rich` | The lifecycle implementation in `crates/xtask/src/agent_lifecycle.rs`; no normative owner | Unchanged. A runtime profile is not a depth result. |
| Surface exclusions in `parity_exclusions` | Each manifest root's `RULES.json` and validator spec, where the root has them | Unchanged. See classification rule 7 for mode exclusions. |
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
  content identities of P and O, the identity of E, each obligation's result for each target and,
  for a `verified` result, the committed evidence it rests on, named by path and content identity,
  and the capability mappings those results serve. It also lists an acceptance entry for every
  closeout, publication or promotion that made an acceptance effect for the version. An entry
  names that closeout, publication or promotion and identifies what the record stated when the
  entry was made. The record is first written when a generation of the version first freezes P,
  and it is extended as O is frozen and as E changes. A later generation of the same version
  continues the same record: the record then states that generation's Event and P, shows O, E and
  results as not yet produced until that generation produces them, and keeps its acceptance
  entries. The record's file name and schema, including the form in which an entry identifies what
  the record stated, are defined in the manifest root's validator spec.
- **Working files.** The per-agent maintenance packet files of the current generation under
  `docs/agents/lifecycle/`, such as `maintenance-request.toml` and `maintenance-closeout.json`. A
  later generation replaces them.
- **Remainder.** Everything outside every depth enrollment.
- **Depth admission.** The check defined in [Depth admission](#depth-admission). It is distinct
  from the stand-down admission gate of `close-agent-maintenance`, which this contract leaves
  unchanged.
- **Depth-gated output** and **depth-gated effect.** An output listed in
  [Depth-gated effects](#depth-gated-effects), and a change to such an output that touches a depth
  scope tuple.
- **Integration branch.** The branch that holds committed publication truth, currently `staging`.
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
- A result whose bindings are no longer current is `unverified` until it is verified again.
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
   depth-enrolled and qualified, `M` included.
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
| **Event** | Maintenance: `request_commit`, `request_recorded_at`, trigger and source. Onboarding: the approval artifact's `approval_commit` and `approval_recorded_at`. | Nobody inside a generation. A later generation has its own Event. | Nothing. It is attribution, not tested code. |
| **P**, policy | Resolved rules, templates and classifications, promises and subsets, overrides and exclusions, depth enrollment selectors, path enablement, the debt delegation and the initial authorization baseline, for the depth scope tuples it governs | The maintainer, through explicit re-freeze or supersession | O, E and every dependent closeout and publication result |
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
   selection, or the enablement of another path, does not change it. A change to a shared
   governing rule does.
6. A binding's identity MUST NOT depend on its own digest, on a future output or on the commit that
   will contain it.
7. The freeze points of the maintenance request contract are unchanged. P is frozen when the
   generation opens, at the request's first freeze. On the acquisition lane that opens without
   target reports, O is frozen at the existing second freeze and the opening placeholder stays
   non-executable. A lane whose reports exist at open keeps its existing treatment. A generation
   in which O is never frozen from target reports, on the docs-only lane or on an acquisition lane
   stood down before its second freeze, cannot reach acceptance for depth-enrolled scope. That
   scope is reported as insufficient depth and never as not depth-enrolled. Where P and O are
   frozen on the onboarding create lane is for the onboarding charter to define; see
   [Path enablement](#path-enablement).
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

## Depth enrollment

1. A depth enrollment is a positive selection that binds the agent, the lifecycle path, one exact
   upstream version, the operations and their promises, their modes and required values and
   explicit targets. It covers every generation of that version. This revision defines depth
   enrollment of one exact upstream version only. It defines no rule that carries a selection to a
   later version.
2. A later version inherits nothing from an earlier depth enrollment: no depth enrollment, no
   qualification and no publication authority. How a later version may move a depth-enrolled
   version's pointer or replace its working files is set out in [Later versions](#later-versions).
3. A production depth enrollment MUST name an enabled lifecycle path. A selection made inside an
   isolated proof workspace is not a production depth enrollment and MUST NOT produce a depth-gated
   effect on production outputs.
4. The remainder is declared once, by rule. It MUST NOT be enumerated per historical operation,
   modeled as disabled placeholders or compared against an adoption baseline.
5. The remainder carries no depth claim. It is not an exclusion, a waiver or debt, and every
   existing maintenance, regression, onboarding, debt, acquisition and publication duty still
   applies to it.
6. Depth enrollment adds obligations. It MUST NOT remove or narrow required uplifts, target
   acquisition completeness, existing shared promises or the release-watch ratchet.
7. Resolution MUST be deterministic. A version's depth enrollment resolves from its depth record.
   Until the record is written, registry-owned authority and the frozen request or approval
   resolve it. An executor's assertion, an optional field and a caller-supplied argument are not
   authority.
8. Missing policy for selected scope, an unsupported schema revision, unresolved or overlapping
   depth enrollment selectors, contradictory generation references and deleted bindings are errors.
   None of them resolves to "not depth-enrolled". A removed depth record is a deleted binding. A
   replaced working file is not, because the depth record identifies the bindings that file
   carried.
9. A flag, value or default discovered on a depth-enrolled operation stays attached to that
   operation. It MUST NOT fall into the remainder because a depth enrollment selector matched
   the earlier values.
10. Work that is due in a frozen generation stays due until it is satisfied or validly
    dispositioned. Moving it to another packet, ending the selection, pausing, removing
    advertising or deleting a declaration does not discharge it.
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
| Stand-down markers | Removal of a marker for the tuple's version. Declaring a marker is not a depth-gated effect. |
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

The following **record invariants** hold for every depth record and every depth-gated output:

1. A depth-enrolled version has a depth record from the time a generation first freezes P for it.
   A depth record MUST NOT be removed, whether by retention pruning or otherwise, and an
   acceptance entry MUST NOT be removed or altered.
2. What a record states changes only when a later generation of the same version continues the
   record, or through a change that [Bindings](#bindings) permits to the binding concerned: the
   selection and the capability mappings belong to P, the obligations to O, and the results and
   their bound evidence to E.
3. A published result is the result the record states, or `unverified`. It MUST be `unverified`
   when the record does not yet state the result, when the result's bindings are not current, or
   when the evidence the record binds for it is absent or has another content identity. No promise
   is published as qualified, and no capability as depth-qualified, on such a result. A tuple the
   record selects MUST NOT be published as not depth-enrolled.
4. Every acceptance effect other than listing an acceptance entry MUST have an acceptance entry
   made for what the record stated when the effect was made. A promise published as qualified, or
   a capability published as depth-qualified, is supported only while an acceptance entry exists
   that was made for what the record states now. In the lifecycle record of an agent that has a
   depth record, a `published` or `closed_baseline` stage and a closeout evidence id MUST report
   a packet or closeout that the same revision holds.

A depth-gated effect is an **acceptance effect** for a tuple when it does any of the following:

- sets or advances a pointer, or the runtime-support projection, to the tuple's version
- sets the tuple's version status to `validated` or `supported`, or records a passed per-target
  outcome
- retires a stand-down marker
- creates or changes a maintenance closeout record, or records a proving-run closeout as `closed`
- advances the lifecycle record's stage to `published` or `closed_baseline`, adds one of its
  closeout evidence ids or clears a drift side state
- lists an acceptance entry in a depth record, or publishes a promise as qualified or a capability
  as depth-qualified
- adds capability advertising

Every other depth-gated effect is a **reporting effect**.

### Depth admission predicate

For every depth scope tuple a depth-gated effect touches, the route MUST establish:

1. that depth enrollment resolves for the tuple under [Depth enrollment](#depth-enrollment);
2. that what it writes states the tuple's results as they are. A result whose bindings are not
   current is written as `unverified`. Nothing is written as better than its current bound
   evidence supports; and
3. that it removes no depth record, removes or alters no acceptance entry, and changes what a
   depth record states only as record invariant 2 permits.

For an acceptance effect the route MUST also establish:

4. that it holds authority for the tuple under the lifecycle path that owns the tuple's depth
   enrollment;
5. that P, O and E are present, mutually consistent and current; and
6. that no unit of the tuple's operations is `classification_required`, and that every
   independently required work item and every due depth obligation authoritative for the tuple is
   satisfied or validly dispositioned under its controlling contract.

A reporting effect needs no lifecycle-path authority. If depth admission fails, every depth-gated
output MUST be left unchanged.

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
   checks that enforce the invariants. They are defined beside the depth record's schema in the
   manifest root's validator spec, and they are part of the enforcement that
   [Path enablement](#path-enablement) item 4 requires. A manifest root without a validator spec
   MUST NOT hold a depth record.

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
   maintenance path and selection authority;
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

The onboarding path MAY be enabled when items 1, 3, 4 and 5 hold for it, the onboarding charter
defines where P and O are frozen on the create lane, at a point where the agent's manifest root and
its exact upstream version exist, and an independent new-entry run in an isolated workspace proves
approval, the onboarding checklist's implementation and evidence steps, runtime evidence
selection, publication and proving-run closeout. Maintenance evidence does not substitute.
Enabling it MUST establish item 3 again for the maintenance path.

A synthetic agent in an isolated workspace MAY prove the onboarding machinery. It qualifies no
real agent. A real agent's promised scope is qualified by its own upstream evidence before any
acceptance effect.

A cross-agent workflow that adds shared mappings MUST NOT be enabled until the maintenance and
onboarding chains are both proven and landed.

When a prerequisite stops holding, new acceptance effects for depth-enrolled tuples on that path
MUST be refused. The path MUST NOT fall back to rules that predate this contract.

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

- Depth facts MUST be published from depth records. A depth record is committed evidence under
  the agent's manifest root in `reports/**`, an evidence category the support matrix's neutral
  root intake already reads. A second evidence store MUST NOT be introduced.
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

## Annex A: route inventory

Not yet written. This contract MUST NOT be approved without it.
