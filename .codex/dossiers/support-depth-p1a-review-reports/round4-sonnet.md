# round4-sonnet: reviewer report

Lane: Sonnet 5.5 max, defects and conformance. Candidate: `5ff62ab8`. Scope: remediation follow-up.
Verbatim reviewer output; the lead's adjudication is in
`../support-depth-p1a-review-adjudication-2026-10-02.md`.

---

**Candidate revision:** 5ff62ab8 (`docs/specs/support-depth-contract.md`, 728 lines; the packet said 726). I made no changes. I ran no builds or tests.

All nine objectives are met on their face. I found two P2 regressions that the remediation itself introduced, plus several non-blocking items.

## Findings

### F1. P2: the evidence-protection mechanism cannot be decided or tested as written

**Lines:** 472, 481, 497, 483-487, 512-513, 551-553; the Terms at 88-98; line 336-337.

**Delta-introduced.** At dd14fb32 the wording was whole-evidence: "the presence and content identity of the evidence it derives from". The term "evidence entry" and item 3 are new, and so is item 3's application to every effect.

There are three defects.

- **(a) "Evidence entry" is undefined.** It appears only at the lines above, and neither Terms nor the proposal defines it. C1 makes entry granularity the whole mechanism.
  - Take the one output the row names. `cli_manifests/codex/reports/0.156.1/coverage.any.json` has the keys `schema_version, generated_at, inputs, platform_filter, deltas`.
  - Its `deltas` lists only gaps and exceptions: 0 missing, plus `excluded_*`, `passthrough_candidates`, `intentionally_unsupported` and `wrapper_only_*`.
  - A covered unit has no row, so for an explicitly covered operation "the evidence entries that belong to the tuple" is empty.
  - The fact that matters (no gap) is an absence, and "presence and content identity of each evidence entry" cannot express it.
  - Currentness invalidation still downgrades such results. Item 3 and rule 7 protect nothing here.
- **(b) The term collides.** Lines 481 and 497 already use "evidence entries" for the lifecycle record's `required_evidence` and `satisfied_evidence`.
  - Line 336-337 treats "a successful closeout" as evidence, and lines 90-91 make the closeout identity the recorded acceptance.
  - Item 3 now applies to every effect, including the reporting effect that Later versions (563-564) assigns to a displaced tuple. On that vocabulary the closeout file is "an evidence entry that a depth record's recorded acceptance derives from".
  - The next generation's closeout would then be refused. This contradicts the delta's own sentence at 97-98 ("Replacing a working file whose content a depth record has captured is not a deleted binding"). It also contradicts "A later generation replaces them" (96-97).
- **(c) The depth record is never said to hold what rule 7 compares.**
  - Line 551-553 compares evidence entries with "what a depth record's recorded acceptance binds".
  - The record definition (88-94) lists "the identity of its E" and "the identities of the closeout and promotion". It lists no per-entry evidence identities.
  - E and the closeout live in working files that a later generation replaces. A digest cannot be expanded into per-entry presence and identity after that.
  - The third contradiction can therefore only be evaluated for the latest version.

**Consequence.** An implementer must guess one of three things.

- Whole-file granularity recreates the round-three deadlock: `manifest-validate` requires older reports to be regenerated after a new parity exclusion.
  - Source: `cli_manifests/codex/VALIDATOR_SPEC.md`, "Report invariant: No report may list an excluded identity under: `deltas.missing_commands` ...".
  - It is enforced for every version at `crates/xtask/src/manifest_validate/versions.rs:279-300`.
- Row granularity is vacuous for covered units.
- Treating the closeout as protected blocks every later maintenance generation.

**Remediation.**
- Add a Terms entry for "Evidence entry": the smallest item that an evidence output under the manifest root keys by version, target or name identity. State that a working file, the lifecycle record and a depth record are not evidence entries.
- Either name the granularity per output or delegate it explicitly to Annex A. For coverage reports, bind the tuple's per-name-identity verdict, absence included.
- Qualify the lifecycle uses at 481 and 497.
- Add "the presence and content identity of each evidence entry its recorded acceptance derives from" to the depth record definition.

### F2. P2: the lifecycle row attributes values to tuples by actor, which merge and validation cannot evaluate

**Line:** 481 ("for the tuples of the generation the writing route acts for"), against 107-108, 532-536 and 548-555.

**Delta-introduced regression.** The row at dd14fb32 read "for the tuple's agent", which was data-keyed and decidable.

Rule 1 (532-533) says tuples "are determined by the values it would change, not by the agent, version or scope the request names". Rule 7 (554-555) says validation fails on any contradiction "whatever produced the change: a supported route, unsupported tooling or a hand edit". This is now the only actor-keyed row.

Two cases leave the attribution undefined.
- A merge is a route (107-108) but acts for no generation.
- A hand edit has no writing route at all.

For both, a lifecycle-record change touches no tuple. A stage advance to `closed_baseline`, an added `maintenance_closeout_written`, or a cleared `drifted` is therefore invisible to integration-step admission and to validation.

Writers such as `crates/xtask/src/historical_lifecycle_backfill.rs:97-107,203-215` loop over several agents' lifecycle records. For these writers "the generation" is not a single one. That run also resets the evidence lists to the `closed_baseline` set (`agent_lifecycle.rs:269-282`), which lacks `maintenance_closeout_written`.

The exposure is limited to the lifecycle record itself. The closeout, pointers and rows are data-keyed and still gated.

**Remediation.** Attribute a lifecycle value to the generation it records or accompanies (the approval, request or closeout written in the same change), not to the invoking route. A change that accompanies no generation record should be a validation contradiction, not an ungated write. This keeps C5's intent that a maintenance write does not touch the onboarding tuples.

### Non-blocking observations (P3)

- **C3 residual.**
  - The working-file exception sits only in Terms (97-98). Depth enrollment rule 8 (452-454) still says "deleted bindings are errors". Add a pointer there.
  - The record omits the Event (374). Bindings rule 8 (401-403) requires "all four bindings" from any consumer acting on depth-enrolled scope. For a displaced tuple the request, which carries the Event, has been replaced.
- **C4 residual.** Final depth admission items 1 and 2 (575-576) still require "serialization ownership" of every route. Lines 584-586 and 599-601 say the ref update "is" the integration step's serialization. State that the ref update satisfies items 1 and 2 for that step.
- **Mis-scoped contradiction (553-554).** "A version whose O has been frozen without a depth record" should read "a depth-enrolled version". Every historical version is remainder, and a mis-scoped check fails the whole repository.
- **C7 edge (334-335).** "Evidence" is unqualified. On the acquisition lane, discovery inputs (snapshots, reports) precede O's freeze (393-395) and are O's own "acquired input identities" (376). Say the rule governs evidence that verifies an obligation.
- **Untestable state (154-155).** "Under investigation" has no recorded state or exit. Say "while a contradiction between qualifying runs is unresolved".
- **Record timing and cardinality (91-94).** The record is "written no later than the freeze of O" but lists fields that exist only later (E identity, results, acceptance). It is also per version and target while "records what a generation froze". Say "first written ... and extended thereafter", and say whether a later generation of the same version extends or replaces the record.
- **Safe simplifications.**
  - The evidence MUST NOT is stated three times (483-485, 512-513, 551-553). Keep item 3 and rule 7.
  - The charter amendment is stated twice (399-400, 649-651), and "does not yet define" goes stale on amendment. State it once as an enablement prerequisite.

## C1 to C9

| Objective | Status | Remediated text |
| --- | --- | --- |
| C1 | Fixed (granularity per F1) | 472, 483-487. Entries, not files, are protected. Regeneration that leaves entries unchanged is permitted. `wrapper_coverage.json` is a dependency. Source-verified: it is a per-root file regenerated by every acquisition (`parity-acquire.yml:548-552,868`). `manifest-report` reads it from the root (`manifest_report.rs:91`). |
| C2 | Fixed (evaluability per F1) | 512-513, predicate item 3 for every effect. 551-553, validation contradiction. |
| C3 | Fixed (see P3) | 88-94, 95-98. |
| C4 | Fixed (see P3) | 580-586, 599-601. |
| C5 | Fixed as stated (regression F2) | 481, 496-497. Source-verified: `agent_lifecycle.rs:92-99,108-109`, `publication_refresh.rs:439`, `close_proving_run.rs:447`, `closeout/write.rs:83-101`. |
| C6 | Fixed | 91-92, 448-451, 553-554. |
| C7 | Fixed (see P3) | 334-337. |
| C8 | Fixed | 399-400, 649-651. Source-verified: `cli-agent-onboarding-charter.md` defines no O or freeze point on the create lane. |
| C9 | Fixed | "Established earlier" 583-584. `classification_required` 520. Qualified "depth records" 550. Insufficient depth 398-399 and 712. `unverified` 154-155. Working-file location 95-96 (verified against `request/paths.rs` and the maintenance request contract line 4). Not an inventory or ledger 93-94. |

Predicate items 1-6 are consistent. No other text refers to them by number: the "item 3" references at 646-653 are path-enablement items. The delta's tables are well-formed and all relative links resolve.

**Areas examined:**
- Every changed hunk in `dd14fb32..5ff62ab8`.
- The cited specs: the maintenance request contract, onboarding charter, support-matrix neutral root intake, wrapper coverage generator contract and the manifest validator spec.
- Lifecycle, closeout, publication-refresh and report-validation code in `crates/xtask/src/**`.
- `.github/workflows/parity-acquire.yml`.
- The real `artifacts.lock.json`, `wrapper_coverage.json` and coverage report shapes.

One existence check for the spec links ran with a `cd` into `docs/specs` inside this worktree. It was read-only.
