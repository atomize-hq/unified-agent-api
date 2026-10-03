# round5-packet-sonnet: bounded delta review packet as dispatched

Reuse as the template for the next bounded round: change the two revisions, the changed-region
line list, the findings table and the lane intent.

---

READ-ONLY BOUNDED DELTA REVIEW PACKET (round five). A second, independent lane reviews the same delta; do not coordinate with it.

### How to read the candidate
You run in your own git worktree. First run, inside that worktree only:
    git switch --detach d78cf535
Then read files normally. Do not use `git -C`, do not `cd` to any other worktree or path, do not edit, write, stage or commit anything. This is a documentation-only candidate: do not run cargo builds or test suites. Allowed commands: read-only git (`show`, `log`, `diff`, `grep`), `grep`/`rg`, `sed -n`, `wc`, and reading files.

### Revisions and scope
- Base: 5ff62ab8 (the revision both lanes reviewed in round four).
- Candidate: d78cf535.
- Changed-file set: exactly one file, `docs/specs/support-depth-contract.md` (771 lines, `Status: Draft`). Confirm with `git diff --stat 5ff62ab8 d78cf535`.
- Read the delta with `git diff 5ff62ab8 d78cf535 -- docs/specs/support-depth-contract.md`.
- Changed regions, by line number at d78cf535: 88-101 (Terms: Depth record, Working files); 158; 337-341 (admissibility); 397-409 (Bindings rules 7 and 8); 454-463 (Depth enrollment rules 7 and 8); 481 and 490-513 (Depth-gated effects: two table rows and three new paragraphs); 523-524 (acceptance effects); 538-539 (predicate item 3); 574-595 (Complete mediation rule 7); 625-629 (integration step); 693-696 (Path enablement); 740-742 (Publication).

THIS REVIEW IS BOUNDED. Review only the changed text and its direct interactions with unchanged text. Do not re-review unchanged sections on their own merits, do not read the research documents end to end, and do not re-litigate decisions. Open another file only to test a specific claim the delta makes. A defect in unchanged text that the delta neither introduced nor made worse is out of scope; if you must mention one, label it "pre-existing" and keep it to one line.

### Context
The repository is a Rust workspace: per-agent CLI wrapper crates, a unified facade crate `crates/agent_api`, and lifecycle automation in `crates/xtask`. `docs/specs/**` is normative. The candidate is a Draft contract for a "support-depth" policy: semantic support obligations added beside the existing name-coverage machinery. It writes rules only; no implementation exists. A "depth record" is the durable per-version record of what a depth-enrolled version froze, verified and had accepted. P, O and E are the policy, obligation and execution bindings of a generation.

### What round four found, and what this delta does about it
| Id | Round-four P2 | What the delta does |
| --- | --- | --- |
| E1 (other lane's F1) | A reporting effect could narrow a depth record (unbind accepted evidence, change the captured selection or the P/O identities), after which evidence protection and validation stopped covering the dropped evidence while a row still said verified. | A record is never removed and a listed acceptance never removed or altered (predicate item 3, applies to every effect). What a record states changes only when a later generation of the same version continues it, or through a change the Bindings table permits to the binding concerned, and every such change shows in the P, O or E identity the record states. Each listed acceptance names the P, O and E identities it accepted and supports a published qualified value only while the record still states them. Rule 7 compares a proposed branch's record with the integration branch's copy. The lead chose identity-bound support over "any change is an acceptance effect" so that truthful re-verification stays possible without closeout authority. |
| E2 (your F1) | "Evidence entry" was undefined and undecidable: coverage reports list only gaps, so a covered unit has no entry; the term collided with lifecycle evidence entries; the closeout would count as protected evidence; the record held no per-entry identities. | The term is removed. The depth record is one committed file per version under `reports/<version>/` and binds each `verified` result to its evidence by content identity. Coverage reports and `wrapper_coverage.json` are not depth-gated outputs. Changing bound evidence is not prohibited; the result is then published as `unverified`. Lifecycle uses now say "evidence id". |
| E3 (other lane's F2) | A depth-enrolled version whose O never freezes had no record, so a later version's packet replaced the working files and the version silently read as not depth-enrolled. | The record is first written when the version's first generation freezes P (the request's first freeze on the maintenance lane) and is extended as O, E and acceptances follow. A generation whose P is frozen for depth-enrolled scope with no record is a contradiction. Resolution comes from the record. Bindings rule 8 now requires the bindings established so far. |
| E4 (your F2) | The lifecycle row attributed values to "the generation the writing route acts for", which a merge, a hand edit or a multi-agent backfill cannot evaluate. | Attribution is by the record a value reports, as it stands in the same revision: `published` reports the publication packet the lifecycle record names; `closed_baseline` and the proving-run closeout evidence id report the proving-run closeout it names; the maintenance closeout evidence id and the absence of a drift side state report the agent's maintenance closeout record. A stage or closeout evidence id that reports a record the revision does not hold is a contradiction. |

### Smaller changes in the same delta (from both lanes' round-four P3s)
The Event is stated in the record; a later generation of the same version continues the same record; the "deleted binding" exception moved from Terms to Depth enrollment rule 8 and speaks of identities; the missing-record check is scoped to depth-enrolled scope; the integration step's tip-conditional ref update is a MUST and is tied to items 1 to 3; admissibility is limited to evidence that verifies an obligation of a production depth enrollment; "under investigation" became "unresolved"; the charter prerequisite is stated once, in Path enablement, and now covers the P freeze point as well as O's.

### Standing decisions (not open for re-litigation)
- Greenfield: no legacy inventory, grandfathering, carry-forward ledger or migration engine.
- Minimum machinery: any new field, artifact or step must prevent a concrete reachable failure existing machinery cannot.
- Reporting effects need no lifecycle-path authority, and truthful assessments (unverified, failed, or none yet) must stay publishable.
- Serialize affected writes first; the integration step's serialization is its tip-conditional ref update.
- One depth enrollment covers one exact upstream version; no successor rule yet.
- Two design choices made by the lead in this delta. Attack them on correctness, not on preference: (1) changing bound evidence is not prohibited and instead invalidates the result; (2) lifecycle attribution is state-based, by the record a value reports, rather than "written in the same change" as you suggested, because `refresh-publication` sets `published` without rewriting the publication packet, so prepare and refresh can merge separately and a same-change rule would misfire.

### Known and excluded: do not report these
- Annex A (route inventory) is absent; the contract says it cannot be approved without it.
- Dependent amendments come later: registry, maintenance request, charter (including where P and O are frozen on the create lane), support matrix, and each manifest root's validator spec (including the record's file name and schema).
- No implementation exists or is claimed. `manifest-retain` deletes `reports/<v>` today; that is an Annex A "present versus required" entry.
- Recorded deferrals: D1 (retain removing a pending version's files; now explicit as "a depth record MUST NOT be removed"), D2 (the admission checker runs from the candidate's own tree), D3 (acceptance for a version whose working files were replaced has only the record's P and O identities; it fails closed or admits from the record, and is decided in Annex A's promotion entry).
- Pre-existing semantics the delta does not change: results are live, so a result whose bindings are no longer current reads as `unverified`; adequacy review happens inside the existing review and closeout trust boundary and is not mechanically decidable; a fabrication made inside that boundary is caught by review unless a listed contradiction covers it.

### Lead-verified facts (rely on them or re-check them)
- `crates/xtask/src/manifest_retain.rs:97-126` deletes `snapshots/<v>` and `reports/<v>` wholesale for versions outside the keep set. No workflow calls it.
- `reports/<version>/` holds `coverage.any.json`, `coverage.<target>.json` and sometimes `coverage.all.json`. Report-presence checks look for coverage files, not the directory (`crates/xtask/src/agent_maintenance/audit_status/evidence.rs:34-51`).
- `docs/specs/unified-agent-api/support-matrix.md`, "Neutral root intake": intake is limited to `versions/*.json`, the pointer files, `current.json` and `reports/**`.
- `docs/specs/maintenance-request-contract-v1.md` lines 55-67: a generation freezes the request twice, once at open and then inside `parity-acquire`; the docs-only lane has no second freeze.
- `LifecycleState` (`crates/xtask/src/agent_lifecycle.rs:302-328`) has no version field. It has `publication_packet_path`/`_sha256` and `closeout_baseline_path`. `publication_refresh.rs:439-467` sets `published`, sets the packet reference and clears the closeout baseline. `close_proving_run.rs:447-474` sets `closed_baseline`, sets both references and clears drift. `agent_maintenance/closeout/write.rs:83-101` clears `drifted` and adds `maintenance_closeout_written`. Evidence ids are at `agent_lifecycle.rs:179-193`.
- Maintenance working files are `docs/agents/lifecycle/<agent>-maintenance/governance/maintenance-request.toml` and `maintenance-closeout.json`.
- On the current tree, all five agents' lifecycle records resolve under the new rule: every `published` or `closed_baseline` stage and every closeout evidence id reports a record that exists.

### Acceptance criteria for this delta
1. E1 to E4 are each fixed: the round-four failure scenario no longer complies with the text.
2. The delta introduces no new P1 or P2: no contradiction with unchanged text or a controlling spec, no legitimate flow made impossible, no bypass of the invariants it adds.
3. Every MUST and every listed contradiction in the changed text is decidable from repository state, or from a proposed branch compared with the integration branch.
4. Terms the delta introduces or changes are defined, used consistently, and do not collide with existing repository terms.
5. Nothing in the changed text states as existing behavior something that does not exist.

### Output
- First line: `CLEAN` or `NOT CLEAN`, with the candidate revision.
- Then a table of E1, E2, E3 and E4: Fixed, Partially fixed or Not fixed, with line numbers.
- Then findings, most severe first. P1 = adopting the delta as written would be unsafe or wrong; P2 = material ambiguity or omission that would plausibly cause a wrong implementation or a wrong dependent amendment; P3 = clarity only, non-blocking. For each: severity; line numbers at d78cf535; whether the delta introduced it; the violated requirement or contradicted source with path and quoted text; a concrete failure scenario or consequence; a narrowly scoped remediation direction.
- Keep it short. No redesign, no scope widening. If no P1 or P2 remains, say `CLEAN` and list the areas you examined.

### Review intent for YOUR lane (defects and conformance)
Work through each changed hunk, and only those. In particular:
- Re-check your own round-four F1 (a), (b) and (c) and F2 against the new text. For F1 (a), take a real `reports/<version>/coverage.any.json` and confirm nothing in the changed text still depends on its entries. For F2, confirm a merge, a hand edit and `historical_lifecycle_backfill` are each attributable.
- For every MUST and every bullet in Complete mediation rule 7 (574-595), state what repository state decides it. Flag any bullet that needs information the tree does not hold, such as what an acceptance "granted", what "published as better" compares, or what "reports a record the same revision does not hold" means for a value with no reference.
- Check the delta's vocabulary: "states" versus "lists"; "acceptance" as a listed entry versus the Acceptance fact in the four-facts table (351-357); "bound evidence"; "evidence id"; "established" in Bindings rule 8; "production depth enrollment". Flag undefined or inconsistent use, and any leftover of the removed terms ("evidence entry", "recorded acceptance", "captured").
- Check each changed hunk against the unchanged text it touches: the Bindings table (who may change P, O and E); Depth enrollment rules 1 to 11; Shared mapping rule 5 (248-253); Later versions (597-607); Final depth admission items 1 to 3 and the Serialization bullets; Path enablement; the Publication outcomes table; and the "What this contract does not change" table. Report only conflicts the delta created.
- Check the delta's factual claims about existing behavior against source: that P is frozen at the request's first freeze; the lifecycle fields and values named at 492-497; that `reports/**` is an intake category; the working-file names.
- Confirm anchors resolve, numbered lists and the nested bullets under rule 7 render and are complete, and no line in the changed prose exceeds the file's 100-column wrap.
- Name any safe simplification that removes words from the changed text without changing meaning.
