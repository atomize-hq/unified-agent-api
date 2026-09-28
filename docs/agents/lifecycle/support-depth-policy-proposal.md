# Support-depth policy and evidence rules — proposal

Status: **Draft; the three clarifications recorded below were accepted for inclusion on 2026-09-28. Broader policy adoption and implementation remain pending.**

Baseline: `f61534be004305a32e36f9631f218f8094572b59` (staging snapshot). The completed Codex and Claude Code rehearsals retain their meaning under the contracts they executed. This proposal defines what future onboarding and maintenance should accept before OpenCode expansion. Its completion claim is a coherent, source-grounded decision package, not qualified wrappers or adopted policy.

Consultation: [ChatGPT Pro recommendation](https://chatgpt.com/c/6ab9a464-4684-83ea-8391-ea88e04e8849), supplied with a bounded 1,325-file repository bundle at this baseline. Pro supplied advice, not a patch; it did not run repository tests. The recommendation is incorporated below with explicit proposed decisions and implementation gates. No historical CI conclusion is being revalidated here.

## 1. Decisions proposed for approval

| Decision | Recommended rule |
| --- | --- |
| Meaning of support | Add independent semantic obligations alongside existing name coverage. Keep `explicit`, `passthrough`, and `intentionally_unsupported` meanings unchanged. |
| Finished forwarding | Permit it for explicitly approved, bounded diagnostic/compatibility operations with documented effects, argument handling, output bounds, errors and lifecycle. An arbitrary argv escape hatch never qualifies an entire family. |
| Typed support | Require dedicated controls and validation where the family needs them. A method name or string argument alone is insufficient evidence. |
| Value subsets | Permit a deliberately constrained subset such as JSON-only output, provided required, accepted and upstream-observed values are separately recorded. Never claim all upstream values from name coverage. |
| Shared API eligibility | Decide separately per operation: wrapper-only, mapping to an existing shared capability, or a separately reviewed extension. Wrapper usefulness does not require shared API inclusion. |
| Work routing | Individual-agent maintenance satisfies required wrapper obligations and preserves or repairs affected existing shared promises. A separate cross-agent workflow discovers and selects optional additive shared integration over landed wrappers. Eligibility or selection neither advertises support nor relaxes due obligations. Onboarding minimum integration and profile meanings remain unchanged. |
| Evidence | Require reviewed assertion adequacy plus reproducible, input-bound verification. Source references and author-entered booleans cannot self-certify support. |
| Debt operations (decision recorded 2026-09-28) | Preserve constrained executor renewal of existing debt and evidence-backed retirement under frozen delegation; new debt or expanded authority requires maintainer approval and explicit re-freezing. See section 4. |
| Identity (clarification recorded 2026-09-28) | Keep normative policy/delegation identity separate from acquisition-derived obligations and evolving grant/evidence state. |
| Adoption | Activate prospectively with an enumerable legacy baseline. Unchanged old declarations remain visibly unassessed; new behavior and selected retrofits receive due obligations. |
| Promotion | For activated generations, require no outstanding due obligations before promotion or retirement of their stand-down marker. Authorized preexisting debt can remain visibly partial. |
| OpenCode | Preserve `run --format json` as canonical prompt transport. Review an additive wrapper-scope amendment before implementing agent-specific helpers; retain separate shared-mapping decisions. |

Approval of this proposal should authorize preparation of normative contracts and bounded implementation packets, not automatic execution of every phase below. Any consequential deviation from these decisions returns for review.

## 2. Why the current machinery needs an additional contract

The existing machinery establishes reproducible acquisition, coverage declarations, frozen maintenance requests, execution boundaries, manual closeout and separate promotion. Those successes do not establish the adequacy of every accepted interface.

- [Coverage generator contract](../../specs/codex-wrapper-coverage-generator-contract.md) allows `explicit` for a first-class field **or** dedicated method. [Scenario 22](../../specs/codex-wrapper-coverage-scenarios-v1.md) requires passthrough labels for its compatibility additions. Neither label encodes validated inputs, parsed results, or lifecycle guarantees.
- [Maintenance request contract](../../specs/maintenance-request-contract-v1.md), “Canonical surface identities,” explicitly uses name-only identities. It records the `run --format` JSON-only example and the `uaa-0041` semantic gap. It also documents that command-level `deltas.unsupported` does not enter the current uplift list (`uaa-0042`).
- [Report filtering](../../../crates/xtask/src/manifest_report/report/filtering.rs) puts passthrough commands in candidates, while passthrough flags and arguments do not enter missing lists. Evaluating only missing names cannot qualify already-declared controls.
- [Maintenance support audit](../../../crates/xtask/src/agent_maintenance/support_audit.rs) initializes missing wrapper and backend support from the same gaps. This cannot determine whether a wrapper-only operation needs shared-adapter work.
- [Support-matrix derivation](../../../crates/xtask/src/support_matrix/derive.rs) classifies missing names as unsupported and intentional/wrapper-only caveats as partial. Passthrough alone does not make a row partial. The [matrix contract](../../specs/unified-agent-api/support-matrix.md) defines this backend support as wrapper-derived; it is not proof of shared API implementation.
- [Runtime follow-on models](../../../crates/xtask/src/runtime_follow_on/models.rs) and [lifecycle derivation](../../../crates/xtask/src/runtime_follow_on/lifecycle.rs) describe integration profiles. `minimal`, `default`, and `feature-rich` must not become semantic qualification tiers.

Keep these existing facts and add a separately identified qualification result. Do not infer that every passthrough declaration is deficient or every explicit declaration is complete.

## 3. Units, dimensions and qualification

A policy applies to an operation and invocation mode, associated with existing command/flag/argument identities, a required value subset, exact upstream version and explicit target triples. One operation can own several name identities; the report retains those links. A family is a reusable requirement template, not an achieved status.

| Dimension | Required statement | Adequate evidence |
| --- | --- | --- |
| Request representation | Public operation, dedicated controls, finite variants, dynamic strings, fixed values and intentional opaque arguments | Bound API symbols and compile/argv tests for those controls; identify residual escape hatches |
| Validation | Invalid values, conflicting controls, unsupported combinations and headless restrictions decidable before spawn | Positive cases plus rejection tests that prove no child was spawned; upstream-only validation is explicitly distinguished |
| Values and grammar | Upstream-observed, wrapper-accepted and policy-required values; arity, ordering, repetition, defaults | Pinned help/source/docs or probes for upstream claims, wrapper tests for accepted behavior; dynamic identifiers stay dynamic |
| Output | Opaque bounded output, parsed finite result, or structured events; unknown/malformed behavior | Version-bound output contract and representative provenance-bearing fixtures, parser tests including negative cases |
| Errors | Validation, spawn, exit/signal, timeout/cancel, malformed output and accepted-request failure distinctions actually promised | Failure-path tests and diagnostic/redaction checks; no invented domain taxonomy where exit status is all that upstream establishes |
| Lifecycle | Finite completion/reaping or persistent ownership; cancellation/timeout policy; readiness, I/O ownership, backpressure, shutdown/drop where promised | Tests of the production ownership paths, terminality and cleanup; a launcher cannot certify protocol management |
| Applicability | Exact upstream versions and targets; any explicit compatibility/portability rule | Test environment and binary/fixture provenance; a fake Linux executable does not prove Windows or upstream behavior |
| Shared mapping | Wrapper-only, existing capability ID, or separately proposed shared extension | For shared mapping: advertising, request/event/error/finality tests through the actual adapter |

Credential handling, environment/filesystem ownership, diagnostic exposure and noninteractive behavior are cross-cutting obligations according to the operation's effects. Resolve each to a dimension/sub-obligation ID governed by the same applicability, due-status, evidence and qualification gates; these requirements cannot remain unenforced prose.

Each required dimension produces `verified`, `unverified`, `failed`, or `not_applicable` with a rationale. `not_applicable` is accepted only when the resolved policy permits it for the declared promise. Unknown output is unverified, not N/A. JSON-only is an applicable constrained value contract. No total score or highest-tier label substitutes for these facts.

An operation qualifies only when every applicable due obligation is verified and every N/A is permitted and justified. A missing test, stale receipt or unknown contract cannot be averaged away. Legacy deferral and authorized unsupported debt affect whether an obligation is due; neither turns it into verified support.

### Family defaults

| Family | Default finished interface | Boundary |
| --- | --- | --- |
| Core run/session | Dedicated controls, pre-spawn validation where decidable, explicit accepted subset, structured parsing required by its runtime protocol, terminal/error/cancel guarantees | Resume/fork/continue do not become finished raw forwarding merely because upstream spells them as flags. Agent-specific controls may remain wrapper-only. |
| Machine-readable inventory/query | Typed selectors/filter/pagination where present, parsed records from a verified output contract, defined malformed/unknown handling | Help's `--json` is not an output schema. Without verified machine output, a separately approved opaque query can be finished under a different promise. |
| State-changing administration | Typed intent and scope, explicit destructive choices, noninteractive execution, validation and bounded outcome/errors | Parsed receipts depend on upstream evidence; exit status and bounded diagnostics can suffice. Mutation scope must not be implicit in arbitrary strings. |
| Diagnostics/compatibility | Named constrained route/mode; reviewed effects, deterministic argument handling, bounded output, truthful errors, timeout/cancel/reaping | Approved forwarding is sufficient only within this boundary. Repair/reset/persistent modes inherit stronger obligations, not diagnostic permission. |
| Persistent servers/helpers | Dedicated handle and explicit process/I/O ownership, readiness and shutdown semantics appropriate to the promise | A launcher can leave protocol management to its caller if the division and cleanup responsibilities are explicit and tested. It must not be advertised as a managed client. |

These are proposed defaults, not certifications of existing methods. Direct TUI invocation remains excluded, explicitly by mode. A command containing a headless mode cannot have its whole subtree excluded just because another mode opens a TUI.

### Classification and overrides

Initially use exact command/mode selectors with explicit references to affected flags/arguments. Deterministic family resolution must reject ambiguous overlapping selectors. An override names the exact scope, replacement obligations, rationale, maintainer approval reference, versions and targets; it cannot be authored as part of ordinary executor remediation.

New units in known families inherit requirements. They inherit a forwarding permission only when evidence establishes that they remain within its approved effects and invocation boundary. An unknown-effect flag, new persistent mode or unknown family becomes `classification_required`. Acquisition can still record it, but executable completion requirements cannot be frozen as satisfied until classification is resolved. Ordinary additions already governed by the policy require no new human ceremony.

A relaxed requirement, broader exclusion, broader value subset with changed risks, or new shared abstraction is a policy/scope decision. Executors may satisfy requirements, gather evidence, or report a blocker; they cannot weaken them to close a packet.

## 4. Authority, bindings and derived facts

Use two authored inputs and extend the existing derived reports. Proposed paths below are designs for later implementation, not files introduced by this draft.

1. **Requirements:** registry-linked `cli_manifests/<agent>/SUPPORT_POLICY.toml`. Maintainer-owned family assignments, dimension requirements, supported subsets, mapping decisions, exceptions, activation and explicit legacy baseline. Refer to existing debt/exclusion authority rather than copying it into a second ledger. No achieved-status fields.
2. **Bindings:** registry-resolved `<crate_path>/support-depth-evidence.toml`. Implementation-owned operation/symbol links, tests, fixtures and upstream provenance submitted for checking. Shared-adapter bindings may reference `crates/agent_api` when mapping is required. No authoritative `typed=true` or `tests_passed=true` fields.
3. **Results:** extend `reports/<version>/coverage.<target>.json` and aggregates with resolved requirements, classification, per-dimension verification, due gaps and applicability, plus policy/input/evidence digests. Packet prose, support matrix and runtime publication consume these facts.

Keep `WrapperCoverageV1.level` unchanged. OpenCode currently has committed coverage without the same generator implementation as Codex; the design must accept declared coverage acquisition methods instead of requiring every wrapper to acquire one generator.

### Evidence qualification and reuse

Qualification needs both a reviewed mapping of assertions to requirements and reproducible execution evidence. The shared checker can verify references and receipts mechanically; it cannot infer that a test's name proves its assertions are adequate.

A qualification binding therefore includes the obligation IDs, relevant source/test/fixture content digests, upstream evidence identity, test invocation and selected test cases, actual executed test inventory, result, toolchain/features/environment, target, and upstream version. A successful command that selected zero tests does not qualify. A maintainer-reviewed adequacy record binds that same mapping and requirement digest. This can be carried by the existing independent review/closeout record with additional structured fields; no new standing reviewer role is required.

The implementation phase must define canonical digest serialization and the test-runner/receipt schema before writing consumers. The first implementation should use a bounded xtask-owned test runner and generated receipts; imported CI receipts are admissible only when they prove the same tested inputs, commands and environment. Receipt validation checks the artifact's producer and provenance under the existing repository/CI trust boundary, not merely an author-supplied success field. Closeout rechecks validity against the live candidate and independently reviews adequacy. This is a repository verification mechanism, not tamper-proof remote attestation.

Keep three evidence classes visible: help/discovery, wrapper mechanics using controlled binaries, and upstream-output/behavior provenance. Help does not prove parsing. An argv echo does not prove cancellation. Pinned source/docs and real-output fixtures can establish an upstream contract without an unnecessary live external call on every check; tests still prove the wrapper's interpretation.

Default reuse is exact version and target. A maintainer-approved portability or version-range rule names which obligations may reuse evidence, why the environment/contract is invariant, and which changes trigger revalidation. It cannot erase an explicit platform-specific runtime proof requirement. Without that rule, evidence is unverified on other targets/versions.

Changed implementation, relevant tests/fixtures, requirements or upstream inputs invalidate affected qualification. Changes outside the declared dependency set need no semantic rerun; defining that set must include transitive production dependencies and build inputs sufficient for the claim. A content-bound adequacy mapping can be reused when those inputs and obligations are unchanged. The [current CI resolver](../../../crates/xtask/src/agent_maintenance/closeout/evidence.rs) explicitly establishes association, not the tree built; retain that narrower meaning.

### Freeze and reconcile

A request carries four separate bindings:

- existing opening `request_commit` and recorded time, preserved as event metadata;
- **normative policy identity:** reference/schema/revision and digest of resolved rules, referenced defaults, approved exceptions, activation/exclusions, and the frozen debt delegation and initial authorization baseline;
- **acquisition obligation identity:** exact version/targets, acquired input digests and the concrete discovered operation/obligation set derived under those rules;
- **execution state identity:** implementation/test/fixture provenance, materialized debt grants and a validated transition history from the frozen authorization baseline.

The policy digest identifies immutable normative rules and authorizations, not the acquisition-derived enumeration or the mutable materialized grant rows. It includes the content identity of the initial authorization baseline and the delegation rules permitting the transitions below. A renewal cannot change that frozen baseline: the current inventory and transition record receive a separate execution-state digest. These are bindings over existing reports/inventory and receipts, not a second mutable governance-status ledger. Canonical serialization belongs to the later contract packet; neither policy nor evidence digests may refer to their own future outputs or commit SHA.

`prepare-agent-maintenance --from-request` preserves opening metadata and normative policy identity while completing the acquisition binding. A newly discovered flag governed by an existing approved family changes the concrete obligation set, not the policy digest or approval. That set is finalized at the existing second freeze; the opening placeholder remains non-executable. Unknown families or effects still require classification and, where rules change, explicit policy re-freezing. After acquisition completion, changed acquisition inputs/obligations require an explicit acquisition re-freeze and invalidate dependent prepared execution/closeout/publication evidence even when normative policy is unchanged.

### Debt operation ownership — decision recorded 2026-09-28

Preserve the current narrowly delegated executor renewal procedure in [contract policy](../../../crates/xtask/src/agent_maintenance/contract_policy.rs), and permit evidence-backed retirement through the same bounded machinery. The earlier suggestion to remove all debt writes from the executor envelope is withdrawn. The later normative contract must distinguish authority to define or expand a grant from authority to execute a frozen, constrained transition:

| Operation | Owner and allowed change | Identity and freeze treatment |
| --- | --- | --- |
| Renew an existing grant | Executor, only for a preexisting identity explicitly covered by the frozen delegation, when the same recorded blocker still holds. Change only target-version authorization, exact observed target scope and authorization evidence reference; no new rows, changed reasons/owners/follow-ons or additional semantic obligations. Targets must be within the frozen delegation envelope and exactly backed by current target reports, without overlapping grants. | Normative policy/delegation and frozen authorization baseline stay unchanged. Validate and record the materialized grant transition and new execution-state digest; rederive audit and invalidate dependent closeout/publication receipts. No fresh maintainer approval or policy re-freeze for an allowed transition. |
| Retire satisfied debt | Executor may retire an existing row only when live evidence proves all of its scoped targets/obligations covered, or proves upstream removal under existing rules. Preserve the row and proof in transition provenance. Partial satisfaction leaves the row in place for unresolved scope; a missing help entry or edited caveat is not proof. | Same delegated-transition reconciliation; policy identity unchanged. Current inventory/audit/evidence bindings change and dependent closeout/publication receipts become stale. No grant expansion or policy re-freeze. An exclusion change follows maintainer policy change control instead. |
| Refresh evidence | Executor may rerun tests or update acquisition/implementation evidence within the selected contract. | Evidence refresh alone does not change policy or grants. Changed acquisition content after its completion freeze follows the acquisition re-freeze rule above; changed tested inputs invalidate affected verification. |
| Add debt or expand authority | Maintainer decision outside ordinary execution. Includes a new debt identity, new reason, added obligation coverage, targets outside the delegation envelope, or any other relaxation not already delegated. Newly discovered non-TUI gaps remain non-deferrable by the executing packet. | Explicit authority change and policy re-freeze/supersession with downstream invalidation; never silently absorb it into ordinary reconciliation. Approval must still satisfy the controlling debt contract or separately amend it. |

The executor's writable envelope includes only these permitted inventory transitions and evidence bindings; normative policy/delegation remains read-only. Shared validation must compare every changed row against the frozen baseline and allowed transition history, not merely accept an asserted unchanged policy digest. Unsupported or unproved transitions fail closed. Manual closeout independently rederives the resulting audit; execution never self-closes or promotes. Exact target/version authorization continues to apply to each materialized grant; delegation does not create an implicit range or blanket future grant.

Implemented progress and valid delegated transitions under identical rules reconcile through an extension of Exact/Satisfied without rebaselining away due work. Prepared execution uses the frozen inputs and its declared transition allowance; downstream closeout/publication binds the resulting validated state. Policy/scope/exclusion/delegation or selected version/target changes require explicit supersession or re-freeze and invalidate downstream prepared evidence. Old-schema packets remain readable only under their explicitly historical, inactive generation; omitting new fields cannot disable qualification after activation.

## 5. Lifecycle integration and publication

One shared evaluator resolves policy and qualifications for both onboarding and maintenance. YAML only transports inputs, invokes shared commands and commits generated outputs atomically.

**Onboarding:** approval and reviewed policy → frozen runtime requirements → implementation/bound verification → independent qualification → publication-ready evidence → publication refresh → manual proving closeout.

**Maintenance:** registry policy → frozen request → acquisition under pinned policy → generated implementation/evidence obligations → execution → live rederived audit → manual closeout → separate parity promotion.

Preparation renders due obligations, exact scope, allowed exceptions, proof requirements and their source references in the canonical HANDOFF and derived prompts/reports. Wrapper writes follow wrapper obligations. Maintenance retains affected existing shared promises and other shared obligations already due for its acceptance scope; adapter edits follow actual repair needs, while verification can establish preservation without source changes. Optional additive integration follows the separate workflow below. Version the misleading mirrored audit fields instead of treating every CLI gap as missing shared API support or removing all backend responsibility.

A valid report with due implementation or evidence gaps is successful acquisition and outstanding audit work (retain exit 3). Invalid/ambiguous policy fails validation. Incomplete acquisition and target mismatch retain their existing distinctions. Required-support classification gaps block execution readiness and remain visible in acquired reports. Pending optional shared-eligibility decisions do not create required-wrapper classification failures; uncertainty about required wrapper behavior cannot be relabeled as an optional shared candidate. Execution and closeout cannot convert a valid gap into success using prose.

The support matrix retains existing `backend_support`, validation and pointer facts, and adds an explicitly named depth assessment with policy identity, applicable/due/verified/unverified counts and concrete gap references. Historical `supported` rows remain name-policy results, accompanied by `legacy_unassessed` depth where applicable. Do not relabel them as newly qualified. JSON and Markdown depth facts must agree. Keep depth details in operator-facing reports and the support matrix. The library runtime projection remains the existing `latest_validated` version-only API; neither its public record nor its interpretation gains depth metadata or a qualification claim. Regenerate/validate its version records from committed pointer truth after publication as today, with the new admission gate applied before future pointer advancement. Existing historical validated records are not removed by activation. Aggregates retain per-target failures and never let one target's proof qualify another.

For activated generations, add the same admission check to promotion and any direct command capable of advancing the relevant pointers/retiring the marker. Outstanding due obligations authoritative for the affected acceptance scope, including already-required shared mappings, block those effects together. Optional additive candidates do not become due merely through discovery. Moving a due obligation to another packet, or removing its capability declaration, cannot erase it; changing its authority or acceptance relationship requires the established approval/re-freeze path. Wrapper promotion never itself advertises a new shared capability. Valid authorized preexisting debt can remain partial; no blanket requirement that validation means full support. Truthful acquisition and partial publication remain possible before promotion. Manual closeout, stand-down protection and separate promotion stay distinct.

### Additive shared integration

A proposed separate xtask entrypoint inventories landed wrapper operations across one or more selected agents, including older eligible unmapped operations. It derives candidates from pinned wrapper source, existing reports/evidence, policy and mapping facts. Group by request meaning, effects, results, errors and lifecycle; command-name similarity is insufficient. Landing alone does not qualify an operation: missing evidence remains visible and must be resolved for any selected integration claim.

Candidate dispositions are mapping to an existing shared capability, proposing a separately reviewed extension, intentional wrapper-only treatment, or explicit deferral with rationale and owner. These are routing decisions grounded in maintainer policy and approval records, not achieved-support labels. Optional pending/deferred candidates are neither verified shared support nor unsupported debt, and cannot excuse required wrapper work. Discovery prepares decisions; implementation requires an approved bounded integration packet. Once an addition is accepted and promised, affected later maintenance must preserve it.

The handoff binds selected operation/agent/mode/version/target identities; landed source/report/policy/evidence inputs; the existing mapping baseline and capability contracts/configuration conditions; disposition, rationale, owner and approval references; and bounded writes, due obligations, adapter proof, acceptance path and non-goals. Its relationship to other acceptance scopes is explicit. Reuse existing packet/evidence/review machinery; exact command names and schemas belong to the subsequent contract packet.

Before execution and final acceptance, revalidate relevant inputs for every selected agent and the shared mapping/contracts. Relevant changes require regeneration or established re-freezing and invalidate dependent evidence; no silent substitution of source, target or shared contract is allowed. Unrelated repository churn need not invalidate unchanged content-bound inputs. Stable candidate identities, semantic/input fingerprints and references to existing policy decisions/packets make unchanged rescans reuse open work rather than create duplicate tasks. Older and deferred candidates remain visible, with relevant changes triggering reconsideration; no separate mutable status ledger is introduced.

Single-agent discovery or selection is permitted. Advertising still requires approved scope, actual adapter evidence and existing capability/publication checks; matrix audit alone is not behavioral proof. Preserve the [onboarding charter's capability promotion rule](../../specs/cli-agent-onboarding-charter.md): non-allowlisted new universal capabilities require at least two lifecycle-eligible backends. Preserve backend namespaces until cross-agent semantics are proven under the [capability schema](../../specs/unified-agent-api/capabilities-schema-spec.md). This routing change does not relax approved onboarding minimum integration, integration profiles, or canonical OpenCode run requirements.

### Concrete change surface for later implementation

Paths in this table are existing entrypoints unless marked proposed. Referenced module directories include their affected models/renderers/validators; this is an implementation map, not permission to edit every file there.

| Concern | Required changes and source locations |
| --- | --- |
| Normative policy | Proposed common `docs/specs/support-depth-contract.md`; [registry contract](../../specs/agent-registry-contract.md), [registry data](../../../crates/xtask/data/agent_registry.toml), [registry implementation](../../../crates/xtask/src/agent_registry.rs). Define schemas, ownership, activation and references. |
| Legacy coverage and schema versioning | [Generator contract](../../specs/codex-wrapper-coverage-generator-contract.md), [scenario contract](../../specs/codex-wrapper-coverage-scenarios-v1.md) prospective qualification amendment; per-agent `SCHEMA.json`, `VERSION_METADATA_SCHEMA.json`, `RULES.json`, `VALIDATOR_SPEC.md` under [cli_manifests](../../../cli_manifests). Preserve historical labels and inactive formats. |
| Shared derivation | Proposed `crates/xtask/src/support_depth.rs`; [manifest_report](../../../crates/xtask/src/manifest_report.rs) and [report modules](../../../crates/xtask/src/manifest_report), [manifest_validate](../../../crates/xtask/src/manifest_validate.rs), [version metadata](../../../crates/xtask/src/manifest_version_metadata.rs) and [metadata modules](../../../crates/xtask/src/manifest_version_metadata). Evaluate all observed/claimed units, including covered flags/args and unsupported commands. |
| Maintenance contract and preparation | [Request contract](../../specs/maintenance-request-contract-v1.md), [maintenance modules](../../../crates/xtask/src/agent_maintenance): `prepare.rs`, `prepare/from_request.rs`, `request.rs`, `request/{raw,validate,support_surface_audit_reconcile}.rs`, `support_audit.rs` and helpers, `audit_status.rs`, `contract_policy.rs`, `docs.rs`. Add frozen requirements, separate mapping obligations, reconciliation and renderer truth; derive routing and required writes from authoritative obligations, retaining shared repairs and excluding optional candidates from maintenance due-work totals. |
| Maintenance execution/closeout | Same maintenance directory: `execute.rs`, `execute/{packet,types,validate,runtime}.rs`, `prepare_closeout.rs`, `closeout/{types,findings,validate,support_audit_truth,render}.rs`. Version strict schemas, carry receipts, enforce write boundary, independently rederive current qualification and preserve existing CI-association semantics. |
| Additive shared integration | Proposed cross-agent xtask entrypoint/workflow (names deferred). Reuse the shared evaluator and packet/evidence/review machinery; bind candidate discovery and selected handoffs to policy, landed inputs and mapping truth. Reuse [capability matrix](../../../crates/xtask/src/capability_matrix.rs), [capability audit](../../../crates/xtask/src/capability_matrix_audit.rs) and [capability publication](../../../crates/xtask/src/capability_publication.rs), including approval and universal-promotion gates. YAML remains transport-only. |
| Onboarding/runtime | [Charter](../../specs/cli-agent-onboarding-charter.md), [operator guide](../../cli-agent-onboarding-factory-operator-guide.md), [approval artifact](../../../crates/xtask/src/approval_artifact.rs), [onboard modules](../../../crates/xtask/src/onboard_agent), [runtime modules](../../../crates/xtask/src/runtime_follow_on), [prompt template](../../../crates/xtask/templates/runtime_follow_on_codex_prompt.md). Carry approved policy through descriptor, approval and runtime input without changing integration-profile meanings. |
| Lifecycle evidence continuity | [Lifecycle entrypoint](../../../crates/xtask/src/agent_lifecycle.rs) and [validation](../../../crates/xtask/src/agent_lifecycle/validation.rs); [evidence bundle](../../../crates/xtask/src/runtime_evidence_bundle.rs), [evidence runner](../../../crates/xtask/src/runtime_evidence_run.rs), [repair](../../../crates/xtask/src/repair_runtime_evidence.rs), [publication preparation](../../../crates/xtask/src/prepare_publication.rs) and its runtime-evidence module, [publication refresh](../../../crates/xtask/src/publication_refresh.rs), [proving closeout preparation](../../../crates/xtask/src/prepare_proving_run_closeout.rs) and proving validation. Repair/publication cannot silently switch policy or evidence. |
| Published consumers | [Matrix contract](../../specs/unified-agent-api/support-matrix.md), canonical [README index](../../specs/unified-agent-api/README.md) and dependent operator publication docs updated together, [matrix modules](../../../crates/xtask/src/support_matrix) derive/consistency/publication and affected root-intake models, [current matrix](../../../cli_manifests/support_matrix/current.json), its Markdown projection, [runtime data](../../../crates/agent_api/src/runtime_support_data.rs). Preserve the [runtime-support contract](../../specs/unified-agent-api/runtime-support-contract.md) and [public model](../../../crates/agent_api/src/runtime_support.rs): validate unchanged version-only behavior in existing public tests; adding depth fields would require a separate API/compatibility decision outside this proposal. [Capability audit](../../../crates/xtask/src/capability_matrix_audit.rs) continues checking shared claims separately. |
| Transport/admission | [Lifecycle spec](../../specs/unified-agent-api/acquisition-maintenance-lifecycle-spec.md), [open-PR workflow](../../../.github/workflows/agent-maintenance-open-pr.yml), [acquisition workflow](../../../.github/workflows/parity-acquire.yml), [promotion workflow](../../../.github/workflows/parity-promote.yml). Invoke shared validation/admission; no family or exception policy in YAML. |

Generated surfaces affected include `maintenance-request.toml`, HANDOFF and scope/review docs, execution prompt and PR summary, prepared `input-contract.json`, `codex-prompt.md`, `validation-report.json` and run summaries; onboarding `approved-agent.toml`, `lifecycle-state.json`, `publication-ready.json` and proving closeout. The separate additive workflow adds a derived candidate report and selected integration handoff using the same bindings. Update their producers and strict loaders, not hand-edited snapshots alone.

## 6. Migration and OpenCode boundary

At activation, enumerate existing declarations and known limitations by agent/version/target and relevant content. A legacy qualifier applies only to that bounded inventory. It is not a family-wide exemption and cannot swallow newly discovered units. New operations, newly required modes/values, materially changed behavior and explicitly selected retrofit work acquire due obligations. A selected retrofit includes a bounded requirement and target/version scope, not an instruction to requalify everything.

### Legacy carry-forward across releases

Adoption explicitly authorizes a content-bound legacy qualifier to carry from the activation version into later upstream versions for the same agent, operation/mode and target. This is a migration rule, **not** reused qualification evidence or a debt grant. Exact-version evidence and debt authorization rules remain unchanged.

For each activation row, freeze a stable operation ID, exact associated surface identities/modes, explicit target set, and a canonical comparison fingerprint. The fingerprint covers the declared request/value subset, observed relevant upstream grammar/defaults/effects/output/lifecycle facts, promised wrapper contract and relevant wrapper source/dependency contents. Record unknown upstream facts explicitly as unknown; a legacy comparison never claims that unknown behavior was tested or that an unchanged help name proves runtime compatibility. The later contract packet must define the canonical serialization and extraction rules before activation, including exclusion of nonsemantic acquisition timestamps and version strings from this comparison (they remain in evidence identity).

For each new version, derive a per-target carry-forward record linking the original activation row and fingerprint to the newly acquired facts. Carry-forward is allowed only when exact identities/modes and target membership match, the comparison fingerprint is equal, and no newly discovered fact contradicts the old promise or indicates materially changed behavior. An unchanged unknown field stays visibly unassessed; new information replacing an unknown field requires reassessment. No target expansion, name-only match, renamed surface, new mode/value or new required policy obligation can inherit the qualifier. New flags/arguments receive their own due obligations even if their parent operation carries forward.

A changed or unavailable comparison input produces `legacy_reassessment_required`, blocking affected execution readiness until classified under the frozen policy or explicitly re-frozen by the maintainer. It must not silently turn the whole agent into a retrofit or silently extend the exemption. Proven nonsemantic observation churn can retain the qualifier only by a recorded maintainer-approved comparison rule; materially changed behavior or selected retrofits become due within their affected scope. Removal must be evidenced as removal, not inferred from a help omission. Each report retains the activation anchor, old/new fingerprints, target/version and carry/reassessment reason; no executor may expand the activation inventory.

Required next-version cases: V+1 with equal comparison inputs on an already listed target carries the qualifier as unassessed without reusing V's verification; V+1 with a new flag, mode, value requirement or changed behavior creates affected due work; a new target cannot inherit; missing/ambiguous inputs or newly learned semantic facts require reassessment. These cases govern both audit and promotion deterministically, without broad next-release qualification by default.

The activation baseline does not authorize false claims or regressions: discovered contradictions to an existing promised contract still require the existing defect process. Where policy newly asks for stronger behavior than the old contract, report the gap as unassessed/selected follow-up according to activation rather than retroactively failing completed work. Maintain visible follow-up ownership for known limitations; do not let an unbounded legacy default become permanent hidden coverage.

Preserve distinct dispositions:

- **Approved finished forwarding:** qualified for its deliberately bounded contract, possibly still labeled passthrough.
- **Insufficient depth:** implementation exists but a due dimension fails or lacks evidence.
- **Unsupported debt:** unavailable behavior carried under valid existing target/version authorization, never verified support.
- **Direct TUI exclusion:** explicit invocation-mode exclusion with rationale and visible accounting.
- **Unknown/unclassified/unverified:** no qualification claim until the missing decision or evidence exists.

Existing debt authorization remains exact target/version and cannot be widened by packet execution. Semantic debt extensions, if required by schema changes, must identify affected obligations explicitly; name-level historical debt cannot automatically defer every new semantic requirement. An executing packet cannot defer newly discovered non-TUI gaps because they are numerous or inconvenient. A blocked classification or architectural seam returns through existing authorized scope/deferral rules; it does not manufacture blanket debt.

The [OpenCode wrapper contract](../../specs/opencode-wrapper-run-contract.md) deliberately centers canonical JSON run and excludes helper dependencies. The [backend contract](../../specs/opencode-agent-api-backend-contract.md) cannot broaden that boundary. A later additive wrapper amendment must reconcile the [CLI manifest contract](../../specs/opencode-cli-manifest-contract.md) and [onboarding evidence contract](../../specs/opencode-onboarding-evidence-contract.md), define selected helpers independently, and preserve canonical-run completion. No automatic shared API expansion follows.

The external OpenCode 1.18.31 packet was inspected separately at `ef73c3538d4e65854d43c7cb734a8b881fece88d`: 481 total required units (60 commands, 394 flags, 27 arguments): 473 new surfaces and eight existing debt identities not yet reauthorized. Those eight are included in the 481, not additional units. These are packet accounting units, not 481 API methods; they are not in the consulted staging bundle, whose included request targets 1.14.49. Refresh that packet before later execution. Do not grandfather its unwrapped inventory as existing supported behavior.

Reuse [backlog](../../backlog.json) items as follows:

| Item | Proposed disposition |
| --- | --- |
| `uaa-0072` | Implement the recorded debt-operation ownership and separate policy/acquisition/execution bindings, including reconciliation and compound lifecycle tests. No runtime work is authorized by this draft update. |
| `uaa-0041` | Semantic requirements/value/arity/output contract and bindings. Universal automatic grammar extraction is outside the minimum phase. |
| `uaa-0042` | Include unsupported commands in semantic obligations and admission/publication; close this accounting hole as part of the new evaluator. |
| `uaa-0053` | Later OpenCode wrapper expansion. Amend scope language that makes shared-API suitability the sole wrapper boundary or equates unwrapped with authorized exclusion. |
| `uaa-0055` | Optional pattern-exclusion ergonomics; retain concrete accounting and unchanged authority. Not a prerequisite. |
| `uaa-0066` | Choose staged lifecycle acceptance: retain completed Codex/Claude work and assess OpenCode breadth after approved expansion. Do not invent exclusions/debt to declare the existing narrow wrapper broadly complete. |

## 7. Bounded delivery sequence and acceptance

1. **Decision and contracts.** Maintainer reviews this proposal; subsequent contract packet defines versioned schemas, resolved policy identity, qualification/admission rules and activation inventories. Settle exact receipt/dependency format and projection compatibility before implementation. No broad wrapper retrofit.
2. **Generic plumbing.** Implement shared evaluator/runner, registry binding, request freeze/reconciliation, all strict consumers, generated packets, publication and promotion admission together as bounded subpackets. Compatibility and negative tests must pass before activating production generations; no interval where policy activates before its final gate exists.
3. **Qualification pilot.** Exercise existing OpenCode canonical run controls with actual bound proof, plus synthetic fixtures representing other families to test policy machinery. Synthetic families do not certify nonexistent OpenCode APIs. Missing proof is a reported outcome, not a reason to expand scope silently.
4. **Subsequent product work.** Reopen the selected OpenCode wrapper scope and execute its separately approved expansion under the policy. Select Codex/Claude retrofits by consumer need and risk. Broad retrofits remain outside this proposal.

### Required implementation acceptance cases

| Gate | Positive and negative proof |
| --- | --- |
| Classification | Approved diagnostic forwarding qualifies; unknown family remains visible and blocks execution readiness; mutating/persistent option cannot inherit diagnostic exception; ambiguous override fails. |
| Accounting | Passthrough flags/args remain evaluated; unsupported commands reach audit and publication; mixed TUI/headless modes retain headless obligations; discovered units cannot hide in legacy inventory. |
| Values/requests | Accepted subset produces correct argv; invalid values/arity/repetition/conflicts reject before spawn where promised; JSON-only never claims all formats; dynamic model IDs are not falsely closed enums. |
| Evidence integrity | Missing test, zero selected tests, wrong target/version, stale source/fixture/build input, unsupported receipt producer and unbound success boolean cannot verify an obligation. Help-only cannot verify parsing; argv-only cannot verify lifecycle. |
| Runtime claims | Malformed/unknown output, nonzero exit, redaction, cancel/timeout, EOF/finality and reaping tested against the promised path. Persistent fixture tests readiness/shutdown/I/O ownership; launcher evidence cannot certify managed protocol. |
| Freeze/reconciliation | Acquisition under unchanged rules discovers a new obligation without changing policy identity; its second freeze binds the concrete set. Post-freeze acquisition changes invalidate downstream evidence. Delegated renewal and fully evidenced retirement preserve policy identity while updating grant/audit state and invalidating stale downstream receipts. New rows, broadened authority, or unsupported transitions refuse; maintainer changes explicitly re-freeze. OpenCode accounting remains 481 = 473 + 8. |
| Migration/mapping | Old labels/outcomes retained; next-version equal-input carry stays unassessed; changed/new inputs, new targets and ambiguous comparisons cannot inherit it; due regressions/new behavior remain visible; wrapper-only does not require adapter changes or capability advertisement. |
| Maintenance versus additive shared routing | Wrapper-only/additive-pending operations retain required wrapper proof without phantom shared deficits or new advertising. Existing shared regressions still block acceptance; moving tasks or deleting declarations cannot erase approved due obligations. Reject stale participating-agent or shared-contract inputs at execution and acceptance. Discover older unmapped operations, reuse unchanged candidates/open packets, and group by proved behavior rather than names. Single-agent selection cannot bypass approval, adapter evidence or existing universal-capability/publication gates. |
| Publication/admission | Per-target JSON/Markdown depth facts agree; the existing public runtime record remains version-only and matches committed validated pointers without claiming depth; truthful partial acquisition/publication works; every promotion entrypoint refuses due gaps and leaves pointers/marker unchanged; approved preexisting debt remains explicitly partial. |
| End-to-end continuity | Onboarding and maintenance fixtures carry one policy/input binding through execution, repair, audit, publication and closeout; stale or missing new-schema fields cannot disable an activated gate. |

Use existing xtask suites for maintenance prepare/audit/execute/closeout, target/version debt, onboarding/runtime/evidence repair/publication, and matrix derivation/consistency. Add evaluator/receipt tests near shared code. Wrapper suites supply only the proof their actual assertions establish. During implementation rerun affected suites and repository-mandated gates; this documentation draft does not claim those future tests pass.

### Completion boundary for this draft

Mandatory proof now: verify source references against the pinned baseline, distinguish current authority from proposed behavior, validate local Markdown links and whitespace, freeze the candidate hash, and perform the authorized bounded independent causal review. Record raw reviews, parent adjudication, fingerprints, causal lineage and P3/P4 advisories outside the candidate. CLEAN means no unresolved valid P1/P2 within this proposal boundary; it does not adopt policy or establish runtime conformance.

Non-goals now: changing normative contracts, running maintenance closeout/promotion, adding wrapper APIs, modifying workflow behavior, creating hundreds of debt rows, or committing/pushing this draft. The next maintainer review should decide the rules in section 1 and any narrower alternatives before a contract/implementation packet is prepared.
