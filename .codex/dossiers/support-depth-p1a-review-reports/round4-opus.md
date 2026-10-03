# round4-opus: reviewer report

Lane: Opus 5.5 xhigh, adversarial. Candidate: `5ff62ab8`. Scope: remediation follow-up.
Verbatim reviewer output; the lead's adjudication is in
`../support-depth-p1a-review-adjudication-2026-10-02.md`.

---

**Not CLEAN.** Two P2 findings remain at 5ff62ab8, and both come from the delta. There is no P1. I made no edits, staged nothing and committed nothing.

File reviewed: `~/__Active_Code/atomize-hq/unified-agent-api/.claude/worktrees/agent-a77853ba4c8311fba/docs/specs/support-depth-contract.md`

## Findings

### F1 — P2: a reporting effect can narrow the depth record, and evidence protection and validation then stop covering the dropped evidence

**Lines:** 472, 483–487, 498, 512–513, 524, 548–555, and 448–450.

**What it violates:**
- **C2:** removing accepted evidence must fail predicate item 3 and must be a validation contradiction "whatever produced the change: … a hand edit" (554–555).
- **Line 154:** "An obligation with no admissible evidence is `unverified`."

**Why it is open:**
- The delta makes the depth record the anchor for three things:
  - resolution ("once a depth record exists it comes from the depth record", 450);
  - evidence protection ("evidence entry that a depth record's recorded acceptance derives from", 483 and 512);
  - the new validation contradiction ("differs from what a depth record's recorded acceptance binds", 551–553).
- Nothing protects the record itself. Line 472 makes its entries depth-gated, but the only record change classed as an acceptance effect is "records acceptance in a depth record" (498).
- So narrowing or removing an evidence binding, changing the captured selection, or changing the P/O identities is a reporting effect. Line 524 says a reporting effect "needs no lifecycle-path authority."
- Item 3 protects evidence entries, not the record's own entries.

**Failure scenario:**
1. Version N is accepted and published. Its record for linux-x64 binds entries e1–e5 in `reports/N/coverage.linux-x64.json`.
2. A branch edits the record so e3 is no longer bound, while keeping `verified` and the recorded acceptance. In the same branch, a parity-exclusion regeneration removes e3.
3. Validation passes all four checks:
   - the row is still supported by the record;
   - there is no unrecorded acceptance effect;
   - every bound entry is unchanged, because e3 is no longer bound;
   - the version whose O is frozen has a record.
4. The row still says verified for an obligation that now has no admissible evidence.
5. A supported two-step path reaches the same result: first a reporting-effect rewrite of the record (items 1–3 hold), then a regeneration that removes e3, which item 3 no longer covers.

**Delta link:** the delta created the validation contradiction and the shift to resolution from the depth record, both on an anchor that can be changed. So C2's root cause is only partly fixed.

**Remediation:** say that changing or narrowing a depth record's recorded acceptance, or the evidence it binds, is an acceptance effect, or is forbidden while a published value derives from it. Also have item 7 compare each record's recorded acceptance on a proposed branch with the integration branch's copy.

### F2 — P2: a depth-enrolled version whose O never freezes has no durable record, so a later version's packet silently un-enrolls it

**Lines:** 92, 95–98, 448–451, 553, 396–399, 712 and 697–699.

**What it violates:**
- **Rule 8 (452–454):** "None of them resolves to 'not depth-enrolled'."
- **Bindings rule 7 (398–399):** "reported as insufficient depth and never as not depth-enrolled."

**Failure scenario:**
1. An agent on the docs-only lane has published pointers, version files and `current.json` (the aider and gemini_cli roots have this shape). Version N is depth-enrolled.
2. P is frozen in `maintenance-request.toml`, but O never freezes. Lines 92 and 553 therefore require no record, and none is written. N is reported as insufficient depth (712).
3. The packet merges without closeout. That has happened before: #215.
4. Upstream releases N+1, and the nightly packet replaces the working files. This is a displacement, which is a reporting effect (559–564).
5. N now has no record and no current generation. Rule 7 resolves from "the frozen request … for the current generation", which is N+1's request, and that does not select N.
6. Validation iterates over depth enrollments, so it never sees N. The next support-matrix regeneration rewrites N's row as "Not depth-enrolled".
7. A second reading leads somewhere else but is still wrong. The converse of 97–98 (replacing content no record has captured is a deleted binding, which rule 8 makes an error) forces the route to refuse N+1's packet. Maintenance for the agent then stalls until the maintainer re-freezes N's P.

Separately, line 697 makes depth records the only published vehicle for depth facts. So the insufficient-depth outcome that 712 requires for versions with O not yet frozen has no published source even before displacement.

**Delta link:** the base text required a record for every depth-enrolled version and fell back to it after working files were replaced. The delta:
- limited the record to versions whose O is frozen (553);
- rewrote rule 7 to say "current generation";
- added the "captured content" sentence (97–98).

**Remediation:** before a generation replaces the working files of a depth-enrolled version that has no record, write that version's record with its resolved selection and P identity. O stays absent, and the scope is reported as insufficient depth. C6's validation scope stays as it is.

### P3 observations (non-blocking)

- **Lines 472 vs 483/512 (which entries belong to a tuple).** Line 472 gates "each evidence entry that belongs to the tuple" but never says what decides membership. Item 3 only runs for tuples the effect touches. After working files are replaced, O's operation-to-surface edges are gone, so a regeneration can treat an accepted entry as belonging to no touched tuple and skip item 3; only validation catches it. Fix: membership is whatever the record binds.
- **Line 481 vs 532–536 (C5 attribution).** The lifecycle row is attributed to "the generation the writing route acts for". That conflicts with mediation rule 1, which says values, not the version a request names, decide what is touched. It also gives state-based validation nothing to attribute a hand edit with; the lifecycle record has no version field (`crates/xtask/src/agent_lifecycle.rs:302–328`). In practice the gap is small:
  - a stage regression is ungated, but the re-advance is gated;
  - publication from `publication_ready` re-runs full admission;
  - the historical adoption (`maintenance_adoption.rs`) is pinned to the old Claude Code baseline.
- **Lines 599–601 (C4).** The integration step is exempted from serialization, and no sentence says its ref update MUST be tip-conditional. Item 1 at 575 plus 586 still requires it, so I see no route landing without one. The workflows only push to packet branches (`parity-acquire.yml:913`; `create-pull-request` with base `staging`). Adding "MUST" would remove the doubt.
- **Lines 89–98 (C3).** The record captures content *identities*, but 97–98 speaks of captured *content*. Resolution survives replacement. Acceptance for a displaced version cannot meet items 5–6 because O's content is gone, but that fails closed.
- **Line 334 (C7).** "some depth enrollment" may take in proof-workspace selections, constrained only by reuse bindings.

## Objective status

| Objective | Status |
|---|---|
| C1 | Fixed: entry-level protection, regeneration permitted, `wrapper_coverage.json` as a dependency (472, 483–487). The protection rests on a record that can be changed (F1). |
| C2 | Partially fixed: item 3 and the contradiction exist, but narrowing the record defeats them (F1). |
| C3 | Fixed for resolution; P3 on content vs identity. |
| C4 | Fixed; P3 wording. |
| C5 | Fixed (481, 496–497); P3 on attribution. |
| C6 | Implemented as specified; regression F2. |
| C7 | Fixed (334–337). |
| C8 | Fixed (399–400, 649–650). |
| C9 | Fixed (154–155, 362–364, 520–522, 550, 583–584, 712, 93–96). |

**Areas examined:**
- the full delta dd14fb32 → 5ff62ab8;
- the depth record, working-file, resolution, depth-gated table, predicate, mediation, later-versions, serialization and publication sections;
- the freeze and lane rules in `maintenance-request-contract-v1.md`;
- neutral root intake in `support-matrix.md`;
- the manifest-root report layout, and the `VALIDATOR_SPEC.md` rule on excluded identities;
- `LifecycleState` and its writers (`closeout/write.rs`, `publication_refresh.rs`, `maintenance_adoption.rs`), and `stand_down.rs`;
- the workflow push and merge paths.
