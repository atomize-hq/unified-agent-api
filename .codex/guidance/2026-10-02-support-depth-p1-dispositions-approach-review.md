# ChatGPT Pro approach review — support-depth P1 dispositions and scope

- Date: 2026-10-02
- Mode: approach review, fresh conversation (no continuity reuse)
- Conversation: https://chatgpt.com/c/6abfdcf5-7c00-83ea-8d98-3d2efb7bb00f
- Thinking effort: Pro (5 of 5); model left on Latest (reported GPT-5.6)
- Spec target: new docs/specs/support-depth-contract.md plus amendments listed in the prompt
- Package under review: docs/agents/lifecycle/support-depth-policy-{proposal,research-decisions}.md
- Repo state at time of asking: staging f61534be; branch docs/support-depth-policy-proposal at
  2797b8d9 (P0 landed: proposal 690dac5d, companion b35b0e79, backlog c5f776b1; unpushed)
- Inputs: prose only. No repository bundle and no managed Project source set; Pro was told the
  prompt is its entire evidence base.
- Response: 34,841 chars extracted, 440s (reported "Worked for 7m 20s")
- Verdict: ADJUST, then proceed with P1. Architecture and all four dispositions kept.

Advisory only; verify against local project truth and authoritative docs; do not reduce scope
without user approval.

## Current approach submitted for review

P0 landed as a local commit. Four maintainer dispositions (D10 charter allowlist to 4+4, D11
maintenance activation before onboarding proof conditional on a writer/effect inventory, D12 one
exact generation for the pilot, D13 serialize publication writes first) were submitted together
with the planned P1 contract-amendment inventory, and four specific questions: disposition
consistency, whether writer enumeration suffices for D11, whether D13 discharges or only narrows
H5, and scope-inventory completeness.

## Verdict

ADJUST while keeping the architecture and all four dispositions. No pivot; do not reopen the
accepted P0 timing correction, do not restore the blanket both-paths-before-either barrier, and do
not introduce a transaction service or migration machinery. The required refinement is to convert
"inventory complete" and "writes serialized" into precise testable guarantees about every affected
acceptance effect, and to state the proof/enrollment/qualification/acceptance distinctions
explicitly.

Pro's stated evidence boundary: the review used the prompt alone. The source-inspection findings
supplied to it are accepted inputs, not inspections it performed. It did not have the repository,
the two Draft documents, the eight capability identifiers, the executable schemas, or the numbered
conformance tests, and explicitly declines to certify the writer inventory or source-level
consistency.

## Prompt sent

```text
We are already working on this project task.

Task/spec: Prepare "P1", a normative-contract packet that adopts a new "support-depth" policy for a Rust workspace. The workspace has per-agent CLI wrapper crates, a unified agent API crate providing a shared facade over them, and an xtask crate holding lifecycle automation (acquisition of CLI surface manifests, maintenance packets, onboarding of new agents, publication of a support matrix and capability matrix, and version-pointer promotion). The policy adds semantic support obligations ("does this wrapper actually implement this operation to a proven standard") alongside the existing name-coverage machinery ("does this command/flag appear in the manifest"). P1 writes rules only: no evaluator code, no runtime work, no enrollment of any agent, no pilot execution. Normative authority in this repo lives in docs/specs/** ; ADRs are subordinate supporting rationale.

Current approach: A research package ("P0") of two Draft documents plus one backlog item just landed as a local commit. The proposal defines the architecture: obligation templates per operation family; four logical bindings separating (Event) why/when a generation opened, (P) normative policy and delegated authority, (O) acquisition-derived concrete obligations, and (E) evolving execution/grant/evidence state; three evidence classes (help/discovery, controlled-binary mechanics, upstream output provenance); and a staged delivery order P1 contracts, P2 shared machinery plus full maintenance proof, P4-M maintenance enrollment, P3 onboarding proof, P4-O onboarding enrollment, P5 a later cross-agent workflow that promotes landed wrapper operations into the shared API. A hard constraint from the maintainer is "minimum necessary machinery": every proposed new field, artifact, registry, schema or approval step must name a concrete reachable failure it prevents that existing machinery cannot, and choose the smallest sufficient addition. A second constraint is that this is greenfield policy development, so no legacy inventory, grandfathering, semantic carry-forward ledger or migration engine is to be built.

Four maintainer dispositions were just made and are the direct input to P1:

D10 (capability allowlist divergence). Verified by inspection, not inference: the onboarding charter states that any new shared capability id, except an allowlist it then names with four entries, is only "promoted" once supported by at least two lifecycle-eligible backends, and that this is CI-enforced by a capability-matrix-audit command. The constant that audit actually reads is an eight-entry array: the charter's four plus four MCP-management capability ids. In the published capability matrix, the MCP "list" capability has two backends, the MCP "get" capability has exactly one backend, and the MCP "add" and "remove" capabilities are config/target-gated and have no row at all, though they are declared and tested in code. So the eight-entry constant is load-bearing today for the single-backend "get" capability, and prospectively for the gated write operations. Tightening the code to the charter's four would fail CI. Disposition: amend the charter's allowlist to the eight, recording the single-backend "get" and the gated write operations as the rationale.

D11 (path-specific activation). The earlier design required both the maintenance and onboarding lifecycles to be proven before either could activate. Disposition: permit maintenance enrollment (P4-M) after the maintenance chain alone is proven and landed, before independent onboarding proof (P3), conditional on a complete writer/effect inventory showing that every alternate entrypoint capable of affecting the relevant acceptance outputs either enforces the same fresh admission check or refuses the affected write. This explicitly includes onboarding and publication routes whose own lifecycles are not yet implemented.

D12 (enrollment recurrence). Disposition: the first pilot enrolls one exact generation (one frozen upstream version); the bounded successor-generation selection rule, which would re-materialize the same approved scope at each new release, is deferred until the first instance has run once.

D13 (final-write conflict control). Disposition: serialize the affected publication writes initially; per-writer compare-and-refuse/retry is a later optimization.

Planned P1 scope inventory: one new normative contract file for the common support-depth rules; amendments to the agent-registry contract (policy binding, ownership, path prerequisites, bounded enrollment, fail-closed authority resolution), the maintenance-request contract (frozen requirements, the four bindings, mapping obligations, debt-transition authority), the onboarding charter (carry approved policy through approval/descriptor/runtime, plus the D10 reconciliation), the wrapper-coverage generator contract and its scenario catalog (prospective qualification amendment, executable schema revisions), the support-matrix contract (depth assessment fields, enrollment/authority, per-target facts), the runtime-support contract (state explicitly that depth facts do not enter its version-only public payload), the acquisition/maintenance lifecycle spec (admission rules plus the writer inventory), and the per-manifest-root schema, rules and validator-spec files.

Project sources: none configured. This repository has no managed Markdown source set for ChatGPT Projects, and no repository bundle is attached to this message. Treat the facts in this prompt as the entire evidence base, and say so explicitly where an answer would require source you do not have rather than inferring it.

Evidence, blockers, constraints, and validation: Your own prior consultation on this package returned ADJUST with exactly one P2 finding: a timing contradiction in which the research companion made real-agent qualification a prerequisite to P4-O enrollment, contradicting the proposal's statement that enrollment can begin with implementation and evidence work outstanding once the path can enforce its rules. That was corrected with a one-line change, now reading: "Before P4-O, prove independent new-entry machinery in the isolated workspace. Before production acceptance, qualify the selected real agent's exact approved scope with its own upstream evidence." The maintainer accepts your prior review as sufficient for that one-line delta and is NOT asking you to re-validate it. The documents' own activation prerequisites for early maintenance activation are: adopted contract and executable schemas with explicit path and selection authority; end-to-end isolated maintenance proof through acquisition, execution, audit, manual closeout and separate promotion; every alternate entrypoint affecting those tuples either sharing the admission check or refusing the write; and landed enforcement with reviewed positive and negative gate evidence before the maintainer authorizes production enrollment. A recommendation in the companion ("H5") requires validating expected live policy, enrollment, input and output identities at the writer, serializing conflicting accepted writes or comparing-and-refusing under a proved ownership boundary, while explicitly disclaiming any assumed cross-process lock or new transaction service; its stated basis is that several distinct code paths write publication outputs, that a publication-refresh path plans mutations and restores snapshots after a failed gate, and that lower-level publication and version-metadata paths also write outputs. The companion states those observations justify enumerating every affected writer but do not establish a cross-process lock or atomic multi-file transaction. Validation reality for P1, verified by inspection: documentation changes are effectively ungated in CI; the integration gate is a hygiene plus flight-check target; the line-count cap applies only to Rust files and skips Markdown; there is no Markdown or link gate; a manifest-validate command is required before pushing if the per-manifest-root files are touched; and a numbered family of spec-conformance tests encodes some of the contracts being amended, so those amendments are not documentation-only in effect. Blockers and open items deliberately deferred: the exact pilot upstream version and target, and the identity of the genuinely-new agent for onboarding proof, are enrollment-checkpoint inputs rather than P1 inputs.

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Four specific questions, in addition to the general recommendation:

1. Do these four dispositions introduce any contradiction with the architecture described above, or with each other?

2. D11's safety now rests entirely on the completeness of a writer/effect inventory. Is enumerating writers from the current source tree a sufficient basis for that conditional, or does the conditional require a stronger property than an enumeration can establish? If stronger, what is the minimum additional property, consistent with the minimum-necessary-machinery constraint?

3. Does D13's serialize-first choice actually discharge the H5 freshness-and-conflict requirement, or does it only narrow it? Specifically, does serializing publication writes remove the need for a freshness re-check between a passing admission gate and the write, or does it leave that need intact while removing only the concurrent-writer case?

4. Is anything missing from the planned P1 scope inventory, given that P1 must also establish the proof-versus-acceptance predicate and the final writer/effect inventory?

Is the current approach the best way forward? Give a direct keep, adjust, or pivot recommendation; tradeoffs; what to keep/change first; inspections; validation; and pitfalls.
```

## Response

Recommendation: ADJUST, while keeping the architecture and all four dispositions

The current approach is the right direction. I would not pivot, reopen the accepted P0 timing correction, restore the requirement to prove both lifecycles before either can activate, or introduce a transaction service or migration machinery.

The adjustment is to make P1’s safety conditions more precise. D11 needs a non-bypassable admission property, not merely a list of writers. D13 needs a defined final-admission boundary, not merely serialized output writes. P1 also needs explicit distinctions between machinery proof, enrollment, qualification, and production acceptance, together with coverage of capability publication, closeout, and version-pointer effects.

Evidence boundary: this review uses only your prompt. The source-inspection findings you supplied are accepted inputs, not inspections I performed. I do not have the repository, the two Draft documents, the actual eight capability identifiers, the executable schemas, or the numbered conformance tests. I can assess the architecture and prescribe the necessary inspections, but cannot certify the actual writer inventory or the packet’s source-level consistency.

1. Do D10–D13 contradict the architecture or one another?

No inherent contradiction is apparent. Each disposition is compatible with the architecture, provided P1 preserves the following boundaries.

D10: Reconcile the charter to the eight-entry exception

Keep this disposition. It is an explicit normative decision to reconcile a verified discrepancy—not a claim that implementation automatically overrides the charter.

The rationale should distinguish the cases you supplied: the MCP “get” operation presently depends on the exception because it has one published backend; the gated write operations need the exception prospectively when they become represented under applicable configurations and targets. The two-backend “list” operation does not presently depend on that exception for its backend count.

The essential limit is that an exception to the two-backend promotion threshold is not an exception to semantic qualification, lifecycle eligibility, configuration or target gating, or publication truthfulness. Being allowlisted does not establish that an operation is implemented to the required standard.

Likewise, the absence of rows for gated operations must not be interpreted as either positive qualification or proof that no obligations exist. P1 should preserve or clarify the owning matrix contract’s treatment of gated facts without inventing unconditional availability.

D10 does not require executing P5 or promoting anything new. It reconciles the rule governing existing capability identifiers.

D11: Permit maintenance activation independently

Keep the path-specific activation decision. It removes an unnecessary dependency between the two proof chains without removing the need to protect shared acceptance outputs.

The distinction is:

The onboarding lifecycle need not be proven before maintenance enrollment, but any onboarding route that can affect maintenance-protected acceptance outputs must already be safely constrained.

Consequently, P2 cannot finish the maintenance proof by examining only maintenance-labelled commands. It must establish that shared publishers, lower-level writers, and relevant alternate routes either enforce the applicable admission rules or refuse the affected effects.

Sharing a gate implementation is insufficient when that gate checks the wrong lifecycle authority. An onboarding-originated action must not acquire maintenance authority merely because both actions eventually invoke the same publisher.

D12: Enroll one exact generation first

Keep this. Exact-generation enrollment is a useful reduction in initial policy surface.

P1 must define how a request matches the approved generation and what happens when it does not. A successor version must not inherit qualification, enrollment, or publication authority merely because it is the next release of the same agent.

One subtle boundary needs explicit treatment: absence of an exact enrollment match must not become a way to bypass protection of an already-enrolled result. A request for another version might be an unaffected ordinary operation, or it might replace an enrolled version pointer or rebuild a matrix containing enrolled facts. Admission must consider those effects, not just the request’s version.

Running the first instance once supplies evidence for a later recurrence decision; it does not itself authorize recurrence. The exact pilot version, target, and later onboarding-proof subject remain checkpoint inputs, not missing P1 inputs.

D13: Serialize affected publication writes initially

Keep serialize-first. It avoids initially having to prove separate conflict-detection and retry behavior for each writer.

But “serialize” must describe an enforceable ownership boundary covering the affected operations. It cannot mean only that the expected CI jobs usually run sequentially. Nor can it exclude lower-level publication, version-metadata updates, or snapshot restoration when those operations can change the same accepted result.

The staged order remains coherent:

P1 → P2 → P4-M → P3 → P4-O → P5, subject to the stated gates.

The important consequence is that P2’s maintenance work includes enough common enforcement—or explicit refusal—to make early P4-M safe. It does not include full onboarding implementation merely because onboarding has access to shared outputs.

2. D11 requires more than enumeration

An enumeration is necessary evidence, but a list of current writer functions does not establish the conditional. The minimum additional property is:

Every reachable operation within the supported lifecycle/publication authority boundary that can affect an enrolled acceptance result must either enforce the applicable fresh admission predicate before that effect, or refuse the effect before changing authoritative state.

This is complete mediation: no supported route to the protected effect bypasses the required decision. It does not imply a new service, registry, or security subsystem.

Inventory effects, then trace their writers

An entrypoint-oriented search alone is vulnerable to omissions. Start with the affected authoritative outputs and work backward to every operation that can alter their meaning. Then trace the known entrypoints forward and reconcile the two views.

For this packet, the inspection must consider at least the effect classes identified in your prompt: support and capability publication, closeout and acceptance facts, version metadata and promotion pointers, and restoration of publication snapshots.

The inventory must also account for indirect and aggregate effects. For example, a command nominally refreshing an unenrolled agent might regenerate an entire matrix and replace an enrolled agent’s facts. Classifying the command solely by its requested agent would miss the protected effect.

Similarly, a lower-level writer is not protected merely because its most common caller performs admission. Its other callable paths must be accounted for.

The smallest sufficient deliverable is an expanded section in the lifecycle spec

The inventory can live in the already-planned acquisition/maintenance lifecycle specification. It need not be another artifact.

For each affected effect, it should establish:

Inventory dimension	What it must establish
Effect and affected scope	Which authoritative fact can change, including aggregate and indirect changes
Reachable routes	Which entrypoints and lower-level operations can produce that change
Applicable authority	Which lifecycle path, enrollment, and policy govern the attempted effect
Enforcement disposition	The admission boundary that must apply, or the explicit refusal required
Conflict and recovery scope	Which other writes and compensating operations can interfere
Verification	Source references and existing or planned positive/negative conformance evidence

P1 must distinguish present behavior from required behavior. A final P1 inventory can identify a writer whose enforcement must land in P2. It must not describe that future enforcement as already proven.

This is not the prohibited legacy inventory. It inventories mutation paths needed to protect new policy enforcement; it does not reconstruct historical semantic obligations or grandfather agents.

What closes the argument beyond the list?

For a bounded revision, source inspection must establish that the relevant admission boundary is unavoidable on every supported route to the effect—or that bypassing routes refuse the effect. Tests then challenge that claim through the alternate entrypoints and failure paths.

The minimal continuing rule is:

Adding or changing a route capable of affecting a protected result requires updating the existing inventory and its conformance coverage before that route may perform the affected write.

That prevents the inventory from becoming a one-time historical list. It does not require a generated writer registry or static-analysis project.

Also state the authority boundary honestly. This need not prevent an administrator from manually editing arbitrary files unless the repository claims that protection. It does need to cover the normal commands, scripts, workflows, and lower-level interfaces that the supported lifecycle permits to publish those results.

Bottom line for D11: an effect-complete inventory plus demonstrated non-bypassable enforcement is sufficient for the bounded activation argument. Enumeration alone is not. Where a route cannot yet satisfy that property, refusing its affected writes is the minimum addition.

3. D13 narrows H5; it does not by itself discharge freshness

Serializing publication writes leaves the need for final freshness validation intact. It removes overlapping-writer behavior within the serialization boundary, but does not prove that a previously admitted plan remains authorized when applied.

There are three distinct failure modes.

Stale admission before serialized execution. A plan passes admission and waits. Another serialized operation completes first and changes relevant output state. The waiting operation then executes its stale plan. The writes did not overlap, but the second plan was no longer valid.

Changed authority or inputs. Enrollment, grants, policy applicability, or relevant evidence state changes after admission. Serializing only publication outputs does not necessarily serialize those changes.

Stale compensation. A publication-refresh route captures a snapshot, later fails a gate, and restores the snapshot. That restoration can overwrite a newer accepted result unless its ownership and freshness are covered too. Restoration is a writer, not harmless cleanup.

Define the final-admission boundary

P1 should require a sequence to this effect:

Before applying an acceptance-affecting write, the writer must hold the applicable serialization ownership and establish that the frozen bindings, current authority, relevant inputs, and expected output state still authorize the proposed effects. That authorization must remain valid through the affected write and applicable failure handling.

This is a logical requirement, not an instruction to build a lock service.

It also does not require rerunning every expensive evaluation immediately before writing. Expensive acquisition, execution, and evidence assessment can happen earlier. Their results can be reused when their relevant identities are immutable and the final check establishes that those are still the authorized inputs.

The final check must not silently substitute the latest policy for the frozen policy binding. Freshness means that the frozen selection remains authorized under the current authority rules. A policy change must follow the specified transition behavior, not opportunistically reinterpret the generation.

A second check is unnecessary only when the first check already has the required boundary

There is a legitimate optimization:

Admission performed inside an exclusive boundary, with all relevant invalidating changes excluded until the effect completes, can itself be the final admission check.

Under that condition, checking twice adds nothing.

But “publication writes are serialized” does not establish that condition. P1 must say which facts can invalidate admission and how their validity is protected during the final write. Some may be immutable bindings. Others may need to participate in the existing ownership protocol.

Because you have not established a cross-process lock, P1 should not name one as an existing guarantee. P2 must demonstrate the actual supported boundary. If that boundary cannot cover a particular route, the route must refuse the affected write rather than relying on scheduling convention.

Serialize the affected operation, including its failure behavior

Serializing individual file writes is too weak when a publication operation has a read/plan/write/restore lifecycle. At minimum, P1 must define the protected operation and when its ownership ends.

That does not require holding exclusive access throughout acquisition or agent execution. It does require preventing a stale plan or stale restoration from becoming authoritative.

Finally, serialization is not crash atomicity. P1 should require that partial publication cannot be mistaken for completed acceptance, but it should not prescribe a new atomic multi-file transaction system without first inspecting the existing closeout and promotion protocol. Reuse existing completion or acceptance machinery wherever it already prevents that failure.

Bottom line for D13: it removes the need for an initial per-writer conflict/retry design. It does not remove H5’s live-identity validation or the need to preserve authorization across the final effect.

4. What is missing from the planned P1 scope?

The proposed file inventory is substantially right. The gaps are mostly explicit contract content and owning-document coverage, not a need for more new files.

A. A proof–enrollment–qualification–acceptance model

The common contract should define these distinctions directly:

Concept	What it establishes	What it does not establish
Path machinery proof	The lifecycle can enforce the adopted rules through its required isolated end-to-end chain	Qualification of the selected production agent
Enrollment	Authorized application of the policy to the exact approved generation and scope	Completion of implementation or evidence work
Qualification	The selected operation scope satisfies its required semantic evidence standard	Automatic closeout, publication, or promotion authority
Production acceptance	The applicable qualification and lifecycle requirements are satisfied for the exact currently authorized effect	Authorization for another generation, target, scope, or future release

These are logical distinctions. They need not become four new status fields.

For maintenance enrollment, the readiness predicate must include the prerequisites already adopted: contracts and executable schemas, explicit path and selection authority, the full isolated maintenance chain, alternate-route enforcement or refusal, and landed enforcement with reviewed positive and negative gate evidence.

For production acceptance, the predicate must additionally require the selected real agent’s exact approved scope and its own required upstream evidence, plus the applicable closeout and separate promotion authority.

This preserves the accepted timing correction: enrollment may begin with implementation and evidence work outstanding; acceptance may not pretend that outstanding work is qualified.

There is an important effect distinction here. Publishing a truthful pending or failed assessment is not the same as publishing qualified support or advancing a supported-version pointer. P1 should not accidentally require successful qualification before recording its absence. Different effects need different admissibility conditions under the same authority model.

B. Explicit capability-matrix and capability-audit coverage

The support-matrix contract is named in the scope inventory. The capability matrix and its audit are not as clearly named, despite being central to D10 and shared-facade publication.

Inspect whichever normative documents own those outputs and amend or cross-reference them as necessary. They need to distinguish capability declaration, applicability under configuration/target gates, cardinality-based promotion eligibility, and semantic qualification.

Also preserve the maintenance/P5 boundary: maintenance must preserve or repair already-promised shared mappings, while the later cross-agent workflow handles selected additive promotion of landed wrapper operations. A generic matrix refresh must not become an implicit P5 path.

This may fit existing owning documents. It does not automatically justify a new capability-policy document.

C. Closeout, promotion, and version-pointer applicability

Stating that runtime-support’s public payload remains version-only is correct, but insufficient by itself.

P1 must also state that excluding depth fields from that payload does not exclude enrolled version-pointer changes from depth admission rules. Payload shape and write authorization are separate matters.

The owning rules for manual closeout, publication refresh and restoration, lower-level publication, version metadata, and separate promotion must align. If the lifecycle specification already owns all of these, keep the rules there and add precise cross-references rather than creating competing authorities.

D. Four-binding transition and amendment rules

Naming Event, P, O, and E is not enough. P1 must state what may change, who may authorize the change, and what existing conclusions that change invalidates.

The key protections are that acquisition cannot grant policy authority; evolving execution or evidence state cannot silently rewrite frozen obligations; prospective qualification amendments cannot retroactively relabel old evidence as meeting a new standard; and debt transitions cannot function as an undeclared policy waiver.

The three evidence classes must remain distinct. Help/discovery evidence and controlled-binary mechanics can support their respective claims, but neither is a substitute for the required upstream-output provenance.

Reuse existing identifiers, references, and immutable artifacts where they already establish these relationships. The four bindings do not imply four files, four registries, or a new ledger.

E. Exact-generation scope and fail-closed non-matches

D12 requires explicit treatment of subsequent versions, additional targets, changed scope, and operations that replace an enrolled result while not matching its exact enrollment.

P1 should leave genuinely unrelated unenrolled work under its existing regime. It should not fabricate depth assessments for it or introduce migration work.

But it must reject the inference:

“This request does not match an enrollment, therefore it can overwrite an enrolled acceptance result without depth admission.”

The effect-based inventory and admission rule should prevent that failure.

F. Contract-to-schema and contract-to-test impact

The planned per-manifest-root schema, rules, and validator-spec amendments are appropriate. The numbered spec-conformance tests must be an explicit part of the impact assessment too.

For each changed rule, identify its authoritative specification, executable representation, relevant conformance assertion or scenario, and implementation enforcement deferred to P2 or P3.

This need not become a permanent new register; a bounded packet section is sufficient unless existing machinery already provides the mapping.

Because P1 writes rules only, do not smuggle evaluator or runtime changes into “test alignment.” Equally, do not claim the packet is coherent while ignoring existing tests that encode incompatible contracts. Inspect those dependencies before finalizing the landing sequence. Where executable enforcement necessarily belongs later, applicability must remain explicitly unenrolled and non-accepting until that enforcement lands.

5. Inspections and validation
Inspect mutation reachability before finalizing the normative edits

The most valuable first inspection is a two-direction trace: from authoritative outputs backward to every writer, and from lifecycle entrypoints forward to their effects.

Include the publication-refresh snapshot restoration, lower-level publication, and version-metadata routes already identified in your evidence. Inspect direct invocation, caller assumptions, aggregate regeneration, and failure handling—not just normal successful execution.

For D10, copy the exact eight identifiers from the audited constant into the authoritative rule, verify that the audit uses that constant, and check the gated capability representation. The prompt supports the disposition but does not supply the identifiers needed to draft that amendment accurately.

Also inspect how the repository currently represents policy selection, grants, enrollment-like authority, immutable inputs, and expected outputs. That is the basis for deciding whether any new field is necessary.

Validate P1’s contracts without claiming P2’s implementation proof

Given the CI reality you supplied, a green integration gate is not sufficient evidence for this packet.

P1 validation should include the required manifest-validate invocation for touched per-manifest-root files; applicable existing conformance tests; executable schema and fixture checks; and a deliberate cross-document review of references, terminology, authority, and staged prerequisites. Markdown line-count and link checking must not be inferred from the current CI gate.

P1’s scenario catalog should establish the expected outcomes for the key activation failures:

Scenario	Required result
Maintenance is ready; onboarding attempts an unauthorized affected write	Refusal before the authoritative effect
An aggregate refresh nominally concerns an unenrolled tuple but changes enrolled facts	Admission applies to those affected facts
Relevant authority or output identity changes after an earlier passing gate	Stale effects are not accepted
Failed publication attempts snapshot restoration	Restoration cannot overwrite state it no longer owns
Exact-generation enrollment is reused for a successor	No inherited depth qualification or publication authority
Enrollment exists but required upstream evidence is outstanding	Truthful pending/failure reporting remains possible; qualification-dependent acceptance is refused

P1 specifies these behaviors and validates their contract/schema representations. Missing runtime tests and the end-to-end proof belong to the later machinery packets. No pilot execution is needed in P1.

The writer inventory should be finalized against an identified source snapshot for P1, then rechecked against the actual landed enforcement revision before P4-M. “Final for P1” must not mean permanently sufficient despite subsequent writer changes.

6. What to keep and change first

First, write the common acceptance semantics and the two safety invariants. Establish effect-based, non-bypassable admission and the final-admission serialization boundary before distributing fields across schemas.

Second, complete the source-grounded writer/effect inventory. Use it to determine the smallest enforcement required for early maintenance activation. Refusing an unready alternate route is often preferable to implementing its whole lifecycle.

Third, amend the owning specifications and executable schemas, with explicit conformance impact. Keep normative authority in docs/specs/**; retain rationale in subordinate material. Do not duplicate the common predicates independently across maintenance, onboarding, and publication contracts.

The tradeoff is favorable. D11 brings maintenance value forward, but it does not eliminate the cost of protecting shared outputs. D12 keeps the first authorization narrow, at the cost of another explicit decision for recurrence. D13 sacrifices some publication concurrency while reducing the number of conflict protocols that initially need proof.

If source inspection shows that safe early activation requires disproportionate new infrastructure, the fallback should be to delay P4-M or refuse the problematic route—not to weaken admission or build a broad transaction system.

Final judgment: ADJUST, then proceed with P1. The architecture and dispositions are compatible. The necessary refinement is to turn “inventory complete” and “writes serialized” into precise, testable guarantees about every affected acceptance effect, without confusing adopted rules, enrolled work, proven semantics, and authorized publication.
