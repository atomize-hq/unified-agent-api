# Support-depth validation test list

Status: Non-normative. Input to the implementation phase.

The [support-depth contract](../../specs/support-depth-contract.md) states four record invariants
under "Depth-gated effects" and requires repository validation to fail when one does not hold. It
does not enumerate the checks. This note carries the checks and the attack scenarios that six
review rounds produced, so they become tests of the validator instead of contract prose.

Annex B of the contract defines the depth record: its file, what it states, its three identities
and its acceptance entries. That definition is not executable yet. The key names, the form of an
operation and the serialization the digests are taken over are left to an executable revision. The
checks below stay non-normative until they are implemented. Each one then becomes normative in the
manifest root's validator spec together with its code and its tests, as the existing validator
checks did: in one change, or a day apart. Until then the checks and scenarios are the acceptance
test of the record's definition: every check has to be decidable from a record as Annex B defines
it, together with the inputs named below, and every scenario has to come out as listed. The
[maintenance request contract](../../specs/maintenance-request-contract-v1.md#depth-enrolled-generations)
states how a maintenance request carries P and O; the onboarding charter does not yet state it for
an approval.

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
| 12 | Any revision | A promise is published as qualified, or a capability as depth-qualified, while a result the claim rests on is published as anything other than `verified` or a permitted `not_applicable` | 3 |
| 6 | Any revision | A tuple the record selects is published as not depth-enrolled | 3 |
| 7 | Any revision | A result is published as `verified`, or a promise or capability as qualified or depth-qualified on it, and the record states it as `verified` and binds no evidence for it, or binds evidence that is absent or has another content identity | 3 |
| 8 | Any revision | A promise is published as qualified, or a capability as depth-qualified, and no acceptance entry identifies what the record states | 4 |
| 9 | Merge | The merge result makes an acceptance effect for a tuple and no acceptance entry identifies what the tuple's depth record states in the merge result | 4 |
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

- **Nightly re-dispatch of an unchanged version.** The record on the integration branch states
  an Event and P and nothing else. A new generation of the same version opens with a new Event,
  the same P and no O yet, and the record restates the Event. Backlog item `uaa-0048` records
  that this re-dispatch happens. An earlier draft failed it.
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

## Open questions for the validator design

- A hand-listed acceptance entry that no closeout, publication or promotion made passes every check
  above. Today it is caught only by review, and listing one is an acceptance effect at the
  integration step.
- An entry is not scoped to an effect or a tuple, so any entry that matches the record's state
  backs any later acceptance effect, such as a hand pointer advance after a genuine closeout.
- Invariant 2 is defined by who may change a binding. History does not show the actor, so for E
  it is checkable only structurally.
- The existing validator specs describe a validator that reads one manifest root. The checks also
  need the registry, lifecycle records, working files and the integration branch tip. The spec
  change that makes a check normative has to state those inputs.
- Clearing a drift side state with no maintenance closeout in the revision is reached by no
  invariant. It cannot be a state rule, because agents with no maintenance closeout have no drift
  state either.
- A record restated consistently, with what it states under a binding and that binding's identity
  rewritten together, passes checks 11 and 13. For P and O the frozen request or approval is a
  second statement, so check 3 fails it while that request or approval stands. For E there is no
  second statement. Acceptance entries stop matching, so qualified and depth-qualified claims
  lapse, but `verified` rows follow the restated record. A selection that shrinks is the
  exception: check 2 fails it whatever else is restated.
- No record invariant reaches a declared version whose generation never froze P, such as a packet
  branch cut before the declaration landed. Only depth admission at the integration step does.
- Before the integration step a route reads the record in its own tree as the one the integration
  branch holds. A candidate record can therefore keep a target that its manifest root has stopped
  listing, because a freeze on that branch finds the target covered. At the integration step the
  declaration is missing policy for that target, and only depth admission refuses it there. No
  check above does.
- A declared version that has no depth record yet has no publication rule, because depth facts
  are published from depth records.
- Annex B names a covered mode apart from the operations that serve it and counts it as covered on
  a target only where an operation serves it there or the mode names the classification entry that
  excludes it, so check 2 compares names and that reference. Whether the operations a P lists for a
  mode still cover all of it after an operation is divided is decided when P is resolved. No check
  reaches it.
- Check 2 compares which capabilities have a mapping, not what a mapping names. A mapping kept with
  fewer operations, or with none, is therefore not a shrink. Shared mapping rule 5 requires P to
  name the complete set, and with an empty set "depth-qualified" would hold of nothing. Decide
  whether the executable revision refuses an empty mapping, and whether anything checks that a
  mapping is complete.
- A covered mode is named by a selector whose form comes with the registry contract's schema
  revision. Annex B fixes that the name is kept, not what it looks like.
- The contract treats a record as irrevocable once the integration branch holds it, and only as
  that branch holds it. So far as the branch does not hold what a record states, the record is part
  of a candidate: it binds a freeze in the tree that holds it, and a packet branch reset discards
  it. A declaration narrowed before the first record reaches the integration branch therefore
  yields a smaller first record there, through a new generation on a reset branch, and check 2 has
  no earlier record to compare it with. A later generation's widening that never reaches the
  integration branch lapses the same way. Depth enrollment rule 10 says so: work that the record
  carries only as part of a candidate is no longer due once that candidate is discarded. Restating
  the request and a record the integration branch never held together, in one tree, reaches the
  same result without a reset, and no check sees it. A candidate that is neither merged nor reset
  keeps its record and its due work in its own tree and changes nothing on the integration branch.
- A covered target that the manifest root stops listing stays covered, and no later generation
  can acquire it. Its tuples stay insufficient depth, and acceptance values that hold for the
  whole version are blocked until a later version displaces it.
- Retargeting the lifecycle record's `publication_packet_path` re-attributes a `published` stage.
  Decide whether the reference itself is checked.
- The absence of a drift side state is attributed to the previous maintenance closeout while a
  newer generation is open.
- Coverage reports, wrapper code and `wrapper_coverage.json` are dependencies whose change no
  check detects. Results that depend on them are invalidated lazily.
- The admission checker runs from the candidate's own tree.
- Promotion of a version whose working files were replaced has only the record's P and O
  identities to check against. Annex A's promotion entry decides whether that suffices.
- On the onboarding create lane, P must be frozen where the manifest root and the exact upstream
  version both exist, or check 1 cannot be satisfied.
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
- The digests are taken over a serialization that the executable revision fixes. Until it does,
  no identity can be computed, and checks 3, 8, 9, 11 and 13 have nothing to compare. Where the
  repository already takes an identity over a structured value it serializes a typed value with
  serde, and test vectors are needed before two implementations can be said to agree.
- A consumer requires the current executable revision of the record and rejects any other, and what
  a record states changes only through a generation of its own version. A second executable
  revision would therefore leave unreadable every held record whose version gets no new generation.
  The revision that follows revision 1 has to say how such a record is read.
- How a policy identity comes to differ after a change to a shared governing rule, such as an
  obligation template in the contract. Annex B has the policy part state the whole binding and
  leaves the form in which it states the governing rules to the executable revision.
- `prepare-agent-closeout` and `close-agent-maintenance` each list an entry for the closeout they
  write. Annex B names a closeout by content identity, so one closeout can leave two entries when
  the prepared artifact is edited before it is recorded, the first naming the prepared artifact.
  Decide whether the first is wanted.
- Whether the record's shape needs an entry in each root's `SCHEMA.json`, or a typed loader that
  rejects unknown fields suffices, as it does for the registry and the request. `RULES.json`
  describes `reports/<version>/` as holding coverage reports only, and the record's path is added
  there with the code that reads it. Read literally, the validator specs' `schemas` check and the
  `ci_validation` note in `RULES.json` cover every committed artifact, while the code validates
  named files only.
- When a declaration renames or divides an operation whose mode a record already covers,
  something has to say which covered mode the new operations serve. The registry contract's
  schema revision has to let a declaration state it.
- A declaration moved to another lifecycle path changes no frozen binding. The record keeps the
  selection its last generation froze, which names the old path, until a generation on the new path
  freezes P and continues the record. Until then predicate item 5 blocks acceptance, because the
  frozen P is not current. Nothing says which path owns the version's depth enrollment for
  predicate item 4 in between. Decide whether that needs a rule.
- A re-freeze after a relay run derives O from snapshots and reports the run may have rewritten.
  A re-freeze refuses when P has changed and has no such refusal for O, and the tree does not
  show which actor changed the acquired inputs.
