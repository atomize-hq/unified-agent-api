# round5-opus: reviewer report

Lane: Opus 5.5 xhigh, adversarial. Candidate: `d78cf535`. Scope: bounded delta 5ff62ab8..d78cf535.
Verbatim reviewer output; the lead's adjudication is in
`../support-depth-p1a-review-adjudication-2026-10-02.md`.

---

NOT CLEAN — candidate `d78cf535` (one file changed against `5ff62ab8`, confirmed). Two P2s remain, both introduced by the delta; no P1.

| Id | Status | Lines | Note |
| --- | --- | --- | --- |
| E1 | Fixed as to compliance | 507-513, 538-539, 579-589 | Unbinding with unchanged identities is a contradiction; unbinding with a new E forces the row off `qualified`. The validation backstop can still be passed in two merges (F2). |
| E2 | Fixed | 88-94, 481, 499-505, 585-587 | "Evidence entry" is gone; the result-to-evidence binding is decidable for repository content. |
| E3 | Fixed | 95, 406-409, 454-463, 577-578 | The record exists from the P freeze, so displacement no longer reads as not depth-enrolled. The fix collides with rule 7 on retry (F1). |
| E4 | Fixed | 490-497, 592-593 | Attribution is state-based and decidable. The five current lifecycle records resolve by file existence; I did not check the packet sha. |

## Findings

**F1 — P2, delta-introduced. Rule 7 makes a later generation's Event restatement a contradiction (581-583 against 90, 96-98, 508-512).**
- **Conflict:** The record "states the Event" and a later generation "continues the same record: what the record states becomes that generation's". Bullet 3 fails any change to what the record states while P, O and E identities match the integration copy. "Every such change is reflected in the P, O or E identity" (511-512) is false for an Event change: P is content-identified and the Event "invalidates nothing" (378).
- **Scenario:** Depth-enrolled V has a P-only record on `staging` (docs-only lane, or a packet merged before its second freeze). The nightly re-dispatch opens a new generation with a new `request_commit` and `request_recorded_at`, the same P, and no O yet. The record must state the new Event, bullet 3 fires, and validation MUST fail. V stays at insufficient depth until the maintainer changes P.
- **Reachability:** `acquisition-maintenance-lifecycle-spec.md:786` records nightly re-dispatch of unchanged versions and a packet merged without promotion (#215).
- **Wider under one reading:** If the record keeps O and E until the later generation re-establishes them, every re-dispatch of a merged version collides.
- **Remediation:** Exempt restating the Event alone from bullet 3, limit the 511-512 sentence to binding changes, and say at 96-98 that a later generation's opening leaves O, E and results not yet established.

**F2 — P2, delta-introduced. Nothing checks that a stated identity is the identity of what it names (511-513, 581-583, 588-589).**
- **Conflict:** Identity-bound support and bullet 3 both rest on 511-512, which is declarative and has no listed contradiction. Rule 7 claims to hold "whatever produced the change" (595-596).
- **Scenario:**
  1. Merge 1 rebinds a `verified` result from H1 to H2, restates E1 as E2 and drops the row from `qualified`. This is a permitted reporting effect and passes all eight bullets.
  2. Merge 2 restates E2 as E1 with H2 still bound and republishes `qualified`. Bullet 3 does not fire because the identity differs from the integration copy; bullet 5 passes because H2 is present; bullet 6 passes because the genuine acceptance of (P1, O1, E1) is listed.
  3. The row now claims `qualified` on evidence no acceptance covered.
- **Variant:** Dropping a tuple from the selection while restating P also passes all eight, and the tuple reads as remainder.
- **Limits:** A compliant route cannot do this (predicate item 3), so the gap is in the backstop for hand edits and unsupported tooling. Depth enrollment rule 8's "contradictory generation references" might catch the variant while the request exists, but it is not in rule 7's list.
- **Remediation:** Add one listed contradiction for a record whose stated P, O or E identity is not that of the binding the same revision holds. Make the stated E identity recomputable from what the record states, so the check survives replaced working files.

## Non-blocking (P3)

- **579-583:** "A proposed branch that removes/changes" names no comparison base. A stale branch reads as removing a later acceptance, and a fast-forward push (allowed at 627) has no proposed branch. Workflows reach `staging` only by PR, so say "the merge result against the tip it replaces".
- **585-587:** A binding to non-repository content passes both limbs. Base placed evidence under the manifest root; the delta dropped that. State that bound evidence is committed content identified by path and identity.
- **499-505 (the round-two question):** In-place overwrite is closed for bound evidence by bullet 5. Target coverage reports, which O is frozen from (400-401), now join code and `wrapper_coverage.json` as dependencies whose change no contradiction detects. That class is pre-existing; the delta moved coverage reports into it.
- **512-513 against 588-591:** "A published value" is broader than the bullets, which bind only `qualified` and depth-qualified to identities. Align the wording so nobody implements pointer rollback on an E change.
- **490-497, 592-593:**
  - The reported records name no version: the packet and proving-run closeout carry only the approval reference, and the maintenance closeout only the request path and sha. Say the version is found through the depth record.
  - Retargeting `publication_packet_path` re-attributes the stage ungated, because "no other lifecycle value belongs to a tuple".
  - Drift absence is attributed to the previous closeout while a newer generation is open. Nothing in xtask sets `Drifted` today.
- **628-629:** "Items 1 to 3" collides with predicate items 1-3 (534-539).
- **158 against 760:** "Unresolved" now collides with the Unresolved publication outcome.
- **340-341:** The last sentence still says "not admissible" without the production qualifier.
- **693, 88-89:** The record's location needs a manifest root and an exact version. `approved-agent.toml` names no version, so the charter's P freeze must fall where both exist, or bullet 1 cannot be satisfied.
- **98 against 124-126:** The record is now the sole resolution source, permanent and mutable, which sits in tension with the "mutable status ledger" prohibition.
- **Pre-existing:** A hand-listed acceptance that nothing granted passes validation; bullet 3's carve-out makes that explicit.

## Examined and held

- **Record removal:** `manifest-retain` removal is covered by 507 and bullet 2 (D1).
- **Record at P freeze:** A skipped record write is caught by bullet 1. `evidence.rs:34-51` treats a `reports/<v>/` directory without coverage files as no reports, and `support-matrix.md:170-175` confirms `reports/**` intake.
- **Lifecycle writers:** `refresh-publication`, `close-proving-run` and the closeout write produce no bullet-8 contradiction in any order. A multi-agent backfill and an agent with no maintenance closeout are also clean.
- **Deadlock candidates:** Evidence refresh, debt renewal, P supersession, displacement and coverage regeneration create none beyond F1. An E change withdraws `qualified` for the whole version until re-accepted, which is coarse but consistent with the Bindings E row.

## Files

Root: `~/__Active_Code/atomize-hq/unified-agent-api/.claude/worktrees/agent-acc4a8d292a06283a`

- `<root>/docs/specs/support-depth-contract.md`
- `<root>/docs/specs/maintenance-request-contract-v1.md`
- `<root>/docs/specs/unified-agent-api/acquisition-maintenance-lifecycle-spec.md`
- `<root>/docs/specs/unified-agent-api/support-matrix.md`
- `<root>/crates/xtask/src/agent_lifecycle.rs`
- `<root>/crates/xtask/src/publication_refresh.rs`
- `<root>/crates/xtask/src/close_proving_run.rs`
- `<root>/crates/xtask/src/agent_maintenance/closeout/write.rs`
- `<root>/crates/xtask/src/agent_maintenance/audit_status/evidence.rs`
