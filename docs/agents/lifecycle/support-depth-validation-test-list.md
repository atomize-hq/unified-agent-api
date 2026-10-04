# Support-depth validation test list

Status: Non-normative. Input to the implementation phase.

The [support-depth contract](../../specs/support-depth-contract.md) states four record invariants
under "Depth-gated effects" and requires repository validation to fail when one does not hold. It
does not enumerate the checks. This note carries the checks and the attack scenarios that five
review rounds produced, so they become tests of the validator instead of contract prose.

The checks belong beside the depth record's schema in each manifest root's validator spec, which
`manifest-validate` implements. They cannot be written as code before that schema and the request
contract's P and O bindings exist. Until then this list is the record of what the validator must
decide.

## Checks

"Any revision" checks are decided from one tree. "Merge" checks compare a merge result with the
integration branch tip it would replace.

| # | Decided on | Fails when | Invariant |
| --- | --- | --- | --- |
| 1 | Any revision | A generation has P frozen for depth-enrolled scope and its version has no depth record | 1 |
| 2 | Merge | A depth record is removed, or an acceptance entry is removed or altered | 1 |
| 3 | Any revision | The revision holds a frozen request or approval for a record's version, and the record's Event, P or O is not that generation's | 2 |
| 4 | Merge | What a depth record states changes while the merge result holds no frozen request or approval for the record's version | 2 |
| 5 | Any revision | A published result is not the result the record states. Publishing `unverified` for a stated `verified` is the one exception | 3 |
| 6 | Any revision | A tuple the record selects is published as not depth-enrolled | 3 |
| 7 | Any revision | A result is published as `verified`, or a promise or capability as qualified or depth-qualified on it, and the record binds no evidence for the result, or binds evidence that is absent or has another content identity | 3 |
| 8 | Any revision | A promise is published as qualified, or a capability as depth-qualified, and no acceptance entry identifies what the record states | 4 |
| 9 | Merge | The merge result makes an acceptance effect for a tuple and no acceptance entry identifies what the tuple's depth record states in the merge result | 4 |
| 10 | Any revision | The lifecycle record of an agent that has a depth record holds a `published` or `closed_baseline` stage, or a closeout evidence id, that reports a packet or closeout the revision does not hold | 4 |

Checks 8 and 9 need the acceptance entry to identify what the record stated in a form the
validator can recompute from the record alone, so the check survives replaced working files. The
last reviewed proposal used the content identity of everything the record states, acceptance
entries aside. The schema decides the form.

## Scenarios

Each scenario is a test case. "Pass" means validation must not fail; "fail" means it must.

### Must pass

- **Nightly re-dispatch of an unchanged version.** The record on the integration branch states
  only P. A new generation of the same version opens with a new Event, the same P and no O yet,
  and the record restates the Event. Backlog item `uaa-0048` records that this re-dispatch
  happens. An earlier draft failed it.
- **Later generation of an accepted version.** The record states the new generation's Event and
  P, shows O, E and results as not yet produced, and keeps its acceptance entries.
- **Truthful downgrade.** Bound evidence changes or is removed, the record is not rewritten, and
  the result is published as `unverified`.
- **Evidence refresh after promotion.** E changes after a pointer moved. The pointer, the version
  status and the lifecycle values stay; only qualified and depth-qualified claims lose support
  until an entry is made for the new state. No pointer rollback.
- **Report regeneration.** Coverage reports and `wrapper_coverage.json` are regenerated after a
  parity exclusion is added. They are not depth-gated outputs.
- **Separate merges of prepare and refresh.** `prepare-publication` and `refresh-publication`
  reach the integration branch in different merges, and `refresh-publication` sets `published`
  without rewriting the publication packet.
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

- A hand-listed acceptance entry that nothing granted passes every check above. Today it is
  caught only by review, and listing one is an acceptance effect at the integration step.
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
