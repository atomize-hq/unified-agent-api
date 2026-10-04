# Support-depth validation test list

Status: Non-normative. Input to the implementation phase.

The [support-depth contract](../../specs/support-depth-contract.md) states four record invariants
under "Depth-gated effects" and requires repository validation to fail when one does not hold. It
does not enumerate the checks. This note carries the checks and the attack scenarios that six
review rounds produced, so they become tests of the validator instead of contract prose.

The checks belong beside the depth record's schema in the manifest root's validator spec, which
`manifest-validate` implements. They cannot be written as code before that schema exists. The
[maintenance request contract](../../specs/maintenance-request-contract-v1.md#depth-enrolled-generations)
states how a maintenance request carries P and O; the onboarding charter does not yet state it
for an approval. Until then this list is the record of what the validator must decide.

## Checks

"Any revision" checks are decided from one tree. "Merge" checks compare a merge result with the
integration branch tip it would replace.

| # | Decided on | Fails when | Invariant |
| --- | --- | --- | --- |
| 1 | Any revision | A generation has P frozen for depth-enrolled scope and its version has no depth record | 1 |
| 2 | Merge | A depth record is removed, or an acceptance entry is removed or altered | 1 |
| 3 | Any revision | The revision holds a frozen request or approval for a record's version, and the record's Event, P or O is not that generation's | 2 |
| 4 | Merge | What a depth record states changes while the merge result holds no frozen request or approval for the record's version | 2 |
| 5 | Any revision | A published result is neither the result the record states nor `unverified`, or is not `unverified` although the record does not yet state it or it was produced under a P, O or E other than the one the record states | 3 |
| 12 | Any revision | A promise is published as qualified, or a capability as depth-qualified, while a result the claim rests on is published as anything other than `verified` or a permitted `not_applicable` | 3 |
| 6 | Any revision | A tuple the record selects is published as not depth-enrolled | 3 |
| 7 | Any revision | A result is published as `verified`, or a promise or capability as qualified or depth-qualified on it, and the record states it as `verified` and binds no evidence for it, or binds evidence that is absent or has another content identity | 3 |
| 8 | Any revision | A promise is published as qualified, or a capability as depth-qualified, and no acceptance entry identifies what the record states | 4 |
| 9 | Merge | The merge result makes an acceptance effect for a tuple and no acceptance entry identifies what the tuple's depth record states in the merge result | 4 |
| 10 | Any revision | The lifecycle record of an agent that has a depth record holds a `published` or `closed_baseline` stage, or a closeout evidence id, that reports a packet or closeout the revision does not hold | 4 |
| 11 | Merge | The selection, mappings, obligations, results or bound evidence a depth record states change while none of the Event, P, O and E identities it states changes | 2 |

Checks 8 and 9 need the acceptance entry to identify what the record stated in a form the
validator can recompute from the record alone, so the check survives replaced working files. The
last reviewed proposal used the content identity of everything the record states, acceptance
entries aside. The schema decides the form.

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
- **Selection narrowing.** A tuple is dropped from the record's selection and P is restated, so
  the tuple would read as remainder.
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
- `cli_manifests/aider` and `cli_manifests/gemini_cli` have no validator spec, and the existing
  specs read one manifest root only. The checks also need lifecycle records, working files and
  the integration branch tip.
- Clearing a drift side state with no maintenance closeout in the revision is reached by no
  invariant. It cannot be a state rule, because agents with no maintenance closeout have no drift
  state either.
- Check 11 maps to invariant 2 only if the E identity the record states changes whenever a result
  or an evidence binding changes. The schema decides that.
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
  inputs, which the same edit can change.
- Comparing a changed debt row with the baseline P froze needs that baseline's content. The
  request and the record hold P's identity only. The request's frozen `[support_surface_audit]`
  rows hold part of it today: each row's identity, `debt_ref`, deferral reason and follow-on. The
  schema decides where the rest is stated.
