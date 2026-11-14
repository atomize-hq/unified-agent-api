# Support depth freeze checkpoints and implementation readiness

Status — documentation decision and implementation requirements, 2026-11-13. The reporting
exception is conditional on authority that the current implementation cannot recover. The open
items below block activation. This packet supplies no implementation or executed proof.

The decision follows the maintainer-delegated consult of 2026-11-13,
`.codex/guidance/2026-11-13-chatgpt-pro-freeze-checkpoint.md`, supplied for this inspection as
`frz-guidance.md`. That advisory recommends option 3 adjusted. Local inspection determines the
limits recorded here. The normative rule is in the
[evidence contract](../../specs/support-depth-evidence-v1.md#authenticated-request-freeze-checkpoints-and-implementation-readiness).
The [original question](support-depth-onboarding-producer-freeze-question.md) retains the
characterization and deferred rollback case.

## Six inspections before amendment

Inspected on `docs/freeze-checkpoint-readiness` at `07f49a3f`, before drafting the amendment.
All code paths below are relative to `crates/xtask/src/`. Blocking means that today's mechanism
cannot supply the recommended exception's predicate. A missing predicate stays required in the
implementation slice. It grants no interim exception.

### 1. IN1 authority and retained history

**Blocking yes.** `depth_acceptance/integration.rs:14-81,148-215` runs owning admission with a
captured isolated context. `depth_acceptance/integration/binding.rs:57-97` authenticates the current tip through
that context, pins the local candidate/parent and checks admission or single-first-parent
observation. `isolated_maintenance/host.rs:72-125,175-186` independently observes repository ids,
protected tips, ancestry, candidate presence and protected registry identity.
`depth_acceptance/integration/generation.rs:105-153` reads old/new committed lifecycle and approval bindings and
calls `runtime_follow_on/depth/integration.rs:9-18` for monotonic progression. These checks supply
current invocation authority. They expose no retained host admission decision, check/run attempt,
approved IN1 implementation/workflow provenance or predecessor admission to the evidence evaluator.
`depth_evidence/verdict/github/progression.rs:15-51` authenticates lifecycle inputs at observed
matching host heads/checkouts, then compares only those sources by ancestry. An evaluator cannot
independently recover the recommended historical IN1 authority today. Its progression check reads
committed lifecycle data and does not authenticate a past IN1 decision. A higher-stage ancestor
with no producer run remains outside that comparison. The slice needs a bounded, independently
authenticated admission-history resolver covering every applicable attempt, reusing the shared
progression predicate. Open item 1 defines the authority choice.

In process, `isolated_maintenance/tests/fixture.rs:6-30,147-187` supplies fake repository, tip,
source and ancestry API observations. `isolated_maintenance/tests/bootstrap.rs:66-90` commits a candidate, calls the
real IN1 predicate, advances the fake protected tip and calls observation. The evidence host is a
separate fake API population (`depth_evidence/verdict/github/tests/host.rs:6-34` and
`depth_evidence/verdict/github/tests/fixture.rs:11-14,80-92`). It supplies constructed workflow/artifact/adoption data. Neither
fixture supplies historical IN1 check provenance or host admission attempts. The shared slice
must model those independently, including forged and unavailable authority. Phase C must supply
actual protected-policy decisions, app/workflow provenance, run/check attempts, predecessor and
protected lineage. A green fake tip alone cannot stand in for those facts.

The history resolver must verify commit/tree/blob object identities against its authenticated
anchors, enforce complete bounded discovery and refuse missing or inconsistent history. The
reverted unbounded walk over unauthenticated local Git objects supplies no admission authority.

### 2. Freeze-only changes

**Blocking yes.** MA2's writer derives P/O, binds the standing request, preserves entries/work,
and checks reporting effects (`agent_maintenance/prepare/depth.rs:36-68,138-228`). Its outputs are
the request and rendered packet (`agent_maintenance/prepare.rs:264-314`), depth record and support
JSON/Markdown/runtime projection. ON2 writes the record, lifecycle P/O object and executable
dry-run packet (`runtime_follow_on/depth.rs:245-290`, `runtime_follow_on/depth/packet.rs:42-98`), plus coherent
reporting publication (`runtime_follow_on/depth/reporting.rs:11-48`). IN1 re-derives onboarding P/O, checks the
enrolled parent, approval, retained execution, entries/work and lifecycle object
(`runtime_follow_on/depth/integration.rs:84-178`). It does not compare the entire candidate delta against an exact
freeze-only output plan. A candidate can combine a structurally valid freeze with implementation
changes without this check certifying a reporting-only context. The record-free bootstrap's
whole-tree allowlist (`isolated_maintenance/onboarding/checkpoint.rs:148-214`) shows a reusable
comparison pattern, but covers different transitions. The slice needs exact writer-derived
paths, bytes, modes, additions and deletions against the authenticated predecessor for both
freezes, and rejection of every additional behavior-affecting change. Open item 2 records how
that definition is to be established.

### 3. Consumer ordering

**Blocking no additional amendment blocker.** IN1's reporting generation checks and implementation
progression do not demand the candidate's own successful host admission
(`depth_acceptance/integration/generation.rs:9-99,105-153`). Current integration publication
selects authenticated verdicts for current maintenance acceptance entries, with a reporting
fallback otherwise (`depth_acceptance/publication.rs:283-349`). The producer builds a finite
fixture executable and collects cases (`depth_ci/collector.rs:302-322,414-468,512-517`), without
calling packet admission. `depth_ci/profile.rs:360-432` pins that invocation boundary. Existing
`.github/workflows/ci.yml:153-177,334-390` still lacks producer dependencies for preflight and validators, while
`.github/workflows/ci.yml:623-659` supplies the terminal 17-job gate. The slice must keep structural freeze authorization
independent of a waived execution receipt and keep execution classification independent of its
own admission success. Readiness authentication must not be added to a verdict that IN1 needs
to authorize that same readiness transition. CP-P5 still owns failure-preserving consumer
ordering after terminal `rust-tests`, including the recursive workspace-test leg, followed by
whole-workflow completion and the terminal gate before conditional MERGE.

### 4. Actual host checkout binding

**Blocking yes for receipt-free reporting classification.** Execution authentication already
distinguishes host head from actual checkout, permits only the exact head or a two-parent PR
test merge with that head second, and checks workflow/build/tree and closure separately
(`depth_evidence/verdict/github/profile.rs:118-158`). Population discovery fetches each host
attempt, then obtains that actual checkout from its authenticated execution witness
(`depth_evidence/verdict/github/population.rs:153-190`). `depth_ci/collector.rs:303-322,661-680` records HEAD and workflow SHA.
There is no equivalent reporting-checkpoint context proof when the substantive witness is
deliberately absent. Nominal head equality would lose the current protection. The slice needs
independent actual checkout, workflow, inputs and relevant source/context authentication for
each reporting attempt. PR test merges against changed bases, merge groups, dispatch inputs,
pushes and retries must each satisfy the sealed permitted relationship. Unsupported or missing
context stays unverified. Open item 3 records the required transport choice.

### 5. Maintenance discovery

**Blocking yes.** `depth_ci/profile.rs:48-54` selects an auxiliary maintenance fixture and the
actual onboarding report respectively. `depth_evidence/verdict/github/population.rs:134-146`
skips a missing profile record or a different Event/P/O before attempt enumeration.
`isolated_maintenance/tests/view_eval.rs:60-62,89-96` commits genuine MA2, then copies its record
to that fixture. `depth_evidence/verdict/github/tests/host.rs:9-14` subsequently installs and
commits producer inputs and serves one run. The connected positive therefore supplies no CI at
the genuine MA2 freeze. The real request and canonical versioned report already carry generation
bindings (`agent_maintenance/prepare/depth.rs:148-200`), so discovery can be repaired. The slice
must authenticate those bindings at actual MA2 before fixture insertion, retain its CI attempts,
and refuse unavailable required binding facts. Open item 4 records the binding alternatives.

### 6. Source-currentness path

**Blocking no additional amendment blocker found by source inspection.** The comparison uses the
independently obtained staging anchor at evaluation time (`depth_evidence/verdict/github/population.rs:50-89`)
and exact source objects (`depth_evidence/verdict/github/profile.rs:65-108`). It imposes no comparison between run start time
and source integration time. Thus an earlier implementation run can qualify at a later evaluation
once identical bytes are approved and remain current, subject to all other predicates. Reporting
implementation can reach IN1 without a qualified entry, as inspection 3 describes. Adoption is
independent and compares actual graph inputs and reviewed source (`depth_evidence/verdict/github/adoption.rs:30-99,145-154`).
An execution at S1 followed by a changed protected closure S2 still refuses. Publication deserves
an explicit guard here. `support_matrix.rs:43-46` names
`crates/agent_api/src/runtime_support_data.rs` inside the protected `crates` inventory, and both
freeze renderers include it. The current renderer embeds only `codex`
(`support_matrix/runtime_support.rs:10,98-129`), so the non-runtime synthetic subject's reporting
changes can preserve those bytes when the baseline is coherent. Later report/lifecycle files
outside `crates` can change subject to their own frozen-input/currentness rules. A later rewrite
of an in-closure fixture, runtime projection, dependency or workflow cannot be exempted. The
existing later-run positive uses typed lifecycle/entry projections
(`depth_evidence/verdict/github/tests/onboarding/checkpoints.rs:1-2,144-161`), so it does not prove
the actual ON3–ON5 writers preserve this closure. The slice must prove delayed approval of the
same execution bytes and byte-stable publication, and keep the S1/S2 refusal. Phase C must prove
the real integration/qualification sequence with those same properties.

## Decision and normative scope

Adopt option 3 adjusted as a conditional rule shared by maintenance MA2 before MA4 and the exact
onboarding ON2 freeze before ON2 implementation. Complete Event/P/O discovery comes first.
Only a positively authenticated exact freeze checkpoint may omit an execution receipt. All
other matching candidates require execution, including failures before any readiness admission.
A checkpoint contributes zero successful obligations, adoption or currentness. The contract
retains the existing verdicts, independent adoption and execution source approval.

The reporting exception seals checkpoint identity and allowed execution contexts. It leaves the
run/attempt population open to discovery. Readiness is the first execution-required protected
IN1 admission linked to that freeze and original generation, irreversible within that generation.
Applicable candidate progression must be checked against authenticated admission history even
when the higher-stage admitted ancestor had no producer run. A new freeze-labelled descendant
or another PR cannot reopen the exception. A retry of the exact historical checkpoint can retain
its reporting classification after readiness only with its original authorized context.

## Open implementation and authority items

These questions require resolution before a positive checkpoint exception can ship. Missing
authority leaves the generation unverified under the normative rule. The current exact-closure
characterization remains accurate until the shared implementation is installed.

1. **Retained IN1 authority.** Exact question — which independently recoverable host representation
   authenticates each freeze/readiness admission decision, its approved IN1 policy/workflow,
   check/run and attempt, predecessor admission and protected lifecycle history? Option A is
   bounded reconstruction from host check/run/attempt data plus authenticated immutable source
   and protected lineage. Option B is a protected-policy-emitted, host-authenticated decision
   payload retained through existing proof storage, with independently checked provenance and
   predecessor links. Both require exact format, discovery bounds, retention, conflict/revocation
   handling and freshness semantics before use. Option C keeps reporting exceptions unavailable
   until one of those mechanisms is specified and implemented. A candidate-authored record or a
   similarly named successful check supplies no option. This packet leaves A/B selection open.
2. **Exact freeze definition.** Exact question — will protected IN1 replay each freeze writer
   against the authenticated predecessor and compare its complete result, or validate a finite
   writer-derived path/structure policy with exact expected bytes and modes? Either option must
   account for the full outputs listed below and reject implementation/dependency/build/workflow
   changes, including changes hidden in packet or profile material. Current structural checks
   alone cannot authorize the exception. The slice must specify this shared policy before use.
3. **Receipt-free context.** Exact question — will existing attempt/job/log facts authenticated
   by approved workflow provenance establish actual checkout/context and absence of substantive
   execution, or is a reporting-only payload from the existing collector required? Both options
   must bind the eight authority facts below, retain outcome/attempt history and detect unexpected
   substantive execution. Neither permits inference from missing artifacts or nominal head.
   Choose this with item 1 so producers can report without waiting on their own IN1 admission.
4. **MA2 binding.** Exact question — should discovery derive the maintenance generation from its
   authenticated standing request and canonical versioned depth report, or should MA2 atomically
   emit the existing producer profile binding? Either option must retain an independently known
   same-generation freeze when the auxiliary fixture is absent. The latter must also account for
   the fixture's location inside the protected execution closure. Later fixture insertion is
   insufficient. The slice must implement and prove the chosen binding on genuine MA2.

## Shared implementation slice

This is the immediately next shared amendment/implementation work after slices 0–3 and 5,
alongside slice 4, and before slice 6. Slice 4 may proceed independently. Slice 6 depends on both
repair and the completed shared classifier/authority work. Remaining slice ids keep their order.
Resolve the authority/context choices above first, then implement one shared classifier inside
complete population evaluation. This work includes the deferred rollback-over-unobserved-history
case. It must finish before treating slice 5's producer positive as a genuine historical-freeze
positive or starting phase C.

### Write scope and dependencies

- Shared private IN1 authority/history and exact freeze-delta checks in
  `depth_acceptance/integration*`, `runtime_follow_on/depth/`, `agent_maintenance/prepare/` and
  `isolated_maintenance/`. Reuse owning predicates and `check_progression`. Define the retained
  authority representation through open item 1 before introducing serialized fields.
- Population/classification/context handling in `depth_evidence/verdict/github/`, the existing
  evaluation population representation, and focused `depth_ci/` collector/profile/archive paths
  as required by open items 3–4. Every applicable row retains identity, outcome, classification
  and authority/reason. Reporting rows carry no successful execution mapping.
- Connected maintenance/onboarding fixtures and focused tests in the existing depth evidence,
  IN1, freeze and isolated-maintenance suites. Update Annex A with measured proof boundaries in
  that implementation PR. Canonical workflow scheduling/host activation belongs to phase C.
- Dependencies are the evidence amendment, resolution of open items 1–4, existing IR-P1/IR-P3 and
  CP-P5, and slices 0–3 and 5. ON3–ON5 writers and slice 10's gate work retain their own scopes.
  Receipt/record revision 1, eleven obligations, cases and finite synthetic selection still govern.

### Exact freeze material to validate

Derive paths from the existing writer plans. MA2 owns the standing
`docs/agents/lifecycle/<agent>-maintenance/governance/maintenance-request.toml`, its rendered
preparation packet inventory (`agent_maintenance/docs.rs:353-488`), and
`<manifest_root>/reports/<version>/depth-record.json`. ON2 owns that versioned report, the
onboarding `governance/lifecycle-state.json` P/O object and the seven dry-run files selected by
`runtime_follow_on/depth/packet.rs:62-80` under `docs/agents/.uaa-temp/runtime-follow-on/runs/<run_id>/`
(`runtime_follow_on.rs:59-70`). These are local preparation artifacts. The protected policy must
derive its permitted committed delta from the plan's committed outputs and authenticate actual
CI context, accounting for any packet files present in the admitted tree.
Both freeze plans include support JSON, support Markdown and the runtime projection named in `support_matrix.rs:43-46`, with acceptance forbidden. Registry, approval and acquired
inputs are authenticated prerequisites. They gain no freeze-write permission. Compare complete
rendered packet bytes and reporting structures so their executable prompt or profile content
cannot silently select substantive support execution. Authenticate the complete historical
source/context separately from current execution-source approval.

The authority for freeze and readiness must bind all eight semantic facts from consult section 2A.
These are required bindings whose serialized representation remains open.

1. Authenticated repository identity and original Event/P/O/enrollment.
2. Authenticated predecessor and its admission.
3. Exact admitted freeze or implementation head and tree.
4. Actual checkout and its permitted relationship to that head.
5. Complete source/context closure, including code, dependencies, workflow and inputs controlling
   reporting versus execution.
6. Independently approved IN1 policy implementation and workflow provenance.
7. Host admission run/check identity and attempt, with the actual decision.
8. Protected lifecycle history, preserving irreversible readiness and all subsequent progression.

### Proving tests

Parameterize maintenance and onboarding where their rule is shared. The following requirements
carry forward all of consult section 8. They are pending tests, with no new test result claimed.

| Positive or boundary | Required result |
| --- | --- |
| Reproduced defect | Genuine freeze head with matching Event/P/O and older closure stays visible with zero support credit. Later qualifying execution/adoption can verify the generation. |
| Real MA2 discovery | Serve CI at actual MA2 before delayed fixture insertion. Missing/mismatching auxiliary fixture cannot hide it. |
| ON2 split | Actual freeze is reporting-only with authenticated authority. Implementation is execution-required. |
| Freeze retries | Include attempt 1 and every retry, including one after readiness, with independently authenticated context and sealed membership. |
| Other PRs/events | Same-generation attempts across PRs, pull_request, merge_group, workflow_dispatch and push remain in one complete population. |
| Only checkpoints | Complete successful reporting-only attempts leave support unverified. |
| Source-approval timing | Earlier execution at S can qualify only after identical S is independently approved/current, without dropping that run. Prove the actual integration path and stable publication closure. |
| Noncircular ordering | Freeze structural admission and collection complete without waiting for their own packet admission. Readiness authentication cannot become its own prerequisite. |

| Refusal | Required result |
| --- | --- |
| Forged authority | Stage edits, copied admission files, wrong generation/enrollment/repository and similarly named green checks grant no exception. |
| Implementation hidden in freeze | Source, dependency, build configuration or workflow changes outside the exact permitted freeze delta refuse reporting classification. |
| Failure before readiness | Initial implementation failure, abort, missing witness or absent admission remains blocking after a later successful admission. |
| Backward progression | A higher-stage authenticated admission with no producer run still blocks a descendant rollback. A new freeze-labelled head after readiness, including another PR, cannot reopen the exception. Exercise both API listing orders. |
| Changed actual checkout | A freeze head test-merged against implementation changes cannot inherit historical reporting authority. |
| Changed dispatch/workflow context | Same source head with different inputs/workflow selecting substantive execution cannot inherit the exception. |
| Unexpected substantive execution | Passing and failing checkpoint execution both invalidate reporting treatment. Retain/authenticate evidence and counterexamples/conflicts. Passing output grants no checkpoint support credit. |
| Failed reporting producer | Failed/cancelled/nonterminal/aborted designated reporting or failed collection remains observable and prevents verification. |
| Historical execution closure | Execution at S1 followed by changed execution closure S2 still refuses exact source approval. |
| Missing/conflicting authority | Absent, expired, revoked, mismatched or conflicting authority, incomplete admission history or unverifiable object identities yields unverified without reopening freeze state. |
| Population incompleteness | Missing attempt 1/rows, expired required artifacts, changing totals, duplicates, pagination gaps, unsupported events and bounds still refuse. |
| Contradictory execution | Qualifying pass/fail preserves the existing unverified conflict result. A qualifying counterexample otherwise fails. Retries/new PRs erase neither. |
| Flow separation and scope | Maintenance cannot satisfy onboarding. Credential-free in-process tests keep production refusals and the absent live profile. |

The existing genuine-freeze characterization becomes the positive only when its actual authority
and reporting outcome are supplied. Retain its old source-mismatch assertion for execution-required
attempts. Retain the observed-source rollback regression and add the unobserved-admission case.
Connected positives must exercise real freeze writers and shared IN1 predicates before advancing
fake protected history. Explicitly model the new host authority facts instead of substituting
candidate-authored admission files.

## Phase C requirements

1. Resolve the disposable-repository arrangement and separately authorized test-only trust binding.
   IR-P1 already supports an authenticated isolated repository for this synthetic subject.
   Channel/CORE's canonical repository identity and production foreign-repository refusal remain
   controls. A checkout of the canonical repository and a distinct disposable repository require
   different host identities to be stated in the plan.
2. Push the genuine freeze so normal CI triggers, then implementation and publication. Independently
   discover the freeze run in the final complete population and prove all eleven obligations
   through execution and independent mapping adoption.
3. Recover actual protected IN1 authority, policy/workflow provenance, check/run attempts,
   predecessor and retained lifecycle history using the chosen representation. Prove a
   higher-stage admission with no producer run still prevents rollback.
4. Exercise actual PR/test-merge and merge-group checkouts, dispatch/push context and actual
   workflow provenance against the classifier's permitted relationships.
5. Rerun the historical freeze after readiness. Authenticate the new attempt without reopening
   readiness. Retain adverse initial implementation history with failure or missing witness
   despite later successful admission, retries or another PR.
6. Exercise pagination, attempt-specific retrieval, job/log and artifact association/availability,
   complete attempt counts and population-stability checks on host data.
7. Prove source satisfiability through actual implementation integration and publication with the
   original execution run retained. Any protected execution-closure revision stays subject to
   exact current source approval.
8. Complete CP-P5's canonical workflow prerequisites, registry-derived per-root validation, full
   history, accepted origin and authorized credential references. Execute real shared/subprocess
   gates from slice 10, including the recursive workspace-test leg, failure-preserving consumers
   after terminal producers, same-run IN1, and terminal workflow/direct checks/gate before native
   conditional MERGE. Review scheduling/latency in ordinary CI. Post-tip observation remains due.

Official host documentation supports attempt-specific retrieval and historical rerun identity.
GitHub's branch protection can require an expected app as check source, and merge queues test
combined changes against the current base. These host facts inform the required authentication
work. Local implementation of that work remains open. Sources checked 2026-11-13.
[Workflow attempts](https://docs.github.com/en/rest/actions/workflow-runs#get-a-workflow-run-attempt),
[reruns](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/re-run-workflows-and-jobs),
[protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches),
[merge queues](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue).

## Adaptation of the consult

The section-6 draft is incorporated as a conditional normative rule. The inspection does not
support treating current IN1 as an independently recoverable admission-history service, and no
serialized admission fields are invented here. Open items 1–4 are prerequisites to activating
the exception. Successful readiness cannot exempt unsuccessful earlier implementation attempts.
The complete test matrix and phase C scope are retained. Existing single-staging execution
approval still refuses differing historical execution closures.
