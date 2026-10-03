# round5-sonnet: reviewer report

Lane: Sonnet 5.5 max, defects and conformance. Candidate: `d78cf535`. Scope: bounded delta 5ff62ab8..d78cf535.
Verbatim reviewer output; the lead's adjudication is in
`../support-depth-p1a-review-adjudication-2026-10-02.md`.

---

NOT CLEAN — candidate d78cf535 (base 5ff62ab8; one file, 771 lines). I edited nothing.

File read: `~/__Active_Code/atomize-hq/unified-agent-api/.claude/worktrees/agent-a4c383158e4adc145/docs/specs/support-depth-contract.md`

## Round-four findings

| Id | Verdict | Lines | Basis |
| --- | --- | --- | --- |
| E1 | Fixed | 507-513, 538-539, 579-583, 585-589 | A record cannot be removed. A listed acceptance cannot be removed or altered. Dropping bound evidence changes E's identity, so the acceptance lapses (512-513). A row still saying verified or qualified then trips bullets 5 and 6. F1 and F2 below are side effects of the fix. |
| E2 | Fixed | 88-98, 499-505, 490, 523, 592 | `grep` finds no "evidence entry", "recorded acceptance", "captured" or "under investigation". The real `cli_manifests/claude_code/reports/2.1.274/coverage.any.json` has `deltas` gap lists only, so a covered unit has no entry. The only coverage mention (500-503) makes those files non-gated and does not read entries. Lifecycle uses now say "evidence id". |
| E3 | Fixed | 95, 397-409, 454-463, 577-578 | The record exists from the first P freeze, outlives the working files and is the resolution source. A hand-removed record is caught only through the proposed-branch comparison (bullet 2). That is consistent with the standing design. |
| E4 | Fixed | 490-497, 592-593 | Attribution is by the record a value reports. Merge: the same files, so the same answer. Hand edit: the writer is irrelevant. `historical_lifecycle_backfill`: it writes a proving-run closeout plus the lifecycle per agent, and `closeout_baseline_path` names it. I re-ran the check: all five lifecycle records resolve (publication packet, closeout baseline and maintenance closeout files all exist). |

## Findings

**F1. P2. Introduced by the delta. A new Event misfires bullet 3 and falsifies "every such change is reflected" (lines 90, 96-98, 511-512, 581-583).**
- Contradicted text: line 378, "Event … A change invalidates: Nothing. It is attribution, not tested code." The P, O and E content rows (379-381) contain no Event.
- Scenario: generation 1 has frozen P, so the record exists with no O or E yet. A nightly re-dispatch of the same version opens generation 2 (lines 77-78). The new request has a new Event and the same P.
- Consequence: the proposed branch changes what the record states, while P, O and E equal the integration copy. Bullet 3 calls that a contradiction. Line 511-512's "Every such change is reflected in the P, O or E identity" is false for it. The same happens whenever a later generation's three identities equal the stored ones.
- Related: nothing else in the contract reads the record's Event (line 90 is the only place it is stated), so it is a field with no consumer.
- Direction: make the Event the continuation discriminator. Bullet 3 becomes "…while the record states the same Event and the same P, O and E identities…". Line 511-512 becomes "Every such change other than a new Event is reflected…". Alternatively drop "the Event" from line 90.

**F2. P2. Rewritten by the delta, with the old gap not closed. "Granted it" and "supports a published value" cannot be decided and split into two readings (lines 590-591, 512-513, 93-94).**
- Bullet 7 needs a link from each acceptance effect to "the acceptance that granted it". The effects are pointer moves, status, stand-down retirement, closeout creation, lifecycle stage and advertising.
- An entry holds only "closeout, publication or promotion" plus P, O and E identities (93-94). Nothing says which kind grants which effect, or how an entry names the effect. The tree does not hold that.
- The base had "does not record", the same gap, but the bullet was rewritten here.
- Line 512-513 says an acceptance supports "a published value" only while the identities match. Your intent (E1 row) is "published qualified value". As written, one reading makes `validated` status, pointers and lifecycle values lose support after any post-promotion E refresh. That is the flow identity-bound support exists to keep legal. The other reading limits the sentence to bullet 6.
- Bullet 7 says "effect" but applies "on the integration branch", where there is no diff. The baseline is unstated.
- Direction: apply the bullet 6 identity test to bullet 7. An acceptance effect in the proposed change needs a listed acceptance of the P, O and E identities the record states at that change. That merges bullets 6 and 7. Also change 512-513 to "a published qualified value".

**P3, non-blocking**
- 576: "the agents concerned" is new and undefined. Suggest "every agent that has a depth record or whose depth-gated outputs the branch changes".
- 584 (and the older 536-537): "better than" has no order. A published `unverified` over a stated `failed` hides a counterexample. State the order, for example failed < unverified < verified, with `not_applicable` only where P permits it.
- 95, 455-456: "the version's first generation" is wrong if enrollment follows detection. Generation 1 is then not depth-enrolled and writes no record. Say "the first generation that freezes P". The base said "the current generation".
- Vocabulary:
  - "Acceptance" is a fact (351-357), an effect (515) and an entry (93, 507). "Granted" collides with debt "grants" (381, 426).
  - "Evidence id" (490, 495, 523, 592) sits beside "evidence identities" (92, 381). Say "lifecycle evidence id".
  - Unqualified "record" at 492-497 and 593 sits among depth, lifecycle and closeout records. Say "packet or closeout".
  - "Established" (406-409) is applied to E, which is never frozen. "Establish" means depth admission elsewhere.
  - Line 158 "unresolved" collides with the Publication outcome "Unresolved" (760) and with 459. It is no more decidable than "under investigation", and line 305 still says "require investigation".

**Safe simplifications**
- Delete "other than by listing a further acceptance," at 581-582. Listing is not "stating" under the delta's own vocabulary.
- Shorten the last clause of 454-458 to "Until the record is written, registry-owned authority and the frozen request or approval resolve it."

## Rule 7: what decides each bullet

| Bullet | Lines | Deciding state | Status |
| --- | --- | --- | --- |
| Intro | 574-576 | Record files, working-file requests and approvals, published outputs | Enumerable, except "agents concerned" (P3) |
| 1 | 577 | Working-file P, registry authority, presence of the record | Decidable. The onboarding "its version" depends on the charter amendment. |
| 2 | 579 | Tree diff against the integration branch | Decidable |
| 3 | 581 | Same diff, stated values and identities | Decidable but misfires (F1) |
| 4 | 584 | Published rows against stated results | Needs an order (P3) |
| 5 | 585 | Published row, record binding, content hash of the bound file | Decidable. The bullet says "repository content"; the Terms (92) do not. |
| 6 | 588 | Published label, listed acceptance identities, stated identities | Decidable |
| 7 | 590 | No deciding state | Not decidable (F2) |
| 8 | 592 | Lifecycle JSON paths and sha; the maintenance closeout file is derived through the registry | Decidable. No lifecycle field names the maintenance closeout. The drift side state has no bullet. |

The other changed MUSTs are decidable by diff or by a merge-mechanism test. They are 507-509, 625-627, 406, 454 and 740.

## Checks that came back clean
- **Prose and anchors:** only table rows 481 and 490 exceed 100 columns, and neighbouring rows do too. `#bindings`, `#reuse-and-invalidation`, `#depth-gated-effects`, `#path-enablement` and `#depth-enrollment` resolve. Bindings items 7 and 8 and Depth enrollment items 7 and 8 stay in sequence. The nested bullets under rule 7 render. `git diff --check` is clean and there are no tabs.
- **Claims about existing behavior, verified against source:**
  - `LifecycleState` has no version field (`crates/xtask/src/agent_lifecycle.rs:302-328`).
  - `close_proving_run.rs:447-474` sets `closed_baseline` and both references and clears Drifted.
  - `refresh-publication` (`crates/xtask/src/publication_refresh.rs`, ~line 439) sets `published` and clears the closeout baseline.
  - The maintenance closeout writer adds `maintenance_closeout_written` and clears Drifted.
  - Support-matrix intake includes `reports/**` (`docs/specs/unified-agent-api/support-matrix.md:166-181`).
  - The maintenance request freezes twice on the acquisition lane and once on the docs-only lane (`docs/specs/maintenance-request-contract-v1.md:55-68`).
- **Controlling-spec conflicts:** none found. The charter prerequisite is stated once, at 692-694. Rule 7 and Path enablement only point to it.
- **Unchanged text touched by the delta:** Shared mapping rule 5, Later versions, Final depth admission items 1 to 3 and the Serialization bullets, the Publication outcomes table, and the "does not change" table. None conflicts.
- **Admissibility (337-341):** consistent with "production depth enrollment" at 444 and with the Verification rejection list.
