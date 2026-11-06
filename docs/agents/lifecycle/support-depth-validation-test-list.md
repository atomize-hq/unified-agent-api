# Support-depth validation test list

Status: Non-normative. Input to the implementation phase.

The [support-depth contract](../../specs/support-depth-contract.md) states four record invariants
under "Depth-gated effects" and requires repository validation to fail when one does not hold. It
does not enumerate the checks. This note carries the checks and the attack scenarios that six
review rounds produced, so they become tests of the validator instead of contract prose.

Annex B of the contract defines the depth record: its file, what it states, its three identities
and its acceptance entries. Its sections B.6 and B.7 are revision 1 of the record's encoding and of the serialization the
digests are taken over, and `crates/xtask/src/depth_record.rs` reads a record by them.
`manifest-validate` and the support matrix generator call that reader. Checks 1 through 13
are normative in the three adopted validator specs with code and tests. Checks 5, 6, 7, 8 and 12
run through support publication; checks 2, 4, 9 and 11 run with `manifest-validate --tip <rev>`.
This note remains non-normative scenario inventory; the validator specs and contract govern.
The
[maintenance request contract](../../specs/maintenance-request-contract-v1.md#depth-enrolled-generations)
states how a maintenance request carries P and O. The
[onboarding charter](../../specs/cli-agent-onboarding-charter.md#support-depth) names the second
committed statement of them on the create lane: the `support_depth` object of the onboarding
pack's `lifecycle-state.json` (charter rule 7, OF-P2). Where a check below reads "the frozen
request or approval", an onboarding generation's second statement is that object, read with its
approval's Event. No validator reads the object yet, so checks 1 and 3 are still decided against
a second statement on the maintenance path only. They cover the create lane once the typed
lifecycle object and its cross-check land (slice 2 of the
[onboarding freeze decisions](support-depth-onboarding-freeze-decisions.md#implementation-slices)).

## Checks

"Any revision" checks are decided from one tree. "Merge" checks compare a merge result with the
integration branch tip it would replace.

| # | Decided on | Fails when | Invariant |
| --- | --- | --- | --- |
| 1 | Any revision | A generation has P frozen for depth-enrolled scope and its version has no depth record | 1 |
| 2 | Merge | A depth record is removed, an acceptance entry is removed or altered, or a record's selection comes to cover less than the record it replaces: it lacks a covered mode of that record, a target the mode was covered on, or the mapping of a capability that record claims. A mode or target of that record is lacking too when it is still listed but no operation of the resolved selection serves the mode on that target and the mode names no classification entry that excludes it | 1 |
| 3 | Any revision | The revision holds a frozen request or approval for a record's version, and the record's Event, P or O is not that generation's | 2 |
| 4 | Merge | What a depth record states, its Event or a part, changes while the merge result holds no frozen request or approval for the record's version | 2 |
| 5 | Any revision | A published result is neither the result the record states nor `unverified`, or is not `unverified` although the record does not yet state it or the P or the O that its E references is not the one the record states | 3 |
| 12 | Any revision | A promise is published as qualified, or a capability as depth-qualified, while a result the claim rests on is published as anything other than `verified` or a permitted `not_applicable`, or a capability is published as depth-qualified for a target on which an operation of its mapping is not depth-enrolled or none of that capability's `M` obligations is published as `verified` | 3 |
| 6 | Any revision | A tuple the record selects is published as not depth-enrolled | 3 |
| 7 | Any revision | A result is published as `verified`, or a promise or capability as qualified or depth-qualified on it, and the record states it as `verified` and binds no evidence for it, or binds evidence that is absent or has another content identity | 3 |
| 8 | Any revision | A promise is published as qualified, or a capability as depth-qualified, and no acceptance entry identifies what the record states | 4 |
| 9 | Merge | The merge result makes an acceptance effect other than listing an entry for a tuple and no acceptance entry identifies what the tuple's depth record states in the merge result | 4 |
| 10 | Any revision | The lifecycle record of an agent that has a depth record holds a `published` or `closed_baseline` stage, or a closeout evidence id, that reports a packet or closeout the revision does not hold | 4 |
| 11 | Merge | Anything a part of a depth record states changes while that part's identity does not | 2 |
| 13 | Any revision | A P, O or E identity a record states is not the digest of its part | 2 |

Check 13 is check 11 decided from one tree. Annex B makes it decidable from the record alone:
each identity is the digest of its own part.

Checks 8 and 9 need the acceptance entry to identify what the record stated in a form the
validator can recompute from the record alone, so the check survives replaced working files.
Annex B defines that form: one digest over the Event and the three parts, with the entries left
out.

Listing an acceptance entry is not a change to what a record states. A promotion that lists its
entry after the working files were replaced therefore does not fail check 4.

Besides the record, the checks read the registry's declarations, the frozen request or approval
among the working files, the committed evidence a result is bound to, and the other depth-gated
outputs: the published support and capability rows, the pointers and the runtime-support
projection, the version file, the stand-down markers, the closeout records, and the lifecycle
record with the publication packet it names. A merge check also reads the integration branch
tip. Inside a workspace `manifest-validate` already reads the published support rows and, to
check them, the registry, the pointers and the version files of every published root. It reads
none of the others.

## Scenarios

Each scenario is a test case. "Pass" means validation must not fail; "fail" means it must.

### Must pass

- **Nightly re-dispatch of an unchanged version.** The record on the integration branch states an
  Event and P and nothing else, and no stand-down marker names the version. A new generation of the
  same version opens with a new Event, the same P and no O yet, and the record restates the Event.
  Backlog item `uaa-0048` records that this re-dispatch happens until a stand-down marker stops it.
  An earlier draft failed it.
- **Later generation of an accepted version.** The record states the new generation's Event and
  P, shows O, E and results as not yet produced, and keeps its acceptance entries. The published
  rows are rewritten to `unverified`, and qualified and depth-qualified claims are dropped, in
  the same merge.
- **Truthful downgrade.** Bound evidence changes or is removed, the record is not rewritten, and
  the result is published as `unverified`.
- **Evidence refresh after promotion.** E changes after a pointer moved. The pointer, the version
  status and the lifecycle values stay; only qualified and depth-qualified claims lose support
  until an entry is made for the new state. No pointer rollback.
- **Report regeneration.** Coverage reports and `wrapper_coverage.json` are regenerated after a
  parity exclusion is added. They are not depth-gated outputs, and no row stays
  `verified` on a result whose bound evidence the regeneration changed.
- **Separate merges of prepare and refresh.** `prepare-publication` and `refresh-publication`
  reach the integration branch in different merges, and `refresh-publication` sets `published`
  without changing the publication packet's content. The refresh merge lists the acceptance
  entry.
- **Displacement.** A later version's packet replaces the working files of a depth-enrolled
  version. The displaced version still resolves from its record.
- **Multi-agent backfill.** `historical-lifecycle-backfill` writes several agents' lifecycle
  records in one change.

### Must fail

- **Two-merge laundering.** Merge one rebinds a `verified` result to other evidence, restates E
  and drops the row from qualified. Merge two restores the old E identity with the new evidence
  still bound and republishes qualified under the original acceptance entry.
- **Selection narrowing.** An invocation mode or target, or a capability's mapping, is dropped from
  the selection of a record the integration branch holds and P is restated, so the mode or target
  would read as remainder, or the capability as not assessed. This includes a registry declaration
  that was narrowed and a later generation that continues the record from it, and a record that
  keeps a covered mode's name, or one of its targets, while no operation serves the mode there any
  more and the mode names no classification entry that excludes it.
- **Edit under unchanged identities.** A hand edit changes a stated selection, mapping,
  obligation, result or evidence binding and leaves the Event, P, O and E identities as they were.
- **Qualified over a failed result.** The record states a promise as all `verified` on one
  target and one of its results as `failed` on another. A hand edit publishes the promise as
  qualified on the second target while its row still shows `failed`.
- **Unbinding accepted evidence.** A reporting effect removes a result's evidence binding while a
  row still says `verified` or qualified.
- **Record removed by retention.** `manifest-retain` deletes `reports/<version>/` for a version
  outside its keep set.
- **Record never written.** A generation freezes P for depth-enrolled scope and skips the record
  write, including on the docs-only lane and on an acquisition lane stood down before its second
  freeze.
- **Stale branch.** A branch cut before an acceptance entry was listed is merged and would drop
  the entry. The comparison is the merge result against the tip, not the branch against the tip.
- **In-place evidence overwrite.** Bound evidence is replaced at the same path while a published
  value still says `verified`.
- **Dangling lifecycle stage.** A hand edit sets `published` or `closed_baseline` while the
  packet or closeout it reports is absent.

## Packet freeze cases

The [maintenance request contract](../../specs/maintenance-request-contract-v1.md#packet-freeze)
requires a packet freeze that names the generation before a depth-enrolled generation's closeout.
A packet freeze commits nothing: a generation is committed when its packet merges. The cases below
are decided by depth admission, not by a check above. Its inputs are the stand-down markers on the
integration branch tip, whether that tip holds a depth record of the version, and the request, the
depth record and the registry declaration in the merge result. Today
`close-agent-maintenance` refuses when the version has no marker. Nothing checks which generation a
marker names, and nothing is checked at the integration step.

Must be admitted:

- **Named generation merges.** The tip holds a marker for the version whose `request_recorded_at`
  is the one the merge result's request and record state, and the merge result carries the closeout
  with its acceptance entry. What the packet branch held when the marker was declared makes
  no difference.
- **Re-freeze after the packet freeze.** The marker states `request_recorded_at`, and a
  maintainer-run re-freeze froze O after it was declared. The Event is unchanged, so the marker
  still names the generation.
- **Replaced before any packet freeze.** No marker names the version and the integration branch
  holds no record of it. Each dispatch resets the packet branch and writes a first record. A
  declaration narrowed between two dispatches yields a smaller first record, and nothing of the
  earlier generation is due.
- **Marker corrected.** The tip holds no record of the version. The marker was declared for a
  generation that a dispatch had just replaced. It is then changed to name the generation on the
  packet branch. This holds whether or not a declaration narrowed between the two dispatches left
  that generation a smaller selection.
- **Narrowed, generation replaced.** Under a packet freeze a maintainer narrows the declaration on
  the integration branch, the packet branch is reset, a new generation opens with the narrower P,
  the marker is changed to name it, and it is closed and merged. The tip holds no record of the
  version, so no generation of it was committed and nothing of the first one is due.

Must be refused:

- **Closed without a packet freeze.** The merge result carries a depth-enrolled generation's
  closeout and the tip holds no marker for the version. It makes no difference that the merge
  result itself carries a marker.
- **Another generation closed.** The tip's marker names one generation, and the request, record and
  closeout in the merge result belong to another generation of the same version.
- **Narrowed, original generation arrives.** The declaration is narrowed after the marker was
  declared, and the generation the marker names arrives unchanged. Its record covers more than the
  declaration does, which the registry contract treats as missing policy.
- **Digest-only marker after a re-freeze.** The marker states only `request_sha256`, and a
  re-freeze after it was declared changed the request file. The marker no longer names the
  generation. The merge result carries its closeout before the marker has been
  restated.

## Onboarding freeze cases

The [onboarding charter](../../specs/cli-agent-onboarding-charter.md#support-depth) rules 5 to 9
and 12 to 14 state the create-lane freeze, its identity artifact, acquisition and write-run
boundary. The cases below are the tests the
[onboarding freeze decisions](support-depth-onboarding-freeze-decisions.md#options-considered-risks-and-proving-tests)
name for them. Each row id is the decision a case proves. They run on the synthetic subject in an
isolated workspace only, with the fake executor and fixture templates, including a template whose
approval states `explicitly_deferred`; none needs a network or a credential. No case is
implemented yet, and every public create-lane entry keeps refusing a depth workspace.

Must be admitted:

- **First admitted dry run (OF-P1).** Rule 3 resolves and rule 4 holds. The dry run writes P, the
  depth record and the lifecycle object in one invocation.
- **Declared, no record yet (OF-P2).** The declaration and acquired surface are in place and no
  dry run has run. The generation reads as insufficient depth, and validation does not fail.
- **New approval before its first dry run (OF-P2).** A new approval with the earlier record keeps
  the earlier lifecycle object and reads as insufficient depth. Its first dry run replaces the
  object.
- **Lifecycle writers preserve the object (OF-P2).** Every lifecycle writer other than the
  `runtime-follow-on --dry-run` that writes or replaces it, that is ON1, `runtime-follow-on
  --write` and `repair-runtime-evidence` of ON2, ON3 to ON5, and NE7, leaves the lifecycle
  `support_depth` object byte for byte unchanged.
- **Deferred maintenance acquires (OF-P3).** An approval states `explicitly_deferred` and the
  registry has no `maintenance.release_watch`. The input-only writers produce a surface the dry
  run admits, and `manifest-acquisition-plan` still refuses the agent. Neither maintenance mode is
  refused for a depth-enrolled approval.
- **Input-only writers in a declared workspace (OF-P3).** They succeed and grant nothing.
- **Re-acquisition under a statement (OF-P4).** A byte-identical re-acquisition covered by an
  acquisition statement on the integration ref is admitted.
- **Retry after a failed write (OF-P4, OF-P5, OF-P9).** A write run fails or is interrupted. The
  caller tree equals the before-tree, and a retried dry run is admitted only with a matching
  statement and an unchanged P.
- **Same O on both paths (OF-P6).** For the same tree the onboarding O equals the maintenance
  derivation's O.
- **Complete single-target union (OF-P7).** It is admitted, and the maintenance derivation is
  unchanged.
- **Unchanged run outputs (OF-P9).** A passing run with an unchanged fake executor is not rejected
  for its controller outputs: stdout, stderr, execution evidence, report, status and written paths.
- **Not depth-enrolled (OF-P9).** A write run of a generation that is not depth-enrolled behaves
  as it does today.

Must be refused:

- **Rule 3 or rule 4 fails (OF-P1).** The dry run refuses and writes nothing.
- **Freeze outside the dry run (OF-P1, OF-P2).** A create-lane command other than
  `runtime-follow-on --dry-run` freezes or re-freezes P or O, or writes or replaces the lifecycle
  object. This does not cover the acceptance entries that publication and closeout list in the
  record under charter rule 9, which change neither P nor O.
- **Deleted binding (OF-P2).** A record without the object, or the object without the record. A
  missing object is never read as "not depth-enrolled".
- **Object mismatch (OF-P2).** The record and the object disagree on Event, P, or O absent versus
  present.
- **Unstated executor output (OF-P4).** An earlier generation's executor output already merged to
  the integration tip, with no statement covering it.
- **Delayed commit (OF-P4).** Executor output produced before the declaration and first committed
  to the integration branch after it, with no statement covering it. Commit timing is never
  consulted.
- **Statement mismatch (OF-P4).** An input whose bytes differ from the statement, an input the
  statement does not cover, or a statement entry with no matching input.
- **Tree differs from the ref (OF-P4).** The dry run names the differing paths. On an isolated
  route, the protected tip is the ref read.
- **Statement only on the onboarding branch (OF-P4).** It does not count before it merges.
- **Write run touches snapshots (OF-P4).** A depth-enrolled write run that writes `snapshots/`
  fails the boundary check.
- **Dry run after integration (OF-P5).** A dry run at `runtime_integrated` of a depth-enrolled
  generation. A dry run that leaves O not yet produced says so and says that a passing write run
  closes the window.
- **Changed P on a re-freeze (OF-P5).**
- **Changed acquired input after the freeze (OF-P6).** ON3, ON4 and ON5 refuse after a frozen
  acquired input changes, `wrapper_coverage.json` included.
- **Union not complete (OF-P7).** A `complete:false` union refuses with the missing targets named.
- **Stale input contract (OF-P8).** A write run refuses when the lifecycle object or the record
  changed after its dry run, and an input contract without the `support_depth` object is refused
  for a depth-enrolled generation. The object is present exactly for depth-enrolled dry runs,
  with `version` equal to the approval's, and `repair-runtime-evidence` copies its identities.
- **Forged baseline (OF-P8, OF-P9).** A forged saved `baseline` in `input-contract.json` with
  unchanged depth identities does not hide a forbidden write.
- **Executor edits a run input (OF-P9).** An executor that edits the input contract, prompt,
  lifecycle state, record or acquisition statement is rejected. An interruption after the executor
  tampered with candidate governance delivers nothing but diagnostics.
- **Forged written paths (OF-P9).** Publication refuses a non-empty `written-paths.json` whose
  SHA-256 does not match the lifecycle state (check (i)).

## Support publication cases

The [support matrix spec](../../specs/unified-agent-api/support-matrix.md#support-depth) states
which rows are depth-enrolled, what such a row publishes and how its `uaa_support` is derived.
Revision 2 of the JSON artifact carries those facts, and the generator writes them. The cases
below are the acceptance test of that section.

Must be published as listed:

- **Row without depth facts.** The version has no depth record and no registry declaration lists
  the target. The row has the fields and the values that revision 1 of the artifact gives it.
- **One target qualifies, another does not.** The record claims two capabilities and covers two
  targets, and the derivation of revision 1 yields `supported` on both. Both capabilities are
  published as depth-qualified on the first target and one of them on the second. The first row is
  `supported` and the second is `partial`.
- **Evidence changed after acceptance.** A bound evidence file has another content identity than
  the record names. The result is published as `unverified`, the capability is no longer published
  as depth-qualified, and the row's `uaa_support` falls from `supported` to `partial` or stays, and
  never rises. That is a reporting effect: it needs no lifecycle-path authority and no acceptance
  entry, and depth admission still applies.
- **No capability claimed.** The record's selection is wrapper-only. The row states each promise
  and its results, and `uaa_support` is derived as for a row without depth facts.
- **Target no longer expected.** `current.json.expected_targets` drops a target the record covers.
  The row for that target is still published, with its depth facts.
- **Enrolled before qualification.** A version whose row is `partial` under revision 1 is
  depth-enrolled with one claimed capability that is not yet depth-qualified. The row stays
  `partial`. A row that was `supported` becomes `partial`.
- **Qualified over an unsupported backend.** The row's `backend_support` is `unsupported` and the
  one capability its record claims is published as depth-qualified for the target. The row stays
  `unsupported`: depth facts raise no state.
- **Mapped operation not enrolled on the target.** The record claims a capability, and one of its
  mapped operations is not depth-enrolled on the row's target. The capability is not published as
  depth-qualified for that target, so the row is not `supported`.
- **Declared, no record yet.** The registry declares the row's version and target, and no
  generation has frozen its policy. The row is depth-enrolled, states that its obligations are not
  yet frozen, and derives `uaa_support` as a row without depth facts does.

Must fail:

- **Unreadable record.** The version's depth record cannot be parsed, or states another schema
  revision. Publication fails and does not publish the version's rows as rows without depth facts.
- **Capability from the inventory.** A row names a capability that the capability matrix lists for
  the agent and the record does not claim.
- **Raised by regeneration alone.** A regeneration would publish a capability as depth-qualified,
  and with it raise `uaa_support`, while no acceptance entry was made for what the record states.
- **Depth fact in a note.** `evidence_notes` states a result, a qualification or a capability's
  depth qualification.
- **Reader of another revision.** A reader written for revision 1 is given an artifact of the next
  revision and reports that no row is depth-enrolled.

## Closeout-published qualification cases (B.7b)

The support-depth contract's Annex A.1, "Closeout-published qualification", states CP-P3
option A: a closeout publishes the qualification its entry makes, through one MA5 batch, and IN1
attributes that qualification to the closeout. Slice B.7b of the
[closeout and promotion design](support-depth-closeout-promotion-design.md) adds these tests. They
run in process under the cfg(test) isolated route with the fake host, are credential-free and live
under `crates/xtask/src/isolated_maintenance/tests/` (new `publication.rs`, plus the chain files of
B.5b, B.5c and B.6b). Every chain test of those slices reruns with option A on; the proofs made
before B.7 do not carry over.

Must pass:

- **Qualified row through the batch, each writer.** For `close-agent-maintenance` and for
  `prepare-agent-closeout --write`, the delivered candidate holds the closeout, exactly one new
  current `MaintenanceCloseout` entry, and the closeout's own agent and version rows published as
  qualified. The rows equal what the delivered post-entry record derives, and every other path of
  the batch is one the closeout already owned.
- **Fresh post-entry evaluation.** The verdicts the rendered rows rest on were evaluated for the
  post-entry record. A fake host that counts evaluations shows one evaluation after the entry was
  appended; verdicts from before the append are not reused.
- **Other rows untouched.** Another agent's rows, other versions of the same agent, the
  runtime-support projection and capability publication are byte-identical before and after the
  batch.
- **IN1 closeout attribution.** IN1 admission (P = T) admits the MA5 candidate and attributes the
  qualification to closeout; post-push observation passes for the same transition.
- **MA6 order.** From the admitted MA5 checkpoint, MA6 appends its `Promotion` entry before it
  renders. Its rendered rows equal what IN1 derives from the delivered post-entry record: the row is
  already qualified and stays so. IN1 admits the promotion candidate before merge and after the push.
- **A.5 replay with option A.** The replay forked from the admitted MA5 checkpoint before any MA6
  still creates the first `Promotion` entry, and its rendered rows equal the derived ones.
- **Scratch publication.** A verified row is rendered in an MA6 scratch view by the view-aware
  evaluator (B.7a) and matches the caller-side derivation.

Must fail, each one leaving caller state byte for byte unchanged:

- **Qualification without a closeout entry.** A candidate publishes a newly qualified row and adds
  no new `MaintenanceCloseout` entry. IN1 attributes it to promotion and refuses.
- **Entry for another tuple.** The new entry names another agent or another version than the newly
  qualified row. IN1 refuses.
- **Closeout and promotion together.** The candidate adds a closeout entry and has a promotion
  effect. The qualification is attributed to promotion; without promotion authority, IN1 refuses.
- **Rows outside the closeout.** The MA5 render would change another agent's row, another version's
  row, the runtime-support projection or capability publication, including by an honest reporting
  downgrade. MA5 refuses the whole batch.
- **Changed rendered rows.** The batch's rows are changed between plan and apply, so they differ
  from what the post-entry record derives. MA5 refuses at apply; the same candidate assembled by
  hand fails IN1 publication consistency.
- **Disagreement with admission.** The post-entry evaluation leaves a result the admission relied
  on unverified (for example, the fake host changes the attempt between admission and evaluation).
  MA5 refuses; it does not deliver the closeout with downgraded rows.
- **Pre-entry verdicts.** A batch rendered from verdicts sealed to the pre-entry record is refused.
- **MA6 rendered before its entry.** Under option A the retained closeout entry already
  qualifies the row, so rows rendered before and after the `Promotion` entry can be identical; the
  test does not rely on a row mismatch. It checks the order and the verdict binding directly: the
  fake host records that the evaluation MA6 renders from observed the record with its `Promotion`
  entry appended, and an MA6 proposal that offers verdicts sealed to the pre-entry record is
  refused, whatever the rows say.
- **Production control.** With no route installed, both MA5 writers and MA6 keep their current
  refusal on a depth workspace and their ordinary behavior otherwise; production publication still
  uses `production_results` and stays `unverified`.

## Open questions for the validator design

- A hand-listed acceptance entry that no closeout, publication or promotion made passes every check
  above. Today it is caught only by review, and listing one is an acceptance effect at the
  integration step.
- An entry is not scoped to an effect or a tuple, so any entry that matches the record's state
  backs any later acceptance effect, such as a hand pointer advance after a genuine closeout.
- Invariant 2 is defined by who may change a binding. History does not show the actor, so for E
  it is checkable only structurally.
- The validator specs state the one-tree inputs and the additional inputs for checks 2, 4 and 11:
  the explicit integration-tip commit and its records/registry, plus a versioned approval for a
  changed onboarding record. Check 2 preserves the old entry sequence as an unchanged prefix.
  Check 4 compares statements with entries excluded and covers first creation too; its matching
  maintenance request must carry the depth P table, and an approval cannot substitute for another
  lifecycle path. The caller supplies the test merge's verified first parent for a staging PR,
  the supplied base_sha for a staging merge group, and event.before for a push. Check 11 follows
  from checking each part's digest in both records (check 13).
- `manifest-validate` reads the standing request's `[support_depth]` table without the validating
  loader, which rejects the table as an unknown key until a depth enrollment can be declared.
  When the loader accepts it, rule 4 of the request contract's request fields is the comparison
  check 3 makes, `DepthRecord::request_mismatch`.
- An onboarding record's Event is not compared with its approval. The approval loader validates
  `approval_commit` and `approval_recorded_at` but does not keep them, and checks 1 and 3 reach
  the create lane only once the charter names a second statement of its P and O. Decide the
  Event comparison with that statement. A maintenance request for a version whose record states an
  onboarding Event fails check 3 today, because its Event is not the record's. That holds only if
  no maintenance generation may continue an onboarding record, which the question below on a
  declaration moved to another lifecycle path has to settle.
- Clearing a drift side state with no maintenance closeout in the revision is reached by no
  invariant. It cannot be a state rule, because agents with no maintenance closeout have no drift
  state either.
- A record restated consistently, with what it states under a binding and that binding's identity
  rewritten together, passes checks 11 and 13. On the maintenance path the frozen request is a
  second statement of P and O, so check 3 fails it while that request stands. On the create lane
  the charter names none. For E there is no second statement. Acceptance entries stop matching, so
  qualified and depth-qualified claims lapse, but `verified` rows follow the restated record. A
  selection that shrinks is the exception: check 2 fails it whatever else is restated.
- No record invariant reaches a declared version whose generation never froze P, such as a packet
  branch cut before the declaration landed. Only depth admission at the integration step does.
- Before the integration step a route reads the record in its own tree as the one the integration
  branch holds. A record the integration branch does not hold can therefore keep a target that its
  manifest root has stopped listing, because a freeze of the request on that branch finds the
  target covered. At the integration step the declaration is missing policy for that target, and
  only depth admission refuses it there. No check above does. The generation cannot merge until
  the target is listed again or a generation frozen from a declaration without the target
  replaces it.
- Annex B names a covered mode apart from the operations that serve it and counts it as covered on
  a target only where an operation serves it there or the mode names the classification entry that
  excludes it, so check 2 compares names and that reference. Whether the operations a P lists for a
  mode still cover all of it after an operation is divided is decided when P is resolved. No check
  reaches it.
- Check 2 compares which capabilities have a mapping, not what a mapping names. A mapping kept with
  fewer operations is therefore not a shrink. Shared mapping rule 5 requires P to name the
  complete set, refuses a mapping that names no operation, and withholds depth-qualified on a
  target where a mapped operation is not depth-enrolled or no `M` obligation of the mapping is
  `verified`, so the support matrix spec's rule 6 withholds `supported` there. Decide whether
  anything checks that a mapping is complete, and how a mapping states an operation that the
  upstream CLI does not offer on one target, since rule 5 leaves the capability not
  depth-qualified there.
  The registry contract's rule 5 answers the second part: the operation is declared with modes
  whose `targets` leave that target out, and the capability is not depth-qualified there.
- A covered mode is named by the `selector` of the registry declaration's `modes` entry that
  first covered it. The registry loader refuses two selectors with one `command_path` and `mode`.
  It does not decide whether two selectors with different `mode` names overlap, which
  classification rule 2 makes an error where their effects or requirements differ. Annex B.4
  rule 7 decides the other question: a kept `selector` whose command path, mode or predicates
  differ from the record the integration branch holds is another mode, so check 2 fails a mode
  narrowed under its old name.
- A generation is committed from the time the integration branch holds the record it wrote or
  continued, which the merge of its packet brings about. Until then the next dispatch replaces it,
  or a maintainer narrows the declaration and reopens it, and check 2 has no earlier record to
  compare a smaller first record with. Depth enrollment rule 10 says so: a generation that was
  never committed leaves no due work. A packet that is merged while still open commits its
  generation at that merge. Nothing refuses a declaration narrowed while a packet is open. Decide
  whether anything should report it.
- An abandoned packet keeps its marker, because only promotion removes one. Automation stays stood
  down for that agent and version. A later generation of the version has to cover what the
  abandoned one covered only where the abandoned packet had merged. Decide whether a superseded
  version's marker is ever retired another way. A marker declared only after its packet was
  abandoned, or after a later version's packet replaced the working files, names its generation
  all the same. Decide whether it should.
- `prepare-agent-maintenance` reads no marker, so a maintainer's run under a packet freeze that is
  not a re-freeze opens a generation that the marker does not name, and its closeout is refused
  until the marker is changed. Decide whether the command should ask before it writes. A check
  there has to let through the re-freeze that freezes O after a packet freeze.
- A marker that states only `request_sha256` cannot be matched to a record once a later version's
  packet has replaced the working files, because the record states the Event and not the request's
  digest.
- A marker names a generation by `request_recorded_at`, and the Event has three more fields. Two
  generations of one version that stated the same `request_recorded_at`, which takes a caller who
  passes it by hand, would both be named.
- A covered target that the manifest root stops listing stays covered, and no later generation
  can acquire it. Its tuples stay insufficient depth, and acceptance values that hold for the
  whole version are blocked until a later version displaces it.
- Retargeting the lifecycle record's `publication_packet_path` re-attributes a `published` stage.
  The reference path and actual raw digest are checked; candidate disappearance alone creates no
  D2 destination. See the contract and check 9 validator section.
- The absence of a drift side state is attributed to the previous maintenance closeout while a
  newer generation is open.
- Coverage reports, wrapper code and `wrapper_coverage.json` are dependencies whose change no
  check detects. Results that depend on them are invalidated lazily.
- The admission checker runs from the candidate's own tree.
- Promotion of a version whose working files were replaced has only the record's P and O identities
  to check against. Annex A's promotion entry decides whether that suffices. The marker is still on
  the tip then and names the generation by `request_recorded_at`, which the record states as its
  Event, so promotion could compare the two without the working files. `parity-promote` reads
  neither the request nor the closeout today (`uaa-0064`).
- On the onboarding create lane the charter freezes P at a `runtime-follow-on` dry run, once the
  manifest root holds version metadata for the approved version, a `RULES.json` and a validator
  spec. No create-lane command writes any of them today, and a write run is rejected when it
  changes anything under the manifest root outside `snapshots/` and `supplement/`. The baseline of
  a write run is taken before the dry run writes its own files, so a depth record written by the
  dry run would count as a change of the write run. Decide which command writes those three files
  for a new root, and how the dry run's record stays out of the write run's comparison.
- A hand edit that restates the request's `[support_depth]` identities and the record together
  passes check 3. Recomputing P needs its debt baseline, and recomputing O needs the acquired
  inputs, which the same edit can change. The same state is reachable without a hand edit: after
  an interrupted relay run, or a restore the relay had to refuse, the next dry run takes the tree
  as its baseline.
- A relay restore returns the tree to what it held when the executor started, and that can
  already be stale. A run that failed only a green gate is left in place; if it changed bound
  evidence, a later failed run's restore puts back a `verified` row that is no longer true.
  Repository validation still fails that tree.
- A freeze that records an Event the target version's depth record already states, with no
  standing request that states it, is a first freeze under an unchanged Event: P is frozen again
  and the record's Event does not change. Decide whether it refuses.
- The digests are taken over the serialization Annex B.7 fixes, with one test vector. The
  reader refuses a record whose part does not have the identity it states, which is check 13.
  `manifest-validate` decides check 3. Check 8 is decided through the support publication, which
  qualifies nothing unless an entry identifies what the record states. With --tip, checks 2 and 4
  compare records across two trees and check 11 follows from check 13 in each. Check 9 compares
  semantic effects and same-revision destinations, including runtime profile additions beyond
  the canonical publication matrix. Route admission and enablement remain uaa-0073 work.
- A declaration keeps at least one operation that is not a mode exclusion, so a version whose
  only covered mode turns out to be interactive-only cannot have it excluded and stays at
  insufficient depth. Dropping that rule later is a compatible change and adding it later is
  not. Decide, once depth admission can tell a first enrollment from a continuation, whether
  the rule moves there and binds a first enrollment only.
- Revision 1 reads a covered mode as kept only when its selector, command path, mode and
  predicates are all unchanged, so a selector cannot be widened in place. Decide whether a
  later revision needs to tell a widened selector from a narrowed one.
- Revision 1 states the independently required acceptance work as surfaces with targets, taken
  from the request. The contract does not define that work further. Decide whether the
  evaluator needs more than the name identity and the targets.
- A consumer requires the current executable revision of the record and rejects any other, and what
  a record states changes only through a generation of its own version. A second executable
  revision would therefore leave unreadable every held record whose version gets no new generation.
  The revision that follows revision 1 has to say how such a record is read.
- A policy identity differs after a change to an obligation template because the policy part
  states BASE and each selected template's obligations as P resolved them. A governing rule
  that is neither, such as a classification rule, changes the identity through the part's
  `rules_revision`, which the change has to bump by hand. Nothing checks that it did.
- `prepare-agent-closeout` and `close-agent-maintenance` each list an entry for the closeout they
  write. Annex B names a closeout by content identity, so one closeout can leave two entries when
  the prepared artifact is edited before it is recorded, the first naming the prepared artifact.
  Decide whether the first is wanted.
- The record's shape is decided by its typed reader, which rejects unknown members, as it is for
  the registry and the request; `SCHEMA.json` has no entry for it. `RULES.json` names the record's
  path as `storage.committed.reports.depth_record`, and that is how a root states that it adopts
  Annex B. Read literally, the validator specs' `schemas` check and the `ci_validation` note in
  `RULES.json` cover every committed artifact, while the code validates named files only.
- When a declaration renames or divides an operation whose mode a record already covers,
  something has to say which covered mode the new operations serve. The registry declaration's
  `serves` states it.
- A declaration moved to another lifecycle path changes no frozen binding. The record keeps the
  selection its last generation froze, which names the old path, until a generation on the new path
  freezes P and continues the record. Until then predicate item 5 blocks acceptance, because the
  frozen P is not current. Nothing says which path owns the version's depth enrollment for
  predicate item 4 in between. Decide whether that needs a rule.
- A re-freeze after a relay run derives O from snapshots and reports the run may have rewritten.
  A re-freeze refuses when P has changed and has no such refusal for O, and the tree does not
  show which actor changed the acquired inputs.
- A depth-enrolled row states every obligation's result, and one promise can have more than a dozen
  obligations. Decide how the Markdown projection renders them so that it still agrees with the
  JSON artifact.
- A `markdown_support_claim` line states one `uaa_support` value with no capability scope. For a
  depth-enrolled row the value is a statement about the capabilities the record claims. Decide
  whether a line about such a row has to name them.
- On a depth-enrolled row `supported` means that the name-level derivation yields it and that every
  claimed capability is depth-qualified, even where the record claims one capability and the agent
  advertises many. The row names the capabilities in `depth.capabilities`, and a reader of the
  one column does not see them.
- Revision 2 lists every capability the record claims on every depth-enrolled row of the
  version, a declared target the record does not yet cover included, so a target on which no
  operation of a capability's mapping is enrolled publishes that capability as not
  depth-qualified and rule 6 keeps its row from being `supported`. Decide whether that is intended, or whether a capability is assessed only on the
  targets its mapping's operations are enrolled on.
- `support-matrix` regenerates every agent's rows. A run made for one agent can find that another
  agent's row would rise because that agent's record now supports it. The support-depth contract
  refuses that acceptance effect for a route without authority and leaves every output unchanged.
  CP-P3 option A (Annex A.1, "Closeout-published qualification") answers this for the closeout's
  own agent and version: the MA5 batch publishes that raise. A raise of another agent's row that
  its record now supports still has no publishing route, and MA5 refuses a batch that would write
  one. Decide which route publishes such a raise under the owning path's authority, and how a
  refused run names the row that blocked it.
- The support-depth contract's Publication table has an outcome for a qualified forwarding promise
  and none for a qualified promise of another kind. The registry declaration states `forwarding`
  for each promise, revision 2 of the support matrix states `qualified_forwarding` for a
  qualified forwarding promise, and a qualified promise of another kind has `qualified: true`
  with no qualification outcome. The registry refuses `forwarding = true` for `CORE`, `SESSION`,
  `QUERY` and `MANAGED`, whose forwarding column never admits it, and whether forwarding
  suffices under the conditions the column states for the other templates is the maintainer's
  call.
- Bound evidence is named by path and can lie outside the manifest root, such as a test source
  under `crates/`. The support matrix reads it only for its content identity. A result that the
  support-depth contract's reuse and invalidation rules invalidate through a dependency outside the
  bound evidence cannot be seen from the record and that evidence. Decide what the generator reads
  to publish such a result as `unverified`. Revision 1 does not confine bound evidence to
  `reports/**`: a file reference is any repository-relative path.
- Decided by charter rules 5, 6, 12 and 14 (OF-P3 to OF-P7): how the create lane acquires the
  surface ahead of the dry run, what admits it as acquired, that no dry run is admitted after
  `runtime_integrated`, the acquired inputs of an onboarding O, and that a union which is not
  complete refuses. The [onboarding freeze cases](#onboarding-freeze-cases) below are their
  tests. Still open, and refused meanwhile: whether a union that is not complete may freeze O for
  exactly the declared targets (OF-P7(b)), and whether publication may regenerate
  `wrapper_coverage.json` for a depth-enrolled generation instead of being refused for it (OF-P6).
  Also open: whether version metadata for charter rule 4 is written before the declaration or
  only by a later writer, since `manifest-version-metadata` refuses a depth manifest, and whether
  candidate execution later applies to write runs that are not depth-enrolled (OF-P9).
- Decided by charter rule 7 (OF-P2): the `support_depth` object of the onboarding pack's
  committed `lifecycle-state.json` states the P and O identities beside the depth record. The dry
  run writes it with the record, and every other lifecycle writer preserves it. A run's input
  contract still cannot serve, and carries the identities only as a cross-check (charter rule 8,
  OF-P8).
- The charter gives the create lane no route for pointers or for a `validated` or `supported`
  version status. A version whose depth enrollment the onboarding path owns reaches them only after
  its proving-run closeout, once its declaration is moved to the maintenance path and a maintenance
  generation is promoted. The registry contract and the charter's multi-target entry rule already
  say what the agent needs for that path. Decide whether the move needs anything else. The question
  above on a declaration moved to another lifecycle path covers what owns the enrollment between
  the move and the first maintenance freeze.
- An approval artifact is committed before `onboard-agent` appends the registry entry, and the
  charter has it state only the exact upstream version. An approval has been amended in place
  before, with its `approval_commit` and `approval_recorded_at` unchanged. Decide whether an
  approval written before the version is known can be amended, whether an amended approval is a new
  approval artifact, and so a new generation, and what a second approval for the same version
  continues.
- The support matrix makes a row depth-enrolled from a registry declaration. A declaration that the
  registry contract makes missing policy, such as one that lists a target outside
  `union.expected_targets`, or one removed after a generation was committed, leaves the version
  unresolved, which the support-depth contract keeps apart from insufficient depth. Decide how a
  row states unresolved, and whether publication fails as it does for a record it cannot read. A
  declared target that an existing record does not cover is a third state: decide whether its row
  states obligations not yet frozen. A fourth is a declaration on the onboarding path of a version
  the agent's approval does not state: the charter keeps its version depth-enrolled with nothing
  frozen, and no dry run signals it. Its rows are those of the case "Declared, no record yet".
  Decide whether validation reports it.

## Implemented check 9 regression coverage

The Git-backed c0 comparison tests cover pointers, status/passed targets, marker retirement,
embedded expected-target exposure, maintenance/proving closeouts, lifecycle publication and raw
retargets, drift clearance and removal, durable ownership/displacement/conflicts, ordinary legacy
approvals, qualification and capability-cell additions, and runtime opt-in/default-feature additions.
They distinguish reporting changes from acceptance, require current entries and preserve the
entry-listing exception. D5 tests separately exercise the complete model domain and unsupported
source/default/feature inputs. No native execution on unavailable hosts or route admission is
inferred from structural comparisons.

## Executed depth-refusal coverage (Annex A ON1–ON5, NE1–NE8, LW1)

Roadmap B.7 of the
[machinery-proof rescope](support-depth-machinery-proof-rescope.md). Each route's public
entrypoint is executed against a workspace that carries the synthetic `depth_fixture` subject
(`crates/xtask/tests/fixtures/depth-synthetic`) and must fail with its depth refusal and leave
the tree byte-, type- and mode-identical, with nothing written to its private `TMPDIR`.

`crates/xtask/tests/depth_refusal_routes.rs` copies every tracked file and runs the real `xtask`
binary from the copy's root with an empty `PATH`, so a route that failed to refuse could not
reach `git`, `cargo` or an upstream CLI. Each case runs against three depth carriers:

- **Declared:** the registry carries the synthetic entry with its `support_depth` table and the
  governing manifest root; no record exists yet.
- **Retained:** the entry stays without its table and the reviewed synthetic record is retained at
  `cli_manifests/depth_fixture/reports/0.2.0/depth-record.json`.
- **Orphaned:** the whole entry is gone; the manifest root and record remain.

`isolated_maintenance/tests/unsupported_routes.rs` (in-crate) installs the authenticated
synthetic isolated route, prepares the subject through MA1/MA2 (declared table, retained record,
a depth-bearing support bundle) and drives the library entrypoints: the route is MA authority only
and grants these writers none. A workspace-wide guard reports the declaration first and a
manifest-scoped guard the record first.

| Route | Entrypoint(s) executed | Executed tests | Earlier coverage reused |
| --- | --- | --- | --- |
| ON1 | `onboard-agent --write` | `onboarding::on1_onboard_agent_write_refuses_depth_workspace`; installed route (library) | — |
| ON2 | `runtime-follow-on --write`, `repair-runtime-evidence --write` | `onboarding::on2_runtime_follow_on_and_repair_runtime_evidence_writes_refuse_depth_workspace`; installed route | — |
| ON3 | `prepare-publication --write` | `onboarding::on3_prepare_publication_write_refuses_depth_workspace`; installed route | — |
| ON4 | `refresh-publication --write` | `onboarding::on4_refresh_publication_write_refuses_depth_workspace`; installed route | — |
| ON5 | `close-proving-run`, `prepare-proving-run-closeout --write` | `onboarding::on5_proving_run_closeout_entrypoints_refuse_depth_workspace`; installed route (`prepare-proving-run-closeout` only: `close_proving_run` is binary-only) | — |
| NE1 | `support-matrix` (write) → `support_matrix/publication.rs::write_publication_artifacts` | `neutral::ne1_support_matrix_write_refuses_depth_workspace` (declared, orphaned); installed route delivers a route-generated bundle | — |
| NE2 | `capability-matrix`, `capability-matrix --out` | `neutral::ne2_capability_matrix_write_refuses_depth_workspace` | — |
| NE3 | `manifest-version-metadata --root`, alias `codex-version-metadata` | `neutral::ne3_version_metadata_and_its_alias_refuse_depth_manifest`; installed route | `depth_promotion.rs::neutral_lower_batch_cannot_write_retained_depth_even_without_a_declaration` (library `run`) |
| NE4 | `manifest-validate --mode fix`, alias `codex-validate --mode fix` | `neutral::ne4_manifest_validate_fix_mode_and_its_alias_refuse_depth_manifest`; installed route (`violation_codes`) | — |
| NE5 | `manifest-retain --apply`, alias `codex-retain --apply` | `neutral::ne5_retention_apply_and_its_alias_refuse_depth_manifest` | — |
| NE6 | `codex-snapshot`, `claude-snapshot`, `opencode-snapshot` with legacy `--out-dir`, including the subject's root and a not-yet-created destination | `neutral::ne6_legacy_snapshot_out_dir_refuses_depth_destination` | — |
| NE7 | `historical-lifecycle-backfill --all`, `--agent` | `neutral::ne7_historical_lifecycle_backfill_refuses_depth_workspace` | — |
| NE8 | `runtime-advertising-model` (write) | `neutral::ne8_runtime_advertising_model_write_refuses_depth_workspace` | — |
| LW1 | `workspace_mutation::apply_mutations`; `agent_lifecycle::{write_lifecycle_state, write_publication_ready_packet}` | `lower::lw1_apply_mutations_refuses_depth_workspace`, `lower::lw1_apply_mutations_refuses_proposed_record_and_enrollment_destinations`, `lower::lw1_lifecycle_and_packet_storage_writers_refuse_depth_workspace`, `lower::lw1_lifecycle_and_packet_storage_writers_refuse_a_record_destination`; installed route | `depth_promotion.rs`: `neutral_lower_batch_cannot_write_retained_depth_even_without_a_declaration`, `nested_root_cannot_hide_retained_depth_from_public_delivery`, `neutral_batch_cannot_introduce_a_record_or_declaration_before_accepting_outputs`, `publication_packet_lower_writer_cannot_overwrite_a_retained_record`, `nested_standalone_manifest_root_cannot_hide_retained_reports`, `empty_inner_anchor_cannot_hide_an_outer_retained_record` |

`copied_workspace_without_synthetic_depth_is_not_refused` is the control: the copied tree without
the synthetic depth passes the guard, so each refusal above is caused by the depth it carries.

Two refusals come from the registry loader rather than the route guard. With a declared table,
`support-matrix` and `historical-lifecycle-backfill` load the registry before their writer guard
and stop at `AgentRegistry::parse`'s `support_depth` refusal (Annex A "Public refusal"); with
retained or orphaned depth they reach the guard. The **Retained** carrier is not run for NE1:
with the subject still registered for support publication, the generator first fails on the
synthetic root's absent `current.json`, which is not a depth refusal.

Every listed entrypoint refuses today; none needed a production change or an ignored test.
