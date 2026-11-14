# Support depth freeze checkpoints and implementation readiness

Status — round-2 mechanisms specified; FR1 implemented for the credential-free isolated route.
FR1 supplies canonical protected-IN1 decision publication and independent bounded admission
history, including progression over admitted ancestors without producer runs. FR2/FR3 and the
remaining connected writer proofs are pending. Production/live activation requires phase-C
emitter-isolation and transport qualification. See the [FR1 report](fr1-admission-history-report.md)
for schema, policy identity, proving-test coverage and measured validation.

The decision follows the maintainer-delegated guidance
`.codex/guidance/2026-11-13-chatgpt-pro-freeze-checkpoint.md`, including its
“Second consult (2026-11-14)”, supplied as `frz-guidance.md`. The second consult resolves the four
mechanisms for option 3 adjusted. Local inspection determines the implementation gaps recorded here. The normative rule is in the
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
progression predicate. Decision 1 below selects the retained authority mechanism.

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
freeze output plan. A candidate can combine a structurally valid freeze with implementation
changes without this check certifying a reporting-only context. The record-free bootstrap's
whole-tree allowlist (`isolated_maintenance/onboarding/checkpoint.rs:148-214`) shows a reusable
comparison pattern, but covers different transitions. The slice needs exact writer-derived
paths, bytes, modes, additions and deletions against the authenticated predecessor for both
freezes, and rejection of every additional behavior-affecting change. Decision 2 below defines the complete-tree planner.

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
context stays unverified. Decision 3 below defines the context record and transport.

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
and refuse unavailable required binding facts. Decision 4 below selects canonical request/report binding.

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

Adopt option 3 adjusted as a specified rule shared by maintenance MA2 before MA4 and the exact
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

## Four mechanism decisions

These decisions adopt the second consult's mechanisms. FR1 implements admission publication and
history under the isolated route; complete freeze planning and reporting-context joins remain pending.
Missing authority yields `unverified`. FR1–FR3 must deliver positive isolated behavior as well as
refusals. Production/live activation remains closed until phase C qualifies the real emitters
and transport.

### 1. Retained IN1 admission authority

Choose option B, a canonical versioned IN1 admission record emitted by protected IN1. Bounded
reconstruction cross-checks the retained decision, its inputs and lineage. Re-evaluating current
observations cannot establish a past protected decision on its own.

The record contains schema version and record kind, canonical repository identity, original
Event/P/O/enrollment and generation identity, candidate and parent commit/tree identities,
actual checkout commit/tree, evaluated source/context and permitted host-head relationship,
old/proposed lifecycle state, checkpoint kind, decision and bounded reason codes. It also contains predecessor admission
identities, independently approved IN1 source/workflow and complete relevant policy/dependency
closure, observed protected roots/tips, accepted origin, registry anchor and other authoritative
inputs, a canonical digest of normalized independently authenticated producer and isolated-host
facts (including authorized adoptions and explicit discovery completeness/absence, with canonical
population ordering), consumed reporting-context/execution-evidence identities
where applicable, and host
workflow/run/attempt/job/check/event claims. An admission without a depth producer is representable.
The first record binds an independently authenticated bootstrap/enrollment anchor. Earlier
admissions must never be invented to fill a pre-mechanism history gap.

Protected IN1 emits after terminal producer observation under CP-P5, authenticated candidate,
predecessor, history and policy checks, owning predicates, shared progression and freeze planning,
and the required per-root freshness/tip loops. Required record publication precedes successful
admission completion. Retain explicit refusals when a decision is reached. Crashes, cancellation
or failed publication leave incomplete attempts. A refusal establishes no readiness or substantive
counterexample by itself. Admission records observed roots and a decision before integration.
Subsequent protected lineage is independently established after the ref update.

Both control-record kinds use deterministic canonical bytes and a format-specific domain-separated
digest. Publish through the existing proof transport. Emit a fixed-format, versioned commitment
with record kind, payload digest and returned artifact identity in that exact emitting job's
host-retained log. The evaluator resolves the approved workflow job through attempt-specific host
job enumeration, authenticates policy provenance and job/run/attempt/event identities, fetches
that job's log and artifact, and verifies the commitment and canonical payload. Same-run artifact
membership, a check name or aggregate workflow-log search supplies insufficient association.
Candidate-authored records and copied commitment text in another job fail authentication.

Admission-history discovery is independent of producer-population discovery. Starting at the
authenticated generation anchor, exhaust the relevant admission attempts and parent relationships
to candidate/protected roots, verifying commit/tree/blob identities against authenticated anchors.
Include higher-stage admitted ancestors without producer runs. Predecessor references cross-check
the full relevant history. The shared progression predicate consumes that history for every
applicable candidate, in either API listing order. Recheck roots and host populations before
accepting. An advanced fake protected tip alone supplies no admission decision.

History-gap detection covers the protected lineage and all candidate ancestors through every
parent to the authenticated anchor. A same-generation ancestor with a stage change against any
same-generation parent, or the first occurrence of that generation, requires its own authenticated
retained successful admission. The enrolled-or-earlier anchor supplies the bootstrap exception.
This applies to unmerged ancestors. Without the retained decision, a never-admitted stage edit and
a genuine admission whose run disappeared are indistinguishable, so verification refuses. Ordinary
ancestors preserving the generation and stage need no admission, and the pending candidate can receive its first
admission after this check. This preserves independent history and noncircular candidate evaluation.

Use v1 bounds of 1,000 runs and 100 attempts for each independently enumerated host stream,
1,000 unique commits per admission-history resolution and 64 KiB per canonical decision/context
record. Existing object/source/artifact bounds also apply. Exceeding a bound yields `unverified`
with complete-discovery failure. Required history is never truncated. Keep records immutable by
decision identity and separately authenticate retries. Conflicting decisions over identical policy
and authoritative inputs, missing predecessors, object mismatches, gaps, ambiguous/duplicate
commitments, stale roots or changed populations prevent verification. Distinguish decisions over
different root snapshots and observed host authority. Candidate-authored files cannot select the
observed-authority digest. Any unavailable evaluation read produces a retained
`observation_incomplete` refusal, excluded from identical-input conflict comparisons and readiness.
Reached contradictory decisions over identical normalized facts still refuse. Revocation changes present eligibility while preserving historical
readiness and execution accountability. Explicitly configure retention for records, logs and
provenance. Expiration or deletion of any required authority yields `unverified`. Verification
requires the evidence to remain available.

### 2. Exact freeze transition planning

Use a pure writer-derived transition planner shared with the real MA2 and ON2 freeze writers.
Approved code takes the authenticated predecessor tree plus any independently validated acquisition
delta, original request/approval and generation inputs, independently derived writer inputs,
retained entries/work and approved planner/policy identity. It returns the exact freeze
create/replace/delete/preserve operations with bytes, modes and object kinds. The complete expected
committed tree equals predecessor + validated acquisition delta + planned freeze delta.
Protected IN1 independently requires equality with the entire candidate tree, including every
preserved path. Genuine MA2 is an authenticated combined acquisition + freeze transition in its
existing single transport commit. Extra unrelated edits also lose reporting eligibility and remain
applicable.

The plan covers every writer effect listed below, including exact executable packet content,
reporting projections and the approved runtime renderer's actual output. Variable values such as
run ids and generated timestamps need explicit independently authenticated input sources bound by
the policy. Candidate-selected values cannot become authority merely by making equality pass.
Writer extraction must make those inputs explicit and test their provenance.

IN1 uses approved planner code over authenticated Git objects. It must never execute candidate
writers, generated packets or candidate local actions, consume candidate-produced expected-output
directories, or accept a candidate-selected policy version. Local worktree bytes cannot substitute
for the authenticated predecessor. Complete equality rejects hidden implementation, dependency,
build and workflow changes and arbitrary bytes inside permitted packet/profile/report locations.
Real writers materialize the same pure plan through their existing checked delivery boundaries.

#### Acquisition authority at the MA2 commit boundary

Inspection at `19d541ca` confirms that `parity-acquire.yml` generates acquisition outputs, stages
the exact inputs, re-freezes, publishes/validates, stages the complete delivery, audits, verifies
the checked bytes and creates one commit. The admission plan must reproduce that boundary.
Acquisition authority validates the acquisition delta before it becomes planner input. Freeze
authority then derives every remaining freeze output from the authenticated predecessor with that
validated delta applied. Complete-tree equality admits only those two deltas. The existing
transport continues to deliver their combined tree in one commit.
[Workflow](../../../.github/workflows/parity-acquire.yml),
[transport contract](../../specs/maintenance-request-contract-v1.md#preparation-delivery-transport).

The acquisition delta is bounded by the independently resolved `AcquisitionPlan.output_paths` for
the authenticated agent/version/targets and protected descriptor. Its `input_paths` enumerate
per-target/union snapshots, `wrapper_coverage.json` and `coverage.*.json`. `output_paths` additionally
names `versions/<version>.json` and `artifacts.lock.json`. Optional missing outputs retain their
authenticated absence. Metadata must match the existing `--status reported` generator and retain
the existing acceptance boundaries. Changed paths outside that exact set must be planned freeze
outputs or refuse. O's derivation also references `RULES.json` and `SCHEMA.json`. Both remain authenticated
predecessor prerequisites with their existing policy authority. Their inclusion in O grants no
acquisition-write permission. Packet, profile, source, dependency, workflow, registry and unrelated
changes cannot acquire authority through an acquisition label.
[Resolved paths](../../../crates/xtask/src/manifest_acquisition/plan.rs),
[O derivation](../../../crates/xtask/src/support_depth/acquisition.rs).

Existing checks supply the following acquisition constraints and byte bindings.

- O's `acquired_inputs` bind exact paths and SHA-256 digests for RULES, SCHEMA, union, target shards,
  wrapper coverage and coverage reports. Shared derivation validates their schemas, reconciles
  union/shards through `manifest_union::intake::validate`, and reconciles coverage reports and
  wrapper claims through `manifest_report::intake::validate`. Those semantic validators consume
  union/report timestamps and omit optional union raw-help conflict evidence from reconciliation.
  The finite bootstrap path additionally uses the existing `validate_bytes` variants and canonical
  encoding checks. Independently authenticated references or complete producer bytes are required
  to bind every byte, including timestamps, provenance and encoding, at MA2.
  [Derivation](../../../crates/xtask/src/support_depth/acquisition.rs),
  [union intake](../../../crates/xtask/src/manifest_union/intake.rs),
  [report intake](../../../crates/xtask/src/manifest_report/intake.rs),
  [finite bootstrap checks](../../../crates/xtask/src/isolated_maintenance/onboarding/inputs.rs).
- The workflow records archive SHA-256, size, version, target, asset, URL and applicable package
  provenance in the lockfile, checks npm's published integrity/shasum when applicable, and verifies
  the archive digest and size again on each target runner before snapshot capture. These checks
  constrain the downloaded assets. Validation of the complete lockfile must also authenticate its
  exact serialized contents and preserve predecessor rows and top-level content according to the
  existing acquisition writer. `manifest-validate` checks manifest/version schemas, required files,
  report/wrapper invariants, pointers and publication coherence. The workflow then runs
  `maintenance-audit-status` over the staged candidate before final transport verification.
  These validators supply no complete lockfile byte authentication.
  [Workflow acquisition checks](../../../.github/workflows/parity-acquire.yml),
  [manifest validation](../../../crates/xtask/src/manifest_validate.rs).
- OF-P4's maintainer statement supplies onboarding provenance through its exact path/digest/command
  set at the named protected integration ref. The dry run requires equality of every acquired
  input with both statement and ref; IN1 repeats the check against its pinned parent. Its authority
  covers exactly O's acquired-input set. Version metadata and lockfile are outside that set.
  OF-P4 introduces no maintenance statement requirement. A same-candidate statement cannot supply
  protected onboarding provenance.
  [Charter rules 13–14](../../specs/cli-agent-onboarding-charter.md),
  [statement parser](../../../crates/xtask/src/depth_record/acquisition.rs),
  [dry-run checks](../../../crates/xtask/src/runtime_follow_on/depth.rs),
  [IN1 checks](../../../crates/xtask/src/runtime_follow_on/depth/integration.rs).
- `maintenance_transport.py` captures acquisition fingerprints before freeze, checks staged input
  bytes, preserves checked delivery fingerprints through audit/commit and bundles present output
  bytes. The workflow uploads that existing checked bundle. This protects local delivery integrity.
  To use that bundle, independent admission must authenticate the acquisition job/run/attempt,
  approved workflow and inputs, and bind it to the candidate and predecessor through the
  retained-authority mechanism. Candidate files and self-asserted fingerprints cannot supply that
  association.
  [Transport implementation](../../../scripts/maintenance_transport.py),
  [bundle upload](../../../.github/workflows/parity-acquire.yml).

**Explicit FR2 prerequisites.** The current evaluator has no independent acquisition-job/bundle join
for MA2's newly acquired target snapshots, union, wrapper coverage or coverage reports. Their
candidate O references and semantic validators alone cannot authenticate complete acquired bytes.
`versions/<version>.json` and the entire `artifacts.lock.json` additionally lack O/OF-P4 digest
coverage. Metadata's schema/status checks cannot authenticate its timestamp, preserved notes and
validation fields or exact encoding. Lock checksums cannot authenticate the complete lockfile's
rows, ancillary fields and encoding. FR2 must bind these exact existing acquisition outputs through
authenticated acquisition observations or independently reproduce complete producer bytes from
authenticated inputs, including release metadata/timestamps and predecessor contents. It must also
establish exact presence/absence, modes and object kinds for every acquisition output. The present
bundle copies file contents and carries no complete absence/mode manifest. Missing authentication
for any of these facts refuses checkpoint admission. Acquisition grants no exception for extra
bytes or changes. These are prerequisites for the combined admission predicate, with the existing
workflow transport and single commit preserved.
[Metadata planner](../../../crates/xtask/src/manifest_version_metadata.rs),
[transport fingerprinting and bundle copy](../../../scripts/maintenance_transport.py).

ON2 has no analogous combined acquisition boundary under OF-P3/OF-P4. Direct input-only acquisition
and its reviewed integration-branch statement precede the dry-run freeze. Its authenticated
predecessor already holds those inputs, so the ON2 acquisition delta is empty. The freeze planner
must preserve them and refuse acquisition or statement changes carried only in the freeze commit.
[Charter rules 12–14](../../specs/cli-agent-onboarding-charter.md),
[OF-P3/OF-P4 decisions](support-depth-onboarding-freeze-decisions.md).

### 3. Reporting-context record and noncircular join

The existing collector emits a versioned reporting-only context record from a trusted dispatcher
branch. That branch is selected before candidate-controlled code executes in the job. The record
contains schema version and reporting-only kind, canonical repository and original
Event/P/O/enrollment, host run/attempt/job/event/head/workflow and dispatch/input identities,
actual candidate-data checkout commit/tree and permitted head relationship, separately identified
approved collector/policy source and build/workflow closure, complete relevant source/context
observations and selected configuration, required indexed/worktree observations, selected mode,
an explicit observation that the enrolled substantive execution path was not invoked, reporting
outcome, and references/digests matching the candidate and predecessor. The trusted source may
inspect an older candidate tree as bounded data. Ordinary job metadata and successful step
statuses cannot replace this record.

The sequence is producer context publication, terminal producer, independent IN1 exact-transition
and context checks, IN1 admission publication, then consuming evaluator join with protected history.
The producer may reference an admitted predecessor. Its record cannot require its own future
admission's digest or success. The eight authority facts below belong to this combined proof.
The collector observes the selected dispatcher path and the absence of invocation of the enrolled
substantive path within its isolated runtime. Its claim is bounded to that observation.

Use the same canonicalization, 64 KiB bound and exact-job/attempt log commitment protocol as IN1.
Each retry needs its own authenticated context. Wrong checkout/tree, changed test-merge base,
wrong parent relationship, changed workflow/dispatch inputs, substituted payload, wrong job or
cross-attempt replay prevents the exception. Missing later IN1 authority yields `unverified`.
Substantive execution indications or receipts invalidate reporting classification for both passing
and failing outcomes. Preserve and authenticate those observations under the existing
counterexample/conflict rules. Failed, aborted, cancelled or nonterminal reporting and failed
collection prevent verification. A late historical freeze retry cannot clear execution failures.

### Trusted emitter isolation

Emitter isolation means candidate-controlled processes cannot modify, share or impersonate the
trusted emitter's runtime, inputs, observations or publication. Protected IN1 requires a clean,
separate VM-backed hosted job running independently approved policy and dependencies. Candidate
trees, records, packets and artifacts are bounded data. Candidate code, local actions, generated
packets, build scripts and candidate-produced executable caches stay outside that runtime.
The collector's trusted `rust-tests` reporting branch must run before any candidate-controlled
process and exclusively use trusted code through publication, with no concurrent candidate
processes. Separate processes, directories, later steps, approved workflow bytes, provenance and
signatures alone cannot provide this boundary against a same-user attacker.

Phase C must prove this separation in the real workflow layout. Today `ci.yml`'s `rust-tests`
executes `cargo test --workspace --all-targets` before collection and restores a Rust cache.
That order cannot establish reporting-emitter isolation. Preserve required tests through a reviewed
workflow arrangement. If the existing job convention cannot provide separation, phase C needs a
reviewed workflow-layout amendment before activation. A clean downstream IN1 cannot recover
runtime truth from a contaminated reporting emitter. CP-P5's read-only credential reference serves
host reads. This mechanism adds no private signing key or paid service.

### 4. Canonical MA2 generation binding

Producer collection and population discovery must share one resolver over the authenticated
standing request and canonical versioned depth report at each exact immutable host head. Validate
supported versions/structures, agreement on original request/approval, frozen P/O, enrollment and
agent identity and every existing binding field, and re-derive required values using approved
logic. Resolve one coherent binding, then enumerate every matching run attempt before examining
substantive artifacts. The auxiliary fixture's presence, absence or content has no generation
authority. Committed records supply authenticated input evidence through their Git identities.
Protected admission and the evaluator join supply checkpoint authority.

Coherent records identifying another generation receive ordinary out-of-generation treatment
when no authenticated association conflicts. A target request with a missing, malformed, stale
or contradictory report, or the reverse, yields `unverified`. Independently associated heads whose
required records were removed/replaced also yield `unverified`. Ambiguous report entries refuse.
Heads with no independent target-generation association retain ordinary unrelated-head treatment.
This avoids granting every repository head membership while retaining genuine MA2 before fixture
insertion. Producer witnesses and discovery must resolve the identical maintenance generation.
Maintenance and onboarding evidence remain separate.

## Ordered implementation sub-slices

Each sub-slice is a separately reviewable PR into the integration branch. After existing slices
0–3 and 5, land **4 → FR1 → FR2 → FR3 → 6 → 7 → 10 → 8 → 9 → 11 → 12 → 13**.
All three require this evidence amendment and existing IR-P1/IR-P3 and CP-P5 contracts. They retain
receipt/record revision 1, eleven obligations, existing cases and finite compiled synthetic
selection. Their new control records have independent versions. Every implementation PR updates
Annex A with its measured boundaries. Canonical workflow deployment belongs to phase C.

### FR1 — Admission record, publication and independent history

Delivered for the isolated route; live activation remains closed. Policy provenance uses runtime
Git tree/blob identities of the source-installed protected installation commit, checked against
that independently authenticated anchor. The fixed closure includes `crates`, Cargo manifest/lock,
`.github`, `scripts` (including the exact workflow and archive reader) and installed profile
scopes. Candidate records cannot select the protected anchor or substitute their own identities.
The Python proof transport only decodes one bounded regular `admission.json`; Rust owns canonical
record parsing, digest and authority checks. No compile-time repository hashing is used.
ZIP transport validates streamed data descriptors in signed, unsigned and ZIP64 forms against
actual payload length and CRC. The 64 KiB limit covers the new versioned control records.
Existing depth and lifecycle inputs retain their existing bounds. Git reads require exact object
types and bounded stdout. Producer applicability filters candidate roots before history resolution,
while admission discovery independently enumerates all retained attempts. Connected proofs drive
the hidden unmerged-ancestor refusal and a same-run late parent retry through real IN1 publication.
Fixed canonical bytes and domain digests pin retained identities.
The [report](fr1-admission-history-report.md) maps every FR1 proving item to executable tests.

- Write scope covers private IN1 decision capture/emission in `depth_acceptance/integration*`,
  the shared progression interface in `runtime_follow_on/depth/`, admission-history resolution
  in `depth_evidence/verdict/github/`, and `isolated_maintenance/` host/fixture support. Add the
  canonical record/parser and fake-host job/log/commitment/artifact protocol in those owning
  modules and focused existing IN1/evidence suites.
- Depends on slices 0–5 in the landing order above. Deliver the real IN1-predicate emitter,
  authenticated bootstrap/enrollment anchor, fake-host publication, bounded independently
  discovered admission history and progression over it. This closes slice 5's deferred
  rollback-over-unobserved-history case, including ancestors with no producer. FR2/FR3 add the
  exact freeze and reporting-context predicates to these interfaces.
- Proving tests accept an actual IN1 decision through independently modeled host facts. Refuse
  candidate JSON, same-name checks, same-run wrong-job artifacts, wrong-attempt commitments,
  missing publication/predecessors, object-hash mismatches, history gaps and every bound overflow.
  Test retries, conflicting records, revocation, changing roots/populations and first-record
  anchoring. A higher-stage admitted ancestor with no producer blocks descendant rollback in
  both API orders and on another PR. Fake tip advance cannot manufacture authority.

### FR2 — Shared exact freeze planner and IN1 complete-tree check

- Write scope covers pure planning extraction in `agent_maintenance/prepare*`,
  `runtime_follow_on/depth/` and their shared reporting renderer interfaces, private IN1
  freeze-only validation, and focused writer/IN1/isolated-route tests. Keep existing checked
  materialization and acceptance boundaries.
- Depends on FR1. Deliver the planner used by real MA2/ON2 writers and independently by IN1
  over authenticated predecessors and authoritative inputs. Resolve the explicit acquisition
  authentication prerequisites above. Admit MA2's combined acquisition + freeze commit through
  predecessor + validated acquisition delta + planned freeze delta equality. ON2 uses its already
  admitted acquisition with an empty acquisition delta. Cover every actual committed effect,
  byte, mode and object kind, with complete expected-tree equality.
- Proving tests compare genuine MA2 and ON2 outputs with complete planned trees and reviewed
  canonical packet/reporting/runtime output expectations. Two calls to the shared planner alone
  cannot establish canonical correctness. Refuse extra implementation/dependency/build/workflow
  edits, unrelated edits, unexpected packet/profile/report bytes, additions, deletions, mode and
  object-kind changes, and wrong predecessor/approval/Event/P/O/enrollment/policy identities.
  Identical authenticated inputs plan deterministically. Candidate expected-output files have
  no influence. Validate provenance for variable writer inputs.
- Prove the actual MA2 transport boundary with one commit containing acquisition snapshots,
  coverage, metadata, lockfile and planned request/record/packet/reporting outputs. Run the existing
  acquisition validators, authenticate the acquisition bytes and predecessor, then require real
  IN1 complete-tree admission and exact canonical freeze output. Refuse that same combined commit
  with an unrelated edit added, including an edit inside an acquisition-owned file that preserves
  schema/semantic validity but changes authenticated bytes. Refuse missing acquisition authority,
  substituted metadata/lockfile bytes, missing/extra files and altered modes/object kinds. Prove
  ON2's actual separate boundary over protected OF-P4 inputs and refuse same-commit acquisition or
  statement substitution.

### FR3 — Reporting context, canonical binding and population join

- Write scope covers reporting-context schema/emitter/dispatcher instrumentation in existing
  `depth_ci/` collector/profile/archive paths, canonical maintenance binding shared with
  `depth_evidence/verdict/github/`, population representation/classifier/evaluator join and
  independent fake-host observations, plus connected maintenance/onboarding tests in the existing
  evidence, IN1 and isolated-route suites.
- Depends on FR1 and FR2. Deliver context first and admission later, full retained population,
  genuine MA2/ON2 reporting positives with zero support credit, canonical binding for both producer
  and discovery, the refusal matrix below and source-currentness proofs available at this stage.
  Positive isolated classification must execute the completed mechanisms.
- Proving tests include genuine older freeze CI followed by qualifying execution/adoption, actual
  MA2 before fixture insertion, fixture absence/mismatch/mutation independence, target request
  with missing/mismatching report and the reverse, ambiguous binding, coherent unrelated heads,
  producer/discovery identity agreement and maintenance/onboarding separation. Refuse nominal
  head plus green steps without context, wrong checkout/base/parents/inputs, candidate records,
  wrong job, cross-attempt replay, substituted bytes and missing later IN1. Passing/failing
  substantive execution invalidates reporting mode consistently. Test every late retry separately
  and producer non-self-admission. Execute every positive/boundary and refusal row below.
- Prove evaluation-time source approval now. Earlier execution at S can qualify after identical
  S reaches protected staging with all other predicates satisfied and the earlier run retained.
  S1 followed by current S2 still refuses. Measure real MA2/ON2 rendering closure stability and
  existing available publication writers. Real ON3–ON5 writer closure stability is delivered
  in slices 6–9 as those writers connect.

### Connected-chain consumers in slices 6–9

Slice 6 consumes FR1–FR3 for controller-owned ON3 reporting E/files/publication, runtime selection
and IN1 reporting admission. Slice 7 adds onboarding owning predicates, exact binding/currentness
and publication/proving entry derivation. Slice 10 supplies read-only gates before slice 8.
Slice 8 delivers ON4 with its real publication writer, exact entry, post-entry evaluation and IN1.
Slice 9 delivers both ON5 writers and their owning IN1 predicates through actual preparation and
closeout. Each extends the preceding real-writer chain and compares the protected closure.
Typed lifecycle projections alone cannot establish ON3–ON5 writer stability. Slice 12 completes
the connected proof documentation after these incremental proofs. Later completion reports must
state in-process implementation/connection and pending phase-C host qualification at their actual
measured boundary. These are pending deliverables in this packet.

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
MA2's acquisition writes use the separately validated acquisition delta specified above, in the
same transport commit as the freeze delta. ON2 preserves its predecessor's acquired inputs.

The combined reporting-context, IN1 admission and protected-history proof must bind all eight
semantic facts from consult section 2A. The reporting payload precedes its own admission.

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
carry forward all of consult section 8 across FR1–FR3 and the later connected writer slices.
FR1's retained-admission authentication, bounded history and backward-progression tests are
implemented for the isolated route; the [FR1 report](fr1-admission-history-report.md) records
results and sandbox limits. Exact reporting-checkpoint classification, complete freeze planning,
reporting-context joins and the remaining connected writer proofs below are still pending.

| Positive or boundary | Required result |
| --- | --- |
| Reproduced defect | Genuine freeze head with matching Event/P/O and older closure stays visible with zero support credit. Later qualifying execution/adoption can verify the generation. |
| Real MA2 discovery | Serve CI at actual MA2 before delayed fixture insertion. Missing/mismatching auxiliary fixture cannot hide it. |
| Real MA2 combined commit | One transport commit containing authenticated acquisition and exact planned freeze passes complete-tree admission with zero support credit. |
| ON2 split | Actual freeze is reporting-only with authenticated authority. Implementation is execution-required. |
| ON2 acquisition boundary | Acquisition and OF-P4 statement are already admitted on protected integration before freeze. The freeze commit preserves them. |
| Freeze retries | Include attempt 1 and every retry, including one after readiness, with independently authenticated context and sealed membership. |
| Other PRs/events | Same-generation attempts across PRs, pull_request, merge_group, workflow_dispatch and push remain in one complete population. |
| Only checkpoints | Complete successful reporting-only attempts leave support unverified. |
| Source-approval timing | Earlier execution at S can qualify only after identical S is independently approved/current, without dropping that run. Prove the actual integration path and stable publication closure. |
| Noncircular ordering | Freeze structural admission and collection complete without waiting for their own packet admission. Readiness authentication cannot become its own prerequisite. |

| Refusal | Required result |
| --- | --- |
| Forged authority | Stage edits, copied admission files, wrong generation/enrollment/repository and similarly named green checks grant no exception. |
| Implementation hidden in freeze | Source, dependency, build configuration or workflow changes outside the exact permitted freeze delta refuse reporting classification. |
| Extra change in combined MA2 | Add an unrelated edit to the genuine acquisition + freeze commit. Complete-tree equality refuses, including altered authenticated acquisition bytes that still pass semantic validation. Missing metadata/lockfile or other acquisition-byte authority also refuses. |
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

## Fake-host proof boundary

The in-process proof must state that it establishes predicate, classification, binding and history
behavior against the modeled host protocol, authentication checks against supplied independent
fake-host observations, and real-writer/source-closure behavior within the isolated route.
Admission observations must be separate from producer artifacts. Fixtures invoke the actual IN1
predicate and model its publication before advancing protected history.

Real issuance by GitHub, exact real job/log/artifact association, deployed emitter isolation,
merge-queue/retry/retention behavior and credentials require phase C. The fixture assumes control
of its test process remains trusted. Control of that process permits fabrication of modeled
authority. Candidate inputs must fail to satisfy modeled authority when
independent facts are missing or contradictory. Use the existing compiled isolated route and its
trust binding. A committed record cannot select a trusted fake host or install a live profile.

## Phase C requirements

1. Resolve the disposable-repository arrangement and separately authorized test-only trust binding.
   IR-P1 already supports an authenticated isolated repository for this synthetic subject.
   Channel/CORE's canonical repository identity and production foreign-repository refusal remain
   controls. A checkout of the canonical repository and a distinct disposable repository require
   different host identities to be stated in the plan.
2. Push the genuine freeze so normal CI triggers, then implementation and publication. Independently
   discover the freeze run in the final complete population and prove all eleven obligations
   through execution and independent mapping adoption.
3. Recover versioned IN1 records and actual policy/workflow provenance, exact job/run attempts,
   job-log commitments, artifacts and predecessor/anchor identities through the specified transport.
   Prove a higher-stage admission with no producer run still prevents rollback.
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
9. Prove emitter isolation in the reviewed real workflow layout, including the trusted `rust-tests`
   reporting branch before candidate code and a clean separate protected IN1 job. Preserve every
   required test. Record any necessary workflow-layout amendment before activation. Attempt forged
   candidate payloads, copied commitment lines in wrong-job logs and misleading same-name checks
   and artifacts. Authenticate actual PR/test-merge/merge-group and dispatch contexts separately.
10. Prove missing-authority refusal for deleted/expired records, logs and provenance, failed
    publication, conflicting decisions, changed populations and stale roots. Configure retention
    explicitly and exercise bound exhaustion and historical attempts through exact-job retrieval.

GitHub documents job-specific logs and attempt-specific job enumeration. The ordinary run-jobs
endpoint defaults to the latest jobs, so historical authentication must deliberately select each
attempt. This supplies platform primitives for the proposed transport. Its deployment remains a
phase-C proof obligation. Sources checked for this docs revision.
[Workflow jobs](https://docs.github.com/en/rest/actions/workflow-jobs).

GitHub documents shared step access within a job and clean VM-backed hosted execution. Combined
with the repository's test-before-collection ordering, this motivates the explicit runtime
separation obligation above.
[Workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idsteps),
[hosted runners](https://docs.github.com/en/actions/concepts/runners/github-hosted-runners),
[secure use](https://docs.github.com/en/actions/reference/security/secure-use).

GitHub's documented default retention is 90 days, with a configurable public-repository range of
1–90 days. The current workflow requests 30 days for existing proof artifacts. Phase C must check
the effective lifetime of every required payload, commitment and provenance object. Availability
bounds verification lifetime.
[Retention](https://docs.github.com/en/organizations/managing-organization-settings/configuring-the-retention-period-for-github-actions-artifacts-and-logs-in-your-organization).

## Adaptation of the consult

All four second-consult recommendations and its four draft tightenings are adopted. The earlier
single shared implementation slice is delivered as FR1–FR3 at the maintainer's requested review
boundaries. The connected ON3 E, owning-predicate and ON4/ON5 writer work stays in slices 6–9,
consuming those interfaces. Source approval at evaluation time is settled by local inspection.
Real writer stability still requires the actual ON3–ON5 operations. FR1's modeled-host admission
and unobserved-ancestor rollback proofs are delivered; FR2/FR3, later writer proofs and real-host
qualification above remain pending. Production/live activation requires phase-C emitter isolation
and exact transport. Existing execution-source approval, complete population and adverse-history
requirements continue to apply.
